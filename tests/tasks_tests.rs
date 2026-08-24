// Rust guideline compliant 2026-08-24

//! Integration test suite for SEP-2663 Tasks Extension, lifecycle transitions,
//! HITL suspension into tasks, input responses via tasks/update, and idempotency deduplication.

use axum::{
    body::to_bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::mpsc;
use warmplane::{
    config::PolicyConfig,
    daemon::{AppState, CapabilityMeta, Policy},
    http_v1::{
        handle_call_capability, handle_cancel_task, handle_get_task, handle_list_tasks,
        handle_update_task, tasks_api::UpdateTaskRequest, CallCapabilityRequest,
    },
};

#[tokio::test]
async fn test_tasks_lifecycle_and_hitl_integration() {
    let mut capabilities = HashMap::new();
    capabilities.insert(
        "payments.charge".to_string(),
        CapabilityMeta {
            server: "stripe".to_string(),
            tool: "charge".to_string(),
            summary: "Charge payment card".to_string(),
            description: "Mutating payment operation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "amount": {"type": "integer"}
                }
            }),
            tags: vec![],
            examples: vec![],
        },
    );

    let (tx, mut rx) = mpsc::channel(1);
    let mut servers = HashMap::new();
    servers.insert("stripe".to_string(), tx);

    let policy_config = PolicyConfig {
        allow: vec!["*".to_string()],
        deny: vec![],
        redact_keys: vec!["card_number".to_string()],
        require_approval: vec!["payments.*".to_string()],
        approval_timeout_secs: Some(30),
        webhook: None,
    };

    let state = AppState::builder()
        .capabilities(capabilities)
        .servers(servers)
        .policy(Policy::from_config(Some(policy_config)))
        .catalog_version("test-ver-tasks")
        .build();

    // 1. Issue an async tool call for a capability requiring HITL approval
    let req = CallCapabilityRequest {
        capability_id: "payments.charge".to_string(),
        args: json!({"amount": 100, "card_number": "4000-1234-5678-9010"}),
        request_id: Some("req-task-1".to_string()),
        context: None,
        idempotency_key: Some("idk_task_test_1".to_string()),
        input_responses: None,
        request_state: None,
        async_task: true,
    };

    let resp = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(None),
        None,
        HeaderMap::new(),
        Json(req),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body_json["resultType"], "task");

    let task_id = body_json["task"]["taskId"].as_str().unwrap().to_string();
    assert_eq!(body_json["task"]["status"], "input_required");
    assert!(body_json["task"]["inputRequests"]["hitl_approval"].is_object());

    // 2. Poll tasks/get endpoint
    let get_resp = handle_get_task(State(state.clone()), axum::extract::Path(task_id.clone()))
        .await
        .into_response();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_bytes = to_bytes(get_resp.into_body(), usize::MAX).await.unwrap();
    let get_json: Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_json["task"]["status"], "input_required");

    // 3. List tasks
    let list_resp = handle_list_tasks(State(state.clone()))
        .await
        .into_response();
    let list_bytes = to_bytes(list_resp.into_body(), usize::MAX).await.unwrap();
    let list_json: Value = serde_json::from_slice(&list_bytes).unwrap();
    assert_eq!(list_json["total"], 1);

    // 4. Update task with input response (approving the execution)
    let mut responses = BTreeMap::new();
    responses.insert(
        "hitl_approval".to_string(),
        json!({"approved": true, "modified_args": {"amount": 120}}),
    );

    let update_resp = handle_update_task(
        State(state.clone()),
        axum::extract::Path(task_id.clone()),
        Json(UpdateTaskRequest {
            input_responses: responses,
        }),
    )
    .await
    .into_response();
    assert_eq!(update_resp.status(), StatusCode::OK);

    // 5. Upstream server actor receives CallTool and replies
    let msg = rx.recv().await.expect("worker must forward tool call");
    match msg {
        warmplane::daemon::ServerMsg::CallTool {
            name,
            params,
            reply,
            ..
        } => {
            assert_eq!(name, "charge");
            assert_eq!(params["amount"], 120);
            let _ = reply.send(Ok(
                json!({"charge_id": "ch_test_999", "status": "succeeded"}),
            ));
        }
        _ => panic!("Expected CallTool message"),
    }

    // 6. Give worker a moment to process reply and verify completed state
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let final_get = handle_get_task(State(state.clone()), axum::extract::Path(task_id.clone()))
        .await
        .into_response();
    let final_bytes = to_bytes(final_get.into_body(), usize::MAX).await.unwrap();
    let final_json: Value = serde_json::from_slice(&final_bytes).unwrap();
    assert_eq!(final_json["task"]["status"], "completed");
    assert_eq!(final_json["task"]["result"]["charge_id"], "ch_test_999");
}

#[tokio::test]
async fn test_task_cancellation() {
    let state = AppState::builder().build();

    let (task, _) = state
        .task_registry
        .create_task(warmplane::tasks::CreateTaskParams {
            capability_id: "batch.heavy_job".to_string(),
            server_id: "compute".to_string(),
            args: json!({}),
            request_id: None,
            context: None,
            idempotency_key: None,
            initial_status: warmplane::tasks::TaskStatus::Working,
            status_message: Some("Job in progress".to_string()),
            input_requests: None,
            ttl_ms: Some(10_000),
            poll_interval_ms: Some(500),
        })
        .await;

    let cancel_resp = handle_cancel_task(
        State(state.clone()),
        axum::extract::Path(task.task_id.clone()),
        Json(warmplane::http_v1::tasks_api::CancelTaskRequest {
            reason: Some("User requested cancellation".to_string()),
        }),
    )
    .await
    .into_response();
    assert_eq!(cancel_resp.status(), StatusCode::OK);

    let get_resp = handle_get_task(State(state), axum::extract::Path(task.task_id))
        .await
        .into_response();
    let get_bytes = to_bytes(get_resp.into_body(), usize::MAX).await.unwrap();
    let get_json: Value = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_json["task"]["status"], "cancelled");
    assert_eq!(
        get_json["task"]["statusMessage"],
        "User requested cancellation"
    );
}
