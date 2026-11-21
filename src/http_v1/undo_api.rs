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
    run_undo_contract_test(
        &state,
        &access,
        &request.alias,
        alias_target.target(),
        &contract,
    )
    .await
}

pub(crate) async fn run_undo_contract_test(
    state: &AppState,
    access: &UndoAccessContext,
    alias: &str,
    forward_id: &str,
    contract: &crate::config::ReversibilityConfig,
) -> axum::response::Response {
    let (forward, contract) = match resolve_test_operations(state, forward_id, contract).await {
        Ok(resolved) => resolved,
        Err(error) => return error.response(),
    };
    let test = match TestContract::new(&forward, &contract) {
        Ok(test) => test,
        Err(error) => return error.response(),
    };
    let runtime = match UndoTestRuntime::initialize(state, access, &test).await {
        Ok(runtime) => runtime,
        Err(error) => return error.response(),
    };
    let result = runtime.run(access, &test).await;
    let (equivalent, error_code) = match result {
        Ok(true) => (true, None),
        Ok(false) => (false, Some("UNDO_TEST_STATE_MISMATCH")),
        Err(code) => (false, Some(code)),
    };
    let result = UndoContractTestResponse {
        ok: equivalent,
        alias: alias.to_owned(),
        test_server: test.target.server.clone(),
        observationally_equivalent: equivalent,
        error_code: error_code.map(str::to_owned),
    };
    let status = if equivalent {
        StatusCode::OK
    } else {
        StatusCode::CONFLICT
    };
    (status, Json(json!({"ok": result.ok, "result": result}))).into_response()
}

struct TestError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl TestError {
    fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
        }
    }

    fn response(self) -> axum::response::Response {
        error_response(self.status, self.code, self.message)
    }
}

async fn resolve_test_operations(
    state: &AppState,
    forward: &str,
    contract: &crate::config::ReversibilityConfig,
) -> Result<(String, crate::config::ReversibilityConfig), TestError> {
    let mut contract = contract.clone();
    let Some(target) = contract
        .test_target
        .as_ref()
        .map(|target| target.server.clone())
    else {
        return Ok((forward.to_owned(), contract));
    };
    if !state
        .server_configs
        .read()
        .await
        .get(&target)
        .is_some_and(|config| config.open_api.is_some())
    {
        return Ok((forward.to_owned(), contract));
    }
    let catalog = state.tool_catalog().await;
    let resolve = |id: &str| {
        let live = catalog
            .items
            .get(id)
            .or_else(|| {
                catalog
                    .items
                    .values()
                    .find(|item| format!("{}.{}", item.server, item.tool) == id)
            })
            .ok_or_else(|| anyhow::anyhow!("A contract capability is unavailable"))?;
        catalog
            .items
            .iter()
            .find(|(_, meta)| meta.server == target && meta.tool == live.tool)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow::anyhow!("The corresponding test operation is unavailable"))
    };
    let missing = |_| {
        TestError::new(
            StatusCode::NOT_FOUND,
            "UNDO_TEST_CAPABILITY_MISSING",
            "A corresponding contract operation is unavailable on the test source",
        )
    };
    let forward = resolve(forward).map_err(missing)?;
    contract.remap_capabilities(resolve).map_err(missing)?;
    Ok((forward, contract))
}

struct TestContract<'a> {
    forward: &'a str,
    target: &'a crate::config::UndoTestTargetConfig,
    snapshot: &'a crate::config::PreSnapshotConfig,
    contract: &'a crate::config::ReversibilityConfig,
    capabilities: Vec<&'a str>,
}

