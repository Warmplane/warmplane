// Rust guideline compliant 2026-11-20

//! Compiled result delivery policy and protocol-aware inspection.

use std::{collections::HashSet, sync::Arc};

use base64::{engine::general_purpose::STANDARD, Engine};
use regex::{RegexSet, RegexSetBuilder};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::{
    audit::{AuditEventStatus, AuditEventType, RawAuditEvent},
    context_filter::{distill_value, DistillationOptions},
    daemon::AppState,
    WarmplaneError,
};

pub(crate) const REDACTED: &str = "[REDACTED]";
const MAX_BYTES: usize = 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_PATTERNS: usize = 64;
const MAX_PATTERN_BYTES: usize = 4 * 1024;
const MAX_PATTERN_SET_BYTES: usize = 16 * 1024;
const UNINSPECTABLE: &str = "warmplane-uninspectable";

pub(crate) fn uninspectable_result() -> Value {
    json!({"resultType": UNINSPECTABLE})
}

pub(crate) fn is_uninspectable_result(value: &Value) -> bool {
    value.get("resultType").and_then(Value::as_str) == Some(UNINSPECTABLE)
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[derive(Default)]
struct Rules {
    field_names: Vec<String>,
    patterns: Vec<String>,
    opaque_exceptions: Vec<OpaqueException>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OpaqueException {
    pub(crate) server: String,
    pub(crate) kind: OpaqueKind,
    pub(crate) mime_type: String,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpaqueKind {
    Image,
    Audio,
    Blob,
}

struct CompiledRules {
    source: Rules,
    fields: HashSet<String>,
    patterns: RegexSet,
    revision: String,
}

/// Compiled global policy for upstream results.
///
/// Deserialization validates the policy and compiles its rules before the
/// configuration can be installed.
#[derive(Clone)]
pub struct ResultSanitization(Arc<CompiledRules>);

impl std::fmt::Debug for ResultSanitization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultSanitization")
            .field("revision", &self.0.revision)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ResultSanitization {
    fn eq(&self, other: &Self) -> bool {
        self.0.source == other.0.source
    }
}

impl Serialize for ResultSanitization {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.source.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ResultSanitization {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::compile(Rules::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Shape {
    Tool,
    ResourceRead,
    Prompt,
    Content,
    Resource,
    PromptMessage,
    Data,
    Essential,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Withheld;

impl From<Withheld> for crate::WarmplaneError {
    fn from(_: Withheld) -> Self {
        Self::new(
            "RESULT_WITHHELD",
            "Result withheld by result sanitization policy",
            false,
        )
    }
}

pub(crate) struct Delivery {
    pub(crate) value: Value,
    pub(crate) revision: Option<String>,
}

/// Tenant and profile provenance for stored results.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResultAccess {
    tenant_id: String,
    profile_id: Option<String>,
}

impl ResultAccess {
    pub(crate) fn new(tenant_id: &str, profile_id: Option<&str>) -> Self {
        Self {
            tenant_id: tenant_id.to_owned(),
            profile_id: profile_id.map(str::to_owned),
        }
    }

    pub(crate) async fn permits_capability(
        &self,
        state: &AppState,
        capability: &str,
        role: &str,
    ) -> bool {
        let Some(server) = state
            .capabilities
            .read()
            .await
            .get(capability)
            .map(|meta| meta.server.clone())
        else {
            return false;
        };
        let base = state.policy.read().await.clone();
        let role_policy = state.rbac_engine.compute_effective_policy(role, &base);
        let scoped_policy = if let Some(profile_id) = self.profile_id.as_deref() {
            let profiles = state.profiles.read().await;
            let Some(profile) = profiles.get(profile_id) else {
                return false;
            };
            if !profile.servers.contains(&server) {
                return false;
            }
            role_policy.merge_with_profile(Some(&crate::daemon::Policy::from_config(
                profile.policy.clone(),
            )))
        } else {
            role_policy
        };
        scoped_policy.allows(capability)
    }
}

impl ResultSanitization {
    fn compile(source: Rules) -> anyhow::Result<Self> {
        anyhow::ensure!(
            source.field_names.iter().all(|name| !name.is_empty()),
            "resultSanitization field names must be nonempty"
        );
        anyhow::ensure!(
            source.opaque_exceptions.iter().all(valid_exception),
            "resultSanitization opaque exceptions require a server and exact MIME type"
        );
        let patterns = compile_patterns(&source.patterns)?;
        let fields = source
            .field_names
            .iter()
            .map(|name| name.to_lowercase())
            .collect();
        let mut digest = Sha256::new();
        digest.update(b"warmplane-result-sanitization-v2\0");
        digest.update(serde_json::to_vec(&source)?);
        Ok(Self(Arc::new(CompiledRules {
            source,
            fields,
            patterns,
            revision: hex::encode(digest.finalize()),
        })))
    }

    pub(crate) fn revision(&self) -> &str {
        &self.0.revision
    }

    pub(crate) fn exception_servers(&self) -> impl Iterator<Item = &str> {
        self.0
            .source
            .opaque_exceptions
            .iter()
            .map(|entry| entry.server.as_str())
    }

    fn sanitize(
        &self,
        server: &str,
        mut value: Value,
        shape: Shape,
    ) -> Result<(Value, HashSet<OpaqueException>), Withheld> {
        let mut scan = Scan {
            rules: &self.0,
            server,
            remaining: MAX_BYTES,
            exceptions: HashSet::new(),
        };
        scan.visit(&mut value, shape, 0)?;
        Ok((value, scan.exceptions))
    }
}

impl AppState {
    pub(crate) async fn complete_tool_delivery(
        &self,
        server: &str,
        trace_id: &str,
        data: Value,
        task_id: Option<&str>,
        distillation: &DistillationOptions,
    ) -> (Result<Value, WarmplaneError>, Option<String>) {
        let result = self
            .sanitize_result(server, trace_id, data, Shape::Tool)
            .await;
        let Delivery { value, revision } = match result {
            Ok(delivery) => delivery,
            Err(withheld) => {
                let error = WarmplaneError::from(withheld);
                if let Some(task_id) = task_id {
                    self.task_registry
                        .fail_execution(task_id, &error.code, &error.message, "completed")
                        .await;
                }
                return (Err(error), None);
            }
        };
        let data = distill_value(value, distillation);
        let outcome = match task_id {
            Some(task_id) => {
                self.task_registry
                    .complete_execution(task_id, data, revision.clone())
                    .await
            }
            None => Ok(data),
        };
        (outcome, revision)
    }

    pub(crate) async fn sanitize_result(
        &self,
        server: &str,
        trace_id: &str,
        value: Value,
        shape: Shape,
    ) -> Result<Delivery, Withheld> {
        let rule = self.policy.read().await.result_sanitization.clone();
        let Some(rule) = rule else {
            if is_uninspectable_result(&value) {
                return Err(Withheld);
            }
            return Ok(Delivery {
                value,
                revision: None,
            });
        };
        let upstream_outcome = if value.get("isError").and_then(Value::as_bool) == Some(true) {
            "tool_error"
        } else {
            "completed"
        };
        let inspection = rule.sanitize(server, value, shape);
        match inspection {
            Ok((value, exceptions)) => {
                if !exceptions.is_empty() {
                    self.audit_handle.send_confirmed(result_audit(
                        server,
                        trace_id,
                        json!({"opaqueExceptions": exceptions, "policyRevision": rule.revision()}),
                        None,
                    )).await.map_err(|_| Withheld)?;
                }
                Ok(Delivery {
                    value,
                    revision: Some(rule.revision().to_owned()),
                })
            }
            Err(withheld) => {
                self.audit_handle.send(result_audit(
                    server,
                    trace_id,
                    json!({"upstreamOutcome": upstream_outcome, "policyRevision": rule.revision()}),
                    Some("RESULT_WITHHELD"),
                ));
                Err(withheld)
            }
        }
    }

    pub(crate) async fn safe_upstream_error(&self, message: String) -> String {
        if self.policy.read().await.result_sanitization.is_some() {
            "Upstream call failed".to_owned()
        } else {
            message
        }
    }

    pub(crate) async fn accepts_result_revision(&self, revision: Option<&str>) -> bool {
        let policy = self.policy.read().await;
        policy
            .result_sanitization
            .as_ref()
            .is_none_or(|rule| revision == Some(rule.revision()))
    }

    pub(crate) async fn validate_replay(
        &self,
        key: &str,
        capability: &str,
        response: &Value,
        access: &ResultAccess,
        role: &str,
    ) -> Result<(), WarmplaneError> {
        if !self.trust_controls_active().await {
            return Ok(());
        }
        let record = self.idempotency_store.get_record(key).await;
        let Some(record) = record.filter(|record| {
            record.response == *response && record.capability_id.as_deref() == Some(capability)
        }) else {
            return Err(Withheld.into());
        };
        if record.result_access.as_ref() != Some(access)
            || !access.permits_capability(self, capability, role).await
        {
            return Err(WarmplaneError::new(
                "POLICY_DENIED",
                "Stored result is outside your access scope",
                false,
            ));
        }
        if !self
            .accepts_result_revision(record.result_policy_revision.as_deref())
            .await
        {
            return Err(Withheld.into());
        }
        Ok(())
    }

    pub(crate) async fn task_view(
        &self,
        record: &crate::tasks::TaskRecord,
    ) -> crate::tasks::TaskResponse {
        use crate::tasks::{TaskResponse, TaskStatus};

        let mut response = TaskResponse::from(record);
        if matches!(record.status, TaskStatus::Completed | TaskStatus::Failed)
            && !self
                .accepts_result_revision(record.result_policy_revision.as_deref())
                .await
        {
            let withheld = WarmplaneError::from(Withheld);
            response.result = None;
            response.error = Some(json!({"code": withheld.code, "message": &withheld.message}));
            response.status_message = Some(withheld.message);
        }
        response
    }
}

fn result_audit(
    server: &str,
    trace_id: &str,
    metadata: Value,
    code: Option<&str>,
) -> RawAuditEvent {
    RawAuditEvent {
        event_type: AuditEventType::ResultSanitization,
        trace_id: trace_id.to_owned(),
        request_id: None,
        actor_id: None,
        work_item_id: None,
        client_ip: None,
        server_id: Some(server.to_owned()),
        capability_id: None,
        resource_uri: None,
        sanitized_args: None,
        sanitized_response: Some(metadata),
        execution_latency_us: None,
        status: if code.is_some() {
            AuditEventStatus::Denied
        } else {
            AuditEventStatus::Success
        },
        error_code: code.map(str::to_owned),
        error_message: None,
        operator_id: None,
        approval_ticket_id: None,
        idempotency_key: None,
        is_replay: None,
    }
}

fn valid_exception(entry: &OpaqueException) -> bool {
    !entry.server.trim().is_empty()
        && !entry.mime_type.contains(['*', ' '])
        && entry
            .mime_type
            .split_once('/')
            .is_some_and(|(kind, subtype)| {
                !kind.is_empty()
                    && !subtype.is_empty()
                    && !subtype.contains('/')
                    && match entry.kind {
                        OpaqueKind::Image => kind == "image",
                        OpaqueKind::Audio => kind == "audio",
                        OpaqueKind::Blob => true,
                    }
            })
}

fn compile_patterns(patterns: &[String]) -> anyhow::Result<RegexSet> {
    anyhow::ensure!(
        patterns.len() <= MAX_PATTERNS
            && patterns
                .iter()
                .all(|pattern| pattern.len() <= MAX_PATTERN_BYTES)
            && patterns.iter().map(String::len).sum::<usize>() <= MAX_PATTERN_SET_BYTES,
        "resultSanitization permits up to 64 regular expressions, 4 KiB each and 16 KiB total"
    );
    RegexSetBuilder::new(patterns)
        .nest_limit(32)
        .size_limit(256 * 1024)
        .dfa_size_limit(512 * 1024)
        .build()
        .map_err(|_| {
            anyhow::anyhow!("resultSanitization contains invalid or oversized regular expressions")
        })
}

struct Scan<'a> {
    rules: &'a CompiledRules,
    server: &'a str,
    remaining: usize,
    exceptions: HashSet<OpaqueException>,
}

impl Scan<'_> {
    fn charge(&mut self, bytes: usize) -> Result<(), Withheld> {
        self.remaining = self.remaining.checked_sub(bytes).ok_or(Withheld)?;
        Ok(())
    }

    fn visit(&mut self, value: &mut Value, shape: Shape, depth: usize) -> Result<(), Withheld> {
        if depth > MAX_DEPTH {
            return Err(Withheld);
        }
        self.charge(1)?;
        match value {
            Value::Object(fields) => self.object(fields, shape, depth),
            Value::Array(values) => values
                .iter_mut()
                .try_for_each(|item| self.visit(item, shape, depth + 1)),
            Value::String(text) => self.text(text, shape, depth),
            _ => Ok(()),
        }
    }

    fn text(&mut self, text: &mut String, shape: Shape, depth: usize) -> Result<(), Withheld> {
        self.charge(text.len())?;
        if matches!(shape, Shape::Essential)
            && text
                .get(..5)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            return Err(Withheld);
        }
        if matches!(shape, Shape::Data) && text.trim_start().starts_with(['{', '[']) {
            if let Some(mut json) = parse_embedded_json(text, MAX_DEPTH - depth)? {
                return self.embedded_json(text, &mut json, depth);
            }
        }
        if self.rules.patterns.is_match(text) {
            if matches!(shape, Shape::Essential) {
                return Err(Withheld);
            }
            *text = REDACTED.to_owned();
        }
        Ok(())
    }

    fn embedded_json(
        &mut self,
        text: &mut String,
        json: &mut Value,
        depth: usize,
    ) -> Result<(), Withheld> {
        self.visit(json, Shape::Data, depth + 1)?;
        // Also check the original representation for patterns spanning JSON fields.
        if self.rules.patterns.is_match(text) {
            *text = REDACTED.to_owned();
        } else if contains_redaction(json) {
            *text = serde_json::to_string(json).map_err(|_| Withheld)?;
        }
        Ok(())
    }

    fn object(
        &mut self,
        fields: &mut Map<String, Value>,
        shape: Shape,
        depth: usize,
    ) -> Result<(), Withheld> {
        let opaque = opaque_field(fields, shape)?;
        if let Some((key, kind)) = opaque {
            self.check_opaque(fields, key, kind)?;
        }
        for (key, value) in fields {
            self.charge(key.len())?;
            if opaque.is_some_and(|(field, _)| field == key) {
                continue;
            }
            let child = shape.child(key);
            if self.rules.patterns.is_match(key) {
                return Err(Withheld);
            }
            let redact = self.rules.fields.contains(&key.to_lowercase());
            if redact && !matches!(child, Shape::Data) {
                return Err(Withheld);
            }
            self.visit(value, child, depth + 1)?;
            if redact {
                *value = Value::String(REDACTED.to_owned());
            }
        }
        Ok(())
    }

    fn check_opaque(
        &mut self,
        fields: &Map<String, Value>,
        key: &str,
        kind: OpaqueKind,
    ) -> Result<(), Withheld> {
        let mime = fields
            .get("mimeType")
            .and_then(Value::as_str)
            .ok_or(Withheld)?;
        let data = fields.get(key).and_then(Value::as_str).ok_or(Withheld)?;
        let exception = self
            .rules
            .source
            .opaque_exceptions
            .iter()
            .find(|entry| {
                entry.server == self.server && entry.kind == kind && entry.mime_type == mime
            })
            .ok_or(Withheld)?;
        if self.rules.fields.contains(key) || self.rules.patterns.is_match(key) {
            return Err(Withheld);
        }
        // Reject oversized encodings before allocating a decode buffer.
        if data.len() > self.remaining.div_ceil(3) * 4 {
            return Err(Withheld);
        }
        self.charge(STANDARD.decode(data).map_err(|_| Withheld)?.len())?;
        self.exceptions.insert(exception.clone());
        Ok(())
    }
}

fn parse_embedded_json(text: &str, max_depth: usize) -> Result<Option<Value>, Withheld> {
    match serde_json::from_str(text) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.is_syntax() && exceeds_json_depth(text, max_depth) => Err(Withheld),
        Err(_) => Ok(None),
    }
}

fn exceeds_json_depth(text: &str, max_depth: usize) -> bool {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut bytes = text.bytes();
    while let Some(byte) = bytes.next() {
        match byte {
            b'"' => quoted = !quoted,
            b'\\' if quoted => {
                bytes.next();
            }
            b'{' | b'[' if !quoted => {
                depth += 1;
                if depth > max_depth {
                    return true;
                }
            }
            b'}' | b']' if !quoted => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}

impl Shape {
    fn child(self, key: &str) -> Self {
        match (self, key) {
            (Self::Data, _)
            | (Self::Tool, "structuredContent")
            | (Self::Resource | Self::Content, "text")
            | (Self::Prompt | Self::Content, "description" | "title")
            | (
                Self::Tool | Self::ResourceRead | Self::Prompt | Self::Resource | Self::Content,
                "_meta",
            ) => Self::Data,
            (Self::Tool | Self::PromptMessage, "content") => Self::Content,
            (Self::Content, "resource") | (Self::ResourceRead, "contents") => Self::Resource,
            (Self::Prompt, "messages") => Self::PromptMessage,
            _ => Self::Essential,
        }
    }
}

fn opaque_field(
    fields: &Map<String, Value>,
    shape: Shape,
) -> Result<Option<(&'static str, OpaqueKind)>, Withheld> {
    if matches!(shape, Shape::Tool | Shape::Prompt | Shape::ResourceRead)
        && fields
            .get("resultType")
            .and_then(Value::as_str)
            .is_some_and(|kind| !matches!(kind, "complete" | "input_required" | "task"))
    {
        return Err(Withheld);
    }
    match shape {
        Shape::Content => match fields.get("type").and_then(Value::as_str) {
            Some("text" | "resource" | "resource_link") => Ok(None),
            Some("image") => Ok(Some(("data", OpaqueKind::Image))),
            Some("audio") => Ok(Some(("data", OpaqueKind::Audio))),
            _ => Err(Withheld),
        },
        Shape::Resource if fields.contains_key("blob") => Ok(Some(("blob", OpaqueKind::Blob))),
        Shape::Resource if !fields.contains_key("text") => Err(Withheld),
        Shape::Tool | Shape::Prompt | Shape::ResourceRead
            if fields.contains_key("requestState")
                || fields.get("isError").and_then(Value::as_bool) == Some(true) =>
        {
            Err(Withheld)
        }
        _ => Ok(None),
    }
}

pub(crate) fn contains_redaction(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains(REDACTED),
        Value::Array(items) => items.iter().any(contains_redaction),
        Value::Object(fields) => fields.values().any(contains_redaction),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "manual latency measurement"]
    fn result_sanitization_latency() {
        use std::{hint::black_box, time::Instant};

        let rule: ResultSanitization = serde_json::from_value(json!({
            "fieldNames": ["privateNote"],
            "patterns": ["customer-[0-9]+"],
        }))
        .unwrap();
        let text = json!({"content": [{"type": "text", "text": "output line with context\n".repeat(2400)}]});
        let rows = (0..256)
            .map(|row| json!({"id": row, "privateNote": "sensitive", "label": "customer-123"}))
            .collect::<Vec<_>>();
        let structured = json!({"structuredContent": {"rows": rows}});
        for (label, sample) in [("text", text), ("structured", structured)] {
            let (p50, p95) = crate::test_support::latency_percentiles(1_000, || {
                let value = sample.clone();
                let start = Instant::now();
                black_box(rule.sanitize("bench", value, Shape::Tool).unwrap());
                start.elapsed()
            });
            eprintln!("sanitization {label}: p50={p50:?} p95={p95:?}");
        }
    }

    #[test]
    fn compile_rejects_invalid_rules() {
        for invalid in [
            json!({"patterns": ["("]}),
            json!({"fieldNames": [""]}),
            json!({"opaqueExceptions": [{"server": "a", "kind": "image", "mimeType": "image/*"}]}),
            json!({"opaqueExceptions": [{"server": "a", "kind": "document", "mimeType": "application/pdf"}]}),
        ] {
            assert!(serde_json::from_value::<ResultSanitization>(invalid).is_err());
        }
    }

    #[test]
    fn regular_expression_limits_are_applied_before_activation() {
        for patterns in [
            vec!["abc".to_owned(); MAX_PATTERNS + 1],
            vec!["a".repeat(MAX_PATTERN_BYTES + 1)],
            vec!["a".repeat(MAX_PATTERN_BYTES); 1 + MAX_PATTERN_SET_BYTES / MAX_PATTERN_BYTES],
            vec![format!("{}a{}", "(?:".repeat(33), ")".repeat(33))],
        ] {
            assert!(compile_patterns(&patterns).is_err());
        }
        let pattern = compile_patterns(&["(?i)\\bexample\\b".to_owned()]).unwrap();
        assert!(pattern.is_match("EXAMPLE"));
    }

    #[test]
    fn embedded_json_shares_the_budget_and_rejects_deep_json() {
        let rules: ResultSanitization =
            serde_json::from_value(json!({"fieldNames": ["private"]})).unwrap();
        let deep = format!("{}0{}", "[".repeat(160), "]".repeat(160));
        let large = json!({"label": "a".repeat(MAX_BYTES / 3)}).to_string();
        for value in [json!([deep]), json!([large.clone(), large])] {
            assert!(rules.sanitize("test", value, Shape::Data).is_err());
        }
        let brackets = "[".repeat(MAX_DEPTH + 1);
        let quoted = json!({"label": brackets, "private": "removed"}).to_string();
        let (result, _) = rules.sanitize("test", json!(quoted), Shape::Data).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(result.as_str().unwrap()).unwrap(),
            json!({"label": brackets, "private": REDACTED})
        );
    }

    #[test]
    fn opaque_exception_requires_exact_type_mime_and_server() {
        let rule: ResultSanitization = serde_json::from_value(json!({
            "opaqueExceptions": [{"server": "reports", "kind": "image", "mimeType": "image/png"}]
        }))
        .unwrap();
        let image =
            json!({"content": [{"type": "image", "mimeType": "image/png", "data": "YQ=="}]});
        let (result, exceptions) = rule
            .sanitize("reports", image.clone(), Shape::Tool)
            .unwrap();
        assert_eq!(result, image);
        assert_eq!(exceptions.len(), 1);
        assert!(rule
            .sanitize("uploads", image.clone(), Shape::Tool)
            .is_err());

        for (kind, mime) in [
            ("image", "image/PNG"),
            ("audio", "image/png"),
            ("unknown", "image/png"),
        ] {
            let value = json!({"content": [{"type": kind, "mimeType": mime, "data": "YQ=="}]});
            assert!(rule.sanitize("reports", value, Shape::Tool).is_err());
        }
    }

    #[test]
    fn inspection_enforces_size_depth_and_essential_fields() {
        let rule: ResultSanitization =
            serde_json::from_value(json!({"patterns": ["confidential"]})).unwrap();
        let uri = json!({"contents": [{"uri": "file:///confidential", "text": "safe"}]});
        assert!(rule.sanitize("reports", uri, Shape::ResourceRead).is_err());
        let data_uri = json!({"content": [{"type": "resource_link", "uri": "data:application/pdf;base64,YQ==", "name": "test"}]});
        assert!(rule.sanitize("reports", data_uri, Shape::Tool).is_err());
        let error =
            json!({"content": [{"type": "text", "text": "any upstream error"}], "isError": true});
        assert!(rule.sanitize("reports", error, Shape::Tool).is_err());
        let large = json!({"structuredContent": "a".repeat(MAX_BYTES)});
        assert!(rule.sanitize("reports", large, Shape::Tool).is_err());
        let deep = (0..=MAX_DEPTH).fold(Value::Null, |value, _| json!([value]));
        assert!(rule
            .sanitize("reports", json!({"structuredContent": deep}), Shape::Tool)
            .is_err());
    }
}
