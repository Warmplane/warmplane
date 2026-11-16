// Rust guideline compliant 2026-11-16

//! Persistent undo records and content-addressed snapshots.

mod fiber;
mod store;
mod types;

pub use fiber::{unwind_batch_fiber, unwind_fiber, FiberExecutionScope, RollbackResult};
pub use store::{CompensableReservation, SnapshotStore, UndoLedgerStore};
pub use types::{CompensableUsage, UndoEntry, UndoEntryStatus, UndoLedger};
