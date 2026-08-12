use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;
use tracing::info;

use crate::daemon::{AppState, CapabilityMeta, ServerMsg, UpstreamCallError};

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn check_if_none_match(req_headers: &HeaderMap, catalog_version: &str) -> bool {
    if let Some(if_none_match) = req_headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            let val_clean = val.trim();
            let version_quoted = format!("\"{}\"", catalog_version);
            return val_clean == catalog_version || val_clean == version_quoted || val_clean == "*";
        }
    }
    false
}

pub fn make_etag_header(catalog_version: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let etag_val = format!("\"{}\"", catalog_version);
    if let Ok(hv) = HeaderValue::from_str(&etag_val) {
        headers.insert(header::ETAG, hv);
    }
    headers
}

#[derive(Deserialize)]
pub struct CatalogEventsQuery {
    pub after: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CatalogEventsResponse {
    pub catalog_version: String,
    pub cursor: String,
    pub events: Vec<crate::catalog::CatalogEvent>,
}

pub async fn handle_catalog_events(
    State(state): State<AppState>,
    Query(query): Query<CatalogEventsQuery>,
) -> impl IntoResponse {
    let (events, next_cursor) = state.event_store.get_events_after(query.after.as_deref());
    (
        make_etag_header(&state.catalog_version),
        Json(CatalogEventsResponse {
            catalog_version: state.catalog_version.clone(),
            cursor: next_cursor,
            events,
        }),
    )
}

#[derive(Deserialize)]
pub struct CallCapabilityRequest {
    pub capability_id: String,
    pub args: Value,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ReadResourceRequest {
    pub resource_id: String,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Deserialize)]
pub struct GetPromptRequest {
    pub prompt_id: String,
    #[serde(default)]
    pub arguments: Option<Value>,
    #[serde(default)]
    pub request_id: Option<String>,
}

fn default_search_limit() -> usize {
    8
}

#[derive(Deserialize)]
pub struct SearchCapabilitiesRequest {
    pub query: Option<String>,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    #[serde(default)]
    pub server_ids: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub modes: Vec<String>,
}

pub async fn handle_search_capabilities(
    State(state): State<AppState>,
    Json(payload): Json<SearchCapabilitiesRequest>,
) -> impl IntoResponse {
    let query_str = payload.query.as_deref().unwrap_or("");
    let filter = crate::search::SearchFilter {
        server_ids: payload.server_ids,
        tags: payload.tags,
        modes: payload.modes,
    };

    let results = state.search_engine.search(
        query_str,
        payload.limit,
        &filter,
        &state.capabilities,
        &state.policy,
    );

    (
        make_etag_header(&state.catalog_version),
        Json(json!({
            "version": "v1",
            "catalog_version": state.catalog_version,
            "capabilities": results,
        })),
    )
}

pub async fn handle_list_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if check_if_none_match(&headers, &state.catalog_version) {
        return (StatusCode::NOT_MODIFIED, make_etag_header(&state.catalog_version), Body::empty()).into_response();
    }

