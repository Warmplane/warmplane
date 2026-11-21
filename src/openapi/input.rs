// Rust guideline compliant 2026-11-21

//! Validates the supported JSON request body contract and advertises its input schema.

use std::collections::BTreeSet;

use anyhow::{ensure, Context, Result};
use reqwest::Method;
use serde_json::{json, Map, Value};

use super::parameter::{self, Parameter};

#[derive(Debug)]
pub(super) struct RequestBody {
    pub(super) required: bool,
    schema: Map<String, Value>,
}

impl RequestBody {
    pub(super) fn parse(value: Option<&Value>, method: &Method) -> Result<Option<Self>> {
        let Some(value) = value else {
            return Ok(None);
        };
        ensure!(
            method != Method::GET,
            "OpenAPI GET request bodies are unsupported"
        );
        let content = value["content"]
            .as_object()
            .context("Invalid OpenAPI request content")?;
        ensure!(
            content.len() == 1 && content.contains_key("application/json"),
            "OpenAPI request bodies must use application/json"
        );
        let schema = body_schema(&content["application/json"]["schema"])?;
        let required = value.get("required").map_or(Ok(false), |required| {
            required
                .as_bool()
                .context("OpenAPI request body required flag must be Boolean")
        })?;
        Ok(Some(Self { required, schema }))
    }

    pub(super) fn matches(&self, value: &Value) -> bool {
        matches_body(value, &self.schema)
    }
}

fn body_schema(value: &Value) -> Result<Map<String, Value>> {
    let kind = value["type"].as_str();
    match kind {
        Some("object") => body_object_schema(value),
        Some("array") => {
            supported_keys(value, &["items", "minItems", "maxItems"])?;
            let mut schema = value.as_object().cloned().unwrap_or_default();
            schema.insert(
                "items".to_owned(),
                Value::Object(body_schema(&value["items"])?),
            );
            for field in ["minItems", "maxItems"] {
                ensure!(
                    value
                        .get(field)
                        .is_none_or(|limit| limit.as_u64().is_some()),
                    "OpenAPI array size limit must be a nonnegative integer"
                );
            }
            Ok(schema)
        }
        _ => parameter::scalar_schema(value),
    }
}

fn body_object_schema(value: &Value) -> Result<Map<String, Value>> {
    supported_keys(value, &["properties", "required", "additionalProperties"])?;
    ensure!(
        value["additionalProperties"] == false,
        "OpenAPI request objects must explicitly disallow additional properties"
    );
    let fields = value["properties"]
        .as_object()
        .context("OpenAPI request object requires properties")?;
    let properties = fields
        .iter()
        .map(|(name, value)| Ok((name.to_owned(), Value::Object(body_schema(value)?))))
        .collect::<Result<Map<String, Value>>>()?;
    let mut required = BTreeSet::new();
    if let Some(value) = value.get("required") {
        for name in value
            .as_array()
            .context("OpenAPI required properties must be an array")?
        {
            let name = name
                .as_str()
                .context("OpenAPI required property must be a string")?;
            ensure!(
                properties.contains_key(name) && required.insert(name),
                "Unknown or duplicate required OpenAPI body property"
            );
        }
    }
    Ok(object_schema(properties, json!(required))
        .as_object()
        .cloned()
        .unwrap_or_default())
}

fn supported_keys(value: &Value, fields: &[&str]) -> Result<()> {
    let schema = value
        .as_object()
        .context("Invalid OpenAPI request body schema")?;
    ensure!(
        schema.keys().all(|key| {
            matches!(
                key.as_str(),
                "type" | "title" | "description" | "example" | "examples"
            ) || fields.contains(&key.as_str())
        }),
        "Unsupported OpenAPI request body constraint"
    );
    Ok(())
}

fn matches_body(value: &Value, schema: &Map<String, Value>) -> bool {
    match schema.get("type").and_then(Value::as_str) {
        Some("object") => value
            .as_object()
            .is_some_and(|value| matches_object(value, schema)),
        Some("array") => value
            .as_array()
            .is_some_and(|value| matches_array(value, schema)),
        _ => parameter::matches_schema(value, schema),
    }
}

fn matches_object(value: &Map<String, Value>, schema: &Map<String, Value>) -> bool {
    schema
        .get("required")
        .and_then(Value::as_array)
        .is_some_and(|required| {
            required
                .iter()
                .all(|name| name.as_str().is_some_and(|name| value.contains_key(name)))
        })
        && schema
            .get("properties")
            .and_then(Value::as_object)
            .is_some_and(|properties| {
                value.iter().all(|(name, field)| {
                    properties
                        .get(name)
                        .and_then(Value::as_object)
                        .is_some_and(|schema| matches_body(field, schema))
                })
            })
}

fn matches_array(value: &[Value], schema: &Map<String, Value>) -> bool {
    let len = value.len() as u64;
    len >= schema.get("minItems").and_then(Value::as_u64).unwrap_or(0)
        && len
            <= schema
                .get("maxItems")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
        && schema
            .get("items")
            .and_then(Value::as_object)
            .is_some_and(|items| value.iter().all(|value| matches_body(value, items)))
}

pub(super) fn operation_schema(
    parameters: &[Parameter],
    body: Option<&RequestBody>,
) -> Map<String, Value> {
    let mut schema = parameter::input_schema(parameters);
    if let Some(body) = body {
        if let Some(fields) = schema.get_mut("properties").and_then(Value::as_object_mut) {
            fields.insert("body".to_owned(), Value::Object(body.schema.clone()));
        }
        if body.required {
            if let Some(required) = schema.get_mut("required").and_then(Value::as_array_mut) {
                required.push(json!("body"));
            }
        }
    }
    schema
}

pub(super) fn object_schema(properties: Map<String, Value>, required: Value) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}
