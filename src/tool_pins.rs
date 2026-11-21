// Rust guideline compliant 2026-11-20

//! Operator-approved definitions and bounded upstream observations.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::Deref,
    path::Path,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::{sync::Mutex, time::Instant};

use crate::{
    audit::{AuditEventStatus, AuditEventType, RawAuditEvent},
    catalog::CatalogEventStore,
    daemon::{AppState, CapabilityMeta},
    idempotency::canonicalize_json,
    storage::AtomicFile,
};

pub(crate) const REFRESH_INTERVAL: Duration = Duration::from_secs(15);
const MAX_OBSERVATION_AGE: Duration = Duration::from_secs(30);

type Definitions = BTreeMap<String, Definition>;
type ApprovedDefinitions = BTreeMap<String, Definitions>;
type PendingApprovals = BTreeSet<(String, String)>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Definition {
    digest: String,
    value: Value,
}

impl Definition {
    pub(crate) fn from_tool(tool: &Tool) -> Self {
        let mut annotations = serde_json::to_value(&tool.annotations).unwrap_or(Value::Null);
        if let Some(fields) = annotations.as_object_mut() {
            fields.remove("title");
            if fields.is_empty() {
                annotations = Value::Null;
            }
        }
        let mut meta = tool.meta.clone();
        if let Some(fields) = &mut meta {
            for key in ["ui", "ui/resourceUri", "mcp/www/modelcontextprotocol.io/ui"] {
                fields.remove(key);
            }
        }
        let value = json!({
            "version": 1,
            "name": tool.name,
            "description": tool.description,
            "inputSchema": tool.input_schema,
            "outputSchema": tool.output_schema,
            "annotations": annotations,
            "meta": meta.filter(|fields| !fields.is_empty()),
        });
        let digest = hex::encode(Sha256::digest(canonicalize_json(&value)));
        Self { digest, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PinSession(u128);

#[derive(Debug)]
pub(crate) struct PinDenied;

impl PinDenied {
    pub(crate) const CODE: &str = "TOOL_DEFINITION_UNAPPROVED";
}

impl From<PinDenied> for crate::WarmplaneError {
    fn from(_: PinDenied) -> Self {
        Self::new(
            PinDenied::CODE,
            "The upstream tool definition is unavailable or does not match its approval",
            false,
        )
    }
}

impl AppState {
    pub(crate) fn notify_tool_quarantine(&self, server: &str, reason: &'static str) {
        let app = self.clone();
        let server = server.to_owned();
        tokio::spawn(async move {
            const EVENT: &str = "tool_pin.quarantined";
            let Some(cfg) = app
                .policy
                .read()
                .await
                .webhook
                .clone()
                .filter(|cfg| cfg.events.iter().any(|event| event == EVENT))
            else {
                return;
            };
            let Some(secret) = cfg.resolve_secret() else {
                tracing::warn!("tool pin quarantine webhook signing secret is unavailable");
                return;
            };
            let payload = crate::chatops::format_webhook_payload(
                cfg.format.unwrap_or_default(),
                EVENT,
                &json!({"serverId": server, "reason": reason}),
                None,
            );
            if let Err(error) =
                crate::chatops::post_signed_webhook(&cfg, EVENT, &payload, &secret).await
            {
                tracing::warn!(%error, server_id = %server, "tool pin quarantine notification failed");
            }
        });
    }

    pub(crate) async fn tool_catalog(&self) -> ToolCatalog<'_> {
        let entries = self.capabilities.read().await;
        let base = self.catalog_version.read().await.clone();
        let state = self.tool_pins.read();
        if state.observations.is_empty() {
            return ToolCatalog {
                items: CatalogItems::Original(entries),
                version: base,
            };
        }
        let mut visible = HashMap::new();
        let mut digests = BTreeMap::new();
        for (id, meta) in entries.iter() {
            if let Ok(binding) = state.binding(&meta.server, &meta.tool) {
                visible.insert(id.clone(), meta.clone());
                if let Some(binding) = binding {
                    digests.insert(id, binding);
                }
            }
        }
        let digest = hex::encode(Sha256::digest(canonicalize_json(
            &json!({"base": base, "pins": digests}),
        )));
        ToolCatalog {
            items: CatalogItems::Filtered(visible),
            version: format!("sha256:{digest}"),
        }
    }

    pub(crate) fn record_tool_catalog_events(
        &self,
        server: &str,
        capabilities: &HashMap<String, CapabilityMeta>,
    ) {
        let mut state = self.tool_pins.write();
        let visible = capabilities
            .iter()
            .filter(|(_, meta)| meta.server == server && state.binding(server, &meta.tool).is_ok())
            .map(|(id, meta)| (id.clone(), meta.clone()))
            .collect();
        state.record_visibility(server, visible, &self.event_store);
    }

    pub(crate) async fn tool_error(
        &self,
        server: &str,
        failure: crate::daemon::UpstreamCallError,
    ) -> (crate::WarmplaneError, &'static str) {
        use crate::daemon::UpstreamCallError;
        match failure {
            UpstreamCallError::Dispatch(failure) => (
                crate::WarmplaneError::new(failure.code, failure.message, false),
                failure.execution_state,
            ),
            UpstreamCallError::ToolDefinitionChanged => (PinDenied.into(), "not_started"),
            UpstreamCallError::Upstream(message) => {
                self.circuit_breakers.record_failure(server).await;
                (
                    crate::WarmplaneError::new(
                        "UPSTREAM_ERROR",
                        self.safe_upstream_error(message).await,
                        false,
                    ),
                    "unknown",
                )
            }
            UpstreamCallError::Timeout => {
                self.circuit_breakers.record_failure(server).await;
                (
                    crate::WarmplaneError::new("UPSTREAM_TIMEOUT", "Upstream call timed out", true),
                    "unknown",
                )
            }
        }
    }
}

pub(crate) struct ToolCatalog<'a> {
    pub(crate) items: CatalogItems<'a>,
    pub(crate) version: String,
}

pub(crate) enum CatalogItems<'a> {
    Original(tokio::sync::RwLockReadGuard<'a, HashMap<String, CapabilityMeta>>),
    Filtered(HashMap<String, CapabilityMeta>),
}

impl Deref for CatalogItems<'_> {
    type Target = HashMap<String, CapabilityMeta>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Original(entries) => entries,
            Self::Filtered(entries) => entries,
        }
    }
}
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PinStatus {
    Missing,
    Matching,
    Changed,
    Unavailable,
}

