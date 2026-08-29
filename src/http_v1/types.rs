// Rust guideline compliant 2026-08-27

//! Request and response data transfer objects and envelope constructors for HTTP v1 facade API.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Query parameters for listing catalog events.
#[derive(Deserialize, Debug, Clone)]
pub struct CatalogEventsQuery {
    /// Optional cursor ID to fetch events after.
    pub after: Option<String>,
}

/// Response envelope for catalog event change feed.
#[derive(Serialize, Debug, Clone)]
pub struct CatalogEventsResponse {
    /// Current catalog version ETag.
    pub catalog_version: String,
    /// Next cursor ID for event pagination.
    pub cursor: String,
    /// List of catalog mutation events.
    pub events: Vec<crate::catalog::CatalogEvent>,
}

/// Request body for capability execution.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CallCapabilityRequest {
    /// Identifier or alias of capability to execute.
    pub capability_id: String,
    /// JSON arguments for capability execution.
    pub args: Value,
    /// Optional request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context metadata envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
    /// Optional key for idempotent request deduplication.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Optional MRTR client input responses for multi-roundtrip retry.
    #[serde(default)]
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Optional MRTR opaque request state for multi-roundtrip retry.
    #[serde(default)]
    pub request_state: Option<String>,
    /// Request asynchronous execution as a SEP-2663 task handle.
    #[serde(default)]
    pub async_task: bool,
}

/// Request body for reading a resource.
#[derive(Deserialize, Debug, Clone)]
pub struct ReadResourceRequest {
    /// Identifier or alias of resource URI to read.
    pub resource_id: String,
    /// Optional request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context metadata envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
    /// Optional key for idempotent request deduplication.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Optional MRTR client input responses for multi-roundtrip retry.
    #[serde(default)]
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Optional MRTR opaque request state for multi-roundtrip retry.
    #[serde(default)]
    pub request_state: Option<String>,
}

/// Request body for fetching a prompt template.
#[derive(Deserialize, Debug, Clone)]
pub struct GetPromptRequest {
    /// Identifier or alias of prompt name to get.
    pub prompt_id: String,
    /// Optional arguments map for prompt rendering.
    #[serde(default)]
    pub arguments: Option<Value>,
    /// Optional request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context metadata envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
    /// Optional key for idempotent request deduplication.
    #[serde(default)]
    pub idempotency_key: Option<String>,
    /// Optional MRTR client input responses for multi-roundtrip retry.
    #[serde(default)]
    pub input_responses: Option<std::collections::BTreeMap<String, Value>>,
    /// Optional MRTR opaque request state for multi-roundtrip retry.
    #[serde(default)]
    pub request_state: Option<String>,
}

pub(crate) fn default_search_limit() -> usize {
    8
}

/// Request body for hybrid capability search.
#[derive(Deserialize, Debug, Clone)]
pub struct SearchCapabilitiesRequest {
    /// Optional plain-text search query string.
    pub query: Option<String>,
    /// Maximum number of search results to return (default 8).
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    /// Filter results to specified server IDs.
    #[serde(default)]
    pub server_ids: Vec<String>,
    /// Filter results to specified tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Filter results to specified execution modes.
    #[serde(default)]
    pub modes: Vec<String>,
}

/// Request body for prompt/resource argument autocompletion.
#[derive(Deserialize, Debug, Clone)]
pub struct CompletionRequest {
    /// Reference type (`"prompt"` or `"resource"`).
    pub ref_type: String,
    /// Identifier or name of the prompt or resource.
    pub ref_name: String,
    /// Name of argument to autocomplete.
    pub argument_name: String,
    /// Current prefix value typed by user/agent.
    #[serde(default)]
    pub argument_value: String,
}

/// Request body for sampling LLM completion delegation.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SamplingRequest {
    /// Server identifier originating the sampling request.
    pub server_id: String,
    /// Conversation messages array.
    #[serde(default)]
    pub messages: Vec<crate::sampling::SamplingMessage>,
    /// Optional model selection hints and preferences.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<crate::sampling::ModelPreferences>,
    /// Optional system instruction prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Optional context inclusion mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_context: Option<String>,
    /// Optional max tokens limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    /// Optional stop sequences.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    /// Optional metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    /// Optional asynchronous mode (if true, returns ticket immediately without blocking).
    #[serde(default)]
    pub async_mode: Option<bool>,
}

