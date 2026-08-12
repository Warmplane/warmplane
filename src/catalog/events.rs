use serde::{Deserialize, Serialize};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEvent {
    pub id: String,
    pub timestamp: String,
    pub object_type: String,
    pub object_id: String,
    pub change_type: String,
}

pub struct CatalogEventStore {
    events: RwLock<Vec<CatalogEvent>>,
}

impl CatalogEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }

    pub fn record(&self, object_type: &str, object_id: &str, change_type: &str) {
        let mut guard = self.events.write().unwrap_or_else(|e| e.into_inner());
        let event_id = format!("evt_{}", guard.len() + 1);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        guard.push(CatalogEvent {
            id: event_id,
            timestamp,
            object_type: object_type.to_string(),
            object_id: object_id.to_string(),
            change_type: change_type.to_string(),
        });
    }

    pub fn get_events_after(&self, after_cursor: Option<&str>) -> (Vec<CatalogEvent>, String) {
        let guard = self.events.read().unwrap_or_else(|e| e.into_inner());
        let start_index = match after_cursor {
            Some(cursor) if !cursor.is_empty() => {
                guard.iter().position(|e| e.id == cursor).map(|idx| idx + 1).unwrap_or(0)
            }
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
