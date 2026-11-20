// Rust guideline compliant 2026-11-18
//! Multi-step chained and batched capability execution with variable reference resolution (`M-CANONICAL-DOCS`).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::{
    approvals::ApprovalResolution,
    config::{ReversibilityClass, ReversibilityConfig},
    daemon::{AppState, ServerMsg, UpstreamCallError},
    tasks::{
        CreateTaskParams, EmissionCancelResult, EmissionPhase, EmissionRecord, TaskRecord,
        TaskStatus,
    },
};

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
    /// Automatically unwind successful reversible steps when the batch fails.
    #[serde(default, rename = "autoRollbackOnFailure")]
    pub auto_rollback_on_failure: bool,
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
    /// Per-entry outcomes when automatic rollback was requested and run.
    #[serde(default)]
    pub rollback_results: Vec<crate::undo::RollbackResult>,
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

struct BatchExecutionOptions {
    auto_rollback_on_failure: bool,
    batch_timeout_ms: u64,
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
    let default_role = state.rbac_engine.default_role();
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
            role: &default_role,
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
    execute_batch_for_tenant_with_rollback(
        state, steps, trace_id, request_id, context, scope, false,
    )
    .await
}

/// Executes a batch with optional transactional rollback after the first failed step.
///
/// When enabled, execution stops at the first failed step and attempts registered inverses for
/// prior successful steps. Existing callers can use `execute_batch_for_tenant` to preserve the
/// default behavior without automatic rollback.
///
/// # Arguments
/// * `state` - Active daemon state.
/// * `steps` - Ordered batch steps.
/// * `trace_id` - Trace identifier shared by the batch and rollback audit events.
/// * `request_id` - Batch identity used to select its undo entries.
/// * `context` - Caller and work-item metadata.
/// * `scope` - Tenant, policy, profile, and authorization context.
/// * `auto_rollback_on_failure` - Enables rollback after the first failed step.
///
/// # Returns
/// Step results and, when applicable, per-inverse rollback outcomes.
pub async fn execute_batch_for_tenant_with_rollback(
    state: &AppState,
    steps: Vec<BatchStep>,
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    scope: BatchExecutionScope<'_>,
    auto_rollback_on_failure: bool,
) -> BatchCallResponse {
    execute_batch_for_tenant_with_budget(
        state,
        steps,
        trace_id,
        request_id,
        context,
        scope,
        BatchExecutionOptions {
            auto_rollback_on_failure,
            batch_timeout_ms: DEFAULT_BATCH_TIMEOUT_MS,
        },
    )
    .await
}

async fn execute_batch_for_tenant_with_budget(
    state: &AppState,
    steps: Vec<BatchStep>,
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    scope: BatchExecutionScope<'_>,
    options: BatchExecutionOptions,
) -> BatchCallResponse {
    let start = Instant::now();
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
            rollback_results: Vec::new(),
        };
    }

    let batch = BatchExecution {
        state,
        scope,
        trace_id: &trace_id,
        request_id: request_id.as_deref(),
        fiber_id: request_id.as_deref().unwrap_or(&trace_id),
        context: context.as_ref(),
        start,
        timeout_ms: options.batch_timeout_ms,
    };
    let (ok, results) = batch
        .run_steps(steps, options.auto_rollback_on_failure)
        .await;
    let rollback_results = if options.auto_rollback_on_failure && !ok {
        batch.rollback().await
    } else {
        Vec::new()
    };
    BatchCallResponse {
        ok,
        request_id,
        trace_id,
        results,
        total_duration_us: start.elapsed().as_micros() as u64,
        rollback_results,
    }
}

struct BatchExecution<'a> {
    state: &'a AppState,
    scope: BatchExecutionScope<'a>,
    trace_id: &'a str,
    request_id: Option<&'a str>,
    fiber_id: &'a str,
    context: Option<&'a crate::context::RequestContext>,
    start: Instant,
    timeout_ms: u64,
}

