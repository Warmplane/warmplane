// Rust guideline compliant 2026-11-21

//! Runs confirmed isolated contract tests and binds activation to the resulting evidence.

use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use anyhow::{ensure, Context, Result};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use super::{generate, revision, Draft, DraftRequest};
use crate::{
    config::{AliasTarget, McpConfig, ServerConfig},
    context::ProfileContext,
    daemon::AppState,
    http_v1::{
        config_api::{trust_admin_required, TrustAdmin},
        undo_api::{make_access, run_undo_contract_test},
    },
    rbac::TenantContext,
};

const REVIEW_TTL: Duration = Duration::from_secs(900);
const MAX_REVIEWS: usize = 512;

#[derive(Default)]
pub(crate) struct Reviews(Mutex<HashMap<String, Proof>>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProofKind {
    Confirmation,
    Activation,
}

struct Proof {
    kind: ProofKind,
    digest: [u8; 32],
    created: Instant,
}

impl Reviews {
    async fn issue(&self, kind: ProofKind, digest: [u8; 32]) -> Result<String> {
        let mut reviews = self.0.lock().await;
        reviews.retain(|_, review| review.created.elapsed() < REVIEW_TTL);
        ensure!(
            reviews.len() < MAX_REVIEWS,
            "Reversibility review storage is full"
        );
        let key = format!("{:032x}", rand::random::<u128>());
        reviews.insert(
            key.clone(),
            Proof {
                kind,
                digest,
                created: Instant::now(),
            },
        );
        Ok(key)
    }
    async fn consume_confirmation(&self, token: &str, digest: [u8; 32]) -> bool {
        let mut reviews = self.0.lock().await;
        let matches = reviews.get(token).is_some_and(|proof| {
            proof.kind == ProofKind::Confirmation
                && proof.created.elapsed() < REVIEW_TTL
                && proof.digest == digest
        });
        if matches {
            reviews.remove(token);
        }
        matches
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RecordRequest {
    draft: DraftRequest,
    revisions: BTreeMap<String, String>,
    confirmation_token: String,
    confirmed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ActivateRequest {
    draft: DraftRequest,
    revisions: BTreeMap<String, String>,
    review_token: String,
    alias: String,
}

pub(crate) async fn handle_record(
    State(state): State<AppState>,
    admin: Option<Extension<TrustAdmin>>,
    Extension(tenant): Extension<Option<TenantContext>>,
    profile: Option<Extension<ProfileContext>>,
    Json(request): Json<RecordRequest>,
) -> Response {
    if !is_admin(admin) {
        return trust_admin_required();
    }
    if !request.confirmed {
        return conflict("Confirm the isolated test invocation before recording");
    }
    let access = make_access(&state, tenant.as_ref(), profile.as_ref(), None).await;
    if state.rbac_engine.is_enabled() && !state.rbac_engine.undo_permissions(&access.role).execute {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"ok": false, "error": "Undo test permission is required"})),
        )
            .into_response();
    }
    let scope = Scope::new(tenant.as_ref(), profile.as_ref().map(|item| &item.0));
    let lifecycle = state.upstream_lifecycle.lock().await;
    let (first, forward, contract) = match confirm_test(&state, &request, &scope).await {
        Ok(confirmed) => confirmed,
        Err(error) => return conflict(&error.to_string()),
    };
    let response =
        run_undo_contract_test(&state, &access, &first.draft.forward, &forward, &contract).await;
    drop(lifecycle);
    if !response.status().is_success() {
        return response;
    }
    match finish_record(&state, request, &scope, first.digest).await {
        Ok((mut draft, token)) => {
            draft.requires_test = false;
            Json(json!({"ok": true, "draft": draft, "reviewToken": token, "cleanup": "verified"}))
                .into_response()
        }
        Err(error) => conflict(&error.to_string()),
    }
}

async fn confirm_test(
    state: &AppState,
    request: &RecordRequest,
    scope: &Scope<'_>,
) -> Result<(Review, String, crate::config::ReversibilityConfig)> {
    let first = review(state, request.draft.clone(), &request.revisions, scope).await?;
    ensure!(
        state
            .reversibility_reviews
            .consume_confirmation(&request.confirmation_token, first.digest)
            .await,
        "Test confirmation expired, changed, or was already used. Generate a fresh draft"
    );
    let (forward, contract) = first.test_bindings.remap(&first.draft)?;
    Ok((first, forward, contract))
}

