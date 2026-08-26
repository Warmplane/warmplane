// Rust guideline compliant 2026-08-26

//! REST API endpoints for incoming ChatOps callbacks and webhook simulation.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::daemon::AppState;

/// Request payload for simulating a test webhook dispatch.
#[derive(Deserialize, Debug, Clone)]
pub struct TestWebhookRequest {
    /// Target webhook URL override (optional).
    #[serde(default)]
    pub url: Option<String>,
    /// Payload format override (optional).
    #[serde(default)]
    pub format: Option<crate::chatops::WebhookFormat>,
}

/// Generic callback payload structure.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenericCallbackPayload {
    /// Action type (`approve` or `reject`).
    pub action: String,
    /// Approval ticket ID.
    pub ticket_id: String,
    /// Operator identifier.
    #[serde(default)]
    pub operator: Option<String>,
    /// Reason if rejected.
    #[serde(default)]
    pub reason: Option<String>,
    /// Modified JSON arguments if approved with edits.
    #[serde(default)]
    pub modified_args: Option<Value>,
}

/// Handles POST `/v1/webhooks/callbacks` processing incoming decisions from Slack, Discord, or generic webhooks.
pub async fn handle_webhook_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> impl IntoResponse {
    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": "Invalid UTF-8 payload" })),
            )
                .into_response()
        }
    };

    // Load active config to check secret
    let config = crate::config::load_or_default_config(&state.config_path).unwrap_or_default();
    if let Some(ref policy) = config.policy {
        if let Some(ref webhook_cfg) = policy.webhook {
            if let Some(secret) = webhook_cfg.resolve_secret() {
                let sig_header = headers
                    .get("x-warmplane-signature")
                    .or_else(|| headers.get("x-slack-signature"))
                    .and_then(|v| v.to_str().ok());

                let ts_header = headers
                    .get("x-warmplane-timestamp")
                    .or_else(|| headers.get("x-slack-request-timestamp"))
                    .and_then(|v| v.to_str().ok());

                if let Some(sig) = sig_header {
                    if !crate::chatops::verify_signature(&secret, body_str, sig, ts_header) {
                        warn!("Rejecting incoming webhook callback: HMAC signature mismatch");
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({ "ok": false, "error": "Signature mismatch" })),
                        )
                            .into_response();
                    }
                }
            }
        }
    }

    // Try parsing as Slack URL-encoded payload first (`payload=...`)
    let (action, ticket_id, operator, reason, modified_args) =
        if let Some(raw_payload) = body_str.strip_prefix("payload=") {
            let decoded = url_decode_str(raw_payload);
            if let Ok(slack_val) = serde_json::from_str::<Value>(&decoded) {
                parse_slack_interaction(&slack_val)
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": "Invalid Slack payload JSON" })),
                )
                    .into_response();
            }
        } else if let Ok(generic_val) = serde_json::from_str::<GenericCallbackPayload>(body_str) {
            (
                generic_val.action,
                generic_val.ticket_id,
                generic_val
                    .operator
                    .unwrap_or_else(|| "chatops-operator".to_string()),
                generic_val.reason,
                generic_val.modified_args,
            )
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": "Unrecognized callback payload structure" })),
            )
                .into_response();
        };

    if action == "approve" {
        match state
            .approval_registry
            .approve(&ticket_id, operator.clone(), modified_args, None)
            .await
        {
            Ok(true) => {
                info!(ticket_id = %ticket_id, operator = %operator, "approved ticket via incoming webhook callback");
                (
                    StatusCode::OK,
                    Json(json!({
                        "ok": true,
                        "status": "approved",
                        "ticket_id": ticket_id,
                        "operator": operator,
                        "text": format!("✅ Approval ticket `{}` was successfully approved by `{}`.", ticket_id, operator)
                    })),
                )
                    .into_response()
            }
            Ok(false) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "ok": false, "error": "Ticket not found or already resolved" })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response(),
        }
    } else if action == "reject" {
        match state
            .approval_registry
            .reject(&ticket_id, operator.clone(), reason.clone(), None)
            .await
        {
            Ok(true) => {
                info!(ticket_id = %ticket_id, operator = %operator, "rejected ticket via incoming webhook callback");
                (
                    StatusCode::OK,
                    Json(json!({
                        "ok": true,
                        "status": "rejected",
                        "ticket_id": ticket_id,
                        "operator": operator,
                        "reason": reason,
                        "text": format!("❌ Approval ticket `{}` was rejected by `{}`.", ticket_id, operator)
                    })),
                )
                    .into_response()
            }
            Ok(false) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "ok": false, "error": "Ticket not found or already resolved" })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": e.to_string() })),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": format!("Unsupported action '{}'", action) })),
        )
            .into_response()
    }
}

