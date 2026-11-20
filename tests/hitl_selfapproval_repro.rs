// Rust guideline compliant 2026-09-30

//! A gated tool call must never be approved by the party that requested it.
//!
//! `POST /v1/tasks/:id/update` cross-resolves the approval registry. It must
//! apply only an explicit decision from the request body. A request carrying no
//! decision, or a non-boolean one, must leave the ticket pending so the wait
//! channel stays closed and the gated call is never executed.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use warmplane::approvals::{ApprovalOwnership, ApprovalStatus, CreateApprovalRequest};
use warmplane::daemon::{AppState, Policy};
use warmplane::http_v1::handle_update_task;
use warmplane::http_v1::tasks_api::UpdateTaskRequest;
use warmplane::tasks::{CreateTaskParams, TaskStatus};

/// Builds a daemon `AppState` with no upstream servers attached.
fn state() -> AppState {
    AppState::builder()
        .capabilities(HashMap::new())
        .servers(HashMap::new())
        .policy(Policy::default())
        .build()
}

/// Creates a pending approval ticket linked to `request_id`.
async fn pending_approval(
    state: &AppState,
    request_id: &str,
    task_id: &str,
) -> (
    String,
    tokio::sync::oneshot::Receiver<warmplane::approvals::ApprovalResolution>,
) {
    let (id, rx) = state
        .approval_registry
        .create_scoped_approval(
            CreateApprovalRequest {
                capability_id: "kubernetes.delete_pod".to_string(),
                server_id: "k8s".to_string(),
                args: json!({"pod": "prod-1"}),
                sanitized_args: json!({"pod": "prod-1"}),
                request_id: Some(request_id.to_string()),
                context: None,
                timeout_secs: 600,
                webhook: None,
            },
            Some(ApprovalOwnership::new(
                "default",
                Some(task_id),
                &json!({"pod": "prod-1"}),
                None,
            )),
        )
        .await;
    (id, rx)
}

/// Creates a task awaiting input, linked to `request_id`.
async fn input_required_task(state: &AppState, request_id: &str) -> String {
    let mut input_requests = BTreeMap::new();
    input_requests.insert(
        "hitl_approval".to_string(),
        json!({"type": "approval_review"}),
    );

    let (record, _rx) = state
        .task_registry
        .create_task(CreateTaskParams {
            capability_id: "kubernetes.delete_pod".to_string(),
            server_id: "k8s".to_string(),
            args: json!({"pod": "prod-1"}),
            request_id: Some(request_id.to_string()),
            context: None,
            idempotency_key: None,
            initial_status: TaskStatus::InputRequired,
            status_message: None,
            input_requests: Some(input_requests),
            ttl_ms: Some(60_000),
            poll_interval_ms: Some(500),
        })
        .await;
    record.task_id
}

/// Sends a task update and returns the resulting status code.
async fn update(state: &AppState, task_id: &str, responses: BTreeMap<String, Value>) -> StatusCode {
    handle_update_task(
        State(state.clone()),
        axum::extract::Path(task_id.to_string()),
        None,
        None,
        Json(UpdateTaskRequest {
            input_responses: responses,
        }),
    )
    .await
    .into_response()
    .status()
}

/// Builds an `input_responses` map from key/value pairs.
fn responses(pairs: Vec<(&str, Value)>) -> BTreeMap<String, Value> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// Reads the current status of an approval ticket.
async fn status_of(state: &AppState, appr_id: &str) -> ApprovalStatus {
    state
        .approval_registry
        .get(appr_id)
        .await
        .expect("ticket must still exist")
        .status
}

#[tokio::test]
async fn empty_input_responses_do_not_approve_a_pending_ticket() {
    let state = state();
    let task_id = input_required_task(&state, "req-empty").await;
    let (appr_id, _waiter) = pending_approval(&state, "req-empty", &task_id).await;

    let code = update(&state, &task_id, responses(vec![])).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);

    let status = status_of(&state, &appr_id).await;
    assert!(
        matches!(status, ApprovalStatus::Pending),
        "an update carrying no decision must not approve, got {:?}",
        status
    );
}

#[tokio::test]
async fn unlinked_approval_task_rejects_generic_input() {
    let state = state();
    let task_id = input_required_task(&state, "req-unlinked").await;
    let code = update(
        &state,
        &task_id,
        responses(vec![("reason", json!("continue"))]),
    )
    .await;
    assert_eq!(code, StatusCode::BAD_REQUEST);
    let task = state.task_registry.get_task(&task_id).await.unwrap();
    assert_eq!(task.status, TaskStatus::InputRequired);
    assert!(task.input_responses.is_empty());
}

#[tokio::test]
async fn non_boolean_decision_does_not_approve_a_pending_ticket() {
    let state = state();
    let task_id = input_required_task(&state, "req-strfalse").await;
    let (appr_id, _waiter) = pending_approval(&state, "req-strfalse", &task_id).await;

    // `"approved": "false"` is a string, not a boolean. Reading it through a
    // truthy default would turn an explicit denial into an approval.
    let code = update(
        &state,
        &task_id,
        responses(vec![("hitl_approval", json!({"approved": "false"}))]),
    )
    .await;
    assert_eq!(
        code,
        StatusCode::BAD_REQUEST,
        "a non-boolean decision is a malformed request, not a denial"
    );

    let status = status_of(&state, &appr_id).await;
    assert!(
        matches!(status, ApprovalStatus::Pending),
        "a non-boolean decision must not approve, got {:?}",
        status
    );
}

#[tokio::test]
async fn explicit_boolean_approval_still_works() {
    let state = state();
    let task_id = input_required_task(&state, "req-yes").await;
    let (appr_id, _waiter) = pending_approval(&state, "req-yes", &task_id).await;

    let code = update(
        &state,
        &task_id,
        responses(vec![(
            "hitl_approval",
            json!({"approved": true, "operator": "alice"}),
        )]),
    )
    .await;
    assert_eq!(code, StatusCode::OK);

    let status = status_of(&state, &appr_id).await;
    assert!(
        matches!(status, ApprovalStatus::Approved { .. }),
        "an explicit approval must still approve, got {:?}",
        status
    );
}

#[tokio::test]
async fn explicit_boolean_rejection_still_works() {
    let state = state();
    let task_id = input_required_task(&state, "req-no").await;
    let (appr_id, _waiter) = pending_approval(&state, "req-no", &task_id).await;

    let code = update(
        &state,
        &task_id,
        responses(vec![(
            "hitl_approval",
            json!({"approved": false, "operator": "alice", "reason": "nope"}),
        )]),
    )
    .await;
    assert_eq!(code, StatusCode::OK);

    let status = status_of(&state, &appr_id).await;
    assert!(
        matches!(status, ApprovalStatus::Rejected { .. }),
        "an explicit rejection must still reject, got {:?}",
        status
    );
}
