// Rust guideline compliant 2026-11-21

//! Compiles operator-selected OpenAPI operations into upstream capability definitions.

mod compiler;
mod input;
mod parameter;
mod remote;
mod worker;

use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use anyhow::{ensure, Context, Result};
use reqwest::Url;
use rmcp::model::{MetaObject, Tool};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::config::{AliasTarget, OpenApiAuth, OpenApiSourceConfig, ServerConfig};
use crate::daemon::state::AppState;

const MAX_SPEC_BYTES: usize = 2 * 1024 * 1024;
const MAX_OPERATIONS: usize = 100;
const MAX_DEPTH: usize = 64;

pub(crate) async fn is_mutation(state: &AppState, server: &str, tool: &str) -> bool {
    state
        .server_configs
        .read()
        .await
        .get(server)
        .and_then(|config| config.open_api.as_ref())
        .is_some_and(|source| source.mutations.iter().any(|operation| operation == tool))
}

pub(crate) async fn verify_local_revision(
    state: &AppState,
    server: &str,
    config: &ServerConfig,
) -> Result<()> {
    let Some(source) = config
        .open_api
        .as_ref()
        .filter(|source| source.spec_path.is_some())
    else {
        return Ok(());
    };
    let current = compiler::load(&state.config_path, source).await?;
    let prepared = prepare_definition(state, source, current)?;
    ensure!(
        state
            .tool_pins
            .matches_compiled_definitions(server, &prepared.tools),
        "The local spec changed. Reload the source before reviewing this draft"
    );
    Ok(())
}

#[derive(Default)]
pub(crate) struct Mounts(std::sync::Mutex<HashMap<String, Arc<MountAttempt>>>);

struct MountAttempt {
    serial: Arc<Mutex<()>>,
    cancel: CancellationToken,
}

struct MountRegistration<'a> {
    mounts: &'a Mounts,
    server: &'a str,
    attempt: Arc<MountAttempt>,
}

impl Drop for MountRegistration<'_> {
    fn drop(&mut self) {
        self.attempt.cancel.cancel();
        let mut active = self.mounts.active();
        if active
            .get(self.server)
            .is_some_and(|current| Arc::ptr_eq(current, &self.attempt))
        {
            active.remove(self.server);
        }
    }
}

