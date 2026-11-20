// Rust guideline compliant 2026-11-18

//! Multi-platform ChatOps payload formatting and incoming callback resolution.
//!
//! Formats outbound notification events into platform-specific rich cards:
//! - Generic JSON
//! - Slack Block Kit (`format: "slack"`) with interactive action buttons
//! - Discord Embeds (`format: "discord"`) with component buttons
//! - Microsoft Teams Adaptive Cards (`format: "teams"`)
//!
//! Also provides signature verification for incoming webhook callbacks.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

pub(crate) async fn post_signed_webhook(
    cfg: &crate::config::WebhookConfig,
    event: &str,
    payload: &Value,
    secret: &str,
) -> anyhow::Result<()> {
    let body = serde_json::to_vec(payload)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(&body);
    let signature = hex::encode(mac.finalize().into_bytes());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let mut request = client
        .post(&cfg.url)
        .header("Content-Type", "application/json")
        .header("X-Warmplane-Timestamp", timestamp.to_string())
        .header("X-Warmplane-Event", event)
        .header("X-Warmplane-Signature", format!("sha256={signature}"));
    if let Some(auth) = &cfg.auth_header {
        request = request.header("Authorization", auth);
    }
    if let Some(headers) = &cfg.headers {
        for (name, value) in headers {
            request = request.header(name, value);
        }
    }
    let response = request
        .body(body)
        .send()
        .await
        .map_err(reqwest::Error::without_url)?;
    anyhow::ensure!(
        response.status().is_success(),
        "webhook returned HTTP {}",
        response.status()
    );
    Ok(())
}

/// Payload layout format for outbound webhooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WebhookFormat {
    /// Standard structured Warmplane JSON payload.
    #[default]
    Generic,
    /// Slack Block Kit payload with interactive blocks.
    Slack,
    /// Discord Embed with action buttons.
    Discord,
    /// Microsoft Teams Adaptive Card format.
    Teams,
}

/// Formats an outbound webhook payload based on the configured format.
pub fn format_webhook_payload(
    format: WebhookFormat,
    event_type: &str,
    data: &Value,
    callback_url: Option<&str>,
) -> Value {
    match format {
        WebhookFormat::Generic => json!({
            "event": event_type,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "data": data,
            "callback_url": callback_url,
        }),
        WebhookFormat::Slack => build_slack_payload(event_type, data, callback_url),
        WebhookFormat::Discord => build_discord_payload(event_type, data, callback_url),
        WebhookFormat::Teams => build_teams_payload(event_type, data, callback_url),
    }
}

fn approval_fields(data: &Value, pretty: bool) -> (&str, &str, &str, String) {
    let text = |key| data.get(key).and_then(Value::as_str).unwrap_or("unknown");
    let args = data.get("sanitized_args").or_else(|| data.get("args"));
    let args = args
        .and_then(|value| {
            if pretty {
                serde_json::to_string_pretty(value).ok()
            } else {
                serde_json::to_string(value).ok()
            }
        })
        .unwrap_or_else(|| "{}".to_string());
    (text("id"), text("capability_id"), text("server_id"), args)
}

fn append_slack_approval(blocks: &mut Vec<Value>, data: &Value) {
    let (ticket_id, cap_id, server_id, args_str) = approval_fields(data, true);

    blocks.push(json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Capability:*\n`{}`", cap_id)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Server:*\n`{}`", server_id)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Ticket ID:*\n`{}`", ticket_id)
                },
                {
                    "type": "mrkdwn",
                    "text": data.get("expires_at").and_then(Value::as_u64).map(|time| format!("*Expires:*\n<!date^{time}^{{time}}|{time}>")).unwrap_or_else(|| "*Expires:*\nSee Control Deck".to_owned())
                }
            ]
        }));

    blocks.push(json!({
        "type": "section",
        "text": {
            "type": "mrkdwn",
            "text": format!("*Parameters:*\n```{}```", args_str)
        }
    }));

    if let Some((approve, reject)) = approval_tokens(data) {
        blocks.push(json!({
            "type": "actions",
            "elements": [
                {"type": "button", "text": {"type": "plain_text", "text": "Approve"},
                 "style": "primary", "value": approve, "action_id": "warmplane_approve_btn"},
                {"type": "button", "text": {"type": "plain_text", "text": "Reject"},
                 "style": "danger", "value": reject, "action_id": "warmplane_reject_btn"}
            ]
        }));
    }
}

