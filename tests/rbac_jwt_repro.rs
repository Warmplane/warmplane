// Rust guideline compliant 2026-09-29

//! Claims that the RBAC JWT verifier must not silently ignore.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use warmplane::daemon::policy::Policy;
use warmplane::rbac::models::{JwtConfig, RbacConfig, RolePolicyConfig, TokenAssignment};
use warmplane::rbac::RbacEngine;

const SECRET: &str = "test_secret_key_super_secure";

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Builds an HS256 JWT with the supplied header and payload.
fn make_jwt(header: serde_json::Value, payload: serde_json::Value) -> String {
    let h_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
    let p_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
    let signing_input = format!("{}.{}", h_b64, p_b64);
    let mut mac = Hmac::<Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
    mac.update(signing_input.as_bytes());
    let sig_b64 = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{}.{}", signing_input, sig_b64)
}

/// Builds an HS256 JWT using the standard `alg`/`typ` header.
fn make_standard_jwt(payload: serde_json::Value) -> String {
    make_jwt(serde_json::json!({"alg": "HS256", "typ": "JWT"}), payload)
}

fn engine() -> RbacEngine {
    let mut roles = HashMap::new();
    roles.insert(
        "admin".to_string(),
        RolePolicyConfig {
            description: None,
            allow: vec!["*".to_string()],
            deny: vec![],
            require_approval: vec![],
            redact_keys: vec![],
        },
    );
    roles.insert(
        "readonly".to_string(),
        RolePolicyConfig {
            description: None,
            allow: vec!["fs.read_file".to_string()],
            deny: vec![],
            require_approval: vec![],
            redact_keys: vec![],
        },
    );

    let mut tokens = HashMap::new();
    tokens.insert(
        "placeholder".to_string(),
        TokenAssignment {
            role: "readonly".to_string(),
            tenant_id: None,
            actor_id: None,
            description: None,
        },
    );

    let cfg = RbacConfig {
        enabled: true,
        default_role: "anonymous".to_string(),
        jwt: Some(JwtConfig {
            issuer: Some("https://auth.warmplane.io".to_string()),
            audience: Some("warmplane-daemon".to_string()),
            secret: Some(SECRET.to_string()),
            ..Default::default()
        }),
        tokens,
        roles,
    };
    RbacEngine::new(Some(cfg))
}

/// A token that expired must be rejected, including when the issuer also sent
/// a long-validity companion claim such as `expires_in`.
#[test]
fn expired_token_with_companion_expiry_claim_is_rejected() {
    let now = now_secs();
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now - 60,
        "expires_in": now + 86_400,
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(err.is_err(), "expired token was accepted: {err:?}");
}

/// The `exp` instant is exclusive, so a token that lapsed a second ago fails.
#[test]
fn token_expired_one_second_ago_is_rejected() {
    let now = now_secs();
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now - 1,
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(err.is_err(), "just-expired token was accepted: {err:?}");
}

/// A JWT without an `exp` claim has no bounded lifetime and must be rejected.
/// Accepting one turns a single leaked token into a permanent credential.
#[test]
fn token_without_exp_claim_is_rejected() {
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(
        err.is_err(),
        "token without exp claim was accepted: {err:?}"
    );
}

/// An explicit `exp: null` must fail closed rather than read as "no expiry".
#[test]
fn token_with_null_exp_claim_is_rejected() {
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": serde_json::Value::Null,
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(
        err.is_err(),
        "token with null exp claim was accepted: {err:?}"
    );
}

/// A non-numeric `exp` must be rejected instead of being silently ignored.
#[test]
fn token_with_non_numeric_exp_claim_is_rejected() {
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": "never",
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(
        err.is_err(),
        "token with non-numeric exp claim was accepted: {err:?}"
    );
}

/// A malformed `nbf` must be rejected instead of being silently dropped.
#[test]
fn token_with_non_numeric_nbf_claim_is_rejected() {
    let now = now_secs();
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now + 3600,
        "nbf": "later",
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(
        err.is_err(),
        "token with non-numeric nbf claim was accepted: {err:?}"
    );
}

/// A `nbf` placed in the future must still block authentication.
#[test]
fn token_with_future_nbf_claim_is_rejected() {
    let now = now_secs();
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now + 3600,
        "nbf": now + 600,
        "role": "admin",
    }));

    let err = engine().authenticate(Some(&token), &Policy::default());
    assert!(err.is_err(), "not-yet-valid token was accepted: {err:?}");
}

/// Sanity check: a well-formed token still authenticates after the fix.
#[test]
fn valid_token_still_authenticates() {
    let now = now_secs();
    let token = make_standard_jwt(serde_json::json!({
        "iss": "https://auth.warmplane.io",
        "aud": "warmplane-daemon",
        "exp": now + 3600,
        "nbf": now - 10,
        "role": "admin",
    }));

    let ctx = engine()
        .authenticate(Some(&token), &Policy::default())
        .expect("valid token must authenticate");
    assert_eq!(ctx.role, "admin");
}
