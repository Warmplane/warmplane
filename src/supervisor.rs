// Rust guideline compliant 2026-11-18

//! Upstream process supervisor and self-healing actor manager (`M-CANONICAL-DOCS`).

use anyhow::{anyhow, Context, Result};
use rmcp::{
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    transport::{
        streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
        TokioChildProcess,
    },
    ClientHandler, ServiceExt,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    process::Command,
    sync::{mpsc, Notify, RwLock},
    time::timeout,
};
use tracing::{error, info, warn};

use crate::{
    config::{AliasTarget, AuthConfig, ServerConfig},
    daemon::{
        state::AppState,
        transport::build_http_headers,
        types::{CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg, UpstreamCallError},
    },
    tool_pins::{PinSession, REFRESH_INTERVAL},
};

macro_rules! handle_supervisor_msg {
    ($mcp_client:expr, $msg:expr, $tool_timeout:expr, $state:expr, $server:expr, $session:expr) => {
        match $msg {
            ServerMsg::CallTool {
                name,
                tool_definition,
                params,
                input_responses,
                request_state,
                reply,
            } => {
                if $state
                    .tool_pins
                    .verify_dispatch($server, $session, &name, tool_definition.as_deref())
                    .is_err()
                {
                    let _ = reply.send(Err(UpstreamCallError::ToolDefinitionChanged));
                    continue;
                }
                let mut req = CallToolRequestParams::new(name);
                if let Some(obj) = params.as_object().cloned() {
                    req = req.with_arguments(obj);
                }
                if let Some(responses) = input_responses {
                    req = req.with_input_responses(responses);
                }
                if let Some(req_st) = request_state {
                    req = req.with_request_state(req_st);
                }

                let result = timeout($tool_timeout, $mcp_client.call_tool(req)).await;
                let (res, is_alive) = match result {
                    Ok(Ok(call_res)) => (
                        Ok(serde_json::to_value(call_res).unwrap_or(Value::Null)),
                        true,
                    ),
                    Ok(Err(err)) => {
                        let err_str = err.to_string();
                        let dead = is_connection_closed_error(&err_str);
                        (Err(UpstreamCallError::Upstream(err_str)), !dead)
                    }
                    Err(_) => (Err(UpstreamCallError::Timeout), true),
                };
                let _ = reply.send(res);
                is_alive
            }
            ServerMsg::ReadResource {
                uri,
                input_responses,
                request_state,
                reply,
            } => {
                let mut req = ReadResourceRequestParams::new(uri);
                if let Some(responses) = input_responses {
                    req = req.with_input_responses(responses);
                }
                if let Some(req_st) = request_state {
                    req = req.with_request_state(req_st);
                }

                let result = timeout($tool_timeout, $mcp_client.read_resource(req)).await;
                let (res, is_alive) = match result {
                    Ok(Ok(read_res)) => (
                        Ok(serde_json::to_value(read_res).unwrap_or(Value::Null)),
                        true,
                    ),
                    Ok(Err(err)) => {
                        let err_str = err.to_string();
                        let dead = is_connection_closed_error(&err_str);
                        (Err(UpstreamCallError::Upstream(err_str)), !dead)
                    }
                    Err(_) => (Err(UpstreamCallError::Timeout), true),
                };
                let _ = reply.send(res);
                is_alive
            }
            ServerMsg::GetPrompt {
                name,
                arguments,
                input_responses,
                request_state,
                reply,
            } => {
                let mut req = GetPromptRequestParams::new(name);
                if let Some(args) = arguments {
                    req = req.with_arguments(args);
                }
                if let Some(responses) = input_responses {
                    req = req.with_input_responses(responses);
                }
                if let Some(req_st) = request_state {
                    req = req.with_request_state(req_st);
                }

                let result = timeout($tool_timeout, $mcp_client.get_prompt(req)).await;
                let (res, is_alive) = match result {
                    Ok(Ok(prompt_res)) => (
                        Ok(serde_json::to_value(prompt_res).unwrap_or(Value::Null)),
                        true,
                    ),
                    Ok(Err(err)) => {
                        let err_str = err.to_string();
                        let dead = is_connection_closed_error(&err_str);
                        (Err(UpstreamCallError::Upstream(err_str)), !dead)
                    }
                    Err(_) => (Err(UpstreamCallError::Timeout), true),
                };
                let _ = reply.send(res);
                is_alive
            }
        }
    };
}