impl BatchExecution<'_> {
    async fn run_steps(
        &self,
        steps: Vec<BatchStep>,
        auto_rollback: bool,
    ) -> (bool, Vec<BatchStepResult>) {
        let mut outputs = HashMap::new();
        let mut results = Vec::new();
        let mut ok = true;
        for step in steps {
            let start = Instant::now();
            let outcome = match self.check_budget() {
                Ok(()) => {
                    BatchStepExecution {
                        batch: self,
                        step: &step,
                        start,
                        idempotency_key: self
                            .request_id
                            .map(|batch_id| format!("batch:{batch_id}:step:{}", step.id)),
                    }
                    .run(&outputs)
                    .await
                }
                Err(failure) => Err(failure),
            };
            let (step_ok, data, error, stop) = match outcome {
                Ok(value) => {
                    outputs.insert(step.id.clone(), value.clone());
                    (true, Some(value), None, false)
                }
                Err(failure) => {
                    ok = false;
                    let (error, stop) = self.handle_step_failure(failure, &step, auto_rollback);
                    (false, None, error, stop)
                }
            };
            results.push(BatchStepResult {
                id: step.id,
                capability_id: step.capability_id,
                ok: step_ok,
                data,
                error,
                duration_us: start.elapsed().as_micros() as u64,
            });
            if stop {
                break;
            }
        }
        (ok, results)
    }

    fn handle_step_failure(
        &self,
        failure: StepFailure,
        step: &BatchStep,
        auto_rollback: bool,
    ) -> (Option<String>, bool) {
        match failure {
            StepFailure::Regular(message) => {
                (Some(message), auto_rollback || !step.continue_on_error)
            }
            StepFailure::Held(message) => (
                message,
                auto_rollback
                    || self.scope.cancellation_token.is_cancelled()
                    || !step.continue_on_error,
            ),
            StepFailure::Stop(message) => (Some(message), true),
        }
    }

    fn check_budget(&self) -> Result<(), StepFailure> {
        if self.scope.cancellation_token.is_cancelled() {
            return Err(StepFailure::Stop(
                "Batch execution was cancelled".to_string(),
            ));
        }
        if self.budget_expired() {
            return Err(StepFailure::Stop(format!(
                "Batch execution timed out after {}ms budget",
                self.timeout_ms
            )));
        }
        Ok(())
    }

    fn budget_expired(&self) -> bool {
        self.start.elapsed().as_millis() as u64 >= self.timeout_ms
    }

    fn remaining_budget(&self) -> Duration {
        Duration::from_millis(
            self.timeout_ms
                .saturating_sub(self.start.elapsed().as_millis() as u64),
        )
    }

    fn approval_expiry_message(&self) -> &'static str {
        if self.scope.cancellation_token.is_cancelled() {
            "Batch execution was cancelled while waiting for HITL approval"
        } else if self.budget_expired() {
            "Batch execution budget expired while waiting for HITL approval"
        } else {
            "HITL approval expired"
        }
    }

    async fn rollback(&self) -> Vec<crate::undo::RollbackResult> {
        crate::undo::unwind_batch_fiber(
            self.state,
            self.scope.tenant_id,
            self.fiber_id,
            crate::undo::FiberExecutionScope {
                policy: self.scope.policy,
                profile: self.scope.profile,
                context: self.context,
                authorization_expires_at_epoch_secs: self.scope.authorization_expires_at_epoch_secs,
                trace_id: self.trace_id,
                operator_id: None,
                force_expired_reason: None,
            },
        )
        .await
    }
}

enum StepFailure {
    Regular(String),
    Held(Option<String>),
    Stop(String),
}

struct StepTarget {
    server: String,
    tool: String,
    tool_definition: Option<String>,
    reversibility: Option<ReversibilityConfig>,
}

struct BatchStepExecution<'a, 'batch> {
    batch: &'a BatchExecution<'batch>,
    step: &'a BatchStep,
    start: Instant,
    idempotency_key: Option<String>,
}

