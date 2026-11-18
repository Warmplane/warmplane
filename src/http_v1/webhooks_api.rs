// Rust guideline compliant 2026-11-16

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

                // A configured secret is the only thing that makes this endpoint
                // safe, because it reaches approve and reject with no bearer token of
                // its own. Require the signature rather than verifying it when one
                // happens to be present: omitting the header is exactly what an
                // attacker sends.
                let Some(sig) = sig_header else {
                    warn!("Rejecting incoming webhook callback: signature header absent");
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "ok": false, "error": "Signature required" })),
                    )
                        .into_response();
                };

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

/// Handles a one-target Undo or Cancel Send action issued from a signed ChatOps card.
///
/// # Arguments
/// * `state` - Active daemon state and tenant-owned action registries.
/// * `body_bytes` - Platform callback body containing the opaque signed action token.
///
/// # Returns
/// A structured result for the completed action, or a status error for invalid authorization or
/// a target that is no longer actionable.
pub async fn handle_webhook_action(
    State(state): State<AppState>,
    body_bytes: Bytes,
) -> impl IntoResponse {
    let body = match std::str::from_utf8(&body_bytes) {
        Ok(body) => body,
        Err(_) => {
            return action_error(
                StatusCode::BAD_REQUEST,
                "INVALID_PAYLOAD",
                "Invalid UTF-8 payload",
            )
        }
    };
    let payload = if let Some(form_payload) = body.strip_prefix("payload=") {
        let decoded = url_decode_str(form_payload);
        match serde_json::from_str::<Value>(&decoded) {
            Ok(value) => value,
            Err(_) => {
                return action_error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_PAYLOAD",
                    "Invalid platform payload",
                )
            }
        }
    } else {
        match serde_json::from_slice::<Value>(&body_bytes) {
            Ok(value) => value,
            Err(_) => {
                return action_error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_PAYLOAD",
                    "Invalid JSON payload",
                )
            }
        }
    };
    let Some(token) = extract_action_token(&payload) else {
        return action_error(
            StatusCode::UNAUTHORIZED,
            "ACTION_TOKEN_REQUIRED",
            "A signed action token is required",
        );
    };
    let Some((target_id, expiry)) = parse_action_token(&token) else {
        return action_error(
            StatusCode::UNAUTHORIZED,
            "INVALID_ACTION_TOKEN",
            "Action token is invalid or expired",
        );
    };
    let config = match crate::config::load_or_default_config(&state.config_path) {
        Ok(config) => config,
        Err(_) => {
            return action_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "CONFIG_UNAVAILABLE",
                "Webhook configuration is unavailable",
            )
        }
    };
    let Some(webhook) = config
        .policy
        .as_ref()
        .and_then(|policy| policy.webhook.as_ref())
    else {
        return action_error(
            StatusCode::UNAUTHORIZED,
            "ACTION_UNAVAILABLE",
            "ChatOps actions are not configured",
        );
    };
    let Some(role) = webhook
        .action_role
        .as_deref()
        .filter(|role| !role.trim().is_empty())
    else {
        return action_error(
            StatusCode::UNAUTHORIZED,
            "ACTION_UNAVAILABLE",
            "ChatOps action role is not configured",
        );
    };
    let Some(secret) = webhook.resolve_secret() else {
        return action_error(
            StatusCode::UNAUTHORIZED,
            "ACTION_UNAVAILABLE",
            "ChatOps signing secret is unavailable",
        );
    };
    if state.rbac_engine.is_enabled()
        && (!state.rbac_engine.recognizes_role(role)
            || !state.rbac_engine.undo_permissions(role).execute)
    {
        return action_error(
            StatusCode::FORBIDDEN,
            "POLICY_DENIED",
            "Configured ChatOps role cannot execute actions",
        );
    }
    let now_ms = unix_time_ms();

    let stored_entry = match state.undo_ledger.clone() {
        Some(ledger) => ledger.get(&target_id).await,
        None => None,
    };
    if let Some(entry) = stored_entry {
        if entry.expires_at_ms != expiry || entry.status != crate::undo::UndoEntryStatus::Armed {
            return action_error(
                StatusCode::NOT_FOUND,
                "ACTION_UNAVAILABLE",
                "Undo action is unavailable",
            );
        }
        if !webhook
            .events
            .iter()
            .any(|event| event == "undo.action_available")
            || crate::chatops::verify_action_token(
                &secret,
                "undo",
                &token,
                &entry.tenant_id,
                role,
                now_ms,
            )
            .as_deref()
                != Some(entry.undo_id.as_str())
        {
            return action_error(
                StatusCode::UNAUTHORIZED,
                "INVALID_ACTION_TOKEN",
                "Action token is invalid or expired",
            );
        }
        let profile = if let Some(profile_id) = entry.profile_id.as_deref() {
            let profiles = state.profiles.read().await;
            let Some(profile_config) = profiles.get(profile_id) else {
                return action_error(
                    StatusCode::FORBIDDEN,
                    "PROFILE_UNAVAILABLE",
                    "The action profile no longer exists",
                );
            };
            crate::context::ProfileContext::scoped_with_policy(
                profile_id,
                profile_config.servers.clone(),
                profile_config
                    .policy
                    .as_ref()
                    .map(|policy| crate::daemon::Policy::from_config(Some(policy.clone()))),
            )
        } else {
            crate::context::ProfileContext::unrestricted()
        };
        let base_policy = state.policy.read().await.clone();
        let policy = state
            .rbac_engine
            .compute_effective_policy(role, &base_policy);
        let context = crate::context::RequestContext {
            operation_id: entry.operation_id.clone(),
            work_item_id: entry.work_item_id.clone(),
            actor_id: Some("chatops-webhook".to_string()),
            grant_id: None,
        };
        let access = crate::undo::UndoAccessContext {
            tenant_id: entry.tenant_id,
            role: role.to_string(),
            actor_id: Some("chatops-webhook".to_string()),
            request_context: context,
            policy,
            profile,
            grant_expires_at_epoch_secs: None,
            trace_id: crate::http_v1::next_trace_id(),
        };
        let action_state = state.clone();
        let action_target = target_id.clone();
        tokio::spawn(async move {
            if let Err(error) = crate::engine::ControlPlaneHandle::new(action_state)
                .execute_undo(&action_target, &access, false, None)
                .await
            {
                tracing::warn!(
                    code = error.code,
                    "ChatOps undo action could not be started"
                );
            }
        });
        return accepted_action();
    }

    let Some(task) = state.task_registry.get_task(&target_id).await else {
        return action_error(
            StatusCode::NOT_FOUND,
            "ACTION_UNAVAILABLE",
            "Action is unavailable",
        );
    };
    let Some(emission) = task.emission.as_ref() else {
        return action_error(
            StatusCode::NOT_FOUND,
            "ACTION_UNAVAILABLE",
            "Action is unavailable",
        );
    };
    if emission.release_at_ms != expiry
        || !emission.cancellable
        || emission.phase != crate::tasks::EmissionPhase::PendingEmission
        || !webhook.events.iter().any(|event| event == "emission.held")
        || crate::chatops::verify_action_token(
            &secret,
            "cancel",
            &token,
            &emission.tenant_id,
            role,
            now_ms,
        )
        .as_deref()
            != Some(task.task_id.as_str())
    {
        return action_error(
            StatusCode::NOT_FOUND,
            "ACTION_UNAVAILABLE",
            "Action is unavailable",
        );
    }
    let action_state = state.clone();
    let action_target = target_id.clone();
    tokio::spawn(async move {
        if action_state
            .hold_coordinator
            .cancel(
                &action_state,
                &action_target,
                Some("Cancelled from ChatOps".to_string()),
            )
            .await
            .is_err()
        {
            tracing::warn!("ChatOps cancel action could not be started");
        }
    });
    accepted_action()
}

