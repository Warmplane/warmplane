// Rust guideline compliant 2026-08-19

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
    config: McpConfig,
    config_path: impl Into<String>,
) -> Result<AppState> {
    let config_path_str = config_path.into();
    let tool_timeout_ms = config.tool_timeout_ms.unwrap_or(DEFAULT_TOOL_TIMEOUT_MS);
    let policy = Policy::from_config(config.policy.clone());

    // Resolve persistent state directory if enabled
    let state_dir_opt = match &config.state {
        Some(s) if s.enabled => s.dir.as_deref().or(Some(".warmplane/state")),
        Some(_) => None,
        None => Some(".warmplane/state"),
    };

    let (event_store, idempotency_store, oauth_registry, approval_registry, sampling_registry) =
        if let Some(dir_str) = state_dir_opt {
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
            let app =
                crate::approvals::ApprovalRegistry::open_or_create(state_dir.approvals_file())?;
            let samp =
                crate::sampling::SamplingRegistry::open_or_create(state_dir.sampling_file())?;
            (ev, idm, oa, app, samp)
        } else {
            (
                Arc::new(crate::catalog::CatalogEventStore::new()),
                Arc::new(crate::idempotency::IdempotencyStore::default()),
                crate::oauth2::OAuthRegistry::default(),
                crate::approvals::ApprovalRegistry::default(),
                crate::sampling::SamplingRegistry::new(),
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

    let (audit_store, audit_handle) = if let Some(ref audit_cfg) = config.audit {
        let hmac_key_bytes = audit_cfg.resolve_hmac_key().map(|k| k.into_bytes());
        if audit_cfg.enabled {
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
        }
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

    let auth_token = config.auth_token.or_else(|| {
        std::env::var("WARMPLANE_AUTH_TOKEN")
            .ok()
            .filter(|t| !t.trim().is_empty())
    });

    let rbac_engine = crate::rbac::RbacEngine::new(config.rbac.clone());

    let mut state_builder = AppState::builder()
        .servers_arc(Arc::new(RwLock::new(HashMap::new())))
        .capabilities_arc(Arc::new(RwLock::new(HashMap::new())))
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
        if let Err(e) = state
            .mount_upstream_server(
                server_id,
                srv_cfg,
                &config.capability_aliases,
                &config.resource_aliases,
                &config.prompt_aliases,
            )
            .await
        {
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

            // Also keep server config in state so circuit breakers and supervisors can manage it
            let mut configs_guard = state.server_configs.write().await;
            configs_guard.insert(server_id.to_string(), srv_cfg.clone());
        }
    }

    Ok(state)
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
            "/v1/config/ecosystem",
            get(http_v1::handle_get_ecosystem_sources),
        )
        .route("/v1/config/import", post(http_v1::handle_import_config))
        .route("/v1/config/alias", post(http_v1::handle_update_alias))
        .route("/v1/config/policy", post(http_v1::handle_update_policy))
        .route("/v1/config/profiles", post(http_v1::handle_upsert_profile))
        .route(
            "/v1/config/profiles/:id",
            axum::routing::delete(http_v1::handle_delete_profile),
        )
        .route("/v1/config/reload", post(http_v1::handle_reload_config))
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

    // 1. Host Validation (prevents DNS rebinding against 127.0.0.1)
    if let Some(host_hdr) = headers.get("host").and_then(|h| h.to_str().ok()) {
        let host_name = host_hdr.split(':').next().unwrap_or(host_hdr);
        let is_valid_host = host_name == "127.0.0.1"
            || host_name == "localhost"
            || host_name == "::1"
            || host_name == "[::1]";

        if !is_valid_host {
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

    // 3. RBAC & Auth Token verification
    let mut tenant_ctx = None;
    let path = req.uri().path();

    if state.rbac_engine.is_enabled() {
        if path.starts_with("/v1/") {
            let token = headers
                .get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|v| {
                    v.strip_prefix("Bearer ")
                        .or_else(|| v.strip_prefix("bearer "))
                })
                .or_else(|| headers.get("x-warmplane-key").and_then(|h| h.to_str().ok()));

            let base_pol = state.policy.read().await.clone();
            match state.rbac_engine.authenticate(token, &base_pol) {
                Ok(ctx) => {
                    tenant_ctx = Some(ctx);
                }
                Err(err_code) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "ok": false,
                            "error": err_code,
                            "message": "Valid Bearer token, JWT, or X-Warmplane-Key required"
                        })),
                    )
                        .into_response();
                }
            }
        }
    } else if let Some(ref expected_token) = state.auth_token {
        if !expected_token.trim().is_empty() && path.starts_with("/v1/") {
            let is_authed = headers
                .get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|v| {
                    v.strip_prefix("Bearer ")
                        .or_else(|| v.strip_prefix("bearer "))
                })
                .map(|t| t == expected_token)
                .unwrap_or(false)
                || headers
                    .get("x-warmplane-key")
                    .and_then(|h| h.to_str().ok())
                    .map(|t| t == expected_token)
                    .unwrap_or(false);

            if !is_authed {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "ok": false,
                        "error": "UNAUTHORIZED",
                        "message": "Valid Bearer token or X-Warmplane-Key required"
                    })),
                )
                    .into_response();
            }
        }
    }

    // 4. Resolve ProfileContext
    if path.starts_with("/v1/") {
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

    req.extensions_mut().insert(tenant_ctx);

    next.run(req).await
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
        for loopback in &["127.0.0.1", "::1", "localhost"] {
            if !allowed_hosts.iter().any(|h| h == loopback) {
                allowed_hosts.push((*loopback).to_string());
            }
        }
        if !matches!(
            bind_addr.as_str(),
            "127.0.0.1" | "::1" | "localhost" | "0.0.0.0" | "::"
        ) {
            allowed_hosts.push(bind_addr.clone());
        }

        let state_for_factory = app_state.clone();
        let profile_for_factory = profile.clone();
        let ct = shutdown_token.clone();

        let mut mcp_server_cfg = StreamableHttpServerConfig::default();
        mcp_server_cfg.sse_keep_alive = sse_keep_alive;
        mcp_server_cfg.json_response = json_response;
        mcp_server_cfg.cancellation_token = ct.clone();
        mcp_server_cfg.allowed_hosts = allowed_hosts;
        mcp_server_cfg.allowed_origins = mcp_cfg.allowed_origins.clone();

        let mcp_service = StreamableHttpService::new(
            move || {
                let s = state_for_factory.clone();
                let p = profile_for_factory.clone();
                Ok(crate::mcp_server::FacadeMcpServer::new(s, p))
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

    Ok(())
}
