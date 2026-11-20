// Rust guideline compliant 2026-11-18
//! MCP SEP-2663 Tasks Extension State Machine and Registry (`M-CANONICAL-DOCS`).
//!
//! Provides asynchronous task management, durable lifecycle transitions,
//! cooperative cancellation, input request resolution (MRTR), and disk persistence.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{oneshot, RwLock};

use crate::storage::{AtomicFile, ReplacementDurabilityUnknown};
use crate::{context::RequestContext, engine::WarmplaneError};

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

/// Dispatch phase for a task holding a delayable emission.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmissionPhase {
    /// The accepted call is waiting for its cancellation deadline.
    PendingEmission,
    /// Dispatch was durably claimed and may have reached the upstream server.
    Dispatching,
}

/// Persisted execution details needed to resume an accepted held emission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmissionRecord {
    /// In-process idempotency ownership for the originating execution.
    #[serde(skip)]
    pub idempotency_claim: Option<crate::idempotency::IdempotencyClaim>,
    /// Current pre-dispatch or dispatching phase.
    pub phase: EmissionPhase,
    /// Absolute Unix deadline in milliseconds.
    pub release_at_ms: u64,
    /// Whether cancellation can still prevent dispatch.
    pub cancellable: bool,
    /// Resolved upstream tool name.
    pub tool_name: String,
    /// Multi-roundtrip client input responses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_responses: Option<BTreeMap<String, Value>>,
    /// Multi-roundtrip upstream request state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_state: Option<String>,
    /// Profile used when the call was accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    /// Authenticated tenant used when the call was accepted.
    pub tenant_id: String,
    /// Authenticated role used when the call was accepted.
    pub role: String,
    /// Verified JWT expiry bound carried for release-time revalidation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_expires_at_epoch_secs: Option<u64>,
    /// Whether required approval completed before the hold was accepted.
    #[serde(default)]
    pub approval_completed: bool,
    /// Parent batch request identifier, when this task represents a batch step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// Batch step identifier, when this task represents a batch step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
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
    /// Effective sanitization policy revision used to produce the stored result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_policy_revision: Option<String>,
    /// Authenticated scope associated with this task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_access: Option<crate::result_sanitization::ResultAccess>,
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
    /// Delayable emission metadata, absent for ordinary tasks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emission: Option<EmissionRecord>,
}

/// Wire representation of a Task for `tasks/get` and `CreateTaskResult`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    #[serde(rename = "taskId")]
    pub task_id: String,
    pub status: TaskStatus,
    #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(rename = "capabilityId", skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
    #[serde(rename = "serverId", skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: String,
    #[serde(rename = "createdAtEpochSecs")]
    pub created_at_epoch_secs: u64,
    #[serde(rename = "expiresAtEpochSecs", skip_serializing_if = "Option::is_none")]
    pub expires_at_epoch_secs: Option<u64>,
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
    /// Request identifier used to correlate operation cancellation.
    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Current delayable emission phase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<EmissionPhase>,
    /// Absolute Unix dispatch deadline in milliseconds.
    #[serde(rename = "releaseAtMs", skip_serializing_if = "Option::is_none")]
    pub release_at_ms: Option<u64>,
    /// Whether cancellation can still prevent upstream dispatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellable: Option<bool>,
}

/// Converts a task lifetime in milliseconds to whole seconds, rounding up.
///
/// Creation times are stamped in whole seconds, so a millisecond TTL must be
/// rounded up rather than truncated. Truncating collapses any lifetime under
/// one second to `0`, which expires the task the instant it is created.
/// Rounding up keeps the real lifetime within a second of the requested one
/// and never shortens it to nothing.
///
/// # Arguments
/// * `ttl_ms` - Task lifetime in milliseconds.
///
/// # Returns
/// The lifetime in whole seconds. A non-zero input never returns `0`.
fn ttl_ms_to_secs_ceil(ttl_ms: u64) -> u64 {
    if ttl_ms == 0 {
        return 0;
    }
    ttl_ms.div_ceil(1000)
}

fn task_expiry(record: &TaskRecord) -> Option<u64> {
    record.ttl_ms.map(|ms| {
        record
            .created_at_epoch_secs
            .saturating_add(ttl_ms_to_secs_ceil(ms))
    })
}

