use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::ErrorKind, path::PathBuf};

/// Default TCP listening port for Warmplane daemon HTTP API.
pub const DEFAULT_PORT: u16 = 9090;
/// Default configuration file path.
pub const DEFAULT_CONFIG_PATH: &str = "mcp_servers.json";
/// Default timeout in milliseconds for tool call execution.
pub const DEFAULT_TOOL_TIMEOUT_MS: u64 = 15_000;

/// Root configuration container for Warmplane MCP proxy.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct McpConfig {
    /// Optional HTTP port override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Tool execution timeout override in milliseconds.
    #[serde(
        default,
        rename = "toolTimeoutMs",
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_timeout_ms: Option<u64>,
    /// Capability alias mapping (alias -> canonical capability ID).
    #[serde(
        default,
        rename = "capabilityAliases",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub capability_aliases: HashMap<String, String>,
    /// Resource alias mapping (alias -> canonical resource URI).
    #[serde(
        default,
        rename = "resourceAliases",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub resource_aliases: HashMap<String, String>,
    /// Prompt alias mapping (alias -> canonical prompt name).
    #[serde(
        default,
        rename = "promptAliases",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub prompt_aliases: HashMap<String, String>,
    /// Optional security policy configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyConfig>,
    /// Optional WORM audit trail configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,
    /// Optional global resilience and circuit breaker configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resilience: Option<crate::circuit_breaker::ResilienceConfig>,
    /// Optional persistent runtime state configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateConfig>,
    /// Optional static API auth token protecting mutating/admin control-plane endpoints.
    #[serde(
        default,
        rename = "authToken",
        alias = "auth_token",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_token: Option<String>,
    /// Upstream MCP server definitions keyed by server identifier.
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
}

/// Persistent runtime state configuration.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct StateConfig {
    /// Whether persistent state is enabled (default: true).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Directory path for storing persistent state files (default: ".warmplane/state").
    #[serde(default = "default_state_dir", skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

fn default_state_dir() -> Option<String> {
    Some(".warmplane/state".to_string())
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dir: default_state_dir(),
        }
    }
}

/// Upstream server configuration definition.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct ServerConfig {
    /// Executable command for stdio-based servers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Command line arguments for stdio-based servers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables to pass to stdio server process.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// URL endpoint for HTTP/SSE-based servers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// MCP protocol version preference.
    #[serde(
        default,
        rename = "protocolVersion",
        skip_serializing_if = "Option::is_none"
    )]
    pub protocol_version: Option<String>,
    /// Whether stateless HTTP calls are allowed for this server.
    #[serde(
        default,
        rename = "allowStateless",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_stateless: Option<bool>,
    /// Additional static HTTP headers.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// Authentication settings for upstream server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    /// Optional per-server resilience and circuit breaker overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resilience: Option<crate::circuit_breaker::ResilienceConfig>,
}

/// Upstream authentication configuration types.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    /// Static or environment-based Bearer token authentication.
    Bearer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        #[serde(default, rename = "tokenEnv", skip_serializing_if = "Option::is_none")]
        token_env: Option<String>,
    },
    /// HTTP Basic authentication.
    Basic {
        username: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        #[serde(
            default,
            rename = "passwordEnv",
            skip_serializing_if = "Option::is_none"
        )]
        password_env: Option<String>,
    },
    /// OAuth2 authorization code flow with PKCE.
    Oauth2 {
        #[serde(rename = "clientId")]
        client_id: String,
        #[serde(rename = "authorizationServerUrl")]
        authorization_server_url: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        scopes: Vec<String>,
        #[serde(
            default,
            rename = "clientMetadataUrl",
            skip_serializing_if = "Option::is_none"
        )]
        client_metadata_url: Option<String>,
    },
}

/// Webhook configuration for external notifications and HITL workflows.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct WebhookConfig {
    /// Target webhook URL.
    pub url: String,
    /// Static HMAC-SHA256 signing secret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Environment variable containing HMAC-SHA256 signing secret.
    #[serde(
        default,
        rename = "secretEnv",
        alias = "secret_env",
        skip_serializing_if = "Option::is_none"
    )]
    pub secret_env: Option<String>,
    /// Authorization header value (e.g. Bearer token).
    #[serde(
        default,
        rename = "authHeader",
        alias = "auth_header",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_header: Option<String>,
    /// Optional custom HTTP headers map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

impl WebhookConfig {
    /// Resolves the HMAC signing secret from env var or direct value.
    pub fn resolve_secret(&self) -> Option<String> {
        if let Some(ref env_name) = self.secret_env {
            if let Ok(val) = std::env::var(env_name) {
                if !val.trim().is_empty() {
                    return Some(val);
                }
            }
        }
        self.secret.clone()
    }
}

