// Rust guideline compliant 2026-11-16

//! REST API endpoints for incoming ChatOps callbacks and webhook simulation.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{de, Deserialize};
use serde_json::json;

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

/// Returns the migration response for retired approval callbacks.
pub async fn handle_webhook_callback() -> impl IntoResponse {
    (
        StatusCode::GONE,
        Json(json!({
            "ok": false,
            "error": {"code": "LEGACY_CALLBACK_RETIRED", "message": "Use signed approval actions"},
            "callback_url": "/v1/webhooks/actions",
        })),
    )
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
    headers: HeaderMap,
    body_bytes: Bytes,
) -> impl IntoResponse {
    let payload = match decode_action_payload(&body_bytes) {
        Ok(payload) => payload,
        Err(message) => return action_error(StatusCode::BAD_REQUEST, "INVALID_PAYLOAD", message),
    };
    let Some(token) = payload.action_token() else {
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
    let token = token.as_ref();
    if target_id.starts_with("appr-") {
        return signed_approval_callback(
            state,
            headers,
            body_bytes,
            payload.user.as_ref().and_then(|user| user.id.as_deref()),
            target_id,
            token,
        )
        .await;
    }
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
                token,
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
            token,
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

pub(crate) const MAX_ACTION_BODY_BYTES: usize = 64 * 1024;
const MAX_ACTION_TOKEN_BYTES: usize = 512;

#[derive(Deserialize)]
struct ActionPayload {
    action_token: Option<String>,
    data: Option<ActionData>,
    #[serde(default, deserialize_with = "first_platform_action")]
    actions: Option<PlatformAction>,
    user: Option<PlatformUser>,
}

#[derive(Deserialize)]
struct ActionData {
    action_token: Option<String>,
    custom_id: Option<String>,
}

#[derive(Deserialize)]
struct PlatformAction {
    value: Option<String>,
}

#[derive(Deserialize)]
struct PlatformUser {
    id: Option<String>,
}

impl ActionPayload {
    fn action_token(&self) -> Option<std::borrow::Cow<'_, str>> {
        let plain = self
            .action_token
            .as_deref()
            .or_else(|| self.data.as_ref().and_then(ActionData::token));
        plain
            .map(std::borrow::Cow::Borrowed)
            .or_else(|| unwrap_platform_token(self.actions.as_ref()?.value.as_deref()?))
    }
}

fn first_platform_action<'de, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<PlatformAction>, D::Error> {
    struct FirstAction;
    impl<'de> de::Visitor<'de> for FirstAction {
        type Value = Option<PlatformAction>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an array of platform actions")
        }

        fn visit_seq<S: de::SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
            let first = seq.next_element()?;
            while seq.next_element::<de::IgnoredAny>()?.is_some() {}
            Ok(first)
        }
    }
    deserializer.deserialize_seq(FirstAction)
}

impl ActionData {
    fn token(&self) -> Option<&str> {
        self.action_token.as_deref().or(self.custom_id.as_deref())
    }
}

fn decode_action_payload(bytes: &[u8]) -> Result<ActionPayload, &'static str> {
    let body = std::str::from_utf8(bytes).map_err(|_| "Invalid UTF-8 payload")?;
    let decoded;
    let (json, message) = if let Some(form) = body.strip_prefix("payload=") {
        decoded = url_decode_str(form).ok_or("Invalid platform payload")?;
        (decoded.as_slice(), "Invalid platform payload")
    } else {
        (bytes, "Invalid JSON payload")
    };
    parse_action_json(json).ok_or(message)
}

fn parse_action_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    if bytes.len() > MAX_ACTION_BODY_BYTES {
        return None;
    }
    let mut input = serde_json::Deserializer::from_slice(bytes);
    let payload = T::deserialize(&mut input).ok()?;
    input.end().ok()?;
    Some(payload)
}

