// Rust guideline compliant 2026-08-17

//! Context distillation, line truncation, byte budgets, and JSONPath filtering for agent payloads (`M-CANONICAL-DOCS`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Modifiers for distilling and truncating large tool/resource responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistillationOptions {
    /// Optional JSONPath selector (e.g. `$.items[*].id` or `data.user`).
    #[serde(default, rename = "_jsonpath", alias = "jsonpath")]
    pub jsonpath: Option<String>,
    /// Optional maximum number of lines to retain in string or array outputs.
    #[serde(default, rename = "_limit_lines", alias = "limit_lines")]
    pub limit_lines: Option<usize>,
    /// Optional maximum byte size budget for the rendered payload.
    #[serde(default, rename = "_truncate_bytes", alias = "truncate_bytes")]
    pub truncate_bytes: Option<usize>,
}

impl DistillationOptions {
    /// Constructs distillation options from an arbitrary JSON arguments map if special `_` keys exist.
    ///
    /// # Arguments
    /// * `args` - Optional JSON value representing arguments object.
    ///
    /// # Returns
    /// An instance of `DistillationOptions`.
    pub fn from_args(args: Option<&Value>) -> Self {
        let Some(Value::Object(map)) = args else {
            return Self::default();
        };

        let jsonpath = map
            .get("_jsonpath")
            .or_else(|| map.get("jsonpath"))
            .and_then(Value::as_str)
            .map(ToString::to_string);

        let limit_lines = map
            .get("_limit_lines")
            .or_else(|| map.get("limit_lines"))
            .and_then(Value::as_u64)
            .map(|n| n as usize);

        let truncate_bytes = map
            .get("_truncate_bytes")
            .or_else(|| map.get("truncate_bytes"))
            .and_then(Value::as_u64)
            .map(|n| n as usize);

        Self {
            jsonpath,
            limit_lines,
            truncate_bytes,
        }
    }

    /// Checks if any distillation rules are active.
    ///
    /// # Returns
    /// `true` if at least one filter modifier is set, otherwise `false`.
    pub fn is_active(&self) -> bool {
        self.jsonpath.is_some() || self.limit_lines.is_some() || self.truncate_bytes.is_some()
    }
}

/// Distills and truncates a JSON value according to specified options.
///
/// # Arguments
/// * `value` - Input JSON data to distill.
/// * `options` - Filtering and truncation rules.
///
/// # Returns
/// Distilled and truncated JSON `Value`.
pub fn distill_value(value: Value, options: &DistillationOptions) -> Value {
    if !options.is_active() {
        return value;
    }

    let mut current = value;

    // 1. JSONPath filtering if requested
    if let Some(ref path) = options.jsonpath {
        current = apply_jsonpath(current, path);
    }

    // 2. Line limiting if requested
    if let Some(limit) = options.limit_lines {
        current = apply_line_limit(current, limit);
    }

    // 3. Byte budget truncation if requested
    if let Some(budget) = options.truncate_bytes {
        current = apply_byte_truncation(current, budget);
    }

    current
}

/// Evaluates a simple dot-notation / wildcard JSONPath selector against a JSON value.
///
/// Supports patterns like:
/// - `foo.bar` / `$.foo.bar`
/// - `items[*].id` / `$.items[*]`
/// - `data[0].name`
fn apply_jsonpath(val: Value, path: &str) -> Value {
    let clean_path = path.trim_start_matches("$.");
    if clean_path.is_empty() || clean_path == "$" {
        return val;
    }

    let segments = parse_path_segments(clean_path);
    extract_path_recursive(&val, &segments)
}

fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some(idx) = part.find('[') {
            let key = &part[..idx];
            if !key.is_empty() {
                segments.push(PathSegment::Key(key.to_string()));
            }
            let index_expr = &part[idx..];
            for bracket in index_expr.split(']').filter(|s| !s.is_empty()) {
                let inner = bracket.trim_start_matches('[');
                if inner == "*" {
                    segments.push(PathSegment::Wildcard);
                } else if let Ok(n) = inner.parse::<usize>() {
                    segments.push(PathSegment::Index(n));
                }
            }
        } else {
            segments.push(PathSegment::Key(part.to_string()));
        }
    }
    segments
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    Key(String),
    Index(usize),
    Wildcard,
}

fn extract_path_recursive(val: &Value, segments: &[PathSegment]) -> Value {
    if segments.is_empty() {
        return val.clone();
    }

    match &segments[0] {
        PathSegment::Key(key) => match val {
            Value::Object(map) => match map.get(key) {
                Some(child) => extract_path_recursive(child, &segments[1..]),
                None => Value::Null,
            },
            _ => Value::Null,
        },
        PathSegment::Index(idx) => match val {
            Value::Array(arr) => match arr.get(*idx) {
                Some(child) => extract_path_recursive(child, &segments[1..]),
                None => Value::Null,
            },
            _ => Value::Null,
        },
        PathSegment::Wildcard => match val {
            Value::Array(arr) => {
                let results = arr
                    .iter()
                    .map(|item| extract_path_recursive(item, &segments[1..]))
                    .filter(|v| !v.is_null())
                    .collect::<Vec<_>>();
                Value::Array(results)
            }
            Value::Object(map) => {
                let results = map
                    .values()
                    .map(|item| extract_path_recursive(item, &segments[1..]))
                    .filter(|v| !v.is_null())
                    .collect::<Vec<_>>();
                Value::Array(results)
            }
            _ => Value::Null,
        },
    }
}