#[derive(Debug, Serialize)]
pub(crate) struct PinReview {
    server: String,
    tool: Option<String>,
    status: PinStatus,
    current: Option<Definition>,
    approved: Option<Definition>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApprovePin {
    pub(crate) server: String,
    pub(crate) tool: String,
    pub(crate) digest: String,
}

#[derive(Debug)]
pub(crate) enum PinApprovalError {
    Stale,
    Audit,
    Storage,
}

struct Observation {
    session: PinSession,
    epoch: u64,
    kind: ObservationKind,
    observed_at: Option<Instant>,
    definitions: Definitions,
    bindings: BTreeMap<String, String>,
}

enum ObservationKind {
    Live,
    Compiled { review_required: bool },
}

impl Observation {
    fn is_current(&self) -> bool {
        self.observed_at.is_some_and(|at| {
            matches!(self.kind, ObservationKind::Compiled { .. })
                || at.elapsed() <= MAX_OBSERVATION_AGE
        })
    }

    fn review_required(&self) -> bool {
        !matches!(
            self.kind,
            ObservationKind::Compiled {
                review_required: false
            }
        )
    }

    fn update(&mut self, definitions: Definitions) {
        self.bindings = definitions
            .iter()
            .map(|(tool, current)| {
                let binding = self
                    .definitions
                    .get(tool)
                    .filter(|previous| previous.digest == current.digest)
                    .and_then(|_| self.bindings.get(tool))
                    .cloned()
                    .unwrap_or_else(|| {
                        format!("{}:{:032x}", current.digest, rand::random::<u128>())
                    });
                (tool.clone(), binding)
            })
            .collect();
        self.definitions = definitions;
        self.observed_at = Some(Instant::now());
    }
}

#[derive(Default)]
struct PinState {
    observations: BTreeMap<String, Observation>,
    approved: ApprovedDefinitions,
    uncertain: PendingApprovals,
    visible: BTreeMap<String, BTreeMap<String, CapabilityMeta>>,
}

impl PinState {
    fn record_visibility(
        &mut self,
        server: &str,
        visible: BTreeMap<String, CapabilityMeta>,
        events: &CatalogEventStore,
    ) {
        let previous = self.visible.remove(server).unwrap_or_default();
        let removed = previous
            .keys()
            .filter(|id| !visible.contains_key(*id))
            .map(|id| (id, "removed"));
        let changes = visible
            .iter()
            .filter_map(|(id, meta)| match previous.get(id) {
                None => Some((id, "added")),
                Some(old) if old != meta => Some((id, "updated")),
                Some(_) => None,
            });
        events.record_capability_changes(removed.chain(changes));
        if !visible.is_empty() {
            self.visible.insert(server.to_owned(), visible);
        }
    }

