// Rust guideline compliant 2026-08-27

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
pub async fn handle_list_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let records = state.task_registry.list_tasks().await;
    let responses: Vec<TaskResponse> = records.iter().map(TaskResponse::from).collect();
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
) -> impl IntoResponse {
    if let Some(record) = state.task_registry.get_task(&id).await {
        let resp = TaskResponse::from(&record);
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
    Json(payload): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let task_opt = state.task_registry.get_task(&id).await;
    let input_responses = payload.input_responses;

    match state.task_registry.update_task(&id, input_responses.clone()).await {
        Ok(true) => {
            // If the task corresponds to an approval request (either explicitly via hitl_approval or linked request_id),
            // cross-resolve the ApprovalRegistry so synchronous callers awaiting approval unblock immediately.
            if let Some(task) = task_opt {
                let req_id = task.request_id.as_deref().unwrap_or(&id);
                if let Some(pending_appr) = state.approval_registry.get_pending_by_request_id(req_id).await {
                    let webhook_cfg = state.policy.read().await.webhook.clone();
                    if let Some(appr_resp) = input_responses.get("hitl_approval") {
                        let is_approved = appr_resp.get("approved").and_then(Value::as_bool).unwrap_or(true);
                        let mod_args = appr_resp.get("modified_args").cloned();
                        let operator = appr_resp
                            .get("operator")
                            .and_then(Value::as_str)
                            .unwrap_or("security-operator")
                            .to_string();
                        let reason = appr_resp.get("reason").and_then(Value::as_str).map(ToString::to_string);

                        if is_approved {
                            let _ = state
                                .approval_registry
                                .approve(&pending_appr.id, operator, mod_args, webhook_cfg.as_ref())
                                .await;
                        } else {
                            let _ = state
                                .approval_registry
                                .reject(&pending_appr.id, operator, reason, webhook_cfg.as_ref())
                                .await;
                        }
                    } else {
                        // Default approval with unmodified args if general input was supplied
                        let _ = state
                            .approval_registry
                            .approve(
                                &pending_appr.id,
                                "security-operator".to_string(),
                                None,
                                webhook_cfg.as_ref(),
                            )
                            .await;
                    }
                }
            }

            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "resultType": "complete",
                    "message": format!("Task '{}' updated with input responses", id),
                })),
            )
                .into_response()
        }
        Ok(false) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "INVALID_TASK_STATE",
                    "message": format!("Task '{}' is not in 'input_required' state or not found", id),
                }
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "TASK_UPDATE_FAILED",
                    "message": e.to_string(),
                }
            })),
        )
            .into_response(),
    }
}

/// Handles `POST /v1/tasks/:id/cancel` - cancels an active task.
pub async fn handle_cancel_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CancelTaskRequest>,
) -> impl IntoResponse {
    let task_opt = state.task_registry.get_task(&id).await;
    match state.task_registry.cancel_task(&id, payload.reason.clone()).await {
        Ok(true) => {
            if let Some(task) = task_opt {
                let req_id = task.request_id.as_deref().unwrap_or(&id);
                if let Some(pending_appr) = state.approval_registry.get_pending_by_request_id(req_id).await {
                    let webhook_cfg = state.policy.read().await.webhook.clone();
                    let _ = state
                        .approval_registry
                        .reject(
                            &pending_appr.id,
                            "security-operator".to_string(),
                            payload.reason.or_else(|| Some("Task cancelled by operator".to_string())),
                            webhook_cfg.as_ref(),
                        )
                        .await;
                }
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
