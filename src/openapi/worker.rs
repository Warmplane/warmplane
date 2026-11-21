// Rust guideline compliant 2026-11-21

//! Runs compiled HTTP operations behind the existing upstream worker interface.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{header, Client, Response, Url};
use serde_json::{json, Map, Value};
use tokio::sync::mpsc;

use super::compiler::{Operation, Segment, Source};
use super::parameter::{self, Location};
use crate::config::OpenApiAuth;
use crate::daemon::state::AppState;
use crate::daemon::types::{ServerMsg, ToolDispatchAccess, UpstreamCallError};
use crate::tool_pins::PinSession;

const MAX_REQUEST_TARGET_BYTES: usize = 8 * 1024;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;

pub(super) struct HttpWorker {
    client: Client,
    authorization: Option<header::HeaderValue>,
}

impl HttpWorker {
    pub(super) fn new(auth: &OpenApiAuth, tool_timeout_ms: u64) -> Result<Self> {
        let authorization = authorization(auth)?;
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_millis(tool_timeout_ms.min(10_000)))
            .build()
            .context("Could not prepare the OpenAPI invocation client")?;
        Ok(Self {
            client,
            authorization,
        })
    }

    fn request(
        &self,
        base: &Url,
        operation: &Operation,
        args: &Value,
    ) -> Result<reqwest::RequestBuilder, UpstreamCallError> {
        let url = invocation_url(base, operation, args)?;
        let mut request = self
            .client
            .request(operation.method.clone(), url)
            .header(header::ACCEPT, "application/json");
        if let Some(authorization) = &self.authorization {
            request = request.header(header::AUTHORIZATION, authorization.clone());
        }
        if let Some(value) = args.get("body") {
            let bytes =
                serde_json::to_vec(value).map_err(|_| UpstreamCallError::INVALID_ARGUMENTS)?;
            if bytes.len() > MAX_REQUEST_BODY_BYTES {
                return Err(UpstreamCallError::INVALID_ARGUMENTS);
            }
            request = request
                .header(header::CONTENT_TYPE, "application/json")
                .body(bytes);
        }
        Ok(request)
    }

    pub(super) fn spawn(
        self,
        state: AppState,
        server: String,
        source: Source,
        session: PinSession,
    ) -> mpsc::Sender<ServerMsg> {
        let (sender, mut receiver) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = state.shutdown_token.cancelled() => break,
                    message = receiver.recv() => match message {
                        Some(message) => self.handle(&state, &server, &source, session, message).await,
                        None => break,
                    }
                }
            }
        });
        sender
    }

    async fn handle(
        &self,
        state: &AppState,
        server: &str,
        source: &Source,
        session: PinSession,
        message: ServerMsg,
    ) {
        match message {
            ServerMsg::CallTool {
                name,
                tool_definition,
                dispatch_claim,
                dispatch_access,
                params,
                reply,
                ..
            } => {
                let operation = source.operations.get(&name).filter(|_| {
                    state
                        .tool_pins
                        .verify_dispatch(server, Some(session), &name, tool_definition.as_deref())
                        .is_ok()
                });
                let result = match operation {
                    Some(operation) => {
                        self.dispatch(Dispatch {
                            state,
                            server,
                            session,
                            operation,
                            base: &source.base,
                            args: &params,
                            binding: tool_definition.as_deref(),
                            claim: dispatch_claim,
                            access: dispatch_access,
                        })
                        .await
                    }
                    None => Err(UpstreamCallError::ToolDefinitionChanged),
                };
                let _ = reply.send(result);
            }
            ServerMsg::ReadResource { reply, .. } | ServerMsg::GetPrompt { reply, .. } => {
                let _ = reply.send(Err(upstream_error("OpenAPI sources only expose tools")));
            }
        }
    }

    async fn dispatch(&self, call: Dispatch<'_>) -> Result<Value, UpstreamCallError> {
        let mutation = call.operation.method != reqwest::Method::GET;
        call.verify_access(mutation).await?;
        let request = self.request(call.base, call.operation, call.args)?;
        if mutation {
            let (key, claim) = call
                .claim
                .as_ref()
                .ok_or(UpstreamCallError::RESERVATION_FAILED)?;
            call.state
                .idempotency_store
                .guard_dispatch(key, *claim)
                .await
                .map_err(|_| UpstreamCallError::RESERVATION_FAILED)?;
            call.verify_access(mutation).await?;
        }
        call.state
            .tool_pins
            .verify_dispatch(
                call.server,
                Some(call.session),
                &call.operation.id,
                call.binding,
            )
            .map_err(|_| UpstreamCallError::ToolDefinitionChanged)?;
        let result = async {
            let response = request.send().await.map_err(transport_error)?;
            read_result(response, call.operation).await
        }
        .await
        .map_err(|error| {
            if mutation {
                UpstreamCallError::OUTCOME_UNKNOWN
            } else {
                error
            }
        })?;
        Ok(json!({"content": [], "structuredContent": result, "isError": false}))
    }
}

