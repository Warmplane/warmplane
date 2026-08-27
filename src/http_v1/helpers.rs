// Rust guideline compliant 2026-08-27

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

/// Query parameters containing optional profile selector.
#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct ProfileQuery {
    /// Optional profile constellation identifier.
    #[serde(default)]
    pub profile: Option<String>,
}

/// Resolves effective `ProfileContext` from HTTP request headers and query parameters.
///
/// Precedence:
/// 1. `X-Warmplane-Profile` header
/// 2. `?profile=` query parameter
///
/// # Returns
/// - `Ok(ProfileContext)`: If no profile requested (unrestricted) or requested profile is found in `state.profiles`.
/// - `Err((StatusCode, Value))`: If requested profile does not exist in `state.profiles` (`PROFILE_NOT_FOUND`).
pub async fn resolve_profile_context(
    state: &crate::daemon::AppState,
    headers: &HeaderMap,
    query: Option<&ProfileQuery>,
) -> Result<crate::context::ProfileContext, (axum::http::StatusCode, Value)> {
    let requested_profile = crate::context::extract_header_str(headers, "x-warmplane-profile")
        .or_else(|| {
            query
                .and_then(|q| q.profile.as_ref().map(|p| p.trim().to_string()))
                .filter(|s| !s.is_empty())
        });

    let Some(profile_id) = requested_profile else {
        return Ok(crate::context::ProfileContext::unrestricted());
    };

    let profiles_guard = state.profiles.read().await;
    match profiles_guard.get(&profile_id) {
        Some(profile_cfg) => {
            let prof_policy = profile_cfg
                .policy
                .as_ref()
                .map(|p| crate::daemon::Policy::from_config(Some(p.clone())));
            Ok(crate::context::ProfileContext::scoped_with_policy(
                profile_id,
                profile_cfg.servers.clone(),
                prof_policy,
            ))
        }
        None => {
            let trace_id = next_trace_id();
            let err_val = crate::http_v1::types::error_envelope(
                trace_id,
                None,
                None,
                crate::idempotency::RetryMetadata::safe("not_started"),
                "PROFILE_NOT_FOUND",
                format!("Profile '{}' is not defined in configuration", profile_id),
                false,
            );
            Err((axum::http::StatusCode::NOT_FOUND, err_val))
        }
    }
}

/// Computes profile-scoped catalog version string given active ProfileContext.
pub fn get_profile_scoped_catalog_version(
    base_version: impl AsRef<str>,
    profile_ctx: &crate::context::ProfileContext,
) -> String {
    match &profile_ctx.profile_id {
        Some(pid) => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            if let Some(srvs) = &profile_ctx.allowed_servers {
                let mut sorted: Vec<&String> = srvs.iter().collect();
                sorted.sort();
                for s in sorted {
                    s.hash(&mut hasher);
                }
            }
            if let Some(pol) = &profile_ctx.profile_policy {
                pol.allow.hash(&mut hasher);
                pol.deny.hash(&mut hasher);
                pol.require_approval.hash(&mut hasher);
            }
            let fingerprint = hasher.finish();
            format!("{}-p:{}:{:x}", base_version.as_ref(), pid, fingerprint)
        }
        None => base_version.as_ref().to_string(),
    }
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