async fn finish_record(
    state: &AppState,
    request: RecordRequest,
    scope: &Scope<'_>,
    original: [u8; 32],
) -> Result<(Draft, String)> {
    let current = review(state, request.draft, &request.revisions, scope).await?;
    ensure!(
        original == current.digest,
        "A capability or test target changed during recording"
    );
    let token = state
        .reversibility_reviews
        .issue(ProofKind::Activation, current.digest)
        .await?;
    Ok((current.draft, token))
}

pub(super) async fn preview(
    state: &AppState,
    request: DraftRequest,
    tenant: Option<&TenantContext>,
    profile: Option<&ProfileContext>,
) -> Result<serde_json::Value> {
    let draft = generate(state, request).await?;
    let scope = Scope::new(tenant, profile);
    let config =
        crate::config::load_config(&state.config_path).context("Configuration is unavailable")?;
    let tests = test_revisions(state, &draft, &config, &scope).await?;
    let digest = scope.digest(&draft, &tests.revisions)?;
    let token = state
        .reversibility_reviews
        .issue(ProofKind::Confirmation, digest)
        .await?;
    Ok(json!({
        "ok": true, "draft": draft,
        "testForward": tests.test_id(&draft.forward)?,
        "confirmationToken": token
    }))
}

pub(crate) async fn handle_activate(
    State(state): State<AppState>,
    admin: Option<Extension<TrustAdmin>>,
    Extension(tenant): Extension<Option<TenantContext>>,
    profile: Option<Extension<ProfileContext>>,
    Json(request): Json<ActivateRequest>,
) -> Response {
    if !is_admin(admin) {
        return trust_admin_required();
    }
    let scope = Scope::new(tenant.as_ref(), profile.as_ref().map(|item| &item.0));
    if let Err(error) = activate(&state, request, &scope).await {
        return conflict(&error.to_string());
    }
    match state.reload_from_disk().await {
        Ok(summary) => Json(json!({"ok": true, "saved": true, "reload": summary})).into_response(),
        Err(_) => (
            StatusCode::ACCEPTED,
            Json(json!({"ok": true, "saved": true, "reload": null})),
        )
            .into_response(),
    }
}

async fn activate(state: &AppState, request: ActivateRequest, scope: &Scope<'_>) -> Result<()> {
    let _lifecycle = state.upstream_lifecycle.lock().await;
    let review = review(state, request.draft, &request.revisions, scope).await?;
    let mut proofs = state.reversibility_reviews.0.lock().await;
    let proof = proofs
        .get(&request.review_token)
        .filter(|proof| proof.kind == ProofKind::Activation && proof.created.elapsed() < REVIEW_TTL)
        .context("Test evidence is unavailable or expired")?;
    ensure!(
        proof.digest == review.digest,
        "Test evidence belongs to a different draft or operator"
    );
    save_reviewed_alias(state, request.alias, review).await?;
    proofs.remove(&request.review_token);
    Ok(())
}

async fn save_reviewed_alias(state: &AppState, alias: String, review: Review) -> Result<()> {
    ensure!(
        !alias.is_empty()
            && alias.len() <= 128
            && alias
                .bytes()
                .all(|byte| { byte.is_ascii_alphanumeric() || b"-_.".contains(&byte) }),
        "Enter a valid capability alias"
    );
    let catalog = state.capabilities.read().await;
    let forward = catalog
        .get(&review.draft.forward)
        .context("Forward capability changed")?;
    let target = format!("{}.{}", forward.server, forward.tool);
    let mut config = review.config;
    let existing = config.capability_aliases.get(&alias);
    ensure!(
        existing.is_none_or(|current| current.target() == target),
        "An existing alias points at a different operation"
    );
    ensure!(
        catalog
            .get(&alias)
            .is_none_or(|entry| entry.server == forward.server && entry.tool == forward.tool),
        "Alias conflicts with an existing capability"
    );
    ensure!(
        !config
            .capability_aliases
            .iter()
            .any(|(name, value)| name != &alias && value.target() == target),
        "Activate the contract under the existing alias for this operation"
    );
    let mut contract = review.draft.contract;
    contract.remap_capabilities(|id| {
        Ok(if id == review.draft.forward {
            alias.clone()
        } else {
            id.to_owned()
        })
    })?;
    let alias_config = AliasTarget::Detailed {
        target,
        summary: existing.and_then(AliasTarget::summary).map(str::to_owned),
        description: existing
            .and_then(AliasTarget::description)
            .map(str::to_owned),
        passthrough: existing.is_some_and(AliasTarget::is_passthrough),
        reversibility: Some(contract),
    };
    config.capability_aliases.insert(alias, alias_config);
    crate::config::save_config(&state.config_path, &config)
        .context("Could not save the reviewed reversibility contract")
}

