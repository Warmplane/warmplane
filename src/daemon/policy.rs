// Rust guideline compliant 2026-08-15

//! Security policy rules governing access, redaction, and Human-in-the-Loop approvals.

use crate::config::PolicyConfig;

/// Active security policy rules governing capability/resource access and approvals.
#[derive(Clone, Default, Debug)]
pub struct Policy {
    /// List of wildcard patterns allowed for execution.
    pub allow: Vec<String>,
    /// List of wildcard patterns explicitly denied.
    pub deny: Vec<String>,
    /// Keys to redact in logged payload envelopes.
    pub redact_keys: Vec<String>,
    /// List of wildcard patterns requiring human operator approval.
    pub require_approval: Vec<String>,
    /// Timeout in seconds before a pending approval expires.
    pub approval_timeout_secs: u64,
    /// Outbound webhook configuration.
    pub webhook: Option<crate::config::WebhookConfig>,
}

impl Policy {
    /// Constructs a `Policy` from optional `PolicyConfig`.
    ///
    /// # Arguments
    /// * `config` - Optional configuration struct.
    ///
    /// # Returns
    /// Constructed `Policy`.
    pub fn from_config(config: Option<PolicyConfig>) -> Self {
        let Some(config) = config else {
            return Self::default();
        };
        Self {
            allow: config.allow,
            deny: config.deny,
            redact_keys: config.redact_keys,
            require_approval: config.require_approval,
            approval_timeout_secs: config.approval_timeout_secs.unwrap_or(300),
            webhook: config.webhook,
        }
    }

    /// Checks whether the given capability or resource ID is permitted under security policy.
    ///
    /// # Arguments
    /// * `id` - Identifier string to test.
    ///
    /// # Returns
    /// `true` if allowed, `false` if denied.
    pub fn allows(&self, id: impl AsRef<str>) -> bool {
        let id_ref = id.as_ref();
        if self
            .deny
            .iter()
            .any(|pattern| wildcard_match(pattern, id_ref))
        {
            return false;
        }

        if self.allow.is_empty() {
            return true;
        }

        self.allow
            .iter()
            .any(|pattern| wildcard_match(pattern, id_ref))
    }

    /// Checks whether the given capability requires human approval prior to execution.
    ///
    /// # Arguments
    /// * `id` - Identifier string to test.
    ///
    /// # Returns
    /// `true` if operator approval is required, `false` otherwise.
    pub fn requires_approval(&self, id: impl AsRef<str>) -> bool {
        let id_ref = id.as_ref();
        self.require_approval
            .iter()
            .any(|pattern| wildcard_match(pattern, id_ref))
    }
}

/// Matches a string against a wildcard expression where `*` denotes wildcards.
///
/// Supports prefix (`db.*`), suffix (`*.delete`), infix (`db.*.write`), and substring (`*query*`) patterns.
///
/// # Arguments
/// * `pattern` - Wildcard pattern string.
/// * `value` - Candidate value to match.
///
/// # Returns
/// `true` if candidate matches pattern, `false` otherwise.
pub fn wildcard_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == value;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let mut remaining = value;

    // First part must match the start if pattern does not start with '*'
    if !pattern.starts_with('*') {
        let first = parts[0];
        if !remaining.starts_with(first) {
            return false;
        }
        remaining = &remaining[first.len()..];
    }

    // Last part must match the end if pattern does not end with '*'
    if !pattern.ends_with('*') {
        let last = parts[parts.len() - 1];
        if !remaining.ends_with(last) {
            return false;
        }
        remaining = &remaining[..remaining.len() - last.len()];
    }

    // Middle parts must appear sequentially in remaining
    if parts.len() > 2 {
        for part in &parts[1..parts.len() - 1] {
            if part.is_empty() {
                continue;
            }
            if let Some(pos) = remaining.find(part) {
                remaining = &remaining[pos + part.len()..];
            } else {
                return false;
            }
        }
    }

    true
}
