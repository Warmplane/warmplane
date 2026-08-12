use anyhow::{anyhow, Context, Result};
use axum::{
    routing::{get, post},
    Router,
};
use base64::Engine as _;
use rmcp::{
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    transport::{streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport, TokioChildProcess},
    ServiceExt,
};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    process::Command,
    sync::{mpsc, oneshot, RwLock},
    time::timeout,
};
use tracing::info;

use crate::{
    config::{AuthConfig, McpConfig, PolicyConfig, ServerConfig, DEFAULT_TOOL_TIMEOUT_MS},
    http_v1,
};

const DEFAULT_MCP_PROTOCOL_VERSION: &str = "2025-11-25";

pub enum ServerMsg {
    CallTool {
        name: String,
        params: Value,
        reply: oneshot::Sender<Result<Value, UpstreamCallError>>,
    },
    ReadResource {
        uri: String,
        reply: oneshot::Sender<Result<Value, UpstreamCallError>>,
    },
    GetPrompt {
        name: String,
        arguments: Option<serde_json::Map<String, Value>>,
        reply: oneshot::Sender<Result<Value, UpstreamCallError>>,
    },
}

#[derive(Debug, Clone)]
pub enum UpstreamCallError {
    Upstream(String),
    Timeout,
}

#[derive(Clone)]
pub struct CapabilityMeta {
    pub server: String,
    pub tool: String,
    pub summary: String,
    pub description: String,
    pub input_schema: Value,
    pub tags: Vec<String>,
    pub examples: Vec<Value>,
}

#[derive(Clone)]
pub struct ResourceMeta {
    pub server: String,
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct PromptMeta {
    pub server: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Vec<Value>,
    pub tags: Vec<String>,
}

#[derive(Clone, Default)]
pub struct Policy {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub redact_keys: Vec<String>,
}

impl Policy {
    pub fn from_config(config: Option<PolicyConfig>) -> Self {
        let Some(config) = config else {
            return Self::default();
        };
        Self {
            allow: config.allow,
            deny: config.deny,
            redact_keys: config
                .redact_keys
                .into_iter()
                .map(|k| k.to_lowercase())
                .collect(),
        }
    }

