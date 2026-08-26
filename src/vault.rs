// Rust guideline compliant 2026-08-26

//! Dynamic Secret Vault & OS Keychain Resolver Engine.
//!
//! Provides zero-disk plaintext secret resolution across:
//! - `keychain://[service/]<account>`: Native macOS Keychain (`security`) & Linux Secret Service / Keyutils.
//! - `op://[vault/]<item>/<field>`: 1Password CLI integration (`op read`).
//! - `env://<VAR_NAME>`: Environment variable resolution.
//! - `cmd://<command...>`: Dynamic subprocess execution and stdout capture.
//!
//! Values are resolved exclusively in-memory when spawning child processes or initiating
//! HTTP requests, ensuring credentials are never saved in plaintext configuration files.

use anyhow::{bail, Context, Result};
use std::process::Command;
use tracing::debug;

/// Default service name used for Warmplane secrets in the OS keychain.
pub const DEFAULT_KEYCHAIN_SERVICE: &str = "warmplane";

/// Resolves a potentially secret URI into its plaintext value in memory.
///
/// If the input string does not match any recognized secret URI prefix (`keychain://`,
/// `op://`, `env://`, `cmd://`), the input string is returned as-is.
///
/// # Arguments
/// * `raw_value` - Input string or URI to resolve.
///
/// # Returns
/// The resolved secret string.
///
/// # Errors
/// Returns an error if resolution fails for a declared secret URI scheme.
pub fn resolve_secret_value(raw_value: &str) -> Result<String> {
    let trimmed = raw_value.trim();

    if let Some(uri) = trimmed.strip_prefix("keychain://") {
        resolve_keychain(uri)
    } else if let Some(uri) = trimmed.strip_prefix("op://") {
        resolve_1password(&format!("op://{}", uri))
    } else if let Some(var_name) = trimmed.strip_prefix("env://") {
        resolve_env(var_name)
    } else if let Some(cmd) = trimmed.strip_prefix("cmd://") {
        resolve_cmd(cmd)
    } else {
        Ok(raw_value.to_string())
    }
}

/// Resolves a secret from the OS Keychain.
/// Format: `keychain://<service>/<account>` or `keychain://<account>` (uses DEFAULT_KEYCHAIN_SERVICE).
fn resolve_keychain(uri: &str) -> Result<String> {
    let parts: Vec<&str> = uri.splitn(2, '/').collect();
    let (service, account) = if parts.len() == 2 && !parts[0].is_empty() {
        (parts[0], parts[1])
    } else {
        (DEFAULT_KEYCHAIN_SERVICE, uri)
    };

    get_os_keychain_secret(service, account)
}

/// Resolves a secret via the 1Password CLI (`op read <uri>`).
fn resolve_1password(op_uri: &str) -> Result<String> {
    debug!(uri = %op_uri, "resolving secret via 1Password CLI");
    let output = Command::new("op")
        .args(["read", op_uri])
        .output()
        .with_context(|| format!("Failed to execute 'op read {}'", op_uri))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("1Password CLI error: {}", stderr.trim());
    }

    let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(secret)
}

/// Resolves a secret from an environment variable.
fn resolve_env(var_name: &str) -> Result<String> {
    let trimmed_var = var_name.trim();
    std::env::var(trimmed_var)
        .with_context(|| format!("Environment variable '{}' is not set", trimmed_var))
}

/// Resolves a secret by running a shell command and capturing its stdout.
fn resolve_cmd(cmd_str: &str) -> Result<String> {
    debug!(cmd = %cmd_str, "resolving secret via command execution");

    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", cmd_str])
        .output()
        .with_context(|| format!("Failed to execute secret command: {}", cmd_str))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", cmd_str])
        .output()
        .with_context(|| format!("Failed to execute secret command: {}", cmd_str))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Secret command '{}' failed: {}", cmd_str, stderr.trim());
    }

    let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(secret)
}

// -----------------------------------------------------------------------------
// OS-Specific Keychain Implementations
// -----------------------------------------------------------------------------

/// Reads a generic password from the OS Keychain.
pub fn get_os_keychain_secret(service: &str, account: &str) -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("security")
            .args(["find-generic-password", "-s", service, "-a", account, "-w"])
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute security CLI for service='{}', account='{}'",
                    service, account
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "macOS Keychain: Secret not found for service='{}', account='{}' ({})",
                service,
                account,
                stderr.trim()
            );
        }

        let secret = String::from_utf8_lossy(&output.stdout)
            .trim_end_matches(['\r', '\n'])
            .to_string();
        Ok(secret)
    }

    #[cfg(target_os = "linux")]
    {
        // Try secret-tool (libsecret) if available
        let output = Command::new("secret-tool")
            .args(["lookup", "service", service, "account", account])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let secret = String::from_utf8_lossy(&out.stdout)
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                return Ok(secret);
            }
        }

        bail!(
            "Linux Secret Service: Secret not found or secret-tool unavailable for service='{}', account='{}'",
            service,
            account
        );
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use cmdkey / powershell credential retrieval fallback
        let ps_cmd = format!(
            "[Windows.Security.Credentials.PasswordVault,Windows.Security.Credentials,ContentType=WindowsRuntime] | Out-Null; (New-Object Windows.Security.Credentials.PasswordVault).Retrieve('{}', '{}').Password",
            service, account
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .with_context(|| {
                format!(
                    "Failed to query Windows PasswordVault for service='{}', account='{}'",
                    service, account
                )
            })?;

        if !output.status.success() {
            bail!(
                "Windows Credential Vault: Secret not found for service='{}', account='{}'",
                service,
                account
            );
        }

        let secret = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(secret)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        bail!("OS Keychain is not supported on this platform");
    }
}

