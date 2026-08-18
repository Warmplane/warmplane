// Rust guideline compliant 2026-08-17

//! HTTP v1 handlers for WORM audit log querying, verification, export, and analytics.

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    audit::{AuditEventStatus, AuditEventType, AuditQueryFilter},
    daemon::AppState,
};

/// Parses a string into an `AuditEventType`.
fn parse_event_type(et: &str) -> Option<AuditEventType> {
    match et.trim().to_lowercase().as_str() {
        "tool_execution" => Some(AuditEventType::ToolExecution),
        "tool_intercepted_hitl" => Some(AuditEventType::ToolInterceptedHitl),
        "approval_granted" => Some(AuditEventType::ApprovalGranted),
        "approval_rejected" => Some(AuditEventType::ApprovalRejected),
        "approval_expired" => Some(AuditEventType::ApprovalExpired),
        "policy_violation" => Some(AuditEventType::PolicyViolation),
        "config_mutation" => Some(AuditEventType::ConfigMutation),
        "sampling_call" => Some(AuditEventType::SamplingCall),
        "resource_access" => Some(AuditEventType::ResourceAccess),
        _ => None,
    }
}

/// Parses a string into an `AuditEventStatus`.
fn parse_event_status(st: &str) -> Option<AuditEventStatus> {
    match st.trim().to_lowercase().as_str() {
        "success" => Some(AuditEventStatus::Success),
        "failed" => Some(AuditEventStatus::Failed),
        "denied" => Some(AuditEventStatus::Denied),
        "intercepted" => Some(AuditEventStatus::Intercepted),
        "cancelled" => Some(AuditEventStatus::Cancelled),
        _ => None,
    }
}

/// Query parameters for GET `/v1/audit/events`.
#[derive(Debug, Deserialize, Default)]
pub struct AuditEventsQuery {
    /// Filter events with timestamp >= start_time_ns.
    pub start_time: Option<u64>,
    /// Filter events with timestamp <= end_time_ns.
    pub end_time: Option<u64>,
    /// Filter by actor ID.
    pub actor_id: Option<String>,
    /// Filter by target upstream server ID.
    pub server_id: Option<String>,
    /// Filter by capability ID.
    pub capability_id: Option<String>,
    /// Filter by event type string.
    pub event_type: Option<String>,
    /// Filter by outcome status string.
    pub status: Option<String>,
    /// Filter by trace ID.
    pub trace_id: Option<String>,
    /// Filter by request ID.
    pub request_id: Option<String>,
    /// General search term or keyword query across multiple fields.
    pub search: Option<String>,
    /// Shorthand alias for `search`.
    pub q: Option<String>,
    /// Page size (default 50).
    pub limit: Option<usize>,
    /// Page offset (default 0).
    pub offset: Option<usize>,
}

/// Query parameters for GET `/v1/audit/export`.
#[derive(Debug, Deserialize, Default)]
pub struct AuditExportQuery {
    /// Format: `jsonl` (default) or `csv`.
    pub format: Option<String>,
    /// Filter events with timestamp >= start_time_ns.
    pub start_time: Option<u64>,
    /// Filter events with timestamp <= end_time_ns.
    pub end_time: Option<u64>,
    /// Filter by actor ID.
    pub actor_id: Option<String>,
    /// Filter by server ID.
    pub server_id: Option<String>,
    /// Filter by capability ID.
    pub capability_id: Option<String>,
    /// Filter by event type string.
    pub event_type: Option<String>,
    /// Filter by outcome status string.
    pub status: Option<String>,
    /// Filter by trace ID.
    pub trace_id: Option<String>,
    /// Filter by request ID.
    pub request_id: Option<String>,
    /// General search term or keyword query across multiple fields.
    pub search: Option<String>,
    /// Shorthand alias for `search`.
    pub q: Option<String>,
}

/// Handles GET `/v1/audit/events` returning paginated audit records matching filters.
pub async fn handle_list_audit_events(
    State(state): State<AppState>,
    Query(query): Query<AuditEventsQuery>,
) -> impl IntoResponse {
    let parsed_event_type = query.event_type.as_deref().and_then(parse_event_type);
    let parsed_status = query.status.as_deref().and_then(parse_event_status);
    let search_query = query.search.or(query.q);

    let filter = AuditQueryFilter {
        start_time_ns: query.start_time,
        end_time_ns: query.end_time,
        actor_id: query.actor_id,
        server_id: query.server_id,
        capability_id: query.capability_id,
        event_type: parsed_event_type,
        status: parsed_status,
        trace_id: query.trace_id,
        request_id: query.request_id,
        search: search_query,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    };

    let (events, total) = state.audit_store.query(&filter).await;

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "events": events,
            "total": total,
            "limit": filter.limit,
            "offset": filter.offset,
        })),
    )
        .into_response()
}

/// Handles GET `/v1/audit/events/:id` returning a single audit event record.
pub async fn handle_get_audit_event(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.audit_store.get_by_id(&id).await {
        Some(event) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "event": event,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": format!("Audit event '{}' not found", id),
            })),
        )
            .into_response(),
    }
}

