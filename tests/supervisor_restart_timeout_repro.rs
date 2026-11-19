// Rust guideline compliant 2026-10-02

//! Regression tests for upstream MCP negotiation and stdio restart.
//!
//! Every other reconnect path in `supervisor.rs` (the initial stdio mount and the
//! streamable HTTP reconnect) wraps its handshake in `timeout(handshake_timeout, ..)`.
//! The stdio restart path was the only one that did not, so a child that starts but
//! never answers its `initialize` request pinned the supervisor task inside
//! `serve().await` forever. That task never returned to its `tokio::select!` to
//! drain `rx`, so the bounded mailbox (capacity 32) filled and every later
//! `tx.send(..).await` blocked. It also held the `RunningService` future, so
//! `kill_on_drop(true)` never fired and the child was never reaped.
//!
//! The test drives the real supervisor with a real child that hangs on stdin without
//! ever writing a JSON-RPC reply.
//!
//! Message one triggers the restart branch. The supervisor answers it immediately,
//! then enters the restart handshake. Message two therefore can only be answered if
//! that handshake gives up and the loop comes back around to `rx.recv()`. Before the
//! fix, message two was never read and its reply never arrived.

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    time::Duration,
};
use tokio::net::TcpListener;

use warmplane::circuit_breaker::ResilienceConfig;
use warmplane::config::ServerConfig;
use warmplane::daemon::state::AppState;
use warmplane::daemon::types::{ServerMsg, UpstreamCallError};
use warmplane::supervisor::{spawn_supervised_http_server, spawn_supervised_stdio_server};

/// Upper bound for the supervisor to answer the follow-up message.
///
/// The restart handshake itself is bounded at 5s, so a correct build answers well
/// inside this window. The bound only exists to turn a wedge into a test failure.
const ANSWER_DEADLINE: Duration = Duration::from_secs(30);

/// Builds a daemon `AppState` with no upstream servers attached.
fn state() -> AppState {
    AppState::builder()
        .capabilities(HashMap::new())
        .servers(HashMap::new())
        .build()
}

/// A stdio server config whose child hangs forever without speaking MCP.
fn hanging_server() -> ServerConfig {
    ServerConfig {
        command: Some("cat".to_string()),
        args: vec![],
        env: HashMap::new(),
        url: None,
        auth: None,
        protocol_version: None,
        allow_stateless: None,
        headers: HashMap::new(),
        resilience: Some(ResilienceConfig {
            // One restart so the test settles instead of looping for 5 attempts.
            max_restarts: 1,
            ..Default::default()
        }),
    }
}

async fn call_tool(
    tx: &tokio::sync::mpsc::Sender<ServerMsg>,
    name: &str,
) -> Result<Result<Value, UpstreamCallError>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    tx.send(ServerMsg::CallTool {
        name: name.to_string(),
        params: json!({}),
        input_responses: None,
        request_state: None,
        reply: reply_tx,
    })
    .await
    .map_err(|e| format!("supervisor mailbox rejected the message: {e}"))?;

    tokio::time::timeout(ANSWER_DEADLINE, reply_rx)
        .await
        .map_err(|_| "supervisor never answered".to_string())?
        .map_err(|_| "supervisor dropped the reply channel".to_string())
}

/// Checks that the supervisor replies even while the upstream is unavailable.
async fn ask(tx: &tokio::sync::mpsc::Sender<ServerMsg>) -> Result<(), String> {
    match call_tool(tx, "noop").await? {
        // The server has no working client, so a bounded upstream error is expected.
        Ok(_) | Err(UpstreamCallError::Upstream(_)) => Ok(()),
        Err(UpstreamCallError::Timeout) => Err("upstream call timed out".to_string()),
    }
}

