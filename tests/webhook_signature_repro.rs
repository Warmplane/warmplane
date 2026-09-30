// Rust guideline compliant 2026-09-30

//! A configured webhook secret must be enforced, not merely checked when the
//! caller happens to send a signature.
//!
//! `POST /v1/webhooks/callbacks` reaches `ApprovalRegistry::approve` and
//! `reject`. When `policy.webhook.secret` resolves, an unsigned or wrongly
//! signed callback must be rejected. A callback with no signature header is
//! the shape an attacker sends, so omitting the header must not disarm the
//! check.

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use warmplane::approvals::{ApprovalStatus, CreateApprovalRequest};
use warmplane::config::{McpConfig, PolicyConfig, WebhookConfig};
use warmplane::daemon::AppState;
use warmplane::http_v1::webhooks_api::handle_webhook_callback;

const SECRET: &str = "shared-webhook-signing-secret";

/// Produces the same HMAC-SHA256 value `verify_signature` expects, so a test
/// can build a correctly signed callback without a signer helper in the crate.
fn sign_body(secret: &str, body: &str, timestamp: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac init");
    mac.update(format!("{timestamp}.{body}").as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

/// Writes a config with a resolved webhook secret and returns an `AppState`.
fn state_with_secret() -> (tempfile::NamedTempFile, AppState) {
    let config = McpConfig {
        policy: Some(PolicyConfig {
            webhook: Some(WebhookConfig {
                url: "https://hooks.example.com/services/T/B/X".to_string(),
                secret: Some(SECRET.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let temp = tempfile::NamedTempFile::new().expect("temp config");
    std::fs::write(temp.path(), serde_json::to_string(&config).unwrap()).expect("write config");

    let state = AppState::builder()
        .config_path(temp.path().to_str().unwrap().to_string())
        .catalog_version("test")
        .build();

    (temp, state)
}

/// Creates a pending approval ticket so a callback has something to resolve.
async fn pending_ticket(state: &AppState, request_id: &str) -> String {
    let (_id, _rx) = state
        .approval_registry
        .create_approval(CreateApprovalRequest {
            capability_id: "k8s.delete_pod".to_string(),
            server_id: "k8s".to_string(),
            args: serde_json::json!({}),
            sanitized_args: serde_json::json!({}),
            request_id: Some(request_id.to_string()),
            context: None,
            timeout_secs: 600,
            webhook: None,
        })
        .await;

    state
        .approval_registry
        .get_pending_by_request_id(request_id)
        .await
        .expect("a pending ticket was created")
        .id
}

/// Sends an `approve` callback with the supplied headers.
async fn post_callback(
    state: AppState,
    ticket_id: &str,
    headers: HeaderMap,
) -> axum::response::Response {
    let body = format!(r#"{{"action":"approve","ticket_id":"{ticket_id}","operator":"mallory"}}"#);
    handle_webhook_callback(State(state), headers, Bytes::from(body))
        .await
        .into_response()
}

/// Reads back the ticket status so a silent mutation cannot pass unnoticed.
async fn status_of(state: &AppState, id: &str) -> Option<ApprovalStatus> {
    state.approval_registry.get(id).await.map(|t| t.status)
}

#[tokio::test]
async fn unsigned_callback_is_rejected_even_when_no_signature_header_is_sent() {
    let (_temp, state) = state_with_secret();
    let ticket_id = pending_ticket(&state, "req-unsigned").await;

    // No signature header at all: the attacker's shape.
    let resp = post_callback(state.clone(), &ticket_id, HeaderMap::new()).await;

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "a callback with no signature must not be able to approve a ticket"
    );

    let status = status_of(&state, &ticket_id).await;
    assert!(
        matches!(status, Some(ApprovalStatus::Pending)),
        "ticket must stay pending, got {status:?}"
    );
}

#[tokio::test]
async fn wrongly_signed_callback_is_rejected() {
    let (_temp, state) = state_with_secret();
    let ticket_id = pending_ticket(&state, "req-badsig").await;

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-warmplane-signature",
        "sha256=deadbeefnotarealsignature".parse().unwrap(),
    );

    let resp = post_callback(state.clone(), &ticket_id, headers).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let status = status_of(&state, &ticket_id).await;
    assert!(
        matches!(status, Some(ApprovalStatus::Pending)),
        "ticket must stay pending, got {status:?}"
    );
}

#[tokio::test]
async fn unsigned_reject_callback_is_rejected() {
    let (_temp, state) = state_with_secret();
    let ticket_id = pending_ticket(&state, "req-unsigned-reject").await;

    let body = format!(r#"{{"action":"reject","ticket_id":"{ticket_id}","operator":"mallory"}}"#);
    let resp = handle_webhook_callback(State(state.clone()), HeaderMap::new(), Bytes::from(body))
        .await
        .into_response();

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "the reject path must be gated by the same signature check"
    );

    let status = status_of(&state, &ticket_id).await;
    assert!(
        matches!(status, Some(ApprovalStatus::Pending)),
        "ticket must stay pending, got {status:?}"
    );
}

#[tokio::test]
async fn correctly_signed_callback_is_accepted() {
    let (_temp, state) = state_with_secret();
    let ticket_id = pending_ticket(&state, "req-goodsig").await;

    let body = format!(r#"{{"action":"approve","ticket_id":"{ticket_id}","operator":"dana"}}"#);
    let ts = "1700000000";
    let sig = sign_body(SECRET, &body, ts);

    let mut headers = HeaderMap::new();
    headers.insert("x-warmplane-signature", sig.parse().unwrap());
    headers.insert("x-warmplane-timestamp", ts.parse().unwrap());

    let resp = handle_webhook_callback(State(state.clone()), headers, Bytes::from(body))
        .await
        .into_response();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "a validly signed callback must still work"
    );

    let status = status_of(&state, &ticket_id).await;
    assert!(
        matches!(status, Some(ApprovalStatus::Approved { .. })),
        "ticket must be approved, got {status:?}"
    );
}
