use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap as AxumHeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use base64::Engine as _;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{oneshot, RwLock};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub issuer: String,
}

#[derive(Debug, Clone)]
pub struct OAuth2TokenState {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub scopes: HashSet<String>,
}

#[derive(Clone)]
pub struct OAuth2ClientState {
    pub server_id: String,
    pub client_id: String,
    pub _authorization_server_url: String,
    pub scopes: Arc<RwLock<HashSet<String>>>,
    pub token_state: Arc<RwLock<Option<OAuth2TokenState>>>,
    pub discovery: DiscoveryMetadata,
    pub client_metadata_url: Option<String>,
    pub remote_base_url: String,
}

// Global registry of active OAuth2 clients to route callbacks and proxy requests.
#[derive(Clone, Default)]
pub struct OAuthRegistry {
    pub clients: Arc<RwLock<HashMap<String, OAuth2ClientState>>>,
    // Channel to send the code/state back to the waiting flow
    pub pending_auths: Arc<RwLock<HashMap<String, oneshot::Sender<CallbackPayload>>>>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub iss: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug)]
pub struct CallbackPayload {
    pub code: String,
    pub iss: Option<String>,
}

// ----------------------------------------------------
// Phase 1: Dynamic Discovery (RFC 9728 & RFC 8414)
// ----------------------------------------------------

pub fn construct_prm_url(resource_url: &str) -> Result<String> {
    let parsed = reqwest::Url::parse(resource_url)
        .with_context(|| format!("Invalid resource URL: {}", resource_url))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("Missing host in resource URL"))?;
    let scheme = parsed.scheme();
    let port = parsed.port();

    let host_with_port = if let Some(p) = port {
        format!("{}:{}", host, p)
    } else {
        host.to_string()
    };

    let path = parsed.path();

    // According to RFC 9728, insert .well-known/oauth-protected-resource between host and path
    let prm_url = if path == "/" || path.is_empty() {
        format!(
            "{}://{}/.well-known/oauth-protected-resource",
            scheme, host_with_port
        )
    } else {
        format!(
            "{}://{}/.well-known/oauth-protected-resource{}",
            scheme, host_with_port, path
        )
    };

    Ok(prm_url)
}

#[derive(Deserialize)]
struct ProtectedResourceMetadata {
    authorization_servers: Vec<String>,
}

#[derive(Deserialize)]
struct AuthorizationServerMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
    issuer: String,
}