fn expire_task(record: &mut TaskRecord, now: SystemTime) -> bool {
    let now_secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    if record.emission.is_some()
        || !matches!(
            record.status,
            TaskStatus::Working | TaskStatus::InputRequired
        )
        || !task_expiry(record).is_some_and(|deadline| now_secs >= deadline)
    {
        return false;
    }

    record.status = TaskStatus::Failed;
    let message = "Task expired due to TTL timeout";
    record.status_message = Some(message.to_string());
    record.error = Some(serde_json::json!({"code": "TASK_EXPIRED", "message": message}));
    record.last_updated_at = format_iso_time(now);
    true
}

impl From<&TaskRecord> for TaskResponse {
    fn from(record: &TaskRecord) -> Self {
        let expires_at_epoch_secs = task_expiry(record);

        Self {
            task_id: record.task_id.clone(),
            status: record.status.clone(),
            status_message: record.status_message.clone(),
            capability_id: Some(record.capability_id.clone()),
            server_id: Some(record.server_id.clone()),
            created_at: record.created_at.clone(),
            last_updated_at: record.last_updated_at.clone(),
            created_at_epoch_secs: record.created_at_epoch_secs,
            expires_at_epoch_secs,
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
            request_id: record.request_id.clone(),
            phase: record
                .emission
                .as_ref()
                .map(|emission| emission.phase.clone()),
            release_at_ms: record
                .emission
                .as_ref()
                .map(|emission| emission.release_at_ms),
            cancellable: record
                .emission
                .as_ref()
                .map(|emission| emission.cancellable),
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

/// Result of attempting to cancel a held emission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmissionCancelResult {
    /// Cancellation was persisted and will prevent dispatch.
    Cancelled,
    /// The emission had already been cancelled.
    AlreadyCancelled,
    /// Dispatch was durably claimed, so prevention can no longer be promised.
    TooLate,
    /// No held emission exists for the requested task.
    NotEmission,
}

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
        Self::default()
    }

