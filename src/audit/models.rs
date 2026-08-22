// Rust guideline compliant 2026-08-15

//! Canonical data models for the WORM audit trail logging system.
//!
//! Adheres to `M-CANONICAL-DOCS` with comprehensive docstrings and serialization formats.

use serde::{Deserialize, Serialize};

/// Categorical event type for audit trail entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Capability / tool invocation requested and executed.
    ToolExecution,
    /// Capability invocation intercepted by Human-in-the-Loop policy.
    ToolInterceptedHitl,
    /// Pending approval ticket approved by human operator.
    ApprovalGranted,
    /// Pending approval ticket rejected by human operator.
    ApprovalRejected,
    /// Pending approval ticket expired due to timeout.
    ApprovalExpired,
    /// Request denied due to policy violation (e.g. deny rule or unlisted capability).
    PolicyViolation,
    /// Configuration or server lifecycle mutation.
    ConfigMutation,
    /// Sampling message creation request.
    SamplingCall,
    /// Resource read or prompt fetch event.
    ResourceAccess,
}

/// Execution or disposition status for an audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventStatus {
    /// Operation completed successfully.
    Success,
    /// Operation failed during upstream or local execution.
    Failed,
    /// Operation was blocked by policy or authorization.
    Denied,
    /// Operation was paused pending Human-in-the-Loop operator approval.
    Intercepted,
    /// Operation was cancelled by client or timeout.
    Cancelled,
}

/// Canonical audit log event record with cryptographic tamper-evidence chain hashes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Monotonic or unique audit event identifier (e.g. `aud_1723700000_1`).
    pub id: String,
    /// Unix timestamp in nanoseconds when the event occurred.
    pub timestamp_ns: u64,
    /// Categorical classification of the audit event.
    pub event_type: AuditEventType,
    /// Distributed tracing identifier.
    pub trace_id: String,
    /// Optional caller request identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Optional authenticated caller/actor identifier (e.g. agent name or user ID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    /// Optional workflow or ticket tracking identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    /// Optional client IP address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    /// Target upstream MCP server identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    /// Target capability identifier if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
    /// Target resource URI if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,
    /// Sanitized parameters (secrets and PII redacted according to policy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sanitized_args: Option<serde_json::Value>,
    /// Sanitized response payload or summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sanitized_response: Option<serde_json::Value>,
    /// Execution duration in microseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_latency_us: Option<u64>,
    /// Final outcome status of the event.
    pub status: AuditEventStatus,
    /// Machine-readable error code if operation failed or was denied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Human-readable error explanation if operation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Operator ID who reviewed/approved/rejected the request if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    /// Linked approval ticket ID if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_ticket_id: Option<String>,
    /// Unique idempotency key if operation was executed under idempotency deduplication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    /// Whether this event reflects a deduplicated replay from stored cache.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_replay: Option<bool>,
    /// Cryptographic SHA-256 hash of the preceding audit record in the sequence.
    pub prev_hash: String,
    /// Cryptographic SHA-256 hash of this audit record.
    pub hash: String,
}

/// Unsigned incoming audit event item prior to cryptographic chain computation and insertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAuditEvent {
    /// Categorical classification of the audit event.
    pub event_type: AuditEventType,
    /// Distributed tracing identifier.
    pub trace_id: String,
    /// Optional caller request identifier.
    pub request_id: Option<String>,
    /// Optional authenticated caller/actor identifier.
    pub actor_id: Option<String>,
    /// Optional workflow or ticket tracking identifier.
    pub work_item_id: Option<String>,
    /// Optional client IP address.
    pub client_ip: Option<String>,
    /// Target upstream MCP server identifier.
    pub server_id: Option<String>,
    /// Target capability identifier if applicable.
    pub capability_id: Option<String>,
    /// Target resource URI if applicable.
    pub resource_uri: Option<String>,
    /// Sanitized parameters (secrets redacted).
    pub sanitized_args: Option<serde_json::Value>,
    /// Sanitized response payload.
    pub sanitized_response: Option<serde_json::Value>,
    /// Execution duration in microseconds.
    pub execution_latency_us: Option<u64>,
    /// Final outcome status of the event.
    pub status: AuditEventStatus,
    /// Error code if failed or denied.
    pub error_code: Option<String>,
    /// Error message if failed or denied.
    pub error_message: Option<String>,
    /// Operator ID if reviewed by human.
    pub operator_id: Option<String>,
    /// Linked approval ticket ID if applicable.
    pub approval_ticket_id: Option<String>,
    /// Idempotency key if applicable.
    pub idempotency_key: Option<String>,
    /// Whether event is a cache deduplication replay.
    pub is_replay: Option<bool>,
}

/// Verification result report for cryptographic audit hash chain checks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationReport {
    /// Whether all records in the verified chain are valid and untampered.
    pub is_valid: bool,
    /// Total number of records inspected.
    pub total_records: usize,
    /// Index and ID of the first corrupted record if tampering was detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corrupted_at_index: Option<usize>,
    /// ID of the corrupted record if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corrupted_record_id: Option<String>,
    /// Detailed diagnostic error message if verification failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// External anchor checkpoint representing the tail integrity state of the audit chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditCheckpoint {
    /// SHA256 / HMAC tail hash of the most recent audit event in the chain.
    pub tail_hash: String,
    /// ID of the most recent audit event.
    pub last_event_id: Option<String>,
    /// Total number of records in the store.
    pub total_records: usize,
    /// Unix timestamp in nanoseconds when this checkpoint snapshot was taken.
    pub timestamp_ns: u64,
}
