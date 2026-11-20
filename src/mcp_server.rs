// Rust guideline compliant 2026-11-18
//! MCP server facade interface exposing compact tools/resources/prompts endpoints.
//!
//! Provides two server transports:
//! - **stdio** via [`run_mcp_server`]: for local process-to-process MCP sessions.
//! - **Streamable HTTP/SSE** via [`run_mcp_http_server`]: for remote network MCP clients.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        GetPromptRequestParams, GetPromptResponse, GetPromptResult, ListResourcesResult,
        ListToolsResult, Prompt, PromptListChangedNotification, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, Resource, ResourceListChangedNotification,
        ServerCapabilities, ServerConfig as McpServerConfig, Tool, ToolListChangedNotification,
    },
    transport::{
        stdio,
        streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
        },
    },
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde_json::{json, Map, Value};
use tokio::{net::TcpListener, sync::oneshot};

use crate::{
    config::{McpConfig, DEFAULT_MCP_HTTP_PORT},
    daemon::{initialize_state, AppState, ServerMsg, UpstreamCallError},
};

const TOOL_CAPABILITIES_LIST: &str = "capabilities_list";
const TOOL_CAPABILITY_SEARCH: &str = "capability_search";
const TOOL_CAPABILITY_DESCRIBE: &str = "capability_describe";
const TOOL_CAPABILITY_CALL: &str = "capability_call";
const TOOL_CAPABILITIES_BATCH_CALL: &str = "capabilities_batch_call";
const TOOL_RESOURCES_LIST: &str = "resources_list";
const TOOL_RESOURCE_READ: &str = "resource_read";
const TOOL_PROMPTS_LIST: &str = "prompts_list";
const TOOL_PROMPT_GET: &str = "prompt_get";
const TOOL_TASK_GET: &str = "task_get";
const TOOL_TASK_UPDATE: &str = "task_update";
const TOOL_TASK_CANCEL: &str = "task_cancel";
const TOOL_UNDO_EXECUTE: &str = "undo_execute";
const TOOL_COMPLETION_COMPLETE: &str = "completion_complete";
const TOOL_SUBSCRIPTIONS_LISTEN: &str = "subscriptions_listen";

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Internal MCP facade server handler implementation.
#[derive(Clone)]
pub struct FacadeMcpServer {
    state: AppState,
    profile: Option<String>,
    supported_protocol_versions: Option<Vec<rmcp::model::ProtocolVersion>>,
}

impl FacadeMcpServer {
    /// Creates a new `FacadeMcpServer` bound to the given state and optional profile.
    pub fn new(state: AppState, profile: Option<String>) -> Self {
        Self {
            state,
            profile,
            supported_protocol_versions: None,
        }
    }

    /// Configures custom supported protocol versions advertised during discovery / initialize.
    #[must_use]
    pub fn with_supported_protocol_versions(
        mut self,
        versions: Vec<rmcp::model::ProtocolVersion>,
    ) -> Self {
        self.supported_protocol_versions = Some(versions);
        self
    }
}

impl ServerHandler for FacadeMcpServer {
    fn get_info(&self) -> McpServerConfig {
        let mut info = McpServerConfig::default();
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_tool_list_changed()
            .enable_resources()
            .enable_resources_list_changed()
            .enable_resources_subscribe()
            .enable_prompts()
            .enable_prompts_list_changed()
            .build();
        info.instructions = Some(
            "Warmplane MCP facade server with deterministic tools/resources/prompts surfaces"
                .to_string(),
        );
        info
    }

