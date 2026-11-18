use serde_json::json;
use std::collections::HashMap;
use tempfile::NamedTempFile;

use warmplane::{
    config::{
        save_config, InverseConfig, McpConfig, PreSnapshotConfig, ReversibilityClass,
        ReversibilityConfig, ServerConfig,
    },
    daemon::{AppState, CapabilityMeta, ServerMsg},
    engine::{ControlPlaneHandle, ExecutionOptions},
    undo::{SnapshotStore, UndoEntry, UndoEntryStatus, UndoLedgerStore},
    EmbeddedWarmplane,
};

#[tokio::test]
async fn test_embedded_warmplane_start_and_shutdown() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let initial_config = McpConfig::default();
    save_config(&config_path, &initial_config).unwrap();

    let (cp, shutdown_token) = EmbeddedWarmplane::start(initial_config)
        .await
        .expect("EmbeddedWarmplane must boot cleanly");

    assert!(!shutdown_token.is_cancelled());

    let health = cp.health_status().await;
    assert!(health.catalog_version.starts_with("sha256:") || health.catalog_version.is_empty());
    assert_eq!(health.total_tool_calls, 0);

    let list = cp
        .list_capabilities(None)
        .await
        .expect("list_capabilities must succeed");
    assert_eq!(list.version, "v1");
    assert!(list.capabilities.is_empty());

    // Trigger handle shutdown
    cp.shutdown().await;
    assert!(shutdown_token.is_cancelled());
}

