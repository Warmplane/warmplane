use clap::{Parser, Subcommand};

use crate::config::DEFAULT_CONFIG_PATH;

#[derive(Parser)]
#[command(
    name = "warmplane",
    about = "The local control plane that keeps MCP sessions warm"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

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
}
