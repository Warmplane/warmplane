// Rust guideline compliant 2026-11-16
//! Core RBAC engine for token verification, claim resolution, and effective policy computation.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use tracing::warn;

use crate::daemon::Policy;
use crate::rbac::models::{RbacConfig, RolePolicyConfig, TenantContext, TokenAssignment};

/// In-memory RBAC evaluation engine.
#[derive(Clone, Debug)]
pub struct RbacEngine {
    config: Arc<RwLock<RbacState>>,
}

#[derive(Clone, Debug)]
struct RbacState {
    enabled: bool,
    default_role: String,
    tokens: HashMap<String, TokenAssignment>,
    roles: HashMap<String, RolePolicyConfig>,
    jwt_secret: Option<String>,
    jwt_config: Option<crate::rbac::models::JwtConfig>,
}

impl RbacEngine {
    /// Initializes an RBAC engine from optional configuration.
    pub fn new(config: Option<RbacConfig>) -> Self {
        Self {
            config: Arc::new(RwLock::new(RbacState::from_config(config))),
        }
    }

    /// Replaces the active RBAC configuration shared by all clones of this engine.
    pub fn reload(&self, config: Option<RbacConfig>) {
        *self.write_state() = RbacState::from_config(config);
    }

    fn read_state(&self) -> RwLockReadGuard<'_, RbacState> {
        self.config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_state(&self) -> RwLockWriteGuard<'_, RbacState> {
        self.config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Whether RBAC enforcement is globally enabled.
    pub fn is_enabled(&self) -> bool {
        self.read_state().enabled
    }

    /// Returns the configured default role name.
    pub fn default_role(&self) -> String {
        self.read_state().default_role.clone()
    }

    /// Returns whether a role can be resolved under the active RBAC configuration.
    pub fn recognizes_role(&self, role: &str) -> bool {
        let config = self.read_state();
        !config.enabled || role == config.default_role || config.roles.contains_key(role)
    }

    /// Revalidates stored tenant and role provenance without retaining caller credentials.
    pub fn recognizes_provenance(
        &self,
        tenant_id: &str,
        role: &str,
        actor_id: Option<&str>,
        grant_id: Option<&str>,
    ) -> bool {
        let config = self.read_state();
        if !config.enabled {
            return tenant_id == "default";
        }
        if role != config.default_role && !config.roles.contains_key(role) {
            return false;
        }
        if let Some(grant_id) = grant_id.filter(|grant| grant.starts_with("tok_")) {
            return config.tokens.iter().any(|(token, assignment)| {
                format!("tok_{}", byte_prefix(token, 8)) == grant_id
                    && assignment.role == role
                    && assignment.tenant_id.as_deref().unwrap_or("default") == tenant_id
                    && assignment.actor_id.as_deref() == actor_id
            });
        }
        tenant_id == "default" || grant_id.is_some()
    }

    /// Resolves the effective policy for a given role name, falling back to base policy.
    pub fn compute_effective_policy(&self, role: &str, base_policy: &Policy) -> Policy {
        compute_effective_policy(&self.read_state(), role, base_policy)
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
        let config = self.read_state();
        if !config.enabled {
            return Ok(TenantContext {
                tenant_id: "default".to_string(),
                role: "admin".to_string(),
                actor_id: None,
                grant_id: None,
                grant_expires_at_epoch_secs: None,
                effective_policy: base_policy.clone(),
            });
        }

        let Some(token) = token_opt.map(str::trim).filter(|token| !token.is_empty()) else {
            let role = config.default_role.clone();
            let effective_policy = compute_effective_policy(&config, &role, base_policy);
            return Ok(TenantContext {
                tenant_id: "default".to_string(),
                role,
                actor_id: None,
                grant_id: None,
                grant_expires_at_epoch_secs: None,
                effective_policy,
            });
        };

        use subtle::ConstantTimeEq;
        for (configured_token, assignment) in &config.tokens {
            let is_match: bool = configured_token.as_bytes().ct_eq(token.as_bytes()).into();
            if is_match {
                let effective_policy =
                    compute_effective_policy(&config, &assignment.role, base_policy);
                return Ok(TenantContext {
                    tenant_id: assignment
                        .tenant_id
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    role: assignment.role.clone(),
                    actor_id: assignment.actor_id.clone(),
                    grant_id: Some(format!("tok_{}", byte_prefix(token, 8))),
                    grant_expires_at_epoch_secs: None,
                    effective_policy,
                });
            }
        }
        drop(config);

        if token.contains('.') && token.split('.').count() == 3 {
            if let Ok(context) = self.verify_jwt_symmetric(token, base_policy) {
                return Ok(context);
            }
        }

        if token.len() >= 32 {
            warn!(token_prefix = %byte_prefix(token, 8), "invalid RBAC token supplied");
        } else {
            warn!(token_prefix = "[redacted]", "invalid RBAC token supplied");
        }
        Err("INVALID_CREDENTIALS".to_string())
    }

