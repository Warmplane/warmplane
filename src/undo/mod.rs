// Rust guideline compliant 2026-11-16

//! Persistent undo records and content-addressed snapshots.

mod store;
mod types;

pub use store::{CompensableReservation, SnapshotStore, UndoLedgerStore};
pub use types::{CompensableUsage, UndoEntry, UndoEntryStatus, UndoLedger};
