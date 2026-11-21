// Rust guideline compliant 2026-11-20

//! Signed approval tokens, Slack request verification, and delivery receipts.

use axum::http::HeaderMap;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::{
    approvals::{
        ApprovalActor, Decision, DecisionReceipt, DecisionRequest, PendingApproval, ReceiptDelivery,
    },
    chatops::{format_webhook_payload, post_signed_webhook, WebhookFormat},
    config::WebhookConfig,
    daemon::AppState,
    rbac::RbacConfig,
};

pub(crate) const REQUESTED: &str = "approval.requested";
const RECEIPT: &str = "approval.receipt";
const SLACK_MAX_SKEW_SECS: u64 = 300;

pub(crate) fn validate_approval_webhook(
    cfg: &WebhookConfig,
    rbac: Option<&RbacConfig>,
) -> anyhow::Result<()> {
    if !cfg.events.iter().any(|event| event == REQUESTED) {
        return Ok(());
    }
    if !signed_callback_url(cfg)
        && cfg.approval_tenant.is_none()
        && cfg.slack_signing_secret.is_none()
        && cfg.slack_signing_secret_env.is_none()
    {
        // Existing unsigned destinations remain readable during migration.
        return Ok(());
    }
    anyhow::ensure!(
        cfg.secret.is_some() || cfg.secret_env.is_some(),
        "policy.webhook.secret or secretEnv is required for signed approval cards"
    );
    anyhow::ensure!(
        signed_callback_url(cfg),
        "signed approval cards require a callbackUrl ending in /v1/webhooks/actions"
    );
    let role = delegated_role(cfg)
        .ok_or_else(|| anyhow::anyhow!("signed approval cards require actionRole"))?;
    if let Some(rbac) = rbac.filter(|rbac| rbac.enabled) {
        anyhow::ensure!(
            cfg.approval_tenant
                .as_deref()
                .is_some_and(|tenant| !tenant.trim().is_empty()),
            "signed approval cards under RBAC require approvalTenant"
        );
        anyhow::ensure!(
            rbac.roles
                .get(role)
                .is_some_and(|role| role.resolve_approvals),
            "approval actionRole must grant resolveApprovals under RBAC"
        );
    }
    anyhow::ensure!(
        cfg.format != Some(WebhookFormat::Slack)
            || cfg.slack_signing_secret.is_some()
            || cfg.slack_signing_secret_env.is_some(),
        "Slack approval cards require slackSigningSecret or slackSigningSecretEnv"
    );
    Ok(())
}

pub(crate) fn subscribed(cfg: &WebhookConfig) -> bool {
    cfg.events.iter().any(|event| event == REQUESTED) && signed_callback_url(cfg)
}

fn signed_callback_url(cfg: &WebhookConfig) -> bool {
    cfg.callback_url
        .as_deref()
        .and_then(|url| reqwest::Url::parse(url).ok())
        .is_some_and(|url| {
            matches!(url.scheme(), "http" | "https") && url.path() == "/v1/webhooks/actions"
        })
}

fn delegated_role(cfg: &WebhookConfig) -> Option<&str> {
    cfg.action_role
        .as_deref()
        .filter(|role| !role.trim().is_empty())
}

fn delegated_tenant(cfg: &WebhookConfig) -> &str {
    cfg.approval_tenant.as_deref().unwrap_or("default")
}

pub(crate) fn issue_card(cfg: WebhookConfig, ticket: PendingApproval) {
    tokio::spawn(async move {
        if let Err(error) = send_card(&cfg, &ticket).await {
            tracing::warn!(%error, ticket_id = %ticket.id, "approval card delivery failed");
        }
    });
}

async fn send_card(cfg: &WebhookConfig, ticket: &PendingApproval) -> anyhow::Result<()> {
    let Some(owner) = ticket
        .ownership
        .as_ref()
        .filter(|owner| subscribed(cfg) && owner.tenant_id == delegated_tenant(cfg))
    else {
        return Ok(());
    };
    let secret = cfg
        .resolve_secret()
        .ok_or_else(|| anyhow::anyhow!("approval signing secret unavailable"))?;
    let role =
        delegated_role(cfg).ok_or_else(|| anyhow::anyhow!("approval delegation unavailable"))?;
    let data = json!({
        "id": ticket.id,
        "capability_id": ticket.capability_id,
        "server_id": ticket.server_id,
        "sanitized_args": ticket.sanitized_args,
        "expires_at": ticket.expires_at,
        "tenant_id": owner.tenant_id,
        "approve_token": sign_approval_token(ticket, &secret, role, Decision::Approve),
        "reject_token": sign_approval_token(ticket, &secret, role, Decision::Reject),
    });
    let payload = format_webhook_payload(
        cfg.format.unwrap_or_default(),
        REQUESTED,
        &data,
        cfg.callback_url.as_deref(),
    );
    post_signed_webhook(cfg, REQUESTED, &payload, &secret).await
}

