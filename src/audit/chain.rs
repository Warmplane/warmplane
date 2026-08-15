// Rust guideline compliant 2026-08-15

//! Cryptographic hash chain algorithms ensuring Write-Once-Read-Many (WORM) tamper evidence.
//!
//! Every audit event is chained to its predecessor using SHA-256 over a canonical representation.

use sha2::{Digest, Sha256};

use crate::audit::models::{AuditEvent, RawAuditEvent};

/// Genesis hash constant used for the root audit record when the log is initialized.
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Computes the SHA-256 hash for an audit record given the previous record's hash.
///
/// # Arguments
/// * `prev_hash` - Hexadecimal SHA-256 hash string of the previous event.
/// * `id` - Event identifier.
/// * `timestamp_ns` - Nanosecond timestamp.
/// * `event` - Raw event fields.
///
/// # Returns
/// 64-character lowercase hexadecimal SHA-256 hash digest.
pub fn compute_event_hash(
    prev_hash: &str,
    id: &str,
    timestamp_ns: u64,
    event: &RawAuditEvent,
) -> String {
    let mut hasher = Sha256::new();

    hasher.update(prev_hash.as_bytes());
    hasher.update(b"|");
    hasher.update(id.as_bytes());
    hasher.update(b"|");
    hasher.update(timestamp_ns.to_string().as_bytes());
    hasher.update(b"|");
    hasher.update(format!("{:?}", event.event_type).as_bytes());
    hasher.update(b"|");
    hasher.update(event.trace_id.as_bytes());
    hasher.update(b"|");
    hasher.update(event.request_id.as_deref().unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(event.actor_id.as_deref().unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(event.server_id.as_deref().unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(event.capability_id.as_deref().unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(
        event
            .sanitized_args
            .as_ref()
            .map(|v| v.to_string())
            .as_deref()
            .unwrap_or("")
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(
        event
            .sanitized_response
            .as_ref()
            .map(|v| v.to_string())
            .as_deref()
            .unwrap_or("")
            .as_bytes(),
    );
    hasher.update(b"|");
    hasher.update(format!("{:?}", event.status).as_bytes());
    hasher.update(b"|");
    hasher.update(event.error_code.as_deref().unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(event.operator_id.as_deref().unwrap_or("").as_bytes());
    hasher.update(b"|");
    hasher.update(event.approval_ticket_id.as_deref().unwrap_or("").as_bytes());

    hex::encode(hasher.finalize())
}

/// Verifies whether an existing `AuditEvent` matches its computed hash given its `prev_hash`.
///
/// # Arguments
/// * `record` - The stored `AuditEvent` to test.
///
/// # Returns
/// `true` if the record hash exactly matches the recomputed hash, otherwise `false`.
pub fn verify_record_hash(record: &AuditEvent) -> bool {
    let raw = RawAuditEvent {
        event_type: record.event_type.clone(),
        trace_id: record.trace_id.clone(),
        request_id: record.request_id.clone(),
        actor_id: record.actor_id.clone(),
        work_item_id: record.work_item_id.clone(),
        client_ip: record.client_ip.clone(),
        server_id: record.server_id.clone(),
        capability_id: record.capability_id.clone(),
        resource_uri: record.resource_uri.clone(),
        sanitized_args: record.sanitized_args.clone(),
        sanitized_response: record.sanitized_response.clone(),
        execution_latency_us: record.execution_latency_us,
        status: record.status.clone(),
        error_code: record.error_code.clone(),
        error_message: record.error_message.clone(),
        operator_id: record.operator_id.clone(),
        approval_ticket_id: record.approval_ticket_id.clone(),
    };

    let expected = compute_event_hash(&record.prev_hash, &record.id, record.timestamp_ns, &raw);
    expected == record.hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::models::{AuditEventStatus, AuditEventType};

    #[test]
    fn test_hash_chain_computation_and_verification() {
        let raw = RawAuditEvent {
            event_type: AuditEventType::ToolExecution,
            trace_id: "trace-123".to_string(),
            request_id: Some("req-abc".to_string()),
            actor_id: Some("agent-smith".to_string()),
            work_item_id: None,
            client_ip: Some("127.0.0.1".to_string()),
            server_id: Some("github".to_string()),
            capability_id: Some("github.create_issue".to_string()),
            resource_uri: None,
            sanitized_args: Some(serde_json::json!({"title": "Fix bug"})),
            sanitized_response: Some(serde_json::json!({"issue_number": 42})),
            execution_latency_us: Some(15000),
            status: AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
        };

        let id = "aud_01".to_string();
        let timestamp_ns = 1723700000000000000;
        let hash1 = compute_event_hash(GENESIS_HASH, &id, timestamp_ns, &raw);
        assert!(!hash1.is_empty());

        let record = AuditEvent {
            id,
            timestamp_ns,
            event_type: raw.event_type.clone(),
            trace_id: raw.trace_id.clone(),
            request_id: raw.request_id.clone(),
            actor_id: raw.actor_id.clone(),
            work_item_id: raw.work_item_id.clone(),
            client_ip: raw.client_ip.clone(),
            server_id: raw.server_id.clone(),
            capability_id: raw.capability_id.clone(),
            resource_uri: raw.resource_uri.clone(),
            sanitized_args: raw.sanitized_args.clone(),
            sanitized_response: raw.sanitized_response.clone(),
            execution_latency_us: raw.execution_latency_us,
            status: raw.status.clone(),
            error_code: raw.error_code.clone(),
            error_message: raw.error_message.clone(),
            operator_id: raw.operator_id.clone(),
            approval_ticket_id: raw.approval_ticket_id.clone(),
            prev_hash: GENESIS_HASH.to_string(),
            hash: hash1.clone(),
        };

        assert!(verify_record_hash(&record));

        // Tamper with record args
        let mut tampered = record.clone();
        tampered.sanitized_args = Some(serde_json::json!({"title": "Malicious altered title"}));
        assert!(!verify_record_hash(&tampered));
    }
}
