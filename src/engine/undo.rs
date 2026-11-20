// Rust guideline compliant 2026-11-18

//! Helpers for capturing pre-state and arming inverse actions.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::sync::oneshot;

use crate::{
    config::{PreSnapshotConfig, ReversibilityClass, ReversibilityConfig},
    context::{ProfileContext, RequestContext},
    daemon::{AppState, Policy, ServerMsg, UpstreamCallError},
    engine::interpolation::interpolate_undo_template,
    undo::{CompensableReservation, UndoEntry, UndoEntryStatus},
};

/// Forward execution data used to resolve and persist an inverse action.
pub(crate) struct ForwardCall<'a> {
    /// Capability identifier used for the forward operation.
    pub capability: &'a str,
    /// Arguments sent to the forward capability.
    pub args: &'a Value,
    /// Raw result returned by the forward capability.
    pub result: &'a Value,
    /// Captured pre-state value.
    pub snapshot: &'a Value,
    /// Hash of the stored snapshot artifact, if configured.
    pub snapshot_hash: Option<String>,
    /// Request context associated with the forward operation.
    pub context: &'a RequestContext,
    /// Batch request identifier when the forward call belongs to a batch.
    pub batch_id: Option<&'a str>,
    /// Tenant owning this forward operation.
    pub tenant_id: &'a str,
    /// Active profile, when scoped.
    pub profile_id: Option<&'a str>,
}

/// Captures and stores the pre-state required by a reversibility contract.
pub(crate) async fn capture_pre_snapshot(
    state: &AppState,
    config: &PreSnapshotConfig,
    forward_args: &Value,
    profile: &ProfileContext,
    policy: &Policy,
) -> Result<(Value, Option<String>)> {
    let snapshot = read_checked_state(state, config, forward_args, profile, policy).await?;
    let store = state
        .snapshot_store
        .as_ref()
        .context("persistent snapshot storage is required for configured pre-snapshots")?;
    let hash = store.put(&snapshot)?;
    Ok((snapshot, Some(hash)))
}

/// Reads and distills a configured state value without exposing it to the caller.
pub(crate) async fn read_checked_state(
    state: &AppState,
    config: &PreSnapshotConfig,
    forward_args: &Value,
    profile: &ProfileContext,
    policy: &Policy,
) -> Result<Value> {
    if !policy.allows(&config.capability) {
        bail!("state-check capability is denied by policy");
    }
    if policy.requires_approval(&config.capability) {
        bail!("state-check capability requires approval and cannot run automatically");
    }
    let (server_id, tool_name) = {
        let capabilities = state.capabilities.read().await;
        let meta = capabilities
            .get(&config.capability)
            .context("state-check capability was not found")?;
        if !profile.is_server_allowed(&meta.server) {
            bail!("state-check capability is outside the active profile");
        }
        (meta.server.clone(), meta.tool.clone())
    };
    let args = interpolate_undo_template(&config.args, forward_args, &Value::Null, &Value::Null)?;
    if !args.is_object() {
        bail!("pre-snapshot arguments must resolve to a JSON object");
    }
    let upstream = {
        let servers = state.servers.read().await;
        servers
            .get(&server_id)
            .cloned()
            .context("state-check upstream server is unavailable")?
    };
    let (reply_tx, reply_rx) = oneshot::channel();
    upstream
        .send(ServerMsg::CallTool {
            name: tool_name,
            params: args,
            input_responses: None,
            request_state: None,
            reply: reply_tx,
        })
        .await
        .context("failed to queue state-check call")?;
    let timeout = tokio::time::Duration::from_millis(state.tool_timeout_ms);
    let result = tokio::time::timeout(timeout, reply_rx)
        .await
        .context("state-check call timed out")?
        .context("state-check upstream dropped its response")?;
    let result = match result {
        Ok(result) => result,
        Err(UpstreamCallError::Timeout) => bail!("state-check upstream timed out"),
        Err(UpstreamCallError::Upstream(message)) => {
            let message = state.safe_upstream_error(message).await;
            bail!("state-check failed: {message}");
        }
    };

    let snapshot = if let Some(path) = &config.capture_json_path {
        crate::context_filter::distill_value(
            result,
            &crate::context_filter::DistillationOptions {
                jsonpath: Some(path.clone()),
                ..Default::default()
            },
        )
    } else {
        result
    };
    Ok(snapshot)
}

