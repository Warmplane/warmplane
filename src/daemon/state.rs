// Rust guideline compliant 2026-08-15

//! Shared daemon application state and builder implementations (`M-INIT-BUILDER`).

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    config::{ServerConfig, DEFAULT_TOOL_TIMEOUT_MS},
    daemon::{
        policy::Policy,
        types::{
            CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg, SharedCapabilities,
            SharedCatalogVersion, SharedPolicy, SharedPrompts, SharedResources,
            SharedServerConfigs, SharedServerStatuses, SharedServers,
        },
    },
};

/// Global application state shared across Axum HTTP handlers in the daemon.
#[derive(Clone)]
pub struct AppState {
    /// Active communication channels to upstream server workers.
    pub servers: SharedServers,
    /// Compact capability catalog metadata map.
    pub capabilities: SharedCapabilities,
    /// Compact resource catalog metadata map.
    pub resources: SharedResources,
    /// Compact prompt catalog metadata map.
    pub prompts: SharedPrompts,
    /// Global tool execution timeout in milliseconds.
    pub tool_timeout_ms: u64,
    /// Security policy rules.
    pub policy: SharedPolicy,
    /// Hybrid search engine instance.
    pub search_engine: Arc<crate::search::HybridSearchEngine>,
    /// SHA256 catalog ETag version string.
    pub catalog_version: SharedCatalogVersion,
    /// Event store for catalog changes.
    pub event_store: Arc<crate::catalog::CatalogEventStore>,
    /// Idempotency deduplication store.
    pub idempotency_store: Arc<crate::idempotency::IdempotencyStore>,
    /// Active operation tracking registry for cancellation.
    pub operation_registry: crate::operations::OperationRegistry,
    /// Real-time broadcast channel for resource update notifications.
    pub resource_update_tx: tokio::sync::broadcast::Sender<crate::catalog::ResourceUpdateEvent>,
    /// Path to active config file.
    pub config_path: String,
    /// Active server configurations keyed by server identifier.
    pub server_configs: SharedServerConfigs,
    /// Active server protocol version & transport info.
    pub server_statuses: SharedServerStatuses,
    /// Live total catalog / capabilities requests.
    pub total_catalog_requests: Arc<std::sync::atomic::AtomicU64>,
    /// Live ETag 304 cache hits.
    pub total_etag_hits: Arc<std::sync::atomic::AtomicU64>,
    /// Live total capability executions.
    pub total_tool_calls: Arc<std::sync::atomic::AtomicU64>,
    /// Live cumulative tool execution duration in microseconds.
    pub total_tool_duration_us: Arc<std::sync::atomic::AtomicU64>,
    /// Optional OAuth proxy server port.
    pub oauth_proxy_port: Option<u16>,
    /// Central OAuth registry.
    pub oauth_registry: crate::oauth2::OAuthRegistry,
    /// Human-in-the-loop approval registry.
    pub approval_registry: crate::approvals::ApprovalRegistry,
    /// Append-only WORM audit store.
    pub audit_store: crate::audit::SharedAuditStore,
    /// Non-blocking async audit event dispatcher handle.
    pub audit_handle: crate::audit::AuditHandle,
}

impl AppState {
    /// Creates a new `AppStateBuilder` for constructing `AppState` (`M-INIT-BUILDER`).
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }
}

/// Builder for constructing `AppState` instances (`M-INIT-BUILDER`).
#[derive(Default)]
pub struct AppStateBuilder {
    servers: Option<SharedServers>,
    capabilities: Option<SharedCapabilities>,
    resources: Option<SharedResources>,
    prompts: Option<SharedPrompts>,
    tool_timeout_ms: Option<u64>,
    policy: Option<SharedPolicy>,
    search_engine: Option<Arc<crate::search::HybridSearchEngine>>,
    catalog_version: Option<SharedCatalogVersion>,
    event_store: Option<Arc<crate::catalog::CatalogEventStore>>,
    idempotency_store: Option<Arc<crate::idempotency::IdempotencyStore>>,
    operation_registry: Option<crate::operations::OperationRegistry>,
    resource_update_tx: Option<tokio::sync::broadcast::Sender<crate::catalog::ResourceUpdateEvent>>,
    config_path: Option<String>,
    server_configs: Option<SharedServerConfigs>,
    server_statuses: Option<SharedServerStatuses>,
    oauth_proxy_port: Option<u16>,
    oauth_registry: Option<crate::oauth2::OAuthRegistry>,
    approval_registry: Option<crate::approvals::ApprovalRegistry>,
    audit_store: Option<crate::audit::SharedAuditStore>,
    audit_handle: Option<crate::audit::AuditHandle>,
}