/// Handles GET `/v1/audit/verify` performing a cryptographic hash chain verification check.
pub async fn handle_verify_audit_chain(State(state): State<AppState>) -> impl IntoResponse {
    let report = state.audit_store.verify_chain().await;
    (
        StatusCode::OK,
        Json(json!({
            "ok": report.is_valid,
            "report": report,
        })),
    )
        .into_response()
}

/// Handles GET `/v1/audit/stats` aggregating audit trail metrics and breakdown.
pub async fn handle_get_audit_stats(State(state): State<AppState>) -> impl IntoResponse {
    let (all_events, total) = state
        .audit_store
        .query(&AuditQueryFilter {
            limit: 10_000,
            ..Default::default()
        })
        .await;

    let mut success_count = 0usize;
    let mut failed_count = 0usize;
    let mut denied_count = 0usize;
    let mut hitl_count = 0usize;

    for e in &all_events {
        match e.status {
            AuditEventStatus::Success => success_count += 1,
            AuditEventStatus::Failed => failed_count += 1,
            AuditEventStatus::Denied => denied_count += 1,
            AuditEventStatus::Intercepted => hitl_count += 1,
            AuditEventStatus::Cancelled => {}
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "total_events": total,
            "by_status": {
                "success": success_count,
                "failed": failed_count,
                "denied": denied_count,
                "intercepted": hitl_count,
            }
        })),
    )
        .into_response()
}

/// Handles GET `/v1/audit/export` streaming audit logs as JSONL or CSV file download.
pub async fn handle_export_audit(
    State(state): State<AppState>,
    Query(query): Query<AuditExportQuery>,
) -> impl IntoResponse {
    let parsed_event_type = query.event_type.as_deref().and_then(parse_event_type);
    let parsed_status = query.status.as_deref().and_then(parse_event_status);
    let search_query = query.search.or(query.q);

    let filter = AuditQueryFilter {
        start_time_ns: query.start_time,
        end_time_ns: query.end_time,
        actor_id: query.actor_id,
        server_id: query.server_id,
        capability_id: query.capability_id,
        event_type: parsed_event_type,
        status: parsed_status,
        trace_id: query.trace_id,
        request_id: query.request_id,
        search: search_query,
        limit: crate::audit::MAX_IN_MEMORY_AUDIT_EVENTS,
        offset: 0,
    };

    let (events, _) = state.audit_store.query(&filter).await;
    let format = query.format.unwrap_or_else(|| "jsonl".to_string());

    let mut headers = HeaderMap::new();
    if format.to_lowercase() == "csv" {
        headers.insert(
            header::CONTENT_TYPE,
            "text/csv; charset=utf-8".parse().unwrap(),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"warmplane_audit.csv\""
                .parse()
                .unwrap(),
        );

        let mut csv = String::new();
        csv.push_str("id,timestamp_ns,event_type,trace_id,request_id,actor_id,server_id,capability_id,status,latency_us,hash,prev_hash\n");
        for e in events {
            csv.push_str(&format!(
                "{},{},{:?},{},{},{},{},{},{:?},{},{},{}\n",
                sanitize_csv_cell(&e.id),
                e.timestamp_ns,
                e.event_type,
                sanitize_csv_cell(&e.trace_id),
                sanitize_csv_cell(e.request_id.as_deref().unwrap_or("")),
                sanitize_csv_cell(e.actor_id.as_deref().unwrap_or("")),
                sanitize_csv_cell(e.server_id.as_deref().unwrap_or("")),
                sanitize_csv_cell(e.capability_id.as_deref().unwrap_or("")),
                e.status,
                e.execution_latency_us.unwrap_or(0),
                sanitize_csv_cell(&e.hash),
                sanitize_csv_cell(&e.prev_hash),
            ));
        }
        (StatusCode::OK, headers, csv).into_response()
    } else {
        headers.insert(
            header::CONTENT_TYPE,
            "application/x-ndjson; charset=utf-8".parse().unwrap(),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"warmplane_audit.jsonl\""
                .parse()
                .unwrap(),
        );

        let mut jsonl = String::new();
        for e in events {
            if let Ok(line) = serde_json::to_string(&e) {
                jsonl.push_str(&line);
                jsonl.push('\n');
            }
        }
        (StatusCode::OK, headers, jsonl).into_response()
    }
}

fn sanitize_csv_cell(cell: &str) -> String {
    let trimmed = cell.trim();
    let escaped = if trimmed.starts_with('=')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('@')
        || trimmed.starts_with('\t')
        || trimmed.starts_with('\r')
    {
        format!("'{}", trimmed)
    } else {
        trimmed.to_string()
    };
    if escaped.contains(',') || escaped.contains('"') || escaped.contains('\n') {
        format!("\"{}\"", escaped.replace('"', "\"\""))
    } else {
        escaped
    }
}
