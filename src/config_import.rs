// Rust guideline compliant 2026-08-27

//! Configuration import utilities from external MCP ecosystems (Claude Desktop, OpenCode, Claude Code, Cursor, Zed, Windsurf, Roo Code / Cline).

use anyhow::{Context, Result};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

use crate::client_sync::{get_supported_clients, resolve_client_config_path, ClientDialect};
use crate::config::{load_or_default_config, save_config, ServerConfig};

/// Discovered configuration source from the host system.
#[derive(Debug, Clone)]
pub struct DiscoveredSource {
    /// Ecosystem source name (e.g. "Claude Desktop", "OpenCode", "Cursor").
    pub name: String,
    /// Absolute or expanded file path.
    pub path: PathBuf,
    /// Number of MCP servers found in the file.
    pub server_count: usize,
    /// Parsed server configurations keyed by server name.
    pub servers: HashMap<String, ServerConfig>,
}

/// Discovers known MCP configuration files on the local system across all supported agent ecosystems.
///
/// # Returns
/// A vector of discovered configuration sources that exist and contain valid servers.
pub fn discover_sources() -> Vec<DiscoveredSource> {
    let mut sources = Vec::new();

    // Iterate through all supported 1-click client adapters
    for client in get_supported_clients() {
        if let Some(path) = resolve_client_config_path(&client.id) {
            if path.exists() {
                if let Ok(src) = parse_client_dialect_source(&client.name, path, client.dialect) {
                    if src.server_count > 0 {
                        sources.push(src);
                    }
                }
            }
        }
    }

    // Cursor Local Workspace (.cursor/mcp.json)
    let cursor_local = PathBuf::from(".cursor/mcp.json");
    if cursor_local.exists() {
        if let Ok(src) = parse_client_dialect_source(
            "Cursor (Workspace)",
            cursor_local,
            ClientDialect::StandardMcpServers,
        ) {
            if src.server_count > 0 {
                sources.push(src);
            }
        }
    }

    sources
}

