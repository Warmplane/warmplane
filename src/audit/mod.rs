// Rust guideline compliant 2026-08-15

//! WORM Audit Trail & SIEM Export subsystem.
//!
//! Provides non-repudiable append-only logging with linear cryptographic SHA-256 hash chaining,
//! background async batching, continuous verification, and telemetry export.

pub mod chain;
pub mod models;
pub mod siem;
pub mod store;
pub mod worker;

#[cfg(test)]
mod tests;

pub use chain::{compute_event_hash, verify_record_hash, GENESIS_HASH};
pub use models::{AuditEvent, AuditEventStatus, AuditEventType, RawAuditEvent, VerificationReport};
pub use siem::SiemDispatcher;
pub use store::{AuditQueryFilter, AuditStore, SharedAuditStore, MAX_IN_MEMORY_AUDIT_EVENTS};
pub use worker::{
    spawn_audit_worker, AuditHandle, DEFAULT_AUDIT_BUFFER_CAPACITY,
    DEFAULT_AUDIT_FLUSH_INTERVAL_MS, DEFAULT_AUDIT_MAX_BATCH_SIZE,
};
