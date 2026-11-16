// Rust guideline compliant 2026-11-16
//! Tenant-scoped reverse-order execution of persisted inverse actions.

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    audit::{AuditEventStatus, AuditEventType, RawAuditEvent},
    context::{ProfileContext, RequestContext},
    daemon::{AppState, Policy},
    undo::{UndoEntry, UndoEntryStatus},
};

const MAX_FIBER_UNWIND_DURATION: std::time::Duration = std::time::Duration::from_secs(60);

/// Outcome for one inverse action attempted during batch rollback.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RollbackResult {
    /// Identifier of the persisted inverse action.
    pub undo_id: String,
    /// Forward operation identifier, when supplied by the caller.
    pub operation_id: Option<String>,
    /// Capability invoked to apply the inverse.
    pub capability: String,
    /// Whether the inverse completed successfully.
    pub ok: bool,
    /// Failure or authorization detail.
    pub error: Option<String>,
}

/// Authorization and tracing context reused for every inverse in one unwind.
///
/// Policy and profile checks are applied to each candidate immediately before dispatch. A
/// time-limited authorization is also checked against its recorded expiry.
pub struct FiberExecutionScope<'a> {
    /// Effective policy to apply to each inverse call.
    pub policy: &'a Policy,
    /// Active profile boundary for inverse capabilities.
    pub profile: &'a ProfileContext,
    /// Context metadata from the triggering request.
    pub context: Option<&'a RequestContext>,
    /// Absolute grant expiry, when the original request used a time-limited grant.
    pub authorization_expires_at_epoch_secs: Option<u64>,
    /// Trace identifier attached to inverse audit events.
    pub trace_id: &'a str,
}

/// Unwinds armed entries associated with a tenant and batch request in LIFO order.
///
/// Entries are durably claimed before dispatch and retained in the ledger after completion.
/// The request ID scopes entries to this batch; the tenant ID prevents cross-tenant selection.
///
/// # Arguments
/// * `state` - Active daemon state and persistent undo ledger.
/// * `tenant_id` - Tenant that owns the batch.
/// * `batch_id` - Stable batch request identifier.
/// * `scope` - Effective policy, profile, authorization expiry, and audit context.
///
/// # Returns
/// Outcomes in the order inverse actions were attempted, newest forward action first.
pub async fn unwind_batch_fiber(
    state: &AppState,
    tenant_id: &str,
    batch_id: &str,
    scope: FiberExecutionScope<'_>,
) -> Vec<RollbackResult> {
    unwind_matching_fiber(
        state,
        tenant_id,
        |entry| entry.batch_id.as_deref() == Some(batch_id),
        scope,
    )
    .await
}

/// Unwinds armed entries associated with a tenant and work item in LIFO order.
///
/// This internal operation supports turn-level rollback. It does not expose an HTTP or CLI
/// endpoint; operator controls are handled in the later operator-plane stage.
///
/// # Arguments
/// * `state` - Active daemon state and persistent undo ledger.
/// * `tenant_id` - Tenant that owns the work item.
/// * `work_item_id` - Workflow identifier used to select the fiber.
/// * `scope` - Effective policy, profile, and audit context.
///
/// # Returns
/// Outcomes in the order inverse actions were attempted, newest forward action first.
pub async fn unwind_fiber(
    state: &AppState,
    tenant_id: &str,
    work_item_id: &str,
    scope: FiberExecutionScope<'_>,
) -> Vec<RollbackResult> {
    unwind_matching_fiber(
        state,
        tenant_id,
        |entry| entry.work_item_id.as_deref() == Some(work_item_id),
        scope,
    )
    .await
}

async fn unwind_matching_fiber(
    state: &AppState,
    tenant_id: &str,
    matches_fiber: impl Fn(&UndoEntry) -> bool,
    scope: FiberExecutionScope<'_>,
) -> Vec<RollbackResult> {
    let FiberExecutionScope {
        policy,
        profile,
        context,
        authorization_expires_at_epoch_secs,
        trace_id,
    } = scope;
    let Some(ledger) = state.undo_ledger.as_ref() else {
        return Vec::new();
    };
    let now_ms = now_ms();
    let mut entries = ledger
        .entries()
        .await
        .into_iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry.tenant_id == tenant_id
                && matches_fiber(entry)
                && entry.status == UndoEntryStatus::Armed
                && entry.expires_at_ms > now_ms
        })
        .collect::<Vec<_>>();
    entries.sort_by(|(left_index, left), (right_index, right)| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right_index.cmp(left_index))
    });

    let mut outcomes = Vec::with_capacity(entries.len());
    let unwind_started = std::time::Instant::now();
    let mut entries = entries.into_iter();
    while let Some((_, candidate)) = entries.next() {
        if unwind_started.elapsed() >= MAX_FIBER_UNWIND_DURATION {
            outcomes.push(failed_result(
                &candidate,
                "fiber unwind budget exhausted; entry remains armed".to_string(),
            ));
            outcomes.extend(entries.map(|(_, entry)| {
                failed_result(
                    &entry,
                    "fiber unwind budget exhausted; entry remains armed".to_string(),
                )
            }));
            break;
        }
        let claimed = match ledger.claim(&candidate.undo_id, now_ms).await {
            Ok(Some(entry)) => entry,
            Ok(None) => continue,
            Err(error) => {
                outcomes.push(failed_result(&candidate, error.to_string()));
                continue;
            }
        };
        let outcome = dispatch_inverse(
            state,
            &claimed,
            policy,
            profile,
            context,
            authorization_expires_at_epoch_secs,
            trace_id,
        )
        .await;
        let terminal = if outcome.ok {
            UndoEntryStatus::Reverted
        } else {
            UndoEntryStatus::Failed
        };
        if let Err(error) = ledger.finish(&claimed.undo_id, terminal).await {
            outcomes.push(failed_result(
                &claimed,
                format!("inverse outcome could not be persisted: {error}"),
            ));
        } else {
            outcomes.push(outcome);
        }
    }
    outcomes
}