/// Security access control and human-in-the-loop policy configuration.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct PolicyConfig {
    /// List of capability ID patterns allowed for execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    /// List of capability ID patterns explicitly denied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
    /// Sensitive key patterns to redact in logged request/response payloads.
    #[serde(
        default,
        rename = "redactKeys",
        alias = "redact_keys",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub redact_keys: Vec<String>,
    /// List of capability ID patterns requiring human operator approval before execution.
    #[serde(
        default,
        rename = "requireApproval",
        alias = "require_approval",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub require_approval: Vec<String>,
    /// Timeout in seconds before a pending approval automatically expires (default 300s).
    #[serde(
        default,
        rename = "approvalTimeoutSecs",
        alias = "approval_timeout_secs",
        skip_serializing_if = "Option::is_none"
    )]
    pub approval_timeout_secs: Option<u64>,
    /// Outbound webhook configuration for dispatching approval events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook: Option<WebhookConfig>,
}

/// WORM audit trail and telemetry logging configuration.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AuditConfig {
    /// Whether audit logging is enabled (defaults to true if block is present).
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Path to the append-only audit log file (e.g. `warmplane_audit.jsonl`).
    #[serde(
        default,
        rename = "filePath",
        alias = "file_path",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_path: Option<String>,
    /// Capacity of the asynchronous in-memory bounded queue (default 10,000).
    #[serde(
        default,
        rename = "bufferCapacity",
        alias = "buffer_capacity",
        skip_serializing_if = "Option::is_none"
    )]
    pub buffer_capacity: Option<usize>,
    /// Flush interval in milliseconds for background batching worker (default 250ms).
    #[serde(
        default,
        rename = "flushIntervalMs",
        alias = "flush_interval_ms",
        skip_serializing_if = "Option::is_none"
    )]
    pub flush_interval_ms: Option<u64>,
    /// Maximum batch size before flushing immediately (default 100).
    #[serde(
        default,
        rename = "maxBatchSize",
        alias = "max_batch_size",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_batch_size: Option<usize>,
    /// Optional HMAC secret key used to compute keyed integrity digests over audit events.
    #[serde(
        default,
        rename = "hmacKey",
        alias = "hmac_key",
        skip_serializing_if = "Option::is_none"
    )]
    pub hmac_key: Option<String>,
    /// Environment variable containing HMAC secret key.
    #[serde(
        default,
        rename = "hmacKeyEnv",
        alias = "hmac_key_env",
        skip_serializing_if = "Option::is_none"
    )]
    pub hmac_key_env: Option<String>,
    /// Optional external SIEM export target configurations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub siem: Option<SiemConfig>,
}

impl AuditConfig {
    /// Resolves the HMAC secret key from env var or direct value.
    pub fn resolve_hmac_key(&self) -> Option<String> {
        if let Some(ref env_name) = self.hmac_key_env {
            if let Ok(val) = std::env::var(env_name) {
                if !val.trim().is_empty() {
                    return Some(val);
                }
            }
        }
        self.hmac_key.clone()
    }
}

/// SIEM telemetry forwarder configuration.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct SiemConfig {
    /// List of telemetry export destinations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<SiemTargetConfig>,
}

/// Target destination configuration for SIEM telemetry export.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SiemTargetConfig {
    /// Generic Webhook / Datadog JSON ingestion endpoint.
    Webhook {
        url: String,
        #[serde(
            default,
            rename = "authHeader",
            alias = "auth_header",
            skip_serializing_if = "Option::is_none"
        )]
        auth_header: Option<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,
    },
    /// Splunk HTTP Event Collector (HEC).
    SplunkHec {
        url: String,
        token: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
}

fn default_true() -> bool {
    true
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file_path: None,
            buffer_capacity: Some(10_000),
            flush_interval_ms: Some(250),
            max_batch_size: Some(100),
            hmac_key: None,
            hmac_key_env: None,
            siem: None,
        }
    }
}

impl McpConfig {
    /// Sanitizes all confidential credentials and secrets in place for safe API exposure.
    pub fn sanitize_secrets(&mut self) {
        if self.auth_token.is_some() {
            self.auth_token = Some("********".to_string());
        }
        if let Some(ref mut policy) = self.policy {
            policy.sanitize_secrets();
        }
        if let Some(ref mut audit) = self.audit {
            audit.sanitize_secrets();
        }
        for server in self.mcp_servers.values_mut() {
            server.sanitize_secrets();
        }
    }
}

impl PolicyConfig {
    /// Sanitizes webhook secrets in place.
    pub fn sanitize_secrets(&mut self) {
        if let Some(ref mut webhook) = self.webhook {
            webhook.sanitize_secrets();
        }
    }
}