async fn signed_approval_callback(
    state: AppState,
    headers: HeaderMap,
    raw: Bytes,
    slack_user: Option<&str>,
    ticket: String,
    token: &str,
) -> axum::response::Response {
    use crate::approval_chatops::{resolve_signed, slack_principal, SignedDecisionError};

    let config = state.policy.read().await.webhook.clone();
    let Some(config) = config else {
        return action_error(
            StatusCode::UNAUTHORIZED,
            "ACTION_UNAVAILABLE",
            "Signed approvals are not configured",
        );
    };
    if config.format == Some(crate::chatops::WebhookFormat::Slack) {
        let Some(principal) =
            slack_principal(&config, &headers, &raw, slack_user, unix_time_ms() / 1000)
        else {
            return action_error(
                StatusCode::UNAUTHORIZED,
                "INVALID_SLACK_SIGNATURE",
                "Slack callback authentication failed",
            );
        };
        let token = token.to_owned();
        tokio::spawn(async move {
            if resolve_signed(&state, &config, &ticket, &token, Some(principal))
                .await
                .is_err()
            {
                tracing::warn!(ticket_id = %ticket, "Slack approval action did not resolve");
            }
        });
        return accepted_action();
    }

    match resolve_signed(&state, &config, &ticket, token, None).await {
        Ok(receipt) => Json(json!({"ok": true, "receipt": receipt})).into_response(),
        Err(SignedDecisionError::Invalid) => action_error(
            StatusCode::UNAUTHORIZED,
            "INVALID_ACTION_TOKEN",
            "Action token is invalid or expired",
        ),
        Err(SignedDecisionError::Decision(error)) => {
            crate::http_v1::approvals_api::decision_error(error)
        }
    }
}

fn accepted_action() -> axum::response::Response {
    StatusCode::OK.into_response()
}

fn parse_action_token(raw: &str) -> Option<(String, u64)> {
    if raw.len() > MAX_ACTION_TOKEN_BYTES {
        return None;
    }
    let (head, signature) = raw.rsplit_once('.')?;
    let (target, expiry) = head.rsplit_once('.')?;
    if signature.len() != 32 || !signature.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    if !valid_action_target(target) {
        return None;
    }
    Some((target.to_owned(), expiry.parse().ok()?))
}

fn valid_action_target(target: &str) -> bool {
    !target.is_empty()
        && target
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[derive(Deserialize)]
struct NestedAction {
    action_token: Option<String>,
}

fn unwrap_platform_token(token: &str) -> Option<std::borrow::Cow<'_, str>> {
    if token.len() > MAX_ACTION_TOKEN_BYTES {
        return None;
    }
    if token.trim_start().starts_with('{') {
        let value: NestedAction = parse_action_json(token.as_bytes())?;
        return value.action_token.map(std::borrow::Cow::Owned);
    }
    Some(std::borrow::Cow::Borrowed(token))
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

fn url_decode_str(input: &str) -> Option<Vec<u8>> {
    let mut result = Vec::with_capacity(input.len());
    let mut bytes = input.bytes();
    while let Some(byte) = bytes.next() {
        let decoded = match byte {
            b'+' => b' ',
            b'%' => {
                let mut decoded = [0];
                hex::decode_to_slice([bytes.next()?, bytes.next()?], &mut decoded).ok()?;
                decoded[0]
            }
            other => other,
        };
        result.push(decoded);
    }
    Some(result)
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
    use serde_json::Value;
    use tempfile::NamedTempFile;

    #[test]
    fn action_payload_requires_the_expected_shape_and_consumes_the_body() {
        for body in [
            &b"[]"[..],
            br#"{"action_token":42}"#,
            br#"{"action_token":"one","action_token":"two"}"#,
            br#"{} {}"#,
            b"payload=%GG",
        ] {
            assert!(decode_action_payload(body).is_err());
        }

        let slack = decode_action_payload(b"payload=%7B%22user%22%3A%7B%22id%22%3A%22U123%22%7D%2C%22actions%22%3A%5B%7B%22value%22%3A%22fixture%22%7D%5D%7D").unwrap();
        assert_eq!(slack.action_token().as_deref(), Some("fixture"));
        assert_eq!(slack.user.unwrap().id.as_deref(), Some("U123"));

        let nested =
            decode_action_payload(br#"{"actions":[{"value":"{\"action_token\":\"fixture\"}"}]}"#)
                .unwrap();
        assert_eq!(nested.action_token().as_deref(), Some("fixture"));

        let first = decode_action_payload(
            br#"{"actions":[{"value":"first"},{"value":{"platform_extension":true}}]}"#,
        )
        .unwrap();
        assert_eq!(first.action_token().as_deref(), Some("first"));
    }

    #[tokio::test]
    async fn action_route_rejects_oversized_bodies() {
        use axum::body::Body;
        use tower::ServiceExt;
        let response = crate::daemon::server::build_router(AppState::builder().build())
            .oneshot(
                axum::http::Request::post("/v1/webhooks/actions")
                    .body(Body::from(vec![b' '; MAX_ACTION_BODY_BYTES + 1]))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

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
                HeaderMap::new(),
                Bytes::from(json!({"action_token": token}).to_string()),
            )
            .await
            .into_response();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert!(server_rx.try_recv().is_err());
        }
        let body = Bytes::from(json!({"action_token": token}).to_string());
        let response = handle_webhook_action(State(state.clone()), HeaderMap::new(), body.clone())
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
        let replay = handle_webhook_action(State(state), HeaderMap::new(), body)
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
