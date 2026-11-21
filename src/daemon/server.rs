// Rust guideline compliant 2026-11-18
//! Daemon HTTP service initialization, routing, and TCP listener runtime.

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tracing::{error, info};

use crate::{
    config::{AuthConfig, McpConfig, DEFAULT_TOOL_TIMEOUT_MS},
    daemon::{policy::Policy, state::AppState},
    http_v1,
};

fn initialize_audit(
    config: Option<&crate::config::AuditConfig>,
) -> Result<(crate::audit::SharedAuditStore, crate::audit::AuditHandle)> {
    let audit = if let Some(audit_cfg) = config.filter(|cfg| cfg.enabled) {
        let hmac_key_bytes = audit_cfg.resolve_hmac_key().map(|k| k.into_bytes());
        let store = if let Some(ref path) = audit_cfg.file_path {
            Arc::new(crate::audit::AuditStore::open_or_create_with_key(
                path,
                hmac_key_bytes,
            )?)
        } else {
            Arc::new(crate::audit::AuditStore::in_memory_with_key(hmac_key_bytes))
        };
        let siem_dispatcher = audit_cfg
            .siem
            .clone()
            .map(|cfg| crate::audit::SiemDispatcher::new(Some(cfg)));
        let handle = crate::audit::spawn_audit_worker(
            store.clone(),
            siem_dispatcher,
            audit_cfg
                .buffer_capacity
                .unwrap_or(crate::audit::DEFAULT_AUDIT_BUFFER_CAPACITY),
            audit_cfg
                .flush_interval_ms
                .unwrap_or(crate::audit::DEFAULT_AUDIT_FLUSH_INTERVAL_MS),
            audit_cfg
                .max_batch_size
                .unwrap_or(crate::audit::DEFAULT_AUDIT_MAX_BATCH_SIZE),
        );
        (store, handle)
    } else {
        let store = Arc::new(crate::audit::AuditStore::in_memory());
        let handle = crate::audit::spawn_audit_worker(
            store.clone(),
            None,
            crate::audit::DEFAULT_AUDIT_BUFFER_CAPACITY,
            crate::audit::DEFAULT_AUDIT_FLUSH_INTERVAL_MS,
            crate::audit::DEFAULT_AUDIT_MAX_BATCH_SIZE,
        );
        (store, handle)
    };
    Ok(audit)
}

