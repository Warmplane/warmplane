// Rust guideline compliant 2026-08-15

//! Configuration and control deck REST API handlers for server management, ecosystem import, aliases, and policies.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::atomic::Ordering;

use crate::{
    daemon::AppState,
    http_v1::types::{ImportConfigRequest, UpdateAliasRequest, UpsertServerRequest},
};

/// Handles GET `/v1/config` returning the active configuration and operational metrics.
pub async fn handle_get_config(State(state): State<AppState>) -> impl IntoResponse {
    let total_reqs = state.total_catalog_requests.load(Ordering::Relaxed);
    let etag_hits = state.total_etag_hits.load(Ordering::Relaxed);
    let tool_calls = state.total_tool_calls.load(Ordering::Relaxed);
    let tool_duration_us = state.total_tool_duration_us.load(Ordering::Relaxed);
    let srv_configs = state.server_configs.read().await.clone();
    let srv_statuses = state.server_statuses.read().await.clone();

    match crate::config::load_or_default_config(&state.config_path) {
        Ok(config) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "config_path": state.config_path,
                "config": config,
                "server_configs": srv_configs,
                "server_statuses": srv_statuses,
                "metrics": {
                    "total_catalog_requests": total_reqs,
                    "total_etag_hits": etag_hits,
                    "total_tool_calls": tool_calls,
                    "total_tool_duration_us": tool_duration_us,
                }
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": err.to_string() })),
        )
            .into_response(),
    }
}

/// Handles POST `/v1/config/servers` to add or update an upstream server.
pub async fn handle_upsert_server(
    State(state): State<AppState>,
    Json(payload): Json<UpsertServerRequest>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    config
        .mcp_servers
        .insert(payload.name.clone(), payload.server.clone());
    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Hot-mount the newly added/updated server into active daemon state
    let mount_result = state
        .mount_upstream_server(
            &payload.name,
            &payload.server,
            &config.capability_aliases,
            &config.resource_aliases,
            &config.prompt_aliases,
        )
        .await;

    let warning = mount_result.err().map(|e| e.to_string());

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message": if warning.is_some() {
                format!("Server '{}' saved to configuration, but initial connection failed", payload.name)
            } else {
                format!("Server '{}' connected and mounted successfully", payload.name)
            },
            "warning": warning,
            "server": payload.name
        })),
    )
        .into_response()
}

/// Handles DELETE `/v1/config/servers/:id` to remove an upstream server.
pub async fn handle_delete_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if config.mcp_servers.remove(&id).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": format!("Server '{}' not found", id) })),
        )
            .into_response();
    }

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Hot-unmount server from runtime
    state.unmount_upstream_server(&id).await;

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "message": format!("Server '{}' unmounted and removed", id),
        })),
    )
        .into_response()
}

/// Handles GET `/v1/config/ecosystem` discovering external MCP sources.
pub async fn handle_get_ecosystem_sources() -> impl IntoResponse {
    let sources = crate::config_import::discover_sources();
    let serializable_sources: Vec<_> = sources
        .into_iter()
        .map(|s| {
            json!({
                "name": s.name,
                "path": s.path.to_string_lossy(),
                "server_count": s.server_count,
                "servers": s.servers.keys().cloned().collect::<Vec<_>>(),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "sources": serializable_sources,
        })),
    )
        .into_response()
}

/// Handles POST `/v1/config/import` importing external MCP configurations.
pub async fn handle_import_config(
    State(state): State<AppState>,
    Json(payload): Json<ImportConfigRequest>,
) -> impl IntoResponse {
    let overwrite = payload.overwrite.unwrap_or(false);
    let mut all_imported = 0;
    let mut all_skipped = Vec::new();

    let sources = if let Some(path_str) = payload.source_path {
        let pb = std::path::PathBuf::from(path_str);
        match crate::config_import::parse_standard_mcp_source("Custom File", pb) {
            Ok(s) => vec![s],
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    } else {
        crate::config_import::discover_sources()
    };

    let active_cfg = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    for src in sources {
        let servers_to_mount = src.servers.clone();
        match crate::config_import::import_servers_into_config(
            &state.config_path,
            src.servers,
            overwrite,
        ) {
            Ok((count, skipped)) => {
                all_imported += count;
                all_skipped.extend(skipped);

                // Dynamically mount newly imported servers
                for (s_id, s_cfg) in servers_to_mount {
                    let _ = state
                        .mount_upstream_server(
                            &s_id,
                            &s_cfg,
                            &active_cfg.capability_aliases,
                            &active_cfg.resource_aliases,
                            &active_cfg.prompt_aliases,
                        )
                        .await;
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "ok": false, "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "imported_count": all_imported,
            "skipped_servers": all_skipped,
        })),
    )
        .into_response()
}

/// Handles POST `/v1/config/alias` adding or removing an alias.
pub async fn handle_update_alias(
    State(state): State<AppState>,
    Json(payload): Json<UpdateAliasRequest>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let kind_lower = payload.kind.to_lowercase();
    if let Some(target) = payload.target {
        match kind_lower.as_str() {
            "tool" | "capability" | "cap" => {
                config
                    .capability_aliases
                    .insert(payload.alias.clone(), target);
            }
            "resource" | "res" => {
                config
                    .resource_aliases
                    .insert(payload.alias.clone(), target);
            }
            "prompt" => {
                config.prompt_aliases.insert(payload.alias.clone(), target);
            }
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": "Invalid alias kind. Expected 'tool', 'resource', or 'prompt'" })),
                ).into_response();
            }
        }
    } else {
        match kind_lower.as_str() {
            "tool" | "capability" | "cap" => {
                config.capability_aliases.remove(&payload.alias);
            }
            "resource" | "res" => {
                config.resource_aliases.remove(&payload.alias);
            }
            "prompt" => {
                config.prompt_aliases.remove(&payload.alias);
            }
            _ => {}
        }
    }

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

/// Handles POST `/v1/config/policy` updating security policies.
pub async fn handle_update_policy(
    State(state): State<AppState>,
    Json(payload): Json<crate::config::PolicyConfig>,
) -> impl IntoResponse {
    let mut config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    config.policy = Some(payload.clone());
    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Immediately update in-memory active policy
    {
        let mut pol_guard = state.policy.write().await;
        *pol_guard = crate::daemon::Policy::from_config(Some(payload));
    }

    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

/// Handles POST `/v1/config/reload` explicitly triggering a hot-reloading reconciliation from disk.
pub async fn handle_reload_config(State(state): State<AppState>) -> impl IntoResponse {
    match state.reload_from_disk().await {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}
