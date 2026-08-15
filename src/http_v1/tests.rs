// Rust guideline compliant 2026-08-15

//! Comprehensive unit and integration test suite for HTTP v1 facade API handlers.

#[cfg(test)]
mod tests {
    use crate::{
        daemon::{AppState, CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg},
        http_v1::{
            approvals_api::{handle_approve_ticket, handle_list_approvals, handle_reject_ticket},
            catalog::{
                handle_catalog_events, handle_completion, handle_get_prompt,
                handle_list_capabilities, handle_list_prompts, handle_list_resources,
                handle_read_resource, handle_sampling_create_message, handle_search_capabilities,
            },
            config_api::{
                handle_get_config, handle_get_ecosystem_sources, handle_reload_config,
                handle_upsert_server,
            },
            execute::handle_call_capability,
            helpers::redact_value,
            types::{
                ApproveTicketRequest, CallCapabilityRequest, CatalogEventsQuery, CompletionRequest,
                GetPromptRequest, ReadResourceRequest, RejectTicketRequest, SamplingRequest,
                SearchCapabilitiesRequest, UpsertServerRequest,
            },
            ui::handle_ui_dashboard,
        },
    };
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
        };

        let async_res = handle_call_capability(State(state.clone()), headers, Json(req))
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

        let sync_res =
            handle_call_capability(State(state.clone()), HeaderMap::new(), Json(sync_req))
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
        };

        let res = handle_call_capability(State(state), HeaderMap::new(), Json(req))
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
        };

        let res = handle_call_capability(State(state.clone()), HeaderMap::new(), Json(req))
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
        let handle = crate::audit::spawn_audit_worker(store.clone(), 100, 50, 10);
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
        let verify_res =
            crate::http_v1::audit_api::handle_verify_audit_chain(State(state.clone()))
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
    }
}
