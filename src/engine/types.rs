// Rust guideline compliant 2026-08-20

//! Strongly typed envelopes, capability representations, errors, and batch results for embedded engine execution (`M-CANONICAL-DOCS`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{context::RequestContext, idempotency::RetryMetadata};

/// Strongly typed response envelope returned by all embedded Warmplane operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope<T> {
    /// Whether the execution completed successfully.
    pub ok: bool,
    /// Optional contextual request identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Request context metadata propagation envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<RequestContext>,
    /// Monotonically generated trace identifier.
    pub trace_id: String,
    /// Payload data on success (`None` if failed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error details on failure (`None` if successful).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<WarmplaneError>,
    /// Retry safety classification and execution state.
    pub retry: RetryMetadata,
}

impl<T> Envelope<T> {
    /// Constructs a successful response envelope.
    pub fn success(
        trace_id: String,
        request_id: Option<String>,
        context: Option<RequestContext>,
        data: T,
        retry: RetryMetadata,
    ) -> Self {
        Self {
            ok: true,
            request_id,
            context,
            trace_id,
            data: Some(data),
            error: None,
            retry,
        }
    }

    /// Constructs an error response envelope.
    pub fn failure(
        trace_id: String,
        request_id: Option<String>,
        context: Option<RequestContext>,
        error: WarmplaneError,
        retry: RetryMetadata,
    ) -> Self {
        Self {
            ok: false,
            request_id,
            context,
            trace_id,
            data: None,
            error: Some(error),
            retry,
        }
    }
}

/// Standard error structure included in failed response envelopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WarmplaneError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable error description.
    pub message: String,
    /// Whether this error is transient and safe to retry.
    pub retryable: bool,
    /// Optional operator identifier if rejected by human-in-the-loop review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
}

impl WarmplaneError {
    /// Creates a new `WarmplaneError`.
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            operator: None,
        }
    }

    /// Creates a HITL operator rejection error.
    pub fn operator_rejected(operator: impl Into<String>, reason: Option<String>) -> Self {
        let op = operator.into();
        let reason_str = reason.map(|r| format!(": {}", r)).unwrap_or_default();
        Self {
            code: "OPERATION_REJECTED_BY_OPERATOR".to_string(),
            message: format!("Human operator rejected execution{}", reason_str),
            retryable: false,
            operator: Some(op),
        }
    }
}

/// Compact capability metadata summary for catalog indexing and discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySummary {
    /// Unique capability identifier or alias.
    pub id: String,
    /// Short summary of tool function.
    pub summary: String,
    /// Upstream MCP server hosting this tool.
    pub server: String,
    /// Underlying tool name on upstream server.
    pub tool: String,
    /// Discovery tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Detailed capability specification including schema and examples for on-demand inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityDetail {
    /// Unique capability identifier.
    pub id: String,
    /// Upstream MCP server hosting this tool.
    pub server: String,
    /// Underlying tool name on upstream server.
    pub tool: String,
    /// Comprehensive tool description.
    pub description: String,
    /// JSON Schema definition for arguments.
    pub input_schema: Value,
    /// Usage examples.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Value>,
}

/// Response payload for capability catalog listing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitiesListResponse {
    /// API schema version (`"v1"`).
    pub version: String,
    /// List of registered capability summaries.
    pub capabilities: Vec<CapabilitySummary>,
}

/// Response payload for capability search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySearchResponse {
    /// API schema version (`"v1"`).
    pub version: String,
    /// Active catalog version ETag.
    pub catalog_version: String,
    /// Plaintext query searched.
    pub query: String,
    /// Total number of matching capabilities.
    pub total: usize,
    /// Search results ranked by relevance.
    pub capabilities: Vec<crate::search::hybrid::CapabilitySearchResult>,
}

/// Response payload for capability schema describe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityDescribeResponse {
    /// API schema version (`"v1"`).
    pub version: String,
    /// Detailed capability metadata.
    pub capability: CapabilityDetail,
}

/// Compact resource metadata summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSummary {
    /// Resource identifier or alias.
    pub id: String,
    /// Upstream server providing resource.
    pub server: String,
    /// Resource URI.
    pub uri: String,
    /// Human-readable resource name.
    pub name: String,
    /// Optional resource description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional MIME type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Resource tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Response payload for listing resources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourcesListResponse {
    /// API schema version (`"v1"`).
    pub version: String,
    /// List of available resources.
    pub resources: Vec<ResourceSummary>,
}

/// Compact prompt metadata summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptSummary {
    /// Prompt identifier or alias.
    pub id: String,
    /// Upstream server providing prompt.
    pub server: String,
    /// Prompt name.
    pub name: String,
    /// Optional human-readable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Expected prompt template arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<Value>,
    /// Prompt tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Response payload for listing prompts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptsListResponse {
    /// API schema version (`"v1"`).
    pub version: String,
    /// List of registered prompts.
    pub prompts: Vec<PromptSummary>,
}

/// Health and status overview of the embedded Warmplane control plane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineHealthStatus {
    /// Current catalog SHA256 version ETag.
    pub catalog_version: String,
    /// Map of connected upstream servers and their statuses.
    pub server_statuses: std::collections::HashMap<String, Value>,
    /// Circuit breaker status for each upstream server.
    pub circuit_breakers: std::collections::HashMap<String, crate::circuit_breaker::CircuitState>,
    /// Total capability tool calls processed.
    pub total_tool_calls: u64,
    /// Total duration in microseconds across all tool executions.
    pub total_tool_duration_us: u64,
}

/// Optional invocation settings for calling a capability tool.
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    /// Contextual request identifier.
    pub request_id: Option<String>,
    /// Distributed tracing or security context metadata.
    pub context: Option<RequestContext>,
    /// Idempotency deduplication key.
    pub idempotency_key: Option<String>,
    /// Multi-roundtrip client input responses.
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Multi-roundtrip opaque request state.
    pub request_state: Option<String>,
    /// Named server profile restricting visibility.
    pub profile: Option<String>,
    /// Request asynchronous execution as a SEP-2663 task handle.
    pub async_task: bool,
}

impl ExecutionOptions {
    /// Creates default `ExecutionOptions`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets request ID.
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Sets profile.
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Sets idempotency key.
    pub fn with_idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
}

/// Optional parameters for reading a resource.
#[derive(Debug, Clone, Default)]
pub struct ReadResourceOptions {
    /// Contextual request identifier.
    pub request_id: Option<String>,
    /// Distributed tracing or security context metadata.
    pub context: Option<RequestContext>,
    /// Multi-roundtrip client input responses.
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Multi-roundtrip opaque request state.
    pub request_state: Option<String>,
    /// Named server profile restricting visibility.
    pub profile: Option<String>,
}

/// Optional parameters for rendering a prompt template.
#[derive(Debug, Clone, Default)]
pub struct GetPromptOptions {
    /// Contextual request identifier.
    pub request_id: Option<String>,
    /// Distributed tracing or security context metadata.
    pub context: Option<RequestContext>,
    /// Template interpolation arguments.
    pub arguments: Option<Value>,
    /// Multi-roundtrip client input responses.
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Multi-roundtrip opaque request state.
    pub request_state: Option<String>,
    /// Named server profile restricting visibility.
    pub profile: Option<String>,
}