/// Boots all configured upstream MCP servers and constructs global `AppState`.
///
/// # Arguments
/// * `config` - Loaded `McpConfig` instance.
/// * `config_path` - Path to configuration file.
///
/// # Returns
/// An initialized `AppState` instance.
///
/// # Errors
/// Returns an error if an upstream server connection or protocol handshake fails.
pub async fn initialize_state(
    mut config: McpConfig,
    config_path: impl Into<String>,
) -> Result<AppState> {
    crate::config::validate_result_sanitization(&config)?;
    crate::config::validate_approval_configuration(&config)?;
    let config_path_str = config_path.into();
    let tool_timeout_ms = config.tool_timeout_ms.unwrap_or(DEFAULT_TOOL_TIMEOUT_MS);
    let policy = Policy::from_config(config.policy.clone());

    // Resolve persistent state directory if enabled
    let state_dir_opt = match &config.state {
        Some(s) if s.enabled => s.dir.as_deref().or(Some(".warmplane/state")),
        Some(_) => None,
        None => Some(".warmplane/state"),
    };
    let tool_pins = match state_dir_opt {
        Some(directory) => crate::tool_pins::ToolPins::open(
            &std::path::Path::new(directory).join("tool_pins.json"),
        )?,
        None => crate::tool_pins::ToolPins::default(),
    };

    let (
        event_store,
        idempotency_store,
        oauth_registry,
        approval_registry,
        task_registry,
        sampling_registry,
        snapshot_store,
        undo_ledger,
    ) = if let Some(dir_str) = state_dir_opt {
        let state_dir = crate::storage::StateDirectory::new(dir_str);
        let _ = state_dir.ensure_exists();
        let ev = Arc::new(crate::catalog::CatalogEventStore::open_or_create(
            state_dir.catalog_events_file(),
        )?);
        let idm = Arc::new(crate::idempotency::IdempotencyStore::open_or_create(
            state_dir.idempotency_file(),
            std::time::Duration::from_secs(3600),
        )?);
        let oa = crate::oauth2::OAuthRegistry::open_or_create(state_dir.oauth_tokens_file());
        let app = crate::approvals::ApprovalRegistry::open_or_create(state_dir.approvals_file())?;
        let tasks = crate::tasks::TaskRegistry::open_or_create(state_dir.tasks_file())?;
        let samp = crate::sampling::SamplingRegistry::open_or_create(state_dir.sampling_file())?;
        let snapshots = crate::undo::SnapshotStore::open(state_dir.artifacts_dir())?;
        let ledger = crate::undo::UndoLedgerStore::open(state_dir.undo_ledger_file())?;
        (ev, idm, oa, app, tasks, samp, Some(snapshots), Some(ledger))
    } else {
        (
            Arc::new(crate::catalog::CatalogEventStore::new()),
            Arc::new(crate::idempotency::IdempotencyStore::default()),
            crate::oauth2::OAuthRegistry::default(),
            crate::approvals::ApprovalRegistry::default(),
            crate::tasks::TaskRegistry::new(),
            crate::sampling::SamplingRegistry::new(),
            None,
            None,
        )
    };

    // Create shutdown token early for subsystems
    let shutdown_token = tokio_util::sync::CancellationToken::new();

    // Initialize central OAuth registry and proxy server if any server uses OAuth2
    let mut oauth_proxy_port = None;

    let has_oauth2 = config
        .mcp_servers
        .values()
        .any(|s| matches!(s.auth, Some(AuthConfig::Oauth2 { .. })));

    if has_oauth2 {
        let port =
            crate::oauth2::start_oauth_proxy_server(oauth_registry.clone(), shutdown_token.clone())
                .await?;
        oauth_proxy_port = Some(port);
    }

    let search_engine = Arc::new(crate::search::HybridSearchEngine::new());

    let (audit_store, audit_handle) = initialize_audit(config.audit.as_ref())?;

    let auth_token = config.auth_token.take().or_else(|| {
        std::env::var("WARMPLANE_AUTH_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty())
    });

    let rbac_engine = crate::rbac::RbacEngine::new(config.rbac.clone());

    let mut state_builder = AppState::builder()
        .servers_arc(Arc::new(RwLock::new(HashMap::new())))
        .capabilities_arc(Arc::new(RwLock::new(HashMap::new())))
        .tool_pins(tool_pins)
        .resources_arc(Arc::new(RwLock::new(HashMap::new())))
        .prompts_arc(Arc::new(RwLock::new(HashMap::new())))
        .tool_timeout_ms(tool_timeout_ms)
        .policy_arc(Arc::new(RwLock::new(policy)))
        .search_engine(search_engine)
        .catalog_version_arc(Arc::new(RwLock::new(String::new())))
        .event_store(event_store)
        .idempotency_store(idempotency_store)
        .operation_registry(crate::operations::OperationRegistry::new())
        .config_path(config_path_str)
        .server_configs_arc(Arc::new(RwLock::new(HashMap::new())))
        .server_statuses_arc(Arc::new(RwLock::new(HashMap::new())))
        .oauth_proxy_port(oauth_proxy_port)
        .oauth_registry(oauth_registry)
        .approval_registry(approval_registry)
        .task_registry(task_registry)
        .snapshot_store_opt(snapshot_store)
        .undo_ledger_opt(undo_ledger)
        .sampling_registry(sampling_registry)
        .audit_store(audit_store)
        .audit_handle(audit_handle)
        .rbac_engine(rbac_engine)
        .profiles(config.profiles.clone())
        .shutdown_token(shutdown_token);

    if let Some(token) = auth_token {
        state_builder = state_builder.auth_token(token);
    }
    let state = state_builder.build();

    info!(
        server_count = config.mcp_servers.len(),
        "booting upstream MCP servers"
    );

    for (server_id, srv_cfg) in &config.mcp_servers {
        if let Err(e) = mount_initial_source(&state, &config, server_id, srv_cfg).await {
            tracing::warn!(
                server_id = %server_id,
                error = %e,
                "upstream server failed initial mount (operating in degraded mode, supervisor will retry)"
            );

            // Record degraded status so operators & web UI see the failure reason
            let mut statuses_guard = state.server_statuses.write().await;
            statuses_guard.insert(
                server_id.to_string(),
                serde_json::json!({
                    "transport": if srv_cfg.command.is_some() { "stdio" } else { "http" },
                    "protocol_version": srv_cfg.protocol_version.as_deref().unwrap_or(crate::daemon::DEFAULT_MCP_PROTOCOL_VERSION),
                    "status": "degraded",
                    "error": e.to_string()
                }),
            );
            drop(statuses_guard);
            // Also keep server config in state so circuit breakers and supervisors can manage it
            let mut configs_guard = state.server_configs.write().await;
            configs_guard.insert(server_id.to_string(), srv_cfg.clone());
        }
    }

    state.hold_coordinator.recover(&state).await;

    Ok(state)
}

async fn mount_initial_source(
    state: &AppState,
    config: &crate::config::McpConfig,
    id: &str,
    server: &crate::config::ServerConfig,
) -> anyhow::Result<()> {
    if let Some(source) = &server.open_api {
        let _ = crate::openapi::mount(
            state,
            id,
            server,
            source,
            &config.capability_aliases,
            crate::openapi::MountOrigin::Startup,
        )
        .await;
        return Ok(());
    }
    state
        .mount_upstream_server(
            id,
            server,
            &config.capability_aliases,
            &config.resource_aliases,
            &config.prompt_aliases,
        )
        .await
}

/// Starts the HTTP daemon server listening on the specified TCP port.
///
/// # Arguments
/// * `port` - TCP listening port.
/// * `config` - `McpConfig` configuration struct.
/// * `config_path` - Path to the config file.
///
/// # Errors
/// Returns an error if binding TCP socket or server execution fails.
/// Builds the Axum router configuring all API routes, state, and security middleware.
pub fn build_router(app_state: AppState) -> Router {
    Router::new()
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
        .route(
            "/v1/tools/batch_call",
            post(http_v1::handle_batch_call_capabilities),
        )
        .route("/v1/catalog/events", get(http_v1::handle_catalog_events))
        .route(
            "/v1/operations/:id/cancel",
            post(http_v1::handle_cancel_operation),
        )
        .route("/v1/completion/complete", post(http_v1::handle_completion))
        .route(
            "/v1/resources/updates",
            get(http_v1::handle_resource_updates),
        )
        .route(
            "/v1/sampling/create_message",
            post(http_v1::handle_sampling_create_message),
        )
        .route(
            "/v1/sampling/requests",
            get(http_v1::handle_list_sampling_requests),
        )
        .route(
            "/v1/sampling/requests/:id",
            get(http_v1::handle_get_sampling_request),
        )
        .route(
            "/v1/sampling/requests/:id/respond",
            post(http_v1::handle_respond_sampling_request),
        )
        // Configuration & Control Deck Endpoints
        .route("/v1/config", get(http_v1::handle_get_config))
        .route("/v1/config/servers", post(http_v1::handle_upsert_server))
        .route(
            "/v1/config/servers/:id",
            axum::routing::delete(http_v1::handle_delete_server),
        )
        .route(
            "/v1/config/servers/:id/restart",
            post(http_v1::handle_restart_server),
        )
        .route(
            "/v1/config/ecosystem",
            get(http_v1::handle_get_ecosystem_sources),
        )
        .route("/v1/config/import", post(http_v1::handle_import_config))
        .route("/v1/config/alias", post(http_v1::handle_update_alias))
        .route(
            "/v1/config/reversibility/draft",
            post(crate::undo::draft::handle_generate),
        )
        .route(
            "/v1/config/reversibility/record",
            post(crate::undo::draft::record::handle_record),
        )
        .route(
            "/v1/config/reversibility/activate",
            post(crate::undo::draft::record::handle_activate),
        )
        .route("/v1/config/policy", post(http_v1::handle_update_policy))
        .route("/v1/config/tool-pins", get(http_v1::handle_list_tool_pins))
        .route(
            "/v1/config/tool-pins/approve",
            post(http_v1::handle_approve_tool_pin),
        )
        .route("/v1/config/profiles", post(http_v1::handle_upsert_profile))
        .route(
            "/v1/config/profiles/:id",
            axum::routing::delete(http_v1::handle_delete_profile),
        )
        .route("/v1/config/reload", post(http_v1::handle_reload_config))
        // Connected External AI Clients
        .route("/v1/clients", get(http_v1::handle_list_clients))
        .route(
            "/v1/clients/:id/attach",
            post(http_v1::handle_attach_client),
        )
        .route(
            "/v1/clients/:id/detach",
            post(http_v1::handle_detach_client),
        )
        // Secrets Vault & OS Keychain Endpoints
        .route(
            "/v1/secrets",
            get(http_v1::handle_list_secrets).post(http_v1::handle_upsert_secret),
        )
        .route(
            "/v1/secrets/:key",
            axum::routing::delete(http_v1::handle_delete_secret),
        )
        // Human-in-the-Loop (HITL) Approvals Endpoints
        .route("/v1/approvals", get(http_v1::handle_list_approvals))
        .route("/v1/approvals/:id", get(http_v1::handle_get_approval))
        .route(
            "/v1/approvals/:id/approve",
            post(http_v1::handle_approve_ticket),
        )
        .route(
            "/v1/approvals/:id/reject",
            post(http_v1::handle_reject_ticket),
        )
        // Webhooks & ChatOps Endpoints
        .route(
            "/v1/webhooks/callbacks",
            post(http_v1::handle_webhook_callback),
        )
        .route(
            "/v1/webhooks/actions",
            post(http_v1::handle_webhook_action).layer(axum::extract::DefaultBodyLimit::max(
                http_v1::webhooks_api::MAX_ACTION_BODY_BYTES,
            )),
        )
        .route("/v1/webhooks/test", post(http_v1::handle_test_webhook))
        // SEP-2663 Tasks Endpoints
        .route("/v1/tasks", get(http_v1::handle_list_tasks))
        .route("/v1/tasks/:id", get(http_v1::handle_get_task))
        .route("/v1/tasks/:id/update", post(http_v1::handle_update_task))
        .route("/v1/tasks/:id/cancel", post(http_v1::handle_cancel_task))
        // Undo inspection and operator actions.
        .route("/v1/undo", get(http_v1::handle_list_undo))
        .route("/v1/undo/test", post(http_v1::handle_test_undo_contract))
        .route("/v1/undo/:undo_id", get(http_v1::handle_get_undo))
        .route(
            "/v1/undo/:undo_id/execute",
            post(http_v1::handle_execute_undo),
        )
        .route(
            "/v1/undo/work-items/:work_item_id/rollback",
            post(http_v1::handle_execute_undo),
        )
        // WORM Audit & Compliance Endpoints
        .route("/v1/audit/events", get(http_v1::handle_list_audit_events))
        .route("/v1/audit/events/:id", get(http_v1::handle_get_audit_event))
        .route("/v1/audit/verify", get(http_v1::handle_verify_audit_chain))
        .route("/v1/audit/stats", get(http_v1::handle_get_audit_stats))
        .route("/v1/audit/export", get(http_v1::handle_export_audit))
        // Idempotency & Effect History Endpoints
        .route(
            "/v1/idempotency/records",
            get(http_v1::handle_list_idempotency_records),
        )
        .route(
            "/v1/idempotency/records/:key",
            get(http_v1::handle_get_idempotency_record),
        )
        // Web UI Control Deck
        .route("/ui", get(http_v1::handle_ui_dashboard))
        .route("/", get(http_v1::handle_ui_dashboard))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            security_guard_middleware,
        ))
        .with_state(app_state)
}

