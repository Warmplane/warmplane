// Rust guideline compliant 2026-11-18

//! Dynamic server mounting, unmounting, and configuration reconciliation lifecycle operations.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{
    config::ServerConfig,
    daemon::{
        policy::Policy,
        state::{compute_catalog_version, AppState},
        transport::DEFAULT_MCP_PROTOCOL_VERSION,
    },
};

impl AppState {
    /// Recomputes catalog identity and emits the server-level resource invalidation.
    pub(crate) async fn refresh_server_catalog(&self, server_id: &str) {
        {
            let caps = self.capabilities.read().await;
            let res = self.resources.read().await;
            let prompts = self.prompts.read().await;
            let new_ver = compute_catalog_version(&caps, &res, &prompts);
            let mut ver_guard = self.catalog_version.write().await;
            *ver_guard = new_ver;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let _ = self
            .resource_update_tx
            .send(crate::catalog::ResourceUpdateEvent {
                uri: format!("server://{}", server_id),
                timestamp,
                server: server_id.to_string(),
            });
    }

    fn audit_server_lifecycle(
        &self,
        server_id: &str,
        action: &str,
        status: &str,
        args: serde_json::Value,
    ) {
        self.audit_handle.send(crate::audit::RawAuditEvent {
            event_type: crate::audit::AuditEventType::ConfigMutation,
            trace_id: format!("trace-{action}-{server_id}"),
            request_id: None,
            actor_id: None,
            work_item_id: None,
            client_ip: None,
            server_id: Some(server_id.to_string()),
            capability_id: None,
            resource_uri: None,
            sanitized_args: Some(args),
            sanitized_response: Some(json!({"status":status})),
            execution_latency_us: None,
            status: crate::audit::AuditEventStatus::Success,
            error_code: None,
            error_message: None,
            operator_id: None,
            approval_ticket_id: None,
            idempotency_key: None,
            is_replay: None,
        });
    }

    /// Mounts a single upstream server dynamically into `AppState`.
    ///
    /// # Arguments
    /// * `server_id` - Unique server identifier.
    /// * `srv_cfg` - Server configuration and transport settings.
    /// * `capability_aliases` - Map of capability alias overrides.
    /// * `resource_aliases` - Map of resource alias overrides.
    /// * `prompt_aliases` - Map of prompt alias overrides.
    ///
    /// # Errors
    /// Returns an error if transport initialization or discovery fails.
    pub async fn mount_upstream_server(
        &self,
        server_id: &str,
        srv_cfg: &ServerConfig,
        capability_aliases: &HashMap<String, crate::config::AliasTarget>,
        resource_aliases: &HashMap<String, crate::config::AliasTarget>,
        prompt_aliases: &HashMap<String, crate::config::AliasTarget>,
    ) -> Result<()> {
        if let Some(source) = &srv_cfg.open_api {
            return crate::openapi::mount(
                self,
                server_id,
                srv_cfg,
                source,
                capability_aliases,
                crate::openapi::MountOrigin::Management(None),
            )
            .await;
        }
        let _lifecycle = self.upstream_lifecycle.lock().await;
        self.withdraw_previous_source(server_id, srv_cfg).await?;
        info!(%server_id, "dynamically mounting upstream server");

        let transport_type = if srv_cfg.command.is_some() {
            "stdio"
        } else {
            "http"
        };

        let (new_capabilities, new_resources, new_prompts, tx) =
            crate::supervisor::spawn_configured_upstream(
                self,
                server_id,
                srv_cfg,
                capability_aliases,
                resource_aliases,
                prompt_aliases,
            )
            .await?;

        let resilience_cfg = srv_cfg.resilience.clone().unwrap_or_default();
        self.circuit_breakers
            .get_or_create(server_id, resilience_cfg)
            .await;

        {
            let mut servers_guard = self.servers.write().await;
            servers_guard.insert(server_id.to_string(), tx);
        }
        {
            let mut configs_guard = self.server_configs.write().await;
            configs_guard.insert(server_id.to_string(), srv_cfg.clone());
        }
        {
            let mut statuses_guard = self.server_statuses.write().await;
            let is_degraded =
                new_capabilities.is_empty() && new_resources.is_empty() && new_prompts.is_empty();

            let mut status_val = json!({
                "transport": transport_type,
                "protocol_version": srv_cfg.protocol_version.as_deref().unwrap_or(DEFAULT_MCP_PROTOCOL_VERSION),
                "status": if is_degraded { "degraded" } else { "connected" }
            });

            if is_degraded {
                if let Some(obj) = status_val.as_object_mut() {
                    obj.insert(
                        "error".to_string(),
                        json!("Initial connection or MCP handshake failed; supervisor retrying in background"),
                    );
                }
            }

            statuses_guard.insert(server_id.to_string(), status_val);
        }
        let new_caps_count = new_capabilities.len();
        let new_res_count = new_resources.len();
        let new_prompts_count = new_prompts.len();
        let has_new_caps = new_caps_count > 0;
        let has_new_res = new_res_count > 0;
        let has_new_prompts = new_prompts_count > 0;

        {
            let mut caps_guard = self.capabilities.write().await;
            crate::supervisor::replace_mcp_capabilities(
                self,
                server_id,
                &mut caps_guard,
                new_capabilities,
            );
        }
        {
            let mut res_guard = self.resources.write().await;
            for (id, meta) in new_resources {
                info!(%server_id, resource_id = %id, "registered upstream resource");
                res_guard.insert(id.clone(), meta);
                self.event_store.record("resource", &id, "added");
            }
        }
        {
            let mut prompts_guard = self.prompts.write().await;
            for (id, meta) in new_prompts {
                info!(%server_id, prompt_id = %id, "registered upstream prompt");
                prompts_guard.insert(id.clone(), meta);
                self.event_store.record("prompt", &id, "added");
            }
        }

        self.refresh_server_catalog(server_id).await;

        if has_new_caps {
            self.notify_tools_changed();
        }
        if has_new_res {
            self.notify_resources_changed();
        }
        if has_new_prompts {
            self.notify_prompts_changed();
        }

        self.audit_server_lifecycle(
            server_id,
            "mount",
            "mounted",
            json!({
                "action": "mount_server",
                "capabilities_count": new_caps_count,
                "resources_count": new_res_count,
                "prompts_count": new_prompts_count,
            }),
        );

        Ok(())
    }

    async fn withdraw_previous_source(
        &self,
        server_id: &str,
        candidate: &ServerConfig,
    ) -> Result<()> {
        self.open_api_mounts.cancel(server_id);
        let previous = self.server_configs.read().await.get(server_id).cloned();
        if must_withdraw_openapi(previous.as_ref(), candidate) {
            self.unmount_locked(server_id, true).await?;
        }
        Ok(())
    }

    /// Unmounts an upstream server dynamically from `AppState`.
    ///
    /// # Arguments
    /// * `server_id` - Identifier of server to unmount.
    pub async fn unmount_upstream_server(&self, server_id: &str) {
        let _lifecycle = self.upstream_lifecycle.lock().await;
        if let Err(error) = self.unmount_locked(server_id, true).await {
            warn!(%server_id, %error, "could not persist upstream pin revocation");
        }
    }

    pub(crate) async fn unmount_locked(&self, server_id: &str, revoke: bool) -> Result<()> {
        info!(%server_id, "dynamically unmounting upstream server");
        self.open_api_mounts.cancel(server_id);
        let pin_result = self.retire_upstream_pins(server_id, revoke).await;

        {
            let mut servers_guard = self.servers.write().await;
            servers_guard.remove(server_id);
        }
        {
            let mut configs_guard = self.server_configs.write().await;
            configs_guard.remove(server_id);
        }
        {
            let mut statuses_guard = self.server_statuses.write().await;
            statuses_guard.remove(server_id);
        }
        self.circuit_breakers.remove(server_id).await;
        let removed_caps: Vec<String> = {
            let caps_guard = self.capabilities.read().await;
            caps_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect()
        };
        let removed_res: Vec<String> = {
            let res_guard = self.resources.read().await;
            res_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect()
        };
        let removed_prompts: Vec<String> = {
            let prompts_guard = self.prompts.read().await;
            prompts_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect()
        };

        let has_removed_caps = !removed_caps.is_empty();
        let has_removed_res = !removed_res.is_empty();
        let has_removed_prompts = !removed_prompts.is_empty();

        {
            let mut caps_guard = self.capabilities.write().await;
            for id in removed_caps {
                caps_guard.remove(&id);
            }
            self.record_tool_catalog_events(server_id, &caps_guard);
        }
        {
            let mut res_guard = self.resources.write().await;
            for id in removed_res {
                res_guard.remove(&id);
                self.event_store.record("resource", &id, "removed");
            }
        }
        {
            let mut prompts_guard = self.prompts.write().await;
            for id in removed_prompts {
                prompts_guard.remove(&id);
                self.event_store.record("prompt", &id, "removed");
            }
        }
        self.refresh_server_catalog(server_id).await;

        if has_removed_caps {
            self.notify_tools_changed();
        }
        if has_removed_res {
            self.notify_resources_changed();
        }
        if has_removed_prompts {
            self.notify_prompts_changed();
        }

        self.audit_server_lifecycle(
            server_id,
            "unmount",
            "unmounted",
            json!({"action":"unmount_server"}),
        );
        pin_result
    }

    async fn retire_upstream_pins(&self, server_id: &str, revoke: bool) -> Result<()> {
        let remote = self
            .server_configs
            .read()
            .await
            .get(server_id)
            .is_some_and(is_remote_openapi);
        if remote && revoke {
            self.tool_pins.withdraw_compiled(server_id)
        } else {
            self.tool_pins.remove_server(server_id);
            Ok(())
        }
    }

    /// Performs graceful shutdown of all upstream servers and background worker queues.
    pub async fn shutdown(&self) {
        info!("initiating graceful shutdown of Warmplane daemon subsystems");

        // 0. Notify all supervisor tasks and workers to stop
        self.shutdown_token.cancel();

        // 1. Collect all active upstream servers and unmount them
        let server_ids: Vec<String> = {
            let guard = self.servers.read().await;
            guard.keys().cloned().collect()
        };

        for server_id in server_ids {
            let _lifecycle = self.upstream_lifecycle.lock().await;
            let _ = self.unmount_locked(&server_id, false).await;
        }

        // 2. Flush and drain audit queue
        self.audit_handle.shutdown().await;

        info!("graceful shutdown of daemon subsystems completed");
    }

    /// Reconciles the runtime daemon state against the on-disk configuration file.
    ///
    /// # Returns
    /// A JSON Value summary of mounted, unmounted, and updated upstream servers.
    ///
    /// # Errors
    /// Returns an error if loading or parsing the config file fails.
    pub async fn reload_from_disk(&self) -> Result<serde_json::Value> {
        self.reload_from_disk_as(None).await
    }

    pub(crate) async fn reload_from_disk_as(
        &self,
        actor: Option<&str>,
    ) -> Result<serde_json::Value> {
        info!(config_path = %self.config_path, "reloading configuration from disk");

        let config = crate::config::load_config(&self.config_path)?;
        self.approval_registry.validate_webhook_storage(
            config
                .policy
                .as_ref()
                .and_then(|policy| policy.webhook.as_ref()),
        )?;

        // 1. Update security policy atomically
        {
            let mut pol_guard = self.policy.write().await;
            *pol_guard = Policy::from_config(config.policy.clone());
        }
        {
            let mut profiles = self.profiles.write().await;
            *profiles = config.profiles.clone();
        }
        self.rbac_engine.reload(config.rbac.clone());

        // 2. Identify active servers vs disk servers
        let mut current_server_ids: std::collections::BTreeSet<String> = {
            let srv_guard = self.server_configs.read().await;
            srv_guard.keys().cloned().collect()
        };
        current_server_ids.extend(self.open_api_mounts.pending_servers());

        let mut unmounted = Vec::new();
        let mut mounted = Vec::new();
        let mut warnings = Vec::new();

        // Unmount servers removed from disk config
        for active_id in &current_server_ids {
            if !config.mcp_servers.contains_key(active_id) {
                self.unmount_upstream_server(active_id).await;
                unmounted.push(active_id.clone());
            }
        }

        // Mount new or updated servers
        for (server_id, srv_cfg) in &config.mcp_servers {
            let res = if let Some(source) = &srv_cfg.open_api {
                crate::openapi::mount(
                    self,
                    server_id,
                    srv_cfg,
                    source,
                    &config.capability_aliases,
                    crate::openapi::MountOrigin::Management(actor),
                )
                .await
            } else {
                self.mount_upstream_server(
                    server_id,
                    srv_cfg,
                    &config.capability_aliases,
                    &config.resource_aliases,
                    &config.prompt_aliases,
                )
                .await
            };

            match res {
                Ok(_) => {
                    mounted.push(server_id.clone());
                }
                Err(e) => {
                    warn!(server_id = %server_id, error = %e, "server failed to mount during reload");
                    warnings.push(format!("Server '{}': {}", server_id, e));
                }
            }
        }

        self.notify_tools_changed();
        self.notify_resources_changed();
        self.notify_prompts_changed();

        Ok(serde_json::json!({
            "ok": true,
            "mounted": mounted,
            "unmounted": unmounted,
            "warnings": warnings,
            "total_active": self.servers.read().await.len(),
            "catalog_version": self.catalog_version.read().await.clone(),
        }))
    }
}

fn is_remote_openapi(config: &ServerConfig) -> bool {
    config
        .open_api
        .as_ref()
        .is_some_and(|source| source.remote_spec.is_some())
}

pub(crate) fn must_withdraw_openapi(
    previous: Option<&ServerConfig>,
    candidate: &ServerConfig,
) -> bool {
    previous.is_some_and(|previous| {
        let source_changed = previous.open_api != candidate.open_api;
        let switches_transport = previous.open_api.is_some() != candidate.open_api.is_some();
        switches_transport
            || (source_changed && (is_remote_openapi(previous) || is_remote_openapi(candidate)))
    })
}