fn accepted_action() -> axum::response::Response {
    StatusCode::OK.into_response()
}

fn extract_action_token(payload: &Value) -> Option<String> {
    if let Some(token) = payload.get("action_token").and_then(Value::as_str) {
        return Some(token.to_string());
    }
    if let Some(token) = payload
        .get("data")
        .and_then(|data| data.get("action_token"))
        .and_then(Value::as_str)
    {
        return Some(token.to_string());
    }
    if let Some(token) = payload
        .get("data")
        .and_then(|data| data.get("custom_id"))
        .and_then(Value::as_str)
    {
        return Some(token.to_string());
    }
    let value = payload
        .get("actions")
        .and_then(Value::as_array)
        .and_then(|actions| actions.first())
        .and_then(|action| action.get("value"))?;
    if let Some(token) = value.as_str() {
        if let Ok(nested) = serde_json::from_str::<Value>(token) {
            return nested
                .get("action_token")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        return Some(token.to_string());
    }
    None
}

fn parse_action_token(token: &str) -> Option<(String, u64)> {
    let (head, signature) = token.rsplit_once('.')?;
    if signature.len() != 32 {
        return None;
    }
    let (target, expiry) = head.rsplit_once('.')?;
    Some((target.to_string(), expiry.parse().ok()?))
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn action_error(status: StatusCode, code: &str, message: &str) -> axum::response::Response {
    (
        status,
        Json(json!({"ok": false, "error": {"code": code, "message": message}})),
    )
        .into_response()
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

    // Validate incoming URL syntax and scheme first
    let _ = match reqwest::Url::parse(&target_url) {
        Ok(u) => {
            if u.scheme() != "http" && u.scheme() != "https" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": "Webhook URL must use http or https scheme" })),
                )
                    .into_response();
            }
            u
        }
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": format!("Invalid webhook URL: {}", e) })),
            )
                .into_response();
        }
    };

    // Verify target_url against configured webhook URL or explicit allowedUrls allowlist,
    // selecting the destination directly from the trusted configuration to break taint flow.
    let trusted_target_url = match webhook_cfg.as_ref() {
        Some(cfg) if cfg.url == target_url => cfg.url.as_str(),
        Some(cfg) => {
            if let Some(matched) = cfg
                .allowed_urls
                .iter()
                .find(|allowed| *allowed == &target_url)
            {
                matched.as_str()
            } else {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "ok": false,
                        "error": "Webhook URL is not permitted. URL must match policy.webhook.url or be present in policy.webhook.allowed_urls."
                    })),
                )
                    .into_response();
            }
        }
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "ok": false,
                    "error": "No policy webhook configuration present."
                })),
            )
                .into_response();
        }
    };

    // Parse the trusted URL into reqwest::Url
    let parsed_trusted_url = match reqwest::Url::parse(trusted_target_url) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": format!("Invalid webhook URL in configuration: {}", e) })),
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

    match client.post(parsed_trusted_url).json(&formatted).send().await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{McpConfig, PolicyConfig, WebhookConfig};
    use axum::body::to_bytes;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_webhook_rejects_unpermitted_url() {
        let temp = NamedTempFile::new().unwrap();
        let config = McpConfig {
            policy: Some(PolicyConfig {
                webhook: Some(WebhookConfig {
                    url: "https://hooks.slack.com/services/T00/B00/X00".to_string(),
                    allowed_urls: vec!["https://discord.com/api/webhooks/1/2".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        std::fs::write(temp.path(), serde_json::to_string(&config).unwrap()).unwrap();

        let state = AppState::builder()
            .config_path(temp.path().to_str().unwrap().to_string())
            .catalog_version("test")
            .build();

        // 1. Target URL not in allowed list
        let req = TestWebhookRequest {
            url: Some("https://evil.internal.attacker.com/webhook".to_string()),
            format: None,
        };
        let resp = handle_test_webhook(State(state.clone()), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body_bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body_json["ok"], false);

        // 2. Target URL with invalid non-http/https scheme
        let req_invalid = TestWebhookRequest {
            url: Some("file:///etc/passwd".to_string()),
            format: None,
        };
        let resp_invalid = handle_test_webhook(State(state), Json(req_invalid))
            .await
            .into_response();
        assert_eq!(resp_invalid.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn signed_chatops_undo_action_runs_only_its_persisted_inverse() {
        use crate::{
            config::{
                McpConfig, PolicyConfig, ReversibilityClass, ReversibilityConfig, WebhookConfig,
            },
            daemon::{CapabilityMeta, ServerMsg},
            undo::{UndoEntry, UndoEntryStatus, UndoLedgerStore},
        };
        use serde_json::json;
        use std::collections::HashMap;

        let directory = tempfile::tempdir().unwrap();
        let config_path = directory.path().join("config.json");
        let expires_at_ms = unix_time_ms() + 60_000;
        let config = McpConfig {
            policy: Some(PolicyConfig {
                webhook: Some(WebhookConfig {
                    url: "https://chat.example/hooks".to_string(),
                    secret: Some("action-secret".to_string()),
                    callback_url: Some("https://warmplane.example/v1/webhooks/actions".to_string()),
                    action_role: Some("operator".to_string()),
                    events: vec!["undo.action_available".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        crate::config::save_config(config_path.to_str().unwrap(), &config).unwrap();

        let ledger = UndoLedgerStore::open(directory.path().join("undo.json")).unwrap();
        ledger
            .register(UndoEntry {
                undo_id: "undo-chatops-test".to_string(),
                operation_id: Some("op-forward".to_string()),
                work_item_id: Some("work-1".to_string()),
                batch_id: None,
                tenant_id: "tenant-a".to_string(),
                profile_id: None,
                actor_id: Some("agent-a".to_string()),
                forward_capability: "mock.forward".to_string(),
                reversibility_class: "reversible".to_string(),
                resolved_inverse_capability: "mock.inverse".to_string(),
                resolved_inverse_args: json!({"key":"safe-id"}),
                change_preview: Vec::new(),
                snapshot_artifact_hash: None,
                pre_undo_check: None,
                expected_post_snapshot_hash: None,
                created_at_ms: expires_at_ms - 60_000,
                expires_at_ms,
                status: UndoEntryStatus::Armed,
            })
            .await
            .unwrap();
        let mut inverse = CapabilityMeta::new(
            "mock",
            "inverse",
            "Inverse",
            "Apply inverse",
            json!({"type":"object"}),
        );
        inverse.reversibility = Some(ReversibilityConfig {
            class: ReversibilityClass::Irreversible,
            hold_window_secs: None,
            ttl_secs: None,
            daily_limit: None,
            pre_snapshot: None,
            pre_undo_check: None,
            test_target: None,
            inverse: None,
        });
        let (server_tx, mut server_rx) = tokio::sync::mpsc::channel(4);
        let state = AppState::builder()
            .config_path(config_path.to_string_lossy().to_string())
            .servers(HashMap::from([("mock".to_string(), server_tx)]))
            .capabilities(HashMap::from([("mock.inverse".to_string(), inverse)]))
            .undo_ledger(ledger.clone())
            .build();
        let token = crate::chatops::sign_action_token(
            "action-secret",
            "undo",
            "undo-chatops-test",
            "tenant-a",
            "operator",
            expires_at_ms,
        )
        .unwrap();
        let wrong_tenant_token = crate::chatops::sign_action_token(
            "action-secret",
            "undo",
            "undo-chatops-test",
            "tenant-b",
            "operator",
            expires_at_ms,
        )
        .unwrap();
        for token in [format!("{token}x"), wrong_tenant_token] {
            let response = handle_webhook_action(
                State(state.clone()),
                Bytes::from(json!({"action_token": token}).to_string()),
            )
            .await
            .into_response();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert!(server_rx.try_recv().is_err());
        }
        let body = Bytes::from(json!({"action_token": token}).to_string());
        let response = handle_webhook_action(State(state.clone()), body.clone())
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let Some(ServerMsg::CallTool {
            name,
            params,
            reply,
            ..
        }) = server_rx.recv().await
        else {
            panic!("expected the configured inverse call");
        };
        assert_eq!(name, "inverse");
        assert_eq!(params, json!({"key":"safe-id"}));
        reply.send(Ok(json!({"ok":true}))).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                if ledger
                    .get("undo-chatops-test")
                    .await
                    .is_some_and(|entry| entry.status == UndoEntryStatus::Reverted)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("background ChatOps action should complete");
        let replay = handle_webhook_action(State(state), body)
            .await
            .into_response();
        assert_eq!(replay.status(), StatusCode::NOT_FOUND);
        assert!(server_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_webhook_rejects_when_no_webhook_configured() {
        let temp = NamedTempFile::new().unwrap();
        let config = McpConfig::default();
        std::fs::write(temp.path(), serde_json::to_string(&config).unwrap()).unwrap();

        let state = AppState::builder()
            .config_path(temp.path().to_str().unwrap().to_string())
            .catalog_version("test")
            .build();

        let req = TestWebhookRequest {
            url: None,
            format: None,
        };
        let resp = handle_test_webhook(State(state), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
