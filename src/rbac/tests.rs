// Rust guideline compliant 2026-08-19

//! Unit tests for RBAC engine, token authentication, and role policy computation.

use super::engine::RbacEngine;
use super::models::{JwtConfig, RbacConfig, RolePolicyConfig, TokenAssignment};
use crate::daemon::Policy;
use std::collections::HashMap;

fn sample_rbac_config() -> RbacConfig {
    let mut tokens = HashMap::new();
    tokens.insert(
        "admin_key_123".to_string(),
        TokenAssignment {
            role: "admin".to_string(),
            tenant_id: Some("corp".to_string()),
            actor_id: Some("agent-admin".to_string()),
            description: Some("Admin token".to_string()),
        },
    );
    tokens.insert(
        "analyst_key_456".to_string(),
        TokenAssignment {
            role: "analyst".to_string(),
            tenant_id: Some("analytics".to_string()),
            actor_id: Some("agent-analyst".to_string()),
            description: Some("Analyst token".to_string()),
        },
    );

    let mut roles = HashMap::new();
    roles.insert(
        "admin".to_string(),
        RolePolicyConfig {
            description: Some("Full access".to_string()),
            allow: vec!["*".to_string()],
            deny: vec![],
            require_approval: vec![],
            redact_keys: vec![],
        },
    );
    roles.insert(
        "analyst".to_string(),
        RolePolicyConfig {
            description: Some("Read only queries".to_string()),
            allow: vec!["db.query".to_string(), "fs.read_*".to_string()],
            deny: vec!["*.write_*".to_string(), "docker.*".to_string()],
            require_approval: vec!["db.query_large".to_string()],
            redact_keys: vec!["ssn".to_string(), "password".to_string()],
        },
    );

    RbacConfig {
        enabled: true,
        default_role: "anonymous".to_string(),
        jwt: Some(JwtConfig {
            secret: Some("test_secret_key_super_secure".to_string()),
            ..Default::default()
        }),
        tokens,
        roles,
    }
}

#[test]
fn test_rbac_disabled_returns_admin_context() {
    let engine = RbacEngine::new(None);
    let base_policy = Policy::default();
    let ctx = engine.authenticate(None, &base_policy).unwrap();
    assert_eq!(ctx.role, "admin");
    assert_eq!(ctx.tenant_id, "default");
}

#[test]
fn test_rbac_static_token_lookup() {
    let engine = RbacEngine::new(Some(sample_rbac_config()));
    let base_policy = Policy::default();

    // Valid admin token
    let admin_ctx = engine
        .authenticate(Some("admin_key_123"), &base_policy)
        .unwrap();
    assert_eq!(admin_ctx.role, "admin");
    assert_eq!(admin_ctx.tenant_id, "corp");
    assert_eq!(admin_ctx.actor_id.as_deref(), Some("agent-admin"));
    assert!(admin_ctx.effective_policy.allows("docker.run"));

    // Valid analyst token
    let analyst_ctx = engine
        .authenticate(Some("analyst_key_456"), &base_policy)
        .unwrap();
    assert_eq!(analyst_ctx.role, "analyst");
    assert_eq!(analyst_ctx.tenant_id, "analytics");
    assert!(analyst_ctx.effective_policy.allows("db.query"));
    assert!(analyst_ctx.effective_policy.allows("fs.read_file"));
    assert!(!analyst_ctx.effective_policy.allows("docker.run"));
    assert!(!analyst_ctx.effective_policy.allows("db.write_record"));
    assert!(analyst_ctx
        .effective_policy
        .requires_approval("db.query_large"));
    assert!(analyst_ctx
        .effective_policy
        .redact_keys
        .contains(&"ssn".to_string()));

    // Invalid token
    let err = engine.authenticate(Some("invalid_token_999"), &base_policy);
    assert!(err.is_err());
}

#[test]
fn test_rbac_default_role_fallback() {
    let engine = RbacEngine::new(Some(sample_rbac_config()));
    let base_policy = Policy::default();

    let anon_ctx = engine.authenticate(None, &base_policy).unwrap();
    assert_eq!(anon_ctx.role, "anonymous");
    assert_eq!(anon_ctx.tenant_id, "default");
}

#[test]
fn test_rbac_jwt_iss_aud_nbf_validation() {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "test_secret_key_super_secure";
    let make_jwt = |payload: serde_json::Value| -> String {
        let header = serde_json::json!({"alg": "HS256", "typ": "JWT"});
        let h_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let p_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        let signing_input = format!("{}.{}", h_b64, p_b64);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signing_input.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        format!("{}.{}", signing_input, sig_b64)
    };

    let mut cfg = sample_rbac_config();
    cfg.jwt = Some(JwtConfig {
        issuer: Some("https://auth.warmplane.io".to_string()),
        audience: Some("warmplane-daemon".to_string()),
        secret: Some(secret.to_string()),
        ..Default::default()
    });

    let engine = RbacEngine::new(Some(cfg));
    let base_policy = Policy::default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 1. Valid token
    let valid_token = make_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now + 3600,
        "nbf": now - 10,
        "role": "admin"
    }));
    let ctx = engine
        .authenticate(Some(&valid_token), &base_policy)
        .unwrap();
    assert_eq!(ctx.role, "admin");

    // 2. Invalid issuer
    let bad_iss_token = make_jwt(serde_json::json!({
        "iss": "https://rogue-service.com",
        "aud": "warmplane-daemon",
        "exp": now + 3600,
        "role": "admin"
    }));
    assert!(engine
        .authenticate(Some(&bad_iss_token), &base_policy)
        .is_err());

    // 3. Invalid audience
    let bad_aud_token = make_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "other-service",
        "exp": now + 3600,
        "role": "admin"
    }));
    assert!(engine
        .authenticate(Some(&bad_aud_token), &base_policy)
        .is_err());

    // 4. Token not valid yet (nbf in future)
    let nbf_token = make_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now + 3600,
        "nbf": now + 1000,
        "role": "admin"
    }));
    assert!(engine.authenticate(Some(&nbf_token), &base_policy).is_err());
}
