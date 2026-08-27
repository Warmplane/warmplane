// Rust guideline compliant 2026-08-27

//! Embedded Warmplane engine interface and ControlPlaneHandle implementation (`M-CANONICAL-DOCS`).
//!
//! Provides direct Rust callable APIs without HTTP or transport serialization overhead:
//! - [`EmbeddedWarmplane`]: In-process entry point to boot and supervise upstream MCP sessions.
//! - [`ControlPlaneHandle`]: Cloneable, `Send + Sync` handle to query catalogs, invoke capabilities, and run batches.

pub mod types;

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
                );
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
                tags: meta.tags.clone(),
            })
            .collect::<Vec<_>>();

        capabilities.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(CapabilitiesListResponse {
            version: "v1".to_string(),
            capabilities,
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

        // Check Idempotency Store
        if let Some(ref key) = idempotency_key {
            match self
                .state
                .idempotency_store
                .check_or_start_with_meta(
                    key,
                    Some(capability_id.to_string()),
                    Some(trace_id.clone()),
                )
                .await
            {
                crate::idempotency::DeduplicateResult::Completed(cached) => {
                    if let Ok(env) = serde_json::from_value::<Envelope<Value>>(cached.clone()) {
                        return env;
                    }
                    return Envelope::success(
                        trace_id,
                        Some(req_id),
                        Some(req_ctx),
                        cached,
                        retry_base("completed"),
                    );
                }
                crate::idempotency::DeduplicateResult::InProgress(mut rx) => {
                    if let Ok(cached) = rx.recv().await {
                        if let Ok(env) = serde_json::from_value::<Envelope<Value>>(cached.clone()) {
                            return env;
                        }
                        return Envelope::success(
                            trace_id,
                            Some(req_id),
                            Some(req_ctx),
                            cached,
                            retry_base("completed"),
                        );
                    }
                }
                crate::idempotency::DeduplicateResult::New => {}
            }
        }

        if !args.is_object() {
            if let Some(ref key) = idempotency_key {
                self.state.idempotency_store.remove(key).await;
            }
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new("INVALID_ARGS", "'args' must be a JSON object", false),
                retry_base("not_started"),
            );
        }

        let (requires_approval, approval_timeout_secs, webhook_cfg, redact_keys) = {
            let pol = self.state.policy.read().await;
            if !pol.allows(capability_id) {
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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

        let (server_id, tool_name) = {
            let caps_guard = self.state.capabilities.read().await;
            let Some(meta) = caps_guard.get(capability_id) else {
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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

            (meta.server.clone(), meta.tool.clone())
        };

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
                                        if let Some(ref key) = idempotency_key_clone {
                                            handle.state.idempotency_store.remove(key).await;
                                        }
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
                                if let Some(ref key) = idempotency_key_clone {
                                    handle.state.idempotency_store.remove(key).await;
                                }
                                return;
                            }
                        }
                    }
                }

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
                            if let Some(ref key) = idempotency_key_clone {
                                handle.state.idempotency_store.remove(key).await;
                            }
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
                    if let Some(ref key) = idempotency_key_clone {
                        handle.state.idempotency_store.remove(key).await;
                    }
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
                        let distilled_data = distill_value(data, &distill_opts);
                        let _ = handle
                            .state
                            .task_registry
                            .complete_task(&task_id_clone, distilled_data.clone())
                            .await;

                        let env = Envelope::success(
                            trace_id_clone,
                            Some(req_id_clone),
                            Some(req_ctx_clone),
                            distilled_data,
                            retry_base("completed"),
                        );
                        if let Some(ref key) = idempotency_key_clone {
                            if let Ok(val) = serde_json::to_value(&env) {
                                handle.state.idempotency_store.complete(key, val).await;
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
                        if let Some(ref key) = idempotency_key_clone {
                            handle.state.idempotency_store.remove(key).await;
                        }
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
                        if let Some(ref key) = idempotency_key_clone {
                            handle.state.idempotency_store.remove(key).await;
                        }
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
                        if let Some(ref key) = idempotency_key_clone {
                            handle.state.idempotency_store.remove(key).await;
                        }
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
                            if let Some(ref key) = idempotency_key {
                                self.state.idempotency_store.remove(key).await;
                            }
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
                            if let Some(ref key) = idempotency_key {
                                self.state.idempotency_store.remove(key).await;
                            }
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
                                if let Some(ref key) = idempotency_key {
                                    self.state.idempotency_store.remove(key).await;
                                }
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
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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
            if let Some(ref key) = idempotency_key {
                self.state.idempotency_store.remove(key).await;
            }
            return Envelope::failure(
                trace_id,
                Some(req_id),
                Some(req_ctx),
                WarmplaneError::new("CIRCUIT_OPEN", cb_err.to_string(), false),
                retry_base("not_started"),
            );
        }

        let distill_opts = DistillationOptions::from_args(Some(&args));
        let (reply_tx, reply_rx) = oneshot::channel();

        if tx
            .send(ServerMsg::CallTool {
                name: tool_name,
                params: effective_args,
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
            if let Some(ref key) = idempotency_key {
                self.state.idempotency_store.remove(key).await;
            }
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

                let distilled_data = distill_value(data, &distill_opts);
                if let Some(ref rec) = task_record {
                    let _ = self
                        .state
                        .task_registry
                        .complete_task(&rec.task_id, distilled_data.clone())
                        .await;
                }

                let env = Envelope::success(
                    trace_id,
                    Some(req_id),
                    Some(req_ctx),
                    distilled_data,
                    retry_base("completed"),
                );

                if let Some(ref key) = idempotency_key {
                    if let Ok(val) = serde_json::to_value(&env) {
                        self.state.idempotency_store.complete(key, val).await;
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
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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
                if let Some(ref key) = idempotency_key {
                    self.state.idempotency_store.remove(key).await;
                }
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
        let policy = self.state.policy.read().await.clone();

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
        if tx
            .send(ServerMsg::ReadResource {
                uri,
                input_responses: options.input_responses,
                request_state: options.request_state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
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
                        "Resource read timed out after {}ms",
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

        let args_map = options.arguments.and_then(|v| match v {
            Value::Object(m) => Some(m),
            _ => None,
        });

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::GetPrompt {
                name,
                arguments: args_map,
                input_responses: options.input_responses,
                request_state: options.request_state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
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
                        "Prompt get timed out after {}ms",
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
