// Rust guideline compliant 2026-08-19

//! Core RBAC engine for token verification, claim resolution, and effective policy computation.

use std::collections::HashMap;
use tracing::warn;

use crate::daemon::Policy;
use crate::rbac::models::{RbacConfig, RolePolicyConfig, TenantContext, TokenAssignment};

/// In-memory RBAC evaluation engine.
#[derive(Clone, Debug)]
pub struct RbacEngine {
    enabled: bool,
    default_role: String,
    tokens: HashMap<String, TokenAssignment>,
    roles: HashMap<String, RolePolicyConfig>,
    jwt_secret: Option<String>,
}

impl RbacEngine {
    /// Initializes an RBAC engine from optional configuration.
    pub fn new(config: Option<RbacConfig>) -> Self {
        let Some(cfg) = config else {
            return Self {
                enabled: false,
                default_role: "anonymous".to_string(),
                tokens: HashMap::new(),
                roles: HashMap::new(),
                jwt_secret: None,
            };
        };

        let jwt_secret = cfg.jwt.as_ref().and_then(|j| j.resolve_secret());

        Self {
            enabled: cfg.enabled,
            default_role: cfg.default_role,
            tokens: cfg.tokens,
            roles: cfg.roles,
            jwt_secret,
        }
    }

    /// Whether RBAC enforcement is globally enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the configured default role name.
    pub fn default_role(&self) -> &str {
        &self.default_role
    }

    /// Resolves the effective policy for a given role name, falling back to base policy.
    pub fn compute_effective_policy(&self, role: &str, base_policy: &Policy) -> Policy {
        if let Some(role_cfg) = self.roles.get(role) {
            let mut policy = base_policy.clone();
            // If the role specifies allow rules, they override base allow
            if !role_cfg.allow.is_empty() {
                policy.allow = role_cfg.allow.clone();
            }
            // Combine role deny rules with base deny rules
            if !role_cfg.deny.is_empty() {
                let mut combined_deny = policy.deny;
                combined_deny.extend(role_cfg.deny.clone());
                combined_deny.dedup();
                policy.deny = combined_deny;
            }
            // Combine role approval rules with base approval rules
            if !role_cfg.require_approval.is_empty() {
                let mut combined_approvals = policy.require_approval;
                combined_approvals.extend(role_cfg.require_approval.clone());
                combined_approvals.dedup();
                policy.require_approval = combined_approvals;
            }
            // Combine role redact keys with base redact keys
            if !role_cfg.redact_keys.is_empty() {
                let mut combined_keys = policy.redact_keys;
                combined_keys.extend(role_cfg.redact_keys.clone());
                combined_keys.dedup();
                policy.redact_keys = combined_keys;
            }
            policy
        } else {
            base_policy.clone()
        }
    }

    /// Authenticates a token or credentials string and builds a `TenantContext`.
    ///
    /// # Arguments
    /// * `token_opt` - Optional bearer token or key string.
    /// * `base_policy` - Baseline global daemon policy.
    ///
    /// # Returns
    /// `Result<TenantContext, String>` - Successfully resolved context or error description.
    pub fn authenticate(
        &self,
        token_opt: Option<&str>,
        base_policy: &Policy,
    ) -> Result<TenantContext, String> {
        if !self.enabled {
            // When RBAC is disabled, return default context with full base policy
            return Ok(TenantContext {
                tenant_id: "default".to_string(),
                role: "admin".to_string(),
                actor_id: None,
                grant_id: None,
                effective_policy: base_policy.clone(),
            });
        }

        let Some(token) = token_opt.map(|t| t.trim()).filter(|t| !t.is_empty()) else {
            // No token provided: check if default_role is allowed
            let effective_policy = self.compute_effective_policy(&self.default_role, base_policy);
            return Ok(TenantContext {
                tenant_id: "default".to_string(),
                role: self.default_role.clone(),
                actor_id: None,
                grant_id: None,
                effective_policy,
            });
        };

        // 1. Check static token map
        if let Some(assignment) = self.tokens.get(token) {
            let effective_policy = self.compute_effective_policy(&assignment.role, base_policy);
            return Ok(TenantContext {
                tenant_id: assignment
                    .tenant_id
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                role: assignment.role.clone(),
                actor_id: assignment.actor_id.clone(),
                grant_id: Some(format!("tok_{}", &token[..token.len().min(8)])),
                effective_policy,
            });
        }

        // 2. Check JWT token format (header.payload.signature)
        if token.contains('.') && token.split('.').count() == 3 {
            if let Ok(ctx) = self.verify_jwt_symmetric(token, base_policy) {
                return Ok(ctx);
            }
        }

        warn!(token_prefix = %&token[..token.len().min(8)], "invalid RBAC token supplied");
        Err("INVALID_CREDENTIALS".to_string())
    }

    /// Verifies HMAC-SHA256 symmetric JWT signature and extracts claims.
    fn verify_jwt_symmetric(&self, token: &str, base_policy: &Policy) -> Result<TenantContext, String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("MALFORMED_JWT".to_string());
        }

        let secret = self.jwt_secret.as_deref().ok_or_else(|| "JWT_SECRET_NOT_CONFIGURED".to_string())?;

        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature_bytes = base64_url_decode(parts[2]).map_err(|_| "INVALID_SIGNATURE_ENCODING".to_string())?;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| format!("HMAC_INIT_ERROR: {}", e))?;
        mac.update(signing_input.as_bytes());

        if mac.verify_slice(&signature_bytes).is_err() {
            return Err("INVALID_JWT_SIGNATURE".to_string());
        }

        // Decode payload
        let payload_bytes = base64_url_decode(parts[1]).map_err(|_| "INVALID_PAYLOAD_ENCODING".to_string())?;
        let payload_val: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|_| "INVALID_PAYLOAD_JSON".to_string())?;

        // Extract standard claims
        if let Some(exp) = payload_val.get("exp").and_then(|v| v.as_u64()) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if now > exp {
                return Err("JWT_EXPIRED".to_string());
            }
        }

        let role = payload_val
            .get("role")
            .or_else(|| payload_val.get("https://warmplane.io/role"))
            .and_then(|v| v.as_str())
            .unwrap_or(&self.default_role)
            .to_string();

        let tenant_id = payload_val
            .get("tenant_id")
            .or_else(|| payload_val.get("https://warmplane.io/tenant_id"))
            .or_else(|| payload_val.get("iss"))
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let actor_id = payload_val
            .get("sub")
            .or_else(|| payload_val.get("actor_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let effective_policy = self.compute_effective_policy(&role, base_policy);

        Ok(TenantContext {
            tenant_id,
            role,
            actor_id,
            grant_id: payload_val.get("jti").and_then(|v| v.as_str()).map(|s| s.to_string()),
            effective_policy,
        })
    }
}

/// Helper function to decode URL-safe base64 strings without padding.
fn base64_url_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.decode(input.trim_end_matches('='))
}
