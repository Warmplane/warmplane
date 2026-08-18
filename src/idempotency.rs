// Rust guideline compliant 2026-08-18

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use tracing::error;

use crate::storage::AtomicFile;

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
    created_at: Instant,
    created_at_epoch_secs: u64,
}

/// Serialized record for persistent idempotency cache storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdempotencyRecord {
    pub key: String,
    pub response: Value,
    pub created_at_epoch_secs: u64,
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
    /// Request was already completed; contains cached payload.
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
                        created_at: created_instant,
                        created_at_epoch_secs: record.created_at_epoch_secs,
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
                            response: val.clone(),
                            created_at_epoch_secs: entry.created_at_epoch_secs,
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
        let key_ref = key.as_ref();
        let mut map = self.entries.write().await;
        let now = Instant::now();
        let now_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Check if existing key is valid before returning
        if let Some(entry) = map.get(key_ref) {
            let age = now.duration_since(entry.created_at);
            match &entry.state {
                EntryState::Completed(val) => {
                    if age < self.ttl {
                        return DeduplicateResult::Completed(val.clone());
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
                created_at: now,
                created_at_epoch_secs: now_epoch_secs,
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
        let mut map = self.entries.write().await;
        if let Some(entry) = map.get_mut(key_ref) {
            if let EntryState::InProgress(tx) = &entry.state {
                let _ = tx.send(result.clone());
            }
            entry.state = EntryState::Completed(result);
        }
        drop(map);
        self.sync_to_disk().await;
    }

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
    }

    #[tokio::test]
    async fn test_idempotency_persistence_across_restarts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_file = temp_dir.path().join("idempotency.json");
        let ttl = Duration::from_secs(300);

        // 1. First run: Start and complete an idempotent key
        let store1 = IdempotencyStore::open_or_create(&cache_file, ttl).unwrap();
        let key = "idem-restart-key";
        match store1.check_or_start(key).await {
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
    }
}
