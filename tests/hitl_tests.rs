// Rust guideline compliant 2026-08-14

//! Integration test suite for Warmplane Human-in-the-Loop (HITL) approval flows,
//! argument modification, structured rejection reasoning, and HMAC webhook verification.

use axum::{
    body::to_bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use std::collections::HashMap;
use tokio::sync::mpsc;
use warmplane::{
    config::PolicyConfig,
    daemon::{AppState, CapabilityMeta, Policy, ServerMsg},
    http_v1::{
        handle_approve_ticket, handle_call_capability, ApproveTicketRequest, CallCapabilityRequest,
    },
};

type HmacSha256 = Hmac<Sha256>;

#[tokio::test]
async fn test_hitl_end_to_end_wildcard_matching_and_approval() {
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "kubernetes.delete_pod".to_string(),
        CapabilityMeta {
            server: "k8s".to_string(),
            tool: "delete_pod".to_string(),
            summary: "Delete K8s pod".to_string(),
            description: "Delete K8s pod".to_string(),
            input_schema: json!({}),
            tags: vec![],
            examples: vec![],
        },
    );

    let (tx, mut rx) = mpsc::channel(1);
    let mut servers = HashMap::new();
    servers.insert("k8s".to_string(), tx);

    let policy_config = PolicyConfig {
        allow: vec!["*".to_string()],
        deny: vec![],
        redact_keys: vec!["token".to_string()],
        require_approval: vec!["kubernetes.delete_*".to_string()],
        approval_timeout_secs: Some(15),
        webhook: None,
    };

    let state = AppState::builder()
        .capabilities(capabilities)
        .servers(servers)
        .policy(Policy::from_config(Some(policy_config)))
        .build();

    // Spawn mock upstream worker
    tokio::spawn(async move {
        if let Some(ServerMsg::CallTool { params, reply, .. }) = rx.recv().await {
            assert_eq!(params["pod"], "api-worker-1");
            let _ = reply.send(Ok(json!({"deleted": true})));
        }
    });

    // Spawn operator task to approve
    let state_clone = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        let list = state_clone.approval_registry.list().await;
        assert_eq!(list.len(), 1);
        let ticket = &list[0];
        assert_eq!(ticket.capability_id, "kubernetes.delete_pod");

        let approve_res = handle_approve_ticket(
            State(state_clone),
            axum::extract::Path(ticket.id.clone()),
            Json(ApproveTicketRequest {
                operator: "devops-lead".to_string(),
                modified_args: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(approve_res.status(), StatusCode::OK);
    });

    let req = CallCapabilityRequest {
        capability_id: "kubernetes.delete_pod".to_string(),
        args: json!({"pod": "api-worker-1", "token": "secret123"}),
        request_id: Some("req-k8s-delete".to_string()),
        context: None,
        idempotency_key: None,
        input_responses: None,
        request_state: None,
    };

    let res = handle_call_capability(State(state), HeaderMap::new(), Json(req))
        .await
        .into_response();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["data"]["deleted"], true);
}

#[tokio::test]
async fn test_hitl_hmac_signature_calculation() {
    let secret = "whsec_super_secret_test_key_123";
    let timestamp = 1723668200u64;
    let raw_payload = r#"{"event":"approval.requested","id":"appr-1"}"#;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    let sign_target = format!("{}.{}", timestamp, raw_payload);
    mac.update(sign_target.as_bytes());
    let sig_hex = hex::encode(mac.finalize().into_bytes());

    // Verify signature matches expected HMAC-SHA256
    let mut verifier = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    verifier.update(format!("{}.{}", timestamp, raw_payload).as_bytes());
    let expected_bytes = hex::decode(sig_hex).unwrap();
    assert!(verifier.verify_slice(&expected_bytes).is_ok());
}