impl<'a> TestContract<'a> {
    fn new(
        forward: &'a str,
        contract: &'a crate::config::ReversibilityConfig,
    ) -> Result<Self, TestError> {
        let missing = |code, message| TestError::new(StatusCode::BAD_REQUEST, code, message);
        let target = contract.test_target.as_deref().ok_or_else(|| {
            missing(
                "UNDO_TEST_TARGET_REQUIRED",
                "Configure reversibility.testTarget before testing this alias",
            )
        })?;
        let snapshot = contract.pre_snapshot.as_deref().ok_or_else(|| {
            missing(
                "UNDO_TEST_SNAPSHOT_REQUIRED",
                "A preSnapshot is required to verify observational equivalence",
            )
        })?;
        let inverse = contract.inverse.as_deref().ok_or_else(|| {
            missing(
                "UNDO_TEST_INVERSE_REQUIRED",
                "The reversibility contract has no inverse",
            )
        })?;
        let mut capabilities = vec![
            forward,
            snapshot.capability.as_str(),
            inverse.capability.as_str(),
        ];
        if let Some(check) = contract.pre_undo_check.as_deref() {
            capabilities.push(&check.capability);
        }
        capabilities.sort_unstable();
        capabilities.dedup();
        Ok(Self {
            forward,
            target,
            snapshot,
            contract,
            capabilities,
        })
    }

    async fn catalog(
        &self,
        state: &AppState,
    ) -> Result<HashMap<String, crate::daemon::CapabilityMeta>, TestError> {
        let active = state.capabilities.read().await;
        let mut catalog = HashMap::new();
        for id in &self.capabilities {
            let mut meta = active.get(*id).cloned().ok_or_else(|| {
                TestError::new(
                    StatusCode::NOT_FOUND,
                    "UNDO_TEST_CAPABILITY_MISSING",
                    "A contract capability is not in the active catalog",
                )
            })?;
            meta.server.clone_from(&self.target.server);
            meta.reversibility = (*id == self.forward).then(|| self.contract.clone());
            catalog.insert((*id).to_owned(), meta);
        }
        Ok(catalog)
    }
}

struct UndoTestRuntime {
    state: AppState,
    policy: crate::daemon::Policy,
    snapshots: crate::undo::SnapshotStore,
    ledger: crate::undo::UndoLedgerStore,
    _directory: TemporaryUndoTestDir,
}