#[allow(dead_code)]
impl AppStateBuilder {
    /// Sets upstream servers map from raw HashMap.
    pub fn servers(
        mut self,
        servers: HashMap<String, tokio::sync::mpsc::Sender<ServerMsg>>,
    ) -> Self {
        self.servers = Some(Arc::new(RwLock::new(servers)));
        self
    }

    /// Sets upstream servers map from Arc.
    pub fn servers_arc(mut self, servers: SharedServers) -> Self {
        self.servers = Some(servers);
        self
    }

    /// Sets capabilities map from raw HashMap.
    pub fn capabilities(mut self, capabilities: HashMap<String, CapabilityMeta>) -> Self {
        self.capabilities = Some(Arc::new(RwLock::new(capabilities)));
        self
    }

    /// Sets capabilities map from Arc.
    pub fn capabilities_arc(mut self, capabilities: SharedCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Sets resources map from raw HashMap.
    pub fn resources(mut self, resources: HashMap<String, ResourceMeta>) -> Self {
        self.resources = Some(Arc::new(RwLock::new(resources)));
        self
    }

    /// Sets resources map from Arc.
    pub fn resources_arc(mut self, resources: SharedResources) -> Self {
        self.resources = Some(resources);
        self
    }

    /// Sets prompts map from raw HashMap.
    pub fn prompts(mut self, prompts: HashMap<String, PromptMeta>) -> Self {
        self.prompts = Some(Arc::new(RwLock::new(prompts)));
        self
    }

    /// Sets prompts map from Arc.
    pub fn prompts_arc(mut self, prompts: SharedPrompts) -> Self {
        self.prompts = Some(prompts);
        self
    }

    /// Sets global tool execution timeout in milliseconds.
    pub fn tool_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.tool_timeout_ms = Some(timeout_ms);
        self
    }

    /// Sets policy from concrete struct.
    pub fn policy(mut self, policy: Policy) -> Self {
        self.policy = Some(Arc::new(RwLock::new(policy)));
        self
    }

    /// Sets policy from Arc.
    pub fn policy_arc(mut self, policy: Arc<RwLock<Policy>>) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Sets hybrid search engine instance.
    pub fn search_engine(mut self, search_engine: Arc<crate::search::HybridSearchEngine>) -> Self {
        self.search_engine = Some(search_engine);
        self
    }

    /// Sets catalog version from string.
    pub fn catalog_version(mut self, version: impl Into<String>) -> Self {
        self.catalog_version = Some(Arc::new(RwLock::new(version.into())));
        self
    }

    /// Sets catalog version from Arc.
    pub fn catalog_version_arc(mut self, version: Arc<RwLock<String>>) -> Self {
        self.catalog_version = Some(version);
        self
    }

    /// Sets catalog event store instance.
    pub fn event_store(mut self, store: Arc<crate::catalog::CatalogEventStore>) -> Self {
        self.event_store = Some(store);
        self
    }

    /// Sets idempotency deduplication store instance.
    pub fn idempotency_store(mut self, store: Arc<crate::idempotency::IdempotencyStore>) -> Self {
        self.idempotency_store = Some(store);
        self
    }

    /// Sets in-flight operation registry.
    pub fn operation_registry(mut self, registry: crate::operations::OperationRegistry) -> Self {
        self.operation_registry = Some(registry);
        self
    }

    /// Sets active configuration file path.
    pub fn config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Sets server configs map from raw HashMap.
    pub fn server_configs(mut self, configs: HashMap<String, ServerConfig>) -> Self {
        self.server_configs = Some(Arc::new(RwLock::new(configs)));
        self
    }

    /// Sets server configs map from Arc.
    pub fn server_configs_arc(
        mut self,
        configs: Arc<RwLock<HashMap<String, ServerConfig>>>,
    ) -> Self {
        self.server_configs = Some(configs);
        self
    }

    /// Sets server statuses map from raw HashMap.
    pub fn server_statuses(mut self, statuses: HashMap<String, serde_json::Value>) -> Self {
        self.server_statuses = Some(Arc::new(RwLock::new(statuses)));
        self
    }