/// Derives a compact, LLM-friendly call signature from JSON Schema input properties.
///
/// # Example
/// Generates `tool_name(req1, req2, [opt1], [opt2])`.
pub fn derive_tool_signature(capability_id: &str, input_schema: &Value) -> String {
    let required_set: std::collections::HashSet<String> = input_schema
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut required_args = Vec::new();
    let mut optional_args = Vec::new();

    if let Some(props) = input_schema.get("properties").and_then(|v| v.as_object()) {
        for (prop_name, prop_val) in props {
            let is_nullable = prop_val
                .get("nullable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if required_set.contains(prop_name) && !is_nullable {
                required_args.push(prop_name.clone());
            } else {
                optional_args.push(format!("[{}]", prop_name));
            }
        }
    }

    let mut all_params = required_args;
    all_params.extend(optional_args);

    if all_params.is_empty() {
        format!("{}()", capability_id)
    } else {
        format!("{}({})", capability_id, all_params.join(", "))
    }
}

fn capability_metadata(
    tools: &[rmcp::model::Tool],
    server_id: &str,
    aliases: &HashMap<String, AliasTarget>,
) -> Vec<(String, CapabilityMeta)> {
    tools
        .iter()
        .map(|tool| {
            let source_id = format!("{server_id}.{}", tool.name);
            let direct_alias = aliases
                .iter()
                .find(|(_, config)| config.target() == source_id);
            let (capability_id, overrides) = match direct_alias {
                Some((alias, config)) => (alias.clone(), Some(config)),
                None => aliases
                    .get(&source_id)
                    .map(|config| (config.target().to_owned(), Some(config)))
                    .unwrap_or_else(|| (source_id.clone(), None)),
            };
            let summary = overrides
                .and_then(AliasTarget::summary)
                .or(tool.description.as_deref())
                .unwrap_or("No summary available")
                .to_owned();
            let description = overrides
                .and_then(AliasTarget::description)
                .map(str::to_owned)
                .unwrap_or_else(|| summary.clone());
            let input_schema = Value::Object(tool.input_schema.as_ref().clone());
            let signature = Some(derive_tool_signature(&capability_id, &input_schema));
            (
                capability_id,
                CapabilityMeta {
                    server: server_id.to_owned(),
                    tool: tool.name.to_string(),
                    summary,
                    description,
                    signature,
                    input_schema,
                    tags: vec![server_id.to_owned()],
                    examples: Vec::new(),
                    reversibility: direct_alias
                        .and_then(|(_, config)| config.reversibility())
                        .cloned(),
                },
            )
        })
        .collect()
}