impl Mounts {
    fn active(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<MountAttempt>>> {
        self.0.lock().unwrap_or_else(|lock| lock.into_inner())
    }

    fn begin<'a>(&'a self, server: &'a str) -> MountRegistration<'a> {
        let mut active = self.active();
        let serial = active
            .get(server)
            .map_or_else(Default::default, |previous| {
                previous.cancel.cancel();
                previous.serial.clone()
            });
        let attempt = Arc::new(MountAttempt {
            serial,
            cancel: CancellationToken::new(),
        });
        active.insert(server.to_owned(), attempt.clone());
        MountRegistration {
            mounts: self,
            server,
            attempt,
        }
    }

    pub(crate) fn cancel(&self, server: &str) {
        if let Some(attempt) = self.active().get(server) {
            attempt.cancel.cancel();
        }
    }

    pub(crate) fn pending_servers(&self) -> Vec<String> {
        self.active()
            .iter()
            .filter(|(_, attempt)| !attempt.cancel.is_cancelled())
            .map(|(server, _)| server.clone())
            .collect()
    }
}

pub(crate) async fn mount(
    state: &AppState,
    server_id: &str,
    config: &ServerConfig,
    source_config: &OpenApiSourceConfig,
    aliases: &HashMap<String, AliasTarget>,
    origin: MountOrigin<'_>,
) -> Result<()> {
    mount_prepared(
        state,
        server_id,
        config,
        aliases,
        origin,
        prepare(state, server_id, config, source_config),
    )
    .await
}

async fn mount_prepared(
    state: &AppState,
    server_id: &str,
    config: &ServerConfig,
    aliases: &HashMap<String, AliasTarget>,
    origin: MountOrigin<'_>,
    prepare: impl std::future::Future<Output = Result<Prepared>>,
) -> Result<()> {
    let registration = {
        let _lifecycle = state.upstream_lifecycle.lock().await;
        let previous = state.server_configs.read().await.get(server_id).cloned();
        if crate::daemon::lifecycle::must_withdraw_openapi(previous.as_ref(), config) {
            state.unmount_locked(server_id, true).await?;
        }
        state.open_api_mounts.begin(server_id)
    };
    let attempt = &registration.attempt;
    let prepared = tokio::select! {
        biased;
        _ = attempt.cancel.cancelled() => return Ok(()),
        _ = state.shutdown_token.cancelled() => return Ok(()),
        result = async {
            let _serial = attempt.serial.lock().await;
            prepare.await
        } => result,
    };
    let _lifecycle = state.upstream_lifecycle.lock().await;
    if attempt.cancel.is_cancelled() || state.shutdown_token.is_cancelled() {
        return Ok(());
    }
    let result = match prepared {
        Ok(prepared) => publish(state, server_id, config, prepared, aliases).await,
        Err(error) => Err(error),
    };
    if let Err(error) = &result {
        record_mount_failure(state, server_id, config, error).await;
    }
    audit_mount(state, server_id, origin, result.is_ok()).await;
    result
}

pub(crate) enum MountOrigin<'a> {
    Startup,
    Management(Option<&'a str>),
}

async fn audit_mount(state: &AppState, server: &str, origin: MountOrigin<'_>, valid: bool) {
    use crate::audit::{AuditEventStatus, AuditEventType, RawAuditEvent};

    let (trigger, actor_id) = match origin {
        MountOrigin::Startup => ("startup", None),
        MountOrigin::Management(actor) => ("configuration_reload", actor.map(str::to_owned)),
    };
    let revision = state
        .server_statuses
        .read()
        .await
        .get(server)
        .and_then(|status| status.get("active_revision"))
        .cloned();
    state.audit_handle.send(RawAuditEvent {
        event_type: AuditEventType::ConfigMutation,
        trace_id: crate::http_v1::next_trace_id(),
        request_id: None,
        actor_id,
        work_item_id: None,
        client_ip: None,
        server_id: Some(server.to_owned()),
        capability_id: None,
        resource_uri: None,
        sanitized_args: Some(json!({"action": "refresh_openapi_source", "trigger": trigger})),
        sanitized_response: Some(json!({
            "state": if valid { "validated" } else { "refresh_failed" },
            "active_revision": revision,
        })),
        execution_latency_us: None,
        status: if valid {
            AuditEventStatus::Success
        } else {
            AuditEventStatus::Failed
        },
        error_code: (!valid).then(|| "OPENAPI_SOURCE_REFRESH_FAILED".to_owned()),
        error_message: None,
        operator_id: None,
        approval_ticket_id: None,
        idempotency_key: None,
        is_replay: None,
    });
}

struct Prepared {
    source: compiler::Source,
    worker: worker::HttpWorker,
    tools: Vec<Tool>,
}

async fn prepare(
    state: &AppState,
    server_id: &str,
    config: &ServerConfig,
    source_config: &OpenApiSourceConfig,
) -> Result<Prepared> {
    validate_source(server_id, config, source_config)?;
    let remote = source_config.remote_spec.is_some();
    if remote {
        ensure!(
            state.tool_pins.has_storage(),
            "Remote OpenAPI sources require persistent tool pin storage"
        );
    }
    if !source_config.mutations.is_empty() {
        ensure!(
            state.idempotency_store.is_persistent(),
            "OpenAPI mutations require persistent idempotency state"
        );
    }
    let source = compiler::load(&state.config_path, source_config).await?;
    prepare_definition(state, source_config, source)
}

fn prepare_definition(
    state: &AppState,
    source_config: &OpenApiSourceConfig,
    source: compiler::Source,
) -> Result<Prepared> {
    let worker = worker::HttpWorker::new(&source_config.auth, state.tool_timeout_ms)?;
    let tools: Vec<_> = source
        .operations
        .values()
        .map(|operation| {
            let contract = MetaObject(
                [(
                    "warmplane/openapi".to_owned(),
                    json!({
                        "method": operation.method.as_str(),
                        "baseUrl": source.base.as_str(),
                        "pathTemplate": operation.template,
                        "responses": operation.responses,
                        "authBinding": source_config.auth
                    }),
                )]
                .into_iter()
                .collect(),
            );
            Tool::new(
                operation.id.clone(),
                operation.description.clone(),
                operation.input_schema(),
            )
            .with_meta(contract)
        })
        .collect();
    Ok(Prepared {
        source,
        worker,
        tools,
    })
}

async fn record_mount_failure(
    state: &AppState,
    server: &str,
    config: &ServerConfig,
    error: &anyhow::Error,
) {
    let serving = state.servers.read().await.contains_key(server);
    let mut statuses = state.server_statuses.write().await;
    let status = statuses
        .entry(server.to_owned())
        .or_insert_with(|| json!({"transport": "openapi", "operation_count": 0}));
    status["status"] = json!("degraded");
    status["error"] = json!(error.to_string());
    status["serving_previous_revision"] = json!(serving);
    drop(statuses);
    if !serving {
        state
            .server_configs
            .write()
            .await
            .insert(server.to_owned(), config.clone());
    }
}

async fn publish(
    state: &AppState,
    server: &str,
    config: &ServerConfig,
    prepared: Prepared,
    aliases: &HashMap<String, AliasTarget>,
) -> Result<()> {
    let Prepared {
        source,
        worker,
        tools,
    } = prepared;
    validate_aliases(server, &tools, aliases)?;
    let capabilities = crate::supervisor::capability_metadata(&tools, server, aliases);
    let count = capabilities.len();
    let remote = config
        .open_api
        .as_ref()
        .is_some_and(|source| source.remote_spec.is_some());
    let revision = hex::encode(Sha256::digest(serde_json::to_vec(&tools)?));
    let identifiers: BTreeSet<_> = capabilities.iter().map(|(id, _)| id).collect();
    ensure!(
        identifiers.len() == count,
        "OpenAPI capabilities have conflicting aliases"
    );
    {
        let mut entries = state.capabilities.write().await;
        let mut workers = state.servers.write().await;
        ensure!(
            identifiers
                .iter()
                .all(|id| entries.get(*id).is_none_or(|entry| entry.server == server)),
            "OpenAPI capability conflicts with another source"
        );
        let session =
            state
                .tool_pins
                .publish_compiled(server, config.require_tool_pins, remote, &tools)?;
        let sender = worker.spawn(state.clone(), server.to_owned(), source, session);
        entries.retain(|_, entry| entry.server != server);
        entries.extend(capabilities);
        workers.insert(server.to_owned(), sender);
        state.record_tool_catalog_events(server, &entries);
    }
    state
        .server_configs
        .write()
        .await
        .insert(server.to_owned(), config.clone());
    state.server_statuses.write().await.insert(
        server.to_owned(),
        json!({
            "transport": "openapi",
            "status": "connected",
            "operation_count": count,
            "active_revision": revision,
            "last_validated_at_ms": now_millis(),
            "serving_previous_revision": false,
            "remote_spec": remote
        }),
    );
    state
        .circuit_breakers
        .get_or_create(server, config.resilience.clone().unwrap_or_default())
        .await;
    state.refresh_server_catalog(server).await;
    state.notify_tools_changed();
    Ok(())
}

fn validate_aliases(
    server: &str,
    tools: &[Tool],
    aliases: &HashMap<String, AliasTarget>,
) -> Result<()> {
    for tool in tools {
        let id = format!("{server}.{}", tool.name);
        ensure!(
            aliases
                .values()
                .filter(|target| target.target() == id)
                .take(2)
                .count()
                < 2,
            "OpenAPI capability has multiple aliases"
        );
    }
    Ok(())
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

pub(crate) fn validate_source(
    server_id: &str,
    server: &ServerConfig,
    source: &OpenApiSourceConfig,
) -> Result<()> {
    ensure!(
        server.command.is_none()
            && server.url.is_none()
            && server.auth.is_none()
            && server.args.is_empty()
            && server.env.is_empty()
            && server.headers.is_empty()
            && server.protocol_version.is_none()
            && server.allow_stateless.is_none(),
        "OpenAPI source '{server_id}' cannot include MCP transport settings"
    );
    validate_spec_location(server_id, server, source)?;
    validate_operations(server_id, source)?;
    validate_auth(&source.auth)?;
    invocation_base(source).map(|_| ())
}

fn validate_spec_location(
    server_id: &str,
    server: &ServerConfig,
    source: &OpenApiSourceConfig,
) -> Result<()> {
    match (&source.spec_path, &source.remote_spec) {
        (Some(path), None) if !path.as_os_str().is_empty() => {}
        (None, Some(remote)) => {
            ensure!(
                server.require_tool_pins,
                "Remote OpenAPI specs require tool pins"
            );
            remote::validate(remote)?;
        }
        _ => anyhow::bail!("OpenAPI source '{server_id}' requires exactly one spec location"),
    }
    Ok(())
}

fn validate_operations(server_id: &str, source: &OpenApiSourceConfig) -> Result<()> {
    ensure!(
        (1..=MAX_OPERATIONS).contains(&source.operations.len()),
        "OpenAPI source '{server_id}' must select between 1 and {MAX_OPERATIONS} operations"
    );
    let distinct = source.operations.iter().collect::<BTreeSet<_>>();
    let mutations = source.mutations.iter().collect::<BTreeSet<_>>();
    ensure!(
        mutations.len() == source.mutations.len()
            && mutations.is_subset(&distinct)
            && (source.remote_spec.is_none() || mutations.is_empty()),
        "OpenAPI mutations must be separately allowlisted on a local source"
    );
    ensure!(
        distinct.len() == source.operations.len()
            && valid_name(server_id)
            && distinct
                .iter()
                .all(|id| { valid_name(id) && server_id.len() + 1 + id.len() <= 128 }),
        "OpenAPI source '{server_id}' has duplicate or invalid capability identifiers"
    );
    Ok(())
}

fn validate_auth(auth: &OpenApiAuth) -> Result<()> {
    if let OpenApiAuth::Bearer { token_env } = auth {
        ensure!(
            !token_env.trim().is_empty() && !token_env.contains(['\0', '=']),
            "OpenAPI source requires a valid credential environment variable"
        );
    }
    Ok(())
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn invocation_base(source: &OpenApiSourceConfig) -> Result<Url> {
    let url = Url::parse(&source.base_url)
        .ok()
        .context("OpenAPI invocation base URL is invalid")?;
    let loopback = url
        .host_str()
        .and_then(|host| {
            host.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .ok()
        })
        .is_some_and(|address| address.is_loopback());
    ensure!(
        (url.scheme() == "https" || (url.scheme() == "http" && loopback))
            && url.host().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "OpenAPI invocation requires an HTTPS base URL or a loopback HTTP fixture URL"
    );
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::Mounts;

    #[test]
    fn superseded_mount_cannot_clear_replacement_and_withdrawal_cancels_it() {
        let mounts = Mounts::default();
        let first = mounts.begin("inventory");
        let replacement = mounts.begin("inventory");
        assert!(first.attempt.cancel.is_cancelled());
        assert!(!replacement.attempt.cancel.is_cancelled());
        drop(first);
        assert_eq!(mounts.pending_servers(), ["inventory"]);
        mounts.cancel("inventory");
        assert!(replacement.attempt.cancel.is_cancelled());
        drop(replacement);
        assert!(mounts.active().is_empty());
    }
}
