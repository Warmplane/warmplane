// Rust guideline compliant 2026-11-21

//! Turns a bounded OpenAPI document into immutable, selected HTTP operations.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{ensure, Context, Result};
use reqwest::{Method, StatusCode, Url};
use serde_json::{Map, Value};
use tokio::io::AsyncReadExt;

use super::input::RequestBody;
use super::parameter::{self, Location, Parameter};
use super::{invocation_base, MAX_DEPTH, MAX_SPEC_BYTES};
use crate::config::{OpenApiAuth, OpenApiSourceConfig};

const MAX_REFERENCE_NODES: usize = 10_000;

#[derive(Debug)]
pub(super) enum Segment {
    Literal(String),
    Parameter(String),
}

#[derive(Debug)]
pub(super) struct Operation {
    pub(super) id: String,
    pub(super) description: String,
    pub(super) template: String,
    pub(super) method: Method,
    pub(super) body: Option<RequestBody>,
    pub(super) segments: Vec<Segment>,
    pub(super) parameters: Vec<Parameter>,
    pub(super) responses: BTreeMap<u16, Value>,
}

impl Operation {
    pub(super) fn input_schema(&self) -> Map<String, Value> {
        super::input::operation_schema(&self.parameters, self.body.as_ref())
    }
}

#[derive(Debug)]
pub(super) struct Source {
    pub(super) base: Url,
    pub(super) operations: BTreeMap<String, Operation>,
}

pub(super) async fn load(config_path: &str, config: &OpenApiSourceConfig) -> Result<Source> {
    let bytes = match (&config.spec_path, &config.remote_spec) {
        (Some(path), None) => read_local(config_path, path).await?,
        (None, Some(remote)) => super::remote::download(remote).await?,
        _ => anyhow::bail!("OpenAPI source requires exactly one spec location"),
    };
    let config = config.clone();
    tokio::task::spawn_blocking(move || compile(&bytes, &config))
        .await
        .ok()
        .context("OpenAPI compiler did not finish")?
}

async fn read_local(config_path: &str, spec: &Path) -> Result<Vec<u8>> {
    let directory = Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let spec_path = directory.join(spec);
    let metadata = tokio::fs::metadata(&spec_path)
        .await
        .ok()
        .context("OpenAPI spec is unavailable")?;
    ensure!(metadata.is_file(), "OpenAPI spec must be a regular file");
    ensure!(
        metadata.len() <= MAX_SPEC_BYTES as u64,
        "OpenAPI spec exceeds the byte limit"
    );
    let file = tokio::fs::File::open(spec_path)
        .await
        .ok()
        .context("OpenAPI spec is unavailable")?;
    let mut bytes = Vec::new();
    file.take((MAX_SPEC_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .ok()
        .context("OpenAPI spec could not be read")?;
    if spec
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        require_json(&bytes)?;
    }
    Ok(bytes)
}

pub(super) fn require_json(bytes: &[u8]) -> Result<()> {
    ensure!(
        bytes.len() <= MAX_SPEC_BYTES,
        "OpenAPI spec exceeds the byte limit"
    );
    serde_json::from_slice::<serde::de::IgnoredAny>(bytes)
        .map(|_| ())
        .ok()
        .context("OpenAPI spec declared as JSON is malformed")
}

pub(super) fn compile(bytes: &[u8], config: &OpenApiSourceConfig) -> Result<Source> {
    let root = document(bytes)?;
    let paths = root["paths"]
        .as_object()
        .context("OpenAPI spec requires a paths map")?;
    let selected = config
        .operations
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut operations = BTreeMap::new();
    for (template, path) in paths {
        for (method, entry) in path.as_object().into_iter().flatten() {
            let Some(id) = entry["operationId"]
                .as_str()
                .filter(|id| selected.contains(id))
            else {
                continue;
            };
            let enabled = config.mutations.iter().any(|operation| operation == id);
            let method = allowed_method(method, enabled)?;
            ensure!(
                !operations.contains_key(id),
                "Duplicate selected OpenAPI operation ID"
            );
            let operation =
                compile_operation(&root, template, path, entry, id, &config.auth, method)
                    .with_context(|| format!("Could not compile OpenAPI operation '{id}'"))?;
            operations.insert(id.to_owned(), operation);
        }
    }
    ensure!(
        operations.len() == selected.len(),
        "An allowlisted OpenAPI operation was not found"
    );
    Ok(Source {
        base: invocation_base(config)?,
        operations,
    })
}

fn allowed_method(method: &str, mutation: bool) -> Result<Method> {
    match (method, mutation) {
        ("get", false) => Ok(Method::GET),
        ("post", true) => Ok(Method::POST),
        ("put", true) => Ok(Method::PUT),
        ("patch", true) => Ok(Method::PATCH),
        ("delete", true) => Ok(Method::DELETE),
        _ => anyhow::bail!("Unsupported or unapproved OpenAPI method"),
    }
}

fn document(bytes: &[u8]) -> Result<Value> {
    ensure!(
        bytes.len() <= MAX_SPEC_BYTES,
        "OpenAPI spec exceeds the byte limit"
    );
    let mut budget = serde_saphyr::Budget::default();
    budget.max_depth = MAX_DEPTH;
    budget.max_documents = 1;
    budget.max_anchors = 0;
    budget.max_aliases = 0;
    let mut options = serde_saphyr::Options::default();
    options.budget = Some(budget);
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    options.with_snippet = false;
    let root: Value = serde_saphyr::from_slice_with_options(bytes, options)
        .ok()
        .context("OpenAPI spec is malformed or exceeds parsing limits")?;
    let version = root["openapi"].as_str().unwrap_or_default();
    ensure!(
        ["3.0.", "3.1."].iter().any(|prefix| {
            version.strip_prefix(prefix).is_some_and(|patch| {
                !patch.is_empty() && patch.bytes().all(|byte| byte.is_ascii_digit())
            })
        }),
        "OpenAPI spec must declare a supported OpenAPI 3.0 or 3.1 version"
    );
    Ok(root)
}

struct References<'a> {
    root: &'a Value,
    stack: Vec<&'a str>,
    nodes: usize,
    bytes: usize,
}

