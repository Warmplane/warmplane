// Rust guideline compliant 2026-11-16
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::ErrorKind, path::PathBuf};

/// Default TCP listening port for Warmplane daemon HTTP API.
pub const DEFAULT_PORT: u16 = 9090;
/// Default TCP listening port for the Warmplane HTTP/SSE MCP server facade.
pub const DEFAULT_MCP_HTTP_PORT: u16 = 9191;
/// Default configuration file path.
pub const DEFAULT_CONFIG_PATH: &str = "mcp_servers.json";
/// Default timeout in milliseconds for tool call execution.
pub const DEFAULT_TOOL_TIMEOUT_MS: u64 = 15_000;

/// Placeholder substituted for a redacted static RBAC bearer token.
///
/// A per-token suffix is appended so an operator still sees how many exist.
const REDACTED_TOKEN: &str = "********";

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
    /// Capability alias mapping (alias -> canonical capability ID or detailed config).
    #[serde(
        default,
        rename = "capabilityAliases",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub capability_aliases: HashMap<String, AliasTarget>,
    /// Resource alias mapping (alias -> canonical resource URI or detailed config).
    #[serde(
        default,
        rename = "resourceAliases",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub resource_aliases: HashMap<String, AliasTarget>,
    /// Prompt alias mapping (alias -> canonical prompt name or detailed config).
    #[serde(
        default,
        rename = "promptAliases",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub prompt_aliases: HashMap<String, AliasTarget>,
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
    /// Optional Multi-Tenant Role-Based Access Control (RBAC) configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rbac: Option<crate::rbac::RbacConfig>,
    /// Named server constellations (profiles) for task-specific catalog partitioning.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub profiles: HashMap<String, ProfileConfig>,
    /// Upstream MCP server definitions keyed by server identifier.
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, ServerConfig>,
    /// Optional HTTP/SSE MCP server facade exposure configuration.
    #[serde(
        default,
        rename = "mcpHttpServer",
        skip_serializing_if = "Option::is_none"
    )]
    pub mcp_http_server: Option<McpHttpServerConfig>,
}

/// Configuration for exposing warmplane itself as a Streamable HTTP/SSE MCP server.
///
/// When present, `warmplane mcp-http-server` (or the daemon with this config block) binds a
/// Streamable-HTTP MCP endpoint that remote MCP clients can connect to directly.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct McpHttpServerConfig {
    /// TCP port to listen on (default: 9191).
    #[serde(default = "default_mcp_http_port")]
    pub port: u16,
    /// Bind address (default: `"127.0.0.1"`). Set to `"0.0.0.0"` for network access;
    /// requires `authToken` or `rbac` to be configured.
    #[serde(default = "default_mcp_http_bind")]
    pub bind: String,
    /// SSE keep-alive ping interval in milliseconds (default: 15 000).
    /// `None` disables keep-alive pings.
    #[serde(
        default = "default_sse_keep_alive_ms",
        rename = "sseKeepAliveMs",
        skip_serializing_if = "Option::is_none"
    )]
    pub sse_keep_alive_ms: Option<u64>,
    /// Prefer `application/json` responses for simple request/response exchanges (default: true).
    /// Falls back to SSE automatically when streaming is required.
    #[serde(default = "default_true", rename = "jsonResponse")]
    pub json_response: bool,
    /// Optional profile name restricting the exposed capability surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Additional hostnames (or `host:port` pairs) accepted in the `Host` header.
    /// Loopback addresses are always accepted; add your public hostname here for network deployments.
    #[serde(
        default,
        rename = "allowedHosts",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub allowed_hosts: Vec<String>,
    /// Browser origins allowed via the `Origin` header (CORS). Empty list disables origin checking.
    #[serde(
        default,
        rename = "allowedOrigins",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub allowed_origins: Vec<String>,
    /// Protocol versions advertised by the facade server. Defaults to session-stable versions up to 2025-11-25.
    #[serde(
        default = "default_mcp_http_protocol_versions",
        rename = "supportedProtocolVersions",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub supported_protocol_versions: Vec<String>,
}

pub(crate) fn default_mcp_http_protocol_versions() -> Vec<String> {
    vec![
        "2024-11-05".to_string(),
        "2025-03-26".to_string(),
        "2025-06-18".to_string(),
        "2025-11-25".to_string(),
    ]
}

