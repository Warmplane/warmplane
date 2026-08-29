// Rust guideline compliant 2026-08-27

//! Catalog discovery and exploration endpoints for tools, resources, prompts, completions, and change feeds.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::Stream;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tokio::sync::oneshot;
use tracing::info;

use crate::{
    daemon::{AppState, CapabilityMeta, ServerMsg, UpstreamCallError},
    http_v1::{
        helpers::{
            check_if_none_match, make_etag_header, next_trace_id, redact_value,
            resolve_idempotency_key, TRACE_COUNTER,
        },
        types::{
            error_envelope, CatalogEventsQuery, CatalogEventsResponse, CompletionRequest,
            GetPromptRequest, ReadResourceRequest, RespondSamplingRequest, SamplingListQuery,
            SamplingRequest, SearchCapabilitiesRequest,
        },
    },
};

/// Handles HTTP GET `/v1/catalog/events` change feed endpoint.
pub async fn handle_catalog_events(
    State(state): State<AppState>,
    Query(query): Query<CatalogEventsQuery>,
) -> impl IntoResponse {
    let (events, next_cursor) = state.event_store.get_events_after(query.after.as_deref());
    let catalog_ver = state.catalog_version.read().await.clone();
    (
        make_etag_header(&catalog_ver),
        Json(CatalogEventsResponse {
            catalog_version: catalog_ver,
            cursor: next_cursor,
            events,
        }),
    )
}

/// Handles HTTP POST `/v1/capabilities/search` hybrid semantic and lexical capability search.
pub async fn handle_search_capabilities(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Json(payload): Json<SearchCapabilitiesRequest>,
) -> impl IntoResponse {
    let query_str = payload.query.as_deref().unwrap_or("");
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();

    // Intersect server_ids with profile allowed_servers if profile is active
    let effective_server_ids = match &prof_ctx.allowed_servers {
        Some(allowed) => {
            if payload.server_ids.is_empty() {
                allowed.iter().cloned().collect()
            } else {
                payload
                    .server_ids
                    .into_iter()
                    .filter(|s| allowed.contains(s))
                    .collect()
            }
        }
        None => payload.server_ids,
    };

    let filter = crate::search::SearchFilter::builder()
        .server_ids(effective_server_ids)
        .tags(payload.tags)
        .modes(payload.modes)
        .build();

    let caps = state.capabilities.read().await;
    let base_pol = state.policy.read().await;
    let pol = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.effective_policy.clone())
        .unwrap_or_else(|| base_pol.clone())
        .merge_with_profile(prof_ctx.profile_policy.as_ref());
    let base_ver = state.catalog_version.read().await.clone();
    let catalog_ver =
        crate::http_v1::helpers::get_profile_scoped_catalog_version(&base_ver, &prof_ctx);
    let limit = payload.limit.clamp(1, 100);

    let results = state
        .search_engine
        .search(query_str, limit, &filter, &caps, &pol);

    (
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "capabilities": results
        })),
    )
}