    fn supported_protocol_versions(
        &self,
    ) -> std::borrow::Cow<'static, [rmcp::model::ProtocolVersion]> {
        if let Some(ref versions) = self.supported_protocol_versions {
            std::borrow::Cow::Owned(versions.clone())
        } else {
            // Default to stable session-based protocol versions up to 2025-11-25.
            // When clients explicitly configure 2026-07-28 via config, it is honored.
            std::borrow::Cow::Borrowed(rmcp::model::ProtocolVersion::known_up_to(
                &rmcp::model::ProtocolVersion::V_2025_11_25,
            ))
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        let mut tools = facade_tools();
        let prof_ctx = self.profile_context().await;
        if self.agent_self_undo_enabled(&prof_ctx).await {
            tools.push(Tool::new(
                TOOL_UNDO_EXECUTE,
                "Execute the newest armed undo for one of this agent's work items",
                schema_object(json!({"type":"object","properties":{"undo_id":{"type":"string","description":"Explicit undo entry identifier"},"work_item_id":{"type":"string","description":"Execute the newest armed inverse for this work item"}},"anyOf":[{"required":["undo_id"]},{"required":["work_item_id"]}],"additionalProperties":false})),
            ));
        }
        let caps_guard = self.state.capabilities.read().await;

        if let Ok(config) = crate::config::load_or_default_config(&self.state.config_path) {
            for (alias_name, alias_target) in &config.capability_aliases {
                if alias_target.is_passthrough() {
                    let target_id = alias_target.target();
                    let cap_meta_opt = caps_guard
                        .get(target_id)
                        .or_else(|| caps_guard.get(alias_name));

                    let is_allowed = if let Some(cap_meta) = cap_meta_opt {
                        prof_ctx.is_server_allowed(&cap_meta.server)
                    } else {
                        true
                    };

                    if is_allowed {
                        let default_desc = format!("Passthrough tool for '{}'", target_id);
                        let description = alias_target
                            .summary()
                            .or_else(|| alias_target.description())
                            .unwrap_or_else(|| {
                                cap_meta_opt
                                    .map(|m| m.summary.as_str())
                                    .unwrap_or(&default_desc)
                            });
                        let input_schema = cap_meta_opt
                            .map(|m| schema_object(m.input_schema.clone()))
                            .unwrap_or_else(|| {
                                schema_object(json!({
                                    "type": "object",
                                    "additionalProperties": true
                                }))
                            });

                        // Standard MCP tool names MUST match ^[a-zA-Z0-9_-]{1,64}$
                        let sanitized_name: String = alias_name
                            .chars()
                            .map(|c| {
                                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                                    c
                                } else {
                                    '_'
                                }
                            })
                            .take(64)
                            .collect();

                        tools.push(Tool::new(
                            sanitized_name,
                            description.to_string(),
                            input_schema,
                        ));
                    }
                }
            }
        }

        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<CallToolResponse, McpError> {
        let args = request.arguments.unwrap_or_default();

        let output = match request.name.as_ref() {
            TOOL_UNDO_EXECUTE => {
                let profile = self.profile_context().await;
                if !self.agent_self_undo_enabled(&profile).await {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Agent self-undo is disabled for this role or profile",
                        )),
                    ));
                }
                let undo_id = args.get("undo_id").and_then(Value::as_str);
                let work_item_id = args.get("work_item_id").and_then(Value::as_str);
                if undo_id.is_none() && work_item_id.is_none() {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Provide 'undo_id' or 'work_item_id'",
                        )),
                    ));
                };
                let role = self.state.rbac_engine.default_role();
                let policy = self
                    .state
                    .policy
                    .read()
                    .await
                    .clone()
                    .merge_with_profile(profile.profile_policy.as_ref());
                let access = crate::undo::UndoAccessContext {
                    tenant_id: "default".to_string(),
                    role,
                    actor_id: None,
                    request_context: Default::default(),
                    policy,
                    profile,
                    grant_expires_at_epoch_secs: None,
                    trace_id: next_trace_id(),
                };
                let handle = crate::engine::ControlPlaneHandle::new(self.state.clone());
                let result = if let Some(work_item_id) = work_item_id {
                    handle
                        .execute_agent_self_undo_for_work_item(work_item_id, &access)
                        .await
                } else if let Some(undo_id) = undo_id {
                    handle.execute_agent_self_undo(undo_id, &access).await
                } else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Provide 'undo_id' or 'work_item_id'",
                        )),
                    ));
                };
                match result {
                    Ok(result) => serde_json::to_value(result).map_err(|error| error.to_string()),
                    Err(error) => {
                        Ok(json!({"ok":false,"error":{"code":error.code,"message":error.message}}))
                    }
                }
            }
            TOOL_CAPABILITIES_LIST => self.list_capabilities_value().await,
            TOOL_CAPABILITY_SEARCH => {
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let server_ids = args.get("server_ids").and_then(Value::as_array).map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                });
                let tags = args.get("tags").and_then(Value::as_array).map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                });
                let modes = args.get("modes").and_then(Value::as_array).map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                });
                let limit = args
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map(|n| n as usize);

                self.search_capabilities_value(query, server_ids, tags, modes, limit)
                    .await
            }
            TOOL_CAPABILITY_DESCRIBE => {
                let Some(id) = args.get("id").and_then(Value::as_str) else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'id'",
                        )),
                    ));
                };
                self.describe_capability_value(id.to_string()).await
            }
            TOOL_CAPABILITY_CALL => {
                let Some(capability_id) = args.get("capability_id").and_then(Value::as_str) else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'capability_id'",
                        )),
                    ));
                };
                let Some(call_args) = args.get("args") else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'args'",
                        )),
                    ));
                };
                if !call_args.is_object() {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "'args' must be a JSON object",
                        )),
                    ));
                }
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                let input_responses: Option<std::collections::BTreeMap<String, Value>> = args
                    .get("input_responses")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .or_else(|| request.input_responses.clone());
                let request_state = args
                    .get("request_state")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| request.request_state.clone());

                self.call_capability_value(
                    capability_id.to_string(),
                    call_args.clone(),
                    request_id,
                    context,
                    input_responses,
                    request_state,
                )
                .await
            }
            TOOL_CAPABILITIES_BATCH_CALL => {
                let Some(steps_val) = args.get("steps") else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'steps'",
                        )),
                    ));
                };
                let Ok(steps) = serde_json::from_value::<Vec<crate::batch_executor::BatchStep>>(
                    steps_val.clone(),
                ) else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "'steps' must be an array of BatchStep objects",
                        )),
                    ));
                };
                let auto_rollback_on_failure = match args.get("autoRollbackOnFailure") {
                    None => false,
                    Some(Value::Bool(enabled)) => *enabled,
                    Some(_) => {
                        return Ok(CallToolResponse::Complete(
                            CallToolResult::structured_error(invalid_args(
                                "'autoRollbackOnFailure' must be a boolean",
                            )),
                        ));
                    }
                };
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let trace_id = next_trace_id();
                let policy = self.state.policy.read().await.clone();
                let prof_ctx = self.profile_context().await;
                let default_role = self.state.rbac_engine.default_role();
                let res = crate::batch_executor::execute_batch_for_tenant_with_rollback(
                    &self.state,
                    steps,
                    trace_id,
                    request_id,
                    context,
                    crate::batch_executor::BatchExecutionScope {
                        policy: &policy,
                        profile: &prof_ctx,
                        tenant_id: "default",
                        role: &default_role,
                        authorization_expires_at_epoch_secs: None,
                        cancellation_token: tokio_util::sync::CancellationToken::new(),
                    },
                    auto_rollback_on_failure,
                )
                .await;
                Ok(serde_json::to_value(res).unwrap_or_default())
            }
            TOOL_RESOURCES_LIST => self.list_resources_value().await,
            TOOL_RESOURCE_READ => {
                let Some(resource_id) = args.get("resource_id").and_then(Value::as_str) else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'resource_id'",
                        )),
                    ));
                };
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                let input_responses: Option<std::collections::BTreeMap<String, Value>> = args
                    .get("input_responses")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .or_else(|| request.input_responses.clone());
                let request_state = args
                    .get("request_state")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| request.request_state.clone());

                self.read_resource_value(
                    resource_id.to_string(),
                    request_id,
                    context,
                    input_responses,
                    request_state,
                )
                .await
            }
            TOOL_PROMPTS_LIST => self.list_prompts_value().await,
            TOOL_PROMPT_GET => {
                let Some(prompt_id) = args.get("prompt_id").and_then(Value::as_str) else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'prompt_id'",
                        )),
                    ));
                };
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                let arguments = args.get("arguments").cloned();
                let input_responses: Option<std::collections::BTreeMap<String, Value>> = args
                    .get("input_responses")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .or_else(|| request.input_responses.clone());
                let request_state = args
                    .get("request_state")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| request.request_state.clone());

                self.get_prompt_value(
                    prompt_id.to_string(),
                    arguments,
                    request_id,
                    context,
                    input_responses,
                    request_state,
                )
                .await
            }
            TOOL_COMPLETION_COMPLETE => {
                let ref_type = args.get("ref_type").and_then(|v| v.as_str()).unwrap_or("");
                let ref_name = args.get("ref_name").and_then(|v| v.as_str()).unwrap_or("");
                let arg_name = args
                    .get("argument_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arg_val = args
                    .get("argument_value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Ok(json!({
                    "ok": true,
                    "trace_id": next_trace_id(),
                    "data": {
                        "ref_type": ref_type,
                        "ref_name": ref_name,
                        "argument_name": arg_name,
                        "argument_value": arg_val,
                        "values": [],
                        "total": 0
                    }
                }))
            }
            TOOL_SUBSCRIPTIONS_LISTEN => {
                let after = args.get("after").and_then(Value::as_str);
                let (events, next_cursor) = self.state.event_store.get_events_after(after);
                let catalog_ver = self.state.catalog_version.read().await.clone();
                Ok(json!({
                    "ok": true,
                    "catalog_version": catalog_ver,
                    "cursor": next_cursor,
                    "events": events,
                }))
            }
            TOOL_TASK_GET => {
                let Some(task_id) = args
                    .get("taskId")
                    .or_else(|| args.get("task_id"))
                    .and_then(Value::as_str)
                else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'taskId'",
                        )),
                    ));
                };
                if let Some(record) = self.state.task_registry.get_task(task_id).await {
                    if !self
                        .state
                        .hold_coordinator
                        .can_access_task(
                            &self.state,
                            &record,
                            "default",
                            &self.state.rbac_engine.default_role(),
                            self.profile.as_deref(),
                        )
                        .await
                    {
                        return Ok(CallToolResponse::Complete(
                            CallToolResult::structured_error(invalid_args(
                                "Task is not accessible to this profile",
                            )),
                        ));
                    }
                    let resp = crate::tasks::TaskResponse::from(&record);
                    Ok(json!({
                        "ok": true,
                        "resultType": "complete",
                        "task": resp,
                    }))
                } else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(format!(
                            "Task '{}' not found",
                            task_id
                        ))),
                    ));
                }
            }
            TOOL_TASK_UPDATE => {
                let Some(task_id) = args
                    .get("taskId")
                    .or_else(|| args.get("task_id"))
                    .and_then(Value::as_str)
                else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'taskId'",
                        )),
                    ));
                };
                let input_responses: std::collections::BTreeMap<String, Value> = args
                    .get("inputResponses")
                    .or_else(|| args.get("input_responses"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                match self
                    .state
                    .task_registry
                    .update_task(task_id, input_responses)
                    .await
                {
                    Ok(true) => Ok(json!({
                        "ok": true,
                        "resultType": "complete",
                        "message": format!("Task '{}' updated with input responses", task_id),
                    })),
                    Ok(false) => Ok(json!({
                        "ok": false,
                        "error": {
                            "code": "INVALID_TASK_STATE",
                            "message": format!("Task '{}' is not in 'input_required' state or not found", task_id),
                        }
                    })),
                    Err(e) => Ok(json!({
                        "ok": false,
                        "error": {
                            "code": "TASK_UPDATE_FAILED",
                            "message": e.to_string(),
                        }
                    })),
                }
            }
            TOOL_TASK_CANCEL => {
                let Some(task_id) = args
                    .get("taskId")
                    .or_else(|| args.get("task_id"))
                    .and_then(Value::as_str)
                else {
                    return Ok(CallToolResponse::Complete(
                        CallToolResult::structured_error(invalid_args(
                            "Missing required field 'taskId'",
                        )),
                    ));
                };
                let reason = args
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                Ok(self.cancel_task_value(task_id, reason).await)
            }
            _ => {
                let tool_name = request.name.as_ref();
                if let Some(result) = self
                    .call_passthrough_value(tool_name, serde_json::Value::Object(args))
                    .await
                {
                    result
                } else {
                    return Err(McpError::invalid_params(
                        format!("Unknown tool '{}'.", request.name),
                        None,
                    ));
                }
            }
        };

        match output {
            Ok(value) => Ok(CallToolResponse::Complete(CallToolResult::structured(
                value,
            ))),
            Err(e) => Ok(CallToolResponse::Complete(CallToolResult::error(vec![
                ContentBlock::text(e),
            ]))),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<ListResourcesResult, McpError> {
        let res_guard = self.state.resources.read().await;
        let items = res_guard
            .values()
            .map(|r| {
                let mut res = Resource::new(r.uri.clone(), r.name.clone());
                if let Some(desc) = &r.description {
                    res = res.with_description(desc.clone());
                }
                if let Some(mime) = &r.mime_type {
                    res = res.with_mime_type(mime.clone());
                }
                res
            })
            .collect::<Vec<_>>();
        Ok(ListResourcesResult::with_all_items(items))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<ReadResourceResponse, McpError> {
        let (server, uri) = {
            let res_guard = self.state.resources.read().await;
            let Some((_, meta)) = res_guard.iter().find(|(_, m)| m.uri == request.uri) else {
                return Err(McpError::invalid_params(
                    format!("Resource URI '{}' not found", request.uri),
                    None,
                ));
            };
            (meta.server.clone(), meta.uri.clone())
        };

        let tx = {
            let servers_guard = self.state.servers.read().await;
            servers_guard
                .get(&server)
                .cloned()
                .ok_or_else(|| McpError::internal_error("Target server unreachable", None))?
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(ServerMsg::ReadResource {
            uri,
            input_responses: request.input_responses,
            request_state: request.request_state,
            reply: reply_tx,
        })
        .await
        .map_err(|_| McpError::internal_error("Server mailbox closed", None))?;

        match reply_rx.await {
            Ok(Ok(value)) => {
                let res: ReadResourceResult = serde_json::from_value(value).map_err(|e| {
                    McpError::internal_error(format!("Invalid resource payload: {e}"), None)
                })?;
                Ok(ReadResourceResponse::Complete(res))
            }
            Ok(Err(UpstreamCallError::Timeout)) => {
                Err(McpError::internal_error("Resource read timed out", None))
            }
            Ok(Err(UpstreamCallError::Upstream(err))) => Err(McpError::internal_error(err, None)),
            Err(_) => Err(McpError::internal_error("Actor task died", None)),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<rmcp::model::ListPromptsResult, McpError> {
        let prompts_guard = self.state.prompts.read().await;
        let prompts = prompts_guard
            .values()
            .map(|p| {
                let args = serde_json::from_value(Value::Array(p.arguments.clone())).ok();
                let mut prompt = Prompt::new(p.name.clone(), p.description.clone(), args);
                if let Some(title) = &p.title {
                    prompt = prompt.with_title(title.clone());
                }
                prompt
            })
            .collect::<Vec<_>>();
        Ok(rmcp::model::ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<GetPromptResponse, McpError> {
        let server = {
            let prompts_guard = self.state.prompts.read().await;
            let Some((_, prompt_meta)) = prompts_guard.iter().find(|(_, p)| p.name == request.name)
            else {
                return Err(McpError::invalid_params(
                    format!("Prompt '{}' not found", request.name),
                    None,
                ));
            };
            prompt_meta.server.clone()
        };

        let tx = {
            let servers_guard = self.state.servers.read().await;
            servers_guard
                .get(&server)
                .cloned()
                .ok_or_else(|| McpError::internal_error("Target server unreachable", None))?
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(ServerMsg::GetPrompt {
            name: request.name,
            arguments: request.arguments,
            input_responses: request.input_responses,
            request_state: request.request_state,
            reply: reply_tx,
        })
        .await
        .map_err(|_| McpError::internal_error("Server mailbox closed", None))?;

        match reply_rx.await {
            Ok(Ok(value)) => {
                let res: GetPromptResult = serde_json::from_value(value).map_err(|e| {
                    McpError::internal_error(format!("Invalid prompt payload: {e}"), None)
                })?;
                Ok(GetPromptResponse::Complete(res))
            }
            Ok(Err(UpstreamCallError::Timeout)) => {
                Err(McpError::internal_error("Prompt get timed out", None))
            }
            Ok(Err(UpstreamCallError::Upstream(err))) => Err(McpError::internal_error(err, None)),
            Err(_) => Err(McpError::internal_error("Actor task died", None)),
        }
    }
}

impl FacadeMcpServer {
    async fn agent_self_undo_enabled(&self, profile: &crate::context::ProfileContext) -> bool {
        self.state
            .rbac_engine
            .allows_agent_self_undo(&self.state.rbac_engine.default_role())
            && (profile.profile_id.is_none() || profile.allow_agent_self_undo)
    }

    async fn profile_context(&self) -> crate::context::ProfileContext {
        if let Some(ref prof_id) = self.profile {
            let profiles_guard = self.state.profiles.read().await;
            if let Some(prof_cfg) = profiles_guard.get(prof_id) {
                let prof_policy = prof_cfg
                    .policy
                    .as_ref()
                    .map(|p| crate::daemon::Policy::from_config(Some(p.clone())));
                return crate::context::ProfileContext::scoped_with_policy(
                    prof_id.clone(),
                    prof_cfg.servers.clone(),
                    prof_policy,
                )
                .with_agent_self_undo(prof_cfg.allow_agent_self_undo);
            }
        }
        crate::context::ProfileContext::unrestricted()
    }

    async fn list_capabilities_value(&self) -> std::result::Result<Value, String> {
        let handle = crate::engine::ControlPlaneHandle::new(self.state.clone());
        let res = handle
            .list_capabilities(self.profile.as_deref())
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_value(res).map_err(|e| e.to_string())
    }

    async fn search_capabilities_value(
        &self,
        query: Option<String>,
        server_ids: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        modes: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> std::result::Result<Value, String> {
        let handle = crate::engine::ControlPlaneHandle::new(self.state.clone());
        let res = handle
            .search_capabilities(
                query.as_deref(),
                server_ids,
                tags,
                modes,
                limit,
                self.profile.as_deref(),
            )
            .await;
        serde_json::to_value(res).map_err(|e| e.to_string())
    }

    async fn describe_capability_value(&self, id: String) -> std::result::Result<Value, String> {
        let handle = crate::engine::ControlPlaneHandle::new(self.state.clone());
        let env = handle
            .describe_capability(&id, self.profile.as_deref())
            .await;
        if env.ok {
            if let Some(cap) = env.data {
                return Ok(json!({
                    "version": "v1",
                    "capability": cap,
                }));
            }
        }
        serde_json::to_value(env).map_err(|e| e.to_string())
    }

    fn engine_handle(&self) -> crate::engine::ControlPlaneHandle {
        crate::engine::ControlPlaneHandle::new(self.state.clone())
    }

    async fn cancel_task_value(&self, task_id: &str, reason: Option<String>) -> Value {
        if let Some(record) = self.state.task_registry.get_task(task_id).await {
            if !self
                .state
                .hold_coordinator
                .can_access_task(
                    &self.state,
                    &record,
                    "default",
                    &self.state.rbac_engine.default_role(),
                    self.profile.as_deref(),
                )
                .await
            {
                return json!({"ok": false, "error": {"code": "TASK_NOT_FOUND", "message": "Task was not found"}});
            }
        }

        match self
            .state
            .hold_coordinator
            .cancel(&self.state, task_id, reason.clone())
            .await
        {
            Ok(crate::tasks::EmissionCancelResult::Cancelled)
            | Ok(crate::tasks::EmissionCancelResult::AlreadyCancelled) => json!({
                "ok": true,
                "resultType": "complete",
                "message": format!("Task '{}' cancelled", task_id),
            }),
            Ok(crate::tasks::EmissionCancelResult::TooLate) => json!({
                "ok": false,
                "error": {
                    "code": "EMISSION_DISPATCHING",
                    "message": "Emission dispatch has already started and may have reached the upstream server",
                }
            }),
            Ok(crate::tasks::EmissionCancelResult::NotEmission) => {
                match self.state.task_registry.cancel_task(task_id, reason).await {
                    Ok(true) => json!({
                        "ok": true,
                        "resultType": "complete",
                        "message": format!("Task '{}' cancelled", task_id),
                    }),
                    Ok(false) => json!({
                        "ok": false,
                        "error": {
                            "code": "TASK_NOT_CANCELLABLE",
                            "message": format!("Task '{}' is already completed, cancelled, or not found", task_id),
                        }
                    }),
                    Err(error) => json!({
                        "ok": false,
                        "error": {
                            "code": "TASK_CANCEL_FAILED",
                            "message": error.to_string(),
                        }
                    }),
                }
            }
            Err(error) => json!({
                "ok": false,
                "error": {
                    "code": "TASK_CANCEL_FAILED",
                    "message": error.to_string(),
                }
            }),
        }
    }

    async fn call_capability_value(
        &self,
        capability_id: String,
        args: Value,
        request_id: Option<String>,
        context: Option<crate::context::RequestContext>,
        input_responses: Option<std::collections::BTreeMap<String, Value>>,
        request_state: Option<String>,
    ) -> std::result::Result<Value, String> {
        let opts = crate::engine::ExecutionOptions {
            request_id,
            context,
            idempotency_key: None,
            input_responses,
            request_state,
            profile: self.profile.clone(),
            async_task: false,
            suppress_undo_registration: false,
        };

        let env = self
            .engine_handle()
            .call_capability(&capability_id, args, opts)
            .await;
        serde_json::to_value(env).map_err(|e| e.to_string())
    }

    async fn call_passthrough_value(
        &self,
        tool_name: &str,
        args: Value,
    ) -> Option<std::result::Result<Value, String>> {
        let caps_guard = self.state.capabilities.read().await;
        let mut target_id = caps_guard
            .contains_key(tool_name)
            .then(|| tool_name.to_string());
        if target_id.is_none() {
            if let Ok(config) = crate::config::load_or_default_config(&self.state.config_path) {
                for (alias, alias_target) in &config.capability_aliases {
                    if !alias_target.is_passthrough() {
                        continue;
                    }
                    let sanitized: String = alias
                        .chars()
                        .map(|character| {
                            if character.is_ascii_alphanumeric()
                                || character == '_'
                                || character == '-'
                            {
                                character
                            } else {
                                '_'
                            }
                        })
                        .take(64)
                        .collect();
                    if alias == tool_name || sanitized == tool_name {
                        target_id = Some(if caps_guard.contains_key(alias) {
                            alias.clone()
                        } else {
                            alias_target.target().to_string()
                        });
                        break;
                    }
                }
            }
        }
        drop(caps_guard);

        match target_id {
            Some(target) => Some(
                self.call_capability_value(target, args, None, None, None, None)
                    .await,
            ),
            None => None,
        }
    }

    async fn list_resources_value(&self) -> std::result::Result<Value, String> {
        let prof_ctx = self.profile_context().await;
        let res_guard = self.state.resources.read().await;
        let mut resources = res_guard
            .iter()
            .filter(|(_, meta)| prof_ctx.is_server_allowed(&meta.server))
            .map(|(id, meta)| {
                json!({
                    "id": id,
                    "server": meta.server,
                    "uri": meta.uri,
                    "name": meta.name,
                    "description": meta.description,
                    "mime_type": meta.mime_type,
                    "tags": meta.tags,
                })
            })
            .collect::<Vec<_>>();

        resources.sort_by(|a, b| {
            a.get("id")
                .and_then(|v| v.as_str())
                .cmp(&b.get("id").and_then(|v| v.as_str()))
        });

        Ok(json!({
            "version": "v1",
            "resources": resources,
        }))
    }

    async fn read_resource_value(
        &self,
        resource_id: String,
        request_id: Option<String>,
        context: Option<crate::context::RequestContext>,
        input_responses: Option<std::collections::BTreeMap<String, Value>>,
        request_state: Option<String>,
    ) -> std::result::Result<Value, String> {
        let options = crate::engine::ReadResourceOptions {
            request_id,
            context,
            input_responses,
            request_state,
            profile: self.profile.clone(),
        };
        serde_json::to_value(
            self.engine_handle()
                .read_resource(&resource_id, options)
                .await,
        )
        .map_err(|e| e.to_string())
    }

    async fn list_prompts_value(&self) -> std::result::Result<Value, String> {
        let prof_ctx = self.profile_context().await;
        let prompts_guard = self.state.prompts.read().await;
        let mut prompts = prompts_guard
            .iter()
            .filter(|(_, meta)| prof_ctx.is_server_allowed(&meta.server))
            .map(|(id, meta)| {
                json!({
                    "id": id,
                    "server": meta.server,
                    "name": meta.name,
                    "title": meta.title,
                    "description": meta.description,
                    "arguments": meta.arguments,
                    "tags": meta.tags,
                })
            })
            .collect::<Vec<_>>();

        prompts.sort_by(|a, b| {
            a.get("id")
                .and_then(|v| v.as_str())
                .cmp(&b.get("id").and_then(|v| v.as_str()))
        });

        Ok(json!({
            "version": "v1",
            "prompts": prompts,
        }))
    }

    async fn get_prompt_value(
        &self,
        prompt_id: String,
        arguments: Option<Value>,
        request_id: Option<String>,
        context: Option<crate::context::RequestContext>,
        input_responses: Option<std::collections::BTreeMap<String, Value>>,
        request_state: Option<String>,
    ) -> std::result::Result<Value, String> {
        let handle = self.engine_handle();
        let env = handle
            .get_prompt(
                &prompt_id,
                crate::engine::GetPromptOptions {
                    request_id,
                    context,
                    arguments,
                    input_responses,
                    request_state,
                    profile: self.profile.clone(),
                },
            )
            .await;
        serde_json::to_value(env).map_err(|e| e.to_string())
    }
}

fn invalid_args(message: impl Into<String>) -> Value {
    error_envelope(
        next_trace_id(),
        None,
        None,
        crate::idempotency::RetryMetadata::safe("not_started"),
        "INVALID_ARGS",
        message.into(),
        false,
    )
}

fn facade_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            TOOL_CAPABILITIES_LIST,
            "List compact capability index",
            schema_object(json!({"type":"object","properties":{},"additionalProperties":false})),
        ),
        Tool::new(
            TOOL_CAPABILITY_SEARCH,
            "Search capabilities using hybrid lexical and semantic matching",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "query":{"type":"string","description":"Search keywords or natural language query"},
                    "server_ids":{"type":"array","items":{"type":"string"},"description":"Optional server ID filter list"},
                    "tags":{"type":"array","items":{"type":"string"},"description":"Optional tag filter list"},
                    "modes":{"type":"array","items":{"type":"string"},"description":"Optional execution mode filter list"},
                    "limit":{"type":"integer","description":"Maximum number of ranked results to return"}
                },
                "additionalProperties":false
            })),
        ),
        Tool::new(
            TOOL_CAPABILITY_DESCRIBE,
            "Describe one capability",
            schema_object(json!({
                "type":"object",
                "properties":{"id":{"type":"string"}},
                "required":["id"],
                "additionalProperties":false
            })),
        ),
        Tool::new(
            TOOL_CAPABILITY_CALL,
            "Call one capability with normalized response envelope",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "capability_id":{"type":"string"},
                    "args":{"type":"object"},
                    "request_id":{"type":"string"},
                    "context":{"type":"object"},
                    "_meta":{"type":"object"}
                },
                "required":["capability_id","args"],
                "additionalProperties":true
            })),
        ),
        Tool::new(
            TOOL_CAPABILITIES_BATCH_CALL,
            "Execute multiple sequential capability steps with output variable reference interpolation",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "steps":{
                        "type":"array",
                        "items":{
                            "type":"object",
                            "properties":{
                                "id":{"type":"string"},
                                "capability_id":{"type":"string"},
                                "args":{"type":"object"},
                                "continue_on_error":{"type":"boolean"}
                            },
                            "required":["id","capability_id","args"]
                        }
                    },
                    "request_id":{"type":"string"},
                    "context":{"type":"object"},
                    "_meta":{"type":"object"},
                    "autoRollbackOnFailure":{
                        "type":"boolean",
                        "default":false,
                        "description":"Stop on the first failed step and attempt registered inverses for earlier successful steps in reverse order"
                    }
                },
                "required":["steps"],
                "additionalProperties":true
            })),
        ),
        Tool::new(
            TOOL_RESOURCES_LIST,
            "List compact resource index",
            schema_object(json!({"type":"object","properties":{},"additionalProperties":false})),
        ),
        Tool::new(
            TOOL_RESOURCE_READ,
            "Read one resource with normalized response envelope",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "resource_id":{"type":"string"},
                    "request_id":{"type":"string"}
                },
                "required":["resource_id"],
                "additionalProperties":false
            })),
        ),
        Tool::new(
            TOOL_PROMPTS_LIST,
            "List compact prompt index",
            schema_object(json!({"type":"object","properties":{},"additionalProperties":false})),
        ),
        Tool::new(
            TOOL_PROMPT_GET,
            "Get one prompt rendering with normalized response envelope",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "prompt_id":{"type":"string"},
                    "arguments":{"type":"object"},
                    "request_id":{"type":"string"}
                },
                "required":["prompt_id"],
                "additionalProperties":false
            })),
        ),
        Tool::new(
            TOOL_COMPLETION_COMPLETE,
            "Request argument autocompletion for a prompt or resource",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "ref_type":{"type":"string"},
                    "ref_name":{"type":"string"},
                    "argument_name":{"type":"string"},
                    "argument_value":{"type":"string"}
                },
                "required":["ref_type","ref_name","argument_name"],
                "additionalProperties":false
            })),
        ),
        Tool::new(
            TOOL_SUBSCRIPTIONS_LISTEN,
            "Query or subscribe to the catalog change feed",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "after":{"type":"string"}
                },
                "additionalProperties":false
            })),
        ),
        Tool::new(
            TOOL_TASK_GET,
            "Query a SEP-2663 task handle for status, progress, input requests, or results",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "taskId":{"type":"string","description":"Unique identifier of the task"}
                },
                "required":["taskId"],
                "additionalProperties":true
            })),
        ),
        Tool::new(
            TOOL_TASK_UPDATE,
            "Submit client input responses to a SEP-2663 task in 'input_required' state",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "taskId":{"type":"string","description":"Unique identifier of the task"},
                    "inputResponses":{"type":"object","description":"Input response key-value map"}
                },
                "required":["taskId","inputResponses"],
                "additionalProperties":true
            })),
        ),
        Tool::new(
            TOOL_TASK_CANCEL,
            "Cancel a SEP-2663 task in progress or awaiting input",
            schema_object(json!({
                "type":"object",
                "properties":{
                    "taskId":{"type":"string","description":"Unique identifier of the task"},
                    "reason":{"type":"string","description":"Optional cancellation reason"}
                },
                "required":["taskId"],
                "additionalProperties":true
            })),
        ),
    ]
}