fn default_mcp_http_port() -> u16 {
    crate::config::DEFAULT_MCP_HTTP_PORT
}

fn default_mcp_http_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_sse_keep_alive_ms() -> Option<u64> {
    Some(15_000)
}

impl Default for McpHttpServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_MCP_HTTP_PORT,
            bind: "127.0.0.1".to_string(),
            sse_keep_alive_ms: Some(15_000),
            json_response: true,
            profile: None,
            allowed_hosts: Vec::new(),
            allowed_origins: Vec::new(),
            supported_protocol_versions: default_mcp_http_protocol_versions(),
        }
    }
}

/// Target mapping configuration for an alias.
///
/// Supports either a simple canonical target string (e.g. `"semble-rs.search"`)
/// or a detailed configuration object with custom docstrings and summaries.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum AliasTarget {
    /// Simple canonical target name (e.g. `"semble-rs.search"`).
    Simple(String),
    /// Detailed configuration with custom summary and description overrides.
    Detailed {
        /// Canonical target identifier.
        target: String,
        /// Optional custom short summary override.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        /// Optional custom detailed description override.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Expose this alias directly as a top-level native tool in tools/list
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        passthrough: bool,
        /// Optional execution reversibility contract.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reversibility: Option<ReversibilityConfig>,
    },
}

/// Execution reversibility behavior configured for an alias.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ReversibilityConfig {
    /// Reversibility class supported by this configuration.
    #[serde(rename = "class")]
    pub class: ReversibilityClass,
    /// Cancellation hold duration in seconds for delayable emissions.
    #[serde(
        default,
        rename = "holdWindowSecs",
        skip_serializing_if = "Option::is_none"
    )]
    pub hold_window_secs: Option<u64>,
    /// Maximum lifetime of a registered inverse action, in seconds.
    #[serde(default, rename = "ttlSecs", skip_serializing_if = "Option::is_none")]
    pub ttl_secs: Option<u64>,
    /// Maximum calls for a compensable capability in a rolling day.
    #[serde(
        default,
        rename = "dailyLimit",
        skip_serializing_if = "Option::is_none"
    )]
    pub daily_limit: Option<u64>,
    /// Read operation used to capture the pre-mutation state.
    #[serde(
        default,
        rename = "preSnapshot",
        skip_serializing_if = "Option::is_none"
    )]
    pub pre_snapshot: Option<Box<PreSnapshotConfig>>,
    /// Read operation used to verify the post-execution state before applying an inverse.
    #[serde(
        default,
        rename = "preUndoCheck",
        skip_serializing_if = "Option::is_none"
    )]
    pub pre_undo_check: Option<Box<PreSnapshotConfig>>,
    /// Explicit non-production target and arguments used by the undo contract test command.
    #[serde(
        default,
        rename = "testTarget",
        skip_serializing_if = "Option::is_none"
    )]
    pub test_target: Option<Box<UndoTestTargetConfig>>,
    /// Inverse operation to register after a successful forward call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse: Option<Box<InverseConfig>>,
}

/// Upstream target reserved for an observational undo contract test.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct UndoTestTargetConfig {
    /// Dedicated test server identifier that exposes the contract's tool names.
    pub server: String,
    /// Fixed, non-production arguments used for the test cycle.
    #[serde(default)]
    pub args: serde_json::Value,
}

/// Configuration for reading and distilling a pre-mutation snapshot.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PreSnapshotConfig {
    /// Capability used to read the current state.
    pub capability: String,
    /// Arguments passed to the read capability.
    #[serde(default)]
    pub args: serde_json::Value,
    /// Optional JSONPath selecting the snapshot value to persist.
    #[serde(
        default,
        rename = "captureJsonPath",
        skip_serializing_if = "Option::is_none"
    )]
    pub capture_json_path: Option<String>,
}

/// Inverse capability and its argument template.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct InverseConfig {
    /// Capability used to reverse the forward operation.
    pub capability: String,
    /// JSON argument template supporting `${args...}`, `${result...}`, and `${snapshot...}` references.
    #[serde(default)]
    pub args: serde_json::Value,
}

