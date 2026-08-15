// Rust guideline compliant 2026-08-15

use super::*;
use std::sync::Arc;
use std::time::Duration;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_audit_worker_batching_and_chain() {
    let store = Arc::new(AuditStore::in_memory());
    let handle = spawn_audit_worker(store.clone(), None, 100, 50, 5);

    for i in 0..12 {
        handle
            .send_async(RawAuditEvent {
                event_type: AuditEventType::ToolExecution,
                trace_id: format!("trace-{}", i),
                request_id: Some(format!("req-{}", i)),
                actor_id: Some("agent-bench".to_string()),
                work_item_id: None,
                client_ip: None,
                server_id: Some("server-1".to_string()),
                capability_id: Some("server-1.exec".to_string()),
                resource_uri: None,
                sanitized_args: Some(serde_json::json!({"step": i})),
                sanitized_response: Some(serde_json::json!({"status": "ok"})),
                execution_latency_us: Some(500),
                status: AuditEventStatus::Success,
                error_code: None,
                error_message: None,
                operator_id: None,
                approval_ticket_id: None,
            })
            .await;
    }

    // Wait for periodic batch flush to complete
    tokio::time::sleep(Duration::from_millis(150)).await;

    assert_eq!(store.count().await, 12);

    let report = store.verify_chain().await;
    assert!(report.is_valid);
    assert_eq!(report.total_records, 12);
    assert!(report.corrupted_at_index.is_none());
}

#[tokio::test]
async fn test_audit_tampering_detection() {
    let file = NamedTempFile::new().unwrap();
    let file_path = file.path().to_path_buf();

    {
        let store = AuditStore::open_or_create(&file_path).unwrap();
        store
            .append(RawAuditEvent {
                event_type: AuditEventType::ToolExecution,
                trace_id: "trace-1".to_string(),
                request_id: None,
                actor_id: None,
                work_item_id: None,
                client_ip: None,
                server_id: None,
                capability_id: Some("tool.one".to_string()),
                resource_uri: None,
                sanitized_args: None,
                sanitized_response: None,
                execution_latency_us: None,
                status: AuditEventStatus::Success,
                error_code: None,
                error_message: None,
                operator_id: None,
                approval_ticket_id: None,
            })
            .await
            .unwrap();

        store
            .append(RawAuditEvent {
                event_type: AuditEventType::ToolExecution,
                trace_id: "trace-2".to_string(),
                request_id: None,
                actor_id: None,
                work_item_id: None,
                client_ip: None,
                server_id: None,
                capability_id: Some("tool.two".to_string()),
                resource_uri: None,
                sanitized_args: None,
                sanitized_response: None,
                execution_latency_us: None,
                status: AuditEventStatus::Success,
                error_code: None,
                error_message: None,
                operator_id: None,
                approval_ticket_id: None,
            })
            .await
            .unwrap();

        let report = store.verify_chain().await;
        assert!(report.is_valid);
    }

    // Reopen clean
    let clean_store = AuditStore::open_or_create(&file_path).unwrap();
    assert_eq!(clean_store.count().await, 2);

    // Tamper with file content directly on disk
    let content = std::fs::read_to_string(&file_path).unwrap();
    let tampered_content = content.replace("tool.one", "tool.malicious");
    std::fs::write(&file_path, tampered_content).unwrap();

    // Reopening must fail due to corrupted hash signature
    let corrupted_open = AuditStore::open_or_create(&file_path);
    assert!(corrupted_open.is_err());
}

#[tokio::test]
async fn test_siem_dispatcher_batch_empty() {
    let dispatcher = SiemDispatcher::new(None);
    // Dispatches empty or populated list safely without panic
    dispatcher.dispatch_batch(&[]).await;
}
