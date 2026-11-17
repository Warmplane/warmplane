// Rust guideline compliant 2026-11-17
//! HTTP v1 endpoints for tenant-scoped undo inspection and execution.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::{collections::HashMap, sync::Arc};

use crate::{
    context::{resolve_request_context, RequestContext},
    daemon::AppState,
    undo::{RollbackResult, UndoAccessContext, UndoEntryStatus, UndoListFilter},
};

/// Query parameters for listing undo entries.
#[derive(Clone, Debug, Deserialize)]
pub struct UndoListQuery {
    /// Optional work-item filter.
    pub work_item_id: Option<String>,
    /// Optional lifecycle state (`armed`, `executing`, `reverted`, `expired`, or `failed`).
    pub status: Option<String>,
    /// Result offset, defaulting to zero.
    pub offset: Option<usize>,
    /// Page size, defaulting to 50 and capped at 200.
    pub limit: Option<usize>,
}

/// Request body for executing one undo or rolling back a work item.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UndoExecutionRequest {
    /// Explicitly overrides an expired undo window; requires elevated permission.
    #[serde(default, rename = "forceExpired")]
    pub force_expired: bool,
    /// Required explanation when overriding expiry.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Request for an isolated round-trip test of one configured inverse contract.
#[derive(Clone, Debug, Deserialize)]
pub struct UndoContractTestRequest {
    /// Capability alias with a configured `testTarget`.
    pub alias: String,
}

