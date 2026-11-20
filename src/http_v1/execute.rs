// Rust guideline compliant 2026-11-18
//! Capability tool execution handlers, cancellation, rate limiting, and HITL approval workflow.

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    ops::ControlFlow,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
use tracing::info;

use crate::{
    config::{ReversibilityClass, ReversibilityConfig},
    context::{ProfileContext, RequestContext},
    daemon::{AppState, Policy, ServerMsg, UpstreamCallError},
    http_v1::{
        helpers::{next_trace_id, redact_value, resolve_client_ip, resolve_idempotency_key},
        types::{error_envelope, CallCapabilityRequest},
    },
    idempotency::{ExecutionClaim, IdempotencyClaim, RetryMetadata},
    rbac::TenantContext,
    tasks::{
        CreateTaskParams, EmissionPhase, EmissionRecord, TaskRecord, TaskResponse, TaskStatus,
    },
};

/// Handles HTTP POST `/v1/tools/call` executing an MCP tool capability.
pub async fn handle_call_capability(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
    Json(payload): Json<CallCapabilityRequest>,
) -> impl IntoResponse {
    let prof_ctx = prof_ext
        .as_ref()
        .map(|extension| extension.0.clone())
        .unwrap_or_default();
    let identity = CallIdentity::new(&state, &payload, &headers, req_ext.0.as_ref());
    let idempotency_claim = match identity
        .claim_call(
            &state,
            &payload,
            req_ext.0.as_ref(),
            prof_ext.as_ref().map(|extension| &extension.0),
        )
        .await
    {
        ControlFlow::Continue(claim) => claim,
        ControlFlow::Break(response) => return response,
    };
    let cancel_token = state
        .operation_registry
        .register(&identity.request_id)
        .await;
    let preflight = CallPreflight {
        state: &state,
        payload: &payload,
        identity: &identity,
        profile: &prof_ctx,
        tenant: req_ext.0.as_ref(),
        claim: idempotency_claim,
    };
    if let Err((status, error)) = preflight.validate_arguments().await {
        return identity.rejected(status, error);
    }
    let effective_policy = match preflight.resolve_policy().await {
        Ok(policy) => policy,
        Err((status, error)) => return identity.rejected(status, error),
    };
    let CallTarget {
        server_id,
        tool_name,
        reversibility,
    } = match preflight.resolve_target().await {
        Ok(target) => target,
        Err((status, error)) => return identity.rejected(status, error),
    };
    let tenant_id = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.tenant_id.as_str())
        .unwrap_or("default")
        .to_string();
    let caller_role = req_ext
        .0
        .as_ref()
        .map(|context| context.role.clone())
        .unwrap_or_else(|| state.rbac_engine.default_role().to_string());
    let caller_grant_expires_at_epoch_secs = req_ext
        .0
        .as_ref()
        .and_then(|context| context.grant_expires_at_epoch_secs);
    let CallApproval {
        required: requires_approval,
        compensable_reserved,
    } = match preflight
        .resolve_approval(&effective_policy, reversibility.as_ref(), &tenant_id)
        .await
    {
        Ok(approval) => approval,
        Err((status, error)) => return identity.rejected(status, error),
    };
    let approval_timeout_secs = effective_policy.approval_timeout_secs;
    let webhook_cfg = effective_policy.webhook.clone();
    let redact_keys = effective_policy.redact_keys.clone();
    let CallIdentity {
        trace_id,
        request_id,
        req_context,
        client_ip,
        idempotency_key,
        retry_base,
        start_time,
    } = identity;

    let mut effective_args = payload.args.clone();

    // Human-in-the-Loop (HITL) Interception Flow
    if requires_approval {
        let sanitized = redact_value(payload.args.clone(), &redact_keys);

        let mut input_requests = std::collections::BTreeMap::new();
        input_requests.insert(
            "hitl_approval".to_string(),
            serde_json::json!({
                "type": "approval_review",
                "capability_id": payload.capability_id,
                "server_id": server_id,
                "sanitized_args": sanitized,
                "timeout_secs": approval_timeout_secs,
            }),
        );

        let (task_record, _task_rx) = state
            .task_registry
            .create_scoped_task(
                crate::tasks::CreateTaskParams {
                    capability_id: payload.capability_id.clone(),
                    server_id: server_id.clone(),
                    args: payload.args.clone(),
                    request_id: Some(request_id.clone()),
                    context: Some(req_context.clone()),
                    idempotency_key: idempotency_key.clone(),
                    initial_status: crate::tasks::TaskStatus::InputRequired,
                    status_message: Some(format!(
                        "Execution suspended awaiting operator approval for capability '{}'",
                        payload.capability_id
                    )),
                    input_requests: Some(input_requests),
                    ttl_ms: Some(approval_timeout_secs * 1000),
                    poll_interval_ms: Some(1000),
                },
                Some(crate::result_sanitization::ResultAccess::new(
                    &tenant_id,
                    prof_ctx.profile_id.as_deref(),
                )),
            )
            .await;

        let approval_request = crate::approvals::CreateApprovalRequest {
            capability_id: payload.capability_id.clone(),
            server_id: server_id.clone(),
            args: payload.args.clone(),
            sanitized_args: sanitized.clone(),
            request_id: Some(request_id.clone()),
            context: Some(req_context.clone()),
            timeout_secs: approval_timeout_secs,
            webhook: webhook_cfg.as_ref(),
        };
        let (approval_id, rx, mut event) = state
            .approval_registry
            .create_audited_approval(approval_request, trace_id.clone(), start_time)
            .await;
        event.client_ip = client_ip.clone();
        event.idempotency_key = idempotency_key.clone();
        event.is_replay = Some(false);
        state.audit_handle.send(event);

        let prefer_async = payload.async_task
            || headers
                .get("prefer")
                .and_then(|h| h.to_str().ok())
                .map(|v| v.to_lowercase().contains("respond-async"))
                .unwrap_or(false);

        if prefer_async {
            let task_id_clone = task_record.task_id.clone();
            let state_clone = state.clone();
            let req_id_clone = request_id.clone();
            let req_ctx_clone = req_context.clone();
            let trace_id_clone = trace_id.clone();
            let server_id_clone = server_id.clone();
            let tool_name_clone = tool_name.clone();
            let effective_args_clone = effective_args.clone();
            let payload_args_clone = payload.args.clone();
            let input_responses_clone = payload.input_responses.clone();
            let request_state_clone = payload.request_state.clone();
            let idempotency_key_clone = idempotency_key.clone();
            let reversibility_clone = reversibility.clone();
            let profile_clone = prof_ctx.clone();
            let policy_clone = effective_policy.clone();
            let tenant_id_clone = tenant_id.clone();
            let caller_role_clone = caller_role.clone();
            let caller_grant_expires_at_epoch_secs_clone = caller_grant_expires_at_epoch_secs;
            let context_clone = req_context.clone();
            let capability_clone = payload.capability_id.clone();
            let reserved_clone = compensable_reserved;

            tokio::spawn(async move {
                let mut worker_args = effective_args_clone;
                if let Some(trx) = _task_rx {
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
                                    let _ = state_clone
                                        .task_registry
                                        .cancel_task(&task_id_clone, reason)
                                        .await;
                                    state_clone
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
                            let _ = state_clone
                                .task_registry
                                .fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "APPROVAL_TIMEOUT"}),
                                    Some("Approval request timed out or cancelled".to_string()),
                                )
                                .await;
                            state_clone
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

                if !reserved_clone {
                    if let Some(contract) = reversibility_clone.as_ref().filter(|contract| {
                        contract.class == crate::config::ReversibilityClass::Compensable
                    }) {
                        match crate::engine::undo::reserve_compensable(
                            &state_clone,
                            contract,
                            &capability_clone,
                            &tenant_id_clone,
                            profile_clone.profile_id.as_deref(),
                            idempotency_key_clone.as_deref(),
                            true,
                        )
                        .await
                        {
                            Ok(true) => {}
                            Ok(false) | Err(_) => {
                                let _ = state_clone.task_registry.fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "COMPENSABLE_LIMIT_STORAGE_FAILED"}),
                                    Some("Could not reserve compensable invocation".to_string()),
                                ).await;
                                state_clone
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

                if let Some(contract) = reversibility_clone
                    .as_ref()
                    .filter(|contract| contract.class == ReversibilityClass::Delayable)
                {
                    EmissionOrigin {
                        profile: &profile_clone,
                        tool_name: &tool_name_clone,
                        input_responses: input_responses_clone.as_ref(),
                        request_state: request_state_clone.as_deref(),
                        tenant_id: &tenant_id_clone,
                        role: &caller_role_clone,
                        authorization_expires_at_epoch_secs:
                            caller_grant_expires_at_epoch_secs_clone,
                        claim: idempotency_claim,
                    }
                    .queue_after_approval(
                        &state_clone,
                        contract,
                        &task_id_clone,
                        worker_args,
                        idempotency_key_clone.as_deref(),
                    )
                    .await;
                    return;
                }

                let (snapshot, snapshot_hash) = match capture_call_snapshot(
                    &state_clone,
                    reversibility_clone.as_ref(),
                    &worker_args,
                    &profile_clone,
                    &policy_clone,
                )
                .await
                {
                    Ok(snapshot) => snapshot,
                    Err((_, error)) => {
                        let _ = state_clone
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                json!({"code": error.code}),
                                Some(error.message),
                            )
                            .await;
                        state_clone
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                        return;
                    }
                };

                let tx = {
                    let servers_guard = state_clone.servers.read().await;
                    match servers_guard.get(&server_id_clone).cloned() {
                        Some(tx) => tx,
                        None => {
                            let _ = state_clone
                                .task_registry
                                .fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "SERVER_UNREACHABLE"}),
                                    Some(format!("Server '{}' is unreachable", server_id_clone)),
                                )
                                .await;
                            state_clone
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
                        input_responses: input_responses_clone,
                        request_state: request_state_clone,
                        reply: reply_tx,
                    })
                    .await
                    .is_err()
                {
                    state_clone
                        .circuit_breakers
                        .record_failure(&server_id_clone)
                        .await;
                    let _ = state_clone
                        .task_registry
                        .fail_task(
                            &task_id_clone,
                            serde_json::json!({"code": "SERVER_UNREACHABLE"}),
                            Some(format!("Server '{}' mailbox is closed", server_id_clone)),
                        )
                        .await;
                    state_clone
                        .idempotency_store
                        .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                        .await;
                    return;
                }

                let distill_opts = crate::context_filter::DistillationOptions::from_args(Some(
                    &payload_args_clone,
                ));
                match reply_rx.await {
                    Ok(Ok(data)) => {
                        state_clone
                            .circuit_breakers
                            .record_success(&server_id_clone)
                            .await;
                        if let Some(contract) = reversibility_clone.as_ref() {
                            if let Err(error) = crate::engine::undo::register_inverse(
                                &state_clone,
                                contract,
                                crate::engine::undo::ForwardCall {
                                    capability: &capability_clone,
                                    args: &worker_args,
                                    result: &data,
                                    snapshot: &snapshot,
                                    snapshot_hash,
                                    context: &context_clone,
                                    batch_id: None,
                                    tenant_id: &tenant_id_clone,
                                    profile_id: profile_clone.profile_id.as_deref(),
                                },
                                &profile_clone,
                                &policy_clone,
                            )
                            .await
                            {
                                let _ = state_clone
                                    .task_registry
                                    .fail_task(
                                        &task_id_clone,
                                        serde_json::json!({"code": "UNDO_REGISTRATION_FAILED"}),
                                        Some(error.to_string()),
                                    )
                                    .await;
                                if let (Some(key), Some(claim)) =
                                    (idempotency_key_clone.as_deref(), idempotency_claim)
                                {
                                    let failed_response = crate::http_v1::types::error_envelope(
                                        trace_id_clone.clone(),
                                        Some(req_id_clone.clone()),
                                        Some(req_ctx_clone.clone()),
                                        retry_base("completed"),
                                        "UNDO_REGISTRATION_FAILED",
                                        format!("Forward call succeeded but inverse registration failed: {error}"),
                                        false,
                                    );
                                    state_clone
                                        .idempotency_store
                                        .complete(key, claim, failed_response)
                                        .await;
                                }
                                return;
                            }
                        }
                        let (outcome, revision) = state_clone
                            .complete_tool_delivery(
                                &server_id_clone,
                                &trace_id_clone,
                                data,
                                Some(&task_id_clone),
                                &distill_opts,
                            )
                            .await;

                        let response_json = match outcome {
                            Ok(data) => json!({
                                "ok": true,
                                "request_id": req_id_clone,
                                "context": req_ctx_clone,
                                "trace_id": trace_id_clone,
                                "data": data,
                                "error": null,
                                "retry": retry_base("completed"),
                            }),
                            Err(error) => error_envelope(
                                trace_id_clone,
                                Some(req_id_clone),
                                Some(req_ctx_clone),
                                retry_base("completed"),
                                &error.code,
                                error.message,
                                error.retryable,
                            ),
                        };

                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            state_clone
                                .idempotency_store
                                .complete_with_revision(key, claim, response_json, revision)
                                .await;
                        }
                    }
                    Ok(Err(UpstreamCallError::Timeout)) => {
                        state_clone
                            .circuit_breakers
                            .record_failure(&server_id_clone)
                            .await;
                        let _ = state_clone
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "UPSTREAM_TIMEOUT"}),
                                Some(format!(
                                    "Tool call timed out after {}ms",
                                    state_clone.tool_timeout_ms
                                )),
                            )
                            .await;
                        state_clone
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                    }
                    Ok(Err(UpstreamCallError::Upstream(err))) => {
                        let err = state_clone.safe_upstream_error(err).await;
                        state_clone
                            .circuit_breakers
                            .record_failure(&server_id_clone)
                            .await;
                        let _ = state_clone
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "UPSTREAM_ERROR", "message": &err}),
                                Some(err),
                            )
                            .await;
                        state_clone
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                    }
                    Err(_) => {
                        state_clone
                            .circuit_breakers
                            .record_failure(&server_id_clone)
                            .await;
                        let _ = state_clone
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "INTERNAL_ERROR"}),
                                Some("Server actor task dropped reply channel".to_string()),
                            )
                            .await;
                        state_clone
                            .idempotency_store
                            .release_execution(idempotency_key_clone.as_deref(), idempotency_claim)
                            .await;
                    }
                }
            });

            let task_resp = crate::tasks::TaskResponse::from(&task_record);
            return (
                StatusCode::ACCEPTED,
                Json(json!({
                    "ok": true,
                    "status": "pending_approval",
                    "approval_id": approval_id,
                    "resultType": "task",
                    "task": task_resp,
                    "request_id": request_id,
                    "trace_id": trace_id,
                })),
            )
                .into_response();
        }

        // Synchronous caller suspension on approval wait channel
        let resolution = tokio::select! {
            _ = cancel_token.cancelled() => {
                release_registered_call(&state, &request_id, idempotency_key.as_deref(), idempotency_claim).await;
                return (
                    StatusCode::BAD_REQUEST,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("unknown"),
                        "OPERATION_CANCELLED",
                        "Tool call operation was cancelled while awaiting approval",
                        false,
                    )),
                ).into_response();
            }
            res = rx => match res {
                Ok(r) => r,
                Err(_) => crate::approvals::ApprovalResolution::Expired,
            }
        };

        match resolution {
            crate::approvals::ApprovalResolution::Approved { modified_args, .. } => {
                if let Some(mod_args) = modified_args {
                    info!(
                        approval_id = %approval_id,
                        capability_id = %payload.capability_id,
                        "resuming tool call with operator-modified arguments"
                    );
                    effective_args = mod_args;
                }
            }
            crate::approvals::ApprovalResolution::Rejected { operator, reason } => {
                release_registered_call(
                    &state,
                    &request_id,
                    idempotency_key.as_deref(),
                    idempotency_claim,
                )
                .await;
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ApprovalRejected,
                    trace_id: trace_id.clone(),
                    request_id: Some(request_id.clone()),
                    actor_id: req_context.actor_id.clone(),
                    work_item_id: req_context.work_item_id.clone(),
                    client_ip: client_ip.clone(),
                    server_id: Some(server_id.clone()),
                    capability_id: Some(payload.capability_id.clone()),
                    resource_uri: None,
                    sanitized_args: Some(sanitized),
                    sanitized_response: None,
                    execution_latency_us: Some(start_time.elapsed().as_micros() as u64),
                    status: crate::audit::AuditEventStatus::Denied,
                    error_code: Some("OPERATION_REJECTED_BY_OPERATOR".to_string()),
                    error_message: reason.clone(),
                    operator_id: Some(operator.clone()),
                    approval_ticket_id: Some(approval_id),
                    idempotency_key: idempotency_key.clone(),
                    is_replay: Some(false),
                });
                let reason_str = reason.map(|r| format!(": {}", r)).unwrap_or_default();
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "ok": false,
                        "request_id": request_id,
                        "context": req_context,
                        "trace_id": trace_id,
                        "data": null,
                        "error": {
                            "code": "OPERATION_REJECTED_BY_OPERATOR",
                            "message": format!("Human operator rejected execution{}", reason_str),
                            "operator": operator,
                            "retryable": false,
                        },
                        "retry": retry_base("not_started"),
                    })),
                )
                    .into_response();
            }
            crate::approvals::ApprovalResolution::Expired => {
                release_registered_call(
                    &state,
                    &request_id,
                    idempotency_key.as_deref(),
                    idempotency_claim,
                )
                .await;
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ApprovalExpired,
                    trace_id: trace_id.clone(),
                    request_id: Some(request_id.clone()),
                    actor_id: req_context.actor_id.clone(),
                    work_item_id: req_context.work_item_id.clone(),
                    client_ip: client_ip.clone(),
                    server_id: Some(server_id.clone()),
                    capability_id: Some(payload.capability_id.clone()),
                    resource_uri: None,
                    sanitized_args: Some(sanitized),
                    sanitized_response: None,
                    execution_latency_us: Some(start_time.elapsed().as_micros() as u64),
                    status: crate::audit::AuditEventStatus::Denied,
                    error_code: Some("APPROVAL_TIMEOUT".to_string()),
                    error_message: Some(format!(
                        "Approval request timed out after {}s",
                        approval_timeout_secs
                    )),
                    operator_id: None,
                    approval_ticket_id: Some(approval_id),
                    idempotency_key: idempotency_key.clone(),
                    is_replay: Some(false),
                });
                return (
                    StatusCode::GATEWAY_TIMEOUT,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("not_started"),
                        "APPROVAL_TIMEOUT",
                        format!(
                            "Approval request timed out after {}s",
                            approval_timeout_secs
                        ),
                        true,
                    )),
                )
                    .into_response();
            }
        }
    }

    if requires_approval && !compensable_reserved {
        if let Some(contract) = reversibility
            .as_ref()
            .filter(|contract| contract.class == crate::config::ReversibilityClass::Compensable)
        {
            if let Err(error) = crate::engine::undo::reserve_compensable(
                &state,
                contract,
                &payload.capability_id,
                &tenant_id,
                prof_ctx.profile_id.as_deref(),
                idempotency_key.as_deref(),
                true,
            )
            .await
            {
                release_registered_call(
                    &state,
                    &request_id,
                    idempotency_key.as_deref(),
                    idempotency_claim,
                )
                .await;
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("not_started"),
                        "COMPENSABLE_LIMIT_STORAGE_FAILED",
                        error.to_string(),
                        true,
                    )),
                )
                    .into_response();
            }
        }
    }

    if let Some(contract) = reversibility
        .as_ref()
        .filter(|contract| contract.class == ReversibilityClass::Delayable)
    {
        return HeldCall {
            state: &state,
            identity: CallIdentity {
                trace_id,
                request_id,
                req_context,
                client_ip,
                idempotency_key,
                retry_base,
                start_time,
            },
            payload: &payload,
            server_id: &server_id,
            origin: EmissionOrigin {
                profile: &prof_ctx,
                tool_name: &tool_name,
                input_responses: payload.input_responses.as_ref(),
                request_state: payload.request_state.as_deref(),
                tenant_id: &tenant_id,
                role: &caller_role,
                authorization_expires_at_epoch_secs: caller_grant_expires_at_epoch_secs,
                claim: idempotency_claim,
            },
        }
        .schedule(contract, effective_args, requires_approval, &headers)
        .await;
    }

    let (snapshot, snapshot_hash) = match capture_call_snapshot(
        &state,
        reversibility.as_ref(),
        &effective_args,
        &prof_ctx,
        &effective_policy,
    )
    .await
    {
        Ok(snapshot) => snapshot,
        Err((status, error)) => {
            release_registered_call(
                &state,
                &request_id,
                idempotency_key.as_deref(),
                idempotency_claim,
            )
            .await;
            return (
                status,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    error.code,
                    error.message,
                    error.retryable,
                )),
            )
                .into_response();
        }
    };

    let tx = {
        let servers_guard = state.servers.read().await;
        let Some(tx) = servers_guard.get(&server_id).cloned() else {
            release_registered_call(
                &state,
                &request_id,
                idempotency_key.as_deref(),
                idempotency_claim,
            )
            .await;
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' is unreachable", server_id),
                    true,
                )),
            )
                .into_response();
        };
        tx
    };

    let redacted_input = redact_value(effective_args.clone(), &redact_keys);
    info!(
        trace_id = %trace_id,
        request_id = %request_id,
        operation_id = ?req_context.operation_id,
        work_item_id = ?req_context.work_item_id,
        actor_id = ?req_context.actor_id,
        grant_id = ?req_context.grant_id,
        capability_id = %payload.capability_id,
        args = %redacted_input,
        "tool call start"
    );

    // Circuit Breaker Permission Check
    if let Err(cb_err) = state.circuit_breakers.check_permission(&server_id).await {
        release_registered_call(
            &state,
            &request_id,
            idempotency_key.as_deref(),
            idempotency_claim,
        )
        .await;
        state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolExecution,
            trace_id: trace_id.clone(),
            request_id: Some(request_id.clone()),
            actor_id: req_context.actor_id.clone(),
            work_item_id: req_context.work_item_id.clone(),
            client_ip: client_ip.clone(),
            server_id: Some(server_id.clone()),
            capability_id: Some(payload.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(redacted_input),
            sanitized_response: None,
            execution_latency_us: Some(start_time.elapsed().as_micros() as u64),
            status: crate::audit::AuditEventStatus::Failed,
            error_code: Some("CIRCUIT_OPEN".to_string()),
            error_message: Some(cb_err.to_string()),
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: idempotency_key.clone(),
            is_replay: Some(false),
        });
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                retry_base("not_started"),
                "CIRCUIT_OPEN",
                cb_err.to_string(),
                false,
            )),
        )
            .into_response();
    }

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::CallTool {
            name: tool_name,
            params: effective_args.clone(),
            input_responses: payload.input_responses,
            request_state: payload.request_state,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        state.circuit_breakers.record_failure(&server_id).await;
        release_registered_call(
            &state,
            &request_id,
            idempotency_key.as_deref(),
            idempotency_claim,
        )
        .await;
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                retry_base("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", server_id),
                true,
            )),
        )
            .into_response();
    }

    let result: Result<Result<Value, UpstreamCallError>, oneshot::error::RecvError> = tokio::select! {
        _ = cancel_token.cancelled() => {
            release_registered_call(&state, &request_id, idempotency_key.as_deref(), idempotency_claim).await;
            return (
                StatusCode::BAD_REQUEST,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("unknown"),
                    "OPERATION_CANCELLED",
                    "Tool call operation was cancelled",
                    false,
                )),
            ).into_response();
        }
        res = reply_rx => res,
    };

    state.operation_registry.unregister(&request_id).await;

    let audit = CallAudit {
        trace_id: &trace_id,
        request_id: &request_id,
        context: &req_context,
        client_ip: client_ip.as_deref(),
        server_id: &server_id,
        capability_id: &payload.capability_id,
        idempotency_key: idempotency_key.as_deref(),
    };
    match result {
        Ok(Ok(data)) => {
            state.circuit_breakers.record_success(&server_id).await;
            if let Some(contract) = reversibility.as_ref() {
                if let Err(error) = crate::engine::undo::register_inverse(
                    &state,
                    contract,
                    crate::engine::undo::ForwardCall {
                        capability: &payload.capability_id,
                        args: &effective_args,
                        result: &data,
                        snapshot: &snapshot,
                        snapshot_hash,
                        context: &req_context,
                        batch_id: None,
                        tenant_id: &tenant_id,
                        profile_id: prof_ctx.profile_id.as_deref(),
                    },
                    &prof_ctx,
                    &effective_policy,
                )
                .await
                {
                    if let (Some(key), Some(claim)) =
                        (idempotency_key.as_deref(), idempotency_claim)
                    {
                        let failed_response = error_envelope(
                            trace_id.clone(),
                            Some(request_id.clone()),
                            Some(req_context.clone()),
                            retry_base("completed"),
                            "UNDO_REGISTRATION_FAILED",
                            format!(
                                "Forward call succeeded but inverse registration failed: {error}"
                            ),
                            false,
                        );
                        state
                            .idempotency_store
                            .complete(key, claim, failed_response)
                            .await;
                    }
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(error_envelope(
                            trace_id,
                            Some(request_id),
                            Some(req_context),
                            retry_base("unknown"),
                            "UNDO_REGISTRATION_FAILED",
                            format!(
                                "Forward call succeeded but inverse registration failed: {error}"
                            ),
                            false,
                        )),
                    )
                        .into_response();
                }
            }
            let delivery = state
                .sanitize_result(
                    &server_id,
                    &trace_id,
                    data,
                    crate::result_sanitization::Shape::Tool,
                )
                .await;
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            state
                .total_tool_duration_us
                .fetch_add(elapsed_us, Ordering::Relaxed);
            let (status, response_json, revision) = match delivery {
                Ok(delivery) => {
                    let redacted_output = if delivery.revision.is_none() {
                        redact_value(delivery.value.clone(), &redact_keys)
                    } else {
                        Value::Null
                    };
                    info!(
                        trace_id = %trace_id,
                        request_id = %request_id,
                        capability_id = %payload.capability_id,
                        data = %redacted_output,
                        "tool call success"
                    );
                    state.audit_handle.send(audit.event(
                        redacted_input,
                        elapsed_us,
                        Ok(redacted_output),
                    ));

                    let distill_opts =
                        crate::context_filter::DistillationOptions::from_args(Some(&payload.args));
                    let distilled_data =
                        crate::context_filter::distill_value(delivery.value, &distill_opts);
                    let response = json!({
                        "ok": true,
                        "request_id": request_id,
                        "context": req_context,
                        "trace_id": trace_id,
                        "data": distilled_data,
                        "error": null,
                        "retry": retry_base("completed"),
                    });
                    (StatusCode::OK, response, delivery.revision)
                }
                Err(withheld) => {
                    let error = crate::WarmplaneError::from(withheld);
                    let response = error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("completed"),
                        &error.code,
                        error.message,
                        false,
                    );
                    (StatusCode::BAD_GATEWAY, response, None)
                }
            };

            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state
                    .idempotency_store
                    .complete_with_revision(key, claim, response_json.clone(), revision)
                    .await;
            }

            (status, Json(response_json)).into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => {
            state.circuit_breakers.record_failure(&server_id).await;
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            state
                .idempotency_store
                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                .await;
            state.audit_handle.send(audit.event(
                redacted_input,
                elapsed_us,
                Err((
                    "UPSTREAM_TIMEOUT",
                    format!("Tool call timed out after {}ms", state.tool_timeout_ms),
                )),
            ));
            (
                StatusCode::GATEWAY_TIMEOUT,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("unknown"),
                    "UPSTREAM_TIMEOUT",
                    format!("Tool call timed out after {}ms", state.tool_timeout_ms),
                    true,
                )),
            )
                .into_response()
        }
        Ok(Err(UpstreamCallError::Upstream(err))) => {
            let err = state.safe_upstream_error(err).await;
            state.circuit_breakers.record_failure(&server_id).await;
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            state
                .idempotency_store
                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                .await;
            state.audit_handle.send(audit.event(
                redacted_input,
                elapsed_us,
                Err(("UPSTREAM_ERROR", err.clone())),
            ));
            (
                StatusCode::BAD_GATEWAY,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("unknown"),
                    "UPSTREAM_ERROR",
                    err,
                    false,
                )),
            )
                .into_response()
        }
        Err(_) => {
            state
                .idempotency_store
                .release_execution(idempotency_key.as_deref(), idempotency_claim)
                .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("unknown"),
                    "INTERNAL_ERROR",
                    "Daemon actor task died",
                    true,
                )),
            )
                .into_response()
        }
    }
}

