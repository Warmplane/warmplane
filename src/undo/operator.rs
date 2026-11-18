// Rust guideline compliant 2026-11-16
//! Authorization-aware operator operations for persisted undo entries.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    audit::{AuditEventStatus, AuditEventType, RawAuditEvent},
    context::{ProfileContext, RequestContext},
    daemon::{AppState, Policy},
    undo::{
        execute_operator_undo, rollback_operator_work_item, FiberExecutionScope, RollbackResult,
        UndoEntry, UndoEntryStatus,
    },
};

/// Trusted caller scope for out-of-band undo operations.
#[derive(Clone, Debug)]
pub struct UndoAccessContext {
    /// Authenticated tenant identifier.
    pub tenant_id: String,
    /// Authenticated role name.
    pub role: String,
    /// Authenticated actor identifier, when available.
    pub actor_id: Option<String>,
    /// Trusted request context retained for audit provenance.
    pub request_context: RequestContext,
    /// Current effective policy for inverse capability execution.
    pub policy: Policy,
    /// Current profile boundary and policy overlay.
    pub profile: ProfileContext,
    /// Verified expiry of the caller grant, if time limited.
    pub grant_expires_at_epoch_secs: Option<u64>,
    /// Trace identifier for inverse dispatch and audit records.
    pub trace_id: String,
}

/// Filters and pagination for listing undo entries.
#[derive(Clone, Debug, Default)]
pub struct UndoListFilter {
    /// Optional work-item identifier filter.
    pub work_item_id: Option<String>,
    /// Optional lifecycle status filter.
    pub status: Option<UndoEntryStatus>,
    /// Number of matching entries to skip.
    pub offset: usize,
    /// Maximum number of entries to return; defaults to 50 and caps at 200.
    pub limit: usize,
}

/// Safe API representation of one undo entry, excluding inverse arguments and snapshot contents.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UndoEntrySummary {
    /// Stable entry identifier.
    pub undo_id: String,
    /// Forward operation identifier, when present.
    pub operation_id: Option<String>,
    /// Work item identifier, when present.
    pub work_item_id: Option<String>,
    /// Tenant identifier owning the entry.
    pub tenant_id: String,
    /// Profile used for the forward call, when present.
    pub profile_id: Option<String>,
    /// Actor that performed the forward call, when present.
    pub actor_id: Option<String>,
    /// Forward capability identifier.
    pub forward_capability: String,
    /// Inverse capability identifier.
    pub inverse_capability: String,
    /// Safe field names used by the configured inverse, when available.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub change_preview: Vec<String>,
    /// Hash of the saved pre-state snapshot, when present.
    pub snapshot_artifact_hash: Option<String>,
    /// Creation timestamp in Unix milliseconds.
    pub created_at_ms: u64,
    /// Undo expiration timestamp in Unix milliseconds.
    pub expires_at_ms: u64,
    /// Current lifecycle status.
    pub status: UndoEntryStatus,
}

impl From<&UndoEntry> for UndoEntrySummary {
    fn from(entry: &UndoEntry) -> Self {
        Self {
            undo_id: entry.undo_id.clone(),
            operation_id: entry.operation_id.clone(),
            work_item_id: entry.work_item_id.clone(),
            tenant_id: entry.tenant_id.clone(),
            profile_id: entry.profile_id.clone(),
            actor_id: entry.actor_id.clone(),
            forward_capability: entry.forward_capability.clone(),
            inverse_capability: entry.resolved_inverse_capability.clone(),
            change_preview: entry.change_preview.clone(),
            snapshot_artifact_hash: entry.snapshot_artifact_hash.clone(),
            created_at_ms: entry.created_at_ms,
            expires_at_ms: entry.expires_at_ms,
            status: entry.status.clone(),
        }
    }
}

/// Paginated result for listing tenant-owned undo entries.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UndoListResponse {
    /// Total matching entries before pagination.
    pub total: usize,
    /// Returned safe entry summaries.
    pub entries: Vec<UndoEntrySummary>,
    /// Pagination offset used for this response.
    pub offset: usize,
    /// Pagination limit used for this response.
    pub limit: usize,
}