impl<'a> References<'a> {
    fn new(root: &'a Value) -> Self {
        Self {
            root,
            stack: Vec::new(),
            nodes: MAX_REFERENCE_NODES,
            bytes: MAX_SPEC_BYTES,
        }
    }

    fn expand(&mut self, value: &'a Value, depth: usize) -> Result<Value> {
        ensure!(
            depth < MAX_DEPTH,
            "OpenAPI reference expansion exceeds the depth limit"
        );
        self.nodes = self
            .nodes
            .checked_sub(1)
            .context("OpenAPI reference node budget exceeded")?;
        match value {
            Value::Object(fields) if fields.contains_key("$ref") => {
                self.expand_reference(fields, depth)
            }
            Value::Object(fields) => self.expand_map(fields, depth),
            Value::Array(values) => values
                .iter()
                .map(|value| self.expand(value, depth + 1))
                .collect(),
            Value::String(value) => {
                self.count_bytes(value)?;
                Ok(Value::String(value.clone()))
            }
            _ => Ok(value.clone()),
        }
    }

    fn count_bytes(&mut self, value: &str) -> Result<()> {
        self.bytes = self
            .bytes
            .checked_sub(value.len())
            .context("OpenAPI expanded byte budget exceeded")?;
        Ok(())
    }

    fn expand_map(&mut self, fields: &'a Map<String, Value>, depth: usize) -> Result<Value> {
        fields
            .iter()
            .map(|(key, value)| {
                self.count_bytes(key)?;
                Ok((key.clone(), self.expand(value, depth + 1)?))
            })
            .collect()
    }

    fn expand_reference(&mut self, fields: &'a Map<String, Value>, depth: usize) -> Result<Value> {
        ensure!(
            fields.len() == 1,
            "OpenAPI reference siblings are unsupported"
        );
        let reference = fields["$ref"]
            .as_str()
            .context("OpenAPI reference must be a string")?;
        self.count_bytes(reference)?;
        let pointer = reference
            .strip_prefix('#')
            .filter(|pointer| pointer.starts_with('/'))
            .context("OpenAPI references must use a same-document JSON Pointer")?;
        ensure!(!self.stack.contains(&reference), "Cyclic OpenAPI reference");
        let target = self
            .root
            .pointer(pointer)
            .context("Unresolved OpenAPI reference")?;
        self.stack.push(reference);
        let result = self.expand(target, depth + 1);
        self.stack.pop();
        result
    }

    fn expand_optional(&mut self, value: Option<&'a Value>) -> Result<Option<Value>> {
        value.map(|value| self.expand(value, 0)).transpose()
    }
}

fn compile_operation(
    root: &Value,
    template: &str,
    path: &Value,
    entry: &Value,
    id: &str,
    auth: &OpenApiAuth,
    method: Method,
) -> Result<Operation> {
    let mut refs = References::new(root);
    let expanded = refs.expand(entry, 0)?;
    let body = RequestBody::parse(expanded.get("requestBody"), &method)?;
    let path_params = refs.expand_optional(path.get("parameters"))?;
    let parameters = parameter::parameters(path_params.as_ref(), expanded.get("parameters"))?;
    let segments = path_segments(template, &parameters)?;
    let security = expanded.get("security").or_else(|| root.get("security"));
    validate_security(security, auth, &mut refs)?;
    let responses = success_responses(&expanded["responses"])?;
    let description = expanded
        .get("description")
        .or_else(|| expanded.get("summary"))
        .and_then(Value::as_str)
        .unwrap_or(id)
        .to_owned();
    ensure!(
        description.len() <= 4096,
        "OpenAPI operation description exceeds the byte limit"
    );
    Ok(Operation {
        id: id.to_owned(),
        description,
        template: template.to_owned(),
        method,
        body,
        segments,
        parameters,
        responses,
    })
}

