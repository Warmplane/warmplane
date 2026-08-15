// Rust guideline compliant 2026-08-15

//! Transport construction, HTTP headers, secret resolution, and client negotiation for upstream MCP servers.

use anyhow::{anyhow, Context, Result};
use base64::Engine as _;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};

use crate::config::{AuthConfig, ServerConfig};

/// Default Model Context Protocol version string supported by the proxy.
pub const DEFAULT_MCP_PROTOCOL_VERSION: &str = "2026-07-28";

/// Resolves a secret from direct config value or environment variable fallback.
///
/// # Arguments
/// * `direct_value` - Direct configuration string value.
/// * `env_name` - Environment variable name.
/// * `server_id` - Upstream server identifier for error reporting.
/// * `field_name` - Field name for error reporting.
///
/// # Returns
/// Resolved secret string value.
///
/// # Errors
/// Returns an error if neither direct value nor environment variable is configured or readable.
pub fn resolve_secret(
    direct_value: &Option<String>,
    env_name: &Option<String>,
    server_id: &str,
    field_name: &str,
) -> Result<String> {
    if let Some(value) = direct_value {
        return Ok(value.clone());
    }
    let env_var = env_name
        .as_ref()
        .ok_or_else(|| anyhow!("Server '{}' missing {}", server_id, field_name))?;
    std::env::var(env_var).with_context(|| {
        format!(
            "Server '{}' could not read env var '{}' for {}",
            server_id, env_var, field_name
        )
    })
}

/// Constructs HTTP header map including protocol version, custom headers, and authentication.
///
/// # Arguments
/// * `server_id` - Upstream server identifier.
/// * `srv_cfg` - Server configuration details.
///
/// # Returns
/// Prepared `HeaderMap` instance.
///
/// # Errors
/// Returns an error if any header name or value is malformed.
pub fn build_http_headers(server_id: &str, srv_cfg: &ServerConfig) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let protocol_version = srv_cfg
        .protocol_version
        .as_deref()
        .unwrap_or(DEFAULT_MCP_PROTOCOL_VERSION);

    headers.insert(
        HeaderName::from_static("mcp-protocol-version"),
        HeaderValue::from_str(protocol_version).with_context(|| {
            format!(
                "Server '{}' has invalid protocolVersion '{}'",
                server_id, protocol_version
            )
        })?,
    );

    for (raw_name, raw_value) in &srv_cfg.headers {
        let name = HeaderName::from_bytes(raw_name.as_bytes()).with_context(|| {
            format!(
                "Server '{}' has invalid HTTP header name '{}'",
                server_id, raw_name
            )
        })?;
        let value = HeaderValue::from_str(raw_value).with_context(|| {
            format!(
                "Server '{}' has invalid HTTP header value for '{}'",
                server_id, raw_name
            )
        })?;
        headers.insert(name, value);
    }

    if let Some(auth) = &srv_cfg.auth {
        match auth {
            AuthConfig::Bearer { token, token_env } => {
                let token = resolve_secret(token, token_env, server_id, "bearer token")?;
                let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", token))
                    .with_context(|| {
                        format!(
                            "Server '{}' has invalid bearer token (header encoding failed)",
                            server_id
                        )
                    })?;
                auth_value.set_sensitive(true);
                headers.insert(AUTHORIZATION, auth_value);
            }
            AuthConfig::Basic {
                username,
                password,
                password_env,
            } => {
                let password = resolve_secret(password, password_env, server_id, "basic password")?;
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(format!("{}:{}", username, password));
                let mut auth_value =
                    HeaderValue::from_str(&format!("Basic {}", encoded)).with_context(|| {
                        format!(
                            "Server '{}' has invalid basic auth credentials (header encoding failed)",
                            server_id
                        )
                    })?;
                auth_value.set_sensitive(true);
                headers.insert(AUTHORIZATION, auth_value);
            }
            AuthConfig::Oauth2 { .. } => {
                // Auth headers are injected dynamically by the local proxy
            }
        }
    }

    Ok(headers)
}