/// Reversibility behavior classes.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClass {
    /// Delay dispatch so an operator can cancel an outbound emission.
    Delayable,
    /// An operation with an inverse that restores captured pre-state.
    Reversible,
    /// An operation whose inverse compensates for its effect.
    Compensable,
    /// An operation with no configured inverse.
    Irreversible,
}

/// Resolves a canonical capability's delayable hold duration from profile policy, alias, and global policy.
pub fn resolve_hold_window_secs(
    aliases: &HashMap<String, AliasTarget>,
    canonical_target: &str,
    profile_window_secs: Option<u64>,
    global_window_secs: Option<u64>,
) -> Option<u64> {
    let contract = aliases
        .values()
        .find(|alias| alias.target() == canonical_target)
        .and_then(AliasTarget::reversibility)?;
    if contract.class != ReversibilityClass::Delayable {
        return None;
    }
    Some(
        profile_window_secs
            .or(contract.hold_window_secs)
            .or(global_window_secs)
            .unwrap_or(30),
    )
}

impl AliasTarget {
    /// Returns the canonical target identifier.
    pub fn target(&self) -> &str {
        match self {
            Self::Simple(t) => t.as_str(),
            Self::Detailed { target, .. } => target.as_str(),
        }
    }

    /// Returns optional custom summary override.
    pub fn summary(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed { summary, .. } => summary.as_deref(),
        }
    }

    /// Returns optional custom description override.
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed { description, .. } => description.as_deref(),
        }
    }

    /// Returns whether this alias is exposed directly as a native passthrough tool.
    pub fn is_passthrough(&self) -> bool {
        match self {
            Self::Simple(_) => false,
            Self::Detailed { passthrough, .. } => *passthrough,
        }
    }

    /// Returns the optional reversibility contract configured on this alias.
    pub fn reversibility(&self) -> Option<&ReversibilityConfig> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed { reversibility, .. } => reversibility.as_ref(),
        }
    }
}

impl From<&str> for AliasTarget {
    fn from(s: &str) -> Self {
        Self::Simple(s.to_string())
    }
}

impl From<String> for AliasTarget {
    fn from(s: String) -> Self {
        Self::Simple(s)
    }
}

/// Profile configuration defining a named constellation of upstream MCP servers.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct ProfileConfig {
    /// Whitelist of upstream server identifiers included in this constellation.
    pub servers: Vec<String>,
    /// Optional human-readable description of this profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional per-profile security and governance policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyConfig>,
    /// Allows an opted-in MCP role to invoke undo for its own work.
    #[serde(default, rename = "allowAgentSelfUndo")]
    pub allow_agent_self_undo: bool,
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
    /// Requires an operator-approved definition for every tool from this server.
    #[serde(
        default,
        rename = "requireToolPins",
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub require_tool_pins: bool,
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
    /// Optional payload format (generic, slack, discord, teams).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<crate::chatops::WebhookFormat>,
    /// Optional event filter for legacy approval notifications and explicitly enabled action events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,
    /// Public callback URL for interactive action buttons.
    #[serde(
        default,
        rename = "callbackUrl",
        alias = "callback_url",
        skip_serializing_if = "Option::is_none"
    )]
    pub callback_url: Option<String>,
    /// RBAC role used for actions initiated by signed ChatOps buttons.
    #[serde(
        default,
        rename = "actionRole",
        skip_serializing_if = "Option::is_none"
    )]
    pub action_role: Option<String>,
    /// Tenant delegated to receive and resolve approval cards.
    #[serde(
        default,
        rename = "approvalTenant",
        skip_serializing_if = "Option::is_none"
    )]
    pub approval_tenant: Option<String>,
    /// Slack app signing secret for authenticating interactive callback requests.
    #[serde(
        default,
        rename = "slackSigningSecret",
        skip_serializing_if = "Option::is_none"
    )]
    pub slack_signing_secret: Option<String>,
    /// Environment variable holding the Slack app signing secret.
    #[serde(
        default,
        rename = "slackSigningSecretEnv",
        skip_serializing_if = "Option::is_none"
    )]
    pub slack_signing_secret_env: Option<String>,
    /// Optional custom HTTP headers map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Optional allowlist of webhook URLs permitted for test dispatches.
    #[serde(
        default,
        rename = "allowedUrls",
        alias = "allowed_urls",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub allowed_urls: Vec<String>,
}

