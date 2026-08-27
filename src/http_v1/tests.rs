// Rust guideline compliant 2026-08-27

//! Comprehensive unit and integration test suite for HTTP v1 facade API handlers.

use crate::{
    daemon::{AppState, CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg},
    http_v1::{
        approvals_api::{handle_approve_ticket, handle_list_approvals, handle_reject_ticket},
        catalog::{
            handle_catalog_events, handle_completion, handle_get_prompt, handle_list_capabilities,
            handle_list_prompts, handle_list_resources, handle_list_sampling_requests,
            handle_read_resource, handle_respond_sampling_request, handle_sampling_create_message,
            handle_search_capabilities,
        },
        config_api::{
            handle_delete_profile, handle_get_config, handle_get_ecosystem_sources,
            handle_reload_config, handle_upsert_profile, handle_upsert_server,
        },
        execute::handle_call_capability,
        helpers::redact_value,
        types::{
            ApproveTicketRequest, CallCapabilityRequest, CatalogEventsQuery, CompletionRequest,
            GetPromptRequest, ReadResourceRequest, RejectTicketRequest, RespondSamplingRequest,
            SamplingListQuery, SamplingRequest, SearchCapabilitiesRequest, UpsertProfileRequest,
            UpsertServerRequest,
        },
        ui::handle_ui_dashboard,
    },
};
use axum::extract::Path;
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
async fn test_sampling_async_delegation_flow() {
    let mut servers = HashMap::new();
    let (tx, _rx) = mpsc::channel(1);
    servers.insert("srv".to_string(), tx);

    let state = AppState::builder().servers(servers).build();

    let req = SamplingRequest {
        server_id: "srv".to_string(),
        messages: vec![crate::sampling::SamplingMessage {
            role: "user".to_string(),
            content: crate::sampling::SamplingContent::Text {
                text: "Analyze logs".to_string(),
            },
        }],
        model_preferences: None,
        system_prompt: None,
        include_context: None,
        max_tokens: Some(100),
        stop_sequences: vec![],
        metadata: None,
        async_mode: Some(true),
    };

    // 1. Submit async sampling request
    let response = handle_sampling_create_message(State(state.clone()), Json(req))
        .await
        .into_response();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // 2. List pending sampling requests
    let list_resp = handle_list_sampling_requests(
        State(state.clone()),
        Query(SamplingListQuery {
            server_id: Some("srv".to_string()),
            status: Some("pending".to_string()),
        }),
    )
    .await
    .into_response();
    assert_eq!(list_resp.status(), StatusCode::OK);

    let all_requests = state
        .sampling_registry
        .list_requests(Some("srv"), Some("pending"))
        .await;
    assert_eq!(all_requests.len(), 1);
    let ticket_id = all_requests[0].id.clone();

    // 3. Respond to sampling request
    let respond_req = RespondSamplingRequest {
        result: crate::sampling::CreateMessageResult {
            role: "assistant".to_string(),
            content: crate::sampling::SamplingContent::Text {
                text: "Log analysis complete: no errors found.".to_string(),
            },
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: Some("endTurn".to_string()),
        },
    };

    let respond_resp = handle_respond_sampling_request(
        State(state.clone()),
        Path(ticket_id.clone()),
        Json(respond_req),
    )
    .await
    .into_response();
    assert_eq!(respond_resp.status(), StatusCode::OK);

    // 4. Verify ticket is now completed
    let ticket = state
        .sampling_registry
        .get_request(&ticket_id)
        .await
        .unwrap();
    assert!(matches!(
        ticket.status,
        crate::sampling::SamplingRequestStatus::Completed { .. }
    ));
}