/// Handles HTTP POST `/v1/completion/complete` autocompletion endpoint.
pub async fn handle_completion(
    State(state): State<AppState>,
    Json(payload): Json<CompletionRequest>,
) -> impl IntoResponse {
    let trace_id = format!("trc_{}", TRACE_COUNTER.fetch_add(1, Ordering::Relaxed));
    let catalog_ver = state.catalog_version.read().await.clone();

    let found = match payload.ref_type.as_str() {
        "prompt" => state.prompts.read().await.contains_key(&payload.ref_name),
        "resource" => state.resources.read().await.contains_key(&payload.ref_name),
        _ => false,
    };

    if !found {
        return (
            StatusCode::NOT_FOUND,
            make_etag_header(&catalog_ver),
            Json(json!({
                "ok": false,
                "trace_id": trace_id,
                "data": null,
                "error": {
                    "code": "NOT_FOUND",
                    "message": format!("Reference '{}' of type '{}' not found", payload.ref_name, payload.ref_type)
                }
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "ok": true,
            "trace_id": trace_id,
            "data": {
                "ref_type": payload.ref_type,
                "ref_name": payload.ref_name,
                "argument_name": payload.argument_name,
                "argument_value": payload.argument_value,
                "values": [],
                "total": 0,
                "has_more": false
            }
        })),
    )
        .into_response()
}

/// Handles Server-Sent Events (SSE) `GET /v1/resources/updates` real-time resource notification stream.
pub async fn handle_resource_updates(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = std::result::Result<Event, std::convert::Infallible>>> {
    let rx = state.resource_update_tx.subscribe();

    let shutdown_token = state.shutdown_token.clone();
    let stream =
        futures::stream::unfold((rx, shutdown_token), |(mut receiver, token)| async move {
            loop {
                tokio::select! {
                    _ = token.cancelled() => return None,
                    msg = receiver.recv() => match msg {
                        Ok(evt) => {
                            if let Ok(data) = serde_json::to_string(&evt) {
                                let event = Event::default().event("resource_updated").data(data);
                                return Some((Ok(event), (receiver, token)));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    }
                }
            }
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Handles HTTP POST `/v1/sampling/create_message` sampling delegation endpoint.
pub async fn handle_sampling_create_message(
    State(state): State<AppState>,
    Json(payload): Json<SamplingRequest>,
) -> impl IntoResponse {
    let trace_id = format!("trc_{}", TRACE_COUNTER.fetch_add(1, Ordering::Relaxed));
    let catalog_ver = state.catalog_version.read().await.clone();

    if !state.servers.read().await.contains_key(&payload.server_id) {
        return (
            StatusCode::NOT_FOUND,
            make_etag_header(&catalog_ver),
            Json(json!({
                "ok": false,
                "trace_id": trace_id,
                "data": null,
                "error": {
                    "code": "SERVER_UNREACHABLE",
                    "message": format!("Server '{}' not connected", payload.server_id)
                }
            })),
        )
            .into_response();
    }

    let is_async = payload.async_mode.unwrap_or(false);
    let timeout_secs = (state.tool_timeout_ms / 1000).max(10);
    let server_id = payload.server_id.clone();

    let params = crate::sampling::CreateMessageParams {
        server_id: payload.server_id.clone(),
        messages: payload.messages,
        model_preferences: payload.model_preferences,
        system_prompt: payload.system_prompt,
        include_context: payload.include_context,
        max_tokens: payload.max_tokens,
        stop_sequences: payload.stop_sequences,
        metadata: payload.metadata,
    };

    let (ticket_id, rx) = state
        .sampling_registry
        .create_request(server_id, params, timeout_secs)
        .await;

    if is_async {
        return (
            StatusCode::ACCEPTED,
            make_etag_header(&catalog_ver),
            Json(json!({
                "ok": true,
                "trace_id": trace_id,
                "ticket_id": ticket_id,
                "status": "pending",
                "message": "Sampling request suspended; awaiting client completion callback."
            })),
        )
            .into_response();
    }

    // Synchronous long-polling: await client response
    match rx.await {
        Ok(Ok(result)) => (
            StatusCode::OK,
            make_etag_header(&catalog_ver),
            Json(json!({
                "ok": true,
                "trace_id": trace_id,
                "ticket_id": ticket_id,
                "data": result,
                "error": null,
            })),
        )
            .into_response(),
        Ok(Err(err)) => (
            StatusCode::BAD_REQUEST,
            make_etag_header(&catalog_ver),
            Json(json!({
                "ok": false,
                "trace_id": trace_id,
                "ticket_id": ticket_id,
                "data": null,
                "error": {
                    "code": "SAMPLING_FAILED",
                    "message": err
                }
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            make_etag_header(&catalog_ver),
            Json(json!({
                "ok": false,
                "trace_id": trace_id,
                "ticket_id": ticket_id,
                "data": null,
                "error": {
                    "code": "SAMPLING_TIMEOUT",
                    "message": "Sampling request timed out awaiting client completion"
                }
            })),
        )
            .into_response(),
    }
}

/// Handles HTTP GET `/v1/sampling/requests` listing pending or historical sampling delegation tickets.
pub async fn handle_list_sampling_requests(
    State(state): State<AppState>,
    Query(query): Query<SamplingListQuery>,
) -> impl IntoResponse {
    let requests = state
        .sampling_registry
        .list_requests(query.server_id.as_deref(), query.status.as_deref())
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "total": requests.len(),
            "requests": requests
        })),
    )
        .into_response()
}

/// Handles HTTP GET `/v1/sampling/requests/:id` retrieving a single sampling delegation ticket.
pub async fn handle_get_sampling_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.sampling_registry.get_request(&id).await {
        Some(req) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "request": req
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "NOT_FOUND",
                    "message": format!("Sampling ticket '{}' not found", id)
                }
            })),
        )
            .into_response(),
    }
}

