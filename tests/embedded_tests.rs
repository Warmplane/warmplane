use serde_json::json;
use std::collections::HashMap;
use tempfile::NamedTempFile;

use warmplane::{
    config::{save_config, McpConfig, ServerConfig},
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
            CapabilityMeta {
                server: "offline_srv".to_string(),
                tool: "delete_record".to_string(),
                summary: "Delete record".to_string(),
                description: "Mutating tool requiring review".to_string(),
                input_schema: json!({"type": "object"}),
                tags: vec![],
                examples: vec![],
            },
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
