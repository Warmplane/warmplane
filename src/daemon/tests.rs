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

    #[tokio::test]
    async fn test_initialize_state_degraded_boot() {
        use crate::config::{McpConfig, ServerConfig};
        use std::collections::HashMap;

        let mut mcp_servers = HashMap::new();
        // Server with an invalid / non-existent binary to simulate offline/broken startup
        mcp_servers.insert(
            "broken_server".to_string(),
            ServerConfig {
                command: Some("non_existent_binary_xyz_1234".to_string()),
                args: vec![],
                env: HashMap::new(),
                url: None,
                auth: None,
                protocol_version: None,
                allow_stateless: Some(false),
                headers: HashMap::new(),
                resilience: None,
            },
        );

        let config = McpConfig {
            mcp_servers,
            ..Default::default()
        };

        let state = crate::daemon::server::initialize_state(config, "test_config.json")
            .await
            .expect(
                "initialize_state must succeed in degraded mode even if an upstream server fails",
            );

        let statuses = state.server_statuses.read().await;
        let broken_status = statuses
            .get("broken_server")
            .expect("broken_server status must be recorded");
        assert_eq!(broken_status["status"], "degraded");
        assert!(broken_status["error"].is_string());
    }
}
