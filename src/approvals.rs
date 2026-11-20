// Rust guideline compliant 2026-11-18

//! Human-in-the-Loop (HITL) Approval Engine and Registry.
//!
//! Provides non-blocking suspension, parameter modification, atomic state transitions,
//! timeout expiration, and signed webhook notifications for sensitive capability execution.

use anyhow::Result;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{oneshot, RwLock};
use tracing::{error, info, warn};

use crate::config::WebhookConfig;
use crate::context::RequestContext;
use crate::storage::AtomicFile;

type HmacSha256 = Hmac<Sha256>;

/// Status of an approval ticket.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Awaiting human operator decision.
    Pending,
    /// Execution approved by operator (optionally with modified arguments).
    Approved {
        operator: String,
        timestamp: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        modified_args: Option<serde_json::Value>,
    },
    /// Execution rejected by operator.
    Rejected {
        operator: String,
        reason: Option<String>,
        timestamp: u64,
    },
    /// Ticket expired due to approval timeout.
    Expired { timestamp: u64 },
}

/// In-memory record of an approval ticket.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingApproval {
    /// Unique ticket identifier (e.g. `appr-1723668200-1`).
    pub id: String,
    /// Target capability ID.
    pub capability_id: String,
    /// Target upstream server ID.
    pub server_id: String,
    /// Raw invocation arguments.
    pub args: serde_json::Value,
    /// Sanitized arguments (PII/secrets redacted).
    pub sanitized_args: serde_json::Value,
    /// Optional incoming request identifier.
    pub request_id: Option<String>,
    /// Optional request context metadata.
    pub context: Option<RequestContext>,
    /// Creation timestamp in epoch seconds.
    pub created_at: u64,
    /// Expiration timestamp in epoch seconds.
    pub expires_at: u64,
    /// Active status.
    #[serde(flatten)]
    pub status: ApprovalStatus,
}

/// Resolution returned through the internal wait channel to the suspended caller.
#[derive(Clone, Debug)]
pub enum ApprovalResolution {
    /// Approved with optional operator-modified arguments.
    Approved {
        operator: String,
        modified_args: Option<serde_json::Value>,
    },
    /// Rejected with operator identifier and rationale.
    Rejected {
        operator: String,
        reason: Option<String>,
    },
    /// Timed out while waiting.
    Expired,
}

/// Global registry managing pending human approvals and active wait channels.
#[derive(Clone, Default)]
pub struct ApprovalRegistry {
    pub pending: Arc<RwLock<HashMap<String, PendingApproval>>>,
    pub wait_channels: Arc<RwLock<HashMap<String, oneshot::Sender<ApprovalResolution>>>>,
    pub storage: Option<AtomicFile<HashMap<String, PendingApproval>>>,
}

/// Parameters for creating a pending approval ticket.
#[derive(Clone, Debug)]
pub struct CreateApprovalRequest<'a> {
    /// Capability identifier.
    pub capability_id: String,
    /// Upstream server identifier.
    pub server_id: String,
    /// Raw invocation arguments.
    pub args: serde_json::Value,
    /// Sanitized arguments.
    pub sanitized_args: serde_json::Value,
    /// Optional incoming request ID.
    pub request_id: Option<String>,
    /// Optional request context metadata.
    pub context: Option<RequestContext>,
    /// Timeout duration in seconds.
    pub timeout_secs: u64,
    /// Optional webhook configuration.
    pub webhook: Option<&'a WebhookConfig>,
}

impl ApprovalRegistry {
    /// Creates a ticket and its interception audit event without cloning raw arguments.
    ///
    /// The caller supplies transport-specific audit fields and submits the returned event.
    pub(crate) async fn create_audited_approval(
        &self,
        request: CreateApprovalRequest<'_>,
        trace_id: String,
        started: std::time::Instant,
    ) -> (
        String,
        oneshot::Receiver<ApprovalResolution>,
        crate::audit::RawAuditEvent,
    ) {
        let mut event = request.audit_event(trace_id);
        let (id, receiver) = self.create_approval(request).await;
        event.approval_ticket_id = Some(id.clone());
        event.execution_latency_us = Some(started.elapsed().as_micros() as u64);
        (id, receiver, event)
    }

