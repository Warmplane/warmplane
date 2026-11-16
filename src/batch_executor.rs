// Rust guideline compliant 2026-11-16
//! Multi-step chained and batched capability execution with variable reference resolution (`M-CANONICAL-DOCS`).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::daemon::{AppState, ServerMsg, UpstreamCallError};

/// Single execution step within a batch capability call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStep {
    /// Unique step identifier within the batch (e.g. `"step1"`, `"lookup"`).
    pub id: String,
    /// Capability identifier to invoke.
    pub capability_id: String,
    /// JSON arguments for capability execution (may contain references like `"$step1.field"`).
    pub args: Value,
    /// Whether to continue executing subsequent steps if this step fails. Default `false`.
    #[serde(default)]
    pub continue_on_error: bool,
}

/// Request payload for batch capability execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCallRequest {
    /// Ordered list of steps to execute sequentially.
    pub steps: Vec<BatchStep>,
    /// Optional overall request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
}

/// Result of an individual step within a batch execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStepResult {
    /// Step identifier matching `BatchStep::id`.
    pub id: String,
    /// Capability invoked.
    pub capability_id: String,
    /// Whether execution succeeded.
    pub ok: bool,
    /// Execution output on success.
    pub data: Option<Value>,
    /// Error description if step failed.
    pub error: Option<String>,
    /// Step latency in microseconds.
    pub duration_us: u64,
}

/// Summary response envelope returned by batch capability execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCallResponse {
    /// Overall batch status.
    pub ok: bool,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Execution trace identifier.
    pub trace_id: String,
    /// Results for each step in execution order.
    pub results: Vec<BatchStepResult>,
    /// Total duration for the entire batch in microseconds.
    pub total_duration_us: u64,
}

/// Maximum number of sequential steps permitted in a single batch request.
pub const MAX_BATCH_STEPS: usize = 50;

/// Default overall execution budget for a multi-step batch (60 seconds).
pub const DEFAULT_BATCH_TIMEOUT_MS: u64 = 60_000;

/// Policy and tenant scope for batch execution.
pub struct BatchExecutionScope<'a> {
    /// Effective policy applied to each batch step.
    pub policy: &'a crate::daemon::Policy,
    /// Active profile and server boundaries.
    pub profile: &'a crate::context::ProfileContext,
    /// Tenant used to scope persisted compensable usage.
    pub tenant_id: &'a str,
    /// Authenticated role used to revalidate delayed batch steps.
    pub role: &'a str,
    /// JWT expiry bound for release-time authorization revalidation.
    pub authorization_expires_at_epoch_secs: Option<u64>,
    /// Token cancelled when the batch operation is explicitly cancelled.
    pub cancellation_token: CancellationToken,
}

/// Executes an ordered sequence of tool execution steps, interpolating prior step outputs.
///
/// # Arguments
/// * `state` - Daemon application state.
/// * `steps` - Ordered vector of `BatchStep` items.
/// * `trace_id` - Trace ID for the batch.
/// * `request_id` - Optional request ID.
/// * `context` - Optional request context.
/// * `policy` - Effective security policy.
///
/// # Returns
/// A `BatchCallResponse` summarizing all step executions.
pub async fn execute_batch(
    state: &AppState,
    steps: Vec<BatchStep>,
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    policy: &crate::daemon::Policy,
    prof_ctx: &crate::context::ProfileContext,
) -> BatchCallResponse {
    execute_batch_for_tenant(
        state,
        steps,
        trace_id,
        request_id,
        context,
        BatchExecutionScope {
            policy,
            profile: prof_ctx,
            tenant_id: "default",
            role: state.rbac_engine.default_role(),
            authorization_expires_at_epoch_secs: None,
            cancellation_token: CancellationToken::new(),
        },
    )
    .await
}

