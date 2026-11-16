// Rust guideline compliant 2026-11-16
//! HTTP v1 endpoints for tenant-scoped undo inspection and execution.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    context::{resolve_request_context, RequestContext},
    daemon::AppState,
    undo::{UndoAccessContext, UndoEntryStatus, UndoListFilter},
};

/// Query parameters for listing undo entries.
#[derive(Clone, Debug, Deserialize)]
pub struct UndoListQuery {
    /// Optional work-item filter.
    pub work_item_id: Option<String>,
    /// Optional lifecycle state (`armed`, `executing`, `reverted`, `expired`, or `failed`).
    pub status: Option<String>,
    /// Result offset, defaulting to zero.
    pub offset: Option<usize>,
    /// Page size, defaulting to 50 and capped at 200.
    pub limit: Option<usize>,
}

/// Request body for executing one undo or rolling back a work item.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UndoExecutionRequest {
    /// Explicitly overrides an expired undo window; requires elevated permission.
    #[serde(default, rename = "forceExpired")]
    pub force_expired: bool,
    /// Required explanation when overriding expiry.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Handles `GET /v1/undo`.
pub async fn handle_list_undo(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Query(query): Query<UndoListQuery>,
) -> impl IntoResponse {
    let status = match query.status.as_deref().map(parse_status).transpose() {
        Ok(status) => status,
        Err(message) => {
            return error_response(StatusCode::BAD_REQUEST, "INVALID_UNDO_STATUS", message)
        }
    };
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), None).await;
    let handle = crate::engine::ControlPlaneHandle::new(state);
    match handle
        .list_undo_entries(
            UndoListFilter {
                work_item_id: query.work_item_id,
                status,
                offset: query.offset.unwrap_or(0),
                limit: query.limit.unwrap_or(50),
            },
            &access,
        )
        .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(json!({"ok": true, "result": response})),
        )
            .into_response(),
        Err(error) => operation_error_response(error),
    }
}

/// Handles `GET /v1/undo/:undo_id`.
pub async fn handle_get_undo(
    State(state): State<AppState>,
    Path(undo_id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
) -> impl IntoResponse {
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), None).await;
    let handle = crate::engine::ControlPlaneHandle::new(state);
    match handle.get_undo_entry(&undo_id, &access).await {
        Ok(entry) => (StatusCode::OK, Json(json!({"ok": true, "result": entry}))).into_response(),
        Err(error) => operation_error_response(error),
    }
}

/// Handles `POST /v1/undo/:undo_id/execute`.
pub async fn handle_execute_undo(
    State(state): State<AppState>,
    Path(undo_id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UndoExecutionRequest>,
) -> impl IntoResponse {
    let context = resolve_request_context(None, &headers);
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), Some(context)).await;
    let handle = crate::engine::ControlPlaneHandle::new(state);
    match handle
        .execute_undo(
            &undo_id,
            &access,
            request.force_expired,
            request.reason.as_deref(),
        )
        .await
    {
        Ok(result) if result.audit_error.is_some() => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": {"code": "AUDIT_PERSISTENCE_FAILED", "message": result.audit_error},
                "result": result
            })),
        )
            .into_response(),
        Ok(result) if !result.ok => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"ok": false, "error": {"code": result.error_code, "message": result.error}, "result": result})),
        )
            .into_response(),
        Ok(result) => (StatusCode::OK, Json(json!({"ok": true, "result": result}))).into_response(),
        Err(error) => operation_error_response(error),
    }
}

/// Handles `POST /v1/undo/work-items/:work_item_id/rollback`.
pub async fn handle_rollback_work_item(
    State(state): State<AppState>,
    Path(work_item_id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UndoExecutionRequest>,
) -> impl IntoResponse {
    let context = resolve_request_context(None, &headers);
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), Some(context)).await;
    let handle = crate::engine::ControlPlaneHandle::new(state);
    match handle
        .rollback_work_item(
            &work_item_id,
            &access,
            request.force_expired,
            request.reason.as_deref(),
        )
        .await
    {
        Ok(results) if results.iter().any(|result| result.audit_error.is_some()) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"ok": false, "error": {"code": "AUDIT_PERSISTENCE_FAILED", "message": "One or more undo audit events could not be committed"}, "results": results})),
        )
            .into_response(),
        Ok(results) => (
            StatusCode::OK,
            Json(json!({
                "ok": results.iter().all(|result| result.ok),
                "results": results
            })),
        )
            .into_response(),
        Err(error) => operation_error_response(error),
    }
}

async fn make_access(
    state: &AppState,
    tenant: Option<&crate::rbac::TenantContext>,
    profile: Option<&axum::extract::Extension<crate::context::ProfileContext>>,
    context: Option<RequestContext>,
) -> UndoAccessContext {
    let profile = profile
        .map(|extension| extension.0.clone())
        .unwrap_or_default();
    let base_policy = state.policy.read().await.clone();
    let policy = tenant
        .map(|tenant| tenant.effective_policy.clone())
        .unwrap_or(base_policy)
        .merge_with_profile(profile.profile_policy.as_ref());
    let default_role = state.rbac_engine.default_role();
    let role = tenant
        .map(|tenant| tenant.role.clone())
        .unwrap_or(default_role);
    let actor_id = tenant
        .and_then(|tenant| tenant.actor_id.clone())
        .or_else(|| {
            context
                .as_ref()
                .and_then(|context| context.actor_id.clone())
        });
    let mut request_context = context.unwrap_or_default();
    if request_context.actor_id.is_none() {
        request_context.actor_id = actor_id.clone();
    }
    UndoAccessContext {
        tenant_id: tenant
            .map(|tenant| tenant.tenant_id.clone())
            .unwrap_or_else(|| "default".to_string()),
        role,
        actor_id,
        request_context,
        policy,
        profile,
        grant_expires_at_epoch_secs: tenant.and_then(|tenant| tenant.grant_expires_at_epoch_secs),
        trace_id: crate::http_v1::next_trace_id(),
    }
}

fn parse_status(value: &str) -> Result<UndoEntryStatus, String> {
    match value {
        "armed" => Ok(UndoEntryStatus::Armed),
        "executing" => Ok(UndoEntryStatus::Executing),
        "reverted" => Ok(UndoEntryStatus::Reverted),
        "expired" => Ok(UndoEntryStatus::Expired),
        "failed" => Ok(UndoEntryStatus::Failed),
        _ => Err("status must be armed, executing, reverted, expired, or failed".to_string()),
    }
}

fn operation_error_response(error: crate::undo::UndoOperationError) -> axum::response::Response {
    let status =
        StatusCode::from_u16(error.http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    error_response(status, error.code, error.message)
}

fn error_response(status: StatusCode, code: &str, message: String) -> axum::response::Response {
    (
        status,
        Json(json!({"ok": false, "error": {"code": code, "message": message}})),
    )
        .into_response()
}