pub async fn discover_auth_server(
    resource_url: &str,
    configured_as_url: Option<&str>,
) -> Result<DiscoveryMetadata> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut auth_servers = Vec::new();

    // Try PRM discovery
    if let Ok(prm_url) = construct_prm_url(resource_url) {
        info!(%prm_url, "attempting Protected Resource Metadata discovery");
        if let Ok(resp) = client.get(&prm_url).send().await {
            if resp.status().is_success() {
                if let Ok(prm) = resp.json::<ProtectedResourceMetadata>().await {
                    auth_servers = prm.authorization_servers;
                }
            }
        }
    }

    // Add configured AS URL as fallback
    if let Some(configured) = configured_as_url {
        if !auth_servers.contains(&configured.to_string()) {
            auth_servers.push(configured.to_string());
        }
    }

    if auth_servers.is_empty() {
        return Err(anyhow!(
            "No authorization servers discovered or configured for resource {}",
            resource_url
        ));
    }

    // Probe the discovered authorization servers in sequence using RFC 8414
    for as_url in auth_servers {
        // Try RFC 8414 discovery locations:
        // Priority 1: path insertion /.well-known/oauth-authorization-server
        // Priority 2: OIDC configuration /.well-known/openid-configuration
        // Priority 3: Root configuration
        let parsed_as = reqwest::Url::parse(&as_url)?;
        let host = parsed_as
            .host_str()
            .ok_or_else(|| anyhow!("AS missing host"))?;
        let scheme = parsed_as.scheme();
        let port = parsed_as.port();
        let host_with_port = if let Some(p) = port {
            format!("{}:{}", host, p)
        } else {
            host.to_string()
        };
        let path = parsed_as.path();

        let urls_to_try = if path == "/" || path.is_empty() {
            vec![
                format!(
                    "{}://{}/.well-known/oauth-authorization-server",
                    scheme, host_with_port
                ),
                format!(
                    "{}://{}/.well-known/openid-configuration",
                    scheme, host_with_port
                ),
            ]
        } else {
            vec![
                format!(
                    "{}://{}/.well-known/oauth-authorization-server{}",
                    scheme, host_with_port, path
                ),
                format!(
                    "{}://{}{}/.well-known/openid-configuration",
                    scheme, host_with_port, path
                ),
                format!(
                    "{}://{}/.well-known/oauth-authorization-server",
                    scheme, host_with_port
                ),
            ]
        };

        for probe_url in urls_to_try {
            info!(%probe_url, "probing AS metadata");
            if let Ok(resp) = client.get(&probe_url).send().await {
                if resp.status().is_success() {
                    let content_type = resp
                        .headers()
                        .get(CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");
                    // Fall back if reverse proxy returns non-JSON HTML payload
                    if !content_type.contains("json") && !content_type.is_empty() {
                        warn!(%probe_url, "non-JSON response received, skipping");
                        continue;
                    }
                    if let Ok(meta) = resp.json::<AuthorizationServerMetadata>().await {
                        // Validate issuer claim matches the AS base URL
                        if !meta
                            .issuer
                            .starts_with(&format!("{}://{}", scheme, host_with_port))
                        {
                            warn!(issuer = %meta.issuer, as_url = %as_url, "issuer verification failed, mismatch");
                            continue;
                        }
                        return Ok(DiscoveryMetadata {
                            authorization_endpoint: meta.authorization_endpoint,
                            token_endpoint: meta.token_endpoint,
                            issuer: meta.issuer,
                        });
                    }
                }
            }
        }
    }

    Err(anyhow!(
        "Failed to discover valid Authorization Server metadata for resource {}",
        resource_url
    ))
}

// ----------------------------------------------------
// Phase 4: PKCE helper
// ----------------------------------------------------

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

pub fn generate_pkce() -> Pkce {
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();

    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

    Pkce {
        verifier,
        challenge,
    }
}

// ----------------------------------------------------
// OAuth2 Flow Orchestration
// ----------------------------------------------------

