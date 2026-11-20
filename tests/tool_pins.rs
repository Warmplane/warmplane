use std::time::Duration;

use axum::{
    body::{to_bytes, Body},
    http::{HeaderMap, Method, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use warmplane::{
    config::{load_config, save_config, McpConfig},
    daemon::server::{build_router, initialize_state},
    engine::{ControlPlaneHandle, ExecutionOptions},
};

const UPSTREAM: &str = r#"
import json, pathlib, sys, threading, time

version = pathlib.Path(sys.argv[1])
calls = pathlib.Path(sys.argv[2])
lock = threading.Lock()

def write(message):
    with lock:
        print(json.dumps({"jsonrpc": "2.0", **message}), flush=True)

def observe():
    previous = version.read_text()
    while True:
        time.sleep(0.02)
        current = version.read_text()
        if current and current != previous:
            previous = current
            write({"method": "notifications/tools/list_changed"})

threading.Thread(target=observe, daemon=True).start()
for line in sys.stdin:
    message = json.loads(line)
    if "id" not in message:
        continue
    method, args = message["method"], message.get("params", {})
    if method == "initialize":
        result = {
            "protocolVersion": "2025-11-25",
            "capabilities": {"tools": {"listChanged": True}},
            "serverInfo": {"name": "pin-fixture", "version": "1"}
        }
    elif method == "tools/list":
        if args.get("cursor") == "tail":
            result = {"tools": [{"name": "ping", "inputSchema": {"type": "object"}}]}
        else:
            result = {"tools": [{"name": "save", "description": version.read_text(),
                                "inputSchema": {"type": "object"}}], "nextCursor": "tail"}
    elif method == "tools/call":
        with calls.open("a") as f:
            f.write(args["name"] + "\n")
        result = {"content": [{"type": "text", "text": "saved"}]}
    else:
        result = {"resources" if method == "resources/list" else "prompts": []}
    write({"id": message["id"], "result": result})
"#;

async fn request(
    router: &Router,
    method: Method,
    path: &str,
    payload: Value,
    token: Option<&str>,
    etag: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    let response = router
        .clone()
        .oneshot(request.body(Body::from(payload.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, headers, body)
}

const TOKEN: &str = "operator-fixture-token";
const REVIEWS: &str = "/v1/config/tool-pins";
const APPROVE: &str = "/v1/config/tool-pins/approve";
const CATALOG: &str = "/v1/capabilities";

async fn reviews(router: &Router) -> Vec<Value> {
    let (status, _, result) =
        request(router, Method::GET, REVIEWS, Value::Null, Some(TOKEN), None).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    result["pins"].as_array().unwrap().clone()
}

async fn approve(router: &Router, pin: &Value) -> StatusCode {
    let (status, _, _) = request(
        router,
        Method::POST,
        APPROVE,
        json!({
            "server": "fixture",
            "tool": pin["tool"],
            "digest": pin["current"]["digest"],
        }),
        Some(TOKEN),
        None,
    )
    .await;
    status
}

fn save_pin(pins: &[Value]) -> &Value {
    pins.iter().find(|pin| pin["tool"] == "save").unwrap()
}

#[tokio::test]
async fn stdio_pin_gates_paginated_discovery_stale_review_and_held_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let script = dir.path().join("upstream.py");
    let version = dir.path().join("version");
    let log = dir.path().join("calls");
    std::fs::write(&script, UPSTREAM).unwrap();
    std::fs::write(&version, "original").unwrap();
    std::fs::write(&log, "").unwrap();
    let config: McpConfig = serde_json::from_value(json!({
        "authToken": TOKEN,
        "state": {"enabled": true, "dir": dir.path().join("state")},
        "toolTimeoutMs": 3000,
        "mcpServers": {"fixture": {
            "command": "python3", "args": ["-u", script.to_str().unwrap(), version.to_str().unwrap(), log.to_str().unwrap()],
            "requireToolPins": true,
        }},
        "capabilityAliases": {"save_alias": {
            "target": "fixture.save", "description": "alias override", "passthrough": true,
            "reversibility": {"class": "delayable", "holdWindowSecs": 2},
        }}
    })).unwrap();
    save_config(path.to_str().unwrap(), &config).unwrap();
    let state = initialize_state(config, path.to_str().unwrap())
        .await
        .unwrap();
    let router = build_router(state.clone());
    let handle = ControlPlaneHandle::new(state);
    assert_eq!(
        request(&router, Method::GET, REVIEWS, Value::Null, None, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let initial = reviews(&router).await;
    assert!(
        initial.iter().any(|pin| pin["tool"] == "ping"),
        "{initial:?}"
    );
    let old = save_pin(&initial);
    assert_eq!(old["status"], "missing");
    assert_eq!(old["current"]["value"]["description"], "original");
    assert!(handle
        .list_capabilities(None)
        .await
        .unwrap()
        .capabilities
        .is_empty());
    assert_eq!(approve(&router, old).await, StatusCode::OK);
    let (status, headers, listed) = request(
        &router,
        Method::GET,
        CATALOG,
        Value::Null,
        Some(TOKEN),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["capabilities"][0]["id"], "save_alias");
    let etag = headers["etag"].to_str().unwrap();

    let pending = handle
        .call_capability(
            "save_alias",
            json!({}),
            ExecutionOptions::new().with_async_task(true),
        )
        .await;
    assert!(pending.ok, "{pending:?}");
    let task = pending.data.unwrap();
    let task_id = task["taskId"].as_str().unwrap();
    std::fs::write(&version, "changed").unwrap();

    let changed = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let latest = reviews(&router).await;
            let pin = save_pin(&latest);
            if pin["status"] == "changed" {
                break pin.clone();
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(changed["current"]["value"]["description"], "changed");
    let (status, _, listed) = request(
        &router,
        Method::GET,
        CATALOG,
        Value::Null,
        Some(TOKEN),
        Some(etag),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["capabilities"], json!([]));
    let search = handle
        .search_capabilities(Some("save"), None, None, None, Some(1), None)
        .await;
    assert!(search.capabilities.is_empty());

    assert_eq!(approve(&router, old).await, StatusCode::CONFLICT);
    assert_eq!(approve(&router, &changed).await, StatusCode::OK);
    let failed = tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            let task = handle.get_task(task_id).await.data.unwrap();
            if task.status == warmplane::tasks::TaskStatus::Failed {
                break task;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(failed.error.unwrap()["code"], "TOOL_DEFINITION_UNAPPROVED");
    assert_eq!(std::fs::read_to_string(&log).unwrap(), "");
    handle.shutdown().await;

    let restarted = initialize_state(
        load_config(path.to_str().unwrap()).unwrap(),
        path.to_str().unwrap(),
    )
    .await
    .unwrap();
    let router = build_router(restarted.clone());
    assert_eq!(save_pin(&reviews(&router).await)["status"], "matching");
    ControlPlaneHandle::new(restarted).shutdown().await;
}

async fn fake_mcp_http(
    axum::extract::State(description): axum::extract::State<
        std::sync::Arc<tokio::sync::RwLock<String>>,
    >,
    axum::Json(msg): axum::Json<Value>,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    let Some(id) = msg.get("id") else {
        return StatusCode::ACCEPTED.into_response();
    };
    let result = match msg["method"].as_str().unwrap() {
        "initialize" => json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {"tools": {}, "resources": {}, "prompts": {}},
            "serverInfo": {"name": "http-pin-fixture", "version": "1"}
        }),
        "tools/list" => json!({"tools": [{
            "name": "save", "description": description.read().await.as_str(), "inputSchema": {"type":"object"}
        }], "ttlMs": 120000, "cacheScope": "public"}),
        "tools/call" => json!({"content":[{"type":"text", "text":"fixture-secret-42"}]}),
        "resources/list" => json!({"resources":[]}),
        "prompts/list" => json!({"prompts":[]}),
        method => panic!("unexpected fixture method {method}"),
    };
    axum::Json(json!({"jsonrpc":"2.0", "id":id, "result":result})).into_response()
}

#[tokio::test]
async fn http_fallback_refresh_observes_definition_changes_and_enforces_result_delivery() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let description = std::sync::Arc::new(tokio::sync::RwLock::new("original".to_owned()));
    let fixture = Router::new()
        .route("/mcp", axum::routing::post(fake_mcp_http))
        .with_state(description.clone());
    let server_task = tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.json");
    let config: McpConfig = serde_json::from_value(json!({
        "authToken": TOKEN,
        "state": {"enabled": true, "dir": directory.path().join("state")},
        "policy": {"resultSanitization": {"patterns":["fixture-secret-[0-9]+"]}},
        "mcpServers": {"fixture": {"url": url, "allowStateless": true, "requireToolPins": true}},
    }))
    .unwrap();
    save_config(path.to_str().unwrap(), &config).unwrap();
    let state = initialize_state(config, path.to_str().unwrap())
        .await
        .unwrap();
    let router = build_router(state.clone());
    let handle = ControlPlaneHandle::new(state);
    let initial = reviews(&router).await;
    let original = save_pin(&initial);
    assert_eq!(approve(&router, original).await, StatusCode::OK);
    let result = handle
        .call_capability("fixture.save", json!({}), ExecutionOptions::new())
        .await;
    assert!(result.ok, "{result:?}");
    assert_eq!(result.data.unwrap()["content"][0]["text"], "[REDACTED]");
    *description.write().await = "changed without notification".to_owned();
    let after_poll = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let latest = reviews(&router).await;
            if save_pin(&latest)["status"] == "changed" {
                break latest;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        save_pin(&after_poll)["current"]["value"]["description"],
        "changed without notification"
    );
    assert!(handle
        .list_capabilities(None)
        .await
        .unwrap()
        .capabilities
        .is_empty());
    handle.shutdown().await;
    server_task.abort();
}
