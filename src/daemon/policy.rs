// Rust guideline compliant 2026-08-27

//! Security policy rules governing access, redaction, and Human-in-the-Loop approvals.

use crate::config::PolicyConfig;

/// Active security policy rules governing capability/resource access and approvals.
#[derive(Clone, Default, Debug, PartialEq)]
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

    /// Combines this base policy with an optional profile-specific policy overlay.
    ///
    /// # Inheritance & Precedence Rules:
    /// - **Deny (Union):** Deny rules from base and profile are unioned (strict defense-in-depth).
    /// - **Require Approval (Union):** HITL triggers from base and profile are unioned.
    /// - **Redact Keys (Union):** Sensitive redaction keys are unioned.
    /// - **Allow:** If profile policy defines a non-empty `allow` list, it restricts capabilities to that list; otherwise inherits base `allow`.
    /// - **Approval Timeout:** Overridden by profile if present.
    /// - **Webhook:** Overridden by profile if present.
    pub fn merge_with_profile(&self, profile_policy: Option<&Policy>) -> Policy {
        let Some(prof_pol) = profile_policy else {
            return self.clone();
        };

        let mut deny = self.deny.clone();
        for d in &prof_pol.deny {
            if !deny.contains(d) {
                deny.push(d.clone());
            }
        }

        let mut require_approval = self.require_approval.clone();
        for r in &prof_pol.require_approval {
            if !require_approval.contains(r) {
                require_approval.push(r.clone());
            }
        }

        let mut redact_keys = self.redact_keys.clone();
        for k in &prof_pol.redact_keys {
            if !redact_keys.contains(k) {
                redact_keys.push(k.clone());
            }
        }

        let allow = if !prof_pol.allow.is_empty() {
            prof_pol.allow.clone()
        } else {
            self.allow.clone()
        };

        Policy {
            allow,
            deny,
            redact_keys,
            require_approval,
            approval_timeout_secs: if prof_pol.approval_timeout_secs > 0
                && prof_pol.approval_timeout_secs != 300
            {
                prof_pol.approval_timeout_secs
            } else {
                self.approval_timeout_secs
            },
            webhook: prof_pol.webhook.clone().or_else(|| self.webhook.clone()),
        }
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
