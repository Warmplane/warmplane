// Rust guideline compliant 2026-08-15

//! Append-only WORM audit log storage engine and query interface.
//!
//! Provides in-memory or persisted append-only storage with linear cryptographic hash chaining,
//! fast indexed filtering, and continuous tamper verification.

use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::audit::chain::{compute_event_hash_with_key, verify_record_hash_with_key, GENESIS_HASH};
use crate::audit::models::{AuditEvent, AuditEventType, RawAuditEvent, VerificationReport};

/// Query filter options for searching audit events.
#[derive(Debug, Clone, Default)]
pub struct AuditQueryFilter {
    /// Filter events with timestamp >= start_time_ns.
    pub start_time_ns: Option<u64>,
    /// Filter events with timestamp <= end_time_ns.
    pub end_time_ns: Option<u64>,
    /// Filter by specific actor ID.
    pub actor_id: Option<String>,
    /// Filter by target capability ID.
    pub capability_id: Option<String>,
    /// Filter by event type.
    pub event_type: Option<AuditEventType>,
    /// Filter by trace ID.
    pub trace_id: Option<String>,
    /// Filter by request ID.
    pub request_id: Option<String>,
    /// Maximum number of records to return.
    pub limit: usize,
    /// Offset index for pagination.
    pub offset: usize,
}

/// Maximum number of audit events cached in RAM.
pub const MAX_IN_MEMORY_AUDIT_EVENTS: usize = 20_000;

/// Append-only audit store protecting log records from retroactive tampering.
pub struct AuditStore {
    /// In-memory sequential audit log.
    events: RwLock<Vec<AuditEvent>>,
    /// Most recent chain hash.
    latest_hash: RwLock<String>,
    /// Atomic sequence counter for event ID generation.
    counter: AtomicU64,
    /// Optional file path for appending serialized audit log entries (JSONL).
    file_path: Option<PathBuf>,
    /// Optional HMAC key for calculating keyed audit event digests.
    hmac_key: Option<Vec<u8>>,
}

impl AuditStore {
    /// Creates an empty in-memory audit store.
    pub fn in_memory() -> Self {
        Self::in_memory_with_key(None)
    }