fn build_slack_payload(event_type: &str, data: &Value, callback_url: Option<&str>) -> Value {
    let title = match event_type {
        "approval.requested" => "🚨 Warmplane: Human Approval Required",
        "circuit_breaker.tripped" => "⚠️ Warmplane: Circuit Breaker Tripped",
        "policy.violation" => "🛡️ Warmplane: Security Policy Violation",
        "task.timeout" => "⏱️ Warmplane: Task Timeout",
        _ => "📢 Warmplane Notification",
    };

    let mut blocks = vec![json!({
        "type": "header",
        "text": {
            "type": "plain_text",
            "text": title,
            "emoji": true
        }
    })];

    if event_type == "approval.requested" {
        append_slack_approval(&mut blocks, data);
    } else if matches!(event_type, "undo.action_available" | "emission.held") {
        let capability = data
            .get("capability")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let actor = data
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let profile = data
            .get("profile")
            .and_then(Value::as_str)
            .unwrap_or("default");
        blocks.push(json!({
            "type": "section",
            "fields": [
                {"type": "mrkdwn", "text": format!("*Capability:*\\n`{capability}`")},
                {"type": "mrkdwn", "text": format!("*Actor:*\\n`{actor}`")},
                {"type": "mrkdwn", "text": format!("*Profile:*\\n`{profile}`")},
                {"type": "mrkdwn", "text": "*Details:*\\nSensitive arguments and snapshots are omitted."}
            ]
        }));
        if let (Some(callback), Some(token)) = (
            callback_url,
            data.get("action_token").and_then(Value::as_str),
        ) {
            let (label, action_id) = if event_type == "emission.held" {
                ("Cancel Send", "warmplane_cancel_send")
            } else {
                ("Undo Action", "warmplane_undo_action")
            };
            blocks.push(json!({"type":"actions","elements":[{
                "type":"button","text":{"type":"plain_text","text":label},
                "value":token,"action_id":action_id,
                "url":callback
            }]}));
        }
    } else {
        let details_str = serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string());
        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*Event Details:*\n```{}```", details_str)
            }
        }));
    }

    json!({
        "text": title,
        "blocks": blocks
    })
}

fn build_discord_payload(event_type: &str, data: &Value, callback_url: Option<&str>) -> Value {
    let (title, color) = match event_type {
        "approval.requested" => ("🚨 Warmplane: Human Approval Required", 0xF59E0B),
        "circuit_breaker.tripped" => ("⚠️ Warmplane: Circuit Breaker Tripped", 0xEF4444),
        "policy.violation" => ("🛡️ Warmplane: Security Policy Violation", 0xDC2626),
        _ => ("📢 Warmplane Alert", 0x3B82F6),
    };

    let mut fields = Vec::new();
    if event_type == "approval.requested" {
        let (ticket_id, cap_id, server_id, args_str) = approval_fields(data, false);

        fields.push(
            json!({ "name": "Capability", "value": format!("`{}`", cap_id), "inline": true }),
        );
        fields
            .push(json!({ "name": "Server", "value": format!("`{}`", server_id), "inline": true }));
        fields.push(
            json!({ "name": "Ticket ID", "value": format!("`{}`", ticket_id), "inline": true }),
        );
        fields.push(json!({ "name": "Arguments", "value": format!("```json\n{}\n```", args_str), "inline": false }));
    } else if matches!(event_type, "undo.action_available" | "emission.held") {
        let capability = data
            .get("capability")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        fields.push(json!({"name":"Capability","value":capability,"inline":true}));
        fields.push(json!({"name":"Actor","value":data.get("actor").and_then(Value::as_str).unwrap_or("unknown"),"inline":true}));
        fields.push(json!({"name":"Profile","value":data.get("profile").and_then(Value::as_str).unwrap_or("default"),"inline":true}));
        fields.push(json!({"name":"Details","value":"Sensitive arguments and snapshots are omitted.","inline":false}));
    }

    let embed = json!({
        "title": title,
        "color": color,
        "fields": fields,
        "footer": { "text": "Warmplane Control Plane" }
    });

    let mut payload = json!({
        "embeds": [embed]
    });

    if callback_url.is_some() {
        if event_type == "approval.requested" {
            if let Some((approve, reject)) = approval_tokens(data) {
                payload["components"] = json!([{"type":1,"components":[
                    {"type":2,"style":3,"label":"Approve","custom_id":approve},
                    {"type":2,"style":4,"label":"Reject","custom_id":reject}
                ]}]);
            }
        } else if matches!(event_type, "undo.action_available" | "emission.held") {
            if let Some(token) = data.get("action_token").and_then(Value::as_str) {
                payload["components"] = json!([{"type":1,"components":[{
                    "type":2,"style": if event_type == "emission.held" { 4 } else { 3 },
                    "label": if event_type == "emission.held" { "Cancel Send" } else { "Undo Action" },
                    "custom_id": token
                }]}]);
            }
        }
    }

    payload
}