/// Saves a generic password into the OS Keychain.
pub fn set_os_keychain_secret(service: &str, account: &str, secret: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("security")
            .args([
                "add-generic-password",
                "-U", // Update if exists
                "-s",
                service,
                "-a",
                account,
                "-w",
                secret,
            ])
            .output()
            .with_context(|| {
                format!(
                    "Failed to store secret in macOS Keychain for service='{}', account='{}'",
                    service, account
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("macOS Keychain write error: {}", stderr.trim());
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        let mut child = Command::new("secret-tool")
            .args([
                "store",
                "--label",
                &format!("{} - {}", service, account),
                "service",
                service,
                "account",
                account,
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn secret-tool on Linux")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(secret.as_bytes())?;
        }

        let status = child.wait()?;
        if !status.success() {
            bail!("Linux secret-tool failed to save secret");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = format!(
            "[Windows.Security.Credentials.PasswordVault,Windows.Security.Credentials,ContentType=WindowsRuntime] | Out-Null; (New-Object Windows.Security.Credentials.PasswordVault).Add((New-Object Windows.Security.Credentials.PasswordCredential('{}', '{}', '{}')))",
            service, account, secret
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .context("Failed to write to Windows PasswordVault")?;

        if !output.status.success() {
            bail!("Windows PasswordVault write failed");
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        bail!("OS Keychain is not supported on this platform");
    }
}

/// Deletes a generic password from the OS Keychain.
pub fn delete_os_keychain_secret(service: &str, account: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("security")
            .args(["delete-generic-password", "-s", service, "-a", account])
            .output()
            .with_context(|| {
                format!(
                    "Failed to delete secret from macOS Keychain for service='{}', account='{}'",
                    service, account
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("macOS Keychain delete error: {}", stderr.trim());
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("secret-tool")
            .args(["clear", "service", service, "account", account])
            .output()
            .context("Failed to execute secret-tool clear")?;

        if !output.status.success() {
            bail!("Linux secret-tool failed to clear secret");
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = format!(
            "[Windows.Security.Credentials.PasswordVault,Windows.Security.Credentials,ContentType=WindowsRuntime] | Out-Null; $v = New-Object Windows.Security.Credentials.PasswordVault; $c = $v.Retrieve('{}', '{}'); if ($c) {{ $v.Remove($c) }}",
            service, account
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .context("Failed to delete from Windows PasswordVault")?;

        if !output.status.success() {
            bail!("Windows PasswordVault delete failed");
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        bail!("OS Keychain is not supported on this platform");
    }
}

/// Formats a value for display in the UI / logs, replacing sensitive content with masked badges.
pub fn redact_secret_for_display(val: &str) -> String {
    let trimmed = val.trim();
    if trimmed.starts_with("keychain://")
        || trimmed.starts_with("op://")
        || trimmed.starts_with("env://")
    {
        format!("🔒 [{}]", trimmed)
    } else if trimmed.starts_with("cmd://") {
        "🔒 [cmd://...]".to_string()
    } else if trimmed.len() > 8 {
        format!("{}...{}", &trimmed[..3], &trimmed[trimmed.len() - 3..])
    } else {
        "********".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_env_scheme() {
        std::env::set_var("TEST_WP_SECRET_ENV", "my_secret_token_123");
        let res = resolve_secret_value("env://TEST_WP_SECRET_ENV").unwrap();
        assert_eq!(res, "my_secret_token_123");

        let err = resolve_secret_value("env://NON_EXISTENT_VAR_XYZ_99");
        assert!(err.is_err());
    }

    #[test]
    fn test_resolve_cmd_scheme() {
        let res = resolve_secret_value("cmd://echo wp_vault_pass").unwrap();
        assert_eq!(res, "wp_vault_pass");
    }

    #[test]
    fn test_passthrough_non_uri() {
        let raw = "plain_static_password";
        let res = resolve_secret_value(raw).unwrap();
        assert_eq!(res, raw);
    }

    #[test]
    fn test_redact_display() {
        assert_eq!(
            redact_secret_for_display("keychain://warmplane/token"),
            "🔒 [keychain://warmplane/token]"
        );
        assert_eq!(
            redact_secret_for_display("op://vault/item/password"),
            "🔒 [op://vault/item/password]"
        );
        assert_eq!(
            redact_secret_for_display("env://API_KEY"),
            "🔒 [env://API_KEY]"
        );
    }
}
