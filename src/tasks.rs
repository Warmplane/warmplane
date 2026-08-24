// Rust guideline compliant 2026-08-24

//! MCP SEP-2663 Tasks Extension State Machine and Registry (`M-CANONICAL-DOCS`).
//!
//! Provides asynchronous task management, durable lifecycle transitions,
//! cooperative cancellation, input request resolution (MRTR), and disk persistence.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{oneshot, RwLock};
use tracing::error;

use crate::context::RequestContext;
use crate::storage::AtomicFile;

/// Task execution status per SEP-2663.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// The request is currently being processed.
    Working,
    /// The server needs input from the client (e.g. HITL approval or parameter review).
    InputRequired,
    /// The request completed successfully and terminal results are available.
    Completed,
    /// The request was cancelled cooperatively before completion.
    Cancelled,
    /// The request failed due to a JSON-RPC error during execution.
    Failed,
}

/// Durable internal record of a Task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Unique task identifier (e.g. `task-1723668200-1001`).
    pub task_id: String,
    /// Current task status.
    pub status: TaskStatus,
    /// Optional human-readable message describing the current status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// Associated capability identifier being executed.
    pub capability_id: String,
    /// Target upstream server ID.
    pub server_id: String,
    /// Execution arguments.
    pub args: Value,
    /// Optional incoming request identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Optional caller context envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestContext>,
    /// Optional idempotency key linked to this task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    /// Pending server-to-client input requests (MRTR) when in `InputRequired` status.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub input_requests: BTreeMap<String, Value>,
    /// Accumulated client input responses.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub input_responses: BTreeMap<String, Value>,
    /// Final execution result on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error payload if task failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    /// ISO 8601 creation timestamp string.
    pub created_at: String,
    /// ISO 8601 last update timestamp string.
    pub last_updated_at: String,
    /// Epoch timestamp (seconds) when task was created.
    pub created_at_epoch_secs: u64,
    /// TTL duration from creation in milliseconds, or null if unlimited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<u64>,
    /// Suggested polling interval in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poll_interval_ms: Option<u64>,
}

/// Wire representation of a Task for `tasks/get` and `CreateTaskResult`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub status: TaskStatus,
    #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: String,
    #[serde(rename = "ttlMs", skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<u64>,
    #[serde(rename = "pollIntervalMs", skip_serializing_if = "Option::is_none")]
    pub poll_interval_ms: Option<u64>,
    #[serde(rename = "inputRequests", skip_serializing_if = "Option::is_none")]
    pub input_requests: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

impl From<&TaskRecord> for TaskResponse {
    fn from(record: &TaskRecord) -> Self {
        Self {
            task_id: record.task_id.clone(),
            status: record.status.clone(),
            status_message: record.status_message.clone(),
            created_at: record.created_at.clone(),
            last_updated_at: record.last_updated_at.clone(),
            ttl_ms: record.ttl_ms,
            poll_interval_ms: record.poll_interval_ms,
            input_requests: if record.status == TaskStatus::InputRequired
                && !record.input_requests.is_empty()
            {
                Some(record.input_requests.clone())
            } else {
                None
            },
            result: record.result.clone(),
            error: record.error.clone(),
        }
    }
}

/// Request parameters for creating a new Task.
#[derive(Clone, Debug)]
pub struct CreateTaskParams {
    pub capability_id: String,
    pub server_id: String,
    pub args: Value,
    pub request_id: Option<String>,
    pub context: Option<RequestContext>,
    pub idempotency_key: Option<String>,
    pub initial_status: TaskStatus,
    pub status_message: Option<String>,
    pub input_requests: Option<BTreeMap<String, Value>>,
    pub ttl_ms: Option<u64>,
    pub poll_interval_ms: Option<u64>,
}

type TaskWaitSender = oneshot::Sender<BTreeMap<String, Value>>;
type TaskWaitReceiver = oneshot::Receiver<BTreeMap<String, Value>>;

/// Global registry managing active and persisted tasks.
#[derive(Clone, Default)]
pub struct TaskRegistry {
    tasks: Arc<RwLock<HashMap<String, TaskRecord>>>,
    wait_channels: Arc<RwLock<HashMap<String, TaskWaitSender>>>,
    storage: Option<AtomicFile<HashMap<String, TaskRecord>>>,
}