    /// Creates a new empty in-memory `ApprovalRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes an `ApprovalRegistry` backed by a persistent atomic JSON file.
    ///
    /// Loads existing tickets, expires overdue pending tickets, and reschedules
    /// timeout expiration tasks for any active pending tickets.
    ///
    /// # Arguments
    /// * `path` - Destination path for persistent approval records.
    ///
    /// # Errors
    /// Returns an error if reading or parsing existing storage fails.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let storage = AtomicFile::new(path);
        let mut loaded: HashMap<String, PendingApproval> = storage.load_opt()?.unwrap_or_default();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut expired_any = false;
        let mut active_pending_timeouts: Vec<(String, u64)> = Vec::new();

        for (id, ticket) in loaded.iter_mut() {
            if ticket.status == ApprovalStatus::Pending {
                if now >= ticket.expires_at {
                    ticket.status = ApprovalStatus::Expired { timestamp: now };
                    expired_any = true;
                } else {
                    let remaining = ticket.expires_at - now;
                    active_pending_timeouts.push((id.clone(), remaining));
                }
            }
        }

        if expired_any {
            let _ = storage.save(&loaded);
        }

        let registry = Self {
            pending: Arc::new(RwLock::new(loaded)),
            wait_channels: Arc::new(RwLock::new(HashMap::new())),
            storage: Some(storage),
        };