struct EmissionOrigin<'a> {
    profile: &'a ProfileContext,
    tool_name: &'a str,
    input_responses: Option<&'a BTreeMap<String, Value>>,
    request_state: Option<&'a str>,
    tenant_id: &'a str,
    role: &'a str,
    authorization_expires_at_epoch_secs: Option<u64>,
    claim: Option<IdempotencyClaim>,
}

impl EmissionOrigin<'_> {
    fn record(&self, release_at_ms: u64, approval_completed: bool) -> EmissionRecord {
        EmissionRecord {
            idempotency_claim: self.claim,
            phase: EmissionPhase::PendingEmission,
            release_at_ms,
            cancellable: true,
            tool_name: self.tool_name.to_owned(),
            input_responses: self.input_responses.cloned(),
            request_state: self.request_state.map(str::to_owned),
            profile_id: self.profile.profile_id.clone(),
            tenant_id: self.tenant_id.to_owned(),
            role: self.role.to_owned(),
            authorization_expires_at_epoch_secs: self.authorization_expires_at_epoch_secs,
            approval_completed,
            batch_id: None,
            step_id: None,
        }
    }

    async fn queue_after_approval(
        &self,
        state: &AppState,
        contract: &ReversibilityConfig,
        task_id: &str,
        args: Value,
        idempotency_key: Option<&str>,
    ) {
        let hold_secs =
            crate::emissions::resolve_call_hold_window_secs(state, self.profile, contract).await;
        let release_at_ms = match crate::emissions::release_deadline_ms(hold_secs) {
            Ok(release_at_ms) => release_at_ms,
            Err(error) => {
                let _ = state
                    .task_registry
                    .fail_task(
                        task_id,
                        json!({"code": "INVALID_HOLD_WINDOW"}),
                        Some(error.to_string()),
                    )
                    .await;
                return;
            }
        };
        if let Err(error) = state
            .hold_coordinator
            .queue_existing(state, task_id, args, self.record(release_at_ms, true))
            .await
        {
            let _ = state
                .task_registry
                .fail_task(
                    task_id,
                    json!({"code": "EMISSION_PERSISTENCE_FAILED"}),
                    Some(error.to_string()),
                )
                .await;
            state
                .idempotency_store
                .release_execution(idempotency_key, self.claim)
                .await;
        }
    }
}