#[tokio::test]
async fn reversible_call_captures_pre_state_and_persists_interpolated_inverse() {
    let capabilities = priority_capabilities(true);
    let (upstream_tx, mut upstream_rx) = tokio::sync::mpsc::channel(4);
    let servers = HashMap::from([("mock".to_string(), upstream_tx)]);
    let temp = tempfile::tempdir().unwrap();
    let snapshot_store = SnapshotStore::open(temp.path().join("artifacts")).unwrap();
    let ledger = UndoLedgerStore::open(temp.path().join("undo_ledger.json")).unwrap();
    let state = AppState::builder()
        .servers(servers)
        .capabilities(capabilities)
        .snapshot_store(snapshot_store.clone())
        .undo_ledger(ledger.clone())
        .build();
    let handle = ControlPlaneHandle::new(state);
    let undo_state = handle.state().clone();
    let mut call_options = ExecutionOptions::default();
    call_options.context = Some(warmplane::context::RequestContext {
        work_item_id: Some("work-drift-test".to_string()),
        ..Default::default()
    });
    let call = tokio::spawn(async move {
        handle
            .call_capability(
                "mock.set_priority",
                json!({"id": "ISSUE-7", "priority": "high"}),
                call_options,
            )
            .await
    });

    match upstream_rx.recv().await.unwrap() {
        ServerMsg::CallTool {
            name,
            params,
            reply,
            ..
        } => {
            assert_eq!(name, "get_issue");
            assert_eq!(params["id"], "ISSUE-7");
            reply.send(Ok(json!({"priority": "low"}))).unwrap();
        }
        _ => panic!("expected pre-snapshot read"),
    }
    match upstream_rx.recv().await.unwrap() {
        ServerMsg::CallTool {
            name,
            params,
            reply,
            ..
        } => {
            assert_eq!(name, "set_priority");
            reply.send(Ok(json!({"ok": true}))).unwrap();
            assert_eq!(params["priority"], "high");
        }
        _ => panic!("expected forward mutation"),
    }

    let Some(ServerMsg::CallTool { name, reply, .. }) = upstream_rx.recv().await else {
        panic!("expected post-execution state check");
    };
    assert_eq!(name, "get_issue");
    reply.send(Ok(json!({"priority": "high"}))).unwrap();

    let response = call.await.unwrap();
    assert!(response.ok, "response error: {:?}", response.error);
    let entries = ledger.entries().await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].status, UndoEntryStatus::Armed);
    assert_eq!(
        entries[0].resolved_inverse_args,
        json!({"id": "ISSUE-7", "priority": "low"})
    );
    let hash = entries[0].snapshot_artifact_hash.as_ref().unwrap();
    assert_eq!(snapshot_store.get(hash).unwrap(), json!("low"));

    let policy = warmplane::daemon::Policy::default();
    let profile = warmplane::context::ProfileContext::default();
    let undo_id = entries[0].undo_id.clone();
    let operator_state = undo_state.clone();
    let undo = tokio::spawn(async move {
        warmplane::undo::execute_operator_undo(
            &operator_state,
            "default",
            &undo_id,
            warmplane::undo::FiberExecutionScope {
                policy: &policy,
                profile: &profile,
                context: None,
                authorization_expires_at_epoch_secs: None,
                trace_id: "drift-check-test",
                operator_id: Some("test-operator"),
                force_expired_reason: None,
            },
            false,
            None,
        )
        .await
        .unwrap()
    });
    let Some(ServerMsg::CallTool { name, reply, .. }) = upstream_rx.recv().await else {
        panic!("expected pre-undo state check");
    };
    assert_eq!(name, "get_issue");
    reply.send(Ok(json!({"priority": "manual-edit"}))).unwrap();
    let outcome = undo.await.unwrap();
    assert!(!outcome.ok);
    assert_eq!(outcome.error_code.as_deref(), Some("UNDO_STATE_DRIFT"));
    assert_eq!(outcome.entry_status, Some(UndoEntryStatus::Armed));
    assert!(
        upstream_rx.try_recv().is_err(),
        "inverse must not dispatch after drift"
    );

    let unwind_state = undo_state.clone();
    let unwind_policy = warmplane::daemon::Policy::default();
    let unwind_profile = warmplane::context::ProfileContext::default();
    let unwind = tokio::spawn(async move {
        warmplane::undo::unwind_fiber(
            &unwind_state,
            "default",
            "work-drift-test",
            warmplane::undo::FiberExecutionScope {
                policy: &unwind_policy,
                profile: &unwind_profile,
                context: None,
                authorization_expires_at_epoch_secs: None,
                trace_id: "automatic-drift-unwind-test",
                operator_id: None,
                force_expired_reason: None,
            },
        )
        .await
    });
    let Some(ServerMsg::CallTool { name, reply, .. }) = upstream_rx.recv().await else {
        panic!("expected automatic pre-undo state check");
    };
    assert_eq!(name, "get_issue");
    reply.send(Ok(json!({"priority": "manual-edit"}))).unwrap();
    let outcomes = unwind.await.unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].error_code.as_deref(), Some("UNDO_STATE_DRIFT"));
    assert_eq!(outcomes[0].entry_status, Some(UndoEntryStatus::Armed));
    assert!(
        upstream_rx.try_recv().is_err(),
        "automatic unwind must not dispatch an inverse after drift"
    );
}

