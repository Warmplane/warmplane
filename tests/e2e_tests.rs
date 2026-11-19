// Rust guideline compliant 2026-11-18

//! Comprehensive end-to-end integration tests for Warmplane (`M-CANONICAL-DOCS`).
//!
//! Exercises real binary execution over stdio, live TCP daemon listeners, real-time SSE streaming,
//! OAuth2 mock provider discovery/refresh, and configuration hot-reloading.

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use futures::StreamExt;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};
use tempfile::NamedTempFile;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpListener,
    process::Command,
    sync::RwLock,
    time::timeout,
};

use warmplane::{
    config::{save_config, McpConfig, ServerConfig},
    daemon::server::{build_router, initialize_state},
    oauth2::{DiscoveryMetadata, OAuth2ClientState, OAuth2TokenState, OAuthRegistry},
};

mod common;
use common::read_jsonrpc_until_id;

// ============================================================================
// Test 1: Full Stdio MCP Server Facade Protocol Handshake
// ============================================================================

#[tokio::test]
async fn test_e2e_stdio_mcp_server_facade_protocol() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let initial_config = McpConfig::default();
    save_config(&config_path, &initial_config).unwrap();

    let bin_path = env!("CARGO_BIN_EXE_warmplane");
    let mut child = Command::new(bin_path)
        .arg("mcp-server")
        .arg("--config")
        .arg(&config_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn warmplane mcp-server process");

    let stdin = child.stdin.as_mut().expect("child stdin must be captured");
    let stdout = child.stdout.take().expect("child stdout must be captured");
    let mut reader = BufReader::new(stdout);

    // 1. Send initialize request
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {
                "name": "e2e-test-client",
                "version": "1.0.0"
            }
        }
    });

    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let init_resp = read_jsonrpc_until_id(&mut reader, 1).await;
    assert_eq!(init_resp["id"], 1);
    assert!(init_resp["result"]["capabilities"]["tools"].is_object());

    // 2. Send initialized notification
    let initialized_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&initialized_notif).unwrap()).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    // 3. Query tools/list
    let tools_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&tools_req).unwrap()).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let tools_resp = read_jsonrpc_until_id(&mut reader, 2).await;
    assert_eq!(tools_resp["id"], 2);
    let tools = tools_resp["result"]["tools"]
        .as_array()
        .expect("tools array must exist");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(tool_names.contains(&"capabilities_list"));
    assert!(tool_names.contains(&"capability_call"));
    assert!(tool_names.contains(&"resources_list"));
    assert!(tool_names.contains(&"prompts_list"));

    // 4. Call capabilities_list facade tool
    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "capabilities_list",
            "arguments": {}
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&call_req).unwrap()).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let call_resp = read_jsonrpc_until_id(&mut reader, 3).await;
    assert_eq!(call_resp["id"], 3);
    assert!(call_resp["result"]["content"].is_array());
    assert!(call_resp["result"]["structuredContent"].is_object());

    let completion = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "completion/complete",
        "params": {
            "ref": {"type": "ref/prompt", "name": "example"},
            "argument": {"name": "query", "value": "ex"}
        }
    });
    stdin
        .write_all(format!("{completion}\n").as_bytes())
        .await
        .unwrap();
    let completion_resp = read_jsonrpc_until_id(&mut reader, 4).await;
    assert_eq!(completion_resp["error"]["code"], -32601);

    // Clean termination upon closing stdin.
    drop(child.stdin.take());
    let exit_status = timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("timeout waiting for child process exit")
        .unwrap();

    assert!(exit_status.success());
}

// ============================================================================
// Test 2: Real-Time SSE Event Streaming & Live Dynamic Server Mount/Unmount
// ============================================================================

