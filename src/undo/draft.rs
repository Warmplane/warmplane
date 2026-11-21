// Rust guideline compliant 2026-11-21

//! Builds reviewable reversibility contracts without editing active configuration.

pub(crate) mod record;

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{ensure, Context, Result};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{
    config::{
        InverseConfig, PreSnapshotConfig, ReversibilityClass, ReversibilityConfig,
        UndoTestTargetConfig,
    },
    daemon::{AppState, CapabilityMeta},
    http_v1::config_api::{trust_admin_required, TrustAdmin},
};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DraftRequest {
    forward: String,
    ttl_secs: u64,
    pre_snapshot: PreSnapshotConfig,
    inverse: InverseConfig,
    pre_undo_check: PreSnapshotConfig,
    test_target: UndoTestTargetConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Draft {
    forward: String,
    contract: ReversibilityConfig,
    revisions: BTreeMap<String, String>,
    dynamic_mappings: BTreeSet<String>,
    requires_test: bool,
}

pub(crate) async fn handle_generate(
    State(state): State<AppState>,
    admin: Option<Extension<TrustAdmin>>,
    Extension(tenant): Extension<Option<crate::rbac::TenantContext>>,
    profile: Option<Extension<crate::context::ProfileContext>>,
    Json(request): Json<DraftRequest>,
) -> Response {
    if !admin.is_some_and(|Extension(TrustAdmin(allowed))| allowed) {
        return trust_admin_required();
    }
    match record::preview(
        &state,
        request,
        tenant.as_ref(),
        profile.as_ref().map(|extension| &extension.0),
    )
    .await
    {
        Ok(draft) => Json(draft).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "ok": false,
                "error": {"code": "INVALID_REVERSIBILITY_DRAFT", "message": error.to_string()}
            })),
        )
            .into_response(),
    }
}

async fn generate(state: &AppState, request: DraftRequest) -> Result<Draft> {
    let entries = state.tool_catalog().await;
    let find = |id: &str| {
        entries
            .items
            .get(id)
            .with_context(|| format!("Capability '{id}' is unavailable"))
    };
    let forward = find(&request.forward)?;
    let before = find(&request.pre_snapshot.capability)?;
    let inverse = find(&request.inverse.capability)?;
    let check = find(&request.pre_undo_check.capability)?;
    let mappings = validate_mappings(&request, forward, before, inverse, check)?;

    let revisions = [
        (&request.forward, forward),
        (&request.pre_snapshot.capability, before),
        (&request.inverse.capability, inverse),
        (&request.pre_undo_check.capability, check),
    ]
    .into_iter()
    .map(|(id, cap)| Ok((id.clone(), revision(state, cap)?)))
    .collect::<Result<BTreeMap<_, _>>>()?;
    let contract = ReversibilityConfig {
        class: ReversibilityClass::Reversible,
        hold_window_secs: None,
        ttl_secs: Some(request.ttl_secs),
        daily_limit: None,
        pre_snapshot: Some(Box::new(request.pre_snapshot)),
        pre_undo_check: Some(Box::new(request.pre_undo_check)),
        test_target: Some(Box::new(request.test_target)),
        inverse: Some(Box::new(request.inverse)),
    };
    crate::config::validate_reversibility_contract(&request.forward, &contract)?;
    Ok(Draft {
        forward: request.forward,
        contract,
        revisions,
        dynamic_mappings: mappings,
        requires_test: true,
    })
}

fn validate_mappings(
    request: &DraftRequest,
    forward: &CapabilityMeta,
    before: &CapabilityMeta,
    inverse: &CapabilityMeta,
    check: &CapabilityMeta,
) -> Result<BTreeSet<String>> {
    ensure!(
        [forward, before, inverse, check]
            .iter()
            .all(|cap| cap.server != request.test_target.server),
        "Reversibility test target must be distinct from the selected source servers"
    );
    let mut mappings = BTreeSet::new();
    validate_mapping(
        &request.test_target.args,
        &forward.input_schema,
        &forward.input_schema,
        false,
        &mut mappings,
    )?;
    ensure!(
        mappings.is_empty(),
        "Reversibility test arguments must be fixed values"
    );
    for (value, cap, inverse_mapping) in [
        (&request.pre_snapshot.args, before, false),
        (&request.pre_undo_check.args, check, false),
        (&request.inverse.args, inverse, true),
    ] {
        validate_mapping(
            value,
            &cap.input_schema,
            &forward.input_schema,
            inverse_mapping,
            &mut mappings,
        )?;
    }
    Ok(mappings)
}