fn schema_object(value: Value) -> Arc<Map<String, Value>> {
    Arc::new(value.as_object().cloned().unwrap_or_default())
}

fn next_trace_id() -> String {
    format!("trace-{}", TRACE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn error_envelope(
    trace_id: String,
    request_id: Option<String>,
    context: Option<crate::context::RequestContext>,
    retry: crate::idempotency::RetryMetadata,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> Value {
    crate::http_v1::types::error_envelope(
        trace_id, request_id, context, retry, code, message, retryable,
    )
}

/// Runs the Warmplane stdio MCP server proxy interface.
///
/// # Arguments
/// * `config` - Loaded `McpConfig` configuration struct.
/// * `config_path` - Path to the config file.
/// * `profile` - Optional named server constellation (profile) to restrict exposed capabilities.
///
/// # Errors
/// Returns an error if initializing upstream state or stdio transport fails.
pub async fn run_mcp_server(
    config: McpConfig,
    config_path: impl Into<String>,
    profile: Option<String>,
) -> Result<()> {
    if let Some(ref prof_id) = profile {
        if !config.profiles.contains_key(prof_id) {
            anyhow::bail!("Profile '{}' is not defined in configuration", prof_id);
        }
    }
    let state = initialize_state(config, config_path).await?;
    let state_for_shutdown = state.clone();
    let shutdown_token = state.shutdown_token.clone();
    let server = FacadeMcpServer::new(state.clone(), profile);
    let running = server.serve(stdio()).await?;

    // Spawn background forwarder for dynamic tool, resource, and prompt list updates
    let peer = running.peer().clone();
    let mut tool_rx = state.tool_list_changed_tx.subscribe();
    let mut res_rx = state.resource_list_changed_tx.subscribe();
    let mut prompt_rx = state.prompt_list_changed_tx.subscribe();
    let cancel_fwd = shutdown_token.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_fwd.cancelled() => break,
                res = tool_rx.recv() => {
                    if res.is_ok() {
                        let mut notif = ToolListChangedNotification::default();
                        let mut meta = rmcp::model::NotificationMetaObject::default();
                        meta.insert(
                            "io.warmplane/discovery_hint".to_string(),
                            serde_json::json!("Upstream tools changed. Run 'capabilities_list' to discover unpromoted capabilities, or invoke promoted native passthrough tools directly."),
                        );
                        notif.extensions.insert(meta);
                        let _ = peer.send_notification(notif.into()).await;
                    }
                }
                res = res_rx.recv() => {
                    if res.is_ok() {
                        let notif = ResourceListChangedNotification::default();
                        let _ = peer.send_notification(notif.into()).await;
                    }
                }
                res = prompt_rx.recv() => {
                    if res.is_ok() {
                        let notif = PromptListChangedNotification::default();
                        let _ = peer.send_notification(notif.into()).await;
                    }
                }
            }
        }
    });

    tokio::select! {
        res = running.waiting() => {
            let _ = res?;
        }
        _ = crate::daemon::shutdown_signal(shutdown_token) => {
            tracing::info!("shutdown signal received; closing MCP facade stdio server");
        }
    }

    state_for_shutdown.shutdown().await;
    Ok(())
}

