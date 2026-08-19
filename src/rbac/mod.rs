// Rust guideline compliant 2026-08-19

//! Multi-Tenant Role-Based Access Control (RBAC) module.
//!
//! Provides token-to-role mappings, JWT claim verification, catalog partitioning,
//! and per-role execution boundary enforcement.

pub mod engine;
pub mod middleware;
pub mod models;

#[cfg(test)]
mod tests;

pub use engine::RbacEngine;
pub use middleware::rbac_auth_middleware;
pub use models::{JwtConfig, RbacConfig, RolePolicyConfig, TenantContext, TokenAssignment};
