// Rust guideline compliant 2026-08-13

//! HTTP v1 facade API handlers for capabilities, resources, prompts, events, and operations.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;
use tracing::info;

use crate::daemon::{AppState, CapabilityMeta, ServerMsg, UpstreamCallError};

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Validates whether HTTP `If-None-Match` header matches catalog version for HTTP 304 response.
///
/// # Arguments
/// * `req_headers` - Incoming request HTTP headers.
/// * `catalog_version` - Current active catalog ETag version string.
///
/// # Returns
/// `true` if catalog version matches `If-None-Match` header, `false` otherwise.
pub fn check_if_none_match(req_headers: &HeaderMap, catalog_version: impl AsRef<str>) -> bool {
    let version_ref = catalog_version.as_ref();
    if let Some(if_none_match) = req_headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            let val_clean = val.trim();
            let version_quoted = format!("\"{}\"", version_ref);
            return val_clean == version_ref || val_clean == version_quoted || val_clean == "*";
        }
    }
    false
}

/// Constructs HTTP `ETag` response headers matching catalog version.
///
/// # Arguments
/// * `catalog_version` - Current active catalog ETag version string.
///
/// # Returns
/// HeaderMap containing formatted `ETag` header.
pub fn make_etag_header(catalog_version: impl AsRef<str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let etag_val = format!("\"{}\"", catalog_version.as_ref());
    if let Ok(hv) = HeaderValue::from_str(&etag_val) {
        headers.insert(header::ETAG, hv);
    }
    headers
}

/// Query parameters for listing catalog events.
#[derive(Deserialize)]
pub struct CatalogEventsQuery {
    /// Optional cursor ID to fetch events after.
    pub after: Option<String>,
}

/// Response envelope for catalog event change feed.
#[derive(serde::Serialize)]
pub struct CatalogEventsResponse {
    /// Current catalog version ETag.
    pub catalog_version: String,
    /// Next cursor ID for event pagination.
    pub cursor: String,
    /// List of catalog mutation events.
    pub events: Vec<crate::catalog::CatalogEvent>,
}

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

/// Request body for capability execution.
#[derive(Deserialize)]
pub struct CallCapabilityRequest {
    /// Identifier or alias of capability to execute.
    pub capability_id: String,
    /// JSON arguments for capability execution.
    pub args: Value,
    /// Optional request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context metadata envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
    /// Optional key for idempotent request deduplication.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Optional MRTR client input responses for multi-roundtrip retry.
    #[serde(default)]
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Optional MRTR opaque request state for multi-roundtrip retry.
    #[serde(default)]
    pub request_state: Option<String>,
}

/// Request body for reading a resource.
#[derive(Deserialize)]
pub struct ReadResourceRequest {
    /// Identifier or alias of resource URI to read.
    pub resource_id: String,
    /// Optional request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context metadata envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
    /// Optional key for idempotent request deduplication.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Optional MRTR client input responses for multi-roundtrip retry.
    #[serde(default)]
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Optional MRTR opaque request state for multi-roundtrip retry.
    #[serde(default)]
    pub request_state: Option<String>,
}

/// Request body for fetching a prompt template.
#[derive(Deserialize)]
pub struct GetPromptRequest {
    /// Identifier or alias of prompt name to get.
    pub prompt_id: String,
    /// Optional arguments map for prompt rendering.
    #[serde(default)]
    pub arguments: Option<Value>,
    /// Optional request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context metadata envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
    /// Optional key for idempotent request deduplication.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Optional MRTR client input responses for multi-roundtrip retry.
    #[serde(default)]
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Optional MRTR opaque request state for multi-roundtrip retry.
    #[serde(default)]
    pub request_state: Option<String>,
}

fn default_search_limit() -> usize {
    8
}

/// Request body for hybrid capability search.
#[derive(Deserialize)]
pub struct SearchCapabilitiesRequest {
    /// Optional plain-text search query string.
    pub query: Option<String>,
    /// Maximum number of search results to return (default 8).
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    /// Filter results to specified server IDs.
    #[serde(default)]
    pub server_ids: Vec<String>,
    /// Filter results to specified tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Filter results to specified execution modes.
    #[serde(default)]
    pub modes: Vec<String>,
}

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