fn failed_result(entry: &UndoEntry, error: String) -> RollbackResult {
    RollbackResult {
        undo_id: entry.undo_id.clone(),
        operation_id: entry.operation_id.clone(),
        capability: entry.resolved_inverse_capability.clone(),
        ok: false,
        error: Some(error),
    }
}

async fn dispatch_inverse(
    state: &AppState,
    entry: &UndoEntry,
    policy: &Policy,
    profile: &ProfileContext,
    context: Option<&RequestContext>,
    authorization_expires_at_epoch_secs: Option<u64>,
    trace_id: &str,
) -> RollbackResult {
    let capability = &entry.resolved_inverse_capability;
    let denied = if authorization_expires_at_epoch_secs.is_some_and(|expiry| now_secs() >= expiry) {
        Some("caller authorization expired before inverse dispatch".to_string())
    } else if !policy.allows(capability) {
        Some("inverse capability is denied by current policy".to_string())
    } else if policy.requires_approval(capability) {
        Some("inverse requires operator approval and cannot run automatically".to_string())
    } else {
        None
    };
    let mut error = denied;
    let idempotency_key = format!(
        "idk_{}",
        hex::encode(Sha256::digest(entry.undo_id.as_bytes()))
    );
    let server_id = {
        let capabilities = state.capabilities.read().await;
        capabilities.get(capability).map(|meta| meta.server.clone())
    };
    let allowed_server = server_id
        .as_deref()
        .is_some_and(|server| profile.is_server_allowed(server));
    if error.is_none() && !allowed_server {
        error = Some("inverse capability is missing or outside the active profile".to_string());
    }
    let ok = if error.is_none() {
        let mut options =
            crate::engine::ExecutionOptions::new().with_idempotency_key(idempotency_key.clone());
        options.request_id = entry.batch_id.clone();
        options.context = Some(context.cloned().unwrap_or_else(|| RequestContext {
            operation_id: entry.operation_id.clone(),
            work_item_id: entry.work_item_id.clone(),
            actor_id: entry.actor_id.clone(),
            grant_id: None,
        }));
        options.profile = profile.profile_id.clone();
        options.suppress_undo_registration = true;
        let response = crate::engine::ControlPlaneHandle::new(state.clone())
            .call_capability(capability, entry.resolved_inverse_args.clone(), options)
            .await;
        if response.ok {
            true
        } else {
            error = response
                .error
                .map(|item| item.message)
                .or_else(|| Some("inverse call failed".to_string()));
            false
        }
    } else {
        false
    };
    let audit_payload = json!({
        "undoId": entry.undo_id,
        "parentOperationId": entry.operation_id,
        "snapshotArtifactHash": entry.snapshot_artifact_hash,
        "idempotencyKey": idempotency_key,
        "outcome": if ok { "executed" } else { "failed" },
        "error": error,
    });
    state.audit_handle.send(RawAuditEvent {
        event_type: if ok {
            AuditEventType::UndoExecuted
        } else {
            AuditEventType::UndoFailed
        },
        trace_id: trace_id.to_string(),
        request_id: entry.batch_id.clone(),
        actor_id: context
            .and_then(|value| value.actor_id.clone())
            .or_else(|| entry.actor_id.clone()),
        work_item_id: entry.work_item_id.clone(),
        client_ip: None,
        server_id,
        capability_id: Some(capability.clone()),
        resource_uri: None,
        sanitized_args: Some(json!({"undoId": entry.undo_id})),
        sanitized_response: Some(audit_payload),
        execution_latency_us: None,
        status: if ok {
            AuditEventStatus::Success
        } else {
            AuditEventStatus::Failed
        },
        error_code: error.as_ref().map(|_| "UNDO_FAILED".to_string()),
        error_message: error.clone(),
        operator_id: None,
        approval_ticket_id: None,
        idempotency_key: Some(idempotency_key),
        is_replay: Some(false),
    });
    RollbackResult {
        undo_id: entry.undo_id.clone(),
        operation_id: entry.operation_id.clone(),
        capability: capability.clone(),
        ok,
        error,
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