    /// Initializes a `TaskRegistry` backed by a persistent atomic JSON file.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let mut loaded: HashMap<String, TaskRecord> = storage.load_opt()?.unwrap_or_default();
        let now = SystemTime::now();
        let mut expired_any = false;
        for record in loaded.values_mut() {
            expired_any |= expire_task(record, now);
        }
        if expired_any {
            storage.save_logged(&loaded, "failed to persist expired tasks on startup");
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
            store.save_logged(&*guard, "failed to persist task registry state to disk");
        }
    }

    /// Creates and registers a new task.
    pub async fn create_task(
        &self,
        params: CreateTaskParams,
    ) -> (TaskRecord, Option<TaskWaitReceiver>) {
        self.create_scoped_task(params, None).await
    }

    pub(crate) async fn create_scoped_task(
        &self,
        params: CreateTaskParams,
        access: Option<crate::result_sanitization::ResultAccess>,
    ) -> (TaskRecord, Option<TaskWaitReceiver>) {
        let now = SystemTime::now();
        let now_secs = now
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let iso_time = format_iso_time(now);
        let task_id = format!("task-{}-{:032x}", now_secs, rand::random::<u128>());

        let (tx, rx) = if params.initial_status == TaskStatus::InputRequired {
            let (tx, rx) = oneshot::channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };

        let mut params = params;
        params.ttl_ms = params.ttl_ms.or(Some(300_000));
        let mut record = new_task_record(task_id, params, now_secs, iso_time, None);
        record.result_access = access;

        let mut guard = self.tasks.write().await;
        while guard.contains_key(&record.task_id) {
            record.task_id = format!("task-{}-{:032x}", now_secs, rand::random::<u128>());
        }
        guard.insert(record.task_id.clone(), record.clone());
        if let Some(channel) = tx {
            let mut channels = self.wait_channels.write().await;
            channels.insert(record.task_id.clone(), channel);
        }

        drop(guard);
        self.sync_to_disk().await;
        (record, rx)
    }

    /// Creates a held emission and stores its task record.
    ///
    /// A visible replacement with uncertain durability counts as accepted. The
    /// returned status message reports the uncertainty, and dispatch still
    /// requires a durable claim.
    ///
    /// # Errors
    /// Returns an error if persistence fails before the replacement becomes visible.
    pub async fn create_emission_task(
        &self,
        params: CreateTaskParams,
        emission: EmissionRecord,
    ) -> Result<TaskRecord> {
        let now = SystemTime::now();
        let now_secs = now
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let iso_time = format_iso_time(now);
        let mut guard = self.tasks.write().await;
        let task_id = loop {
            let candidate = format!("task-{now_secs}-{:032x}", rand::random::<u128>());
            if !guard.contains_key(&candidate) {
                break candidate;
            }
        };
        let record = new_task_record(task_id.clone(), params, now_secs, iso_time, Some(emission));
        guard.insert(task_id.clone(), record);
        match self.persist_emission_acceptance(&mut guard, &task_id) {
            Ok(record) => Ok(record),
            Err(error) => {
                guard.remove(&task_id);
                Err(error).context("failed to persist accepted emission task")
            }
        }
    }

    fn persist_emission_acceptance(
        &self,
        tasks: &mut HashMap<String, TaskRecord>,
        task_id: &str,
    ) -> Result<TaskRecord> {
        let persistence = self
            .storage
            .as_ref()
            .map_or(Ok(()), |storage| storage.save(tasks));
        let record = tasks
            .get_mut(task_id)
            .context("emission task missing during acceptance")?;
        if let Err(error) = persistence {
            if !error.is::<ReplacementDurabilityUnknown>() {
                return Err(error);
            }
            tracing::warn!(task_id, error = %error, "accepted visible emission with uncertain durability");
            record.status_message = Some(
                "Emission accepted and waiting for release. Its persistence across restart is uncertain."
                    .to_string(),
            );
        }
        Ok(record.clone())
    }

    /// Durably claims a pending emission for dispatch, preventing later cancellation.
    pub async fn claim_emission_dispatch(&self, task_id: &str) -> Result<Option<TaskRecord>> {
        let mut guard = self.tasks.write().await;
        let Some(record) = guard.get_mut(task_id) else {
            return Ok(None);
        };
        if matches!(
            record.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            return Ok(None);
        }
        let Some(emission) = record.emission.as_ref() else {
            return Ok(None);
        };
        if emission.phase != EmissionPhase::PendingEmission || !emission.cancellable {
            return Ok(None);
        }
        let previous = record.clone();
        let Some(emission) = record.emission.as_mut() else {
            return Ok(None);
        };
        emission.phase = EmissionPhase::Dispatching;
        emission.cancellable = false;
        record.status = TaskStatus::Working;
        record.status_message = Some("Emission dispatch has started".to_string());
        record.last_updated_at = format_iso_time(SystemTime::now());
        if let Some(storage) = &self.storage {
            if let Err(error) = storage.save(&guard) {
                if !error.is::<ReplacementDurabilityUnknown>() {
                    guard.insert(task_id.to_string(), previous);
                } else {
                    self.wait_channels.write().await.remove(task_id);
                }
                return Err(error).context("failed to persist emission dispatch claim");
            }
        }
        self.wait_channels.write().await.remove(task_id);
        Ok(guard.get(task_id).cloned())
    }

    /// Attaches a held-emission payload to a task already created for approval.
    ///
    /// Acceptance follows the same persistence rules as [Self::create_emission_task].
    ///
    /// # Errors
    /// Returns an error if persistence fails before the replacement becomes visible.
    pub async fn attach_emission(
        &self,
        task_id: &str,
        approved_args: Value,
        emission: EmissionRecord,
    ) -> Result<Option<TaskRecord>> {
        let mut guard = self.tasks.write().await;
        let Some(record) = guard.get_mut(task_id) else {
            return Ok(None);
        };
        if matches!(
            record.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            return Ok(None);
        }
        let previous = record.clone();
        record.args = approved_args;
        record.status = TaskStatus::Working;
        record.status_message = Some("Waiting for the emission release deadline".to_string());
        record.ttl_ms = None;
        record.last_updated_at = format_iso_time(SystemTime::now());
        record.emission = Some(emission);
        let attached = match self.persist_emission_acceptance(&mut guard, task_id) {
            Ok(record) => record,
            Err(error) => {
                guard.insert(task_id.to_string(), previous);
                return Err(error).context("failed to persist approved emission task");
            }
        };
        self.wait_channels.write().await.remove(task_id);
        Ok(Some(attached))
    }

    /// Cancels a held emission only while it remains pending and cancellable.
    pub async fn cancel_emission_task(
        &self,
        task_id: &str,
        reason: Option<String>,
    ) -> Result<EmissionCancelResult> {
        let mut guard = self.tasks.write().await;
        let Some(record) = guard.get_mut(task_id) else {
            return Ok(EmissionCancelResult::NotEmission);
        };
        let Some(emission) = record.emission.as_ref() else {
            return Ok(EmissionCancelResult::NotEmission);
        };
        match record.status {
            TaskStatus::Cancelled => return Ok(EmissionCancelResult::AlreadyCancelled),
            TaskStatus::Completed | TaskStatus::Failed => {
                return Ok(EmissionCancelResult::NotEmission)
            }
            TaskStatus::Working | TaskStatus::InputRequired => {}
        }
        match emission.phase {
            EmissionPhase::PendingEmission if emission.cancellable => {}
            EmissionPhase::PendingEmission | EmissionPhase::Dispatching => {
                return Ok(EmissionCancelResult::TooLate)
            }
        }
        let previous = record.clone();
        if let Some(emission) = record.emission.as_mut() {
            emission.cancellable = false;
        }
        record.status = TaskStatus::Cancelled;
        record.status_message =
            reason.or_else(|| Some("Emission cancelled before dispatch".to_string()));
        record.last_updated_at = format_iso_time(SystemTime::now());
        if let Some(storage) = &self.storage {
            if let Err(error) = storage.save(&guard) {
                if !error.is::<ReplacementDurabilityUnknown>() {
                    guard.insert(task_id.to_string(), previous);
                }
                return Err(error).context("failed to persist emission cancellation");
            }
        }
        Ok(EmissionCancelResult::Cancelled)
    }

    async fn persist_expirations(
        &self,
        tasks: &HashMap<String, TaskRecord>,
        expired_ids: &[String],
    ) {
        if expired_ids.is_empty() {
            return;
        }
        let mut channels = self.wait_channels.write().await;
        for id in expired_ids {
            channels.remove(id);
        }
        if let Some(storage) = &self.storage {
            storage.save_logged(tasks, "failed to persist expired tasks");
        }
    }

    /// Retrieves a task by its task ID, checking for TTL expiry.
    pub async fn get_task(&self, task_id: &str) -> Option<TaskRecord> {
        let mut tasks = self.tasks.write().await;
        let record = tasks.get_mut(task_id)?;
        let expired = expire_task(record, SystemTime::now());
        let record = record.clone();
        if expired {
            self.persist_expirations(&tasks, std::slice::from_ref(&record.task_id))
                .await;
        }
        Some(record)
    }

    /// Finds an active or completed task by idempotency key and checks for expiry.
    pub async fn find_by_idempotency_key(&self, key: &str) -> Option<TaskRecord> {
        let mut tasks = self.tasks.write().await;
        let record = tasks
            .values_mut()
            .find(|task| task.idempotency_key.as_deref() == Some(key))?;
        let expired = expire_task(record, SystemTime::now());
        let record = record.clone();
        if expired {
            self.persist_expirations(&tasks, std::slice::from_ref(&record.task_id))
                .await;
        }
        Some(record)
    }

    /// Lists all tasks after applying TTL expiry.
    pub async fn list_tasks(&self) -> Vec<TaskRecord> {
        let mut tasks = self.tasks.write().await;
        let now = SystemTime::now();
        let mut expired = Vec::new();
        let mut list = Vec::with_capacity(tasks.len());
        for record in tasks.values_mut() {
            if expire_task(record, now) {
                expired.push(record.task_id.clone());
            }
            list.push(record.clone());
        }
        self.persist_expirations(&tasks, &expired).await;
        drop(tasks);
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

        if expire_task(record, SystemTime::now()) {
            self.persist_expirations(&guard, &[task_id.to_string()])
                .await;
            return Ok(false);
        }

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

        drop(channels);
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

        if expire_task(record, SystemTime::now()) {
            self.persist_expirations(&guard, &[task_id.to_string()])
                .await;
            return Ok(false);
        }

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

        drop(channels);
        drop(guard);
        self.sync_to_disk().await;
        Ok(true)
    }

    /// Records a completion attempt and returns the resulting task state.
    ///
    /// An expired task is returned as failed, an existing terminal state is
    /// preserved, and a missing task returns `None`.
    ///
    /// # Errors
    /// Returns an error if a new completed state could not replace the stored state.
    pub async fn complete_task(&self, task_id: &str, result: Value) -> Result<Option<TaskRecord>> {
        self.store_completed_task(task_id, result, None).await
    }

    async fn store_completed_task(
        &self,
        task_id: &str,
        result: Value,
        revision: Option<String>,
    ) -> Result<Option<TaskRecord>> {
        let mut guard = self.tasks.write().await;
        let Some(record) = guard.get_mut(task_id) else {
            return Ok(None);
        };
        if expire_task(record, SystemTime::now()) {
            let record = record.clone();
            self.persist_expirations(&guard, std::slice::from_ref(&record.task_id))
                .await;
            return Ok(Some(record));
        }
        if matches!(
            record.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            return Ok(Some(record.clone()));
        }
        let previous = record.clone();
        if let Some(emission) = record.emission.as_mut() {
            emission.cancellable = false;
        }
        record.status = TaskStatus::Completed;
        record.status_message = Some("Execution completed successfully".to_string());
        record.result = Some(result);
        record.result_policy_revision = revision;
        record.last_updated_at = format_iso_time(SystemTime::now());
        let completed = record.clone();
        if let Some(storage) = &self.storage {
            if let Err(error) = storage.save(&guard) {
                if !error.is::<ReplacementDurabilityUnknown>() {
                    guard.insert(task_id.to_string(), previous);
                    return Err(error).context("failed to persist completed task");
                }
                tracing::warn!(task_id, error = %error, "task completion is visible with uncertain durability");
            }
        }

        self.wait_channels.write().await.remove(task_id);
        Ok(Some(completed))
    }

    pub(crate) async fn complete_execution(
        &self,
        task_id: &str,
        upstream_result: Value,
        revision: Option<String>,
    ) -> std::result::Result<Value, WarmplaneError> {
        let record = match self
            .store_completed_task(task_id, upstream_result, revision)
            .await
        {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Err(WarmplaneError::new(
                    "TASK_NOT_FOUND",
                    "Task no longer exists",
                    false,
                ))
            }
            Err(error) => {
                tracing::error!(task_id, %error, "failed to store task result");
                return Err(WarmplaneError::new(
                    "TASK_COMPLETION_FAILED",
                    "Could not store the completed task result",
                    false,
                ));
            }
        };
        if record.status == TaskStatus::Completed {
            return Ok(record.result.unwrap_or(Value::Null));
        }

        let fallback_code = match record.status {
            TaskStatus::Cancelled => "TASK_CANCELLED",
            _ => "TASK_FAILED",
        };
        let code = record
            .error
            .as_ref()
            .and_then(|error| error.get("code"))
            .and_then(Value::as_str)
            .unwrap_or(fallback_code);
        let message = record
            .status_message
            .unwrap_or_else(|| "Task did not complete".to_string());
        Err(WarmplaneError::new(code, message, false))
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
            if matches!(
                record.status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ) {
                return Ok(());
            }
            let previous = record.clone();
            if let Some(emission) = record.emission.as_mut() {
                emission.cancellable = false;
            }
            record.status = TaskStatus::Failed;
            record.status_message = message.or_else(|| Some("Execution failed".to_string()));
            record.error = Some(error);
            record.last_updated_at = format_iso_time(SystemTime::now());
            if let Some(storage) = &self.storage {
                if let Err(error) = storage.save(&guard) {
                    if !error.is::<ReplacementDurabilityUnknown>() {
                        guard.insert(task_id.to_string(), previous);
                    } else {
                        self.wait_channels.write().await.remove(task_id);
                    }
                    return Err(error).context("failed to persist failed task");
                }
            }
        }

        let mut channels = self.wait_channels.write().await;
        channels.remove(task_id);

        drop(guard);
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