struct HeldCall<'a> {
    state: &'a AppState,
    identity: CallIdentity,
    payload: &'a CallCapabilityRequest,
    server_id: &'a str,
    origin: EmissionOrigin<'a>,
}

impl HeldCall<'_> {
    async fn schedule(
        &self,
        contract: &ReversibilityConfig,
        args: Value,
        approval_completed: bool,
        headers: &HeaderMap,
    ) -> Response {
        let (hold_ms, release_at_ms) = match self.hold_deadline(contract).await {
            Ok(deadline) => deadline,
            Err(error) => return self.reject(StatusCode::BAD_REQUEST, error).await,
        };
        let emission = self.origin.record(release_at_ms, approval_completed);
        let task = match self.persist(args, emission).await {
            Ok(Some(task)) => task,
            Ok(None) => {
                return self
                    .reject(
                        StatusCode::CONFLICT,
                        CallRejection::new(
                            "EMISSION_TASK_STATE_INVALID",
                            "Could not attach the approved emission to its task",
                        ),
                    )
                    .await
            }
            Err(error) => return self.persistence_failure(error).await,
        };
        self.unregister().await;
        let prefer_async = self.payload.async_task
            || headers
                .get("prefer")
                .and_then(|header| header.to_str().ok())
                .is_some_and(|value| value.to_ascii_lowercase().contains("respond-async"));
        if !prefer_async && hold_ms < self.state.tool_timeout_ms {
            if let Some(response) = self.wait_for_result(&task.task_id, hold_ms).await {
                return response;
            }
        }
        self.identity
            .held_receipt(StatusCode::ACCEPTED, self.state.task_view(&task).await)
    }

    async fn hold_deadline(
        &self,
        contract: &ReversibilityConfig,
    ) -> Result<(u64, u64), CallRejection> {
        let seconds = crate::emissions::resolve_call_hold_window_secs(
            self.state,
            self.origin.profile,
            contract,
        )
        .await;
        let hold_ms = seconds.checked_mul(1_000).ok_or_else(|| {
            CallRejection::new("INVALID_HOLD_WINDOW", "Hold window overflows milliseconds")
        })?;
        let release_at_ms = crate::emissions::release_deadline_ms(seconds)
            .map_err(|error| CallRejection::new("INVALID_HOLD_WINDOW", error.to_string()))?;
        Ok((hold_ms, release_at_ms))
    }

    async fn persist(
        &self,
        args: Value,
        emission: EmissionRecord,
    ) -> anyhow::Result<Option<TaskRecord>> {
        let id = &self.identity;
        let coordinator = &self.state.hold_coordinator;
        let existing = self
            .state
            .task_registry
            .list_tasks()
            .await
            .into_iter()
            .find(|task| {
                task.request_id.as_deref() == Some(id.request_id.as_str())
                    && task.emission.is_none()
            });
        if let Some(task) = existing {
            return coordinator
                .queue_existing(self.state, &task.task_id, args, emission)
                .await
                .context("failed to persist approved emission");
        }
        coordinator
            .accept(
                self.state,
                CreateTaskParams {
                    capability_id: self.payload.capability_id.clone(),
                    server_id: self.server_id.to_owned(),
                    args,
                    request_id: Some(id.request_id.clone()),
                    context: Some(id.req_context.clone()),
                    idempotency_key: id.idempotency_key.clone(),
                    initial_status: TaskStatus::Working,
                    status_message: Some("Waiting for the emission release deadline".to_string()),
                    input_requests: None,
                    ttl_ms: None,
                    poll_interval_ms: Some(1_000),
                },
                emission,
            )
            .await
            .map(Some)
    }

    async fn unregister(&self) {
        self.state
            .operation_registry
            .unregister(&self.identity.request_id)
            .await;
    }

    async fn reject(&self, status: StatusCode, error: CallRejection) -> Response {
        self.unregister().await;
        self.identity.rejected(status, error)
    }

    async fn persistence_failure(&self, error: anyhow::Error) -> Response {
        release_registered_call(
            self.state,
            &self.identity.request_id,
            self.identity.idempotency_key.as_deref(),
            self.origin.claim,
        )
        .await;
        self.identity.rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            CallRejection {
                code: "EMISSION_PERSISTENCE_FAILED",
                message: error.to_string(),
                retryable: true,
            },
        )
    }

    async fn wait_for_result(&self, task_id: &str, hold_ms: u64) -> Option<Response> {
        let wait = Duration::from_millis(hold_ms.saturating_add(self.state.tool_timeout_ms));
        let task = self
            .state
            .hold_coordinator
            .wait_for_terminal(self.state, task_id, wait)
            .await?;
        self.completed_response(self.state.task_view(&task).await)
    }

    fn completed_response(&self, task: TaskResponse) -> Option<Response> {
        let id = &self.identity;
        let retry = (id.retry_base)("completed");
        match task.status {
            TaskStatus::Completed if task.error.is_none() => Some(
                (
                    StatusCode::OK,
                    Json(json!({
                        "ok": true,
                        "request_id": id.request_id,
                        "context": id.req_context,
                        "trace_id": id.trace_id,
                        "data": task.result.unwrap_or(Value::Null),
                        "error": null,
                        "retry": retry,
                    })),
                )
                    .into_response(),
            ),
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                let code = task
                    .error
                    .as_ref()
                    .and_then(|error| error.get("code"))
                    .and_then(Value::as_str)
                    .unwrap_or("EMISSION_FAILED");
                let message = task
                    .status_message
                    .unwrap_or_else(|| "Held emission did not complete".to_string());
                Some(
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(error_envelope(
                            id.trace_id.clone(),
                            Some(id.request_id.clone()),
                            Some(id.req_context.clone()),
                            retry,
                            code,
                            message,
                            false,
                        )),
                    )
                        .into_response(),
                )
            }
            _ => None,
        }
    }
}

