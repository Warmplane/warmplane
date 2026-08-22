// Rust guideline compliant 2026-08-15

//! Unit tests for policy rules and pattern matching in daemon module.

use crate::daemon::{
    policy::{wildcard_match, Policy},
    AppState,
};

#[test]
fn wildcard_prefix_match_works() {
    assert!(wildcard_match("db.*", "db.query"));
    assert!(!wildcard_match("db.*", "fs.read"));
}

#[test]
fn wildcard_suffix_and_infix_match_works() {
    assert!(wildcard_match("*.delete", "db.delete"));
    assert!(wildcard_match("*.delete", "fs.delete"));
    assert!(!wildcard_match("*.delete", "db.select"));

    assert!(wildcard_match("db.*.write", "db.users.write"));
    assert!(!wildcard_match("db.*.write", "db.users.read"));

    assert!(wildcard_match("*auth*", "service.auth.login"));
    assert!(!wildcard_match("*auth*", "service.billing.pay"));
}

#[test]
fn policy_deny_overrides_allow() {
    let policy = Policy {
        allow: vec!["db.*".to_string()],
        deny: vec!["db.delete".to_string()],
        ..Default::default()
    };

    assert!(policy.allows("db.query"));
    assert!(!policy.allows("db.delete"));
}

#[tokio::test]
async fn test_initialize_state_degraded_boot() {
    use crate::config::{McpConfig, ServerConfig};
    use std::collections::HashMap;

    let mut mcp_servers = HashMap::new();
    // Server with an invalid / non-existent binary to simulate offline/broken startup
    mcp_servers.insert(
        "broken_server".to_string(),
        ServerConfig {
            command: Some("non_existent_binary_xyz_1234".to_string()),
            args: vec![],
            env: HashMap::new(),
            url: None,
            auth: None,
            protocol_version: None,
            allow_stateless: Some(false),
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let config = McpConfig {
        mcp_servers,
        ..Default::default()
    };

    let state = crate::daemon::server::initialize_state(config, "test_config.json")
        .await
        .expect("initialize_state must succeed in degraded mode even if an upstream server fails");

    let statuses = state.server_statuses.read().await;
    let broken_status = statuses
        .get("broken_server")
        .expect("broken_server status must be recorded");
    assert_eq!(broken_status["status"], "degraded");
    assert!(broken_status["error"].is_string());
}

#[tokio::test]
async fn test_security_guard_host_and_origin() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    let state = AppState::builder().build();
    let app = crate::daemon::server::build_router(state);

    // 1. Valid host
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "127.0.0.1:9090")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Invalid host (DNS rebinding attempt)
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "evil.attacker.com")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 3. Untrusted browser origin (CSRF attempt)
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "127.0.0.1:9090")
        .header("origin", "https://malicious-site.com")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 4. Trusted browser origin
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "127.0.0.1:9090")
        .header("origin", "http://127.0.0.1:9090")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_security_guard_token_auth() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    let state = AppState::builder().auth_token("secret-token-12345").build();
    let app = crate::daemon::server::build_router(state);

    // 1. Unauthenticated request to /v1/ should fail with 401
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "127.0.0.1:9090")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Authenticated with Authorization Bearer header
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "127.0.0.1:9090")
        .header("authorization", "Bearer secret-token-12345")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Authenticated with X-Warmplane-Key header
    let req = Request::builder()
        .uri("/v1/capabilities")
        .header("host", "127.0.0.1:9090")
        .header("x-warmplane-key", "secret-token-12345")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_app_state_graceful_shutdown() {
    use crate::audit::{AuditEventStatus, AuditEventType, AuditStore, RawAuditEvent};
    use std::sync::Arc;

    let audit_store = Arc::new(AuditStore::in_memory());
    let audit_handle =
        crate::audit::spawn_audit_worker(audit_store.clone(), None, 100, 10_000, 100);

    let state = AppState::builder()
        .audit_store(audit_store.clone())
        .audit_handle(audit_handle.clone())
        .build();

    // Enqueue an audit event into the daemon state
    audit_handle.send(RawAuditEvent {
        event_type: AuditEventType::ToolExecution,
        trace_id: "trace-shutdown-test".to_string(),
        request_id: None,
        actor_id: None,
        work_item_id: None,
        client_ip: None,
        server_id: Some("srv".to_string()),
        capability_id: Some("srv.tool".to_string()),
        resource_uri: None,
        sanitized_args: None,
        sanitized_response: None,
        execution_latency_us: None,
        status: AuditEventStatus::Success,
        error_code: None,
        error_message: None,
        operator_id: None,
        approval_ticket_id: None,
        idempotency_key: None,
        is_replay: None,
    });

    // Execute graceful shutdown
    state.shutdown().await;

    // Verify audit logs are 100% committed
    assert_eq!(audit_store.count().await, 1);
    let report = audit_store.verify_chain().await;
    assert!(report.is_valid);
}