/// Safe result of an inverse contract test, excluding tool outputs and state values.
#[derive(Clone, Debug, Serialize)]
pub struct UndoContractTestResponse {
    /// Whether the entire forward and inverse cycle succeeded.
    pub ok: bool,
    /// Alias whose contract was tested.
    pub alias: String,
    /// Dedicated upstream server used for the test cycle.
    pub test_server: String,
    /// Whether the observed state matched the pre-execution snapshot.
    pub observationally_equivalent: bool,
    /// Machine-readable failure code, when the cycle did not pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// Handles `POST /v1/undo/test` using only an explicitly configured test target.
///
/// The handler maps the forward, snapshot, and inverse tools to the configured test server in an
/// isolated temporary state directory. It returns only cycle status, never state or tool output.
///
/// # Arguments
/// * `state` - Active daemon state and connected test server.
/// * `req_ext` - Authenticated tenant and role, when RBAC is enabled.
/// * `prof_ext` - Active profile boundary.
/// * `request` - Alias identifier to test.
///
/// # Returns
/// A safe test result, or an HTTP error when authorization, configuration, or storage checks fail.
pub async fn handle_test_undo_contract(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Json(request): Json<UndoContractTestRequest>,
) -> impl IntoResponse {
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), None).await;
    if state.rbac_engine.is_enabled() && !state.rbac_engine.undo_permissions(&access.role).execute {
        return error_response(
            StatusCode::FORBIDDEN,
            "POLICY_DENIED",
            "Caller role is not authorized to test undo contracts",
        );
    }
    let config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(config) => config,
        Err(_) => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "CONFIG_UNAVAILABLE",
                "Configuration is unavailable",
            )
        }
    };
    let Some(alias_target) = config.capability_aliases.get(&request.alias) else {
        return error_response(
            StatusCode::NOT_FOUND,
            "UNDO_TEST_ALIAS_NOT_FOUND",
            "Capability alias was not found",
        );
    };
    let Some(contract) = alias_target.reversibility().cloned() else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "UNDO_TEST_CONTRACT_MISSING",
            "Alias has no reversibility contract",
        );
    };
    let Some(test_target) = contract.test_target.as_deref() else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "UNDO_TEST_TARGET_REQUIRED",
            "Configure reversibility.testTarget before testing this alias",
        );
    };
    let Some(pre_snapshot) = contract.pre_snapshot.as_deref() else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "UNDO_TEST_SNAPSHOT_REQUIRED",
            "A preSnapshot is required to verify observational equivalence",
        );
    };
    let Some(inverse) = contract.inverse.as_deref() else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "UNDO_TEST_INVERSE_REQUIRED",
            "The reversibility contract has no inverse",
        );
    };
    let Some(test_sender) = state.servers.read().await.get(&test_target.server).cloned() else {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "UNDO_TEST_SERVER_UNAVAILABLE",
            "Configured test server is not connected",
        );
    };
    if !access.profile.is_server_allowed(&test_target.server) {
        return error_response(
            StatusCode::FORBIDDEN,
            "PROFILE_DENIED",
            "The configured test server is outside the active profile",
        );
    }
    let forward_id = alias_target.target().to_string();
    let mut capability_map = state.capabilities.read().await.clone();
    let mut remap_ids = vec![
        forward_id.clone(),
        pre_snapshot.capability.clone(),
        inverse.capability.clone(),
    ];
    if let Some(check) = contract.pre_undo_check.as_deref() {
        remap_ids.push(check.capability.clone());
    }
    remap_ids.sort();
    remap_ids.dedup();
    for capability_id in remap_ids {
        let Some(meta) = capability_map.get_mut(&capability_id) else {
            return error_response(
                StatusCode::NOT_FOUND,
                "UNDO_TEST_CAPABILITY_MISSING",
                "A contract capability is not in the active catalog",
            );
        };
        meta.server = test_target.server.clone();
        meta.reversibility = None;
        if capability_id == forward_id {
            meta.reversibility = Some(contract.clone());
        }
    }

    let temporary_state = match TemporaryUndoTestDir::create() {
        Ok(directory) => directory,
        Err(_) => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "UNDO_TEST_STORAGE_UNAVAILABLE",
                "Temporary test storage is unavailable",
            )
        }
    };
    let snapshot_store =
        match crate::undo::SnapshotStore::open(temporary_state.path().join("artifacts")) {
            Ok(store) => store,
            Err(_) => {
                return error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "UNDO_TEST_STORAGE_UNAVAILABLE",
                    "Temporary test storage is unavailable",
                )
            }
        };
    let ledger = match crate::undo::UndoLedgerStore::open(temporary_state.path().join("undo.json"))
    {
        Ok(ledger) => ledger,
        Err(_) => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "UNDO_TEST_STORAGE_UNAVAILABLE",
                "Temporary test storage is unavailable",
            )
        }
    };
    let isolated_config_path = temporary_state.path().join("config.json");
    if crate::config::save_config(
        isolated_config_path.to_string_lossy().as_ref(),
        &crate::config::McpConfig::default(),
    )
    .is_err()
    {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "UNDO_TEST_STORAGE_UNAVAILABLE",
            "Temporary test configuration is unavailable",
        );
    }
    let mut test_state = state.clone();
    let test_policy = state
        .rbac_engine
        .compute_effective_policy(&access.role, &access.policy)
        .merge_with_profile(access.profile.profile_policy.as_ref());
    if !test_policy.allows(&forward_id)
        || !test_policy.allows(&pre_snapshot.capability)
        || !test_policy.allows(&inverse.capability)
    {
        return error_response(
            StatusCode::FORBIDDEN,
            "POLICY_DENIED",
            "The test contract is denied by the current effective policy",
        );
    }
    test_state.servers = Arc::new(tokio::sync::RwLock::new(HashMap::from([(
        test_target.server.clone(),
        test_sender,
    )])));
    test_state.capabilities = Arc::new(tokio::sync::RwLock::new(capability_map));
    test_state.policy = Arc::new(tokio::sync::RwLock::new(test_policy.clone()));
    test_state.snapshot_store = Some(snapshot_store.clone());
    test_state.undo_ledger = Some(ledger.clone());
    test_state.config_path = isolated_config_path.to_string_lossy().to_string();
    test_state.task_registry = crate::tasks::TaskRegistry::default();
    test_state.hold_coordinator = crate::emissions::HoldCoordinator::default();
    let test_handle = crate::engine::ControlPlaneHandle::new(test_state.clone());
    let options = crate::engine::ExecutionOptions {
        profile: access.profile.profile_id.clone(),
        context: Some(crate::context::RequestContext {
            operation_id: Some(crate::http_v1::next_trace_id()),
            work_item_id: None,
            actor_id: access.actor_id.clone(),
            grant_id: None,
        }),
        ..Default::default()
    };
    let forward_result = test_handle
        .call_capability(&forward_id, test_target.args.clone(), options)
        .await;
    if !forward_result.ok {
        return test_result(
            false,
            &request.alias,
            &test_target.server,
            false,
            "UNDO_TEST_FORWARD_FAILED",
        );
    }
    let Some(entry) = ledger.entries().await.into_iter().last() else {
        return test_result(
            false,
            &request.alias,
            &test_target.server,
            false,
            "UNDO_TEST_REGISTRATION_FAILED",
        );
    };
    let Some(snapshot_hash) = entry.snapshot_artifact_hash.as_deref() else {
        return test_result(
            false,
            &request.alias,
            &test_target.server,
            false,
            "UNDO_TEST_SNAPSHOT_MISSING",
        );
    };
    let initial_state = match snapshot_store.get(snapshot_hash) {
        Ok(snapshot) => snapshot,
        Err(_) => {
            return test_result(
                false,
                &request.alias,
                &test_target.server,
                false,
                "UNDO_TEST_SNAPSHOT_MISSING",
            )
        }
    };
    let inverse_options = crate::engine::ExecutionOptions {
        suppress_undo_registration: true,
        profile: access.profile.profile_id.clone(),
        context: entry
            .work_item_id
            .as_ref()
            .map(|work_item_id| crate::context::RequestContext {
                operation_id: entry.operation_id.clone(),
                work_item_id: Some(work_item_id.clone()),
                actor_id: entry.actor_id.clone(),
                grant_id: None,
            }),
        ..Default::default()
    };
    let inverse_result = test_handle
        .call_capability(
            &entry.resolved_inverse_capability,
            entry.resolved_inverse_args.clone(),
            inverse_options,
        )
        .await;
    if !inverse_result.ok {
        return test_result(
            false,
            &request.alias,
            &test_target.server,
            false,
            "UNDO_TEST_INVERSE_FAILED",
        );
    }
    let observed_state = match crate::engine::undo::read_checked_state(
        &test_state,
        pre_snapshot,
        &test_target.args,
        &access.profile,
        &test_policy,
    )
    .await
    {
        Ok(state) => state,
        Err(_) => {
            return test_result(
                false,
                &request.alias,
                &test_target.server,
                false,
                "UNDO_TEST_VERIFY_FAILED",
            )
        }
    };
    let equivalent = observed_state == initial_state;
    let result = UndoContractTestResponse {
        ok: equivalent,
        alias: request.alias,
        test_server: test_target.server.clone(),
        observationally_equivalent: equivalent,
        error_code: (!equivalent).then(|| "UNDO_TEST_STATE_MISMATCH".to_string()),
    };
    let status = if equivalent {
        StatusCode::OK
    } else {
        StatusCode::CONFLICT
    };
    (status, Json(json!({"ok": result.ok, "result": result}))).into_response()
}

