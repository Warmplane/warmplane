// Rust guideline compliant 2026-08-22

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use tracing::error;

use crate::storage::AtomicFile;

/// Canonical deterministic idempotency-key derivation algorithm.
///
/// Combines the capability identifier, canonically normalized JSON arguments,
/// optional caller actor identifier, and optional request/turn identifier using SHA-256.
///
/// # Arguments
/// * `capability_id` - Target capability identifier.
/// * `args` - JSON arguments payload.
/// * `actor_id` - Optional authenticated caller identifier.
/// * `turn_or_step` - Optional turn or batch step identifier.
///
/// # Returns
/// A 68-character prefixed SHA-256 hexadecimal string (`"idk_<64-hex>"`).
pub fn derive_idempotency_key(
    capability_id: impl AsRef<str>,
    args: &Value,
    actor_id: Option<&str>,
    turn_or_step: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(capability_id.as_ref().as_bytes());
    hasher.update(b":");

    // Canonical JSON string representation for stable argument hashing
    let canonical_args = canonicalize_json(args);
    hasher.update(canonical_args.as_bytes());
    hasher.update(b":");

    if let Some(actor) = actor_id {
        hasher.update(actor.as_bytes());
    }
    hasher.update(b":");

    if let Some(step) = turn_or_step {
        hasher.update(step.as_bytes());
    }

    let digest = hex::encode(hasher.finalize());
    format!("idk_{}", digest)
}

/// Recursively sorts object keys to produce deterministic, canonical JSON strings.
fn canonicalize_json(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s)),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonicalize_json).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            let entries: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    let k_str = serde_json::to_string(k).unwrap_or_else(|_| format!("\"{}\"", k));
                    let v_str = canonicalize_json(&map[k]);
                    format!("{}:{}", k_str, v_str)
                })
                .collect();
            format!("{{{}}}", entries.join(","))
        }
    }
}

/// Metadata attached to responses indicating retry safety and upstream execution status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryMetadata {
    /// Classification of operation safety (`"safe"`, `"unsafe"`, or `"idempotent"`).
    pub classification: String,
    /// Upstream execution state (`"not_started"`, `"completed"`, or `"unknown"`).
    pub upstream_execution_state: String,
}

impl RetryMetadata {
    /// Helper constructor for safe read-only operations.
    pub fn safe(state: impl AsRef<str>) -> Self {
        Self {
            classification: "safe".to_string(),
            upstream_execution_state: state.as_ref().to_string(),
        }
    }

    /// Helper constructor for non-idempotent unsafe operations.
    pub fn unsafe_op(state: impl AsRef<str>) -> Self {
        Self {
            classification: "unsafe".to_string(),
            upstream_execution_state: state.as_ref().to_string(),
        }
    }

    /// Helper constructor for explicitly idempotent write operations.
    pub fn idempotent(state: impl AsRef<str>) -> Self {
        Self {
            classification: "idempotent".to_string(),
            upstream_execution_state: state.as_ref().to_string(),
        }
    }
}

enum EntryState {
    InProgress(broadcast::Sender<Value>),
    Completed(Value),
}

struct IdempotencyEntry {
    state: EntryState,
    capability_id: Option<String>,
    created_at: Instant,
    created_at_epoch_secs: u64,
    completed_at_epoch_secs: Option<u64>,
    replay_count: u64,
    first_trace_id: Option<String>,
}

/// Serialized record for persistent idempotency cache storage and external inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersistedIdempotencyRecord {
    /// Unique idempotency key.
    pub key: String,
    /// Target capability identifier if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
    /// Completed JSON payload.
    pub response: Value,
    /// Unix timestamp when the operation first started.
    pub created_at_epoch_secs: u64,
    /// Unix timestamp when the operation completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at_epoch_secs: Option<u64>,
    /// Number of times this cached response was replayed.
    #[serde(default)]
    pub replay_count: u64,
    /// Distributed trace identifier of the original initiating call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_trace_id: Option<String>,
}

/// Maximum duration an operation can remain in the InProgress state before being treated as abandoned.
pub const IN_PROGRESS_TIMEOUT: Duration = Duration::from_secs(60);

/// In-memory or disk-persisted idempotency deduplication store with configurable TTL.
pub struct IdempotencyStore {
    ttl: Duration,
    entries: RwLock<HashMap<String, IdempotencyEntry>>,
    storage: Option<AtomicFile<HashMap<String, PersistedIdempotencyRecord>>>,
}