/// Resolves a secret from an environment variable name, falling back to a direct string value.
pub(crate) fn resolve_env_or_direct(
    env_name: Option<&str>,
    direct: Option<&str>,
) -> Option<String> {
    if let Some(name) = env_name {
        if let Ok(val) = std::env::var(name) {
            if !val.trim().is_empty() {
                return Some(val);
            }
        }
    }
    direct.map(ToString::to_string)
}

impl WebhookConfig {
    /// Resolves the HMAC signing secret from env var or direct value.
    pub fn resolve_secret(&self) -> Option<String> {
        resolve_env_or_direct(self.secret_env.as_deref(), self.secret.as_deref())
    }

    pub(crate) fn resolve_slack_signing_secret(&self) -> Option<String> {
        resolve_env_or_direct(
            self.slack_signing_secret_env.as_deref(),
            self.slack_signing_secret.as_deref(),
        )
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
    /// Compiled global policy for sanitizing upstream results before delivery.
    #[serde(
        default,
        rename = "resultSanitization",
        skip_serializing_if = "Option::is_none"
    )]
    pub result_sanitization: Option<crate::result_sanitization::ResultSanitization>,
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
    /// Default hold window in seconds for delayable capabilities.
    #[serde(
        default,
        rename = "holdWindowSecs",
        alias = "hold_window_secs",
        skip_serializing_if = "Option::is_none"
    )]
    pub hold_window_secs: Option<u64>,
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
        resolve_env_or_direct(self.hmac_key_env.as_deref(), self.hmac_key.as_deref())
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

        // The static RBAC token map is keyed by the live bearer secret, since
        // `RbacEngine::authenticate` compares each configured key against the
        // inbound credential. Leaving the keys intact discloses every token
        // in the map, including higher-privilege ones.
        if let Some(ref mut rbac) = self.rbac {
            rbac.sanitize_secrets();
        }

        // A profile can carry its own policy, and that policy can carry its
        // own webhook secret and auth header.
        for profile in self.profiles.values_mut() {
            if let Some(ref mut policy) = profile.policy {
                policy.sanitize_secrets();
            }
        }
    }
}

impl crate::rbac::RbacConfig {
    /// Redacts static bearer tokens while preserving their role assignments.
    ///
    /// Each key is replaced individually so an operator still sees how many
    /// tokens exist and which role each one grants. Collapsing them onto a
    /// single shared key would hide that count and would not survive a second
    /// sanitize pass unchanged.
    pub fn sanitize_secrets(&mut self) {
        if self.tokens.is_empty() {
            return;
        }
        let existing = std::mem::take(&mut self.tokens);
        self.tokens = existing
            .into_iter()
            .enumerate()
            .map(|(index, (token, assignment))| {
                let key = if token == REDACTED_TOKEN {
                    REDACTED_TOKEN.to_string()
                } else {
                    format!("{REDACTED_TOKEN}_{index}")
                };
                (key, assignment)
            })
            .collect();
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
        if self.slack_signing_secret.is_some() {
            self.slack_signing_secret = Some("********".to_owned());
        }
    }
}

