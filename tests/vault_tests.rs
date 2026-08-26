// Rust guideline compliant 2026-08-26

//! Integration tests for dynamic secret URI resolution, OS Keychain, 1Password CLI, and child process injection.

use warmplane::vault::{redact_secret_for_display, resolve_secret_value};

#[test]
fn test_env_secret_resolution() {
    std::env::set_var("WARMPLANE_TEST_API_KEY", "wp_secret_key_abcdef12345");
    let resolved = resolve_secret_value("env://WARMPLANE_TEST_API_KEY").unwrap();
    assert_eq!(resolved, "wp_secret_key_abcdef12345");

    let err = resolve_secret_value("env://UNSET_TEST_VAR_XYZ");
    assert!(err.is_err());
}

#[test]
fn test_cmd_secret_resolution() {
    let resolved = resolve_secret_value("cmd://echo token_from_subshell_99").unwrap();
    assert_eq!(resolved, "token_from_subshell_99");
}

#[test]
fn test_raw_passthrough() {
    let raw = "standard_plaintext_config_val";
    let resolved = resolve_secret_value(raw).unwrap();
    assert_eq!(resolved, raw);
}

#[test]
fn test_redaction_formatting() {
    assert_eq!(
        redact_secret_for_display("keychain://warmplane/db_pass"),
        "🔒 [keychain://warmplane/db_pass]"
    );
    assert_eq!(
        redact_secret_for_display("op://Dev/Stripe/secret_key"),
        "🔒 [op://Dev/Stripe/secret_key]"
    );
    assert_eq!(
        redact_secret_for_display("env://GITHUB_TOKEN"),
        "🔒 [env://GITHUB_TOKEN]"
    );
    assert_eq!(
        redact_secret_for_display("cmd://vault kv get -field=val secret"),
        "🔒 [cmd://...]"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_keychain_crud_lifecycle() {
    use warmplane::vault::{
        delete_os_keychain_secret, get_os_keychain_secret, set_os_keychain_secret,
    };

    let service = "warmplane_test_ci";
    let account = "test_user_account";
    let secret = "mock_super_secret_password_42";

    // Set
    set_os_keychain_secret(service, account, secret).unwrap();

    // Get
    let retrieved = get_os_keychain_secret(service, account).unwrap();
    assert_eq!(retrieved, secret);

    // Resolve URI
    let uri = format!("keychain://{}/{}", service, account);
    let resolved = resolve_secret_value(&uri).unwrap();
    assert_eq!(resolved, secret);

    // Delete
    delete_os_keychain_secret(service, account).unwrap();

    // Verify deletion
    let after_delete = get_os_keychain_secret(service, account);
    assert!(after_delete.is_err());
}
