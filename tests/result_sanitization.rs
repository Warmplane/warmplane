use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};

use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tower::ServiceExt;
use warmplane::{
    config::PolicyConfig,
    daemon::{AppState, CapabilityMeta, Policy, PromptMeta, ResourceMeta, ServerMsg},
    engine::{ControlPlaneHandle, ExecutionOptions},
    idempotency::IdempotencyStore,
};

fn policy(rules: Value) -> Policy {
    Policy::from_config(Some(
        serde_json::from_value::<PolicyConfig>(json!({
            "resultSanitization": rules
        }))
        .unwrap(),
    ))
}

fn state(path: &Path) -> (AppState, mpsc::Receiver<ServerMsg>) {
    let store = IdempotencyStore::open_or_create(path, Duration::from_secs(3600)).unwrap();
    let (sender, upstream) = mpsc::channel(4);
    let upstream_id = "fixture".to_owned();
    let capabilities = HashMap::from([(
        "fixture.save".to_owned(),
        CapabilityMeta::new(&upstream_id, "save", "Save", "Save", json!({})),
    )]);
    let prompts = HashMap::from([(
        "fixture.prompt".to_owned(),
        PromptMeta {
            server: upstream_id.clone(),
            name: "review".to_owned(),
            title: None,
            description: None,
            arguments: Vec::new(),
            tags: Vec::new(),
        },
    )]);
    let resources = HashMap::from([(
        "fixture.resource".to_owned(),
        ResourceMeta {
            server: upstream_id.clone(),
            uri: "fixture:/record".to_owned(),
            name: "record".to_owned(),
            description: None,
            mime_type: None,
            tags: Vec::new(),
        },
    )]);
    let state = AppState::builder()
        .servers(HashMap::from([(upstream_id, sender)]))
        .capabilities(capabilities)
        .prompts(prompts)
        .resources(resources)
        .idempotency_store(Arc::new(store))
        .policy(policy(
            json!({"fieldNames": ["private"], "patterns": ["sensitive-[0-9]+"]}),
        ))
        .build();
    (state, upstream)
}