/// Handles HTTP POST `/v1/sampling/requests/:id/respond` resolving a pending sampling ticket with LLM completion.
pub async fn handle_respond_sampling_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RespondSamplingRequest>,
) -> impl IntoResponse {
    let success = state
        .sampling_registry
        .respond_to_request(&id, payload.result)
        .await;

    if success {
        (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "ticket_id": id,
                "status": "completed",
                "message": "Sampling completion accepted and delivered to caller."
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": {
                    "code": "TICKET_NOT_FOUND_OR_NOT_PENDING",
                    "message": format!("Sampling ticket '{}' not found or not in pending status", id)
                }
            })),
        )
            .into_response()
    }
}

/// Handles HTTP GET `/v1/capabilities` listing all registered capabilities.
pub async fn handle_list_capabilities(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    state.total_catalog_requests.fetch_add(1, Ordering::Relaxed);

    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let base_pol = state.policy.read().await;
    let pol = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.effective_policy.clone())
        .unwrap_or_else(|| base_pol.clone())
        .merge_with_profile(prof_ctx.profile_policy.as_ref());

    let base_ver = state.catalog_version.read().await.clone();
    let catalog_ver =
        crate::http_v1::helpers::get_profile_scoped_catalog_version(&base_ver, &prof_ctx);

    if check_if_none_match(&headers, &catalog_ver) {
        state.total_etag_hits.fetch_add(1, Ordering::Relaxed);
        return (
            StatusCode::NOT_MODIFIED,
            make_etag_header(&catalog_ver),
            Body::empty(),
        )
            .into_response();
    }

    let caps_guard = state.capabilities.read().await;
    let total_unfiltered = caps_guard.len();
    let mut capabilities = caps_guard
        .iter()
        .filter(|(id, meta)| pol.allows(id) && prof_ctx.is_server_allowed(&meta.server))
        .map(|(id, meta)| {
            json!({
                "id": id,
                "summary": meta.summary,
                "description": meta.description,
                "server": meta.server,
                "tool": meta.tool,
                "tags": meta.tags,
                "input_schema": meta.input_schema,
                "examples": meta.examples,
            })
        })
        .collect::<Vec<_>>();

    capabilities.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let allowed_count = capabilities.len();
    let hidden_by_policy = total_unfiltered.saturating_sub(allowed_count);

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "ttl_ms": 300000,
            "cache_scope": "public",
            "capabilities": capabilities,
            "total_unfiltered": total_unfiltered,
            "hidden_by_policy": hidden_by_policy,
        })),
    )
        .into_response()
}

/// Handles HTTP GET `/v1/capabilities/:id` describing a capability schema and metadata.
pub async fn handle_describe_capability(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let base_pol = state.policy.read().await;
    let pol = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.effective_policy.clone())
        .unwrap_or_else(|| base_pol.clone())
        .merge_with_profile(prof_ctx.profile_policy.as_ref());

    if !pol.allows(&id) {
        return (
            StatusCode::FORBIDDEN,
            make_etag_header(""),
            Json(error_envelope(
                next_trace_id(),
                None,
                None,
                crate::idempotency::RetryMetadata::safe("not_started"),
                "CAPABILITY_UNAUTHORIZED",
                format!("Access to capability '{}' is denied by policy", id),
                false,
            )),
        )
            .into_response();
    }

    let base_ver = state.catalog_version.read().await.clone();
    let catalog_ver =
        crate::http_v1::helpers::get_profile_scoped_catalog_version(&base_ver, &prof_ctx);

    if check_if_none_match(&headers, &catalog_ver) {
        return (
            StatusCode::NOT_MODIFIED,
            make_etag_header(&catalog_ver),
            Body::empty(),
        )
            .into_response();
    }

    let caps_guard = state.capabilities.read().await;
    match caps_guard.get(&id) {
        Some(CapabilityMeta {
            server,
            tool,
            summary: _,
            description,
            signature: _,
            input_schema,
            tags: _,
            examples,
        }) if prof_ctx.is_server_allowed(server) => (
            StatusCode::OK,
            make_etag_header(&catalog_ver),
            Json(json!({
                "version": "v1",
                "catalog_version": catalog_ver,
                "capability": {
                    "id": id,
                    "server": server,
                    "tool": tool,
                    "description": description,
                    "input_schema": input_schema,
                    "examples": examples,
                }
            })),
        )
            .into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            make_etag_header(&catalog_ver),
            Json(error_envelope(
                next_trace_id(),
                None,
                None,
                crate::idempotency::RetryMetadata::safe("not_started"),
                "TOOL_NOT_FOUND",
                format!("Capability '{}' not found", id),
                false,
            )),
        )
            .into_response(),
    }
}