/// Inbound security middleware enforcing Host header validation (DNS rebinding defense),
/// browser cross-origin CSRF filtering, and optional API token authorization.
pub async fn security_guard_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    let headers = req.headers();
    let path = req.uri().path();
    let signed_chatops_action = path == "/v1/webhooks/actions";
    let legacy_chatops_callback = path == "/v1/webhooks/callbacks";
    let public_chatops_route = signed_chatops_action || legacy_chatops_callback;

    // 1. Host Validation (prevents DNS rebinding against 127.0.0.1)
    if let Some(host_hdr) = headers.get("host").and_then(|h| h.to_str().ok()) {
        let host_name = host_hdr.split(':').next().unwrap_or(host_hdr);
        let is_valid_host = host_name == "127.0.0.1"
            || host_name == "localhost"
            || host_name == "::1"
            || host_name == "[::1]";

        let configured_chatops_host =
            public_chatops_route && configured_chatops_callback_host(&state.config_path, host_hdr);
        if !is_valid_host && !configured_chatops_host {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "ok": false,
                    "error": "FORBIDDEN_HOST",
                    "message": format!("Invalid Host header: '{}'. Loopback direct access only.", host_hdr)
                })),
            )
                .into_response();
        }
    }

    // 2. Cross-Origin (CSRF) protection for browser requests
    if let Some(origin_hdr) = headers.get("origin").and_then(|h| h.to_str().ok()) {
        let is_valid_origin = origin_hdr.starts_with("http://127.0.0.1")
            || origin_hdr.starts_with("http://localhost")
            || origin_hdr.starts_with("vscode-webview://")
            || origin_hdr.starts_with("chrome-extension://")
            || origin_hdr == "null";

        if !is_valid_origin {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "ok": false,
                    "error": "FORBIDDEN_ORIGIN",
                    "message": "Cross-origin browser requests from untrusted origins are blocked."
                })),
            )
                .into_response();
        }
    }

    let tenant_ctx = if path.starts_with("/v1/") && !public_chatops_route {
        match authenticate_api_request(&state, headers).await {
            Ok(tenant) => tenant,
            Err(code) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "ok": false,
                        "error": code,
                        "message": "Valid Bearer token or X-Warmplane-Key required",
                    })),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let trust_admin = check_trust_admin(&state, tenant_ctx.as_ref(), path);
    if path.starts_with("/v1/config/")
        && !req.method().is_safe()
        && !trust_admin
        && state.trust_controls_active().await
    {
        return http_v1::trust_admin_required();
    }

    // 4. Resolve ProfileContext
    if path.starts_with("/v1/") && !public_chatops_route {
        let query_profile = req.uri().query().and_then(|q| {
            for param in q.split('&') {
                if let Some((k, v)) = param.split_once('=') {
                    if k == "profile" {
                        return Some(crate::http_v1::ProfileQuery {
                            profile: Some(v.to_string()),
                        });
                    }
                }
            }
            None
        });
        match crate::http_v1::resolve_profile_context(&state, headers, query_profile.as_ref()).await
        {
            Ok(prof_ctx) => {
                req.extensions_mut().insert(prof_ctx);
            }
            Err((status, err_val)) => {
                return (status, Json(err_val)).into_response();
            }
        }
    } else {
        req.extensions_mut()
            .insert(crate::context::ProfileContext::unrestricted());
    }

    req.extensions_mut()
        .insert(http_v1::TrustAdmin(trust_admin));
    req.extensions_mut().insert(tenant_ctx);

    next.run(req).await
}