impl BatchStepExecution<'_, '_> {
    async fn run(&self, outputs: &HashMap<String, Value>) -> Result<Value, StepFailure> {
        let state = self.batch.state;
        let enforcing = state.policy.read().await.result_sanitization.is_some();
        let mut args = crate::engine::interpolation::interpolate_batch_args(
            &self.step.args,
            outputs,
            enforcing,
        )
        .map_err(|error| StepFailure::Regular(error.to_string()))?;
        let target = self.resolve_target().await.map_err(StepFailure::Regular)?;
        let approved = self
            .apply_approval(&target, &mut args)
            .await
            .map_err(StepFailure::Regular)?;
        self.batch
            .state
            .circuit_breakers
            .check_permission(&target.server)
            .await
            .map_err(|error| {
                StepFailure::Regular(format!(
                    "Circuit open for server '{}': {error}",
                    target.server
                ))
            })?;

        if let Some(contract) = target
            .reversibility
            .as_ref()
            .filter(|contract| contract.class == ReversibilityClass::Delayable)
        {
            let task = self
                .accept_emission(&target, &args, contract, approved)
                .await
                .map_err(StepFailure::Regular)?;
            let value = self.wait_for_emission(&task.task_id).await?;
            self.batch
                .state
                .circuit_breakers
                .record_success(&target.server)
                .await;
            self.audit_success(
                &target.server,
                args,
                (!enforcing).then_some(&value),
                Some(&task.task_id),
            );
            return Ok(value);
        }

        let value = self
            .call_and_register_inverse(&target, &args)
            .await
            .map_err(StepFailure::Regular)?;
        let delivery = state
            .sanitize_result(
                &target.server,
                self.batch.trace_id,
                value,
                crate::result_sanitization::Shape::Tool,
            )
            .await
            .map_err(|withheld| {
                StepFailure::Regular(crate::WarmplaneError::from(withheld).message)
            })?;
        self.audit_success(
            &target.server,
            args,
            delivery.revision.is_none().then_some(&delivery.value),
            None,
        );
        Ok(delivery.value)
    }

    async fn resolve_target(&self) -> Result<StepTarget, String> {
        let capability = &self.step.capability_id;
        if !self.batch.scope.policy.allows(capability) {
            return Err(format!("Capability '{capability}' blocked by policy"));
        }
        let capabilities = self.batch.state.capabilities.read().await;
        let meta = capabilities
            .get(capability)
            .ok_or_else(|| format!("Capability '{capability}' not found"))?;
        if !self.batch.scope.profile.is_server_allowed(&meta.server) {
            return Err(format!(
                "Capability '{capability}' belongs to server '{}' which is not in active profile",
                meta.server
            ));
        }
        Ok(StepTarget {
            server: meta.server.clone(),
            tool: meta.tool.clone(),
            tool_definition: self
                .batch
                .state
                .tool_pins
                .binding(&meta.server, &meta.tool)
                .map_err(|denied| crate::WarmplaneError::from(denied).message)?,
            reversibility: meta.reversibility.clone(),
        })
    }

    async fn apply_approval(&self, target: &StepTarget, args: &mut Value) -> Result<bool, String> {
        let compensable = target
            .reversibility
            .as_ref()
            .filter(|contract| contract.class == ReversibilityClass::Compensable);
        let reserved = match compensable {
            Some(contract) => self.reserve_compensable(contract, false).await?,
            None => true,
        };
        if reserved
            && !self
                .batch
                .scope
                .policy
                .requires_approval(&self.step.capability_id)
        {
            return Ok(false);
        }

        if let Some(modified_args) = self.request_approval(&target.server, args).await? {
            *args = modified_args;
        }
        if let Some(contract) = compensable.filter(|_| !reserved) {
            self.reserve_compensable(contract, true).await?;
        }
        Ok(true)
    }

    fn require_undo_ledger(&self) -> Result<(), String> {
        self.batch
            .state
            .undo_ledger
            .as_ref()
            .map(|_| ())
            .ok_or_else(|| "Persistent undo storage is unavailable".to_string())
    }

    async fn reserve_compensable(
        &self,
        contract: &ReversibilityConfig,
        approved_over_limit: bool,
    ) -> Result<bool, String> {
        self.require_undo_ledger()?;
        let batch = self.batch;
        crate::engine::undo::reserve_compensable(
            batch.state,
            contract,
            &self.step.capability_id,
            batch.scope.tenant_id,
            batch.scope.profile.profile_id.as_deref(),
            self.idempotency_key.as_deref(),
            approved_over_limit,
        )
        .await
        .map_err(|error| error.to_string())
    }

