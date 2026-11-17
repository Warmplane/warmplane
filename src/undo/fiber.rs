// Rust guideline compliant 2026-11-17
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
    /// Machine-readable outcome code, when the inverse could not be completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Sanitized output from the inverse call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Committed audit event identifier for this inverse outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_event_id: Option<String>,
    /// Committed audit event hash for this inverse outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_hash: Option<String>,
    /// Audit persistence error, independent of the inverse execution outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_error: Option<String>,
    /// Whether the expired-window operator override was used.
    #[serde(default)]
    pub forced_expired: bool,
    /// Persisted ledger status after the operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_status: Option<UndoEntryStatus>,
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
    /// Operator identity for an explicitly authorized undo action.
    pub operator_id: Option<&'a str>,
    /// Reason recorded when an operator forces execution after expiry.
    pub force_expired_reason: Option<&'a str>,
}

enum FiberSelector<'a> {
    Batch(&'a str),
    WorkItem(&'a str),
}

impl FiberSelector<'_> {
    fn matches(&self, entry: &UndoEntry) -> bool {
        match self {
            Self::Batch(batch_id) => entry.batch_id.as_deref() == Some(*batch_id),
            Self::WorkItem(work_item_id) => entry.work_item_id.as_deref() == Some(*work_item_id),
        }
    }
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
    unwind_matching_fiber(state, tenant_id, FiberSelector::Batch(batch_id), scope).await
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
        FiberSelector::WorkItem(work_item_id),
        scope,
    )
    .await
}

async fn unwind_matching_fiber(
    state: &AppState,
    tenant_id: &str,
    selector: FiberSelector<'_>,
    scope: FiberExecutionScope<'_>,
) -> Vec<RollbackResult> {
    let FiberExecutionScope {
        policy,
        profile,
        context,
        authorization_expires_at_epoch_secs,
        trace_id,
        operator_id,
        force_expired_reason,
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
                && selector.matches(entry)
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
        let dispatch_scope = FiberExecutionScope {
            policy,
            profile,
            context,
            authorization_expires_at_epoch_secs,
            operator_id,
            force_expired_reason,
            trace_id,
        };
        if let Some(outcome) =
            block_on_pre_undo_drift(state, ledger, &claimed, &dispatch_scope).await
        {
            outcomes.push(outcome);
            continue;
        }
        let outcome = dispatch_inverse(state, &claimed, dispatch_scope).await;
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
        error_code: Some("UNDO_FAILED".to_string()),
        output: None,
        audit_event_id: None,
        audit_hash: None,
        audit_error: None,
        forced_expired: false,
        entry_status: None,
    }
}

async fn block_on_pre_undo_drift(
    state: &AppState,
    ledger: &crate::undo::UndoLedgerStore,
    claimed: &UndoEntry,
    scope: &FiberExecutionScope<'_>,
) -> Option<RollbackResult> {
    let check = claimed.pre_undo_check.as_ref()?;
    let checked_state = crate::engine::undo::read_checked_state(
        state,
        check,
        &serde_json::Value::Null,
        scope.profile,
        scope.policy,
    )
    .await;
    let current_hash = checked_state.and_then(|value| {
        state
            .snapshot_store
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("snapshot storage is unavailable"))?
            .put(&value)
    });
    let guard_error = match current_hash {
        Ok(hash) if claimed.expected_post_snapshot_hash.as_deref() == Some(hash.as_str()) => {
            return None;
        }
        Ok(_) => Some((
            "UNDO_STATE_DRIFT",
            "upstream state changed after the forward action",
        )),
        Err(_) => Some((
            "UNDO_STATE_CHECK_FAILED",
            "could not verify upstream state before undo",
        )),
    };
    let (code, message) = guard_error?;
    let release_result = ledger.release_operator_claim(&claimed.undo_id).await;
    state.notify_action_capsules_changed();
    let mut result = failed_result(claimed, message.to_string());
    result.error_code = Some(if release_result.is_ok() {
        code.to_string()
    } else {
        "UNDO_PERSISTENCE_FAILED".to_string()
    });
    result.entry_status = Some(if release_result.is_ok() {
        UndoEntryStatus::Armed
    } else {
        UndoEntryStatus::Executing
    });
    let server_id = state
        .capabilities
        .read()
        .await
        .get(&check.capability)
        .map(|meta| meta.server.clone());
    let audit = state
        .audit_handle
        .send_confirmed(RawAuditEvent {
            event_type: AuditEventType::UndoFailed,
            trace_id: scope.trace_id.to_string(),
            request_id: claimed.batch_id.clone(),
            actor_id: scope
                .operator_id
                .map(ToString::to_string)
                .or_else(|| claimed.actor_id.clone()),
            work_item_id: claimed.work_item_id.clone(),
            client_ip: None,
            server_id,
            capability_id: Some(check.capability.clone()),
            resource_uri: None,
            sanitized_args: Some(json!({"undoId": claimed.undo_id})),
            sanitized_response: Some(json!({
                "outcome": "blocked",
                "reasonCode": result.error_code,
                "expectedPostSnapshotHash": claimed.expected_post_snapshot_hash,
            })),
            execution_latency_us: None,
            status: AuditEventStatus::Failed,
            error_code: result.error_code.clone(),
            error_message: result.error.clone(),
            operator_id: scope.operator_id.map(ToString::to_string),
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: Some(false),
        })
        .await;
    match audit {
        Ok(record) => {
            result.audit_event_id = Some(record.id.clone());
            result.audit_hash = Some(record.hash.clone());
        }
        Err(error) => result.audit_error = Some(error.to_string()),
    }
    Some(result)
}

