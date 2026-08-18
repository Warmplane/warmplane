// Rust guideline compliant 2026-08-18

//! Model Context Protocol (MCP) Sampling Delegation Engine & Registry.
//!
//! Provides client-delegated LLM completion request suspension, ticketing, parameter tracking,
//! timeout expiration, and persistent atomic state storage (`M-CANONICAL-DOCS`).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{oneshot, RwLock};
use tracing::{info, warn};

use crate::storage::AtomicFile;

/// Content element within a sampling message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SamplingContent {
    /// Plain text content.
    Text { text: String },
    /// Image content (e.g. base64 png/jpeg).
    Image { data: String, mime_type: String },
}

/// A message exchanged in a sampling request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SamplingMessage {
    /// Role identifier (`"user"` or `"assistant"`).
    pub role: String,
    /// Message content payload.
    pub content: SamplingContent,
}

/// Model selection hint and priority preferences.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct ModelPreferences {
    /// Model name hints (e.g. `["claude-3-5-sonnet", "gpt-4o"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
    /// Priority for low cost (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f32>,
    /// Priority for low latency / high speed (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f32>,
    /// Priority for intelligence / capabilities (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f32>,
}

/// Parameters for creating a sampling completion message (`sampling/createMessage`).
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct CreateMessageParams {
    /// Target server identifier.
    #[serde(default)]
    pub server_id: String,
    /// Conversation messages history.
    #[serde(default)]
    pub messages: Vec<SamplingMessage>,
    /// Optional model selection hints and priorities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    /// Optional system instruction prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Optional context inclusion mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_context: Option<String>,
    /// Maximum tokens to generate in assistant completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    /// Stop sequences terminating output generation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    /// Optional metadata map passed by caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Assistant completion result payload returned by client.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageResult {
    /// Role identifier (`"assistant"`).
    pub role: String,
    /// Generated message content.
    pub content: SamplingContent,
    /// Model identifier that produced the completion.
    pub model: String,
    /// Optional stop reason (`"endTurn"`, `"stopSequence"`, `"maxTokens"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Lifecycle status of a sampling ticket.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SamplingRequestStatus {
    /// Awaiting client/agent completion response.
    Pending,
    /// Completed successfully with assistant completion response.
    Completed {
        result: CreateMessageResult,
        timestamp: u64,
    },
    /// Rejected by operator or downstream client.
    Rejected {
        reason: Option<String>,
        timestamp: u64,
    },
    /// Expired due to execution timeout.
    Expired { timestamp: u64 },
    /// Failed with processing error.
    Failed { error: String, timestamp: u64 },
}

/// In-memory and persistent record of a tracked sampling request ticket.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingSamplingRequest {
    /// Unique sampling ticket ID (e.g. `samp_1723668200_1`).
    pub id: String,
    /// Originating server or caller identifier.
    pub server_id: String,
    /// Request parameters payload.
    pub params: CreateMessageParams,
    /// Creation timestamp in epoch seconds.
    pub created_at: u64,
    /// Expiration timestamp in epoch seconds.
    pub expires_at: u64,
    /// Ticket lifecycle status.
    #[serde(flatten)]
    pub status: SamplingRequestStatus,
}

/// Type alias for sampling completion sender channel.
pub type SamplingWaitSender = oneshot::Sender<Result<CreateMessageResult, String>>;

/// Central registry managing pending sampling tickets and asynchronous wait channels.
#[derive(Clone, Default)]
pub struct SamplingRegistry {
    /// Tracked sampling requests keyed by ticket ID.
    pub requests: Arc<RwLock<HashMap<String, PendingSamplingRequest>>>,
    /// Suspended wait channels for asynchronous caller notifications.
    pub wait_channels: Arc<RwLock<HashMap<String, SamplingWaitSender>>>,
    /// Optional atomic persistent storage on disk.
    pub storage: Option<AtomicFile<HashMap<String, PendingSamplingRequest>>>,
}