/// Request body for prompt/resource argument autocompletion.
#[derive(Deserialize)]
pub struct CompletionRequest {
    /// Reference type (`"prompt"` or `"resource"`).
    pub ref_type: String,
    /// Identifier or name of the prompt or resource.
    pub ref_name: String,
    /// Name of argument to autocomplete.
    pub argument_name: String,
    /// Current prefix value typed by user/agent.
    #[serde(default)]
    pub argument_value: String,
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

/// Request body for sampling LLM completion delegation.
#[derive(Deserialize)]
pub struct SamplingRequest {
    /// Server identifier originating the sampling request.
    pub server_id: String,
    /// System prompt or context messages array.
    #[serde(default)]
    pub messages: Vec<Value>,
    /// Optional max tokens limit.
    #[serde(default)]
    pub max_tokens: Option<usize>,
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

fn resolve_idempotency_key(payload_key: Option<String>, headers: &HeaderMap) -> Option<String> {
    if let Some(k) = payload_key.filter(|s| !s.trim().is_empty()) {
        return Some(k);
    }
    crate::context::extract_header_str(headers, "idempotency-key")
        .or_else(|| crate::context::extract_header_str(headers, "x-idempotency-key"))
}

pub async fn handle_call_capability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CallCapabilityRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();
    state.total_tool_calls.fetch_add(1, Ordering::Relaxed);
    let trace_id = next_trace_id();
    let request_id =
        crate::context::resolve_request_id(payload.request_id.clone(), &headers, trace_id.clone());
    let req_context = crate::context::resolve_request_context(payload.context.clone(), &headers);
    let idempotency_key = resolve_idempotency_key(payload.idempotency_key.clone(), &headers);

    let retry_base = if idempotency_key.is_some() {
        crate::idempotency::RetryMetadata::idempotent
    } else {
        crate::idempotency::RetryMetadata::unsafe_op
    };

    if let Some(ref key) = idempotency_key {
        match state.idempotency_store.check_or_start(key).await {
            crate::idempotency::DeduplicateResult::Completed(cached) => {
                return (StatusCode::OK, Json(cached)).into_response();
            }
            crate::idempotency::DeduplicateResult::InProgress(mut rx) => {
                if let Ok(cached) = rx.recv().await {
                    return (StatusCode::OK, Json(cached)).into_response();
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

    let policy_guard = state.policy.read().await;
    if !policy_guard.allows(&payload.capability_id) {
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
                "INVALID_ARGS",
                format!("Capability '{}' blocked by policy", payload.capability_id),
                false,
            )),
        )
            .into_response();
    }
    let redact_keys = policy_guard.redact_keys.clone();
    drop(policy_guard);

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
        (meta.server.clone(), meta.tool.clone())
    };

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