impl UndoTestRuntime {
    async fn initialize(
        live: &AppState,
        access: &UndoAccessContext,
        test: &TestContract<'_>,
    ) -> Result<Self, TestError> {
        let policy = Self::test_policy(live, access, test)?;
        let sender = live
            .servers
            .read()
            .await
            .get(&test.target.server)
            .cloned()
            .ok_or_else(|| {
                TestError::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "UNDO_TEST_SERVER_UNAVAILABLE",
                    "Configured test server is not connected",
                )
            })?;
        let capabilities = test.catalog(live).await?;
        let (directory, path, snapshots, ledger) = Self::storage().map_err(|_| {
            TestError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "UNDO_TEST_STORAGE_UNAVAILABLE",
                "Temporary test storage is unavailable",
            )
        })?;
        let mut state = live.clone();
        state.servers = Arc::new(tokio::sync::RwLock::new(HashMap::from([(
            test.target.server.clone(),
            sender,
        )])));
        state.capabilities = Arc::new(tokio::sync::RwLock::new(capabilities));
        state.policy = Arc::new(tokio::sync::RwLock::new(policy.clone()));
        state.snapshot_store = Some(snapshots.clone());
        state.undo_ledger = Some(ledger.clone());
        state.config_path = path;
        state.task_registry = crate::tasks::TaskRegistry::default();
        state.hold_coordinator = crate::emissions::HoldCoordinator::default();
        Ok(Self {
            state,
            policy,
            snapshots,
            ledger,
            _directory: directory,
        })
    }

    fn test_policy(
        live: &AppState,
        access: &UndoAccessContext,
        test: &TestContract<'_>,
    ) -> Result<crate::daemon::Policy, TestError> {
        if !access.profile.is_server_allowed(&test.target.server) {
            return Err(TestError::new(
                StatusCode::FORBIDDEN,
                "PROFILE_DENIED",
                "The configured test server is outside the active profile",
            ));
        }
        let mut policy = live
            .rbac_engine
            .compute_effective_policy(&access.role, &access.policy)
            .merge_with_profile(access.profile.profile_policy.as_ref());
        if test.capabilities.iter().any(|id| !policy.allows(id)) {
            return Err(TestError::new(
                StatusCode::FORBIDDEN,
                "POLICY_DENIED",
                "The test contract is denied by the current effective policy",
            ));
        }
        policy.webhook = None;
        Ok(policy)
    }

    fn storage() -> anyhow::Result<(
        TemporaryUndoTestDir,
        String,
        crate::undo::SnapshotStore,
        crate::undo::UndoLedgerStore,
    )> {
        let directory = TemporaryUndoTestDir::create()?;
        let snapshots = crate::undo::SnapshotStore::open(directory.path().join("artifacts"))?;
        let ledger = crate::undo::UndoLedgerStore::open(directory.path().join("undo.json"))?;
        let path = directory
            .path()
            .join("config.json")
            .to_string_lossy()
            .into_owned();
        crate::config::save_config(&path, &crate::config::McpConfig::default())?;
        Ok((directory, path, snapshots, ledger))
    }

    async fn run(
        &self,
        access: &UndoAccessContext,
        test: &TestContract<'_>,
    ) -> Result<bool, &'static str> {
        self.forward(access, test).await?;
        let entry = self
            .ledger
            .entries()
            .await
            .into_iter()
            .last()
            .ok_or("UNDO_TEST_REGISTRATION_FAILED")?;
        let snapshot = entry
            .snapshot_artifact_hash
            .as_deref()
            .ok_or("UNDO_TEST_SNAPSHOT_MISSING")?;
        let before = self
            .snapshots
            .get(snapshot)
            .map_err(|_| "UNDO_TEST_SNAPSHOT_MISSING")?;
        let result =
            crate::undo::execute_undo(&self.state, access, &entry.undo_id, false, None).await;
        if !result.as_ref().is_ok_and(|result| result.ok) {
            return Err("UNDO_TEST_INVERSE_FAILED");
        }
        let after = crate::engine::undo::read_checked_state(
            &self.state,
            test.snapshot,
            &test.target.args,
            &access.profile,
            &self.policy,
        )
        .await
        .map_err(|_| "UNDO_TEST_VERIFY_FAILED")?;
        Ok(before == after)
    }

    async fn forward(
        &self,
        access: &UndoAccessContext,
        test: &TestContract<'_>,
    ) -> Result<(), &'static str> {
        let handle = crate::engine::ControlPlaneHandle::new(self.state.clone())
            .with_tenant_context(Some(crate::rbac::TenantContext {
                tenant_id: access.tenant_id.clone(),
                role: access.role.clone(),
                actor_id: access.actor_id.clone(),
                grant_id: access.request_context.grant_id.clone(),
                grant_expires_at_epoch_secs: access.grant_expires_at_epoch_secs,
                effective_policy: self.policy.clone(),
            }));
        let options = crate::engine::ExecutionOptions {
            request_id: Some(crate::http_v1::next_trace_id()),
            profile: access.profile.profile_id.clone(),
            context: Some(crate::context::RequestContext {
                operation_id: Some(crate::http_v1::next_trace_id()),
                actor_id: access.actor_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = handle
            .call_capability(test.forward, test.target.args.clone(), options)
            .await;
        if result.ok {
            Ok(())
        } else if result.retry.upstream_execution_state == "unknown" {
            Err("UNDO_TEST_FORWARD_OUTCOME_UNKNOWN")
        } else {
            Err("UNDO_TEST_FORWARD_FAILED")
        }
    }
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
    Entry(Box<RollbackResult>),
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
            .map(|result| UndoExecutionResponse::Entry(Box::new(result))),
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

pub(crate) async fn make_access(
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
