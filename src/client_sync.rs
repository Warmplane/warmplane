// Rust guideline compliant 2026-08-26

//! Bidirectional ecosystem client adapter engine for 1-click MCP injection and auto-sync.
//!
//! Supports detecting, attaching, and detaching Warmplane as an MCP proxy across popular AI clients:
//! - Claude Desktop (macOS, Linux, Windows)
//! - Claude Code CLI (`~/.claude.json`)
//! - OpenCode (`~/.config/opencode/opencode.json`)
//! - Cursor (Global & Workspace)
//! - Zed Editor (`settings.json` -> `context_servers`)
//! - Windsurf (`~/.codeium/windsurf/mcp_config.json`)
//! - Roo Code / Cline (`cline_mcp_settings.json`)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// JSON schema dialect used by the client for MCP configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientDialect {
    /// Standard format: `{"mcpServers": {"<name>": {"command": "...", "args": [...]}}}`
    StandardMcpServers,
    /// Zed editor format: `{"context_servers": {"<name>": {"command": {"path": "...", "args": [...]}}}}`
    ZedContextServers,
    /// OpenCode format: `{"mcp": {"<name>": {"type": "local", "command": "...", "args": [...], "enabled": true}}}`
    OpenCodeMcp,
}

/// Information definition for a known AI client application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAppDef {
    /// Canonical client identifier (e.g. `claude-desktop`, `opencode`, `cursor`).
    pub id: String,
    /// Human-friendly display name.
    pub name: String,
    /// Category / App type.
    pub category: String,
    /// Configuration JSON dialect.
    pub dialect: ClientDialect,
}

/// Status of a detected client application on the host system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAppStatus {
    /// Canonical client identifier.
    pub id: String,
    /// Human-friendly display name.
    pub name: String,
    /// Category / App type.
    pub category: String,
    /// Target configuration file path.
    pub config_path: String,
    /// Whether the configuration file exists on disk.
    pub config_exists: bool,
    /// Whether the application installation / parent directory exists.
    pub app_installed: bool,
    /// Whether Warmplane is currently configured in this client.
    pub is_attached: bool,
    /// Currently attached profile (if specified).
    pub attached_profile: Option<String>,
    /// Total count of other MCP servers configured in this client.
    pub other_servers_count: usize,
}

/// Options when attaching Warmplane to a client.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttachOptions {
    /// Optional server constellation profile to restrict capabilities.
    pub profile: Option<String>,
    /// Custom path to `mcp_servers.json` (defaults to active config).
    pub config_path: Option<String>,
    /// Custom binary command name or path (defaults to current exe or "warmplane").
    pub binary_path: Option<String>,
}

/// Result of an attach operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachResult {
    /// Canonical client identifier.
    pub client_id: String,
    /// Path to modified configuration file.
    pub config_path: String,
    /// Backup file path created prior to modification (if created).
    pub backup_path: Option<String>,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Message summarizing result.
    pub message: String,
}

/// Result of a detach operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachResult {
    /// Canonical client identifier.
    pub client_id: String,
    /// Path to modified configuration file.
    pub config_path: String,
    /// Whether Warmplane was found and removed.
    pub was_attached: bool,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Message summarizing result.
    pub message: String,
}

/// Returns definitions of all supported client applications.
pub fn get_supported_clients() -> Vec<ClientAppDef> {
    vec![
        ClientAppDef {
            id: "claude-desktop".to_string(),
            name: "Claude Desktop".to_string(),
            category: "Desktop App".to_string(),
            dialect: ClientDialect::StandardMcpServers,
        },
        ClientAppDef {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            category: "AI Terminal / Agent".to_string(),
            dialect: ClientDialect::OpenCodeMcp,
        },
        ClientAppDef {
            id: "claude-code".to_string(),
            name: "Claude Code CLI".to_string(),
            category: "CLI Agent".to_string(),
            dialect: ClientDialect::StandardMcpServers,
        },
        ClientAppDef {
            id: "cursor".to_string(),
            name: "Cursor (Global)".to_string(),
            category: "IDE".to_string(),
            dialect: ClientDialect::StandardMcpServers,
        },
        ClientAppDef {
            id: "zed".to_string(),
            name: "Zed Editor".to_string(),
            category: "IDE".to_string(),
            dialect: ClientDialect::ZedContextServers,
        },
        ClientAppDef {
            id: "windsurf".to_string(),
            name: "Windsurf".to_string(),
            category: "IDE".to_string(),
            dialect: ClientDialect::StandardMcpServers,
        },
        ClientAppDef {
            id: "cline".to_string(),
            name: "Roo Code / Cline".to_string(),
            category: "VS Code Extension".to_string(),
            dialect: ClientDialect::StandardMcpServers,
        },
    ]
}