/// Parses an MCP configuration file according to a specific client dialect.
///
/// # Arguments
/// * `name` - Source name.
/// * `path` - File path.
/// * `dialect` - Target configuration dialect (`StandardMcpServers`, `OpenCodeMcp`, `ZedContextServers`).
///
/// # Returns
/// DiscoveredSource containing parsed servers.
///
/// # Errors
/// Returns an error if file reading or JSON parsing fails.
pub fn parse_client_dialect_source(
    name: &str,
    path: PathBuf,
    dialect: ClientDialect,
) -> Result<DiscoveredSource> {
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let val: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON in {}", path.display()))?;

    let mut servers = HashMap::new();

    match dialect {
        ClientDialect::StandardMcpServers => {
            if let Some(mcp_servers_val) = val.get("mcpServers").and_then(Value::as_object) {
                for (s_name, s_obj) in mcp_servers_val {
                    if s_name == "warmplane" {
                        continue;
                    }
                    if let Ok(server_cfg) = serde_json::from_value::<ServerConfig>(s_obj.clone()) {
                        servers.insert(s_name.clone(), server_cfg);
                    }
                }
            }
        }
        ClientDialect::OpenCodeMcp => {
            if let Some(mcp_val) = val.get("mcp").and_then(Value::as_object) {
                for (s_name, s_obj) in mcp_val {
                    if s_name == "warmplane" {
                        continue;
                    }
                    if let Some(cmd) = s_obj.get("command").and_then(Value::as_str) {
                        let args = s_obj
                            .get("args")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let env = s_obj
                            .get("env")
                            .and_then(Value::as_object)
                            .map(|obj| {
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        servers.insert(
                            s_name.clone(),
                            ServerConfig {
                                command: Some(cmd.to_string()),
                                args,
                                env,
                                url: None,
                                protocol_version: None,
                                allow_stateless: None,
                                headers: HashMap::new(),
                                auth: None,
                                resilience: None,
                            },
                        );
                    } else if let Some(url) = s_obj.get("url").and_then(Value::as_str) {
                        servers.insert(
                            s_name.clone(),
                            ServerConfig {
                                command: None,
                                args: vec![],
                                env: HashMap::new(),
                                url: Some(url.to_string()),
                                protocol_version: None,
                                allow_stateless: None,
                                headers: HashMap::new(),
                                auth: None,
                                resilience: None,
                            },
                        );
                    }
                }
            }
        }
        ClientDialect::ZedContextServers => {
            if let Some(ctx_val) = val.get("context_servers").and_then(Value::as_object) {
                for (s_name, s_obj) in ctx_val {
                    if s_name == "warmplane" {
                        continue;
                    }
                    if let Some(cmd_obj) = s_obj.get("command").and_then(Value::as_object) {
                        let path_str = cmd_obj.get("path").and_then(Value::as_str);
                        if let Some(p) = path_str {
                            let args = cmd_obj
                                .get("args")
                                .and_then(Value::as_array)
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let env = cmd_obj
                                .get("env")
                                .and_then(Value::as_object)
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            servers.insert(
                                s_name.clone(),
                                ServerConfig {
                                    command: Some(p.to_string()),
                                    args,
                                    env,
                                    url: None,
                                    protocol_version: None,
                                    allow_stateless: None,
                                    headers: HashMap::new(),
                                    auth: None,
                                    resilience: None,
                                },
                            );
                        }
                    } else if let Some(url) = s_obj.get("url").and_then(Value::as_str) {
                        servers.insert(
                            s_name.clone(),
                            ServerConfig {
                                command: None,
                                args: vec![],
                                env: HashMap::new(),
                                url: Some(url.to_string()),
                                protocol_version: None,
                                allow_stateless: None,
                                headers: HashMap::new(),
                                auth: None,
                                resilience: None,
                            },
                        );
                    }
                }
            }
        }
    }

    let server_count = servers.len();
    Ok(DiscoveredSource {
        name: name.to_string(),
        path,
        server_count,
        servers,
    })
}

/// Parses standard `mcpServers` or auto-detected MCP server configurations from any JSON file.
///
/// # Arguments
/// * `name` - Source name.
/// * `path` - File path.
///
/// # Returns
/// DiscoveredSource containing parsed servers.
///
/// # Errors
/// Returns an error if file reading or JSON parsing fails.
pub fn parse_standard_mcp_source(name: &str, path: PathBuf) -> Result<DiscoveredSource> {
    if let Ok(src) =
        parse_client_dialect_source(name, path.clone(), ClientDialect::StandardMcpServers)
    {
        if src.server_count > 0 {
            return Ok(src);
        }
    }
    if let Ok(src) = parse_client_dialect_source(name, path.clone(), ClientDialect::OpenCodeMcp) {
        if src.server_count > 0 {
            return Ok(src);
        }
    }
    if let Ok(src) =
        parse_client_dialect_source(name, path.clone(), ClientDialect::ZedContextServers)
    {
        if src.server_count > 0 {
            return Ok(src);
        }
    }
    parse_client_dialect_source(name, path, ClientDialect::StandardMcpServers)
}

/// Imports servers from a map into the Warmplane target configuration file.
///
/// # Arguments
/// * `config_path` - Path to Warmplane config file.
/// * `imported_servers` - Map of server name to server configuration.
/// * `overwrite` - Whether to overwrite existing servers.
///
/// # Returns
/// A tuple containing (number of imported servers, list of skipped server names).
///
/// # Errors
/// Returns an error if saving config fails.
pub fn import_servers_into_config(
    config_path: &str,
    imported_servers: HashMap<String, ServerConfig>,
    overwrite: bool,
) -> Result<(usize, Vec<String>)> {
    let mut config = load_or_default_config(config_path)?;
    let mut imported_count = 0;
    let mut skipped = Vec::new();

    for (name, server) in imported_servers {
        if config.mcp_servers.contains_key(&name) && !overwrite {
            skipped.push(name);
        } else {
            config.mcp_servers.insert(name, server);
            imported_count += 1;
        }
    }

    save_config(config_path, &config)?;
    Ok((imported_count, skipped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_claude_desktop_format() {
        let json_str = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
                },
                "github": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-github"],
                    "env": {
                        "GITHUB_PERSONAL_ACCESS_TOKEN": "token123"
                    }
                },
                "warmplane": {
                    "command": "warmplane",
                    "args": ["stdio"]
                }
            }
        }"#;

        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_import_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("claude_desktop_config.json");
        std::fs::write(&path, json_str).unwrap();

        let source = parse_standard_mcp_source("Test Claude", path).unwrap();
        assert_eq!(source.server_count, 2);
        assert!(source.servers.contains_key("filesystem"));
        assert!(source.servers.contains_key("github"));
        assert!(!source.servers.contains_key("warmplane")); // Ensures warmplane self-entry is ignored

        let target_cfg_path = temp_dir.join("mcp_servers.json");
        let (imported, skipped) =
            import_servers_into_config(target_cfg_path.to_str().unwrap(), source.servers, false)
                .unwrap();

        assert_eq!(imported, 2);
        assert_eq!(skipped.len(), 0);

        let reloaded = load_or_default_config(target_cfg_path.to_str().unwrap()).unwrap();
        assert_eq!(reloaded.mcp_servers.len(), 2);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn parse_opencode_format() {
        let json_str = r#"{
            "mcp": {
                "sqlite": {
                    "type": "local",
                    "command": "uvx",
                    "args": ["mcp-server-sqlite", "--db-path", "app.db"],
                    "enabled": true
                },
                "warmplane": {
                    "type": "local",
                    "command": "warmplane",
                    "args": ["stdio"]
                }
            }
        }"#;

        let temp_dir = std::env::temp_dir().join(format!(
            "warmplane_opencode_import_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("opencode.json");
        std::fs::write(&path, json_str).unwrap();

        let source =
            parse_client_dialect_source("OpenCode", path, ClientDialect::OpenCodeMcp).unwrap();
        assert_eq!(source.server_count, 1);
        assert!(source.servers.contains_key("sqlite"));
        assert_eq!(source.servers["sqlite"].command.as_deref(), Some("uvx"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn parse_zed_format() {
        let json_str = r#"{
            "context_servers": {
                "memory": {
                    "command": {
                        "path": "npx",
                        "args": ["-y", "@modelcontextprotocol/server-memory"]
                    }
                }
            }
        }"#;

        let temp_dir =
            std::env::temp_dir().join(format!("warmplane_zed_import_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("settings.json");
        std::fs::write(&path, json_str).unwrap();

        let source =
            parse_client_dialect_source("Zed Editor", path, ClientDialect::ZedContextServers)
                .unwrap();
        assert_eq!(source.server_count, 1);
        assert!(source.servers.contains_key("memory"));
        assert_eq!(source.servers["memory"].command.as_deref(), Some("npx"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