struct CallAudit<'a> {
    trace_id: &'a str,
    request_id: &'a str,
    context: &'a RequestContext,
    client_ip: Option<&'a str>,
    server_id: &'a str,
    capability_id: &'a str,
    idempotency_key: Option<&'a str>,
}

impl CallAudit<'_> {
    fn event(
        &self,
        sanitized_args: Value,
        elapsed_us: u64,
        outcome: Result<Value, (&str, String)>,
    ) -> crate::audit::RawAuditEvent {
        use crate::audit::{AuditEventStatus, AuditEventType, RawAuditEvent};

        let (status, sanitized_response, error_code, error_message) = match outcome {
            Ok(response) => (AuditEventStatus::Success, Some(response), None, None),
            Err((code, message)) => (
                AuditEventStatus::Failed,
                None,
                Some(code.to_owned()),
                Some(message),
            ),
        };
        RawAuditEvent {
            event_type: AuditEventType::ToolExecution,
            trace_id: self.trace_id.to_owned(),
            request_id: Some(self.request_id.to_owned()),
            actor_id: self.context.actor_id.clone(),
            work_item_id: self.context.work_item_id.clone(),
            client_ip: self.client_ip.map(str::to_owned),
            server_id: Some(self.server_id.to_owned()),
            capability_id: Some(self.capability_id.to_owned()),
            resource_uri: None,
            sanitized_args: Some(sanitized_args),
            sanitized_response,
            execution_latency_us: Some(elapsed_us),
            status,
            error_code,
            error_message,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: self.idempotency_key.map(str::to_owned),
            is_replay: Some(false),
        }
    }
}