/// Executes a batch with an explicit tenant scope for persisted undo accounting.
pub async fn execute_batch_for_tenant(
    state: &AppState,
    steps: Vec<BatchStep>,
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    scope: BatchExecutionScope<'_>,
) -> BatchCallResponse {
    let BatchExecutionScope {
        policy,
        profile: prof_ctx,
        tenant_id,
        role,
        authorization_expires_at_epoch_secs,
        cancellation_token,
    } = scope;
    let start_all = std::time::Instant::now();

    if steps.len() > MAX_BATCH_STEPS {
        return BatchCallResponse {
            trace_id,
            request_id,
            ok: false,
            results: vec![BatchStepResult {
                id: "batch_validation".to_string(),
                capability_id: "batch".to_string(),
                ok: false,
                data: None,
                error: Some(format!(
                    "Batch exceeds maximum allowed steps of {} (requested {})",
                    MAX_BATCH_STEPS,
                    steps.len()
                )),
                duration_us: 0,
            }],
            total_duration_us: 0,
        };
    }

    let mut step_outputs: HashMap<String, Value> = HashMap::new();
    let mut results = Vec::new();
    let mut overall_ok = true;
    let fallback_context = crate::context::RequestContext::default();

    for step in steps {
        let step_start = std::time::Instant::now();

        // Check aggregate batch timeout budget
        if cancellation_token.is_cancelled() {
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some("Batch execution was cancelled".to_string()),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            break;
        }
        if start_all.elapsed().as_millis() as u64 >= DEFAULT_BATCH_TIMEOUT_MS {
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some(format!(
                    "Batch execution timed out after {}ms budget",
                    DEFAULT_BATCH_TIMEOUT_MS
                )),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            break;
        }

        let mut interpolated_args = interpolate_step_references(&step.args, &step_outputs);

        // Check policy allow/deny
        if !policy.allows(&step.capability_id) {
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some(format!(
                    "Capability '{}' blocked by policy",
                    step.capability_id
                )),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            if !step.continue_on_error {
                break;
            }
            continue;
        }

        // Lookup capability target
        let (server, tool, reversibility) = {
            let caps_guard = state.capabilities.read().await;
            let Some(meta) = caps_guard.get(&step.capability_id) else {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(format!("Capability '{}' not found", step.capability_id)),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
            };

            if !prof_ctx.is_server_allowed(&meta.server) {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(format!(
                        "Capability '{}' belongs to server '{}' which is not in active profile",
                        step.capability_id, meta.server
                    )),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
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
            if state.undo_ledger.is_none() {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Persistent undo storage is unavailable".to_string()),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
            }
            match crate::engine::undo::reserve_compensable(
                state,
                contract,
                &step.capability_id,
                tenant_id,
                prof_ctx.profile_id.as_deref(),
                None,
                false,
            )
            .await
            {
                Ok(true) => compensable_reserved = true,
                Ok(false) => {}
                Err(error) => {
                    results.push(BatchStepResult {
                        id: step.id.clone(),
                        capability_id: step.capability_id.clone(),
                        ok: false,
                        data: None,
                        error: Some(error.to_string()),
                        duration_us: step_start.elapsed().as_micros() as u64,
                    });
                    overall_ok = false;
                    if !step.continue_on_error {
                        break;
                    }
                    continue;
                }
            }
        }

        // Enforce HITL approval gate if required
        let (mut requires_approval, redact_keys, approval_timeout_secs, webhook_cfg) = (
            policy.requires_approval(&step.capability_id),
            policy.redact_keys.clone(),
            policy.approval_timeout_secs,
            policy.webhook.clone(),
        );

        if reversibility.as_ref().is_some_and(|contract| {
            contract.class == crate::config::ReversibilityClass::Compensable
        }) && !compensable_reserved
        {
            requires_approval = true;
        }

        if requires_approval {
            let sanitized =
                crate::http_v1::helpers::redact_value(interpolated_args.clone(), &redact_keys);
            let (approval_id, rx) = state
                .approval_registry
                .create_approval(crate::approvals::CreateApprovalRequest {
                    capability_id: step.capability_id.clone(),
                    server_id: server.clone(),
                    args: interpolated_args.clone(),
                    sanitized_args: sanitized.clone(),
                    request_id: request_id.clone(),
                    context: context.clone(),
                    timeout_secs: approval_timeout_secs,
                    webhook: webhook_cfg.as_ref(),
                })
                .await;

            state.audit_handle.send(crate::audit::RawAuditEvent {
                event_type: crate::audit::AuditEventType::ToolInterceptedHitl,
                trace_id: trace_id.clone(),
                request_id: request_id.clone(),
                actor_id: context.as_ref().and_then(|c| c.actor_id.clone()),
                work_item_id: context.as_ref().and_then(|c| c.work_item_id.clone()),
                client_ip: None,
                server_id: Some(server.clone()),
                capability_id: Some(step.capability_id.clone()),
                resource_uri: None,
                sanitized_args: Some(sanitized),
                sanitized_response: None,
                execution_latency_us: Some(step_start.elapsed().as_micros() as u64),
                status: crate::audit::AuditEventStatus::Intercepted,
                error_code: None,
                error_message: None,
                operator_id: None,
                approval_ticket_id: Some(approval_id),
                idempotency_key: None,
                is_replay: None,
            });

            let approval_budget_ms =
                DEFAULT_BATCH_TIMEOUT_MS.saturating_sub(start_all.elapsed().as_millis() as u64);
            let resolution = tokio::select! {
                _ = cancellation_token.cancelled() => crate::approvals::ApprovalResolution::Expired,
                result = tokio::time::timeout(Duration::from_millis(approval_budget_ms), rx) => {
                    match result {
                        Ok(Ok(resolution)) => resolution,
                        _ => crate::approvals::ApprovalResolution::Expired,
                    }
                }
            };

            match resolution {
                crate::approvals::ApprovalResolution::Approved { modified_args, .. } => {
                    if let Some(mod_args) = modified_args {
                        interpolated_args = mod_args;
                    }
                }
                crate::approvals::ApprovalResolution::Rejected { reason, operator } => {
                    results.push(BatchStepResult {
                        id: step.id.clone(),
                        capability_id: step.capability_id.clone(),
                        ok: false,
                        data: None,
                        error: Some(format!(
                            "HITL approval rejected by operator '{}': {}",
                            operator,
                            reason.unwrap_or_else(|| "No reason provided".to_string())
                        )),
                        duration_us: step_start.elapsed().as_micros() as u64,
                    });
                    overall_ok = false;
                    if !step.continue_on_error {
                        break;
                    }
                    continue;
                }
                crate::approvals::ApprovalResolution::Expired => {
                    results.push(BatchStepResult {
                        id: step.id.clone(),
                        capability_id: step.capability_id.clone(),
                        ok: false,
                        data: None,
                        error: Some(if cancellation_token.is_cancelled() {
                            "Batch execution was cancelled while waiting for HITL approval"
                                .to_string()
                        } else if start_all.elapsed().as_millis() as u64 >= DEFAULT_BATCH_TIMEOUT_MS
                        {
                            "Batch execution budget expired while waiting for HITL approval"
                                .to_string()
                        } else {
                            "HITL approval expired".to_string()
                        }),
                        duration_us: step_start.elapsed().as_micros() as u64,
                    });
                    overall_ok = false;
                    if !step.continue_on_error {
                        break;
                    }
                    continue;
                }
            }
        }

        if requires_approval && !compensable_reserved {
            if let Some(contract) = reversibility
                .as_ref()
                .filter(|contract| contract.class == crate::config::ReversibilityClass::Compensable)
            {
                if let Err(error) = crate::engine::undo::reserve_compensable(
                    state,
                    contract,
                    &step.capability_id,
                    tenant_id,
                    prof_ctx.profile_id.as_deref(),
                    None,
                    true,
                )
                .await
                {
                    results.push(BatchStepResult {
                        id: step.id.clone(),
                        capability_id: step.capability_id.clone(),
                        ok: false,
                        data: None,
                        error: Some(error.to_string()),
                        duration_us: step_start.elapsed().as_micros() as u64,
                    });
                    overall_ok = false;
                    if !step.continue_on_error {
                        break;
                    }
                    continue;
                }
            }
        }

        // Circuit breaker check
        if let Err(cb_err) = state.circuit_breakers.check_permission(&server).await {
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some(format!("Circuit open for server '{}': {}", server, cb_err)),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            if !step.continue_on_error {
                break;
            }
            continue;
        }

        if let Some(contract) = reversibility
            .as_ref()
            .filter(|contract| contract.class == crate::config::ReversibilityClass::Delayable)
        {
            let hold_secs = crate::emissions::effective_hold_window_secs(
                contract,
                prof_ctx
                    .profile_policy
                    .as_ref()
                    .and_then(|item| item.hold_window_secs),
                state.policy.read().await.hold_window_secs,
            )
            .unwrap_or(30);
            let release_at_ms = match crate::emissions::release_deadline_ms(hold_secs) {
                Ok(deadline) => deadline,
                Err(error) => {
                    results.push(BatchStepResult {
                        id: step.id.clone(),
                        capability_id: step.capability_id.clone(),
                        ok: false,
                        data: None,
                        error: Some(error.to_string()),
                        duration_us: step_start.elapsed().as_micros() as u64,
                    });
                    overall_ok = false;
                    if !step.continue_on_error {
                        break;
                    }
                    continue;
                }
            };
            let emission = crate::tasks::EmissionRecord {
                phase: crate::tasks::EmissionPhase::PendingEmission,
                release_at_ms,
                cancellable: true,
                tool_name: tool.clone(),
                input_responses: None,
                request_state: None,
                profile_id: prof_ctx.profile_id.clone(),
                tenant_id: tenant_id.to_string(),
                role: role.to_string(),
                authorization_expires_at_epoch_secs,
                approval_completed: requires_approval,
                batch_id: request_id.clone(),
                step_id: Some(step.id.clone()),
            };
            let task = state
                .hold_coordinator
                .accept(
                    state,
                    crate::tasks::CreateTaskParams {
                        capability_id: step.capability_id.clone(),
                        server_id: server.clone(),
                        args: interpolated_args.clone(),
                        request_id: request_id.clone(),
                        context: context.clone(),
                        idempotency_key: request_id
                            .as_ref()
                            .map(|batch_id| format!("batch:{batch_id}:step:{}", step.id)),
                        initial_status: crate::tasks::TaskStatus::Working,
                        status_message: Some(
                            "Waiting for the batch emission release deadline".to_string(),
                        ),
                        input_requests: None,
                        ttl_ms: None,
                        poll_interval_ms: Some(500),
                    },
                    emission,
                )
                .await;
            let task = match task {
                Ok(task) => task,
                Err(error) => {
                    results.push(BatchStepResult {
                        id: step.id.clone(),
                        capability_id: step.capability_id.clone(),
                        ok: false,
                        data: None,
                        error: Some(format!("Could not persist held emission: {error}")),
                        duration_us: step_start.elapsed().as_micros() as u64,
                    });
                    overall_ok = false;
                    if !step.continue_on_error {
                        break;
                    }
                    continue;
                }
            };
            let remaining_ms =
                DEFAULT_BATCH_TIMEOUT_MS.saturating_sub(start_all.elapsed().as_millis() as u64);
            let terminal = tokio::select! {
                _ = cancellation_token.cancelled() => None,
                result = state.hold_coordinator.wait_for_terminal(
                    state,
                    &task.task_id,
                    Duration::from_millis(remaining_ms),
                ) => result,
            };
            let Some(terminal) = terminal else {
                let cancellation = state
                    .hold_coordinator
                    .cancel(
                        state,
                        &task.task_id,
                        Some("Batch execution cancelled".to_string()),
                    )
                    .await;
                let message = match cancellation {
                    Ok(
                        crate::tasks::EmissionCancelResult::Cancelled
                        | crate::tasks::EmissionCancelResult::AlreadyCancelled,
                    ) => {
                        "Batch cancelled; held emission was prevented from dispatching".to_string()
                    }
                    _ => "Batch cancelled after dispatch began; emission outcome may be pending"
                        .to_string(),
                };
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(message),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                break;
            };
            if terminal.status == crate::tasks::TaskStatus::Completed {
                let value = terminal.result.unwrap_or(Value::Null);
                state.circuit_breakers.record_success(&server).await;
                let sanitized_args =
                    crate::http_v1::helpers::redact_value(interpolated_args.clone(), &redact_keys);
                let sanitized_response =
                    crate::http_v1::helpers::redact_value(value.clone(), &redact_keys);
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ToolExecution,
                    trace_id: trace_id.clone(),
                    request_id: request_id.clone(),
                    actor_id: context.as_ref().and_then(|item| item.actor_id.clone()),
                    work_item_id: context.as_ref().and_then(|item| item.work_item_id.clone()),
                    client_ip: None,
                    server_id: Some(server.clone()),
                    capability_id: Some(step.capability_id.clone()),
                    resource_uri: None,
                    sanitized_args: Some(sanitized_args),
                    sanitized_response: Some(
                        serde_json::json!({"taskId": task.task_id, "result": sanitized_response}),
                    ),
                    execution_latency_us: Some(step_start.elapsed().as_micros() as u64),
                    status: crate::audit::AuditEventStatus::Success,
                    error_code: None,
                    error_message: None,
                    operator_id: None,
                    approval_ticket_id: None,
                    idempotency_key: request_id
                        .as_ref()
                        .map(|batch_id| format!("batch:{batch_id}:step:{}", step.id)),
                    is_replay: Some(false),
                });
                step_outputs.insert(step.id.clone(), value.clone());
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: true,
                    data: Some(value),
                    error: None,
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                continue;
            }
            if !matches!(
                terminal.status,
                crate::tasks::TaskStatus::Failed | crate::tasks::TaskStatus::Cancelled
            ) {
                let cancellation = state
                    .hold_coordinator
                    .cancel(
                        state,
                        &task.task_id,
                        Some("Batch execution budget expired".to_string()),
                    )
                    .await;
                let message = match cancellation {
                    Ok(
                        crate::tasks::EmissionCancelResult::Cancelled
                        | crate::tasks::EmissionCancelResult::AlreadyCancelled,
                    ) => "Batch budget expired; held emission was cancelled before dispatch"
                        .to_string(),
                    _ => {
                        "Batch budget expired after dispatch began; emission outcome may be pending"
                            .to_string()
                    }
                };
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(message),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
            } else {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: terminal.status_message,
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
            }
            overall_ok = false;
            if !step.continue_on_error {
                break;
            }
            continue;
        }

        let (snapshot, snapshot_hash) = if let Some(contract) = reversibility.as_ref() {
            if matches!(
                contract.class,
                crate::config::ReversibilityClass::Reversible
                    | crate::config::ReversibilityClass::Compensable
            ) && state.undo_ledger.is_none()
            {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Persistent undo storage is unavailable".to_string()),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
            }
            if let Some(pre_snapshot) = &contract.pre_snapshot {
                match crate::engine::undo::capture_pre_snapshot(
                    state,
                    pre_snapshot,
                    &interpolated_args,
                    prof_ctx,
                    policy,
                )
                .await
                {
                    Ok(value) => value,
                    Err(error) => {
                        results.push(BatchStepResult {
                            id: step.id.clone(),
                            capability_id: step.capability_id.clone(),
                            ok: false,
                            data: None,
                            error: Some(format!("Pre-snapshot failed: {error}")),
                            duration_us: step_start.elapsed().as_micros() as u64,
                        });
                        overall_ok = false;
                        if !step.continue_on_error {
                            break;
                        }
                        continue;
                    }
                }
            } else {
                (Value::Null, None)
            }
        } else {
            (Value::Null, None)
        };

        // Lookup server sender channel
        let tx = {
            let servers_guard = state.servers.read().await;
            let Some(tx) = servers_guard.get(&server).cloned() else {
                state.circuit_breakers.record_failure(&server).await;
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(format!("Server '{}' unreachable", server)),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
            };
            tx
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        let send_res = tx
            .send(ServerMsg::CallTool {
                name: tool,
                params: interpolated_args.clone(),
                input_responses: None,
                request_state: None,
                reply: reply_tx,
            })
            .await;

        if send_res.is_err() {
            state.circuit_breakers.record_failure(&server).await;
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some(format!("Server '{}' mailbox closed", server)),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            if !step.continue_on_error {
                break;
            }
            continue;
        }

        match reply_rx.await {
            Ok(Ok(val)) => {
                state.circuit_breakers.record_success(&server).await;
                if let Some(contract) = reversibility.as_ref() {
                    if let Err(error) = crate::engine::undo::register_inverse(
                        state,
                        contract,
                        crate::engine::undo::ForwardCall {
                            capability: &step.capability_id,
                            args: &interpolated_args,
                            result: &val,
                            snapshot: &snapshot,
                            snapshot_hash,
                            context: context.as_ref().unwrap_or(&fallback_context),
                            tenant_id,
                            profile_id: prof_ctx.profile_id.as_deref(),
                        },
                    )
                    .await
                    {
                        results.push(BatchStepResult {
                            id: step.id.clone(),
                            capability_id: step.capability_id.clone(),
                            ok: false,
                            data: None,
                            error: Some(format!(
                                "Forward call succeeded but inverse registration failed: {error}"
                            )),
                            duration_us: step_start.elapsed().as_micros() as u64,
                        });
                        overall_ok = false;
                        if !step.continue_on_error {
                            break;
                        }
                        continue;
                    }
                }
                let sanitized_res =
                    crate::http_v1::helpers::redact_value(val.clone(), &redact_keys);
                let sanitized_args =
                    crate::http_v1::helpers::redact_value(interpolated_args, &redact_keys);
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ToolExecution,
                    trace_id: trace_id.clone(),
                    request_id: request_id.clone(),
                    actor_id: context.as_ref().and_then(|c| c.actor_id.clone()),
                    work_item_id: context.as_ref().and_then(|c| c.work_item_id.clone()),
                    client_ip: None,
                    server_id: Some(server),
                    capability_id: Some(step.capability_id.clone()),
                    resource_uri: None,
                    sanitized_args: Some(sanitized_args),
                    sanitized_response: Some(sanitized_res),
                    execution_latency_us: Some(step_start.elapsed().as_micros() as u64),
                    status: crate::audit::AuditEventStatus::Success,
                    error_code: None,
                    error_message: None,
                    operator_id: None,
                    approval_ticket_id: None,
                    idempotency_key: None,
                    is_replay: Some(false),
                });

                step_outputs.insert(step.id.clone(), val.clone());
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: true,
                    data: Some(val),
                    error: None,
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
            }
            Ok(Err(UpstreamCallError::Timeout)) => {
                state.circuit_breakers.record_failure(&server).await;
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Tool execution timed out".to_string()),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
            }
            Ok(Err(UpstreamCallError::Upstream(err))) => {
                state.circuit_breakers.record_failure(&server).await;
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(err),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
            }
            Err(_) => {
                state.circuit_breakers.record_failure(&server).await;
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Daemon actor task died".to_string()),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
            }
        }
    }

    BatchCallResponse {
        ok: overall_ok,
        request_id,
        trace_id,
        results,
        total_duration_us: start_all.elapsed().as_micros() as u64,
    }
}

/// Recursively traverses a JSON Value, replacing any `$step_id.path` string references
/// with corresponding values extracted from previous completed steps.
pub fn interpolate_step_references(val: &Value, outputs: &HashMap<String, Value>) -> Value {
    crate::engine::interpolation::interpolate_step_references(val, outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_reference_interpolation() {
        let mut outputs = HashMap::new();
        outputs.insert(
            "step1".to_string(),
            json!({
                "user_id": 42,
                "profile": {
                    "username": "alice"
                }
            }),
        );

        let input_args = json!({
            "target_id": "$step1.user_id",
            "username": "$step1.profile.username",
            "literal": "hello"
        });

        let resolved = interpolate_step_references(&input_args, &outputs);
        assert_eq!(resolved["target_id"], 42);
        assert_eq!(resolved["username"], "alice");
        assert_eq!(resolved["literal"], "hello");
    }
}
