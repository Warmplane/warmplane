// Rust guideline compliant 2026-08-13

use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

/// Request tracing context metadata passed across HTTP envelopes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RequestContext {
    /// Operation identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// Work item identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    /// Actor or user identity identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    /// Grant or authorization token identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<String>,
}

#[allow(dead_code)]
impl RequestContext {
    /// Creates a builder for constructing a `RequestContext` instance (`M-INIT-BUILDER`).
    pub fn builder() -> RequestContextBuilder {
        RequestContextBuilder::default()
    }
}

/// Builder pattern implementation for `RequestContext` (`M-INIT-BUILDER`).
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RequestContextBuilder {
    operation_id: Option<String>,
    work_item_id: Option<String>,
    actor_id: Option<String>,
    grant_id: Option<String>,
}

#[allow(dead_code)]
impl RequestContextBuilder {
    /// Sets the operation identifier.
    pub fn operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }

    /// Sets the work item identifier.
    pub fn work_item_id(mut self, id: impl Into<String>) -> Self {
        self.work_item_id = Some(id.into());
        self
    }

    /// Sets the actor identifier.
    pub fn actor_id(mut self, id: impl Into<String>) -> Self {
        self.actor_id = Some(id.into());
        self
    }

    /// Sets the grant identifier.
    pub fn grant_id(mut self, id: impl Into<String>) -> Self {
        self.grant_id = Some(id.into());
        self
    }

    /// Builds the `RequestContext`.
    pub fn build(self) -> RequestContext {
        RequestContext {
            operation_id: self.operation_id,
            work_item_id: self.work_item_id,
            actor_id: self.actor_id,
            grant_id: self.grant_id,
        }
    }
}

/// Helper function to extract and trim string values from HTTP headers.
///
/// # Arguments
/// * `headers` - Reference to HTTP HeaderMap.
/// * `key` - Header field key name.
///
/// # Returns
/// `Option<String>` containing non-empty header string value if present.
pub fn extract_header_str(headers: &HeaderMap, key: impl AsRef<str>) -> Option<String> {
    headers
        .get(key.as_ref())
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Resolves request identifier prioritizing payload ID, HTTP header, or fallback trace ID.
///
/// # Arguments
/// * `payload_id` - Optional ID from request payload envelope.
/// * `headers` - HTTP headers.
/// * `fallback_trace_id` - Fallback trace ID string.
///
/// # Returns
/// Resolved request ID string.
pub fn resolve_request_id(
    payload_id: Option<String>,
    headers: &HeaderMap,
    fallback_trace_id: impl Into<String>,
) -> String {
    if let Some(id) = payload_id.filter(|s| !s.trim().is_empty()) {
        return id;
    }
    if let Some(id) = extract_header_str(headers, "x-request-id")
        .or_else(|| extract_header_str(headers, "x-correlation-id"))
    {
        return id;
    }
    fallback_trace_id.into()
}

/// Merges context fields from payload envelope and HTTP headers.
///
/// # Arguments
/// * `payload_context` - Optional context from payload envelope.
/// * `headers` - HTTP headers.
///
/// # Returns
/// Merged `RequestContext` instance.
pub fn resolve_request_context(
    payload_context: Option<RequestContext>,
    headers: &HeaderMap,
) -> RequestContext {
    let base = payload_context.unwrap_or_default();
    RequestContext {
        operation_id: base
            .operation_id
            .or_else(|| extract_header_str(headers, "x-operation-id")),
        work_item_id: base
            .work_item_id
            .or_else(|| extract_header_str(headers, "x-work-item-id")),
        actor_id: base
            .actor_id
            .or_else(|| extract_header_str(headers, "x-actor-id")),
        grant_id: base
            .grant_id
            .or_else(|| extract_header_str(headers, "x-grant-id")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_resolution_combines_payload_and_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", "hdr-req-123".parse().unwrap());
        headers.insert("x-actor-id", "hdr-actor-9".parse().unwrap());

        let payload_ctx = RequestContext {
            operation_id: Some("payload-op-1".to_string()),
            ..Default::default()
        };

        let req_id = resolve_request_id(None, &headers, "fallback-1".to_string());
        assert_eq!(req_id, "hdr-req-123");

        let resolved_ctx = resolve_request_context(Some(payload_ctx), &headers);
        assert_eq!(resolved_ctx.operation_id.as_deref(), Some("payload-op-1"));
        assert_eq!(resolved_ctx.actor_id.as_deref(), Some("hdr-actor-9"));
        assert_eq!(resolved_ctx.work_item_id, None);
    }
}