    let mut capabilities = state
        .capabilities
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
        make_etag_header(&state.catalog_version),
        Json(json!({
            "version": "v1",
            "catalog_version": state.catalog_version,
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
    if check_if_none_match(&headers, &state.catalog_version) {
        return (StatusCode::NOT_MODIFIED, make_etag_header(&state.catalog_version), Body::empty()).into_response();
    }

    match state.capabilities.get(&id) {
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
            make_etag_header(&state.catalog_version),
            Json(json!({
                "version": "v1",
                "catalog_version": state.catalog_version,
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
            make_etag_header(&state.catalog_version),
            Json(error_envelope(
                next_trace_id(),
                None,
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
    if check_if_none_match(&headers, &state.catalog_version) {
        return (StatusCode::NOT_MODIFIED, make_etag_header(&state.catalog_version), Body::empty()).into_response();
    }

    let mut resources = state
        .resources
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
        make_etag_header(&state.catalog_version),
        Json(json!({
            "version": "v1",
            "catalog_version": state.catalog_version,
            "resources": resources,
        })),
    )
        .into_response()
}

pub async fn handle_list_prompts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if check_if_none_match(&headers, &state.catalog_version) {
        return (StatusCode::NOT_MODIFIED, make_etag_header(&state.catalog_version), Body::empty()).into_response();
    }

    let mut prompts = state
        .prompts
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
        make_etag_header(&state.catalog_version),
        Json(json!({
            "version": "v1",
            "catalog_version": state.catalog_version,
            "prompts": prompts,
        })),
    )
        .into_response()
}

pub async fn handle_read_resource(
    State(state): State<AppState>,
    Json(payload): Json<ReadResourceRequest>,
) -> impl IntoResponse {
    let trace_id = next_trace_id();

    if !state.policy.allows(&payload.resource_id) {
        return (
            StatusCode::FORBIDDEN,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "INVALID_ARGS",
                format!("Resource '{}' blocked by policy", payload.resource_id),
                false,
            )),
        )
            .into_response();
    }

    let Some(meta) = state.resources.get(&payload.resource_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "RESOURCE_NOT_FOUND",
                format!("Resource '{}' not found", payload.resource_id),
                false,
            )),
        )
            .into_response();
    };

    let Some(tx) = state.servers.get(&meta.server) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unreachable", meta.server),
                true,
            )),
        )
            .into_response();
    };

    let request_id = payload.request_id;
    info!(
        trace_id = %trace_id,
        resource_id = %payload.resource_id,
        uri = %meta.uri,
        "resource read start"
    );

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::ReadResource {
            uri: meta.uri.clone(),
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                request_id,
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", meta.server),
                true,
            )),
        )
            .into_response();
    }

    match reply_rx.await {
        Ok(Ok(data)) => {
            let redacted_output = redact_value(data.clone(), &state.policy.redact_keys);
            info!(
                trace_id = %trace_id,
                resource_id = %payload.resource_id,
                data = %redacted_output,
                "resource read success"
            );
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "request_id": request_id,
                    "trace_id": trace_id,
                    "data": data,
                    "error": null,
                })),
            )
                .into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(error_envelope(
                trace_id,
                request_id,
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
                request_id,
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
                request_id,
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
    Json(payload): Json<GetPromptRequest>,
) -> impl IntoResponse {
    let trace_id = next_trace_id();

    if !state.policy.allows(&payload.prompt_id) {
        return (
            StatusCode::FORBIDDEN,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "INVALID_ARGS",
                format!("Prompt '{}' blocked by policy", payload.prompt_id),
                false,
            )),
        )
            .into_response();
    }

    let Some(meta) = state.prompts.get(&payload.prompt_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "PROMPT_NOT_FOUND",
                format!("Prompt '{}' not found", payload.prompt_id),
                false,
            )),
        )
            .into_response();
    };

    let Some(tx) = state.servers.get(&meta.server) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unreachable", meta.server),
                true,
            )),
        )
            .into_response();
    };

    let arguments = match payload.arguments {
        Some(Value::Object(map)) => Some(map),
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(error_envelope(
                    trace_id,
                    payload.request_id,
                    "INVALID_ARGS",
                    "'arguments' must be a JSON object when provided",
                    false,
                )),
            )
                .into_response();
        }
        None => None,
    };

    let request_id = payload.request_id;
    let redacted_input =
        redact_value(serde_json::to_value(&arguments).unwrap_or(Value::Null), &state.policy.redact_keys);
    info!(
        trace_id = %trace_id,
        prompt_id = %payload.prompt_id,
        arguments = %redacted_input,
        "prompt get start"
    );

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::GetPrompt {
            name: meta.name.clone(),
            arguments,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                request_id,
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", meta.server),
                true,
            )),
        )
            .into_response();
    }

    match reply_rx.await {
        Ok(Ok(data)) => {
            let redacted_output = redact_value(data.clone(), &state.policy.redact_keys);
            info!(
                trace_id = %trace_id,
                prompt_id = %payload.prompt_id,
                data = %redacted_output,
                "prompt get success"
            );
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "request_id": request_id,
                    "trace_id": trace_id,
                    "data": data,
                    "error": null,
                })),
            )
                .into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(error_envelope(
                trace_id,
                request_id,
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
                request_id,
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
                request_id,
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        )
            .into_response(),
    }
}

pub async fn handle_call_capability(
    State(state): State<AppState>,
    Json(payload): Json<CallCapabilityRequest>,
) -> impl IntoResponse {
    let trace_id = next_trace_id();
    if !payload.args.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "INVALID_ARGS",
                "'args' must be a JSON object",
                false,
            )),
        )
            .into_response();
    }

    if !state.policy.allows(&payload.capability_id) {
        return (
            StatusCode::FORBIDDEN,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "INVALID_ARGS",
                format!(
                    "Capability '{}' blocked by policy",
                    payload.capability_id
                ),
                false,
            )),
        )
            .into_response();
    }

    let Some(meta) = state.capabilities.get(&payload.capability_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "TOOL_NOT_FOUND",
                format!("Capability '{}' not found", payload.capability_id),
                false,
            )),
        )
            .into_response();
    };

    let Some(tx) = state.servers.get(&meta.server) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                payload.request_id,
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unreachable", meta.server),
                true,
            )),
        )
            .into_response();
    };

    let request_id = payload.request_id;
    let redacted_input = redact_value(payload.args.clone(), &state.policy.redact_keys);
    info!(
        trace_id = %trace_id,
        capability_id = %payload.capability_id,
        args = %redacted_input,
        "tool call start"
    );

    let (reply_tx, reply_rx) = oneshot::channel();
    if tx
        .send(ServerMsg::CallTool {
            name: meta.tool.clone(),
            params: payload.args,
            reply: reply_tx,
        })
        .await
        .is_err()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(error_envelope(
                trace_id,
                request_id,
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", meta.server),
                true,
            )),
        )
            .into_response();
    }

    match reply_rx.await {
        Ok(Ok(data)) => {
            let redacted_output = redact_value(data.clone(), &state.policy.redact_keys);
            info!(
                trace_id = %trace_id,
                capability_id = %payload.capability_id,
                data = %redacted_output,
                "tool call success"
            );
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "request_id": request_id,
                    "trace_id": trace_id,
                    "data": data,
                    "error": null,
                })),
            )
                .into_response()
        }
        Ok(Err(UpstreamCallError::Timeout)) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(error_envelope(
                trace_id,
                request_id,
                "UPSTREAM_TIMEOUT",
                format!("Tool call timed out after {}ms", state.tool_timeout_ms),
                true,
            )),
        )
            .into_response(),
        Ok(Err(UpstreamCallError::Upstream(err))) => (
            StatusCode::BAD_GATEWAY,
            Json(error_envelope(
                trace_id,
                request_id,
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
                request_id,
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        )
            .into_response(),
    }
}

