// Rust guideline compliant 2026-08-19

//! Data models and configuration schema for Multi-Tenant Role-Based Access Control (RBAC).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Multi-tenant RBAC configuration structure.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct RbacConfig {
    /// Whether RBAC enforcement is active.
    #[serde(default)]
    pub enabled: bool,
    /// Default fallback role if no valid credentials supplied.
    #[serde(
        default = "default_fallback_role",
        rename = "defaultRole",
        alias = "default_role"
    )]
    pub default_role: String,
    /// Optional JWT verification configuration for OIDC/SSO integrations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
    /// Map of static API tokens to role assignments.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tokens: HashMap<String, TokenAssignment>,
    /// Role definitions configuring specific capability policies.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub roles: HashMap<String, RolePolicyConfig>,
}

fn default_fallback_role() -> String {
    "anonymous".to_string()
}

/// Configuration for JWT / OIDC token validation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct JwtConfig {
    /// Expected issuer (`iss` claim).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Expected audience (`aud` claim).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    /// JWKS URL for fetching public keys.
    #[serde(
        default,
        rename = "jwksUrl",
        alias = "jwks_url",
        skip_serializing_if = "Option::is_none"
    )]
    pub jwks_url: Option<String>,
    /// Custom claim name containing the role (default: "role" or "https://warmplane.io/role").
    #[serde(
        default,
        rename = "roleClaim",
        alias = "role_claim",
        skip_serializing_if = "Option::is_none"
    )]
    pub role_claim: Option<String>,
    /// Custom claim name containing the tenant ID (default: "tenant_id" or "https://warmplane.io/tenant_id").
    #[serde(
        default,
        rename = "tenantClaim",
        alias = "tenant_claim",
        skip_serializing_if = "Option::is_none"
    )]
    pub tenant_claim: Option<String>,
    /// Static shared secret (HMAC-SHA256) for symmetric JWT verification (if JWKS not used).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Environment variable name holding the JWT shared secret.
    #[serde(
        default,
        rename = "secretEnv",
        alias = "secret_env",
        skip_serializing_if = "Option::is_none"
    )]
    pub secret_env: Option<String>,
}

impl JwtConfig {
    /// Resolves the JWT HMAC secret from environment variable or direct string.
    pub fn resolve_secret(&self) -> Option<String> {
        if let Some(ref env_name) = self.secret_env {
            if let Ok(val) = std::env::var(env_name) {
                if !val.trim().is_empty() {
                    return Some(val);
                }
            }
        }
        self.secret.clone()
    }
}

/// Token binding assignment for static API tokens.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TokenAssignment {
    /// Assigned role name.
    pub role: String,
    /// Tenant or organization identifier.
    #[serde(
        default,
        rename = "tenantId",
        alias = "tenant_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub tenant_id: Option<String>,
    /// Optional fixed actor identifier for audit trail.
    #[serde(
        default,
        rename = "actorId",
        alias = "actor_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub actor_id: Option<String>,
    /// Human readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Role capability and execution policy rules.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct RolePolicyConfig {
    /// Description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Capability/resource patterns allowed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    /// Capability/resource patterns denied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
    /// Capability patterns requiring operator approval.
    #[serde(
        default,
        rename = "requireApproval",
        alias = "require_approval",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub require_approval: Vec<String>,
    /// Sensitive keys to redact for this role.
    #[serde(
        default,
        rename = "redactKeys",
        alias = "redact_keys",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub redact_keys: Vec<String>,
}

/// Verified tenant caller context attached to HTTP request extensions.
#[derive(Clone, Debug, PartialEq)]
pub struct TenantContext {
    /// Verified tenant identifier (e.g. "default", "acme-corp").
    pub tenant_id: String,
    /// Verified role name (e.g. "admin", "analyst", "anonymous").
    pub role: String,
    /// Optional verified actor identifier.
    pub actor_id: Option<String>,
    /// Optional grant or token identifier.
    pub grant_id: Option<String>,
    /// Effective policy computed for this role.
    pub effective_policy: crate::daemon::Policy,
}

impl TenantContext {
    /// Constructs an unauthenticated fallback context using default role.
    pub fn default_for_policy(default_role: &str, base_policy: &crate::daemon::Policy) -> Self {
        Self {
            tenant_id: "default".to_string(),
            role: default_role.to_string(),
            actor_id: None,
            grant_id: None,
            effective_policy: base_policy.clone(),
        }
    }
}
