// Rust guideline compliant 2026-08-26

//! Integration tests for multi-platform ChatOps formatting, webhook dispatch, and bidirectional callback ingestion.

use serde_json::json;
use warmplane::chatops::{format_webhook_payload, verify_signature, WebhookFormat};

#[test]
fn test_slack_block_kit_formatting() {
    let data = json!({
        "id": "appr-1724700000-99",
        "capability_id": "postgres.drop_table",
        "server_id": "production_db",
        "sanitized_args": {
            "table": "users"
        }
    });

    let payload = format_webhook_payload(
        WebhookFormat::Slack,
        "approval.requested",
        &data,
        Some("https://warmplane.internal/v1/webhooks/callbacks"),
    );

    let blocks = payload
        .get("blocks")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    assert!(blocks.len() >= 3);
    assert_eq!(blocks[0]["type"], "header");
    assert_eq!(
        blocks[0]["text"]["text"],
        "🚨 Warmplane: Human Approval Required"
    );

    let action_block = blocks.iter().find(|b| b["type"] == "actions").unwrap();
    let elements = action_block["elements"].as_array().unwrap();
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0]["text"]["text"], "✅ Approve");
    assert_eq!(elements[1]["text"]["text"], "❌ Reject");
}

#[test]
fn test_discord_embed_formatting() {
    let data = json!({
        "id": "appr-1724700000-99",
        "capability_id": "docker.stop_container",
        "server_id": "local_docker",
        "sanitized_args": {
            "container_id": "c-101"
        }
    });

    let payload = format_webhook_payload(
        WebhookFormat::Discord,
        "approval.requested",
        &data,
        Some("https://warmplane.internal/ui"),
    );

    let embeds = payload
        .get("embeds")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    assert_eq!(embeds.len(), 1);
    assert_eq!(embeds[0]["title"], "🚨 Warmplane: Human Approval Required");

    let components = payload
        .get("components")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    assert_eq!(components.len(), 1);
    let buttons = components[0]["components"].as_array().unwrap();
    assert_eq!(buttons.len(), 3);
}

#[test]
fn test_teams_adaptive_card_formatting() {
    let data = json!({
        "id": "appr-1724700000-99",
        "capability_id": "payments.refund",
        "server_id": "stripe",
        "sanitized_args": {
            "charge_id": "ch_123"
        }
    });

    let payload = format_webhook_payload(
        WebhookFormat::Teams,
        "approval.requested",
        &data,
        Some("https://warmplane.internal/ui"),
    );

    let attachments = payload
        .get("attachments")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(
        attachments[0]["contentType"],
        "application/vnd.microsoft.card.adaptive"
    );
}

#[test]
fn test_hmac_sha256_verification() {
    let secret = "test-webhook-secret-xyz";
    let body = r#"{"action":"approve","ticket_id":"appr-1724700000-99"}"#;
    let ts = "1724700000";

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", ts, body).as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());

    assert!(verify_signature(
        secret,
        body,
        &format!("sha256={}", sig),
        Some(ts)
    ));
    assert!(verify_signature(
        secret,
        body,
        &format!("v0={}", sig),
        Some(ts)
    ));
    assert!(!verify_signature(
        secret,
        body,
        "sha256=invalid_hash",
        Some(ts)
    ));
    assert!(!verify_signature(
        "wrong-secret",
        body,
        &format!("sha256={}", sig),
        Some(ts)
    ));
}