/// Runs the Warmplane facade as a Streamable HTTP/SSE MCP server.
///
/// Binds a dedicated TCP port and serves the same facade surface as the stdio variant,
/// but over the MCP Streamable HTTP transport. One [`FacadeMcpServer`] is created per
/// incoming client session, sharing the underlying [`AppState`].
///
/// # Arguments
/// * `config` - Loaded `McpConfig` instance.
/// * `config_path` - Path to the configuration file (used by hot-reload).
/// * `port_override` - Optional port from CLI `--port`; overrides config and default.
/// * `bind_override` - Optional bind address from CLI `--bind`; overrides config and default.
/// * `profile_override` - Optional profile from CLI `--profile`; overrides config block value.
///
/// # Errors
/// Returns an error if state initialization, TCP binding, or transport negotiation fails.
pub async fn run_mcp_http_server(
    config: McpConfig,
    config_path: impl Into<String>,
    port_override: Option<u16>,
    bind_override: Option<String>,
    profile_override: Option<String>,
) -> Result<()> {
    // Resolve effective profile: CLI flag > mcpHttpServer.profile > None
    let effective_profile = profile_override.or_else(|| {
        config
            .mcp_http_server
            .as_ref()
            .and_then(|c| c.profile.clone())
    });

    if let Some(ref prof_id) = effective_profile {
        if !config.profiles.contains_key(prof_id) {
            anyhow::bail!("Profile '{}' is not defined in configuration", prof_id);
        }
    }

    // Resolve bind/port: CLI overrides > mcpHttpServer config > defaults
    let http_cfg = config.mcp_http_server.clone();
    let bind_addr = bind_override
        .or_else(|| http_cfg.as_ref().map(|c| c.bind.clone()))
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port = port_override
        .or_else(|| http_cfg.as_ref().map(|c| c.port))
        .unwrap_or(DEFAULT_MCP_HTTP_PORT);
    let sse_keep_alive = http_cfg
        .as_ref()
        .and_then(|c| c.sse_keep_alive_ms)
        .map(Duration::from_millis);
    let json_response = http_cfg.as_ref().map(|c| c.json_response).unwrap_or(true);
    let mut allowed_hosts: Vec<String> = http_cfg
        .as_ref()
        .map(|c| c.allowed_hosts.clone())
        .unwrap_or_default();
    let allowed_origins: Vec<String> = http_cfg
        .as_ref()
        .map(|c| c.allowed_origins.clone())
        .unwrap_or_default();

    add_mcp_http_hosts(&mut allowed_hosts, &bind_addr);

    let state = initialize_state(config, config_path).await?;
    let shutdown_token = state.shutdown_token.clone();
    let state_for_shutdown = state.clone();

    // Build rmcp StreamableHttpServerConfig
    let mut mcp_server_cfg = StreamableHttpServerConfig::default();
    mcp_server_cfg.sse_keep_alive = sse_keep_alive;
    mcp_server_cfg.json_response = json_response;
    mcp_server_cfg.cancellation_token = shutdown_token.clone();
    mcp_server_cfg.allowed_hosts = allowed_hosts;
    mcp_server_cfg.allowed_origins = allowed_origins;

    let parsed_protocol_versions = parse_mcp_protocol_versions(
        &http_cfg
            .map(|config| config.supported_protocol_versions)
            .unwrap_or_else(crate::config::default_mcp_http_protocol_versions),
    );

    // Build the Tower service factory — one FacadeMcpServer clone per session
    let state_for_factory = state.clone();
    let profile_for_factory = effective_profile.clone();
    let mcp_service = StreamableHttpService::new(
        move || {
            let s = state_for_factory.clone();
            let p = profile_for_factory.clone();
            let server = FacadeMcpServer::new(s, p)
                .with_supported_protocol_versions(parsed_protocol_versions.clone());
            Ok(server)
        },
        Arc::new(LocalSessionManager::default()),
        mcp_server_cfg,
    );

    // Mount on a dedicated Axum router so we can re-use Axum's serve infrastructure
    let router = axum::Router::new()
        .route_service("/mcp", mcp_service.clone())
        // Also accept root path for clients that omit the /mcp suffix
        .route_service("/", mcp_service);

    let listen_addr = format!("{}:{}", bind_addr, port);
    let listener = TcpListener::bind(&listen_addr).await?;
    tracing::info!(
        bind = %listen_addr,
        profile = ?effective_profile,
        "Warmplane MCP HTTP/SSE facade listening"
    );

    let server_fut = axum::serve(listener, router)
        .with_graceful_shutdown(crate::daemon::shutdown_signal(shutdown_token));

    if let Err(err) = server_fut.await {
        tracing::error!(error = %err, "MCP HTTP server error");
    }

    state_for_shutdown.shutdown().await;
    Ok(())
}