/// Handles HTTP GET `/v1/resources` listing all registered resources.
pub async fn handle_list_resources(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let base_pol = state.policy.read().await;
    let pol = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.effective_policy.clone())
        .unwrap_or_else(|| base_pol.clone())
        .merge_with_profile(prof_ctx.profile_policy.as_ref());

    let base_ver = state.catalog_version.read().await.clone();
    let catalog_ver =
        crate::http_v1::helpers::get_profile_scoped_catalog_version(&base_ver, &prof_ctx);

    if check_if_none_match(&headers, &catalog_ver) {
        return (
            StatusCode::NOT_MODIFIED,
            make_etag_header(&catalog_ver),
            Body::empty(),
        )
            .into_response();
    }

    let res_guard = state.resources.read().await;
    let total_unfiltered = res_guard.len();
    let mut resources = res_guard
        .iter()
        .filter(|(id, meta)| pol.allows(id) && prof_ctx.is_server_allowed(&meta.server))
        .map(|(id, meta)| {
            json!({
                "id": id,
                "server": meta.server,
                "uri": meta.uri,
                "name": meta.name,
                "description": meta.description,
                "mime_type": meta.mime_type,
                "tags": meta.tags,
            })
        })
        .collect::<Vec<_>>();

    resources.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let allowed_count = resources.len();
    let hidden_by_policy = total_unfiltered.saturating_sub(allowed_count);

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "ttl_ms": 300000,
            "cache_scope": "public",
            "resources": resources,
            "total_unfiltered": total_unfiltered,
            "hidden_by_policy": hidden_by_policy,
        })),
    )
        .into_response()
}

/// Handles HTTP GET `/v1/prompts` listing all registered prompt templates.
pub async fn handle_list_prompts(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let base_pol = state.policy.read().await;
    let pol = req_ext
        .0
        .as_ref()
        .map(|ctx| ctx.effective_policy.clone())
        .unwrap_or_else(|| base_pol.clone())
        .merge_with_profile(prof_ctx.profile_policy.as_ref());

    let base_ver = state.catalog_version.read().await.clone();
    let catalog_ver =
        crate::http_v1::helpers::get_profile_scoped_catalog_version(&base_ver, &prof_ctx);

    if check_if_none_match(&headers, &catalog_ver) {
        return (
            StatusCode::NOT_MODIFIED,
            make_etag_header(&catalog_ver),
            Body::empty(),
        )
            .into_response();
    }

    let prompts_guard = state.prompts.read().await;
    let total_unfiltered = prompts_guard.len();
    let mut prompts = prompts_guard
        .iter()
        .filter(|(id, meta)| pol.allows(id) && prof_ctx.is_server_allowed(&meta.server))
        .map(|(id, meta)| {
            json!({
                "id": id,
                "server": meta.server,
                "name": meta.name,
                "title": meta.title,
                "description": meta.description,
                "arguments": meta.arguments,
                "tags": meta.tags,
            })
        })
        .collect::<Vec<_>>();

    prompts.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let allowed_count = prompts.len();
    let hidden_by_policy = total_unfiltered.saturating_sub(allowed_count);

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "ttl_ms": 300000,
            "cache_scope": "public",
            "prompts": prompts,
            "total_unfiltered": total_unfiltered,
            "hidden_by_policy": hidden_by_policy,
        })),
    )
        .into_response()
}