pub(crate) async fn authenticate_api_request(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<Option<crate::rbac::TenantContext>, String> {
    let bearer = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        });
    let api_key = headers
        .get("x-warmplane-key")
        .and_then(|value| value.to_str().ok());
    if state.rbac_engine.is_enabled() {
        let policy = state.policy.read().await.clone();
        return state
            .rbac_engine
            .authenticate(bearer.or(api_key), &policy)
            .map(Some);
    }
    if let Some(expected) = state
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    {
        use subtle::ConstantTimeEq;
        if !bearer
            .into_iter()
            .chain(api_key)
            .any(|token| bool::from(token.as_bytes().ct_eq(expected.as_bytes())))
        {
            return Err("UNAUTHORIZED".to_owned());
        }
    }
    Ok(None)
}

fn check_trust_admin(
    state: &AppState,
    tenant: Option<&crate::rbac::TenantContext>,
    path: &str,
) -> bool {
    if path != "/v1/config" && !path.starts_with("/v1/config/") {
        return false;
    }
    if state.rbac_engine.is_enabled() {
        tenant.is_some_and(|tenant| state.rbac_engine.can_manage_trust_controls(tenant))
    } else {
        // The middleware has already verified the configured token for this path.
        state
            .auth_token
            .as_ref()
            .is_some_and(|token| !token.trim().is_empty())
    }
}
fn configured_chatops_callback_host(config_path: &str, host_header: &str) -> bool {
    let Ok(config) = crate::config::load_or_default_config(config_path) else {
        return false;
    };
    let Some(callback_url) = config
        .policy
        .and_then(|policy| policy.webhook)
        .and_then(|webhook| webhook.callback_url)
    else {
        return false;
    };
    let Ok(callback_url) = reqwest::Url::parse(&callback_url) else {
        return false;
    };
    let Ok(authority) = host_header.parse::<axum::http::uri::Authority>() else {
        return false;
    };
    let Some(callback_host) = callback_url.host_str() else {
        return false;
    };
    let port_matches = authority
        .port_u16()
        .is_none_or(|port| callback_url.port_or_known_default() == Some(port));
    authority.host().eq_ignore_ascii_case(callback_host) && port_matches
}

