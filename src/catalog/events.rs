use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::RwLock;
use tracing::error;

use crate::storage::AtomicFile;

/// Represents a single catalog mutation event for capabilities, resources, prompts, or server logs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogEvent {
    /// Unique identifier for the catalog event.
    pub id: String,
    /// Unix timestamp when the event occurred.
    pub timestamp: String,
    /// Object type affected (e.g. `capability`, `resource`, `prompt`, `server_log`, `catalog_reindex`).
    pub object_type: String,
    /// Identifier of the object affected.
    pub object_id: String,
    /// Type of change performed (e.g. `added`, `updated`, `removed`, `log`, `reindexed`).
    pub change_type: String,
    /// Optional detail string payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Represents a real-time resource update notification event emitted by an upstream MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdateEvent {
    /// Resource URI that was updated.
    pub uri: String,
    /// Timestamp of update event.
    pub timestamp: String,
    /// Originating server identifier.
    pub server: String,
}

/// Serialized state container for catalog event persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedCatalogState {
    pub counter: u64,
    pub events: Vec<CatalogEvent>,
}

/// Maximum number of catalog events retained in memory.
pub const MAX_CATALOG_EVENTS: usize = 5_000;

/// In-memory or disk-persisted event store recording catalog state changes.
pub struct CatalogEventStore {
    events: RwLock<Vec<CatalogEvent>>,
    counter: std::sync::atomic::AtomicU64,
    storage: Option<AtomicFile<PersistedCatalogState>>,
}

impl Default for CatalogEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogEventStore {
    /// Creates a new empty in-memory `CatalogEventStore`.
    ///
    /// # Returns
    /// An empty `CatalogEventStore` instance.
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            counter: std::sync::atomic::AtomicU64::new(1),
            storage: None,
        }
    }

    /// Initializes a `CatalogEventStore` backed by a persistent atomic JSON file.
    ///
    /// # Arguments
    /// * `path` - Destination path for persistent catalog events.
    ///
    /// # Errors
    /// Returns an error if reading or parsing existing storage fails.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let loaded: Option<PersistedCatalogState> = storage.load_opt()?;

        let (counter_val, events_vec) = match loaded {
            Some(state) => (state.counter.max(1), state.events),
            None => (1, Vec::new()),
        };

        Ok(Self {
            events: RwLock::new(events_vec),
            counter: std::sync::atomic::AtomicU64::new(counter_val),
            storage: Some(storage),
        })
    }

    fn sync_to_disk(&self) {
        if let Some(ref store) = self.storage {
            let guard = self.events.read().unwrap_or_else(|e| e.into_inner());
            let current_counter = self.counter.load(std::sync::atomic::Ordering::Relaxed);
            let state = PersistedCatalogState {
                counter: current_counter,
                events: guard.clone(),
            };
            drop(guard);
            if let Err(e) = store.save(&state) {
                error!(error = %e, path = %store.path().display(), "failed to persist catalog events to disk");
            }
        }
    }

    /// Records a catalog mutation event with an optional detail string.
    ///
    /// # Arguments
    /// * `object_type` - Type of object mutated.
    /// * `object_id` - Unique identifier of mutated object.
    /// * `change_type` - Mutation classification.
    /// * `detail` - Optional detail string payload.
    pub fn record_with_detail(
        &self,
        object_type: impl AsRef<str>,
        object_id: impl AsRef<str>,
        change_type: impl AsRef<str>,
        detail: Option<impl Into<String>>,
    ) {
        let seq = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let event_id = format!("evt_{}", seq);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let mut guard = self.events.write().unwrap_or_else(|e| e.into_inner());
        if guard.len() >= MAX_CATALOG_EVENTS {
            let excess = guard.len() - (MAX_CATALOG_EVENTS - 1);
            guard.drain(0..excess);
        }

        guard.push(CatalogEvent {
            id: event_id,
            timestamp,
            object_type: object_type.as_ref().to_string(),
            object_id: object_id.as_ref().to_string(),
            change_type: change_type.as_ref().to_string(),
            detail: detail.map(|d| d.into()),
        });
        drop(guard);

        self.sync_to_disk();
    }

    /// Records a catalog mutation event.
    ///
    /// # Arguments
    /// * `object_type` - Type of object mutated.
    /// * `object_id` - Unique identifier of mutated object.
    /// * `change_type` - Mutation classification.
    pub fn record(
        &self,
        object_type: impl AsRef<str>,
        object_id: impl AsRef<str>,
        change_type: impl AsRef<str>,
    ) {
        self.record_with_detail(object_type, object_id, change_type, None::<String>);
    }

    /// Retrieves events occurring after an optional cursor position.
    ///
    /// # Arguments
    /// * `after_cursor` - Optional cursor ID pointing to last processed event.
    ///
    /// # Returns
    /// A tuple containing matching `CatalogEvent` items and the latest cursor ID string.
    pub fn get_events_after(&self, after_cursor: Option<&str>) -> (Vec<CatalogEvent>, String) {
        let guard = self.events.read().unwrap_or_else(|e| e.into_inner());
        let start_index = match after_cursor {
            Some(cursor) if !cursor.is_empty() => guard
                .iter()
                .position(|e| e.id == cursor)
                .map(|idx| idx + 1)
                .unwrap_or(0),
            _ => 0,
        };

        let result_events = guard[start_index..].to_vec();
        let next_cursor = guard.last().map(|e| e.id.clone()).unwrap_or_default();
        (result_events, next_cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_store_records_and_filters_by_cursor() {
        let store = CatalogEventStore::new();
        store.record("capability", "github.issues.search", "added");
        store.record("resource", "fs.readme", "added");

        let (all, cursor) = store.get_events_after(None);
        assert_eq!(all.len(), 2);
        assert_eq!(cursor, "evt_2");

        let (filtered, _) = store.get_events_after(Some("evt_1"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "evt_2");
        assert_eq!(filtered[0].object_id, "fs.readme");
    }

    #[test]
    fn test_catalog_event_store_persistence_across_restarts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state_file = temp_dir.path().join("catalog_events.json");

        // 1. Record events on initial instance
        let store1 = CatalogEventStore::open_or_create(&state_file).unwrap();
        store1.record("capability", "docker.start", "added");
        store1.record("capability", "docker.stop", "added");

        let (events1, cursor1) = store1.get_events_after(None);
        assert_eq!(events1.len(), 2);
        assert_eq!(cursor1, "evt_2");

        drop(store1); // Simulate restart

        // 2. Re-open and record 3rd event
        let store2 = CatalogEventStore::open_or_create(&state_file).unwrap();
        store2.record("resource", "docker.status", "updated");

        let (all_events, cursor2) = store2.get_events_after(None);
        assert_eq!(all_events.len(), 3);
        assert_eq!(cursor2, "evt_3");

        // Polling from old cursor `evt_2` returns only `evt_3`
        let (paged, _) = store2.get_events_after(Some("evt_2"));
        assert_eq!(paged.len(), 1);
        assert_eq!(paged[0].id, "evt_3");
        assert_eq!(paged[0].object_id, "docker.status");
    }
}