impl AuditConfig {
    /// Sanitizes SIEM target secrets and HMAC keys in place.
    pub fn sanitize_secrets(&mut self) {
        if let Some(key) = self.hmac_key.as_mut() {
            key.replace_range(.., "********");
        }
        if let Some(siem) = self.siem.as_mut() {
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
        // Upstream credentials are commonly supplied through the environment.
        // `env` is a process environment block, so every value in it is
        // treated as a secret: there is no reliable name convention to filter
        // on and leaking one would hand the caller a working credential.
        for v in self.env.values_mut() {
            *v = "********".to_string();
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
    validate_result_sanitization(config)?;
    validate_policy_holds_and_webhooks(config)?;
    validate_reversibility_contracts(&config.capability_aliases)?;
    for (server_id, server) in &config.mcp_servers {
        validate_server_transport(server_id, server)?;
        if let Some(auth) = &server.auth {
            validate_server_auth(server_id, auth)?;
        }
    }
    validate_rbac(config.rbac.as_ref())?;
    validate_profile_references(config)?;
    validate_mcp_http_server(config)
}

fn validate_hold_window(value: Option<u64>, context: &str) -> Result<()> {
    if let Some(seconds) = value {
        if seconds == 0 || seconds.checked_mul(1000).is_none() {
            anyhow::bail!(
                "{} must be between 1 and {} seconds",
                context,
                u64::MAX / 1000
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_result_sanitization(config: &McpConfig) -> Result<()> {
    if let Some(rules) = config
        .policy
        .as_ref()
        .and_then(|policy| policy.result_sanitization.as_ref())
    {
        for server in rules.exception_servers() {
            anyhow::ensure!(
                config.mcp_servers.contains_key(server),
                "resultSanitization exception references unknown server '{server}'"
            );
        }
    }
    anyhow::ensure!(
        !config.profiles.values().any(|profile| {
            profile
                .policy
                .as_ref()
                .is_some_and(|policy| policy.result_sanitization.is_some())
        }),
        "resultSanitization is supported only in the global policy"
    );
    Ok(())
}

fn validate_policy_holds_and_webhooks(config: &McpConfig) -> Result<()> {
    if let Some(policy) = &config.policy {
        validate_hold_window(policy.hold_window_secs, "policy.holdWindowSecs")?;
        if let Some(webhook) = &policy.webhook {
            validate_action_webhook(webhook, config.rbac.as_ref())?;
        }
    }
    for (name, profile) in &config.profiles {
        if let Some(policy) = &profile.policy {
            validate_hold_window(
                policy.hold_window_secs,
                &format!("profiles.{}.policy.holdWindowSecs", name),
            )?;
        }
    }
    Ok(())
}

fn validate_action_webhook(
    webhook: &WebhookConfig,
    rbac: Option<&crate::rbac::RbacConfig>,
) -> Result<()> {
    crate::approval_chatops::validate_approval_webhook(webhook, rbac)?;
    anyhow::ensure!(
        !webhook
            .events
            .iter()
            .any(|event| event == "tool_pin.quarantined")
            || webhook.secret.is_some()
            || webhook.secret_env.is_some(),
        "policy.webhook.secret or secretEnv is required when subscribing to tool_pin.quarantined"
    );
    for event in ["undo.action_available", "emission.held"] {
        if webhook.events.iter().any(|configured| configured == event) {
            validate_webhook_subscription(event, webhook, rbac)?;
        }
    }
    Ok(())
}

fn validate_webhook_subscription(
    event: &str,
    webhook: &WebhookConfig,
    rbac: Option<&crate::rbac::RbacConfig>,
) -> Result<()> {
    if webhook.secret.is_none() && webhook.secret_env.is_none() {
        anyhow::bail!("policy.webhook.secret or secretEnv is required when subscribing to {event}");
    }
    if webhook.callback_url.as_deref().is_none_or(str::is_empty) {
        anyhow::bail!("policy.webhook.callbackUrl is required when subscribing to {event}");
    }
    let role = webhook
        .action_role
        .as_deref()
        .filter(|role| !role.trim().is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!("policy.webhook.actionRole is required when subscribing to {event}")
        })?;
    if let Some(rbac) = rbac.filter(|rbac| rbac.enabled) {
        let role_config = rbac.roles.get(role);
        if !role_config.is_some_and(|role| role.undo_permissions.execute) {
            anyhow::bail!("policy.webhook.actionRole must name an RBAC role with undoPermissions.execute for {event}");
        }
    }
    Ok(())
}

fn validate_reversibility_contracts(
    capability_aliases: &HashMap<String, AliasTarget>,
) -> Result<()> {
    for (alias, target) in capability_aliases {
        if let Some(contract) = target.reversibility() {
            validate_contract_limits(alias, contract)?;
            validate_contract_requirements(alias, contract)?;
            validate_contract_verification(alias, contract)?;
            validate_snapshot_and_inverse(alias, contract)?;
        }
    }
    let mut contracts = HashMap::new();
    for (alias, target) in capability_aliases {
        if let Some(contract) = target.reversibility() {
            match contracts.insert(target.target(), contract) {
                Some(existing) if existing != contract => anyhow::bail!(
                    "Capability aliases targeting '{}' define conflicting reversibility contracts (including '{}')",
                    target.target(), alias
                ),
                _ => {}
            }
        }
    }
    Ok(())
}

fn requires_inverse(contract: &ReversibilityConfig) -> bool {
    matches!(
        contract.class,
        ReversibilityClass::Reversible | ReversibilityClass::Compensable
    )
}

fn validate_contract_limits(alias: &str, contract: &ReversibilityConfig) -> Result<()> {
    validate_hold_window(
        contract.hold_window_secs,
        &format!("capabilityAliases.{}.reversibility.holdWindowSecs", alias),
    )?;
    validate_hold_window(
        contract.ttl_secs,
        &format!("capabilityAliases.{}.reversibility.ttlSecs", alias),
    )?;
    if contract.daily_limit == Some(0) {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.dailyLimit must be greater than zero",
            alias
        );
    }
    if contract.hold_window_secs.is_some() && contract.class != ReversibilityClass::Delayable {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.holdWindowSecs is only valid for delayable class",
            alias
        );
    }
    if contract.daily_limit.is_some() && contract.class != ReversibilityClass::Compensable {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.dailyLimit is only valid for compensable class",
            alias
        );
    }
    if contract.ttl_secs.is_some() && !requires_inverse(contract) {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.ttlSecs is only valid for reversible and compensable classes",
            alias
        );
    }
    Ok(())
}

fn validate_contract_requirements(alias: &str, contract: &ReversibilityConfig) -> Result<()> {
    let requires_inverse = requires_inverse(contract);
    match (requires_inverse, contract.inverse.is_some()) {
        (true, false) => anyhow::bail!(
            "capabilityAliases.{}.reversibility.inverse is required for reversible and compensable classes",
            alias
        ),
        (false, true) => anyhow::bail!(
            "capabilityAliases.{}.reversibility.inverse is only valid for reversible and compensable classes",
            alias
        ),
        _ => {}
    }
    if contract.class == ReversibilityClass::Compensable && contract.daily_limit.is_none() {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.dailyLimit is required for compensable class",
            alias
        );
    }
    if contract.pre_snapshot.is_some() && !requires_inverse {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.preSnapshot is only valid for reversible and compensable classes",
            alias
        );
    }
    if contract.pre_undo_check.is_some() && !requires_inverse {
        anyhow::bail!(
            "capabilityAliases.{}.reversibility.preUndoCheck is only valid for reversible and compensable classes",
            alias
        );
    }
    Ok(())
}

