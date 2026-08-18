// Rust guideline compliant 2026-08-18

//! Dynamic server mounting, unmounting, and configuration reconciliation lifecycle operations.

use anyhow::{anyhow, Result};
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
        capability_aliases: &HashMap<String, String>,
        resource_aliases: &HashMap<String, String>,
        prompt_aliases: &HashMap<String, String>,
    ) -> Result<()> {
        info!(%server_id, "dynamically mounting upstream server");

        let transport_type = if srv_cfg.command.is_some() {
            "stdio"
        } else {
            "http"
        };

        let (new_capabilities, new_resources, new_prompts, tx) = if srv_cfg.command.is_some() {
            crate::supervisor::spawn_supervised_stdio_server(
                self,
                server_id,
                srv_cfg,
                capability_aliases,
                resource_aliases,
                prompt_aliases,
            )
            .await?
        } else if srv_cfg.url.is_some() {
            crate::supervisor::spawn_supervised_http_server(
                self,
                server_id,
                srv_cfg,
                capability_aliases,
                resource_aliases,
                prompt_aliases,
            )
            .await?
        } else {
            return Err(anyhow!(
                "Server '{}' missing transport selector: set exactly one of 'command' or 'url'",
                server_id
            ));
        };

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
        {
            let mut caps_guard = self.capabilities.write().await;
            for (id, meta) in new_capabilities {
                info!(%server_id, capability_id = %id, "registered upstream capability");
                caps_guard.insert(id.clone(), meta);
                self.event_store.record("capability", &id, "added");
            }
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

        Ok(())
    }

    /// Unmounts an upstream server dynamically from `AppState`.
    ///
    /// # Arguments
    /// * `server_id` - Identifier of server to unmount.
    pub async fn unmount_upstream_server(&self, server_id: &str) {
        info!(%server_id, "dynamically unmounting upstream server");

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
        {
            let mut caps_guard = self.capabilities.write().await;
            let removed_caps: Vec<String> = caps_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect();
            for id in removed_caps {
                caps_guard.remove(&id);
                self.event_store.record("capability", &id, "removed");
            }
        }
        {
            let mut res_guard = self.resources.write().await;
            let removed_res: Vec<String> = res_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect();
            for id in removed_res {
                res_guard.remove(&id);
                self.event_store.record("resource", &id, "removed");
            }
        }
        {
            let mut prompts_guard = self.prompts.write().await;
            let removed_prompts: Vec<String> = prompts_guard
                .iter()
                .filter(|(_, meta)| meta.server == server_id)
                .map(|(k, _)| k.clone())
                .collect();
            for id in removed_prompts {
                prompts_guard.remove(&id);
                self.event_store.record("prompt", &id, "removed");
            }
        }

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

    /// Performs graceful shutdown of all upstream servers and background worker queues.
    pub async fn shutdown(&self) {
        info!("initiating graceful shutdown of Warmplane daemon subsystems");

        // 1. Collect all active upstream servers and unmount them
        let server_ids: Vec<String> = {
            let guard = self.servers.read().await;
            guard.keys().cloned().collect()
        };

        for server_id in server_ids {
            self.unmount_upstream_server(&server_id).await;
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
        info!(config_path = %self.config_path, "reloading configuration from disk");

        let config = crate::config::load_config(&self.config_path)?;

        // 1. Update security policy atomically
        {
            let mut pol_guard = self.policy.write().await;
            *pol_guard = Policy::from_config(config.policy.clone());
        }

        // 2. Identify active servers vs disk servers
        let current_server_ids: Vec<String> = {
            let srv_guard = self.servers.read().await;
            srv_guard.keys().cloned().collect()
        };

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
            let res = self
                .mount_upstream_server(
                    server_id,
                    srv_cfg,
                    &config.capability_aliases,
                    &config.resource_aliases,
                    &config.prompt_aliases,
                )
                .await;

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