    let redacted_input = redact_value(payload.args.clone(), &redact_keys);
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

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::CallTool {
            name: tool_name,
            params: payload.args,
            input_responses: payload.input_responses,
            request_state: payload.request_state,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
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

            let response_json = json!({
                "ok": true,
                "request_id": request_id,
                "context": req_context,
                "trace_id": trace_id,
                "data": data,
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
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
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
            if let Some(ref key) = idempotency_key {
                state.idempotency_store.remove(key).await;
            }
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

fn next_trace_id() -> String {
    format!("trace-{}", TRACE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn error_envelope(
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    retry: crate::idempotency::RetryMetadata,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> Value {
    let ctx_val = context.unwrap_or_default();
    json!({
        "ok": false,
        "request_id": request_id,
        "context": ctx_val,
        "trace_id": trace_id,
        "data": null,
        "error": {
            "code": code,
            "message": message.into(),
            "retryable": retryable,
        },
        "retry": retry,
    })
}

pub async fn handle_cancel_operation(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let cancelled = state.operation_registry.cancel(&id).await;
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "request_id": id,
            "cancelled": cancelled,
        })),
    )
        .into_response()
}

fn redact_value(value: Value, redact_keys: &[String]) -> Value {
    match value {
        Value::Object(map) => {
            let mut output = serde_json::Map::new();
            for (key, nested) in map {
                if redact_keys.iter().any(|k| k == &key.to_lowercase()) {
                    output.insert(key, Value::String("<redacted>".to_string()));
                } else {
                    output.insert(key, redact_value(nested, redact_keys));
                }
            }
            Value::Object(output)
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|entry| redact_value(entry, redact_keys))
                .collect(),
        ),
        primitive => primitive,
    }
}

// ============================================================
// CONFIGURATION & CONTROL DECK REST API HANDLERS
// ============================================================

/// Handles GET `/v1/config` returning the active configuration.
pub async fn handle_get_config(State(state): State<AppState>) -> impl IntoResponse {
    let total_reqs = state.total_catalog_requests.load(Ordering::Relaxed);
    let etag_hits = state.total_etag_hits.load(Ordering::Relaxed);
    let tool_calls = state.total_tool_calls.load(Ordering::Relaxed);
    let tool_duration_us = state.total_tool_duration_us.load(Ordering::Relaxed);
    let srv_configs = state.server_configs.read().await.clone();
    let srv_statuses = state.server_statuses.read().await.clone();

    match crate::config::load_or_default_config(&state.config_path) {
        Ok(config) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "config_path": state.config_path,
                "config": config,
                "server_configs": srv_configs,
                "server_statuses": srv_statuses,
                "metrics": {
                    "total_catalog_requests": total_reqs,
                    "total_etag_hits": etag_hits,
                    "total_tool_calls": tool_calls,
                    "total_tool_duration_us": tool_duration_us,
                }
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": err.to_string() })),
        )
            .into_response(),
    }
}

/// Request payload for adding or updating an upstream server.
#[derive(Deserialize)]
pub struct UpsertServerRequest {
    pub name: String,
    pub server: crate::config::ServerConfig,
}

/// Handles POST `/v1/config/servers` to add or update an upstream server.
pub async fn handle_upsert_server(
    State(state): State<AppState>,
    Json(payload): Json<UpsertServerRequest>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    config
        .mcp_servers
        .insert(payload.name.clone(), payload.server.clone());
    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Hot-mount the newly added/updated server into active daemon state
    let mount_result = state
        .mount_upstream_server(
            &payload.name,
            &payload.server,
            &config.capability_aliases,
            &config.resource_aliases,
            &config.prompt_aliases,
        )
        .await;

    let warning = mount_result.err().map(|e| e.to_string());

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message": if warning.is_some() {
                format!("Server '{}' saved to configuration, but initial connection failed", payload.name)
            } else {
                format!("Server '{}' connected and mounted successfully", payload.name)
            },
            "warning": warning,
            "server": payload.name
        })),
    )
        .into_response()
}

/// Handles DELETE `/v1/config/servers/:id` to remove an upstream server.
pub async fn handle_delete_server(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if config.mcp_servers.remove(&id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": format!("Server '{}' not found", id) })),
        )
            .into_response();
    }

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Hot-unmount server from runtime
    state.unmount_upstream_server(&id).await;

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message": format!("Server '{}' unmounted and removed", id),
        })),
    )
        .into_response()
}

/// Handles GET `/v1/config/ecosystem` discovering external MCP sources.
pub async fn handle_get_ecosystem_sources() -> impl IntoResponse {
    let sources = crate::config_import::discover_sources();
    let serializable_sources: Vec<_> = sources
        .into_iter()
        .map(|s| {
            json!({
                "name": s.name,
                "path": s.path.to_string_lossy(),
                "server_count": s.server_count,
                "servers": s.servers.keys().cloned().collect::<Vec<_>>(),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "sources": serializable_sources,
        })),
    )
        .into_response()
}

