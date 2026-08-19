// Rust guideline compliant 2026-08-15

//! Daemon runtime module managing upstream MCP process connections, runtime state, policies, and lifecycle.

pub mod lifecycle;
pub mod policy;
pub mod server;
pub mod state;
pub mod transport;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all core types, policies, state, and server runners for backward compatibility
pub use policy::{wildcard_match, Policy};
pub use server::{initialize_state, run_daemon};
pub use state::{compute_catalog_version, AppState, AppStateBuilder};
pub use transport::{build_http_headers, resolve_secret, DEFAULT_MCP_PROTOCOL_VERSION};
pub use types::{
    CapabilityMeta, PromptMeta, ResourceMeta, ServerMsg, SharedCapabilities, SharedCatalogVersion,
    SharedPolicy, SharedProfiles, SharedPrompts, SharedResources, SharedServerConfigs,
    SharedServerStatuses, SharedServers, UpstreamCallError,
};
