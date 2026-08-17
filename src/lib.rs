// Rust guideline compliant 2026-08-14

//! Warmplane: Local control plane and facade proxy for Model Context Protocol (MCP) sessions.

pub mod approvals;
pub mod audit;
pub mod batch_executor;
pub mod catalog;
pub mod cli_config;
pub mod config;
pub mod config_import;
pub mod context;
pub mod daemon;
pub mod http_v1;
pub mod idempotency;
pub mod interactive;
pub mod mcp_server;
pub mod models;
pub mod oauth2;
pub mod operations;
pub mod search;
pub mod telemetry;