#[tokio::test]
async fn test_e2e_realtime_sse_event_streaming() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let initial_config = McpConfig::default();
    let (_state, port, server_task) = start_test_daemon(initial_config, &config_path).await;

    let client = reqwest::Client::new();
    let sse_url = format!("http://127.0.0.1:{}/v1/resources/updates", port);

    let sse_res = client
        .get(&sse_url)
        .send()
        .await
        .expect("failed to connect to SSE stream endpoint");

    assert_eq!(sse_res.status(), StatusCode::OK);
    let mut stream = sse_res.bytes_stream();

    // Dynamically mount an upstream server via REST API
    let mount_url = format!("http://127.0.0.1:{}/v1/config/servers", port);
    let mount_payload = json!({
        "name": "dynamic_test_srv",
        "server": {
            "command": "echo",
            "args": ["running"]
        }
    });

    let mount_res = client
        .post(&mount_url)
        .json(&mount_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(mount_res.status(), StatusCode::OK);

    // Read the SSE event stream for the dynamic mount notification
    let mut received_event = false;
    let read_future = async {
        while let Some(chunk_res) = stream.next().await {
            if let Ok(bytes) = chunk_res {
                let text = String::from_utf8_lossy(&bytes);
                if text.contains("server://dynamic_test_srv") || text.contains("resource_update") {
                    received_event = true;
                    break;
                }
            }
        }
    };

    timeout(Duration::from_secs(5), read_future)
        .await
        .expect("timeout waiting for real-time SSE event");

    assert!(
        received_event,
        "SSE stream must yield live mount notification"
    );

    // Unmount server via REST API
    let unmount_url = format!(
        "http://127.0.0.1:{}/v1/config/servers/dynamic_test_srv",
        port
    );
    let unmount_res = client.delete(&unmount_url).send().await.unwrap();
    assert_eq!(unmount_res.status(), StatusCode::OK);

    server_task.abort();
}

// ============================================================================
// Test 3: Configuration Hot-Reloading E2E
// ============================================================================