fn revision(state: &AppState, capability: &CapabilityMeta) -> Result<String> {
    let binding = state
        .tool_pins
        .binding(&capability.server, &capability.tool)
        .ok()
        .context("Capability definition changed while reviewing the draft")?;
    let fields = json!({
        "server": capability.server, "tool": capability.tool, "binding": binding,
        "schema": capability.input_schema, "description": capability.description
    });
    let bytes = serde_json::to_vec(&fields)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn validate_mapping(
    value: &Value,
    schema: &Value,
    forward_schema: &Value,
    allow_inverse_sources: bool,
    dynamic: &mut BTreeSet<String>,
) -> Result<()> {
    if let Some(text) = value.as_str().filter(|text| text.contains("${")) {
        return validate_reference(text, schema, forward_schema, allow_inverse_sources, dynamic);
    }
    if let Some(kind) = schema["type"].as_str() {
        ensure!(
            matching_type(value, kind),
            "Reversibility mapping literal does not match the selected capability schema"
        );
    }
    match value {
        Value::Object(fields) => {
            validate_fields(
                fields,
                schema,
                forward_schema,
                allow_inverse_sources,
                dynamic,
            )?;
        }
        Value::Array(values) => {
            for item in values {
                validate_mapping(
                    item,
                    &schema["items"],
                    forward_schema,
                    allow_inverse_sources,
                    dynamic,
                )?;
            }
        }
        _ => {}
    }
    if let Some(choices) = schema["enum"].as_array() {
        ensure!(
            choices.contains(value),
            "Reversibility mapping is outside the selected capability enum"
        );
    }
    Ok(())
}

fn validate_fields(
    fields: &serde_json::Map<String, Value>,
    schema: &Value,
    forward_schema: &Value,
    allow_inverse_sources: bool,
    dynamic: &mut BTreeSet<String>,
) -> Result<()> {
    validate_required(fields, schema)?;
    for (name, field) in fields {
        let property = schema["properties"].get(name);
        ensure!(
            property.is_some() || schema["additionalProperties"] != false,
            "Reversibility mapping contains an unknown property"
        );
        validate_mapping(
            field,
            property.unwrap_or(&Value::Null),
            forward_schema,
            allow_inverse_sources,
            dynamic,
        )?;
    }
    Ok(())
}

fn validate_required(fields: &serde_json::Map<String, Value>, schema: &Value) -> Result<()> {
    if let Some(required) = schema["required"].as_array() {
        ensure!(
            required
                .iter()
                .all(|name| name.as_str().is_some_and(|name| fields.contains_key(name))),
            "Reversibility mapping is missing a required argument"
        );
    }
    Ok(())
}

fn matching_type(value: &Value, kind: &str) -> bool {
    match kind {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn validate_reference(
    text: &str,
    schema: &Value,
    forward_schema: &Value,
    allow_inverse_sources: bool,
    dynamic: &mut BTreeSet<String>,
) -> Result<()> {
    let reference = text
        .strip_prefix("${")
        .and_then(|text| text.strip_suffix('}'))
        .filter(|text| !text.contains(['{', '}']))
        .context("Generated drafts use a single typed template reference per mapped argument")?;
    let (root, path) = reference.split_once('.').unwrap_or((reference, ""));
    ensure!(
        root == "args" || (allow_inverse_sources && matches!(root, "result" | "snapshot")),
        "Reversibility mapping uses a template source unavailable at dispatch"
    );
    if root == "args" {
        let input = argument_schema(forward_schema, path)?;
        if let (Some(source), Some(target)) = (input["type"].as_str(), schema["type"].as_str()) {
            ensure!(
                source == target || (source == "integer" && target == "number"),
                "Reversibility mapping uses an incompatible forward argument type"
            );
        }
    }
    dynamic.insert(reference.to_owned());
    Ok(())
}

fn argument_schema<'a>(mut schema: &'a Value, path: &str) -> Result<&'a Value> {
    if path.is_empty() {
        return Ok(schema);
    }
    for part in path.split('.') {
        ensure!(!part.is_empty(), "Invalid forward argument reference");
        schema = if part.parse::<usize>().is_ok() && schema["type"] == "array" {
            &schema["items"]
        } else {
            schema["properties"]
                .get(part)
                .context("Unknown forward argument reference")?
        };
    }
    Ok(schema)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn fixture_schema(field: &str, kind: &str) -> Value {
        json!({
            "type":"object",
            "properties":{field:{"type":kind}},
            "required":[field],
            "additionalProperties":false
        })
    }

    fn request() -> DraftRequest {
        serde_json::from_value(json!({
            "forward":"live.update",
            "ttlSecs":60,
            "preSnapshot":{"capability":"live.read","args":{"id":"${args.id}"}},
            "inverse":{"capability":"live.restore","args":{"before":"${snapshot}"}},
            "preUndoCheck":{"capability":"live.read","args":{"id":"${args.id}"}},
            "testTarget":{"server":"test","args":{"id":"fixture-1"}}
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn draft_generation_preserves_config_and_rejects_incompatible_templates() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("mcp_servers.json");
        std::fs::write(&config, b"{}\n").unwrap();
        let capabilities = HashMap::from([
            (
                "live.update".to_owned(),
                CapabilityMeta::new(
                    "live",
                    "update",
                    "Update",
                    "Update",
                    fixture_schema("id", "string"),
                ),
            ),
            (
                "live.read".to_owned(),
                CapabilityMeta::new(
                    "live",
                    "read",
                    "Read",
                    "Read",
                    fixture_schema("id", "string"),
                ),
            ),
            (
                "live.restore".to_owned(),
                CapabilityMeta::new(
                    "live",
                    "restore",
                    "Restore",
                    "Restore",
                    fixture_schema("before", "object"),
                ),
            ),
        ]);
        let state = AppState::builder()
            .config_path(config.to_string_lossy())
            .capabilities(capabilities)
            .build();
        let draft = generate(&state, request()).await.unwrap();
        assert!(draft.requires_test);
        assert_eq!(draft.revisions.len(), 3);
        assert_eq!(draft.contract.ttl_secs, Some(60));
        assert_eq!(
            draft.dynamic_mappings,
            BTreeSet::from(["args.id".into(), "snapshot".into()])
        );

        let mut invalid = request();
        invalid.inverse.args = json!({"before":"${args.id}"});
        assert!(generate(&state, invalid).await.is_err());
        let mut invalid = request();
        invalid.test_target.args = json!({"id":"${args.id}"});
        assert!(generate(&state, invalid).await.is_err());
        assert_eq!(std::fs::read(config).unwrap(), b"{}\n");
    }
}
