use std::{collections::HashMap, time::Duration};

use axum::{
    body::to_bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tokio::sync::mpsc;
use warmplane::{
    config::{ReversibilityClass, ReversibilityConfig},
    daemon::{AppState, CapabilityMeta, ServerMsg},
    emissions::HoldCoordinator,
    tasks::{CreateTaskParams, EmissionCancelResult, EmissionPhase, EmissionRecord, TaskStatus},
};

fn delayable_contract(hold_window_secs: u64) -> ReversibilityConfig {
    ReversibilityConfig {
        class: ReversibilityClass::Delayable,
        hold_window_secs: Some(hold_window_secs),
        ttl_secs: None,
        daily_limit: None,
        pre_snapshot: None,
        inverse: None,
    }
}

fn create_state() -> (AppState, mpsc::Receiver<ServerMsg>) {
    let (server_tx, server_rx) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(1));
    let state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(2_000)
        .build();
    (state, server_rx)
}

fn params(idempotency_key: Option<&str>) -> CreateTaskParams {
    CreateTaskParams {
        capability_id: "mail.send".to_string(),
        server_id: "mail".to_string(),
        args: json!({"to": "person@example.com", "body": "Hello"}),
        request_id: Some("request-mail-1".to_string()),
        context: None,
        idempotency_key: idempotency_key.map(ToString::to_string),
        initial_status: TaskStatus::Working,
        status_message: Some("Waiting for release".to_string()),
        input_requests: None,
        ttl_ms: None,
        poll_interval_ms: Some(50),
    }
}

