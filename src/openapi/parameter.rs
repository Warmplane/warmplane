// Rust guideline compliant 2026-11-21

//! OpenAPI parameter validation and serialization rules for the supported scalar subset.

use std::collections::BTreeMap;

use anyhow::{anyhow, ensure, Context, Result};
use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Location {
    Path,
    Query,
}

impl Location {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
        }
    }

    fn style(self) -> &'static str {
        match self {
            Self::Path => "simple",
            Self::Query => "form",
        }
    }
}

#[derive(Debug)]
pub(super) struct Parameter {
    pub(super) location: Location,
    pub(super) name: String,
    pub(super) required: bool,
    pub(super) schema: Map<String, Value>,
}

impl Parameter {
    fn parse(value: &Value) -> Result<Self> {
        let location = match value.get("in").and_then(Value::as_str) {
            Some("path") => Location::Path,
            Some("query") => Location::Query,
            _ => return Err(anyhow!("Unsupported OpenAPI parameter location")),
        };
        let name = value["name"]
            .as_str()
            .filter(|name| !name.is_empty() && !name.chars().any(char::is_control))
            .context("Invalid OpenAPI parameter name")?;
        let required = boolean(value.get("required"), false)?;
        ensure!(
            value
                .get("style")
                .is_none_or(|style| style == location.style())
                && !boolean(value.get("allowReserved"), false)?
                && !boolean(value.get("allowEmptyValue"), false)?
                && value.get("content").is_none(),
            "Unsupported OpenAPI parameter serialization"
        );
        boolean(value.get("explode"), location == Location::Query)?;
        ensure!(
            location != Location::Path || required,
            "OpenAPI path parameters must be required"
        );
        let schema = scalar_schema(&value["schema"])?;
        Ok(Self {
            location,
            name: name.to_owned(),
            required,
            schema,
        })
    }
}

fn boolean(value: Option<&Value>, default: bool) -> Result<bool> {
    value.map_or(Ok(default), |value| {
        value
            .as_bool()
            .context("OpenAPI parameter flag must be Boolean")
    })
}

pub(super) fn scalar_schema(value: &Value) -> Result<Map<String, Value>> {
    let schema = value
        .as_object()
        .context("OpenAPI parameter requires a schema")?;
    let kind = value["type"]
        .as_str()
        .context("OpenAPI parameter requires a scalar type")?;
    ensure!(
        matches!(kind, "string" | "integer" | "number" | "boolean"),
        "Unsupported OpenAPI parameter schema type"
    );
    for (key, constraint) in schema {
        ensure!(
            match key.as_str() {
                "type" | "title" | "description" | "example" | "examples" | "default"
                | "deprecated" => true,
                "enum" => constraint.as_array().is_some_and(|values| {
                    !values.is_empty() && values.iter().all(|value| matches_type(value, kind))
                }),
                "minLength" | "maxLength" => kind == "string" && constraint.as_u64().is_some(),
                "minimum" | "maximum" =>
                    matches!(kind, "integer" | "number")
                        && exact_numeric_range(constraint).is_some(),
                "nullable" => constraint == false,
                _ => false,
            },
            "Unsupported or invalid OpenAPI scalar constraint"
        );
    }
    Ok(schema.clone())
}

fn matches_type(value: &Value, kind: &str) -> bool {
    match kind {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        _ => false,
    }
}

fn exact_numeric_range(value: &Value) -> Option<f64> {
    const MAX_EXACT_INTEGER: u64 = 1 << 53;
    if value
        .as_i64()
        .is_some_and(|number| number.unsigned_abs() > MAX_EXACT_INTEGER)
        || value
            .as_u64()
            .is_some_and(|number| number > MAX_EXACT_INTEGER)
    {
        return None;
    }
    value.as_f64()
}

pub(super) fn matches_schema(value: &Value, schema: &Map<String, Value>) -> bool {
    let kind = schema
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    matches_type(value, kind)
        && schema
            .get("enum")
            .and_then(Value::as_array)
            .is_none_or(|choices| choices.contains(value))
        && matches_length(value, schema)
        && matches_bounds(value, schema)
}

fn matches_length(value: &Value, schema: &Map<String, Value>) -> bool {
    let Some(string) = value.as_str() else {
        return true;
    };
    let length = string.chars().count() as u64;
    length >= schema.get("minLength").and_then(Value::as_u64).unwrap_or(0)
        && length
            <= schema
                .get("maxLength")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
}

fn matches_bounds(value: &Value, schema: &Map<String, Value>) -> bool {
    let minimum = schema.get("minimum").and_then(Value::as_f64);
    let maximum = schema.get("maximum").and_then(Value::as_f64);
    (minimum.is_none() && maximum.is_none())
        || exact_numeric_range(value).is_some_and(|number| {
            minimum.is_none_or(|bound| number >= bound)
                && maximum.is_none_or(|bound| number <= bound)
        })
}

fn parse_list(value: Option<&Value>) -> Result<BTreeMap<(Location, String), Parameter>> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let mut params = BTreeMap::new();
    for value in value
        .as_array()
        .context("OpenAPI parameters must be an array")?
    {
        let parameter = Parameter::parse(value)?;
        let key = (parameter.location, parameter.name.clone());
        ensure!(
            params.insert(key, parameter).is_none(),
            "Duplicate OpenAPI parameter"
        );
    }
    Ok(params)
}

pub(super) fn parameters(
    path: Option<&Value>,
    operation: Option<&Value>,
) -> Result<Vec<Parameter>> {
    let mut params = parse_list(path)?;
    params.extend(parse_list(operation)?);
    Ok(params.into_values().collect())
}

pub(super) fn input_schema(parameters: &[Parameter]) -> Map<String, Value> {
    let mut sections = Map::new();
    let mut required = Vec::new();
    for location in [Location::Path, Location::Query] {
        let members = parameters.iter().filter(|param| param.location == location);
        let (properties, mandatory) = members.fold(
            (Map::new(), Vec::new()),
            |(mut properties, mut mandatory), param| {
                properties.insert(param.name.clone(), Value::Object(param.schema.clone()));
                if param.required {
                    mandatory.push(param.name.clone());
                }
                (properties, mandatory)
            },
        );
        if properties.is_empty() {
            continue;
        }
        if !mandatory.is_empty() {
            required.push(location.key());
        }
        sections.insert(
            location.key().to_owned(),
            object_schema(properties, json!(mandatory)),
        );
    }
    object_schema(sections, json!(required))
        .as_object()
        .cloned()
        .unwrap_or_default()
}

fn object_schema(properties: Map<String, Value>, required: Value) -> Value {
    super::input::object_schema(properties, required)
}