    fn observed(&self, server: &str, tool: &str) -> Option<&Definition> {
        self.observations
            .get(server)
            .filter(|observation| observation.is_current())?
            .definitions
            .get(tool)
    }

    fn approved(&self, server: &str, tool: &str) -> Option<&Definition> {
        self.approved.get(server)?.get(tool)
    }

    fn binding(&self, server: &str, tool: &str) -> Result<Option<&str>, PinDenied> {
        let Some(observation) = self.observations.get(server) else {
            return Ok(None);
        };
        if observation.review_required() {
            let observed = self.observed(server, tool).ok_or(PinDenied)?;
            let approved = self.approved(server, tool).ok_or(PinDenied)?;
            if approved.digest != observed.digest
                || self.uncertain.iter().any(|(s, t)| s == server && t == tool)
            {
                return Err(PinDenied);
            }
        }
        let binding = observation.bindings.get(tool).ok_or(PinDenied)?;
        Ok(Some(binding))
    }

    fn review(&self, server: &str, tool: &str) -> PinReview {
        let current = self.observed(server, tool).cloned();
        let approved = self.approved(server, tool).cloned();
        let status = match (&current, &approved) {
            (None, _) => PinStatus::Unavailable,
            (Some(_), None) => PinStatus::Missing,
            (Some(_), Some(_)) if self.binding(server, tool).is_ok() => PinStatus::Matching,
            (Some(_), Some(_)) => PinStatus::Changed,
        };
        PinReview {
            server: server.to_owned(),
            tool: Some(tool.to_owned()),
            status,
            current,
            approved,
        }
    }
}

#[derive(Default)]
pub(crate) struct ToolPins {
    inner: RwLock<PinState>,
    storage: Option<AtomicFile<ApprovedDefinitions>>,
    pending_storage: Option<AtomicFile<PendingApprovals>>,
    approvals: Mutex<()>,
}

impl ToolPins {
    pub(crate) fn open(path: &Path) -> anyhow::Result<Self> {
        let storage = AtomicFile::new(path);
        let pending_storage = AtomicFile::new(path.with_extension("pending.json"));
        let approved = storage.load_opt()?.unwrap_or_default();
        let uncertain = pending_storage.load_opt()?.unwrap_or_default();
        Ok(Self {
            inner: RwLock::new(PinState {
                approved,
                uncertain,
                ..Default::default()
            }),
            storage: Some(storage),
            pending_storage: Some(pending_storage),
            approvals: Mutex::new(()),
        })
    }

    fn read(&self) -> RwLockReadGuard<'_, PinState> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, PinState> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn has_required_servers(&self) -> bool {
        self.read()
            .observations
            .values()
            .any(Observation::review_required)
    }

    pub(crate) fn has_storage(&self) -> bool {
        self.storage.is_some()
    }

    pub(crate) fn is_compiled_source(&self, server: &str) -> bool {
        self.read()
            .observations
            .get(server)
            .is_some_and(|observation| matches!(observation.kind, ObservationKind::Compiled { .. }))
    }

    pub(crate) fn matches_compiled_definitions(&self, server: &str, tools: &[Tool]) -> bool {
        let state = self.read();
        state.observations.get(server).is_some_and(|observation| {
            observation.definitions.len() == tools.len()
                && tools.iter().all(|tool| {
                    observation
                        .definitions
                        .get(tool.name.as_ref())
                        .is_some_and(|definition| {
                            definition.digest == Definition::from_tool(tool).digest
                        })
                })
        })
    }

    pub(crate) fn begin_session(
        &self,
        server: &str,
        required: bool,
        events: &CatalogEventStore,
    ) -> Option<PinSession> {
        let mut state = self.write();
        if !required {
            state.observations.remove(server);
            return None;
        }
        let session = PinSession(rand::random());
        state.record_visibility(server, BTreeMap::new(), events);
        state.observations.insert(
            server.to_owned(),
            Observation {
                session,
                epoch: 0,
                kind: ObservationKind::Live,
                observed_at: None,
                definitions: BTreeMap::new(),
                bindings: BTreeMap::new(),
            },
        );
        Some(session)
    }

