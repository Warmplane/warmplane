// Rust guideline compliant 2026-08-15

//! HTTP v1 facade API handlers for capabilities, resources, prompts, events, operations, and control deck.

pub mod approvals_api;
pub mod audit_api;
pub mod catalog;
pub mod config_api;
pub mod execute;
pub mod helpers;
pub mod types;
pub mod ui;

#[cfg(test)]
mod tests;

// Re-export all handlers and types for seamless backwards compatibility
pub use approvals_api::{
    handle_approve_ticket, handle_get_approval, handle_list_approvals, handle_reject_ticket,
};
pub use audit_api::{
    handle_export_audit, handle_get_audit_event, handle_get_audit_stats, handle_list_audit_events,
    handle_verify_audit_chain,
};
pub use catalog::{
    handle_catalog_events, handle_completion, handle_describe_capability, handle_get_prompt,
    handle_get_sampling_request, handle_list_capabilities, handle_list_prompts,
    handle_list_resources, handle_list_sampling_requests, handle_read_resource,
    handle_resource_updates, handle_respond_sampling_request, handle_sampling_create_message,
    handle_search_capabilities,
};
pub use config_api::{
    handle_delete_server, handle_get_config, handle_get_ecosystem_sources, handle_import_config,
    handle_reload_config, handle_update_alias, handle_update_policy, handle_upsert_server,
};
pub use execute::{
    handle_batch_call_capabilities, handle_call_capability, handle_cancel_operation,
};
pub use helpers::{
    check_if_none_match, make_etag_header, next_trace_id, redact_value, resolve_idempotency_key,
};
pub use types::{
    error_envelope, ApproveTicketRequest, CallCapabilityRequest, CatalogEventsQuery,
    CatalogEventsResponse, CompletionRequest, GetPromptRequest, ImportConfigRequest,
    ReadResourceRequest, RejectTicketRequest, RespondSamplingRequest, SamplingListQuery,
    SamplingRequest, SearchCapabilitiesRequest, UpdateAliasRequest, UpsertServerRequest,
};
pub use ui::handle_ui_dashboard;