fn new_task_record(
    task_id: String,
    params: CreateTaskParams,
    now_secs: u64,
    iso_time: String,
    emission: Option<EmissionRecord>,
) -> TaskRecord {
    TaskRecord {
        task_id,
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
        result_policy_revision: None,
        result_access: None,
        error: None,
        created_at: iso_time.clone(),
        last_updated_at: iso_time,
        created_at_epoch_secs: now_secs,
        ttl_ms: params.ttl_ms,
        poll_interval_ms: params.poll_interval_ms.or(Some(1_000)),
        emission,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn expired_tasks_reject_input_release_waiters_and_persist() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tasks.json");
        let registry = TaskRegistry::open_or_create(&path).unwrap();
        let params = CreateTaskParams {
            capability_id: "db.query".to_string(),
            server_id: "sqlite".to_string(),
            args: json!({}),
            request_id: None,
            context: None,
            idempotency_key: None,
            initial_status: TaskStatus::InputRequired,
            status_message: None,
            input_requests: None,
            ttl_ms: Some(0),
            poll_interval_ms: None,
        };
        let (list_task, list_waiter) = registry.create_task(params.clone()).await;
        let (update_task, update_waiter) = registry.create_task(params).await;

        assert!(!registry
            .update_task(&update_task.task_id, BTreeMap::new())
            .await
            .unwrap());
        assert!(matches!(
            list_waiter.unwrap().try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ));
        assert!(matches!(
            update_waiter.unwrap().try_recv(),
            Err(oneshot::error::TryRecvError::Closed)
        ));

        let tasks = registry.list_tasks().await;
        assert!(tasks.iter().all(|task| task.status == TaskStatus::Failed));
        assert!(registry.wait_channels.read().await.is_empty());
        let saved: HashMap<String, TaskRecord> =
            AtomicFile::new(&path).load_opt().unwrap().unwrap();
        assert_eq!(saved[&list_task.task_id].status, TaskStatus::Failed);
        assert_eq!(saved[&update_task.task_id].status, TaskStatus::Failed);
    }

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

    #[tokio::test]
    async fn emission_cancellation_wins_before_dispatch_and_is_idempotent() {
        let registry = TaskRegistry::new();
        let record = registry
            .create_emission_task(
                CreateTaskParams {
                    capability_id: "mail.send".to_string(),
                    server_id: "mail".to_string(),
                    args: json!({"to": "person@example.com"}),
                    request_id: Some("request-emission-1".to_string()),
                    context: None,
                    idempotency_key: Some("emission-key-1".to_string()),
                    initial_status: TaskStatus::Working,
                    status_message: Some("Waiting to dispatch".to_string()),
                    input_requests: None,
                    ttl_ms: None,
                    poll_interval_ms: Some(100),
                },
                EmissionRecord {
                    idempotency_claim: None,
                    phase: EmissionPhase::PendingEmission,
                    release_at_ms: 2_000,
                    cancellable: true,
                    tool_name: "send_message".to_string(),
                    input_responses: None,
                    request_state: None,
                    profile_id: None,
                    tenant_id: "tenant-a".to_string(),
                    role: "operator".to_string(),
                    authorization_expires_at_epoch_secs: None,
                    approval_completed: false,
                    batch_id: None,
                    step_id: None,
                },
            )
            .await
            .unwrap();

        let receipt = TaskResponse::from(&record);
        assert_eq!(receipt.request_id.as_deref(), Some("request-emission-1"));
        assert_eq!(receipt.phase, Some(EmissionPhase::PendingEmission));
        assert_eq!(receipt.release_at_ms, Some(2_000));
        assert_eq!(receipt.cancellable, Some(true));
        assert_eq!(
            registry
                .cancel_emission_task(&record.task_id, None)
                .await
                .unwrap(),
            EmissionCancelResult::Cancelled
        );
        assert_eq!(
            registry
                .cancel_emission_task(&record.task_id, None)
                .await
                .unwrap(),
            EmissionCancelResult::AlreadyCancelled
        );
        assert_eq!(
            registry.get_task(&record.task_id).await.unwrap().status,
            TaskStatus::Cancelled
        );
        assert!(registry
            .claim_emission_dispatch(&record.task_id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn dispatch_claim_makes_cancellation_too_late_and_terminal_status_sticks() {
        let registry = TaskRegistry::new();
        let record = registry
            .create_emission_task(
                CreateTaskParams {
                    capability_id: "mail.send".to_string(),
                    server_id: "mail".to_string(),
                    args: json!({}),
                    request_id: None,
                    context: None,
                    idempotency_key: None,
                    initial_status: TaskStatus::Working,
                    status_message: None,
                    input_requests: None,
                    ttl_ms: None,
                    poll_interval_ms: None,
                },
                EmissionRecord {
                    idempotency_claim: None,
                    phase: EmissionPhase::PendingEmission,
                    release_at_ms: 2_000,
                    cancellable: true,
                    tool_name: "send_message".to_string(),
                    input_responses: None,
                    request_state: None,
                    profile_id: None,
                    tenant_id: "tenant-a".to_string(),
                    role: "operator".to_string(),
                    authorization_expires_at_epoch_secs: None,
                    approval_completed: false,
                    batch_id: None,
                    step_id: None,
                },
            )
            .await
            .unwrap();

        let claimed = registry
            .claim_emission_dispatch(&record.task_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(claimed.emission.unwrap().phase, EmissionPhase::Dispatching);
        assert_eq!(
            registry
                .cancel_emission_task(&record.task_id, None)
                .await
                .unwrap(),
            EmissionCancelResult::TooLate
        );
        registry
            .cancel_task(&record.task_id, Some("legacy cancellation".to_string()))
            .await
            .unwrap();
        registry
            .complete_task(&record.task_id, json!({"sent": true}))
            .await
            .unwrap();
        assert_eq!(
            registry.get_task(&record.task_id).await.unwrap().status,
            TaskStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn emission_phase_transitions_survive_registry_reopen() {
        check_emission_recovery(false).await;
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn visible_emission_is_accepted_after_uncertain_directory_sync() {
        check_emission_recovery(true).await;
    }

    async fn check_emission_recovery(uncertain: bool) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tasks.json");
        let registry = TaskRegistry::open_or_create(&path).unwrap();
        #[cfg(unix)]
        if uncertain {
            registry
                .storage
                .as_ref()
                .unwrap()
                .fail_next_directory_sync();
        }
        let record = registry
            .create_emission_task(
                CreateTaskParams {
                    capability_id: "mail.send".to_string(),
                    server_id: "mail".to_string(),
                    args: json!({"body": "private"}),
                    request_id: Some("request-persisted".to_string()),
                    context: None,
                    idempotency_key: None,
                    initial_status: TaskStatus::Working,
                    status_message: None,
                    input_requests: None,
                    ttl_ms: None,
                    poll_interval_ms: None,
                },
                EmissionRecord {
                    idempotency_claim: None,
                    phase: EmissionPhase::PendingEmission,
                    release_at_ms: 4_000,
                    cancellable: true,
                    tool_name: "send_message".to_string(),
                    input_responses: None,
                    request_state: None,
                    profile_id: Some("mail-profile".to_string()),
                    tenant_id: "tenant-a".to_string(),
                    role: "operator".to_string(),
                    authorization_expires_at_epoch_secs: None,
                    approval_completed: false,
                    batch_id: Some("batch-1".to_string()),
                    step_id: Some("send".to_string()),
                },
            )
            .await
            .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        if uncertain {
            assert!(record
                .status_message
                .as_deref()
                .unwrap()
                .contains("uncertain"));
        }
        let reopened = TaskRegistry::open_or_create(&path).unwrap();
        let claimed = reopened
            .claim_emission_dispatch(&record.task_id)
            .await
            .unwrap();
        assert!(claimed.is_some());
        let reopened = TaskRegistry::open_or_create(&path).unwrap();
        let loaded = reopened.get_task(&record.task_id).await.unwrap();
        assert_eq!(loaded.emission.unwrap().phase, EmissionPhase::Dispatching);
        assert_eq!(loaded.args, json!({"body": "private"}));
    }
}