pub async fn run_oauth2_flow(
    client_state: &OAuth2ClientState,
    registry: &OAuthRegistry,
    local_proxy_port: u16,
) -> Result<OAuth2TokenState> {
    let state_param: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let pkce = generate_pkce();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", local_proxy_port);

    // Set up callback channel
    let (tx, rx) = oneshot::channel();
    {
        let mut pendings = registry.pending_auths.write().await;
        pendings.insert(state_param.clone(), tx);
    }

    let scopes_guard = client_state.scopes.read().await;
    let scopes_str = scopes_guard.iter().cloned().collect::<Vec<_>>().join(" ");

    // Build Authorization URL
    let mut auth_url = reqwest::Url::parse(&client_state.discovery.authorization_endpoint)?;
    {
        let mut query = auth_url.query_pairs_mut();
        query.append_pair("response_type", "code");
        query.append_pair("client_id", &client_state.client_id);
        query.append_pair("redirect_uri", &redirect_uri);
        query.append_pair("code_challenge", &pkce.challenge);
        query.append_pair("code_challenge_method", "S256");
        query.append_pair("state", &state_param);
        if !scopes_str.is_empty() {
            query.append_pair("scope", &scopes_str);
        }
        // RFC 8707 Resource Indicators
        query.append_pair("resource", &client_state.remote_base_url);
    }

    let auth_url_str = auth_url.to_string();
    tracing::warn!(
        server_id = %client_state.server_id,
        auth_url = %auth_url_str,
        "Warmplane authorization required for server. Please visit URL in browser to authorize access."
    );

    // Open browser securely
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open")
        .arg(&auth_url_str)
        .spawn();

    // Wait for redirect callback
    let callback_payload = tokio::time::timeout(Duration::from_secs(300), rx)
        .await
        .map_err(|_| anyhow!("Authorization timed out (5 minutes)"))?
        .map_err(|_| anyhow!("Callback channel closed unexpectedly"))?;

    // Phase 3 & SEP-2468: Validate Issuer iss parameter
    if let Some(callback_iss) = callback_payload.iss {
        // String comparison must be exact without normalization
        if callback_iss != client_state.discovery.issuer {
            return Err(anyhow!(
                "Authorization issuer mismatch: expected '{}', got '{}' (possible mix-up attack)",
                client_state.discovery.issuer,
                callback_iss
            ));
        }
    }

    // Exchange auth code for token
    let client = reqwest::Client::new();
    let mut form = HashMap::new();
    form.insert("grant_type", "authorization_code");
    form.insert("code", &callback_payload.code);
    form.insert("redirect_uri", &redirect_uri);
    form.insert("client_id", &client_state.client_id);
    form.insert("code_verifier", &pkce.verifier);
    form.insert("resource", &client_state.remote_base_url);

    info!(token_endpoint = %client_state.discovery.token_endpoint, "exchanging code for token");
    let resp = client
        .post(&client_state.discovery.token_endpoint)
        .form(&form)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await?;
        return Err(anyhow!("Token exchange failed: {}", err_text));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        refresh_token: Option<String>,
        scope: Option<String>,
    }

    let token_resp = resp.json::<TokenResponse>().await?;
    let mut scopes = HashSet::new();
    if let Some(scope_field) = token_resp.scope {
        for s in scope_field.split_whitespace() {
            scopes.insert(s.to_string());
        }
    } else {
        scopes = scopes_guard.clone();
    }

    let token_state = OAuth2TokenState {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        scopes,
    };

    Ok(token_state)
}

// ----------------------------------------------------
// Phase 6: Silent token refresh
// ----------------------------------------------------

pub async fn refresh_access_token(client_state: &OAuth2ClientState) -> Result<OAuth2TokenState> {
    let current_token_state = {
        let guard = client_state.token_state.read().await;
        guard.clone()
    };

    let Some(token_state) = current_token_state else {
        return Err(anyhow!("No active token state found to refresh"));
    };

    let Some(refresh_token) = token_state.refresh_token.as_ref() else {
        return Err(anyhow!("No refresh token available"));
    };

    info!(server_id = %client_state.server_id, "attempting silent token refresh");

    let client = reqwest::Client::new();
    let mut form = HashMap::new();
    form.insert("grant_type", "refresh_token");
    form.insert("refresh_token", refresh_token);
    form.insert("client_id", &client_state.client_id);
    form.insert("resource", &client_state.remote_base_url);

    let resp = client
        .post(&client_state.discovery.token_endpoint)
        .form(&form)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err_text = resp.text().await?;
        return Err(anyhow!("Refresh token exchange failed: {}", err_text));
    }

    #[derive(Deserialize)]
    struct RefreshResponse {
        access_token: String,
        refresh_token: Option<String>,
        scope: Option<String>,
    }

    let refresh_resp = resp.json::<RefreshResponse>().await?;
    let mut scopes = HashSet::new();
    if let Some(scope_field) = refresh_resp.scope {
        for s in scope_field.split_whitespace() {
            scopes.insert(s.to_string());
        }
    } else {
        scopes = token_state.scopes.clone();
    }

    // Standard public client refresh token rotation (use new refresh token if returned, fallback to current)
    let new_refresh_token = refresh_resp.refresh_token.or(Some(refresh_token.clone()));

    let new_token_state = OAuth2TokenState {
        access_token: refresh_resp.access_token,
        refresh_token: new_refresh_token,
        scopes,
    };

    let mut guard = client_state.token_state.write().await;
    *guard = Some(new_token_state.clone());

    Ok(new_token_state)
}