/// API-level failure from an undo operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoOperationError {
    /// Machine-readable error code.
    pub code: &'static str,
    /// Human-readable explanation.
    pub message: String,
    /// HTTP status code to use for REST callers.
    pub http_status: u16,
}

/// Lists only entries owned by the authenticated tenant.
pub async fn list_undo_entries(
    state: &AppState,
    access: &UndoAccessContext,
    filter: UndoListFilter,
) -> Result<UndoListResponse, UndoOperationError> {
    require_permission(state, access, UndoPermission::List)?;
    ensure_grant_valid(access)?;
    let ledger = state.undo_ledger.as_ref().ok_or_else(|| {
        operation_error(
            "UNDO_STORAGE_UNAVAILABLE",
            "Undo storage is unavailable",
            503,
        )
    })?;
    let entries = ledger
        .list_for_tenant(
            &access.tenant_id,
            filter.work_item_id.as_deref(),
            filter.status.as_ref(),
        )
        .await;
    let total = entries.len();
    let limit = if filter.limit == 0 {
        50
    } else {
        filter.limit.min(200)
    };
    let entries = entries
        .into_iter()
        .skip(filter.offset)
        .take(limit)
        .map(|entry| UndoEntrySummary::from(&entry))
        .collect();
    Ok(UndoListResponse {
        total,
        entries,
        offset: filter.offset,
        limit,
    })
}

/// Shows one entry only when it belongs to the authenticated tenant.
pub async fn get_undo_entry(
    state: &AppState,
    access: &UndoAccessContext,
    undo_id: &str,
) -> Result<UndoEntrySummary, UndoOperationError> {
    require_permission(state, access, UndoPermission::List)?;
    ensure_grant_valid(access)?;
    let entry = state
        .undo_ledger
        .clone()
        .ok_or_else(|| {
            operation_error(
                "UNDO_STORAGE_UNAVAILABLE",
                "Undo storage is unavailable",
                503,
            )
        })?
        .get(undo_id)
        .await
        .filter(|entry| entry.tenant_id == access.tenant_id)
        .ok_or_else(not_found)?;
    Ok(UndoEntrySummary::from(&entry))
}

/// Executes one undo entry with permission, tenant, expiry, and audit checks.
pub async fn execute_undo(
    state: &AppState,
    access: &UndoAccessContext,
    undo_id: &str,
    force_expired: bool,
    force_reason: Option<&str>,
) -> Result<RollbackResult, UndoOperationError> {
    require_permission(state, access, UndoPermission::Execute)?;
    ensure_grant_valid(access)?;
    if state.undo_ledger.is_none() {
        return Err(operation_error(
            "UNDO_STORAGE_UNAVAILABLE",
            "Undo storage is unavailable",
            503,
        ));
    }
    if force_expired {
        require_permission(state, access, UndoPermission::ForceExpired)?;
        if force_reason.is_none_or(|reason| reason.trim().is_empty()) {
            return Err(operation_error(
                "UNDO_FORCE_REASON_REQUIRED",
                "A reason is required to force an expired undo",
                400,
            ));
        }
        let entry = state
            .undo_ledger
            .as_ref()
            .ok_or_else(|| {
                operation_error(
                    "UNDO_STORAGE_UNAVAILABLE",
                    "Undo storage is unavailable",
                    503,
                )
            })?
            .get(undo_id)
            .await
            .filter(|entry| entry.tenant_id == access.tenant_id)
            .ok_or_else(not_found)?;
        persist_force_authorization(
            state,
            access,
            force_reason.unwrap_or_default(),
            json!({
                "undoId": entry.undo_id,
            }),
        )
        .await?;
    }
    let result = execute_operator_undo(
        state,
        &access.tenant_id,
        undo_id,
        fiber_scope(access, force_reason),
        force_expired,
        force_reason,
    )
    .await
    .ok_or_else(not_found)?;
    map_result(result)
}