impl WebhookConfig {
    /// Sanitizes secrets and auth headers.
    pub fn sanitize_secrets(&mut self) {
        if self.secret.is_some() {
            self.secret = Some("********".to_string());
        }
        if self.auth_header.is_some() {
            self.auth_header = Some("********".to_string());
        }
    }
}

impl AuditConfig {
    /// Sanitizes SIEM target secrets and HMAC keys in place.
    pub fn sanitize_secrets(&mut self) {
        if self.hmac_key.is_some() {
            self.hmac_key = Some("********".to_string());
        }
        if let Some(ref mut siem) = self.siem {
            siem.sanitize_secrets();
        }
    }
}

impl SiemConfig {
    /// Sanitizes target secrets in place.
    pub fn sanitize_secrets(&mut self) {
        for target in &mut self.targets {
            match target {
                SiemTargetConfig::Webhook { auth_header, .. } => {
                    if auth_header.is_some() {
                        *auth_header = Some("********".to_string());
                    }
                }
                SiemTargetConfig::SplunkHec { token, .. } => {
                    *token = "********".to_string();
                }
            }
        }
    }
}

impl ServerConfig {
    /// Sanitizes authentication tokens/passwords and sensitive headers in place.
    pub fn sanitize_secrets(&mut self) {
        if let Some(ref mut auth) = self.auth {
            auth.sanitize_secrets();
        }
        for (k, v) in self.headers.iter_mut() {
            let k_lower = k.to_lowercase();
            if k_lower.contains("auth")
                || k_lower.contains("key")
                || k_lower.contains("secret")
                || k_lower.contains("token")
            {
                *v = "********".to_string();
            }
        }
    }
}

impl AuthConfig {
    /// Sanitizes credentials in place.
    pub fn sanitize_secrets(&mut self) {
        match self {
            AuthConfig::Bearer { token, .. } => {
                if token.is_some() {
                    *token = Some("********".to_string());
                }
            }
            AuthConfig::Basic { password, .. } => {
                if password.is_some() {
                    *password = Some("********".to_string());
                }
            }
            AuthConfig::Oauth2 { .. } => {}
        }
    }
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

/// Loads config if present, or returns an empty default config.
///
/// # Arguments
/// * `config_path` - Path to the JSON configuration file.
///
/// # Returns
/// Existing or newly initialized `McpConfig`.
///
/// # Errors
/// Returns an error if file exists but contains invalid JSON.
pub fn load_or_default_config(config_path: impl AsRef<str>) -> Result<McpConfig> {
    let path_ref = config_path.as_ref();
    match fs::read_to_string(path_ref) {
        Ok(config_str) => {
            let config: McpConfig =
                serde_json::from_str(&config_str).context("Failed to parse config JSON")?;
            validate_config(&config)?;
            Ok(config)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(McpConfig::default()),
        Err(err) => Err(err).with_context(|| format!("Failed to read config file: {}", path_ref)),
    }
}

/// Validates and atomically saves configuration to disk.
///
/// # Arguments
/// * `config_path` - Path to the JSON configuration file.
/// * `config` - `McpConfig` instance to serialize and save.
///
/// # Errors
/// Returns an error if validation fails or file write/rename fails.
pub fn save_config(config_path: impl AsRef<str>, config: &McpConfig) -> Result<()> {
    validate_config(config)?;
    let target_path = PathBuf::from(config_path.as_ref());
    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    let json_bytes = serde_json::to_string_pretty(config)
        .context("Failed to serialize configuration to JSON")?;
    let tmp_path = target_path.with_extension(format!("tmp.{}", std::process::id()));

    fs::write(&tmp_path, format!("{}\n", json_bytes)).with_context(|| {
        format!(
            "Failed to write temporary config file: {}",
            tmp_path.display()
        )
    })?;

    fs::rename(&tmp_path, &target_path).with_context(|| {
        format!(
            "Failed to replace target config file {} with {}",
            target_path.display(),
            tmp_path.display()
        )
    })?;

    Ok(())
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
            resilience: None,
        }
    }

    fn config_with_server(server: ServerConfig) -> McpConfig {
        let mut mcp_servers = HashMap::new();
        mcp_servers.insert("s1".to_string(), server);
        McpConfig {
            mcp_servers,
            ..Default::default()
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

    #[test]
    fn save_and_load_config_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("warmplane_test_{}", std::process::id()));
        let config_file = temp_dir.join("test_servers.json");

        let mut server = empty_server();
        server.command = Some("node".to_string());
        server.args = vec!["server.js".to_string()];
        let mut servers = HashMap::new();
        servers.insert("node_server".to_string(), server);
        let config = McpConfig {
            port: Some(9999),
            mcp_servers: servers,
            ..Default::default()
        };

        super::save_config(config_file.to_str().unwrap(), &config).unwrap();
        let loaded = super::load_config(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config, loaded);

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
