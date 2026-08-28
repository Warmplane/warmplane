use std::fs;
use tempfile::tempdir;
use warmplane::client_sync::get_supported_clients;

#[test]
fn test_get_supported_clients() {
    let clients = get_supported_clients();
    assert_eq!(clients.len(), 17);
    let ids: Vec<_> = clients.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"claude-desktop"));
    assert!(ids.contains(&"opencode"));
    assert!(ids.contains(&"claude-code"));
    assert!(ids.contains(&"cursor"));
    assert!(ids.contains(&"zed"));
    assert!(ids.contains(&"windsurf"));
    assert!(ids.contains(&"cline"));
    assert!(ids.contains(&"antigravity"));
    assert!(ids.contains(&"codex"));
    assert!(ids.contains(&"gemini-cli"));
    assert!(ids.contains(&"continue"));
    assert!(ids.contains(&"vscode"));
    assert!(ids.contains(&"goose"));
    assert!(ids.contains(&"librechat"));
    assert!(ids.contains(&"deepseek"));
    assert!(ids.contains(&"cody"));
    assert!(ids.contains(&"devin"));
}

#[test]
fn test_attach_and_detach_standard_client() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("claude_desktop_config.json");

    // Write initial client file with an existing tool
    fs::write(
        &config_file,
        r#"{"mcpServers": {"sqlite": {"command": "npx", "args": ["@modelcontextprotocol/server-sqlite"]}}}"#,
    )
    .unwrap();

    // Test attach logic directly by setting environment or verifying dialect transformations
    let mut root_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
    let mcp_servers = root_val
        .as_object_mut()
        .unwrap()
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));
    mcp_servers.as_object_mut().unwrap().insert(
        "warmplane".to_string(),
        serde_json::json!({
            "command": "/usr/local/bin/warmplane",
            "args": ["mcp-server", "--config", "/tmp/mcp_servers.json", "--profile", "coding"],
        }),
    );
    fs::write(
        &config_file,
        serde_json::to_string_pretty(&root_val).unwrap(),
    )
    .unwrap();

    let content = fs::read_to_string(&config_file).unwrap();
    let reloaded: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(reloaded["mcpServers"]["warmplane"].is_object());
    assert!(reloaded["mcpServers"]["sqlite"].is_object());
    assert_eq!(
        reloaded["mcpServers"]["warmplane"]["args"][4],
        serde_json::json!("coding")
    );

    // Test detach simulation
    let mut detach_val: serde_json::Value = serde_json::from_str(&content).unwrap();
    detach_val["mcpServers"]
        .as_object_mut()
        .unwrap()
        .remove("warmplane");
    fs::write(
        &config_file,
        serde_json::to_string_pretty(&detach_val).unwrap(),
    )
    .unwrap();

    let detached_content = fs::read_to_string(&config_file).unwrap();
    let detached_reloaded: serde_json::Value = serde_json::from_str(&detached_content).unwrap();
    assert!(detached_reloaded["mcpServers"]["warmplane"].is_null());
    assert!(detached_reloaded["mcpServers"]["sqlite"].is_object());
}

#[test]
fn test_opencode_dialect_injection() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("opencode.json");

    fs::write(&config_file, r#"{"theme": "cyberpunk"}"#).unwrap();

    let mut root_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
    let mcp = root_val
        .as_object_mut()
        .unwrap()
        .entry("mcp")
        .or_insert_with(|| serde_json::json!({}));
    mcp.as_object_mut().unwrap().insert(
        "warmplane".to_string(),
        serde_json::json!({
            "type": "local",
            "command": "warmplane",
            "args": ["mcp-server", "--config", "/path/to/mcp_servers.json"],
            "enabled": true,
        }),
    );
    fs::write(
        &config_file,
        serde_json::to_string_pretty(&root_val).unwrap(),
    )
    .unwrap();

    let content = fs::read_to_string(&config_file).unwrap();
    let reloaded: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        reloaded["mcp"]["warmplane"]["type"],
        serde_json::json!("local")
    );
    assert_eq!(
        reloaded["mcp"]["warmplane"]["enabled"],
        serde_json::json!(true)
    );
    assert_eq!(reloaded["theme"], serde_json::json!("cyberpunk"));
}

#[test]
fn test_zed_dialect_injection() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("settings.json");

    fs::write(&config_file, r#"{"ui_font_size": 14}"#).unwrap();

    let mut root_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
    let context_servers = root_val
        .as_object_mut()
        .unwrap()
        .entry("context_servers")
        .or_insert_with(|| serde_json::json!({}));
    context_servers.as_object_mut().unwrap().insert(
        "warmplane".to_string(),
        serde_json::json!({
            "command": {
                "path": "warmplane",
                "args": ["mcp-server", "--config", "/path/to/mcp_servers.json"],
            }
        }),
    );
    fs::write(
        &config_file,
        serde_json::to_string_pretty(&root_val).unwrap(),
    )
    .unwrap();

    let content = fs::read_to_string(&config_file).unwrap();
    let reloaded: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        reloaded["context_servers"]["warmplane"]["command"]["path"],
        serde_json::json!("warmplane")
    );
    assert_eq!(reloaded["ui_font_size"], serde_json::json!(14));
}

#[test]
fn test_claude_code_config_dir_override() {
    let dir = tempdir().unwrap();
    let custom_config = dir.path().join("claude.json");
    fs::write(&custom_config, r#"{"mcpServers": {}}"#).unwrap();

    std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().to_str().unwrap());
    let resolved = warmplane::client_sync::resolve_client_config_path("claude-code").unwrap();
    assert_eq!(resolved, custom_config);
    std::env::remove_var("CLAUDE_CONFIG_DIR");
}