    async fn request_approval(&self, server: &str, args: &Value) -> Result<Option<Value>, String> {
        let batch = self.batch;
        let policy = batch.scope.policy;
        let request = crate::approvals::CreateApprovalRequest {
            capability_id: self.step.capability_id.clone(),
            server_id: server.to_owned(),
            args: args.clone(),
            sanitized_args: crate::http_v1::helpers::redact_value(
                args.clone(),
                &policy.redact_keys,
            ),
            request_id: batch.request_id.map(str::to_owned),
            context: batch.context.cloned(),
            timeout_secs: policy.approval_timeout_secs,
            webhook: policy.webhook.as_ref(),
        };
        let (_, receiver, event) = batch
            .state
            .approval_registry
            .create_audited_approval(request, batch.trace_id.to_owned(), self.start)
            .await;
        batch.state.audit_handle.send(event);

        let resolution = tokio::select! {
            _ = batch.scope.cancellation_token.cancelled() => ApprovalResolution::Expired,
            result = tokio::time::timeout(batch.remaining_budget(), receiver) => {
                match result {
                    Ok(Ok(resolution)) => resolution,
                    _ => ApprovalResolution::Expired,
                }
            }
        };
        match resolution {
            ApprovalResolution::Approved { modified_args, .. } => Ok(modified_args),
            ApprovalResolution::Rejected { reason, operator } => Err(format!(
                "HITL approval rejected by operator '{}': {}",
                operator,
                reason.unwrap_or_else(|| "No reason provided".to_string())
            )),
            ApprovalResolution::Expired => Err(batch.approval_expiry_message().to_string()),
        }
    }

    async fn accept_emission(
        &self,
        target: &StepTarget,
        args: &Value,
        contract: &ReversibilityConfig,
        approved: bool,
    ) -> Result<TaskRecord, String> {
        let batch = self.batch;
        let scope = &batch.scope;
        let hold_secs =
            crate::emissions::resolve_call_hold_window_secs(batch.state, scope.profile, contract)
                .await;
        let release_at_ms =
            crate::emissions::release_deadline_ms(hold_secs).map_err(|error| error.to_string())?;
        let emission = EmissionRecord {
            idempotency_claim: None,
            phase: EmissionPhase::PendingEmission,
            release_at_ms,
            cancellable: true,
            tool_name: target.tool.clone(),
            tool_definition: target.tool_definition.clone(),
            input_responses: None,
            request_state: None,
            profile_id: scope.profile.profile_id.clone(),
            tenant_id: scope.tenant_id.to_owned(),
            role: scope.role.to_owned(),
            authorization_expires_at_epoch_secs: scope.authorization_expires_at_epoch_secs,
            approval_completed: approved,
            batch_id: batch.request_id.map(str::to_owned),
            step_id: Some(self.step.id.clone()),
        };
        batch
            .state
            .hold_coordinator
            .accept(
                batch.state,
                CreateTaskParams {
                    capability_id: self.step.capability_id.clone(),
                    server_id: target.server.clone(),
                    args: args.clone(),
                    request_id: batch.request_id.map(str::to_owned),
                    context: batch.context.cloned(),
                    idempotency_key: self.idempotency_key.clone(),
                    initial_status: TaskStatus::Working,
                    status_message: Some(
                        "Waiting for the batch emission release deadline".to_string(),
                    ),
                    input_requests: None,
                    ttl_ms: None,
                    poll_interval_ms: Some(500),
                },
                emission,
            )
            .await
            .map_err(|error| format!("Could not persist held emission: {error}"))
    }