macro_rules! discover_supervisor_items {
    ($mcp_client:expr, $server_id:expr, $new_capabilities:expr, $resource_aliases:expr, $prompt_aliases:expr) => {{
        let discovery_timeout = std::time::Duration::from_millis(3000);
        let new_capabilities = $new_capabilities;

        let mut new_resources = Vec::new();
        match tokio::time::timeout(
            discovery_timeout,
            $mcp_client.list_resources(Default::default()),
        )
        .await
        {
            Ok(Ok(listed_resources)) => {
                if let Ok(resources_json) = serde_json::to_value(&listed_resources) {
                    if let Some(resource_array) =
                        resources_json.get("resources").and_then(|r| r.as_array())
                    {
                        for resource in resource_array {
                            let Some(uri) = resource.get("uri").and_then(|v| v.as_str()) else {
                                continue;
                            };
                            let source_id = format!("{}.{}", $server_id, uri);
                            let (resource_id, alias_summary, alias_desc) = $resource_aliases
                                .iter()
                                .find_map(|(alias, target_cfg)| {
                                    if target_cfg.target() == source_id {
                                        Some((
                                            alias.clone(),
                                            target_cfg.summary().map(ToString::to_string),
                                            target_cfg.description().map(ToString::to_string),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .or_else(|| {
                                    $resource_aliases.get(&source_id).map(|target_cfg| {
                                        (
                                            target_cfg.target().to_string(),
                                            target_cfg.summary().map(ToString::to_string),
                                            target_cfg.description().map(ToString::to_string),
                                        )
                                    })
                                })
                                .unwrap_or((source_id, None, None));

                            let default_name = resource
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or(uri)
                                .to_string();
                            let name = alias_summary.unwrap_or(default_name);
                            let description = alias_desc.or_else(|| {
                                resource
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .map(ToString::to_string)
                            });
                            let mime_type = resource
                                .get("mime_type")
                                .and_then(|v| v.as_str())
                                .map(ToString::to_string);

                            new_resources.push((
                                resource_id,
                                ResourceMeta {
                                    server: $server_id.to_string(),
                                    uri: uri.to_string(),
                                    name,
                                    description,
                                    mime_type,
                                    tags: vec![$server_id.to_string()],
                                },
                            ));
                        }
                    }
                }
            }
            Ok(Err(err)) => {
                tracing::warn!(server_id = %$server_id, error = %err, "supervisor failed to list resources during discovery");
            }
            Err(_) => {
                tracing::warn!(server_id = %$server_id, timeout_ms = 3000, "supervisor timed out listing resources during discovery");
            }
        }

        let mut new_prompts = Vec::new();
        match tokio::time::timeout(
            discovery_timeout,
            $mcp_client.list_prompts(Default::default()),
        )
        .await
        {
            Ok(Ok(listed_prompts)) => {
                if let Ok(prompts_json) = serde_json::to_value(&listed_prompts) {
                    if let Some(prompt_array) = prompts_json.get("prompts").and_then(|p| p.as_array()) {
                        for prompt in prompt_array {
                            let Some(name) = prompt.get("name").and_then(|v| v.as_str()) else {
                                continue;
                            };

                            let source_id = format!("{}.{}", $server_id, name);
                            let (prompt_id, alias_summary, alias_desc) = $prompt_aliases
                                .iter()
                                .find_map(|(alias, target_cfg)| {
                                    if target_cfg.target() == source_id {
                                        Some((
                                            alias.clone(),
                                            target_cfg.summary().map(ToString::to_string),
                                            target_cfg.description().map(ToString::to_string),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .or_else(|| {
                                    $prompt_aliases.get(&source_id).map(|target_cfg| {
                                        (
                                            target_cfg.target().to_string(),
                                            target_cfg.summary().map(ToString::to_string),
                                            target_cfg.description().map(ToString::to_string),
                                        )
                                    })
                                })
                                .unwrap_or((source_id, None, None));

                            let title = alias_summary.or_else(|| {
                                prompt
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .map(ToString::to_string)
                            });
                            let description = alias_desc.or_else(|| {
                                prompt
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .map(ToString::to_string)
                            });
                            let arguments = prompt
                                .get("arguments")
                                .and_then(|v| v.as_array())
                                .cloned()
                                .unwrap_or_default();

                            new_prompts.push((
                                prompt_id,
                                PromptMeta {
                                    server: $server_id.to_string(),
                                    name: name.to_string(),
                                    title,
                                    description,
                                    arguments,
                                    tags: vec![$server_id.to_string()],
                                },
                            ));
                        }
                    }
                }
            }
            Ok(Err(err)) => {
                tracing::warn!(server_id = %$server_id, error = %err, "supervisor failed to list prompts during discovery");
            }
            Err(_) => {
                tracing::warn!(server_id = %$server_id, timeout_ms = 3000, "supervisor timed out listing prompts during discovery");
            }
        }

        (new_capabilities, new_resources, new_prompts)
    }};
}

/// Helper function to establish a streamable HTTP connection to a remote MCP server.
///
/// # Arguments
/// * `state` - Shared application state reference.
/// * `server_id` - Server identifier string.
/// * `srv_cfg` - Server configuration parameters.
///
/// # Returns
/// Active rmcp Client instance.
///
/// # Errors
/// Returns an error if transport creation or handshake fails.
async fn connect_http_mcp_client(
    state: &AppState,
    server_id: &str,
    srv_cfg: &ServerConfig,
    handler: SupervisorHandler,
) -> Result<SupervisorClient> {
    let url = srv_cfg
        .url
        .as_ref()
        .context("URL required for HTTP server")?;

    let mut target_url = url.clone();

    if let Some(AuthConfig::Oauth2 {
        client_id,
        authorization_server_url,
        scopes,
        client_metadata_url,
    }) = &srv_cfg.auth
    {
        let proxy_port = state
            .oauth_proxy_port
            .ok_or_else(|| anyhow!("OAuth proxy server not running"))?;

        let discovery =
            crate::oauth2::discover_auth_server(url, Some(authorization_server_url)).await?;

        let client_state = crate::oauth2::OAuth2ClientState {
            server_id: server_id.to_string(),
            client_id: client_id.clone(),
            _authorization_server_url: authorization_server_url.clone(),
            scopes: Arc::new(RwLock::new(scopes.iter().cloned().collect())),
            token_state: Arc::new(RwLock::new(None)),
            discovery,
            client_metadata_url: client_metadata_url.clone(),
            remote_base_url: url.clone(),
        };

        let token = if let Some(saved) = state.oauth_registry.get_saved_token(server_id).await {
            saved
        } else {
            crate::oauth2::run_oauth2_flow(&client_state, &state.oauth_registry, proxy_port).await?
        };
        {
            let mut guard = client_state.token_state.write().await;
            *guard = Some(token);
        }

        {
            let mut clients = state.oauth_registry.clients.write().await;
            clients.insert(server_id.to_string(), client_state);
        }

        target_url = format!("http://127.0.0.1:{}/proxy/{}/", proxy_port, server_id);
    }

    let headers = build_http_headers(server_id, srv_cfg)?;
    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .with_context(|| format!("Failed to build HTTP client for {}", server_id))?;

    let mut transport_config = StreamableHttpClientTransportConfig::with_uri(target_url);
    if let Some(allow_stateless) = srv_cfg.allow_stateless {
        transport_config.allow_stateless = allow_stateless;
    }
    let transport = StreamableHttpClientTransport::with_client(http_client, transport_config);
    let mcp_client = handler.serve(transport).await.with_context(|| {
        format!(
            "Failed to negotiate streamable HTTP MCP connection for {}",
            server_id
        )
    })?;

    Ok(mcp_client)
}

/// Spawns an auto-restarting supervisor for an upstream stdio child process.
///
/// # Arguments
/// * `state` - Shared application state reference.
/// * `server_id` - Server identifier string.
/// * `srv_cfg` - Server configuration and environment parameters.
/// * `capability_aliases` - Aliases mapping for capabilities.
/// * `resource_aliases` - Aliases mapping for resources.
/// * `prompt_aliases` - Aliases mapping for prompts.
///
/// # Returns
/// Discovered initial capabilities, resources, and prompts, plus the server sender mailbox.
pub async fn spawn_supervised_stdio_server(
    state: &AppState,
    server_id: &str,
    srv_cfg: &ServerConfig,
    capability_aliases: &HashMap<String, AliasTarget>,
    resource_aliases: &HashMap<String, AliasTarget>,
    prompt_aliases: &HashMap<String, AliasTarget>,
) -> Result<(
    Vec<(String, CapabilityMeta)>,
    Vec<(String, ResourceMeta)>,
    Vec<(String, PromptMeta)>,
    mpsc::Sender<ServerMsg>,
)> {
    spawn_upstream(
        state,
        server_id,
        srv_cfg,
        capability_aliases,
        resource_aliases,
        prompt_aliases,
        SupervisorTransport::Stdio,
    )
    .await
}

/// Spawns an auto-restarting supervisor for an upstream Streamable HTTP / SSE MCP server.
///
/// # Arguments
/// * `state` - Shared application state reference.
/// * `server_id` - Server identifier string.
/// * `srv_cfg` - Server configuration and authentication parameters.
/// * `capability_aliases` - Aliases mapping for capabilities.
/// * `resource_aliases` - Aliases mapping for resources.
/// * `prompt_aliases` - Aliases mapping for prompts.
///
/// # Returns
/// Discovered initial capabilities, resources, and prompts, plus the server sender mailbox.
pub async fn spawn_supervised_http_server(
    state: &AppState,
    server_id: &str,
    srv_cfg: &ServerConfig,
    capability_aliases: &HashMap<String, AliasTarget>,
    resource_aliases: &HashMap<String, AliasTarget>,
    prompt_aliases: &HashMap<String, AliasTarget>,
) -> Result<(
    Vec<(String, CapabilityMeta)>,
    Vec<(String, ResourceMeta)>,
    Vec<(String, PromptMeta)>,
    mpsc::Sender<ServerMsg>,
)> {
    spawn_upstream(
        state,
        server_id,
        srv_cfg,
        capability_aliases,
        resource_aliases,
        prompt_aliases,
        SupervisorTransport::Http,
    )
    .await
}

/// Determines whether an error string represents an unrecoverable connection closure, transport failure,
/// or dropped process that necessitates supervisor restart/reconnection.
fn is_connection_closed_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("channel closed")
        || lower.contains("broken pipe")
        || lower.contains("connection reset")
        || lower.contains("connection abort")
        || lower.contains("transport closed")
        || lower.contains("child process exited")
        || lower.contains("process terminated")
        || lower.contains("io error")
        || lower.contains("unexpected eof")
        || lower.contains("connection refused")
        || lower.contains("connect error")
        || lower.contains("error sending request")
        || lower.contains("stream closed")
        || lower.contains("client disconnected")
        || lower.contains("status code: 502")
        || lower.contains("status code: 503")
        || lower.contains("status code: 504")
        || lower.contains("hyper::error")
        || lower.contains("reqwest::error")
}

fn calculate_backoff_ms(restart_count: u32) -> u64 {
    let base = 500u64;
    let factor = 2u64.saturating_pow(restart_count.saturating_sub(1));
    (base.saturating_mul(factor)).min(30_000)
}

async fn reconcile_restarted_catalog(
    state: &AppState,
    server_id: &str,
    new_capabilities: Vec<(String, CapabilityMeta)>,
    new_resources: Vec<(String, ResourceMeta)>,
    new_prompts: Vec<(String, PromptMeta)>,
) {
    {
        let new_cap_map: HashMap<String, CapabilityMeta> = new_capabilities.into_iter().collect();
        let mut caps_guard = state.capabilities.write().await;
        let stale_keys: Vec<String> = caps_guard
            .iter()
            .filter(|(id, meta)| meta.server == server_id && !new_cap_map.contains_key(*id))
            .map(|(id, _)| id.clone())
            .collect();
        for k in stale_keys {
            caps_guard.remove(&k);
            state.event_store.record("capability", &k, "removed");
        }
        for (id, meta) in new_cap_map {
            caps_guard.insert(id.clone(), meta);
            state.event_store.record("capability", &id, "added");
        }
    }
    {
        let new_res_map: HashMap<String, ResourceMeta> = new_resources.into_iter().collect();
        let mut res_guard = state.resources.write().await;
        let stale_keys: Vec<String> = res_guard
            .iter()
            .filter(|(id, meta)| meta.server == server_id && !new_res_map.contains_key(*id))
            .map(|(id, _)| id.clone())
            .collect();
        for k in stale_keys {
            res_guard.remove(&k);
            state.event_store.record("resource", &k, "removed");
        }
        for (id, meta) in new_res_map {
            res_guard.insert(id.clone(), meta);
            state.event_store.record("resource", &id, "added");
        }
    }
    {
        let new_prompt_map: HashMap<String, PromptMeta> = new_prompts.into_iter().collect();
        let mut prompts_guard = state.prompts.write().await;
        let stale_keys: Vec<String> = prompts_guard
            .iter()
            .filter(|(id, meta)| meta.server == server_id && !new_prompt_map.contains_key(*id))
            .map(|(id, _)| id.clone())
            .collect();
        for k in stale_keys {
            prompts_guard.remove(&k);
            state.event_store.record("prompt", &k, "removed");
        }
        for (id, meta) in new_prompt_map {
            prompts_guard.insert(id.clone(), meta);
            state.event_store.record("prompt", &id, "added");
        }
    }

    state.refresh_server_catalog(server_id).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculation() {
        assert_eq!(calculate_backoff_ms(1), 500);
        assert_eq!(calculate_backoff_ms(2), 1000);
        assert_eq!(calculate_backoff_ms(3), 2000);
        assert_eq!(calculate_backoff_ms(4), 4000);
        assert_eq!(calculate_backoff_ms(10), 30000);
    }

    #[test]
    fn test_closed_error_detection() {
        assert!(is_connection_closed_error("channel closed"));
        assert!(is_connection_closed_error("Broken pipe (os error 32)"));
        assert!(is_connection_closed_error("transport closed"));
        assert!(is_connection_closed_error("connection refused"));
        assert!(is_connection_closed_error(
            "error sending request for url (http://localhost:8000/): connection reset by peer"
        ));
        assert!(is_connection_closed_error(
            "HTTP status code: 502 Bad Gateway"
        ));
        assert!(!is_connection_closed_error("invalid params"));
        assert!(!is_connection_closed_error("method not found"));
    }

    #[test]
    fn test_derive_tool_signature() {
        let schema1 = serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "repo": { "type": "string", "nullable": true },
                "mode": { "type": "string" },
                "top_k": { "type": "integer" }
            },
            "required": ["query"]
        });
        let sig1 = derive_tool_signature("semble.search", &schema1);
        assert_eq!(sig1, "semble.search(query, [mode], [repo], [top_k])");

        let schema_empty = serde_json::json!({
            "type": "object"
        });
        let sig_empty = derive_tool_signature("test.ping", &schema_empty);
        assert_eq!(sig_empty, "test.ping()");
    }
}

#[derive(Clone, Copy)]
enum SupervisorTransport {
    Stdio,
    Http,
}

impl SupervisorTransport {
    fn disconnected_message(self) -> &'static str {
        match self {
            Self::Stdio => "stdio child process exited; supervisor retrying in background",
            Self::Http => "Remote HTTP connection dropped; supervisor retrying in background",
        }
    }
    fn restarting_message(self) -> &'static str {
        match self {
            Self::Stdio => "Server process is restarting",
            Self::Http => "Remote HTTP server is reconnecting",
        }
    }
}

type SupervisorClient = rmcp::service::RunningService<rmcp::RoleClient, SupervisorHandler>;
type SupervisorAliases = (
    HashMap<String, AliasTarget>,
    HashMap<String, AliasTarget>,
    HashMap<String, AliasTarget>,
);

#[derive(Clone)]
struct PinnedUpstream {
    state: AppState,
    server: Arc<str>,
    session: PinSession,
    changes: Arc<Notify>,
}

impl PinnedUpstream {
    fn invalidate(&self) {
        if self.state.tool_pins.invalidate(&self.server, self.session) {
            self.state.notify_tools_changed();
            self.state
                .notify_tool_quarantine(&self.server, "observation_unavailable");
        }
    }

    fn changed(&self) {
        self.invalidate();
        self.changes.notify_one();
    }

    async fn discover(
        &self,
        client: &rmcp::service::Peer<rmcp::RoleClient>,
    ) -> Option<PinnedDiscovery> {
        let epoch = self.state.tool_pins.epoch(&self.server, self.session)?;
        match timeout(Duration::from_secs(3), client.list_all_tools()).await {
            Ok(Ok(tools)) => Some(PinnedDiscovery { epoch, tools }),
            _ => {
                self.invalidate();
                warn!(server_id = %self.server, "pinned tool discovery failed");
                None
            }
        }
    }

    fn publish(&self, discovered: &PinnedDiscovery) -> bool {
        let changed = self.state.tool_pins.observe(
            &self.server,
            self.session,
            discovered.epoch,
            &discovered.tools,
        );
        if changed == Some(true) {
            self.state.notify_tools_changed();
            if self.state.tool_pins.is_quarantined(&self.server) {
                self.state
                    .notify_tool_quarantine(&self.server, "definition_requires_review");
            }
        }
        changed.is_some()
    }

    fn start_monitor(
        &self,
        client: &SupervisorClient,
        aliases: HashMap<String, AliasTarget>,
    ) -> PinMonitor {
        let stop = tokio_util::sync::CancellationToken::new();
        let listener = self.clone();
        let peer = client.peer().clone();
        let listener_stop = stop.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = listener_stop.cancelled() => {}
                _ = listen_for_tool_changes(&peer, &listener) => {}
            }
        });
        let pinned = self.clone();
        let peer = client.peer().clone();
        let refresh_stop = stop.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = refresh_stop.cancelled() => {}
                _ = pinned.refresh_loop(&peer, &aliases) => {}
            }
        });
        PinMonitor {
            pinned: self.clone(),
            stop,
        }
    }

    async fn refresh_loop(
        &self,
        peer: &rmcp::service::Peer<rmcp::RoleClient>,
        aliases: &HashMap<String, AliasTarget>,
    ) {
        let mut ticker = tokio::time::interval_at(
            tokio::time::Instant::now() + REFRESH_INTERVAL,
            REFRESH_INTERVAL,
        );
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut health = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = self.state.shutdown_token.cancelled() => return,
                _ = self.changes.notified() => self.refresh(peer, aliases).await,
                _ = ticker.tick() => self.refresh(peer, aliases).await,
                _ = health.tick() => {
                    if peer.is_transport_closed() || self.state.tool_pins.epoch(&self.server, self.session).is_none() {
                        self.invalidate();
                        return;
                    }
                }
            }
        }
    }

    async fn refresh(
        &self,
        peer: &rmcp::service::Peer<rmcp::RoleClient>,
        aliases: &HashMap<String, AliasTarget>,
    ) {
        let Some(discovery) = self.discover(peer).await else {
            return;
        };
        let fresh = capability_metadata(&discovery.tools, &self.server, aliases);
        {
            let mut capabilities = self.state.capabilities.write().await;
            if !self.publish(&discovery) {
                return;
            }
            capabilities.retain(|_, capability| capability.server != self.server.as_ref());
            capabilities.extend(fresh);
        }
        self.state.refresh_server_catalog(&self.server).await;
    }
}