        // Reschedule timeout tasks for pending tickets
        for (ticket_id, remaining_secs) in active_pending_timeouts {
            let reg_clone = registry.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(remaining_secs)).await;
                reg_clone.expire_if_pending(&ticket_id).await;
            });
        }

        Ok(registry)
    }

    async fn sync_to_disk(&self) {
        if let Some(ref store) = self.storage {
            let guard = self.pending.read().await;
            store.save_logged(&*guard, "failed to persist approval registry state to disk");
        }
    }

    /// Registers a new pending approval ticket and returns its ID and wait receiver.
    pub async fn create_approval(
        &self,
        req: CreateApprovalRequest<'_>,
    ) -> (String, oneshot::Receiver<ApprovalResolution>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let expires_at = now + req.timeout_secs;
        let suffix: u32 = rand::random::<u32>() % 9000 + 1000;
        let id = format!("appr-{}-{}", now, suffix);

        let approval = PendingApproval {
            id: id.clone(),
            capability_id: req.capability_id.clone(),
            server_id: req.server_id,
            args: req.args,
            sanitized_args: req.sanitized_args,
            request_id: req.request_id,
            context: req.context,
            created_at: now,
            expires_at,
            status: ApprovalStatus::Pending,
        };

        let (tx, rx) = oneshot::channel();

        {
            let mut pending_guard = self.pending.write().await;
            if pending_guard.len() > 1000 {
                pending_guard.retain(|_, v| {
                    v.status == ApprovalStatus::Pending || now.saturating_sub(v.expires_at) < 3600
                });
            }
            pending_guard.insert(id.clone(), approval.clone());
        }
        {
            let mut chan_guard = self.wait_channels.write().await;
            chan_guard.insert(id.clone(), tx);
        }

        // Spawn timeout reaper task
        let self_clone = self.clone();
        let ticket_id = id.clone();
        let timeout_secs = req.timeout_secs;
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)).await;
            self_clone.expire_if_pending(&ticket_id).await;
        });

        // Persist ticket to disk
        self.sync_to_disk().await;

        // Dispatch outbound webhook if configured
        if let Some(cfg) = req.webhook {
            let webhook_cfg = cfg.clone();
            let approval_data = approval.clone();
            tokio::spawn(async move {
                dispatch_webhook(&webhook_cfg, "approval.requested", &approval_data).await;
            });
        }

        info!(approval_id = %id, capability_id = %req.capability_id, "created pending approval ticket");
        (id, rx)
    }

    /// Atomically approves a pending ticket, notifying the suspended caller.
    pub async fn approve(
        &self,
        id: &str,
        operator: String,
        modified_args: Option<serde_json::Value>,
        webhook: Option<&WebhookConfig>,
    ) -> Result<bool> {
        let timestamp = approval_timestamp();
        self.resolve_pending(
            id,
            ApprovalStatus::Approved {
                operator: operator.clone(),
                timestamp,
                modified_args: modified_args.clone(),
            },
            ApprovalResolution::Approved {
                operator,
                modified_args,
            },
            "approval.granted",
            webhook,
        )
        .await
    }

    /// Atomically rejects a pending ticket, notifying the suspended caller.
    pub async fn reject(
        &self,
        id: &str,
        operator: String,
        reason: Option<String>,
        webhook: Option<&WebhookConfig>,
    ) -> Result<bool> {
        let timestamp = approval_timestamp();
        self.resolve_pending(
            id,
            ApprovalStatus::Rejected {
                operator: operator.clone(),
                reason: reason.clone(),
                timestamp,
            },
            ApprovalResolution::Rejected { operator, reason },
            "approval.rejected",
            webhook,
        )
        .await
    }

    async fn resolve_pending(
        &self,
        id: &str,
        status: ApprovalStatus,
        resolution: ApprovalResolution,
        event: &'static str,
        webhook: Option<&WebhookConfig>,
    ) -> Result<bool> {
        let mut pending = self.pending.write().await;
        let Some(approval) = pending.get_mut(id) else {
            return Ok(false);
        };
        if approval.status != ApprovalStatus::Pending {
            return Ok(false);
        }
        approval.status = status;
        let snapshot = approval.clone();
        drop(pending);
        self.sync_to_disk().await;
        let mut channels = self.wait_channels.write().await;
        if let Some(sender) = channels.remove(id) {
            let _ = sender.send(resolution);
        }
        if let Some(config) = webhook {
            let config = config.clone();
            tokio::spawn(async move {
                dispatch_webhook(&config, event, &snapshot).await;
            });
        }
        info!(approval_id = %id, resolution = event, "approval resolved by operator");
        Ok(true)
    }

    /// Expires a ticket if still pending after the timeout.
    pub async fn expire_if_pending(&self, id: &str) {
        let mut pending_guard = self.pending.write().await;
        let Some(approval) = pending_guard.get_mut(id) else {
            return;
        };

        if approval.status != ApprovalStatus::Pending {
            return;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        approval.status = ApprovalStatus::Expired { timestamp: now };
        drop(pending_guard);

        self.sync_to_disk().await;

        let mut chan_guard = self.wait_channels.write().await;
        if let Some(tx) = chan_guard.remove(id) {
            let _ = tx.send(ApprovalResolution::Expired);
        }

        warn!(approval_id = %id, "approval ticket expired");
    }

    /// Lists all pending approvals and recent history.
    pub async fn list(&self) -> Vec<PendingApproval> {
        let pending_guard = self.pending.read().await;
        let mut list: Vec<_> = pending_guard.values().cloned().collect();
        list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        list
    }

    /// Gets a single approval ticket by ID.
    pub async fn get(&self, id: &str) -> Option<PendingApproval> {
        let pending_guard = self.pending.read().await;
        pending_guard.get(id).cloned()
    }

    /// Finds an active pending approval ticket matching a given request ID.
    pub async fn get_pending_by_request_id(&self, request_id: &str) -> Option<PendingApproval> {
        let pending_guard = self.pending.read().await;
        pending_guard
            .values()
            .find(|a| {
                a.status == ApprovalStatus::Pending && a.request_id.as_deref() == Some(request_id)
            })
            .cloned()
    }
}

