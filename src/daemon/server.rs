// Rust guideline compliant 2026-08-15

//! Daemon HTTP service initialization, routing, and TCP listener runtime.

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tracing::info;

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
    let event_store = Arc::new(crate::catalog::CatalogEventStore::new());

    // Initialize central OAuth registry and proxy server if any server uses OAuth2
    let oauth_registry = crate::oauth2::OAuthRegistry::default();
    let mut oauth_proxy_port = None;

    let has_oauth2 = config
        .mcp_servers
        .values()
        .any(|s| matches!(s.auth, Some(AuthConfig::Oauth2 { .. })));

    if has_oauth2 {
        let port = crate::oauth2::start_oauth_proxy_server(oauth_registry.clone()).await?;
        oauth_proxy_port = Some(port);
    }

    let search_engine = Arc::new(crate::search::HybridSearchEngine::new());

    let (audit_store, audit_handle) = if let Some(ref audit_cfg) = config.audit {
        if audit_cfg.enabled {
            let store = if let Some(ref path) = audit_cfg.file_path {
                Arc::new(crate::audit::AuditStore::open_or_create(path)?)
            } else {
                Arc::new(crate::audit::AuditStore::in_memory())
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

    let state = AppState::builder()
        .servers_arc(Arc::new(RwLock::new(HashMap::new())))
        .capabilities_arc(Arc::new(RwLock::new(HashMap::new())))
        .resources_arc(Arc::new(RwLock::new(HashMap::new())))
        .prompts_arc(Arc::new(RwLock::new(HashMap::new())))
        .tool_timeout_ms(tool_timeout_ms)
        .policy_arc(Arc::new(RwLock::new(policy)))
        .search_engine(search_engine)
        .catalog_version_arc(Arc::new(RwLock::new(String::new())))
        .event_store(event_store)
        .idempotency_store(Arc::new(crate::idempotency::IdempotencyStore::default()))
        .operation_registry(crate::operations::OperationRegistry::new())
        .config_path(config_path_str)
        .server_configs_arc(Arc::new(RwLock::new(HashMap::new())))
        .server_statuses_arc(Arc::new(RwLock::new(HashMap::new())))
        .oauth_proxy_port(oauth_proxy_port)
        .oauth_registry(oauth_registry)
        .audit_store(audit_store)
        .audit_handle(audit_handle)
        .build();

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
    req: axum::extract::Request,
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

    // 3. Optional Auth Token verification if configured on state
    if let Some(ref expected_token) = state.auth_token {
        if !expected_token.trim().is_empty() {
            let path = req.uri().path();
            if path.starts_with("/v1/") {
                let is_authed = headers
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|v| v.strip_prefix("Bearer "))
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
    }

    next.run(req).await
}

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
    let app_state = initialize_state(config, config_path).await?;
    let app = build_router(app_state);

    info!(port, "all upstream servers connected; daemon listening");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
