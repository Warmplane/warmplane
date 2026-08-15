// Rust guideline compliant 2026-08-15

//! Human-in-the-Loop (HITL) approval endpoints for querying, approving, and rejecting suspended executions.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    daemon::AppState,
    http_v1::types::{ApproveTicketRequest, RejectTicketRequest},
};

/// Handles GET `/v1/approvals` returning active pending tickets and recent history.
pub async fn handle_list_approvals(State(state): State<AppState>) -> impl IntoResponse {
    let approvals = state.approval_registry.list().await;
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "approvals": approvals,
            "total": approvals.len(),
        })),
    )
        .into_response()
}

/// Handles GET `/v1/approvals/:id` returning details of a single approval ticket.
pub async fn handle_get_approval(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.approval_registry.get(&id).await {
        Some(approval) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "approval": approval,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": format!("Approval ticket '{}' not found", id),
            })),
        )
            .into_response(),
    }
}

/// Handles POST `/v1/approvals/:id/approve` approving a suspended capability execution.
pub async fn handle_approve_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ApproveTicketRequest>,
) -> impl IntoResponse {
    let webhook_cfg = state.policy.read().await.webhook.clone();
    match state
        .approval_registry
        .approve(
            &id,
            payload.operator,
            payload.modified_args,
            webhook_cfg.as_ref(),
        )
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "message": format!("Ticket '{}' approved successfully", id),
            })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::CONFLICT,
            Json(json!({
                "ok": false,
                "error": format!("Ticket '{}' is not pending or already processed", id),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": e.to_string(),
            })),
        )
            .into_response(),
    }
}

/// Handles POST `/v1/approvals/:id/reject` rejecting a suspended capability execution.
pub async fn handle_reject_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RejectTicketRequest>,
) -> impl IntoResponse {
    let webhook_cfg = state.policy.read().await.webhook.clone();
    match state
        .approval_registry
        .reject(&id, payload.operator, payload.reason, webhook_cfg.as_ref())
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "message": format!("Ticket '{}' rejected successfully", id),
            })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::CONFLICT,
            Json(json!({
                "ok": false,
                "error": format!("Ticket '{}' is not pending or already processed", id),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": e.to_string(),
            })),
        )
            .into_response(),
    }
}
