use std::sync::Arc;

use axum::{extract::State, http::Uri, routing::get, Json, Router};
use serde_json::{json, Value};
use tokio::{net::TcpListener, sync::Mutex};
use warmplane::{engine::ExecutionOptions, EmbeddedWarmplane};

const YAML_SPEC: &str = include_str!("fixtures/openapi/inventory.yaml");
const JSON_SPEC: &str = include_str!("fixtures/openapi/inventory.json");

async fn inventory(State(requests): State<Arc<Mutex<Vec<String>>>>, uri: Uri) -> Json<Value> {
    requests.lock().await.push(uri.to_string());
    Json(json!({"asset": "inventory-fixture", "private": "synthetic-sensitive"}))
}

#[tokio::test]
async fn embedded_openapi_inventory_has_stable_catalog_and_governed_calls() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let fixture = Router::new()
        .route("/api/assets", get(inventory))
        .route("/api/assets/:asset", get(inventory))
        .with_state(requests.clone());
    let server = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });

    let temp = tempfile::tempdir().unwrap();
    let spec = temp.path().join("inventory.yaml");
    let config = temp.path().join("mcp_servers.json");
    std::fs::write(&spec, YAML_SPEC).unwrap();
    let mut settings = json!({
        "state": {"enabled": false},
        "mcpServers": {
            "inventory": {"openApi": {
                "baseUrl": format!("http://{address}/api/"),
                "specPath": "inventory.yaml",
                "operations": ["getAsset", "listAssets"],
                "auth": {"type": "none"}
            }}
        },
        "policy": {"resultSanitization": {"fieldNames": ["private"]}}
    });
    std::fs::write(&config, settings.to_string()).unwrap();
    let (embedded, shutdown) = EmbeddedWarmplane::start_from_path(config.to_str().unwrap())
        .await
        .unwrap();

    let result = embedded
        .call_capability(
            "inventory.getAsset",
            json!({"path": {"assetId": "A 42"}}),
            ExecutionOptions::default(),
        )
        .await;
    assert_eq!(
        result.data.unwrap()["structuredContent"],
        json!({"asset": "inventory-fixture", "private": "[REDACTED]"})
    );

    let invalid = embedded
        .call_capability(
            "inventory.getAsset",
            json!({"path": {"assetId": ".."}}),
            ExecutionOptions::default(),
        )
        .await;
    assert_eq!(invalid.error.unwrap().code, "INVALID_ARGUMENTS");

    std::fs::write(&spec, JSON_SPEC).unwrap();
    assert_eq!(
        embedded.reload_from_disk().await.unwrap()["mounted"],
        json!(["inventory"])
    );
    std::fs::write(&spec, "openapi: 3.1.1\npaths: {}").unwrap();
    let diagnostics = embedded.reload_from_disk().await.unwrap();
    assert_eq!(diagnostics["warnings"].as_array().unwrap().len(), 1);
    let list = embedded
        .call_capability(
            "inventory.listAssets",
            json!({"query": {"tag": "Q & A"}}),
            ExecutionOptions::default(),
        )
        .await;
    assert!(list.ok, "{:?}", list.error);
    assert_eq!(
        *requests.lock().await,
        ["/api/assets/A%2042", "/api/assets?tag=Q+%26+A"]
    );

    settings["mcpServers"] = json!({});
    std::fs::write(&config, settings.to_string()).unwrap();
    assert_eq!(
        embedded.reload_from_disk().await.unwrap()["unmounted"],
        json!(["inventory"])
    );
    let removed = embedded
        .call_capability(
            "inventory.listAssets",
            json!({}),
            ExecutionOptions::default(),
        )
        .await;
    assert!(!removed.ok);
    assert_eq!(requests.lock().await.len(), 2);
    shutdown.cancel();
    server.abort();
}