    /// Verifies HMAC-SHA256 symmetric JWT signature and extracts claims.
    fn verify_jwt_symmetric(
        &self,
        token: &str,
        base_policy: &Policy,
    ) -> Result<TenantContext, String> {
        let config = self.read_state();
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("MALFORMED_JWT".to_string());
        }

        let secret = config
            .jwt_secret
            .as_deref()
            .ok_or_else(|| "JWT_SECRET_NOT_CONFIGURED".to_string())?;

        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature_bytes =
            base64_url_decode(parts[2]).map_err(|_| "INVALID_SIGNATURE_ENCODING".to_string())?;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| format!("HMAC_INIT_ERROR: {}", e))?;
        mac.update(signing_input.as_bytes());

        if mac.verify_slice(&signature_bytes).is_err() {
            return Err("INVALID_JWT_SIGNATURE".to_string());
        }

        let payload_bytes =
            base64_url_decode(parts[1]).map_err(|_| "INVALID_PAYLOAD_ENCODING".to_string())?;
        let payload_val: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|_| "INVALID_PAYLOAD_JSON".to_string())?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let expiry = match payload_val.get("exp") {
            Some(serde_json::Value::Number(number)) => match number.as_u64() {
                Some(expiry) if now <= expiry => expiry,
                _ => return Err("JWT_EXPIRED".to_string()),
            },
            Some(_) => return Err("JWT_EXPIRED".to_string()),
            None => return Err("JWT_EXPIRATION_MISSING".to_string()),
        };
        if let Some(nbf_value) = payload_val.get("nbf") {
            let Some(nbf) = nbf_value.as_u64() else {
                return Err("JWT_INVALID_NOT_BEFORE".to_string());
            };
            if now < nbf {
                return Err("JWT_NOT_YET_VALID".to_string());
            }
        }

        if let Some(ref jwt_config) = config.jwt_config {
            if let Some(ref expected_iss) = jwt_config.issuer {
                let actual_iss = payload_val.get("iss").and_then(|value| value.as_str());
                if actual_iss != Some(expected_iss.as_str()) {
                    return Err("INVALID_JWT_ISSUER".to_string());
                }
            }
            if let Some(ref expected_aud) = jwt_config.audience {
                let aud_match = match payload_val.get("aud") {
                    Some(serde_json::Value::String(audience)) => audience == expected_aud,
                    Some(serde_json::Value::Array(audiences)) => audiences
                        .iter()
                        .any(|value| value.as_str() == Some(expected_aud.as_str())),
                    _ => false,
                };
                if !aud_match {
                    return Err("INVALID_JWT_AUDIENCE".to_string());
                }
            }
        }

        let role = payload_val
            .get("role")
            .or_else(|| payload_val.get("https://warmplane.io/role"))
            .and_then(|value| value.as_str())
            .unwrap_or(&config.default_role)
            .to_string();
        let tenant_id = payload_val
            .get("tenant_id")
            .or_else(|| payload_val.get("https://warmplane.io/tenant_id"))
            .or_else(|| payload_val.get("iss"))
            .and_then(|value| value.as_str())
            .unwrap_or("default")
            .to_string();
        let actor_id = payload_val
            .get("sub")
            .or_else(|| payload_val.get("actor_id"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string);
        let effective_policy = compute_effective_policy(&config, &role, base_policy);

        Ok(TenantContext {
            tenant_id,
            role,
            actor_id,
            grant_id: payload_val
                .get("jti")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            grant_expires_at_epoch_secs: Some(expiry),
            effective_policy,
        })
    }
}

impl RbacState {
    fn from_config(config: Option<RbacConfig>) -> Self {
        let Some(config) = config else {
            return Self {
                enabled: false,
                default_role: "anonymous".to_string(),
                tokens: HashMap::new(),
                roles: HashMap::new(),
                jwt_secret: None,
                jwt_config: None,
            };
        };
        Self {
            enabled: config.enabled,
            default_role: config.default_role,
            tokens: config.tokens,
            roles: config.roles,
            jwt_secret: config.jwt.as_ref().and_then(|jwt| jwt.resolve_secret()),
            jwt_config: config.jwt,
        }
    }
}

fn compute_effective_policy(config: &RbacState, role: &str, base_policy: &Policy) -> Policy {
    if let Some(role_cfg) = config.roles.get(role) {
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

/// Returns the first `max_bytes` bytes of `value` without splitting a character.
///
/// Credentials are attacker supplied and may hold multi-byte UTF-8. Slicing a
/// `&str` at a fixed byte offset panics whenever that offset falls inside a
/// character, so the cut is walked back to the nearest character boundary.
///
/// # Arguments
/// * `value` - Arbitrary text to truncate.
/// * `max_bytes` - Upper bound on the returned length in bytes.
///
/// # Returns
/// A prefix of `value` no longer than `max_bytes`.
fn byte_prefix(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }

    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

/// Helper function to decode URL-safe base64 strings without padding.
fn base64_url_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.decode(input.trim_end_matches('='))
}