async fn dispatch_inverse(
    state: &AppState,
    entry: &UndoEntry,
    scope: FiberExecutionScope<'_>,
) -> RollbackResult {
    let FiberExecutionScope {
        policy,
        profile,
        context,
        authorization_expires_at_epoch_secs,
        operator_id,
        force_expired_reason,
        trace_id,
    } = scope;
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
    let mut output = None;
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
            output = response
                .data
                .map(|value| crate::http_v1::helpers::redact_value(value, &policy.redact_keys));
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
    let audit_error = error
        .as_ref()
        .map(|_| "inverse call failed; see the operator response");
    let audit_payload = json!({
        "undoId": entry.undo_id,
        "parentOperationId": entry.operation_id,
        "snapshotArtifactHash": entry.snapshot_artifact_hash,
        "idempotencyKey": idempotency_key,
        "forceExpiredReason": force_expired_reason,
        "outcome": if ok { "executed" } else { "failed" },
        "error": audit_error,
        "result": output,
    });
    let audit_result = state
        .audit_handle
        .send_confirmed(RawAuditEvent {
            event_type: if ok {
                AuditEventType::UndoExecuted
            } else {
                AuditEventType::UndoFailed
            },
            trace_id: trace_id.to_string(),
            request_id: entry.batch_id.clone(),
            actor_id: operator_id.map(ToString::to_string).or_else(|| {
                context
                    .and_then(|value| value.actor_id.clone())
                    .or_else(|| entry.actor_id.clone())
            }),
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
            error_message: audit_error.map(ToString::to_string),
            operator_id: operator_id.map(ToString::to_string),
            approval_ticket_id: None,
            idempotency_key: Some(idempotency_key),
            is_replay: Some(false),
        })
        .await;
    RollbackResult {
        undo_id: entry.undo_id.clone(),
        operation_id: entry.operation_id.clone(),
        capability: capability.clone(),
        ok,
        error,
        error_code: if ok {
            None
        } else {
            Some("UNDO_FAILED".to_string())
        },
        output,
        audit_event_id: audit_result.as_ref().ok().map(|record| record.id.clone()),
        audit_hash: audit_result.as_ref().ok().map(|record| record.hash.clone()),
        audit_error: audit_result.err(),
        forced_expired: force_expired_reason.is_some(),
        entry_status: None,
    }
}