struct PinMonitor {
    pinned: PinnedUpstream,
    stop: tokio_util::sync::CancellationToken,
}

impl Drop for PinMonitor {
    fn drop(&mut self) {
        self.stop.cancel();
        self.pinned.invalidate();
    }
}

async fn listen_for_tool_changes(
    peer: &rmcp::service::Peer<rmcp::RoleClient>,
    pinned: &PinnedUpstream,
) {
    use rmcp::model::{ProtocolVersion, SubscriptionFilterBuilder};

    if !peer.peer_info().is_some_and(|info| {
        info.protocol_version == ProtocolVersion::V_2026_07_28
            && info
                .capabilities
                .tools
                .as_ref()
                .is_some_and(|tools| tools.list_changed == Some(true))
    }) {
        return;
    }
    let filter = SubscriptionFilterBuilder::default()
        .tools_list_changed()
        .build();
    if let Ok(Ok(mut subscription)) = timeout(Duration::from_secs(3), peer.listen(filter)).await {
        while let Ok(Some(_)) = subscription.next().await {
            pinned.changed();
        }
        pinned.changed();
    }
}

#[derive(Clone, Default)]
struct SupervisorHandler(Option<PinnedUpstream>);

impl ClientHandler for SupervisorHandler {
    async fn on_tool_list_changed(&self, _: rmcp::service::NotificationContext<rmcp::RoleClient>) {
        if let Some(pinned) = &self.0 {
            pinned.changed();
        }
    }
}

