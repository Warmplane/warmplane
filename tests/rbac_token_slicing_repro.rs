// Rust guideline compliant 2026-09-29

//! Regression tests for byte-index slicing of non-ASCII credentials.
//!
//! `RbacEngine::authenticate` builds a grant id from the first eight bytes of a
//! static token and logs an eight byte prefix of any rejected credential. Both
//! slice the `&str` directly, so an index that lands inside a multi-byte UTF-8
//! sequence panics. Credentials are attacker-supplied input, so the offset is
//! always untrusted.

use std::collections::HashMap;
use warmplane::daemon::Policy;
use warmplane::rbac::{RbacConfig, RbacEngine, RolePolicyConfig, TokenAssignment};

/// Builds an enabled engine with a single static token mapped to `analyst`.
fn engine_with_token(token: &str) -> RbacEngine {
    let mut tokens = HashMap::new();
    tokens.insert(
        token.to_string(),
        TokenAssignment {
            role: "analyst".to_string(),
            tenant_id: None,
            actor_id: None,
            description: None,
        },
    );

    let mut roles = HashMap::new();
    roles.insert(
        "analyst".to_string(),
        RolePolicyConfig {
            description: None,
            allow: vec!["fs.read_*".to_string()],
            deny: vec![],
            require_approval: vec![],
            redact_keys: vec![],
        },
    );

    RbacEngine::new(Some(RbacConfig {
        enabled: true,
        default_role: "anonymous".to_string(),
        tokens,
        roles,
        jwt: None,
    }))
}

#[test]
fn static_token_whose_prefix_splits_a_character_is_accepted() {
    // Six ASCII bytes, then a three-byte euro sign, then more text. Byte
    // offset 8 therefore lands one byte inside the euro sign.
    let token = concat!("abc123", "\u{20AC}", "tail-tail-tail-secret");
    assert!(token.len() > 8, "token must exceed the slice width");
    assert!(
        !token.is_char_boundary(8),
        "byte offset 8 must split a character for this input"
    );

    let engine = engine_with_token(token);
    let ctx = engine
        .authenticate(Some(token), &Policy::default())
        .expect("a multi-byte static token must authenticate, not panic");

    assert_eq!(ctx.role, "analyst");
    assert!(ctx.grant_id.is_some(), "grant id must be derived");
}

#[test]
fn rejected_token_splitting_a_character_is_reported_not_panicking() {
    let token = concat!("abc123", "\u{20AC}", "tail-tail-not-a-real-token");
    assert!(
        !token.is_char_boundary(8),
        "offset 8 must split a character"
    );

    let engine = engine_with_token("valid-static-token");
    let err = engine
        .authenticate(Some(token), &Policy::default())
        .expect_err("an unknown token must be rejected");
    assert_eq!(err, "INVALID_CREDENTIALS");
}

#[test]
fn three_byte_character_prefix_is_accepted() {
    // A single three-byte character followed by filler: offset 8 lands inside
    // the second character.
    let token = "€€€€€-secret-key";
    assert!(
        !token.is_char_boundary(8),
        "offset 8 must split a character"
    );

    let engine = engine_with_token(token);
    let ctx = engine
        .authenticate(Some(token), &Policy::default())
        .expect("a three-byte-prefix token must authenticate, not panic");

    assert_eq!(ctx.role, "analyst");
}

#[test]
fn ascii_token_still_derives_an_eight_byte_grant_prefix() {
    let token = "wp_live_admin_secret_key";
    let engine = engine_with_token(token);

    let ctx = engine
        .authenticate(Some(token), &Policy::default())
        .expect("an ASCII static token must authenticate");

    assert_eq!(
        ctx.grant_id.as_deref(),
        Some("tok_wp_live_"),
        "ASCII tokens keep their existing eight byte grant prefix"
    );
}