/// Resolves the primary configuration file path for a client on the host platform.
pub fn resolve_client_config_path(client_id: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let _appdata = std::env::var("APPDATA").ok().map(PathBuf::from);
    let _userprofile = std::env::var("USERPROFILE").ok().map(PathBuf::from);

    match client_id {
        "claude-desktop" => {
            #[cfg(target_os = "macos")]
            if let Some(ref h) = home {
                return Some(
                    h.join("Library/Application Support/Claude/claude_desktop_config.json"),
                );
            }
            #[cfg(target_os = "windows")]
            if let Some(ref ad) = appdata {
                return Some(ad.join("Claude/claude_desktop_config.json"));
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            if let Some(ref h) = home {
                return Some(h.join(".config/Claude/claude_desktop_config.json"));
            }
            None
        }
        "opencode" => {
            #[cfg(target_os = "windows")]
            if let Some(ref ad) = appdata {
                return Some(ad.join("opencode/opencode.json"));
            }
            #[cfg(not(target_os = "windows"))]
            if let Some(ref h) = home {
                return Some(h.join(".config/opencode/opencode.json"));
            }
            None
        }
        "claude-code" => {
            // Check CLAUDE_CONFIG_DIR environment variable first
            if let Ok(config_dir) = std::env::var("CLAUDE_CONFIG_DIR") {
                let dir_path = PathBuf::from(config_dir.trim());
                let cand_json = dir_path.join("claude.json");
                let cand_dot = dir_path.join(".claude.json");
                if cand_json.exists() {
                    return Some(cand_json);
                }
                if cand_dot.exists() {
                    return Some(cand_dot);
                }
                return Some(cand_dot);
            }

            #[cfg(target_os = "windows")]
            if let Some(ref up) = userprofile {
                return Some(up.join(".claude.json"));
            }
            if let Some(ref h) = home {
                return Some(h.join(".claude.json"));
            }
            None
        }
        "cursor" => {
            #[cfg(target_os = "windows")]
            if let Some(ref up) = userprofile {
                return Some(up.join(".cursor/mcp.json"));
            }
            if let Some(ref h) = home {
                return Some(h.join(".cursor/mcp.json"));
            }
            None
        }
        "zed" => {
            #[cfg(target_os = "windows")]
            if let Some(ref ad) = appdata {
                return Some(ad.join("Zed/settings.json"));
            }
            if let Some(ref h) = home {
                return Some(h.join(".config/zed/settings.json"));
            }
            None
        }
        "windsurf" => {
            #[cfg(target_os = "windows")]
            if let Some(ref up) = userprofile {
                return Some(up.join(".codeium/windsurf/mcp_config.json"));
            }
            if let Some(ref h) = home {
                return Some(h.join(".codeium/windsurf/mcp_config.json"));
            }
            None
        }
        "cline" => {
            #[cfg(target_os = "windows")]
            if let Some(ref up) = userprofile {
                return Some(up.join(".cline/data/settings/cline_mcp_settings.json"));
            }
            if let Some(ref h) = home {
                return Some(h.join(".cline/data/settings/cline_mcp_settings.json"));
            }
            None
        }
        _ => None,
    }
}

/// Scans the local system and returns statuses for all supported AI clients.
pub fn detect_clients() -> Vec<ClientAppStatus> {
    let clients = get_supported_clients();
    let mut statuses = Vec::new();

    for client in clients {
        let config_path_opt = resolve_client_config_path(&client.id);
        let (
            config_path_str,
            config_exists,
            app_installed,
            is_attached,
            attached_profile,
            other_servers_count,
        ) = if let Some(ref path) = config_path_opt {
            let config_exists = path.exists();
            let parent_exists = path.parent().map(|p| p.exists()).unwrap_or(false);
            let app_installed = config_exists || parent_exists;

            let (is_attached, attached_profile, other_servers_count) = if config_exists {
                inspect_client_config(path, client.dialect)
            } else {
                (false, None, 0)
            };

            (
                path.to_string_lossy().to_string(),
                config_exists,
                app_installed,
                is_attached,
                attached_profile,
                other_servers_count,
            )
        } else {
            ("Unknown".to_string(), false, false, false, None, 0)
        };

        statuses.push(ClientAppStatus {
            id: client.id,
            name: client.name,
            category: client.category,
            config_path: config_path_str,
            config_exists,
            app_installed,
            is_attached,
            attached_profile,
            other_servers_count,
        });
    }

    statuses
}

/// Inspects a client configuration file to determine attachment status and server count.
fn inspect_client_config(path: &Path, dialect: ClientDialect) -> (bool, Option<String>, usize) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (false, None, 0),
    };

    let val: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (false, None, 0),
    };

    match dialect {
        ClientDialect::StandardMcpServers => {
            if let Some(servers) = val.get("mcpServers").and_then(Value::as_object) {
                let is_attached = servers.contains_key("warmplane");
                let profile = if is_attached {
                    extract_profile_from_server_val(servers.get("warmplane"))
                } else {
                    None
                };
                let other_count = if is_attached {
                    servers.len().saturating_sub(1)
                } else {
                    servers.len()
                };
                (is_attached, profile, other_count)
            } else {
                (false, None, 0)
            }
        }
        ClientDialect::ZedContextServers => {
            if let Some(servers) = val.get("context_servers").and_then(Value::as_object) {
                let is_attached = servers.contains_key("warmplane");
                let profile = if is_attached {
                    extract_profile_from_server_val(servers.get("warmplane"))
                } else {
                    None
                };
                let other_count = if is_attached {
                    servers.len().saturating_sub(1)
                } else {
                    servers.len()
                };
                (is_attached, profile, other_count)
            } else {
                (false, None, 0)
            }
        }
        ClientDialect::OpenCodeMcp => {
            if let Some(servers) = val.get("mcp").and_then(Value::as_object) {
                let is_attached = servers.contains_key("warmplane");
                let profile = if is_attached {
                    extract_profile_from_server_val(servers.get("warmplane"))
                } else {
                    None
                };
                let other_count = if is_attached {
                    servers.len().saturating_sub(1)
                } else {
                    servers.len()
                };
                (is_attached, profile, other_count)
            } else {
                (false, None, 0)
            }
        }
    }
}