async fn rest(router: Router, method: Method, path: &str, payload: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn live_rest_tool_resource_and_prompt_results_are_sanitized() {
    let dir = tempfile::tempdir().unwrap();
    let (state, mut upstream) = state(&dir.path().join("cache.json"));
    let router = warmplane::daemon::server::build_router(state);
    let fixture = tokio::spawn(async move {
        for _ in 0..3 {
            let (reply, value) = match upstream.recv().await.unwrap() {
                ServerMsg::CallTool { reply, .. } => (
                    reply,
                    json!({
                        "content": [{"type": "text", "text": "{\"PRIVATE\": \"hidden\", \"safe\": true}"}],
                        "structuredContent": {"PRIVATE": {"nested": "hidden"}, "safe": 1},
                    }),
                ),
                ServerMsg::ReadResource { reply, .. } => (
                    reply,
                    json!({
                        "contents": [{"uri": "fixture:/record", "text": "value sensitive-12", "mimeType": "text/plain"}],
                    }),
                ),
                ServerMsg::GetPrompt { reply, .. } => (
                    reply,
                    json!({
                        "messages": [{"role": "user", "content": {"type": "text", "text": "sensitive-34"}}],
                    }),
                ),
            };
            reply.send(Ok(value)).unwrap();
        }
    });
    let (status, tool) = rest(
        router.clone(),
        Method::POST,
        "/v1/tools/call",
        json!({
            "capability_id": "fixture.save", "args": {},
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{tool}");
    assert_eq!(
        tool["data"]["structuredContent"],
        json!({"PRIVATE": "[REDACTED]", "safe": 1})
    );
    let text: Value =
        serde_json::from_str(tool["data"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(text, json!({"PRIVATE": "[REDACTED]", "safe": true}));
    let (status, resource) = rest(
        router.clone(),
        Method::POST,
        "/v1/resources/read",
        json!({
            "resource_id": "fixture.resource",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{resource}");
    assert_eq!(resource["data"]["contents"][0]["text"], "[REDACTED]");
    assert_eq!(resource["data"]["contents"][0]["uri"], "fixture:/record");
    let (status, prompt) = rest(
        router,
        Method::POST,
        "/v1/prompts/get",
        json!({
            "prompt_id": "fixture.prompt",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{prompt}");
    assert_eq!(prompt["data"]["messages"][0]["role"], "user");
    assert_eq!(
        prompt["data"]["messages"][0]["content"]["text"],
        "[REDACTED]"
    );
    fixture.await.unwrap();
}

#[tokio::test]
async fn withheld_mutation_is_not_dispatched_again_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.json");
    let (before, mut upstream) = state(&cache);
    let handle = ControlPlaneHandle::new(before);
    let options = ExecutionOptions::new().with_idempotency_key("mutation");
    let worker = tokio::spawn(async move {
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected tool call");
        };
        reply
            .send(Ok(json!({
                "content": [{"type": "image", "data": "YQ==", "mimeType": "image/png"}],
            })))
            .unwrap();
    });
    let result = handle
        .call_capability("fixture.save", json!({}), options.clone())
        .await;
    assert!(!result.ok);
    assert_eq!(result.error.unwrap().code, "RESULT_WITHHELD");
    assert_eq!(result.retry.upstream_execution_state, "completed");
    worker.await.unwrap();
    drop(handle);

    let (after, mut upstream) = state(&cache);
    let handle = ControlPlaneHandle::new(after);
    let result = handle
        .call_capability("fixture.save", json!({}), options)
        .await;
    assert_eq!(result.error.unwrap().code, "RESULT_WITHHELD");
    assert_eq!(result.retry.upstream_execution_state, "completed");
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn stale_stamped_cache_response_is_withheld_after_policy_change() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.json");
    let (state, mut upstream) = state(&cache);
    let handle = ControlPlaneHandle::new(state.clone());
    let options = ExecutionOptions::new().with_idempotency_key("changed");
    let worker = tokio::spawn(async move {
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected tool call");
        };
        reply
            .send(Ok(
                json!({"content": [{"type": "text", "text": "customer-id"}]}),
            ))
            .unwrap();
        upstream
    });
    let first = handle
        .call_capability("fixture.save", json!({}), options.clone())
        .await;
    assert!(first.ok);
    let same_revision = handle
        .call_capability("fixture.save", json!({}), options.clone())
        .await;
    assert_eq!(first, same_revision);
    *state.policy.write().await = policy(json!({"patterns": ["customer-"]}));
    let stale = handle
        .call_capability("fixture.save", json!({}), options)
        .await;
    assert_eq!(stale.error.unwrap().code, "RESULT_WITHHELD");

    let router = warmplane::daemon::server::build_router(state);
    let (status, inspection) = rest(
        router,
        Method::GET,
        "/v1/idempotency/records/changed",
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{inspection}");
    assert_eq!(
        inspection["record"]["response"]["error"]["code"],
        "RESULT_WITHHELD"
    );
    assert!(!inspection.to_string().contains("customer-id"));
    assert!(worker.await.unwrap().try_recv().is_err());
}

#[tokio::test]
async fn batch_rejects_reference_to_a_protected_result_before_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let (state, mut upstream) = state(&dir.path().join("cache.json"));
    let router = warmplane::daemon::server::build_router(state);
    let worker = tokio::spawn(async move {
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected tool call");
        };
        reply
            .send(Ok(json!({"structuredContent": {"private": "credential"}})))
            .unwrap();
        upstream
    });
    let (status, batch) = rest(router, Method::POST, "/v1/tools/batch_call", json!({
        "steps": [
            {"id": "first", "capability_id": "fixture.save", "args": {}},
            {"id": "second", "capability_id": "fixture.save", "args": {"field": "$first.structuredContent.private"}}
        ],
    })).await;
    assert_eq!(status, StatusCode::OK, "{batch}");
    assert_eq!(
        batch["results"][0]["data"]["structuredContent"]["private"],
        "[REDACTED]"
    );
    assert_eq!(batch["results"][1]["ok"], false);
    assert!(batch["results"][1]["error"]
        .as_str()
        .unwrap()
        .contains("protected result"));
    assert!(worker.await.unwrap().try_recv().is_err());
}

#[tokio::test]
async fn task_polling_retains_completion_and_withholds_a_stale_result() {
    let dir = tempfile::tempdir().unwrap();
    let (state, mut upstream) = state(&dir.path().join("cache.json"));
    let handle = ControlPlaneHandle::new(state.clone());
    let worker = tokio::spawn(async move {
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected tool call");
        };
        reply
            .send(Ok(
                json!({"content": [{"type": "text", "text": "customer record"}]}),
            ))
            .unwrap();
    });
    let pending = handle
        .call_capability(
            "fixture.save",
            json!({}),
            ExecutionOptions::new().with_async_task(true),
        )
        .await;
    let task = pending.data.unwrap();
    let task_id = task["taskId"].as_str().unwrap();
    worker.await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let status = handle.get_task(task_id).await.data.unwrap();
            if let Some(result) = status.result {
                break result;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(result["content"][0]["text"], "customer record");

    *state.policy.write().await = policy(json!({"patterns": ["customer"]}));
    let after = handle.get_task(task_id).await.data.unwrap();
    assert_eq!(after.status, warmplane::tasks::TaskStatus::Completed);
    assert!(after.result.is_none());
    assert_eq!(after.error.unwrap()["code"], "RESULT_WITHHELD");
}

#[tokio::test]
async fn enforcing_results_keep_raw_payloads_and_errors_out_of_logs_audit_and_siem() {
    use tracing::instrument::WithSubscriber;
    use warmplane::{
        audit::{spawn_audit_worker, SiemDispatcher},
        config::{SiemConfig, SiemTargetConfig},
        daemon::UpstreamCallError,
    };

    let directory = tempfile::tempdir().unwrap();
    let (mut state, mut upstream) = state(&directory.path().join("cache.json"));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/audit", listener.local_addr().unwrap());
    let (exports, mut observed) = mpsc::channel(8);
    let receiver = Router::new().route(
        "/audit",
        axum::routing::post(move |axum::Json(events): axum::Json<Value>| {
            let exports = exports.clone();
            async move {
                exports.send(events).await.unwrap();
                StatusCode::OK
            }
        }),
    );
    let collector = tokio::spawn(async move { axum::serve(listener, receiver).await.unwrap() });
    let siem = SiemDispatcher::new(Some(SiemConfig {
        targets: vec![SiemTargetConfig::Webhook {
            url,
            auth_header: None,
            headers: HashMap::new(),
        }],
    }));
    state.audit_handle = spawn_audit_worker(state.audit_store.clone(), Some(siem), 64, 5, 32);
    let router = warmplane::daemon::server::build_router(state.clone());

    let worker = tokio::spawn(async move {
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected a tool call");
        };
        reply
            .send(Ok(json!({
                "structuredContent": {"Private": "secret-response-758"},
                "content": [{"type":"text", "text":"sensitive-12345"}]
            })))
            .unwrap();
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected a tool call");
        };
        reply
            .send(Err(UpstreamCallError::Upstream(
                "secret-upstream-error-329".to_owned(),
            )))
            .unwrap();
    });

    let log = directory.path().join("trace.log");
    let output = std::fs::File::create(&log).unwrap();
    let tracing = tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(output))
        .with_max_level(tracing::Level::TRACE)
        .finish();
    let (success, failure) = async {
        let success = rest(
            router.clone(),
            Method::POST,
            "/v1/tools/call",
            json!({
                "capability_id": "fixture.save", "args": {"attempt": 1}
            }),
        )
        .await;
        let failure = rest(
            router,
            Method::POST,
            "/v1/tools/call",
            json!({
                "capability_id": "fixture.save", "args": {"attempt": 2}
            }),
        )
        .await;
        (success, failure)
    }
    .with_subscriber(tracing)
    .await;

    assert_eq!(success.0, StatusCode::OK);
    assert_eq!(
        success.1["data"]["structuredContent"]["Private"],
        "[REDACTED]"
    );
    assert_eq!(failure.0, StatusCode::BAD_GATEWAY);
    assert_eq!(failure.1["error"]["message"], "Upstream call failed");
    worker.await.unwrap();

    state.audit_handle.shutdown().await;
    let (audit, _) = state.audit_store.query(&Default::default()).await;
    assert_eq!(audit.len(), 2);
    let mut exported = Vec::new();
    while let Ok(batch) = observed.try_recv() {
        exported.extend(batch.as_array().unwrap().iter().cloned());
    }
    assert_eq!(exported.len(), 2);
    let exposures = [
        std::fs::read_to_string(log).unwrap(),
        serde_json::to_string(&audit).unwrap(),
        serde_json::to_string(&exported).unwrap(),
    ];
    for text in exposures {
        assert!(!text.contains("secret-response-758"));
        assert!(!text.contains("sensitive-12345"));
        assert!(!text.contains("secret-upstream-error-329"));
    }
    collector.abort();
}

#[tokio::test]
async fn mcp_http_uses_verified_tenant_scope_for_a_cached_result() {
    use rmcp::{
        model::CallToolRequestParams,
        service::{RoleClient, RunningService},
        transport::{
            streamable_http_client::StreamableHttpClientTransportConfig,
            streamable_http_server::{
                session::local::LocalSessionManager, StreamableHttpServerConfig,
                StreamableHttpService,
            },
            StreamableHttpClientTransport,
        },
        ServiceExt,
    };

    async fn connect(url: &str, token: &'static str) -> RunningService<RoleClient, ()> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-Warmplane-Key",
            reqwest::header::HeaderValue::from_static(token),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let config = StreamableHttpClientTransportConfig::with_uri(url.to_owned());
        let transport = StreamableHttpClientTransport::with_client(client, config);
        ().serve(transport).await.unwrap()
    }

    async fn request(client: &RunningService<RoleClient, ()>) -> Value {
        let arguments = json!({
            "capability_id":"fixture.save",
            "args":{},
            "request_id":"same-client-key",
        })
        .as_object()
        .unwrap()
        .clone();
        let reply = client
            .call_tool(CallToolRequestParams::new("capability_call").with_arguments(arguments))
            .await
            .unwrap();
        reply.structured_content.unwrap()
    }

    let directory = tempfile::tempdir().unwrap();
    let (mut state, mut upstream) = state(&directory.path().join("cache.json"));
    let rbac = serde_json::from_value(json!({
        "enabled":true,
        "defaultRole":"anonymous",
        "roles":{"caller":{}},
        "tokens":{
            "fixture-token-a":{"role":"caller","tenantId":"tenant-a","actorId":"same-actor"},
            "fixture-token-b":{"role":"caller","tenantId":"tenant-b","actorId":"same-actor"}
        },
    }))
    .unwrap();
    state.rbac_engine = warmplane::rbac::RbacEngine::new(Some(rbac));
    let facade = StreamableHttpService::new(
        move || {
            Ok(warmplane::mcp_server::FacadeMcpServer::new(
                state.clone(),
                None,
            ))
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let app = Router::new().route_service("/mcp", facade);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let owner = connect(&url, "fixture-token-a").await;
    let other = connect(&url, "fixture-token-b").await;
    let worker = tokio::spawn(async move {
        let ServerMsg::CallTool { reply, .. } = upstream.recv().await.unwrap() else {
            panic!("expected a tool call");
        };
        reply
            .send(Ok(
                json!({"structuredContent": {"private":"confidential-value"}}),
            ))
            .unwrap();
        upstream
    });
    let first = tokio::time::timeout(Duration::from_secs(3), request(&owner))
        .await
        .unwrap();
    assert_eq!(
        first["data"]["structuredContent"]["private"], "[REDACTED]",
        "{first}"
    );
    assert_eq!(request(&owner).await, first);
    let refused = request(&other).await;
    assert_eq!(refused["error"]["code"], "POLICY_DENIED");
    assert!(worker.await.unwrap().try_recv().is_err());
    server.abort();
}