/// Adds required loopback and explicit bind hosts for the MCP HTTP facade.
pub(crate) fn add_mcp_http_hosts(allowed_hosts: &mut Vec<String>, bind_addr: &str) {
    // Always allow loopback; add the bind address if it is not already covered
    for loopback in &["127.0.0.1", "::1", "localhost"] {
        if !allowed_hosts.iter().any(|h| h == loopback) {
            allowed_hosts.push((*loopback).to_string());
        }
    }
    if !matches!(
        bind_addr,
        "127.0.0.1" | "::1" | "localhost" | "0.0.0.0" | "::"
    ) {
        allowed_hosts.push(bind_addr.to_string());
    }
}

/// Parses the advertised MCP protocol versions using the established fallback.
pub(crate) fn parse_mcp_protocol_versions(
    versions: &[String],
) -> Vec<rmcp::model::ProtocolVersion> {
    versions
        .iter()
        .map(|version| match version.as_str() {
            "2024-11-05" => rmcp::model::ProtocolVersion::V_2024_11_05,
            "2025-03-26" => rmcp::model::ProtocolVersion::V_2025_03_26,
            "2025-06-18" => rmcp::model::ProtocolVersion::V_2025_06_18,
            "2025-11-25" => rmcp::model::ProtocolVersion::V_2025_11_25,
            _ => rmcp::model::ProtocolVersion::V_2026_07_28,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{facade_tools, invalid_args};

    #[tokio::test]
    async fn mcp_self_undo_requires_role_and_active_profile_opt_in() {
        let rbac = crate::rbac::RbacConfig {
            enabled: true,
            default_role: "agent".to_string(),
            roles: std::collections::HashMap::from([(
                "agent".to_string(),
                crate::rbac::RolePolicyConfig {
                    allow_agent_self_undo: true,
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let state = crate::daemon::AppState::builder()
            .rbac_engine(crate::rbac::RbacEngine::new(Some(rbac)))
            .build();
        let facade = super::FacadeMcpServer::new(state, None);
        let disabled_profile = crate::context::ProfileContext::scoped("coding", ["repo"]);
        assert!(!facade.agent_self_undo_enabled(&disabled_profile).await);
        let enabled_profile = disabled_profile.with_agent_self_undo(true);
        assert!(facade.agent_self_undo_enabled(&enabled_profile).await);
        assert!(
            facade
                .agent_self_undo_enabled(&crate::context::ProfileContext::unrestricted())
                .await
        );
    }

    #[tokio::test]
    async fn mcp_facade_call_uses_the_delayable_hold_before_upstream_dispatch() {
        use crate::daemon::ServerMsg;
        use serde_json::json;
        use std::time::Duration;

        let (state, mut upstream) = crate::test_support::delayable_mail(1, 2000);
        let facade = super::FacadeMcpServer::new(state, None);
        let call = tokio::spawn(async move {
            facade
                .call_capability_value(
                    "mail.send".to_string(),
                    json!({"to": "person@example.com"}),
                    Some("mcp-delayable-call".to_string()),
                    None,
                    None,
                    None,
                )
                .await
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(upstream.try_recv().is_err());
        let Some(ServerMsg::CallTool { reply, .. }) =
            tokio::time::timeout(Duration::from_secs(2), upstream.recv())
                .await
                .unwrap()
        else {
            panic!("MCP facade should dispatch after the configured hold");
        };
        reply.send(Ok(json!({"sent": true}))).unwrap();
        let response = call.await.unwrap().unwrap();
        assert_eq!(response["ok"], true);
        assert_eq!(response["data"], json!({"sent": true}));
        assert!(upstream.try_recv().is_err());
    }

    #[tokio::test]
    async fn mcp_passthrough_alias_uses_the_delayable_hold() {
        use crate::{
            config::{AliasTarget, McpConfig, ReversibilityClass, ReversibilityConfig},
            daemon::{AppState, CapabilityMeta, ServerMsg},
        };
        use serde_json::json;
        use std::{collections::HashMap, time::Duration};
        use tokio::sync::mpsc;

        let directory = tempfile::tempdir().unwrap();
        let config_path = directory.path().join("config.json");
        let mut config = McpConfig::default();
        config.capability_aliases.insert(
            "send_mail".to_string(),
            AliasTarget::Detailed {
                target: "mail.send".to_string(),
                summary: None,
                description: None,
                passthrough: true,
                reversibility: Some(ReversibilityConfig {
                    class: ReversibilityClass::Delayable,
                    hold_window_secs: Some(1),
                    ttl_secs: None,
                    daily_limit: None,
                    pre_snapshot: None,
                    pre_undo_check: None,
                    test_target: None,
                    inverse: None,
                }),
            },
        );
        crate::config::save_config(config_path.to_str().unwrap(), &config).unwrap();

        let (server_tx, mut upstream) = mpsc::channel(8);
        let mut capability = CapabilityMeta::new(
            "mail",
            "send",
            "Send",
            "Send a message",
            json!({"type": "object"}),
        );
        capability.reversibility = Some(ReversibilityConfig {
            class: ReversibilityClass::Delayable,
            hold_window_secs: Some(1),
            ttl_secs: None,
            daily_limit: None,
            pre_snapshot: None,
            pre_undo_check: None,
            test_target: None,
            inverse: None,
        });
        let state = AppState::builder()
            .config_path(config_path.to_string_lossy().into_owned())
            .servers(HashMap::from([("mail".to_string(), server_tx)]))
            .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
            .tool_timeout_ms(2_000)
            .build();
        let facade = super::FacadeMcpServer::new(state, None);
        let call = tokio::spawn(async move {
            facade
                .call_passthrough_value("send_mail", json!({"to": "person@example.com"}))
                .await
                .expect("passthrough alias should resolve")
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(upstream.try_recv().is_err());
        let Some(ServerMsg::CallTool { name, reply, .. }) =
            tokio::time::timeout(Duration::from_secs(2), upstream.recv())
                .await
                .unwrap()
        else {
            panic!("passthrough emission should dispatch after the configured hold");
        };
        assert_eq!(name, "send");
        reply.send(Ok(json!({"sent": true}))).unwrap();
        let response = call.await.unwrap().unwrap();
        assert_eq!(response["ok"], true);
        assert_eq!(response["data"], json!({"sent": true}));
        assert!(upstream.try_recv().is_err());
    }

    #[tokio::test]
    async fn mcp_task_cancel_routes_through_the_hold_coordinator() {
        use crate::{
            emissions::release_deadline_ms,
            tasks::{CreateTaskParams, EmissionPhase, EmissionRecord, TaskStatus},
        };
        use serde_json::json;
        use std::time::Duration;

        let (state, mut upstream) =
            crate::test_support::delayable_mail(2, crate::config::DEFAULT_TOOL_TIMEOUT_MS);
        let task = state
            .hold_coordinator
            .accept(
                &state,
                CreateTaskParams {
                    capability_id: "mail.send".to_string(),
                    server_id: "mail".to_string(),
                    args: json!({"to": "person@example.com"}),
                    request_id: Some("mcp-cancel-request".to_string()),
                    context: None,
                    idempotency_key: None,
                    initial_status: TaskStatus::Working,
                    status_message: Some("waiting".to_string()),
                    input_requests: None,
                    ttl_ms: None,
                    poll_interval_ms: None,
                },
                EmissionRecord {
                    idempotency_claim: None,
                    phase: EmissionPhase::PendingEmission,
                    release_at_ms: release_deadline_ms(2).unwrap(),
                    cancellable: true,
                    tool_name: "send".to_string(),
                    input_responses: None,
                    request_state: None,
                    profile_id: None,
                    tenant_id: "default".to_string(),
                    role: "anonymous".to_string(),
                    authorization_expires_at_epoch_secs: None,
                    approval_completed: false,
                    batch_id: None,
                    step_id: None,
                },
            )
            .await
            .unwrap();
        let facade = super::FacadeMcpServer::new(state.clone(), None);

        let response = facade
            .cancel_task_value(&task.task_id, Some("operator cancel".to_string()))
            .await;
        assert_eq!(response["ok"], true);
        assert_eq!(
            state
                .task_registry
                .get_task(&task.task_id)
                .await
                .unwrap()
                .status,
            TaskStatus::Cancelled
        );
        tokio::time::sleep(Duration::from_millis(2_100)).await;
        assert!(upstream.try_recv().is_err());
    }

    #[test]
    fn facade_tools_include_all_lightweight_operations() {
        let names = facade_tools()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 14);
        assert!(names.contains(&"capabilities_list".to_string()));
        assert!(names.contains(&"capability_search".to_string()));
        assert!(names.contains(&"capability_describe".to_string()));
        assert!(names.contains(&"capability_call".to_string()));
        assert!(names.contains(&"capabilities_batch_call".to_string()));
        assert!(names.contains(&"resources_list".to_string()));
        assert!(names.contains(&"resource_read".to_string()));
        assert!(names.contains(&"prompts_list".to_string()));
        assert!(names.contains(&"prompt_get".to_string()));
        assert!(names.contains(&"completion_complete".to_string()));
        assert!(names.contains(&"subscriptions_listen".to_string()));
        assert!(names.contains(&"task_get".to_string()));
        assert!(names.contains(&"task_update".to_string()));
        assert!(names.contains(&"task_cancel".to_string()));
    }

    #[test]
    fn invalid_args_envelope_has_expected_shape() {
        let payload = invalid_args("bad input");
        assert_eq!(payload["ok"], false);
        assert_eq!(payload["error"]["code"], "INVALID_ARGS");
        assert_eq!(payload["error"]["message"], "bad input");
        assert_eq!(payload["data"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_facade_search_capabilities() {
        use crate::daemon::{AppState, CapabilityMeta, Policy};
        use std::collections::HashMap;

        let mut caps = HashMap::new();
        caps.insert(
            "db.query".to_string(),
            CapabilityMeta::new(
                "sqlite",
                "read_query",
                "Execute read-only SQL queries",
                "Run SQL SELECT queries against SQLite database",
                serde_json::json!({"type": "object"}),
            ),
        );
        caps.insert(
            "fs.read".to_string(),
            CapabilityMeta::new(
                "fs",
                "read_file",
                "Read file contents from filesystem",
                "Read utf-8 contents of a file",
                serde_json::json!({"type": "object"}),
            ),
        );

        let state = AppState::builder()
            .capabilities(caps)
            .policy(Policy::default())
            .catalog_version("test-ver")
            .build();

        let server = super::FacadeMcpServer::new(state, None);
        let res = server
            .search_capabilities_value(Some("SQL database".to_string()), None, None, None, Some(5))
            .await
            .expect("search should succeed");

        assert_eq!(res["version"], "v1");
        assert_eq!(res["query"], "SQL database");
        assert!(res["total"].as_u64().unwrap_or(0) >= 1);
        assert_eq!(res["capabilities"][0]["id"], "db.query");
    }

    #[tokio::test]
    async fn test_facade_profile_partitioning() {
        use crate::config::ProfileConfig;
        use crate::daemon::{AppState, CapabilityMeta, Policy};
        use std::collections::HashMap;

        let mut caps = HashMap::new();
        caps.insert(
            "db.query".to_string(),
            CapabilityMeta::new(
                "sqlite",
                "read_query",
                "Execute read-only SQL queries",
                "Run SQL SELECT queries",
                serde_json::json!({"type": "object"}),
            ),
        );
        caps.insert(
            "fs.read".to_string(),
            CapabilityMeta::new(
                "fs",
                "read_file",
                "Read file contents",
                "Read utf-8 contents",
                serde_json::json!({"type": "object"}),
            ),
        );

        let mut profiles = HashMap::new();
        profiles.insert(
            "db_only".to_string(),
            ProfileConfig {
                servers: vec!["sqlite".to_string()],
                description: None,
                policy: None,
                allow_agent_self_undo: false,
            },
        );

        let state = AppState::builder()
            .capabilities(caps)
            .profiles(profiles)
            .policy(Policy::default())
            .catalog_version("test-ver")
            .build();

        let server_scoped = super::FacadeMcpServer::new(state.clone(), Some("db_only".to_string()));

        let list_res = server_scoped.list_capabilities_value().await.unwrap();
        let list_arr = list_res["capabilities"].as_array().unwrap();
        assert_eq!(list_arr.len(), 1);
        assert_eq!(list_arr[0]["id"], "db.query");

        // Calling capability outside profile should fail with TOOL_NOT_IN_PROFILE
        let call_res = server_scoped
            .call_capability_value(
                "fs.read".to_string(),
                serde_json::json!({}),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(call_res["ok"], false);
        assert_eq!(call_res["error"]["code"], "TOOL_NOT_IN_PROFILE");
    }
}

#[cfg(test)]
mod batch_tests;