fn validate_contract_verification(alias: &str, contract: &ReversibilityConfig) -> Result<()> {
    if let Some(check) = &contract.pre_undo_check {
        if check.capability.trim().is_empty() || !check.args.is_object() {
            anyhow::bail!("capabilityAliases.{}.reversibility.preUndoCheck requires a capability and object args", alias);
        }
    }
    if contract.test_target.is_some() && !requires_inverse(contract) {
        anyhow::bail!("capabilityAliases.{}.reversibility.testTarget is only valid for reversible and compensable classes", alias);
    }
    if let Some(test_target) = &contract.test_target {
        if test_target.server.trim().is_empty() || !test_target.args.is_object() {
            anyhow::bail!(
                "capabilityAliases.{}.reversibility.testTarget requires a server and object args",
                alias
            );
        }
        if contract.pre_snapshot.is_none() {
            anyhow::bail!("capabilityAliases.{}.reversibility.testTarget requires preSnapshot to verify observational equivalence", alias);
        }
    }
    Ok(())
}

fn validate_snapshot_and_inverse(alias: &str, contract: &ReversibilityConfig) -> Result<()> {
    if let Some(snapshot) = &contract.pre_snapshot {
        if snapshot.capability.trim().is_empty() || !snapshot.args.is_object() {
            anyhow::bail!("capabilityAliases.{}.reversibility.preSnapshot requires a capability and object args", alias);
        }
    }
    if let Some(inverse) = &contract.inverse {
        if inverse.capability.trim().is_empty() || !inverse.args.is_object() {
            anyhow::bail!(
                "capabilityAliases.{}.reversibility.inverse requires a capability and object args",
                alias
            );
        }
    }
    Ok(())
}

fn validate_server_transport(server_id: &str, server: &ServerConfig) -> Result<()> {
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
    Ok(())
}

fn validate_server_auth(server_id: &str, auth: &AuthConfig) -> Result<()> {
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
    Ok(())
}