    pub(crate) fn remove_server(&self, server: &str) {
        self.write().observations.remove(server);
    }

    pub(crate) fn publish_compiled(
        &self,
        server: &str,
        review_required: bool,
        revoke_stale: bool,
        tools: &[Tool],
    ) -> anyhow::Result<PinSession> {
        let definitions: Definitions = tools
            .iter()
            .map(|tool| (tool.name.to_string(), Definition::from_tool(tool)))
            .collect();
        let mut state = self.write();
        if revoke_stale {
            self.revoke_compiled(&mut state, server, &definitions)?;
        }
        let session = PinSession(rand::random());
        let kind = ObservationKind::Compiled { review_required };
        let item = state
            .observations
            .entry(server.to_owned())
            .or_insert_with(|| Observation {
                session,
                epoch: 0,
                kind: ObservationKind::Live,
                observed_at: None,
                definitions: Definitions::new(),
                bindings: BTreeMap::new(),
            });
        item.session = session;
        item.kind = kind;
        item.update(definitions);
        Ok(session)
    }

    fn revoke_compiled(
        &self,
        state: &mut PinState,
        server: &str,
        definitions: &Definitions,
    ) -> anyhow::Result<()> {
        let storage = self.storage.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Remote OpenAPI sources require persistent tool pin storage")
        })?;
        let pending_storage = self.pending_storage.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Remote OpenAPI sources require persistent tool pin storage")
        })?;
        let mut approved = state.approved.clone();
        if let Some(previous) = approved.get_mut(server) {
            let mut pending = state.uncertain.clone();
            let count = previous.len();
            previous.retain(|name, definition| {
                let unchanged = definitions
                    .get(name)
                    .is_some_and(|item| item.digest == definition.digest);
                if !unchanged {
                    pending.insert((server.to_owned(), name.clone()));
                }
                unchanged
            });
            if previous.len() != count {
                pending_storage.save(&pending)?;
                state.uncertain = pending;
                storage.save(&approved)?;
                state.approved = approved;
            }
        }
        Ok(())
    }

    pub(crate) fn withdraw_compiled(&self, server: &str) -> anyhow::Result<()> {
        let mut state = self.write();
        state.observations.remove(server);
        self.revoke_compiled(&mut state, server, &Definitions::new())
    }

    pub(crate) fn invalidate(
        &self,
        server: &str,
        session: PinSession,
        events: &CatalogEventStore,
    ) -> bool {
        let mut state = self.write();
        let Some(item) = state
            .observations
            .get_mut(server)
            .filter(|observation| observation.session == session)
        else {
            return false;
        };
        item.epoch = item.epoch.wrapping_add(1);
        let changed = item.observed_at.take().is_some();
        if changed {
            state.record_visibility(server, BTreeMap::new(), events);
        }
        changed
    }

    pub(crate) fn epoch(&self, server: &str, session: PinSession) -> Option<u64> {
        let state = self.read();
        state
            .observations
            .get(server)
            .filter(|item| item.session == session)
            .map(|item| item.epoch)
    }

    pub(crate) fn observe(
        &self,
        server: &str,
        session: PinSession,
        epoch: u64,
        tools: &[Tool],
        events: &CatalogEventStore,
    ) -> Option<bool> {
        let mut definitions = Definitions::new();
        for tool in tools {
            if definitions
                .insert(tool.name.to_string(), Definition::from_tool(tool))
                .is_some()
            {
                self.invalidate(server, session, events);
                return None;
            }
        }
        let mut state = self.write();
        let item = state.observations.get_mut(server)?;
        if item.session != session || item.epoch != epoch {
            return None;
        }
        let changed = !item.is_current() || !same_digests(&item.definitions, &definitions);
        item.update(definitions);
        Some(changed)
    }

    pub(crate) fn binding(&self, server: &str, tool: &str) -> Result<Option<String>, PinDenied> {
        self.read()
            .binding(server, tool)
            .map(|binding| binding.map(str::to_owned))
    }

    pub(crate) fn target_requires_pin(&self, target: &str) -> bool {
        self.read().observations.keys().any(|server| {
            target
                .strip_prefix(server)
                .is_some_and(|rest| rest.starts_with('.'))
        })
    }

    pub(crate) fn is_quarantined(&self, server: &str) -> bool {
        let state = self.read();
        state.observations.get(server).is_some_and(|observation| {
            !observation.is_current()
                || observation
                    .definitions
                    .keys()
                    .any(|tool| state.binding(server, tool).is_err())
        })
    }

    pub(crate) fn verify_dispatch(
        &self,
        server: &str,
        session: Option<PinSession>,
        tool: &str,
        expected: Option<&str>,
    ) -> Result<(), PinDenied> {
        let state = self.read();
        if state.observations.get(server).map(|item| item.session) != session
            || state.binding(server, tool)? != expected
        {
            return Err(PinDenied);
        }
        Ok(())
    }

    pub(crate) fn reviews(&self) -> Vec<PinReview> {
        let state = self.read();
        let mut result = Vec::new();
        for (server, item) in &state.observations {
            if !item.review_required() {
                continue;
            }
            let tools: BTreeSet<_> = item
                .definitions
                .keys()
                .chain(
                    state
                        .approved
                        .get(server)
                        .into_iter()
                        .flat_map(BTreeMap::keys),
                )
                .collect();
            if tools.is_empty() && !item.is_current() {
                result.push(PinReview {
                    server: server.clone(),
                    tool: None,
                    status: PinStatus::Unavailable,
                    current: None,
                    approved: None,
                });
            } else {
                result.extend(tools.into_iter().map(|tool| state.review(server, tool)));
            }
        }
        result
    }

    pub(crate) async fn approve(
        &self,
        app: &AppState,
        request: &ApprovePin,
        actor: Option<String>,
    ) -> Result<PinReview, PinApprovalError> {
        let _approval = self.approvals.lock().await;
        if self.review_candidate(request).is_none() {
            return Err(PinApprovalError::Stale);
        }
        let storage = self.storage.as_ref().ok_or(PinApprovalError::Storage)?;
        let pending_storage = self
            .pending_storage
            .as_ref()
            .ok_or(PinApprovalError::Storage)?;
        app.audit_handle
            .send_confirmed(pin_audit(request, actor))
            .await
            .map_err(|_| PinApprovalError::Audit)?;
        let mut state = self.write();
        let definition = reviewed_definition(&state, request)
            .cloned()
            .ok_or(PinApprovalError::Stale)?;
        let mut approved = state.approved.clone();
        approved
            .entry(request.server.clone())
            .or_default()
            .insert(request.tool.clone(), definition);
        let key = (request.server.clone(), request.tool.clone());
        let mut pending = state.uncertain.clone();
        pending.insert(key.clone());
        pending_storage
            .save(&pending)
            .map_err(|_| PinApprovalError::Storage)?;
        // The durable marker also quarantines a candidate visible after an uncertain write.
        state.uncertain = pending.clone();
        storage
            .save(&approved)
            .map_err(|_| PinApprovalError::Storage)?;
        pending.remove(&key);
        pending_storage
            .save(&pending)
            .map_err(|_| PinApprovalError::Storage)?;
        state.approved = approved;
        state.uncertain = pending;
        Ok(state.review(&request.server, &request.tool))
    }

    fn review_candidate(&self, request: &ApprovePin) -> Option<Definition> {
        reviewed_definition(&self.read(), request).cloned()
    }
}

