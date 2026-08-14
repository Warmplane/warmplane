// Rust guideline compliant 2026-08-13

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, io::ErrorKind};

/// Default TCP listening port for Warmplane daemon HTTP API.
pub const DEFAULT_PORT: u16 = 9090;
/// Default configuration file path.
pub const DEFAULT_CONFIG_PATH: &str = "mcp_servers.json";
/// Default timeout in milliseconds for tool call execution.
pub const DEFAULT_TOOL_TIMEOUT_MS: u64 = 15_000;

/// Root configuration container for Warmplane MCP proxy.
#[derive(Deserialize, Clone)]
pub struct McpConfig {
    /// Optional HTTP port override.
    #[serde(default)]
    pub port: Option<u16>,
    /// Tool execution timeout override in milliseconds.
    #[serde(default, rename = "toolTimeoutMs")]
    pub tool_timeout_ms: Option<u64>,
    /// Capability alias mapping (alias -> canonical capability ID).
    #[serde(default, rename = "capabilityAliases")]
    pub capability_aliases: HashMap<String, String>,
    /// Resource alias mapping (alias -> canonical resource URI).
    #[serde(default, rename = "resourceAliases")]
    pub resource_aliases: HashMap<String, String>,
    /// Prompt alias mapping (alias -> canonical prompt name).
    #[serde(default, rename = "promptAliases")]
    pub prompt_aliases: HashMap<String, String>,
    /// Optional security policy configuration.
    #[serde(default)]
    pub policy: Option<PolicyConfig>,
    /// Upstream MCP server definitions keyed by server identifier.
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

/// Upstream server configuration definition.
#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    /// Executable command for stdio-based servers.
    #[serde(default)]
    pub command: Option<String>,
    /// Command line arguments for stdio-based servers.
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables to pass to stdio server process.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// URL endpoint for HTTP/SSE-based servers.
    #[serde(default)]
    pub url: Option<String>,
    /// MCP protocol version preference.
    #[serde(default, rename = "protocolVersion")]
    pub protocol_version: Option<String>,
    /// Whether stateless HTTP calls are allowed for this server.
    #[serde(default, rename = "allowStateless")]
    pub allow_stateless: Option<bool>,
    /// Additional static HTTP headers.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Authentication settings for upstream server.
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

/// Upstream authentication configuration types.
#[derive(Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    /// Static or environment-based Bearer token authentication.
    Bearer {
        #[serde(default)]
        token: Option<String>,
        #[serde(default, rename = "tokenEnv")]
        token_env: Option<String>,
    },
    /// HTTP Basic authentication.
    Basic {
        username: String,
        #[serde(default)]
        password: Option<String>,
        #[serde(default, rename = "passwordEnv")]
        password_env: Option<String>,
    },
    /// OAuth2 authorization code flow with PKCE.
    Oauth2 {
        #[serde(rename = "clientId")]
        client_id: String,
        #[serde(rename = "authorizationServerUrl")]
        authorization_server_url: String,
        #[serde(default)]
        scopes: Vec<String>,
        #[serde(default, rename = "clientMetadataUrl")]
        client_metadata_url: Option<String>,
    },
}

/// Security access control policy configuration.
#[derive(Deserialize, Clone, Default)]
pub struct PolicyConfig {
    /// List of capability ID patterns allowed for execution.
    #[serde(default)]
    pub allow: Vec<String>,
    /// List of capability ID patterns explicitly denied.
    #[serde(default)]
    pub deny: Vec<String>,
    /// Sensitive key patterns to redact in logged request/response payloads.
    #[serde(default, rename = "redactKeys")]
    pub redact_keys: Vec<String>,
}

/// Loads and validates Warmplane server configuration from JSON file.
///
/// # Arguments
/// * `config_path` - Path to the JSON configuration file.
///
/// # Returns
/// Parsed `McpConfig` instance.
///
/// # Errors
/// Returns an error if reading file fails or JSON schema is invalid.
pub fn load_config(config_path: impl AsRef<str>) -> Result<McpConfig> {
    let path_ref = config_path.as_ref();
    let config_str = fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read config file: {}", path_ref))?;
    let config: McpConfig =
        serde_json::from_str(&config_str).context("Failed to parse config JSON")?;
    validate_config(&config)?;
    Ok(config)
}