fn validate_rbac(rbac: Option<&crate::rbac::RbacConfig>) -> Result<()> {
    if let Some(rbac) = rbac {
        if rbac.enabled {
            if rbac.default_role.trim().is_empty() {
                anyhow::bail!("RBAC configuration requires a non-empty 'defaultRole'");
            }
            for (token, assignment) in &rbac.tokens {
                if token.trim().is_empty() {
                    anyhow::bail!("RBAC token map contains an empty token key");
                }
                if assignment.role.trim().is_empty() {
                    anyhow::bail!(
                        "RBAC token assignment for token '{}' must specify a non-empty role",
                        token
                    );
                }
            }
        }
    }
    Ok(())
}

fn validate_profile_references(config: &McpConfig) -> Result<()> {
    for (profile_name, profile) in &config.profiles {
        if profile_name.trim().is_empty() {
            anyhow::bail!("Profile name cannot be empty");
        }
        for server_id in &profile.servers {
            if !config.mcp_servers.contains_key(server_id) {
                anyhow::bail!(
                    "Profile '{}' references unknown server '{}' (not defined in 'mcpServers')",
                    profile_name,
                    server_id
                );
            }
        }
    }
    Ok(())
}

fn validate_mcp_http_server(config: &McpConfig) -> Result<()> {
    // Validate mcpHttpServer block if present
    if let Some(ref http_srv) = config.mcp_http_server {
        // Non-loopback bind requires at least one auth mechanism to prevent open network exposure
        let is_loopback = matches!(http_srv.bind.as_str(), "127.0.0.1" | "::1" | "localhost");
        if !is_loopback {
            let has_auth = config
                .auth_token
                .as_ref()
                .map(|t| !t.trim().is_empty())
                .unwrap_or(false);
            let has_rbac = config.rbac.as_ref().map(|r| r.enabled).unwrap_or(false);
            if !has_auth && !has_rbac {
                anyhow::bail!(
                    "'mcpHttpServer.bind' is set to '{}' (non-loopback) but no 'authToken' or \
                     'rbac' authentication is configured. This would expose the MCP server to \
                     the network without any authentication.",
                    http_srv.bind
                );
            }
        }

        // Validate profile reference if specified
        if let Some(ref prof_id) = http_srv.profile {
            if !config.profiles.contains_key(prof_id) {
                anyhow::bail!(
                    "'mcpHttpServer.profile' references unknown profile '{}' (not defined in 'profiles')",
                    prof_id
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_config, AuthConfig, McpConfig, ServerConfig};
    use std::collections::HashMap;

    #[test]
    fn delayable_alias_hold_window_uses_global_policy_then_default() {
        let aliases = HashMap::from([(
            "mail.send".to_string(),
            super::AliasTarget::Detailed {
                target: "gmail.send".to_string(),
                summary: None,
                description: None,
                passthrough: false,
                reversibility: Some(super::ReversibilityConfig {
                    class: super::ReversibilityClass::Delayable,
                    hold_window_secs: None,
                    ttl_secs: None,
                    daily_limit: None,
                    pre_snapshot: None,
                    pre_undo_check: None,
                    test_target: None,
                    inverse: None,
                }),
            },
        )]);
        assert_eq!(
            super::resolve_hold_window_secs(&aliases, "gmail.send", None, Some(12)),
            Some(12)
        );
        assert_eq!(
            super::resolve_hold_window_secs(&aliases, "gmail.send", None, None),
            Some(30)
        );
        assert_eq!(
            super::resolve_hold_window_secs(&HashMap::new(), "gmail.send", None, Some(12)),
            None
        );
    }

    #[test]
    fn alias_hold_window_overrides_global_policy_and_profile_overrides_alias() {
        let aliases = HashMap::from([(
            "mail.send".to_string(),
            super::AliasTarget::Detailed {
                target: "gmail.send".to_string(),
                summary: None,
                description: None,
                passthrough: false,
                reversibility: Some(super::ReversibilityConfig {
                    class: super::ReversibilityClass::Delayable,
                    hold_window_secs: Some(7),
                    ttl_secs: None,
                    daily_limit: None,
                    pre_snapshot: None,
                    pre_undo_check: None,
                    test_target: None,
                    inverse: None,
                }),
            },
        )]);
        assert_eq!(
            super::resolve_hold_window_secs(&aliases, "gmail.send", None, Some(12)),
            Some(7)
        );
        assert_eq!(
            super::resolve_hold_window_secs(&aliases, "gmail.send", Some(4), Some(12)),
            Some(4)
        );
        assert_eq!(
            super::resolve_hold_window_secs(&aliases, "gmail.send", None, None),
            Some(7)
        );
    }

    #[test]
    fn config_rejects_zero_hold_window_and_conflicting_target_contracts() {
        let mut config = McpConfig {
            policy: Some(super::PolicyConfig {
                hold_window_secs: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());

        config.policy = None;
        let contract = |seconds| super::AliasTarget::Detailed {
            target: "mail.send".to_string(),
            summary: None,
            description: None,
            passthrough: false,
            reversibility: Some(super::ReversibilityConfig {
                class: super::ReversibilityClass::Delayable,
                hold_window_secs: Some(seconds),
                ttl_secs: None,
                daily_limit: None,
                pre_snapshot: None,
                pre_undo_check: None,
                test_target: None,
                inverse: None,
            }),
        };
        config
            .capability_aliases
            .insert("send_one".to_string(), contract(5));
        config
            .capability_aliases
            .insert("send_two".to_string(), contract(9));
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn delayable_alias_contract_deserializes_and_round_trips() {
        let value = serde_json::json!({
            "target": "gmail.send",
            "reversibility": {"class": "delayable", "holdWindowSecs": 4}
        });
        let alias: super::AliasTarget = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(alias.reversibility().unwrap().hold_window_secs, Some(4));
        assert_eq!(serde_json::to_value(alias).unwrap(), value);
        let reversible: super::ReversibilityConfig = serde_json::from_value(serde_json::json!({
            "class": "reversible",
            "inverse": {"capability": "tickets.set_priority", "args": {}}
        }))
        .unwrap();
        assert_eq!(reversible.class, super::ReversibilityClass::Reversible);
    }

    fn empty_server() -> ServerConfig {
        ServerConfig {
            command: None,
            args: vec![],
            env: HashMap::new(),
            url: None,
            protocol_version: None,
            allow_stateless: None,
            require_tool_pins: false,
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

    #[test]
    fn test_validate_profiles() {
        use super::ProfileConfig;

        let mut srv = empty_server();
        srv.command = Some("echo".to_string());
        let mut servers = HashMap::new();
        servers.insert("srv_a".to_string(), srv.clone());
        servers.insert("srv_b".to_string(), srv);

        let mut profiles = HashMap::new();
        profiles.insert(
            "valid_prof".to_string(),
            ProfileConfig {
                servers: vec!["srv_a".to_string()],
                description: Some("test profile".to_string()),
                policy: None,
                allow_agent_self_undo: false,
            },
        );

        let config = McpConfig {
            mcp_servers: servers.clone(),
            profiles: profiles.clone(),
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());

        // Invalid: references unknown server
        profiles.insert(
            "invalid_prof".to_string(),
            ProfileConfig {
                servers: vec!["srv_c".to_string()],
                description: None,
                policy: None,
                allow_agent_self_undo: false,
            },
        );
        let bad_config = McpConfig {
            mcp_servers: servers,
            profiles,
            ..Default::default()
        };
        let err = validate_config(&bad_config).unwrap_err();
        assert!(err
            .to_string()
            .contains("references unknown server 'srv_c'"));
    }

    #[test]
    fn test_alias_target_serialization() {
        let json_str = r#"{
            "capabilityAliases": {
                "short.tool": "server.original_tool",
                "custom.search": {
                    "target": "server.search",
                    "summary": "Custom short summary",
                    "description": "Custom detailed description"
                }
            }
        }"#;

        let cfg: McpConfig = serde_json::from_str(json_str).unwrap();
        assert_eq!(cfg.capability_aliases.len(), 2);

        let simple = cfg.capability_aliases.get("short.tool").unwrap();
        assert_eq!(simple.target(), "server.original_tool");
        assert_eq!(simple.summary(), None);
        assert_eq!(simple.description(), None);

        let detailed = cfg.capability_aliases.get("custom.search").unwrap();
        assert_eq!(detailed.target(), "server.search");
        assert_eq!(detailed.summary(), Some("Custom short summary"));
        assert_eq!(detailed.description(), Some("Custom detailed description"));

        // Roundtrip serialization
        let serialized = serde_json::to_string(&cfg).unwrap();
        let deserialized: McpConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cfg, deserialized);
    }
}
