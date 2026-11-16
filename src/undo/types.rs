// Rust guideline compliant 2026-11-16

//! Serializable records for registered single-step undo actions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Lifecycle state for one registered inverse action.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UndoEntryStatus {
    /// The inverse action is available for execution.
    Armed,
    /// The inverse action is currently executing.
    Executing,
    /// The forward action has been reverted.
    Reverted,
    /// The configured undo window expired.
    Expired,
    /// The inverse action failed.
    Failed,
}

/// A persisted inverse action associated with one successful forward operation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UndoEntry {
    /// Unique identifier for this undo action.
    pub undo_id: String,
    /// Identifier of the forward operation that created this action.
    pub operation_id: Option<String>,
    /// Workflow item associated with the forward operation.
    pub work_item_id: Option<String>,
    /// Batch request identifier when this entry was created by a batch call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    /// Tenant that owns the undo action.
    pub tenant_id: String,
    /// Active profile associated with the forward operation.
    pub profile_id: Option<String>,
    /// Actor that performed the forward operation.
    pub actor_id: Option<String>,
    /// Capability identifier used for the forward operation.
    pub forward_capability: String,
    /// Reversibility class of the forward operation.
    pub reversibility_class: String,
    /// Capability identifier used to apply the inverse.
    pub resolved_inverse_capability: String,
    /// Arguments resolved for the inverse action.
    pub resolved_inverse_args: Value,
    /// Content hash of the pre-execution snapshot, when configured.
    pub snapshot_artifact_hash: Option<String>,
    /// Unix timestamp in milliseconds when this action was registered.
    pub created_at_ms: u64,
    /// Unix timestamp in milliseconds when the undo window expires.
    pub expires_at_ms: u64,
    /// Current lifecycle state.
    pub status: UndoEntryStatus,
}

/// Persistent collection of undo actions.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct UndoLedger {
    /// Registered forward operations and their inverse actions.
    pub entries: Vec<UndoEntry>,
    /// Recent compensable invocations used to enforce rolling daily caps.
    #[serde(default)]
    pub compensable_usage: Vec<CompensableUsage>,
}

/// Rolling-window usage record for a compensable capability invocation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CompensableUsage {
    /// Tenant that owns the invocation.
    pub tenant_id: String,
    /// Profile active for the invocation, when specified.
    pub profile_id: Option<String>,
    /// Capability identifier invoked.
    pub capability: String,
    /// Idempotency key associated with the invocation, if any.
    pub idempotency_key: Option<String>,
    /// Unix timestamp in milliseconds when the invocation was reserved.
    pub timestamp_ms: u64,
}
