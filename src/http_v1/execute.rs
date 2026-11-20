// Rust guideline compliant 2026-11-18
//! Capability tool execution handlers, cancellation, rate limiting, and HITL approval workflow.

use anyhow::Context;
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
        helpers::{next_trace_id, redact_value, resolve_client_ip, resolve_idempotency_key},
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
    let prof_ctx = prof_ext
        .as_ref()
        .map(|extension| extension.0.clone())
        .unwrap_or_default();
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

    let client_ip = resolve_client_ip(&headers);

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

    if let Some(key) = idempotency_key.as_deref() {
        if let Some(task) = state.task_registry.find_by_idempotency_key(key).await {
            if task.emission.is_some() {
                if !crate::http_v1::helpers::can_access_emission_task(
                    &state,
                    &task,
                    req_ext.0.as_ref(),
                    prof_ext.as_ref().map(|extension| &extension.0),
                )
                .await
                {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(error_envelope(
                            trace_id,
                            Some(request_id),
                            Some(req_context),
                            retry_base("not_started"),
                            "POLICY_DENIED",
                            "Held task is not accessible to this caller",
                            false,
                        )),
                    )
                        .into_response();
                }
                let status = if matches!(
                    task.status,
                    crate::tasks::TaskStatus::Completed
                        | crate::tasks::TaskStatus::Failed
                        | crate::tasks::TaskStatus::Cancelled
                ) {
                    StatusCode::OK
                } else {
                    StatusCode::ACCEPTED
                };
                return (status, Json(json!({"ok": true, "resultType": "task", "task": crate::tasks::TaskResponse::from(&task), "request_id": request_id, "trace_id": trace_id}))).into_response();
            }
        }
    }

    let idempotency_claim = if let Some(ref key) = idempotency_key {
        loop {
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
                        client_ip: client_ip.clone(),
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
                crate::idempotency::DeduplicateResult::New(claim) => break Some(claim),
            }
        }
    } else {
        None
    };

    let cancel_token: tokio_util::sync::CancellationToken =
        state.operation_registry.register(&request_id).await;

    if !payload.args.is_object() {
        state.operation_registry.unregister(&request_id).await;
        if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
            state.idempotency_store.remove(key, claim).await;
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

    let (mut requires_approval, approval_timeout_secs, webhook_cfg, redact_keys, effective_policy) = {
        let base_pol = state.policy.read().await;
        let pol = req_ext
            .0
            .as_ref()
            .map(|ctx| ctx.effective_policy.clone())
            .unwrap_or_else(|| base_pol.clone())
            .merge_with_profile(prof_ctx.profile_policy.as_ref());

        if !pol.allows(&payload.capability_id) {
            state.operation_registry.unregister(&request_id).await;
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
            }
            state.audit_handle.send(crate::audit::RawAuditEvent {
                event_type: crate::audit::AuditEventType::PolicyViolation,
                trace_id: trace_id.clone(),
                request_id: Some(request_id.clone()),
                actor_id: req_context.actor_id.clone(),
                work_item_id: req_context.work_item_id.clone(),
                client_ip: client_ip.clone(),
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
            pol,
        )
    };

    let (server_id, tool_name, reversibility) = {
        let caps_guard = state.capabilities.read().await;
        let Some(meta) = caps_guard.get(&payload.capability_id) else {
            state.operation_registry.unregister(&request_id).await;
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
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
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
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

        (
            meta.server.clone(),
            meta.tool.clone(),
            meta.reversibility.clone(),
        )
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
    let mut compensable_reserved = false;
    if let Some(contract) = reversibility
        .as_ref()
        .filter(|contract| contract.class == crate::config::ReversibilityClass::Compensable)
    {
        if state.undo_ledger.is_none() {
            state.operation_registry.unregister(&request_id).await;
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
            }
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    "UNDO_STORAGE_UNAVAILABLE",
                    "Persistent undo storage is unavailable",
                    false,
                )),
            )
                .into_response();
        }
        if !requires_approval {
            match crate::engine::undo::reserve_compensable(
                &state,
                contract,
                &payload.capability_id,
                &tenant_id,
                prof_ctx.profile_id.as_deref(),
                idempotency_key.as_deref(),
                false,
            )
            .await
            {
                Ok(true) => compensable_reserved = true,
                Ok(false) => requires_approval = true,
                Err(error) => {
                    state.operation_registry.unregister(&request_id).await;
                    if let (Some(key), Some(claim)) =
                        (idempotency_key.as_deref(), idempotency_claim)
                    {
                        state.idempotency_store.remove(key, claim).await;
                    }
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
    }

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
                                    if let (Some(key), Some(claim)) =
                                        (idempotency_key_clone.as_deref(), idempotency_claim)
                                    {
                                        state_clone.idempotency_store.remove(key, claim).await;
                                    }
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
                            if let (Some(key), Some(claim)) =
                                (idempotency_key_clone.as_deref(), idempotency_claim)
                            {
                                state_clone.idempotency_store.remove(key, claim).await;
                            }
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
                                if let (Some(key), Some(claim)) =
                                    (idempotency_key_clone.as_deref(), idempotency_claim)
                                {
                                    state_clone.idempotency_store.remove(key, claim).await;
                                }
                                return;
                            }
                        }
                    }
                }

                if let Some(contract) = reversibility_clone.as_ref().filter(|contract| {
                    contract.class == crate::config::ReversibilityClass::Delayable
                }) {
                    let global_window = state_clone.policy.read().await.hold_window_secs;
                    let profile_window = profile_clone
                        .profile_policy
                        .as_ref()
                        .and_then(|profile_policy| profile_policy.hold_window_secs);
                    let hold_secs = crate::emissions::effective_hold_window_secs(
                        contract,
                        profile_window,
                        global_window,
                    )
                    .unwrap_or(30);
                    let emission = match crate::emissions::release_deadline_ms(hold_secs) {
                        Ok(release_at_ms) => crate::tasks::EmissionRecord {
                            idempotency_claim,
                            phase: crate::tasks::EmissionPhase::PendingEmission,
                            release_at_ms,
                            cancellable: true,
                            tool_name: tool_name_clone.clone(),
                            input_responses: input_responses_clone.clone(),
                            request_state: request_state_clone.clone(),
                            profile_id: profile_clone.profile_id.clone(),
                            tenant_id: tenant_id_clone.clone(),
                            role: caller_role_clone.clone(),
                            authorization_expires_at_epoch_secs:
                                caller_grant_expires_at_epoch_secs_clone,
                            approval_completed: true,
                            batch_id: None,
                            step_id: None,
                        },
                        Err(error) => {
                            let _ = state_clone
                                .task_registry
                                .fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "INVALID_HOLD_WINDOW"}),
                                    Some(error.to_string()),
                                )
                                .await;
                            return;
                        }
                    };
                    match state_clone
                        .hold_coordinator
                        .queue_existing(&state_clone, &task_id_clone, worker_args.clone(), emission)
                        .await
                    {
                        Ok(Some(_)) | Ok(None) => return,
                        Err(error) => {
                            let _ = state_clone
                                .task_registry
                                .fail_task(
                                    &task_id_clone,
                                    serde_json::json!({"code": "EMISSION_PERSISTENCE_FAILED"}),
                                    Some(error.to_string()),
                                )
                                .await;
                            if let (Some(key), Some(claim)) =
                                (idempotency_key_clone.as_deref(), idempotency_claim)
                            {
                                state_clone.idempotency_store.remove(key, claim).await;
                            }
                            return;
                        }
                    }
                }

                let (snapshot, snapshot_hash) = if let Some(contract) = reversibility_clone.as_ref()
                {
                    if matches!(
                        contract.class,
                        crate::config::ReversibilityClass::Reversible
                            | crate::config::ReversibilityClass::Compensable
                    ) && state_clone.undo_ledger.is_none()
                    {
                        let _ = state_clone
                            .task_registry
                            .fail_task(
                                &task_id_clone,
                                serde_json::json!({"code": "UNDO_STORAGE_UNAVAILABLE"}),
                                Some("Persistent undo storage is unavailable".to_string()),
                            )
                            .await;
                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            state_clone.idempotency_store.remove(key, claim).await;
                        }
                        return;
                    }
                    if let Some(pre_snapshot) = &contract.pre_snapshot {
                        match crate::engine::undo::capture_pre_snapshot(
                            &state_clone,
                            pre_snapshot,
                            &worker_args,
                            &profile_clone,
                            &policy_clone,
                        )
                        .await
                        {
                            Ok(value) => value,
                            Err(error) => {
                                let _ = state_clone
                                    .task_registry
                                    .fail_task(
                                        &task_id_clone,
                                        serde_json::json!({"code": "PRE_SNAPSHOT_FAILED"}),
                                        Some(error.to_string()),
                                    )
                                    .await;
                                if let (Some(key), Some(claim)) =
                                    (idempotency_key_clone.as_deref(), idempotency_claim)
                                {
                                    state_clone.idempotency_store.remove(key, claim).await;
                                }
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
                            if let (Some(key), Some(claim)) =
                                (idempotency_key_clone.as_deref(), idempotency_claim)
                            {
                                state_clone.idempotency_store.remove(key, claim).await;
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
                    if let (Some(key), Some(claim)) =
                        (idempotency_key_clone.as_deref(), idempotency_claim)
                    {
                        state_clone.idempotency_store.remove(key, claim).await;
                    }
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
                        let distilled_data =
                            crate::context_filter::distill_value(data, &distill_opts);
                        let _ = state_clone
                            .task_registry
                            .complete_task(&task_id_clone, distilled_data.clone())
                            .await;

                        let response_json = json!({
                            "ok": true,
                            "request_id": req_id_clone,
                            "context": req_ctx_clone,
                            "trace_id": trace_id_clone,
                            "data": distilled_data,
                            "error": null,
                            "retry": retry_base("completed"),
                        });

                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            state_clone
                                .idempotency_store
                                .complete(key, claim, response_json)
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
                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            state_clone.idempotency_store.remove(key, claim).await;
                        }
                    }
                    Ok(Err(UpstreamCallError::Upstream(err))) => {
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
                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            state_clone.idempotency_store.remove(key, claim).await;
                        }
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
                        if let (Some(key), Some(claim)) =
                            (idempotency_key_clone.as_deref(), idempotency_claim)
                        {
                            state_clone.idempotency_store.remove(key, claim).await;
                        }
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
                state.operation_registry.unregister(&request_id).await;
                if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                    state.idempotency_store.remove(key, claim).await;
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
                if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                    state.idempotency_store.remove(key, claim).await;
                }
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
                state.operation_registry.unregister(&request_id).await;
                if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                    state.idempotency_store.remove(key, claim).await;
                }
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
                state.operation_registry.unregister(&request_id).await;
                if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                    state.idempotency_store.remove(key, claim).await;
                }
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
        .filter(|contract| contract.class == crate::config::ReversibilityClass::Delayable)
    {
        let global_window = state.policy.read().await.hold_window_secs;
        let profile_window = prof_ctx
            .profile_policy
            .as_ref()
            .and_then(|profile_policy| profile_policy.hold_window_secs);
        let hold_secs =
            crate::emissions::effective_hold_window_secs(contract, profile_window, global_window)
                .unwrap_or(30);
        let hold_ms = match hold_secs.checked_mul(1_000) {
            Some(value) => value,
            None => {
                state.operation_registry.unregister(&request_id).await;
                return (
                    StatusCode::BAD_REQUEST,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("not_started"),
                        "INVALID_HOLD_WINDOW",
                        "Hold window overflows milliseconds",
                        false,
                    )),
                )
                    .into_response();
            }
        };
        let release_at_ms = match crate::emissions::release_deadline_ms(hold_secs) {
            Ok(deadline) => deadline,
            Err(error) => {
                state.operation_registry.unregister(&request_id).await;
                return (
                    StatusCode::BAD_REQUEST,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("not_started"),
                        "INVALID_HOLD_WINDOW",
                        error.to_string(),
                        false,
                    )),
                )
                    .into_response();
            }
        };
        let emission = crate::tasks::EmissionRecord {
            idempotency_claim,
            phase: crate::tasks::EmissionPhase::PendingEmission,
            release_at_ms,
            cancellable: true,
            tool_name: tool_name.clone(),
            input_responses: payload.input_responses.clone(),
            request_state: payload.request_state.clone(),
            profile_id: prof_ctx.profile_id.clone(),
            tenant_id: tenant_id.clone(),
            role: caller_role.clone(),
            authorization_expires_at_epoch_secs: caller_grant_expires_at_epoch_secs,
            approval_completed: requires_approval,
            batch_id: None,
            step_id: None,
        };
        let existing_task = state
            .task_registry
            .list_tasks()
            .await
            .into_iter()
            .find(|task| {
                task.request_id.as_deref() == Some(request_id.as_str()) && task.emission.is_none()
            });
        let accepted = if let Some(existing) = existing_task {
            state
                .hold_coordinator
                .queue_existing(&state, &existing.task_id, effective_args.clone(), emission)
                .await
                .context("failed to persist approved emission")
        } else {
            state
                .hold_coordinator
                .accept(
                    &state,
                    crate::tasks::CreateTaskParams {
                        capability_id: payload.capability_id.clone(),
                        server_id: server_id.clone(),
                        args: effective_args.clone(),
                        request_id: Some(request_id.clone()),
                        context: Some(req_context.clone()),
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
                .map(Some)
        };
        let task = match accepted {
            Ok(Some(task)) => task,
            Ok(None) => {
                state.operation_registry.unregister(&request_id).await;
                return (
                    StatusCode::CONFLICT,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("not_started"),
                        "EMISSION_TASK_STATE_INVALID",
                        "Could not attach the approved emission to its task",
                        false,
                    )),
                )
                    .into_response();
            }
            Err(error) => {
                state.operation_registry.unregister(&request_id).await;
                if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                    state.idempotency_store.remove(key, claim).await;
                }
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(error_envelope(
                        trace_id,
                        Some(request_id),
                        Some(req_context),
                        retry_base("not_started"),
                        "EMISSION_PERSISTENCE_FAILED",
                        error.to_string(),
                        true,
                    )),
                )
                    .into_response();
            }
        };
        state.operation_registry.unregister(&request_id).await;
        let prefer_async = payload.async_task
            || headers
                .get("prefer")
                .and_then(|header| header.to_str().ok())
                .is_some_and(|value| value.to_ascii_lowercase().contains("respond-async"));
        if !prefer_async && hold_ms < state.tool_timeout_ms {
            let wait =
                std::time::Duration::from_millis(hold_ms.saturating_add(state.tool_timeout_ms));
            if let Some(finished) = state
                .hold_coordinator
                .wait_for_terminal(&state, &task.task_id, wait)
                .await
            {
                match finished.status {
                    crate::tasks::TaskStatus::Completed => {
                        let data = finished.result.unwrap_or(Value::Null);
                        return (StatusCode::OK, Json(json!({"ok": true, "request_id": request_id, "context": req_context, "trace_id": trace_id, "data": data, "error": null, "retry": retry_base("completed")}))).into_response();
                    }
                    crate::tasks::TaskStatus::Failed | crate::tasks::TaskStatus::Cancelled => {
                        let code = finished
                            .error
                            .as_ref()
                            .and_then(|error| error.get("code"))
                            .and_then(Value::as_str)
                            .unwrap_or("EMISSION_FAILED");
                        let message = finished
                            .status_message
                            .unwrap_or_else(|| "Held emission did not complete".to_string());
                        return (
                            StatusCode::BAD_GATEWAY,
                            Json(error_envelope(
                                trace_id,
                                Some(request_id),
                                Some(req_context),
                                retry_base("completed"),
                                code,
                                message,
                                false,
                            )),
                        )
                            .into_response();
                    }
                    _ => {}
                }
            }
        }
        return (
            StatusCode::ACCEPTED,
            Json(json!({
                "ok": true,
                "resultType": "task",
                "task": crate::tasks::TaskResponse::from(&task),
                "request_id": request_id,
                "trace_id": trace_id,
            })),
        )
            .into_response();
    }

    let (snapshot, snapshot_hash) = if let Some(contract) = reversibility.as_ref() {
        if matches!(
            contract.class,
            crate::config::ReversibilityClass::Reversible
                | crate::config::ReversibilityClass::Compensable
        ) && state.undo_ledger.is_none()
        {
            state.operation_registry.unregister(&request_id).await;
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
            }
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    retry_base("not_started"),
                    "UNDO_STORAGE_UNAVAILABLE",
                    "Persistent undo storage is unavailable",
                    false,
                )),
            )
                .into_response();
        }
        if let Some(pre_snapshot) = &contract.pre_snapshot {
            match crate::engine::undo::capture_pre_snapshot(
                &state,
                pre_snapshot,
                &effective_args,
                &prof_ctx,
                &effective_policy,
            )
            .await
            {
                Ok(value) => value,
                Err(error) => {
                    state.operation_registry.unregister(&request_id).await;
                    if let (Some(key), Some(claim)) =
                        (idempotency_key.as_deref(), idempotency_claim)
                    {
                        state.idempotency_store.remove(key, claim).await;
                    }
                    return (
                        StatusCode::BAD_GATEWAY,
                        Json(error_envelope(
                            trace_id,
                            Some(request_id),
                            Some(req_context),
                            retry_base("not_started"),
                            "PRE_SNAPSHOT_FAILED",
                            error.to_string(),
                            true,
                        )),
                    )
                        .into_response();
                }
            }
        } else {
            (Value::Null, None)
        }
    } else {
        (Value::Null, None)
    };

    let tx = {
        let servers_guard = state.servers.read().await;
        let Some(tx) = servers_guard.get(&server_id).cloned() else {
            state.operation_registry.unregister(&request_id).await;
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
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
        if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
            state.idempotency_store.remove(key, claim).await;
        }
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
        state.operation_registry.unregister(&request_id).await;
        if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
            state.idempotency_store.remove(key, claim).await;
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
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
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
                client_ip: client_ip.clone(),
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

            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state
                    .idempotency_store
                    .complete(key, claim, response_json.clone())
                    .await;
            }

            (StatusCode::OK, Json(response_json)).into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => {
            state.circuit_breakers.record_failure(&server_id).await;
            let elapsed_us = start_time.elapsed().as_micros() as u64;
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
            }
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
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
            }
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
            if let (Some(key), Some(claim)) = (idempotency_key.as_deref(), idempotency_claim) {
                state.idempotency_store.remove(key, claim).await;
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