/// Awaits process termination signals (`SIGINT` or `SIGTERM`) for graceful server shutdown.
pub async fn shutdown_signal(shutdown_token: tokio_util::sync::CancellationToken) {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(error = %err, "failed to install Ctrl+C signal handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                error!(error = %err, "failed to install SIGTERM signal handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("received SIGINT (Ctrl+C); initiating graceful shutdown");
        },
        _ = terminate => {
            info!("received SIGTERM; initiating graceful shutdown");
        },
    }

    // Immediately trigger cancellation of all background workers, SSE streams, and supervisors
    shutdown_token.cancel();
}

/// Runs the Warmplane MCP aggregation daemon.
///
/// # Arguments
/// * `port` - TCP listening port.
/// * `config` - `McpConfig` configuration struct.
/// * `config_path` - Path to the config file.
///
/// # Errors
/// Returns an error if binding TCP socket or server execution fails.
pub async fn run_daemon(
    port: u16,
    config: McpConfig,
    config_path: impl Into<String>,
) -> Result<()> {
    let mcp_http_cfg = config.mcp_http_server.clone();
    let app_state = initialize_state(config, config_path).await?;
    let app = build_router(app_state.clone());

    info!(port, "all upstream servers connected; daemon listening");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    let shutdown_token = app_state.shutdown_token.clone();

    // Co-host the Streamable HTTP MCP facade on a separate port when configured
    if let Some(mcp_cfg) = mcp_http_cfg {
        use rmcp::transport::streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService,
        };
        use std::time::Duration;

        let bind_addr = mcp_cfg.bind.clone();
        let mcp_port = mcp_cfg.port;
        let profile = mcp_cfg.profile.clone();
        let sse_keep_alive = mcp_cfg.sse_keep_alive_ms.map(Duration::from_millis);
        let json_response = mcp_cfg.json_response;

        let mut allowed_hosts = mcp_cfg.allowed_hosts.clone();
        crate::mcp_server::add_mcp_http_hosts(&mut allowed_hosts, &bind_addr);

        let state_for_factory = app_state.clone();
        let profile_for_factory = profile.clone();
        let ct = shutdown_token.clone();

        let mut mcp_server_cfg = StreamableHttpServerConfig::default();
        mcp_server_cfg.sse_keep_alive = sse_keep_alive;
        mcp_server_cfg.json_response = json_response;
        mcp_server_cfg.cancellation_token = ct.clone();
        mcp_server_cfg.allowed_hosts = allowed_hosts;
        mcp_server_cfg.allowed_origins = mcp_cfg.allowed_origins.clone();

        let parsed_protocol_versions =
            crate::mcp_server::parse_mcp_protocol_versions(&mcp_cfg.supported_protocol_versions);

        let mcp_service = StreamableHttpService::new(
            move || {
                let s = state_for_factory.clone();
                let p = profile_for_factory.clone();
                let server = crate::mcp_server::FacadeMcpServer::new(s, p)
                    .with_supported_protocol_versions(parsed_protocol_versions.clone());
                Ok(server)
            },
            Arc::new(rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default()),
            mcp_server_cfg,
        );

        let mcp_router = axum::Router::new()
            .route_service("/mcp", mcp_service.clone())
            .route_service("/", mcp_service);

        let mcp_listen = format!("{}:{}", bind_addr, mcp_port);
        match TcpListener::bind(&mcp_listen).await {
            Ok(mcp_listener) => {
                info!(
                    bind = %mcp_listen,
                    profile = ?profile,
                    "daemon co-hosting Streamable HTTP MCP facade"
                );
                tokio::spawn(async move {
                    let server_fut = axum::serve(mcp_listener, mcp_router)
                        .with_graceful_shutdown(ct.cancelled_owned());
                    if let Err(err) = server_fut.await {
                        error!(error = %err, "co-hosted MCP HTTP server error");
                    }
                });
            }
            Err(err) => {
                error!(
                    bind = %mcp_listen,
                    error = %err,
                    "failed to bind co-hosted MCP HTTP facade; continuing without it"
                );
            }
        }
    }

    let server_fut =
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(shutdown_token));
    if let Err(err) = server_fut.await {
        error!(error = %err, "HTTP server error during execution or shutdown");
    }

    info!("HTTP server drained and stopped; shutting down daemon subsystems");
    let drain_timeout = std::time::Duration::from_secs(3);
    let _ = tokio::time::timeout(drain_timeout, app_state.shutdown()).await;
    info!("daemon shutdown sequence finished; exiting");

    Ok(())
}
