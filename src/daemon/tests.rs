// Rust guideline compliant 2026-08-15

//! Unit tests for policy rules and pattern matching in daemon module.

#[cfg(test)]
mod tests {
    use crate::daemon::policy::{wildcard_match, Policy};

    #[test]
    fn wildcard_prefix_match_works() {
        assert!(wildcard_match("db.*", "db.query"));
        assert!(!wildcard_match("db.*", "fs.read"));
    }

    #[test]
    fn policy_deny_overrides_allow() {
        let policy = Policy {
            allow: vec!["db.*".to_string()],
            deny: vec!["db.delete".to_string()],
            ..Default::default()
        };

        assert!(policy.allows("db.query"));
        assert!(!policy.allows("db.delete"));
    }

    #[test]
    fn policy_allow_list_is_enforced() {
        let policy = Policy {
            allow: vec!["fs.read".to_string()],
            deny: vec![],
            ..Default::default()
        };

        assert!(policy.allows("fs.read"));
        assert!(!policy.allows("fs.write"));
    }
}