struct PinnedDiscovery {
    epoch: u64,
    tools: Vec<rmcp::model::Tool>,
}

type InitialUpstream = (
    Vec<(String, CapabilityMeta)>,
    Vec<(String, ResourceMeta)>,
    Vec<(String, PromptMeta)>,
    mpsc::Sender<ServerMsg>,
);

async fn spawn_upstream(
    state: &AppState,
    server_id: &str,
    config: &ServerConfig,
    capability_aliases: &HashMap<String, AliasTarget>,
    resource_aliases: &HashMap<String, AliasTarget>,
    prompt_aliases: &HashMap<String, AliasTarget>,
    transport: SupervisorTransport,
) -> Result<InitialUpstream> {
    let pinned = state
        .tool_pins
        .begin_session(server_id, config.require_tool_pins)
        .map(|session| PinnedUpstream {
            state: state.clone(),
            server: Arc::from(server_id),
            session,
            changes: Arc::new(Notify::new()),
        });
    let handler = SupervisorHandler(pinned.clone());
    let connection = timeout(
        Duration::from_secs(5),
        connect_supervised_client(state, server_id, config, transport, handler),
    )
    .await;
    let aliases = (
        capability_aliases.clone(),
        resource_aliases.clone(),
        prompt_aliases.clone(),
    );
    let (client, capabilities, resources, prompts) = match connection {
        Ok(Ok(client)) => {
            let (caps, res, prompts) =
                discover_connection(&client, server_id, &aliases, pinned.as_ref()).await;
            (Some(client), caps, res, prompts)
        }
        _ => {
            warn!(%server_id, "upstream handshake failed; supervisor will retry in background");
            (None, Vec::new(), Vec::new(), Vec::new())
        }
    };
    let tx = spawn_supervisor(state, server_id, config, aliases, client, transport, pinned);
    Ok((capabilities, resources, prompts, tx))
}