/// Handles HTTP POST `/v1/resources/read` reading an MCP resource content.
pub async fn handle_read_resource(
    State(state): State<AppState>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
    Json(payload): Json<ReadResourceRequest>,
) -> impl IntoResponse {
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let trace_id = next_trace_id();
    let request_id =
        crate::context::resolve_request_id(payload.request_id.clone(), &headers, trace_id.clone());
    let req_context = crate::context::resolve_request_context(payload.context.clone(), &headers);
    let _idempotency_key = resolve_idempotency_key(payload.idempotency_key.clone(), &headers);
    let cancel_token: tokio_util::sync::CancellationToken =
        state.operation_registry.register(&request_id).await;

    if !state.policy.read().await.allows(&payload.resource_id) {
        state.operation_registry.unregister(&request_id).await;
        return (
            StatusCode::FORBIDDEN,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "POLICY_DENIED",
                format!("Resource '{}' blocked by policy", payload.resource_id),
                false,
            )),
        )
            .into_response();
    }

    let (server_id, uri) = {
        let res_guard = state.resources.read().await;
        let Some(meta) = res_guard.get(&payload.resource_id) else {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::NOT_FOUND,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "RESOURCE_NOT_FOUND",
                    format!("Resource '{}' not found", payload.resource_id),
                    false,
                )),
            )
                .into_response();
        };

        if !prof_ctx.is_server_allowed(&meta.server) {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::NOT_FOUND,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "RESOURCE_NOT_FOUND",
                    format!(
                        "Resource '{}' not available in active profile",
                        payload.resource_id
                    ),
                    false,
                )),
            )
                .into_response();
        }

        (meta.server.clone(), meta.uri.clone())
    };

    let tx = {
        let servers_guard = state.servers.read().await;
        let Some(tx) = servers_guard.get(&server_id).cloned() else {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' is unreachable", server_id),
                    true,
                )),
            )
                .into_response();
        };
        tx
    };

    let redact_keys = state.policy.read().await.redact_keys.clone();

    info!(
        trace_id = %trace_id,
        request_id = %request_id,
        operation_id = ?req_context.operation_id,
        work_item_id = ?req_context.work_item_id,
        actor_id = ?req_context.actor_id,
        grant_id = ?req_context.grant_id,
        resource_id = %payload.resource_id,
        uri = %uri,
        "resource read start"
    );

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::ReadResource {
            uri,
            input_responses: payload.input_responses,
            request_state: payload.request_state,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        state.operation_registry.unregister(&request_id).await;
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("not_started"),
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
            return (
                StatusCode::BAD_REQUEST,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("unknown"),
                    "OPERATION_CANCELLED",
                    "Resource read operation was cancelled",
                    false,
                )),
            ).into_response();
        }
        res = reply_rx => res,
    };

    state.operation_registry.unregister(&request_id).await;

    match result {
        Ok(Ok(data)) => {
            let redacted_output = redact_value(data.clone(), &redact_keys);
            info!(
                trace_id = %trace_id,
                request_id = %request_id,
                resource_id = %payload.resource_id,
                data = %redacted_output,
                "resource read success"
            );
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "request_id": request_id,
                    "context": req_context,
                    "trace_id": trace_id,
                    "data": data,
                    "error": null,
                    "retry": crate::idempotency::RetryMetadata::safe("completed"),
                })),
            )
                .into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_TIMEOUT",
                format!("Resource read timed out after {}ms", state.tool_timeout_ms),
                true,
            )),
        )
            .into_response(),
        Ok(Err(UpstreamCallError::Upstream(err))) => (
            StatusCode::BAD_GATEWAY,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_ERROR",
                err,
                false,
            )),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        )
            .into_response(),
    }
}