/// Executes one tenant-owned undo entry after atomically claiming it in the ledger.
pub async fn execute_operator_undo(
    state: &AppState,
    tenant_id: &str,
    undo_id: &str,
    scope: FiberExecutionScope<'_>,
    force_expired: bool,
    force_reason: Option<&str>,
) -> Option<RollbackResult> {
    let ledger = state.undo_ledger.as_ref()?;
    let entry = ledger.get(undo_id).await?;
    if entry.tenant_id != tenant_id {
        return None;
    }
    if force_expired && force_reason.is_none_or(str::is_empty) {
        let mut result = failed_result(
            &entry,
            "a reason is required to force an expired undo".to_string(),
        );
        result.error_code = Some("UNDO_FORCE_REASON_REQUIRED".to_string());
        result.entry_status = Some(entry.status);
        return Some(result);
    }
    let claim = match ledger
        .claim_for_operator(undo_id, now_ms(), force_expired)
        .await
    {
        Ok(claim) => claim,
        Err(error) => {
            let mut result = failed_result(&entry, error.to_string());
            result.error_code = Some("UNDO_PERSISTENCE_FAILED".to_string());
            result.entry_status = Some(entry.status);
            return Some(result);
        }
    };
    let claimed = match claim {
        crate::undo::UndoClaimResult::Claimed(entry) => entry,
        crate::undo::UndoClaimResult::Expired(entry) => {
            let mut result = failed_result(&entry, "undo window expired".to_string());
            result.error_code = Some("UNDO_WINDOW_EXPIRED".to_string());
            result.entry_status = Some(UndoEntryStatus::Expired);
            return Some(result);
        }
        crate::undo::UndoClaimResult::Unavailable(Some(entry)) => {
            let code = match entry.status {
                UndoEntryStatus::Reverted => "UNDO_ALREADY_REVERTED",
                UndoEntryStatus::Executing => "UNDO_OUTCOME_UNKNOWN",
                UndoEntryStatus::Expired => "UNDO_WINDOW_EXPIRED",
                UndoEntryStatus::Failed => "UNDO_PREVIOUSLY_FAILED",
                UndoEntryStatus::Armed => "UNDO_UNAVAILABLE",
            };
            let mut result = failed_result(
                &entry,
                format!("undo entry is {}", status_name(&entry.status)),
            );
            result.error_code = Some(code.to_string());
            result.entry_status = Some(entry.status);
            return Some(result);
        }
        crate::undo::UndoClaimResult::Unavailable(None) => return None,
    };
    let mut dispatch_scope = scope;
    dispatch_scope.force_expired_reason = force_reason;
    if let Some(result) = block_on_pre_undo_drift(state, ledger, &claimed, &dispatch_scope).await {
        return Some(result);
    }
    let mut outcome = dispatch_inverse(state, &claimed, dispatch_scope).await;
    let terminal = if outcome.ok {
        UndoEntryStatus::Reverted
    } else {
        UndoEntryStatus::Failed
    };
    match ledger.finish(undo_id, terminal.clone()).await {
        Ok(()) => outcome.entry_status = Some(terminal),
        Err(error) => {
            outcome.error_code = Some("UNDO_PERSISTENCE_FAILED".to_string());
            outcome.error = Some(format!(
                "inverse dispatched but ledger completion was not persisted: {error}"
            ));
            outcome.entry_status = Some(UndoEntryStatus::Executing);
        }
    }
    state.notify_action_capsules_changed();
    Some(outcome)
}

/// Rolls back active entries for a work item in reverse creation order.
pub async fn rollback_operator_work_item(
    state: &AppState,
    tenant_id: &str,
    work_item_id: &str,
    scope: FiberExecutionScope<'_>,
    force_expired: bool,
    force_reason: Option<&str>,
) -> Vec<RollbackResult> {
    let Some(ledger) = state.undo_ledger.as_ref() else {
        return Vec::new();
    };
    let entries = ledger
        .list_for_tenant(tenant_id, Some(work_item_id), None)
        .await
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.status,
                UndoEntryStatus::Armed | UndoEntryStatus::Expired
            )
        })
        .collect::<Vec<_>>();
    let started = std::time::Instant::now();
    let mut outcomes = Vec::with_capacity(entries.len());
    for entry in entries {
        if started.elapsed() >= MAX_FIBER_UNWIND_DURATION {
            outcomes.push(failed_result(
                &entry,
                "work-item rollback budget exhausted".to_string(),
            ));
            continue;
        }
        if let Some(outcome) = execute_operator_undo(
            state,
            tenant_id,
            &entry.undo_id,
            FiberExecutionScope {
                policy: scope.policy,
                profile: scope.profile,
                context: scope.context,
                authorization_expires_at_epoch_secs: scope.authorization_expires_at_epoch_secs,
                trace_id: scope.trace_id,
                operator_id: scope.operator_id,
                force_expired_reason: scope.force_expired_reason,
            },
            force_expired,
            force_reason,
        )
        .await
        {
            outcomes.push(outcome);
        }
    }
    outcomes
}

fn status_name(status: &UndoEntryStatus) -> &'static str {
    match status {
        UndoEntryStatus::Armed => "armed",
        UndoEntryStatus::Executing => "executing",
        UndoEntryStatus::Reverted => "reverted",
        UndoEntryStatus::Expired => "expired",
        UndoEntryStatus::Failed => "failed",
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
