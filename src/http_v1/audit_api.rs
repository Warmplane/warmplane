// Rust guideline compliant 2026-11-18
//! HTTP v1 handlers for WORM audit log querying, verification, export, and analytics.

use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    audit::{AuditEventStatus, AuditEventType, AuditQueryFilter},
    context::ProfileContext,
    daemon::AppState,
    idempotency::PersistedIdempotencyRecord,
    rbac::TenantContext,
    result_sanitization::{ResultAccess, Withheld},
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
        "result_sanitization" => Some(AuditEventType::ResultSanitization),
        "emission_held" => Some(AuditEventType::EmissionHeld),
        "emission_cancelled" => Some(AuditEventType::EmissionCancelled),
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
    /// Filter by idempotency key.
    pub idempotency_key: Option<String>,
    /// Filter by whether the event was a deduplicated replay.
    pub is_replay: Option<bool>,
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
    /// Filter by idempotency key.
    pub idempotency_key: Option<String>,
    /// Filter by whether the event was a deduplicated replay.
    pub is_replay: Option<bool>,
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
    let filter = query.into_filter();

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

/// Query parameters for GET `/v1/idempotency/records`.
#[derive(Debug, Deserialize, Default)]
pub struct IdempotencyListQuery {
    /// Page size (default 50).
    pub limit: Option<usize>,
    /// Page offset (default 0).
    pub offset: Option<usize>,
}

/// Handles GET `/v1/idempotency/records` listing cached idempotency records.
pub async fn handle_list_idempotency_records(
    State(state): State<AppState>,
    Extension(tenant): Extension<Option<TenantContext>>,
    profile: Option<Extension<ProfileContext>>,
    Query(query): Query<IdempotencyListQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let mut records = Vec::new();
    for record in state.idempotency_store.list_records(limit, offset).await {
        if let Some(record) =
            inspect_idempotency_record(&state, record, tenant.as_ref(), profile.as_deref()).await
        {
            records.push(record);
        }
    }
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "records": records,
            "limit": limit,
            "offset": offset,
        })),
    )
        .into_response()
}

/// Handles GET `/v1/idempotency/records/:key` fetching a single cached idempotency record.
pub async fn handle_get_idempotency_record(
    State(state): State<AppState>,
    Extension(tenant): Extension<Option<TenantContext>>,
    profile: Option<Extension<ProfileContext>>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let record = match state.idempotency_store.get_record(&key).await {
        Some(record) => {
            inspect_idempotency_record(&state, record, tenant.as_ref(), profile.as_deref()).await
        }
        None => None,
    };
    match record {
        Some(record) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "record": record,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "ok": false,
                "error": format!("Idempotency record '{}' not found or expired", key),
            })),
        )
            .into_response(),
    }
}

async fn inspect_idempotency_record(
    state: &AppState,
    mut record: PersistedIdempotencyRecord,
    tenant: Option<&TenantContext>,
    profile: Option<&ProfileContext>,
) -> Option<PersistedIdempotencyRecord> {
    if !state.trust_controls_active().await {
        return Some(record);
    }
    let profile_id = profile.and_then(|profile| profile.profile_id.as_deref());
    let access = ResultAccess::new(
        tenant.map_or("default", |context| context.tenant_id.as_str()),
        profile_id,
    );
    let default_role = state.rbac_engine.default_role();
    let role = tenant.map_or(default_role.as_str(), |context| context.role.as_str());
    let capability = record.capability_id.as_deref()?;
    let local_legacy_record =
        !state.rbac_engine.is_enabled() && profile_id.is_none() && record.result_access.is_none();
    if (!local_legacy_record && record.result_access.as_ref() != Some(&access))
        || !access.permits_capability(state, capability, role).await
    {
        return None;
    }
    if !state
        .accepts_result_revision(record.result_policy_revision.as_deref())
        .await
    {
        record.response = json!({"ok": false, "error": crate::WarmplaneError::from(Withheld)});
    }
    Some(record)
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
    let format = query.format.clone().unwrap_or_else(|| "jsonl".to_string());
    let filter = AuditEventsQuery {
        limit: Some(crate::audit::MAX_IN_MEMORY_AUDIT_EVENTS),
        offset: Some(0),
        start_time: query.start_time,
        end_time: query.end_time,
        actor_id: query.actor_id,
        server_id: query.server_id,
        capability_id: query.capability_id,
        event_type: query.event_type,
        status: query.status,
        trace_id: query.trace_id,
        request_id: query.request_id,
        idempotency_key: query.idempotency_key,
        is_replay: query.is_replay,
        search: query.search,
        q: query.q,
    }
    .into_filter();

    let (events, _) = state.audit_store.query(&filter).await;

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

impl AuditEventsQuery {
    fn into_filter(self) -> AuditQueryFilter {
        AuditQueryFilter {
            start_time_ns: self.start_time,
            end_time_ns: self.end_time,
            actor_id: self.actor_id,
            server_id: self.server_id,
            capability_id: self.capability_id,
            event_type: self.event_type.as_deref().and_then(parse_event_type),
            status: self.status.as_deref().and_then(parse_event_status),
            trace_id: self.trace_id,
            request_id: self.request_id,
            idempotency_key: self.idempotency_key,
            is_replay: self.is_replay,
            search: self.search.or(self.q),
            limit: self.limit.unwrap_or(50),
            offset: self.offset.unwrap_or(0),
        }
    }
}