/// Handles HTTP POST `/v1/prompts/get` fetching rendered prompt template.
pub async fn handle_get_prompt(
    State(state): State<AppState>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: HeaderMap,
    Json(payload): Json<GetPromptRequest>,
) -> impl IntoResponse {
    let prof_ctx = prof_ext.map(|e| e.0).unwrap_or_default();
    let trace_id = next_trace_id();
    let request_id =
        crate::context::resolve_request_id(payload.request_id.clone(), &headers, trace_id.clone());
    let req_context = crate::context::resolve_request_context(payload.context.clone(), &headers);
    let _idempotency_key = resolve_idempotency_key(payload.idempotency_key.clone(), &headers);
    let cancel_token: tokio_util::sync::CancellationToken =
        state.operation_registry.register(&request_id).await;

    let policy_guard = state.policy.read().await;
    if !policy_guard.allows(&payload.prompt_id) {
        state.operation_registry.unregister(&request_id).await;
        return (
            StatusCode::FORBIDDEN,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "POLICY_DENIED",
                format!("Prompt '{}' blocked by policy", payload.prompt_id),
                false,
            )),
        )
            .into_response();
    }
    let redact_keys = policy_guard.redact_keys.clone();
    drop(policy_guard);

    let (server_id, prompt_name) = {
        let prompts_guard = state.prompts.read().await;
        let Some(meta) = prompts_guard.get(&payload.prompt_id) else {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::NOT_FOUND,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "PROMPT_NOT_FOUND",
                    format!("Prompt '{}' not found", payload.prompt_id),
                    false,
                )),
            )
                .into_response();
        };

        if !prof_ctx.is_server_allowed(&meta.server) {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::NOT_FOUND,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "PROMPT_NOT_FOUND",
                    format!(
                        "Prompt '{}' not available in active profile",
                        payload.prompt_id
                    ),
                    false,
                )),
            )
                .into_response();
        }

        (meta.server.clone(), meta.name.clone())
    };

    let tx = {
        let servers_guard = state.servers.read().await;
        let Some(tx) = servers_guard.get(&server_id).cloned() else {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' is unreachable", server_id),
                    true,
                )),
            )
                .into_response();
        };
        tx
    };

    let arguments = match payload.arguments {
        Some(Value::Object(map)) => Some(map),
        Some(_) => {
            state.operation_registry.unregister(&request_id).await;
            return (
                StatusCode::BAD_REQUEST,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "INVALID_ARGS",
                    "'arguments' must be a JSON object when provided",
                    false,
                )),
            )
                .into_response();
        }
        None => None,
    };

    let redacted_input = redact_value(
        serde_json::to_value(&arguments).unwrap_or(Value::Null),
        &redact_keys,
    );
    info!(
        trace_id = %trace_id,
        request_id = %request_id,
        operation_id = ?req_context.operation_id,
        work_item_id = ?req_context.work_item_id,
        actor_id = ?req_context.actor_id,
        grant_id = ?req_context.grant_id,
        prompt_id = %payload.prompt_id,
        arguments = %redacted_input,
        "prompt get start"
    );

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::GetPrompt {
            name: prompt_name,
            arguments,
            input_responses: payload.input_responses,
            request_state: payload.request_state,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        state.operation_registry.unregister(&request_id).await;
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("not_started"),
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
            return (
                StatusCode::BAD_REQUEST,
                Json(error_envelope(
                    trace_id,
                    Some(request_id),
                    Some(req_context),
                    crate::idempotency::RetryMetadata::safe("unknown"),
                    "OPERATION_CANCELLED",
                    "Prompt get operation was cancelled",
                    false,
                )),
            ).into_response();
        }
        res = reply_rx => res,
    };

    state.operation_registry.unregister(&request_id).await;

    match result {
        Ok(Ok(data)) => {
            let redacted_output = redact_value(data.clone(), &redact_keys);
            info!(
                trace_id = %trace_id,
                request_id = %request_id,
                prompt_id = %payload.prompt_id,
                data = %redacted_output,
                "prompt get success"
            );
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "request_id": request_id,
                    "context": req_context,
                    "trace_id": trace_id,
                    "data": data,
                    "error": null,
                    "retry": crate::idempotency::RetryMetadata::safe("completed"),
                })),
            )
                .into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_TIMEOUT",
                format!("Prompt get timed out after {}ms", state.tool_timeout_ms),
                true,
            )),
        )
            .into_response(),
        Ok(Err(UpstreamCallError::Upstream(err))) => (
            StatusCode::BAD_GATEWAY,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_ERROR",
                err,
                false,
            )),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_envelope(
                trace_id,
                Some(request_id),
                Some(req_context),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        )
            .into_response(),
    }
}