/// Rolls back one work item in LIFO order with per-entry outcomes.
pub async fn rollback_work_item(
    state: &AppState,
    access: &UndoAccessContext,
    work_item_id: &str,
    force_expired: bool,
    force_reason: Option<&str>,
) -> Result<Vec<RollbackResult>, UndoOperationError> {
    require_permission(state, access, UndoPermission::Rollback)?;
    ensure_grant_valid(access)?;
    if force_expired {
        require_permission(state, access, UndoPermission::ForceExpired)?;
        if force_reason.is_none_or(|reason| reason.trim().is_empty()) {
            return Err(operation_error(
                "UNDO_FORCE_REASON_REQUIRED",
                "A reason is required to force expired entries",
                400,
            ));
        }
    }
    if state.undo_ledger.is_none() {
        return Err(operation_error(
            "UNDO_STORAGE_UNAVAILABLE",
            "Undo storage is unavailable",
            503,
        ));
    }
    let Some(ledger) = state.undo_ledger.as_ref() else {
        return Err(operation_error(
            "UNDO_STORAGE_UNAVAILABLE",
            "Undo storage is unavailable",
            503,
        ));
    };
    if !ledger
        .list_for_tenant(&access.tenant_id, Some(work_item_id), None)
        .await
        .is_empty()
    {
        // The caller owns at least one entry, so an empty rollback is a valid no-op when all
        // entries are terminal. Otherwise the presence of another tenant's matching fiber is
        // concealed as not-found.
    } else if ledger
        .entries()
        .await
        .iter()
        .any(|entry| entry.work_item_id.as_deref() == Some(work_item_id))
    {
        return Err(not_found());
    }
    if force_expired {
        persist_force_authorization(
            state,
            access,
            force_reason.unwrap_or_default(),
            json!({"workItemId": work_item_id}),
        )
        .await?;
    }
    Ok(rollback_operator_work_item(
        state,
        &access.tenant_id,
        work_item_id,
        fiber_scope(access, force_reason),
        force_expired,
        force_reason,
    )
    .await)
}

async fn persist_force_authorization(
    state: &AppState,
    access: &UndoAccessContext,
    reason: &str,
    target: serde_json::Value,
) -> Result<(), UndoOperationError> {
    let work_item_id = target
        .get("workItemId")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .or_else(|| access.request_context.work_item_id.clone());
    state
        .audit_handle
        .send_confirmed(RawAuditEvent {
            event_type: AuditEventType::UndoForceAuthorized,
            trace_id: access.trace_id.clone(),
            request_id: access.request_context.operation_id.clone(),
            actor_id: access.actor_id.clone(),
            work_item_id,
            client_ip: None,
            server_id: None,
            capability_id: None,
            resource_uri: None,
            sanitized_args: Some(target),
            sanitized_response: Some(json!({"reason": reason})),
            execution_latency_us: None,
            status: AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: access.actor_id.clone(),
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: Some(false),
        })
        .await
        .map(|_| ())
        .map_err(|error| {
            operation_error(
                "AUDIT_PERSISTENCE_FAILED",
                format!("Could not persist expiry override authorization: {error}"),
                503,
            )
        })
}

/// Executes an agent-requested self-undo only when role and profile both opt in.
pub async fn execute_agent_self_undo(
    state: &AppState,
    access: &UndoAccessContext,
    undo_id: &str,
) -> Result<RollbackResult, UndoOperationError> {
    ensure_grant_valid(access)?;
    let role_opt_in = state.rbac_engine.allows_agent_self_undo(&access.role);
    let profile_opt_in =
        access.profile.profile_id.is_none() || access.profile.allow_agent_self_undo;
    if !role_opt_in || !profile_opt_in {
        return Err(operation_error(
            "POLICY_DENIED",
            "Agent self-undo is not enabled for this role and profile",
            403,
        ));
    }
    let _entry = state
        .undo_ledger
        .as_ref()
        .ok_or_else(|| {
            operation_error(
                "UNDO_STORAGE_UNAVAILABLE",
                "Undo storage is unavailable",
                503,
            )
        })?
        .get(undo_id)
        .await
        .filter(|entry| entry.tenant_id == access.tenant_id)
        .ok_or_else(not_found)?;
    execute_operator_undo(
        state,
        &access.tenant_id,
        undo_id,
        fiber_scope(access, None),
        false,
        None,
    )
    .await
    .ok_or_else(not_found)
    .and_then(map_result)
}

