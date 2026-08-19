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
    let admin_ctx = engine.authenticate(Some("admin_key_123"), &base_policy).unwrap();
    assert_eq!(admin_ctx.role, "admin");
    assert_eq!(admin_ctx.tenant_id, "corp");
    assert_eq!(admin_ctx.actor_id.as_deref(), Some("agent-admin"));
    assert!(admin_ctx.effective_policy.allows("docker.run"));

    // Valid analyst token
    let analyst_ctx = engine.authenticate(Some("analyst_key_456"), &base_policy).unwrap();
    assert_eq!(analyst_ctx.role, "analyst");
    assert_eq!(analyst_ctx.tenant_id, "analytics");
    assert!(analyst_ctx.effective_policy.allows("db.query"));
    assert!(analyst_ctx.effective_policy.allows("fs.read_file"));
    assert!(!analyst_ctx.effective_policy.allows("docker.run"));
    assert!(!analyst_ctx.effective_policy.allows("db.write_record"));
    assert!(analyst_ctx.effective_policy.requires_approval("db.query_large"));
    assert!(analyst_ctx.effective_policy.redact_keys.contains(&"ssn".to_string()));

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
