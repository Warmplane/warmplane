// Rust guideline compliant 2026-08-17

//! MCP stdio server facade interface exposing compact tools/resources/prompts endpoints.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::Arc,
};

use anyhow::Result;
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
        GetPromptRequestParams, GetPromptResponse, GetPromptResult, ListResourcesResult,
        ListToolsResult, Prompt, ReadResourceRequestParams, ReadResourceResponse,
        ReadResourceResult, Resource, ServerCapabilities, ServerInfo, Tool,
    },
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde_json::{json, Map, Value};
use tokio::sync::oneshot;

use crate::{
    config::McpConfig,
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
const TOOL_COMPLETION_COMPLETE: &str = "completion_complete";
const TOOL_SUBSCRIPTIONS_LISTEN: &str = "subscriptions_listen";

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct FacadeMcpServer {
    state: AppState,
}

impl ServerHandler for FacadeMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .enable_prompts()
            .build();
        info.instructions = Some(
            "Warmplane MCP facade server with deterministic tools/resources/prompts surfaces"
                .to_string(),
        );
        info
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        let tools = facade_tools();
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<CallToolResponse, McpError> {
        let args = request.arguments.unwrap_or_default();

        let output = match request.name.as_ref() {
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
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let trace_id = next_trace_id();
                let res = crate::batch_executor::execute_batch(
                    &self.state,
                    steps,
                    trace_id,
                    request_id,
                    context,
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
            _ => {
                return Err(McpError::invalid_params(
                    format!("Unknown tool '{}'.", request.name),
                    None,
                ));
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
    async fn list_capabilities_value(&self) -> std::result::Result<Value, String> {
        let caps_guard = self.state.capabilities.read().await;
        let mut capabilities = caps_guard
            .iter()
            .map(|(id, meta)| {
                json!({
                    "id": id,
                    "summary": meta.summary,
                    "server": meta.server,
                    "tool": meta.tool,
                    "tags": meta.tags,
                })
            })
            .collect::<Vec<_>>();

        capabilities.sort_by(|a, b| {
            a.get("id")
                .and_then(|v| v.as_str())
                .cmp(&b.get("id").and_then(|v| v.as_str()))
        });

        Ok(json!({
            "version": "v1",
            "capabilities": capabilities,
        }))
    }

    async fn search_capabilities_value(
        &self,
        query: Option<String>,
        server_ids: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        modes: Option<Vec<String>>,
        limit: Option<usize>,
    ) -> std::result::Result<Value, String> {
        let mut filter_builder = crate::search::SearchFilter::builder();
        if let Some(servers) = server_ids {
            filter_builder = filter_builder.server_ids(servers);
        }
        if let Some(t) = tags {
            filter_builder = filter_builder.tags(t);
        }
        if let Some(m) = modes {
            filter_builder = filter_builder.modes(m);
        }
        let filter = filter_builder.build();

        let caps = self.state.capabilities.read().await;
        let pol = self.state.policy.read().await;
        let catalog_ver = self.state.catalog_version.read().await.clone();
        let query_str = query.as_deref().unwrap_or("");
        let limit = limit.unwrap_or(8);

        let results = self
            .state
            .search_engine
            .search(query_str, limit, &filter, &caps, &pol);

        Ok(json!({
            "version": "v1",
            "catalog_version": catalog_ver,
            "query": query_str,
            "total": results.len(),
            "capabilities": results,
        }))
    }

    async fn describe_capability_value(&self, id: String) -> std::result::Result<Value, String> {
        let caps_guard = self.state.capabilities.read().await;
        match caps_guard.get(&id) {
            Some(meta) => Ok(json!({
                "version": "v1",
                "capability": {
                    "id": id,
                    "server": meta.server,
                    "tool": meta.tool,
                    "description": meta.description,
                    "input_schema": meta.input_schema,
                    "examples": meta.examples,
                }
            })),
            None => Ok(error_envelope(
                next_trace_id(),
                None,
                None,
                crate::idempotency::RetryMetadata::safe("not_started"),
                "TOOL_NOT_FOUND",
                format!("Capability '{}' not found", id),
                false,
            )),
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
        let trace_id = next_trace_id();
        let ctx = context.unwrap_or_default();
        if !self.state.policy.read().await.allows(&capability_id) {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::unsafe_op("not_started"),
                "INVALID_ARGS",
                format!("Capability '{}' blocked by policy", capability_id),
                false,
            ));
        }

        let (server, tool) = {
            let caps_guard = self.state.capabilities.read().await;
            let Some(meta) = caps_guard.get(&capability_id) else {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::unsafe_op("not_started"),
                    "TOOL_NOT_FOUND",
                    format!("Capability '{}' not found", capability_id),
                    false,
                ));
            };
            (meta.server.clone(), meta.tool.clone())
        };

        let tx = {
            let servers_guard = self.state.servers.read().await;
            let Some(tx) = servers_guard.get(&server).cloned() else {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::unsafe_op("not_started"),
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' is unreachable", server),
                    true,
                ));
            };
            tx
        };

        let distill_opts = crate::context_filter::DistillationOptions::from_args(Some(&args));

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::CallTool {
                name: tool,
                params: args,
                input_responses,
                request_state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::unsafe_op("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", server),
                true,
            ));
        }

        match reply_rx.await {
            Ok(Ok(data)) => {
                let distilled_data = crate::context_filter::distill_value(data, &distill_opts);
                Ok(json!({
                    "ok": true,
                    "request_id": request_id,
                    "context": ctx,
                    "trace_id": trace_id,
                    "data": distilled_data,
                    "error": null,
                    "retry": crate::idempotency::RetryMetadata::unsafe_op("completed"),
                }))
            }
            Ok(Err(UpstreamCallError::Timeout)) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::unsafe_op("unknown"),
                "UPSTREAM_TIMEOUT",
                format!("Tool call timed out after {}ms", self.state.tool_timeout_ms),
                true,
            )),
            Ok(Err(UpstreamCallError::Upstream(err))) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::unsafe_op("unknown"),
                "UPSTREAM_ERROR",
                err,
                false,
            )),
            Err(_) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::unsafe_op("unknown"),
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        }
    }

    async fn list_resources_value(&self) -> std::result::Result<Value, String> {
        let res_guard = self.state.resources.read().await;
        let mut resources = res_guard
            .iter()
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
        let trace_id = next_trace_id();
        let ctx = context.unwrap_or_default();
        if !self.state.policy.read().await.allows(&resource_id) {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "INVALID_ARGS",
                format!("Resource '{}' blocked by policy", resource_id),
                false,
            ));
        }

        let (server, uri) = {
            let res_guard = self.state.resources.read().await;
            let Some(meta) = res_guard.get(&resource_id) else {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "RESOURCE_NOT_FOUND",
                    format!("Resource '{}' not found", resource_id),
                    false,
                ));
            };
            (meta.server.clone(), meta.uri.clone())
        };

        let tx = {
            let servers_guard = self.state.servers.read().await;
            let Some(tx) = servers_guard.get(&server).cloned() else {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' is unreachable", server),
                    true,
                ));
            };
            tx
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::ReadResource {
                uri,
                input_responses,
                request_state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", server),
                true,
            ));
        }

        match reply_rx.await {
            Ok(Ok(data)) => Ok(json!({
                "ok": true,
                "request_id": request_id,
                "context": ctx,
                "trace_id": trace_id,
                "data": data,
                "error": null,
                "retry": crate::idempotency::RetryMetadata::safe("completed"),
            })),
            Ok(Err(UpstreamCallError::Timeout)) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_TIMEOUT",
                format!(
                    "Resource read timed out after {}ms",
                    self.state.tool_timeout_ms
                ),
                true,
            )),
            Ok(Err(UpstreamCallError::Upstream(err))) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_ERROR",
                err,
                false,
            )),
            Err(_) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        }
    }

    async fn list_prompts_value(&self) -> std::result::Result<Value, String> {
        let prompts_guard = self.state.prompts.read().await;
        let mut prompts = prompts_guard
            .iter()
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
        let trace_id = next_trace_id();
        let ctx = context.unwrap_or_default();
        if !self.state.policy.read().await.allows(&prompt_id) {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "INVALID_ARGS",
                format!("Prompt '{}' blocked by policy", prompt_id),
                false,
            ));
        }

        let (server, name) = {
            let prompts_guard = self.state.prompts.read().await;
            let Some(meta) = prompts_guard.get(&prompt_id) else {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "PROMPT_NOT_FOUND",
                    format!("Prompt '{}' not found", prompt_id),
                    false,
                ));
            };
            (meta.server.clone(), meta.name.clone())
        };

        let arguments = match arguments {
            Some(Value::Object(map)) => Some(map),
            Some(_) => {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "INVALID_ARGS",
                    "'arguments' must be a JSON object when provided",
                    false,
                ));
            }
            None => None,
        };

        let tx = {
            let servers_guard = self.state.servers.read().await;
            let Some(tx) = servers_guard.get(&server).cloned() else {
                return Ok(error_envelope(
                    trace_id,
                    request_id,
                    Some(ctx),
                    crate::idempotency::RetryMetadata::safe("not_started"),
                    "SERVER_UNREACHABLE",
                    format!("Server '{}' is unreachable", server),
                    true,
                ));
            };
            tx
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::GetPrompt {
                name,
                arguments,
                input_responses,
                request_state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' mailbox is closed", server),
                true,
            ));
        }

        match reply_rx.await {
            Ok(Ok(data)) => Ok(json!({
                "ok": true,
                "request_id": request_id,
                "context": ctx,
                "trace_id": trace_id,
                "data": data,
                "error": null,
                "retry": crate::idempotency::RetryMetadata::safe("completed"),
            })),
            Ok(Err(UpstreamCallError::Timeout)) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_TIMEOUT",
                format!(
                    "Prompt get timed out after {}ms",
                    self.state.tool_timeout_ms
                ),
                true,
            )),
            Ok(Err(UpstreamCallError::Upstream(err))) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "UPSTREAM_ERROR",
                err,
                false,
            )),
            Err(_) => Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("unknown"),
                "INTERNAL_ERROR",
                "Daemon actor task died",
                true,
            )),
        }
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
                    "_meta":{"type":"object"}
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
    let ctx_val = context.unwrap_or_default();
    json!({
        "ok": false,
        "request_id": request_id,
        "context": ctx_val,
        "trace_id": trace_id,
        "data": null,
        "error": {
            "code": code,
            "message": message.into(),
            "retryable": retryable,
        },
        "retry": retry,
    })
}

