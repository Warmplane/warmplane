// Rust guideline compliant 2026-08-27

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
    let mut srv_configs = state.server_configs.read().await.clone();
    for srv in srv_configs.values_mut() {
        srv.sanitize_secrets();
    }
    let srv_statuses = state.server_statuses.read().await.clone();
    let circuit_breakers = state.circuit_breakers.all_statuses().await;

    match crate::config::load_or_default_config(&state.config_path) {
        Ok(mut config) => {
            config.sanitize_secrets();
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "config_path": state.config_path,
                    "config": config,
                    "server_configs": srv_configs,
                    "server_statuses": srv_statuses,
                    "circuit_breakers": circuit_breakers,
                    "metrics": {
                        "total_catalog_requests": total_reqs,
                        "total_etag_hits": etag_hits,
                        "total_tool_calls": tool_calls,
                        "total_tool_duration_us": tool_duration_us,
                    }
                })),
            )
                .into_response()
        }
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

    // Cascade scrub server from all profile constellations
    for profile in config.profiles.values_mut() {
        profile.servers.retain(|s| s != &id);
    }

    // Cascade scrub server from alias targets if server-qualified
    config
        .capability_aliases
        .retain(|_, target| !target.starts_with(&format!("{}.", id)));
    config
        .resource_aliases
        .retain(|_, target| !target.starts_with(&format!("{}.", id)));
    config
        .prompt_aliases
        .retain(|_, target| !target.starts_with(&format!("{}.", id)));

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Sync in-memory profiles state
    {
        let mut prof_guard = state.profiles.write().await;
        for (prof_name, prof_cfg) in &config.profiles {
            prof_guard.insert(prof_name.clone(), prof_cfg.clone());
        }
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

/// Handles POST `/v1/config/servers/:id/restart` dynamically unmounting and remounting a server.
pub async fn handle_restart_server(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    let config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let srv_cfg =
        match config.mcp_servers.get(&server_id) {
            Some(s) => s.clone(),
            None => return (
                StatusCode::NOT_FOUND,
                Json(json!({ "ok": false, "error": format!("Server '{}' not found", server_id) })),
            )
                .into_response(),
        };

    state.unmount_upstream_server(&server_id).await;
    match state
        .mount_upstream_server(
            &server_id,
            &srv_cfg,
            &config.capability_aliases,
            &config.resource_aliases,
            &config.prompt_aliases,
        )
        .await
    {
        Ok(()) => {
            let statuses_guard = state.server_statuses.read().await;
            let status = statuses_guard.get(&server_id).cloned();
            (
                StatusCode::OK,
                Json(json!({
                    "ok": true,
                    "server_id": server_id,
                    "status": status,
                    "message": format!("Server '{}' restarted successfully", server_id),
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
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

/// Handles GET `/v1/clients` returning status of all supported external AI clients.
pub async fn handle_list_clients() -> impl IntoResponse {
    let statuses = crate::client_sync::detect_clients();
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "clients": statuses,
        })),
    )
        .into_response()
}

/// Handles POST `/v1/clients/:id/attach` attaching Warmplane to an external client.
pub async fn handle_attach_client(
    State(state): State<AppState>,
    Path(client_id): Path<String>,
    Json(payload): Json<crate::http_v1::types::AttachClientApiRequest>,
) -> impl IntoResponse {
    let options = crate::client_sync::AttachOptions {
        profile: payload.profile,
        config_path: Some(payload.config_path.unwrap_or(state.config_path.clone())),
        binary_path: None,
    };

    match crate::client_sync::attach_client(&client_id, &options) {
        Ok(res) => (
            StatusCode::OK,
            Json(json!({
                "ok": res.ok,
                "client_id": res.client_id,
                "config_path": res.config_path,
                "backup_path": res.backup_path,
                "message": res.message,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": err.to_string(),
            })),
        )
            .into_response(),
    }
}

/// Handles POST `/v1/clients/:id/detach` detaching Warmplane from an external client.
pub async fn handle_detach_client(Path(client_id): Path<String>) -> impl IntoResponse {
    match crate::client_sync::detach_client(&client_id) {
        Ok(res) => (
            StatusCode::OK,
            Json(json!({
                "ok": res.ok,
                "client_id": res.client_id,
                "config_path": res.config_path,
                "was_attached": res.was_attached,
                "message": res.message,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "ok": false,
                "error": err.to_string(),
            })),
        )
            .into_response(),
    }
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

/// Handles POST `/v1/config/profiles` to add or update a profile constellation.
pub async fn handle_upsert_profile(
    State(state): State<AppState>,
    Json(payload): Json<crate::http_v1::types::UpsertProfileRequest>,
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

    // Validate that all referenced servers exist in mcp_servers
    for srv in &payload.servers {
        if !config.mcp_servers.contains_key(srv) {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "ok": false,
                    "error": format!("Profile references unknown upstream server '{}'", srv)
                })),
            )
                .into_response();
        }
    }

    let prof_cfg = crate::config::ProfileConfig {
        servers: payload.servers,
        description: payload.description,
        policy: payload.policy,
    };

    config
        .profiles
        .insert(payload.name.clone(), prof_cfg.clone());

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Sync in-memory active profiles state
    {
        let mut prof_guard = state.profiles.write().await;
        prof_guard.insert(payload.name, prof_cfg);
    }

    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

/// Handles DELETE `/v1/config/profiles/:id` to remove a profile constellation.
pub async fn handle_delete_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<String>,
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

    config.profiles.remove(&profile_id);

    if let Err(e) = crate::config::save_config(&state.config_path, &config) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response();
    }

    // Sync in-memory active profiles state
    {
        let mut prof_guard = state.profiles.write().await;
        prof_guard.remove(&profile_id);
    }

    (StatusCode::OK, Json(json!({ "ok": true }))).into_response()
}

