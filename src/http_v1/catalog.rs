// Rust guideline compliant 2026-08-15

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
            GetPromptRequest, ReadResourceRequest, SamplingRequest, SearchCapabilitiesRequest,
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
    Json(payload): Json<SearchCapabilitiesRequest>,
) -> impl IntoResponse {
    let query_str = payload.query.as_deref().unwrap_or("");
    let filter = crate::search::SearchFilter::builder()
        .server_ids(payload.server_ids)
        .tags(payload.tags)
        .modes(payload.modes)
        .build();

    let caps = state.capabilities.read().await;
    let pol = state.policy.read().await;
    let catalog_ver = state.catalog_version.read().await.clone();

    let results = state
        .search_engine
        .search(query_str, payload.limit, &filter, &caps, &pol);

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

    let stream = futures::stream::unfold(
        rx,
        |mut receiver: tokio::sync::broadcast::Receiver<crate::catalog::ResourceUpdateEvent>| async move {
            loop {
                match receiver.recv().await {
                    Ok(evt) => {
                        if let Ok(data) = serde_json::to_string(&evt) {
                            let event = Event::default().event("resource_updated").data(data);
                            return Some((Ok(event), receiver));
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                }
            }
        },
    );

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

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "ok": true,
            "trace_id": trace_id,
            "data": {
                "role": "assistant",
                "content": {
                    "type": "text",
                    "text": "Warmplane sampling proxy response."
                },
                "model": "warmplane-sampling-v1",
                "stop_reason": "end_turn",
                "messages_count": payload.messages.len(),
                "max_tokens": payload.max_tokens
            }
        })),
    )
        .into_response()
}

/// Handles HTTP GET `/v1/capabilities` listing all registered capabilities.
pub async fn handle_list_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    state.total_catalog_requests.fetch_add(1, Ordering::Relaxed);

    let catalog_ver = state.catalog_version.read().await.clone();

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
    let mut capabilities = caps_guard
        .iter()
        .map(|(id, meta)| {
            json!({
                "id": id,
                "summary": meta.summary,
                "server": meta.server,
                "tool": meta.tool,
                "tags": meta.tags,
            })
        })
        .collect::<Vec<_>>();

    capabilities.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "ttl_ms": 300000,
            "cache_scope": "public",
            "capabilities": capabilities,
        })),
    )
        .into_response()
}

/// Handles HTTP GET `/v1/capabilities/:id` describing a capability schema and metadata.
pub async fn handle_describe_capability(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let catalog_ver = state.catalog_version.read().await.clone();

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
            input_schema,
            tags: _,
            examples,
        }) => (
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
        None => (
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
    headers: HeaderMap,
) -> impl IntoResponse {
    let catalog_ver = state.catalog_version.read().await.clone();

    if check_if_none_match(&headers, &catalog_ver) {
        return (
            StatusCode::NOT_MODIFIED,
            make_etag_header(&catalog_ver),
            Body::empty(),
        )
            .into_response();
    }

    let res_guard = state.resources.read().await;
    let mut resources = res_guard
        .iter()
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

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "ttl_ms": 300000,
            "cache_scope": "public",
            "resources": resources,
        })),
    )
        .into_response()
}

/// Handles HTTP GET `/v1/prompts` listing all registered prompt templates.
pub async fn handle_list_prompts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let catalog_ver = state.catalog_version.read().await.clone();

    if check_if_none_match(&headers, &catalog_ver) {
        return (
            StatusCode::NOT_MODIFIED,
            make_etag_header(&catalog_ver),
            Body::empty(),
        )
            .into_response();
    }

    let prompts_guard = state.prompts.read().await;
    let mut prompts = prompts_guard
        .iter()
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

    (
        StatusCode::OK,
        make_etag_header(&catalog_ver),
        Json(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "ttl_ms": 300000,
            "cache_scope": "public",
            "prompts": prompts,
        })),
    )
        .into_response()
}

/// Handles HTTP POST `/v1/resources/read` reading an MCP resource content.
pub async fn handle_read_resource(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReadResourceRequest>,
) -> impl IntoResponse {
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
                "INVALID_ARGS",
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
    headers: HeaderMap,
    Json(payload): Json<GetPromptRequest>,
) -> impl IntoResponse {
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
                "INVALID_ARGS",
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
