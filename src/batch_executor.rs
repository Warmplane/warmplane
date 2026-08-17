// Rust guideline compliant 2026-08-17

//! Multi-step chained and batched capability execution with variable reference resolution (`M-CANONICAL-DOCS`).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::daemon::{AppState, ServerMsg, UpstreamCallError};

/// Single execution step within a batch capability call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStep {
    /// Unique step identifier within the batch (e.g. `"step1"`, `"lookup"`).
    pub id: String,
    /// Capability identifier to invoke.
    pub capability_id: String,
    /// JSON arguments for capability execution (may contain references like `"$step1.field"`).
    pub args: Value,
    /// Whether to continue executing subsequent steps if this step fails. Default `false`.
    #[serde(default)]
    pub continue_on_error: bool,
}

/// Request payload for batch capability execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCallRequest {
    /// Ordered list of steps to execute sequentially.
    pub steps: Vec<BatchStep>,
    /// Optional overall request trace identifier.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional request context envelope.
    #[serde(default)]
    pub context: Option<crate::context::RequestContext>,
}

/// Result of an individual step within a batch execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStepResult {
    /// Step identifier matching `BatchStep::id`.
    pub id: String,
    /// Capability invoked.
    pub capability_id: String,
    /// Whether execution succeeded.
    pub ok: bool,
    /// Execution output on success.
    pub data: Option<Value>,
    /// Error description if step failed.
    pub error: Option<String>,
    /// Step latency in microseconds.
    pub duration_us: u64,
}

/// Summary response envelope returned by batch capability execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCallResponse {
    /// Overall batch status.
    pub ok: bool,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Execution trace identifier.
    pub trace_id: String,
    /// Results for each step in execution order.
    pub results: Vec<BatchStepResult>,
    /// Total duration for the entire batch in microseconds.
    pub total_duration_us: u64,
}

/// Executes a series of batch capability steps sequentially with variable interpolation.
///
/// # Arguments
/// * `state` - Shared daemon `AppState`.
/// * `steps` - Ordered vector of `BatchStep` items.
/// * `trace_id` - Trace ID for the batch.
/// * `request_id` - Optional request ID.
/// * `context` - Optional request context.
///
/// # Returns
/// A `BatchCallResponse` summarizing all step executions.
pub async fn execute_batch(
    state: &AppState,
    steps: Vec<BatchStep>,
    trace_id: String,
    request_id: Option<String>,
    _context: Option<crate::context::RequestContext>,
) -> BatchCallResponse {
    let start_all = std::time::Instant::now();
    let mut step_outputs: HashMap<String, Value> = HashMap::new();
    let mut results = Vec::new();
    let mut overall_ok = true;

    for step in steps {
        let step_start = std::time::Instant::now();
        let interpolated_args = interpolate_step_references(&step.args, &step_outputs);

        // Check policy
        if !state.policy.read().await.allows(&step.capability_id) {
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some(format!(
                    "Capability '{}' blocked by policy",
                    step.capability_id
                )),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            if !step.continue_on_error {
                break;
            }
            continue;
        }

        // Lookup capability target
        let (server, tool) = {
            let caps_guard = state.capabilities.read().await;
            let Some(meta) = caps_guard.get(&step.capability_id) else {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(format!("Capability '{}' not found", step.capability_id)),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
            };
            (meta.server.clone(), meta.tool.clone())
        };

        // Lookup server sender channel
        let tx = {
            let servers_guard = state.servers.read().await;
            let Some(tx) = servers_guard.get(&server).cloned() else {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(format!("Server '{}' unreachable", server)),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
                continue;
            };
            tx
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        let send_res = tx
            .send(ServerMsg::CallTool {
                name: tool,
                params: interpolated_args.clone(),
                input_responses: None,
                request_state: None,
                reply: reply_tx,
            })
            .await;

        if send_res.is_err() {
            results.push(BatchStepResult {
                id: step.id.clone(),
                capability_id: step.capability_id.clone(),
                ok: false,
                data: None,
                error: Some(format!("Server '{}' mailbox closed", server)),
                duration_us: step_start.elapsed().as_micros() as u64,
            });
            overall_ok = false;
            if !step.continue_on_error {
                break;
            }
            continue;
        }

        match reply_rx.await {
            Ok(Ok(val)) => {
                step_outputs.insert(step.id.clone(), val.clone());
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: true,
                    data: Some(val),
                    error: None,
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
            }
            Ok(Err(UpstreamCallError::Timeout)) => {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Tool execution timed out".to_string()),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
            }
            Ok(Err(UpstreamCallError::Upstream(err))) => {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some(err),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
            }
            Err(_) => {
                results.push(BatchStepResult {
                    id: step.id.clone(),
                    capability_id: step.capability_id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Daemon actor task died".to_string()),
                    duration_us: step_start.elapsed().as_micros() as u64,
                });
                overall_ok = false;
                if !step.continue_on_error {
                    break;
                }
            }
        }
    }

    BatchCallResponse {
        ok: overall_ok,
        request_id,
        trace_id,
        results,
        total_duration_us: start_all.elapsed().as_micros() as u64,
    }
}

/// Recursively traverses a JSON Value, replacing any `$step_id.path` string references
/// with corresponding values extracted from previous completed steps.
pub fn interpolate_step_references(val: &Value, outputs: &HashMap<String, Value>) -> Value {
    match val {
        Value::String(s) if s.starts_with('$') => {
            let ref_expr = &s[1..];
            resolve_reference(ref_expr, outputs).unwrap_or_else(|| val.clone())
        }
        Value::Array(arr) => Value::Array(
            arr.iter()
                .map(|item| interpolate_step_references(item, outputs))
                .collect(),
        ),
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), interpolate_step_references(v, outputs));
            }
            Value::Object(new_map)
        }
        other => other.clone(),
    }
}

fn resolve_reference(expr: &str, outputs: &HashMap<String, Value>) -> Option<Value> {
    let mut parts = expr.splitn(2, '.');
    let step_id = parts.next()?;
    let step_val = outputs.get(step_id)?;

    if let Some(subpath) = parts.next() {
        let mut cur = step_val;
        for key in subpath.split('.') {
            cur = cur.get(key)?;
        }
        Some(cur.clone())
    } else {
        Some(step_val.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_reference_interpolation() {
        let mut outputs = HashMap::new();
        outputs.insert(
            "step1".to_string(),
            json!({
                "user_id": 42,
                "profile": {
                    "username": "alice"
                }
            }),
        );

        let input_args = json!({
            "target_id": "$step1.user_id",
            "username": "$step1.profile.username",
            "literal": "hello"
        });

        let resolved = interpolate_step_references(&input_args, &outputs);
        assert_eq!(resolved["target_id"], 42);
        assert_eq!(resolved["username"], "alice");
        assert_eq!(resolved["literal"], "hello");
    }
}
