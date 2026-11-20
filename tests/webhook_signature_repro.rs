// Rust guideline compliant 2026-11-20

//! Retired callbacks return the migration URL even when daemon authentication is enabled.

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use warmplane::{
    approvals::{ApprovalStatus, CreateApprovalRequest},
    daemon::{server::build_router, AppState},
};

#[tokio::test]
async fn legacy_callback_is_gone_and_cannot_resolve_its_target() {
    let state = AppState::builder().auth_token("fixture-admin").build();
    let (id, _waiter) = state
        .approval_registry
        .create_approval(CreateApprovalRequest {
            capability_id: "fixture.call".to_owned(),
            server_id: "fixture".to_owned(),
            args: json!({}),
            sanitized_args: json!({}),
            request_id: None,
            context: None,
            timeout_secs: 120,
            webhook: None,
        })
        .await;
    let request = Request::post("/v1/webhooks/callbacks")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({"ticket_id": id, "action": "approve"}).to_string(),
        ))
        .unwrap();
    let response = build_router(state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::GONE);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["callback_url"], "/v1/webhooks/actions");
    assert_eq!(
        state.approval_registry.get(&id).await.unwrap().status,
        ApprovalStatus::Pending
    );
}