struct Scope<'a> {
    tenant: Option<&'a TenantContext>,
    profile: Option<&'a ProfileContext>,
}

impl<'a> Scope<'a> {
    fn new(tenant: Option<&'a TenantContext>, profile: Option<&'a ProfileContext>) -> Self {
        Self { tenant, profile }
    }

    fn digest(&self, draft: &Draft, test_tools: &BTreeMap<String, String>) -> Result<[u8; 32]> {
        let identity = self.tenant.map(|tenant| {
            (
                &tenant.tenant_id,
                &tenant.role,
                &tenant.actor_id,
                &tenant.grant_id,
            )
        });
        Ok(Sha256::digest(serde_json::to_vec(&json!({
            "identity": identity,
            "profile": self.profile.and_then(|profile| profile.profile_id.as_deref()),
            "draft": draft,
            "testTools": test_tools
        }))?)
        .into())
    }
}

struct Review {
    draft: Draft,
    digest: [u8; 32],
    config: McpConfig,
    test_bindings: TestBindings,
}

async fn review(
    state: &AppState,
    request: DraftRequest,
    expected: &BTreeMap<String, String>,
    scope: &Scope<'_>,
) -> Result<Review> {
    let draft = generate(state, request).await?;
    ensure!(
        &draft.revisions == expected,
        "Capability definitions changed. Generate a fresh draft"
    );
    let config =
        crate::config::load_config(&state.config_path).context("Configuration is unavailable")?;
    let tests = test_revisions(state, &draft, &config, scope).await?;
    let digest = scope.digest(&draft, &tests.revisions)?;
    Ok(Review {
        draft,
        digest,
        config,
        test_bindings: tests,
    })
}

struct TestBindings {
    ids: BTreeMap<String, String>,
    revisions: BTreeMap<String, String>,
}

impl TestBindings {
    fn remap(&self, draft: &Draft) -> Result<(String, crate::config::ReversibilityConfig)> {
        let mut contract = draft.contract.clone();
        contract.remap_capabilities(|id| self.test_id(id).map(str::to_owned))?;
        Ok((self.test_id(&draft.forward)?.to_owned(), contract))
    }

    fn test_id(&self, live: &str) -> Result<&str> {
        self.ids
            .get(live)
            .map(String::as_str)
            .context("A matching test capability is unavailable")
    }
}

async fn test_revisions(
    state: &AppState,
    draft: &Draft,
    config: &McpConfig,
    scope: &Scope<'_>,
) -> Result<TestBindings> {
    let target = draft
        .contract
        .test_target
        .as_ref()
        .context("A test target is required")?;
    let test = config
        .mcp_servers
        .get(&target.server)
        .filter(|server| server.test_only)
        .context("The test source must be configured with testOnly")?;
    let active = state.server_configs.read().await;
    ensure!(
        active.get(&target.server) == Some(test),
        "Reload the test source before recording"
    );
    let catalog = state.tool_catalog().await;
    let mut checked = BTreeMap::from([(
        format!("source:{}", target.server),
        hex::encode(Sha256::digest(serde_json::to_vec(test)?)),
    )]);
    crate::openapi::verify_local_revision(state, &target.server, test).await?;
    let mut ids = BTreeMap::new();
    for id in draft.revisions.keys() {
        let live = catalog
            .items
            .get(id)
            .context("A reviewed capability is no longer available")?;
        check_live_source(state, &live.server, config, &active, test, &mut checked).await?;
        let (test_id, test_tool) = match_test_tool(&catalog.items, live, &target.server, scope)?;
        check_scope(state, scope, id, test_id, &target.server).await?;
        ids.insert(id.clone(), test_id.clone());
        checked.insert(test_id.clone(), revision(state, test_tool)?);
    }
    Ok(TestBindings {
        ids,
        revisions: checked,
    })
}

async fn check_live_source(
    state: &AppState,
    server: &str,
    config: &McpConfig,
    active: &HashMap<String, ServerConfig>,
    test: &ServerConfig,
    checked: &mut BTreeMap<String, String>,
) -> Result<()> {
    let config = config
        .mcp_servers
        .get(server)
        .context("A reviewed source is no longer configured")?;
    ensure!(
        active.get(server) == Some(config),
        "Reload the selected source before recording"
    );
    check_isolation(test, config)?;
    check_source_revision(state, server, config, checked).await
}