fn build_teams_payload(event_type: &str, data: &Value, callback_url: Option<&str>) -> Value {
    let title = match event_type {
        "approval.requested" => "🚨 Warmplane: Human Approval Required",
        "circuit_breaker.tripped" => "⚠️ Warmplane: Circuit Breaker Tripped",
        _ => "📢 Warmplane Event",
    };

    let mut body = vec![json!({
        "type": "TextBlock",
        "size": "Medium",
        "weight": "Bolder",
        "text": title
    })];

    if event_type == "approval.requested" {
        let ticket_id = data.get("id").and_then(Value::as_str).unwrap_or("unknown");
        let cap_id = data
            .get("capability_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let server_id = data
            .get("server_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        body.push(json!({
            "type": "FactSet",
            "facts": [
                { "title": "Ticket ID", "value": ticket_id },
                { "title": "Capability", "value": cap_id },
                { "title": "Server", "value": server_id }
            ]
        }));
    } else if matches!(event_type, "undo.action_available" | "emission.held") {
        body.push(json!({"type":"FactSet","facts":[
            {"title":"Capability","value":data.get("capability").and_then(Value::as_str).unwrap_or("unknown")},
            {"title":"Actor","value":data.get("actor").and_then(Value::as_str).unwrap_or("unknown")},
            {"title":"Profile","value":data.get("profile").and_then(Value::as_str).unwrap_or("default")},
            {"title":"Details","value":"Sensitive arguments and snapshots are omitted."}
        ]}));
    }

    let mut card = json!({
        "type": "message",
        "attachments": [
            {
                "contentType": "application/vnd.microsoft.card.adaptive",
                "content": {
                    "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                    "type": "AdaptiveCard",
                    "version": "1.4",
                    "body": body
                }
            }
        ]
    });

    if callback_url.is_some() {
        if event_type == "approval.requested" {
            if let Some((approve, reject)) = approval_tokens(data) {
                card["attachments"][0]["content"]["actions"] = json!([
                    {"type":"Action.Submit","title":"Approve","data":{"action_token":approve}},
                    {"type":"Action.Submit","title":"Reject","data":{"action_token":reject}}
                ]);
            }
        } else if matches!(event_type, "undo.action_available" | "emission.held") {
            if let Some(token) = data.get("action_token").and_then(Value::as_str) {
                card["attachments"][0]["content"]["actions"] = json!([{
                    "type":"Action.Submit",
                    "title": if event_type == "emission.held" { "Cancel Send" } else { "Undo Action" },
                    "data":{"action_token":token}
                }]);
            }
        }
    }

    card
}

fn approval_tokens(data: &Value) -> Option<(&str, &str)> {
    Some((
        data.get("approve_token")?.as_str()?,
        data.get("reject_token")?.as_str()?,
    ))
}

/// Verifies an HMAC-SHA256 signature for incoming webhook payloads.
pub fn verify_signature(
    secret: &str,
    body: &str,
    signature_header: &str,
    timestamp: Option<&str>,
) -> bool {
    let sig_to_check = if let Some(stripped) = signature_header.strip_prefix("sha256=") {
        stripped
    } else if let Some(stripped) = signature_header.strip_prefix("v0=") {
        stripped
    } else {
        signature_header
    };

    let signature = match hex::decode(sig_to_check) {
        Ok(signature) => signature,
        Err(_) => return false,
    };
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };
    if let Some(ts) = timestamp {
        mac.update(format!("{}.{}", ts, body).as_bytes());
    } else {
        mac.update(body.as_bytes());
    }
    mac.verify_slice(&signature).is_ok()
}

/// Creates a short bearer token for one signed ChatOps action.
///
/// The target and expiry are included in the token. Tenant and role are bound into the HMAC
/// input and must be recovered from trusted server state when verifying it.
///
/// # Arguments
/// * `secret` - Webhook secret used as the HMAC key.
/// * `action` - Purpose-bound action name, such as `undo` or `cancel`.
/// * `target_id` - One persisted undo or held-task identifier.
/// * `tenant_id` - Trusted tenant that owns the target.
/// * `role` - Configured RBAC role for the action.
/// * `expires_at_ms` - Absolute expiry in Unix milliseconds.
///
/// # Returns
/// An opaque target, expiry, and truncated HMAC token, or `None` if HMAC initialization fails.
pub fn sign_action_token(
    secret: &str,
    action: &str,
    target_id: &str,
    tenant_id: &str,
    role: &str,
    expires_at_ms: u64,
) -> Option<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(format!("{action}|{target_id}|{tenant_id}|{role}|{expires_at_ms}").as_bytes());
    let tag = mac.finalize().into_bytes();
    Some(format!(
        "{target_id}.{expires_at_ms}.{}",
        hex::encode(&tag[..16])
    ))
}

