// Rust guideline compliant 2026-08-15

//! Asynchronous background batch flusher for non-blocking audit logging.
//!
//! Provides a bounded lock-free channel producer handle and a worker task that batches events
//! to avoid adding latency to critical-path capability invocations.

use std::time::Duration;
use tokio::sync::mpsc;
use tracing::error;

use crate::audit::models::RawAuditEvent;
use crate::audit::siem::SiemDispatcher;
use crate::audit::store::SharedAuditStore;

/// Default buffer capacity for the audit logging channel.
pub const DEFAULT_AUDIT_BUFFER_CAPACITY: usize = 10_000;
/// Default batch flush interval in milliseconds.
pub const DEFAULT_AUDIT_FLUSH_INTERVAL_MS: u64 = 250;
/// Maximum batch size before triggering immediate store flush.
pub const DEFAULT_AUDIT_MAX_BATCH_SIZE: usize = 100;

/// Message sent over the audit queue to the worker task.
#[derive(Debug)]
pub enum AuditWorkerMsg {
    /// Raw audit event record to append.
    Event(Box<RawAuditEvent>),
    /// Signals the background flusher to drain all buffered events and terminate.
    FlushAndShutdown(tokio::sync::oneshot::Sender<()>),
}

/// Producer handle for sending raw audit events into the background flusher queue.
#[derive(Clone)]
pub struct AuditHandle {
    sender: mpsc::Sender<AuditWorkerMsg>,
}

impl AuditHandle {
    /// Creates a new `AuditHandle` with the given channel sender.
    pub fn new(sender: mpsc::Sender<AuditWorkerMsg>) -> Self {
        Self { sender }
    }

    /// Dispatches a raw audit event non-blockingly to the background queue.
    /// If the queue is saturated, logs a warning rather than stalling execution.
    pub fn send(&self, event: RawAuditEvent) {
        if let Err(e) = self.sender.try_send(AuditWorkerMsg::Event(Box::new(event))) {
            match e {
                mpsc::error::TrySendError::Full(_) => {
                    tracing::warn!("Audit queue is full; dropping audit event to prevent stalling");
                }
                mpsc::error::TrySendError::Closed(_) => {
                    tracing::warn!("Audit worker channel closed; could not enqueue event");
                }
            }
        }
    }

    /// Asynchronously sends a raw audit event, waiting if the buffer is currently full.
    pub async fn send_async(&self, event: RawAuditEvent) {
        if let Err(e) = self
            .sender
            .send(AuditWorkerMsg::Event(Box::new(event)))
            .await
        {
            tracing::warn!("Audit worker channel closed: {:?}", e);
        }
    }

    /// Triggers an immediate buffer drain and shutdown of the background worker task.
    pub async fn shutdown(&self) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self
            .sender
            .send(AuditWorkerMsg::FlushAndShutdown(tx))
            .await
            .is_ok()
        {
            let _ = tokio::time::timeout(Duration::from_secs(5), rx).await;
        }
    }
}

/// Spawns the background audit worker task.
///
/// # Arguments
/// * `store` - Shared append-only audit store.
/// * `siem_dispatcher` - Optional SIEM dispatcher.
/// * `buffer_capacity` - Max in-memory channel capacity.
/// * `flush_interval_ms` - Max time before flushing buffered events to storage.
/// * `max_batch_size` - Max events accumulated before flushing immediately.
///
/// # Returns
/// An `AuditHandle` producer.
pub fn spawn_audit_worker(
    store: SharedAuditStore,
    siem_dispatcher: Option<SiemDispatcher>,
    buffer_capacity: usize,
    flush_interval_ms: u64,
    max_batch_size: usize,
) -> AuditHandle {
    let (tx, mut rx) = mpsc::channel(buffer_capacity);

    tokio::spawn(async move {
        let mut buffer: Vec<RawAuditEvent> = Vec::with_capacity(max_batch_size);
        let mut interval = tokio::time::interval(Duration::from_millis(flush_interval_ms));

        loop {
            tokio::select! {
                biased;
                Some(msg) = rx.recv() => {
                    match msg {
                        AuditWorkerMsg::Event(event) => {
                            buffer.push(*event);
                            if buffer.len() >= max_batch_size {
                                let batch = std::mem::take(&mut buffer);
                                match store.append_batch(batch).await {
                                    Ok(committed) => {
                                        if let Some(ref siem) = siem_dispatcher {
                                            siem.dispatch_batch(&committed).await;
                                        }
                                    }
                                    Err(err) => {
                                        error!("Failed to flush audit batch to store: {:?}", err);
                                    }
                                }
                            }
                        }
                        AuditWorkerMsg::FlushAndShutdown(reply) => {
                            // Drain any remaining messages already queued in channel
                            while let Ok(msg) = rx.try_recv() {
                                if let AuditWorkerMsg::Event(ev) = msg {
                                    buffer.push(*ev);
                                }
                            }
                            if !buffer.is_empty() {
                                let batch = std::mem::take(&mut buffer);
                                if let Ok(committed) = store.append_batch(batch).await {
                                    if let Some(ref siem) = siem_dispatcher {
                                        siem.dispatch_batch(&committed).await;
                                    }
                                }
                            }
                            let _ = reply.send(());
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        let batch = std::mem::take(&mut buffer);
                        match store.append_batch(batch).await {
                            Ok(committed) => {
                                if let Some(ref siem) = siem_dispatcher {
                                    siem.dispatch_batch(&committed).await;
                                }
                            }
                            Err(err) => {
                                error!("Failed to flush periodic audit batch to store: {:?}", err);
                            }
                        }
                    }
                }
                else => {
                    // Channel closed, flush remaining
                    if !buffer.is_empty() {
                        let batch = std::mem::take(&mut buffer);
                        if let Ok(committed) = store.append_batch(batch).await {
                            if let Some(ref siem) = siem_dispatcher {
                                siem.dispatch_batch(&committed).await;
                            }
                        }
                    }
                    break;
                }
            }
        }
    });

    AuditHandle::new(tx)
}