/// Runs the Warmplane stdio MCP server proxy interface.
///
/// # Arguments
/// * `config` - Loaded `McpConfig` configuration struct.
/// * `config_path` - Path to the config file.
///
/// # Errors
/// Returns an error if initializing upstream state or stdio transport fails.
pub async fn run_mcp_server(config: McpConfig, config_path: impl Into<String>) -> Result<()> {
    let state = initialize_state(config, config_path).await?;
    let server = FacadeMcpServer { state };
    let running = server.serve(stdio()).await?;
    let _ = running.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{facade_tools, invalid_args};

    #[test]
    fn facade_tools_include_all_lightweight_operations() {
        let names = facade_tools()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 10);
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
            CapabilityMeta {
                server: "sqlite".to_string(),
                tool: "read_query".to_string(),
                summary: "Execute read-only SQL queries".to_string(),
                description: "Run SQL SELECT queries against SQLite database".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                tags: vec!["database".to_string(), "sql".to_string()],
                examples: vec![],
            },
        );
        caps.insert(
            "fs.read".to_string(),
            CapabilityMeta {
                server: "fs".to_string(),
                tool: "read_file".to_string(),
                summary: "Read file contents from filesystem".to_string(),
                description: "Read utf-8 contents of a file".to_string(),
                input_schema: serde_json::json!({"type": "object"}),
                tags: vec!["filesystem".to_string()],
                examples: vec![],
            },
        );

        let state = AppState::builder()
            .capabilities(caps)
            .policy(Policy::default())
            .catalog_version("test-ver")
            .build();

        let server = super::FacadeMcpServer { state };
        let res = server
            .search_capabilities_value(Some("SQL database".to_string()), None, None, None, Some(5))
            .await
            .expect("search should succeed");

        assert_eq!(res["version"], "v1");
        assert_eq!(res["query"], "SQL database");
        assert_eq!(res["total"], 1);
        assert_eq!(res["capabilities"][0]["id"], "db.query");
    }
}
