// Rust guideline compliant 2026-08-13

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Represents a single catalog mutation event for capabilities, resources, prompts, or server logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// In-memory thread-safe event store recording catalog state changes.
pub struct CatalogEventStore {
    events: RwLock<Vec<CatalogEvent>>,
}

impl Default for CatalogEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogEventStore {
    /// Creates a new empty `CatalogEventStore`.
    ///
    /// # Returns
    /// An empty `CatalogEventStore` instance.
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
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
        let mut guard = self.events.write().unwrap_or_else(|e| e.into_inner());
        let event_id = format!("evt_{}", guard.len() + 1);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        guard.push(CatalogEvent {
            id: event_id,
            timestamp,
            object_type: object_type.as_ref().to_string(),
            object_id: object_id.as_ref().to_string(),
            change_type: change_type.as_ref().to_string(),
            detail: detail.map(|d| d.into()),
        });
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
}
