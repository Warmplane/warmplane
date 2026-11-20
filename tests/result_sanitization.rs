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