/// Dispatches an outbound webhook event with HMAC-SHA256 signature and bearer headers.
pub async fn dispatch_webhook(cfg: &WebhookConfig, event_type: &str, approval: &PendingApproval) {
    // Check if event is subscribed to
    if !cfg.events.is_empty() && !cfg.events.iter().any(|e| e == event_type) {
        return;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let sanitized_approval = serde_json::json!({
        "id": approval.id,
        "capability_id": approval.capability_id,
        "server_id": approval.server_id,
        "args": approval.sanitized_args,
        "request_id": approval.request_id,
        "context": approval.context,
        "created_at": approval.created_at,
        "expires_at": approval.expires_at,
        "status": approval.status,
    });

    let format = cfg.format.unwrap_or_default();
    let payload = crate::chatops::format_webhook_payload(
        format,
        event_type,
        &sanitized_approval,
        cfg.callback_url.as_deref(),
    );

    let payload_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "failed to serialize webhook payload");
            return;
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "failed to build HTTP client for webhook");
            return;
        }
    };

    let mut req = client
        .post(&cfg.url)
        .header("Content-Type", "application/json")
        .header("X-Warmplane-Timestamp", now.to_string())
        .header("X-Warmplane-Event", event_type);

    if let Some(auth) = &cfg.auth_header {
        req = req.header("Authorization", auth);
    }

    if let Some(headers) = &cfg.headers {
        for (k, v) in headers {
            req = req.header(k, v);
        }
    }

    if let Some(secret) = cfg.resolve_secret() {
        if let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
            let sign_target = format!("{}.{}", now, payload_str);
            mac.update(sign_target.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            req = req.header("X-Warmplane-Signature", format!("sha256={}", signature));
        }
    }

    match req.body(payload_str).send().await {
        Ok(resp) if resp.status().is_success() => {
            info!(url = %cfg.url, event = %event_type, "dispatched webhook event successfully");
        }
        Ok(resp) => {
            warn!(url = %cfg.url, status = %resp.status(), event = %event_type, "webhook returned non-success response");
        }
        Err(e) => {
            error!(url = %cfg.url, error = %e, event = %event_type, "failed to dispatch webhook event");
        }
    }
}

/// Metadata needed to issue a signed, single-target ChatOps action card.
pub(crate) struct SignedActionDispatch {
    pub(crate) event_type: &'static str,
    pub(crate) action: &'static str,
    pub(crate) target_id: String,
    pub(crate) tenant_id: String,
    pub(crate) expires_at_ms: u64,
    pub(crate) capability: String,
    pub(crate) actor: Option<String>,
    pub(crate) profile_id: Option<String>,
}

/// Schedules a signed, single-target ChatOps action card without blocking tool execution.
pub(crate) fn dispatch_signed_action(
    state: crate::daemon::AppState,
    dispatch: SignedActionDispatch,
) {
    tokio::spawn(async move {
        let config = match crate::config::load_or_default_config(&state.config_path) {
            Ok(config) => config,
            Err(error) => {
                warn!(error = %error, "failed to load ChatOps action configuration");
                return;
            }
        };
        let Some(webhook) = config.policy.and_then(|policy| policy.webhook) else {
            return;
        };
        if !webhook
            .events
            .iter()
            .any(|event| event == dispatch.event_type)
        {
            return;
        }
        let (Some(secret), Some(callback_url), Some(role)) = (
            webhook.resolve_secret(),
            webhook.callback_url.as_deref(),
            webhook
                .action_role
                .as_deref()
                .filter(|role| !role.trim().is_empty()),
        ) else {
            warn!(
                dispatch.event_type,
                "ChatOps action event requires secret, callbackUrl, and actionRole"
            );
            return;
        };
        let Some(action_token) = crate::chatops::sign_action_token(
            &secret,
            dispatch.action,
            &dispatch.target_id,
            &dispatch.tenant_id,
            role,
            dispatch.expires_at_ms,
        ) else {
            return;
        };
        let data = serde_json::json!({
            "target_id": dispatch.target_id,
            "capability": dispatch.capability,
            "actor": dispatch.actor,
            "profile": dispatch.profile_id,
            "action_token": action_token,
        });
        let payload = crate::chatops::format_webhook_payload(
            webhook.format.unwrap_or_default(),
            dispatch.event_type,
            &data,
            Some(callback_url),
        );
        if let Err(error) =
            crate::chatops::post_signed_webhook(&webhook, dispatch.event_type, &payload, &secret)
                .await
        {
            warn!(%error, event_type = dispatch.event_type, "ChatOps action notification failed");
        }
    });
}

fn approval_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