async fn capture_call_snapshot(
    state: &AppState,
    contract: Option<&ReversibilityConfig>,
    args: &Value,
    profile: &ProfileContext,
    policy: &Policy,
) -> Result<(Value, Option<String>), (StatusCode, CallRejection)> {
    let Some(contract) = contract else {
        return Ok((Value::Null, None));
    };
    if matches!(
        contract.class,
        ReversibilityClass::Reversible | ReversibilityClass::Compensable
    ) && state.undo_ledger.is_none()
    {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            CallRejection::new(
                "UNDO_STORAGE_UNAVAILABLE",
                "Persistent undo storage is unavailable",
            ),
        ));
    }
    let Some(snapshot) = &contract.pre_snapshot else {
        return Ok((Value::Null, None));
    };
    crate::engine::undo::capture_pre_snapshot(state, snapshot, args, profile, policy)
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                CallRejection {
                    code: "PRE_SNAPSHOT_FAILED",
                    message: error.to_string(),
                    retryable: true,
                },
            )
        })
}

struct CallIdentity {
    trace_id: String,
    request_id: String,
    req_context: RequestContext,
    client_ip: Option<String>,
    idempotency_key: Option<String>,
    retry_base: fn(&'static str) -> RetryMetadata,
    start_time: Instant,
}

impl CallIdentity {
    fn new(
        state: &AppState,
        payload: &CallCapabilityRequest,
        headers: &HeaderMap,
        tenant: Option<&TenantContext>,
    ) -> Self {
        let start_time = Instant::now();
        state.total_tool_calls.fetch_add(1, Ordering::Relaxed);
        let trace_id = next_trace_id();
        let request_id = crate::context::resolve_request_id(
            payload.request_id.clone(),
            headers,
            trace_id.clone(),
        );
        let mut req_context =
            crate::context::resolve_request_context(payload.context.clone(), headers);
        if let Some(tenant) = tenant {
            if let Some(actor) = &tenant.actor_id {
                req_context.actor_id = Some(actor.clone());
            }
            if let Some(grant) = &tenant.grant_id {
                req_context.grant_id = Some(grant.clone());
            }
        }

        let client_ip = resolve_client_ip(headers);
        let idempotency_key = resolve_idempotency_key(payload.idempotency_key.clone(), headers)
            .or_else(|| {
                payload.args.is_object().then(|| {
                    crate::idempotency::derive_idempotency_key(
                        &payload.capability_id,
                        &payload.args,
                        req_context.actor_id.as_deref(),
                        payload.request_id.as_deref(),
                    )
                })
            });
        let retry_base = if idempotency_key.is_some() {
            RetryMetadata::idempotent
        } else {
            RetryMetadata::unsafe_op
        };
        Self {
            trace_id,
            request_id,
            req_context,
            client_ip,
            idempotency_key,
            retry_base,
            start_time,
        }
    }

