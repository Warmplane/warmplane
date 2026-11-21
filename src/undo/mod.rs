// Rust guideline compliant 2026-11-16

//! Persistent undo records and content-addressed snapshots.

pub(crate) mod draft;
mod fiber;
mod operator;
mod store;
mod types;

pub use fiber::{
    execute_operator_undo, rollback_operator_work_item, unwind_batch_fiber, unwind_fiber,
    FiberExecutionScope, RollbackResult,
};
pub use operator::{
    execute_agent_self_undo, execute_agent_self_undo_for_work_item, execute_undo, get_undo_entry,
    list_undo_entries, rollback_work_item, UndoAccessContext, UndoEntrySummary, UndoListFilter,
    UndoListResponse, UndoOperationError,
};
pub use store::{CompensableReservation, SnapshotStore, UndoClaimResult, UndoLedgerStore};
pub use types::{CompensableUsage, UndoEntry, UndoEntryStatus, UndoLedger};