#[tokio::test]
async fn uncertain_openapi_write_is_never_replayed_after_restart() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let record = requests.clone();
    let fixture = Router::new().route(
        "/api/assets/A42",
        axum::routing::patch(move |Json(body): Json<Value>| {
            let record = record.clone();
            async move {
                record.lock().await.push(body.to_string());
                axum::http::StatusCode::BAD_GATEWAY
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });
    let temp = tempfile::tempdir().unwrap();
    let spec_path = temp.path().join("write-spec.json");
    let config_path = temp.path().join("mcp_servers.json");
    let spec = json!({
        "openapi": "3.1.1",
        "paths": {
            "/assets/{assetId}": {
                "patch": {
                    "operationId": "updateAsset",
                    "parameters": [{"in":"path", "name":"assetId", "required":true, "schema":{"type":"string"}}],
                    "requestBody": {"required":true,"content":{"application/json":{"schema":{
                        "type":"object","additionalProperties":false,
                        "properties":{"name":{"type":"string"}},"required":["name"]
                    }}}},
                    "responses": {"204":{"description":"Updated"}}
                }
            }
        }
    });
    let config = json!({
        "state": {"enabled": true, "dir": temp.path().join("state")},
        "mcpServers": {"inventory": {"openApi": {
            "baseUrl": format!("http://{address}/api/"), "specPath": "write-spec.json",
            "operations": ["updateAsset"], "mutations": ["updateAsset"], "auth": {"type": "none"}
        }}},
        "policy": {"resultSanitization": {"fieldNames": ["private"]}}
    });
    std::fs::write(&spec_path, spec.to_string()).unwrap();
    std::fs::write(&config_path, config.to_string()).unwrap();
    let config = config_path.to_str().unwrap();
    let (first, _) = EmbeddedWarmplane::start_from_path(config).await.unwrap();
    let args = json!({"path":{"assetId":"A42"}, "body":{"name":"new name"}});
    let mut options = ExecutionOptions::default();
    options.idempotency_key = Some("fixture-write-1".to_owned());
    let write = first
        .call_capability("inventory.updateAsset", args.clone(), options.clone())
        .await;
    assert_eq!(write.retry.upstream_execution_state, "unknown");
    assert_eq!(write.error.unwrap().code, "UPSTREAM_OUTCOME_UNKNOWN");
    first.shutdown().await;
    let (restarted, _) = EmbeddedWarmplane::start_from_path(config).await.unwrap();
    let replay = restarted
        .call_capability("inventory.updateAsset", args, options)
        .await;
    assert_eq!(replay.error.unwrap().code, "UPSTREAM_OUTCOME_UNKNOWN");
    assert_eq!(*requests.lock().await, [r#"{"name":"new name"}"#]);
    restarted.shutdown().await;
    server.abort();
}

async fn request_json(
    router: &Router,
    path: &str,
    payload: &Value,
) -> (axum::http::StatusCode, Value) {
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    let response = router
        .clone()
        .oneshot(
            Request::post(path)
                .header("authorization", "Bearer fixture-admin")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let data = to_bytes(response.into_body(), 1 << 20).await.unwrap();
    (status, serde_json::from_slice(&data).unwrap())
}

struct InventoryFixture {
    base_url: String,
    name: Arc<Mutex<String>>,
    mutations: Arc<Mutex<Vec<String>>>,
    listener: tokio::task::JoinHandle<()>,
}

impl InventoryFixture {
    async fn start(initial: &str) -> Self {
        use axum::http::StatusCode;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}/api", listener.local_addr().unwrap());
        let name = Arc::new(Mutex::new(initial.to_owned()));
        let mutations = Arc::new(Mutex::new(Vec::new()));
        let read_name = name.clone();
        let write_name = name.clone();
        let history = mutations.clone();
        let fixture = Router::new().route(
            "/api/assets/:asset",
            get(move || {
                let name = read_name.clone();
                async move { Json(json!({"name": name.lock().await.as_str()})) }
            })
            .patch(move |Json(input): Json<Value>| {
                let name = write_name.clone();
                let history = history.clone();
                async move {
                    let value = input["name"].as_str().unwrap().to_owned();
                    *name.lock().await = value.clone();
                    history.lock().await.push(value);
                    StatusCode::NO_CONTENT
                }
            }),
        );
        let listener = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });
        Self {
            base_url,
            name,
            mutations,
            listener,
        }
    }

    fn config(&self, test_only: bool) -> Value {
        json!({"testOnly":test_only, "openApi":{
            "baseUrl":self.base_url, "specPath":"inventory-write.json",
            "operations":["readAsset","writeAsset"], "mutations":["writeAsset"],
            "auth":{"type":"none"}
        }})
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        self.listener.abort();
    }
}

fn inventory_write_spec() -> Value {
    json!({
        "openapi":"3.1.1",
        "paths": {"/assets/{assetId}":{
            "parameters":[{"in":"path","name":"assetId","required":true,"schema":{"type":"string"}}],
            "get":{"operationId":"readAsset","responses":{"200":{
                "description":"Asset","content":{"application/json":{"schema":{"type":"object"}}}
            }}},
            "patch":{"operationId":"writeAsset","requestBody":{"required":true,"content":{"application/json":{"schema":{
                "type":"object","additionalProperties":false,"properties":{"name":{"type":"string"}},"required":["name"]
            }}}},"responses":{"204":{"description":"Updated"}}}
        }}
    })
}

async fn boot_fixture_daemon(config: &std::path::Path) -> (warmplane::daemon::AppState, Router) {
    let path = config.to_str().unwrap();
    let config = warmplane::config::load_config(path).unwrap();
    let state = warmplane::daemon::server::initialize_state(config, path)
        .await
        .unwrap();
    let router = warmplane::daemon::server::build_router(state.clone());
    (state, router)
}

#[tokio::test]
async fn confirmed_recording_restores_only_test_state_and_activation_requires_evidence() {
    use axum::http::StatusCode;
    let live = InventoryFixture::start("production-original").await;
    let test = InventoryFixture::start("fixture-original").await;
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("mcp_servers.json");
    let spec_path = temp.path().join("inventory-write.json");
    std::fs::write(&spec_path, inventory_write_spec().to_string()).unwrap();
    let config = json!({
        "state":{"enabled":true,"dir":temp.path().join("state")},
        "authToken":"fixture-admin",
        "mcpServers":{"inventory":live.config(false), "inventory-test":test.config(true)}
    })
    .to_string();
    std::fs::write(&config_path, &config).unwrap();
    let (state, router) = boot_fixture_daemon(&config_path).await;
    let read = json!({
        "capability":"inventory.readAsset",
        "args":{"path":{"assetId":"${args.path.assetId}"}},
        "captureJsonPath":"$.structuredContent.name"
    });
    let draft = json!({
        "forward":"inventory.writeAsset","ttlSecs":90,
        "preSnapshot":read,"preUndoCheck":read,
        "inverse":{"capability":"inventory.writeAsset","args":{
            "path":{"assetId":"${args.path.assetId}"},"body":{"name":"${snapshot}"}
        }},
        "testTarget":{"server":"inventory-test","args":{
            "path":{"assetId":"A42"},"body":{"name":"fixture-changed"}
        }}
    });
    let mut wrong_target = draft.clone();
    wrong_target["testTarget"]["server"] = json!("inventory");
    let (status, _) = request_json(&router, "/v1/config/reversibility/draft", &wrong_target).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(live.mutations.lock().await.is_empty());

    let (_, preview) = request_json(&router, "/v1/config/reversibility/draft", &draft).await;
    assert_eq!(
        preview["testForward"], "inventory-test.writeAsset",
        "{preview}"
    );
    let mut record = json!({"draft":draft,"revisions":preview["draft"]["revisions"],"confirmationToken":preview["confirmationToken"],"confirmed":false});
    let (status, _) = request_json(&router, "/v1/config/reversibility/record", &record).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(test.mutations.lock().await.is_empty());

    record["confirmed"] = json!(true);
    let mut stale = record.clone();
    stale["confirmationToken"] = json!("obsolete");
    let (status, _) = request_json(&router, "/v1/config/reversibility/record", &stale).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(test.mutations.lock().await.is_empty());

    let (status, recorded) =
        request_json(&router, "/v1/config/reversibility/record", &record).await;
    assert_eq!(status, StatusCode::OK, "{recorded}");
    assert_eq!(recorded["cleanup"], "verified");
    let (status, _) = request_json(&router, "/v1/config/reversibility/record", &record).await;
    assert_eq!(status, StatusCode::CONFLICT);

    assert_eq!(test.name.lock().await.as_str(), "fixture-original");
    assert_eq!(
        *test.mutations.lock().await,
        ["fixture-changed", "fixture-original"]
    );
    assert!(live.mutations.lock().await.is_empty());
    assert_eq!(std::fs::read_to_string(&config_path).unwrap(), config);

    let activate = json!({
        "draft":draft,"revisions":record["revisions"],
        "reviewToken":recorded["reviewToken"],"alias":"restore-ready"
    });
    let mut revised_spec = inventory_write_spec();
    revised_spec["paths"]["/assets/{assetId}"]["patch"]["description"] =
        json!("A changed write contract");
    std::fs::write(&spec_path, revised_spec.to_string()).unwrap();
    let (status, _) = request_json(&router, "/v1/config/reversibility/activate", &activate).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(std::fs::read_to_string(&config_path).unwrap(), config);
    std::fs::write(&spec_path, inventory_write_spec().to_string()).unwrap();
    let (status, result) =
        request_json(&router, "/v1/config/reversibility/activate", &activate).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    let saved: Value = serde_json::from_slice(&std::fs::read(config_path).unwrap()).unwrap();
    assert_eq!(
        saved
            .pointer("/capabilityAliases/restore-ready/reversibility/inverse/capability")
            .unwrap(),
        "restore-ready"
    );
    assert!(live.mutations.lock().await.is_empty());
    let (status, _) = request_json(&router, "/v1/config/reversibility/activate", &activate).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let call = |key: &str, name: &str| {
        json!({
            "capability_id":"restore-ready","idempotency_key":key,
            "args":{"path":{"assetId":"A42"},"body":{"name":name}}
        })
    };
    let (_, changed) = request_json(&router, "/v1/tools/call", &call("live-1", "first")).await;
    assert_eq!(changed["ok"], true, "{changed}");
    let ledger = state.undo_ledger.as_ref().unwrap();
    let first = ledger.entries().await.into_iter().next().unwrap();
    let (_, undo) = request_json(
        &router,
        &format!("/v1/undo/{}/execute", first.undo_id),
        &json!({}),
    )
    .await;
    assert_eq!(undo["ok"], true, "{undo}");
    assert_eq!(live.name.lock().await.as_str(), "production-original");

    let changed = call_mcp(
        &state,
        json!({
            "capability_id":"restore-ready", "request_id":"mcp-write-2",
            "args":{"path":{"assetId":"A42"},"body":{"name":"second"}}
        }),
    )
    .await;
    assert_eq!(changed["ok"], true, "{changed}");
    let second = ledger
        .entries()
        .await
        .into_iter()
        .find(|entry| entry.undo_id != first.undo_id)
        .unwrap();
    *live.name.lock().await = "intervening-change".into();
    let (_, undo) = request_json(
        &router,
        &format!("/v1/undo/{}/execute", second.undo_id),
        &json!({}),
    )
    .await;
    assert_eq!(undo["ok"], false, "{undo}");
    assert!(undo.to_string().contains("UNDO_STATE_DRIFT"), "{undo}");
    assert_eq!(
        *live.mutations.lock().await,
        ["first", "production-original", "second"]
    );
    state.shutdown().await;
}

#[tokio::test]
async fn rest_batch_replays_completed_openapi_write_after_restart() {
    let live = InventoryFixture::start("initial").await;
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("mcp_servers.json");
    std::fs::write(
        temp.path().join("inventory-write.json"),
        inventory_write_spec().to_string(),
    )
    .unwrap();
    std::fs::write(
        &config_path,
        json!({
            "state":{"enabled":true,"dir":temp.path().join("state")},
            "authToken":"fixture-admin","mcpServers":{"inventory":live.config(false)}
        })
        .to_string(),
    )
    .unwrap();
    let payload = json!({
        "request_id":"inventory-unique-batch",
        "steps":[{"id":"write","capability_id":"inventory.writeAsset","args":{
            "path":{"assetId":"A42"},"body":{"name":"updated"}
        }}]
    });
    let (first, router) = boot_fixture_daemon(&config_path).await;
    let (_, response) = request_json(&router, "/v1/tools/batch_call", &payload).await;
    assert_eq!(response["ok"], true, "{response}");
    first.shutdown().await;
    let (second, router) = boot_fixture_daemon(&config_path).await;
    let (_, response) = request_json(&router, "/v1/tools/batch_call", &payload).await;
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(*live.mutations.lock().await, ["updated"]);
    second.shutdown().await;
}

async fn call_mcp(state: &warmplane::daemon::AppState, arguments: Value) -> Value {
    use rmcp::{model::CallToolRequestParams, ServiceExt as _};

    let (client_io, server_io) = tokio::io::duplex(65_536);
    let facade = warmplane::mcp_server::FacadeMcpServer::new(state.clone(), None);
    let server = tokio::spawn(async move { facade.serve(server_io).await.unwrap() });
    let client = ().serve(client_io).await.unwrap();
    let server = server.await.unwrap();
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.call_tool(
            CallToolRequestParams::new("capability_call")
                .with_arguments(arguments.as_object().unwrap().clone()),
        ),
    )
    .await
    .unwrap()
    .unwrap();
    client.cancel().await.unwrap();
    server.cancel().await.unwrap();
    response.structured_content.unwrap()
}