/// Executes the newest armed undo entry in a work item for an opted-in MCP agent.
pub async fn execute_agent_self_undo_for_work_item(
    state: &AppState,
    access: &UndoAccessContext,
    work_item_id: &str,
) -> Result<RollbackResult, UndoOperationError> {
    ensure_grant_valid(access)?;
    let role_opt_in = state.rbac_engine.allows_agent_self_undo(&access.role);
    let profile_opt_in =
        access.profile.profile_id.is_none() || access.profile.allow_agent_self_undo;
    if !role_opt_in || !profile_opt_in {
        return Err(operation_error(
            "POLICY_DENIED",
            "Agent self-undo is not enabled for this role and profile",
            403,
        ));
    }
    let ledger = state.undo_ledger.as_ref().ok_or_else(|| {
        operation_error(
            "UNDO_STORAGE_UNAVAILABLE",
            "Undo storage is unavailable",
            503,
        )
    })?;
    let entry = ledger
        .list_for_tenant(
            &access.tenant_id,
            Some(work_item_id),
            Some(&UndoEntryStatus::Armed),
        )
        .await
        .into_iter()
        .next()
        .ok_or_else(not_found)?;
    execute_operator_undo(
        state,
        &access.tenant_id,
        &entry.undo_id,
        fiber_scope(access, None),
        false,
        None,
    )
    .await
    .ok_or_else(not_found)
    .and_then(map_result)
}

#[derive(Clone, Copy)]
enum UndoPermission {
    List,
    Execute,
    Rollback,
    ForceExpired,
}

fn require_permission(
    state: &AppState,
    access: &UndoAccessContext,
    permission: UndoPermission,
) -> Result<(), UndoOperationError> {
    if !state.rbac_engine.is_enabled() && !matches!(permission, UndoPermission::ForceExpired) {
        return Ok(());
    }
    let permissions = state.rbac_engine.undo_permissions(&access.role);
    let allowed = match permission {
        UndoPermission::List => permissions.list,
        UndoPermission::Execute => permissions.execute,
        UndoPermission::Rollback => permissions.rollback,
        UndoPermission::ForceExpired => permissions.force_expired,
    };
    if allowed {
        Ok(())
    } else {
        Err(operation_error(
            "POLICY_DENIED",
            "Caller role is not authorized for this undo operation",
            403,
        ))
    }
}

fn ensure_grant_valid(access: &UndoAccessContext) -> Result<(), UndoOperationError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if access
        .grant_expires_at_epoch_secs
        .is_some_and(|expiry| expiry <= now)
    {
        Err(operation_error(
            "UNAUTHORIZED",
            "Caller authorization expired",
            401,
        ))
    } else {
        Ok(())
    }
}

fn fiber_scope<'a>(
    access: &'a UndoAccessContext,
    force_reason: Option<&'a str>,
) -> FiberExecutionScope<'a> {
    FiberExecutionScope {
        policy: &access.policy,
        profile: &access.profile,
        context: Some(&access.request_context),
        authorization_expires_at_epoch_secs: access.grant_expires_at_epoch_secs,
        trace_id: &access.trace_id,
        operator_id: access.actor_id.as_deref(),
        force_expired_reason: force_reason,
    }
}