impl CreateApprovalRequest<'_> {
    /// Builds the interception audit event using the same request context as the approval.
    fn audit_event(&self, trace_id: String) -> crate::audit::RawAuditEvent {
        crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolInterceptedHitl,
            trace_id,
            request_id: self.request_id.clone(),
            actor_id: self
                .context
                .as_ref()
                .and_then(|context| context.actor_id.clone()),
            work_item_id: self
                .context
                .as_ref()
                .and_then(|context| context.work_item_id.clone()),
            client_ip: None,
            server_id: Some(self.server_id.clone()),
            capability_id: Some(self.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(self.sanitized_args.clone()),
            sanitized_response: None,
            execution_latency_us: None,
            status: crate::audit::AuditEventStatus::Intercepted,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_approval_lifecycle_approve() {
        let registry = ApprovalRegistry::new();
        let (id, rx) = registry
            .create_approval(CreateApprovalRequest {
                capability_id: "db.delete_table".to_string(),
                server_id: "postgres".to_string(),
                args: json!({"table": "users"}),
                sanitized_args: json!({"table": "users"}),
                request_id: Some("req-100".to_string()),
                context: None,
                timeout_secs: 10,
                webhook: None,
            })
            .await;

        let ticket = registry.get(&id).await.expect("ticket must exist");
        assert_eq!(ticket.status, ApprovalStatus::Pending);

        let approved = registry
            .approve(&id, "operator-alice".to_string(), None, None)
            .await
            .expect("approve should succeed");
        assert!(approved);

        let resolution = rx.await.expect("receiver should get resolution");
        match resolution {
            ApprovalResolution::Approved {
                operator,
                modified_args,
            } => {
                assert_eq!(operator, "operator-alice");
                assert!(modified_args.is_none());
            }
            _ => panic!("Expected Approved resolution"),
        }

        let ticket_after = registry.get(&id).await.expect("ticket must exist");
        match ticket_after.status {
            ApprovalStatus::Approved { operator, .. } => {
                assert_eq!(operator, "operator-alice");
            }
            _ => panic!("Expected Approved status"),
        }

        // Second approval attempt should fail atomically
        let second = registry
            .approve(&id, "operator-bob".to_string(), None, None)
            .await
            .expect("second action result");
        assert!(!second);
    }

    #[tokio::test]
    async fn test_approval_lifecycle_approve_with_modified_args() {
        let registry = ApprovalRegistry::new();
        let (id, rx) = registry
            .create_approval(CreateApprovalRequest {
                capability_id: "fs.delete_file".to_string(),
                server_id: "local_fs".to_string(),
                args: json!({"path": "/etc/passwd"}),
                sanitized_args: json!({"path": "/etc/passwd"}),
                request_id: Some("req-101".to_string()),
                context: None,
                timeout_secs: 10,
                webhook: None,
            })
            .await;

        let modified = json!({"path": "/tmp/test.txt"});
        let approved = registry
            .approve(
                &id,
                "security-admin".to_string(),
                Some(modified.clone()),
                None,
            )
            .await
            .expect("approve should succeed");
        assert!(approved);

        let resolution = rx.await.expect("receiver should get resolution");
        match resolution {
            ApprovalResolution::Approved {
                operator,
                modified_args,
            } => {
                assert_eq!(operator, "security-admin");
                assert_eq!(modified_args, Some(modified));
            }
            _ => panic!("Expected Approved resolution"),
        }
    }

    #[tokio::test]
    async fn test_approval_lifecycle_reject() {
        let registry = ApprovalRegistry::new();
        let (id, rx) = registry
            .create_approval(CreateApprovalRequest {
                capability_id: "k8s.delete_namespace".to_string(),
                server_id: "k8s_cluster".to_string(),
                args: json!({"namespace": "production"}),
                sanitized_args: json!({"namespace": "production"}),
                request_id: Some("req-102".to_string()),
                context: None,
                timeout_secs: 10,
                webhook: None,
            })
            .await;

        let rejected = registry
            .reject(
                &id,
                "sre-lead".to_string(),
                Some("Forbidden in prod".to_string()),
                None,
            )
            .await
            .expect("reject should succeed");
        assert!(rejected);

        let resolution = rx.await.expect("receiver should get resolution");
        match resolution {
            ApprovalResolution::Rejected { operator, reason } => {
                assert_eq!(operator, "sre-lead");
                assert_eq!(reason, Some("Forbidden in prod".to_string()));
            }
            _ => panic!("Expected Rejected resolution"),
        }
    }

    #[tokio::test]
    async fn test_approval_timeout_expiration() {
        let registry = ApprovalRegistry::new();
        let (id, rx) = registry
            .create_approval(CreateApprovalRequest {
                capability_id: "aws.terminate_instance".to_string(),
                server_id: "aws_srv".to_string(),
                args: json!({"instance_id": "i-12345"}),
                sanitized_args: json!({"instance_id": "i-12345"}),
                request_id: Some("req-103".to_string()),
                context: None,
                timeout_secs: 1, // 1 second timeout
                webhook: None,
            })
            .await;

        registry.expire_if_pending(&id).await;

        let resolution = rx.await.expect("channel should receive expired");
        match resolution {
            ApprovalResolution::Expired => {}
            _ => panic!("Expected Expired resolution"),
        }

        let ticket = registry.get(&id).await.expect("ticket exists");
        match ticket.status {
            ApprovalStatus::Expired { .. } => {}
            _ => panic!("Expected Expired status in ticket"),
        }
    }

    #[tokio::test]
    async fn test_approval_persistence_across_restarts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let state_file = temp_dir.path().join("approvals.json");

        // 1. Initialize registry and create pending ticket
        let registry1 = ApprovalRegistry::open_or_create(&state_file).unwrap();
        let (id1, _) = registry1
            .create_approval(CreateApprovalRequest {
                capability_id: "db.drop_database".to_string(),
                server_id: "postgres".to_string(),
                args: json!({"db": "analytics"}),
                sanitized_args: json!({"db": "analytics"}),
                request_id: Some("req-persist-1".to_string()),
                context: None,
                timeout_secs: 60,
                webhook: None,
            })
            .await;

        let (id2, _) = registry1
            .create_approval(CreateApprovalRequest {
                capability_id: "aws.delete_s3".to_string(),
                server_id: "aws".to_string(),
                args: json!({"bucket": "backups"}),
                sanitized_args: json!({"bucket": "backups"}),
                request_id: Some("req-persist-2".to_string()),
                context: None,
                timeout_secs: 60,
                webhook: None,
            })
            .await;

        // Approve ticket 1
        let approved = registry1
            .approve(&id1, "operator-bob".to_string(), None, None)
            .await
            .unwrap();
        assert!(approved);

        drop(registry1); // Simulate daemon shutdown

        // 2. Reboot daemon and load from persistent state file
        let registry2 = ApprovalRegistry::open_or_create(&state_file).unwrap();
        let ticket1 = registry2
            .get(&id1)
            .await
            .expect("ticket 1 must survive reboot");
        match ticket1.status {
            ApprovalStatus::Approved { operator, .. } => {
                assert_eq!(operator, "operator-bob");
            }
            _ => panic!("Expected ticket 1 to be Approved"),
        }

        let ticket2 = registry2
            .get(&id2)
            .await
            .expect("ticket 2 must survive reboot");
        assert_eq!(ticket2.status, ApprovalStatus::Pending);

        // Reject ticket 2 after reboot
        let rejected = registry2
            .reject(
                &id2,
                "operator-alice".to_string(),
                Some("Not allowed".to_string()),
                None,
            )
            .await
            .unwrap();
        assert!(rejected);

        drop(registry2); // Simulate second restart

        // 3. Second reboot verification
        let registry3 = ApprovalRegistry::open_or_create(&state_file).unwrap();
        let list = registry3.list().await;
        assert_eq!(list.len(), 2);
        let ticket2_final = registry3.get(&id2).await.unwrap();
        match ticket2_final.status {
            ApprovalStatus::Rejected {
                operator, reason, ..
            } => {
                assert_eq!(operator, "operator-alice");
                assert_eq!(reason, Some("Not allowed".to_string()));
            }
            _ => panic!("Expected ticket 2 to be Rejected"),
        }
    }
}