/// Request payload for ecosystem config import.
#[derive(Deserialize)]
pub struct ImportConfigRequest {
    pub source_path: Option<String>,
    pub overwrite: Option<bool>,
}

/// Handles POST `/v1/config/import` importing external MCP configurations.
pub async fn handle_import_config(
    State(state): State<AppState>,
    Json(payload): Json<ImportConfigRequest>,
) -> impl IntoResponse {
    let overwrite = payload.overwrite.unwrap_or(false);
    let mut all_imported = 0;
    let mut all_skipped = Vec::new();

    let sources = if let Some(path_str) = payload.source_path {
        let pb = std::path::PathBuf::from(path_str);
        match crate::config_import::parse_standard_mcp_source("Custom File", pb) {
            Ok(s) => vec![s],
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    } else {
        crate::config_import::discover_sources()
    };

    let active_cfg = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    for src in sources {
        let servers_to_mount = src.servers.clone();
        match crate::config_import::import_servers_into_config(
            &state.config_path,
            src.servers,
            overwrite,
        ) {
            Ok((count, skipped)) => {
                all_imported += count;
                all_skipped.extend(skipped);

                // Dynamically mount newly imported servers
                for (s_id, s_cfg) in servers_to_mount {
                    let _ = state
                        .mount_upstream_server(
                            &s_id,
                            &s_cfg,
                            &active_cfg.capability_aliases,
                            &active_cfg.resource_aliases,
                            &active_cfg.prompt_aliases,
                        )
                        .await;
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "ok": false, "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "imported_count": all_imported,
            "skipped_servers": all_skipped,
        })),
    )
        .into_response()
}

/// Request payload for updating alias mappings.
#[derive(Deserialize)]
pub struct UpdateAliasRequest {
    pub kind: String,
    pub alias: String,
    pub target: Option<String>,
}

/// Handles POST `/v1/config/alias` adding or removing an alias.
pub async fn handle_update_alias(
    State(state): State<AppState>,
    Json(payload): Json<UpdateAliasRequest>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let kind_lower = payload.kind.to_lowercase();
    if let Some(target) = payload.target {
        match kind_lower.as_str() {
            "tool" | "capability" | "cap" => {
                config
                    .capability_aliases
                    .insert(payload.alias.clone(), target);
            }
            "resource" | "res" => {
                config
                    .resource_aliases
                    .insert(payload.alias.clone(), target);
            }
            "prompt" => {
                config.prompt_aliases.insert(payload.alias.clone(), target);
            }
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": "Invalid alias kind. Expected 'tool', 'resource', or 'prompt'" })),
                ).into_response();
            }
        }
    } else {
        match kind_lower.as_str() {
            "tool" | "capability" | "cap" => {
                config.capability_aliases.remove(&payload.alias);
            }
            "resource" | "res" => {
                config.resource_aliases.remove(&payload.alias);
            }
            "prompt" => {
                config.prompt_aliases.remove(&payload.alias);
            }
            _ => {}
        }
    }

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