struct Dispatch<'a> {
    state: &'a AppState,
    server: &'a str,
    session: PinSession,
    operation: &'a Operation,
    base: &'a Url,
    args: &'a Value,
    binding: Option<&'a str>,
    claim: Option<(String, crate::idempotency::IdempotencyClaim)>,
    access: Option<ToolDispatchAccess>,
}

impl Dispatch<'_> {
    async fn verify_access(&self, mutation: bool) -> Result<(), UpstreamCallError> {
        let Some(access) = &self.access else {
            return if mutation {
                Err(UpstreamCallError::DISPATCH_DENIED)
            } else {
                Ok(())
            };
        };
        if access
            .allows(self.state, self.server, &self.operation.id)
            .await
        {
            Ok(())
        } else {
            Err(UpstreamCallError::DISPATCH_DENIED)
        }
    }
}

impl ToolDispatchAccess {
    async fn allows(&self, state: &AppState, server: &str, tool: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if self
            .grant_expires_at_epoch_secs
            .is_some_and(|expiry| expiry <= now)
        {
            return false;
        }
        let matches = state
            .capabilities
            .read()
            .await
            .get(&self.capability)
            .is_some_and(|meta| meta.server == server && meta.tool == tool);
        if !matches {
            return false;
        }
        let global = state.policy.read().await.clone();
        let mut policy = state
            .rbac_engine
            .compute_effective_policy(&self.role, &global);
        if let Some(id) = self.profile.as_deref() {
            let profiles = state.profiles.read().await;
            let Some(profile) = profiles
                .get(id)
                .filter(|profile| profile.servers.iter().any(|item| item == server))
            else {
                return false;
            };
            let overlay = crate::daemon::Policy::from_config(profile.policy.clone());
            policy = policy.merge_with_profile(Some(&overlay));
        }
        policy.allows(&self.capability)
            && (self.approval_completed || !policy.requires_approval(&self.capability))
    }
}

fn invocation_url(
    base: &Url,
    operation: &Operation,
    args: &Value,
) -> Result<Url, UpstreamCallError> {
    let args = args
        .as_object()
        .ok_or(UpstreamCallError::INVALID_ARGUMENTS)?;
    validate_arguments(operation, args)?;
    let mut url = base.clone();
    append_path(
        &mut url,
        &operation.segments,
        args.get("path").and_then(Value::as_object),
    )?;
    if let Some(query) = args
        .get("query")
        .and_then(Value::as_object)
        .filter(|query| !query.is_empty())
    {
        url.query_pairs_mut()
            .extend_pairs(query.iter().map(|(key, value)| (key, scalar_text(value))));
    }
    let prefix = base.path().trim_end_matches('/');
    let target_len = url.path().len() + url.query().map_or(0, |query| 1 + query.len());
    if target_len > MAX_REQUEST_TARGET_BYTES
        || url.origin() != base.origin()
        || !url
            .path()
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('/'))
    {
        return Err(UpstreamCallError::INVALID_ARGUMENTS);
    }
    Ok(url)
}

fn append_path(
    url: &mut Url,
    template: &[Segment],
    values: Option<&Map<String, Value>>,
) -> Result<(), UpstreamCallError> {
    let mut segments = url
        .path_segments_mut()
        .map_err(|_| UpstreamCallError::INVALID_ARGUMENTS)?;
    segments.pop_if_empty();
    for segment in template {
        match segment {
            Segment::Literal(value) => segments.push(value),
            Segment::Parameter(name) => {
                let value = values
                    .and_then(|values| values.get(name))
                    .ok_or(UpstreamCallError::INVALID_ARGUMENTS)?;
                let value = scalar_text(value);
                if !valid_path_value(&value) {
                    return Err(UpstreamCallError::INVALID_ARGUMENTS);
                }
                segments.push(&value)
            }
        };
    }
    Ok(())
}

fn valid_path_value(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && !value.contains(['/', '\\', '%'])
        && !value.chars().any(char::is_control)
}