fn reviewed_definition<'a>(state: &'a PinState, request: &ApprovePin) -> Option<&'a Definition> {
    state
        .observed(&request.server, &request.tool)
        .filter(|definition| definition.digest == request.digest)
}

fn same_digests(first: &Definitions, second: &Definitions) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .zip(second)
            .all(|((a, left), (b, right))| a == b && left.digest == right.digest)
}

fn pin_audit(request: &ApprovePin, actor_id: Option<String>) -> RawAuditEvent {
    RawAuditEvent {
        event_type: AuditEventType::ConfigMutation,
        trace_id: crate::engine::next_trace_id(),
        request_id: None,
        actor_id,
        work_item_id: None,
        client_ip: None,
        server_id: Some(request.server.clone()),
        capability_id: None,
        resource_uri: None,
        sanitized_args: Some(
            json!({"action": "authorize_tool_pin", "tool": request.tool, "digest": request.digest}),
        ),
        sanitized_response: None,
        execution_latency_us: None,
        status: AuditEventStatus::Success,
        error_code: None,
        error_message: None,
        operator_id: None,
        approval_ticket_id: None,
        idempotency_key: None,
        is_replay: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "manual pin gate latency measurement"]
    async fn pin_guard_latency() {
        use std::hint::black_box;

        let directory = tempfile::tempdir().unwrap();
        let pins = ToolPins::open(&directory.path().join("pins.json")).unwrap();
        let (session, request) = observed(&pins);
        let app = AppState::builder().build();
        pins.approve(&app, &request, None).await.unwrap();
        let binding = pins.binding("source", "write").unwrap().unwrap();
        for (server, session, expected) in [
            ("unpinned", None, None),
            ("source", Some(session), Some(binding.as_str())),
        ] {
            let (p50, p95) = crate::test_support::latency_percentiles(20_000, || {
                let time = std::time::Instant::now();
                black_box(pins.verify_dispatch(black_box(server), session, "write", expected))
                    .unwrap();
                time.elapsed()
            });
            eprintln!("dispatch guard {server}: p50={p50:?} p95={p95:?}");
        }
    }

    fn observed(pins: &ToolPins) -> (PinSession, ApprovePin) {
        let events = CatalogEventStore::new();
        let session = pins.begin_session("source", true, &events).unwrap();
        let tool: Tool = serde_json::from_value(json!({
            "name": "write",
            "description": "Fixture",
            "inputSchema": {"type": "object"}
        }))
        .unwrap();
        let digest = Definition::from_tool(&tool).digest;
        let epoch = pins.epoch("source", session).unwrap();
        assert_eq!(
            pins.observe("source", session, epoch, &[tool], &events),
            Some(true)
        );
        (
            session,
            ApprovePin {
                server: "source".to_owned(),
                tool: "write".to_owned(),
                digest,
            },
        )
    }

    #[test]
    fn canonical_digest_excludes_presentation_and_preserves_array_order() {
        let original: Tool = serde_json::from_value(json!({
            "name": "write",
            "title": "Original",
            "description": "Same",
            "inputSchema": {"type": "object", "required": ["a", "b"]},
            "outputSchema": {"type": "object"},
            "annotations": {"title": "Original", "readOnlyHint": false},
            "_meta": {"ui": {"resourceUri": "ui://old"}}
        }))
        .unwrap();
        let mut next = original.clone();
        next.title = Some("New title".to_owned());
        next.annotations.as_mut().unwrap().title = Some("Updated".to_owned());
        next.meta =
            Some(serde_json::from_value(json!({"ui": {"resourceUri": "ui://new"}})).unwrap());
        assert_eq!(
            Definition::from_tool(&original).digest,
            Definition::from_tool(&next).digest
        );
        next.input_schema = std::sync::Arc::new(
            serde_json::from_value(json!({
                "required": ["b", "a"], "type": "object"
            }))
            .unwrap(),
        );
        assert_ne!(
            Definition::from_tool(&original).digest,
            Definition::from_tool(&next).digest
        );
    }

    #[tokio::test]
    async fn pin_approval_stays_quarantined_if_audit_or_storage_fails() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("pins.json");
        let pins = ToolPins::open(&file).unwrap();
        let (session, request) = observed(&pins);
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        drop(receiver);
        let app = AppState::builder()
            .audit_handle(crate::audit::AuditHandle::new(sender))
            .build();
        assert!(matches!(
            pins.approve(&app, &request, None).await,
            Err(PinApprovalError::Audit)
        ));
        assert!(pins
            .verify_dispatch("source", Some(session), "write", None)
            .is_err());
        assert!(!file.exists());

        let app = AppState::builder().build();
        std::fs::create_dir(&file).unwrap();
        assert!(matches!(
            pins.approve(&app, &request, None).await,
            Err(PinApprovalError::Storage)
        ));
        assert!(pins.binding("source", "write").is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn pin_with_uncertain_directory_sync_stays_quarantined_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pins.json");
        let pins = ToolPins::open(&path).unwrap();
        let (_, request) = observed(&pins);
        pins.storage.as_ref().unwrap().fail_next_directory_sync();
        let app = AppState::builder().build();
        assert!(matches!(
            pins.approve(&app, &request, None).await,
            Err(PinApprovalError::Storage)
        ));
        assert!(pins.binding("source", "write").is_err());
        let pins = ToolPins::open(&path).unwrap();
        observed(&pins);
        assert!(pins.binding("source", "write").is_err());
        assert_eq!(
            pins.approve(&app, &request, None).await.unwrap().status,
            PinStatus::Matching
        );
    }
}
