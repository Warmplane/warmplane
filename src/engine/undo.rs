// Rust guideline compliant 2026-11-16

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
    if !policy.allows(&config.capability) {
        bail!("pre-snapshot capability is denied by policy");
    }
    let (server_id, tool_name) = {
        let capabilities = state.capabilities.read().await;
        let meta = capabilities
            .get(&config.capability)
            .context("pre-snapshot capability was not found")?;
        if !profile.is_server_allowed(&meta.server) {
            bail!("pre-snapshot capability is outside the active profile");
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
            .context("pre-snapshot upstream server is unavailable")?
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
        .context("failed to queue pre-snapshot call")?;
    let timeout = tokio::time::Duration::from_millis(state.tool_timeout_ms);
    let result = tokio::time::timeout(timeout, reply_rx)
        .await
        .context("pre-snapshot call timed out")?
        .context("pre-snapshot upstream dropped its response")?
        .map_err(|error| match error {
            UpstreamCallError::Timeout => anyhow::anyhow!("pre-snapshot upstream timed out"),
            UpstreamCallError::Upstream(message) => {
                anyhow::anyhow!("pre-snapshot failed: {message}")
            }
        })?;

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
    let store = state
        .snapshot_store
        .as_ref()
        .context("persistent snapshot storage is required for configured pre-snapshots")?;
    let hash = store.put(&snapshot)?;
    Ok((snapshot, Some(hash)))
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
    let entry = UndoEntry {
        undo_id: generate_undo_id(now_ms),
        operation_id: forward.context.operation_id.clone(),
        work_item_id: forward.context.work_item_id.clone(),
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
        snapshot_artifact_hash: forward.snapshot_hash,
        created_at_ms: now_ms,
        expires_at_ms,
        status: UndoEntryStatus::Armed,
    };
    state
        .undo_ledger
        .as_ref()
        .context("persistent undo ledger is required for inverse registration")?
        .register(entry)
        .await
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
    use super::{generate_undo_id, undo_expiry_ms};

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
}
