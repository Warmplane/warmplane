// Rust guideline compliant 2026-08-15

//! Helper utilities for HTTP headers, trace identifiers, idempotency keys, and redaction.

use axum::http::{header, HeaderMap, HeaderValue};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a monotonically increasing trace identifier string.
///
/// # Returns
/// Formatted trace ID string (e.g., `trace-1`).
pub fn next_trace_id() -> String {
    format!("trace-{}", TRACE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Validates whether HTTP `If-None-Match` header matches catalog version for HTTP 304 response.
///
/// # Arguments
/// * `req_headers` - Incoming request HTTP headers.
/// * `catalog_version` - Current active catalog ETag version string.
///
/// # Returns
/// `true` if catalog version matches `If-None-Match` header, `false` otherwise.
pub fn check_if_none_match(req_headers: &HeaderMap, catalog_version: impl AsRef<str>) -> bool {
    let version_ref = catalog_version.as_ref();
    if let Some(if_none_match) = req_headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            let val_clean = val.trim();
            if val_clean == "*" {
                return true;
            }
            let unquoted = val_clean.trim_matches('"');
            return unquoted == version_ref;
        }
    }
    false
}

/// Constructs HTTP `ETag` response headers matching catalog version.
///
/// # Arguments
/// * `catalog_version` - Current active catalog ETag version string.
///
/// # Returns
/// HeaderMap containing formatted `ETag` header.
pub fn make_etag_header(catalog_version: impl AsRef<str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let etag_val = format!("\"{}\"", catalog_version.as_ref());
    if let Ok(hv) = HeaderValue::from_str(&etag_val) {
        headers.insert(header::ETAG, hv);
    }
    headers
}

/// Resolves idempotency key from payload body or standard HTTP headers.
///
/// # Arguments
/// * `payload_key` - Key extracted directly from JSON request body.
/// * `headers` - HTTP request headers (`idempotency-key` or `x-idempotency-key`).
///
/// # Returns
/// Resolved idempotency key string if present and non-empty.
pub fn resolve_idempotency_key(payload_key: Option<String>, headers: &HeaderMap) -> Option<String> {
    if let Some(k) = payload_key.filter(|s| !s.trim().is_empty()) {
        return Some(k);
    }
    crate::context::extract_header_str(headers, "idempotency-key")
        .or_else(|| crate::context::extract_header_str(headers, "x-idempotency-key"))
}

/// Default sensitive key names automatically redacted from log/event payloads.
pub const DEFAULT_REDACT_KEYS: &[&str] = &[
    "token",
    "api_key",
    "apikey",
    "password",
    "secret",
    "authorization",
    "auth",
    "access_token",
    "private_key",
    "bearer",
];

/// Recursively masks sensitive JSON keys according to security policy.
///
/// # Arguments
/// * `value` - JSON object, array, or primitive value to sanitize.
/// * `redact_keys` - Slice of key names whose values should be replaced with `<redacted>`.
///
/// # Returns
/// Sanitized JSON Value.
pub fn redact_value(value: Value, redact_keys: &[String]) -> Value {
    let lower_keys: Vec<String> = redact_keys.iter().map(|k| k.to_lowercase()).collect();
    redact_value_internal(value, &lower_keys)
}

fn redact_value_internal(value: Value, lower_keys: &[String]) -> Value {
    match value {
        Value::Object(map) => {
            let mut output = serde_json::Map::new();
            for (key, nested) in map {
                let key_lower = key.to_lowercase();
                let should_redact = lower_keys.iter().any(|k| k == &key_lower)
                    || DEFAULT_REDACT_KEYS.iter().any(|&k| k == key_lower);
                if should_redact {
                    output.insert(key, Value::String("<redacted>".to_string()));
                } else {
                    output.insert(key, redact_value_internal(nested, lower_keys));
                }
            }
            Value::Object(output)
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|entry| redact_value_internal(entry, lower_keys))
                .collect(),
        ),
        primitive => primitive,
    }
}