#[tokio::test]
async fn inverse_upstream_error_text_is_not_written_to_the_audit_chain() {
    let directory = tempfile::tempdir().unwrap();
    let ledger = UndoLedgerStore::open(directory.path().join("undo_ledger.json")).unwrap();
    let expires_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 60_000;
    ledger
        .register(UndoEntry {
            undo_id: "undo-audit-error-test".to_string(),
            operation_id: Some("forward-operation".to_string()),
            work_item_id: None,
            batch_id: None,
            tenant_id: "default".to_string(),
            profile_id: None,
            actor_id: Some("test-actor".to_string()),
            forward_capability: "mock.forward".to_string(),
            reversibility_class: "reversible".to_string(),
            resolved_inverse_capability: "mock.inverse".to_string(),
            resolved_inverse_args: json!({"password": "private-value"}),
            change_preview: vec!["password".to_string()],
            snapshot_artifact_hash: None,
            pre_undo_check: None,
            expected_post_snapshot_hash: None,
            created_at_ms: expires_at_ms - 60_000,
            expires_at_ms,
            status: UndoEntryStatus::Armed,
        })
        .await
        .unwrap();

    let inverse = CapabilityMeta::new(
        "mock",
        "inverse",
        "Apply inverse",
        "Apply inverse",
        json!({"type": "object"}),
    );
    let (server_tx, mut server_rx) = tokio::sync::mpsc::channel(4);
    let state = AppState::builder()
        .servers(HashMap::from([("mock".to_string(), server_tx)]))
        .capabilities(HashMap::from([("mock.inverse".to_string(), inverse)]))
        .undo_ledger(ledger.clone())
        .build();
    let policy = warmplane::daemon::Policy::default();
    let profile = warmplane::context::ProfileContext::default();
    let undo_state = state.clone();
    let execution = tokio::spawn(async move {
        warmplane::undo::execute_operator_undo(
            &undo_state,
            "default",
            "undo-audit-error-test",
            warmplane::undo::FiberExecutionScope {
                policy: &policy,
                profile: &profile,
                context: None,
                authorization_expires_at_epoch_secs: None,
                trace_id: "audit-error-test",
                operator_id: Some("operator"),
                force_expired_reason: None,
            },
            false,
            None,
        )
        .await
        .unwrap()
    });
    let Some(ServerMsg::CallTool { reply, .. }) = server_rx.recv().await else {
        panic!("expected inverse tool call");
    };
    reply
        .send(Err(warmplane::daemon::UpstreamCallError::Upstream(
            "upstream rejected password=upstream-private-value".to_string(),
        )))
        .unwrap();
    assert!(!execution.await.unwrap().ok);
    state.audit_handle.shutdown().await;

    let filter = warmplane::audit::AuditQueryFilter {
        event_type: Some(warmplane::audit::AuditEventType::UndoFailed),
        limit: 10,
        ..Default::default()
    };
    let (events, count) = state.audit_store.query(&filter).await;
    assert_eq!(count, 1);
    let serialized = serde_json::to_string(&events[0]).unwrap();
    assert!(!serialized.contains("upstream-private-value"));
    assert!(!serialized.contains("private-value"));
}

#[tokio::test]
async fn failed_pre_snapshot_prevents_the_forward_mutation() {
    let capabilities = priority_capabilities(false);
    let (upstream_tx, mut upstream_rx) = tokio::sync::mpsc::channel(4);
    let temp = tempfile::tempdir().unwrap();
    let snapshot_store = SnapshotStore::open(temp.path().join("artifacts")).unwrap();
    let ledger = UndoLedgerStore::open(temp.path().join("undo_ledger.json")).unwrap();
    let state = AppState::builder()
        .servers(HashMap::from([("mock".to_string(), upstream_tx)]))
        .capabilities(capabilities)
        .snapshot_store(snapshot_store)
        .undo_ledger(ledger.clone())
        .build();
    let handle = ControlPlaneHandle::new(state);
    let call = tokio::spawn(async move {
        handle
            .call_capability(
                "mock.set_priority",
                json!({"id": "ISSUE-7", "priority": "high"}),
                ExecutionOptions::default(),
            )
            .await
    });

    let Some(ServerMsg::CallTool { name, reply, .. }) = upstream_rx.recv().await else {
        panic!("expected the pre-snapshot read");
    };
    assert_eq!(name, "get_issue");
    reply
        .send(Err(warmplane::daemon::UpstreamCallError::Upstream(
            "snapshot read failed".to_string(),
        )))
        .unwrap();

    let response = call.await.unwrap();
    assert!(!response.ok);
    assert_eq!(response.error.unwrap().code, "PRE_SNAPSHOT_FAILED");
    assert!(upstream_rx.try_recv().is_err());
    assert!(ledger.entries().await.is_empty());
}