    fn rejected(&self, status: StatusCode, error: CallRejection) -> Response {
        (
            status,
            Json(error_envelope(
                self.trace_id.clone(),
                Some(self.request_id.clone()),
                Some(self.req_context.clone()),
                (self.retry_base)("not_started"),
                error.code,
                error.message,
                error.retryable,
            )),
        )
            .into_response()
    }

    async fn replay_held_call(
        &self,
        state: &AppState,
        key: &str,
        tenant: Option<&TenantContext>,
        profile: Option<&ProfileContext>,
    ) -> Option<Response> {
        let task = state.task_registry.find_by_idempotency_key(key).await?;
        task.emission.as_ref()?;
        if !crate::http_v1::helpers::can_access_emission_task(state, &task, tenant, profile).await {
            return Some(self.rejected(
                StatusCode::FORBIDDEN,
                CallRejection::new(
                    "POLICY_DENIED",
                    "Held task is not accessible to this caller",
                ),
            ));
        }
        let status = if matches!(
            task.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            StatusCode::OK
        } else {
            StatusCode::ACCEPTED
        };
        Some(self.held_receipt(status, state.task_view(&task).await))
    }

    fn held_receipt(&self, status: StatusCode, task: TaskResponse) -> Response {
        (
            status,
            Json(json!({
                "ok": true,
                "resultType": "task",
                "task": task,
                "request_id": self.request_id,
                "trace_id": self.trace_id,
            })),
        )
            .into_response()
    }

