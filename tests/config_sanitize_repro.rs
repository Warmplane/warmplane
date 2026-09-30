// Rust guideline compliant 2026-09-30

//! Secrets that `McpConfig::sanitize_secrets` must redact before config exposure.
//!
//! `GET /v1/config` serializes the whole config after calling
//! `sanitize_secrets`. Anything that method skips is returned in plaintext.

use serde_json::json;
use warmplane::config::McpConfig;

const REDACTED: &str = "********";

/// Parses a config fixture, failing the test on invalid JSON.
fn parse(raw: serde_json::Value) -> McpConfig {
    serde_json::from_value(raw).expect("fixture must deserialize into McpConfig")
}

/// Builds a webhook block carrying a secret and an auth header.
fn webhook_block() -> serde_json::Value {
    json!({
        "url": "https://hooks.example.com/services/T/B/XYZ",
        "secret": "top-level-webhook-secret",
        "authHeader": "Bearer top-level-auth-header"
    })
}

#[test]
fn rbac_static_token_keys_are_redacted() {
    // The map KEYS are the live bearer secrets: rbac/engine.rs compares each
    // configured key against the inbound Authorization header.
    let mut config = parse(json!({
        "rbac": {
            "enabled": true,
            "defaultRole": "anonymous",
            "tokens": {
                "live-bearer-abc123": { "role": "admin" },
                "live-bearer-def456": { "role": "analyst" }
            },
            "roles": {
                "admin": { "allow": ["*"] }
            }
        }
    }));

    config.sanitize_secrets();

    let rendered = serde_json::to_string(&config).expect("config must serialize");
    assert!(
        !rendered.contains("live-bearer-abc123"),
        "static RBAC admin token leaked in plaintext: {rendered}"
    );
    assert!(
        !rendered.contains("live-bearer-def456"),
        "static RBAC analyst token leaked in plaintext: {rendered}"
    );

    let tokens = config.rbac.as_ref().expect("rbac block").tokens.clone();
    assert_eq!(tokens.len(), 2, "the token map itself must be preserved");
    for (key, assignment) in &tokens {
        assert!(
            key.starts_with(REDACTED),
            "token key must be replaced, not merely the role payload"
        );
        assert!(
            assignment.role == "admin" || assignment.role == "analyst",
            "role assignment must survive redaction, got {}",
            assignment.role
        );
    }
}

#[test]
fn rbac_role_definitions_survive_redaction() {
    // Roles are authorization rules, not secrets. Blanking them would break
    // the config view for operators without improving confidentiality.
    let mut config = parse(json!({
        "rbac": {
            "enabled": true,
            "tokens": { "secret-token": { "role": "analyst" } },
            "roles": {
                "analyst": { "allow": ["fs.*"], "deny": ["docker.*"] }
            }
        }
    }));

    config.sanitize_secrets();

    let rbac = config.rbac.as_ref().expect("rbac block");
    let analyst = rbac.roles.get("analyst").expect("analyst role");
    assert_eq!(analyst.allow, vec!["fs.*".to_string()]);
    assert_eq!(analyst.deny, vec!["docker.*".to_string()]);
}

#[test]
fn profile_policy_webhook_secrets_are_redacted() {
    let mut config = parse(json!({
        "profiles": {
            "prod": {
                "servers": ["fs"],
                "policy": {
                    "webhook": {
                        "url": "https://hooks.example.com/services/P/B/AAA",
                        "secret": "profile-webhook-secret",
                        "authHeader": "Bearer profile-auth-header"
                    }
                }
            }
        }
    }));

    config.sanitize_secrets();

    let rendered = serde_json::to_string(&config).expect("config must serialize");
    assert!(
        !rendered.contains("profile-webhook-secret"),
        "profile webhook secret leaked in plaintext: {rendered}"
    );
    assert!(
        !rendered.contains("profile-auth-header"),
        "profile webhook auth header leaked in plaintext: {rendered}"
    );

    let profile = config.profiles.get("prod").expect("prod profile");
    let webhook = profile
        .policy
        .as_ref()
        .and_then(|p| p.webhook.as_ref())
        .expect("profile policy webhook");
    assert_eq!(webhook.secret.as_deref(), Some(REDACTED));
    assert_eq!(webhook.auth_header.as_deref(), Some(REDACTED));
    assert_eq!(
        webhook.url.as_str(),
        "https://hooks.example.com/services/P/B/AAA",
        "the webhook destination is configuration, not a secret"
    );
}

#[test]
fn already_sanitized_sections_are_not_corrupted() {
    // The existing four call sites must keep working; this pins that a second
    // sanitize pass stays stable rather than double-encoding anything.
    let mut config = parse(json!({
        "authToken": "top-level-auth-token",
        "policy": { "webhook": webhook_block() },
        "mcpServers": {
            "fs": { "command": "npx", "args": ["server"], "env": { "TOKEN": "server-env-secret" } }
        },
        "rbac": { "enabled": true, "tokens": { "rbac-secret": { "role": "admin" } } },
        "profiles": {
            "prod": { "servers": ["fs"], "policy": { "webhook": webhook_block() } }
        }
    }));

    config.sanitize_secrets();
    let first = serde_json::to_string(&config).expect("first pass");
    config.sanitize_secrets();
    let second = serde_json::to_string(&config).expect("second pass");

    assert_eq!(first, second, "sanitize must be idempotent");
    assert!(!first.contains("top-level-auth-token"), "authToken leaked");
    assert!(
        !first.contains("top-level-webhook-secret"),
        "webhook secret leaked"
    );
    assert!(
        !first.contains("server-env-secret"),
        "server env secret leaked"
    );
    assert!(!first.contains("rbac-secret"), "rbac token leaked");
}
