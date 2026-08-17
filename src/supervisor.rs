// Rust guideline compliant 2026-08-17

//! Upstream process supervisor and self-healing actor manager (`M-CANONICAL-DOCS`).

use anyhow::{Context, Result};
use rmcp::{
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    transport::TokioChildProcess,
    ServiceExt,
};
use serde_json::{json, Value};
use std::{collections::HashMap, time::Duration};
use tokio::{process::Command, sync::mpsc, time::timeout};
use tracing::{error, info, warn};

use crate::{
    config::ServerConfig,
    daemon::{
        state::{compute_catalog_version, AppState},
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
                        let description = resource
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        let mime_type = resource
                            .get("mimeType")
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

    let mut cmd = Command::new(command);
    cmd.args(&srv_cfg.args);
    cmd.envs(&srv_cfg.env);

    let transport = TokioChildProcess::new(cmd)
        .with_context(|| format!("Failed to spawn process for {}", server_id))?;

    let handshake_timeout = Duration::from_millis(5000);
    let initial_mcp_client = match timeout(handshake_timeout, ().serve(transport)).await {
        Ok(Ok(client)) => client,
        Ok(Err(err)) => {
            return Err(anyhow::anyhow!(
                "Failed to negotiate stdio MCP connection for {}: {}",
                server_id,
                err
            ))
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Stdio MCP connection negotiation for {} timed out after {}ms",
                server_id,
                handshake_timeout.as_millis()
            ))
        }
    };

    let (initial_caps, initial_res, initial_prompts) = discover_supervisor_items!(
        &initial_mcp_client,
        server_id,
        capability_aliases,
        resource_aliases,
        prompt_aliases
    );

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

    tokio::spawn(async move {
        let mut client_opt = Some(initial_mcp_client);
        let mut restart_count: u32 = 0;

        while let Some(msg) = rx.recv().await {
            let mut is_alive = false;

            if let Some(client) = &client_opt {
                is_alive = handle_supervisor_msg!(client, msg, tool_timeout);
                if !is_alive {
                    client_opt = None;
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

            // Attempt reconnection if process died
            if !is_alive && client_opt.is_none() && resilience_cfg.auto_restart {
                if restart_count < resilience_cfg.max_restarts {
                    restart_count += 1;
                    let backoff_ms = calculate_backoff_ms(restart_count);
                    warn!(
                        server_id = %server_id_owned,
                        restart_attempt = restart_count,
                        backoff_ms = backoff_ms,
                        "stdio child process exited; supervisor scheduling restart"
                    );

                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                    let mut cmd = Command::new(&command_owned);
                    cmd.args(&args_owned);
                    cmd.envs(&env_owned);

                    if let Ok(transport) = TokioChildProcess::new(cmd) {
                        if let Ok(new_client) = ().serve(transport).await {
                            info!(
                                server_id = %server_id_owned,
                                restart_attempt = restart_count,
                                "supervisor successfully restarted stdio child process"
                            );

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

                            client_opt = Some(new_client);
                            restart_count = 0;
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

fn is_connection_closed_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("channel closed")
        || lower.contains("broken pipe")
        || lower.contains("connection reset")
        || lower.contains("transport closed")
        || lower.contains("child process exited")
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
        let mut caps_guard = state.capabilities.write().await;
        for (id, meta) in new_capabilities {
            caps_guard.insert(id.clone(), meta);
            state.event_store.record("capability", &id, "added");
        }
    }
    {
        let mut res_guard = state.resources.write().await;
        for (id, meta) in new_resources {
            res_guard.insert(id.clone(), meta);
            state.event_store.record("resource", &id, "added");
        }
    }
    {
        let mut prompts_guard = state.prompts.write().await;
        for (id, meta) in new_prompts {
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
        assert!(!is_connection_closed_error("invalid params"));
    }
}