fn test_result(
    ok: bool,
    alias: &str,
    test_server: &str,
    observationally_equivalent: bool,
    error_code: &str,
) -> axum::response::Response {
    let result = UndoContractTestResponse {
        ok,
        alias: alias.to_string(),
        test_server: test_server.to_string(),
        observationally_equivalent,
        error_code: Some(error_code.to_string()),
    };
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({"ok": false, "result": result})),
    )
        .into_response()
}

/// Handles `GET /v1/undo`.
pub async fn handle_list_undo(
    State(state): State<AppState>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    Query(query): Query<UndoListQuery>,
) -> impl IntoResponse {
    let status = match query.status.as_deref().map(parse_status).transpose() {
        Ok(status) => status,
        Err(message) => {
            return error_response(StatusCode::BAD_REQUEST, "INVALID_UNDO_STATUS", message)
        }
    };
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), None).await;
    let handle = crate::engine::ControlPlaneHandle::new(state);
    match handle
        .list_undo_entries(
            UndoListFilter {
                work_item_id: query.work_item_id,
                status,
                offset: query.offset.unwrap_or(0),
                limit: query.limit.unwrap_or(50),
            },
            &access,
        )
        .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(json!({"ok": true, "result": response})),
        )
            .into_response(),
        Err(error) => operation_error_response(error).into_response(),
    }
}

/// Handles `GET /v1/undo/:undo_id`.
pub async fn handle_get_undo(
    State(state): State<AppState>,
    Path(undo_id): Path<String>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
) -> impl IntoResponse {
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), None).await;
    let handle = crate::engine::ControlPlaneHandle::new(state);
    match handle.get_undo_entry(&undo_id, &access).await {
        Ok(entry) => (StatusCode::OK, Json(json!({"ok": true, "result": entry}))).into_response(),
        Err(error) => operation_error_response(error).into_response(),
    }
}

/// Handles single-entry execution and work-item rollback routes.
pub async fn handle_execute_undo(
    State(state): State<AppState>,
    Path(mut path_params): Path<HashMap<String, String>>,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<UndoExecutionRequest>,
) -> impl IntoResponse {
    let target = if let Some(undo_id) = path_params.remove("undo_id") {
        UndoExecutionTarget::Entry(undo_id)
    } else if let Some(work_item_id) = path_params.remove("work_item_id") {
        UndoExecutionTarget::WorkItem(work_item_id)
    } else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_UNDO_TARGET",
            "Undo route is missing its target identifier",
        );
    };
    prepare_undo_http(state, req_ext, prof_ext, headers, target, request).await
}

async fn prepare_undo_http(
    state: AppState,
    req_ext: axum::extract::Extension<Option<crate::rbac::TenantContext>>,
    prof_ext: Option<axum::extract::Extension<crate::context::ProfileContext>>,
    headers: axum::http::HeaderMap,
    target: UndoExecutionTarget,
    request: UndoExecutionRequest,
) -> Response {
    let context = resolve_request_context(None, &headers);
    let access = make_access(&state, req_ext.0.as_ref(), prof_ext.as_ref(), Some(context)).await;
    execute_undo_http(state, access, target, request).await
}

enum UndoExecutionTarget {
    Entry(String),
    WorkItem(String),
}