    async fn claim_call(
        &self,
        state: &AppState,
        payload: &CallCapabilityRequest,
        tenant: Option<&TenantContext>,
        profile: Option<&ProfileContext>,
    ) -> ControlFlow<Response, Option<IdempotencyClaim>> {
        let Some(key) = self.idempotency_key.as_deref() else {
            return ControlFlow::Continue(None);
        };
        if let Some(response) = self.replay_held_call(state, key, tenant, profile).await {
            return ControlFlow::Break(response);
        }
        let access = crate::result_sanitization::ResultAccess::new(
            tenant.map_or("default", |context| context.tenant_id.as_str()),
            profile.and_then(|profile| profile.profile_id.as_deref()),
        );
        match state
            .idempotency_store
            .claim_or_replay(key, &payload.capability_id, &self.trace_id, access.clone())
            .await
        {
            ExecutionClaim::New(claim) => ControlFlow::Continue(Some(claim)),
            ExecutionClaim::Replay {
                response,
                from_cache,
            } => {
                let default_role = state.rbac_engine.default_role();
                let role = tenant.map_or(default_role.as_str(), |context| context.role.as_str());
                if let Err(error) = state
                    .validate_replay(key, &payload.capability_id, &response, &access, role)
                    .await
                {
                    let status = if error.code == "POLICY_DENIED" {
                        StatusCode::FORBIDDEN
                    } else {
                        StatusCode::BAD_GATEWAY
                    };
                    return ControlFlow::Break(
                        (
                            status,
                            Json(error_envelope(
                                self.trace_id.clone(),
                                Some(self.request_id.clone()),
                                Some(self.req_context.clone()),
                                (self.retry_base)("completed"),
                                &error.code,
                                error.message,
                                false,
                            )),
                        )
                            .into_response(),
                    );
                }
                if from_cache {
                    let logging_enabled = state.accepts_result_revision(None).await;
                    self.audit_cached_replay(
                        state,
                        key,
                        payload,
                        logging_enabled.then_some(&response),
                    );
                }
                let mut headers = HeaderMap::new();
                headers.insert(
                    "x-warmplane-deduplicated",
                    axum::http::HeaderValue::from_static("true"),
                );
                ControlFlow::Break((StatusCode::OK, headers, Json(response)).into_response())
            }
        }
    }

    fn audit_cached_replay(
        &self,
        state: &AppState,
        key: &str,
        payload: &CallCapabilityRequest,
        response: Option<&Value>,
    ) {
        state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolExecution,
            trace_id: self.trace_id.clone(),
            request_id: Some(self.request_id.clone()),
            actor_id: self.req_context.actor_id.clone(),
            work_item_id: self.req_context.work_item_id.clone(),
            client_ip: self.client_ip.clone(),
            server_id: None,
            capability_id: Some(payload.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(payload.args.clone()),
            sanitized_response: response.cloned(),
            execution_latency_us: Some(self.start_time.elapsed().as_micros() as u64),
            status: crate::audit::AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: Some(key.to_owned()),
            is_replay: Some(true),
        });
    }
}

struct CallRejection {
    code: &'static str,
    message: String,
    retryable: bool,
}

impl CallRejection {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: false,
        }
    }
}

struct CallTarget {
    server_id: String,
    tool_name: String,
    reversibility: Option<ReversibilityConfig>,
}

struct CallApproval {
    required: bool,
    compensable_reserved: bool,
}

struct CallPreflight<'a> {
    state: &'a AppState,
    payload: &'a CallCapabilityRequest,
    identity: &'a CallIdentity,
    profile: &'a ProfileContext,
    tenant: Option<&'a TenantContext>,
    claim: Option<IdempotencyClaim>,
}

impl CallPreflight<'_> {
    async fn reject(
        &self,
        status: StatusCode,
        error: CallRejection,
    ) -> (StatusCode, CallRejection) {
        release_registered_call(
            self.state,
            &self.identity.request_id,
            self.identity.idempotency_key.as_deref(),
            self.claim,
        )
        .await;
        (status, error)
    }

    async fn validate_arguments(&self) -> Result<(), (StatusCode, CallRejection)> {
        if self.payload.args.is_object() {
            return Ok(());
        }
        Err(self
            .reject(
                StatusCode::BAD_REQUEST,
                CallRejection::new("INVALID_ARGS", "'args' must be a JSON object"),
            )
            .await)
    }

    async fn resolve_policy(&self) -> Result<Policy, (StatusCode, CallRejection)> {
        let base = self.state.policy.read().await;
        let policy = self
            .tenant
            .map(|tenant| tenant.effective_policy.clone())
            .unwrap_or_else(|| base.clone())
            .merge_with_profile(self.profile.profile_policy.as_ref());

        if policy.allows(&self.payload.capability_id) {
            return Ok(policy);
        }
        let message = format!(
            "Capability '{}' blocked by policy",
            self.payload.capability_id
        );
        release_registered_call(
            self.state,
            &self.identity.request_id,
            self.identity.idempotency_key.as_deref(),
            self.claim,
        )
        .await;
        self.audit_policy_denial(&policy, &message);
        Err((
            StatusCode::FORBIDDEN,
            CallRejection::new("POLICY_DENIED", message),
        ))
    }

    fn audit_policy_denial(&self, policy: &Policy, message: &str) {
        let identity = self.identity;
        self.state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::PolicyViolation,
            trace_id: identity.trace_id.clone(),
            request_id: Some(identity.request_id.clone()),
            actor_id: identity.req_context.actor_id.clone(),
            work_item_id: identity.req_context.work_item_id.clone(),
            client_ip: identity.client_ip.clone(),
            server_id: None,
            capability_id: Some(self.payload.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(redact_value(self.payload.args.clone(), &policy.redact_keys)),
            sanitized_response: None,
            execution_latency_us: Some(identity.start_time.elapsed().as_micros() as u64),
            status: crate::audit::AuditEventStatus::Denied,
            error_code: Some("POLICY_DENIED".to_string()),
            error_message: Some(message.to_owned()),
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: identity.idempotency_key.clone(),
            is_replay: Some(false),
        });
    }

    async fn resolve_target(&self) -> Result<CallTarget, (StatusCode, CallRejection)> {
        let capabilities = self.state.capabilities.read().await;
        let capability_id = &self.payload.capability_id;
        let Some(meta) = capabilities.get(capability_id) else {
            return Err(self
                .reject(
                    StatusCode::NOT_FOUND,
                    CallRejection::new(
                        "TOOL_NOT_FOUND",
                        format!("Capability '{capability_id}' not found"),
                    ),
                )
                .await);
        };
        if !self.profile.is_server_allowed(&meta.server) {
            return Err(self.reject(
                StatusCode::FORBIDDEN,
                CallRejection::new(
                    "TOOL_NOT_IN_PROFILE",
                    format!("Capability '{capability_id}' belongs to server '{}' which is not in active profile", meta.server),
                ),
            ).await);
        }
        Ok(CallTarget {
            server_id: meta.server.clone(),
            tool_name: meta.tool.clone(),
            reversibility: meta.reversibility.clone(),
        })
    }

    async fn resolve_approval(
        &self,
        policy: &Policy,
        contract: Option<&ReversibilityConfig>,
        tenant_id: &str,
    ) -> Result<CallApproval, (StatusCode, CallRejection)> {
        let required = policy.requires_approval(&self.payload.capability_id);
        let Some(contract) =
            contract.filter(|contract| contract.class == ReversibilityClass::Compensable)
        else {
            return Ok(CallApproval {
                required,
                compensable_reserved: false,
            });
        };
        if self.state.undo_ledger.is_none() {
            return Err(self
                .reject(
                    StatusCode::SERVICE_UNAVAILABLE,
                    CallRejection::new(
                        "UNDO_STORAGE_UNAVAILABLE",
                        "Persistent undo storage is unavailable",
                    ),
                )
                .await);
        }
        if required {
            return Ok(CallApproval {
                required,
                compensable_reserved: false,
            });
        }
        match crate::engine::undo::reserve_compensable(
            self.state,
            contract,
            &self.payload.capability_id,
            tenant_id,
            self.profile.profile_id.as_deref(),
            self.identity.idempotency_key.as_deref(),
            false,
        )
        .await
        {
            Ok(reserved) => Ok(CallApproval {
                required: !reserved,
                compensable_reserved: reserved,
            }),
            Err(error) => Err(self
                .reject(
                    StatusCode::SERVICE_UNAVAILABLE,
                    CallRejection {
                        code: "COMPENSABLE_LIMIT_STORAGE_FAILED",
                        message: error.to_string(),
                        retryable: true,
                    },
                )
                .await),
        }
    }
}

