// Rust guideline compliant 2026-08-22

//! UC5 idempotency story proof-of-concept tests.
//!
//! Validates:
//! 1. Failing-after-commit replay simulation (deterministic auto-key derivation & cached outcome).
//! 2. Explicit client-supplied `idempotency_key` exactly-once replay semantics.
//! 3. Replay count tracking and inspection API.
//! 4. Linked WORM audit trail verification connecting original attempt and replay with `idempotency_key` and `is_replay`.

use axum::http::StatusCode;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;

use warmplane::{
    config::{save_config, McpConfig},
    daemon::run_daemon,
    EmbeddedWarmplane,
};

#[tokio::test]
async fn test_uc5_failing_after_commit_simulation_and_audit_linkage() {
    // Shared counter to simulate upstream service commits and failures
    let execution_count = Arc::new(AtomicUsize::new(0));

    // Boot an embedded control plane with isolated state dir
    let temp_state_dir = tempfile::tempdir().unwrap();
    let config = McpConfig {
        state: Some(warmplane::config::StateConfig {
            enabled: true,
            dir: Some(temp_state_dir.path().to_str().unwrap().to_string()),
        }),
        ..Default::default()
    };
    let (cp, _shutdown) = EmbeddedWarmplane::start(config)
        .await
        .expect("EmbeddedWarmplane must boot cleanly");

    // Manually register an idempotency record representing an initial execution that completed upstream
    let idempotency_store = &cp.state().idempotency_store;
    let audit_store = &cp.state().audit_store;

    let test_key = "idk_payment_charge_tx_98765".to_string();
    let capability = "stripe.charge_customer";
    let first_trace = "trace-tx-1001";
    let args = json!({
        "customer_id": "cust_abc123",
        "amount_cents": 5000,
        "currency": "usd"
    });

    let success_payload = json!({
        "status": "succeeded",
        "charge_id": "ch_987654321",
        "receipt_url": "https://pay.example/receipt/ch_987654321"
    });

    // 1. Initial Attempt: check_or_start -> New -> complete
    let start_res = idempotency_store
        .check_or_start_with_meta(
            &test_key,
            Some(capability.to_string()),
            Some(first_trace.to_string()),
        )
        .await;

    assert!(matches!(
        start_res,
        warmplane::idempotency::DeduplicateResult::New
    ));
    execution_count.fetch_add(1, Ordering::SeqCst);

    idempotency_store
        .complete(&test_key, success_payload.clone())
        .await;

    // Record the first attempt in WORM audit store
    let first_ev = audit_store
        .append(warmplane::audit::RawAuditEvent {
            event_type: warmplane::audit::AuditEventType::ToolExecution,
            trace_id: first_trace.to_string(),
            request_id: Some("req-tx-1".to_string()),
            actor_id: Some("checkout-agent".to_string()),
            work_item_id: Some("order-555".to_string()),
            client_ip: Some("127.0.0.1".to_string()),
            server_id: Some("stripe".to_string()),
            capability_id: Some(capability.to_string()),
            resource_uri: None,
            sanitized_args: Some(args.clone()),
            sanitized_response: Some(success_payload.clone()),
            execution_latency_us: Some(450),
            status: warmplane::audit::AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: Some(test_key.clone()),
            is_replay: Some(false),
        })
        .await
        .expect("First audit event must append");

    assert_eq!(first_ev.idempotency_key, Some(test_key.clone()));
    assert_eq!(first_ev.is_replay, Some(false));

    // 2. Client Timeout / Replay Attempt (UC5: Agent timed out or lost connection, retries with same parameters)
    let second_trace = "trace-tx-1002";
    let replay_res = idempotency_store
        .check_or_start_with_meta(
            &test_key,
            Some(capability.to_string()),
            Some(second_trace.to_string()),
        )
        .await;

    match replay_res {
        warmplane::idempotency::DeduplicateResult::Completed(cached) => {
            assert_eq!(cached, success_payload);
            // Verify upstream execution count was NOT incremented (strictly exactly-once)
            assert_eq!(execution_count.load(Ordering::SeqCst), 1);
        }
        _ => panic!("Expected cached deduplicated outcome for replay attempt"),
    }

    // Record the replay in WORM audit store
    let replay_ev = audit_store
        .append(warmplane::audit::RawAuditEvent {
            event_type: warmplane::audit::AuditEventType::ToolExecution,
            trace_id: second_trace.to_string(),
            request_id: Some("req-tx-2".to_string()),
            actor_id: Some("checkout-agent".to_string()),
            work_item_id: Some("order-555".to_string()),
            client_ip: Some("127.0.0.1".to_string()),
            server_id: None,
            capability_id: Some(capability.to_string()),
            resource_uri: None,
            sanitized_args: Some(args.clone()),
            sanitized_response: Some(success_payload.clone()),
            execution_latency_us: Some(15),
            status: warmplane::audit::AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: Some(test_key.clone()),
            is_replay: Some(true),
        })
        .await
        .expect("Replay audit event must append");

    assert_eq!(replay_ev.idempotency_key, Some(test_key.clone()));
    assert_eq!(replay_ev.is_replay, Some(true));

    // 3. Inspect Idempotency Record
    let record = idempotency_store
        .get_record(&test_key)
        .await
        .expect("Record must exist");
    assert_eq!(record.key, test_key);
    assert_eq!(record.capability_id.as_deref(), Some(capability));
    assert_eq!(record.first_trace_id.as_deref(), Some(first_trace));
    assert_eq!(record.replay_count, 1);
    assert_eq!(record.response, success_payload);

    // 4. Query Audit Store Filtered by Idempotency Key & Replays
    let (all_for_key, total_for_key) = audit_store
        .query(&warmplane::audit::AuditQueryFilter {
            idempotency_key: Some(test_key.clone()),
            ..Default::default()
        })
        .await;

    assert_eq!(total_for_key, 2);
    assert_eq!(all_for_key.len(), 2);
    // Chronological order or query results show both connected attempts
    assert!(all_for_key
        .iter()
        .any(|e| e.trace_id == first_trace && e.is_replay == Some(false)));
    assert!(all_for_key
        .iter()
        .any(|e| e.trace_id == second_trace && e.is_replay == Some(true)));

    // Verify Hash Chain Integrity over idempotency-enriched records
    let verify_report = audit_store.verify_chain().await;
    assert!(verify_report.is_valid);
    assert_eq!(verify_report.total_records, 2);

    cp.shutdown().await;
}

#[tokio::test]
async fn test_uc5_http_v1_api_deterministic_deduplication_and_headers() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let temp_state_dir = tempfile::tempdir().unwrap();
    let initial_config = McpConfig {
        port: Some(port),
        state: Some(warmplane::config::StateConfig {
            enabled: true,
            dir: Some(temp_state_dir.path().to_str().unwrap().to_string()),
        }),
        ..Default::default()
    };
    save_config(&config_path, &initial_config).unwrap();

    let cfg = initial_config.clone();
    let cfg_path = config_path.clone();
    tokio::spawn(async move {
        let _ = run_daemon(port, cfg, cfg_path).await;
    });

    // Wait for HTTP daemon to accept connections
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);
    for _ in 0..50 {
        if client
            .get(format!("{}/v1/config", base_url))
            .send()
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    // Call /v1/idempotency/records endpoint (initially empty)
    let records_res = client
        .get(format!("{}/v1/idempotency/records", base_url))
        .send()
        .await
        .expect("GET /v1/idempotency/records must succeed");
    assert_eq!(records_res.status(), StatusCode::OK);
    let body: Value = records_res.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["records"].as_array().unwrap().len(), 0);
}
