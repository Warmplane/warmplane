// Rust guideline compliant 2026-08-29

//! Integration tests for passthrough tool exposure and direct invocation in Warmplane.

use serde_json::{json, Value};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time::timeout,
};

use warmplane::config::{save_config, AliasTarget, McpConfig};

async fn read_jsonrpc_until_id<R: AsyncBufReadExt + Unpin>(
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

#[tokio::test]
async fn test_mcp_passthrough_tools_listing_and_execution() {
    let bin_path = env!("CARGO_BIN_EXE_warmplane");

    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mut mcp_config = McpConfig::default();
    mcp_config.capability_aliases.insert(
        "search_docs".to_string(),
        AliasTarget::Detailed {
            target: "mock_fs.search".to_string(),
            summary: Some("Search indexed documentation".to_string()),
            description: Some("Search documentation using full text".to_string()),
            passthrough: true,
        },
    );
    mcp_config.capability_aliases.insert(
        "hidden_internal".to_string(),
        AliasTarget::Detailed {
            target: "mock_fs.internal".to_string(),
            summary: Some("Internal hidden operation".to_string()),
            description: None,
            passthrough: false,
        },
    );
    save_config(&config_path, &mcp_config).unwrap();

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

    // 1. Initialize handshake
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "test-client", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let init_resp = read_jsonrpc_until_id(&mut reader, 1).await;
    assert_eq!(init_resp["id"], 1);

    // 2. Initialized notification
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

    // Core facade tools exist
    assert!(tool_names.contains(&"capabilities_list"));
    assert!(tool_names.contains(&"capability_call"));

    // Passthrough alias exists directly in tools/list
    assert!(
        tool_names.contains(&"search_docs"),
        "Passthrough tool 'search_docs' should be in tools/list"
    );

    // Non-passthrough alias should NOT exist
    assert!(
        !tool_names.contains(&"hidden_internal"),
        "Non-passthrough alias 'hidden_internal' should NOT be in tools/list"
    );

    // 4. Call the passthrough tool directly with flat arguments
    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "search_docs",
            "arguments": {
                "query": "architecture overview"
            }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&call_req).unwrap()).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let call_resp = read_jsonrpc_until_id(&mut reader, 3).await;
    assert_eq!(call_resp["id"], 3);
    assert!(
        call_resp["result"]["structuredContent"].is_object()
            || call_resp["result"]["content"].is_array()
    );

    // 5. Clean exit
    drop(child.stdin.take());
    let exit_status = timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("timeout waiting for child process exit")
        .unwrap();
    assert!(exit_status.success());
}