enum UndoExecutionResponse {
    Entry(RollbackResult),
    WorkItem(Vec<RollbackResult>),
}

async fn execute_undo_http(
    state: AppState,
    access: UndoAccessContext,
    target: UndoExecutionTarget,
    request: UndoExecutionRequest,
) -> Response {
    let handle = crate::engine::ControlPlaneHandle::new(state);
    let result = match target {
        UndoExecutionTarget::Entry(undo_id) => handle
            .execute_undo(
                &undo_id,
                &access,
                request.force_expired,
                request.reason.as_deref(),
            )
            .await
            .map(UndoExecutionResponse::Entry),
        UndoExecutionTarget::WorkItem(work_item_id) => handle
            .rollback_work_item(
                &work_item_id,
                &access,
                request.force_expired,
                request.reason.as_deref(),
            )
            .await
            .map(UndoExecutionResponse::WorkItem),
    };
    match result {
        Ok(UndoExecutionResponse::Entry(result)) if result.audit_error.is_some() => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ok": false,
                "error": {"code": "AUDIT_PERSISTENCE_FAILED", "message": result.audit_error},
                "result": result
            })),
        )
            .into_response(),
        Ok(UndoExecutionResponse::Entry(result)) if !result.ok => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"ok": false, "error": {"code": result.error_code, "message": result.error}, "result": result})),
        )
            .into_response(),
        Ok(UndoExecutionResponse::Entry(result)) => {
            (StatusCode::OK, Json(json!({"ok": true, "result": result}))).into_response()
        }
        Ok(UndoExecutionResponse::WorkItem(results))
            if results.iter().any(|result| result.audit_error.is_some()) =>
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"ok": false, "error": {"code": "AUDIT_PERSISTENCE_FAILED", "message": "One or more undo audit events could not be committed"}, "results": results})),
        )
        .into_response(),
        Ok(UndoExecutionResponse::WorkItem(results)) => (
            StatusCode::OK,
            Json(json!({
                "ok": results.iter().all(|result| result.ok),
                "results": results
            })),
        )
            .into_response(),
        Err(error) => operation_error_response(error),
    }
}

async fn make_access(
    state: &AppState,
    tenant: Option<&crate::rbac::TenantContext>,
    profile: Option<&axum::extract::Extension<crate::context::ProfileContext>>,
    context: Option<RequestContext>,
) -> UndoAccessContext {
    let profile = profile
        .map(|extension| extension.0.clone())
        .unwrap_or_default();
    let base_policy = state.policy.read().await.clone();
    let policy = tenant
        .map(|tenant| tenant.effective_policy.clone())
        .unwrap_or(base_policy)
        .merge_with_profile(profile.profile_policy.as_ref());
    let default_role = state.rbac_engine.default_role();
    let role = tenant
        .map(|tenant| tenant.role.clone())
        .unwrap_or(default_role);
    let actor_id = tenant
        .and_then(|tenant| tenant.actor_id.clone())
        .or_else(|| {
            context
                .as_ref()
                .and_then(|context| context.actor_id.clone())
        });
    let mut request_context = context.unwrap_or_default();
    if request_context.actor_id.is_none() {
        request_context.actor_id = actor_id.clone();
    }
    UndoAccessContext {
        tenant_id: tenant
            .map(|tenant| tenant.tenant_id.clone())
            .unwrap_or_else(|| "default".to_string()),
        role,
        actor_id,
        request_context,
        policy,
        profile,
        grant_expires_at_epoch_secs: tenant.and_then(|tenant| tenant.grant_expires_at_epoch_secs),
        trace_id: crate::http_v1::next_trace_id(),
    }
}

fn parse_status(value: &str) -> Result<UndoEntryStatus, String> {
    match value {
        "armed" => Ok(UndoEntryStatus::Armed),
        "executing" => Ok(UndoEntryStatus::Executing),
        "reverted" => Ok(UndoEntryStatus::Reverted),
        "expired" => Ok(UndoEntryStatus::Expired),
        "failed" => Ok(UndoEntryStatus::Failed),
        _ => Err("status must be armed, executing, reverted, expired, or failed".to_string()),
    }
}

fn operation_error_response(error: crate::undo::UndoOperationError) -> axum::response::Response {
    let status =
        StatusCode::from_u16(error.http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    error_response(status, error.code, error.message)
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
) -> axum::response::Response {
    (
        status,
        Json(json!({"ok": false, "error": {"code": code, "message": message.into()}})),
    )
        .into_response()
}

struct TemporaryUndoTestDir(PathBuf);

impl TemporaryUndoTestDir {
    fn create() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "warmplane-undo-test-{}-{:032x}",
            std::process::id(),
            rand::random::<u128>()
        ));
        std::fs::create_dir(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self(path))
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TemporaryUndoTestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
