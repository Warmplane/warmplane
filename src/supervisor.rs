// Rust guideline compliant 2026-08-28

//! Upstream process supervisor and self-healing actor manager (`M-CANONICAL-DOCS`).

use anyhow::{anyhow, Context, Result};
use rmcp::{
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    transport::{
        streamable_http_client::StreamableHttpClientTransportConfig, StreamableHttpClientTransport,
        TokioChildProcess,
    },
    ServiceExt,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{process::Command, sync::mpsc, sync::RwLock, time::timeout};
use tracing::{error, info, warn};

use crate::{
    config::{AuthConfig, ServerConfig},
    daemon::{
        state::{compute_catalog_version, AppState},
        transport::build_http_headers,
        types::{CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg, UpstreamCallError},
    },
};

macro_rules! handle_supervisor_msg {
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

macro_rules! discover_supervisor_items {
    ($mcp_client:expr, $server_id:expr, $capability_aliases:expr, $resource_aliases:expr, $prompt_aliases:expr) => {{
        let discovery_timeout = std::time::Duration::from_millis(3000);
        let mut new_capabilities = Vec::new();
        if let Ok(Ok(tools)) = tokio::time::timeout(
            discovery_timeout,
            $mcp_client.list_tools(Default::default()),
        )
        .await
        {
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
        if let Ok(Ok(listed_resources)) = tokio::time::timeout(
            discovery_timeout,
            $mcp_client.list_resources(Default::default()),
        )
        .await
        {
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
                        let description = resource
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        let mime_type = resource
                            .get("mime_type")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);

                        let source_id = format!("{}.{}", $server_id, uri);
                        let resource_id = $resource_aliases
                            .get(&source_id)
                            .cloned()
                            .unwrap_or(source_id);

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
        if let Ok(Ok(listed_prompts)) = tokio::time::timeout(
            discovery_timeout,
            $mcp_client.list_prompts(Default::default()),
        )
        .await
        {
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
) -> Result<rmcp::service::RunningService<rmcp::RoleClient, ()>> {
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
    let mcp_client = ().serve(transport).await.with_context(|| {
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
    capability_aliases: &HashMap<String, String>,
    resource_aliases: &HashMap<String, String>,
    prompt_aliases: &HashMap<String, String>,
) -> Result<(
    Vec<(String, CapabilityMeta)>,
    Vec<(String, ResourceMeta)>,
    Vec<(String, PromptMeta)>,
    mpsc::Sender<ServerMsg>,
)> {
    let command = srv_cfg
        .command
        .as_ref()
        .context("Command required for stdio server")?;

    let mut resolved_env = HashMap::new();
    for (k, v) in &srv_cfg.env {
        let resolved = crate::vault::resolve_secret_value(v)
            .unwrap_or_else(|e| {
                warn!(server_id = %server_id, key = %k, error = %e, "Failed to resolve secret from vault; using raw string");
                v.clone()
            });
        resolved_env.insert(k.clone(), resolved);
    }

    let mut cmd = Command::new(command);
    cmd.args(&srv_cfg.args);
    cmd.envs(&resolved_env);
    cmd.stderr(std::process::Stdio::inherit());
    cmd.kill_on_drop(true);

    let transport = TokioChildProcess::new(cmd);

    let handshake_timeout = Duration::from_millis(5000);
    let (initial_client_opt, initial_caps, initial_res, initial_prompts) = match transport {
        Ok(proc) => match timeout(handshake_timeout, ().serve(proc)).await {
            Ok(Ok(client)) => {
                let (caps, res, prompts) = discover_supervisor_items!(
                    &client,
                    server_id,
                    capability_aliases,
                    resource_aliases,
                    prompt_aliases
                );
                (Some(client), caps, res, prompts)
            }
            Ok(Err(err)) => {
                warn!(
                    server_id = %server_id,
                    error = %err,
                    "Failed to negotiate initial stdio MCP handshake; supervisor will retry in background"
                );
                (None, Vec::new(), Vec::new(), Vec::new())
            }
            Err(_) => {
                warn!(
                    server_id = %server_id,
                    "Stdio MCP handshake timed out after 5000ms; supervisor will retry in background"
                );
                (None, Vec::new(), Vec::new(), Vec::new())
            }
        },
        Err(err) => {
            warn!(
                server_id = %server_id,
                error = %err,
                "Failed to spawn initial stdio child process; supervisor will retry in background"
            );
            (None, Vec::new(), Vec::new(), Vec::new())
        }
    };

    let (tx, mut rx) = mpsc::channel::<ServerMsg>(32);

    let resilience_cfg = srv_cfg.resilience.clone().unwrap_or_default();
    let server_id_owned = server_id.to_string();
    let command_owned = command.clone();
    let args_owned = srv_cfg.args.clone();
    let env_owned = srv_cfg.env.clone();
    let cap_aliases_owned = capability_aliases.clone();
    let res_aliases_owned = resource_aliases.clone();
    let prompt_aliases_owned = prompt_aliases.clone();
    let tool_timeout = Duration::from_millis(state.tool_timeout_ms);
    let state_clone = state.clone();
    let shutdown_token = state.shutdown_token.clone();

    tokio::spawn(async move {
        let mut client_opt = initial_client_opt;
        let mut restart_count: u32 = 0;
        let mut last_restart_time: Option<Instant> = None;
        const STABLE_UPTIME_WINDOW: Duration = Duration::from_secs(60);

        loop {
            let msg = tokio::select! {
                _ = shutdown_token.cancelled() => {
                    info!(
                        server_id = %server_id_owned,
                        "shutdown token cancelled; terminating stdio supervisor loop"
                    );
                    break;
                }
                opt = rx.recv() => match opt {
                    Some(m) => m,
                    None => break,
                }
            };

            let mut is_alive = false;

            if let Some(client) = &client_opt {
                is_alive = handle_supervisor_msg!(client, msg, tool_timeout);
                if !is_alive {
                    client_opt = None;
                    let mut statuses_guard = state_clone.server_statuses.write().await;
                    if let Some(status_val) = statuses_guard.get_mut(&server_id_owned) {
                        if let Some(obj) = status_val.as_object_mut() {
                            obj.insert("status".to_string(), json!("degraded"));
                            obj.insert(
                                "error".to_string(),
                                json!(
                                    "stdio child process exited; supervisor retrying in background"
                                ),
                            );
                        }
                    }
                }
            } else {
                match msg {
                    ServerMsg::CallTool { reply, .. }
                    | ServerMsg::ReadResource { reply, .. }
                    | ServerMsg::GetPrompt { reply, .. } => {
                        let _ = reply.send(Err(UpstreamCallError::Upstream(
                            "Server process is restarting".to_string(),
                        )));
                    }
                }
            }

            // Attempt reconnection if process died and shutdown is not in progress
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
                        "stdio child process exited; supervisor scheduling restart"
                    );

                    tokio::select! {
                        _ = shutdown_token.cancelled() => {
                            info!(
                                server_id = %server_id_owned,
                                "shutdown requested during restart backoff; cancelling supervisor"
                            );
                            break;
                        }
                        _ = tokio::time::sleep(Duration::from_millis(backoff_ms)) => {}
                    }

                    let mut resolved_restart_env = HashMap::new();
                    for (k, v) in &env_owned {
                        let resolved = crate::vault::resolve_secret_value(v)
                            .unwrap_or_else(|e| {
                                warn!(server_id = %server_id_owned, key = %k, error = %e, "Failed to resolve secret from vault on restart; using raw string");
                                v.clone()
                            });
                        resolved_restart_env.insert(k.clone(), resolved);
                    }

                    let mut cmd = Command::new(&command_owned);
                    cmd.args(&args_owned);
                    cmd.envs(&resolved_restart_env);
                    cmd.stderr(std::process::Stdio::inherit());
                    cmd.kill_on_drop(true);

                    if let Ok(transport) = TokioChildProcess::new(cmd) {
                        if let Ok(new_client) = ().serve(transport).await {
                            info!(
                                server_id = %server_id_owned,
                                restart_attempt = restart_count,
                                "supervisor successfully restarted stdio child process"
                            );

                            // Actively reset circuit breaker upon successful recovery
                            state_clone.circuit_breakers.reset(&server_id_owned).await;

                            let (new_caps, new_res, new_prompts) = discover_supervisor_items!(
                                &new_client,
                                &server_id_owned,
                                &cap_aliases_owned,
                                &res_aliases_owned,
                                &prompt_aliases_owned
                            );

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

                            client_opt = Some(new_client);
                        } else {
                            error!(
                                server_id = %server_id_owned,
                                "supervisor failed to negotiate MCP handshake upon restart"
                            );
                        }
                    } else {
                        error!(
                            server_id = %server_id_owned,
                            "supervisor failed to spawn new child process upon restart"
                        );
                    }
                } else {
                    error!(
                        server_id = %server_id_owned,
                        max_restarts = resilience_cfg.max_restarts,
                        "supervisor exceeded maximum restart attempts; server process disabled"
                    );
                }
            }
        }
    });

    Ok((initial_caps, initial_res, initial_prompts, tx))
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
    capability_aliases: &HashMap<String, String>,
    resource_aliases: &HashMap<String, String>,
    prompt_aliases: &HashMap<String, String>,
) -> Result<(
    Vec<(String, CapabilityMeta)>,
    Vec<(String, ResourceMeta)>,
    Vec<(String, PromptMeta)>,
    mpsc::Sender<ServerMsg>,
)> {
    let handshake_timeout = Duration::from_millis(5000);
    let initial_attempt = timeout(
        handshake_timeout,
        connect_http_mcp_client(state, server_id, srv_cfg),
    )
    .await;

    let (initial_client_opt, initial_caps, initial_res, initial_prompts) = match initial_attempt {
        Ok(Ok(client)) => {
            let (caps, res, prompts) = discover_supervisor_items!(
                &client,
                server_id,
                capability_aliases,
                resource_aliases,
                prompt_aliases
            );
            (Some(client), caps, res, prompts)
        }
        Ok(Err(err)) => {
            warn!(
                server_id = %server_id,
                error = %err,
                "Failed to negotiate initial streamable HTTP MCP connection; supervisor will retry in background"
            );
            (None, Vec::new(), Vec::new(), Vec::new())
        }
        Err(_) => {
            warn!(
                server_id = %server_id,
                "Streamable HTTP MCP handshake timed out after 5000ms; supervisor will retry in background"
            );
            (None, Vec::new(), Vec::new(), Vec::new())
        }
    };

    let (tx, mut rx) = mpsc::channel::<ServerMsg>(32);

    let resilience_cfg = srv_cfg.resilience.clone().unwrap_or_default();
    let server_id_owned = server_id.to_string();
    let srv_cfg_owned = srv_cfg.clone();
    let cap_aliases_owned = capability_aliases.clone();
    let res_aliases_owned = resource_aliases.clone();
    let prompt_aliases_owned = prompt_aliases.clone();
    let tool_timeout = Duration::from_millis(state.tool_timeout_ms);
    let state_clone = state.clone();
    let shutdown_token = state.shutdown_token.clone();

    tokio::spawn(async move {
        let mut client_opt = initial_client_opt;
        let mut restart_count: u32 = 0;
        let mut last_restart_time: Option<Instant> = None;
        const STABLE_UPTIME_WINDOW: Duration = Duration::from_secs(60);

        loop {
            let msg = tokio::select! {
                _ = shutdown_token.cancelled() => {
                    info!(
                        server_id = %server_id_owned,
                        "shutdown token cancelled; terminating HTTP supervisor loop"
                    );
                    break;
                }
                opt = rx.recv() => match opt {
                    Some(m) => m,
                    None => break,
                }
            };

            let mut is_alive = false;

            if let Some(client) = &client_opt {
                is_alive = handle_supervisor_msg!(client, msg, tool_timeout);
                if !is_alive {
                    client_opt = None;
                    let mut statuses_guard = state_clone.server_statuses.write().await;
                    if let Some(status_val) = statuses_guard.get_mut(&server_id_owned) {
                        if let Some(obj) = status_val.as_object_mut() {
                            obj.insert("status".to_string(), json!("degraded"));
                            obj.insert(
                                "error".to_string(),
                                json!("Remote HTTP connection dropped; supervisor retrying in background"),
                            );
                        }
                    }
                }
            } else {
                match msg {
                    ServerMsg::CallTool { reply, .. }
                    | ServerMsg::ReadResource { reply, .. }
                    | ServerMsg::GetPrompt { reply, .. } => {
                        let _ = reply.send(Err(UpstreamCallError::Upstream(
                            "Remote HTTP server is reconnecting".to_string(),
                        )));
                    }
                }
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
                        "Remote HTTP server disconnected; supervisor scheduling reconnect"
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
                        connect_http_mcp_client(&state_clone, &server_id_owned, &srv_cfg_owned),
                    )
                    .await;

                    match reconnect_attempt {
                        Ok(Ok(new_client)) => {
                            info!(
                                server_id = %server_id_owned,
                                restart_attempt = restart_count,
                                "supervisor successfully reconnected remote HTTP MCP server"
                            );

                            // Actively reset circuit breaker upon successful recovery
                            state_clone.circuit_breakers.reset(&server_id_owned).await;

                            let (new_caps, new_res, new_prompts) = discover_supervisor_items!(
                                &new_client,
                                &server_id_owned,
                                &cap_aliases_owned,
                                &res_aliases_owned,
                                &prompt_aliases_owned
                            );

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

                            client_opt = Some(new_client);
                        }
                        Ok(Err(err)) => {
                            error!(
                                server_id = %server_id_owned,
                                error = %err,
                                "supervisor failed to reconnect HTTP MCP server"
                            );
                        }
                        Err(_) => {
                            error!(
                                server_id = %server_id_owned,
                                "supervisor HTTP reconnect attempt timed out"
                            );
                        }
                    }
                } else {
                    error!(
                        server_id = %server_id_owned,
                        max_restarts = resilience_cfg.max_restarts,
                        "supervisor exceeded maximum reconnect attempts for HTTP server"
                    );
                }
            }
        }
    });

    Ok((initial_caps, initial_res, initial_prompts, tx))
}

fn is_connection_closed_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("channel closed")
        || lower.contains("broken pipe")
        || lower.contains("connection reset")
        || lower.contains("transport closed")
        || lower.contains("child process exited")
        || lower.contains("io error")
        || lower.contains("unexpected eof")
        || lower.contains("connection refused")
        || lower.contains("connect error")
        || lower.contains("error sending request")
        || lower.contains("stream closed")
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

    {
        let caps = state.capabilities.read().await;
        let res = state.resources.read().await;
        let prompts = state.prompts.read().await;
        let new_ver = compute_catalog_version(&caps, &res, &prompts);
        let mut ver_guard = state.catalog_version.write().await;
        *ver_guard = new_ver;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());

    let _ = state
        .resource_update_tx
        .send(crate::catalog::ResourceUpdateEvent {
            uri: format!("server://{}", server_id),
            timestamp,
            server: server_id.to_string(),
        });
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
}