/// Verifies a ChatOps action token using a constant-time comparison.
///
/// # Arguments
/// * `secret` - Webhook secret used as the HMAC key.
/// * `action` - Expected purpose-bound action name.
/// * `token` - Opaque token received from a platform callback.
/// * `tenant_id` - Tenant recovered from the persisted target.
/// * `role` - Current configured ChatOps role.
/// * `now_ms` - Current Unix time in milliseconds.
///
/// # Returns
/// The target identifier only when the signature and expiry are valid.
pub fn verify_action_token(
    secret: &str,
    action: &str,
    token: &str,
    tenant_id: &str,
    role: &str,
    now_ms: u64,
) -> Option<String> {
    let (target_and_expiry, provided_tag) = token.rsplit_once('.')?;
    let (target_id, expiry_text) = target_and_expiry.rsplit_once('.')?;
    let expires_at_ms = expiry_text.parse::<u64>().ok()?;
    if target_id.is_empty() || expires_at_ms <= now_ms || provided_tag.len() != 32 {
        return None;
    }
    let provided_tag = hex::decode(provided_tag).ok()?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(format!("{action}|{target_id}|{tenant_id}|{role}|{expires_at_ms}").as_bytes());
    let expected_tag = mac.finalize().into_bytes();
    if expected_tag[..16]
        .ct_eq(provided_tag.as_slice())
        .unwrap_u8()
        != 1
    {
        return None;
    }
    Some(target_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_block_generation() {
        let data = json!({
            "id": "appr-123",
            "capability_id": "db.drop_database",
        "approve_token": "approve-fixture-token",
        "reject_token": "reject-fixture-token",
            "server_id": "production_db",
            "sanitized_args": { "db": "users" }
        });

        let payload = format_webhook_payload(
            WebhookFormat::Slack,
            "approval.requested",
            &data,
            Some("http://127.0.0.1:9090"),
        );

        assert!(payload.get("blocks").is_some());
        let blocks = payload["blocks"].as_array().unwrap();
        assert!(blocks.len() >= 3);
        assert_eq!(
            blocks[0]["text"]["text"],
            "🚨 Warmplane: Human Approval Required"
        );
    }

    #[test]
    fn test_signature_verification() {
        let secret = "super-secret-key-123";
        let body = r#"{"action":"approve","ticket_id":"appr-123"}"#;
        let ts = "1724700000";

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("{}.{}", ts, body).as_bytes());
        let valid_sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        assert!(verify_signature(secret, body, &valid_sig, Some(ts)));
        assert!(!verify_signature(secret, body, "sha256=invalid", Some(ts)));
    }

    #[test]
    fn action_tokens_are_target_bound_expiring_and_tamper_evident() {
        let token =
            sign_action_token("secret", "undo", "undo_1", "tenant-a", "operator", 10_000).unwrap();
        assert_eq!(
            verify_action_token("secret", "undo", &token, "tenant-a", "operator", 9_999).as_deref(),
            Some("undo_1")
        );
        assert!(verify_action_token("wrong", "undo", &token, "tenant-a", "operator", 1).is_none());
        assert!(
            verify_action_token("secret", "cancel", &token, "tenant-a", "operator", 1).is_none()
        );
        assert!(verify_action_token("secret", "undo", &token, "tenant-b", "operator", 1).is_none());
        assert!(
            verify_action_token("secret", "undo", &token, "tenant-a", "operator", 10_000).is_none()
        );
    }

    #[test]
    fn action_cards_omit_arguments_and_include_signed_button_values() {
        let payload = format_webhook_payload(
            WebhookFormat::Slack,
            "undo.action_available",
            &json!({
                "capability": "tickets.update",
                "actor": "operator-a",
                "action_token": "undo_1.1000.deadbeef",
                "args": {"secret": "must-not-appear"},
                "snapshot": {"secret": "must-not-appear"}
            }),
            Some("https://warmplane.example/v1/webhooks/actions"),
        );
        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(serialized.contains("undo_1.1000.deadbeef"));
        assert!(!serialized.contains("must-not-appear"));
        let discord = format_webhook_payload(
            WebhookFormat::Discord,
            "emission.held",
            &json!({"capability":"mail.send","action_token":"task_1.1000.deadbeef"}),
            Some("https://warmplane.example/v1/webhooks/actions"),
        );
        assert_eq!(
            discord["components"][0]["components"][0]["custom_id"],
            "task_1.1000.deadbeef"
        );
        let teams = format_webhook_payload(
            WebhookFormat::Teams,
            "undo.action_available",
            &json!({"capability":"tickets.update","action_token":"undo_1.1000.deadbeef"}),
            Some("https://warmplane.example/v1/webhooks/actions"),
        );
        assert_eq!(
            teams["attachments"][0]["content"]["actions"][0]["data"]["action_token"],
            "undo_1.1000.deadbeef"
        );
    }
}
