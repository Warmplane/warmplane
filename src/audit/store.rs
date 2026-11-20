// Rust guideline compliant 2026-11-18

//! Append-only WORM audit log storage engine and query interface.
//!
//! Provides in-memory or persisted append-only storage with linear cryptographic hash chaining,
//! fast indexed filtering, and continuous tamper verification.

use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Read, Take, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::audit::chain::{compute_event_hash_with_key, verify_record_hash_with_key, GENESIS_HASH};
use crate::audit::models::{
    AuditEvent, AuditEventStatus, AuditEventType, RawAuditEvent, VerificationReport,
};

/// Query filter options for searching audit events.
#[derive(Debug, Clone, Default)]
pub struct AuditQueryFilter {
    /// Filter events with timestamp >= start_time_ns.
    pub start_time_ns: Option<u64>,
    /// Filter events with timestamp <= end_time_ns.
    pub end_time_ns: Option<u64>,
    /// Filter by specific actor ID.
    pub actor_id: Option<String>,
    /// Filter by target upstream server ID.
    pub server_id: Option<String>,
    /// Filter by target capability ID.
    pub capability_id: Option<String>,
    /// Filter by event type.
    pub event_type: Option<AuditEventType>,
    /// Filter by event status.
    pub status: Option<AuditEventStatus>,
    /// Filter by trace ID.
    pub trace_id: Option<String>,
    /// Filter by request ID.
    pub request_id: Option<String>,
    /// Filter by idempotency key.
    pub idempotency_key: Option<String>,
    /// Filter by whether the event was a deduplicated replay.
    pub is_replay: Option<bool>,
    /// Case-insensitive multi-field search substring query.
    pub search: Option<String>,
    /// Maximum number of records to return.
    pub limit: usize,
    /// Offset index for pagination.
    pub offset: usize,
}

/// Maximum number of audit events cached in RAM.
pub const MAX_IN_MEMORY_AUDIT_EVENTS: usize = 20_000;

struct AuditLog {
    events: Vec<AuditEvent>,
    total_records: usize,
    last_sequence: u64,
    append_blocked: bool,
}

impl AuditLog {
    fn tail_hash(&self) -> &str {
        self.events.last().map_or(GENESIS_HASH, |event| &event.hash)
    }
}

/// Append-only audit store protecting log records from retroactive tampering.
pub struct AuditStore {
    log: RwLock<AuditLog>,
    file_path: Option<PathBuf>,
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
            log: RwLock::new(AuditLog {
                events: Vec::new(),
                total_records: 0,
                last_sequence: 0,
                append_blocked: false,
            }),
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
        let path = path.as_ref().to_path_buf();
        let mut events = Vec::new();
        let mut last_sequence = 0;
        for record in read_records(open_log(&path)?) {
            let record = record?;
            let prev_hash = events
                .last()
                .map_or(GENESIS_HASH, |event: &AuditEvent| event.hash.as_str());
            validate_record(&record, prev_hash, hmac_key.as_deref())?;
            if let Some(sequence) = record
                .id
                .strip_prefix("aud_")
                .and_then(|id| id.parse().ok())
            {
                last_sequence = last_sequence.max(sequence);
            }
            events.push(record);
        }
        let total_records = events.len();