#[tokio::test]
async fn test_sampling_sync_long_poll_flow() {
    let mut servers = HashMap::new();
    let (tx, _rx) = mpsc::channel(1);
    servers.insert("srv".to_string(), tx);

    let state = AppState::builder().servers(servers).build();

    let req = SamplingRequest {
        server_id: "srv".to_string(),
        messages: vec![crate::sampling::SamplingMessage {
            role: "user".to_string(),
            content: crate::sampling::SamplingContent::Text {
                text: "Quick summary".to_string(),
            },
        }],
        model_preferences: None,
        system_prompt: None,
        include_context: None,
        max_tokens: Some(50),
        stop_sequences: vec![],
        metadata: None,
        async_mode: Some(false),
    };

    let state_for_spawn = state.clone();
    let spawn_handle = tokio::spawn(async move {
        handle_sampling_create_message(State(state_for_spawn), Json(req))
            .await
            .into_response()
    });

    // Wait briefly for ticket to be registered
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let pending = state
        .sampling_registry
        .list_requests(Some("srv"), Some("pending"))
        .await;
    assert_eq!(pending.len(), 1);
    let ticket_id = pending[0].id.clone();

    // Complete ticket
    let respond_req = RespondSamplingRequest {
        result: crate::sampling::CreateMessageResult {
            role: "assistant".to_string(),
            content: crate::sampling::SamplingContent::Text {
                text: "Done summary.".to_string(),
            },
            model: "gpt-4o".to_string(),
            stop_reason: Some("endTurn".to_string()),
        },
    };
    state
        .sampling_registry
        .respond_to_request(&ticket_id, respond_req.result)
        .await;

    let response = spawn_handle.await.unwrap();
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

    let response = handle_list_resources(
        State(state),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
    )
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
        None,
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
async fn get_prompt_returns_not_found_code() {
    let state = AppState::builder().catalog_version("sha256:test").build();

    let response = handle_get_prompt(
        State(state),
        None,
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

    let response = handle_list_prompts(
        State(state),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
    )
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
        None,
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
        axum::extract::Extension(None),
        None,
        Json(SearchCapabilitiesRequest {
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

    let response =
        handle_list_capabilities(State(state), axum::extract::Extension(None), None, headers)
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

    let response = handle_catalog_events(State(state), Query(CatalogEventsQuery { after: None }))
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
        None,
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

    let response = handle_call_capability(
        State(state),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
        Json(req),
    )
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

    let response = handle_read_resource(State(state), None, HeaderMap::new(), Json(req))
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
    let ui_res = handle_ui_dashboard().await.into_response();
    assert_eq!(ui_res.status(), StatusCode::OK);
    assert_eq!(
        ui_res.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    // 2. Test Get Config (empty/default)
    let get_cfg_res = handle_get_config(State(state.clone()))
        .await
        .into_response();
    assert_eq!(get_cfg_res.status(), StatusCode::OK);

    // 3. Test Upsert Server
    let new_server = crate::config::ServerConfig {
        command: Some("node".to_string()),
        args: vec!["index.js".to_string()],
        ..Default::default()
    };

    let upsert_res = handle_upsert_server(
        State(state.clone()),
        Json(UpsertServerRequest {
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
    let eco_res = handle_get_ecosystem_sources().await.into_response();
    assert_eq!(eco_res.status(), StatusCode::OK);

    // 5. Test Reload Config handler
    let reload_res = handle_reload_config(State(state.clone()))
        .await
        .into_response();
    assert_eq!(reload_res.status(), StatusCode::OK);

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[tokio::test]
async fn test_get_config_sanitizes_secrets() {
    use crate::config::{
        AuditConfig, AuthConfig, McpConfig, PolicyConfig, ServerConfig, SiemConfig,
        SiemTargetConfig, WebhookConfig,
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let cfg_path = temp_dir.path().join("secure_servers.json");
    let cfg_str = cfg_path.to_str().unwrap().to_string();

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "remote_api".to_string(),
        ServerConfig {
            url: Some("https://api.example.com".to_string()),
            auth: Some(AuthConfig::Bearer {
                token: Some("super_secret_token_123".to_string()),
                token_env: None,
            }),
            headers: HashMap::from([(
                "Authorization".to_string(),
                "Bearer secret_header".to_string(),
            )]),
            ..Default::default()
        },
    );

    let config = McpConfig {
        mcp_servers: mcp_servers.clone(),
        policy: Some(PolicyConfig {
            webhook: Some(WebhookConfig {
                url: "https://hooks.slack.com".to_string(),
                secret: Some("hmac_secret_999".to_string()),
                auth_header: Some("Bearer hook_auth".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        audit: Some(AuditConfig {
            enabled: true,
            siem: Some(SiemConfig {
                targets: vec![SiemTargetConfig::SplunkHec {
                    url: "https://splunk.corp:8088".to_string(),
                    token: "splunk_token_abc".to_string(),
                    index: None,
                    source: None,
                }],
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    crate::config::save_config(&cfg_str, &config).unwrap();

    let state = AppState::builder()
        .config_path(&cfg_str)
        .server_configs(mcp_servers)
        .build();

    let res = handle_get_config(State(state)).await.into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();

    // Verify secrets are masked
    let cfg_json = payload["config"].to_string();
    assert!(!cfg_json.contains("super_secret_token_123"));
    assert!(!cfg_json.contains("hmac_secret_999"));
    assert!(!cfg_json.contains("hook_auth"));
    assert!(!cfg_json.contains("splunk_token_abc"));
    assert!(!cfg_json.contains("secret_header"));
    assert!(cfg_json.contains("********"));

    let srv_json = payload["server_configs"].to_string();
    assert!(!srv_json.contains("super_secret_token_123"));
    assert!(!srv_json.contains("secret_header"));
    assert!(srv_json.contains("********"));
}

#[tokio::test]
async fn test_hitl_approval_flow_and_endpoints() {
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "docker.run".to_string(),
        CapabilityMeta {
            server: "docker_srv".to_string(),
            tool: "run".to_string(),
            summary: "Run docker container".to_string(),
            description: "Run docker container".to_string(),
            input_schema: json!({}),
            tags: vec![],
            examples: vec![],
        },
    );

    let (tx, mut rx) = mpsc::channel(1);
    let mut servers = HashMap::new();
    servers.insert("docker_srv".to_string(), tx);

    let policy = crate::daemon::Policy {
        allow: vec!["*".to_string()],
        deny: vec![],
        redact_keys: vec![],
        require_approval: vec!["docker.run*".to_string()],
        approval_timeout_secs: 10,
        webhook: None,
    };

    let state = AppState::builder()
        .capabilities(capabilities)
        .servers(servers)
        .policy(policy)
        .build();

    // 1. Test Prefer: respond-async returns 202 Accepted with approval ticket
    let mut headers = HeaderMap::new();
    headers.insert("prefer", "respond-async".parse().unwrap());

    let req = CallCapabilityRequest {
        capability_id: "docker.run".to_string(),
        args: json!({"image": "ubuntu", "command": "rm -rf /"}),
        request_id: Some("req-hitl-async".to_string()),
        context: None,
        idempotency_key: None,
        input_responses: None,
        request_state: None,
        async_task: false,
    };

    let async_res = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        headers,
        Json(req),
    )
    .await
    .into_response();

    assert_eq!(async_res.status(), StatusCode::ACCEPTED);
    let bytes = to_bytes(async_res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["status"], "pending_approval");
    let approval_id = payload["approval_id"].as_str().unwrap().to_string();

    // 2. Test List Approvals contains our ticket
    let list_res = handle_list_approvals(State(state.clone()))
        .await
        .into_response();
    assert_eq!(list_res.status(), StatusCode::OK);
    let list_bytes = to_bytes(list_res.into_body(), usize::MAX).await.unwrap();
    let list_payload: Value = serde_json::from_slice(&list_bytes).unwrap();
    let apprs = list_payload["approvals"].as_array().unwrap();
    assert_eq!(apprs.len(), 1);
    assert_eq!(apprs[0]["id"], approval_id.as_str());

    // 3. Test Synchronous wait channel resolution with modified arguments
    let state_clone = state.clone();
    let approval_id_clone = approval_id.clone();

    tokio::spawn(async move {
        // Wait for receiver to register
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Operator approves ticket with modified arguments
        let appr_res = handle_approve_ticket(
            State(state_clone.clone()),
            axum::extract::Path(approval_id_clone),
            Json(ApproveTicketRequest {
                operator: "sec-admin".to_string(),
                modified_args: Some(json!({"image": "ubuntu", "command": "echo safe"})),
            }),
        )
        .await
        .into_response();
        assert_eq!(appr_res.status(), StatusCode::OK);
    });

    tokio::spawn(async move {
        if let Some(ServerMsg::CallTool { params, reply, .. }) = rx.recv().await {
            // Verify upstream received operator-modified arguments
            assert_eq!(params["command"], "echo safe");
            let _ = reply.send(Ok(json!({"stdout": "safe execution"})));
        }
    });

    // Execute synchronous call (should suspend and resolve when approved)
    let sync_req = CallCapabilityRequest {
        capability_id: "docker.run".to_string(),
        args: json!({"image": "ubuntu", "command": "rm -rf /"}),
        request_id: Some("req-hitl-sync".to_string()),
        context: None,
        idempotency_key: None,
        input_responses: None,
        request_state: None,
        async_task: false,
    };

    // Note: this creates a second approval ticket since it's a new call
    // Spawn an operator to approve this second ticket as well
    let state_for_sync = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let list = state_for_sync.approval_registry.list().await;
        if let Some(second_ticket) = list.iter().find(|t| t.id != approval_id) {
            let _ = handle_approve_ticket(
                State(state_for_sync.clone()),
                axum::extract::Path(second_ticket.id.clone()),
                Json(ApproveTicketRequest {
                    operator: "sec-admin".to_string(),
                    modified_args: Some(json!({"image": "ubuntu", "command": "echo safe"})),
                }),
            )
            .await;
        }
    });

    let sync_res = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
        Json(sync_req),
    )
    .await
    .into_response();

    assert_eq!(sync_res.status(), StatusCode::OK);
    let sync_bytes = to_bytes(sync_res.into_body(), usize::MAX).await.unwrap();
    let sync_payload: Value = serde_json::from_slice(&sync_bytes).unwrap();
    assert_eq!(sync_payload["ok"], true);
    assert_eq!(sync_payload["data"]["stdout"], "safe execution");
}

#[tokio::test]
async fn test_hitl_rejection_returns_structured_envelope() {
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "db.drop_table".to_string(),
        CapabilityMeta {
            server: "db_srv".to_string(),
            tool: "drop_table".to_string(),
            summary: "Drop database table".to_string(),
            description: "Drop table".to_string(),
            input_schema: json!({}),
            tags: vec![],
            examples: vec![],
        },
    );

    let (tx, _rx) = mpsc::channel(1);
    let mut servers = HashMap::new();
    servers.insert("db_srv".to_string(), tx);

    let policy = crate::daemon::Policy {
        allow: vec!["*".to_string()],
        deny: vec![],
        redact_keys: vec![],
        require_approval: vec!["db.drop*".to_string()],
        approval_timeout_secs: 5,
        webhook: None,
    };

    let state = AppState::builder()
        .capabilities(capabilities)
        .servers(servers)
        .policy(policy)
        .build();

    let state_clone = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        let list = state_clone.approval_registry.list().await;
        if let Some(ticket) = list.first() {
            let _ = handle_reject_ticket(
                State(state_clone.clone()),
                axum::extract::Path(ticket.id.clone()),
                Json(RejectTicketRequest {
                    operator: "lead-dba".to_string(),
                    reason: Some("Cannot drop production table".to_string()),
                }),
            )
            .await;
        }
    });

    let req = CallCapabilityRequest {
        capability_id: "db.drop_table".to_string(),
        args: json!({"table": "users"}),
        request_id: Some("req-reject-test".to_string()),
        context: None,
        idempotency_key: None,
        input_responses: None,
        request_state: None,
        async_task: false,
    };

    let res = handle_call_capability(
        State(state),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
        Json(req),
    )
    .await
    .into_response();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "OPERATION_REJECTED_BY_OPERATOR");
    assert!(payload["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Cannot drop production table"));
    assert_eq!(payload["error"]["operator"], "lead-dba");
}

#[tokio::test]
async fn test_tool_call_emits_audit_events_and_hash_chain() {
    let (server_tx, mut server_rx) = mpsc::channel(16);
    let mut servers = HashMap::new();
    servers.insert("srv".to_string(), server_tx);

    let mut capabilities = HashMap::new();
    capabilities.insert(
        "srv.echo".to_string(),
        CapabilityMeta {
            server: "srv".to_string(),
            tool: "echo".to_string(),
            summary: "Echo test".to_string(),
            description: "Echo test description".to_string(),
            tags: vec![],
            input_schema: Value::Null,
            examples: vec![],
        },
    );

    let state = AppState::builder()
        .servers(servers)
        .capabilities(capabilities)
        .build();

    tokio::spawn(async move {
        while let Some(msg) = server_rx.recv().await {
            if let ServerMsg::CallTool { reply, params, .. } = msg {
                let _ = reply.send(Ok(params));
            }
        }
    });

    let req = CallCapabilityRequest {
        capability_id: "srv.echo".to_string(),
        args: json!({"message": "hello audit", "secret": "shhh"}),
        request_id: Some("req-audit-test".to_string()),
        context: None,
        idempotency_key: None,
        input_responses: None,
        request_state: None,
        async_task: false,
    };

    let res = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
        Json(req),
    )
    .await
    .into_response();

    assert_eq!(res.status(), StatusCode::OK);

    // Allow background flusher to process event
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let (events, total) = state
        .audit_store
        .query(&crate::audit::AuditQueryFilter {
            capability_id: Some("srv.echo".to_string()),
            ..Default::default()
        })
        .await;

    assert_eq!(total, 1);
    assert_eq!(events[0].capability_id.as_deref(), Some("srv.echo"));
    assert_eq!(events[0].status, crate::audit::AuditEventStatus::Success);

    let report = state.audit_store.verify_chain().await;
    assert!(report.is_valid);
}

#[tokio::test]
async fn test_audit_api_endpoints_and_export() {
    let store = Arc::new(crate::audit::AuditStore::in_memory());
    let handle = crate::audit::spawn_audit_worker(store.clone(), None, 100, 50, 10);
    let state = AppState::builder()
        .audit_store(store.clone())
        .audit_handle(handle.clone())
        .build();

    handle
        .send_async(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ToolExecution,
            trace_id: "trace-audit-api".to_string(),
            request_id: Some("req-100".to_string()),
            actor_id: Some("agent-compliance".to_string()),
            work_item_id: None,
            client_ip: Some("127.0.0.1".to_string()),
            server_id: Some("github".to_string()),
            capability_id: Some("github.get_repo".to_string()),
            resource_uri: None,
            sanitized_args: Some(json!({"repo": "warmplane"})),
            sanitized_response: Some(json!({"stars": 500})),
            execution_latency_us: Some(1200),
            status: crate::audit::AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: None,
        })
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // Test GET /v1/audit/events
    let list_res = crate::http_v1::audit_api::handle_list_audit_events(
        State(state.clone()),
        Query(crate::http_v1::audit_api::AuditEventsQuery {
            actor_id: Some("agent-compliance".to_string()),
            ..Default::default()
        }),
    )
    .await
    .into_response();

    assert_eq!(list_res.status(), StatusCode::OK);
    let bytes = to_bytes(list_res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["total"], 1);
    let event_id = payload["events"][0]["id"].as_str().unwrap().to_string();

    // Test GET /v1/audit/events/:id
    let get_res = crate::http_v1::audit_api::handle_get_audit_event(
        State(state.clone()),
        axum::extract::Path(event_id.clone()),
    )
    .await
    .into_response();

    assert_eq!(get_res.status(), StatusCode::OK);

    // Test GET /v1/audit/verify
    let verify_res = crate::http_v1::audit_api::handle_verify_audit_chain(State(state.clone()))
        .await
        .into_response();

    assert_eq!(verify_res.status(), StatusCode::OK);
    let verify_bytes = to_bytes(verify_res.into_body(), usize::MAX).await.unwrap();
    let verify_json: Value = serde_json::from_slice(&verify_bytes).unwrap();
    assert_eq!(verify_json["ok"], true);
    assert_eq!(verify_json["report"]["is_valid"], true);

    // Test GET /v1/audit/stats
    let stats_res = crate::http_v1::audit_api::handle_get_audit_stats(State(state.clone()))
        .await
        .into_response();

    assert_eq!(stats_res.status(), StatusCode::OK);

    // Test GET /v1/audit/export as CSV
    let export_csv = crate::http_v1::audit_api::handle_export_audit(
        State(state.clone()),
        Query(crate::http_v1::audit_api::AuditExportQuery {
            format: Some("csv".to_string()),
            ..Default::default()
        }),
    )
    .await
    .into_response();

    assert_eq!(export_csv.status(), StatusCode::OK);
    let csv_bytes = to_bytes(export_csv.into_body(), usize::MAX).await.unwrap();
    let csv_str = String::from_utf8(csv_bytes.to_vec()).unwrap();
    assert!(csv_str.contains("github.get_repo"));
    assert!(csv_str.contains("agent-compliance"));

    // Test GET /v1/audit/events with status & search filtering
    let search_res = crate::http_v1::audit_api::handle_list_audit_events(
        State(state.clone()),
        Query(crate::http_v1::audit_api::AuditEventsQuery {
            status: Some("success".to_string()),
            server_id: Some("github".to_string()),
            search: Some("get_repo".to_string()),
            limit: Some(10),
            offset: Some(0),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(search_res.status(), StatusCode::OK);
    let s_bytes = to_bytes(search_res.into_body(), usize::MAX).await.unwrap();
    let s_payload: Value = serde_json::from_slice(&s_bytes).unwrap();
    assert_eq!(s_payload["ok"], true);
    assert_eq!(s_payload["total"], 1);

    // Test GET /v1/audit/events with non-matching status
    let nomatch_res = crate::http_v1::audit_api::handle_list_audit_events(
        State(state.clone()),
        Query(crate::http_v1::audit_api::AuditEventsQuery {
            status: Some("denied".to_string()),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(nomatch_res.status(), StatusCode::OK);
    let nm_bytes = to_bytes(nomatch_res.into_body(), usize::MAX).await.unwrap();
    let nm_payload: Value = serde_json::from_slice(&nm_bytes).unwrap();
    assert_eq!(nm_payload["ok"], true);
    assert_eq!(nm_payload["total"], 0);
}

#[tokio::test]
async fn test_circuit_breaker_fast_fail_and_recovery() {
    let (tx, mut rx) = mpsc::channel(1);
    let mut servers = HashMap::new();
    servers.insert("flaky_srv".to_string(), tx);

    let mut caps = HashMap::new();
    caps.insert(
        "flaky.error".to_string(),
        CapabilityMeta {
            server: "flaky_srv".to_string(),
            tool: "error".to_string(),
            summary: "Simulated error tool".to_string(),
            description: "Simulated error tool".to_string(),
            input_schema: json!({}),
            examples: vec![],
            tags: vec![],
        },
    );

    let cb_registry = crate::circuit_breaker::CircuitBreakerRegistry::default();
    cb_registry
        .get_or_create(
            "flaky_srv",
            crate::circuit_breaker::ResilienceConfig {
                failure_threshold: 2,
                cooldown_ms: 100,
                consecutive_successes: 1,
                ..Default::default()
            },
        )
        .await;

    let state = AppState::builder()
        .servers(servers)
        .capabilities(caps)
        .circuit_breakers(cb_registry)
        .build();

    // Spawn mock server responder that always errors
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let ServerMsg::CallTool { reply, .. } = msg {
                let _ = reply.send(Err(crate::daemon::types::UpstreamCallError::Upstream(
                    "500 Internal Error".to_string(),
                )));
            }
        }
    });

    let headers = HeaderMap::new();

    // Call 1 -> UPSTREAM_ERROR
    let res1 = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        headers.clone(),
        Json(CallCapabilityRequest {
            capability_id: "flaky.error".to_string(),
            args: json!({}),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(res1.status(), StatusCode::BAD_GATEWAY);

    // Call 2 -> UPSTREAM_ERROR, trips circuit to OPEN
    let res2 = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        headers.clone(),
        Json(CallCapabilityRequest {
            capability_id: "flaky.error".to_string(),
            args: json!({}),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(res2.status(), StatusCode::BAD_GATEWAY);

    // Call 3 -> Fast fail with CIRCUIT_OPEN (no call forwarded)
    let res3 = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        headers.clone(),
        Json(CallCapabilityRequest {
            capability_id: "flaky.error".to_string(),
            args: json!({}),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(res3.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = to_bytes(res3.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "CIRCUIT_OPEN");
    assert_eq!(payload["retry"]["upstream_execution_state"], "not_started");
}

#[tokio::test]
async fn test_ui_config_profiles_crud() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let config_path = tmp.path().to_str().unwrap().to_string();

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "s1".to_string(),
        crate::config::ServerConfig {
            command: Some("echo".to_string()),
            args: vec![],
            env: HashMap::new(),
            url: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            auth: None,
            resilience: None,
        },
    );

    let initial_cfg = crate::config::McpConfig {
        mcp_servers,
        ..Default::default()
    };
    crate::config::save_config(&config_path, &initial_cfg).unwrap();

    let profiles = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    let state = AppState::builder()
        .config_path(config_path.clone())
        .profiles_arc(profiles.clone())
        .build();

    // 1. Create profile "dev"
    let create_res = handle_upsert_profile(
        State(state.clone()),
        Json(UpsertProfileRequest {
            name: "dev".to_string(),
            servers: vec!["s1".to_string()],
            description: Some("Developer tools".to_string()),
            policy: None,
        }),
    )
    .await
    .into_response();
    assert_eq!(create_res.status(), StatusCode::OK);

    // Verify in-memory state and disk
    {
        let prof_guard = profiles.read().await;
        assert!(prof_guard.contains_key("dev"));
        assert_eq!(prof_guard["dev"].servers, vec!["s1".to_string()]);
    }
    let loaded = crate::config::load_or_default_config(&config_path).unwrap();
    assert!(loaded.profiles.contains_key("dev"));

    // 2. Reject unknown server
    let reject_res = handle_upsert_profile(
        State(state.clone()),
        Json(UpsertProfileRequest {
            name: "invalid".to_string(),
            servers: vec!["nonexistent_srv".to_string()],
            description: None,
            policy: None,
        }),
    )
    .await
    .into_response();
    assert_eq!(reject_res.status(), StatusCode::BAD_REQUEST);

    // 3. Delete profile "dev"
    let del_res =
        handle_delete_profile(State(state.clone()), axum::extract::Path("dev".to_string()))
            .await
            .into_response();
    assert_eq!(del_res.status(), StatusCode::OK);

    {
        let prof_guard = profiles.read().await;
        assert!(!prof_guard.contains_key("dev"));
    }
    let loaded_after = crate::config::load_or_default_config(&config_path).unwrap();
    assert!(!loaded_after.profiles.contains_key("dev"));
}

#[tokio::test]
async fn test_task_update_bridges_to_approval_resolution() {
    let state = AppState::builder().build();

    // 1. Create a pending approval ticket
    let (appr_id, _rx) = state
        .approval_registry
        .create_approval(crate::approvals::CreateApprovalRequest {
            capability_id: "sqlite.list_tables".to_string(),
            server_id: "sqlite".to_string(),
            args: json!({}),
            sanitized_args: json!({}),
            request_id: Some("req-test-bridge-1".to_string()),
            context: None,
            timeout_secs: 60,
            webhook: None,
        })
        .await;

    // 2. Create matching Task with the same request_id
    let mut input_requests = std::collections::BTreeMap::new();
    input_requests.insert(
        "hitl_approval".to_string(),
        json!({
            "type": "approval_review",
            "capability_id": "sqlite.list_tables",
            "server_id": "sqlite",
            "sanitized_args": {},
            "timeout_secs": 60,
        }),
    );

    let (task_record, _task_rx) = state
        .task_registry
        .create_task(crate::tasks::CreateTaskParams {
            capability_id: "sqlite.list_tables".to_string(),
            server_id: "sqlite".to_string(),
            args: json!({}),
            request_id: Some("req-test-bridge-1".to_string()),
            context: None,
            idempotency_key: None,
            initial_status: crate::tasks::TaskStatus::InputRequired,
            status_message: Some("Awaiting approval".to_string()),
            input_requests: Some(input_requests),
            ttl_ms: Some(60_000),
            poll_interval_ms: Some(1000),
        })
        .await;

    // Verify both are pending
    let ticket_before = state.approval_registry.get(&appr_id).await.unwrap();
    assert_eq!(
        ticket_before.status,
        crate::approvals::ApprovalStatus::Pending
    );

    // 3. Submit Task update via handle_update_task
    let mut responses = std::collections::BTreeMap::new();
    responses.insert(
        "hitl_approval".to_string(),
        json!({
            "approved": true,
            "operator": "test-admin",
            "modified_args": { "table": "users" }
        }),
    );

    let update_res = crate::http_v1::tasks_api::handle_update_task(
        State(state.clone()),
        Path(task_record.task_id.clone()),
        Json(crate::http_v1::tasks_api::UpdateTaskRequest {
            input_responses: responses,
        }),
    )
    .await
    .into_response();

    assert_eq!(update_res.status(), StatusCode::OK);

    // 4. Verify ApprovalRegistry ticket is now Approved with modified args
    let ticket_after = state.approval_registry.get(&appr_id).await.unwrap();
    match ticket_after.status {
        crate::approvals::ApprovalStatus::Approved {
            operator,
            modified_args,
            ..
        } => {
            assert_eq!(operator, "test-admin");
            assert_eq!(modified_args, Some(json!({ "table": "users" })));
        }
        _ => panic!(
            "Expected ticket to be approved, found {:?}",
            ticket_after.status
        ),
    }
}

#[tokio::test]
async fn test_catalog_policy_hidden_counts() {
    let policy = crate::daemon::Policy {
        allow: vec!["sqlite.*".to_string()],
        ..Default::default()
    };

    let mut capabilities = HashMap::new();
    capabilities.insert(
        "sqlite.query".to_string(),
        CapabilityMeta {
            summary: "query".to_string(),
            description: "query".to_string(),
            server: "sqlite".to_string(),
            tool: "query".to_string(),
            tags: vec![],
            input_schema: serde_json::Value::Null,
            examples: vec![],
        },
    );
    capabilities.insert(
        "filesystem.read".to_string(),
        CapabilityMeta {
            summary: "read".to_string(),
            description: "read".to_string(),
            server: "filesystem".to_string(),
            tool: "read".to_string(),
            tags: vec![],
            input_schema: serde_json::Value::Null,
            examples: vec![],
        },
    );

    let state = AppState::builder()
        .policy(policy)
        .capabilities(capabilities)
        .build();

    let res = crate::http_v1::catalog::handle_list_capabilities(
        State(state),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
    )
    .await
    .into_response();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let val: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(val["total_unfiltered"], 2);
    assert_eq!(val["hidden_by_policy"], 1);
    assert_eq!(val["capabilities"].as_array().unwrap().len(), 1);
    assert_eq!(val["capabilities"][0]["id"], "sqlite.query");
}

#[tokio::test]
async fn test_per_profile_policy_filtering_and_execution() {
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "sqlite.read".to_string(),
        CapabilityMeta {
            summary: "read".to_string(),
            description: "read".to_string(),
            server: "sqlite".to_string(),
            tool: "read".to_string(),
            tags: vec![],
            input_schema: serde_json::Value::Null,
            examples: vec![],
        },
    );
    capabilities.insert(
        "sqlite.delete".to_string(),
        CapabilityMeta {
            summary: "delete".to_string(),
            description: "delete".to_string(),
            server: "sqlite".to_string(),
            tool: "delete".to_string(),
            tags: vec![],
            input_schema: serde_json::Value::Null,
            examples: vec![],
        },
    );

    // Profile with deny rule on sqlite.delete
    let mut profile_policy = crate::daemon::Policy::default();
    profile_policy.deny.push("sqlite.delete".to_string());

    let prof_ctx = crate::context::ProfileContext::scoped_with_policy(
        "readonly",
        vec!["sqlite".to_string()],
        Some(profile_policy),
    );

    let state = AppState::builder().capabilities(capabilities).build();

    // 1. Catalog listing with profile policy
    let res = crate::http_v1::catalog::handle_list_capabilities(
        State(state.clone()),
        axum::extract::Extension(None),
        Some(axum::extract::Extension(prof_ctx.clone())),
        HeaderMap::new(),
    )
    .await
    .into_response();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let val: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(val["total_unfiltered"], 2);
    assert_eq!(val["hidden_by_policy"], 1);
    assert_eq!(val["capabilities"].as_array().unwrap().len(), 1);
    assert_eq!(val["capabilities"][0]["id"], "sqlite.read");

    // 2. Call denied capability under profile policy
    let exec_res = crate::http_v1::execute::handle_call_capability(
        State(state),
        axum::extract::Extension(None),
        Some(axum::extract::Extension(prof_ctx)),
        HeaderMap::new(),
        Json(CallCapabilityRequest {
            capability_id: "sqlite.delete".to_string(),
            args: json!({}),
            request_id: None,
            context: None,
            idempotency_key: None,
            input_responses: None,
            request_state: None,
            async_task: false,
        }),
    )
    .await
    .into_response();

    assert_eq!(exec_res.status(), StatusCode::FORBIDDEN);
    let exec_bytes = to_bytes(exec_res.into_body(), usize::MAX).await.unwrap();
    let exec_val: Value = serde_json::from_slice(&exec_bytes).unwrap();
    assert_eq!(exec_val["error"]["code"], "POLICY_DENIED");
}