async fn discover_connection(
    client: &SupervisorClient,
    server: &str,
    aliases: &SupervisorAliases,
    pinned: Option<&PinnedUpstream>,
) -> (
    Vec<(String, CapabilityMeta)>,
    Vec<(String, ResourceMeta)>,
    Vec<(String, PromptMeta)>,
) {
    let capabilities = match pinned {
        Some(pinned) => pinned
            .discover(client)
            .await
            .filter(|discovered| pinned.publish(discovered))
            .map_or_else(Vec::new, |discovered| {
                capability_metadata(&discovered.tools, server, &aliases.0)
            }),
        None => match timeout(
            Duration::from_secs(3),
            client.list_tools(Default::default()),
        )
        .await
        {
            Ok(Ok(list)) => capability_metadata(&list.tools, server, &aliases.0),
            _ => {
                warn!(server_id = %server, "failed to list upstream tools");
                Vec::new()
            }
        },
    };
    discover_supervisor_items!(client, server, capabilities, &aliases.1, &aliases.2)
}

async fn connect_supervised_client(
    state: &AppState,
    server_id: &str,
    config: &ServerConfig,
    transport: SupervisorTransport,
    handler: SupervisorHandler,
) -> Result<SupervisorClient> {
    match transport {
        SupervisorTransport::Http => {
            connect_http_mcp_client(state, server_id, config, handler).await
        }
        SupervisorTransport::Stdio => {
            let command = config
                .command
                .as_ref()
                .context("Command required for stdio server")?;
            let mut environment = HashMap::new();
            for (key, value) in &config.env {
                let resolved = crate::vault::resolve_secret_value(value).unwrap_or_else(|error| {
                    warn!(server_id, key, %error, "Failed to resolve secret from vault on restart; using raw string");
                    value.clone()
                });
                environment.insert(key.clone(), resolved);
            }
            let mut process = Command::new(command);
            process
                .args(&config.args)
                .envs(environment)
                .stderr(std::process::Stdio::inherit())
                .kill_on_drop(true);
            let process = TokioChildProcess::new(process)?;
            Ok(handler.serve(process).await?)
        }
    }
}