        Ok(Self {
            log: RwLock::new(AuditLog {
                events,
                total_records,
                last_sequence: last_sequence.max(u64::try_from(total_records)?),
                append_blocked: false,
            }),
            file_path: Some(path),
            hmac_key,
        })
    }

    /// Appends a single event and returns its committed, signed record.
    ///
    /// # Errors
    /// Returns an error if the event cannot be committed to the audit log.
    pub async fn append(&self, raw: RawAuditEvent) -> Result<AuditEvent> {
        self.append_batch(vec![raw])
            .await?
            .pop()
            .context("audit append produced no record")
    }

    /// Appends a batch and publishes its signed records after the file is synced.
    ///
    /// # Errors
    /// Returns an error if the batch cannot be committed to the audit log.
    pub async fn append_batch(&self, batch: Vec<RawAuditEvent>) -> Result<Vec<AuditEvent>> {
        if batch.is_empty() {
            return Ok(Vec::new());
        }

        let mut log = self.log.write().await;
        anyhow::ensure!(
            !log.append_blocked,
            "audit log requires recovery after a failed append rollback"
        );

        let mut records = Vec::with_capacity(batch.len());
        let mut previous = log.tail_hash().to_owned();
        let mut sequence = log.last_sequence;
        for raw in batch {
            sequence = sequence
                .checked_add(1)
                .context("audit sequence exhausted")?;
            let id = format!("aud_{sequence:012}");
            let now_ns = unix_time_ns();
            let hash =
                compute_event_hash_with_key(&previous, &id, now_ns, &raw, self.hmac_key.as_deref());
            records.push(chained_record(id, now_ns, raw, previous, hash.clone()));
            previous = hash;
        }

        if let Some(path) = &self.file_path {
            persist_audit_batch(path, &records, &mut log.append_blocked)?;
        }

        log.total_records += records.len();
        log.last_sequence = sequence;
        let excess = (log.events.len() + records.len()).saturating_sub(MAX_IN_MEMORY_AUDIT_EVENTS);
        let evict_cached = excess.min(log.events.len());
        log.events.drain(..evict_cached);
        log.events
            .extend(records[excess - evict_cached..].iter().cloned());
        Ok(records)
    }

    /// Captures an external anchor checkpoint summary of the current tail state of the audit log.
    pub async fn get_checkpoint(&self) -> crate::audit::models::AuditCheckpoint {
        let log = self.log.read().await;
        crate::audit::models::AuditCheckpoint {
            tail_hash: log.tail_hash().to_owned(),
            last_event_id: log.events.last().map(|event| event.id.clone()),
            total_records: log.total_records,
            timestamp_ns: unix_time_ns(),
        }
    }

    /// Verifies the persisted chain or the retained suffix of an in-memory log.
    pub async fn verify_chain(&self) -> VerificationReport {
        let log = self.log.read().await;
        match &self.file_path {
            Some(path) => {
                // Capture the committed tail and a stable file handle before letting appends resume.
                let snapshot = AuditSnapshot::capture(path, &log);
                let total_records = log.total_records;
                drop(log);
                verify_audit_snapshot(snapshot, total_records, self.hmac_key.clone()).await
            }
            None => verify_retained_events(&log, self.hmac_key.as_deref()),
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
        let log = self.log.read().await;
        let search_pattern = filter.search.as_ref().map(|s| s.to_lowercase());

        let filtered: Vec<&AuditEvent> = log
            .events
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
                if let Some(ref srv) = filter.server_id {
                    if e.server_id.as_deref() != Some(srv.as_str()) {
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
                if let Some(ref st) = filter.status {
                    if &e.status != st {
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
                if let Some(ref idk) = filter.idempotency_key {
                    if e.idempotency_key.as_deref() != Some(idk.as_str()) {
                        return false;
                    }
                }
                if let Some(replay) = filter.is_replay {
                    if e.is_replay != Some(replay) {
                        return false;
                    }
                }
                search_pattern
                    .as_deref()
                    .is_none_or(|needle| matches_search(e, needle))
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
        let log = self.log.read().await;
        let events = &log.events;
        events.iter().find(|e| e.id == id).cloned()
    }

    /// Returns the total count of audit events in the store.
    pub async fn count(&self) -> usize {
        self.log.read().await.events.len()
    }
}

pub type SharedAuditStore = Arc<AuditStore>;

fn matches_search(event: &AuditEvent, needle: &str) -> bool {
    [
        Some(event.id.as_str()),
        Some(event.trace_id.as_str()),
        event.request_id.as_deref(),
        event.idempotency_key.as_deref(),
        event.actor_id.as_deref(),
        event.server_id.as_deref(),
        event.capability_id.as_deref(),
        event.resource_uri.as_deref(),
        event.error_code.as_deref(),
        event.error_message.as_deref(),
        event.operator_id.as_deref(),
        event.approval_ticket_id.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|field| field.to_lowercase().contains(needle))
}

fn unix_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn persist_audit_batch(
    path: &Path,
    records: &[AuditEvent],
    append_blocked: &mut bool,
) -> Result<()> {
    let mut bytes = Vec::new();
    for record in records {
        serde_json::to_writer(&mut bytes, record)?;
        bytes.push(b'\n');
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let original_len = file.metadata()?.len();
    if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_data()) {
        // Keep the next append from chaining across a partial batch.
        if let Err(rollback) = file.set_len(original_len).and_then(|()| file.sync_data()) {
            *append_blocked = true;
            return Err(error).context(format!("failed to restore audit log: {rollback}"));
        }
        return Err(error).context("failed to append audit records");
    }
    Ok(())
}

async fn verify_audit_snapshot(
    snapshot: Result<AuditSnapshot>,
    total_records: usize,
    key: Option<Vec<u8>>,
) -> VerificationReport {
    let snapshot = match snapshot {
        Ok(snapshot) => snapshot,
        Err(error) => return failed_verification(total_records, None, None, None, error),
    };
    match tokio::task::spawn_blocking(move || snapshot.verify(key.as_deref())).await {
        Ok(report) => report,
        Err(error) => failed_verification(total_records, None, None, None, error),
    }
}

fn verify_retained_events(log: &AuditLog, key: Option<&[u8]>) -> VerificationReport {
    let retained_start = log.total_records - log.events.len();
    let first_retained_index = (retained_start > 0).then_some(retained_start);
    let mut previous = GENESIS_HASH;
    if let (Some(_), Some(first)) = (first_retained_index, log.events.first()) {
        previous = &first.prev_hash;
    }
    for (offset, record) in log.events.iter().enumerate() {
        if let Err(error) = validate_record(record, previous, key) {
            return failed_verification(
                log.total_records,
                first_retained_index,
                Some(retained_start + offset),
                Some(record.id.clone()),
                error,
            );
        }
        previous = &record.hash;
    }
    valid_verification(log.total_records, first_retained_index)
}

struct AuditSnapshot {
    file: Option<Take<File>>,
    total_records: usize,
    tail_hash: String,
}

impl AuditSnapshot {
    fn capture(path: &Path, log: &AuditLog) -> Result<Self> {
        let file = match open_log(path)? {
            Some(file) => {
                let len = file
                    .metadata()
                    .context("failed to measure audit log")?
                    .len();
                Some(file.take(len))
            }
            None => None,
        };
        Ok(Self {
            file,
            total_records: log.total_records,
            tail_hash: log.tail_hash().to_owned(),
        })
    }

    fn verify(self, key: Option<&[u8]>) -> VerificationReport {
        let mut previous = GENESIS_HASH.to_owned();
        let mut count = 0;
        for result in read_records(self.file) {
            let record = match result {
                Ok(record) => record,
                Err(error) => {
                    return failed_verification(self.total_records, None, Some(count), None, error)
                }
            };
            if let Err(error) = validate_record(&record, &previous, key) {
                return failed_verification(
                    self.total_records,
                    None,
                    Some(count),
                    Some(record.id),
                    error,
                );
            }
            previous = record.hash;
            count += 1;
        }
        if count != self.total_records || previous != self.tail_hash {
            return failed_verification(
                self.total_records,
                None,
                Some(count),
                None,
                "persisted audit chain differs from the committed tail",
            );
        }
        valid_verification(self.total_records, None)
    }
}

fn valid_verification(
    total_records: usize,
    first_retained_index: Option<usize>,
) -> VerificationReport {
    VerificationReport {
        is_valid: true,
        total_records,
        first_retained_index,
        corrupted_at_index: None,
        corrupted_record_id: None,
        message: None,
    }
}

fn failed_verification(
    total_records: usize,
    first_retained_index: Option<usize>,
    corrupted_at_index: Option<usize>,
    corrupted_record_id: Option<String>,
    error: impl std::fmt::Display,
) -> VerificationReport {
    VerificationReport {
        is_valid: false,
        total_records,
        first_retained_index,
        corrupted_at_index,
        corrupted_record_id,
        message: Some(error.to_string()),
    }
}

fn open_log(path: &Path) -> Result<Option<File>> {
    match File::open(path) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("failed to open audit log"),
    }
}

fn read_records(input: Option<impl Read>) -> impl Iterator<Item = Result<AuditEvent>> {
    let mut reader = input.map(BufReader::new);
    let mut line = String::new();
    std::iter::from_fn(move || loop {
        line.clear();
        let file = reader.as_mut()?;
        match file.read_line(&mut line) {
            Ok(0) => return None,
            Ok(_) if !line.ends_with('\n') => {
                return Some(Err(anyhow::anyhow!("unterminated audit log record")));
            }
            Ok(_) if line.trim().is_empty() => continue,
            Ok(_) => {
                return Some(serde_json::from_str(&line).context("invalid audit log record"));
            }
            Err(error) => {
                return Some(Err(error).context("failed to read audit log record"));
            }
        }
    })
}

fn validate_record(record: &AuditEvent, previous: &str, hmac_key: Option<&[u8]>) -> Result<()> {
    anyhow::ensure!(
        record.prev_hash == previous,
        "broken audit chain at event '{}': expected previous hash '{}', found '{}'",
        record.id,
        previous,
        record.prev_hash
    );
    anyhow::ensure!(
        verify_record_hash_with_key(record, hmac_key),
        "hash signature mismatch at audit event '{}'",
        record.id
    );
    Ok(())
}

fn chained_record(
    id: String,
    now_ns: u64,
    raw: RawAuditEvent,
    prev_hash: String,
    hash: String,
) -> AuditEvent {
    AuditEvent {
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
        idempotency_key: raw.idempotency_key,
        is_replay: raw.is_replay,
        prev_hash,
        hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::models::AuditEventStatus;

    fn tool_event() -> RawAuditEvent {
        RawAuditEvent {
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
            idempotency_key: None,
            is_replay: None,
        }
    }

    #[tokio::test]
    async fn failed_batch_leaves_the_committed_tail_intact() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audit.jsonl");
        let store = AuditStore::open_or_create(&path).unwrap();
        std::fs::create_dir(&path).unwrap();

        assert!(store
            .append_batch(vec![tool_event(), tool_event()])
            .await
            .is_err());
        assert_eq!(store.get_checkpoint().await.tail_hash, GENESIS_HASH);
        assert_eq!(store.count().await, 0);

        std::fs::remove_dir(&path).unwrap();
        let event = store.append(tool_event()).await.unwrap();
        assert_eq!(event.id, "aud_000000000001");
        assert!(AuditStore::open_or_create(&path).is_ok());
    }

    #[tokio::test]
    async fn reopening_preserves_legacy_sequence_gaps_and_rejects_unterminated_records() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audit.jsonl");
        let store = AuditStore::open_or_create(&path).unwrap();
        store.log.write().await.last_sequence = 2;
        let prior = store.append(tool_event()).await.unwrap();
        assert_eq!(prior.id, "aud_000000000003");
        drop(store);

        let reopened = AuditStore::open_or_create(&path).unwrap();
        let next = reopened.append(tool_event()).await.unwrap();
        assert_eq!(next.id, "aud_000000000004");

        let mut content = std::fs::read(&path).unwrap();
        assert_eq!(content.pop(), Some(b'\n'));
        std::fs::write(&path, content).unwrap();
        assert!(!reopened.verify_chain().await.is_valid);
        assert!(AuditStore::open_or_create(&path).is_err());
    }

    #[tokio::test]
    async fn audit_snapshot_excludes_concurrent_appends() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audit.jsonl");
        let store = AuditStore::open_or_create(&path).unwrap();
        store.append(tool_event()).await.unwrap();
        let snapshot = {
            let log = store.log.read().await;
            AuditSnapshot::capture(&path, &log).unwrap()
        };

        store.append(tool_event()).await.unwrap();

        let captured = snapshot.verify(None);
        assert!(captured.is_valid);
        assert_eq!(captured.total_records, 1);
        let latest = store.verify_chain().await;
        assert!(latest.is_valid);
        assert_eq!(latest.total_records, 2);
    }

    #[tokio::test]
    async fn verification_covers_disk_history_beyond_the_cache() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audit.jsonl");
        let file_store = AuditStore::open_or_create(&path).unwrap();
        let memory_store = AuditStore::in_memory();
        let batch = vec![tool_event(); MAX_IN_MEMORY_AUDIT_EVENTS + 1];

        file_store.append_batch(batch.clone()).await.unwrap();
        memory_store.append_batch(batch).await.unwrap();
        assert_eq!(file_store.count().await, MAX_IN_MEMORY_AUDIT_EVENTS);
        let disk_report = file_store.verify_chain().await;
        assert!(disk_report.is_valid);
        assert_eq!(disk_report.first_retained_index, None);
        assert_eq!(disk_report.total_records, MAX_IN_MEMORY_AUDIT_EVENTS + 1);

        let memory_report = memory_store.verify_chain().await;
        assert!(memory_report.is_valid);
        assert_eq!(memory_report.first_retained_index, Some(1));
        assert_eq!(memory_report.total_records, disk_report.total_records);

        let content = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, content.replacen("trace-1", "trace-changed", 1)).unwrap();
        let report = file_store.verify_chain().await;
        assert!(!report.is_valid);
        assert_eq!(report.corrupted_at_index, Some(0));
    }

    #[tokio::test]
    async fn test_store_append_and_verify() {
        let store = AuditStore::in_memory();

        let raw1 = tool_event();

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
            idempotency_key: None,
            is_replay: None,
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

    #[tokio::test]
    async fn test_store_filter_status_server_search_and_pagination() {
        let store = AuditStore::in_memory();

        let raw1 = RawAuditEvent {
            event_type: AuditEventType::ToolExecution,
            trace_id: "trace-101".to_string(),
            request_id: Some("req-101".to_string()),
            actor_id: Some("agent-alpha".to_string()),
            work_item_id: None,
            client_ip: None,
            server_id: Some("github-mcp".to_string()),
            capability_id: Some("github-mcp.create_issue".to_string()),
            resource_uri: None,
            sanitized_args: Some(serde_json::json!({"title": "Bug in query"})),
            sanitized_response: None,
            execution_latency_us: Some(250),
            status: AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: None,
        };

        let raw2 = RawAuditEvent {
            event_type: AuditEventType::PolicyViolation,
            trace_id: "trace-102".to_string(),
            request_id: Some("req-102".to_string()),
            actor_id: Some("agent-beta".to_string()),
            work_item_id: None,
            client_ip: None,
            server_id: Some("postgres-mcp".to_string()),
            capability_id: Some("postgres-mcp.drop_table".to_string()),
            resource_uri: None,
            sanitized_args: None,
            sanitized_response: None,
            execution_latency_us: Some(10),
            status: AuditEventStatus::Denied,
            error_code: Some("POLICY_DENY".to_string()),
            error_message: Some("Dangerous operation forbidden by security policy".to_string()),
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: None,
        };

        let raw3 = RawAuditEvent {
            event_type: AuditEventType::ToolInterceptedHitl,
            trace_id: "trace-103".to_string(),
            request_id: Some("req-103".to_string()),
            actor_id: Some("agent-gamma".to_string()),
            work_item_id: None,
            client_ip: None,
            server_id: Some("postgres-mcp".to_string()),
            capability_id: Some("postgres-mcp.delete_records".to_string()),
            resource_uri: None,
            sanitized_args: None,
            sanitized_response: None,
            execution_latency_us: Some(15),
            status: AuditEventStatus::Intercepted,
            error_code: None,
            error_message: None,
            operator_id: Some("admin-bob".to_string()),
            approval_ticket_id: Some("ticket-999".to_string()),
            idempotency_key: None,
            is_replay: None,
        };

        let ev1 = store.append(raw1).await.unwrap();
        let ev2 = store.append(raw2).await.unwrap();
        let ev3 = store.append(raw3).await.unwrap();

        // Filter by status = Denied
        let (denied, total_denied) = store
            .query(&AuditQueryFilter {
                status: Some(AuditEventStatus::Denied),
                ..Default::default()
            })
            .await;
        assert_eq!(total_denied, 1);
        assert_eq!(denied[0].id, ev2.id);

        // Filter by server_id = postgres-mcp
        let (pg_events, total_pg) = store
            .query(&AuditQueryFilter {
                server_id: Some("postgres-mcp".to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(total_pg, 2);
        assert_eq!(pg_events[0].id, ev3.id);
        assert_eq!(pg_events[1].id, ev2.id);

        // Search by keyword "ticket-999"
        let (searched, total_searched) = store
            .query(&AuditQueryFilter {
                search: Some("ticket-999".to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(total_searched, 1);
        assert_eq!(searched[0].id, ev3.id);

        // Search case-insensitively for "FORBIDDEN"
        let (searched_err, total_err) = store
            .query(&AuditQueryFilter {
                search: Some("forbidden".to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(total_err, 1);
        assert_eq!(searched_err[0].id, ev2.id);

        // Pagination: limit 1, offset 1
        let (paged, total_all) = store
            .query(&AuditQueryFilter {
                limit: 1,
                offset: 1,
                ..Default::default()
            })
            .await;
        assert_eq!(total_all, 3);
        assert_eq!(paged.len(), 1);
        assert_eq!(paged[0].id, ev2.id);

        // Pagination: limit 1, offset 2
        let (paged2, _) = store
            .query(&AuditQueryFilter {
                limit: 1,
                offset: 2,
                ..Default::default()
            })
            .await;
        assert_eq!(paged2.len(), 1);
        assert_eq!(paged2[0].id, ev1.id);
    }
}
