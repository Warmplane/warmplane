// Rust guideline compliant 2026-09-29

//! Reproduces TTL truncation on task lifetimes.
//!
//! A sub-second `ttlMs` is divided by 1000 to compare against a whole-second
//! creation timestamp, so every lifetime under one second collapses to zero
//! and the task expires the instant it is created.

use serde_json::json;
use std::time::Duration;
use warmplane::tasks::{CreateTaskParams, TaskRegistry, TaskStatus};

fn params(ttl_ms: u64) -> CreateTaskParams {
    CreateTaskParams {
        capability_id: "db.query".to_string(),
        server_id: "srv-1".to_string(),
        args: json!({"q": "select 1"}),
        request_id: None,
        context: None,
        idempotency_key: None,
        initial_status: TaskStatus::Working,
        status_message: None,
        input_requests: None,
        ttl_ms: Some(ttl_ms),
        poll_interval_ms: Some(1_000),
    }
}

#[tokio::test]
async fn sub_second_ttl_does_not_expire_immediately() {
    let registry = TaskRegistry::new();
    let (record, _) = registry.create_task(params(500)).await;

    let fetched = registry.get_task(&record.task_id).await.unwrap();
    assert_ne!(
        fetched.status,
        TaskStatus::Failed,
        "a 500ms TTL must not expire the task immediately, got status {:?} ({:?})",
        fetched.status,
        fetched.status_message
    );
}

#[tokio::test]
async fn zero_ttl_is_reported_as_instant_expiry_not_unbounded() {
    let registry = TaskRegistry::new();
    let (record, _) = registry.create_task(params(0)).await;

    let response = warmplane::tasks::TaskResponse::from(&record);
    let created = record.created_at_epoch_secs;
    let expires = response
        .expires_at_epoch_secs
        .expect("a task with a TTL must report an expiry");

    assert!(
        expires >= created,
        "expiry {expires} must not be reported before creation {created}"
    );
}

#[tokio::test]
async fn multi_second_ttl_still_expires_once_elapsed() {
    let registry = TaskRegistry::new();
    let (record, _) = registry.create_task(params(1_000)).await;

    tokio::time::sleep(Duration::from_millis(1_100)).await;

    let fetched = registry.get_task(&record.task_id).await.unwrap();
    assert_eq!(
        fetched.status,
        TaskStatus::Failed,
        "a 1000ms TTL must expire once its window has passed"
    );
}