fn spawn_supervisor(
    state: &AppState,
    server_id: &str,
    srv_cfg: &ServerConfig,
    aliases: SupervisorAliases,
    initial_client_opt: Option<SupervisorClient>,
    transport: SupervisorTransport,
    pinned: Option<PinnedUpstream>,
) -> mpsc::Sender<ServerMsg> {
    let handshake_timeout = Duration::from_millis(5000);
    let (tx, mut rx) = mpsc::channel::<ServerMsg>(32);

    let resilience_cfg = srv_cfg.resilience.clone().unwrap_or_default();
    let server_id_owned = server_id.to_string();
    let srv_cfg_owned = srv_cfg.clone();
    let tool_timeout = Duration::from_millis(state.tool_timeout_ms);
    let state_clone = state.clone();
    let shutdown_token = state.shutdown_token.clone();

    tokio::spawn(async move {
        let mut client_opt = initial_client_opt;
        let mut monitor = pinned
            .as_ref()
            .zip(client_opt.as_ref())
            .map(|(pinned, client)| pinned.start_monitor(client, aliases.0.clone()));
        let mut restart_count: u32 = 0;
        let mut last_restart_time: Option<Instant> = None;
        let mut health = tokio::time::interval(Duration::from_secs(1));
        const STABLE_UPTIME_WINDOW: Duration = Duration::from_secs(60);

        loop {
            let msg = tokio::select! {
                _ = shutdown_token.cancelled() => {
                    info!(
                        server_id = %server_id_owned,
                        "shutdown token cancelled; terminating supervisor loop"
                    );
                    break;
                }
                opt = rx.recv() => match opt {
                    Some(m) => Some(m),
                    None => break,
                },
                _ = health.tick(), if pinned.is_some() => None,
            };

            if pinned.is_some()
                && client_opt
                    .as_ref()
                    .is_some_and(|client| client.is_transport_closed())
            {
                monitor.take();
                client_opt = None;
                mark_disconnected(&state_clone, &server_id_owned, transport).await;
            }
            let is_alive = match (client_opt.as_ref(), msg) {
                (Some(client), Some(msg)) => handle_supervisor_msg!(
                    client,
                    msg,
                    tool_timeout,
                    state_clone,
                    &server_id_owned,
                    pinned.as_ref().map(|pinned| pinned.session)
                ),
                (Some(_), None) => true,
                (None, msg) => {
                    if let Some(msg) = msg {
                        reject_unavailable_message(msg, transport);
                    }
                    false
                }
            };
            if !is_alive && client_opt.take().is_some() {
                monitor.take();
                mark_disconnected(&state_clone, &server_id_owned, transport).await;
            }

            // Attempt reconnection if client dropped or initial connection was deferred
            if !is_alive
                && client_opt.is_none()
                && resilience_cfg.auto_restart
                && !shutdown_token.is_cancelled()
            {
                let now = Instant::now();
                if let Some(last_time) = last_restart_time {
                    if now.duration_since(last_time) >= STABLE_UPTIME_WINDOW {
                        restart_count = 0;
                    }
                }

                if restart_count < resilience_cfg.max_restarts {
                    restart_count += 1;
                    last_restart_time = Some(now);
                    let backoff_ms = calculate_backoff_ms(restart_count);
                    warn!(
                        server_id = %server_id_owned,
                        restart_attempt = restart_count,
                        backoff_ms = backoff_ms,
                        "server disconnected; supervisor scheduling reconnect"
                    );

                    tokio::select! {
                        _ = shutdown_token.cancelled() => {
                            info!(
                                server_id = %server_id_owned,
                                "shutdown requested during reconnect backoff; cancelling supervisor"
                            );
                            break;
                        }
                        _ = tokio::time::sleep(Duration::from_millis(backoff_ms)) => {}
                    }

                    let reconnect_attempt = timeout(
                        handshake_timeout,
                        connect_supervised_client(
                            &state_clone,
                            &server_id_owned,
                            &srv_cfg_owned,
                            transport,
                            SupervisorHandler(pinned.clone()),
                        ),
                    )
                    .await;

                    match reconnect_attempt {
                        Ok(Ok(new_client)) => {
                            info!(
                                server_id = %server_id_owned,
                                restart_attempt = restart_count,
                                "supervisor successfully reconnected MCP server"
                            );

                            // Actively reset circuit breaker and restart counter upon successful recovery
                            state_clone.circuit_breakers.reset(&server_id_owned).await;
                            restart_count = 0;

                            let (new_caps, new_res, new_prompts) = discover_connection(
                                &new_client,
                                &server_id_owned,
                                &aliases,
                                pinned.as_ref(),
                            )
                            .await;

                            reconcile_restarted_catalog(
                                &state_clone,
                                &server_id_owned,
                                new_caps,
                                new_res,
                                new_prompts,
                            )
                            .await;

                            {
                                let mut statuses_guard = state_clone.server_statuses.write().await;
                                if let Some(status_val) = statuses_guard.get_mut(&server_id_owned) {
                                    if let Some(obj) = status_val.as_object_mut() {
                                        obj.insert("status".to_string(), json!("connected"));
                                        obj.remove("error");
                                    }
                                }
                            }

                            monitor = pinned
                                .as_ref()
                                .map(|pinned| pinned.start_monitor(&new_client, aliases.0.clone()));
                            client_opt = Some(new_client);
                        }
                        Ok(Err(err)) => {
                            error!(
                                server_id = %server_id_owned,
                                error = %err,
                                "supervisor failed to reconnect MCP server"
                            );
                        }
                        Err(_) => {
                            error!(
                                server_id = %server_id_owned,
                                "supervisor reconnect attempt timed out"
                            );
                        }
                    }
                } else {
                    error!(
                        server_id = %server_id_owned,
                        max_restarts = resilience_cfg.max_restarts,
                        "supervisor exceeded maximum reconnect attempts for server"
                    );
                }
            }
        }
    });

    tx
}

async fn mark_disconnected(state: &AppState, server: &str, transport: SupervisorTransport) {
    let mut statuses = state.server_statuses.write().await;
    if let Some(status) = statuses.get_mut(server).and_then(Value::as_object_mut) {
        status.insert("status".to_string(), json!("degraded"));
        status.insert("error".to_string(), json!(transport.disconnected_message()));
    }
}

fn reject_unavailable_message(message: ServerMsg, transport: SupervisorTransport) {
    let reply = match message {
        ServerMsg::CallTool { reply, .. }
        | ServerMsg::ReadResource { reply, .. }
        | ServerMsg::GetPrompt { reply, .. } => reply,
    };
    let _ = reply.send(Err(UpstreamCallError::Upstream(
        transport.restarting_message().to_owned(),
    )));
}