    /// Sets server statuses map from Arc.
    pub fn server_statuses_arc(
        mut self,
        statuses: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.server_statuses = Some(statuses);
        self
    }

    /// Sets optional OAuth proxy server port.
    pub fn oauth_proxy_port(mut self, port: Option<u16>) -> Self {
        self.oauth_proxy_port = port;
        self
    }

    /// Sets OAuth client registry.
    pub fn oauth_registry(mut self, registry: crate::oauth2::OAuthRegistry) -> Self {
        self.oauth_registry = Some(registry);
        self
    }

    /// Sets HITL approval registry.
    pub fn approval_registry(mut self, registry: crate::approvals::ApprovalRegistry) -> Self {
        self.approval_registry = Some(registry);
        self
    }

    /// Sets append-only WORM audit store.
    pub fn audit_store(mut self, store: crate::audit::SharedAuditStore) -> Self {
        self.audit_store = Some(store);
        self
    }

    /// Sets non-blocking audit event handle.
    pub fn audit_handle(mut self, handle: crate::audit::AuditHandle) -> Self {
        self.audit_handle = Some(handle);
        self
    }

    /// Builds the `AppState` struct instance with defaults for unspecified fields.
    pub fn build(self) -> AppState {
        let audit_store = self
            .audit_store
            .unwrap_or_else(|| Arc::new(crate::audit::AuditStore::in_memory()));
        let audit_handle = self.audit_handle.unwrap_or_else(|| {
            crate::audit::spawn_audit_worker(
                audit_store.clone(),
                crate::audit::DEFAULT_AUDIT_BUFFER_CAPACITY,
                crate::audit::DEFAULT_AUDIT_FLUSH_INTERVAL_MS,
                crate::audit::DEFAULT_AUDIT_MAX_BATCH_SIZE,
            )
        });

        AppState {
            servers: self.servers.unwrap_or_default(),
            capabilities: self.capabilities.unwrap_or_default(),
            resources: self.resources.unwrap_or_default(),
            prompts: self.prompts.unwrap_or_default(),
            tool_timeout_ms: self.tool_timeout_ms.unwrap_or(DEFAULT_TOOL_TIMEOUT_MS),
            policy: self.policy.unwrap_or_default(),
            search_engine: self
                .search_engine
                .unwrap_or_else(|| Arc::new(crate::search::HybridSearchEngine::new())),
            catalog_version: self.catalog_version.unwrap_or_default(),
            event_store: self
                .event_store
                .unwrap_or_else(|| Arc::new(crate::catalog::CatalogEventStore::new())),
            idempotency_store: self
                .idempotency_store
                .unwrap_or_else(|| Arc::new(crate::idempotency::IdempotencyStore::default())),
            operation_registry: self.operation_registry.unwrap_or_default(),
            resource_update_tx: self
                .resource_update_tx
                .unwrap_or_else(|| tokio::sync::broadcast::channel(64).0),
            config_path: self.config_path.unwrap_or_default(),
            server_configs: self.server_configs.unwrap_or_default(),
            server_statuses: self.server_statuses.unwrap_or_default(),
            total_catalog_requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_etag_hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_tool_calls: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_tool_duration_us: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            oauth_proxy_port: self.oauth_proxy_port,
            oauth_registry: self.oauth_registry.unwrap_or_default(),
            approval_registry: self.approval_registry.unwrap_or_default(),
            audit_store,
            audit_handle,
        }
    }
}

/// Computes deterministic SHA256 ETag version string over catalog keys.
///
/// # Arguments
/// * `capabilities` - Capabilities catalog map.
/// * `resources` - Resources catalog map.
/// * `prompts` - Prompts catalog map.
///
/// # Returns
/// SHA256 catalog version string prefixed with `sha256:`.
pub fn compute_catalog_version(
    capabilities: &HashMap<String, CapabilityMeta>,
    resources: &HashMap<String, ResourceMeta>,
    prompts: &HashMap<String, PromptMeta>,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut cap_keys: Vec<_> = capabilities.keys().collect();
    cap_keys.sort();
    for k in cap_keys {
        hasher.update(k.as_bytes());
    }
    let mut res_keys: Vec<_> = resources.keys().collect();
    res_keys.sort();
    for k in res_keys {
        hasher.update(k.as_bytes());
    }
    let mut prompt_keys: Vec<_> = prompts.keys().collect();
    prompt_keys.sort();
    for k in prompt_keys {
        hasher.update(k.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}
