// Rust guideline compliant 2026-09-14

//! Shared test utilities and helper functions for integration tests (`M-CANONICAL-DOCS`).

use serde_json::Value;
use std::time::Duration;
use tokio::{io::AsyncBufReadExt, time::timeout};

/// Reads lines from an async buffered reader until a JSON-RPC response with the expected `id` arrives.
///
/// # Arguments
/// * `reader` - Mutable reference to an asynchronous buffered line reader.
/// * `expected_id` - Numerical JSON-RPC message ID expected in the response payload.
///
/// # Returns
/// The deserialized `serde_json::Value` payload matching `expected_id`.
///
/// # Panics
/// Panics if the stream closes before receiving `expected_id` or if reading times out (5 seconds).
pub async fn read_jsonrpc_until_id<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
    expected_id: u64,
) -> Value {
    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = timeout(Duration::from_secs(5), reader.read_line(&mut line))
            .await
            .expect("timeout waiting for JSON-RPC message")
            .expect("stdout read error");

        if bytes_read == 0 {
            panic!("stdout closed before receiving id={}", expected_id);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('{') {
            continue;
        }

        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            if val.get("id").and_then(Value::as_u64) == Some(expected_id) {
                return val;
            }
        }
    }
}