fn emission(release_at_ms: u64) -> EmissionRecord {
    EmissionRecord {
        phase: EmissionPhase::PendingEmission,
        release_at_ms,
        cancellable: true,
        tool_name: "send".to_string(),
        input_responses: None,
        request_state: None,
        profile_id: None,
        tenant_id: "default".to_string(),
        role: "anonymous".to_string(),
        authorization_expires_at_epoch_secs: None,
        approval_completed: false,
        batch_id: None,
        step_id: None,
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[tokio::test]
async fn cancelling_two_second_hold_prevents_upstream_submission() {
    let (state, mut upstream) = create_state();
    let coordinator = HoldCoordinator::default();
    let task = coordinator
        .accept(
            &state,
            params(Some("cancel-before-release")),
            emission(now_ms() + 2_000),
        )
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        coordinator
            .cancel(
                &state,
                &task.task_id,
                Some("operator cancelled".to_string())
            )
            .await
            .unwrap(),
        EmissionCancelResult::Cancelled
    );
    tokio::time::sleep(Duration::from_millis(1_600)).await;
    assert!(upstream.try_recv().is_err());
    assert_eq!(
        state
            .task_registry
            .get_task(&task.task_id)
            .await
            .unwrap()
            .status,
        TaskStatus::Cancelled
    );
}

#[tokio::test]
async fn uncancelled_hold_submits_exactly_once_and_completes_task() {
    let (state, mut upstream) = create_state();
    let coordinator = HoldCoordinator::default();
    let task = coordinator
        .accept(&state, params(None), emission(now_ms() + 50))
        .await
        .unwrap();

    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected one upstream tool call");
    };
    reply.send(Ok(serde_json::json!({"sent": true}))).unwrap();
    let completed = coordinator
        .wait_for_terminal(&state, &task.task_id, Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(completed.status, TaskStatus::Completed);
    assert_eq!(completed.result, Some(json!({"sent": true})));
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn recovery_never_resends_a_durably_claimed_dispatch() {
    let (state, mut upstream) = create_state();
    let task = state
        .task_registry
        .create_emission_task(params(None), emission(now_ms().saturating_sub(1)))
        .await
        .unwrap();
    state
        .task_registry
        .claim_emission_dispatch(&task.task_id)
        .await
        .unwrap();

    state.hold_coordinator.recover(&state).await;
    let failed = state
        .hold_coordinator
        .wait_for_terminal(&state, &task.task_id, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(failed.status, TaskStatus::Failed);
    assert_eq!(failed.error.unwrap()["code"], "EMISSION_OUTCOME_UNKNOWN");
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn release_revalidates_current_policy_before_dispatch() {
    let (state, mut upstream) = create_state();
    let task = state
        .hold_coordinator
        .accept(&state, params(None), emission(now_ms() + 50))
        .await
        .unwrap();
    state
        .policy
        .write()
        .await
        .deny
        .push("mail.send".to_string());

    let failed = state
        .hold_coordinator
        .wait_for_terminal(&state, &task.task_id, Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(failed.status, TaskStatus::Failed);
    assert_eq!(
        failed.error.unwrap()["code"],
        "EMISSION_REVALIDATION_FAILED"
    );
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn completed_approval_is_preserved_through_the_hold() {
    let (state, mut upstream) = create_state();
    state
        .policy
        .write()
        .await
        .require_approval
        .push("mail.send".to_string());
    let mut approved_emission = emission(now_ms() + 50);
    approved_emission.approval_completed = true;
    let task = state
        .hold_coordinator
        .accept(&state, params(None), approved_emission)
        .await
        .unwrap();

    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected approved call to dispatch after its hold");
    };
    reply.send(Ok(json!({"sent": true}))).unwrap();
    assert_eq!(
        state
            .hold_coordinator
            .wait_for_terminal(&state, &task.task_id, Duration::from_secs(2))
            .await
            .unwrap()
            .status,
        TaskStatus::Completed
    );
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn converting_approval_task_to_hold_removes_approval_ttl() {
    let (state, _upstream) = create_state();
    let (task, _waiter) = state
        .task_registry
        .create_task(CreateTaskParams {
            initial_status: TaskStatus::InputRequired,
            ttl_ms: Some(30_000),
            status_message: Some("Waiting for approval".to_string()),
            input_requests: Some(std::collections::BTreeMap::new()),
            ..params(None)
        })
        .await;
    let held = state
        .hold_coordinator
        .queue_existing(
            &state,
            &task.task_id,
            json!({"to": "approved@example.com"}),
            emission(now_ms() + 10_000),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(held.ttl_ms, None);
    assert_eq!(held.status, TaskStatus::Working);
}

#[tokio::test]
async fn batch_delayable_step_persists_a_stable_idempotency_key() {
    let (state, mut upstream) = create_state();
    let policy = warmplane::daemon::Policy::default();
    let batch_state = state.clone();
    let batch = tokio::spawn(async move {
        warmplane::batch_executor::execute_batch(
            &batch_state,
            vec![warmplane::batch_executor::BatchStep {
                id: "send-1".to_string(),
                capability_id: "mail.send".to_string(),
                args: json!({"to": "person@example.com"}),
                continue_on_error: false,
            }],
            "trace-batch".to_string(),
            Some("batch-request".to_string()),
            None,
            &policy,
            &warmplane::context::ProfileContext::unrestricted(),
        )
        .await
    });

    let Some(ServerMsg::CallTool { reply, .. }) = upstream.recv().await else {
        panic!("expected batch step to dispatch");
    };
    reply.send(Ok(json!({"sent": true}))).unwrap();
    let result = batch.await.unwrap();
    assert!(result.ok);

    let task = state
        .task_registry
        .find_by_idempotency_key("batch:batch-request:step:send-1")
        .await
        .expect("batch emission task should have a stable idempotency key");
    assert_eq!(task.emission.unwrap().step_id.as_deref(), Some("send-1"));
    assert_eq!(result.results[0].data, Some(json!({"sent": true})));
}

#[tokio::test]
async fn rest_async_receipt_can_cancel_the_shared_hold() {
    let (state, mut upstream) = create_state();
    let response = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "person@example.com"}),
            request_id: Some("rest-emission-1".to_string()),
            idempotency_key: Some("rest-emission-key".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    let task_id = body["task"]["taskId"].as_str().unwrap().to_string();
    assert_eq!(body["task"]["phase"], "pending_emission");
    assert_eq!(body["task"]["cancellable"], true);

    let cancel = warmplane::http_v1::tasks_api::handle_cancel_task(
        State(state.clone()),
        Path(task_id.clone()),
        axum::Extension(None),
        None,
        Json(warmplane::http_v1::tasks_api::CancelTaskRequest::default()),
    )
    .await
    .into_response();
    assert_eq!(cancel.status(), StatusCode::OK);
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    assert!(upstream.try_recv().is_err());
    assert_eq!(
        state.task_registry.get_task(&task_id).await.unwrap().status,
        TaskStatus::Cancelled
    );
}

#[tokio::test]
async fn configuration_reload_replaces_profiles_used_by_release_revalidation() {
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("config.json");
    let profiles = HashMap::from([(
        "work".to_string(),
        warmplane::config::ProfileConfig {
            servers: Vec::new(),
            description: None,
            policy: Some(warmplane::config::PolicyConfig {
                hold_window_secs: Some(9),
                ..Default::default()
            }),
        },
    )]);
    let config = warmplane::config::McpConfig {
        profiles: profiles.clone(),
        ..Default::default()
    };
    warmplane::config::save_config(config_path.to_str().unwrap(), &config).unwrap();

    let state = AppState::builder()
        .config_path(config_path.to_string_lossy().into_owned())
        .profiles(HashMap::new())
        .build();
    state.reload_from_disk().await.unwrap();

    let current = state.profiles.read().await;
    assert_eq!(
        current["work"].policy.as_ref().unwrap().hold_window_secs,
        Some(9)
    );
}