#[tokio::test]
async fn compensable_daily_limit_requires_approval_before_another_forward_call() {
    let mut capability = CapabilityMeta::new(
        "mock",
        "charge",
        "Charge account",
        "Charge account",
        json!({"type": "object"}),
    );
    capability.reversibility = Some(ReversibilityConfig {
        class: ReversibilityClass::Compensable,
        hold_window_secs: None,
        ttl_secs: Some(3600),
        daily_limit: Some(1),
        pre_snapshot: None,
        pre_undo_check: None,
        test_target: None,
        inverse: Some(Box::new(InverseConfig {
            capability: "mock.refund".to_string(),
            args: json!({"charge_id": "${result.charge_id}"}),
        })),
    });
    let (upstream_tx, mut upstream_rx) = tokio::sync::mpsc::channel(8);
    let temp = tempfile::tempdir().unwrap();
    let ledger = UndoLedgerStore::open(temp.path().join("undo_ledger.json")).unwrap();
    let state = AppState::builder()
        .servers(HashMap::from([("mock".to_string(), upstream_tx)]))
        .capabilities(HashMap::from([("mock.charge".to_string(), capability)]))
        .undo_ledger(ledger.clone())
        .build();
    let handle = ControlPlaneHandle::new(state.clone());

    let first_handle = handle.clone();
    let first_call = tokio::spawn(async move {
        first_handle
            .call_capability(
                "mock.charge",
                json!({"amount": 10}),
                ExecutionOptions::default().with_async_task(true),
            )
            .await
    });
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(std::time::Duration::from_secs(2), upstream_rx.recv())
            .await
            .unwrap()
    else {
        panic!("first compensable call should dispatch under the daily limit");
    };
    reply.send(Ok(json!({"charge_id": "charge-1"}))).unwrap();
    let first_response = first_call.await.unwrap();
    assert!(first_response.ok);
    let first_task_id = first_response.data.unwrap()["taskId"]
        .as_str()
        .unwrap()
        .to_string();
    let first_terminal = state
        .hold_coordinator
        .wait_for_terminal(&state, &first_task_id, std::time::Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(
        first_terminal.status,
        warmplane::tasks::TaskStatus::Completed,
        "task failed: {first_terminal:#?}"
    );
    assert_eq!(ledger.entries().await.len(), 1);

    let second = handle
        .call_capability(
            "mock.charge",
            json!({"amount": 20}),
            ExecutionOptions::default()
                .with_async_task(true)
                .with_request_id("over-limit-charge")
                .with_idempotency_key("over-limit-charge-key"),
        )
        .await;
    assert!(second.ok);
    let task_id = second.data.unwrap()["taskId"].as_str().unwrap().to_string();
    let pending = state.task_registry.get_task(&task_id).await.unwrap();
    assert_eq!(pending.status, warmplane::tasks::TaskStatus::InputRequired);
    assert!(upstream_rx.try_recv().is_err());

    let responses = std::collections::BTreeMap::from([(
        "hitl_approval".to_string(),
        json!({"approved": true}),
    )]);
    assert!(state
        .task_registry
        .update_task(&task_id, responses)
        .await
        .unwrap());
    let Some(ServerMsg::CallTool { reply, .. }) =
        tokio::time::timeout(std::time::Duration::from_secs(2), upstream_rx.recv())
            .await
            .unwrap()
    else {
        panic!("approved over-limit compensable call should dispatch");
    };
    reply.send(Ok(json!({"charge_id": "charge-2"}))).unwrap();
    assert_eq!(
        state
            .hold_coordinator
            .wait_for_terminal(&state, &task_id, std::time::Duration::from_secs(2))
            .await
            .unwrap()
            .status,
        warmplane::tasks::TaskStatus::Completed
    );
    assert_eq!(ledger.entries().await.len(), 2);
    assert!(upstream_rx.try_recv().is_err());
}

#[tokio::test]
async fn test_embedded_warmplane_call_capability_lifecycle() {
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "offline_srv".to_string(),
        ServerConfig {
            command: None,
            args: vec![],
            env: HashMap::new(),
            url: Some("http://127.0.0.1:59998/mcp".to_string()),
            auth: None,
            protocol_version: None,
            allow_stateless: Some(true),
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let config = McpConfig {
        mcp_servers,
        ..Default::default()
    };

    let (cp, _shutdown) = EmbeddedWarmplane::start(config)
        .await
        .expect("EmbeddedWarmplane must boot in degraded mode for unreachable server");

    // Call unknown capability returns typed failure envelope
    let env = cp
        .call_capability(
            "unknown_cap",
            json!({}),
            warmplane::engine::ExecutionOptions::default().with_request_id("req-123"),
        )
        .await;

    assert!(!env.ok);
    assert_eq!(env.request_id, Some("req-123".to_string()));

    let err = env.error.expect("error must be populated");
    assert_eq!(err.code, "TOOL_NOT_FOUND");
    assert!(!err.retryable);

    // Describe unknown capability returns typed failure envelope
    let desc_env = cp.describe_capability("unknown_cap", None).await;
    assert!(!desc_env.ok);
    assert_eq!(desc_env.error.expect("error").code, "TOOL_NOT_FOUND");

    // Batch call with invalid step returns typed failure
    let batch_res = cp
        .batch_call(
            vec![warmplane::batch_executor::BatchStep {
                id: "step1".to_string(),
                capability_id: "missing_cap".to_string(),
                args: json!({}),
                continue_on_error: false,
            }],
            Some("batch-req-1".to_string()),
            None,
            None,
        )
        .await;

    assert!(!batch_res.ok);
    assert_eq!(batch_res.results.len(), 1);
    assert_eq!(batch_res.results[0].id, "step1");
    assert!(!batch_res.results[0].ok);

    cp.shutdown().await;
}

#[tokio::test]
async fn test_embedded_warmplane_start_from_path() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let config = McpConfig::default();
    save_config(&config_path, &config).unwrap();

    let (cp, _shutdown) = EmbeddedWarmplane::start_from_path(&config_path)
        .await
        .expect("EmbeddedWarmplane::start_from_path must succeed");

    let list = cp.list_capabilities(None).await.unwrap();
    assert_eq!(list.version, "v1");

    cp.shutdown().await;
}

#[tokio::test]
async fn test_embedded_task_management_lifecycle() {
    use std::collections::BTreeMap;
    use warmplane::config::{PolicyConfig, ServerConfig};
    use warmplane::daemon::CapabilityMeta;
    use warmplane::engine::{ExecutionOptions, TaskStatus};

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "offline_srv".to_string(),
        ServerConfig {
            command: None,
            args: vec![],
            env: HashMap::new(),
            url: Some("http://127.0.0.1:59997/mcp".to_string()),
            auth: None,
            protocol_version: None,
            allow_stateless: Some(true),
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let policy = PolicyConfig {
        require_approval: vec!["offline_srv/*".to_string()],
        approval_timeout_secs: Some(30),
        ..Default::default()
    };

    let config = McpConfig {
        mcp_servers,
        policy: Some(policy),
        state: Some(warmplane::config::StateConfig {
            enabled: false,
            dir: None,
        }),
        ..Default::default()
    };

    let (cp, _shutdown) = EmbeddedWarmplane::start(config)
        .await
        .expect("EmbeddedWarmplane must boot cleanly");

    // Manually register capability on app_state for testing
    {
        let mut caps = cp.state().capabilities.write().await;
        caps.insert(
            "offline_srv/delete_record".to_string(),
            CapabilityMeta::new(
                "offline_srv",
                "delete_record",
                "Delete record",
                "Mutating tool requiring review",
                json!({"type": "object"}),
            ),
        );
    }

    // Initially task registry is empty
    let initial_tasks = cp.list_tasks().await;
    assert!(initial_tasks.is_empty());

    // Call capability requiring approval with async_task: true
    let async_res = cp
        .call_capability(
            "offline_srv/delete_record",
            json!({"id": "rec-99"}),
            ExecutionOptions::default()
                .with_request_id("req-task-test")
                .with_async_task(true),
        )
        .await;

    assert!(async_res.ok);
    let task_payload = async_res.data.expect("data must contain task info");
    let task_id = task_payload
        .get("taskId")
        .and_then(|v| v.as_str())
        .expect("taskId must be present");

    // Query task list via embedded handle
    let tasks = cp.list_tasks().await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, task_id);
    assert_eq!(tasks[0].status, TaskStatus::InputRequired);
    assert!(tasks[0].input_requests.is_some());

    // Query single task via embedded handle
    let get_env = cp.get_task(task_id).await;
    assert!(get_env.ok);
    let task_resp = get_env.data.expect("task response");
    assert_eq!(task_resp.task_id, task_id);
    assert_eq!(task_resp.status, TaskStatus::InputRequired);

    // Cancel task cooperatively via embedded handle
    let cancel_env = cp
        .cancel_task(task_id, Some("Operator cancelled test run".to_string()))
        .await;
    assert!(cancel_env.ok);
    assert!(cancel_env.data.unwrap_or(false));

    // Verify task status transitioned to Cancelled
    let updated_task = cp.get_task(task_id).await;
    assert_eq!(
        updated_task.data.expect("task").status,
        TaskStatus::Cancelled
    );

    // Repeated cancel on already cancelled task is rejected cleanly
    let repeat_cancel = cp.cancel_task(task_id, None).await;
    assert!(!repeat_cancel.ok);
    assert_eq!(
        repeat_cancel.error.expect("error").code,
        "TASK_CANCEL_REJECTED"
    );

    // Non-existent task get
    let missing_env = cp.get_task("task-non-existent").await;
    assert!(!missing_env.ok);
    assert_eq!(missing_env.error.expect("error").code, "TASK_NOT_FOUND");

    // Update non-existent task
    let mut fake_responses = BTreeMap::new();
    fake_responses.insert("operator_approval".to_string(), json!({"approved": true}));
    let missing_update = cp.update_task("task-non-existent", fake_responses).await;
    assert!(!missing_update.ok);
    assert_eq!(
        missing_update.error.expect("error").code,
        "TASK_UPDATE_REJECTED"
    );

    cp.shutdown().await;
}

#[tokio::test]
async fn test_embedded_async_task_hitl_and_completion_lifecycle() {
    use std::collections::BTreeMap;
    use tokio::sync::mpsc;
    use warmplane::config::{PolicyConfig, ServerConfig};
    use warmplane::daemon::{CapabilityMeta, ServerMsg};
    use warmplane::engine::{ExecutionOptions, TaskStatus};

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "mock_srv".to_string(),
        ServerConfig {
            command: None,
            args: vec![],
            env: HashMap::new(),
            url: Some("http://127.0.0.1:59996/mcp".to_string()),
            auth: None,
            protocol_version: None,
            allow_stateless: Some(true),
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let policy = PolicyConfig {
        require_approval: vec!["mock_srv/sensitive_action".to_string()],
        approval_timeout_secs: Some(30),
        ..Default::default()
    };

    let config = McpConfig {
        mcp_servers,
        policy: Some(policy),
        state: Some(warmplane::config::StateConfig {
            enabled: false,
            dir: None,
        }),
        ..Default::default()
    };

    let (cp, _shutdown) = EmbeddedWarmplane::start(config)
        .await
        .expect("EmbeddedWarmplane must boot cleanly");

    // Install mock server mailbox
    let (tx, mut rx) = mpsc::channel(4);
    {
        let mut servers = cp.state().servers.write().await;
        servers.insert("mock_srv".to_string(), tx);

        let mut caps = cp.state().capabilities.write().await;
        caps.insert(
            "mock_srv/sensitive_action".to_string(),
            CapabilityMeta::new(
                "mock_srv",
                "sensitive_action",
                "Sensitive Action",
                "Requires approval",
                json!({"type": "object"}),
            ),
        );
        caps.insert(
            "mock_srv/fast_action".to_string(),
            CapabilityMeta::new(
                "mock_srv",
                "fast_action",
                "Fast Action",
                "Direct async execution",
                json!({"type": "object"}),
            ),
        );
    }

    // 1. Call sensitive capability with async_task: true -> should be InputRequired
    let async_res = cp
        .call_capability(
            "mock_srv/sensitive_action",
            json!({"param": "val"}),
            ExecutionOptions::default().with_async_task(true),
        )
        .await;

    assert!(async_res.ok);
    let task_payload = async_res.data.expect("task payload");
    let task_id = task_payload["taskId"].as_str().unwrap().to_string();
    assert_eq!(task_payload["status"], "input_required");

    // 2. Submit HITL approval via embedded update_task
    let mut responses = BTreeMap::new();
    responses.insert(
        "hitl_approval".to_string(),
        json!({"approved": true, "modified_args": {"param": "val_approved"}}),
    );
    let update_env = cp.update_task(&task_id, responses).await;
    assert!(update_env.ok);
    assert!(update_env.data.unwrap_or(false));

    // 3. Upstream mock actor receives call and responds
    let msg = rx.recv().await.expect("worker must send CallTool");
    match msg {
        ServerMsg::CallTool {
            name,
            params,
            reply,
            ..
        } => {
            assert_eq!(name, "sensitive_action");
            assert_eq!(params["param"], "val_approved");
            let _ = reply.send(Ok(json!({"action_result": "success_123"})));
        }
        _ => panic!("Expected CallTool message"),
    }

    // 4. Poll get_task until completed
    let mut completed = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let get_env = cp.get_task(&task_id).await;
        if let Some(task) = get_env.data {
            if task.status == TaskStatus::Completed {
                assert_eq!(task.result.expect("result")["action_result"], "success_123");
                completed = true;
                break;
            }
        }
    }
    assert!(completed, "Task should reach Completed status with result");

    // 5. Test direct async task (no HITL approval)
    let fast_res = cp
        .call_capability(
            "mock_srv/fast_action",
            json!({"count": 42}),
            ExecutionOptions::default().with_async_task(true),
        )
        .await;

    assert!(fast_res.ok);
    let fast_payload = fast_res.data.expect("fast task payload");
    let fast_task_id = fast_payload["taskId"].as_str().unwrap().to_string();
    assert_eq!(fast_payload["status"], "working");

    let fast_msg = rx
        .recv()
        .await
        .expect("worker must send CallTool for fast action");
    match fast_msg {
        ServerMsg::CallTool {
            name,
            params,
            reply,
            ..
        } => {
            assert_eq!(name, "fast_action");
            assert_eq!(params["count"], 42);
            let _ = reply.send(Ok(json!({"count_doubled": 84})));
        }
        _ => panic!("Expected CallTool message"),
    }

    let mut fast_completed = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let get_env = cp.get_task(&fast_task_id).await;
        if let Some(task) = get_env.data {
            if task.status == TaskStatus::Completed {
                assert_eq!(task.result.expect("result")["count_doubled"], 84);
                fast_completed = true;
                break;
            }
        }
    }
    assert!(
        fast_completed,
        "Fast async task should reach Completed status with result"
    );

    cp.shutdown().await;
}