/// Truncates string contents or array elements to at most `max_lines`.
fn apply_line_limit(val: Value, max_lines: usize) -> Value {
    match val {
        Value::String(s) => {
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() <= max_lines {
                Value::String(s)
            } else {
                let retained = lines[..max_lines].join("\n");
                let omitted = lines.len() - max_lines;
                Value::String(format!(
                    "{}\n[... truncated {} lines by Warmplane]",
                    retained, omitted
                ))
            }
        }
        Value::Array(arr) => {
            if arr.len() <= max_lines {
                Value::Array(arr)
            } else {
                let mut truncated = arr[..max_lines].to_vec();
                let omitted = arr.len() - max_lines;
                truncated.push(json!({
                    "_warmplane_truncated": true,
                    "omitted_items": omitted
                }));
                Value::Array(truncated)
            }
        }
        Value::Object(mut map) => {
            for (_, v) in map.iter_mut() {
                *v = apply_line_limit(v.clone(), max_lines);
            }
            Value::Object(map)
        }
        other => other,
    }
}

/// Truncates a payload to fit within a byte size budget.
fn apply_byte_truncation(val: Value, budget_bytes: usize) -> Value {
    let serialized = match serde_json::to_string(&val) {
        Ok(s) => s,
        Err(_) => return val,
    };

    if serialized.len() <= budget_bytes {
        return val;
    }

    match val {
        Value::String(s) => {
            let bytes = s.as_bytes();
            if bytes.len() <= budget_bytes {
                return Value::String(s);
            }
            let truncated_slice = &bytes[..budget_bytes.min(bytes.len())];
            let safe_str = String::from_utf8_lossy(truncated_slice);
            json!({
                "_warmplane_truncated": true,
                "original_bytes": bytes.len(),
                "truncated_bytes": budget_bytes,
                "data": format!("{}... [truncated]", safe_str)
            })
        }
        Value::Array(arr) => {
            let mut result = Vec::new();
            let mut running_size = 2; // "[]"
            for item in arr.iter() {
                let item_size = serde_json::to_string(item).map(|s| s.len()).unwrap_or(0);
                if running_size + item_size + 1 > budget_bytes {
                    break;
                }
                running_size += item_size + 1;
                result.push(item.clone());
            }
            json!({
                "_warmplane_truncated": true,
                "original_bytes": serialized.len(),
                "truncated_bytes": running_size,
                "retained_items": result.len(),
                "data": result
            })
        }
        _ => json!({
            "_warmplane_truncated": true,
            "original_bytes": serialized.len(),
            "truncated_bytes": budget_bytes,
            "data": format!("{}... [truncated]", &serialized[..budget_bytes.min(serialized.len())])
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_args_extraction() {
        let args = json!({
            "param1": "value",
            "_jsonpath": "$.items[*].name",
            "_limit_lines": 5,
            "_truncate_bytes": 1024
        });
        let opts = DistillationOptions::from_args(Some(&args));
        assert_eq!(opts.jsonpath.as_deref(), Some("$.items[*].name"));
        assert_eq!(opts.limit_lines, Some(5));
        assert_eq!(opts.truncate_bytes, Some(1024));
        assert!(opts.is_active());
    }

    #[test]
    fn test_apply_jsonpath_object_and_wildcard() {
        let data = json!({
            "users": [
                { "id": 1, "name": "Alice" },
                { "id": 2, "name": "Bob" }
            ],
            "meta": { "total": 2 }
        });

        let extracted_names = apply_jsonpath(data.clone(), "users[*].name");
        assert_eq!(extracted_names, json!(["Alice", "Bob"]));

        let extracted_meta = apply_jsonpath(data.clone(), "meta.total");
        assert_eq!(extracted_meta, json!(2));
    }

    #[test]
    fn test_apply_line_limit_on_string_and_array() {
        let text = json!("line 1\nline 2\nline 3\nline 4\nline 5");
        let truncated = apply_line_limit(text, 2);
        assert_eq!(
            truncated.as_str().unwrap(),
            "line 1\nline 2\n[... truncated 3 lines by Warmplane]"
        );

        let arr = json!([1, 2, 3, 4, 5]);
        let truncated_arr = apply_line_limit(arr, 3);
        assert_eq!(truncated_arr.as_array().unwrap().len(), 4);
        assert_eq!(
            truncated_arr.as_array().unwrap()[3]["_warmplane_truncated"],
            true
        );
    }

    #[test]
    fn test_apply_byte_truncation() {
        let long_str = json!("A".repeat(500));
        let truncated = apply_byte_truncation(long_str, 50);
        assert_eq!(truncated["_warmplane_truncated"], true);
        assert_eq!(truncated["original_bytes"], 500);
    }
}