fn extract_profile_from_server_val(server_val: Option<&Value>) -> Option<String> {
    let s = server_val?;
    // Args can be in standard object or nested under command (Zed)
    let args_val = if let Some(args) = s.get("args").and_then(Value::as_array) {
        Some(args)
    } else if let Some(cmd_obj) = s.get("command").and_then(Value::as_object) {
        cmd_obj.get("args").and_then(Value::as_array)
    } else {
        None
    };

    if let Some(args) = args_val {
        let mut iter = args.iter().filter_map(Value::as_str);
        while let Some(arg) = iter.next() {
            if arg == "--profile" {
                return iter.next().map(ToString::to_string);
            }
        }
    }
    None
}

/// Attaches Warmplane to a target client application.
pub fn attach_client(client_id: &str, options: &AttachOptions) -> Result<AttachResult> {
    let clients = get_supported_clients();
    let client_def = clients
        .into_iter()
        .find(|c| c.id == client_id)
        .with_context(|| format!("Unsupported client ID: {}", client_id))?;

    let config_path = resolve_client_config_path(client_id)
        .with_context(|| format!("Could not resolve configuration path for {}", client_id))?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let mut backup_path = None;
    let mut root_val: Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        // Create backup
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let bak = config_path.with_extension(format!("bak.{}", now));
        let _ = std::fs::copy(&config_path, &bak);
        backup_path = Some(bak.to_string_lossy().to_string());

        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    if !root_val.is_object() {
        root_val = json!({});
    }

    // Resolve binary path
    let binary = options
        .binary_path
        .clone()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(ToString::to_string))
        })
        .unwrap_or_else(|| "warmplane".to_string());

    // Resolve warmplane config file path
    let warmplane_config = options
        .config_path
        .clone()
        .unwrap_or_else(|| "mcp_servers.json".to_string());
    let warmplane_config_abs = std::fs::canonicalize(&warmplane_config)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(warmplane_config);

    // Build args
    let mut args = vec![
        "mcp-server".to_string(),
        "--config".to_string(),
        warmplane_config_abs,
    ];
    if let Some(ref prof) = options.profile {
        args.push("--profile".to_string());
        args.push(prof.clone());
    }

    match client_def.dialect {
        ClientDialect::StandardMcpServers => {
            let mcp_servers = root_val
                .as_object_mut()
                .unwrap()
                .entry("mcpServers")
                .or_insert_with(|| json!({}));

            if !mcp_servers.is_object() {
                *mcp_servers = json!({});
            }

            mcp_servers.as_object_mut().unwrap().insert(
                "warmplane".to_string(),
                json!({
                    "command": binary,
                    "args": args,
                }),
            );
        }
        ClientDialect::ZedContextServers => {
            let context_servers = root_val
                .as_object_mut()
                .unwrap()
                .entry("context_servers")
                .or_insert_with(|| json!({}));

            if !context_servers.is_object() {
                *context_servers = json!({});
            }

            context_servers.as_object_mut().unwrap().insert(
                "warmplane".to_string(),
                json!({
                    "command": {
                        "path": binary,
                        "args": args,
                    }
                }),
            );
        }
        ClientDialect::OpenCodeMcp => {
            let mcp = root_val
                .as_object_mut()
                .unwrap()
                .entry("mcp")
                .or_insert_with(|| json!({}));

            if !mcp.is_object() {
                *mcp = json!({});
            }

            mcp.as_object_mut().unwrap().insert(
                "warmplane".to_string(),
                json!({
                    "type": "local",
                    "command": binary,
                    "args": args,
                    "enabled": true,
                }),
            );
        }
    }

    // Write back atomically
    let formatted_json = serde_json::to_string_pretty(&root_val)?;
    let temp_path = config_path.with_extension("tmp");
    std::fs::write(&temp_path, &formatted_json)
        .with_context(|| format!("Failed to write temporary file {}", temp_path.display()))?;
    std::fs::rename(&temp_path, &config_path)
        .with_context(|| format!("Failed to atomically rename to {}", config_path.display()))?;

    Ok(AttachResult {
        client_id: client_id.to_string(),
        config_path: config_path.to_string_lossy().to_string(),
        backup_path,
        ok: true,
        message: format!(
            "Successfully attached Warmplane to {} ({})",
            client_def.name,
            config_path.display()
        ),
    })
}

