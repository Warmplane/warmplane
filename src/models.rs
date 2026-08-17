// Rust guideline compliant 2026-08-13

use clap::{Parser, Subcommand};

use crate::config::DEFAULT_CONFIG_PATH;

/// Top-level command line argument parser model for Warmplane CLI.
#[derive(Parser)]
#[command(
    name = "warmplane",
    about = "The local control plane that keeps MCP sessions warm"
)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Warmplane CLI subcommands.
#[derive(Subcommand)]
pub enum Commands {
    /// Validate config file and exit (no daemon startup)
    ValidateConfig {
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Boot the background daemon
    Daemon {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Run as an MCP stdio server exposing the lightweight facade
    McpServer {
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Manage upstream MCP servers (add, remove, list, get, test)
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
    /// Manage Warmplane configuration, aliases, policies, and imports
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Trigger a hot-reload of active upstream servers and policies from config file
    Reload {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// List compact capabilities from the v1 facade API
    ListCapabilities {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Search capabilities using hybrid lexical and semantic search
    SearchCapabilities {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        query: String,
        #[arg(short, long, default_value = "8")]
        limit: usize,
        #[arg(short, long)]
        server: Vec<String>,
        #[arg(short, long)]
        tag: Vec<String>,
    },
    /// Describe one capability with full on-demand schema from the v1 facade API
    DescribeCapability {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        id: String,
    },
    /// Call one capability through the v1 facade API
    CallCapability {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        id: String,
        #[arg(short, long, default_value = "{}")]
        params: String,
        #[arg(long)]
        request_id: Option<String>,
        #[arg(long)]
        operation_id: Option<String>,
        #[arg(long)]
        work_item_id: Option<String>,
        #[arg(long)]
        actor_id: Option<String>,
        #[arg(long)]
        grant_id: Option<String>,
        #[arg(long)]
        idempotency_key: Option<String>,
        /// Filter result payload with JSONPath expression (e.g. "$.items[*].name")
        #[arg(long)]
        jsonpath: Option<String>,
        /// Limit returned output lines
        #[arg(long)]
        limit_lines: Option<usize>,
        /// Truncate returned output byte size
        #[arg(long)]
        truncate_bytes: Option<usize>,
    },
    /// Execute a chained batch of tool capabilities
    BatchCallCapabilities {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        /// JSON array of step objects, or inline JSON string
        #[arg(short, long)]
        steps: Option<String>,
        /// Path to JSON file containing steps array
        #[arg(short = 'f', long)]
        file: Option<String>,
    },
    /// List compact resources from the v1 facade API
    ListResources {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Read one resource through the v1 facade API
    ReadResource {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        id: String,
        #[arg(long)]
        request_id: Option<String>,
        #[arg(long)]
        operation_id: Option<String>,
        #[arg(long)]
        work_item_id: Option<String>,
        #[arg(long)]
        actor_id: Option<String>,
        #[arg(long)]
        grant_id: Option<String>,
    },
    /// List compact prompts from the v1 facade API
    ListPrompts {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Get one prompt through the v1 facade API
    GetPrompt {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        id: String,
        #[arg(short, long, default_value = "{}")]
        arguments: String,
        #[arg(long)]
        request_id: Option<String>,
        #[arg(long)]
        operation_id: Option<String>,
        #[arg(long)]
        work_item_id: Option<String>,
        #[arg(long)]
        actor_id: Option<String>,
        #[arg(long)]
        grant_id: Option<String>,
    },
    /// List catalog events from the v1 change feed API
    ListCatalogEvents {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        #[arg(short, long)]
        after: Option<String>,
    },
    /// Cancel an active in-flight operation by request ID
    CancelOperation {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        id: String,
    },
    /// Manage Human-in-the-Loop (HITL) capability approvals
    Approvals {
        #[command(subcommand)]
        command: ApprovalCommands,
    },
}

/// Upstream server management subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum ServerCommands {
    /// Add or configure an upstream MCP server
    Add(Box<ServerAddArgs>),
    /// Remove an upstream MCP server by identifier
    Remove {
        /// Server identifier
        name: String,
        /// Bypass confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// List all configured upstream MCP servers
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Get details of a specific upstream MCP server
    Get {
        /// Server identifier
        name: String,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Test reachability and capability discovery with upstream server
    Test {
        /// Server identifier
        name: String,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// Global Warmplane configuration management subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Initialize a new Warmplane configuration file
    Init {
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
        /// Overwrite existing configuration if present
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// Show current merged configuration
    Show {
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Import MCP server configurations from Claude Desktop, Cursor, or Zed
    Import {
        /// Automatically overwrite existing servers with same identifier
        #[arg(short = 'y', long)]
        yes: bool,
        /// Specific file path to import from (optional)
        #[arg(long)]
        from_file: Option<String>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Manage capability, resource, and prompt aliases
    Alias {
        #[command(subcommand)]
        command: AliasCommands,
    },
    /// Manage security access policies
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
    /// Manage global fault tolerance and process supervision
    Resilience {
        #[command(subcommand)]
        command: ResilienceCommands,
    },
    /// Manage WORM audit trail and SIEM exporter settings
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },
    /// Hot-reload daemon configuration and upstream servers from disk
    Reload {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// Global resilience and supervisor subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum ResilienceCommands {
    /// Configure global resilience and circuit breaker settings
    Set {
        /// Consecutive failures required to trip circuit to Open
        #[arg(long)]
        failure_threshold: Option<u32>,
        /// Cooldown time in milliseconds in Open state before probe attempts
        #[arg(long)]
        cooldown_ms: Option<u64>,
        /// Consecutive successful probe calls in HalfOpen to reset circuit
        #[arg(long)]
        consecutive_successes: Option<u32>,
        /// Automatically restart crashed stdio child processes
        #[arg(long)]
        auto_restart: Option<bool>,
        /// Maximum restart attempts before giving up
        #[arg(long)]
        max_restarts: Option<u32>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Show current resilience and supervisor settings
    Show {
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// WORM audit logging and SIEM exporter subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AuditCommands {
    /// Configure WORM audit logging and SIEM export targets
    Set {
        /// Enable or disable audit logging
        #[arg(long)]
        enabled: Option<bool>,
        /// Append-only audit log file path on disk (e.g. warmplane_audit.jsonl)
        #[arg(long)]
        file_path: Option<String>,
        /// Queue buffer capacity
        #[arg(long)]
        buffer_capacity: Option<usize>,
        /// Flush interval in milliseconds
        #[arg(long)]
        flush_interval_ms: Option<u64>,
        /// Maximum batch size
        #[arg(long)]
        max_batch_size: Option<usize>,
        /// Add generic SIEM HTTP Webhook destination URL
        #[arg(long)]
        siem_webhook_url: Option<String>,
        /// SIEM HTTP Webhook Bearer authorization header
        #[arg(long)]
        siem_webhook_auth: Option<String>,
        /// Add Splunk HEC collector endpoint URL
        #[arg(long)]
        siem_splunk_url: Option<String>,
        /// Splunk HEC authentication token
        #[arg(long)]
        siem_splunk_token: Option<String>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Show current WORM audit logging and SIEM configuration
    Show {
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// Alias management subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum AliasCommands {
    /// Set an alias
    Set {
        /// Alias type (tool, resource, prompt)
        kind: String,
        /// Alias name
        alias: String,
        /// Target canonical identifier or URI
        target: String,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Remove an alias
    Remove {
        /// Alias type (tool, resource, prompt)
        kind: String,
        /// Alias name
        alias: String,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// List all aliases
    List {
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// Policy management subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum PolicyCommands {
    /// Add capability patterns to allow list
    Allow {
        /// Capability pattern(s) (e.g. "github.*")
        patterns: Vec<String>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Add capability patterns to deny list
    Deny {
        /// Capability pattern(s) (e.g. "filesystem.write*")
        patterns: Vec<String>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Add capability patterns requiring human operator approval
    RequireApproval {
        /// Capability pattern(s) (e.g. "docker.run*", "db.write*")
        patterns: Vec<String>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Add sensitive keys to redact list
    Redact {
        /// Sensitive key names (e.g. "api_key", "password")
        keys: Vec<String>,
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Show current security policy rules
    Show {
        /// Path to Warmplane configuration file
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// Human-in-the-Loop (HITL) approval subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum ApprovalCommands {
    /// List all pending approvals and recent history
    List {
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Get details of a specific approval ticket
    Get {
        /// Approval ticket identifier (e.g. appr-1723668200-1)
        id: String,
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Approve a pending capability execution
    Approve {
        /// Approval ticket identifier
        id: String,
        /// Operator identifier
        #[arg(short = 'o', long, default_value = "cli-operator")]
        operator: String,
        /// Optional modified JSON arguments to execute
        #[arg(short = 'm', long)]
        modified_args: Option<String>,
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
    /// Reject a pending capability execution
    Reject {
        /// Approval ticket identifier
        id: String,
        /// Operator identifier
        #[arg(short = 'o', long, default_value = "cli-operator")]
        operator: String,
        /// Reason explaining the rejection
        #[arg(short = 'r', long)]
        reason: Option<String>,
        #[arg(short = 'p', long)]
        port: Option<u16>,
        #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
        config: String,
    },
}

/// Arguments for `warmplane server add` command.
#[derive(clap::Args, Debug, Clone)]
pub struct ServerAddArgs {
    /// Server identifier (e.g. github, filesystem)
    pub name: Option<String>,
    /// Stdio executable command (e.g. npx, uvx, python)
    #[arg(long)]
    pub command: Option<String>,
    /// Command line arguments (repeat flag or space/comma separated)
    #[arg(short, long)]
    pub arg: Vec<String>,
    /// Environment variables in KEY=VALUE format
    #[arg(short, long)]
    pub env: Vec<String>,
    /// Remote HTTP/SSE URL endpoint
    #[arg(long)]
    pub url: Option<String>,
    /// Static Bearer token
    #[arg(long)]
    pub bearer_token: Option<String>,
    /// Environment variable containing Bearer token
    #[arg(long)]
    pub bearer_env: Option<String>,
    /// HTTP Basic auth username
    #[arg(long)]
    pub username: Option<String>,
    /// HTTP Basic auth password
    #[arg(long)]
    pub password: Option<String>,
    /// HTTP Basic auth password environment variable
    #[arg(long)]
    pub password_env: Option<String>,
    /// OAuth2 client ID
    #[arg(long)]
    pub client_id: Option<String>,
    /// OAuth2 authorization server URL
    #[arg(long)]
    pub auth_server: Option<String>,
    /// OAuth2 scopes (comma-separated)
    #[arg(long)]
    pub scopes: Option<String>,
    /// Consecutive failures required to trip circuit to Open
    #[arg(long)]
    pub failure_threshold: Option<u32>,
    /// Cooldown time in milliseconds in Open state before probe attempts
    #[arg(long)]
    pub cooldown_ms: Option<u64>,
    /// Consecutive successful probe calls in HalfOpen to reset circuit
    #[arg(long)]
    pub consecutive_successes: Option<u32>,
    /// Automatically restart crashed stdio child processes
    #[arg(long)]
    pub auto_restart: Option<bool>,
    /// Maximum restart attempts before giving up
    #[arg(long)]
    pub max_restarts: Option<u32>,
    /// Force interactive guided setup prompt
    #[arg(short, long)]
    pub interactive: bool,
    /// Path to Warmplane configuration file
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH)]
    pub config: String,
}