#[tokio::test]
async fn worker_rechecks_authorization_after_a_call_waits_in_the_queue() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::sync::Notify;
    use warmplane::{config::PolicyConfig, daemon::Policy, ControlPlaneHandle};

    let calls = Arc::new(AtomicUsize::new(0));
    let entered = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (counter, first, gate) = (calls.clone(), entered.clone(), resume.clone());
    let fixture = Router::new().route(
        "/api/assets/:asset",
        get(move || {
            let (calls, entered, resume) = (counter.clone(), first.clone(), gate.clone());
            async move {
                if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    entered.notify_one();
                    resume.notified().await;
                }
                Json(json!({"asset":true}))
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("mcp_servers.json");
    std::fs::write(directory.path().join("inventory.json"), JSON_SPEC).unwrap();
    std::fs::write(
        &config,
        json!({
            "state":{"enabled":false},
            "mcpServers":{"inventory":{"openApi":{
                "specPath":"inventory.json","baseUrl":format!("http://{address}/api"),
                "operations":["getAsset"],"auth":{"type":"none"}
            }}}
        })
        .to_string(),
    )
    .unwrap();
    let (state, _) = boot_fixture_daemon(&config).await;
    let handle = ControlPlaneHandle::new(state.clone());
    let request = |handle: ControlPlaneHandle, id: &'static str| {
        tokio::spawn(async move {
            handle
                .call_capability(
                    "inventory.getAsset",
                    json!({"path":{"assetId":id}}),
                    ExecutionOptions::default(),
                )
                .await
        })
    };
    let one = request(handle.clone(), "first");
    tokio::time::timeout(Duration::from_secs(2), entered.notified())
        .await
        .unwrap();
    let worker = state.servers.read().await["inventory"].clone();
    let free = worker.capacity();
    let two = request(handle, "queued");
    tokio::time::timeout(Duration::from_secs(2), async {
        while worker.capacity() == free {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    *state.policy.write().await = Policy::from_config(Some(
        serde_json::from_value::<PolicyConfig>(json!({"deny":["inventory.getAsset"]})).unwrap(),
    ));
    resume.notify_one();
    assert!(one.await.unwrap().ok);
    assert_eq!(two.await.unwrap().error.unwrap().code, "POLICY_DENIED");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    state.shutdown().await;
    server.abort();
}

#[tokio::test]
async fn process_exit_during_openapi_write_preserves_the_dispatch_reservation() {
    use std::{
        process::Stdio,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };
    use tokio::sync::Notify;

    let writes = Arc::new(AtomicUsize::new(0));
    let sent = Arc::new(Notify::new());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let notify = sent.clone();
    let count = writes.clone();
    let fixture = Router::new().route(
        "/api/assets/:asset",
        axum::routing::patch(move || {
            let (sent, writes) = (notify.clone(), count.clone());
            async move {
                writes.fetch_add(1, Ordering::SeqCst);
                sent.notify_one();
                std::future::pending::<axum::http::StatusCode>().await
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("mcp_servers.json");
    std::fs::write(
        directory.path().join("inventory-write.json"),
        inventory_write_spec().to_string(),
    )
    .unwrap();
    std::fs::write(
        &config_path,
        json!({
            "state":{"enabled":true,"dir":directory.path().join("state")},
            "authToken":"fixture-admin",
            "mcpServers":{"inventory":{"openApi":{
                "baseUrl":format!("http://{address}/api"),"specPath":"inventory-write.json",
                "operations":["writeAsset"],"mutations":["writeAsset"],"auth":{"type":"none"}
            }}}
        })
        .to_string(),
    )
    .unwrap();
    let reserved = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = reserved.local_addr().unwrap().port();
    drop(reserved);
    let mut daemon = tokio::process::Command::new(env!("CARGO_BIN_EXE_warmplane"))
        .arg("daemon")
        .arg("--port")
        .arg(port.to_string())
        .arg("--config")
        .arg(&config_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let origin = format!("http://127.0.0.1:{port}");
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if client
                .get(format!("{origin}/v1/config"))
                .bearer_auth("fixture-admin")
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                break;
            }
            assert!(daemon.try_wait().unwrap().is_none());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let payload = json!({
        "capability_id":"inventory.writeAsset","idempotency_key":"crash-write",
        "args":{"path":{"assetId":"A42"},"body":{"name":"mutated"}}
    });
    let attempt_payload = payload.clone();
    let request = tokio::spawn(async move {
        client
            .post(format!("{origin}/v1/tools/call"))
            .bearer_auth("fixture-admin")
            .json(&attempt_payload)
            .send()
            .await
    });
    tokio::time::timeout(Duration::from_secs(5), sent.notified())
        .await
        .unwrap();
    daemon.kill().await.unwrap();
    assert!(request.await.unwrap().is_err());

    let (restarted, router) = boot_fixture_daemon(&config_path).await;
    let (_, replay) = request_json(&router, "/v1/tools/call", &payload).await;
    assert_eq!(
        replay.pointer("/error/code").and_then(Value::as_str),
        Some("UPSTREAM_OUTCOME_UNKNOWN"),
        "{replay}"
    );
    assert_eq!(writes.load(Ordering::SeqCst), 1);
    restarted.shutdown().await;
    server.abort();
}
