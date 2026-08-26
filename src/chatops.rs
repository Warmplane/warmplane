// Rust guideline compliant 2026-08-26

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

type HmacSha256 = Hmac<Sha256>;

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
        let ticket_id = data.get("id").and_then(Value::as_str).unwrap_or("unknown");
        let cap_id = data
            .get("capability_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let server_id = data
            .get("server_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let args_str = data
            .get("sanitized_args")
            .or_else(|| data.get("args"))
            .map(|a| serde_json::to_string_pretty(a).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

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
                    "text": "*Expires In:*\n300 seconds"
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

        if let Some(cb_url) = callback_url {
            blocks.push(json!({
                "type": "actions",
                "elements": [
                    {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "✅ Approve",
                            "emoji": true
                        },
                        "style": "primary",
                        "value": json!({
                            "action": "approve",
                            "ticket_id": ticket_id,
                            "callback_url": cb_url
                        }).to_string(),
                        "action_id": "warmplane_approve_btn"
                    },
                    {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "❌ Reject",
                            "emoji": true
                        },
                        "style": "danger",
                        "value": json!({
                            "action": "reject",
                            "ticket_id": ticket_id,
                            "callback_url": cb_url
                        }).to_string(),
                        "action_id": "warmplane_reject_btn"
                    }
                ]
            }));
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
        let ticket_id = data.get("id").and_then(Value::as_str).unwrap_or("unknown");
        let cap_id = data
            .get("capability_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let server_id = data
            .get("server_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let args_str = data
            .get("sanitized_args")
            .or_else(|| data.get("args"))
            .map(|a| serde_json::to_string(a).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        fields.push(
            json!({ "name": "Capability", "value": format!("`{}`", cap_id), "inline": true }),
        );
        fields
            .push(json!({ "name": "Server", "value": format!("`{}`", server_id), "inline": true }));
        fields.push(
            json!({ "name": "Ticket ID", "value": format!("`{}`", ticket_id), "inline": true }),
        );
        fields.push(json!({ "name": "Arguments", "value": format!("```json\n{}\n```", args_str), "inline": false }));
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

    if let Some(cb_url) = callback_url {
        if event_type == "approval.requested" {
            let ticket_id = data.get("id").and_then(Value::as_str).unwrap_or("unknown");
            payload["components"] = json!([
                {
                    "type": 1,
                    "components": [
                        {
                            "type": 2,
                            "style": 3,
                            "label": "Approve",
                            "custom_id": format!("approve:{}", ticket_id)
                        },
                        {
                            "type": 2,
                            "style": 4,
                            "label": "Reject",
                            "custom_id": format!("reject:{}", ticket_id)
                        },
                        {
                            "type": 2,
                            "style": 5,
                            "label": "Open Control Deck",
                            "url": cb_url
                        }
                    ]
                }
            ]);
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

    if let Some(cb_url) = callback_url {
        if event_type == "approval.requested" {
            let ticket_id = data.get("id").and_then(Value::as_str).unwrap_or("unknown");
            card["attachments"][0]["content"]["actions"] = json!([
                {
                    "type": "Action.Submit",
                    "title": "Approve",
                    "data": {
                        "action": "approve",
                        "ticket_id": ticket_id
                    }
                },
                {
                    "type": "Action.Submit",
                    "title": "Reject",
                    "data": {
                        "action": "reject",
                        "ticket_id": ticket_id
                    }
                },
                {
                    "type": "Action.OpenUrl",
                    "title": "Open Control Deck",
                    "url": cb_url
                }
            ]);
        }
    }

    card
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

    let expected_mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mut mac) => {
            if let Some(ts) = timestamp {
                mac.update(format!("{}.{}", ts, body).as_bytes());
            } else {
                mac.update(body.as_bytes());
            }
            hex::encode(mac.finalize().into_bytes())
        }
        Err(_) => return false,
    };

    sig_to_check.eq_ignore_ascii_case(&expected_mac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_block_generation() {
        let data = json!({
            "id": "appr-123",
            "capability_id": "db.drop_database",
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
}