#[tokio::test]
async fn test_e2e_config_file_hot_reload() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "server_alpha".to_string(),
        ServerConfig {
            command: Some("echo".to_string()),
            args: vec!["alpha".to_string()],
            env: HashMap::new(),
            url: None,
            auth: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let initial_config = McpConfig {
        mcp_servers,
        ..Default::default()
    };
    let (_state, port, server_task) = start_test_daemon(initial_config, &config_path).await;

    let client = reqwest::Client::new();

    // Verify initial config reports server_alpha
    let config_res = client
        .get(format!("http://127.0.0.1:{}/v1/config", port))
        .send()
        .await
        .unwrap();
    let config_json: Value = config_res.json().await.unwrap();
    assert!(config_json["config"]["mcpServers"]["server_alpha"].is_object());

    // Update config on disk: replace server_alpha with server_beta
    let mut updated_servers = HashMap::new();
    updated_servers.insert(
        "server_beta".to_string(),
        ServerConfig {
            command: Some("echo".to_string()),
            args: vec!["beta".to_string()],
            env: HashMap::new(),
            url: None,
            auth: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let updated_config = McpConfig {
        mcp_servers: updated_servers,
        ..Default::default()
    };
    save_config(&config_path, &updated_config).unwrap();

    // Trigger dynamic reload
    let reload_res = client
        .post(format!("http://127.0.0.1:{}/v1/config/reload", port))
        .send()
        .await
        .unwrap();

    assert_eq!(reload_res.status(), StatusCode::OK);
    let reload_json: Value = reload_res.json().await.unwrap();
    assert!(reload_json["mounted"]
        .as_array()
        .unwrap()
        .contains(&json!("server_beta")));
    assert!(reload_json["unmounted"]
        .as_array()
        .unwrap()
        .contains(&json!("server_alpha")));

    // Verify updated state reflection
    let config_res2 = client
        .get(format!("http://127.0.0.1:{}/v1/config", port))
        .send()
        .await
        .unwrap();
    let config_json2: Value = config_res2.json().await.unwrap();
    assert!(config_json2["config"]["mcpServers"]["server_beta"].is_object());
    assert!(config_json2["config"]["mcpServers"]["server_alpha"].is_null());

    server_task.abort();
}

// ============================================================================
// Test 4: OAuth2 Authorization Server Mock Round-Trip & Silent Token Refresh
// ============================================================================

#[derive(Clone, Default)]
struct MockOAuthState {
    refresh_calls: Arc<AtomicU32>,
    data_calls: Arc<AtomicU32>,
}

#[tokio::test]
async fn test_e2e_oauth2_mock_provider_flow_and_refresh() {
    let mock_state = MockOAuthState::default();

    // 1. Spin up mock OAuth2 upstream server with 401 step-up and token refresh
    let mock_app = Router::new()
        .route(
            "/token",
            post({
                let state_clone = mock_state.clone();
                move |_body: axum::body::Bytes| {
                    let state = state_clone.clone();
                    async move {
                        state.refresh_calls.fetch_add(1, Ordering::SeqCst);
                        Json(json!({
                            "access_token": "fresh_access_token_777",
                            "token_type": "Bearer",
                            "expires_in": 3600,
                            "refresh_token": "fresh_refresh_token_888",
                            "scope": "read write"
                        }))
                    }
                }
            }),
        )
        .route(
            "/api/data",
            post({
                let state_clone = mock_state.clone();
                move |headers: HeaderMap| {
                    let state = state_clone.clone();
                    async move {
                        state.data_calls.fetch_add(1, Ordering::SeqCst);
                        let auth = headers
                            .get("authorization")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("");

                        // Reject initial expired token with 401, accept refreshed token
                        if auth == "Bearer fresh_access_token_777" {
                            (
                                StatusCode::OK,
                                Json(json!({"status": "success", "content": "protected_payload"})),
                            )
                                .into_response()
                        } else {
                            (
                                StatusCode::UNAUTHORIZED,
                                Json(json!({"error": "invalid_token", "error_description": "Token expired"})),
                            )
                                .into_response()
                        }
                    }
                }
            }),
        )
        .with_state(mock_state.clone());

    let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_port = upstream_listener.local_addr().unwrap().port();
    let upstream_task = tokio::spawn(async move {
        axum::serve(upstream_listener, mock_app).await.unwrap();
    });

    let upstream_base_url = format!("http://127.0.0.1:{}", upstream_port);

    // 2. Set up Warmplane OAuthRegistry with an expired token
    let temp_token_file = NamedTempFile::new().unwrap();
    let token_storage_path = temp_token_file.path().to_str().unwrap().to_string();

    let registry = OAuthRegistry::open_or_create(&token_storage_path);

    let mut scopes = HashSet::new();
    scopes.insert("read".to_string());

    let expired_token = OAuth2TokenState {
        access_token: "expired_token_111".to_string(),
        refresh_token: Some("valid_refresh_token_222".to_string()),
        scopes: scopes.clone(),
    };

    let client_state = OAuth2ClientState {
        server_id: "mock_oauth_srv".to_string(),
        client_id: "warmplane_client".to_string(),
        _authorization_server_url: upstream_base_url.clone(),
        scopes: Arc::new(RwLock::new(scopes)),
        token_state: Arc::new(RwLock::new(Some(expired_token.clone()))),
        discovery: DiscoveryMetadata {
            authorization_endpoint: format!("{}/authorize", upstream_base_url),
            token_endpoint: format!("{}/token", upstream_base_url),
            issuer: upstream_base_url.clone(),
        },
        client_metadata_url: None,
        remote_base_url: upstream_base_url.clone(),
    };

    {
        let mut clients_guard = registry.clients.write().await;
        clients_guard.insert("mock_oauth_srv".to_string(), client_state);
    }

    // Start OAuth proxy server
    let proxy_port = warmplane::oauth2::start_oauth_proxy_server(
        registry.clone(),
        tokio_util::sync::CancellationToken::new(),
    )
    .await
    .expect("OAuth proxy server must start");

    // 3. Send request through the Warmplane OAuth proxy
    let proxy_url = format!(
        "http://127.0.0.1:{}/proxy/mock_oauth_srv/api/data",
        proxy_port
    );
    let http_client = reqwest::Client::new();

    let proxy_resp = http_client
        .post(&proxy_url)
        .json(&json!({"query": "get_data"}))
        .send()
        .await
        .expect("Proxy request must succeed");

    assert_eq!(proxy_resp.status(), StatusCode::OK);
    let body: Value = proxy_resp.json().await.unwrap();
    assert_eq!(body["status"], "success");
    assert_eq!(body["content"], "protected_payload");

    // Verify token was refreshed and updated in storage
    assert_eq!(mock_state.refresh_calls.load(Ordering::SeqCst), 1);
    assert_eq!(mock_state.data_calls.load(Ordering::SeqCst), 2); // 1st attempt (401) + 2nd attempt (200)

    let persisted = registry
        .get_saved_token("mock_oauth_srv")
        .await
        .expect("refreshed token must be persisted");
    assert_eq!(persisted.access_token, "fresh_access_token_777");

    upstream_task.abort();
}

// ============================================================================
// Test 5: Supervisor Crash Detection and Degraded Status Recording
// ============================================================================

#[tokio::test]
async fn test_e2e_supervisor_crash_detection_and_degraded_status() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mut mcp_servers = HashMap::new();
    // Server with a command that immediately fails / terminates
    mcp_servers.insert(
        "failing_server".to_string(),
        ServerConfig {
            command: Some("false".to_string()),
            args: vec![],
            env: HashMap::new(),
            url: None,
            auth: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let initial_config = McpConfig {
        mcp_servers,
        ..Default::default()
    };
    save_config(&config_path, &initial_config).unwrap();

    let state = initialize_state(initial_config, &config_path)
        .await
        .unwrap();

    // Give the background supervisor loop time to record failure
    tokio::time::sleep(Duration::from_millis(150)).await;

    let statuses = state.server_statuses.read().await;
    let status = statuses
        .get("failing_server")
        .expect("server status must be recorded for failing_server");

    assert_eq!(status["status"], "degraded");
    assert!(status["error"].is_string());
}

// ============================================================================
// Test 6: HTTP Supervisor Degraded Boot on Unreachable Endpoint
// ============================================================================

#[tokio::test]
async fn test_e2e_http_supervisor_degraded_boot_on_unreachable_endpoint() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mut mcp_servers = HashMap::new();
    // HTTP server pointing to an offline port
    mcp_servers.insert(
        "offline_http_server".to_string(),
        ServerConfig {
            command: None,
            args: vec![],
            env: HashMap::new(),
            url: Some("http://127.0.0.1:59999/mcp".to_string()),
            auth: None,
            protocol_version: None,
            allow_stateless: Some(true),
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let initial_config = McpConfig {
        mcp_servers,
        ..Default::default()
    };
    save_config(&config_path, &initial_config).unwrap();

    // Verify daemon boots without error even though the HTTP server is unreachable
    let state = initialize_state(initial_config, &config_path)
        .await
        .expect("Daemon must boot cleanly with unreachable HTTP server");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let statuses = state.server_statuses.read().await;
    let status = statuses
        .get("offline_http_server")
        .expect("server status must be recorded for offline_http_server");

    assert_eq!(status["status"], "degraded");
    assert_eq!(status["transport"], "http");
    assert!(status["error"].is_string());
}

#[tokio::test]
async fn test_e2e_server_restart_endpoint() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "restart_test_srv".to_string(),
        ServerConfig {
            command: Some("echo".to_string()),
            args: vec!["hello".to_string()],
            env: HashMap::new(),
            url: None,
            auth: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            resilience: None,
        },
    );

    let initial_config = McpConfig {
        mcp_servers,
        ..Default::default()
    };
    let (_state, port, server_task) = start_test_daemon(initial_config, &config_path).await;

    let client = reqwest::Client::new();
    let restart_url = format!(
        "http://127.0.0.1:{}/v1/config/servers/restart_test_srv/restart",
        port
    );

    let restart_res = client
        .post(&restart_url)
        .send()
        .await
        .expect("failed to call restart endpoint");

    assert_eq!(restart_res.status(), StatusCode::OK);
    let res_json: serde_json::Value = restart_res.json().await.unwrap();
    assert_eq!(res_json["ok"], true);
    assert_eq!(res_json["server_id"], "restart_test_srv");

    server_task.abort();
}

// This fixture runs in a separate test process so MCP owns real stdin/stdout while the
// operator HTTP listener shares exactly the same ledger, event bus, and audit worker.
#[tokio::test]
#[ignore = "subprocess fixture invoked by test_e2e_undo_stdio_to_signed_callback"]
async fn undo_lifecycle_fixture() {
    use rmcp::ServiceExt;
    use warmplane::{
        audit::AuditStore,
        daemon::{AppState, CapabilityMeta, ServerMsg},
        mcp_server::FacadeMcpServer,
        undo::{SnapshotStore, UndoLedgerStore},
    };

    let Ok(directory) = std::env::var("WARMPLANE_UNDO_E2E_DIRECTORY") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let config =
        warmplane::config::load_config(directory.join("config.json").to_string_lossy()).unwrap();
    let mut forward = CapabilityMeta::new(
        "mock",
        "set",
        "Set priority",
        "Set priority",
        json!({"type":"object"}),
    );
    forward.reversibility = config.capability_aliases["set-priority"]
        .reversibility()
        .cloned();
    let read = CapabilityMeta::new(
        "mock",
        "read",
        "Read priority",
        "Read priority",
        json!({"type":"object"}),
    );
    let (sender, mut receiver) = tokio::sync::mpsc::channel(8);
    let state = AppState::builder()
        .config_path(directory.join("config.json").to_string_lossy().into_owned())
        .servers(HashMap::from([("mock".to_string(), sender)]))
        .capabilities(HashMap::from([
            ("mock.set".to_string(), forward),
            ("mock.read".to_string(), read),
        ]))
        .snapshot_store(SnapshotStore::open(directory.join("artifacts")).unwrap())
        .undo_ledger(UndoLedgerStore::open(directory.join("undo.json")).unwrap())
        .audit_store(Arc::new(
            AuditStore::open_or_create(directory.join("audit.jsonl")).unwrap(),
        ))
        .build();
    let upstream_directory = directory.clone();
    let upstream = tokio::spawn(async move {
        let mut priority = json!("low");
        let mut calls = Vec::new();
        while let Some(message) = receiver.recv().await {
            if let ServerMsg::CallTool {
                name,
                params,
                reply,
                ..
            } = message
            {
                calls.push(json!({"name":name,"args":params}));
                let output = if name == "read" {
                    json!({"priority":priority})
                } else {
                    priority = params["priority"].clone();
                    json!({"ok":true})
                };
                std::fs::write(
                    upstream_directory.join("upstream.json"),
                    json!({"priority":priority,"calls":calls}).to_string(),
                )
                .unwrap();
                let _ = reply.send(Ok(output));
            }
        }
    });
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    std::fs::write(
        directory.join("port"),
        listener.local_addr().unwrap().port().to_string(),
    )
    .unwrap();
    let app = build_router(state.clone());
    let http = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let server = FacadeMcpServer::new(state.clone(), None)
        .serve(rmcp::transport::stdio())
        .await
        .unwrap();
    server.waiting().await.unwrap();
    state.audit_handle.shutdown().await;
    http.abort();
    upstream.abort();
}

#[tokio::test]
async fn test_e2e_undo_stdio_to_signed_callback() {
    use warmplane::{
        audit::{AuditEventType, AuditQueryFilter, AuditStore},
        undo::SnapshotStore,
    };

    let directory = tempfile::tempdir().unwrap();
    let config: McpConfig = serde_json::from_value(json!({
        "capabilityAliases":{"set-priority":{
            "target":"mock.set",
            "reversibility":{
                "class":"reversible","ttlSecs":60,
                "preSnapshot":{"capability":"mock.read","args":{},"captureJsonPath":"$.priority"},
                "preUndoCheck":{"capability":"mock.read","args":{},"captureJsonPath":"$.priority"},
                "inverse":{"capability":"mock.set","args":{"priority":"${snapshot}"}}
            }
        }},
        "policy":{"webhook":{
            "url":"http://127.0.0.1:1",
            "secret":"e2e-action-secret",
            "callbackUrl":"http://127.0.0.1/v1/webhooks/actions",
            "actionRole":"operator",
            "events":["undo.action_available"]
        }}
    }))
    .unwrap();
    save_config(
        directory.path().join("config.json").to_string_lossy(),
        &config,
    )
    .unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "undo_lifecycle_fixture",
            "--nocapture",
        ])
        .env("WARMPLANE_UNDO_E2E_DIRECTORY", directory.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let port = timeout(Duration::from_secs(10), async {
        loop {
            if let Ok(contents) = std::fs::read_to_string(directory.path().join("port")) {
                if let Ok(port) = contents.trim().parse::<std::num::NonZeroU16>() {
                    break port.get();
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("fixture HTTP listener should start");
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();
    let mut events = client
        .get(format!("{base}/v1/resources/updates"))
        .send()
        .await
        .unwrap()
        .bytes_stream();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(format!("{}\n",json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"undo-e2e","version":"1"}}
    })).as_bytes()).await.unwrap();
    assert!(read_jsonrpc_until_id(&mut reader, 1).await["result"].is_object());
    stdin.write_all(format!("{}\n{}\n",
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
            "name":"capability_call","arguments":{"capability_id":"mock.set","args":{"priority":"high"}}
        }})
    ).as_bytes()).await.unwrap();
    let response = read_jsonrpc_until_id(&mut reader, 2).await;
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");

    timeout(Duration::from_secs(5), async {
        let mut buffer = String::new();
        while let Some(chunk) = events.next().await {
            buffer.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
            if buffer.contains("event: action_capsule") && buffer.contains("\"refresh\":true") {
                return;
            }
        }
        panic!("SSE closed before action invalidation");
    })
    .await
    .expect("forward call should emit action_capsule");
    let listed: Value = client
        .get(format!("{base}/v1/undo"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entries = listed["result"]["entries"]
        .as_array()
        .expect("undo entries");
    assert_eq!(entries.len(), 1);
    let id = entries[0]["undo_id"].as_str().unwrap();
    assert_eq!(entries[0]["status"], "armed");
    let upstream = || -> Value {
        serde_json::from_slice(&std::fs::read(directory.path().join("upstream.json")).unwrap())
            .unwrap()
    };
    assert_eq!(upstream()["priority"], "high");
    let ledger: Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("undo.json")).unwrap())
            .unwrap();
    assert!(ledger.to_string().contains(id));
    let snapshots = SnapshotStore::open(directory.path().join("artifacts")).unwrap();
    let hash = entries[0]["snapshot_artifact_hash"].as_str().unwrap();
    assert_eq!(snapshots.get(hash).unwrap(), json!("low"));
    let token = warmplane::chatops::sign_action_token(
        "e2e-action-secret",
        "undo",
        id,
        "default",
        "operator",
        entries[0]["expires_at_ms"].as_u64().unwrap(),
    )
    .unwrap();
    let callback = format!("{base}/v1/webhooks/actions");
    let invalid = client
        .post(&callback)
        .json(&json!({"action_token":format!("{token}x")}))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    let calls_before = upstream()["calls"].as_array().unwrap().len();
    assert_eq!(
        upstream()["calls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|call| call["name"] == "set")
            .count(),
        1
    );
    let accepted = client
        .post(&callback)
        .json(&json!({"action_token":token}))
        .send()
        .await
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);
    timeout(Duration::from_secs(5), async {
        loop {
            let entry: Value = client
                .get(format!("{base}/v1/undo/{id}"))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if entry["result"]["status"] == "reverted" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("background inverse should complete");
    assert_eq!(upstream()["priority"], "low");
    assert_eq!(
        upstream()["calls"].as_array().unwrap().len(),
        calls_before + 3
    );
    assert_eq!(
        upstream()["calls"].as_array().unwrap().last().unwrap()["args"],
        json!({"priority":"low"})
    );
    let replay = client
        .post(&callback)
        .json(&json!({"action_token":token}))
        .send()
        .await
        .unwrap();
    assert_eq!(replay.status(), StatusCode::NOT_FOUND);
    drop(child.stdin.take());
    assert!(timeout(Duration::from_secs(5), child.wait())
        .await
        .unwrap()
        .unwrap()
        .success());
    let store = AuditStore::open_or_create(directory.path().join("audit.jsonl")).unwrap();
    let (audits, count) = store
        .query(&AuditQueryFilter {
            event_type: Some(AuditEventType::UndoExecuted),
            limit: 10,
            ..Default::default()
        })
        .await;
    assert_eq!(count, 1);
    assert!(!audits[0].hash.is_empty());
    assert!(serde_json::to_string(&audits[0]).unwrap().contains(id));
    assert!(store.verify_chain().await.is_valid);
}

async fn start_test_daemon(
    config: McpConfig,
    config_path: &str,
) -> (
    warmplane::daemon::AppState,
    u16,
    tokio::task::JoinHandle<()>,
) {
    save_config(config_path, &config).unwrap();
    let state = initialize_state(config, config_path).await.unwrap();
    let app = build_router(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (state, port, task)
}