fn map_result(result: RollbackResult) -> Result<RollbackResult, UndoOperationError> {
    let error_code = result.error_code.clone().unwrap_or_default();
    match error_code.as_str() {
        "UNDO_WINDOW_EXPIRED" => Err(operation_error(
            "UNDO_WINDOW_EXPIRED",
            "Undo window expired",
            409,
        )),
        "UNDO_STATE_DRIFT" => Err(operation_error(
            "UNDO_STATE_DRIFT",
            "Upstream state changed after the forward action; the inverse was not dispatched",
            409,
        )),
        "UNDO_STATE_CHECK_FAILED" => Err(operation_error(
            "UNDO_STATE_CHECK_FAILED",
            "Upstream state could not be verified; the inverse was not dispatched",
            503,
        )),
        "UNDO_ALREADY_REVERTED" => Err(operation_error(
            "UNDO_ALREADY_REVERTED",
            result
                .error
                .unwrap_or_else(|| "Undo entry is unavailable".to_string()),
            409,
        )),
        "UNDO_OUTCOME_UNKNOWN" => Err(operation_error(
            "UNDO_OUTCOME_UNKNOWN",
            result
                .error
                .unwrap_or_else(|| "Undo outcome is unknown and must be reconciled".to_string()),
            409,
        )),
        "UNDO_PREVIOUSLY_FAILED" => Err(operation_error(
            "UNDO_PREVIOUSLY_FAILED",
            result
                .error
                .unwrap_or_else(|| "Undo previously failed".to_string()),
            409,
        )),
        "UNDO_UNAVAILABLE" => Err(operation_error(
            "UNDO_UNAVAILABLE",
            result
                .error
                .unwrap_or_else(|| "Undo entry is unavailable".to_string()),
            409,
        )),
        "UNDO_FORCE_REASON_REQUIRED" => Err(operation_error(
            "UNDO_FORCE_REASON_REQUIRED",
            "A reason is required to force an expired undo",
            400,
        )),
        "UNDO_PERSISTENCE_FAILED" => Err(operation_error(
            "UNDO_PERSISTENCE_FAILED",
            result
                .error
                .unwrap_or_else(|| "Undo state could not be persisted".to_string()),
            503,
        )),
        _ => Ok(result),
    }
}

fn not_found() -> UndoOperationError {
    operation_error("UNDO_NOT_FOUND", "Undo entry was not found", 404)
}

fn operation_error(
    code: &'static str,
    message: impl Into<String>,
    http_status: u16,
) -> UndoOperationError {
    UndoOperationError {
        code,
        message: message.into(),
        http_status,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        persist_force_authorization, require_permission, UndoAccessContext, UndoPermission,
    };

    #[tokio::test]
    async fn force_expiry_reason_is_committed_before_dispatch() {
        let state = crate::daemon::AppState::builder().build();
        let access = UndoAccessContext {
            tenant_id: "default".to_string(),
            role: "operator".to_string(),
            actor_id: Some("human-1".to_string()),
            request_context: crate::context::RequestContext {
                work_item_id: Some("work-1".to_string()),
                ..Default::default()
            },
            policy: crate::daemon::Policy::default(),
            profile: crate::context::ProfileContext::unrestricted(),
            grant_expires_at_epoch_secs: None,
            trace_id: "force-trace".to_string(),
        };

        persist_force_authorization(
            &state,
            &access,
            "incident-42",
            serde_json::json!({"undoId": "undo-1"}),
        )
        .await
        .unwrap();
        let (events, _) = state.audit_store.query(&Default::default()).await;
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event_type,
            crate::audit::AuditEventType::UndoForceAuthorized
        );
        assert_eq!(
            events[0].sanitized_response.as_ref().unwrap()["reason"],
            "incident-42"
        );
        assert_eq!(events[0].work_item_id.as_deref(), Some("work-1"));
        state.shutdown().await;
    }

    #[tokio::test]
    async fn expiry_override_requires_explicit_permission_even_without_general_rbac() {
        let role = "local-operator".to_string();
        let state = crate::daemon::AppState::builder()
            .rbac_engine(crate::rbac::RbacEngine::new(Some(
                crate::rbac::RbacConfig {
                    enabled: false,
                    default_role: role.clone(),
                    roles: std::collections::HashMap::from([(
                        role.clone(),
                        crate::rbac::RolePolicyConfig::default(),
                    )]),
                    ..Default::default()
                },
            )))
            .build();
        let access = UndoAccessContext {
            tenant_id: "default".to_string(),
            role,
            actor_id: None,
            request_context: crate::context::RequestContext::default(),
            policy: crate::daemon::Policy::default(),
            profile: crate::context::ProfileContext::unrestricted(),
            grant_expires_at_epoch_secs: None,
            trace_id: "test-trace".to_string(),
        };

        assert_eq!(
            require_permission(&state, &access, UndoPermission::ForceExpired)
                .unwrap_err()
                .http_status,
            403
        );
    }
}