impl TaskRegistry {
    /// Creates a new in-memory `TaskRegistry`.
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            wait_channels: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
        }
    }

    /// Initializes a `TaskRegistry` backed by a persistent atomic JSON file.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let mut loaded: HashMap<String, TaskRecord> = storage.load_opt()?.unwrap_or_default();
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut expired_any = false;
        for record in loaded.values_mut() {
            if record.status == TaskStatus::Working || record.status == TaskStatus::InputRequired {
                if let Some(ttl_ms) = record.ttl_ms {
                    let ttl_secs = ttl_ms / 1000;
                    if now_secs >= record.created_at_epoch_secs + ttl_secs {
                        record.status = TaskStatus::Failed;
                        record.status_message = Some("Task expired due to TTL timeout".to_string());
                        expired_any = true;
                    }
                }
            }
        }

        if expired_any {
            let _ = storage.save(&loaded);
        }

        Ok(Self {
            tasks: Arc::new(RwLock::new(loaded)),
            wait_channels: Arc::new(RwLock::new(HashMap::new())),
            storage: Some(storage),
        })
    }

    async fn sync_to_disk(&self) {
        if let Some(ref store) = self.storage {
            let guard = self.tasks.read().await;
            if let Err(e) = store.save(&*guard) {
                error!(error = %e, path = %store.path().display(), "failed to persist task registry state to disk");
            }
        }
    }

    /// Creates and registers a new task.
    pub async fn create_task(
        &self,
        params: CreateTaskParams,
    ) -> (TaskRecord, Option<TaskWaitReceiver>) {
        let now = SystemTime::now();
        let now_secs = now
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let iso_time = format_iso_time(now);
        let suffix: u32 = rand::random::<u32>() % 9000 + 1000;
        let task_id = format!("task-{}-{}", now_secs, suffix);

        let (tx, rx) = if params.initial_status == TaskStatus::InputRequired {
            let (tx, rx) = oneshot::channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };

        if let Some(channel) = tx {
            let mut channels = self.wait_channels.write().await;
            channels.insert(task_id.clone(), channel);
        }

        let record = TaskRecord {
            task_id: task_id.clone(),
            status: params.initial_status,
            status_message: params.status_message,
            capability_id: params.capability_id,
            server_id: params.server_id,
            args: params.args,
            request_id: params.request_id,
            context: params.context,
            idempotency_key: params.idempotency_key,
            input_requests: params.input_requests.unwrap_or_default(),
            input_responses: BTreeMap::new(),
            result: None,
            error: None,
            created_at: iso_time.clone(),
            last_updated_at: iso_time,
            created_at_epoch_secs: now_secs,
            ttl_ms: params.ttl_ms.or(Some(300_000)), // default 5m TTL
            poll_interval_ms: params.poll_interval_ms.or(Some(1_000)),
        };

        {
            let mut guard = self.tasks.write().await;
            guard.insert(task_id, record.clone());
        }

        self.sync_to_disk().await;
        (record, rx)
    }

    /// Retrieves a task by its task ID, checking for TTL expiry.
    pub async fn get_task(&self, task_id: &str) -> Option<TaskRecord> {
        let mut guard = self.tasks.write().await;
        if let Some(record) = guard.get_mut(task_id) {
            if record.status == TaskStatus::Working || record.status == TaskStatus::InputRequired {
                if let Some(ttl_ms) = record.ttl_ms {
                    let now_secs = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let ttl_secs = ttl_ms / 1000;
                    if now_secs >= record.created_at_epoch_secs + ttl_secs {
                        record.status = TaskStatus::Failed;
                        record.status_message = Some("Task expired due to TTL timeout".to_string());
                        record.last_updated_at = format_iso_time(SystemTime::now());
                    }
                }
            }
            Some(record.clone())
        } else {
            None
        }
    }

    /// Finds an active or completed task by idempotency key.
    pub async fn find_by_idempotency_key(&self, key: &str) -> Option<TaskRecord> {
        let guard = self.tasks.read().await;
        guard
            .values()
            .find(|t| t.idempotency_key.as_deref() == Some(key))
            .cloned()
    }

    /// Lists all tasks.
    pub async fn list_tasks(&self) -> Vec<TaskRecord> {
        let guard = self.tasks.read().await;
        let mut list: Vec<TaskRecord> = guard.values().cloned().collect();
        list.sort_by_key(|t| std::cmp::Reverse(t.created_at_epoch_secs));
        list
    }

    /// Updates a task with client input responses.
    pub async fn update_task(
        &self,
        task_id: &str,
        input_responses: BTreeMap<String, Value>,
    ) -> Result<bool> {
        let mut guard = self.tasks.write().await;
        let Some(record) = guard.get_mut(task_id) else {
            return Ok(false);
        };

        if record.status != TaskStatus::InputRequired {
            return Ok(false);
        }

        for (k, v) in &input_responses {
            record.input_responses.insert(k.clone(), v.clone());
        }

        record.status = TaskStatus::Working;
        record.status_message = Some("Input responses received; resuming execution".to_string());
        record.last_updated_at = format_iso_time(SystemTime::now());

        // Notify waiting worker thread
        let mut channels = self.wait_channels.write().await;
        if let Some(tx) = channels.remove(task_id) {
            let _ = tx.send(input_responses);
        }

        drop(guard);
        self.sync_to_disk().await;
        Ok(true)
    }

    /// Cancels an in-progress or suspended task cooperatively.
    pub async fn cancel_task(&self, task_id: &str, reason: Option<String>) -> Result<bool> {
        let mut guard = self.tasks.write().await;
        let Some(record) = guard.get_mut(task_id) else {
            return Ok(false);
        };

        if record.status == TaskStatus::Completed
            || record.status == TaskStatus::Failed
            || record.status == TaskStatus::Cancelled
        {
            return Ok(false);
        }

        record.status = TaskStatus::Cancelled;
        record.status_message = reason.or_else(|| Some("Task cancelled by caller".to_string()));
        record.last_updated_at = format_iso_time(SystemTime::now());

        let mut channels = self.wait_channels.write().await;
        channels.remove(task_id);

        drop(guard);
        self.sync_to_disk().await;
        Ok(true)
    }

    /// Marks a task as completed with results.
    pub async fn complete_task(&self, task_id: &str, result: Value) -> Result<()> {
        let mut guard = self.tasks.write().await;
        if let Some(record) = guard.get_mut(task_id) {
            record.status = TaskStatus::Completed;
            record.status_message = Some("Execution completed successfully".to_string());
            record.result = Some(result);
            record.last_updated_at = format_iso_time(SystemTime::now());
        }

        let mut channels = self.wait_channels.write().await;
        channels.remove(task_id);

        drop(guard);
        self.sync_to_disk().await;
        Ok(())
    }

    /// Marks a task as failed with an error payload.
    pub async fn fail_task(
        &self,
        task_id: &str,
        error: Value,
        message: Option<String>,
    ) -> Result<()> {
        let mut guard = self.tasks.write().await;
        if let Some(record) = guard.get_mut(task_id) {
            record.status = TaskStatus::Failed;
            record.status_message = message.or_else(|| Some("Execution failed".to_string()));
            record.error = Some(error);
            record.last_updated_at = format_iso_time(SystemTime::now());
        }

        let mut channels = self.wait_channels.write().await;
        channels.remove(task_id);

        drop(guard);
        self.sync_to_disk().await;
        Ok(())
    }
}

