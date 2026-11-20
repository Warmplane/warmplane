// Rust guideline compliant 2026-11-18
//! HTTP v1 API endpoints for managing SEP-2663 Tasks (`M-CANONICAL-DOCS`).
//!
//! Exposes REST endpoints for querying task status, submitting input responses,
//! and cooperatively cancelling in-progress executions.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use crate::daemon::AppState;
use crate::tasks::TaskResponse;

/// Request payload for updating a task with input responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    /// Responses to outstanding inputRequests previously surfaced by the task.
    #[serde(rename = "inputResponses")]
    pub input_responses: BTreeMap<String, Value>,
}

/// Request payload for cancelling a task.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CancelTaskRequest {
    /// Optional cancellation reason.
    pub reason: Option<String>,
}

/// Response payload for listing tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResponse {
    pub ok: bool,
    pub total: usize,
    pub tasks: Vec<TaskResponse>,
}

/// Handles `GET /v1/tasks` - lists all known tasks.
pub async fn handle_list_tasks(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
) -> impl IntoResponse {
    let records = state.task_registry.list_tasks().await;
    let mut responses = Vec::new();
    for record in &records {
        if crate::http_v1::helpers::can_access_emission_task(
            &state,
            record,
            req_ext.0.as_ref(),
            prof_ext.as_ref().map(|extension| &extension.0),
        )
        .await
        {
            responses.push(state.task_view(record).await);
        }
    }
    let total = responses.len();

    (
        StatusCode::OK,
        Json(ListTasksResponse {
            ok: true,
            total,
            tasks: responses,
        }),
    )
}

/// Handles `GET /v1/tasks/:id` - gets single task status.
pub async fn handle_get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
) -> impl IntoResponse {
    if let Some(record) = state.task_registry.get_task(&id).await {
        if !crate::http_v1::helpers::can_access_emission_task(
            &state,
            &record,
            req_ext.0.as_ref(),
            prof_ext.as_ref().map(|extension| &extension.0),
        )
        .await
        {
            return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "error": {"code": "TASK_NOT_FOUND", "message": format!("Task '{}' was not found", id)}}))).into_response();
        }
        let resp = state.task_view(&record).await;
        (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "resultType": "complete",
                "task": resp,
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "TASK_NOT_FOUND",
                    "message": format!("Task '{}' was not found", id),
                }
            })),
        )
            .into_response()
    }
}

/// Handles `POST /v1/tasks/:id/update` - submits input responses to `input_required` task.
pub async fn handle_update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    tenant: Option<axum::extract::Extension<Option<crate::rbac::TenantContext>>>,
    profile: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Json(payload): Json<UpdateTaskRequest>,
) -> axum::response::Response {
    let tenant = tenant.as_ref().and_then(|tenant| tenant.0.as_ref());
    let profile = profile
        .as_ref()
        .and_then(|profile| profile.0.profile_id.as_deref());
    match state
        .submit_task_input(&id, payload.input_responses, tenant, profile)
        .await
    {
        Ok(receipt) => Json(json!({
            "ok": true,
            "resultType": "complete",
            "receipt": receipt,
        }))
        .into_response(),
        Err(error) => crate::http_v1::approvals_api::decision_error(error),
    }
}

/// Handles `POST /v1/tasks/:id/cancel` - cancels an active task.
pub async fn handle_cancel_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Json(payload): Json<CancelTaskRequest>,
) -> impl IntoResponse {
    let task_opt = state.task_registry.get_task(&id).await;
    if let Some(task) = task_opt.as_ref() {
        if !crate::http_v1::helpers::can_access_emission_task(
            &state,
            task,
            req_ext.0.as_ref(),
            prof_ext.as_deref(),
        )
        .await
        {
            return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "error": {"code": "TASK_NOT_FOUND", "message": format!("Task '{}' was not found", id)}}))).into_response();
        }
    }
    match state
        .hold_coordinator
        .cancel(&state, &id, payload.reason.clone())
        .await
    {
        Ok(crate::tasks::EmissionCancelResult::Cancelled)
        | Ok(crate::tasks::EmissionCancelResult::AlreadyCancelled) => {
            return (
                StatusCode::OK,
                Json(json!({"ok": true, "resultType": "complete", "message": format!("Task '{}' cancelled", id)})),
            )
                .into_response();
        }
        Ok(crate::tasks::EmissionCancelResult::TooLate) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({"ok": false, "error": {"code": "EMISSION_DISPATCHING", "message": "Emission dispatch has already started and may have reached the upstream server"}})),
            )
                .into_response();
        }
        Ok(crate::tasks::EmissionCancelResult::NotEmission) => {}
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"ok": false, "error": {"code": "TASK_CANCEL_FAILED", "message": error.to_string()}})),
            )
                .into_response();
        }
    }
    cancel_regular_task(&state, &id, payload.reason).await
}

async fn cancel_regular_task(
    state: &AppState,
    id: &str,
    reason: Option<String>,
) -> axum::response::Response {
    match state.task_registry.cancel_task(id, reason).await {
        Ok(true) => {
            if let Some(ticket) = state.approval_registry.find_by_task(id).await {
                state.approval_registry.expire_if_pending(&ticket.id).await;
            }

            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "resultType": "complete",
                    "message": format!("Task '{}' cancelled", id),
                })),
            )
                .into_response()
        }
        Ok(false) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "TASK_NOT_CANCELLABLE",
                    "message": format!("Task '{}' is already completed, cancelled, or not found", id),
                }
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "TASK_CANCEL_FAILED",
                    "message": e.to_string(),
                }
            })),
        )
            .into_response(),
    }
}