fn validate_arguments(
    operation: &Operation,
    args: &Map<String, Value>,
) -> Result<(), UpstreamCallError> {
    for (section, values) in args {
        let location = match section.as_str() {
            "path" => Location::Path,
            "query" => Location::Query,
            "body"
                if operation
                    .body
                    .as_ref()
                    .is_some_and(|body| body.matches(values)) =>
            {
                continue;
            }
            _ => return Err(UpstreamCallError::INVALID_ARGUMENTS),
        };
        if !known_parameters(operation, location, values) {
            return Err(UpstreamCallError::INVALID_ARGUMENTS);
        }
    }
    if operation.body.as_ref().is_some_and(|body| body.required) && !args.contains_key("body") {
        return Err(UpstreamCallError::INVALID_ARGUMENTS);
    }
    for param in &operation.parameters {
        let value = args
            .get(param.location.key())
            .and_then(|values| values.get(&param.name));
        if !value.map_or(!param.required, |value| {
            parameter::matches_schema(value, &param.schema)
        }) {
            return Err(UpstreamCallError::INVALID_ARGUMENTS);
        }
    }
    Ok(())
}

fn known_parameters(operation: &Operation, location: Location, values: &Value) -> bool {
    let Some(values) = values.as_object() else {
        return false;
    };
    let mut params = operation
        .parameters
        .iter()
        .filter(|param| param.location == location)
        .peekable();
    params.peek().is_some()
        && values
            .keys()
            .all(|name| params.clone().any(|param| &param.name == name))
}

fn scalar_text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

async fn read_result(
    mut response: Response,
    operation: &Operation,
) -> Result<Value, UpstreamCallError> {
    let status = response.status().as_u16();
    if !operation.responses.contains_key(&status) {
        return Err(upstream_error(
            "OpenAPI invocation returned an undeclared status",
        ));
    }
    if status == 204 {
        return Ok(Value::Null);
    }
    let media_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next());
    if !media_type.is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json")) {
        return Err(upstream_error(
            "OpenAPI invocation returned an unsupported content type",
        ));
    }
    let bytes = bounded_body(&mut response, MAX_RESPONSE_BYTES).await?;
    serde_json::from_slice(&bytes)
        .map_err(|_| upstream_error("OpenAPI invocation returned malformed JSON"))
}

pub(super) async fn bounded_body(
    response: &mut Response,
    limit: usize,
) -> Result<Vec<u8>, UpstreamCallError> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
        if chunk.len() > limit - bytes.len() {
            return Err(upstream_error(
                "OpenAPI HTTP response exceeds the byte limit",
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn upstream_error(message: &str) -> UpstreamCallError {
    UpstreamCallError::Upstream(message.to_owned())
}

fn transport_error(error: reqwest::Error) -> UpstreamCallError {
    if error.is_timeout() {
        UpstreamCallError::Timeout
    } else {
        upstream_error("OpenAPI HTTP request failed")
    }
}

pub(super) fn authorization(auth: &OpenApiAuth) -> Result<Option<header::HeaderValue>> {
    Ok(match auth {
        OpenApiAuth::None => None,
        OpenApiAuth::Bearer { token_env } => {
            let token = std::env::var(token_env)
                .ok()
                .filter(|token| !token.is_empty())
                .context("OpenAPI invocation credential is unavailable")?;
            let mut value = header::HeaderValue::from_str(&format!("Bearer {token}"))
                .ok()
                .context("OpenAPI invocation credential is invalid")?;
            value.set_sensitive(true);
            Some(value)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_request_target_size_is_checked_before_dispatch() {
        let base = Url::parse("https://inventory.example.test").unwrap();
        let operation = Operation {
            id: "listAssets".to_owned(),
            description: String::new(),
            template: "/assets".to_owned(),
            method: reqwest::Method::GET,
            body: None,
            segments: vec![Segment::Literal("assets".to_owned())],
            parameters: vec![parameter::Parameter {
                location: Location::Query,
                name: "tag".to_owned(),
                required: false,
                schema: json!({"type": "string"}).as_object().unwrap().clone(),
            }],
            responses: std::collections::BTreeMap::new(),
        };
        let fixed = "/assets?tag=".len();
        let mut value = "x".repeat(MAX_REQUEST_TARGET_BYTES - fixed);
        assert!(invocation_url(&base, &operation, &json!({"query":{"tag":value}})).is_ok());
        value.push('x');
        assert!(invocation_url(&base, &operation, &json!({"query":{"tag":value}})).is_err());
    }
}