/// Request body for resolving a pending sampling ticket.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RespondSamplingRequest {
    /// Generated assistant completion result.
    pub result: crate::sampling::CreateMessageResult,
}

/// Query parameters for listing sampling requests.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct SamplingListQuery {
    /// Optional server identifier filter.
    pub server_id: Option<String>,
    /// Optional status filter (`pending`, `completed`, `expired`, `rejected`).
    pub status: Option<String>,
}

/// Request payload for adding or updating an upstream server.
#[derive(Deserialize, Debug, Clone)]
pub struct UpsertServerRequest {
    /// Server name identifier.
    pub name: String,
    /// Server configuration details.
    pub server: crate::config::ServerConfig,
}

/// Request payload for ecosystem config import.
#[derive(Deserialize, Debug, Clone)]
pub struct ImportConfigRequest {
    /// Optional custom path to source file.
    pub source_path: Option<String>,
    /// Whether to overwrite existing server entries.
    pub overwrite: Option<bool>,
}

/// Request payload for updating alias mappings.
#[derive(Deserialize, Debug, Clone)]
pub struct UpdateAliasRequest {
    /// Target alias kind (`tool`, `resource`, `prompt`).
    pub kind: String,
    /// Alias name.
    pub alias: String,
    /// Canonical target identifier (or `None` to delete alias).
    pub target: Option<String>,
    /// Optional custom summary override.
    #[serde(default)]
    pub summary: Option<String>,
    /// Optional custom detailed description override.
    #[serde(default)]
    pub description: Option<String>,
}

/// Request body for creating or updating a server profile constellation.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpsertProfileRequest {
    /// Profile identifier.
    pub name: String,
    /// List of upstream server identifiers included in constellation.
    pub servers: Vec<String>,
    /// Optional human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional per-profile security and governance policy.
    #[serde(default)]
    pub policy: Option<crate::config::PolicyConfig>,
}

/// Request body for approving a pending capability execution.
#[derive(Deserialize, Debug, Clone)]
pub struct ApproveTicketRequest {
    /// Identifier of the human operator approving the action.
    pub operator: String,
    /// Optional modified arguments to execute instead of original payload.
    #[serde(default)]
    pub modified_args: Option<Value>,
}

/// Request body for rejecting a pending capability execution.
#[derive(Deserialize, Debug, Clone)]
pub struct RejectTicketRequest {
    /// Identifier of the human operator rejecting the action.
    pub operator: String,
    /// Reason explaining why the action was rejected.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Request body for attaching Warmplane to an external AI client.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct AttachClientApiRequest {
    /// Optional server constellation profile.
    #[serde(default)]
    pub profile: Option<String>,
    /// Custom path to Warmplane configuration file.
    #[serde(default)]
    pub config_path: Option<String>,
}

/// Request body for storing a secret in OS Keychain.
#[derive(Deserialize, Debug, Clone)]
pub struct UpsertSecretRequest {
    /// Secret key / account name
    pub key: String,
    /// Secret plaintext value
    pub value: String,
    /// Optional service name (defaults to 'warmplane')
    #[serde(default)]
    pub service: Option<String>,
}

/// Constructs a standardized error response envelope.
///
/// # Arguments
/// * `trace_id` - Monotonically increasing trace identifier.
/// * `request_id` - Contextual request ID.
/// * `context` - Request context metadata envelope.
/// * `retry` - Retry policy metadata.
/// * `code` - Machine-readable error code string.
/// * `message` - Human-readable error message.
/// * `retryable` - Boolean indicating whether error is transient and safe to retry.
///
/// # Returns
/// Standardized JSON Value error envelope.
pub fn error_envelope(
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    retry: crate::idempotency::RetryMetadata,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> Value {
    let ctx_val = context.unwrap_or_default();
    json!({
        "ok": false,
        "request_id": request_id,
        "context": ctx_val,
        "trace_id": trace_id,
        "data": null,
        "error": {
            "code": code,
            "message": message.into(),
            "retryable": retryable,
        },
        "retry": retry,
    })
}
