use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::Arc,
};

use anyhow::Result;
use rmcp::{
    model::{
        AnnotateAble, CallToolRequestParams, CallToolResult, Content, GetPromptRequestParams,
        GetPromptResult, ListResourcesResult, ListToolsResult, Prompt, RawResource,
        ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo, Tool,
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
const TOOL_CAPABILITY_DESCRIBE: &str = "capability_describe";
const TOOL_CAPABILITY_CALL: &str = "capability_call";
const TOOL_RESOURCES_LIST: &str = "resources_list";
const TOOL_RESOURCE_READ: &str = "resource_read";
const TOOL_PROMPTS_LIST: &str = "prompts_list";
const TOOL_PROMPT_GET: &str = "prompt_get";

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct FacadeMcpServer {
    state: AppState,
}

impl ServerHandler for FacadeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            instructions: Some(
                "Warmplane MCP facade server with deterministic tools/resources/prompts surfaces"
                    .to_string(),
            ),
            ..Default::default()
        }
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
    ) -> std::result::Result<CallToolResult, McpError> {
        let args = request.arguments.unwrap_or_default();

        let output = match request.name.as_ref() {
            TOOL_CAPABILITIES_LIST => self.list_capabilities_value().await,
            TOOL_CAPABILITY_DESCRIBE => {
                let Some(id) = args.get("id").and_then(Value::as_str) else {
                    return Ok(CallToolResult::structured_error(invalid_args(
                        "Missing required field 'id'",
                    )));
                };
                self.describe_capability_value(id.to_string()).await
            }
            TOOL_CAPABILITY_CALL => {
                let Some(capability_id) = args.get("capability_id").and_then(Value::as_str) else {
                    return Ok(CallToolResult::structured_error(invalid_args(
                        "Missing required field 'capability_id'",
                    )));
                };
                let Some(call_args) = args.get("args") else {
                    return Ok(CallToolResult::structured_error(invalid_args(
                        "Missing required field 'args'",
                    )));
                };
                if !call_args.is_object() {
                    return Ok(CallToolResult::structured_error(invalid_args(
                        "'args' must be a JSON object",
                    )));
                }
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                self.call_capability_value(
                    capability_id.to_string(),
                    call_args.clone(),
                    request_id,
                    context,
                )
                .await
            }
            TOOL_RESOURCES_LIST => self.list_resources_value().await,
            TOOL_RESOURCE_READ => {
                let Some(resource_id) = args.get("resource_id").and_then(Value::as_str) else {
                    return Ok(CallToolResult::structured_error(invalid_args(
                        "Missing required field 'resource_id'",
                    )));
                };
                let request_id = args
                    .get("request_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let context: Option<crate::context::RequestContext> = args
                    .get("_meta")
                    .or_else(|| args.get("context"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                self.read_resource_value(resource_id.to_string(), request_id, context)
                    .await
            }
            TOOL_PROMPTS_LIST => self.list_prompts_value().await,
            TOOL_PROMPT_GET => {
                let Some(prompt_id) = args.get("prompt_id").and_then(Value::as_str) else {
                    return Ok(CallToolResult::structured_error(invalid_args(
                        "Missing required field 'prompt_id'",
                    )));
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
                self.get_prompt_value(prompt_id.to_string(), arguments, request_id, context)
                    .await
            }
            _ => {
                return Err(McpError::invalid_params(
                    format!("Unknown tool '{}'.", request.name),
                    None,
                ));
            }
        };

        match output {
            Ok(value) => Ok(CallToolResult::structured(value)),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<ListResourcesResult, McpError> {
        let items = self
            .state
            .resources
            .values()
            .map(|r| {
                RawResource {
                    uri: r.uri.clone(),
                    name: r.name.clone(),
                    title: None,
                    description: r.description.clone(),
                    mime_type: r.mime_type.clone(),
                    size: None,
                    icons: None,
                    meta: None,
                }
                .no_annotation()
            })
            .collect::<Vec<_>>();
        Ok(ListResourcesResult::with_all_items(items))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<ReadResourceResult, McpError> {
        let Some((_, meta)) = self
            .state
            .resources
            .iter()
            .find(|(_, m)| m.uri == request.uri)
        else {
            return Err(McpError::invalid_params(
                format!("Resource URI '{}' not found", request.uri),
                None,
            ));
        };

        let tx = self
            .state
            .servers
            .get(&meta.server)
            .ok_or_else(|| McpError::internal_error("Target server unreachable", None))?;

        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(ServerMsg::ReadResource {
            uri: meta.uri.clone(),
            reply: reply_tx,
        })
        .await
        .map_err(|_| McpError::internal_error("Server mailbox closed", None))?;

        match reply_rx.await {
            Ok(Ok(value)) => serde_json::from_value(value).map_err(|e| {
                McpError::internal_error(format!("Invalid resource payload: {e}"), None)
            }),
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
        let prompts = self
            .state
            .prompts
            .values()
            .map(|p| Prompt {
                name: p.name.clone(),
                title: p.title.clone(),
                description: p.description.clone(),
                arguments: serde_json::from_value(Value::Array(p.arguments.clone())).ok(),
                icons: None,
                meta: None,
            })
            .collect::<Vec<_>>();
        Ok(rmcp::model::ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<GetPromptResult, McpError> {
        let Some((_, prompt_meta)) = self
            .state
            .prompts
            .iter()
            .find(|(_, p)| p.name == request.name)
        else {
            return Err(McpError::invalid_params(
                format!("Prompt '{}' not found", request.name),
                None,
            ));
        };

        let tx = self
            .state
            .servers
            .get(&prompt_meta.server)
            .ok_or_else(|| McpError::internal_error("Target server unreachable", None))?;

        let (reply_tx, reply_rx) = oneshot::channel();
        tx.send(ServerMsg::GetPrompt {
            name: request.name,
            arguments: request.arguments,
            reply: reply_tx,
        })
        .await
        .map_err(|_| McpError::internal_error("Server mailbox closed", None))?;

        match reply_rx.await {
            Ok(Ok(value)) => serde_json::from_value(value).map_err(|e| {
                McpError::internal_error(format!("Invalid prompt payload: {e}"), None)
            }),
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
        let mut capabilities = self
            .state
            .capabilities
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

    async fn describe_capability_value(&self, id: String) -> std::result::Result<Value, String> {
        match self.state.capabilities.get(&id) {
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
    ) -> std::result::Result<Value, String> {
        let trace_id = next_trace_id();
        let ctx = context.unwrap_or_default();
        if !self.state.policy.allows(&capability_id) {
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

        let Some(meta) = self.state.capabilities.get(&capability_id) else {
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

        let Some(tx) = self.state.servers.get(&meta.server) else {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::unsafe_op("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unreachable", meta.server),
                true,
            ));
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::CallTool {
                name: meta.tool.clone(),
                params: args,
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
                format!("Server '{}' mailbox is closed", meta.server),
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
                "retry": crate::idempotency::RetryMetadata::unsafe_op("completed"),
            })),
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
        let mut resources = self
            .state
            .resources
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
    ) -> std::result::Result<Value, String> {
        let trace_id = next_trace_id();
        let ctx = context.unwrap_or_default();
        if !self.state.policy.allows(&resource_id) {
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

        let Some(meta) = self.state.resources.get(&resource_id) else {
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

        let Some(tx) = self.state.servers.get(&meta.server) else {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unreachable", meta.server),
                true,
            ));
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::ReadResource {
                uri: meta.uri.clone(),
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
                format!("Server '{}' mailbox is closed", meta.server),
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
        let mut prompts = self
            .state
            .prompts
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
    ) -> std::result::Result<Value, String> {
        let trace_id = next_trace_id();
        let ctx = context.unwrap_or_default();
        if !self.state.policy.allows(&prompt_id) {
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

        let Some(meta) = self.state.prompts.get(&prompt_id) else {
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

        let Some(tx) = self.state.servers.get(&meta.server) else {
            return Ok(error_envelope(
                trace_id,
                request_id,
                Some(ctx),
                crate::idempotency::RetryMetadata::safe("not_started"),
                "SERVER_UNREACHABLE",
                format!("Server '{}' is unreachable", meta.server),
                true,
            ));
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        if tx
            .send(ServerMsg::GetPrompt {
                name: meta.name.clone(),
                arguments,
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
                format!("Server '{}' mailbox is closed", meta.server),
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

pub async fn run_mcp_server(config: McpConfig) -> Result<()> {
    let state = initialize_state(config).await?;
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
        assert_eq!(names.len(), 7);
        assert!(names.contains(&"capabilities_list".to_string()));
        assert!(names.contains(&"capability_describe".to_string()));
        assert!(names.contains(&"capability_call".to_string()));
        assert!(names.contains(&"resources_list".to_string()));
        assert!(names.contains(&"resource_read".to_string()));
        assert!(names.contains(&"prompts_list".to_string()));
        assert!(names.contains(&"prompt_get".to_string()));
    }

    #[test]
    fn invalid_args_envelope_has_expected_shape() {
        let payload = invalid_args("bad input");
        assert_eq!(payload["ok"], false);
        assert_eq!(payload["error"]["code"], "INVALID_ARGS");
        assert_eq!(payload["error"]["message"], "bad input");
        assert_eq!(payload["data"], serde_json::Value::Null);
    }
}