/// Handles POST `/v1/config/policy` updating security policies.
pub async fn handle_update_policy(
    State(state): State<AppState>,
    Json(payload): Json<crate::config::PolicyConfig>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    config.policy = Some(payload.clone());
    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Immediately update in-memory active policy
    {
        let mut pol_guard = state.policy.write().await;
        *pol_guard = crate::daemon::Policy::from_config(Some(payload));
    }

    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

/// Handles POST `/v1/config/reload` explicitly triggering a hot-reloading reconciliation from disk.
pub async fn handle_reload_config(State(state): State<AppState>) -> impl IntoResponse {
    match state.reload_from_disk().await {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Serves the Warmplane Control Deck single-page web UI.
pub async fn handle_ui_dashboard() -> impl IntoResponse {
    let html = include_str!("../ui/dist/index.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

#[cfg(test)]
mod tests {
    use super::{
        handle_call_capability, handle_catalog_events, handle_completion, handle_get_prompt,
        handle_list_capabilities, handle_list_prompts, handle_list_resources, handle_read_resource,
        handle_sampling_create_message, redact_value, AppState, CallCapabilityRequest,
        CatalogEventsQuery, CompletionRequest, GetPromptRequest, ReadResourceRequest,
        SamplingRequest,
    };
    use crate::daemon::{CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg};
    use axum::{
        body::to_bytes,
        extract::{Query, State},
        http::{header, HeaderMap, HeaderValue, StatusCode},
        response::IntoResponse,
        Json,
    };
    use serde_json::{json, Value};
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::mpsc;

    #[test]
    fn redact_value_masks_nested_keys_case_insensitive() {
        let input = json!({
            "token": "abc",
            "nested": {
                "Api_Key": "xyz",
                "safe": 1
            }
        });

        let redacted = redact_value(input, &["token".to_string(), "api_key".to_string()]);

        assert_eq!(redacted["token"], "<redacted>");
        assert_eq!(redacted["nested"]["Api_Key"], "<redacted>");
        assert_eq!(redacted["nested"]["safe"], 1);
    }

    #[tokio::test]
    async fn test_completion_endpoint() {
        let mut prompts = HashMap::new();
        prompts.insert(
            "prompt.test".to_string(),
            PromptMeta {
                server: "srv".to_string(),
                name: "test".to_string(),
                title: None,
                description: None,
                arguments: vec![],
                tags: vec![],
            },
        );

        let state = AppState::builder().prompts(prompts).build();

        let req = CompletionRequest {
            ref_type: "prompt".to_string(),
            ref_name: "prompt.test".to_string(),
            argument_name: "file".to_string(),
            argument_value: "main".to_string(),
        };

        let response = handle_completion(State(state), Json(req))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_sampling_endpoint() {
        let mut servers = HashMap::new();
        let (tx, _rx) = mpsc::channel(1);
        servers.insert("srv".to_string(), tx);

        let state = AppState::builder().servers(servers).build();

        let req = SamplingRequest {
            server_id: "srv".to_string(),
            messages: vec![json!({"role": "user", "content": "hello"})],
            max_tokens: Some(100),
        };

        let response = handle_sampling_create_message(State(state), Json(req))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn list_resources_returns_sorted_ids() {
        let mut resources = HashMap::new();
        resources.insert(
            "zeta.res".to_string(),
            ResourceMeta {
                server: "s1".to_string(),
                uri: "file:///zeta".to_string(),
                name: "zeta".to_string(),
                description: None,
                mime_type: None,
                tags: vec!["s1".to_string()],
            },
        );
        resources.insert(
            "alpha.res".to_string(),
            ResourceMeta {
                server: "s1".to_string(),
                uri: "file:///alpha".to_string(),
                name: "alpha".to_string(),
                description: Some("a".to_string()),
                mime_type: Some("text/plain".to_string()),
                tags: vec!["s1".to_string()],
            },
        );

        let state = AppState::builder()
            .resources(resources)
            .catalog_version("sha256:test")
            .build();

        let response = handle_list_resources(State(state), HeaderMap::new())
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body: Value = serde_json::from_slice(&bytes).expect("json");
        let entries = body
            .get("resources")
            .and_then(Value::as_array)
            .expect("resources array");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["id"], "alpha.res");
        assert_eq!(entries[1]["id"], "zeta.res");
    }

    #[tokio::test]
    async fn read_resource_returns_not_found_code() {
        let state = AppState::builder().catalog_version("sha256:test").build();

        let response = handle_read_resource(
            State(state),
            HeaderMap::new(),
            Json(ReadResourceRequest {
                resource_id: "missing.resource".to_string(),
                request_id: None,
                context: None,
                idempotency_key: None,
                input_responses: None,
                request_state: None,
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(payload["error"]["code"], "RESOURCE_NOT_FOUND");
    }

    #[tokio::test]
    async fn list_prompts_returns_sorted_ids() {
        let mut prompts = HashMap::new();
        prompts.insert(
            "zeta.prompt".to_string(),
            PromptMeta {
                server: "s1".to_string(),
                name: "zeta".to_string(),
                title: None,
                description: Some("z".to_string()),
                arguments: vec![],
                tags: vec!["s1".to_string()],
            },
        );
        prompts.insert(
            "alpha.prompt".to_string(),
            PromptMeta {
                server: "s1".to_string(),
                name: "alpha".to_string(),
                title: Some("Alpha".to_string()),
                description: None,
                arguments: vec![],
                tags: vec!["s1".to_string()],
            },
        );

        let state = AppState::builder()
            .prompts(prompts)
            .catalog_version("sha256:test")
            .build();

        let response = handle_list_prompts(State(state), HeaderMap::new())
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let body: Value = serde_json::from_slice(&bytes).expect("json");
        let entries = body
            .get("prompts")
            .and_then(Value::as_array)
            .expect("prompts array");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["id"], "alpha.prompt");
        assert_eq!(entries[1]["id"], "zeta.prompt");
    }

    #[tokio::test]
    async fn get_prompt_returns_not_found_code() {
        let state = AppState::builder().catalog_version("sha256:test").build();

        let response = handle_get_prompt(
            State(state),
            HeaderMap::new(),
            Json(GetPromptRequest {
                prompt_id: "missing.prompt".to_string(),
                arguments: None,
                request_id: None,
                context: None,
                idempotency_key: None,
                input_responses: None,
                request_state: None,
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(payload["error"]["code"], "PROMPT_NOT_FOUND");
    }

    #[tokio::test]
    async fn get_prompt_rejects_non_object_arguments() {
        let mut prompts = HashMap::new();
        prompts.insert(
            "alpha.prompt".to_string(),
            PromptMeta {
                server: "s1".to_string(),
                name: "alpha".to_string(),
                title: None,
                description: None,
                arguments: vec![],
                tags: vec!["s1".to_string()],
            },
        );

        let (tx, _rx) = mpsc::channel(1);
        let mut servers = HashMap::new();
        servers.insert("s1".to_string(), tx);

        let state = AppState::builder()
            .servers(servers)
            .prompts(prompts)
            .catalog_version("sha256:test")
            .build();

        let response = handle_get_prompt(
            State(state),
            HeaderMap::new(),
            Json(GetPromptRequest {
                prompt_id: "alpha.prompt".to_string(),
                arguments: Some(json!("not-an-object")),
                request_id: None,
                context: None,
                idempotency_key: None,
                input_responses: None,
                request_state: None,
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let payload: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(payload["error"]["code"], "INVALID_ARGS");
    }

    #[tokio::test]
    async fn handle_search_capabilities_returns_matched_results() {
        use super::handle_search_capabilities;

        let mut capabilities = HashMap::new();
        capabilities.insert(
            "github.issues.search".to_string(),
            CapabilityMeta {
                server: "github".to_string(),
                tool: "issues.search".to_string(),
                summary: "Search open GitHub issues".to_string(),
                description: "Search open GitHub issues".to_string(),
                input_schema: json!({}),
                tags: vec!["github".to_string(), "issues".to_string()],
                examples: vec![],
            },
        );

        let state = AppState::builder()
            .capabilities(capabilities)
            .catalog_version("sha256:test_cat")
            .build();

        let response = handle_search_capabilities(
            State(state),
            Json(super::SearchCapabilitiesRequest {
                query: Some("issues".to_string()),
                limit: 5,
                server_ids: vec![],
                tags: vec![],
                modes: vec![],
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let res: Value = serde_json::from_slice(&bytes).expect("json");

        assert_eq!(res["version"], "v1");
        assert_eq!(res["catalog_version"], "sha256:test_cat");
        let caps = res["capabilities"].as_array().expect("capabilities array");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0]["id"], "github.issues.search");
    }

    #[tokio::test]
    async fn if_none_match_returns_304_not_modified() {
        let state = AppState::builder()
            .catalog_version("sha256:abc1234")
            .build();

        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_static("\"sha256:abc1234\""),
        );

        let response = handle_list_capabilities(State(state), headers)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(
            response
                .headers()
                .get(header::ETAG)
                .unwrap()
                .to_str()
                .unwrap(),
            "\"sha256:abc1234\""
        );
    }

    #[tokio::test]
    async fn catalog_events_endpoint_returns_event_feed() {
        let event_store = Arc::new(crate::catalog::CatalogEventStore::new());
        event_store.record("capability", "test.tool", "added");

        let state = AppState::builder()
            .catalog_version("sha256:v1")
            .event_store(event_store)
            .build();

        let response =
            handle_catalog_events(State(state), Query(CatalogEventsQuery { after: None }))
                .await
                .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&bytes).expect("json");

        assert_eq!(payload["catalog_version"], "sha256:v1");
        assert_eq!(payload["cursor"], "evt_1");
        let events = payload["events"].as_array().expect("events array");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["object_id"], "test.tool");
    }

    #[tokio::test]
    async fn test_request_context_and_header_fallback_in_envelope() {
        let state = AppState::builder().catalog_version("sha256:test").build();

        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", "req-hdr-999".parse().unwrap());
        headers.insert("x-actor-id", "actor-hdr-12".parse().unwrap());
        headers.insert("x-grant-id", "grant-hdr-55".parse().unwrap());

        let response = handle_read_resource(
            State(state),
            headers,
            Json(ReadResourceRequest {
                resource_id: "missing.res".to_string(),
                request_id: None,
                context: Some(crate::context::RequestContext {
                    operation_id: Some("op-payload-1".to_string()),
                    ..Default::default()
                }),
                idempotency_key: None,
                input_responses: None,
                request_state: None,
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&bytes).expect("json");

        assert_eq!(payload["request_id"], "req-hdr-999");
        assert_eq!(payload["context"]["operation_id"], "op-payload-1");
        assert_eq!(payload["context"]["actor_id"], "actor-hdr-12");
        assert_eq!(payload["context"]["grant_id"], "grant-hdr-55");
    }

    #[tokio::test]
    async fn test_mrtr_call_capability_round_trip() {
        let mut capabilities = HashMap::new();
        capabilities.insert(
            "test.interactive_tool".to_string(),
            CapabilityMeta {
                server: "interactive_srv".to_string(),
                tool: "interactive_tool".to_string(),
                summary: "Interactive Tool".to_string(),
                description: "Interactive tool description".to_string(),
                input_schema: json!({"type": "object"}),
                tags: vec![],
                examples: vec![],
            },
        );

        let (tx, mut rx) = mpsc::channel(1);
        let mut servers = HashMap::new();
        servers.insert("interactive_srv".to_string(), tx);

        let state = AppState::builder()
            .capabilities(capabilities)
            .servers(servers)
            .catalog_version("sha256:test")
            .build();

        // Spawn mock upstream worker to verify MRTR fields received
        tokio::spawn(async move {
            if let Some(ServerMsg::CallTool {
                name,
                params,
                input_responses,
                request_state,
                reply,
            }) = rx.recv().await
            {
                assert_eq!(name, "interactive_tool");
                assert_eq!(params["param1"], "val1");
                let responses = input_responses.expect("input_responses present");
                assert_eq!(responses.get("prompt_1").unwrap(), "user_input_value");
                assert_eq!(request_state.as_deref(), Some("opaque_step_2_state"));

                let _ = reply.send(Ok(json!({
                    "resultType": "complete",
                    "content": [{"type": "text", "text": "MRTR success"}]
                })));
            }
        });

        // Test JSON deserialization round-trip
        let request_json = json!({
            "capability_id": "test.interactive_tool",
            "args": {"param1": "val1"},
            "request_id": "req-mrtr-101",
            "input_responses": {
                "prompt_1": "user_input_value"
            },
            "request_state": "opaque_step_2_state"
        });
        let req: CallCapabilityRequest =
            serde_json::from_value(request_json).expect("valid CallCapabilityRequest JSON");

        let response = handle_call_capability(State(state), HeaderMap::new(), Json(req))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(payload["ok"], true);
        assert_eq!(payload["request_id"], "req-mrtr-101");
        assert_eq!(payload["data"]["resultType"], "complete");
    }

    #[tokio::test]
    async fn test_mrtr_read_resource_round_trip() {
        let mut resources = HashMap::new();
        resources.insert(
            "test.interactive_res".to_string(),
            ResourceMeta {
                server: "interactive_srv".to_string(),
                uri: "custom://res/1".to_string(),
                name: "Interactive Res".to_string(),
                description: None,
                mime_type: None,
                tags: vec![],
            },
        );

        let (tx, mut rx) = mpsc::channel(1);
        let mut servers = HashMap::new();
        servers.insert("interactive_srv".to_string(), tx);

        let state = AppState::builder()
            .resources(resources)
            .servers(servers)
            .catalog_version("sha256:test")
            .build();

        // Spawn mock upstream worker to verify MRTR fields received
        tokio::spawn(async move {
            if let Some(ServerMsg::ReadResource {
                uri,
                input_responses,
                request_state,
                reply,
            }) = rx.recv().await
            {
                assert_eq!(uri, "custom://res/1");
                let responses = input_responses.expect("input_responses present");
                assert_eq!(responses.get("auth_token").unwrap(), "token_123");
                assert_eq!(request_state.as_deref(), Some("step_state_res"));

                let _ = reply.send(Ok(json!({
                    "contents": [{"uri": "custom://res/1", "text": "Resource content"}]
                })));
            }
        });

        // Test JSON deserialization round-trip
        let request_json = json!({
            "resource_id": "test.interactive_res",
            "request_id": "req-mrtr-res-202",
            "input_responses": {
                "auth_token": "token_123"
            },
            "request_state": "step_state_res"
        });
        let req: ReadResourceRequest =
            serde_json::from_value(request_json).expect("valid ReadResourceRequest JSON");

        let response = handle_read_resource(State(state), HeaderMap::new(), Json(req))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(payload["ok"], true);
        assert_eq!(payload["request_id"], "req-mrtr-res-202");
        assert_eq!(payload["data"]["contents"][0]["text"], "Resource content");
    }

    #[tokio::test]
    async fn test_ui_and_config_rest_api() {
        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_http_cfg_test_{}", std::process::id()));
        let config_file = temp_dir.join("mcp_servers.json");
        let cfg_str = config_file.to_str().unwrap().to_string();

        let state = AppState::builder()
            .config_path(cfg_str.clone())
            .catalog_version("sha256:test")
            .build();

        // 1. Test UI Dashboard handler
        let ui_res = super::handle_ui_dashboard().await.into_response();
        assert_eq!(ui_res.status(), StatusCode::OK);
        assert_eq!(
            ui_res.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        // 2. Test Get Config (empty/default)
        let get_cfg_res = super::handle_get_config(State(state.clone()))
            .await
            .into_response();
        assert_eq!(get_cfg_res.status(), StatusCode::OK);

        // 3. Test Upsert Server
        let mut new_server = crate::config::ServerConfig::default();
        new_server.command = Some("node".to_string());
        new_server.args = vec!["index.js".to_string()];

        let upsert_res = super::handle_upsert_server(
            State(state.clone()),
            Json(super::UpsertServerRequest {
                name: "node_srv".to_string(),
                server: new_server,
            }),
        )
        .await
        .into_response();
        assert_eq!(upsert_res.status(), StatusCode::OK);

        // Verify config written
        let reloaded = crate::config::load_config(&cfg_str).unwrap();
        assert!(reloaded.mcp_servers.contains_key("node_srv"));

        // 4. Test Ecosystem Sources
        let eco_res = super::handle_get_ecosystem_sources().await.into_response();
        assert_eq!(eco_res.status(), StatusCode::OK);

        // 5. Test Reload Config handler
        let reload_res = super::handle_reload_config(State(state.clone()))
            .await
            .into_response();
        assert_eq!(reload_res.status(), StatusCode::OK);

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
