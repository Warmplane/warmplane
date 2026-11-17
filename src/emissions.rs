// Rust guideline compliant 2026-11-16

//! Durable cancellation windows for delayable tool calls.

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use serde_json::json;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::{
    config::{ReversibilityClass, ReversibilityConfig},
    daemon::{AppState, Policy},
    tasks::{EmissionCancelResult, EmissionPhase, EmissionRecord, TaskRecord, TaskStatus},
};

/// Resolves the policy duration using profile, alias, global, then default precedence.
pub fn effective_hold_window_secs(
    contract: &ReversibilityConfig,
    profile_window_secs: Option<u64>,
    global_window_secs: Option<u64>,
) -> Option<u64> {
    (contract.class == ReversibilityClass::Delayable).then(|| {
        profile_window_secs
            .or(contract.hold_window_secs)
            .or(global_window_secs)
            .unwrap_or(30)
    })
}

/// Converts a hold duration to an absolute Unix deadline, checking overflow.
pub fn release_deadline_ms(duration_secs: u64) -> Result<u64> {
    let duration_ms = duration_secs
        .checked_mul(1_000)
        .context("hold window milliseconds overflow")?;
    unix_time_ms()
        .checked_add(duration_ms)
        .context("hold window release deadline overflow")
}

/// Coordinates held emissions and serializes cancellation against dispatch claims.
#[derive(Clone, Default)]
pub struct HoldCoordinator {
    transition_lock: Arc<Mutex<()>>,
    workers: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl HoldCoordinator {
    /// Checks that a caller still owns and is authorized to inspect an emission task.
    pub async fn can_access_task(
        &self,
        state: &AppState,
        record: &TaskRecord,
        tenant_id: &str,
        role: &str,
        profile_id: Option<&str>,
    ) -> bool {
        let Some(emission) = record.emission.as_ref() else {
            return true;
        };
        if emission
            .authorization_expires_at_epoch_secs
            .is_some_and(|expiry| unix_time_secs() > expiry)
        {
            return false;
        }
        if emission.tenant_id != tenant_id
            || !state.rbac_engine.recognizes_provenance(
                tenant_id,
                role,
                record
                    .context
                    .as_ref()
                    .and_then(|context| context.actor_id.as_deref()),
                record
                    .context
                    .as_ref()
                    .and_then(|context| context.grant_id.as_deref()),
            )
        {
            return false;
        }
        if emission.profile_id.as_deref() != profile_id {
            return false;
        }
        let base = state.policy.read().await.clone();
        let role_policy = state.rbac_engine.compute_effective_policy(role, &base);
        let profile_policy = if let Some(profile_id) = profile_id {
            let profiles = state.profiles.read().await;
            let Some(profile) = profiles.get(profile_id) else {
                return false;
            };
            if !profile.servers.contains(&record.server_id) {
                return false;
            }
            Some(Policy::from_config(profile.policy.clone()))
        } else {
            None
        };
        role_policy
            .merge_with_profile(profile_policy.as_ref())
            .allows(&record.capability_id)
    }

    /// Accepts a held emission, persists its task, and starts its release timer.
    pub async fn accept(
        &self,
        state: &AppState,
        params: crate::tasks::CreateTaskParams,
        emission: EmissionRecord,
    ) -> Result<TaskRecord> {
        let _gate = self.transition_lock.lock().await;
        if let Some(key) = params.idempotency_key.as_deref() {
            if let Some(existing) = state.task_registry.find_by_idempotency_key(key).await {
                if existing.emission.is_some() {
                    return Ok(existing);
                }
                anyhow::bail!("idempotency key is already associated with a non-emission task");
            }
        }
        let record = state
            .task_registry
            .create_emission_task(params, emission)
            .await?;
        state.notify_action_capsules_changed();
        dispatch_held_chatops_action(state, &record);
        self.start_worker(state.clone(), record.clone()).await;
        self.audit_emission(state, &record, crate::audit::AuditEventType::EmissionHeld)
            .await;
        Ok(record)
    }

    /// Converts an approval task into a durably held emission and starts its timer.
    pub async fn queue_existing(
        &self,
        state: &AppState,
        task_id: &str,
        approved_args: serde_json::Value,
        emission: EmissionRecord,
    ) -> Result<Option<TaskRecord>> {
        let _gate = self.transition_lock.lock().await;
        let record = state
            .task_registry
            .attach_emission(task_id, approved_args, emission)
            .await?;
        if let Some(record) = record.as_ref() {
            state.notify_action_capsules_changed();
            dispatch_held_chatops_action(state, record);
            self.audit_emission(state, record, crate::audit::AuditEventType::EmissionHeld)
                .await;
            self.start_worker(state.clone(), record.clone()).await;
        }
        Ok(record)
    }

    /// Attempts to cancel a held emission while holding the shared transition gate.
    pub async fn cancel(
        &self,
        state: &AppState,
        task_id: &str,
        reason: Option<String>,
    ) -> Result<EmissionCancelResult> {
        let _gate = self.transition_lock.lock().await;
        let result = state
            .task_registry
            .cancel_emission_task(task_id, reason)
            .await?;
        if result == EmissionCancelResult::Cancelled {
            state.notify_action_capsules_changed();
        }
        if matches!(
            result,
            EmissionCancelResult::Cancelled
                | EmissionCancelResult::AlreadyCancelled
                | EmissionCancelResult::TooLate
        ) {
            if let Some(batch_id) = state
                .task_registry
                .get_task(task_id)
                .await
                .and_then(|record| record.emission.and_then(|emission| emission.batch_id))
            {
                let _ = state.operation_registry.cancel(batch_id).await;
            }
        }
        if result == EmissionCancelResult::Cancelled {
            if let Some(token) = self.workers.lock().await.remove(task_id) {
                token.cancel();
            }
            if let Some(record) = state.task_registry.get_task(task_id).await {
                self.audit_emission(
                    state,
                    &record,
                    crate::audit::AuditEventType::EmissionCancelled,
                )
                .await;
                if let Some(key) = record.idempotency_key.as_deref() {
                    let envelope = crate::engine::Envelope::<serde_json::Value>::failure(
                        record
                            .request_id
                            .clone()
                            .unwrap_or_else(|| record.task_id.clone()),
                        record.request_id.clone(),
                        record.context.clone(),
                        crate::engine::WarmplaneError::new(
                            "EMISSION_CANCELLED",
                            record.status_message.clone().unwrap_or_else(|| {
                                "Emission cancelled before dispatch".to_string()
                            }),
                            false,
                        ),
                        crate::idempotency::RetryMetadata::idempotent("not_started"),
                    );
                    if let Ok(value) = serde_json::to_value(envelope) {
                        state.idempotency_store.complete(key, value).await;
                    }
                }
            }
        }
        Ok(result)
    }

    /// Reconciles persisted emissions after policy, profiles, and upstreams are ready.
    pub async fn recover(&self, state: &AppState) {
        for record in state.task_registry.list_tasks().await {
            let Some(emission) = record.emission.as_ref() else {
                continue;
            };
            if emission.phase == EmissionPhase::Dispatching
                && !matches!(
                    record.status,
                    TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                )
            {
                self.finish_failure(
                    state,
                    &record,
                    "EMISSION_OUTCOME_UNKNOWN",
                    "Dispatch began before restart; automatic resend was prevented".to_string(),
                )
                .await;
            } else if emission.phase == EmissionPhase::PendingEmission
                && record.status != TaskStatus::Cancelled
                && record.status != TaskStatus::Completed
                && record.status != TaskStatus::Failed
            {
                self.start_worker(state.clone(), record).await;
            }
        }
    }

    /// Waits for a task to reach a terminal status for at most `timeout`.
    pub async fn wait_for_terminal(
        &self,
        state: &AppState,
        task_id: &str,
        timeout: Duration,
    ) -> Option<TaskRecord> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let record = state.task_registry.get_task(task_id).await?;
            if matches!(
                record.status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ) {
                return Some(record);
            }
            if tokio::time::timeout_at(deadline, tokio::time::sleep(Duration::from_millis(10)))
                .await
                .is_err()
            {
                return Some(record);
            }
        }
    }

    async fn start_worker(&self, state: AppState, record: TaskRecord) {
        let token = CancellationToken::new();
        self.workers
            .lock()
            .await
            .insert(record.task_id.clone(), token.clone());
        let coordinator = self.clone();
        let task_id = record.task_id.clone();
        tokio::spawn(async move {
            coordinator.run_worker(state, record, token.clone()).await;
            coordinator.workers.lock().await.remove(&task_id);
        });
    }

    async fn run_worker(&self, state: AppState, initial: TaskRecord, token: CancellationToken) {
        let Some(emission) = initial.emission.as_ref() else {
            return;
        };
        let now_ms = unix_time_ms();
        let wait_ms = emission.release_at_ms.saturating_sub(now_ms);
        tokio::select! {
            _ = token.cancelled() => return,
            _ = state.shutdown_token.cancelled() => return,
            _ = tokio::time::sleep(Duration::from_millis(wait_ms)) => {}
        }

        let _gate = self.transition_lock.lock().await;
        if token.is_cancelled() || state.shutdown_token.is_cancelled() {
            return;
        }
        if let Err(error) = revalidate(&state, &initial).await {
            self.finish_failure(
                &state,
                &initial,
                "EMISSION_REVALIDATION_FAILED",
                error.to_string(),
            )
            .await;
            return;
        }
        if token.is_cancelled() || state.shutdown_token.is_cancelled() {
            return;
        }
        let Some(record) = (match state
            .task_registry
            .claim_emission_dispatch(&initial.task_id)
            .await
        {
            Ok(record) => record,
            Err(error) => {
                tracing::error!(task_id = %initial.task_id, error = %error, "failed to persist emission dispatch claim");
                return;
            }
        }) else {
            return;
        };
        state.notify_action_capsules_changed();
        drop(_gate);

        let Some(emission) = record.emission.as_ref() else {
            return;
        };
        let sender = {
            let servers = state.servers.read().await;
            servers.get(&record.server_id).cloned()
        };
        let Some(sender) = sender else {
            state
                .circuit_breakers
                .record_failure(&record.server_id)
                .await;
            self.finish_failure(
                &state,
                &record,
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unavailable", record.server_id),
            )
            .await;
            return;
        };
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        let send_result = sender
            .send(crate::daemon::ServerMsg::CallTool {
                name: emission.tool_name.clone(),
                params: record.args.clone(),
                input_responses: emission.input_responses.clone(),
                request_state: emission.request_state.clone(),
                reply: reply_tx,
            })
            .await;
        if send_result.is_err() {
            state
                .circuit_breakers
                .record_failure(&record.server_id)
                .await;
            self.finish_failure(
                &state,
                &record,
                "SERVER_UNREACHABLE",
                "Upstream server mailbox is closed".to_string(),
            )
            .await;
            return;
        }
        let result =
            tokio::time::timeout(Duration::from_millis(state.tool_timeout_ms), reply_rx).await;
        match result {
            Ok(Ok(Ok(value))) => {
                state
                    .circuit_breakers
                    .record_success(&record.server_id)
                    .await;
                self.finish_success(&state, &record, value).await;
            }
            Ok(Ok(Err(error))) => {
                state
                    .circuit_breakers
                    .record_failure(&record.server_id)
                    .await;
                let message = match error {
                    crate::daemon::UpstreamCallError::Timeout => {
                        "Upstream call timed out".to_string()
                    }
                    crate::daemon::UpstreamCallError::Upstream(message) => message,
                };
                self.finish_failure(&state, &record, "UPSTREAM_CALL_FAILED", message)
                    .await;
            }
            Ok(Err(_)) => {
                state
                    .circuit_breakers
                    .record_failure(&record.server_id)
                    .await;
                self.finish_failure(
                    &state,
                    &record,
                    "UPSTREAM_DROPPED",
                    "Upstream server dropped its response".to_string(),
                )
                .await;
            }
            Err(_) => {
                state
                    .circuit_breakers
                    .record_failure(&record.server_id)
                    .await;
                self.finish_failure(
                    &state,
                    &record,
                    "TOOL_TIMEOUT",
                    "Upstream call timed out".to_string(),
                )
                .await;
            }
        }
    }

    async fn finish_success(
        &self,
        state: &AppState,
        record: &TaskRecord,
        result: serde_json::Value,
    ) {
        let _ = state
            .task_registry
            .complete_task(&record.task_id, result.clone())
            .await;
        self.audit_execution(state, record, Some(result.clone()), None)
            .await;
        if let Some(key) = record.idempotency_key.as_deref() {
            let retry = if record.idempotency_key.is_some() {
                crate::idempotency::RetryMetadata::idempotent("completed")
            } else {
                crate::idempotency::RetryMetadata::unsafe_op("completed")
            };
            let envelope = crate::engine::Envelope::success(
                record
                    .request_id
                    .clone()
                    .unwrap_or_else(|| record.task_id.clone()),
                record.request_id.clone(),
                record.context.clone(),
                result,
                retry,
            );
            if let Ok(value) = serde_json::to_value(envelope) {
                state.idempotency_store.complete(key, value).await;
            }
        }
    }

    async fn finish_failure(
        &self,
        state: &AppState,
        record: &TaskRecord,
        code: &str,
        message: String,
    ) {
        let _ = state
            .task_registry
            .fail_task(
                &record.task_id,
                json!({"code": code, "message": message}),
                Some(message.clone()),
            )
            .await;
        self.audit_execution(state, record, None, Some((code, message.clone())))
            .await;
        if let Some(key) = record.idempotency_key.as_deref() {
            let envelope = crate::engine::Envelope::<serde_json::Value>::failure(
                record
                    .request_id
                    .clone()
                    .unwrap_or_else(|| record.task_id.clone()),
                record.request_id.clone(),
                record.context.clone(),
                crate::engine::WarmplaneError::new(code, message, false),
                crate::idempotency::RetryMetadata::idempotent("completed"),
            );
            if let Ok(value) = serde_json::to_value(envelope) {
                state.idempotency_store.complete(key, value).await;
            }
        }
    }

    async fn audit_execution(
        &self,
        state: &AppState,
        record: &TaskRecord,
        result: Option<serde_json::Value>,
        failure: Option<(&str, String)>,
    ) {
        let policy = current_emission_policy(state, record).await;
        let sanitized_args =
            crate::http_v1::helpers::redact_value(record.args.clone(), &policy.redact_keys);
        let sanitized_response =
            result.map(|value| crate::http_v1::helpers::redact_value(value, &policy.redact_keys));
        state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolExecution,
            trace_id: record
                .request_id
                .clone()
                .unwrap_or_else(|| record.task_id.clone()),
            request_id: record.request_id.clone(),
            actor_id: record
                .context
                .as_ref()
                .and_then(|context| context.actor_id.clone()),
            work_item_id: record
                .context
                .as_ref()
                .and_then(|context| context.work_item_id.clone()),
            client_ip: None,
            server_id: Some(record.server_id.clone()),
            capability_id: Some(record.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(sanitized_args),
            sanitized_response: Some(
                json!({"taskId": record.task_id, "result": sanitized_response}),
            ),
            execution_latency_us: None,
            status: if failure.is_some() {
                crate::audit::AuditEventStatus::Failed
            } else {
                crate::audit::AuditEventStatus::Success
            },
            error_code: failure.as_ref().map(|(code, _)| (*code).to_string()),
            error_message: failure
                .map(|_| "Emission failed; see the task error for details".to_string()),
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: record.idempotency_key.clone(),
            is_replay: Some(false),
        });
    }

    async fn audit_emission(
        &self,
        state: &AppState,
        record: &TaskRecord,
        event_type: crate::audit::AuditEventType,
    ) {
        let policy = current_emission_policy(state, record).await;
        let sanitized_args =
            crate::http_v1::helpers::redact_value(record.args.clone(), &policy.redact_keys);
        state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type,
            trace_id: record
                .request_id
                .clone()
                .unwrap_or_else(|| record.task_id.clone()),
            request_id: record.request_id.clone(),
            actor_id: record
                .context
                .as_ref()
                .and_then(|context| context.actor_id.clone()),
            work_item_id: record
                .context
                .as_ref()
                .and_then(|context| context.work_item_id.clone()),
            client_ip: None,
            server_id: Some(record.server_id.clone()),
            capability_id: Some(record.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(sanitized_args),
            sanitized_response: Some(json!({"taskId": record.task_id})),
            execution_latency_us: None,
            status: if record.status == TaskStatus::Cancelled {
                crate::audit::AuditEventStatus::Cancelled
            } else {
                crate::audit::AuditEventStatus::Intercepted
            },
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: record.idempotency_key.clone(),
            is_replay: Some(false),
        });
    }
}

