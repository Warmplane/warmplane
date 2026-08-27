// Rust guideline compliant 2026-08-27

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
    let opt_ticket = state.approval_registry.get(&id).await;
    match state
        .approval_registry
        .approve(
            &id,
            payload.operator.clone(),
            payload.modified_args.clone(),
            webhook_cfg.as_ref(),
        )
        .await
    {
        Ok(true) => {
            if let Some(ticket) = opt_ticket {
                // If a matching Task exists for this approval, unblock it with input responses
                if let Some(ref req_id) = ticket.request_id {
                    let tasks = state.task_registry.list_tasks().await;
                    if let Some(matching_task) = tasks
                        .iter()
                        .find(|t| t.request_id.as_deref() == Some(req_id.as_str()))
                    {
                        let mut resp_map = std::collections::BTreeMap::new();
                        resp_map.insert(
                            "hitl_approval".to_string(),
                            serde_json::json!({
                                "approved": true,
                                "operator": payload.operator,
                                "modified_args": payload.modified_args,
                            }),
                        );
                        let _ = state
                            .task_registry
                            .update_task(&matching_task.task_id, resp_map)
                            .await;
                    }
                }

                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ApprovalGranted,
                    trace_id: format!("appr_{}", id),
                    request_id: ticket.request_id,
                    actor_id: ticket.context.as_ref().and_then(|c| c.actor_id.clone()),
                    work_item_id: ticket.context.as_ref().and_then(|c| c.work_item_id.clone()),
                    client_ip: None,
                    server_id: Some(ticket.server_id),
                    capability_id: Some(ticket.capability_id),
                    resource_uri: None,
                    sanitized_args: payload.modified_args.or(Some(ticket.sanitized_args)),
                    sanitized_response: None,
                    execution_latency_us: None,
                    status: crate::audit::AuditEventStatus::Success,
                    error_code: None,
                    error_message: None,
                    operator_id: Some(payload.operator),
                    approval_ticket_id: Some(id.clone()),
                    idempotency_key: None,
                    is_replay: None,
                });
            }
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "message": format!("Ticket '{}' approved successfully", id),
                })),
            )
                .into_response()
        }
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
    let opt_ticket = state.approval_registry.get(&id).await;
    match state
        .approval_registry
        .reject(
            &id,
            payload.operator,
            payload.reason.clone(),
            webhook_cfg.as_ref(),
        )
        .await
    {
        Ok(true) => {
            if let Some(ticket) = opt_ticket {
                // If a matching Task exists for this approval, cancel it
                if let Some(ref req_id) = ticket.request_id {
                    let tasks = state.task_registry.list_tasks().await;
                    if let Some(matching_task) = tasks
                        .iter()
                        .find(|t| t.request_id.as_deref() == Some(req_id.as_str()))
                    {
                        let _ = state
                            .task_registry
                            .cancel_task(&matching_task.task_id, payload.reason)
                            .await;
                    }
                }
            }

            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "message": format!("Ticket '{}' rejected successfully", id),
                })),
            )
                .into_response()
        }
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