#[tokio::test]
async fn stdio_restart_handshake_times_out_and_leaves_supervisor_responsive() {
    let state = state();
    let server_id = "hanging_stdio";

    // The initial mount also hangs, so it must give up and hand back a mailbox.
    let (_caps, _res, _prompts, tx) = spawn_supervised_stdio_server(
        &state,
        server_id,
        &hanging_server(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    )
    .await
    .expect("stdio supervisor must spawn even when the handshake never completes");

    // First message: the supervisor has no client, so it answers this immediately and
    // then enters the restart handshake for the hanging child.
    ask(&tx).await.expect("initial message must be answered");

    // Second message: this can only be read if the restart handshake was bounded.
    ask(&tx)
        .await
        .unwrap_or_else(|e| panic!("supervisor stayed wedged in the stdio restart handshake: {e}"));

    state.shutdown_token.cancel();
}

#[tokio::test]
async fn stdio_negotiates_a_legacy_session_on_startup_and_restart() {
    let state = state();
    let config = ServerConfig {
        command: Some(
            std::env::current_exe()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
        ),
        args: [
            "--ignored",
            "--exact",
            "stdio_upstream_fixture",
            "--nocapture",
        ]
        .map(str::to_owned)
        .to_vec(),
        ..hanging_server()
    };
    let aliases = HashMap::new();
    let (tools, _, _, tx) =
        spawn_supervised_stdio_server(&state, "stdio", &config, &aliases, &aliases, &aliases)
            .await
            .expect("spawn stdio supervisor");

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].0, "stdio.ping");
    let first = call_tool(&tx, "ping").await.unwrap().unwrap();
    assert_eq!(first["content"][0]["text"], "pong");

    let exit = call_tool(&tx, "exit").await.unwrap();
    assert!(matches!(exit, Err(UpstreamCallError::Upstream(_))));

    let restarted = call_tool(&tx, "ping").await.unwrap().unwrap();
    assert_eq!(restarted["content"][0]["text"], "pong");
    assert!(state.capabilities.read().await.contains_key("stdio.ping"));
    state.shutdown_token.cancel();
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2025-11-25",
        "capabilities": {"tools": {}},
        "serverInfo": {"name": "upstream fixture", "version": "1"}
    })
}

fn catalog_result(method: &str) -> Option<Value> {
    match method {
        "tools/list" => {
            Some(json!({"tools": [{"name": "ping", "inputSchema": {"type": "object"}}]}))
        }
        "resources/list" => Some(json!({"resources": []})),
        "prompts/list" => Some(json!({"prompts": []})),
        _ => None,
    }
}

#[test]
#[ignore = "subprocess fixture invoked by stdio_negotiates_a_legacy_session_on_startup_and_restart"]
fn stdio_upstream_fixture() {
    let mut stdout = io::stdout().lock();
    let mut initialized = false;
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let method = request["method"].as_str().unwrap();
        let result = match method {
            "initialize" => {
                assert_eq!(request["params"]["protocolVersion"], "2026-07-28");
                initialize_result()
            }
            "notifications/initialized" => {
                initialized = true;
                continue;
            }
            "tools/call" if initialized && request["params"]["name"] == "exit" => break,
            "tools/call" if initialized => json!({"content": [{"type": "text", "text": "pong"}]}),
            method if initialized => catalog_result(method).expect("unexpected catalog method"),
            _ => panic!("received {method} before initialization"),
        };
        let reply = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
        writeln!(stdout, "{reply}").unwrap();
        stdout.flush().unwrap();
    }
}

async fn http_upstream(headers: HeaderMap, Json(request): Json<Value>) -> Response {
    let method = request["method"].as_str();
    if method != Some("initialize")
        && headers
            .get("mcp-session-id")
            .is_none_or(|id| id != "legacy")
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let result = match method {
        Some("initialize") if request["params"]["protocolVersion"] == "2026-07-28" => {
            return (
                [("mcp-session-id", "legacy")],
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "result": initialize_result()
                })),
            )
                .into_response();
        }
        Some("notifications/initialized") => return StatusCode::NO_CONTENT.into_response(),
        Some("tools/call") if request["params"]["name"] == "large" => {
            json!({"content": [{"type": "text", "text": "x".repeat(16 * 1024 * 1024)}]})
        }
        Some("tools/call") => json!({"_meta": {"io.warmplane/test": "pong"}}),
        Some(method) => match catalog_result(method) {
            Some(result) => result,
            None => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => return StatusCode::BAD_REQUEST.into_response(),
    };
    let mut response =
        Json(json!({"jsonrpc": "2.0", "id": request["id"], "result": result})).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("Application/JSON"),
    );
    response
}

#[tokio::test]
async fn http_negotiates_a_legacy_session_and_enforces_response_limits() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let upstream = tokio::spawn(async move {
        axum::serve(listener, Router::new().route("/mcp", post(http_upstream)))
            .await
            .unwrap();
    });
    let state = state();
    let config = ServerConfig {
        command: None,
        url: Some(url),
        allow_stateless: Some(false),
        resilience: None,
        ..hanging_server()
    };
    let aliases = HashMap::new();
    let (tools, _, _, tx) =
        spawn_supervised_http_server(&state, "http", &config, &aliases, &aliases, &aliases)
            .await
            .expect("spawn HTTP supervisor");

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].0, "http.ping");
    let reply = call_tool(&tx, "ping").await.unwrap().unwrap();
    assert_eq!(reply["_meta"]["io.warmplane/test"], "pong");
    assert_eq!(reply["content"], json!([]));

    let oversized = call_tool(&tx, "large").await.unwrap();
    assert!(
        matches!(&oversized, Err(UpstreamCallError::Upstream(error)) if error.contains("maximum size of 16777216 bytes")),
        "expected the SDK response size limit, got {oversized:?}"
    );
    state.shutdown_token.cancel();
    upstream.abort();
}