/// Handles GET `/v1/secrets` returning detected secret references and vault status.
pub async fn handle_list_secrets(State(state): State<AppState>) -> impl IntoResponse {
    let config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let mut secrets = Vec::new();
    for (srv_name, srv_cfg) in &config.mcp_servers {
        for (k, v) in &srv_cfg.env {
            let is_vault = v.starts_with("keychain://")
                || v.starts_with("op://")
                || v.starts_with("env://")
                || v.starts_with("cmd://");

            let backend = if v.starts_with("keychain://") {
                "OS Keychain"
            } else if v.starts_with("op://") {
                "1Password"
            } else if v.starts_with("env://") {
                "Environment"
            } else if v.starts_with("cmd://") {
                "Command Subprocess"
            } else {
                "Plaintext (Unsecured)"
            };

            let exists = if is_vault {
                crate::vault::resolve_secret_value(v).is_ok()
            } else {
                true
            };

            secrets.push(json!({
                "server": srv_name,
                "key": k,
                "uri": v,
                "is_vault": is_vault,
                "exists": exists,
                "backend": backend,
                "display": crate::vault::redact_secret_for_display(v),
            }));
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "secrets": secrets,
            "keychain_service": crate::vault::DEFAULT_KEYCHAIN_SERVICE,
        })),
    )
        .into_response()
}

/// Handles POST `/v1/secrets` storing a secret in OS Keychain.
pub async fn handle_upsert_secret(
    State(_state): State<AppState>,
    Json(payload): Json<crate::http_v1::types::UpsertSecretRequest>,
) -> impl IntoResponse {
    let service = payload
        .service
        .unwrap_or_else(|| crate::vault::DEFAULT_KEYCHAIN_SERVICE.to_string());

    match crate::vault::set_os_keychain_secret(&service, &payload.key, &payload.value) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "key": payload.key,
                "service": service,
                "uri": format!("keychain://{}/{}", service, payload.key),
                "message": format!("Secret '{}' saved in OS Keychain", payload.key),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Handles DELETE `/v1/secrets/:key` removing a secret from OS Keychain.
pub async fn handle_delete_secret(
    State(_state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let service = crate::vault::DEFAULT_KEYCHAIN_SERVICE;
    match crate::vault::delete_os_keychain_secret(service, &key) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "key": key,
                "message": format!("Secret '{}' removed from OS Keychain", key),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}