    async fn wait_for_emission(&self, task_id: &str) -> Result<Value, StepFailure> {
        let batch = self.batch;
        let terminal = tokio::select! {
            _ = batch.scope.cancellation_token.cancelled() => None,
            result = batch.state.hold_coordinator.wait_for_terminal(
                batch.state, task_id, batch.remaining_budget(),
            ) => result,
        };
        let Some(terminal) = terminal else {
            let cancelled = self
                .cancel_emission(task_id, "Batch execution cancelled")
                .await;
            let message = if cancelled {
                "Batch cancelled; held emission was prevented from dispatching"
            } else {
                "Batch cancelled after dispatch began; emission outcome may be pending"
            };
            return Err(StepFailure::Stop(message.to_string()));
        };
        let view = batch.state.task_view(&terminal).await;
        match view.status {
            TaskStatus::Completed if view.error.is_none() => Ok(view.result.unwrap_or(Value::Null)),
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                Err(StepFailure::Held(view.status_message))
            }
            _ => {
                let cancelled = self
                    .cancel_emission(task_id, "Batch execution budget expired")
                    .await;
                let message = if cancelled {
                    "Batch budget expired; held emission was cancelled before dispatch"
                } else {
                    "Batch budget expired after dispatch began; emission outcome may be pending"
                };
                Err(StepFailure::Held(Some(message.to_string())))
            }
        }
    }

    async fn cancel_emission(&self, task_id: &str, reason: &str) -> bool {
        let state = self.batch.state;
        matches!(
            state
                .hold_coordinator
                .cancel(state, task_id, Some(reason.to_string()))
                .await,
            Ok(EmissionCancelResult::Cancelled | EmissionCancelResult::AlreadyCancelled)
        )
    }

    async fn capture_snapshot(
        &self,
        contract: Option<&ReversibilityConfig>,
        args: &Value,
    ) -> Result<(Value, Option<String>), String> {
        let Some(contract) = contract else {
            return Ok((Value::Null, None));
        };
        if matches!(
            contract.class,
            ReversibilityClass::Reversible | ReversibilityClass::Compensable
        ) {
            self.require_undo_ledger()?;
        }
        let Some(snapshot) = &contract.pre_snapshot else {
            return Ok((Value::Null, None));
        };
        let batch = self.batch;
        crate::engine::undo::capture_pre_snapshot(
            batch.state,
            snapshot,
            args,
            batch.scope.profile,
            batch.scope.policy,
        )
        .await
        .map_err(|error| format!("Pre-snapshot failed: {error}"))
    }

    async fn call_and_register_inverse(
        &self,
        target: &StepTarget,
        args: &Value,
    ) -> Result<Value, String> {
        let batch = self.batch;
        let contract = target.reversibility.as_ref();
        let (snapshot, snapshot_hash) = self.capture_snapshot(contract, args).await?;
        let value = self.call_upstream(target, args).await?;
        if let Some(contract) = contract {
            let fallback_context = crate::context::RequestContext::default();
            crate::engine::undo::register_inverse(
                batch.state,
                contract,
                crate::engine::undo::ForwardCall {
                    capability: &self.step.capability_id,
                    args,
                    result: &value,
                    snapshot: &snapshot,
                    snapshot_hash,
                    context: batch.context.unwrap_or(&fallback_context),
                    batch_id: Some(batch.fiber_id),
                    tenant_id: batch.scope.tenant_id,
                    profile_id: batch.scope.profile.profile_id.as_deref(),
                },
                batch.scope.profile,
                batch.scope.policy,
            )
            .await
            .map_err(|error| {
                format!("Forward call succeeded but inverse registration failed: {error}")
            })?;
        }
        Ok(value)
    }

    async fn call_upstream(&self, target: &StepTarget, args: &Value) -> Result<Value, String> {
        let state = self.batch.state;
        let result = send_step_call(state, target, args).await;
        if result.is_ok() {
            state.circuit_breakers.record_success(&target.server).await;
        } else {
            state.circuit_breakers.record_failure(&target.server).await;
        }
        result
    }

    fn audit_success(
        &self,
        server: &str,
        args: Value,
        result: Option<&Value>,
        held_task_id: Option<&str>,
    ) {
        let batch = self.batch;
        let redact_keys = &batch.scope.policy.redact_keys;
        let sanitized_response =
            result.map(|value| crate::http_v1::helpers::redact_value(value.clone(), redact_keys));
        let (sanitized_response, idempotency_key) = match held_task_id {
            Some(task_id) => (
                serde_json::json!({"taskId": task_id, "result": sanitized_response}),
                self.idempotency_key.clone(),
            ),
            None => (sanitized_response.unwrap_or(Value::Null), None),
        };
        batch.state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolExecution,
            trace_id: batch.trace_id.to_owned(),
            request_id: batch.request_id.map(str::to_owned),
            actor_id: batch.context.and_then(|context| context.actor_id.clone()),
            work_item_id: batch
                .context
                .and_then(|context| context.work_item_id.clone()),
            client_ip: None,
            server_id: Some(server.to_owned()),
            capability_id: Some(self.step.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(crate::http_v1::helpers::redact_value(args, redact_keys)),
            sanitized_response: Some(sanitized_response),
            execution_latency_us: Some(self.start.elapsed().as_micros() as u64),
            status: crate::audit::AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key,
            is_replay: Some(false),
        });
    }
}

