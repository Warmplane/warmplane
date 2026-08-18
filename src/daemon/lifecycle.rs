// Rust guideline compliant 2026-08-15

//! Dynamic server mounting, unmounting, and configuration reconciliation lifecycle operations.

use anyhow::{anyhow, Context, Result};
use rmcp::{
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    transport::{
        streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
    },
    ServiceExt,
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::timeout};
use tracing::{info, warn};

use crate::{
    config::{AuthConfig, ServerConfig},
    daemon::{
        policy::Policy,
        state::{compute_catalog_version, AppState},
        transport::{build_http_headers, DEFAULT_MCP_PROTOCOL_VERSION},
        types::{CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg, UpstreamCallError},
    },
};

macro_rules! handle_upstream_msg {
    ($mcp_client:expr, $msg:expr, $tool_timeout:expr) => {
        match $msg {
            ServerMsg::CallTool {
                name,
                params,
                input_responses,
                request_state,
                reply,
            } => {
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
                let res = match result {
                    Ok(Ok(call_res)) => Ok(serde_json::to_value(call_res).unwrap_or(Value::Null)),
                    Ok(Err(err)) => Err(UpstreamCallError::Upstream(err.to_string())),
                    Err(_) => Err(UpstreamCallError::Timeout),
                };
                let _ = reply.send(res);
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
                let res = match result {
                    Ok(Ok(read_res)) => Ok(serde_json::to_value(read_res).unwrap_or(Value::Null)),
                    Ok(Err(err)) => Err(UpstreamCallError::Upstream(err.to_string())),
                    Err(_) => Err(UpstreamCallError::Timeout),
                };
                let _ = reply.send(res);
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
    };
}

macro_rules! discover_upstream_items {
    ($mcp_client:expr, $server_id:expr, $capability_aliases:expr, $resource_aliases:expr, $prompt_aliases:expr) => {{
        let mut new_capabilities = Vec::new();
        if let Ok(tools) = $mcp_client.list_tools(Default::default()).await {
            if let Ok(tools_json) = serde_json::to_value(&tools) {
                if let Some(tools_array) = tools_json.get("tools").and_then(|t| t.as_array()) {
                    for tool in tools_array {
                        if let Some(tool_name) = tool.get("name").and_then(|n| n.as_str()) {
                            let source_id = format!("{}.{}", $server_id, tool_name);
                            let capability_id = $capability_aliases
                                .get(&source_id)
                                .cloned()
                                .unwrap_or(source_id);

                            let summary = tool
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("No summary available")
                                .to_string();
                            let description = summary.clone();
                            let input_schema = tool
                                .get("inputSchema")
                                .cloned()
                                .unwrap_or_else(|| json!({}));

                            new_capabilities.push((
                                capability_id,
                                CapabilityMeta {
                                    server: $server_id.to_string(),
                                    tool: tool_name.to_string(),
                                    summary,
                                    description,
                                    input_schema,
                                    tags: vec![$server_id.to_string()],
                                    examples: vec![],
                                },
                            ));
                        }
                    }
                }
            }
        }

        let mut new_resources = Vec::new();
        if let Ok(listed_resources) = $mcp_client.list_resources(Default::default()).await {
            if let Ok(resources_json) = serde_json::to_value(&listed_resources) {
                if let Some(resource_array) =
                    resources_json.get("resources").and_then(|r| r.as_array())
                {
                    for resource in resource_array {
                        let Some(uri) = resource.get("uri").and_then(|v| v.as_str()) else {
                            continue;
                        };
                        let name = resource
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(uri)
                            .to_string();

                        let source_id = format!("{}.{}", $server_id, uri);
                        let resource_id = $resource_aliases
                            .get(&source_id)
                            .cloned()
                            .unwrap_or(source_id);

                        let description = resource
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        let mime_type = resource
                            .get("mimeType")
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

        let mut new_prompts = Vec::new();
        if let Ok(listed_prompts) = $mcp_client.list_prompts(Default::default()).await {
            if let Ok(prompts_json) = serde_json::to_value(&listed_prompts) {
                if let Some(prompt_array) = prompts_json.get("prompts").and_then(|p| p.as_array()) {
                    for prompt in prompt_array {
                        let Some(name) = prompt.get("name").and_then(|v| v.as_str()) else {
                            continue;
                        };

                        let source_id = format!("{}.{}", $server_id, name);
                        let prompt_id = $prompt_aliases
                            .get(&source_id)
                            .cloned()
                            .unwrap_or(source_id);

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

        (new_capabilities, new_resources, new_prompts)
    }};
}

impl AppState {
    /// Mounts a single upstream server dynamically into `AppState`.
    ///
    /// # Arguments
    /// * `server_id` - Unique server identifier.
    /// * `srv_cfg` - Server configuration and transport settings.
    /// * `capability_aliases` - Map of capability alias overrides.
    /// * `resource_aliases` - Map of resource alias overrides.
    /// * `prompt_aliases` - Map of prompt alias overrides.
    ///
    /// # Errors
    /// Returns an error if transport initialization or discovery fails.
    pub async fn mount_upstream_server(
        &self,
        server_id: &str,
        srv_cfg: &ServerConfig,
        capability_aliases: &HashMap<String, String>,
        resource_aliases: &HashMap<String, String>,
        prompt_aliases: &HashMap<String, String>,
    ) -> Result<()> {
        info!(%server_id, "dynamically mounting upstream server");

        let transport_type = if srv_cfg.command.is_some() {
            "stdio"
        } else {
            "http"
        };

        let (new_capabilities, new_resources, new_prompts, tx) = if srv_cfg.command.is_some() {
            crate::supervisor::spawn_supervised_stdio_server(
                self,
                server_id,
                srv_cfg,
                capability_aliases,
                resource_aliases,
                prompt_aliases,
            )
            .await?
        } else if let Some(url) = &srv_cfg.url {
            let mut target_url = url.clone();

            if let Some(AuthConfig::Oauth2 {
                client_id,
                authorization_server_url,
                scopes,
                client_metadata_url,
            }) = &srv_cfg.auth
            {
                let proxy_port = self
                    .oauth_proxy_port
                    .ok_or_else(|| anyhow!("OAuth proxy server not running"))?;

                let discovery =
                    crate::oauth2::discover_auth_server(url, Some(authorization_server_url))
                        .await?;

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

                let token = if let Some(saved) =
                    self.oauth_registry.get_saved_token(server_id).await
                {
                    saved
                } else {
                    crate::oauth2::run_oauth2_flow(&client_state, &self.oauth_registry, proxy_port)
                        .await?
                };
                {
                    let mut guard = client_state.token_state.write().await;
                    *guard = Some(token);
                }

                {
                    let mut clients = self.oauth_registry.clients.write().await;
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
            let transport =
                StreamableHttpClientTransport::with_client(http_client, transport_config);
            let mcp_client = ().serve(transport).await.with_context(|| {
                format!(
                    "Failed to negotiate streamable HTTP MCP connection for {}",
                    server_id
                )
            })?;

            let (caps, res, prompts) = discover_upstream_items!(
                &mcp_client,
                server_id,
                capability_aliases,
                resource_aliases,
                prompt_aliases
            );

            let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMsg>(32);
            let per_server_timeout = Duration::from_millis(self.tool_timeout_ms);
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    handle_upstream_msg!(&mcp_client, msg, per_server_timeout);
                }
            });

            (caps, res, prompts, tx)
        } else {
            return Err(anyhow!(
                "Server '{}' missing transport selector: set exactly one of 'command' or 'url'",
                server_id
            ));
        };

        let resilience_cfg = srv_cfg.resilience.clone().unwrap_or_default();
        self.circuit_breakers
            .get_or_create(server_id, resilience_cfg)
            .await;

        {
            let mut servers_guard = self.servers.write().await;
            servers_guard.insert(server_id.to_string(), tx);
        }
        {
            let mut configs_guard = self.server_configs.write().await;
            configs_guard.insert(server_id.to_string(), srv_cfg.clone());
        }
        {
            let mut statuses_guard = self.server_statuses.write().await;
            let is_degraded = srv_cfg.command.is_some()
                && new_capabilities.is_empty()
                && new_resources.is_empty()
                && new_prompts.is_empty();

            let mut status_val = json!({
                "transport": transport_type,
                "protocol_version": srv_cfg.protocol_version.as_deref().unwrap_or(DEFAULT_MCP_PROTOCOL_VERSION),
                "status": if is_degraded { "degraded" } else { "connected" }
            });

            if is_degraded {
                if let Some(obj) = status_val.as_object_mut() {
                    obj.insert(
                        "error".to_string(),
                        json!("Initial process spawn or MCP handshake failed; supervisor retrying in background"),
                    );
                }
            }

            statuses_guard.insert(server_id.to_string(), status_val);
        }
        {
            let mut caps_guard = self.capabilities.write().await;
            for (id, meta) in new_capabilities {
                info!(%server_id, capability_id = %id, "registered upstream capability");
                caps_guard.insert(id.clone(), meta);
                self.event_store.record("capability", &id, "added");
            }
        }
        {
            let mut res_guard = self.resources.write().await;
            for (id, meta) in new_resources {
                info!(%server_id, resource_id = %id, "registered upstream resource");
                res_guard.insert(id.clone(), meta);
                self.event_store.record("resource", &id, "added");
            }
        }
        {
            let mut prompts_guard = self.prompts.write().await;
            for (id, meta) in new_prompts {
                info!(%server_id, prompt_id = %id, "registered upstream prompt");
                prompts_guard.insert(id.clone(), meta);
                self.event_store.record("prompt", &id, "added");
            }
        }

        {
            let caps = self.capabilities.read().await;
            let res = self.resources.read().await;
            let prompts = self.prompts.read().await;
            let new_ver = compute_catalog_version(&caps, &res, &prompts);
            let mut ver_guard = self.catalog_version.write().await;
            *ver_guard = new_ver;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let _ = self
            .resource_update_tx
            .send(crate::catalog::ResourceUpdateEvent {
                uri: format!("server://{}", server_id),
                timestamp,
                server: server_id.to_string(),
            });

        Ok(())
    }

    /// Unmounts an upstream server dynamically from `AppState`.
    ///
    /// # Arguments
    /// * `server_id` - Identifier of server to unmount.
    pub async fn unmount_upstream_server(&self, server_id: &str) {
        info!(%server_id, "dynamically unmounting upstream server");

        {
            let mut servers_guard = self.servers.write().await;
            servers_guard.remove(server_id);
        }
        {
            let mut configs_guard = self.server_configs.write().await;
            configs_guard.remove(server_id);
        }
        {
            let mut statuses_guard = self.server_statuses.write().await;
            statuses_guard.remove(server_id);
        }
        self.circuit_breakers.remove(server_id).await;
        {
            let mut caps_guard = self.capabilities.write().await;
            let removed_caps: Vec<String> = caps_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect();
            for id in removed_caps {
                caps_guard.remove(&id);
                self.event_store.record("capability", &id, "removed");
            }
        }
        {
            let mut res_guard = self.resources.write().await;
            let removed_res: Vec<String> = res_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect();
            for id in removed_res {
                res_guard.remove(&id);
                self.event_store.record("resource", &id, "removed");
            }
        }
        {
            let mut prompts_guard = self.prompts.write().await;
            let removed_prompts: Vec<String> = prompts_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect();
            for id in removed_prompts {
                prompts_guard.remove(&id);
                self.event_store.record("prompt", &id, "removed");
            }
        }

        {
            let caps = self.capabilities.read().await;
            let res = self.resources.read().await;
            let prompts = self.prompts.read().await;
            let new_ver = compute_catalog_version(&caps, &res, &prompts);
            let mut ver_guard = self.catalog_version.write().await;
            *ver_guard = new_ver;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let _ = self
            .resource_update_tx
            .send(crate::catalog::ResourceUpdateEvent {
                uri: format!("server://{}", server_id),
                timestamp,
                server: server_id.to_string(),
            });
    }

    /// Reconciles the runtime daemon state against the on-disk configuration file.
    ///
    /// # Returns
    /// A JSON Value summary of mounted, unmounted, and updated upstream servers.
    ///
    /// # Errors
    /// Returns an error if loading or parsing the config file fails.
    pub async fn reload_from_disk(&self) -> Result<serde_json::Value> {
        info!(config_path = %self.config_path, "reloading configuration from disk");

        let config = crate::config::load_config(&self.config_path)?;

        // 1. Update security policy atomically
        {
            let mut pol_guard = self.policy.write().await;
            *pol_guard = Policy::from_config(config.policy.clone());
        }

        // 2. Identify active servers vs disk servers
        let current_server_ids: Vec<String> = {
            let srv_guard = self.servers.read().await;
            srv_guard.keys().cloned().collect()
        };

        let mut unmounted = Vec::new();
        let mut mounted = Vec::new();
        let mut warnings = Vec::new();

        // Unmount servers removed from disk config
        for active_id in &current_server_ids {
            if !config.mcp_servers.contains_key(active_id) {
                self.unmount_upstream_server(active_id).await;
                unmounted.push(active_id.clone());
            }
        }

        // Mount new or updated servers
        for (server_id, srv_cfg) in &config.mcp_servers {
            let res = self
                .mount_upstream_server(
                    server_id,
                    srv_cfg,
                    &config.capability_aliases,
                    &config.resource_aliases,
                    &config.prompt_aliases,
                )
                .await;

            match res {
                Ok(_) => {
                    mounted.push(server_id.clone());
                }
                Err(e) => {
                    warn!(server_id = %server_id, error = %e, "server failed to mount during reload");
                    warnings.push(format!("Server '{}': {}", server_id, e));
                }
            }
        }

        Ok(serde_json::json!({
            "ok": true,
            "mounted": mounted,
            "unmounted": unmounted,
            "warnings": warnings,
            "total_active": self.servers.read().await.len(),
            "catalog_version": self.catalog_version.read().await.clone(),
        }))
    }
}