/// Resolves client server listening port from CLI override or config file.
///
/// # Arguments
/// * `port_override` - Optional port specified via CLI flag.
/// * `config_path` - Path to configuration file.
///
/// # Returns
/// Resolved `u16` port number.
///
/// # Errors
/// Returns an error if config file exists but contains invalid JSON.
pub fn resolve_client_port(
    port_override: Option<u16>,
    config_path: impl AsRef<str>,
) -> Result<u16> {
    if let Some(port) = port_override {
        return Ok(port);
    }

    let path_ref = config_path.as_ref();
    match fs::read_to_string(path_ref) {
        Ok(config_str) => {
            let config: McpConfig =
                serde_json::from_str(&config_str).context("Failed to parse config JSON")?;
            Ok(config.port.unwrap_or(DEFAULT_PORT))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(DEFAULT_PORT),
        Err(err) => Err(err).with_context(|| format!("Failed to read config file: {}", path_ref)),
    }
}

fn validate_config(config: &McpConfig) -> Result<()> {
    for (server_id, server) in &config.mcp_servers {
        let has_command = server
            .command
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let has_url = server
            .url
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);

        match (has_command, has_url) {
            (true, true) => {
                anyhow::bail!(
                    "Server '{}' is ambiguous: configure exactly one of 'command' or 'url'",
                    server_id
                );
            }
            (false, false) => {
                anyhow::bail!(
                    "Server '{}' is invalid: configure exactly one of 'command' or 'url'",
                    server_id
                );
            }
            _ => {}
        }

        if has_command {
            if server.auth.is_some() {
                anyhow::bail!(
                    "Server '{}' uses stdio ('command') and cannot define 'auth'",
                    server_id
                );
            }
            if !server.headers.is_empty() {
                anyhow::bail!(
                    "Server '{}' uses stdio ('command') and cannot define HTTP 'headers'",
                    server_id
                );
            }
            if server.protocol_version.is_some() {
                anyhow::bail!(
                    "Server '{}' uses stdio ('command') and cannot define 'protocolVersion'",
                    server_id
                );
            }
            if server.allow_stateless.is_some() {
                anyhow::bail!(
                    "Server '{}' uses stdio ('command') and cannot define 'allowStateless'",
                    server_id
                );
            }
        }

        if let Some(auth) = &server.auth {
            match auth {
                AuthConfig::Bearer { token, token_env } => {
                    let has_token = token.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
                    let has_token_env = token_env
                        .as_ref()
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false);
                    if has_token == has_token_env {
                        anyhow::bail!(
                            "Server '{}' bearer auth requires exactly one of 'token' or 'tokenEnv'",
                            server_id
                        );
                    }
                }
                AuthConfig::Basic {
                    username,
                    password,
                    password_env,
                } => {
                    if username.trim().is_empty() {
                        anyhow::bail!(
                            "Server '{}' basic auth requires non-empty 'username'",
                            server_id
                        );
                    }
                    let has_password = password.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
                    let has_password_env = password_env
                        .as_ref()
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false);
                    if has_password == has_password_env {
                        anyhow::bail!(
                            "Server '{}' basic auth requires exactly one of 'password' or 'passwordEnv'",
                            server_id
                        );
                    }
                }
                AuthConfig::Oauth2 {
                    client_id,
                    authorization_server_url,
                    scopes: _,
                    client_metadata_url: _,
                } => {
                    if client_id.trim().is_empty() {
                        anyhow::bail!(
                            "Server '{}' oauth2 auth requires non-empty 'clientId'",
                            server_id
                        );
                    }
                    if authorization_server_url.trim().is_empty() {
                        anyhow::bail!(
                            "Server '{}' oauth2 auth requires non-empty 'authorizationServerUrl'",
                            server_id
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_config, AuthConfig, McpConfig, ServerConfig};
    use std::collections::HashMap;

    fn empty_server() -> ServerConfig {
        ServerConfig {
            command: None,
            args: vec![],
            env: HashMap::new(),
            url: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            auth: None,
        }
    }

    fn config_with_server(server: ServerConfig) -> McpConfig {
        let mut mcp_servers = HashMap::new();
        mcp_servers.insert("s1".to_string(), server);
        McpConfig {
            port: None,
            tool_timeout_ms: None,
            capability_aliases: HashMap::new(),
            resource_aliases: HashMap::new(),
            prompt_aliases: HashMap::new(),
            policy: None,
            mcp_servers,
        }
    }

    #[test]
    fn server_requires_exactly_one_transport_selector() {
        let server = empty_server();
        let err = validate_config(&config_with_server(server)).unwrap_err();
        assert!(err
            .to_string()
            .contains("configure exactly one of 'command' or 'url'"));
    }

    #[test]
    fn server_rejects_both_transport_selectors() {
        let mut server = empty_server();
        server.command = Some("node".to_string());
        server.url = Some("https://example.com/mcp".to_string());
        let err = validate_config(&config_with_server(server)).unwrap_err();
        assert!(err.to_string().contains("is ambiguous"));
    }

    #[test]
    fn stdio_server_rejects_http_only_fields() {
        let mut server = empty_server();
        server.command = Some("node".to_string());
        server.headers.insert("X-Test".to_string(), "1".to_string());
        let err = validate_config(&config_with_server(server)).unwrap_err();
        assert!(err.to_string().contains("cannot define HTTP 'headers'"));
    }

    #[test]
    fn bearer_auth_requires_one_credential_source() {
        let mut server = empty_server();
        server.url = Some("https://example.com/mcp".to_string());
        server.auth = Some(AuthConfig::Bearer {
            token: None,
            token_env: None,
        });
        let err = validate_config(&config_with_server(server)).unwrap_err();
        assert!(err
            .to_string()
            .contains("requires exactly one of 'token' or 'tokenEnv'"));
    }

    #[test]
    fn basic_auth_requires_one_password_source() {
        let mut server = empty_server();
        server.url = Some("https://example.com/mcp".to_string());
        server.auth = Some(AuthConfig::Basic {
            username: "alice".to_string(),
            password: Some("pw".to_string()),
            password_env: Some("PW_ENV".to_string()),
        });
        let err = validate_config(&config_with_server(server)).unwrap_err();
        assert!(err
            .to_string()
            .contains("requires exactly one of 'password' or 'passwordEnv'"));
    }

    #[test]
    fn valid_http_server_passes_validation() {
        let mut server = empty_server();
        server.url = Some("https://example.com/mcp".to_string());
        server.protocol_version = Some("2026-07-28".to_string());
        server.auth = Some(AuthConfig::Bearer {
            token: None,
            token_env: Some("MCP_TOKEN".to_string()),
        });
        assert!(validate_config(&config_with_server(server)).is_ok());
    }

    #[test]
    fn oauth2_auth_requires_non_empty_fields() {
        let mut server = empty_server();
        server.url = Some("https://example.com/mcp".to_string());
        server.auth = Some(AuthConfig::Oauth2 {
            client_id: "".to_string(),
            authorization_server_url: "https://auth.example.com".to_string(),
            scopes: vec![],
            client_metadata_url: None,
        });
        let err = validate_config(&config_with_server(server.clone())).unwrap_err();
        assert!(err.to_string().contains("requires non-empty 'clientId'"));

        server.auth = Some(AuthConfig::Oauth2 {
            client_id: "client1".to_string(),
            authorization_server_url: "  ".to_string(),
            scopes: vec![],
            client_metadata_url: None,
        });
        let err = validate_config(&config_with_server(server.clone())).unwrap_err();
        assert!(err
            .to_string()
            .contains("requires non-empty 'authorizationServerUrl'"));

        server.auth = Some(AuthConfig::Oauth2 {
            client_id: "client1".to_string(),
            authorization_server_url: "https://auth.example.com".to_string(),
            scopes: vec!["read".to_string()],
            client_metadata_url: Some("https://example.com/metadata.json".to_string()),
        });
        assert!(validate_config(&config_with_server(server)).is_ok());
    }
}