/// Result of checking or starting an idempotent request execution.
pub enum DeduplicateResult {
    /// Request is new; caller must execute the underlying operation.
    New,
    /// Request is already in progress; receiver yields completed payload.
    InProgress(broadcast::Receiver<Value>),
    /// Request was already completed; contains cached payload and whether it was previously recorded.
    Completed(Value),
}

impl IdempotencyStore {
    /// Creates a new in-memory `IdempotencyStore` with specified entry TTL duration.
    ///
    /// # Arguments
    /// * `ttl` - Entry expiry duration.
    ///
    /// # Returns
    /// An empty in-memory `IdempotencyStore`.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: RwLock::new(HashMap::new()),
            storage: None,
        }
    }

    /// Initializes an `IdempotencyStore` backed by a persistent atomic JSON file.
    ///
    /// Loads existing completed records, pruning those older than the TTL.
    ///
    /// # Arguments
    /// * `path` - Destination path for persistent idempotency cache.
    /// * `ttl` - Cache expiration duration.
    ///
    /// # Errors
    /// Returns an error if reading or parsing existing storage fails.
    pub fn open_or_create(path: impl AsRef<Path>, ttl: Duration) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let loaded: HashMap<String, PersistedIdempotencyRecord> =
            storage.load_opt()?.unwrap_or_default();

        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let now_instant = Instant::now();

        let mut in_memory_map = HashMap::new();
        let mut valid_persisted = HashMap::new();
        let mut pruned_any = false;

        for (k, record) in loaded {
            let age_secs = now_epoch_secs.saturating_sub(record.created_at_epoch_secs);
            if age_secs < ttl.as_secs() {
                let age_dur = Duration::from_secs(age_secs);
                let created_instant = now_instant.checked_sub(age_dur).unwrap_or(now_instant);

                in_memory_map.insert(
                    k.clone(),
                    IdempotencyEntry {
                        state: EntryState::Completed(record.response.clone()),
                        capability_id: record.capability_id.clone(),
                        created_at: created_instant,
                        created_at_epoch_secs: record.created_at_epoch_secs,
                        completed_at_epoch_secs: record.completed_at_epoch_secs,
                        replay_count: record.replay_count,
                        first_trace_id: record.first_trace_id.clone(),
                    },
                );
                valid_persisted.insert(k, record);
            } else {
                pruned_any = true;
            }
        }

        if pruned_any {
            let _ = storage.save(&valid_persisted);
        }

        Ok(Self {
            ttl,
            entries: RwLock::new(in_memory_map),
            storage: Some(storage),
        })
    }

    async fn sync_to_disk(&self) {
        if let Some(ref store) = self.storage {
            let guard = self.entries.read().await;
            let mut persisted = HashMap::new();
            for (k, entry) in guard.iter() {
                if let EntryState::Completed(ref val) = entry.state {
                    persisted.insert(
                        k.clone(),
                        PersistedIdempotencyRecord {
                            key: k.clone(),
                            capability_id: entry.capability_id.clone(),
                            response: val.clone(),
                            created_at_epoch_secs: entry.created_at_epoch_secs,
                            completed_at_epoch_secs: entry.completed_at_epoch_secs,
                            replay_count: entry.replay_count,
                            first_trace_id: entry.first_trace_id.clone(),
                        },
                    );
                }
            }
            drop(guard);
            if let Err(e) = store.save(&persisted) {
                error!(error = %e, path = %store.path().display(), "failed to persist idempotency cache to disk");
            }
        }
    }

    /// Checks existing idempotency state or marks a new key as in-progress.
    ///
    /// # Arguments
    /// * `key` - Unique idempotency key string.
    ///
    /// # Returns
    /// `DeduplicateResult` enum variant indicating execution state.
    pub async fn check_or_start(&self, key: impl AsRef<str>) -> DeduplicateResult {
        self.check_or_start_with_meta(key, None, None).await
    }

    /// Checks existing idempotency state with associated trace ID and capability ID.
    ///
    /// Increments replay count on cache hits.
    pub async fn check_or_start_with_meta(
        &self,
        key: impl AsRef<str>,
        capability_id: Option<String>,
        trace_id: Option<String>,
    ) -> DeduplicateResult {
        let key_ref = key.as_ref();
        let mut map = self.entries.write().await;
        let now = Instant::now();
        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Check if existing key is valid before returning
        if let Some(entry) = map.get_mut(key_ref) {
            let age = now.duration_since(entry.created_at);
            match &entry.state {
                EntryState::Completed(val) => {
                    if age < self.ttl {
                        entry.replay_count = entry.replay_count.saturating_add(1);
                        let ret_val = val.clone();
                        drop(map);
                        self.sync_to_disk().await;
                        return DeduplicateResult::Completed(ret_val);
                    }
                }
                EntryState::InProgress(tx) => {
                    if age < IN_PROGRESS_TIMEOUT {
                        return DeduplicateResult::InProgress(tx.subscribe());
                    }
                }
            }
            // Expired completed entry or stale/abandoned in-progress entry, remove it
            map.remove(key_ref);
        }

        // Opportunistic batch eviction only if map size exceeds threshold (amortized O(1))
        if map.len() > 1024 {
            let ttl = self.ttl;
            map.retain(|_, entry| now.duration_since(entry.created_at) < ttl);
            if map.len() >= 4096 {
                let keys: Vec<String> = map.keys().take(1024).cloned().collect();
                for k in keys {
                    map.remove(&k);
                }
            }
        }

        let (tx, _) = broadcast::channel(1);
        map.insert(
            key_ref.to_string(),
            IdempotencyEntry {
                state: EntryState::InProgress(tx),
                capability_id,
                created_at: now,
                created_at_epoch_secs: now_epoch_secs,
                completed_at_epoch_secs: None,
                replay_count: 0,
                first_trace_id: trace_id,
            },
        );
        DeduplicateResult::New
    }

    /// Completes an in-progress idempotency entry and broadcasts the final JSON payload.
    ///
    /// # Arguments
    /// * `key` - Unique idempotency key string.
    /// * `result` - Completed JSON payload.
    pub async fn complete(&self, key: impl AsRef<str>, result: Value) {
        let key_ref = key.as_ref();
        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut map = self.entries.write().await;
        if let Some(entry) = map.get_mut(key_ref) {
            if let EntryState::InProgress(tx) = &entry.state {
                let _ = tx.send(result.clone());
            }
            entry.state = EntryState::Completed(result);
            entry.completed_at_epoch_secs = Some(now_epoch_secs);
        }
        drop(map);
        self.sync_to_disk().await;
    }

    /// Fetches a persisted/cached record by key for inspection.
    pub async fn get_record(&self, key: impl AsRef<str>) -> Option<PersistedIdempotencyRecord> {
        let guard = self.entries.read().await;
        guard.get(key.as_ref()).and_then(|entry| {
            if let EntryState::Completed(ref res) = entry.state {
                Some(PersistedIdempotencyRecord {
                    key: key.as_ref().to_string(),
                    capability_id: entry.capability_id.clone(),
                    response: res.clone(),
                    created_at_epoch_secs: entry.created_at_epoch_secs,
                    completed_at_epoch_secs: entry.completed_at_epoch_secs,
                    replay_count: entry.replay_count,
                    first_trace_id: entry.first_trace_id.clone(),
                })
            } else {
                None
            }
        })
    }

    /// Lists all completed records up to `limit` with optional `offset`.
    pub async fn list_records(
        &self,
        limit: usize,
        offset: usize,
    ) -> Vec<PersistedIdempotencyRecord> {
        let guard = self.entries.read().await;
        let mut records: Vec<PersistedIdempotencyRecord> = guard
            .iter()
            .filter_map(|(k, entry)| {
                if let EntryState::Completed(ref res) = entry.state {
                    Some(PersistedIdempotencyRecord {
                        key: k.clone(),
                        capability_id: entry.capability_id.clone(),
                        response: res.clone(),
                        created_at_epoch_secs: entry.created_at_epoch_secs,
                        completed_at_epoch_secs: entry.completed_at_epoch_secs,
                        replay_count: entry.replay_count,
                        first_trace_id: entry.first_trace_id.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort reverse chronological by created timestamp
        records.sort_by(|a, b| b.created_at_epoch_secs.cmp(&a.created_at_epoch_secs));
        records.into_iter().skip(offset).take(limit).collect()
    }

    /// Clears expired entries according to TTL and returns number of cleared records.
    pub async fn clear_expired(&self) -> usize {
        let mut map = self.entries.write().await;
        let now = Instant::now();
        let ttl = self.ttl;
        let initial_len = map.len();
        map.retain(|_, entry| now.duration_since(entry.created_at) < ttl);
        let removed = initial_len.saturating_sub(map.len());
        drop(map);
        if removed > 0 {
            self.sync_to_disk().await;
        }
        removed
    }

    /// Removes an idempotency record by key.
    pub async fn remove(&self, key: &str) {
        let mut map = self.entries.write().await;
        map.remove(key);
        drop(map);
        self.sync_to_disk().await;
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_derive_idempotency_key_deterministic_and_canonical() {
        let args1 = serde_json::json!({
            "recipient": "alice@example.com",
            "amount": 100,
            "currency": "USD"
        });
        let args2 = serde_json::json!({
            "currency": "USD",
            "amount": 100,
            "recipient": "alice@example.com"
        });

        let key1 =
            derive_idempotency_key("payments.charge", &args1, Some("agent-1"), Some("turn-4"));
        let key2 =
            derive_idempotency_key("payments.charge", &args2, Some("agent-1"), Some("turn-4"));

        assert_eq!(key1, key2);
        assert!(key1.starts_with("idk_"));
        assert_eq!(key1.len(), 68);

        // Different step/turn produces different key
        let key3 =
            derive_idempotency_key("payments.charge", &args1, Some("agent-1"), Some("turn-5"));
        assert_ne!(key1, key3);

        // Different actor produces different key
        let key4 =
            derive_idempotency_key("payments.charge", &args1, Some("agent-2"), Some("turn-4"));
        assert_ne!(key1, key4);
    }

    #[tokio::test]
    async fn test_idempotency_deduplication_lifecycle() {
        let store = IdempotencyStore::new(Duration::from_secs(60));
        let key = "idem-key-1";

        match store.check_or_start(key).await {
            DeduplicateResult::New => {}
            _ => panic!("Expected New"),
        }

        let store_arc = Arc::new(store);
        let store_clone = store_arc.clone();
        let key_str = key.to_string();

        let (started_tx, started_rx) = tokio::sync::oneshot::channel();

        let handle = tokio::spawn(async move {
            let res = store_clone.check_or_start(&key_str).await;
            started_tx.send(()).unwrap();
            match res {
                DeduplicateResult::InProgress(mut rx) => rx.recv().await.unwrap(),
                _ => panic!("Expected InProgress"),
            }
        });

        started_rx.await.unwrap();
        let res_value = serde_json::json!({"ok": true, "data": "test"});
        store_arc.complete(key, res_value.clone()).await;

        let received = handle.await.unwrap();
        assert_eq!(received, res_value);

        match store_arc.check_or_start(key).await {
            DeduplicateResult::Completed(cached) => assert_eq!(cached, res_value),
            _ => panic!("Expected Completed"),
        }

        let rec = store_arc
            .get_record(key)
            .await
            .expect("record should exist");
        assert_eq!(rec.replay_count, 1);
    }

    #[tokio::test]
    async fn test_idempotency_persistence_across_restarts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_file = temp_dir.path().join("idempotency.json");
        let ttl = Duration::from_secs(300);

        // 1. First run: Start and complete an idempotent key
        let store1 = IdempotencyStore::open_or_create(&cache_file, ttl).unwrap();
        let key = "idem-restart-key";
        match store1
            .check_or_start_with_meta(
                key,
                Some("tools.calc".to_string()),
                Some("trace-initial".to_string()),
            )
            .await
        {
            DeduplicateResult::New => {}
            _ => panic!("Expected New"),
        }
        let completed_payload = serde_json::json!({
            "status": "success",
            "transaction_id": "tx-999"
        });
        store1.complete(key, completed_payload.clone()).await;

        drop(store1); // Simulate daemon restart

        // 2. Second run: Re-open from disk and check key
        let store2 = IdempotencyStore::open_or_create(&cache_file, ttl).unwrap();
        match store2.check_or_start(key).await {
            DeduplicateResult::Completed(cached) => {
                assert_eq!(cached, completed_payload);
            }
            _ => panic!("Expected Completed payload from persisted store"),
        }

        let rec = store2.get_record(key).await.expect("record should exist");
        assert_eq!(rec.capability_id.as_deref(), Some("tools.calc"));
        assert_eq!(rec.first_trace_id.as_deref(), Some("trace-initial"));
        assert_eq!(rec.replay_count, 1);
    }
}