fn dispatch_held_chatops_action(state: &AppState, record: &TaskRecord) {
    let Some(emission) = record.emission.as_ref() else {
        return;
    };
    crate::approvals::dispatch_signed_action(
        state.clone(),
        crate::approvals::SignedActionDispatch {
            event_type: "emission.held",
            action: "cancel",
            target_id: record.task_id.clone(),
            tenant_id: emission.tenant_id.clone(),
            expires_at_ms: emission.release_at_ms,
            capability: record.capability_id.clone(),
            actor: record
                .context
                .as_ref()
                .and_then(|context| context.actor_id.clone()),
            profile_id: emission.profile_id.clone(),
        },
    );
}

async fn revalidate(state: &AppState, record: &TaskRecord) -> Result<()> {
    let emission = record
        .emission
        .as_ref()
        .context("emission metadata is missing")?;
    anyhow::ensure!(
        !emission
            .authorization_expires_at_epoch_secs
            .is_some_and(|expiry| unix_time_secs() > expiry),
        "caller grant expired before emission release"
    );
    let meta = state
        .capabilities
        .read()
        .await
        .get(&record.capability_id)
        .cloned()
        .context("capability is no longer available")?;
    anyhow::ensure!(
        meta.server == record.server_id,
        "capability server mapping changed"
    );
    anyhow::ensure!(
        meta.tool == emission.tool_name,
        "capability tool mapping changed"
    );
    anyhow::ensure!(
        meta.reversibility
            .as_ref()
            .is_some_and(|contract| contract.class == ReversibilityClass::Delayable),
        "capability is no longer delayable"
    );

    let base_policy = state.policy.read().await.clone();
    let profile = if let Some(profile_id) = emission.profile_id.as_deref() {
        let profiles = state.profiles.read().await;
        let config = profiles
            .get(profile_id)
            .context("active profile no longer exists")?;
        anyhow::ensure!(
            config.servers.contains(&record.server_id),
            "server is no longer in the active profile"
        );
        Some(Policy::from_config(config.policy.clone()))
    } else {
        None
    };
    let effective_policy = state
        .rbac_engine
        .compute_effective_policy(&emission.role, &base_policy)
        .merge_with_profile(profile.as_ref());
    anyhow::ensure!(
        state.rbac_engine.recognizes_provenance(
            &emission.tenant_id,
            &emission.role,
            record
                .context
                .as_ref()
                .and_then(|context| context.actor_id.as_deref()),
            record
                .context
                .as_ref()
                .and_then(|context| context.grant_id.as_deref()),
        ),
        "caller role is no longer authorized"
    );
    anyhow::ensure!(
        effective_policy.allows(&record.capability_id),
        "capability is denied by current policy"
    );
    anyhow::ensure!(
        !effective_policy.requires_approval(&record.capability_id) || emission.approval_completed,
        "current policy now requires an approval that was not completed"
    );
    anyhow::ensure!(
        state.servers.read().await.contains_key(&record.server_id),
        "upstream server is unavailable"
    );
    state
        .circuit_breakers
        .check_permission(&record.server_id)
        .await
        .context("upstream circuit breaker denied dispatch")?;
    Ok(())
}

async fn current_emission_policy(state: &AppState, record: &TaskRecord) -> Policy {
    let emission = record.emission.as_ref();
    let base = state.policy.read().await.clone();
    let default_role = state.rbac_engine.default_role();
    let role = emission
        .map(|emission| emission.role.as_str())
        .unwrap_or(default_role.as_str());
    let mut effective = state.rbac_engine.compute_effective_policy(role, &base);
    if let Some(profile_id) = emission.and_then(|emission| emission.profile_id.as_deref()) {
        let profiles = state.profiles.read().await;
        if let Some(profile) = profiles.get(profile_id) {
            effective =
                effective.merge_with_profile(Some(&Policy::from_config(profile.policy.clone())));
        }
    }
    effective
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn unix_time_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