// ----------------------------------------------------
// OAuth2 Local Proxy and Callback Axum Server
// ----------------------------------------------------

async fn handle_callback(
    State(registry): State<OAuthRegistry>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    if let Some(err) = query.error {
        let desc = query.error_description.unwrap_or_default();
        error!(error = %err, description = %desc, "received error during OAuth callback");
        return (
            StatusCode::BAD_REQUEST,
            format!("Authorization failed: {}. Description: {}", err, desc),
        );
    }

    let Some(code) = query.code else {
        return (
            StatusCode::BAD_REQUEST,
            "Missing code parameter".to_string(),
        );
    };

    let Some(state) = query.state else {
        return (
            StatusCode::BAD_REQUEST,
            "Missing state parameter".to_string(),
        );
    };

    let mut pendings = registry.pending_auths.write().await;
    if let Some(tx) = pendings.remove(&state) {
        let _ = tx.send(CallbackPayload {
            code,
            iss: query.iss,
        });
        (
            StatusCode::OK,
            "Authorization successful! You can close this window and return to the terminal."
                .to_string(),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            "State mismatch or expired request".to_string(),
        )
    }
}

async fn handle_client_metadata(
    State(registry): State<OAuthRegistry>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    let clients = registry.clients.read().await;
    let Some(client_state) = clients.get(&server_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // Serves the CIMD document dynamically
    let client_name = format!("Warmplane MCP Gateway ({})", client_state.server_id);
    let redirect_uri = "http://127.0.0.1:0/callback".to_string(); // loopback standard placeholder
    Json(serde_json::json!({
        "client_id": client_state.client_metadata_url.as_ref().cloned().unwrap_or_default(),
        "client_name": client_name,
        "redirect_uris": [ redirect_uri ]
    }))
    .into_response()
}

async fn handle_proxy_request(
    State(registry): State<OAuthRegistry>,
    Path((server_id, path)): Path<(String, String)>,
    method: Method,
    headers: AxumHeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let client_state = {
        let clients = registry.clients.read().await;
        let Some(c) = clients.get(&server_id) else {
            return (
                StatusCode::NOT_FOUND,
                format!("Client for server_id '{}' not found", server_id),
            )
                .into_response();
        };
        c.clone()
    };

    // Resolve upstream URL
    let separator = if client_state.remote_base_url.ends_with('/') || path.starts_with('/') {
        ""
    } else {
        "/"
    };
    let upstream_url = format!("{}{}{}", client_state.remote_base_url, separator, path);

    // Fetch request with retry loop for 401/403 challenges
    let max_attempts = 3;
    for attempt in 1..=max_attempts {
        let access_token = {
            let guard = client_state.token_state.read().await;
            guard.as_ref().map(|s| s.access_token.clone())
        };

        let Some(token) = access_token else {
            return (
                StatusCode::UNAUTHORIZED,
                "No valid OAuth2 token found. Authorization required.".to_string(),
            )
                .into_response();
        };

        // Forward headers except host/auth
        let mut forward_headers = HeaderMap::new();
        for (name, value) in headers.iter() {
            if name != "host" && name != "authorization" {
                if let Ok(hname) = HeaderName::from_bytes(name.as_str().as_bytes()) {
                    forward_headers.insert(hname, value.clone());
                }
            }
        }
        match HeaderValue::from_str(&format!("Bearer {}", token)) {
            Ok(val) => {
                forward_headers.insert(AUTHORIZATION, val);
            }
            Err(e) => {
                tracing::error!(error = %e, "Invalid token header value");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid token header value".to_string(),
                )
                    .into_response();
            }
        }

        let client = reqwest::Client::new();
        let resp = match client
            .request(method.clone(), &upstream_url)
            .headers(forward_headers)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to reach upstream server: {}", e),
                )
                    .into_response();
            }
        };

        let status = resp.status();

        // Phase 5: Silent token refresh on 401 Unauthorized
        if status == StatusCode::UNAUTHORIZED && attempt < max_attempts {
            info!("received 401 Unauthorized. Attempting silent token refresh...");
            if refresh_access_token(&client_state).await.is_ok() {
                continue;
            }
        }

        // Phase 4: Dynamic scope negotiation on 403 Forbidden with insufficient_scope
        if status == StatusCode::FORBIDDEN && attempt < max_attempts {
            let www_auth = resp
                .headers()
                .get("www-authenticate")
                .and_then(|v| v.to_str().ok());
            if let Some(header_str) = www_auth {
                if header_str.contains("insufficient_scope") {
                    // Extract scope
                    if let Some(scope_pos) = header_str.find("scope=\"") {
                        let scope_part = &header_str[scope_pos + 7..];
                        if let Some(end_pos) = scope_part.find('\"') {
                            let requested_scopes_str = &scope_part[..end_pos];
                            let requested_scopes: HashSet<String> = requested_scopes_str
                                .split_whitespace()
                                .map(ToString::to_string)
                                .collect();

                            // Accumulate scopes (union)
                            let mut current_scopes = client_state.scopes.write().await;
                            let original_size = current_scopes.len();
                            for s in requested_scopes {
                                current_scopes.insert(s);
                            }

                            if current_scopes.len() > original_size {
                                info!("received 403 insufficient_scope. Triggering Step-Up authorization...");
                                // Trigger interactive flow for accumulated scopes
                                // Temporary local port to fetch callback
                                let ephemeral_port =
                                    local_port_from_url(&upstream_url).unwrap_or(9095);
                                drop(current_scopes); // Release write lock before flow
                                if let Ok(new_token) =
                                    run_oauth2_flow(&client_state, &registry, ephemeral_port).await
                                {
                                    let mut guard = client_state.token_state.write().await;
                                    *guard = Some(new_token);
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Forward response headers
        let mut res_headers = AxumHeaderMap::new();
        for (name, value) in resp.headers().iter() {
            if let Ok(hname) = axum::http::HeaderName::from_bytes(name.as_str().as_bytes()) {
                res_headers.insert(hname, value.clone());
            }
        }

        let body_bytes = resp.bytes().await.unwrap_or_default();
        return (status, res_headers, body_bytes).into_response();
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to authorize request after multiple attempts.".to_string(),
    )
        .into_response()
}

fn local_port_from_url(url_str: &str) -> Option<u16> {
    reqwest::Url::parse(url_str)
        .ok()
        .and_then(|u: reqwest::Url| u.port())
}

// Spawns the central background proxy / callback server on an ephemeral port.
pub async fn start_oauth_proxy_server(registry: OAuthRegistry) -> Result<u16> {
    let app = Router::new()
        .route("/callback", get(handle_callback))
        .route("/client-metadata/:server_id", get(handle_client_metadata))
        .route("/proxy/:server_id/*path", any(handle_proxy_request))
        .with_state(registry);

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;
    let port = local_addr.port();

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!(error = %e, "OAuth proxy server failed");
        }
    });

    info!(port, "OAuth proxy and callback server started");
    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_prm_url() {
        assert_eq!(
            construct_prm_url("https://api.example.com").unwrap(),
            "https://api.example.com/.well-known/oauth-protected-resource"
        );
        assert_eq!(
            construct_prm_url("https://api.example.com/mcp").unwrap(),
            "https://api.example.com/.well-known/oauth-protected-resource/mcp"
        );
        assert_eq!(
            construct_prm_url("https://tenant.example.com/v1/agents").unwrap(),
            "https://tenant.example.com/.well-known/oauth-protected-resource/v1/agents"
        );
    }

    #[test]
    fn test_pkce_generation() {
        let pkce = generate_pkce();
        assert_eq!(pkce.verifier.len(), 64);
        assert!(pkce.challenge.len() > 30);
    }
}
