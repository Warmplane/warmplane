// Rust guideline compliant 2026-10-02

//! The stdio restart handshake must be bounded by the shared handshake timeout.
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

use std::collections::HashMap;
use std::time::Duration;

use warmplane::circuit_breaker::ResilienceConfig;
use warmplane::config::ServerConfig;
use warmplane::daemon::state::AppState;
use warmplane::daemon::types::{ServerMsg, UpstreamCallError};
use warmplane::supervisor::spawn_supervised_stdio_server;

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

/// Sends one tool call and waits for the supervisor to answer it.
async fn ask(tx: &tokio::sync::mpsc::Sender<ServerMsg>) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    tx.send(ServerMsg::CallTool {
        name: "noop".to_string(),
        params: serde_json::json!({}),
        input_responses: None,
        request_state: None,
        reply: reply_tx,
    })
    .await
    .map_err(|e| format!("supervisor mailbox rejected the message: {e}"))?;

    match tokio::time::timeout(ANSWER_DEADLINE, reply_rx).await {
        Err(_) => Err("supervisor never answered".to_string()),
        Ok(Err(_)) => Err("supervisor dropped the reply channel".to_string()),
        Ok(Ok(Ok(_))) => Ok(()),
        // The server has no working client, so a bounded upstream error is expected.
        Ok(Ok(Err(UpstreamCallError::Upstream(_)))) => Ok(()),
        Ok(Ok(Err(UpstreamCallError::Timeout))) => Err("upstream call timed out".to_string()),
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