fn path_segments(template: &str, params: &[Parameter]) -> Result<Vec<Segment>> {
    let relative = template
        .strip_prefix('/')
        .context("OpenAPI path must begin with '/'")?;
    let mut expected: BTreeSet<&str> = params
        .iter()
        .filter(|param| param.location == Location::Path)
        .map(|param| param.name.as_str())
        .collect();
    let segments = relative
        .split('/')
        .map(|segment| {
            if let Some(name) = segment
                .strip_prefix('{')
                .and_then(|name| name.strip_suffix('}'))
            {
                ensure!(
                    expected.remove(name),
                    "OpenAPI path placeholder has no unique path parameter"
                );
                return Ok(Segment::Parameter(name.to_owned()));
            }
            ensure!(
                !matches!(segment, "" | "." | "..")
                    && segment
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"-._~".contains(&byte)),
                "Unsupported OpenAPI path segment"
            );
            Ok(Segment::Literal(segment.to_owned()))
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        expected.is_empty(),
        "OpenAPI path parameter is absent from its template"
    );
    Ok(segments)
}

fn validate_security(
    security: Option<&Value>,
    auth: &OpenApiAuth,
    refs: &mut References<'_>,
) -> Result<()> {
    let Some(security) = security else {
        ensure!(
            matches!(auth, OpenApiAuth::None),
            "OpenAPI operation does not declare bearer authentication"
        );
        return Ok(());
    };
    let alternatives = security
        .as_array()
        .context("OpenAPI security requirements must be an array")?;
    let mut compatible = alternatives.is_empty() && matches!(auth, OpenApiAuth::None);
    for requirement in alternatives {
        let requirement = requirement
            .as_object()
            .context("Invalid OpenAPI security requirement")?;
        compatible |= match auth {
            OpenApiAuth::None => requirement.is_empty(),
            OpenApiAuth::Bearer { .. } => bearer_requirement(requirement, refs)?,
        };
    }
    ensure!(
        compatible,
        "OpenAPI operation does not support the configured authentication"
    );
    Ok(())
}

fn bearer_requirement(requirement: &Map<String, Value>, refs: &mut References<'_>) -> Result<bool> {
    let Some((name, scopes)) = requirement.iter().next().filter(|_| requirement.len() == 1) else {
        return Ok(false);
    };
    if !scopes.as_array().is_some_and(Vec::is_empty) {
        return Ok(false);
    }
    let scheme = refs.root["components"]["securitySchemes"]
        .get(name)
        .context("OpenAPI security requirement has no scheme definition")?;
    let scheme = refs.expand(scheme, 0)?;
    Ok(scheme["type"] == "http"
        && scheme["scheme"]
            .as_str()
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("bearer")))
}

fn success_responses(responses: &Value) -> Result<BTreeMap<u16, Value>> {
    let definitions = responses
        .as_object()
        .context("OpenAPI operation requires declared responses")?;
    let mut selected = BTreeMap::new();
    for (key, response) in definitions {
        ensure!(
            key != "2XX",
            "OpenAPI success status ranges are unsupported"
        );
        let Some(status) = key
            .parse::<u16>()
            .ok()
            .filter(|status| (200..300).contains(status))
        else {
            continue;
        };
        ensure!(response.is_object(), "Invalid OpenAPI success response");
        let schema = response.pointer("/content/application~1json/schema");
        if status == StatusCode::NO_CONTENT.as_u16() {
            ensure!(
                response.get("content").is_none(),
                "OpenAPI 204 response must have no content"
            );
        } else {
            ensure!(
                schema.is_some_and(Value::is_object),
                "OpenAPI success response requires a JSON schema"
            );
        }
        selected.insert(status, schema.cloned().unwrap_or(Value::Null));
    }
    ensure!(
        !selected.is_empty(),
        "OpenAPI operation has no supported success response"
    );
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_and_expansion_limits_accept_the_boundary() {
        let mut document = vec![b' '; MAX_SPEC_BYTES];
        document[..2].copy_from_slice(b"{}");
        assert!(require_json(&document).is_ok());
        document.push(b' ');
        assert!(require_json(&document).is_err());

        let root = Value::Null;
        let boundary = Value::Array(vec![Value::Null; MAX_REFERENCE_NODES - 1]);
        let oversized = Value::Array(vec![Value::Null; MAX_REFERENCE_NODES]);
        assert!(References::new(&root).expand(&boundary, 0).is_ok());
        assert!(References::new(&root).expand(&oversized, 0).is_err());
    }
}
