// Rust guideline compliant 2026-11-20

//! Tenant-scoped approval views and operator decisions.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::{
    approvals::{Decision, DecisionError, DecisionRequest, PendingApproval},
    daemon::AppState,
    http_v1::types::{ApproveTicketRequest, RejectTicketRequest},
    rbac::TenantContext,
};

/// Handles GET `/v1/approvals` for the authenticated tenant.
pub async fn handle_list_approvals(
    State(state): State<AppState>,
    tenant: Option<Extension<Option<TenantContext>>>,
) -> Response {
    let actor = match state.approval_actor(tenant_ref(&tenant)) {
        Ok(actor) => actor,
        Err(error) => return decision_error(error),
    };
    let mut approvals = Vec::new();
    for ticket in state.approval_registry.list().await {
        if owns_approval(&ticket, &actor.tenant) {
            approvals.push(approval_summary(&state, ticket).await);
        }
    }
    Json(json!({"ok": true, "total": approvals.len(), "approvals": approvals})).into_response()
}

/// Handles GET `/v1/approvals/:id` for the authenticated tenant.
pub async fn handle_get_approval(
    State(state): State<AppState>,
    Path(id): Path<String>,
    tenant: Option<Extension<Option<TenantContext>>>,
) -> Response {
    let actor = match state.approval_actor(tenant_ref(&tenant)) {
        Ok(actor) => actor,
        Err(error) => return decision_error(error),
    };
    let ticket = state.approval_registry.get(&id).await;
    match ticket.filter(|ticket| owns_approval(ticket, &actor.tenant)) {
        Some(ticket) => {
            Json(json!({"ok": true, "approval": approval_summary(&state, ticket).await}))
                .into_response()
        }
        None => decision_error(DecisionError::NotFound),
    }
}

/// Handles POST `/v1/approvals/:id/approve` for a verified operator.
pub async fn handle_approve_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    tenant: Option<Extension<Option<TenantContext>>>,
    Json(payload): Json<ApproveTicketRequest>,
) -> Response {
    resolve(&state, &id, tenant_ref(&tenant), payload.into()).await
}

/// Handles POST `/v1/approvals/:id/reject` for a verified operator.
pub async fn handle_reject_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    tenant: Option<Extension<Option<TenantContext>>>,
    Json(payload): Json<RejectTicketRequest>,
) -> Response {
    let mut rejection = DecisionRequest::unchanged(Decision::Reject);
    rejection.reason = payload.reason;
    resolve(&state, &id, tenant_ref(&tenant), rejection).await
}

impl From<ApproveTicketRequest> for DecisionRequest {
    fn from(payload: ApproveTicketRequest) -> Self {
        Self {
            choice: Decision::Approve,
            modified_args: payload.modified_args,
            reason: None,
        }
    }
}

fn tenant_ref(tenant: &Option<Extension<Option<TenantContext>>>) -> Option<&TenantContext> {
    tenant.as_ref().and_then(|tenant| tenant.0.as_ref())
}

fn owns_approval(ticket: &PendingApproval, tenant: &str) -> bool {
    ticket
        .ownership
        .as_ref()
        .is_some_and(|owner| owner.tenant_id == tenant)
}

async fn approval_summary(state: &AppState, ticket: PendingApproval) -> Value {
    use crate::approvals::ApprovalStatus;
    let status = match ticket.status {
        ApprovalStatus::Pending => "pending",
        ApprovalStatus::Approved { .. } => "approved",
        ApprovalStatus::Rejected { .. } => "rejected",
        ApprovalStatus::Expired { .. } => "expired",
    };
    let task_id = ticket
        .ownership
        .as_ref()
        .and_then(|owner| owner.task_id.as_deref());
    let task = match task_id {
        Some(id) => state.task_registry.get_task(id).await,
        None => None,
    };
    let execution = task.as_ref().map(|task| &task.status);
    let upstream_state = task.as_ref().and_then(|task| {
        if task.status == crate::tasks::TaskStatus::Completed {
            Some("completed")
        } else {
            task.error
                .as_ref()
                .and_then(|error| error.get("upstreamExecutionState"))
                .and_then(Value::as_str)
        }
    });
    json!({
        "id": ticket.id,
        "capability_id": ticket.capability_id,
        "server_id": ticket.server_id,
        "sanitized_args": ticket.sanitized_args,
        "created_at": ticket.created_at,
        "expires_at": ticket.expires_at,
        "status": status,
        "task_id": task_id,
        "decision_receipt": ticket.decision_receipt,
        "receipt_delivery": ticket.receipt_delivery,
        "execution_status": execution,
        "upstream_execution_state": upstream_state,
    })
}

async fn resolve(
    state: &AppState,
    id: &str,
    tenant: Option<&TenantContext>,
    decision: DecisionRequest,
) -> Response {
    let actor = match state.approval_actor(tenant) {
        Ok(actor) => actor,
        Err(error) => return decision_error(error),
    };
    match state.decide_approval(id, &actor, decision, false).await {
        Ok(receipt) => Json(json!({"ok": true, "receipt": receipt})).into_response(),
        Err(error) => decision_error(error),
    }
}

pub(crate) fn decision_error(error: DecisionError) -> Response {
    let status = match error {
        DecisionError::NotFound => StatusCode::NOT_FOUND,
        DecisionError::Forbidden => StatusCode::FORBIDDEN,
        DecisionError::Conflict => StatusCode::CONFLICT,
        DecisionError::InvalidInput | DecisionError::TaskNotInputRequired => {
            StatusCode::BAD_REQUEST
        }
        DecisionError::Persistence => StatusCode::SERVICE_UNAVAILABLE,
    };
    (
        status,
        Json(json!({"ok": false, "error": crate::approvals::decision_error(error)})),
    )
        .into_response()
}