fn match_test_tool<'a>(
    catalog: &'a HashMap<String, crate::daemon::CapabilityMeta>,
    live: &crate::daemon::CapabilityMeta,
    server: &str,
    scope: &Scope<'_>,
) -> Result<(&'a String, &'a crate::daemon::CapabilityMeta)> {
    let (id, test) = catalog
        .iter()
        .find(|(_, meta)| meta.server == server && meta.tool == live.tool)
        .context("A corresponding operation is unavailable on the test source")?;
    ensure!(
        test.input_schema == live.input_schema,
        "A test operation schema differs from the reviewed operation"
    );
    ensure!(
        scope
            .profile
            .is_none_or(|profile| profile.is_server_allowed(&live.server)),
        "The reviewed operation is outside the current profile"
    );
    Ok((id, test))
}

async fn check_source_revision(
    state: &AppState,
    server: &str,
    config: &ServerConfig,
    checked: &mut BTreeMap<String, String>,
) -> Result<()> {
    let key = format!("source:{server}");
    if let std::collections::btree_map::Entry::Vacant(entry) = checked.entry(key) {
        crate::openapi::verify_local_revision(state, server, config).await?;
        entry.insert(hex::encode(Sha256::digest(serde_json::to_vec(config)?)));
    }
    Ok(())
}

async fn check_scope(
    state: &AppState,
    scope: &Scope<'_>,
    id: &str,
    test: &str,
    test_server: &str,
) -> Result<()> {
    let base = state.policy.read().await;
    let role = scope
        .tenant
        .map(|tenant| tenant.role.clone())
        .unwrap_or_else(|| state.rbac_engine.default_role());
    let policy = state
        .rbac_engine
        .compute_effective_policy(&role, &base)
        .merge_with_profile(
            scope
                .profile
                .and_then(|profile| profile.profile_policy.as_ref()),
        );
    ensure!(
        scope
            .profile
            .is_none_or(|profile| profile.is_server_allowed(test_server))
            && policy.allows(id)
            && policy.allows(test),
        "Test or reviewed operation is denied by the current policy or profile"
    );
    ensure!(
        !policy.requires_approval(test),
        "The isolated recorder cannot dispatch operations awaiting approval"
    );
    Ok(())
}

fn check_isolation(test: &ServerConfig, live: &ServerConfig) -> Result<()> {
    ensure!(
        !live.test_only,
        "The reviewed source must be distinct from test-only sources"
    );
    match (&test.open_api, &live.open_api) {
        (Some(test), Some(live)) => {
            different_origin(&test.base_url, &live.base_url)?;
            ensure!(
                test.auth != live.auth || matches!(test.auth, crate::config::OpenApiAuth::None),
                "The test source must use a separate credential binding"
            );
        }
        (None, None) if test.url.is_some() && live.url.is_some() => {
            different_origin(
                test.url.as_deref().unwrap_or_default(),
                live.url.as_deref().unwrap_or_default(),
            )?;
            ensure!(
                test.auth.is_none() || test.auth != live.auth,
                "The test source must use a separate credential binding"
            );
        }
        (None, None) if test.command.is_some() && live.command.is_some() => {
            ensure!(
                test.command != live.command || test.args != live.args,
                "The stdio test source must use a distinct command or arguments"
            );
            ensure!(
                test.env.is_empty() || test.env != live.env,
                "The test source must use a separate environment binding"
            );
        }
        _ => anyhow::bail!("Use the same transport type for the live and isolated test source"),
    }
    Ok(())
}

fn different_origin(test: &str, live: &str) -> Result<()> {
    let test = reqwest::Url::parse(test)
        .ok()
        .context("Invalid test origin")?;
    let live = reqwest::Url::parse(live)
        .ok()
        .context("Invalid live origin")?;
    ensure!(
        test.origin() != live.origin(),
        "The test source must use a distinct origin"
    );
    Ok(())
}

fn is_admin(admin: Option<Extension<TrustAdmin>>) -> bool {
    admin.is_some_and(|Extension(TrustAdmin(admin))| admin)
}

fn conflict(message: &str) -> Response {
    (
        StatusCode::CONFLICT,
        Json(json!({"ok": false, "error": message})),
    )
        .into_response()
}
