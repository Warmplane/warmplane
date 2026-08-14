// Rust guideline compliant 2026-08-14

//! Configuration import utilities from external MCP ecosystems (Claude Desktop, Cursor, Zed).

use anyhow::{Context, Result};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

use crate::config::{load_or_default_config, save_config, ServerConfig};

/// Discovered configuration source from the host system.
#[derive(Debug, Clone)]
pub struct DiscoveredSource {
    /// Ecosystem source name (e.g. "Claude Desktop", "Cursor").
    pub name: String,
    /// Absolute or expanded file path.
    pub path: PathBuf,
    /// Number of MCP servers found in the file.
    pub server_count: usize,
    /// Parsed server configurations keyed by server name.
    pub servers: HashMap<String, ServerConfig>,
}

/// Discovers known MCP configuration files on the local system.
///
/// # Returns
/// A vector of discovered configuration sources that exist and contain valid servers.
pub fn discover_sources() -> Vec<DiscoveredSource> {
    let mut sources = Vec::new();
    let home = std::env::var("HOME").ok().map(PathBuf::from);

    if let Some(ref home_dir) = home {
        // Claude Desktop on macOS
        let claude_mac =
            home_dir.join("Library/Application Support/Claude/claude_desktop_config.json");
        if claude_mac.exists() {
            if let Ok(src) = parse_standard_mcp_source("Claude Desktop (macOS)", claude_mac) {
                if src.server_count > 0 {
                    sources.push(src);
                }
            }
        }

        // Claude Desktop on Linux
        let claude_linux = home_dir.join(".config/Claude/claude_desktop_config.json");
        if claude_linux.exists() {
            if let Ok(src) = parse_standard_mcp_source("Claude Desktop (Linux)", claude_linux) {
                if src.server_count > 0 {
                    sources.push(src);
                }
            }
        }

        // Cursor Global
        let cursor_global = home_dir.join(".cursor/mcp.json");
        if cursor_global.exists() {
            if let Ok(src) = parse_standard_mcp_source("Cursor (Global)", cursor_global) {
                if src.server_count > 0 {
                    sources.push(src);
                }
            }
        }

        // Zed Editor
        let zed_settings = home_dir.join(".config/zed/settings.json");
        if zed_settings.exists() {
            if let Ok(src) = parse_standard_mcp_source("Zed Editor", zed_settings) {
                if src.server_count > 0 {
                    sources.push(src);
                }
            }
        }
    }

    // Cursor Local Workspace
    let cursor_local = PathBuf::from(".cursor/mcp.json");
    if cursor_local.exists() {
        if let Ok(src) = parse_standard_mcp_source("Cursor (Workspace)", cursor_local) {
            if src.server_count > 0 {
                sources.push(src);
            }
        }
    }

    sources
}

/// Parses standard `mcpServers` object from JSON file.
///
/// # Arguments
/// * `name` - Source name.
/// * `path` - File path.
///
/// # Returns
/// DiscoveredSource containing parsed servers.
///
/// # Errors
/// Returns an error if file cannot be read or JSON parsing fails.
pub fn parse_standard_mcp_source(name: &str, path: PathBuf) -> Result<DiscoveredSource> {
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let val: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON in {}", path.display()))?;

    let mut servers = HashMap::new();

    // Standard Claude/Cursor structure has top-level "mcpServers" key
    if let Some(mcp_servers_val) = val.get("mcpServers").and_then(Value::as_object) {
        for (server_name, server_obj) in mcp_servers_val {
            if let Ok(server_cfg) = serde_json::from_value::<ServerConfig>(server_obj.clone()) {
                servers.insert(server_name.clone(), server_cfg);
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
}
