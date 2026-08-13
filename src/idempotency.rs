use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryMetadata {
    pub classification: String,           // "safe", "unsafe", "idempotent"
    pub upstream_execution_state: String, // "not_started", "completed", "unknown"
}

impl RetryMetadata {
    pub fn safe(state: &str) -> Self {
        Self {
            classification: "safe".to_string(),
            upstream_execution_state: state.to_string(),
        }
    }

    pub fn unsafe_op(state: &str) -> Self {
        Self {
            classification: "unsafe".to_string(),
            upstream_execution_state: state.to_string(),
        }
    }

    pub fn idempotent(state: &str) -> Self {
        Self {
            classification: "idempotent".to_string(),
            upstream_execution_state: state.to_string(),
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

pub struct IdempotencyStore {
    ttl: Duration,
    entries: RwLock<HashMap<String, IdempotencyEntry>>,
}

pub enum DeduplicateResult {
    New,
    InProgress(broadcast::Receiver<Value>),
    Completed(Value),
}

impl IdempotencyStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub async fn check_or_start(&self, key: &str) -> DeduplicateResult {
        let mut map = self.entries.write().await;
        let now = Instant::now();

        // Evict expired entries
        map.retain(|_, entry| now.duration_since(entry.created_at) < self.ttl);

        if let Some(entry) = map.get(key) {
            match &entry.state {
                EntryState::Completed(val) => DeduplicateResult::Completed(val.clone()),
                EntryState::InProgress(tx) => DeduplicateResult::InProgress(tx.subscribe()),
            }
        } else {
            let (tx, _) = broadcast::channel(1);
            map.insert(
                key.to_string(),
                IdempotencyEntry {
                    state: EntryState::InProgress(tx),
                    created_at: now,
                },
            );
            DeduplicateResult::New
        }
    }

    pub async fn complete(&self, key: &str, result: Value) {
        let mut map = self.entries.write().await;
        if let Some(entry) = map.get_mut(key) {
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