async fn send_step_call(
    state: &AppState,
    target: &StepTarget,
    args: &Value,
) -> Result<Value, String> {
    let sender = state
        .servers
        .read()
        .await
        .get(&target.server)
        .cloned()
        .ok_or_else(|| format!("Server '{}' unreachable", target.server))?;
    let (reply, receiver) = oneshot::channel();
    sender
        .send(ServerMsg::CallTool {
            name: target.tool.clone(),
            tool_definition: target.tool_definition.clone(),
            params: args.clone(),
            input_responses: None,
            request_state: None,
            reply,
        })
        .await
        .map_err(|_| format!("Server '{}' mailbox closed", target.server))?;
    match receiver.await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(UpstreamCallError::Timeout)) => Err("Tool execution timed out".to_string()),
        Ok(Err(UpstreamCallError::Upstream(error))) => Err(state.safe_upstream_error(error).await),
        Ok(Err(UpstreamCallError::ToolDefinitionChanged)) => {
            Err(crate::WarmplaneError::from(crate::tool_pins::PinDenied).message)
        }
        Err(_) => Err("Daemon actor task died".to_string()),
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

    #[tokio::test]
    async fn batch_budget_cancels_a_pending_emission_before_dispatch() {
        use crate::{context::ProfileContext, daemon::Policy, tasks::TaskStatus};

        let (state, mut upstream) =
            crate::test_support::delayable_mail(30, crate::config::DEFAULT_TOOL_TIMEOUT_MS);
        let policy = Policy::default();
        let profile = ProfileContext::unrestricted();
        let response = execute_batch_for_tenant_with_budget(
            &state,
            vec![BatchStep {
                id: "send".to_string(),
                capability_id: "mail.send".to_string(),
                args: json!({"to": "person@example.com"}),
                continue_on_error: false,
            }],
            "batch-budget-test".to_string(),
            Some("batch-budget-test".to_string()),
            None,
            BatchExecutionScope {
                policy: &policy,
                profile: &profile,
                tenant_id: "default",
                role: "anonymous",
                authorization_expires_at_epoch_secs: None,
                cancellation_token: CancellationToken::new(),
            },
            BatchExecutionOptions {
                auto_rollback_on_failure: false,
                batch_timeout_ms: 50,
            },
        )
        .await;

        assert!(!response.ok);
        assert!(response.results[0]
            .error
            .as_deref()
            .is_some_and(|message| message.contains("budget expired")));
        let task = state.task_registry.list_tasks().await.pop().unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
        assert!(upstream.try_recv().is_err());
    }

    #[tokio::test]
    async fn failed_batch_unwinds_successful_steps_in_lifo_order_once() {
        use crate::{
            config::{InverseConfig, PreSnapshotConfig, ReversibilityClass, ReversibilityConfig},
            context::ProfileContext,
            daemon::{AppState, CapabilityMeta, Policy, UpstreamCallError},
            undo::{SnapshotStore, UndoLedgerStore},
        };
        use std::{collections::HashMap, sync::Arc};
        use tokio::sync::mpsc;

        let mut mutate = CapabilityMeta::new(
            "mock",
            "mutate",
            "Mutate",
            "Mutate state",
            json!({"type":"object"}),
        );
        mutate.reversibility = Some(ReversibilityConfig {
            class: ReversibilityClass::Reversible,
            hold_window_secs: None,
            ttl_secs: Some(60),
            daily_limit: None,
            pre_snapshot: Some(Box::new(PreSnapshotConfig {
                capability: "mock.get_state".to_string(),
                args: json!({}),
                capture_json_path: None,
            })),
            pre_undo_check: None,
            test_target: None,
            inverse: Some(Box::new(InverseConfig {
                capability: "mock.restore".to_string(),
                args: json!({"value":"${snapshot}"}),
            })),
        });
        let capabilities = HashMap::from([
            ("mock.mutate".to_string(), mutate),
            (
                "mock.get_state".to_string(),
                CapabilityMeta::new(
                    "mock",
                    "get_state",
                    "Get",
                    "Read state",
                    json!({"type":"object"}),
                ),
            ),
            (
                "mock.restore".to_string(),
                CapabilityMeta::new(
                    "mock",
                    "restore",
                    "Restore",
                    "Restore state",
                    json!({"type":"object"}),
                ),
            ),
            (
                "mock.fail".to_string(),
                CapabilityMeta::new("mock", "fail", "Fail", "Fail", json!({"type":"object"})),
            ),
        ]);
        let (server_tx, mut server_rx) = mpsc::channel(16);
        let temp = tempfile::tempdir().unwrap();
        let ledger = UndoLedgerStore::open(temp.path().join("undo.json")).unwrap();
        let snapshots = SnapshotStore::open(temp.path().join("snapshots")).unwrap();
        let state = AppState::builder()
            .servers(HashMap::from([("mock".to_string(), server_tx)]))
            .capabilities(capabilities)
            .undo_ledger(ledger.clone())
            .snapshot_store(snapshots)
            .build();
        let mock_state = Arc::new(tokio::sync::Mutex::new(json!({"value": 0})));
        let observed = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let server_state = mock_state.clone();
        let server_observed = observed.clone();
        let server = tokio::spawn(async move {
            for _ in 0..7 {
                let Some(crate::daemon::ServerMsg::CallTool {
                    name,
                    params,
                    reply,
                    ..
                }) = server_rx.recv().await
                else {
                    break;
                };
                server_observed.lock().await.push(name.clone());
                let result = match name.as_str() {
                    "get_state" => Ok(server_state.lock().await.clone()),
                    "mutate" | "restore" => {
                        *server_state.lock().await = params["value"].clone();
                        Ok(json!({"ok": true}))
                    }
                    "fail" => Err(UpstreamCallError::Upstream("planned failure".to_string())),
                    _ => Err(UpstreamCallError::Upstream("unexpected tool".to_string())),
                };
                let _ = reply.send(result);
            }
        });
        let policy = Policy::default();
        let profile = ProfileContext::unrestricted();
        let response = execute_batch_for_tenant_with_rollback(
            &state,
            vec![
                BatchStep {
                    id: "one".to_string(),
                    capability_id: "mock.mutate".to_string(),
                    args: json!({"value":{"value":1}}),
                    continue_on_error: true,
                },
                BatchStep {
                    id: "two".to_string(),
                    capability_id: "mock.mutate".to_string(),
                    args: json!({"value":{"value":2}}),
                    continue_on_error: true,
                },
                BatchStep {
                    id: "three".to_string(),
                    capability_id: "mock.fail".to_string(),
                    args: json!({}),
                    continue_on_error: true,
                },
            ],
            "trace-stage3".to_string(),
            Some("batch-stage3".to_string()),
            Some(crate::context::RequestContext {
                operation_id: Some("op-stage3".to_string()),
                work_item_id: Some("work-stage3".to_string()),
                ..Default::default()
            }),
            BatchExecutionScope {
                policy: &policy,
                profile: &profile,
                tenant_id: "tenant-a",
                role: "admin",
                authorization_expires_at_epoch_secs: None,
                cancellation_token: CancellationToken::new(),
            },
            true,
        )
        .await;
        assert!(!response.ok);
        assert_eq!(response.rollback_results.len(), 2);
        assert!(response.rollback_results.iter().all(|outcome| outcome.ok));
        assert_eq!(*mock_state.lock().await, json!({"value":0}));
        assert_eq!(
            *observed.lock().await,
            vec![
                "get_state",
                "mutate",
                "get_state",
                "mutate",
                "fail",
                "restore",
                "restore"
            ]
        );
        assert!(crate::undo::unwind_batch_fiber(
            &state,
            "tenant-a",
            "batch-stage3",
            crate::undo::FiberExecutionScope {
                policy: &policy,
                profile: &profile,
                context: None,
                authorization_expires_at_epoch_secs: None,
                trace_id: "trace-again",
                operator_id: None,
                force_expired_reason: None,
            }
        )
        .await
        .is_empty());
        assert_eq!(observed.lock().await.len(), 7);
        server.await.unwrap();
        assert!(ledger
            .entries()
            .await
            .iter()
            .all(|entry| entry.status == crate::undo::UndoEntryStatus::Reverted));
        let mut undo_events = Vec::new();
        for _ in 0..50 {
            undo_events = state
                .audit_store
                .query(&crate::audit::AuditQueryFilter {
                    event_type: Some(crate::audit::AuditEventType::UndoExecuted),
                    limit: 100,
                    ..Default::default()
                })
                .await
                .0;
            if undo_events.len() >= 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(undo_events.len(), 2);
        assert!(undo_events.iter().all(|event| {
            event.sanitized_response.as_ref().is_some_and(|details| {
                details.get("parentOperationId") == Some(&json!("op-stage3"))
                    && details
                        .get("snapshotArtifactHash")
                        .is_some_and(Value::is_string)
            })
        }));
        assert!(state.audit_store.verify_chain().await.is_valid);
    }
}