/// Reserves use of a compensable capability within its configured rolling daily limit.
pub(crate) async fn reserve_compensable(
    state: &AppState,
    contract: &ReversibilityConfig,
    capability: &str,
    tenant_id: &str,
    profile_id: Option<&str>,
    idempotency_key: Option<&str>,
    approved_over_limit: bool,
) -> Result<bool> {
    if contract.class != ReversibilityClass::Compensable {
        return Ok(true);
    }
    let ledger = state
        .undo_ledger
        .as_ref()
        .context("persistent undo ledger is required for compensable capabilities")?;
    let limit = contract
        .daily_limit
        .context("dailyLimit is required for compensable capabilities")?;
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    ledger
        .reserve_compensable(CompensableReservation {
            tenant_id,
            profile_id,
            capability,
            daily_limit: limit,
            idempotency_key,
            approved_over_limit,
            now_ms,
        })
        .await
}

/// Interpolates and durably registers the inverse for a successful forward operation.
pub(crate) async fn register_inverse(
    state: &AppState,
    contract: &ReversibilityConfig,
    forward: ForwardCall<'_>,
    profile: &ProfileContext,
    policy: &Policy,
) -> Result<()> {
    if !matches!(
        contract.class,
        ReversibilityClass::Reversible | ReversibilityClass::Compensable
    ) {
        return Ok(());
    }
    let inverse = contract
        .inverse
        .as_ref()
        .context("inverse config missing for reversible capability")?;
    let inverse_args = interpolate_undo_template(
        &inverse.args,
        forward.args,
        forward.result,
        forward.snapshot,
    )?;
    if !inverse_args.is_object() {
        bail!("inverse arguments must resolve to a JSON object");
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let ttl_secs = contract.ttl_secs.unwrap_or(3600);
    let expires_at_ms = undo_expiry_ms(now_ms, ttl_secs)?;
    let change_preview = inverse_args
        .as_object()
        .into_iter()
        .flat_map(|object| object.keys())
        .filter(|key| is_safe_preview_key(key, &policy.redact_keys))
        .cloned()
        .collect();
    let (pre_undo_check, expected_post_snapshot_hash) =
        if let Some(check) = &contract.pre_undo_check {
            let mut resolved_check = check.as_ref().clone();
            resolved_check.args = interpolate_undo_template(
                &resolved_check.args,
                forward.args,
                forward.result,
                forward.snapshot,
            )?;
            if !resolved_check.args.is_object() {
                bail!("preUndoCheck arguments must resolve to a JSON object");
            }
            let post_state =
                read_checked_state(state, &resolved_check, &Value::Null, profile, policy).await?;
            let hash = Some(
                state
                    .snapshot_store
                    .as_ref()
                    .context("persistent snapshot storage is required for preUndoCheck")?
                    .put(&post_state)?,
            );
            (Some(resolved_check), hash)
        } else {
            (None, None)
        };
    let entry = UndoEntry {
        undo_id: generate_undo_id(now_ms),
        operation_id: forward.context.operation_id.clone(),
        work_item_id: forward.context.work_item_id.clone(),
        batch_id: forward.batch_id.map(ToString::to_string),
        tenant_id: forward.tenant_id.to_string(),
        profile_id: forward.profile_id.map(ToString::to_string),
        actor_id: forward.context.actor_id.clone(),
        forward_capability: forward.capability.to_string(),
        reversibility_class: match contract.class {
            ReversibilityClass::Reversible => "reversible",
            ReversibilityClass::Compensable => "compensable",
            _ => "irreversible",
        }
        .to_string(),
        resolved_inverse_capability: inverse.capability.clone(),
        resolved_inverse_args: inverse_args,
        change_preview,
        snapshot_artifact_hash: forward.snapshot_hash,
        pre_undo_check,
        expected_post_snapshot_hash,
        created_at_ms: now_ms,
        expires_at_ms,
        status: UndoEntryStatus::Armed,
    };
    let action_entry = entry.clone();
    state
        .undo_ledger
        .as_ref()
        .context("persistent undo ledger is required for inverse registration")?
        .register(entry)
        .await?;
    state.notify_action_capsules_changed();
    crate::approvals::dispatch_signed_action(
        state.clone(),
        crate::approvals::SignedActionDispatch {
            event_type: "undo.action_available",
            action: "undo",
            target_id: action_entry.undo_id,
            tenant_id: action_entry.tenant_id,
            expires_at_ms: action_entry.expires_at_ms,
            capability: action_entry.forward_capability,
            actor: action_entry.actor_id,
            profile_id: action_entry.profile_id,
        },
    );
    Ok(())
}

fn is_safe_preview_key(key: &str, redact_keys: &[String]) -> bool {
    let normalized = key.to_ascii_lowercase();
    let sensitive = [
        "password",
        "secret",
        "token",
        "credential",
        "authorization",
        "private",
        "api_key",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    let redacted = redact_keys
        .iter()
        .any(|redact_key| redact_key.eq_ignore_ascii_case(key));
    !sensitive && !redacted
}

fn generate_undo_id(now_ms: u64) -> String {
    format!("undo_{now_ms}_{:032x}", rand::random::<u128>())
}

fn undo_expiry_ms(now_ms: u64, ttl_secs: u64) -> Result<u64> {
    let ttl_ms = ttl_secs
        .checked_mul(1000)
        .context("undo TTL milliseconds overflow")?;
    now_ms
        .checked_add(ttl_ms)
        .context("undo expiry timestamp overflow".to_string())
}

#[cfg(test)]
mod tests {
    use super::{generate_undo_id, is_safe_preview_key, undo_expiry_ms};

    #[test]
    fn undo_ids_remain_unique_within_the_same_millisecond() {
        let ids = (0..32)
            .map(|_| generate_undo_id(1234))
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(ids.len(), 32);
    }

    #[test]
    fn undo_expiry_rejects_millisecond_and_deadline_overflow() {
        assert!(undo_expiry_ms(0, u64::MAX).is_err());
        assert!(undo_expiry_ms(u64::MAX - 10, 1).is_err());
        assert_eq!(undo_expiry_ms(100, 2).unwrap(), 2_100);
    }

    #[test]
    fn change_preview_hides_sensitive_and_policy_redacted_fields() {
        let redact_keys = vec!["customer_name".to_string()];
        assert!(is_safe_preview_key("priority", &redact_keys));
        assert!(!is_safe_preview_key("api_token", &redact_keys));
        assert!(!is_safe_preview_key("customer_name", &redact_keys));
    }
}

#[cfg(test)]
mod approval_regression_tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn internal_state_reads_cannot_bypass_approval_rules() {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let state = AppState::builder()
            .servers(HashMap::from([("mock".to_string(), sender)]))
            .capabilities(HashMap::from([(
                "mock.read".to_string(),
                crate::daemon::CapabilityMeta::new(
                    "mock",
                    "read",
                    "Read",
                    "Read",
                    serde_json::json!({"type":"object"}),
                ),
            )]))
            .build();
        let policy = Policy {
            require_approval: vec!["mock.*".to_string()],
            ..Default::default()
        };
        let check = PreSnapshotConfig {
            capability: "mock.read".to_string(),
            args: serde_json::json!({}),
            capture_json_path: None,
        };
        let error = read_checked_state(
            &state,
            &check,
            &Value::Null,
            &ProfileContext::default(),
            &policy,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("requires approval"));
        let snapshot_error = capture_pre_snapshot(
            &state,
            &check,
            &Value::Null,
            &ProfileContext::default(),
            &policy,
        )
        .await
        .unwrap_err();
        assert!(snapshot_error.to_string().contains("requires approval"));
        let contract = ReversibilityConfig {
            class: ReversibilityClass::Reversible,
            hold_window_secs: None,
            ttl_secs: None,
            daily_limit: None,
            pre_snapshot: None,
            pre_undo_check: Some(Box::new(check)),
            test_target: None,
            inverse: Some(Box::new(crate::config::InverseConfig {
                capability: "mock.restore".to_string(),
                args: serde_json::json!({}),
            })),
        };
        let post_error = register_inverse(
            &state,
            &contract,
            ForwardCall {
                capability: "mock.set",
                args: &Value::Null,
                result: &Value::Null,
                snapshot: &Value::Null,
                snapshot_hash: None,
                context: &RequestContext::default(),
                batch_id: None,
                tenant_id: "default",
                profile_id: None,
            },
            &ProfileContext::default(),
            &policy,
        )
        .await
        .unwrap_err();
        assert!(post_error.to_string().contains("requires approval"));
        assert!(receiver.try_recv().is_err());
    }
}