fn priority_capabilities(drift_check: bool) -> HashMap<String, CapabilityMeta> {
    let mut forward = CapabilityMeta::new(
        "mock",
        "set_priority",
        "Set issue priority",
        "Set issue priority",
        json!({"type": "object"}),
    );
    forward.reversibility = Some(ReversibilityConfig {
        class: ReversibilityClass::Reversible,
        hold_window_secs: None,
        ttl_secs: Some(60),
        daily_limit: None,
        pre_snapshot: Some(Box::new(PreSnapshotConfig {
            capability: "mock.get_issue".to_string(),
            args: json!({"id": "${args.id}"}),
            capture_json_path: Some("$.priority".to_string()),
        })),
        pre_undo_check: drift_check.then(|| {
            Box::new(PreSnapshotConfig {
                capability: "mock.get_issue".to_string(),
                args: json!({"id": "${args.id}"}),
                capture_json_path: Some("$.priority".to_string()),
            })
        }),
        test_target: None,
        inverse: Some(Box::new(InverseConfig {
            capability: "mock.set_priority".to_string(),
            args: json!({"id": "${args.id}", "priority": "${snapshot}"}),
        })),
    });
    let getter = CapabilityMeta::new(
        "mock",
        "get_issue",
        "Get issue",
        "Get issue",
        json!({"type": "object"}),
    );
    HashMap::from([
        ("mock.set_priority".to_string(), forward),
        ("mock.get_issue".to_string(), getter),
    ])
}
