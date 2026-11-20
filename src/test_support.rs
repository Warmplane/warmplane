// Rust guideline compliant 2026-11-18
//! Shared fixtures for unit tests exercising delayable tool dispatch.

use crate::{
    config::{ReversibilityClass, ReversibilityConfig},
    daemon::{AppState, CapabilityMeta, ServerMsg},
};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub(crate) fn latency_percentiles(
    samples: usize,
    mut measure: impl FnMut() -> std::time::Duration,
) -> (std::time::Duration, std::time::Duration) {
    let mut measured = (0..samples).map(|_| measure()).collect::<Vec<_>>();
    measured.sort_unstable();
    (measured[samples / 2], measured[samples * 95 / 100])
}

/// Creates a mail capability with an explicit hold and a receiving upstream mailbox.
pub(crate) fn delayable_mail(
    hold_secs: u64,
    timeout_ms: u64,
) -> (AppState, mpsc::Receiver<ServerMsg>) {
    let (sender, receiver) = mpsc::channel(8);
    let mut capability = CapabilityMeta::new(
        "mail",
        "send",
        "Send",
        "Send a message",
        serde_json::json!({"type": "object"}),
    );
    capability.reversibility = Some(ReversibilityConfig {
        class: ReversibilityClass::Delayable,
        hold_window_secs: Some(hold_secs),
        ttl_secs: None,
        daily_limit: None,
        pre_snapshot: None,
        pre_undo_check: None,
        test_target: None,
        inverse: None,
    });
    let state = AppState::builder()
        .servers(HashMap::from([("mail".to_string(), sender)]))
        .capabilities(HashMap::from([("mail.send".to_string(), capability)]))
        .tool_timeout_ms(timeout_ms)
        .build();
    (state, receiver)
}