    /// Creates an empty in-memory audit store with an optional HMAC key.
    pub fn in_memory_with_key(hmac_key: Option<Vec<u8>>) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            latest_hash: RwLock::new(GENESIS_HASH.to_string()),
            counter: AtomicU64::new(1),
            file_path: None,
            hmac_key,
        }
    }

    /// Initializes an audit store backed by an append-only JSONL file on disk.
    /// Loads existing records and validates the integrity of the existing chain upon startup.
    ///
    /// # Arguments
    /// * `path` - File path to the append-only audit log file.
    ///
    /// # Errors
    /// Returns an error if the file exists and its hash chain is corrupted or cannot be read.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_or_create_with_key(path, None)
    }

    /// Initializes an audit store backed by an append-only JSONL file on disk with an optional HMAC key.
    pub fn open_or_create_with_key(
        path: impl AsRef<Path>,
        hmac_key: Option<Vec<u8>>,
    ) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let mut events = Vec::new();
        let mut latest_hash = GENESIS_HASH.to_string();
        let mut max_seq = 0u64;

        if path_buf.exists() {
            let file = File::open(&path_buf)?;
            let reader = BufReader::new(file);
            for line_res in reader.lines() {
                let line = line_res?;
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let event: AuditEvent = serde_json::from_str(trimmed)?;

                // Verify record
                if event.prev_hash != latest_hash {
                    anyhow::bail!(
                        "Corrupted audit chain at event ID '{}': expected prev_hash '{}', found '{}'",
                        event.id,
                        latest_hash,
                        event.prev_hash
                    );
                }
                if !verify_record_hash_with_key(&event, hmac_key.as_deref()) {
                    anyhow::bail!("Tampered audit record detected at event ID '{}'", event.id);
                }

                latest_hash = event.hash.clone();
                events.push(event);
                max_seq += 1;
            }
        }

        Ok(Self {
            events: RwLock::new(events),
            latest_hash: RwLock::new(latest_hash),
            counter: AtomicU64::new(max_seq + 1),
            file_path: Some(path_buf),
            hmac_key,
        })
    }

    /// Appends a raw audit event to the append-only log atomically calculating its chain hash.
    ///
    /// # Arguments
    /// * `raw` - Unsigned raw event payload.
    ///
    /// # Returns
    /// The fully signed, chained `AuditEvent`.
    ///
    /// # Errors
    /// Returns an error if writing to disk file fails.
    pub async fn append(&self, raw: RawAuditEvent) -> Result<AuditEvent> {
        let seq = self.counter.fetch_add(1, Ordering::Relaxed);
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let id = format!("aud_{:012}", seq);

        let mut events_guard = self.events.write().await;
        let mut latest_hash_guard = self.latest_hash.write().await;

        let prev_hash = latest_hash_guard.clone();
        let hash =
            compute_event_hash_with_key(&prev_hash, &id, now_ns, &raw, self.hmac_key.as_deref());

        let record = AuditEvent {
            id,
            timestamp_ns: now_ns,
            event_type: raw.event_type,
            trace_id: raw.trace_id,
            request_id: raw.request_id,
            actor_id: raw.actor_id,
            work_item_id: raw.work_item_id,
            client_ip: raw.client_ip,
            server_id: raw.server_id,
            capability_id: raw.capability_id,
            resource_uri: raw.resource_uri,
            sanitized_args: raw.sanitized_args,
            sanitized_response: raw.sanitized_response,
            execution_latency_us: raw.execution_latency_us,
            status: raw.status,
            error_code: raw.error_code,
            error_message: raw.error_message,
            operator_id: raw.operator_id,
            approval_ticket_id: raw.approval_ticket_id,
            prev_hash,
            hash: hash.clone(),
        };

        if let Some(ref path) = self.file_path {
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            let serialized = serde_json::to_string(&record)?;
            writeln!(file, "{}", serialized)?;
            file.flush()?;
        }

        *latest_hash_guard = hash;
        if events_guard.len() >= MAX_IN_MEMORY_AUDIT_EVENTS {
            let excess = events_guard.len() - (MAX_IN_MEMORY_AUDIT_EVENTS - 1);
            events_guard.drain(0..excess);
        }
        events_guard.push(record.clone());

        Ok(record)
    }

    /// Appends a batch of raw audit events atomically within a single write lock.
    ///
    /// # Arguments
    /// * `batch` - Slice of raw event payloads.
    ///
    /// # Returns
    /// List of signed, chained `AuditEvent` records.
    pub async fn append_batch(&self, batch: Vec<RawAuditEvent>) -> Result<Vec<AuditEvent>> {
        if batch.is_empty() {
            return Ok(Vec::new());
        }

        let mut events_guard = self.events.write().await;
        let mut latest_hash_guard = self.latest_hash.write().await;

        let mut out = Vec::with_capacity(batch.len());
        let mut disk_lines = Vec::with_capacity(batch.len());

        for raw in batch {
            let seq = self.counter.fetch_add(1, Ordering::Relaxed);
            let now_ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;

            let id = format!("aud_{:012}", seq);
            let prev_hash = latest_hash_guard.clone();
            let hash = compute_event_hash_with_key(
                &prev_hash,
                &id,
                now_ns,
                &raw,
                self.hmac_key.as_deref(),
            );

            let record = AuditEvent {
                id,
                timestamp_ns: now_ns,
                event_type: raw.event_type,
                trace_id: raw.trace_id,
                request_id: raw.request_id,
                actor_id: raw.actor_id,
                work_item_id: raw.work_item_id,
                client_ip: raw.client_ip,
                server_id: raw.server_id,
                capability_id: raw.capability_id,
                resource_uri: raw.resource_uri,
                sanitized_args: raw.sanitized_args,
                sanitized_response: raw.sanitized_response,
                execution_latency_us: raw.execution_latency_us,
                status: raw.status,
                error_code: raw.error_code,
                error_message: raw.error_message,
                operator_id: raw.operator_id,
                approval_ticket_id: raw.approval_ticket_id,
                prev_hash,
                hash: hash.clone(),
            };

            *latest_hash_guard = hash;
            disk_lines.push(serde_json::to_string(&record)?);
            if events_guard.len() >= MAX_IN_MEMORY_AUDIT_EVENTS {
                let excess = events_guard.len() - (MAX_IN_MEMORY_AUDIT_EVENTS - 1);
                events_guard.drain(0..excess);
            }
            events_guard.push(record.clone());
            out.push(record);
        }

        if let Some(ref path) = self.file_path {
            let mut file = OpenOptions::new().create(true).append(true).open(path)?;
            for line in disk_lines {
                writeln!(file, "{}", line)?;
            }
            file.flush()?;
        }

        Ok(out)
    }

    /// Captures an external anchor checkpoint summary of the current tail state of the audit log.
    pub async fn get_checkpoint(&self) -> crate::audit::models::AuditCheckpoint {
        let events = self.events.read().await;
        let latest_hash = self.latest_hash.read().await.clone();
        let last_event_id = events.last().map(|e| e.id.clone());
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        crate::audit::models::AuditCheckpoint {
            tail_hash: latest_hash,
            last_event_id,
            total_records: events.len(),
            timestamp_ns: now_ns,
        }
    }

    /// Verifies the complete cryptographic hash chain across all stored events.
    ///
    /// # Returns
    /// `VerificationReport` indicating whether all records are untampered or where tampering occurred.
    pub async fn verify_chain(&self) -> VerificationReport {
        let events = self.events.read().await;
        let mut expected_prev_hash = GENESIS_HASH.to_string();

        for (idx, record) in events.iter().enumerate() {
            if record.prev_hash != expected_prev_hash {
                return VerificationReport {
                    is_valid: false,
                    total_records: events.len(),
                    corrupted_at_index: Some(idx),
                    corrupted_record_id: Some(record.id.clone()),
                    message: Some(format!(
                        "Broken hash link at record #{}: prev_hash '{}' != expected '{}'",
                        idx, record.prev_hash, expected_prev_hash
                    )),
                };
            }

            if !verify_record_hash_with_key(record, self.hmac_key.as_deref()) {
                return VerificationReport {
                    is_valid: false,
                    total_records: events.len(),
                    corrupted_at_index: Some(idx),
                    corrupted_record_id: Some(record.id.clone()),
                    message: Some(format!(
                        "Hash signature mismatch at record #{} (ID: '{}')",
                        idx, record.id
                    )),
                };
            }

            expected_prev_hash = record.hash.clone();
        }

        VerificationReport {
            is_valid: true,
            total_records: events.len(),
            corrupted_at_index: None,
            corrupted_record_id: None,
            message: None,
        }
    }

    /// Queries audit events matching the provided filter criteria.
    ///
    /// # Arguments
    /// * `filter` - Search and pagination options.
    ///
    /// # Returns
    /// (Matching records subset, total matching count).
    pub async fn query(&self, filter: &AuditQueryFilter) -> (Vec<AuditEvent>, usize) {
        let events = self.events.read().await;

        let filtered: Vec<&AuditEvent> = events
            .iter()
            .rev() // Newest first
            .filter(|e| {
                if let Some(st) = filter.start_time_ns {
                    if e.timestamp_ns < st {
                        return false;
                    }
                }
                if let Some(et) = filter.end_time_ns {
                    if e.timestamp_ns > et {
                        return false;
                    }
                }
                if let Some(ref act) = filter.actor_id {
                    if e.actor_id.as_deref() != Some(act.as_str()) {
                        return false;
                    }
                }
                if let Some(ref cap) = filter.capability_id {
                    if e.capability_id.as_deref() != Some(cap.as_str()) {
                        return false;
                    }
                }
                if let Some(ref et) = filter.event_type {
                    if &e.event_type != et {
                        return false;
                    }
                }
                if let Some(ref tid) = filter.trace_id {
                    if &e.trace_id != tid {
                        return false;
                    }
                }
                if let Some(ref rid) = filter.request_id {
                    if e.request_id.as_deref() != Some(rid.as_str()) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let total_matched = filtered.len();
        let limit = if filter.limit == 0 { 50 } else { filter.limit };
        let paginated = filtered
            .into_iter()
            .skip(filter.offset)
            .take(limit)
            .cloned()
            .collect();

        (paginated, total_matched)
    }

    /// Retrieves an audit event by its unique ID.
    pub async fn get_by_id(&self, id: &str) -> Option<AuditEvent> {
        let events = self.events.read().await;
        events.iter().find(|e| e.id == id).cloned()
    }

    /// Returns the total count of audit events in the store.
    pub async fn count(&self) -> usize {
        self.events.read().await.len()
    }
}

pub type SharedAuditStore = Arc<AuditStore>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::models::AuditEventStatus;

    #[tokio::test]
    async fn test_store_append_and_verify() {
        let store = AuditStore::in_memory();

        let raw1 = RawAuditEvent {
            event_type: AuditEventType::ToolExecution,
            trace_id: "trace-1".to_string(),
            request_id: Some("req-1".to_string()),
            actor_id: Some("agent-1".to_string()),
            work_item_id: None,
            client_ip: None,
            server_id: Some("srv".to_string()),
            capability_id: Some("srv.tool1".to_string()),
            resource_uri: None,
            sanitized_args: Some(serde_json::json!({"x": 1})),
            sanitized_response: Some(serde_json::json!({"res": "ok"})),
            execution_latency_us: Some(100),
            status: AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
        };

        let raw2 = RawAuditEvent {
            event_type: AuditEventType::ApprovalGranted,
            trace_id: "trace-2".to_string(),
            request_id: None,
            actor_id: Some("agent-2".to_string()),
            work_item_id: None,
            client_ip: None,
            server_id: None,
            capability_id: Some("srv.tool2".to_string()),
            resource_uri: None,
            sanitized_args: None,
            sanitized_response: None,
            execution_latency_us: None,
            status: AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: Some("operator-alice".to_string()),
            approval_ticket_id: Some("appr-123".to_string()),
        };

        let ev1 = store.append(raw1).await.unwrap();
        assert_eq!(ev1.prev_hash, GENESIS_HASH);

        let ev2 = store.append(raw2).await.unwrap();
        assert_eq!(ev2.prev_hash, ev1.hash);

        let report = store.verify_chain().await;
        assert!(report.is_valid);
        assert_eq!(report.total_records, 2);

        let (queried, total) = store
            .query(&AuditQueryFilter {
                actor_id: Some("agent-1".to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(total, 1);
        assert_eq!(queried[0].id, ev1.id);
    }
}
