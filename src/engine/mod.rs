// Rust guideline compliant 2026-11-18
//! Embedded Warmplane engine interface and ControlPlaneHandle implementation (`M-CANONICAL-DOCS`).
//!
//! Provides direct Rust callable APIs without HTTP or transport serialization overhead:
//! - [`EmbeddedWarmplane`]: In-process entry point to boot and supervise upstream MCP sessions.
//! - [`ControlPlaneHandle`]: Cloneable, `Send + Sync` handle to query catalogs, invoke capabilities, and run batches.

pub mod interpolation;
pub mod types;
pub(crate) mod undo;

use crate::undo::{RollbackResult, UndoAccessContext, UndoEntrySummary, UndoOperationError};
use anyhow::Result;
use serde_json::Value;
use std::sync::atomic::Ordering;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::{
    batch_executor::{execute_batch, BatchCallResponse, BatchStep},
    config::McpConfig,
    context::{ProfileContext, RequestContext},
    context_filter::{distill_value, DistillationOptions},
    daemon::{
        initialize_state,
        state::AppState,
        types::{ServerMsg, UpstreamCallError},
    },
    idempotency::RetryMetadata,
};

pub use types::*;

/// Generates a trace ID for request correlation.
#[inline]
pub fn next_trace_id() -> String {
    use std::sync::atomic::AtomicU64;
    static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);
    format!(
        "trc-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        TRACE_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// In-process entry point for booting and managing Warmplane MCP sessions.
pub struct EmbeddedWarmplane;

impl EmbeddedWarmplane {
    /// Spawns upstream MCP supervisors and background workers on the current Tokio runtime.
    ///
    /// # Arguments
    /// * `config` - Typed `McpConfig` specifying servers, policy, resilience, and state settings.
    ///
    /// # Returns
    /// A tuple containing `(ControlPlaneHandle, CancellationToken)`.
    ///
    /// # Errors
    /// Returns an error if initial state configuration fails.
    pub async fn start(config: McpConfig) -> Result<(ControlPlaneHandle, CancellationToken)> {
        let app_state = initialize_state(config, "embedded").await?;
        let shutdown_token = app_state.shutdown_token.clone();
        let handle = ControlPlaneHandle::new(app_state);
        Ok((handle, shutdown_token))
    }

    /// Helper to load a config from a JSON file and boot the embedded engine.
    pub async fn start_from_path(
        config_path: impl AsRef<str>,
    ) -> Result<(ControlPlaneHandle, CancellationToken)> {
        let cfg = crate::config::load_config(config_path.as_ref())?;
        Self::start(cfg).await
    }
}

/// Cloneable handle to invoke capabilities, query resources/prompts, and inspect engine state directly.
#[derive(Clone)]
pub struct ControlPlaneHandle {
    state: AppState,
}

// Keep the thin undo facade methods and their context forwarding consistent.
macro_rules! forward_undo_entry {
    ($(#[$docs:meta])* $method:ident($id:ident) -> $output:ty) => {
        $(#[$docs])*
        pub async fn $method(&self, $id: &str, access: &UndoAccessContext) -> Result<$output, UndoOperationError> {
            crate::undo::$method(&self.state, access, $id).await
        }
    };
}

impl ControlPlaneHandle {
    /// Creates a new `ControlPlaneHandle` wrapping an active `AppState`.
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Returns a reference to the underlying daemon `AppState`.
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Resolves profile context for server scoping and policy enforcement.
    pub async fn resolve_profile_context(&self, profile: Option<&str>) -> ProfileContext {
        if let Some(prof_id) = profile {
            let profiles_guard = self.state.profiles.read().await;
            if let Some(prof_cfg) = profiles_guard.get(prof_id) {
                let prof_policy = prof_cfg
                    .policy
                    .as_ref()
                    .map(|p| crate::daemon::Policy::from_config(Some(p.clone())));
                return ProfileContext::scoped_with_policy(
                    prof_id.to_string(),
                    prof_cfg.servers.clone(),
                    prof_policy,
                )
                .with_agent_self_undo(prof_cfg.allow_agent_self_undo);
            }
        }
        ProfileContext::unrestricted()
    }

    /// Returns current health and diagnostic status of all upstream servers and circuit breakers.
    pub async fn health_status(&self) -> EngineHealthStatus {
        let catalog_version = self.state.catalog_version.read().await.clone();
        let server_statuses = self.state.server_statuses.read().await.clone();
        let mut circuit_breakers = std::collections::HashMap::new();

        for snapshot in self.state.circuit_breakers.all_statuses().await {
            circuit_breakers.insert(snapshot.server_id, snapshot.state);
        }

        EngineHealthStatus {
            catalog_version,
            server_statuses,
            circuit_breakers,
            total_tool_calls: self.state.total_tool_calls.load(Ordering::Relaxed),
            total_tool_duration_us: self.state.total_tool_duration_us.load(Ordering::Relaxed),
        }
    }

    /// Lists safe summaries of undo entries available to an authorized caller.
    ///
    /// Inverse arguments and snapshot contents are not included in list responses. Tenant
    /// ownership is always taken from `access`, never from the filter.
    ///
    /// # Arguments
    /// * `filter` - Optional status/work-item filters and pagination.
    /// * `access` - Trusted tenant, role, policy, profile, and caller context.
    ///
    /// # Returns
    /// A tenant-scoped page and the total number of matching entries.
    ///
    /// # Errors
    /// Returns an authorization or persistent-storage error.
    pub async fn list_undo_entries(
        &self,
        filter: crate::undo::UndoListFilter,
        access: &UndoAccessContext,
    ) -> Result<crate::undo::UndoListResponse, UndoOperationError> {
        crate::undo::list_undo_entries(&self.state, access, filter).await
    }

    forward_undo_entry! {
        /// Returns safe details for one undo entry belonging to the caller's tenant.
        ///
        /// # Arguments
        /// * `undo_id` - Stable undo-entry identifier.
        /// * `access` - Trusted tenant and role context.
        ///
        /// # Returns
        /// A safe summary without inverse arguments or snapshot contents.
        ///
        /// # Errors
        /// Returns not-found for missing or cross-tenant entries, or an authorization error.
        get_undo_entry(undo_id) -> UndoEntrySummary
    }

    /// Executes one authorized undo entry and returns its inverse output and audit receipt.
    ///
    /// # Arguments
    /// * `undo_id` - Stable undo-entry identifier.
    /// * `access` - Trusted tenant, policy, profile, and operator context.
    /// * `force_expired` - Whether the authorized caller explicitly overrides expiration.
    /// * `force_reason` - Required audit reason when `force_expired` is true.
    ///
    /// # Returns
    /// Per-entry execution outcome, sanitized inverse output, and committed audit receipt.
    ///
    /// # Errors
    /// Returns authorization, expiry, state-conflict, or storage errors.
    pub async fn execute_undo(
        &self,
        undo_id: &str,
        access: &UndoAccessContext,
        force_expired: bool,
        force_reason: Option<&str>,
    ) -> Result<RollbackResult, UndoOperationError> {
        crate::undo::execute_undo(&self.state, access, undo_id, force_expired, force_reason).await
    }

    /// Rolls back all eligible entries for a work item in LIFO order.
    ///
    /// # Arguments
    /// * `work_item_id` - Work-item fiber to unwind.
    /// * `access` - Trusted tenant, policy, profile, and operator context.
    /// * `force_expired` - Whether the authorized caller overrides expired entries.
    /// * `force_reason` - Required audit reason when `force_expired` is true.
    ///
    /// # Returns
    /// One structured outcome per selected undo entry.
    ///
    /// # Errors
    /// Returns authorization or persistent-storage errors.
    pub async fn rollback_work_item(
        &self,
        work_item_id: &str,
        access: &UndoAccessContext,
        force_expired: bool,
        force_reason: Option<&str>,
    ) -> Result<Vec<RollbackResult>, UndoOperationError> {
        crate::undo::rollback_work_item(
            &self.state,
            access,
            work_item_id,
            force_expired,
            force_reason,
        )
        .await
    }

    forward_undo_entry! {
        /// Executes an undo requested by an opted-in MCP agent.
        ///
        /// # Arguments
        /// * `undo_id` - Stable undo-entry identifier.
        /// * `access` - Trusted role, profile, tenant, and actor context.
        ///
        /// # Returns
        /// The inverse execution result and committed audit receipt.
        ///
        /// # Errors
        /// Returns policy, ownership, expiry, or storage errors.
        execute_agent_self_undo(undo_id) -> RollbackResult
    }

    forward_undo_entry! {
        /// Executes the newest armed undo entry for an opted-in MCP agent's work item.
        ///
        /// # Arguments
        /// * `work_item_id` - Work-item fiber associated with the agent's current turn.
        /// * `access` - Trusted role, profile, tenant, and caller context.
        ///
        /// # Returns
        /// The inverse execution result and committed audit receipt.
        ///
        /// # Errors
        /// Returns policy, ownership, expiry, or storage errors.
        execute_agent_self_undo_for_work_item(work_item_id) -> RollbackResult
    }

    /// Lists registered capabilities, optionally filtered by a named profile constellation.
    pub async fn list_capabilities(
        &self,
        profile: Option<&str>,
    ) -> Result<CapabilitiesListResponse> {
        let prof_ctx = self.resolve_profile_context(profile).await;
        let caps_guard = self.state.capabilities.read().await;
        let mut capabilities = caps_guard
            .iter()
            .filter(|(_, meta)| prof_ctx.is_server_allowed(&meta.server))
            .map(|(id, meta)| CapabilitySummary {
                id: id.clone(),
                summary: meta.summary.clone(),
                server: meta.server.clone(),
                tool: meta.tool.clone(),
                signature: meta.signature.clone(),
                tags: meta.tags.clone(),
            })
            .collect::<Vec<_>>();

        capabilities.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(CapabilitiesListResponse {
            version: "v1".to_string(),
            capabilities,
            search_engine: Some(self.state.search_engine.info()),
        })
    }

    /// Describes a single capability schema and examples.
    pub async fn describe_capability(
        &self,
        id: &str,
        profile: Option<&str>,
    ) -> Envelope<CapabilityDetail> {
        let trace_id = next_trace_id();
        let prof_ctx = self.resolve_profile_context(profile).await;
        let caps_guard = self.state.capabilities.read().await;

        match caps_guard.get(id) {
            Some(meta) if prof_ctx.is_server_allowed(&meta.server) => Envelope::success(
                trace_id,
                None,
                None,
                CapabilityDetail {
                    id: id.to_string(),
                    server: meta.server.clone(),
                    tool: meta.tool.clone(),
                    description: meta.description.clone(),
                    input_schema: meta.input_schema.clone(),
                    examples: meta.examples.clone(),
                },
                RetryMetadata::safe("completed"),
            ),
            _ => Envelope::failure(
                trace_id,
                None,
                None,
                WarmplaneError::new(
                    "TOOL_NOT_FOUND",
                    format!("Capability '{}' not found", id),
                    false,
                ),
                RetryMetadata::safe("not_started"),
            ),
        }
    }

    /// Performs hybrid lexical and semantic search over registered capabilities.
    pub async fn search_capabilities(
        &self,
        query: Option<&str>,
        server_ids: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        modes: Option<Vec<String>>,
        limit: Option<usize>,
        profile: Option<&str>,
    ) -> CapabilitySearchResponse {
        let prof_ctx = self.resolve_profile_context(profile).await;
        let mut filter_builder = crate::search::SearchFilter::builder();
        let effective_servers = match &prof_ctx.allowed_servers {
            Some(allowed) => match server_ids {
                Some(servers) => servers
                    .into_iter()
                    .filter(|s| allowed.contains(s))
                    .collect(),
                None => allowed.iter().cloned().collect(),
            },
            None => server_ids.unwrap_or_default(),
        };

        if !effective_servers.is_empty() {
            filter_builder = filter_builder.server_ids(effective_servers);
        }
        if let Some(t) = tags {
            filter_builder = filter_builder.tags(t);
        }
        if let Some(m) = modes {
            filter_builder = filter_builder.modes(m);
        }
        let filter = filter_builder.build();

        let caps = self.state.capabilities.read().await;
        let pol = self.state.policy.read().await;
        let base_ver = self.state.catalog_version.read().await.clone();
        let catalog_ver =
            crate::http_v1::helpers::get_profile_scoped_catalog_version(&base_ver, &prof_ctx);
        let query_str = query.unwrap_or("");
        let limit_val = limit.unwrap_or(8);

        let results = self
            .state
            .search_engine
            .search(query_str, limit_val, &filter, &caps, &pol);

        CapabilitySearchResponse {
            version: "v1".to_string(),
            catalog_version: catalog_ver,
            query: query_str.to_string(),
            total: results.len(),
            capabilities: results,
        }
    }

    /// Invokes a capability tool with policy enforcement, circuit breaker checks, HITL approvals, and idempotency deduplication.
    pub async fn call_capability(
        &self,
        capability_id: &str,
        args: Value,
        options: ExecutionOptions,
    ) -> Envelope<Value> {
        let prof_ctx = self
            .resolve_profile_context(options.profile.as_deref())
            .await;
        let start_time = std::time::Instant::now();
        self.state.total_tool_calls.fetch_add(1, Ordering::Relaxed);
        let trace_id = next_trace_id();
        let req_id = options
            .request_id
            .clone()
            .unwrap_or_else(|| trace_id.clone());
        let req_ctx = options.context.clone().unwrap_or_default();
        let suppress_undo_registration = options.suppress_undo_registration;
        let explicit_key = options.idempotency_key.clone();
        let idempotency_key = explicit_key.or_else(|| {
            if args.is_object() {
                Some(crate::idempotency::derive_idempotency_key(
                    capability_id,
                    &args,
                    req_ctx.actor_id.as_deref(),
                    options.request_id.as_deref(),
                ))
            } else {
                None
            }
        });

        let retry_base = if idempotency_key.is_some() {
            RetryMetadata::idempotent
        } else {
            RetryMetadata::unsafe_op
        };

        if let Some(key) = idempotency_key.as_deref() {
            if let Some(task) = self.state.task_registry.find_by_idempotency_key(key).await {
                if task.emission.is_some() {
                    if !self
                        .state
                        .hold_coordinator
                        .can_access_task(
                            &self.state,
                            &task,
                            "default",
                            &self.state.rbac_engine.default_role(),
                            prof_ctx.profile_id.as_deref(),
                        )
                        .await
                    {
                        return Envelope::failure(
                            trace_id,
                            Some(req_id),
                            Some(req_ctx),
                            WarmplaneError::new(
                                "POLICY_DENIED",
                                "Held task is not accessible to this caller",
                                false,
                            ),
                            retry_base("not_started"),
                        );
                    }
                    return Envelope::success(
                        trace_id,
                        Some(req_id),
                        Some(req_ctx),
                        serde_json::to_value(crate::tasks::TaskResponse::from(&task))
                            .unwrap_or_default(),
                        retry_base(
                            if matches!(
                                task.status,
                                crate::tasks::TaskStatus::Completed
                                    | crate::tasks::TaskStatus::Failed
                                    | crate::tasks::TaskStatus::Cancelled
                            ) {
                                "completed"
                            } else {
                                "in_progress"
                            },
                        ),
                    );
                }
            }
        }

        // Check Idempotency Store
        let idempotency_claim = if let Some(key) = idempotency_key.as_deref() {
            match self
                .state
                .idempotency_store
                .claim_or_replay(key, capability_id, &trace_id)
                .await
            {
                crate::idempotency::ExecutionClaim::New(claim) => Some(claim),
                crate::idempotency::ExecutionClaim::Replay { response, .. } => {
                    if let Ok(env) = serde_json::from_value::<Envelope<Value>>(response.clone()) {
                        return env;
                    }
                    return Envelope::success(
                        trace_id,
                        Some(req_id),
                        Some(req_ctx),
                        response,
                        retry_base("completed"),
                    );
                }
            }
        } else {
            None
        };

        if !args.is_object() {
            self.state
                .idempotency_store
                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                .await;
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new("INVALID_ARGS", "'args' must be a JSON object", false),
                retry_base("not_started"),
            );
        }

        let (mut requires_approval, approval_timeout_secs, webhook_cfg, redact_keys) = {
            let pol = self.state.policy.read().await;
            if !pol.allows(capability_id) {
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "POLICY_DENIED",
                        format!("Capability '{}' blocked by policy", capability_id),
                        false,
                    ),
                    retry_base("not_started"),
                );
            }
            (
                pol.requires_approval(capability_id),
                pol.approval_timeout_secs,
                pol.webhook.clone(),
                pol.redact_keys.clone(),
            )
        };

        let (server_id, tool_name, reversibility) = {
            let caps_guard = self.state.capabilities.read().await;
            let Some(meta) = caps_guard.get(capability_id) else {
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "TOOL_NOT_FOUND",
                        format!("Capability '{}' not found", capability_id),
                        false,
                    ),
                    retry_base("not_started"),
                );
            };

            if !prof_ctx.is_server_allowed(&meta.server) {
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "TOOL_NOT_IN_PROFILE",
                        format!(
                            "Capability '{}' belongs to server '{}' which is not in active profile",
                            capability_id, meta.server
                        ),
                        false,
                    ),
                    retry_base("not_started"),
                );
            }

            (
                meta.server.clone(),
                meta.tool.clone(),
                meta.reversibility.clone(),
            )
        };

        let mut compensable_reserved = false;
        if let Some(contract) = reversibility
            .as_ref()
            .filter(|contract| contract.class == crate::config::ReversibilityClass::Compensable)
        {
            if self.state.undo_ledger.is_none() {
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "UNDO_STORAGE_UNAVAILABLE",
                        "Persistent usage storage is required for compensable capabilities",
                        false,
                    ),
                    retry_base("not_started"),
                );
            }
            if !requires_approval {
                match undo::reserve_compensable(
                    &self.state,
                    contract,
                    capability_id,
                    "default",
                    prof_ctx.profile_id.as_deref(),
                    idempotency_key.as_deref(),
                    false,
                )
                .await
                {
                    Ok(true) => compensable_reserved = true,
                    Ok(false) => requires_approval = true,
                    Err(error) => {
                        self.state
                            .idempotency_store
                            .release_execution(idempotency_key.as_deref(), idempotency_claim)
                            .await;
                        return Envelope::failure(
                            trace_id,
                            Some(req_id),
                            Some(req_ctx),
                            WarmplaneError::new(
                                "COMPENSABLE_LIMIT_STORAGE_FAILED",
                                error.to_string(),
                                true,
                            ),
                            retry_base("not_started"),
                        );
                    }
                }
            }
        }

        let mut effective_args = args.clone();

        // If caller requested async task handle, create task and spawn background worker
        if options.async_task {
            let initial_status = if requires_approval {
                crate::tasks::TaskStatus::InputRequired
            } else {
                crate::tasks::TaskStatus::Working
            };

            let status_msg = if requires_approval {
                Some(format!(
                    "Execution suspended awaiting operator approval for capability '{}'",
                    capability_id
                ))
            } else {
                Some("Task accepted and running".to_string())
            };

            let input_requests = if requires_approval {
                let sanitized = crate::http_v1::helpers::redact_value(args.clone(), &redact_keys);
                let mut map = std::collections::BTreeMap::new();
                map.insert(
                    "hitl_approval".to_string(),
                    serde_json::json!({
                        "type": "approval_review",
                        "capability_id": capability_id,
                        "server_id": server_id,
                        "sanitized_args": sanitized,
                        "timeout_secs": approval_timeout_secs,
                    }),
                );
                Some(map)
            } else {
                None
            };

            let (task_record, task_rx) = self
                .state
                .task_registry
                .create_task(crate::tasks::CreateTaskParams {
                    capability_id: capability_id.to_string(),
                    server_id: server_id.clone(),
                    args: args.clone(),
                    request_id: Some(req_id.clone()),
                    context: Some(req_ctx.clone()),
                    idempotency_key: idempotency_key.clone(),
                    initial_status,
                    status_message: status_msg,
                    input_requests,
                    ttl_ms: Some(approval_timeout_secs * 1000),
                    poll_interval_ms: Some(1000),
                })
                .await;

            let handle = self.clone();
            let task_id_clone = task_record.task_id.clone();
            let server_id_clone = server_id.clone();
            let tool_name_clone = tool_name.clone();
            let args_clone = args.clone();
            let idempotency_key_clone = idempotency_key.clone();
            let trace_id_clone = trace_id.clone();
            let req_id_clone = req_id.clone();
            let req_ctx_clone = req_ctx.clone();
            let reversibility_clone = reversibility.clone();
            let profile_context_clone = prof_ctx.clone();
            let capability_id_clone = capability_id.to_string();
            let compensable_reserved_clone = compensable_reserved;
            let suppress_undo_registration_clone = suppress_undo_registration;
            let policy_for_undo = {
                let policy = self.state.policy.read().await;
                policy.merge_with_profile(prof_ctx.profile_policy.as_ref())
            };

            tokio::spawn(async move {
                let mut worker_args = args_clone.clone();
                if requires_approval {
                    if let Some(trx) = task_rx {
                        match trx.await {
                            Ok(responses) => {
                                if let Some(appr) = responses.get("hitl_approval") {
                                    if appr.get("approved").and_then(Value::as_bool) == Some(true) {
                                        if let Some(mod_args) = appr.get("modified_args").cloned() {
                                            worker_args = mod_args;
                                        }
                                    } else {
                                        let reason = appr
                                            .get("reason")
                                            .and_then(Value::as_str)
                                            .map(ToString::to_string);
                                        let _ = handle
                                            .state
                                            .task_registry
                                            .cancel_task(&task_id_clone, reason)
                                            .await;
                                        handle
                                            .state
                                            .idempotency_store
                                            .release_execution(
                                                idempotency_key_clone.as_deref(),
                                                idempotency_claim,
                                            )
                                            .await;
                                        return;
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = handle
                                    .state
                                    .task_registry
                                    .fail_task(
                                        &task_id_clone,
                                        serde_json::json!({"code": "APPROVAL_TIMEOUT"}),
                                        Some("Approval request timed out or cancelled".to_string()),
                                    )
                                    .await;
                                handle
                                    .state
                                    .idempotency_store
                                    .release_execution(
                                        idempotency_key_clone.as_deref(),
                                        idempotency_claim,
                                    )
                                    .await;
                                return;
                            }
                        }
                    }
                }

                if !compensable_reserved_clone {
                    if let Some(contract) = reversibility_clone.as_ref().filter(|contract| {
                        contract.class == crate::config::ReversibilityClass::Compensable
                    }) {
                        match undo::reserve_compensable(
                            &handle.state,
                            contract,
                            &capability_id_clone,
                            "default",
                            profile_context_clone.profile_id.as_deref(),
                            idempotency_key_clone.as_deref(),
                            true,
                        )
                        .await
                        {
                            Ok(true) => {}
                            Ok(false) | Err(_) => {
                                let _ = handle
                                    .state
                                    .task_registry
                                    .fail_task(
                                        &task_id_clone,
                                        serde_json::json!({"code": "COMPENSABLE_LIMIT_STORAGE_FAILED"}),
                                        Some("Could not reserve compensable invocation".to_string()),
                                    )
                                    .await;
                                handle
                                    .state
                                    .idempotency_store
                                    .release_execution(
                                        idempotency_key_clone.as_deref(),
                                        idempotency_claim,
                                    )
                                    .await;
                                return;
                            }
                        }
                    }
                }

                if let Some(contract) = reversibility_clone.as_ref().filter(|contract| {
                    contract.class == crate::config::ReversibilityClass::Delayable
                }) {
                    let global_window = handle.state.policy.read().await.hold_window_secs;
                    let profile_window = profile_context_clone
                        .profile_policy
                        .as_ref()
                        .and_then(|profile_policy| profile_policy.hold_window_secs);
                    let hold_secs = crate::emissions::effective_hold_window_secs(
                        contract,
                        profile_window,
                        global_window,
                    )
                    .unwrap_or(30);
                    let emission = crate::tasks::EmissionRecord {
                        idempotency_claim,
                        phase: crate::tasks::EmissionPhase::PendingEmission,
                        release_at_ms: match crate::emissions::release_deadline_ms(hold_secs) {
                            Ok(deadline) => deadline,
                            Err(error) => {
                                let _ = handle
                                    .state
                                    .task_registry
                                    .fail_task(
                                        &task_id_clone,
                                        serde_json::json!({"code": "INVALID_HOLD_WINDOW"}),
                                        Some(error.to_string()),
                                    )
                                    .await;
                                return;
                            }
                        },
                        cancellable: true,
                        tool_name: tool_name_clone.clone(),
                        input_responses: options.input_responses.clone(),
                        request_state: options.request_state.clone(),
                        profile_id: profile_context_clone.profile_id.clone(),
                        tenant_id: "default".to_string(),
                        role: handle.state.rbac_engine.default_role().to_string(),
                        authorization_expires_at_epoch_secs: None,
                        approval_completed: requires_approval,
                        batch_id: None,
                        step_id: None,
                    };
                    match handle
                        .state
                        .hold_coordinator
                        .queue_existing(
                            &handle.state,
                            &task_id_clone,
                            worker_args.clone(),
                            emission,
                        )
                        .await
                    {
                        Ok(Some(_)) => return,
                        Ok(None) => return,
                        Err(error) => {
                            let _ = handle
                                .state
                                .task_registry
                                .fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "EMISSION_PERSISTENCE_FAILED"}),
                                    Some(error.to_string()),
                                )
                                .await;
                            handle
                                .state
                                .idempotency_store
                                .release_execution(
                                    idempotency_key_clone.as_deref(),
                                    idempotency_claim,
                                )
                                .await;
                            return;
                        }
                    }
                }

                let (snapshot, snapshot_hash) = if let Some(contract) = &reversibility_clone {
                    if matches!(
                        contract.class,
                        crate::config::ReversibilityClass::Reversible
                            | crate::config::ReversibilityClass::Compensable
                    ) && handle.state.undo_ledger.is_none()
                    {
                        let _ = handle
                            .state
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "UNDO_STORAGE_UNAVAILABLE"}),
                                Some("Persistent undo ledger is unavailable".to_string()),
                            )
                            .await;
                        handle
                            .state
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                        return;
                    }
                    if let Some(pre_snapshot) = &contract.pre_snapshot {
                        match undo::capture_pre_snapshot(
                            &handle.state,
                            pre_snapshot,
                            &worker_args,
                            &profile_context_clone,
                            &policy_for_undo,
                        )
                        .await
                        {
                            Ok(snapshot) => snapshot,
                            Err(error) => {
                                let _ = handle
                                    .state
                                    .task_registry
                                    .fail_task(
                                        &task_id_clone,
                                        serde_json::json!({"code": "PRE_SNAPSHOT_FAILED"}),
                                        Some(error.to_string()),
                                    )
                                    .await;
                                handle
                                    .state
                                    .idempotency_store
                                    .release_execution(
                                        idempotency_key_clone.as_deref(),
                                        idempotency_claim,
                                    )
                                    .await;
                                return;
                            }
                        }
                    } else {
                        (Value::Null, None)
                    }
                } else {
                    (Value::Null, None)
                };

                let tx = {
                    let servers_guard = handle.state.servers.read().await;
                    match servers_guard.get(&server_id_clone).cloned() {
                        Some(tx) => tx,
                        None => {
                            let _ = handle
                                .state
                                .task_registry
                                .fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "SERVER_UNREACHABLE"}),
                                    Some(format!("Server '{}' is unreachable", server_id_clone)),
                                )
                                .await;
                            handle
                                .state
                                .idempotency_store
                                .release_execution(
                                    idempotency_key_clone.as_deref(),
                                    idempotency_claim,
                                )
                                .await;
                            return;
                        }
                    }
                };

                let (reply_tx, reply_rx) = oneshot::channel();
                if tx
                    .send(ServerMsg::CallTool {
                        name: tool_name_clone,
                        params: worker_args.clone(),
                        input_responses: options.input_responses.clone(),
                        request_state: options.request_state.clone(),
                        reply: reply_tx,
                    })
                    .await
                    .is_err()
                {
                    handle
                        .state
                        .circuit_breakers
                        .record_failure(&server_id_clone)
                        .await;
                    let _ = handle
                        .state
                        .task_registry
                        .fail_task(
                            &task_id_clone,
                            serde_json::json!({"code": "SERVER_UNREACHABLE"}),
                            Some(format!("Server '{}' mailbox is closed", server_id_clone)),
                        )
                        .await;
                    handle
                        .state
                        .idempotency_store
                        .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                        .await;
                    return;
                }

                let distill_opts = DistillationOptions::from_args(Some(&args_clone));
                match reply_rx.await {
                    Ok(Ok(data)) => {
                        handle
                            .state
                            .circuit_breakers
                            .record_success(&server_id_clone)
                            .await;
                        if !suppress_undo_registration_clone {
                            if let Some(contract) = &reversibility_clone {
                                if let Err(error) = undo::register_inverse(
                                    &handle.state,
                                    contract,
                                    undo::ForwardCall {
                                        capability: &capability_id_clone,
                                        args: &worker_args,
                                        result: &data,
                                        snapshot: &snapshot,
                                        snapshot_hash,
                                        context: &req_ctx_clone,
                                        batch_id: None,
                                        tenant_id: "default",
                                        profile_id: profile_context_clone.profile_id.as_deref(),
                                    },
                                    &profile_context_clone,
                                    &policy_for_undo,
                                )
                                .await
                                {
                                    let message = format!(
                                        "Forward call succeeded but inverse registration failed: {error}"
                                    );
                                    let _ = handle
                                        .state
                                        .task_registry
                                        .fail_task(
                                            &task_id_clone,
                                            serde_json::json!({"code": "UNDO_REGISTRATION_FAILED"}),
                                            Some(message.clone()),
                                        )
                                        .await;
                                    let env: Envelope<Value> = Envelope::failure(
                                        trace_id_clone.clone(),
                                        Some(req_id_clone.clone()),
                                        Some(req_ctx_clone.clone()),
                                        WarmplaneError::new(
                                            "UNDO_REGISTRATION_FAILED",
                                            message,
                                            false,
                                        ),
                                        retry_base("completed"),
                                    );
                                    if let (Some(key), Some(claim)) =
                                        (idempotency_key_clone.as_deref(), idempotency_claim)
                                    {
                                        if let Ok(value) = serde_json::to_value(&env) {
                                            handle
                                                .state
                                                .idempotency_store
                                                .complete(key, claim, value)
                                                .await;
                                        }
                                    }
                                    return;
                                }
                            }
                        }
                        let distilled_data = distill_value(data, &distill_opts);
                        let outcome = handle
                            .state
                            .task_registry
                            .complete_execution(&task_id_clone, distilled_data)
                            .await;

                        let env = EnvelopeBuilder::new(trace_id_clone, retry_base("completed"))
                            .request_id(Some(req_id_clone))
                            .context(Some(req_ctx_clone))
                            .outcome(outcome);
                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            if let Ok(val) = serde_json::to_value(&env) {
                                handle
                                    .state
                                    .idempotency_store
                                    .complete(key, claim, val)
                                    .await;
                            }
                        }
                    }
                    Ok(Err(UpstreamCallError::Timeout)) => {
                        handle
                            .state
                            .circuit_breakers
                            .record_failure(&server_id_clone)
                            .await;
                        let _ = handle
                            .state
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "UPSTREAM_TIMEOUT"}),
                                Some("Tool call timed out".to_string()),
                            )
                            .await;
                        handle
                            .state
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                    }
                    Ok(Err(UpstreamCallError::Upstream(err))) => {
                        handle
                            .state
                            .circuit_breakers
                            .record_failure(&server_id_clone)
                            .await;
                        let _ = handle
                            .state
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "UPSTREAM_ERROR", "message": &err}),
                                Some(err),
                            )
                            .await;
                        handle
                            .state
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                    }
                    Err(_) => {
                        handle
                            .state
                            .circuit_breakers
                            .record_failure(&server_id_clone)
                            .await;
                        let _ = handle
                            .state
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "INTERNAL_ERROR"}),
                                Some("Server actor task dropped reply channel".to_string()),
                            )
                            .await;
                        handle
                            .state
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                    }
                }
            });

            let task_resp = crate::tasks::TaskResponse::from(&task_record);
            return Envelope::success(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                serde_json::to_value(task_resp).unwrap_or_default(),
                retry_base("in_progress"),
            );
        }

        // Synchronous Human-in-the-Loop (HITL) Interception Flow
        let task_record = if requires_approval {
            let sanitized = crate::http_v1::helpers::redact_value(args.clone(), &redact_keys);

            let mut input_requests = std::collections::BTreeMap::new();
            input_requests.insert(
                "hitl_approval".to_string(),
                serde_json::json!({
                    "type": "approval_review",
                    "capability_id": capability_id,
                    "server_id": server_id,
                    "sanitized_args": sanitized,
                    "timeout_secs": approval_timeout_secs,
                }),
            );

            let (rec, trx) = self
                .state
                .task_registry
                .create_task(crate::tasks::CreateTaskParams {
                    capability_id: capability_id.to_string(),
                    server_id: server_id.clone(),
                    args: args.clone(),
                    request_id: Some(req_id.clone()),
                    context: Some(req_ctx.clone()),
                    idempotency_key: idempotency_key.clone(),
                    initial_status: crate::tasks::TaskStatus::InputRequired,
                    status_message: Some(format!(
                        "Execution suspended awaiting operator approval for capability '{}'",
                        capability_id
                    )),
                    input_requests: Some(input_requests),
                    ttl_ms: Some(approval_timeout_secs * 1000),
                    poll_interval_ms: Some(1000),
                })
                .await;

            let (_approval_id, rx) = self
                .state
                .approval_registry
                .create_approval(crate::approvals::CreateApprovalRequest {
                    capability_id: capability_id.to_string(),
                    server_id: server_id.clone(),
                    args: args.clone(),
                    sanitized_args: sanitized,
                    request_id: Some(req_id.clone()),
                    context: Some(req_ctx.clone()),
                    timeout_secs: approval_timeout_secs,
                    webhook: webhook_cfg.as_ref(),
                })
                .await;

            // Wait for either the HITL approval channel or task input responses
            tokio::select! {
                resolution = rx => {
                    let res = match resolution {
                        Ok(r) => r,
                        Err(_) => crate::approvals::ApprovalResolution::Expired,
                    };
                    match res {
                        crate::approvals::ApprovalResolution::Approved { modified_args, .. } => {
                            if let Some(mod_args) = modified_args {
                                effective_args = mod_args;
                            }
                        }
                        crate::approvals::ApprovalResolution::Rejected { operator, reason } => {
                            let _ = self.state.task_registry.cancel_task(&rec.task_id, reason.clone()).await;
                            self.state.idempotency_store
                                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                                .await;
                            return Envelope::failure(
                                trace_id,
                                Some(req_id),
                                Some(req_ctx),
                                WarmplaneError::operator_rejected(operator, reason),
                                retry_base("not_started"),
                            );
                        }
                        crate::approvals::ApprovalResolution::Expired => {
                            let _ = self.state.task_registry.fail_task(&rec.task_id, serde_json::json!({"code": "APPROVAL_TIMEOUT"}), Some("Approval request timed out".to_string())).await;
                            self.state.idempotency_store
                                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                                .await;
                            return Envelope::failure(
                                trace_id,
                                Some(req_id),
                                Some(req_ctx),
                                WarmplaneError::new(
                                    "APPROVAL_TIMEOUT",
                                    format!("Approval request timed out after {}s", approval_timeout_secs),
                                    true,
                                ),
                                retry_base("not_started"),
                            );
                        }
                    }
                }
                task_res = async {
                    if let Some(trx) = trx {
                        trx.await.ok()
                    } else {
                        None
                    }
                } => {
                    if let Some(responses) = task_res {
                        if let Some(appr) = responses.get("hitl_approval") {
                            if appr.get("approved").and_then(Value::as_bool) == Some(true) {
                                if let Some(mod_args) = appr.get("modified_args").cloned() {
                                    effective_args = mod_args;
                                }
                            } else {
                                let reason = appr.get("reason").and_then(Value::as_str).map(ToString::to_string);
                                let _ = self.state.task_registry.cancel_task(&rec.task_id, reason.clone()).await;
                                self.state.idempotency_store
                                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                                    .await;
                                return Envelope::failure(
                                    trace_id,
                                    Some(req_id),
                                    Some(req_ctx),
                                    WarmplaneError::operator_rejected("client-agent", reason),
                                    retry_base("not_started"),
                                );
                            }
                        }
                    }
                }
            }
            Some(rec)
        } else {
            None
        };

        if !compensable_reserved {
            if let Some(contract) = reversibility
                .as_ref()
                .filter(|contract| contract.class == crate::config::ReversibilityClass::Compensable)
            {
                match undo::reserve_compensable(
                    &self.state,
                    contract,
                    capability_id,
                    "default",
                    prof_ctx.profile_id.as_deref(),
                    idempotency_key.as_deref(),
                    true,
                )
                .await
                {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        self.state
                            .idempotency_store
                            .release_execution(idempotency_key.as_deref(), idempotency_claim)
                            .await;
                        return Envelope::failure(
                            trace_id,
                            Some(req_id),
                            Some(req_ctx),
                            WarmplaneError::new(
                                "COMPENSABLE_LIMIT_STORAGE_FAILED",
                                "Could not reserve compensable invocation",
                                true,
                            ),
                            retry_base("not_started"),
                        );
                    }
                }
            }
        }

        if let Some(contract) = reversibility
            .as_ref()
            .filter(|contract| contract.class == crate::config::ReversibilityClass::Delayable)
        {
            let global_window = self.state.policy.read().await.hold_window_secs;
            let profile_window = prof_ctx
                .profile_policy
                .as_ref()
                .and_then(|profile_policy| profile_policy.hold_window_secs);
            let hold_secs = crate::emissions::effective_hold_window_secs(
                contract,
                profile_window,
                global_window,
            )
            .unwrap_or(30);
            let hold_ms = match hold_secs.checked_mul(1_000) {
                Some(value) => value,
                None => {
                    return Envelope::failure(
                        trace_id,
                        Some(req_id),
                        Some(req_ctx),
                        WarmplaneError::new(
                            "INVALID_HOLD_WINDOW",
                            "Hold window overflows milliseconds",
                            false,
                        ),
                        retry_base("not_started"),
                    );
                }
            };
            let release_at_ms = match crate::emissions::release_deadline_ms(hold_secs) {
                Ok(deadline) => deadline,
                Err(error) => {
                    return Envelope::failure(
                        trace_id,
                        Some(req_id),
                        Some(req_ctx),
                        WarmplaneError::new("INVALID_HOLD_WINDOW", error.to_string(), false),
                        retry_base("not_started"),
                    );
                }
            };
            let emission = crate::tasks::EmissionRecord {
                idempotency_claim,
                phase: crate::tasks::EmissionPhase::PendingEmission,
                release_at_ms,
                cancellable: true,
                tool_name: tool_name.clone(),
                input_responses: options.input_responses.clone(),
                request_state: options.request_state.clone(),
                profile_id: prof_ctx.profile_id.clone(),
                tenant_id: "default".to_string(),
                role: self.state.rbac_engine.default_role().to_string(),
                authorization_expires_at_epoch_secs: None,
                approval_completed: requires_approval,
                batch_id: None,
                step_id: None,
            };
            let task = if let Some(existing) = task_record.as_ref() {
                self.state
                    .hold_coordinator
                    .queue_existing(
                        &self.state,
                        &existing.task_id,
                        effective_args.clone(),
                        emission,
                    )
                    .await
                    .ok()
                    .flatten()
            } else {
                self.state
                    .hold_coordinator
                    .accept(
                        &self.state,
                        crate::tasks::CreateTaskParams {
                            capability_id: capability_id.to_string(),
                            server_id: server_id.clone(),
                            args: effective_args.clone(),
                            request_id: Some(req_id.clone()),
                            context: Some(req_ctx.clone()),
                            idempotency_key: idempotency_key.clone(),
                            initial_status: crate::tasks::TaskStatus::Working,
                            status_message: Some(
                                "Waiting for the emission release deadline".to_string(),
                            ),
                            input_requests: None,
                            ttl_ms: None,
                            poll_interval_ms: Some(1_000),
                        },
                        emission,
                    )
                    .await
                    .ok()
            };
            let Some(task) = task else {
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "EMISSION_PERSISTENCE_FAILED",
                        "Could not persist held emission",
                        true,
                    ),
                    retry_base("not_started"),
                );
            };
            if hold_ms < self.state.tool_timeout_ms {
                let wait = std::time::Duration::from_millis(
                    hold_ms.saturating_add(self.state.tool_timeout_ms),
                );
                if let Some(finished) = self
                    .state
                    .hold_coordinator
                    .wait_for_terminal(&self.state, &task.task_id, wait)
                    .await
                {
                    match finished.status {
                        crate::tasks::TaskStatus::Completed => {
                            let env = Envelope::success(
                                trace_id,
                                Some(req_id),
                                Some(req_ctx),
                                finished.result.unwrap_or(Value::Null),
                                retry_base("completed"),
                            );
                            if let (Some(key), Some(claim)) =
                                (idempotency_key.as_deref(), idempotency_claim)
                            {
                                if let Ok(value) = serde_json::to_value(&env) {
                                    self.state
                                        .idempotency_store
                                        .complete(key, claim, value)
                                        .await;
                                }
                            }
                            return env;
                        }
                        crate::tasks::TaskStatus::Failed | crate::tasks::TaskStatus::Cancelled => {
                            return Envelope::failure(
                                trace_id,
                                Some(req_id),
                                Some(req_ctx),
                                WarmplaneError::new(
                                    finished
                                        .error
                                        .as_ref()
                                        .and_then(|error| error.get("code"))
                                        .and_then(Value::as_str)
                                        .unwrap_or("EMISSION_FAILED"),
                                    finished.status_message.unwrap_or_else(|| {
                                        "Held emission did not complete".to_string()
                                    }),
                                    false,
                                ),
                                retry_base("completed"),
                            );
                        }
                        _ => {}
                    }
                }
            }
            let receipt = crate::tasks::TaskResponse::from(&task);
            return Envelope::success(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                serde_json::to_value(receipt).unwrap_or_default(),
                retry_base("in_progress"),
            );
        }

        let tx = {
            let servers_guard = self.state.servers.read().await;
            let Some(tx) = servers_guard.get(&server_id).cloned() else {
                if let Some(ref rec) = task_record {
                    let _ = self
                        .state
                        .task_registry
                        .fail_task(
                            &rec.task_id,
                            serde_json::json!({"code": "SERVER_UNREACHABLE"}),
                            Some(format!("Server '{}' is unreachable", server_id)),
                        )
                        .await;
                }
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "SERVER_UNREACHABLE",
                        format!("Server '{}' is unreachable", server_id),
                        true,
                    ),
                    retry_base("not_started"),
                );
            };
            tx
        };

        // Circuit Breaker Permission Check
        if let Err(cb_err) = self
            .state
            .circuit_breakers
            .check_permission(&server_id)
            .await
        {
            if let Some(ref rec) = task_record {
                let _ = self
                    .state
                    .task_registry
                    .fail_task(
                        &rec.task_id,
                        serde_json::json!({"code": "CIRCUIT_OPEN"}),
                        Some(cb_err.to_string()),
                    )
                    .await;
            }
            self.state
                .idempotency_store
                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                .await;
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new("CIRCUIT_OPEN", cb_err.to_string(), false),
                retry_base("not_started"),
            );
        }

        let (snapshot, snapshot_hash) = if let Some(contract) = &reversibility {
            if matches!(
                contract.class,
                crate::config::ReversibilityClass::Reversible
                    | crate::config::ReversibilityClass::Compensable
            ) && self.state.undo_ledger.is_none()
            {
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "UNDO_STORAGE_UNAVAILABLE",
                        "Persistent undo ledger is required for reversible capabilities",
                        false,
                    ),
                    retry_base("not_started"),
                );
            }
            if let Some(pre_snapshot) = &contract.pre_snapshot {
                match undo::capture_pre_snapshot(
                    &self.state,
                    pre_snapshot,
                    &effective_args,
                    &prof_ctx,
                    &self
                        .state
                        .policy
                        .read()
                        .await
                        .merge_with_profile(prof_ctx.profile_policy.as_ref()),
                )
                .await
                {
                    Ok(snapshot) => snapshot,
                    Err(error) => {
                        self.state
                            .idempotency_store
                            .release_execution(idempotency_key.as_deref(), idempotency_claim)
                            .await;
                        return Envelope::failure(
                            trace_id,
                            Some(req_id),
                            Some(req_ctx),
                            WarmplaneError::new("PRE_SNAPSHOT_FAILED", error.to_string(), true),
                            retry_base("not_started"),
                        );
                    }
                }
            } else {
                (Value::Null, None)
            }
        } else {
            (Value::Null, None)
        };

        let distill_opts = DistillationOptions::from_args(Some(&args));
        let (reply_tx, reply_rx) = oneshot::channel();

        if tx
            .send(ServerMsg::CallTool {
                name: tool_name,
                params: effective_args.clone(),
                input_responses: options.input_responses,
                request_state: options.request_state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            self.state.circuit_breakers.record_failure(&server_id).await;
            if let Some(ref rec) = task_record {
                let _ = self
                    .state
                    .task_registry
                    .fail_task(
                        &rec.task_id,
                        serde_json::json!({"code": "SERVER_UNREACHABLE"}),
                        Some(format!("Server '{}' mailbox is closed", server_id)),
                    )
                    .await;
            }
            self.state
                .idempotency_store
                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                .await;
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new(
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' mailbox is closed", server_id),
                    true,
                ),
                retry_base("not_started"),
            );
        }

        let result = reply_rx.await;

        match result {
            Ok(Ok(data)) => {
                self.state.circuit_breakers.record_success(&server_id).await;
                let elapsed_us = start_time.elapsed().as_micros() as u64;
                self.state
                    .total_tool_duration_us
                    .fetch_add(elapsed_us, Ordering::Relaxed);

                if !suppress_undo_registration {
                    if let Some(contract) = &reversibility {
                        if let Err(error) = undo::register_inverse(
                            &self.state,
                            contract,
                            undo::ForwardCall {
                                capability: capability_id,
                                args: &effective_args,
                                result: &data,
                                snapshot: &snapshot,
                                snapshot_hash,
                                context: &req_ctx,
                                batch_id: None,
                                tenant_id: "default",
                                profile_id: prof_ctx.profile_id.as_deref(),
                            },
                            &prof_ctx,
                            &self
                                .state
                                .policy
                                .read()
                                .await
                                .merge_with_profile(prof_ctx.profile_policy.as_ref()),
                        )
                        .await
                        {
                            if let Some(ref rec) = task_record {
                                let _ = self
                                    .state
                                    .task_registry
                                    .fail_task(
                                        &rec.task_id,
                                        serde_json::json!({"code": "UNDO_REGISTRATION_FAILED"}),
                                        Some(error.to_string()),
                                    )
                                    .await;
                            }
                            return Envelope::failure(
                                trace_id,
                                Some(req_id),
                                Some(req_ctx),
                                WarmplaneError::new(
                                    "UNDO_REGISTRATION_FAILED",
                                    format!("Forward call succeeded but inverse registration failed: {error}"),
                                    false,
                                ),
                                retry_base("completed"),
                            );
                        }
                    }
                }
                let distilled_data = distill_value(data, &distill_opts);
                let outcome = match task_record.as_ref() {
                    Some(record) => {
                        self.state
                            .task_registry
                            .complete_execution(&record.task_id, distilled_data)
                            .await
                    }
                    None => Ok(distilled_data),
                };
                let env = EnvelopeBuilder::new(trace_id, retry_base("completed"))
                    .request_id(Some(req_id))
                    .context(Some(req_ctx))
                    .outcome(outcome);

                if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                    if let Ok(val) = serde_json::to_value(&env) {
                        self.state.idempotency_store.complete(key, claim, val).await;
                    }
                }

                env
            }
            Ok(Err(UpstreamCallError::Timeout)) => {
                self.state.circuit_breakers.record_failure(&server_id).await;
                if let Some(ref rec) = task_record {
                    let _ = self
                        .state
                        .task_registry
                        .fail_task(
                            &rec.task_id,
                            serde_json::json!({"code": "UPSTREAM_TIMEOUT"}),
                            Some(format!(
                                "Tool call timed out after {}ms",
                                self.state.tool_timeout_ms
                            )),
                        )
                        .await;
                }
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "UPSTREAM_TIMEOUT",
                        format!("Tool call timed out after {}ms", self.state.tool_timeout_ms),
                        true,
                    ),
                    retry_base("unknown"),
                )
            }
            Ok(Err(UpstreamCallError::Upstream(err))) => {
                self.state.circuit_breakers.record_failure(&server_id).await;
                if let Some(ref rec) = task_record {
                    let _ = self
                        .state
                        .task_registry
                        .fail_task(
                            &rec.task_id,
                            serde_json::json!({"code": "UPSTREAM_ERROR", "message": &err}),
                            Some(err.clone()),
                        )
                        .await;
                }
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new("UPSTREAM_ERROR", err, false),
                    retry_base("unknown"),
                )
            }
            Err(_) => {
                self.state.circuit_breakers.record_failure(&server_id).await;
                if let Some(ref rec) = task_record {
                    let _ = self
                        .state
                        .task_registry
                        .fail_task(
                            &rec.task_id,
                            serde_json::json!({"code": "INTERNAL_ERROR"}),
                            Some("Server actor task dropped reply channel".to_string()),
                        )
                        .await;
                }
                self.state
                    .idempotency_store
                    .release_execution(idempotency_key.as_deref(), idempotency_claim)
                    .await;
                Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "INTERNAL_ERROR",
                        "Server actor task dropped reply channel",
                        true,
                    ),
                    retry_base("unknown"),
                )
            }
        }
    }

    /// Executes an ordered sequence of tool execution steps, interpolating prior step outputs.
    pub async fn batch_call(
        &self,
        steps: Vec<BatchStep>,
        request_id: Option<String>,
        context: Option<RequestContext>,
        profile: Option<&str>,
    ) -> BatchCallResponse {
        let trace_id = next_trace_id();
        let prof_ctx = self.resolve_profile_context(profile).await;
        let policy = self
            .state
            .policy
            .read()
            .await
            .merge_with_profile(prof_ctx.profile_policy.as_ref());

        execute_batch(
            &self.state,
            steps,
            trace_id,
            request_id,
            context,
            &policy,
            &prof_ctx,
        )
        .await
    }

    /// Reads a registered resource URI.
    pub async fn read_resource(
        &self,
        resource_id: &str,
        options: ReadResourceOptions,
    ) -> Envelope<Value> {
        let prof_ctx = self
            .resolve_profile_context(options.profile.as_deref())
            .await;
        let trace_id = next_trace_id();
        let req_id = options.request_id.unwrap_or_else(|| trace_id.clone());
        let req_ctx = options.context.unwrap_or_default();

        if !self.state.policy.read().await.allows(resource_id) {
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new(
                    "INVALID_ARGS",
                    format!("Resource '{}' blocked by policy", resource_id),
                    false,
                ),
                RetryMetadata::safe("not_started"),
            );
        }

        let (server, uri) = {
            let res_guard = self.state.resources.read().await;
            let Some(meta) = res_guard.get(resource_id) else {
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "RESOURCE_NOT_FOUND",
                        format!("Resource '{}' not found", resource_id),
                        false,
                    ),
                    RetryMetadata::safe("not_started"),
                );
            };

            if !prof_ctx.is_server_allowed(&meta.server) {
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "RESOURCE_NOT_IN_PROFILE",
                        format!(
                            "Resource '{}' belongs to server '{}' which is not in active profile",
                            resource_id, meta.server
                        ),
                        false,
                    ),
                    RetryMetadata::safe("not_started"),
                );
            }

            (meta.server.clone(), meta.uri.clone())
        };

        self.dispatch_catalog_call(
            server,
            trace_id,
            req_id,
            req_ctx,
            "Resource read",
            |reply| ServerMsg::ReadResource {
                uri,
                input_responses: options.input_responses,
                request_state: options.request_state,
                reply,
            },
        )
        .await
    }

    /// Renders a registered prompt template.
    pub async fn get_prompt(&self, prompt_id: &str, options: GetPromptOptions) -> Envelope<Value> {
        let prof_ctx = self
            .resolve_profile_context(options.profile.as_deref())
            .await;
        let trace_id = next_trace_id();
        let req_id = options.request_id.unwrap_or_else(|| trace_id.clone());
        let req_ctx = options.context.unwrap_or_default();

        if !self.state.policy.read().await.allows(prompt_id) {
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new(
                    "INVALID_ARGS",
                    format!("Prompt '{}' blocked by policy", prompt_id),
                    false,
                ),
                RetryMetadata::safe("not_started"),
            );
        }

        let (server, name) = {
            let prompts_guard = self.state.prompts.read().await;
            let Some(meta) = prompts_guard.get(prompt_id) else {
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "PROMPT_NOT_FOUND",
                        format!("Prompt '{}' not found", prompt_id),
                        false,
                    ),
                    RetryMetadata::safe("not_started"),
                );
            };

            if !prof_ctx.is_server_allowed(&meta.server) {
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "PROMPT_NOT_IN_PROFILE",
                        format!(
                            "Prompt '{}' belongs to server '{}' which is not in active profile",
                            prompt_id, meta.server
                        ),
                        false,
                    ),
                    RetryMetadata::safe("not_started"),
                );
            }

            (meta.server.clone(), meta.name.clone())
        };

        let arguments = options
            .arguments
            .and_then(|value| value.as_object().cloned());
        self.dispatch_catalog_call(server, trace_id, req_id, req_ctx, "Prompt get", |reply| {
            ServerMsg::GetPrompt {
                name,
                arguments,
                input_responses: options.input_responses,
                request_state: options.request_state,
                reply,
            }
        })
        .await
    }

    async fn dispatch_catalog_call(
        &self,
        server: String,
        trace_id: String,
        req_id: String,
        req_ctx: RequestContext,
        operation: &str,
        message: impl FnOnce(
            oneshot::Sender<std::result::Result<Value, UpstreamCallError>>,
        ) -> ServerMsg,
    ) -> Envelope<Value> {
        let tx = {
            let servers_guard = self.state.servers.read().await;
            let Some(tx) = servers_guard.get(&server).cloned() else {
                return Envelope::failure(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    WarmplaneError::new(
                        "SERVER_UNREACHABLE",
                        format!("Server '{}' is unreachable", server),
                        true,
                    ),
                    RetryMetadata::safe("not_started"),
                );
            };
            tx
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx.send(message(reply_tx)).await.is_err() {
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new(
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' mailbox is closed", server),
                    true,
                ),
                RetryMetadata::safe("not_started"),
            );
        }

        match reply_rx.await {
            Ok(Ok(data)) => Envelope::success(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                data,
                RetryMetadata::safe("completed"),
            ),
            Ok(Err(UpstreamCallError::Timeout)) => Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new(
                    "UPSTREAM_TIMEOUT",
                    format!(
                        "{operation} timed out after {}ms",
                        self.state.tool_timeout_ms
                    ),
                    true,
                ),
                RetryMetadata::safe("unknown"),
            ),
            Ok(Err(UpstreamCallError::Upstream(err))) => Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new("UPSTREAM_ERROR", err, false),
                RetryMetadata::safe("unknown"),
            ),
            Err(_) => Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new("INTERNAL_ERROR", "Server actor task died", true),
                RetryMetadata::safe("unknown"),
            ),
        }
    }

    /// Lists all active and historical tasks in the task registry.
    ///
    /// # Returns
    /// A vector of [`TaskResponse`] wire representations sorted from newest to oldest.
    pub async fn list_tasks(&self) -> Vec<TaskResponse> {
        let tasks = self.state.task_registry.list_tasks().await;
        tasks.iter().map(TaskResponse::from).collect()
    }

    /// Retrieves a task by its task ID, performing TTL checks.
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier.
    ///
    /// # Returns
    /// An [`Envelope`] containing the [`TaskResponse`] if found, or an error envelope if not found.
    pub async fn get_task(&self, task_id: &str) -> Envelope<TaskResponse> {
        let trace_id = next_trace_id();
        if let Some(record) = self.state.task_registry.get_task(task_id).await {
            Envelope::success(
                trace_id,
                record.request_id.clone(),
                record.context.clone(),
                TaskResponse::from(&record),
                RetryMetadata::safe("completed"),
            )
        } else {
            Envelope::failure(
                trace_id,
                None,
                None,
                WarmplaneError::new(
                    "TASK_NOT_FOUND",
                    format!("Task '{}' not found", task_id),
                    false,
                ),
                RetryMetadata::safe("not_started"),
            )
        }
    }

    /// Resolves an `InputRequired` task by submitting client input responses.
    ///
    /// # Arguments
    /// * `task_id` - Target task identifier.
    /// * `input_responses` - Map of input keys to response values (e.g. `{"hitl_approval": {"approved": true}}`).
    ///
    /// # Returns
    /// An [`Envelope`] indicating whether the update succeeded.
    pub async fn update_task(
        &self,
        task_id: &str,
        input_responses: std::collections::BTreeMap<String, Value>,
    ) -> Envelope<bool> {
        let trace_id = next_trace_id();
        match self
            .state
            .task_registry
            .update_task(task_id, input_responses)
            .await
        {
            Ok(true) => {
                Envelope::success(trace_id, None, None, true, RetryMetadata::safe("completed"))
            }
            Ok(false) => Envelope::failure(
                trace_id,
                None,
                None,
                WarmplaneError::new(
                    "TASK_UPDATE_REJECTED",
                    format!(
                        "Task '{}' is not in 'input_required' status or does not exist",
                        task_id
                    ),
                    false,
                ),
                RetryMetadata::safe("not_started"),
            ),
            Err(e) => Envelope::failure(
                trace_id,
                None,
                None,
                WarmplaneError::new("INTERNAL_ERROR", e.to_string(), false),
                RetryMetadata::safe("unknown"),
            ),
        }
    }

    /// Cooperatively cancels an in-progress or suspended task.
    ///
    /// # Arguments
    /// * `task_id` - Target task identifier.
    /// * `reason` - Optional reason for cancellation.
    ///
    /// # Returns
    /// An [`Envelope`] indicating whether the task was cancelled.
    pub async fn cancel_task(&self, task_id: &str, reason: Option<String>) -> Envelope<bool> {
        let trace_id = next_trace_id();
        match self
            .state
            .hold_coordinator
            .cancel(&self.state, task_id, reason.clone())
            .await
        {
            Ok(crate::tasks::EmissionCancelResult::Cancelled)
            | Ok(crate::tasks::EmissionCancelResult::AlreadyCancelled) => {
                return Envelope::success(
                    trace_id,
                    None,
                    None,
                    true,
                    RetryMetadata::safe("completed"),
                );
            }
            Ok(crate::tasks::EmissionCancelResult::TooLate) => {
                return Envelope::failure(
                    trace_id,
                    None,
                    None,
                    WarmplaneError::new(
                        "EMISSION_DISPATCHING",
                        "Emission dispatch has already started and may have reached the upstream server",
                        false,
                    ),
                    RetryMetadata::safe("not_started"),
                );
            }
            Ok(crate::tasks::EmissionCancelResult::NotEmission) => {}
            Err(error) => {
                return Envelope::failure(
                    trace_id,
                    None,
                    None,
                    WarmplaneError::new("TASK_CANCEL_FAILED", error.to_string(), false),
                    RetryMetadata::safe("not_started"),
                );
            }
        }
        match self.state.task_registry.cancel_task(task_id, reason).await {
            Ok(true) => {
                Envelope::success(trace_id, None, None, true, RetryMetadata::safe("completed"))
            }
            Ok(false) => Envelope::failure(
                trace_id,
                None,
                None,
                WarmplaneError::new(
                    "TASK_CANCEL_REJECTED",
                    format!(
                        "Task '{}' is already completed, failed, cancelled, or not found",
                        task_id
                    ),
                    false,
                ),
                RetryMetadata::safe("not_started"),
            ),
            Err(e) => Envelope::failure(
                trace_id,
                None,
                None,
                WarmplaneError::new("INTERNAL_ERROR", e.to_string(), false),
                RetryMetadata::safe("unknown"),
            ),
        }
    }

    /// Gracefully initiates shutdown of all background supervisors and actor channels.
    pub async fn shutdown(&self) {
        self.state.shutdown().await;
    }
}
