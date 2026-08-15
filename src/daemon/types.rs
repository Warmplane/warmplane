// Rust guideline compliant 2026-08-15

//! Core metadata, actor message types, error categories, and shared state aliases for daemon workers.

use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, oneshot, RwLock};

use crate::config::ServerConfig;

/// Messages dispatched across worker threads to upstream MCP servers.
pub enum ServerMsg {
    /// Execute a tool call.
    CallTool {
        /// Tool name on upstream server.
        name: String,
        /// Execution parameters.
        params: Value,
        /// Multi-roundtrip client input responses.
        input_responses: Option<std::collections::BTreeMap<String, Value>>,
        /// Multi-roundtrip request state.
        request_state: Option<String>,
        /// Response reply channel.
        reply: oneshot::Sender<Result<Value, UpstreamCallError>>,
    },
    /// Read a resource URI.
    ReadResource {
        /// Resource URI identifier.
        uri: String,
        /// Multi-roundtrip client input responses.
        input_responses: Option<std::collections::BTreeMap<String, Value>>,
        /// Multi-roundtrip request state.
        request_state: Option<String>,
        /// Response reply channel.
        reply: oneshot::Sender<Result<Value, UpstreamCallError>>,
    },
    /// Render a prompt template.
    GetPrompt {
        /// Prompt identifier name.
        name: String,
        /// Arguments map for prompt rendering.
        arguments: Option<serde_json::Map<String, Value>>,
        /// Multi-roundtrip client input responses.
        input_responses: Option<std::collections::BTreeMap<String, Value>>,
        /// Multi-roundtrip request state.
        request_state: Option<String>,
        /// Response reply channel.
        reply: oneshot::Sender<Result<Value, UpstreamCallError>>,
    },
}

/// Upstream MCP execution error categories.
#[derive(Debug, Clone)]
pub enum UpstreamCallError {
    /// Upstream error message string.
    Upstream(String),
    /// Operation timed out.
    Timeout,
}

/// Metadata describing a registered capability tool.
#[derive(Clone, Debug)]
pub struct CapabilityMeta {
    /// Server identifier providing this capability.
    pub server: String,
    /// Tool name on upstream server.
    pub tool: String,
    /// Short summary description.
    pub summary: String,
    /// Detailed description.
    pub description: String,
    /// JSON schema for tool arguments.
    pub input_schema: Value,
    /// Metadata tags.
    pub tags: Vec<String>,
    /// Usage examples.
    pub examples: Vec<Value>,
}

/// Metadata describing a registered resource.
#[derive(Clone, Debug)]
pub struct ResourceMeta {
    /// Server identifier providing this resource.
    pub server: String,
    /// Resource URI identifier.
    pub uri: String,
    /// Resource human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional MIME type string.
    pub mime_type: Option<String>,
    /// Metadata tags.
    pub tags: Vec<String>,
}

/// Metadata describing a registered prompt template.
#[derive(Clone, Debug)]
pub struct PromptMeta {
    /// Server identifier providing this prompt.
    pub server: String,
    /// Prompt identifier name.
    pub name: String,
    /// Optional human-readable title.
    pub title: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// List of expected prompt arguments.
    pub arguments: Vec<Value>,
    /// Metadata tags.
    pub tags: Vec<String>,
}

/// Thread-safe map of active upstream server sender mailboxes.
pub type SharedServers = Arc<RwLock<HashMap<String, mpsc::Sender<ServerMsg>>>>;
/// Thread-safe map of registered capabilities.
pub type SharedCapabilities = Arc<RwLock<HashMap<String, CapabilityMeta>>>;
/// Thread-safe map of registered resources.
pub type SharedResources = Arc<RwLock<HashMap<String, ResourceMeta>>>;
/// Thread-safe map of registered prompt templates.
pub type SharedPrompts = Arc<RwLock<HashMap<String, PromptMeta>>>;
/// Thread-safe active security policy.
pub type SharedPolicy = Arc<RwLock<crate::daemon::policy::Policy>>;
/// Thread-safe active catalog SHA256 version string.
pub type SharedCatalogVersion = Arc<RwLock<String>>;
/// Thread-safe map of active server configurations.
pub type SharedServerConfigs = Arc<RwLock<HashMap<String, ServerConfig>>>;
/// Thread-safe map of server status envelopes.
pub type SharedServerStatuses = Arc<RwLock<HashMap<String, Value>>>;