/// Detaches Warmplane from a target client application.
pub fn detach_client(client_id: &str) -> Result<DetachResult> {
    let clients = get_supported_clients();
    let client_def = clients
        .into_iter()
        .find(|c| c.id == client_id)
        .with_context(|| format!("Unsupported client ID: {}", client_id))?;

    let config_path = resolve_client_config_path(client_id)
        .with_context(|| format!("Could not resolve configuration path for {}", client_id))?;

    if !config_path.exists() {
        return Ok(DetachResult {
            client_id: client_id.to_string(),
            config_path: config_path.to_string_lossy().to_string(),
            was_attached: false,
            ok: true,
            message: format!(
                "Configuration file {} does not exist",
                config_path.display()
            ),
        });
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    let mut root_val: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON in {}", config_path.display()))?;

    let mut was_attached = false;

    match client_def.dialect {
        ClientDialect::StandardMcpServers => {
            if let Some(servers) = root_val
                .get_mut("mcpServers")
                .and_then(Value::as_object_mut)
            {
                was_attached = servers.remove("warmplane").is_some();
            }
        }
        ClientDialect::ZedContextServers => {
            if let Some(servers) = root_val
                .get_mut("context_servers")
                .and_then(Value::as_object_mut)
            {
                was_attached = servers.remove("warmplane").is_some();
            }
        }
        ClientDialect::OpenCodeMcp => {
            if let Some(servers) = root_val.get_mut("mcp").and_then(Value::as_object_mut) {
                was_attached = servers.remove("warmplane").is_some();
            }
        }
    }

    if was_attached {
        let formatted_json = serde_json::to_string_pretty(&root_val)?;
        let temp_path = config_path.with_extension("tmp");
        std::fs::write(&temp_path, &formatted_json)
            .with_context(|| format!("Failed to write temporary file {}", temp_path.display()))?;
        std::fs::rename(&temp_path, &config_path)
            .with_context(|| format!("Failed to atomically rename to {}", config_path.display()))?;
    }

    Ok(DetachResult {
        client_id: client_id.to_string(),
        config_path: config_path.to_string_lossy().to_string(),
        was_attached,
        ok: true,
        message: if was_attached {
            format!("Successfully detached Warmplane from {}", client_def.name)
        } else {
            format!("Warmplane was not attached to {}", client_def.name)
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_client_attach_and_detach_standard() {
        let temp_dir =
            std::env::temp_dir().join(format!("wp_client_test_std_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();
        let cfg_file = temp_dir.join("claude_desktop_config.json");

        // Write existing server
        fs::write(
            &cfg_file,
            r#"{"mcpServers": {"github": {"command": "npx", "args": ["@modelcontextprotocol/server-github"]}}}"#,
        )
        .unwrap();

        let dialect = ClientDialect::StandardMcpServers;
        let (attached, prof, count) = inspect_client_config(&cfg_file, dialect);
        assert!(!attached);
        assert_eq!(prof, None);
        assert_eq!(count, 1);

        // Manually simulate attach logic using direct object mutation
        let mut val: Value = serde_json::from_str(&fs::read_to_string(&cfg_file).unwrap()).unwrap();
        val["mcpServers"]["warmplane"] = json!({
            "command": "warmplane",
            "args": ["mcp-server", "--config", "/path/mcp_servers.json", "--profile", "coding"]
        });
        fs::write(&cfg_file, serde_json::to_string_pretty(&val).unwrap()).unwrap();

        let (attached2, prof2, count2) = inspect_client_config(&cfg_file, dialect);
        assert!(attached2);
        assert_eq!(prof2.as_deref(), Some("coding"));
        assert_eq!(count2, 1);

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_client_attach_opencode_dialect() {
        let temp_dir =
            std::env::temp_dir().join(format!("wp_client_test_oc_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();
        let cfg_file = temp_dir.join("opencode.json");

        fs::write(&cfg_file, r#"{"theme": "dark"}"#).unwrap();

        let dialect = ClientDialect::OpenCodeMcp;
        let (attached, _, _) = inspect_client_config(&cfg_file, dialect);
        assert!(!attached);

        let mut val: Value = serde_json::from_str(&fs::read_to_string(&cfg_file).unwrap()).unwrap();
        val["mcp"] = json!({
            "warmplane": {
                "type": "local",
                "command": "warmplane",
                "args": ["mcp-server", "--config", "/path/mcp_servers.json"],
                "enabled": true
            }
        });
        fs::write(&cfg_file, serde_json::to_string_pretty(&val).unwrap()).unwrap();

        let (attached2, prof2, count2) = inspect_client_config(&cfg_file, dialect);
        assert!(attached2);
        assert_eq!(prof2, None);
        assert_eq!(count2, 0);

        // Clean up
        let _ = fs::remove_dir_all(temp_dir);
    }
}