async fn release_registered_call(
    state: &AppState,
    request_id: &str,
    idempotency_key: Option<&str>,
    claim: Option<IdempotencyClaim>,
) {
    state.operation_registry.unregister(request_id).await;
    state
        .idempotency_store
        .release_execution(idempotency_key, claim)
        .await;
}

/// Handles HTTP POST `/v1/operations/:id/cancel` cancelling an in-flight operation.
pub async fn handle_cancel_operation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
) -> impl IntoResponse {
    if let Some(task) = state
        .task_registry
        .list_tasks()
        .await
        .into_iter()
        .find(|task| task.request_id.as_deref() == Some(id.as_str()) && task.emission.is_some())
    {
        if !crate::http_v1::helpers::can_access_emission_task(
            &state,
            &task,
            req_ext.0.as_ref(),
            prof_ext.as_ref().map(|extension| &extension.0),
        )
        .await
        {
            return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "request_id": id, "cancelled": false, "error": {"code": "NOT_FOUND", "message": "Operation not found"}}))).into_response();
        }
        match state
            .hold_coordinator
            .cancel(
                &state,
                &task.task_id,
                Some("Operation cancelled by caller".to_string()),
            )
            .await
        {
            Ok(crate::tasks::EmissionCancelResult::Cancelled)
            | Ok(crate::tasks::EmissionCancelResult::AlreadyCancelled) => {
                let _ = state.operation_registry.cancel(&id).await;
                return (StatusCode::OK, Json(json!({"ok": true, "request_id": id, "cancelled": true, "task_id": task.task_id}))).into_response();
            }
            Ok(crate::tasks::EmissionCancelResult::TooLate) => {
                return (StatusCode::CONFLICT, Json(json!({"ok": false, "request_id": id, "cancelled": false, "error": {"code": "EMISSION_DISPATCHING", "message": "Emission dispatch has already started and may have reached the upstream server"}}))).into_response();
            }
            Ok(crate::tasks::EmissionCancelResult::NotEmission) => {}
            Err(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "request_id": id, "cancelled": false, "error": {"code": "TASK_CANCEL_FAILED", "message": error.to_string()}}))).into_response();
            }
        }
    }
    let cancelled = state.operation_registry.cancel(&id).await;
    if cancelled {
        if let Some(task) = state
            .task_registry
            .list_tasks()
            .await
            .into_iter()
            .find(|task| task.request_id.as_deref() == Some(id.as_str()) && task.emission.is_some())
        {
            if !crate::http_v1::helpers::can_access_emission_task(
                &state,
                &task,
                req_ext.0.as_ref(),
                prof_ext.as_ref().map(|extension| &extension.0),
            )
            .await
            {
                return (StatusCode::NOT_FOUND, Json(json!({"ok": false, "request_id": id, "cancelled": false, "error": {"code": "NOT_FOUND", "message": "Operation not found"}}))).into_response();
            }
            match state
                .hold_coordinator
                .cancel(
                    &state,
                    &task.task_id,
                    Some("Operation cancelled by caller".to_string()),
                )
                .await
            {
                Ok(
                    crate::tasks::EmissionCancelResult::Cancelled
                    | crate::tasks::EmissionCancelResult::AlreadyCancelled,
                ) => {
                    let _ = state.operation_registry.cancel(&id).await;
                    return (StatusCode::OK, Json(json!({"ok": true, "request_id": id, "cancelled": true, "task_id": task.task_id}))).into_response();
                }
                Ok(crate::tasks::EmissionCancelResult::TooLate) => {
                    return (StatusCode::CONFLICT, Json(json!({"ok": false, "request_id": id, "cancelled": false, "error": {"code": "EMISSION_DISPATCHING", "message": "Emission dispatch has already started and may have reached the upstream server"}}))).into_response();
                }
                Ok(crate::tasks::EmissionCancelResult::NotEmission) => {}
                Err(error) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "request_id": id, "cancelled": false, "error": {"code": "TASK_CANCEL_FAILED", "message": error.to_string()}}))).into_response();
                }
            }
        }
    }
    if !cancelled {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "request_id": id,
                "cancelled": false,
                "error": {
                    "code": "NOT_FOUND",
                    "message": format!("Operation '{}' not found or already completed", id)
                }
            })),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "request_id": id,
            "cancelled": true,
        })),
    )
        .into_response()
}

/// Handles HTTP POST `/v1/tools/batch_call` executing multiple chained tool capabilities.
pub async fn handle_batch_call_capabilities(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
    Json(payload): Json<crate::batch_executor::BatchCallRequest>,
) -> impl IntoResponse {
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let trace_id = next_trace_id();
    let request_id =
        crate::context::resolve_request_id(payload.request_id.clone(), &headers, trace_id.clone());
    let mut req_context =
        crate::context::resolve_request_context(payload.context.clone(), &headers);

    if let Some(ref tenant_ctx) = req_ext.0 {
        if let Some(ref actor) = tenant_ctx.actor_id {
            req_context.actor_id = Some(actor.clone());
        }
        if let Some(ref grant) = tenant_ctx.grant_id {
            req_context.grant_id = Some(grant.clone());
        }
    }

    let base_pol = state.policy.read().await;
    let pol = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.effective_policy.clone())
        .unwrap_or_else(|| base_pol.clone())
        .merge_with_profile(prof_ctx.profile_policy.as_ref());
    let cancellation_token = state.operation_registry.register(&request_id).await;

    let default_role = state.rbac_engine.default_role();
    let response = crate::batch_executor::execute_batch_for_tenant_with_rollback(
        &state,
        payload.steps,
        trace_id,
        Some(request_id.clone()),
        Some(req_context),
        crate::batch_executor::BatchExecutionScope {
            policy: &pol,
            profile: &prof_ctx,
            tenant_id: req_ext
                .0
                .as_ref()
                .map(|ctx| ctx.tenant_id.as_str())
                .unwrap_or("default"),
            role: req_ext
                .0
                .as_ref()
                .map(|ctx| ctx.role.as_str())
                .unwrap_or(default_role.as_str()),
            authorization_expires_at_epoch_secs: req_ext
                .0
                .as_ref()
                .and_then(|ctx| ctx.grant_expires_at_epoch_secs),
            cancellation_token,
        },
        payload.auto_rollback_on_failure,
    )
    .await;
    state.operation_registry.unregister(&request_id).await;

    (StatusCode::OK, Json(response)).into_response()
}
