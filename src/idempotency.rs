// Rust guideline compliant 2026-08-13

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

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
}

/// In-memory idempotency deduplication store with configurable time-to-live TTL.
pub struct IdempotencyStore {
    ttl: Duration,
    entries: RwLock<HashMap<String, IdempotencyEntry>>,
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
    /// Creates a new `IdempotencyStore` with specified entry TTL duration.
    ///
    /// # Arguments
    /// * `ttl` - Entry expiry duration.
    ///
    /// # Returns
    /// An empty `IdempotencyStore`.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: RwLock::new(HashMap::new()),
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

        // Check if existing key is expired before returning
        if let Some(entry) = map.get(key_ref) {
            if now.duration_since(entry.created_at) < self.ttl {
                return match &entry.state {
                    EntryState::Completed(val) => DeduplicateResult::Completed(val.clone()),
                    EntryState::InProgress(tx) => DeduplicateResult::InProgress(tx.subscribe()),
                };
            }
            // Expired entry for this specific key, remove it
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
    }

    pub async fn remove(&self, key: &str) {
        let mut map = self.entries.write().await;
        map.remove(key);
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
}