fn parse_slack_interaction(val: &Value) -> (String, String, String, Option<String>, Option<Value>) {
    let operator = val
        .get("user")
        .and_then(|u| u.get("username").or_else(|| u.get("name")))
        .and_then(Value::as_str)
        .unwrap_or("slack-user")
        .to_string();

    let mut action = "unknown".to_string();
    let mut ticket_id = "unknown".to_string();

    if let Some(actions) = val.get("actions").and_then(Value::as_array) {
        if let Some(first) = actions.first() {
            let action_id = first.get("action_id").and_then(Value::as_str).unwrap_or("");
            if action_id.contains("approve") {
                action = "approve".to_string();
            } else if action_id.contains("reject") {
                action = "reject".to_string();
            }

            if let Some(val_str) = first.get("value").and_then(Value::as_str) {
                if let Ok(parsed_btn_val) = serde_json::from_str::<Value>(val_str) {
                    if let Some(t_id) = parsed_btn_val.get("ticket_id").and_then(Value::as_str) {
                        ticket_id = t_id.to_string();
                    }
                    if let Some(act) = parsed_btn_val.get("action").and_then(Value::as_str) {
                        action = act.to_string();
                    }
                } else {
                    ticket_id = val_str.to_string();
                }
            }
        }
    }

    (action, ticket_id, operator, None, None)
}

fn url_decode_str(input: &str) -> String {
    let mut result = Vec::new();
    let mut bytes = input.bytes();
    while let Some(b) = bytes.next() {
        match b {
            b'+' => result.push(b' '),
            b'%' => {
                let h1 = bytes.next();
                let h2 = bytes.next();
                if let (Some(h1), Some(h2)) = (h1, h2) {
                    if let Ok(hex_byte) =
                        u8::from_str_radix(&format!("{}{}", h1 as char, h2 as char), 16)
                    {
                        result.push(hex_byte);
                        continue;
                    }
                }
                result.push(b'%');
                if let Some(h1) = h1 {
                    result.push(h1);
                }
                if let Some(h2) = h2 {
                    result.push(h2);
                }
            }
            other => result.push(other),
        }
    }
    String::from_utf8_lossy(&result).into_owned()
}

/// Handles POST `/v1/webhooks/test` sending a simulated test event to the configured webhook endpoint.
pub async fn handle_test_webhook(
    State(state): State<AppState>,
    Json(payload): Json<TestWebhookRequest>,
) -> impl IntoResponse {
    let config = crate::config::load_or_default_config(&state.config_path).unwrap_or_default();
    let webhook_cfg = if let Some(ref pol) = config.policy {
        pol.webhook.clone()
    } else {
        None
    };

    let target_url = payload
        .url
        .or_else(|| webhook_cfg.as_ref().map(|w| w.url.clone()));
    let target_format = payload
        .format
        .or_else(|| webhook_cfg.as_ref().and_then(|w| w.format))
        .unwrap_or_default();

    let target_url = match target_url {
        Some(u) if !u.trim().is_empty() => u,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": "No webhook URL configured or provided" })),
            )
                .into_response();
        }
    };

    let test_data = json!({
        "id": "appr-test-101",
        "capability_id": "db.drop_database",
        "server_id": "postgres_production",
        "sanitized_args": {
            "database": "customer_data",
            "cascade": true
        },
        "request_id": "req-test-sim",
        "created_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        "expires_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() + 300,
        "status": "pending"
    });

    let formatted = crate::chatops::format_webhook_payload(
        target_format,
        "approval.requested",
        &test_data,
        Some("http://127.0.0.1:9090"),
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    match client.post(&target_url).json(&formatted).send().await {
        Ok(resp) if resp.status().is_success() => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "message": format!("Successfully sent test webhook ({:?}) to {}", target_format, target_url),
                "status_code": resp.status().as_u16(),
            })),
        )
            .into_response(),
        Ok(resp) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "ok": false,
                "error": format!("Webhook target responded with HTTP {}", resp.status()),
                "status_code": resp.status().as_u16(),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "ok": false,
                "error": format!("Failed to send test webhook: {}", e),
            })),
        )
            .into_response(),
    }
}