    pub fn allows(&self, id: &str) -> bool {
        if self
            .deny
            .iter()
            .any(|pattern| wildcard_match(pattern, id))
        {
            return false;
        }

        if self.allow.is_empty() {
            return true;
        }

        self.allow
            .iter()
            .any(|pattern| wildcard_match(pattern, id))
    }
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    pattern == value
}

fn resolve_secret(
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

fn build_http_headers(server_id: &str, srv_cfg: &ServerConfig) -> Result<HeaderMap> {
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
            format!("Server '{}' has invalid HTTP header name '{}'", server_id, raw_name)
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

#[derive(Clone)]
pub struct AppState {
    pub servers: Arc<HashMap<String, mpsc::Sender<ServerMsg>>>,
    pub capabilities: Arc<HashMap<String, CapabilityMeta>>,
    pub resources: Arc<HashMap<String, ResourceMeta>>,
    pub prompts: Arc<HashMap<String, PromptMeta>>,
    pub tool_timeout_ms: u64,
    pub policy: Policy,
    pub search_engine: Arc<crate::search::HybridSearchEngine>,
    pub catalog_version: String,
    pub event_store: Arc<crate::catalog::CatalogEventStore>,
    pub idempotency_store: Arc<crate::idempotency::IdempotencyStore>,
    pub operation_registry: crate::operations::OperationRegistry,
}

pub fn compute_catalog_version(
    capabilities: &HashMap<String, CapabilityMeta>,
    resources: &HashMap<String, ResourceMeta>,
    prompts: &HashMap<String, PromptMeta>,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut cap_keys: Vec<_> = capabilities.keys().collect();
    cap_keys.sort();
    for k in cap_keys {
        hasher.update(k.as_bytes());
    }
    let mut res_keys: Vec<_> = resources.keys().collect();
    res_keys.sort();
    for k in res_keys {
        hasher.update(k.as_bytes());
    }
    let mut prompt_keys: Vec<_> = prompts.keys().collect();
    prompt_keys.sort();
    for k in prompt_keys {
        hasher.update(k.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

pub async fn initialize_state(config: McpConfig) -> Result<AppState> {
    let mut server_channels = HashMap::new();
    let mut capabilities = HashMap::new();
    let mut resources = HashMap::new();
    let mut prompts = HashMap::new();
    let tool_timeout_ms = config.tool_timeout_ms.unwrap_or(DEFAULT_TOOL_TIMEOUT_MS);
    let policy = Policy::from_config(config.policy.clone());
    let event_store = Arc::new(crate::catalog::CatalogEventStore::new());

    info!(server_count = config.mcp_servers.len(), "booting upstream MCP servers");

    // Initialize central OAuth registry and proxy server if any server uses OAuth2
    let oauth_registry = crate::oauth2::OAuthRegistry::default();
    let mut oauth_proxy_port = None;

    let has_oauth2 = config.mcp_servers.values().any(|s| {
        matches!(s.auth, Some(AuthConfig::Oauth2 { .. }))
    });

    if has_oauth2 {
        let port = crate::oauth2::start_oauth_proxy_server(oauth_registry.clone()).await?;
        oauth_proxy_port = Some(port);
    }

    for (server_id, srv_cfg) in config.mcp_servers {
        info!(%server_id, "starting upstream server");
        let mcp_client = if let Some(command) = &srv_cfg.command {
            let mut cmd = Command::new(command);
            cmd.args(&srv_cfg.args);
            cmd.envs(&srv_cfg.env);

            let transport = TokioChildProcess::new(cmd)
                .with_context(|| format!("Failed to spawn process for {}", server_id))?;

            ().serve(transport).await.with_context(|| {
                format!("Failed to negotiate stdio MCP connection for {}", server_id)
            })?
        } else if let Some(url) = &srv_cfg.url {
            let mut target_url = url.clone();

            if let Some(AuthConfig::Oauth2 { client_id, authorization_server_url, scopes, client_metadata_url }) = &srv_cfg.auth {
                let proxy_port = oauth_proxy_port.ok_or_else(|| anyhow!("OAuth proxy server failed to initialize"))?;

                // Discover the OAuth authorization server metadata
                let discovery = crate::oauth2::discover_auth_server(url, Some(authorization_server_url)).await?;

                let client_state = crate::oauth2::OAuth2ClientState {
                    server_id: server_id.clone(),
                    client_id: client_id.clone(),
                    _authorization_server_url: authorization_server_url.clone(),
                    scopes: Arc::new(RwLock::new(scopes.iter().cloned().collect())),
                    token_state: Arc::new(RwLock::new(None)),
                    discovery,
                    client_metadata_url: client_metadata_url.clone(),
                    remote_base_url: url.clone(),
                };

                // Perform the initial browser-based PKCE auth flow
                let initial_token = crate::oauth2::run_oauth2_flow(&client_state, &oauth_registry, proxy_port).await?;
                {
                    let mut guard = client_state.token_state.write().await;
                    *guard = Some(initial_token);
                }

                // Add to registry so proxy can handle incoming requests
                {
                    let mut clients = oauth_registry.clients.write().await;
                    clients.insert(server_id.clone(), client_state);
                }

                // Rewrite transport target URL to route through the local proxy
                target_url = format!("http://127.0.0.1:{}/proxy/{}/", proxy_port, server_id);
            }

            let headers = build_http_headers(&server_id, &srv_cfg)?;
            let http_client = reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .with_context(|| format!("Failed to build HTTP client for {}", server_id))?;

            let mut transport_config = StreamableHttpClientTransportConfig::with_uri(target_url);
            if let Some(allow_stateless) = srv_cfg.allow_stateless {
                transport_config.allow_stateless = allow_stateless;
            }
            let transport = StreamableHttpClientTransport::with_client(http_client, transport_config);
            ().serve(transport).await.with_context(|| {
                format!(
                    "Failed to negotiate streamable HTTP MCP connection for {}",
                    server_id
                )
            })?
        } else {
            return Err(anyhow!(
                "Server '{}' missing transport selector: set exactly one of 'command' or 'url'",
                server_id
            ));
        };

        if let Ok(tools) = mcp_client.list_tools(Default::default()).await {
            if let Ok(tools_json) = serde_json::to_value(&tools) {
                if let Some(tools_array) = tools_json.get("tools").and_then(|t| t.as_array()) {
                    for tool in tools_array {
                        if let Some(tool_name) = tool.get("name").and_then(|n| n.as_str()) {
                            info!(%server_id, tool_name = %tool_name, "registered upstream tool");

                            let source_id = format!("{}.{}", server_id, tool_name);
                            let capability_id = config
                                .capability_aliases
                                .get(&source_id)
                                .cloned()
                                .unwrap_or(source_id);

                            if capabilities.contains_key(&capability_id) {
                                return Err(anyhow!(
                                    "Duplicate capability id '{}'. Use capabilityAliases to disambiguate",
                                    capability_id
                                ));
                            }

                            let summary = tool
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("No summary available")
                                .to_string();
                            let description = summary.clone();
                            let input_schema =
                                tool.get("inputSchema").cloned().unwrap_or_else(|| json!({}));

                            capabilities.insert(
                                capability_id.clone(),
                                CapabilityMeta {
                                    server: server_id.clone(),
                                    tool: tool_name.to_string(),
                                    summary,
                                    description,
                                    input_schema,
                                    tags: vec![server_id.clone()],
                                    examples: vec![],
                                },
                            );
                            event_store.record("capability", &capability_id, "added");
                        }
                    }
                }
            }
        }

        if let Ok(listed_resources) = mcp_client.list_resources(Default::default()).await {
            if let Ok(resources_json) = serde_json::to_value(&listed_resources) {
                if let Some(resource_array) = resources_json.get("resources").and_then(|r| r.as_array()) {
                    for resource in resource_array {
                        let Some(uri) = resource.get("uri").and_then(|v| v.as_str()) else {
                            continue;
                        };
                        let name = resource
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(uri)
                            .to_string();

                        let source_id = format!("{}.{}", server_id, uri);
                        let resource_id = config
                            .resource_aliases
                            .get(&source_id)
                            .cloned()
                            .unwrap_or(source_id);

                        if resources.contains_key(&resource_id) {
                            return Err(anyhow!(
                                "Duplicate resource id '{}'. Use resourceAliases to disambiguate",
                                resource_id
                            ));
                        }

                        let description = resource
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        let mime_type = resource
                            .get("mimeType")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);

                        info!(%server_id, uri = %uri, "registered upstream resource");
                        resources.insert(
                            resource_id.clone(),
                            ResourceMeta {
                                server: server_id.clone(),
                                uri: uri.to_string(),
                                name,
                                description,
                                mime_type,
                                tags: vec![server_id.clone()],
                            },
                        );
                        event_store.record("resource", &resource_id, "added");
                    }
                }
            }
        }

        if let Ok(listed_prompts) = mcp_client.list_all_prompts().await {
            if let Ok(prompts_json) = serde_json::to_value(&listed_prompts) {
                if let Some(prompt_array) = prompts_json.as_array() {
                    for prompt in prompt_array {
                        let Some(name) = prompt.get("name").and_then(|v| v.as_str()) else {
                            continue;
                        };

                        let source_id = format!("{}.{}", server_id, name);
                        let prompt_id = config
                            .prompt_aliases
                            .get(&source_id)
                            .cloned()
                            .unwrap_or(source_id);

                        if prompts.contains_key(&prompt_id) {
                            return Err(anyhow!(
                                "Duplicate prompt id '{}'. Use promptAliases to disambiguate",
                                prompt_id
                            ));
                        }

                        let title = prompt
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        let description = prompt
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        let arguments = prompt
                            .get("arguments")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();

                        info!(%server_id, prompt_name = %name, "registered upstream prompt");
                        prompts.insert(
                            prompt_id.clone(),
                            PromptMeta {
                                server: server_id.clone(),
                                name: name.to_string(),
                                title,
                                description,
                                arguments,
                                tags: vec![server_id.clone()],
                            },
                        );
                        event_store.record("prompt", &prompt_id, "added");
                    }
                }
            }
        }

        let (tx, mut rx) = mpsc::channel::<ServerMsg>(32);
        let per_server_timeout = Duration::from_millis(tool_timeout_ms);
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    ServerMsg::CallTool { name, params, reply } => {
                        let req = CallToolRequestParams {
                            name: name.into(),
                            arguments: params.as_object().cloned(),
                            meta: None,
                            task: None,
                        };

                        let result = timeout(per_server_timeout, mcp_client.call_tool(req)).await;
                        let res = match result {
                            Ok(Ok(call_res)) => {
                                Ok(serde_json::to_value(call_res).unwrap_or(Value::Null))
                            }
                            Ok(Err(err)) => Err(UpstreamCallError::Upstream(err.to_string())),
                            Err(_) => Err(UpstreamCallError::Timeout),
                        };
                        let _ = reply.send(res);
                    }
                    ServerMsg::ReadResource { uri, reply } => {
                        let req = ReadResourceRequestParams { meta: None, uri };
                        let result = timeout(per_server_timeout, mcp_client.read_resource(req)).await;
                        let res = match result {
                            Ok(Ok(read_res)) => {
                                Ok(serde_json::to_value(read_res).unwrap_or(Value::Null))
                            }
                            Ok(Err(err)) => Err(UpstreamCallError::Upstream(err.to_string())),
                            Err(_) => Err(UpstreamCallError::Timeout),
                        };
                        let _ = reply.send(res);
                    }
                    ServerMsg::GetPrompt {
                        name,
                        arguments,
                        reply,
                    } => {
                        let req = GetPromptRequestParams {
                            meta: None,
                            name,
                            arguments,
                        };
                        let result = timeout(per_server_timeout, mcp_client.get_prompt(req)).await;
                        let res = match result {
                            Ok(Ok(prompt_res)) => {
                                Ok(serde_json::to_value(prompt_res).unwrap_or(Value::Null))
                            }
                            Ok(Err(err)) => Err(UpstreamCallError::Upstream(err.to_string())),
                            Err(_) => Err(UpstreamCallError::Timeout),
                        };
                        let _ = reply.send(res);
                    }
                }
            }
        });

        server_channels.insert(server_id, tx);
    }

    let catalog_version = compute_catalog_version(&capabilities, &resources, &prompts);
    let search_engine = Arc::new(crate::search::HybridSearchEngine::new());

    Ok(AppState {
        servers: Arc::new(server_channels),
        capabilities: Arc::new(capabilities),
        resources: Arc::new(resources),
        prompts: Arc::new(prompts),
        tool_timeout_ms,
        policy,
        search_engine,
        catalog_version,
        event_store,
        idempotency_store: Arc::new(crate::idempotency::IdempotencyStore::default()),
        operation_registry: crate::operations::OperationRegistry::new(),
    })
}

pub async fn run_daemon(port: u16, config: McpConfig) -> Result<()> {
    let app_state = initialize_state(config).await?;
    let app = Router::new()
        .route("/v1/capabilities", get(http_v1::handle_list_capabilities))
        .route(
            "/v1/capabilities/search",
            post(http_v1::handle_search_capabilities),
        )
        .route(
            "/v1/capabilities/:id",
            get(http_v1::handle_describe_capability),
        )
        .route("/v1/resources", get(http_v1::handle_list_resources))
        .route("/v1/resources/read", post(http_v1::handle_read_resource))
        .route("/v1/prompts", get(http_v1::handle_list_prompts))
        .route("/v1/prompts/get", post(http_v1::handle_get_prompt))
        .route("/v1/tools/call", post(http_v1::handle_call_capability))
        .route("/v1/catalog/events", get(http_v1::handle_catalog_events))
        .route(
            "/v1/operations/:id/cancel",
            post(http_v1::handle_cancel_operation),
        )
        .with_state(app_state);

    info!(port, "all upstream servers connected; daemon listening");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{wildcard_match, Policy};

    #[test]
    fn wildcard_prefix_match_works() {
        assert!(wildcard_match("db.*", "db.query"));
        assert!(!wildcard_match("db.*", "fs.read"));
    }

    #[test]
    fn policy_deny_overrides_allow() {
        let policy = Policy {
            allow: vec!["db.*".to_string()],
            deny: vec!["db.delete".to_string()],
            redact_keys: vec![],
        };

        assert!(policy.allows("db.query"));
        assert!(!policy.allows("db.delete"));
    }

    #[test]
    fn policy_allow_list_is_enforced() {
        let policy = Policy {
            allow: vec!["fs.read".to_string()],
            deny: vec![],
            redact_keys: vec![],
        };

        assert!(policy.allows("fs.read"));
        assert!(!policy.allows("fs.write"));
    }
}
