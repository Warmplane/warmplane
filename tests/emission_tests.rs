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
    tasks::{
        CreateTaskParams, EmissionCancelResult, EmissionPhase, EmissionRecord, TaskRegistry,
        TaskStatus,
    },
};

fn delayable_contract(hold_window_secs: u64) -> ReversibilityConfig {
    ReversibilityConfig {
        class: ReversibilityClass::Delayable,
        hold_window_secs: Some(hold_window_secs),
        ttl_secs: None,
        daily_limit: None,
        pre_snapshot: None,
        pre_undo_check: None,
        test_target: None,
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
async fn cancellation_racing_release_has_one_durable_winner() {
    let (state, mut upstream) = create_state();
    let task = state
        .hold_coordinator
        .accept(&state, params(None), emission(now_ms() + 100))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let outcome = state
        .hold_coordinator
        .cancel(&state, &task.task_id, Some("race cancellation".to_string()))
        .await
        .unwrap();

    match outcome {
        EmissionCancelResult::Cancelled | EmissionCancelResult::AlreadyCancelled => {
            tokio::time::sleep(Duration::from_millis(50)).await;
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
        EmissionCancelResult::TooLate => {
            let Some(ServerMsg::CallTool { reply, .. }) =
                tokio::time::timeout(Duration::from_secs(2), upstream.recv())
                    .await
                    .unwrap()
            else {
                panic!("claimed release should produce exactly one upstream call");
            };
            reply.send(Ok(json!({"sent": true}))).unwrap();
            assert_eq!(
                state
                    .hold_coordinator
                    .wait_for_terminal(&state, &task.task_id, Duration::from_secs(1))
                    .await
                    .unwrap()
                    .status,
                TaskStatus::Completed
            );
            assert!(upstream.try_recv().is_err());
        }
        EmissionCancelResult::NotEmission => panic!("active emission should remain addressable"),
    }
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
async fn rest_synchronous_short_hold_waits_and_returns_upstream_result() {
    let (state, mut upstream) = create_state();
    let call_state = state.clone();
    let call = tokio::spawn(async move {
        warmplane::http_v1::execute::handle_call_capability(
            State(call_state),
            axum::Extension(None),
            None,
            HeaderMap::new(),
            Json(warmplane::http_v1::CallCapabilityRequest {
                capability_id: "mail.send".to_string(),
                args: json!({"to": "person@example.com"}),
                request_id: Some("rest-sync-emission".to_string()),
                ..Default::default()
            }),
        )
        .await
        .into_response()
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(upstream.try_recv().is_err());
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected synchronous REST hold to dispatch after its deadline");
    };
    reply.send(Ok(json!({"sent": true}))).unwrap();
    let response = call.await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["data"], json!({"sent": true}));
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn prefer_async_and_equal_hold_timeout_return_receipts() {
    let (state, mut upstream) = create_state();
    let mut headers = HeaderMap::new();
    headers.insert(
        "prefer",
        axum::http::HeaderValue::from_static("respond-async"),
    );
    let async_response = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        headers,
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "person@example.com"}),
            request_id: Some("rest-prefer-async".to_string()),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(async_response.status(), StatusCode::ACCEPTED);
    let async_body: serde_json::Value = serde_json::from_slice(
        &to_bytes(async_response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let async_task_id = async_body["task"]["taskId"].as_str().unwrap().to_string();
    assert_eq!(
        state
            .hold_coordinator
            .cancel(&state, &async_task_id, None)
            .await
            .unwrap(),
        EmissionCancelResult::Cancelled
    );

    let (server_tx, mut equality_upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(2));
    let equality_state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(2_000)
        .build();
    let started = tokio::time::Instant::now();
    let equality_response = warmplane::http_v1::execute::handle_call_capability(
        State(equality_state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "person@example.com"}),
            request_id: Some("rest-equal-window".to_string()),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(equality_response.status(), StatusCode::ACCEPTED);
    let equality_body: serde_json::Value = serde_json::from_slice(
        &to_bytes(equality_response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(equality_body["task"]["phase"], "pending_emission");
    let equality_task_id = equality_body["task"]["taskId"].as_str().unwrap();
    assert_eq!(
        equality_state
            .hold_coordinator
            .cancel(&equality_state, equality_task_id, None)
            .await
            .unwrap(),
        EmissionCancelResult::Cancelled
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(upstream.try_recv().is_err());
    assert!(equality_upstream.try_recv().is_err());

    let (server_tx, mut long_upstream) = mpsc::channel(8);
    let mut long_capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    long_capability.reversibility = Some(delayable_contract(3));
    let long_state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), long_capability)]))
        .tool_timeout_ms(1_000)
        .build();
    let started = tokio::time::Instant::now();
    let long_response = warmplane::http_v1::execute::handle_call_capability(
        State(long_state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "person@example.com"}),
            request_id: Some("rest-long-window".to_string()),
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(long_response.status(), StatusCode::ACCEPTED);
    let long_body: serde_json::Value = serde_json::from_slice(
        &to_bytes(long_response.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(long_body["task"]["phase"], "pending_emission");
    let long_task_id = long_body["task"]["taskId"].as_str().unwrap();
    assert_eq!(
        long_state
            .hold_coordinator
            .cancel(&long_state, long_task_id, None)
            .await
            .unwrap(),
        EmissionCancelResult::Cancelled
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(long_upstream.try_recv().is_err());
}

#[tokio::test]
async fn rest_idempotency_after_completion_returns_the_original_task() {
    let (state, mut upstream) = create_state();
    let first = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "person@example.com"}),
            request_id: Some("rest-completed-idem".to_string()),
            idempotency_key: Some("rest-completed-idem-key".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    let first_body: serde_json::Value =
        serde_json::from_slice(&to_bytes(first.into_body(), usize::MAX).await.unwrap()).unwrap();
    let task_id = first_body["task"]["taskId"].as_str().unwrap().to_string();
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected idempotent emission to dispatch");
    };
    reply.send(Ok(json!({"sent": true}))).unwrap();
    let completed = state
        .hold_coordinator
        .wait_for_terminal(&state, &task_id, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(completed.status, TaskStatus::Completed);

    let duplicate = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "different@example.com"}),
            request_id: Some("rest-completed-idem-retry".to_string()),
            idempotency_key: Some("rest-completed-idem-key".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate_body: serde_json::Value =
        serde_json::from_slice(&to_bytes(duplicate.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    assert_eq!(duplicate_body["task"]["taskId"], task_id);
    assert_eq!(duplicate_body["task"]["status"], "completed");
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn cancelled_idempotency_outcome_survives_registry_restart() {
    let directory = tempfile::tempdir().unwrap();
    let task_path = directory.path().join("tasks.json");
    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(30));
    let first_state = AppState::builder()
        .task_registry(TaskRegistry::open_or_create(&task_path).unwrap())
        .servers(HashMap::from([("mail".to_string(), server_tx.clone())]))
        .capabilities(HashMap::from([(
            "mail.send".to_string(),
            capability.clone(),
        )]))
        .tool_timeout_ms(2_000)
        .build();
    let task = first_state
        .hold_coordinator
        .accept(
            &first_state,
            CreateTaskParams {
                idempotency_key: Some("restart-cancel-key".to_string()),
                ..params(None)
            },
            emission(now_ms() + 30_000),
        )
        .await
        .unwrap();
    assert_eq!(
        first_state
            .hold_coordinator
            .cancel(
                &first_state,
                &task.task_id,
                Some("cancel before restart".to_string())
            )
            .await
            .unwrap(),
        EmissionCancelResult::Cancelled
    );
    drop(first_state);

    let restarted = AppState::builder()
        .task_registry(TaskRegistry::open_or_create(&task_path).unwrap())
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(2_000)
        .build();
    let response = warmplane::http_v1::execute::handle_call_capability(
        State(restarted.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "changed@example.com"}),
            request_id: Some("restart-cancel-retry".to_string()),
            idempotency_key: Some("restart-cancel-key".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["task"]["taskId"], task.task_id);
    assert_eq!(body["task"]["status"], "cancelled");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn task_read_and_cancel_require_matching_emission_ownership() {
    let (state, mut upstream) = create_state();
    let task = state
        .hold_coordinator
        .accept(&state, params(None), emission(now_ms() + 30_000))
        .await
        .unwrap();
    let other_tenant = warmplane::rbac::TenantContext {
        tenant_id: "another-tenant".to_string(),
        role: "anonymous".to_string(),
        actor_id: None,
        grant_id: None,
        grant_expires_at_epoch_secs: None,
        effective_policy: warmplane::daemon::Policy::default(),
    };
    let denied_read = warmplane::http_v1::tasks_api::handle_get_task(
        State(state.clone()),
        Path(task.task_id.clone()),
        axum::Extension(Some(other_tenant.clone())),
        None,
    )
    .await
    .into_response();
    assert_eq!(denied_read.status(), StatusCode::NOT_FOUND);
    let denied_cancel = warmplane::http_v1::tasks_api::handle_cancel_task(
        State(state.clone()),
        Path(task.task_id.clone()),
        axum::Extension(Some(other_tenant)),
        None,
        Json(warmplane::http_v1::tasks_api::CancelTaskRequest::default()),
    )
    .await
    .into_response();
    assert_eq!(denied_cancel.status(), StatusCode::NOT_FOUND);

    let wrong_profile = warmplane::context::ProfileContext::scoped("another-profile", ["mail"]);
    let denied_profile_read = warmplane::http_v1::tasks_api::handle_get_task(
        State(state.clone()),
        Path(task.task_id.clone()),
        axum::Extension(None),
        Some(axum::Extension(wrong_profile)),
    )
    .await
    .into_response();
    assert_eq!(denied_profile_read.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        state
            .hold_coordinator
            .cancel(&state, &task.task_id, Some("cleanup".to_string()))
            .await
            .unwrap(),
        EmissionCancelResult::Cancelled
    );
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
async fn recovery_resumes_future_and_overdue_pending_emissions() {
    let directory = tempfile::tempdir().unwrap();
    let task_path = directory.path().join("tasks.json");
    let registry = TaskRegistry::open_or_create(&task_path).unwrap();
    let overdue = registry
        .create_emission_task(
            params(Some("recovery-overdue")),
            emission(now_ms().saturating_sub(1)),
        )
        .await
        .unwrap();
    let future = registry
        .create_emission_task(params(Some("recovery-future")), emission(now_ms() + 100))
        .await
        .unwrap();
    drop(registry);

    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(1));
    let state = AppState::builder()
        .task_registry(TaskRegistry::open_or_create(&task_path).unwrap())
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(2_000)
        .build();

    let duplicate = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "changed@example.com"}),
            request_id: Some("recovery-retry".to_string()),
            idempotency_key: Some("recovery-overdue".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(duplicate.status(), StatusCode::ACCEPTED);
    let duplicate: serde_json::Value =
        serde_json::from_slice(&to_bytes(duplicate.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    assert_eq!(duplicate["task"]["taskId"], overdue.task_id);
    assert_eq!(state.task_registry.list_tasks().await.len(), 2);

    state.hold_coordinator.recover(&state).await;

    for _ in 0..2 {
        let Some(ServerMsg::CallTool { reply, .. }) =
            tokio::time::timeout(Duration::from_secs(2), upstream.recv())
                .await
                .unwrap()
        else {
            panic!("expected recovered pending emission to dispatch");
        };
        reply.send(Ok(json!({"sent": true}))).unwrap();
    }
    for task_id in [overdue.task_id, future.task_id] {
        let result = state
            .hold_coordinator
            .wait_for_terminal(&state, &task_id, Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
    }
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn failed_acceptance_persistence_does_not_register_or_dispatch() {
    let directory = tempfile::tempdir().unwrap();
    let task_path = directory.path().join("tasks.json");
    let registry = TaskRegistry::open_or_create(&task_path).unwrap();
    std::fs::create_dir(&task_path).unwrap();
    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(1));
    let state = AppState::builder()
        .task_registry(registry.clone())
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(2_000)
        .build();

    assert!(state
        .hold_coordinator
        .accept(&state, params(None), emission(now_ms() + 50))
        .await
        .is_err());
    assert!(registry.list_tasks().await.is_empty());
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn failed_dispatch_claim_persistence_prevents_upstream_submission() {
    let directory = tempfile::tempdir().unwrap();
    let task_path = directory.path().join("tasks.json");
    let registry = TaskRegistry::open_or_create(&task_path).unwrap();
    let pending = registry
        .create_emission_task(params(None), emission(now_ms() + 100))
        .await
        .unwrap();
    std::fs::remove_file(&task_path).unwrap();
    std::fs::create_dir(&task_path).unwrap();

    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(1));
    let state = AppState::builder()
        .task_registry(registry)
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(2_000)
        .build();
    state.hold_coordinator.recover(&state).await;
    tokio::time::sleep(Duration::from_millis(250)).await;

    let unchanged = state
        .task_registry
        .get_task(&pending.task_id)
        .await
        .unwrap();
    assert_eq!(unchanged.status, TaskStatus::Working);
    assert_eq!(
        unchanged.emission.unwrap().phase,
        EmissionPhase::PendingEmission
    );
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn registry_loads_legacy_tasks_without_emission_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tasks.json");
    let registry = TaskRegistry::new();
    let (task, _) = registry.create_task(params(None)).await;
    let task_id = task.task_id.clone();
    let mut legacy = serde_json::to_value(HashMap::from([(task_id.clone(), task)])).unwrap();
    let tasks = legacy.as_object_mut().unwrap();
    let task = tasks.values_mut().next().unwrap();
    task.as_object_mut().unwrap().remove("emission");
    std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();

    let reopened = TaskRegistry::open_or_create(&path).unwrap();
    let loaded = reopened.get_task(&task_id).await.unwrap();
    assert!(loaded.emission.is_none());
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
async fn release_rejects_a_changed_upstream_tool_mapping() {
    let (state, mut upstream) = create_state();
    let task = state
        .hold_coordinator
        .accept(&state, params(None), emission(now_ms() + 50))
        .await
        .unwrap();
    state
        .capabilities
        .write()
        .await
        .get_mut("mail.send")
        .unwrap()
        .tool = "send_to_different_endpoint".to_string();

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
async fn batch_waits_for_held_result_before_interpolating_the_next_step() {
    let (state, mut upstream) = create_state();
    let policy = warmplane::daemon::Policy::default();
    let batch_state = state.clone();
    let batch = tokio::spawn(async move {
        warmplane::batch_executor::execute_batch(
            &batch_state,
            vec![
                warmplane::batch_executor::BatchStep {
                    id: "first".to_string(),
                    capability_id: "mail.send".to_string(),
                    args: json!({"to": "first@example.com"}),
                    continue_on_error: false,
                },
                warmplane::batch_executor::BatchStep {
                    id: "second".to_string(),
                    capability_id: "mail.send".to_string(),
                    args: json!({"to": "$first.receipt"}),
                    continue_on_error: false,
                },
            ],
            "trace-batch-reference".to_string(),
            Some("batch-reference".to_string()),
            None,
            &policy,
            &warmplane::context::ProfileContext::unrestricted(),
        )
        .await
    });

    let Some(ServerMsg::CallTool { params, reply, .. }) = upstream.recv().await else {
        panic!("expected first batch emission to release");
    };
    assert_eq!(params["to"], "first@example.com");
    reply.send(Ok(json!({"receipt": "receipt-1"}))).unwrap();

    let Some(ServerMsg::CallTool { params, reply, .. }) = upstream.recv().await else {
        panic!("expected second batch emission after first result");
    };
    assert_eq!(params["to"], "receipt-1");
    reply.send(Ok(json!({"sent": true}))).unwrap();
    let result = batch.await.unwrap();
    assert!(result.ok);
    assert_eq!(result.results[1].data, Some(json!({"sent": true})));
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn cancelling_batch_operation_stops_the_current_hold_and_later_steps() {
    let (state, mut upstream) = create_state();
    let batch_state = state.clone();
    let batch = tokio::spawn(async move {
        warmplane::http_v1::execute::handle_batch_call_capabilities(
            State(batch_state),
            axum::Extension(None),
            None,
            HeaderMap::new(),
            Json(warmplane::batch_executor::BatchCallRequest {
                steps: vec![
                    warmplane::batch_executor::BatchStep {
                        id: "send-1".to_string(),
                        capability_id: "mail.send".to_string(),
                        args: json!({"to": "person@example.com"}),
                        continue_on_error: true,
                    },
                    warmplane::batch_executor::BatchStep {
                        id: "send-2".to_string(),
                        capability_id: "mail.send".to_string(),
                        args: json!({"to": "second@example.com"}),
                        continue_on_error: false,
                    },
                ],
                request_id: Some("batch-cancel-request".to_string()),
                context: None,
                auto_rollback_on_failure: false,
            }),
        )
        .await
        .into_response()
    });

    let mut held_task = None;
    for _ in 0..50 {
        held_task = state
            .task_registry
            .list_tasks()
            .await
            .into_iter()
            .find(|task| task.emission.is_some());
        if held_task.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(held_task.is_some(), "batch should accept its first hold");

    let cancellation = warmplane::http_v1::execute::handle_cancel_operation(
        State(state.clone()),
        Path("batch-cancel-request".to_string()),
        axum::Extension(None),
        None,
    )
    .await
    .into_response();
    assert_eq!(cancellation.status(), StatusCode::OK);
    let response = batch.await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["ok"], false);
    assert_eq!(body["results"].as_array().unwrap().len(), 1);
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    assert!(upstream.try_recv().is_err());
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
    let release_at = body["task"]["releaseAtMs"].as_u64().unwrap();

    let duplicate = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "different@example.com"}),
            request_id: Some("rest-emission-duplicate".to_string()),
            idempotency_key: Some("rest-emission-key".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(duplicate.status(), StatusCode::ACCEPTED);
    let duplicate: serde_json::Value =
        serde_json::from_slice(&to_bytes(duplicate.into_body(), usize::MAX).await.unwrap())
            .unwrap();
    assert_eq!(duplicate["task"]["taskId"], task_id);
    assert_eq!(duplicate["task"]["releaseAtMs"], release_at);

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
    let repeated_cancel = warmplane::http_v1::tasks_api::handle_cancel_task(
        State(state.clone()),
        Path(task_id.clone()),
        axum::Extension(None),
        None,
        Json(warmplane::http_v1::tasks_api::CancelTaskRequest::default()),
    )
    .await
    .into_response();
    assert_eq!(repeated_cancel.status(), StatusCode::OK);
    let duplicate_after_cancel = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "person@example.com"}),
            request_id: Some("rest-emission-after-cancel".to_string()),
            idempotency_key: Some("rest-emission-key".to_string()),
            async_task: true,
            ..Default::default()
        }),
    )
    .await
    .into_response();
    assert_eq!(duplicate_after_cancel.status(), StatusCode::OK);
    let duplicate_after_cancel: serde_json::Value = serde_json::from_slice(
        &to_bytes(duplicate_after_cancel.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(duplicate_after_cancel["task"]["taskId"], task_id);
    assert_eq!(duplicate_after_cancel["task"]["status"], "cancelled");
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
            allow_agent_self_undo: false,
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

#[tokio::test]
async fn embedded_synchronous_call_waits_for_short_hold_then_returns_result() {
    let (state, mut upstream) = create_state();
    let handle = warmplane::engine::ControlPlaneHandle::new(state);
    let call = tokio::spawn(async move {
        handle
            .call_capability(
                "mail.send",
                json!({"to": "person@example.com"}),
                warmplane::engine::ExecutionOptions::default()
                    .with_request_id("embedded-emission-1"),
            )
            .await
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(upstream.try_recv().is_err());
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected embedded delayable call to dispatch after its hold");
    };
    reply.send(Ok(json!({"sent": true}))).unwrap();
    let result = call.await.unwrap();
    assert!(result.ok);
    assert_eq!(result.data, Some(json!({"sent": true})));
    assert!(upstream.try_recv().is_err());
}

#[tokio::test]
async fn held_and_cancelled_audit_events_are_redacted_and_hash_chained() {
    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(30));
    let policy = warmplane::daemon::Policy::from_config(Some(warmplane::config::PolicyConfig {
        redact_keys: vec!["body".to_string()],
        ..Default::default()
    }));
    let state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .policy(policy)
        .build();
    let coordinator = HoldCoordinator::default();
    let mut task_params = params(None);
    task_params.args = json!({"to": "person@example.com", "body": "secret text"});
    let task = coordinator
        .accept(&state, task_params, emission(now_ms() + 30_000))
        .await
        .unwrap();
    coordinator
        .cancel(&state, &task.task_id, Some("operator request".to_string()))
        .await
        .unwrap();
    assert_eq!(
        coordinator
            .cancel(&state, &task.task_id, Some("repeat request".to_string()))
            .await
            .unwrap(),
        EmissionCancelResult::AlreadyCancelled
    );

    let mut release_params = params(Some("audit-release-key"));
    release_params.args = json!({"to": "person@example.com", "body": "released secret"});
    let released = coordinator
        .accept(&state, release_params, emission(now_ms() + 50))
        .await
        .unwrap();
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected audit test emission to dispatch");
    };
    reply.send(Ok(json!({"sent": true}))).unwrap();
    assert_eq!(
        coordinator
            .wait_for_terminal(&state, &released.task_id, Duration::from_secs(1))
            .await
            .unwrap()
            .status,
        TaskStatus::Completed
    );
    state.audit_handle.shutdown().await;

    let (events, count) =
        audit_events(&state, warmplane::audit::AuditEventType::EmissionHeld).await;
    assert_eq!(count, 2);
    assert!(events
        .iter()
        .all(|event| { event.sanitized_args.as_ref().unwrap()["body"] == "<redacted>" }));
    assert!(state.audit_store.verify_chain().await.is_valid);

    let (_, count) =
        audit_events(&state, warmplane::audit::AuditEventType::EmissionCancelled).await;
    assert_eq!(count, 1);

    let (events, count) =
        audit_events(&state, warmplane::audit::AuditEventType::ToolExecution).await;
    assert_eq!(count, 1);
    assert_eq!(
        events[0].sanitized_args.as_ref().unwrap()["body"],
        "<redacted>"
    );
    assert_eq!(
        events[0].sanitized_response.as_ref().unwrap()["taskId"],
        released.task_id
    );
    assert!(state.audit_store.verify_chain().await.is_valid);
}

#[tokio::test]
async fn failed_emission_audit_does_not_include_raw_upstream_error_text() {
    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(1));
    let policy = warmplane::daemon::Policy::from_config(Some(warmplane::config::PolicyConfig {
        redact_keys: vec!["password".to_string()],
        ..Default::default()
    }));
    let state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .policy(policy)
        .build();
    let coordinator = HoldCoordinator::default();
    let mut task_params = params(None);
    task_params.args = json!({"password": "request-secret"});
    let task = coordinator
        .accept(&state, task_params, emission(now_ms() + 50))
        .await
        .unwrap();
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(Duration::from_secs(2), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected failed emission to be dispatched");
    };
    reply
        .send(Err(warmplane::daemon::UpstreamCallError::Upstream(
            "upstream rejected password=upstream-secret".to_string(),
        )))
        .unwrap();
    assert_eq!(
        coordinator
            .wait_for_terminal(&state, &task.task_id, Duration::from_secs(1))
            .await
            .unwrap()
            .status,
        TaskStatus::Failed
    );
    state.audit_handle.shutdown().await;

    let (events, count) =
        audit_events(&state, warmplane::audit::AuditEventType::ToolExecution).await;
    assert_eq!(count, 1);
    assert_eq!(
        events[0].sanitized_args.as_ref().unwrap()["password"],
        "<redacted>"
    );
    assert_eq!(
        events[0].error_message.as_deref(),
        Some("Emission failed; see the task error for details")
    );
    assert!(!events[0]
        .error_message
        .as_deref()
        .unwrap_or_default()
        .contains("upstream-secret"));
    assert!(state.audit_store.verify_chain().await.is_valid);
}

#[tokio::test]
async fn approval_argument_edits_are_the_payload_held_and_dispatched() {
    let (server_tx, mut upstream) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(delayable_contract(1));
    let policy = warmplane::daemon::Policy::from_config(Some(warmplane::config::PolicyConfig {
        require_approval: vec!["mail.send".to_string()],
        ..Default::default()
    }));
    let state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .policy(policy)
        .tool_timeout_ms(2_000)
        .build();
    let response = warmplane::http_v1::execute::handle_call_capability(
        State(state.clone()),
        axum::Extension(None),
        None,
        HeaderMap::new(),
        Json(warmplane::http_v1::CallCapabilityRequest {
            capability_id: "mail.send".to_string(),
            args: json!({"to": "original@example.com", "body": "draft"}),
            request_id: Some("approved-emission-1".to_string()),
            idempotency_key: Some("approved-emission-key".to_string()),
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
    assert_eq!(body["task"]["status"], "input_required");

    let mut responses = std::collections::BTreeMap::new();
    responses.insert(
        "hitl_approval".to_string(),
        json!({
            "approved": true,
            "modified_args": {"to": "approved@example.com", "body": "approved body"}
        }),
    );
    let updated = warmplane::http_v1::tasks_api::handle_update_task(
        State(state.clone()),
        Path(task_id.clone()),
        Json(warmplane::http_v1::tasks_api::UpdateTaskRequest {
            input_responses: responses,
        }),
    )
    .await
    .into_response();
    assert_eq!(updated.status(), StatusCode::OK);

    let Some(ServerMsg::CallTool { params, reply, .. }) =
        tokio::time::timeout(Duration::from_secs(3), upstream.recv())
            .await
            .unwrap()
    else {
        panic!("expected approved emission to dispatch after its hold");
    };
    assert_eq!(params["to"], "approved@example.com");
    assert_eq!(params["body"], "approved body");
    reply.send(Ok(json!({"sent": true}))).unwrap();
    let completed = state
        .hold_coordinator
        .wait_for_terminal(&state, &task_id, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(completed.status, TaskStatus::Completed);
}

async fn audit_events(
    state: &AppState,
    event_type: warmplane::audit::AuditEventType,
) -> (Vec<warmplane::audit::AuditEvent>, usize) {
    state
        .audit_store
        .query(&warmplane::audit::AuditQueryFilter {
            event_type: Some(event_type),
            limit: 10,
            ..Default::default()
        })
        .await
}