impl SamplingRegistry {
    /// Creates a new in-memory `SamplingRegistry`.
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            wait_channels: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
        }
    }

    /// Initializes a `SamplingRegistry` backed by atomic persistent JSON storage.
    ///
    /// # Arguments
    /// * `path` - Destination file path for `sampling.json`.
    ///
    /// # Errors
    /// Returns an error if reading or parsing existing storage fails.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let loaded: HashMap<String, PendingSamplingRequest> =
            storage.load_opt()?.unwrap_or_default();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut requests = loaded;
        for req in requests.values_mut() {
            if matches!(req.status, SamplingRequestStatus::Pending) && now >= req.expires_at {
                req.status = SamplingRequestStatus::Expired { timestamp: now };
            }
        }

        let _ = storage.save(&requests);

        Ok(Self {
            requests: Arc::new(RwLock::new(requests)),
            wait_channels: Arc::new(RwLock::new(HashMap::new())),
            storage: Some(storage),
        })
    }

    /// Creates and registers a new pending sampling ticket.
    ///
    /// # Arguments
    /// * `server_id` - Originating server ID.
    /// * `params` - Sampling request parameters.
    /// * `timeout_secs` - Timeout duration before expiring the ticket.
    ///
    /// # Returns
    /// Pair of ticket ID and oneshot receiver channel for awaiting resolution.
    pub async fn create_request(
        &self,
        server_id: String,
        params: CreateMessageParams,
        timeout_secs: u64,
    ) -> (
        String,
        oneshot::Receiver<Result<CreateMessageResult, String>>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let expires_at = now + timeout_secs.max(5);
        let id = format!("samp_{}_{}", now, rand::random::<u32>());

        let ticket = PendingSamplingRequest {
            id: id.clone(),
            server_id,
            params,
            created_at: now,
            expires_at,
            status: SamplingRequestStatus::Pending,
        };

        let (tx, rx) = oneshot::channel();

        {
            let mut req_guard = self.requests.write().await;
            req_guard.insert(id.clone(), ticket);
            if let Some(storage) = &self.storage {
                let _ = storage.save(&*req_guard);
            }
        }

        {
            let mut chan_guard = self.wait_channels.write().await;
            chan_guard.insert(id.clone(), tx);
        }

        info!(ticket_id = %id, timeout_secs = timeout_secs, "created pending MCP sampling delegation ticket");

        // Spawn timeout reaper task
        let self_clone = self.clone();
        let ticket_id_clone = id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(timeout_secs.max(5))).await;
            self_clone.expire_ticket(&ticket_id_clone).await;
        });

        (id, rx)
    }

    /// Resolves a pending sampling ticket with a successful LLM completion.
    ///
    /// # Arguments
    /// * `id` - Ticket ID.
    /// * `result` - Completion result.
    ///
    /// # Returns
    /// `true` if ticket was found and resolved, `false` otherwise.
    pub async fn respond_to_request(&self, id: &str, result: CreateMessageResult) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut req_guard = self.requests.write().await;
        if let Some(ticket) = req_guard.get_mut(id) {
            if matches!(ticket.status, SamplingRequestStatus::Pending) {
                ticket.status = SamplingRequestStatus::Completed {
                    result: result.clone(),
                    timestamp: now,
                };

                if let Some(storage) = &self.storage {
                    let _ = storage.save(&*req_guard);
                }

                let mut chan_guard = self.wait_channels.write().await;
                if let Some(tx) = chan_guard.remove(id) {
                    let _ = tx.send(Ok(result));
                }

                info!(ticket_id = %id, "resolved MCP sampling ticket with client completion");
                return true;
            }
        }

        false
    }

    /// Rejects a pending sampling ticket with a given reason.
    pub async fn reject_request(&self, id: &str, reason: Option<String>) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut req_guard = self.requests.write().await;
        if let Some(ticket) = req_guard.get_mut(id) {
            if matches!(ticket.status, SamplingRequestStatus::Pending) {
                ticket.status = SamplingRequestStatus::Rejected {
                    reason: reason.clone(),
                    timestamp: now,
                };

                if let Some(storage) = &self.storage {
                    let _ = storage.save(&*req_guard);
                }

                let mut chan_guard = self.wait_channels.write().await;
                if let Some(tx) = chan_guard.remove(id) {
                    let _ = tx.send(Err(
                        reason.unwrap_or_else(|| "Sampling request rejected".to_string())
                    ));
                }

                info!(ticket_id = %id, "rejected MCP sampling ticket");
                return true;
            }
        }

        false
    }

    /// Expires an overdue pending sampling ticket.
    pub async fn expire_ticket(&self, id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut req_guard = self.requests.write().await;
        if let Some(ticket) = req_guard.get_mut(id) {
            if matches!(ticket.status, SamplingRequestStatus::Pending) {
                ticket.status = SamplingRequestStatus::Expired { timestamp: now };

                if let Some(storage) = &self.storage {
                    let _ = storage.save(&*req_guard);
                }

                let mut chan_guard = self.wait_channels.write().await;
                if let Some(tx) = chan_guard.remove(id) {
                    let _ = tx.send(Err(
                        "Sampling request timed out awaiting client completion".to_string()
                    ));
                }

                warn!(ticket_id = %id, "expired pending MCP sampling ticket");
            }
        }
    }

    /// Lists all tracked sampling tickets sorted by creation time descending.
    pub async fn list_requests(
        &self,
        server_id: Option<&str>,
        status_filter: Option<&str>,
    ) -> Vec<PendingSamplingRequest> {
        let req_guard = self.requests.read().await;
        let mut list: Vec<PendingSamplingRequest> = req_guard
            .values()
            .filter(|req| {
                if let Some(s) = server_id {
                    if req.server_id != s {
                        return false;
                    }
                }
                if let Some(sf) = status_filter {
                    let matches = matches!(
                        (&req.status, sf),
                        (SamplingRequestStatus::Pending, "pending")
                            | (SamplingRequestStatus::Completed { .. }, "completed")
                            | (SamplingRequestStatus::Rejected { .. }, "rejected")
                            | (SamplingRequestStatus::Expired { .. }, "expired")
                            | (SamplingRequestStatus::Failed { .. }, "failed")
                    );
                    if !matches {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    /// Retrieves a single sampling ticket by ID.
    pub async fn get_request(&self, id: &str) -> Option<PendingSamplingRequest> {
        let req_guard = self.requests.read().await;
        req_guard.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sampling_ticket_creation_and_completion() {
        let registry = SamplingRegistry::new();
        let params = CreateMessageParams {
            server_id: "test-server".to_string(),
            messages: vec![SamplingMessage {
                role: "user".to_string(),
                content: SamplingContent::Text {
                    text: "Summarize this data".to_string(),
                },
            }],
            ..Default::default()
        };

        let (ticket_id, rx) = registry
            .create_request("test-server".to_string(), params, 10)
            .await;

        let res_payload = CreateMessageResult {
            role: "assistant".to_string(),
            content: SamplingContent::Text {
                text: "Here is the summary.".to_string(),
            },
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: Some("endTurn".to_string()),
        };

        let resolved = registry
            .respond_to_request(&ticket_id, res_payload.clone())
            .await;
        assert!(resolved);

        let completion_res = rx.await.unwrap().unwrap();
        assert_eq!(completion_res, res_payload);

        let ticket = registry.get_request(&ticket_id).await.unwrap();
        assert!(matches!(
            ticket.status,
            SamplingRequestStatus::Completed { .. }
        ));
    }

    #[tokio::test]
    async fn test_sampling_persistence_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("sampling.json");

        {
            let registry = SamplingRegistry::open_or_create(&file_path).unwrap();
            let params = CreateMessageParams {
                server_id: "srv1".to_string(),
                messages: vec![],
                ..Default::default()
            };
            let (ticket_id, _) = registry
                .create_request("srv1".to_string(), params, 60)
                .await;
            assert!(registry.get_request(&ticket_id).await.is_some());
        }

        // Reopen from disk
        {
            let reopened = SamplingRegistry::open_or_create(&file_path).unwrap();
            let all = reopened.list_requests(None, None).await;
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].server_id, "srv1");
        }
    }
}
