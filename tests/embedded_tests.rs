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