fn next_trace_id() -> String {
    format!("trace-{}", TRACE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn error_envelope(
    trace_id: String,
    request_id: Option<String>,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> Value {
    json!({
        "ok": false,
        "request_id": request_id,
        "trace_id": trace_id,
        "data": null,
        "error": {
            "code": code,
            "message": message.into(),
            "retryable": retryable,
        }
    })
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

#[cfg(test)]
mod tests {
    use super::{
        handle_catalog_events, handle_get_prompt, handle_list_capabilities, handle_list_prompts,
        handle_list_resources, handle_read_resource, redact_value, AppState, CatalogEventsQuery,
        GetPromptRequest, ReadResourceRequest,
    };
    use crate::daemon::{CapabilityMeta, Policy, PromptMeta, ResourceMeta};
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

        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(resources),
            prompts: Arc::new(HashMap::new()),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:test".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

        let response = handle_list_resources(State(state), HeaderMap::new()).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body");
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
        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(HashMap::new()),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:test".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

        let response = handle_read_resource(
            State(state),
            Json(ReadResourceRequest {
                resource_id: "missing.resource".to_string(),
                request_id: None,
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

        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(prompts),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:test".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

        let response = handle_list_prompts(State(state), HeaderMap::new()).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body");
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
        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(HashMap::new()),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:test".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

        let response = handle_get_prompt(
            State(state),
            Json(GetPromptRequest {
                prompt_id: "missing.prompt".to_string(),
                arguments: None,
                request_id: None,
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

        let state = AppState {
            servers: Arc::new(servers),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(prompts),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:test".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

        let response = handle_get_prompt(
            State(state),
            Json(GetPromptRequest {
                prompt_id: "alpha.prompt".to_string(),
                arguments: Some(json!("not-an-object")),
                request_id: None,
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

        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(capabilities),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(HashMap::new()),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:test_cat".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

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
        let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body");
        let res: Value = serde_json::from_slice(&bytes).expect("json");

        assert_eq!(res["version"], "v1");
        assert_eq!(res["catalog_version"], "sha256:test_cat");
        let caps = res["capabilities"].as_array().expect("capabilities array");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0]["id"], "github.issues.search");
    }

    #[tokio::test]
    async fn if_none_match_returns_304_not_modified() {
        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(HashMap::new()),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:abc1234".to_string(),
            event_store: Arc::new(crate::catalog::CatalogEventStore::new()),
        };

        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, HeaderValue::from_static("\"sha256:abc1234\""));

        let response = handle_list_capabilities(State(state), headers).await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(
            response.headers().get(header::ETAG).unwrap().to_str().unwrap(),
            "\"sha256:abc1234\""
        );
    }

    #[tokio::test]
    async fn catalog_events_endpoint_returns_event_feed() {
        let event_store = Arc::new(crate::catalog::CatalogEventStore::new());
        event_store.record("capability", "test.tool", "added");

        let state = AppState {
            servers: Arc::new(HashMap::new()),
            capabilities: Arc::new(HashMap::new()),
            resources: Arc::new(HashMap::new()),
            prompts: Arc::new(HashMap::new()),
            tool_timeout_ms: 1000,
            policy: Policy::default(),
            search_engine: Arc::new(crate::search::HybridSearchEngine::new()),
            catalog_version: "sha256:v1".to_string(),
            event_store,
        };

        let response = handle_catalog_events(
            State(state),
            Query(CatalogEventsQuery { after: None }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body");
        let payload: Value = serde_json::from_slice(&bytes).expect("json");

        assert_eq!(payload["catalog_version"], "sha256:v1");
        assert_eq!(payload["cursor"], "evt_1");
        let events = payload["events"].as_array().expect("events array");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["object_id"], "test.tool");
    }
}
