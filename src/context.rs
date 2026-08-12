use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RequestContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<String>,
}

pub fn extract_header_str(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn resolve_request_id(
    payload_id: Option<String>,
    headers: &HeaderMap,
    fallback_trace_id: String,
) -> String {
    if let Some(id) = payload_id.filter(|s| !s.trim().is_empty()) {
        return id;
    }
    if let Some(id) = extract_header_str(headers, "x-request-id")
        .or_else(|| extract_header_str(headers, "x-correlation-id"))
    {
        return id;
    }
    fallback_trace_id
}

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