fn sign_approval_token(
    ticket: &PendingApproval,
    secret: &str,
    role: &str,
    choice: Decision,
) -> Option<String> {
    let owner = ticket.ownership.as_ref()?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    let binding = json!([
        "warmplane-approval-v1",
        choice,
        ticket.id,
        owner.tenant_id,
        role,
        ticket.expires_at,
        owner.argument_digest,
        owner.tool_definition,
    ]);
    mac.update(&serde_json::to_vec(&binding).ok()?);
    let tag = mac.finalize().into_bytes();
    Some(format!(
        "{}.{}.{}",
        ticket.id,
        ticket.expires_at.saturating_mul(1000),
        hex::encode(&tag[..16])
    ))
}

pub(crate) fn slack_principal(
    cfg: &WebhookConfig,
    headers: &HeaderMap,
    body: &[u8],
    slack_user: Option<&str>,
    now: u64,
) -> Option<String> {
    verify_slack_signature(cfg, headers, body, now)?;
    let slack_user = slack_user?;
    if slack_user.is_empty()
        || slack_user.len() > 100
        || !slack_user.bytes().all(|byte| byte.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(format!("slack:{slack_user}"))
}

fn verify_slack_signature(
    cfg: &WebhookConfig,
    headers: &HeaderMap,
    body: &[u8],
    now: u64,
) -> Option<()> {
    let timestamp = headers.get("x-slack-request-timestamp")?.to_str().ok()?;
    if timestamp.parse::<u64>().ok()?.abs_diff(now) > SLACK_MAX_SKEW_SECS {
        return None;
    }
    let signature = headers
        .get("x-slack-signature")?
        .to_str()
        .ok()?
        .strip_prefix("v0=")?;
    if signature.len() != 64 {
        return None;
    }
    let signature = hex::decode(signature).ok()?;
    let secret = cfg.resolve_slack_signing_secret()?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(format!("v0:{timestamp}:").as_bytes());
    mac.update(body);
    mac.verify_slice(&signature).ok()
}

pub(crate) async fn resolve_signed(
    state: &AppState,
    cfg: &WebhookConfig,
    id: &str,
    token: &str,
    principal: Option<String>,
) -> Result<DecisionReceipt, SignedDecisionError> {
    let role = delegated_role(cfg)
        .filter(|role| subscribed(cfg) && state.rbac_engine.can_resolve_approvals(role))
        .ok_or(SignedDecisionError::Invalid)?;
    state
        .approval_registry
        .validate_webhook_storage(Some(cfg))
        .map_err(|_| SignedDecisionError::Decision(crate::approvals::DecisionError::Persistence))?;
    let secret = cfg.resolve_secret().ok_or(SignedDecisionError::Invalid)?;
    let ticket = state
        .approval_registry
        .get(id)
        .await
        .ok_or(SignedDecisionError::Invalid)?;
    let owner = ticket
        .ownership
        .as_ref()
        .filter(|owner| owner.tenant_id == delegated_tenant(cfg))
        .ok_or(SignedDecisionError::Invalid)?;
    let choice = [Decision::Approve, Decision::Reject]
        .into_iter()
        .find(|choice| {
            sign_approval_token(&ticket, &secret, role, *choice)
                .is_some_and(|expected| bool::from(expected.as_bytes().ct_eq(token.as_bytes())))
        })
        .ok_or(SignedDecisionError::Invalid)?;
    let actor = ApprovalActor {
        tenant: owner.tenant_id.clone(),
        principal: principal.unwrap_or_else(|| format!("chatops:{role}")),
        role: role.to_owned(),
        authorization_expires_at: None,
    };
    state
        .decide_approval(id, &actor, DecisionRequest::unchanged(choice), true)
        .await
        .map_err(SignedDecisionError::Decision)
}

#[derive(Debug)]
pub(crate) enum SignedDecisionError {
    Invalid,
    Decision(crate::approvals::DecisionError),
}

impl AppState {
    pub(crate) async fn trust_controls_active(&self) -> bool {
        let policy = self.policy.read().await;
        policy.result_sanitization.is_some()
            || policy.webhook.as_ref().is_some_and(subscribed)
            || self.tool_pins.has_required_servers()
    }

    pub(crate) fn schedule_approval_receipt(&self, tenant: String, receipt: DecisionReceipt) {
        let state = self.clone();
        tokio::spawn(async move {
            let Some(cfg) = state
                .policy
                .read()
                .await
                .webhook
                .clone()
                .filter(|cfg| subscribed(cfg) && delegated_tenant(cfg) == tenant)
            else {
                return;
            };
            let delivery = dispatch_receipt(&cfg, &receipt).await;
            if state
                .approval_registry
                .record_receipt_delivery(&receipt.ticket_id, delivery)
                .await
                .is_err()
            {
                tracing::error!(ticket_id = %receipt.ticket_id, "could not record approval receipt delivery");
            }
        });
    }
}

async fn dispatch_receipt(cfg: &WebhookConfig, receipt: &DecisionReceipt) -> ReceiptDelivery {
    let Some(secret) = cfg.resolve_secret() else {
        return ReceiptDelivery::Failed;
    };
    let data = json!(receipt);
    let payload = format_webhook_payload(cfg.format.unwrap_or_default(), RECEIPT, &data, None);
    match post_signed_webhook(cfg, RECEIPT, &payload, &secret).await {
        Ok(()) => ReceiptDelivery::Delivered,
        Err(error) => {
            tracing::warn!(%error, ticket_id = %receipt.ticket_id, "approval receipt delivery failed");
            ReceiptDelivery::Failed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        Router,
    };
    use serde_json::Value;
    use std::time::{Duration, Instant};
    use tower::ServiceExt;

    use crate::{
        approvals::{
            ApprovalOwnership, ApprovalRegistry, ApprovalResolution, CreateApprovalRequest,
        },
        daemon::{server::build_router, Policy},
        rbac::RbacEngine,
    };

    const SIGNING_KEY: &str = "fixture-approval-secret";
    const SLACK_KEY: &str = "separate-slack-fixture-signing-secret";

    fn webhook(format: WebhookFormat) -> WebhookConfig {
        WebhookConfig {
            url: "http://127.0.0.1:9/decisions".to_owned(),
            secret: Some(SIGNING_KEY.to_owned()),
            format: Some(format),
            events: vec![REQUESTED.to_owned()],
            callback_url: Some("https://warmplane.example/v1/webhooks/actions".to_owned()),
            action_role: Some("reviewer".to_owned()),
            approval_tenant: Some("tenant-a".to_owned()),
            slack_signing_secret: Some(SLACK_KEY.to_owned()),
            ..Default::default()
        }
    }

    fn state(config: WebhookConfig, registry: ApprovalRegistry) -> AppState {
        let rbac = serde_json::from_value(json!({
            "enabled": true,
            "defaultRole": "anonymous",
            "roles": { "reviewer": { "resolveApprovals": true } },
            "tokens": {
                "fixture-a": { "role": "reviewer", "tenantId": "tenant-a", "actorId": "alex" },
                "fixture-b": { "role": "reviewer", "tenantId": "tenant-b", "actorId": "blair" }
            }
        }))
        .unwrap();
        AppState::builder()
            .policy(Policy {
                webhook: Some(config),
                ..Default::default()
            })
            .rbac_engine(RbacEngine::new(Some(rbac)))
            .approval_registry(registry)
            .build()
    }

    fn persistent_registry() -> (tempfile::TempDir, ApprovalRegistry) {
        let directory = tempfile::tempdir().unwrap();
        let registry =
            ApprovalRegistry::open_or_create(directory.path().join("approvals.json")).unwrap();
        (directory, registry)
    }

    async fn ticket(
        state: &AppState,
        tenant: &str,
    ) -> (
        PendingApproval,
        tokio::sync::oneshot::Receiver<ApprovalResolution>,
    ) {
        let args = json!({"target": "fictional fixture"});
        let owner = ApprovalOwnership::new(tenant, None, &args, Some("pin-revision"));
        let (id, waiter) = state
            .approval_registry
            .create_scoped_approval(
                CreateApprovalRequest {
                    capability_id: "fixture.save".to_owned(),
                    server_id: "fixture".to_owned(),
                    args,
                    sanitized_args: json!({"target": "fixture"}),
                    request_id: None,
                    context: None,
                    timeout_secs: 120,
                    webhook: None,
                },
                Some(owner),
            )
            .await;
        (state.approval_registry.get(&id).await.unwrap(), waiter)
    }

    async fn post(router: &Router, body: Value) -> (StatusCode, Value) {
        let request = Request::post("/v1/webhooks/actions")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let value =
            serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
        (status, value)
    }

    #[tokio::test]
    async fn teams_relay_replays_one_receipt_and_rejects_conflicts_and_cross_tenant_tokens() {
        let (directory, registry) = persistent_registry();
        let state = state(webhook(WebhookFormat::Teams), registry);
        let router = build_router(state.clone());
        let (ticket_a, waiter) = ticket(&state, "tenant-a").await;
        let approve =
            sign_approval_token(&ticket_a, SIGNING_KEY, "reviewer", Decision::Approve).unwrap();
        let reject =
            sign_approval_token(&ticket_a, SIGNING_KEY, "reviewer", Decision::Reject).unwrap();
        let card = format_webhook_payload(
            WebhookFormat::Teams,
            REQUESTED,
            &json!({
                "id": ticket_a.id,
                "approve_token": approve, "reject_token": reject,
            }),
            Some("https://warmplane.example/v1/webhooks/actions"),
        );
        let choice = card
            .pointer("/attachments/0/content/actions/0/data")
            .unwrap();
        let body =
            json!({"data": choice, "operator": "spoofed", "modified_args": {"target":"different"}});
        let (one, two) = tokio::join!(post(&router, body.clone()), post(&router, body));
        assert_eq!(one.0, StatusCode::OK, "{one:?}");
        assert_eq!(one, two);
        assert_eq!(one.1["receipt"]["principal"], "chatops:reviewer");
        let reopened =
            ApprovalRegistry::open_or_create(directory.path().join("approvals.json")).unwrap();
        let receipt = reopened
            .get(&ticket_a.id)
            .await
            .unwrap()
            .decision_receipt
            .unwrap();
        assert_eq!(json!(receipt), one.1["receipt"]);
        assert!(matches!(
            waiter.await.unwrap(),
            ApprovalResolution::Approved {
                modified_args: None,
                ..
            }
        ));
        let (status, _) = post(&router, json!({"action_token": reject})).await;
        assert_eq!(status, StatusCode::CONFLICT);

        let (other, mut waiter) = ticket(&state, "tenant-b").await;
        let other_token =
            sign_approval_token(&other, SIGNING_KEY, "reviewer", Decision::Approve).unwrap();
        assert_eq!(
            post(&router, json!({"action_token": other_token})).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert!(matches!(
            waiter.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
    }

    #[tokio::test]
    async fn slack_acknowledges_authenticated_actions_before_waiting_on_the_decision_store() {
        let (_directory, registry) = persistent_registry();
        let state = state(webhook(WebhookFormat::Slack), registry);
        let (ticket, waiter) = ticket(&state, "tenant-a").await;
        let token =
            sign_approval_token(&ticket, SIGNING_KEY, "reviewer", Decision::Reject).unwrap();
        let payload = json!({"user": {"id": "U123"}, "actions": [{"value": token}]});
        let mut url = reqwest::Url::parse("http://localhost").unwrap();
        url.query_pairs_mut()
            .append_pair("payload", &payload.to_string());
        let body = url.query().unwrap();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut mac = Hmac::<Sha256>::new_from_slice(SLACK_KEY.as_bytes()).unwrap();
        mac.update(format!("v0:{timestamp}:{body}").as_bytes());
        let signature = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

        let router = build_router(state.clone());
        let forged = Request::post("/v1/webhooks/actions")
            .body(Body::from(body.to_owned()))
            .unwrap();
        assert_eq!(
            router.clone().oneshot(forged).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        let request = Request::post("/v1/webhooks/actions")
            .header("X-Slack-Request-Timestamp", timestamp.to_string())
            .header("X-Slack-Signature", signature)
            .body(Body::from(body.to_owned()))
            .unwrap();
        let storage_busy = state.approval_registry.pending.write().await;
        let started = Instant::now();
        let timed = tokio::time::timeout(Duration::from_secs(3), router.oneshot(request))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(timed.status(), StatusCode::OK);
        eprintln!("Slack callback acknowledgment: {:?}", started.elapsed());
        assert!(started.elapsed() < Duration::from_secs(3));
        drop(storage_busy);
        let resolution_started = Instant::now();
        let resolution = tokio::time::timeout(Duration::from_secs(2), waiter)
            .await
            .unwrap()
            .unwrap();
        assert!(
            matches!(resolution, ApprovalResolution::Rejected { operator, .. } if operator == "slack:U123")
        );
        eprintln!(
            "Slack decision after releasing the store: {:?}",
            resolution_started.elapsed()
        );
    }

    #[tokio::test]
    async fn approval_persistence_failure_does_not_release_the_waiter() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("approvals.json");
        let registry = ApprovalRegistry::open_or_create(&path).unwrap();
        let state = state(webhook(WebhookFormat::Generic), registry);
        let (ticket, mut waiter) = ticket(&state, "tenant-a").await;
        let token =
            sign_approval_token(&ticket, SIGNING_KEY, "reviewer", Decision::Approve).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        let router = build_router(state.clone());
        let (status, _) = post(&router, json!({"action_token": token})).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(matches!(
            waiter.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        assert!(state
            .approval_registry
            .get(&ticket.id)
            .await
            .unwrap()
            .decision_receipt
            .is_none());
    }

    #[tokio::test]
    async fn an_expired_approval_token_never_releases_the_waiter() {
        let (_directory, registry) = persistent_registry();
        let state = state(webhook(WebhookFormat::Generic), registry);
        let (mut ticket, mut waiter) = ticket(&state, "tenant-a").await;
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(1);
        ticket.expires_at = expiry;
        state
            .approval_registry
            .pending
            .write()
            .await
            .get_mut(&ticket.id)
            .unwrap()
            .expires_at = expiry;
        let token =
            sign_approval_token(&ticket, SIGNING_KEY, "reviewer", Decision::Approve).unwrap();
        let router = build_router(state.clone());
        assert_eq!(
            post(&router, json!({"action_token": token})).await.0,
            StatusCode::CONFLICT
        );
        assert!(matches!(
            waiter.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        assert!(state
            .approval_registry
            .get(&ticket.id)
            .await
            .unwrap()
            .decision_receipt
            .is_none());
    }

    #[tokio::test]
    async fn delivery_failure_is_recorded_after_a_signed_decision() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut config = webhook(WebhookFormat::Generic);
        config.url = format!("http://{}/events", listener.local_addr().unwrap());
        let (events, mut observed) = tokio::sync::mpsc::channel(4);
        let destination = Router::new().route(
            "/events",
            axum::routing::post(move |headers: HeaderMap, raw: String| {
                let events = events.clone();
                async move {
                    assert!(crate::chatops::verify_signature(
                        SIGNING_KEY,
                        &raw,
                        headers["x-warmplane-signature"].to_str().unwrap(),
                        headers["x-warmplane-timestamp"].to_str().ok(),
                    ));
                    let event = headers["x-warmplane-event"].to_str().unwrap();
                    events
                        .send(serde_json::from_str::<Value>(&raw).unwrap())
                        .await
                        .unwrap();
                    if event == RECEIPT {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::OK
                    }
                }
            }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, destination).await.unwrap() });
        let (_directory, registry) = persistent_registry();
        let state = state(config.clone(), registry);
        let (unrelated, _) = ticket(&state, "tenant-b").await;
        send_card(&config, &unrelated).await.unwrap();
        assert!(observed.try_recv().is_err());

        let (pending, waiter) = ticket(&state, "tenant-a").await;
        send_card(&config, &pending).await.unwrap();
        let card = observed.recv().await.unwrap();
        assert_eq!(card["event"], REQUESTED);
        assert_eq!(card["data"]["sanitized_args"], json!({"target": "fixture"}));
        let token = &card["data"]["approve_token"];
        let router = build_router(state.clone());
        let (status, first) = post(&router, json!({"action_token": token})).await;
        assert_eq!(status, StatusCode::OK);
        assert!(matches!(
            waiter.await.unwrap(),
            ApprovalResolution::Approved { .. }
        ));
        let receipt = observed.recv().await.unwrap();
        assert_eq!(receipt["data"], first["receipt"]);
        assert!(receipt["data"].get("upstream_execution_state").is_none());
        let record = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let ticket = state.approval_registry.get(&pending.id).await.unwrap();
                if ticket.receipt_delivery.is_some() {
                    break ticket;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(record.receipt_delivery, Some(ReceiptDelivery::Failed));
        assert_eq!(record.decision_receipt.unwrap().decision, Decision::Approve);
        assert!(observed.try_recv().is_err());
        server.abort();
    }
}