fn format_iso_time(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple standard format YYYY-MM-DDTHH:MM:SSZ
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    // Approximate year/month/day calculation
    let mut y = 1970;
    let mut d = days;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        d + 1,
        hours,
        minutes,
        seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_task_lifecycle() {
        let registry = TaskRegistry::new();
        let (task, rx) = registry
            .create_task(CreateTaskParams {
                capability_id: "db.query".to_string(),
                server_id: "sqlite".to_string(),
                args: json!({"query": "SELECT * FROM users"}),
                request_id: Some("req-1".to_string()),
                context: None,
                idempotency_key: Some("idk_123".to_string()),
                initial_status: TaskStatus::InputRequired,
                status_message: Some("Approval needed".to_string()),
                input_requests: Some({
                    let mut m = BTreeMap::new();
                    m.insert("approval".to_string(), json!({"type": "hitl_approval"}));
                    m
                }),
                ttl_ms: Some(60_000),
                poll_interval_ms: Some(500),
            })
            .await;

        assert_eq!(task.status, TaskStatus::InputRequired);
        assert!(rx.is_some());

        // Update task with input responses
        let mut responses = BTreeMap::new();
        responses.insert("approval".to_string(), json!({"approved": true}));
        let updated = registry
            .update_task(&task.task_id, responses.clone())
            .await
            .unwrap();
        assert!(updated);

        // Verify receiver gets response
        let res_map = rx.unwrap().await.unwrap();
        assert_eq!(res_map.get("approval").unwrap(), &json!({"approved": true}));

        let fetched = registry.get_task(&task.task_id).await.unwrap();
        assert_eq!(fetched.status, TaskStatus::Working);

        // Complete task
        registry
            .complete_task(&task.task_id, json!({"rows": [1, 2, 3]}))
            .await
            .unwrap();

        let final_task = registry.get_task(&task.task_id).await.unwrap();
        assert_eq!(final_task.status, TaskStatus::Completed);
        assert_eq!(final_task.result.unwrap(), json!({"rows": [1, 2, 3]}));
    }
}
