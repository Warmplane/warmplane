// Rust guideline compliant 2026-08-15

//! Capability tool execution handlers, cancellation, rate limiting, and HITL approval workflow.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tokio::sync::oneshot;
use tracing::info;

use crate::{
    daemon::{AppState, ServerMsg, UpstreamCallError},
    http_v1::{
        helpers::{next_trace_id, redact_value, resolve_idempotency_key},
        types::{error_envelope, CallCapabilityRequest},
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
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let start_time = std::time::Instant::now();
    state.total_tool_calls.fetch_add(1, Ordering::Relaxed);
    let trace_id = next_trace_id();
    let request_id =
        crate::context::resolve_request_id(payload.request_id.clone(), &headers, trace_id.clone());
    let mut req_context =
        crate::context::resolve_request_context(payload.context.clone(), &headers);

    // Bind authenticated tenant/actor if available
    if let Some(ref tenant_ctx) = req_ext.0 {
        if let Some(ref actor) = tenant_ctx.actor_id {
            req_context.actor_id = Some(actor.clone());
        }
        if let Some(ref grant) = tenant_ctx.grant_id {
            req_context.grant_id = Some(grant.clone());
        }
    }

    let explicit_key = resolve_idempotency_key(payload.idempotency_key.clone(), &headers);
    let idempotency_key = explicit_key.or_else(|| {
        if payload.args.is_object() {
            Some(crate::idempotency::derive_idempotency_key(
                &payload.capability_id,
                &payload.args,
                req_context.actor_id.as_deref(),
                payload.request_id.as_deref(),
            ))
        } else {
            None
        }
    });

    let retry_base = if idempotency_key.is_some() {
        crate::idempotency::RetryMetadata::idempotent
    } else {
        crate::idempotency::RetryMetadata::unsafe_op
    };

    if let Some(ref key) = idempotency_key {
        match state
            .idempotency_store
            .check_or_start_with_meta(
                key,
                Some(payload.capability_id.clone()),
                Some(trace_id.clone()),
            )
            .await
        {
            crate::idempotency::DeduplicateResult::Completed(cached) => {
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ToolExecution,
                    trace_id: trace_id.clone(),
                    request_id: Some(request_id.clone()),
                    actor_id: req_context.actor_id.clone(),
                    work_item_id: req_context.work_item_id.clone(),
                    client_ip: headers
                        .get("x-forwarded-for")
                        .and_then(|h| h.to_str().ok())
                        .map(|s| s.to_string()),
                    server_id: None,
                    capability_id: Some(payload.capability_id.clone()),
                    resource_uri: None,
                    sanitized_args: Some(payload.args.clone()),
                    sanitized_response: Some(cached.clone()),
                    execution_latency_us: Some(start_time.elapsed().as_micros() as u64),
                    status: crate::audit::AuditEventStatus::Success,
                    error_code: None,
                    error_message: None,
                    operator_id: None,
                    approval_ticket_id: None,
                    idempotency_key: Some(key.clone()),
                    is_replay: Some(true),
                });

                let mut resp_headers = HeaderMap::new();
                resp_headers.insert("x-warmplane-deduplicated", "true".parse().unwrap());
                return (StatusCode::OK, resp_headers, Json(cached)).into_response();
            }
            crate::idempotency::DeduplicateResult::InProgress(mut rx) => {
                if let Ok(cached) = rx.recv().await {
                    let mut resp_headers = HeaderMap::new();
                    resp_headers.insert("x-warmplane-deduplicated", "true".parse().unwrap());
                    return (StatusCode::OK, resp_headers, Json(cached)).into_response();
                }
            }
            crate::idempotency::DeduplicateResult::New => {}
        }
    }

    let cancel_token: tokio_util::sync::CancellationToken =
        state.operation_registry.register(&request_id).await;

    if !payload.args.is_object() {
        state.operation_registry.unregister(&request_id).await;
        if let Some(ref key) = idempotency_key {
            state.idempotency_store.remove(key).await;
        }
        return (
            StatusCode::BAD_REQUEST,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                retry_base("not_started"),
                "INVALID_ARGS",
                "'args' must be a JSON object",
                false,
            )),
        )
            .into_response();
    }

    let (requires_approval, approval_timeout_secs, webhook_cfg, redact_keys) = {
        let base_pol = state.policy.read().await;
        let pol = req_ext
            .0
            .as_ref()
            .map(|ctx| ctx.effective_policy.clone())
            .unwrap_or_else(|| base_pol.clone());

        if !pol.allows(&payload.capability_id) {
            state.operation_registry.unregister(&request_id).await;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
            state.audit_handle.send(crate::audit::RawAuditEvent {
                event_type: crate::audit::AuditEventType::PolicyViolation,
                trace_id: trace_id.clone(),
                request_id: Some(request_id.clone()),
                actor_id: req_context.actor_id.clone(),
                work_item_id: req_context.work_item_id.clone(),
                client_ip: headers
                    .get("x-forwarded-for")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string()),
                server_id: None,
                capability_id: Some(payload.capability_id.clone()),
                resource_uri: None,
                sanitized_args: Some(redact_value(payload.args.clone(), &pol.redact_keys)),
                sanitized_response: None,
                execution_latency_us: Some(start_time.elapsed().as_micros() as u64),
                status: crate::audit::AuditEventStatus::Denied,
                error_code: Some("POLICY_DENIED".to_string()),
                error_message: Some(format!(
                    "Capability '{}' blocked by policy",
                    payload.capability_id
                )),
                operator_id: None,
                approval_ticket_id: None,
                idempotency_key: idempotency_key.clone(),
                is_replay: Some(false),
            });
            return (
                StatusCode::FORBIDDEN,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    "POLICY_DENIED",
                    format!("Capability '{}' blocked by policy", payload.capability_id),
                    false,
                )),
            )
                .into_response();
        }
        (
            pol.requires_approval(&payload.capability_id),
            pol.approval_timeout_secs,
            pol.webhook.clone(),
            pol.redact_keys.clone(),
        )
    };

    let (server_id, tool_name) = {
        let caps_guard = state.capabilities.read().await;
        let Some(meta) = caps_guard.get(&payload.capability_id) else {
            state.operation_registry.unregister(&request_id).await;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
            return (
                StatusCode::NOT_FOUND,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    "TOOL_NOT_FOUND",
                    format!("Capability '{}' not found", payload.capability_id),
                    false,
                )),
            )
                .into_response();
        };

        if !prof_ctx.is_server_allowed(&meta.server) {
            state.operation_registry.unregister(&request_id).await;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
            return (
                StatusCode::FORBIDDEN,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    "TOOL_NOT_IN_PROFILE",
                    format!(
                        "Capability '{}' belongs to server '{}' which is not in active profile",
                        payload.capability_id, meta.server
                    ),
                    false,
                )),
            )
                .into_response();
        }

        (meta.server.clone(), meta.tool.clone())
    };

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
            .create_task(crate::tasks::CreateTaskParams {
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
            })
            .await;

        let (approval_id, rx) = state
            .approval_registry
            .create_approval(crate::approvals::CreateApprovalRequest {
                capability_id: payload.capability_id.clone(),
                server_id: server_id.clone(),
                args: payload.args.clone(),
                sanitized_args: sanitized.clone(),
                request_id: Some(request_id.clone()),
                context: Some(req_context.clone()),
                timeout_secs: approval_timeout_secs,
                webhook: webhook_cfg.as_ref(),
            })
            .await;

        state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolInterceptedHitl,
            trace_id: trace_id.clone(),
            request_id: Some(request_id.clone()),
            actor_id: req_context.actor_id.clone(),
            work_item_id: req_context.work_item_id.clone(),
            client_ip: headers
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
            server_id: Some(server_id.clone()),
            capability_id: Some(payload.capability_id.clone()),
            resource_uri: None,
            sanitized_args: Some(sanitized.clone()),
            sanitized_response: None,
            execution_latency_us: Some(start_time.elapsed().as_micros() as u64),
            status: crate::audit::AuditEventStatus::Intercepted,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: Some(approval_id.clone()),
            idempotency_key: idempotency_key.clone(),
            is_replay: Some(false),
        });

        let prefer_async = payload.async_task
            || headers
                .get("prefer")
                .and_then(|h| h.to_str().ok())
                .map(|v| v.to_lowercase().contains("respond-async"))
                .unwrap_or(false);

        if prefer_async {
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
                state.operation_registry.unregister(&request_id).await;
                if let Some(ref key) = idempotency_key {
                    state.idempotency_store.remove(key).await;
                }
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
                state.operation_registry.unregister(&request_id).await;
                if let Some(ref key) = idempotency_key {
                    state.idempotency_store.remove(key).await;
                }
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ApprovalRejected,
                    trace_id: trace_id.clone(),
                    request_id: Some(request_id.clone()),
                    actor_id: req_context.actor_id.clone(),
                    work_item_id: req_context.work_item_id.clone(),
                    client_ip: headers
                        .get("x-forwarded-for")
                        .and_then(|h| h.to_str().ok())
                        .map(|s| s.to_string()),
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
                state.operation_registry.unregister(&request_id).await;
                if let Some(ref key) = idempotency_key {
                    state.idempotency_store.remove(key).await;
                }
                state.audit_handle.send(crate::audit::RawAuditEvent {
                    event_type: crate::audit::AuditEventType::ApprovalExpired,
                    trace_id: trace_id.clone(),
                    request_id: Some(request_id.clone()),
                    actor_id: req_context.actor_id.clone(),
                    work_item_id: req_context.work_item_id.clone(),
                    client_ip: headers
                        .get("x-forwarded-for")
                        .and_then(|h| h.to_str().ok())
                        .map(|s| s.to_string()),
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

    let tx = {
        let servers_guard = state.servers.read().await;
        let Some(tx) = servers_guard.get(&server_id).cloned() else {
            state.operation_registry.unregister(&request_id).await;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
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
        state.operation_registry.unregister(&request_id).await;
        if let Some(ref key) = idempotency_key {
            state.idempotency_store.remove(key).await;
        }
        state.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolExecution,
            trace_id: trace_id.clone(),
            request_id: Some(request_id.clone()),
            actor_id: req_context.actor_id.clone(),
            work_item_id: req_context.work_item_id.clone(),
            client_ip: headers
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string()),
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
            params: effective_args,
            input_responses: payload.input_responses,
            request_state: payload.request_state,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        state.circuit_breakers.record_failure(&server_id).await;
        state.operation_registry.unregister(&request_id).await;
        if let Some(ref key) = idempotency_key {
            state.idempotency_store.remove(key).await;
        }
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
            state.operation_registry.unregister(&request_id).await;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
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

    match result {
        Ok(Ok(data)) => {
            state.circuit_breakers.record_success(&server_id).await;
            let redacted_output = redact_value(data.clone(), &redact_keys);
            info!(
                trace_id = %trace_id,
                request_id = %request_id,
                capability_id = %payload.capability_id,
                data = %redacted_output,
                "tool call success"
            );
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            state
                .total_tool_duration_us
                .fetch_add(elapsed_us, Ordering::Relaxed);

            state.audit_handle.send(crate::audit::RawAuditEvent {
                event_type: crate::audit::AuditEventType::ToolExecution,
                trace_id: trace_id.clone(),
                request_id: Some(request_id.clone()),
                actor_id: req_context.actor_id.clone(),
                work_item_id: req_context.work_item_id.clone(),
                client_ip: headers
                    .get("x-forwarded-for")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string()),
                server_id: Some(server_id.clone()),
                capability_id: Some(payload.capability_id.clone()),
                resource_uri: None,
                sanitized_args: Some(redacted_input),
                sanitized_response: Some(redacted_output),
                execution_latency_us: Some(elapsed_us),
                status: crate::audit::AuditEventStatus::Success,
                error_code: None,
                error_message: None,
                operator_id: None,
                approval_ticket_id: None,
                idempotency_key: idempotency_key.clone(),
                is_replay: Some(false),
            });

            let distill_opts =
                crate::context_filter::DistillationOptions::from_args(Some(&payload.args));
            let distilled_data = crate::context_filter::distill_value(data, &distill_opts);

            let response_json = json!({
                "ok": true,
                "request_id": request_id,
                "context": req_context,
                "trace_id": trace_id,
                "data": distilled_data,
                "error": null,
                "retry": retry_base("completed"),
            });

            if let Some(ref key) = idempotency_key {
                state
                    .idempotency_store
                    .complete(key, response_json.clone())
                    .await;
            }

            (StatusCode::OK, Json(response_json)).into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => {
            state.circuit_breakers.record_failure(&server_id).await;
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
            state.audit_handle.send(crate::audit::RawAuditEvent {
                event_type: crate::audit::AuditEventType::ToolExecution,
                trace_id: trace_id.clone(),
                request_id: Some(request_id.clone()),
                actor_id: req_context.actor_id.clone(),
                work_item_id: req_context.work_item_id.clone(),
                client_ip: headers
                    .get("x-forwarded-for")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string()),
                server_id: Some(server_id.clone()),
                capability_id: Some(payload.capability_id.clone()),
                resource_uri: None,
                sanitized_args: Some(redacted_input),
                sanitized_response: None,
                execution_latency_us: Some(elapsed_us),
                status: crate::audit::AuditEventStatus::Failed,
                error_code: Some("UPSTREAM_TIMEOUT".to_string()),
                error_message: Some(format!(
                    "Tool call timed out after {}ms",
                    state.tool_timeout_ms
                )),
                operator_id: None,
                approval_ticket_id: None,
                idempotency_key: idempotency_key.clone(),
                is_replay: Some(false),
            });
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
            state.circuit_breakers.record_failure(&server_id).await;
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
            state.audit_handle.send(crate::audit::RawAuditEvent {
                event_type: crate::audit::AuditEventType::ToolExecution,
                trace_id: trace_id.clone(),
                request_id: Some(request_id.clone()),
                actor_id: req_context.actor_id.clone(),
                work_item_id: req_context.work_item_id.clone(),
                client_ip: headers
                    .get("x-forwarded-for")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string()),
                server_id: Some(server_id.clone()),
                capability_id: Some(payload.capability_id.clone()),
                resource_uri: None,
                sanitized_args: Some(redacted_input),
                sanitized_response: None,
                execution_latency_us: Some(elapsed_us),
                status: crate::audit::AuditEventStatus::Failed,
                error_code: Some("UPSTREAM_ERROR".to_string()),
                error_message: Some(err.clone()),
                operator_id: None,
                approval_ticket_id: None,
                idempotency_key: idempotency_key.clone(),
                is_replay: Some(false),
            });
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
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
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

/// Handles HTTP POST `/v1/operations/:id/cancel` cancelling an in-flight operation.
pub async fn handle_cancel_operation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let cancelled = state.operation_registry.cancel(&id).await;
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
        .unwrap_or_else(|| base_pol.clone());

    let response = crate::batch_executor::execute_batch(
        &state,
        payload.steps,
        trace_id,
        Some(request_id),
        Some(req_context),
        &pol,
        &prof_ctx,
    )
    .await;

    (StatusCode::OK, Json(response)).into_response()
}
