// Rust guideline compliant 2026-11-16

//! Resolve inverse argument templates against forward-call data.

use anyhow::{bail, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Resolves references to prior batch step outputs recursively.
pub fn interpolate_step_references(val: &Value, outputs: &HashMap<String, Value>) -> Value {
    interpolate_batch_args(val, outputs, false).unwrap_or_else(|_| val.clone())
}

pub(crate) fn interpolate_batch_args(
    val: &Value,
    outputs: &HashMap<String, Value>,
    enforce: bool,
) -> Result<Value> {
    match val {
        Value::String(text) if text.starts_with('$') => {
            let reference = &text[1..];
            Ok(resolve_step_reference(reference, outputs, enforce)?.unwrap_or_else(|| val.clone()))
        }
        Value::Array(items) => Ok(Value::Array(
            items
                .iter()
                .map(|item| interpolate_batch_args(item, outputs, enforce))
                .collect::<Result<_>>()?,
        )),
        Value::Object(fields) => Ok(Value::Object(
            fields
                .iter()
                .map(|(key, value)| {
                    Ok((
                        key.clone(),
                        interpolate_batch_args(value, outputs, enforce)?,
                    ))
                })
                .collect::<Result<_>>()?,
        )),
        scalar => Ok(scalar.clone()),
    }
}

fn resolve_step_reference(
    reference: &str,
    outputs: &HashMap<String, Value>,
    enforce: bool,
) -> Result<Option<Value>> {
    use crate::result_sanitization::{contains_redaction, REDACTED};

    let mut parts = reference.splitn(2, '.');
    let Some(step_id) = parts.next() else {
        return Ok(None);
    };
    let Some(mut value) = outputs.get(step_id) else {
        return Ok(None);
    };
    for segment in parts.flat_map(|path| path.split('.')) {
        if enforce && value.as_str().is_some_and(|text| text.contains(REDACTED)) {
            bail!("Batch reference targets a protected result");
        }
        let Some(next) = value.get(segment) else {
            return Ok(None);
        };
        value = next;
    }
    if enforce && contains_redaction(value) {
        bail!("Batch reference targets a protected result");
    }
    Ok(Some(value.clone()))
}

/// Resolves `${args.path}`, `${result.path}`, and `${snapshot.path}` references recursively.
///
/// A string containing only one reference retains the referenced JSON type. References embedded
/// in a larger string are rendered as strings; object and array values use compact JSON encoding.
pub fn interpolate_undo_template(
    template: &Value,
    args: &Value,
    result: &Value,
    snapshot: &Value,
) -> Result<Value> {
    match template {
        Value::String(text) => interpolate_string(text, args, result, snapshot),
        Value::Array(items) => items
            .iter()
            .map(|item| interpolate_undo_template(item, args, result, snapshot))
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Value::Object(fields) => fields
            .iter()
            .map(|(key, value)| {
                Ok((
                    key.clone(),
                    interpolate_undo_template(value, args, result, snapshot)?,
                ))
            })
            .collect::<Result<serde_json::Map<_, _>>>()
            .map(Value::Object),
        scalar => Ok(scalar.clone()),
    }
}

fn interpolate_string(text: &str, args: &Value, result: &Value, snapshot: &Value) -> Result<Value> {
    if let Some(reference) = exact_reference(text) {
        return resolve_reference(reference, args, result, snapshot).cloned();
    }

    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find('}') else {
            bail!("unterminated undo template reference");
        };
        let reference = &after_start[..end];
        let value = resolve_reference(reference, args, result, snapshot)?;
        match value {
            Value::String(text) => output.push_str(text),
            other => output.push_str(&serde_json::to_string(other)?),
        }
        rest = &after_start[end + 1..];
    }
    output.push_str(rest);
    Ok(Value::String(output))
}

fn exact_reference(text: &str) -> Option<&str> {
    text.strip_prefix("${")?.strip_suffix('}')
}

fn resolve_reference<'a>(
    reference: &str,
    args: &'a Value,
    result: &'a Value,
    snapshot: &'a Value,
) -> Result<&'a Value> {
    let (root, path) = reference.split_once('.').unwrap_or((reference, ""));
    let value = match root {
        "args" => args,
        "result" => result,
        "snapshot" => snapshot,
        _ => bail!("unsupported undo template root '{root}'"),
    };
    if path.is_empty() {
        return Ok(value);
    }
    path.split('.').try_fold(value, |current, segment| {
        let next = match current {
            Value::Array(items) => segment
                .parse::<usize>()
                .ok()
                .and_then(|index| items.get(index)),
            _ => current.get(segment),
        };
        next.ok_or_else(|| anyhow::anyhow!("missing undo template path '${{{reference}}}'"))
    })
}

#[cfg(test)]
mod tests {
    use super::interpolate_undo_template;
    use serde_json::json;

    #[test]
    fn resolves_nested_paths_arrays_and_preserves_json_types() {
        let template = json!({
            "id": "${args.issue.id}",
            "priority": "${snapshot.priority}",
            "audit": "restored ${result.id}",
            "items": ["${args.flags.0}"]
        });
        let result = interpolate_undo_template(
            &template,
            &json!({"issue": {"id": 42}, "flags": [true]}),
            &json!({"id": "op-7"}),
            &json!({"priority": "low"}),
        )
        .unwrap();
        assert_eq!(result["id"], 42);
        assert_eq!(result["priority"], "low");
        assert_eq!(result["audit"], "restored op-7");
        assert_eq!(result["items"][0], true);
    }

    #[test]
    fn rejects_missing_or_unsupported_references() {
        assert!(interpolate_undo_template(
            &json!("${args.missing}"),
            &json!({}),
            &json!({}),
            &json!({})
        )
        .is_err());
        assert!(interpolate_undo_template(
            &json!("${env.secret}"),
            &json!({}),
            &json!({}),
            &json!({})
        )
        .is_err());
    }
}
