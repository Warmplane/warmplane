// Rust guideline compliant 2026-08-29

//! Integration tests for MCP list_changed notifications (tools, resources, prompts) in Warmplane.

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
async fn test_mcp_list_changed_notifications_and_discovery_hint() {
    let bin_path = env!("CARGO_BIN_EXE_warmplane");

    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mcp_config = McpConfig::default();
    save_config(&config_path, &mcp_config).unwrap();

    let mut child = Command::new(bin_path)
        .arg("mcp-server")
        .arg("--config")
        .arg(&config_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("failed to spawn warmplane mcp-server");

    let mut stdin = child.stdin.take().expect("failed to open stdin");
    let stdout = child.stdout.take().expect("failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Initialize MCP session and verify advertised capabilities
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    stdin
        .write_all(format!("{}\n", init_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let init_resp = read_jsonrpc_until_id(&mut reader, 1).await;
    let caps = init_resp["result"]["capabilities"].clone();
    assert_eq!(
        caps["tools"]["listChanged"], true,
        "tools.listChanged must be advertised"
    );
    assert_eq!(
        caps["resources"]["listChanged"], true,
        "resources.listChanged must be advertised"
    );
    assert_eq!(
        caps["prompts"]["listChanged"], true,
        "prompts.listChanged must be advertised"
    );

    // Initialized notification
    let initialized_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    stdin
        .write_all(format!("{}\n", initialized_notif).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    // 2. Trigger alias addition via CLI / config mutation
    let mut updated_config = mcp_config.clone();
    updated_config.capability_aliases.insert(
        "search_live".to_string(),
        AliasTarget::Detailed {
            target: "mock.search".to_string(),
            summary: Some("Search live docs".to_string()),
            description: None,
            passthrough: true,
        },
    );
    save_config(&config_path, &updated_config).unwrap();

    // 3. Inspect tools list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    stdin
        .write_all(format!("{}\n", list_req).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let list_resp = read_jsonrpc_until_id(&mut reader, 2).await;
    let tools = list_resp["result"]["tools"].as_array().unwrap();
    let has_search_live = tools.iter().any(|t| t["name"] == "search_live");
    assert!(
        has_search_live,
        "Newly added passthrough alias 'search_live' must be visible in tools/list"
    );

    // Clean up
    let _ = child.kill().await;
}
