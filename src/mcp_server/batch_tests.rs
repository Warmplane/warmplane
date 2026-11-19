// Rust guideline compliant 2026-11-19
//! MCP wire-level regression coverage for optional batch rollback.

use super::FacadeMcpServer;
use crate::{
    daemon::{AppState, CapabilityMeta, Policy, ServerMsg, UpstreamCallError},
    undo::{SnapshotStore, UndoEntryStatus, UndoLedgerStore},
};
use rmcp::{model::CallToolRequestParams, ServiceExt};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle};

struct BatchFixture {
    state: AppState,
    value: Arc<Mutex<Value>>,
    calls: Arc<Mutex<Vec<String>>>,
    worker: JoinHandle<()>,
    _directory: tempfile::TempDir,
}

impl BatchFixture {
    fn new(block_inverse: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let mut capabilities = ["mutate", "read", "restore", "fail"]
            .into_iter()
            .map(|name| {
                (
                    format!("mock.{name}"),
                    CapabilityMeta::new("mock", name, name, name, json!({"type":"object"})),
                )
            })
            .collect::<HashMap<_, _>>();
        capabilities.get_mut("mock.mutate").unwrap().reversibility = Some(
            serde_json::from_value(json!({
                "class":"reversible",
                "ttlSecs":60,
                "preSnapshot":{"capability":"mock.read","args":{}},
                "inverse":{"capability":"mock.restore","args":{"value":"${snapshot}"}}
            }))
            .unwrap(),
        );
        let (sender, mut receiver) = tokio::sync::mpsc::channel(16);
        let policy = Policy {
            deny: if block_inverse {
                vec!["mock.restore".into()]
            } else {
                vec![]
            },
            ..Default::default()
        };
        let state = AppState::builder()
            .capabilities(capabilities)
            .servers(HashMap::from([("mock".into(), sender)]))
            .policy(policy)
            .undo_ledger(UndoLedgerStore::open(directory.path().join("undo.json")).unwrap())
            .snapshot_store(SnapshotStore::open(directory.path().join("snapshots")).unwrap())
            .build();
        let value = Arc::new(Mutex::new(json!(0)));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let worker_value = value.clone();
        let worker_calls = calls.clone();
        let worker = tokio::spawn(async move {
            while let Some(ServerMsg::CallTool {
                name,
                params,
                reply,
                ..
            }) = receiver.recv().await
            {
                worker_calls.lock().await.push(name.clone());
                let result = match name.as_str() {
                    "read" => Ok(worker_value.lock().await.clone()),
                    "mutate" | "restore" => {
                        *worker_value.lock().await = params["value"].clone();
                        Ok(json!({"changed":true}))
                    }
                    _ => Err(UpstreamCallError::Upstream("planned failure".into())),
                };
                let _ = reply.send(result);
            }
        });
        Self {
            state,
            value,
            calls,
            worker,
            _directory: directory,
        }
    }

    async fn request(&self, args: Value) -> Value {
        let (client_io, server_io) = tokio::io::duplex(65536);
        let server_state = self.state.clone();
        let server = tokio::spawn(async move {
            FacadeMcpServer::new(server_state, None)
                .serve(server_io)
                .await
                .unwrap()
        });
        let client = ().serve(client_io).await.unwrap();
        let server = server.await.unwrap();
        let tools = client.list_all_tools().await.unwrap();
        let batch = tools
            .iter()
            .find(|tool| tool.name == "capabilities_batch_call")
            .unwrap();
        let option = &batch.input_schema["properties"]["autoRollbackOnFailure"];
        assert_eq!(option["type"], "boolean");
        assert_eq!(option["default"], false);
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            client.call_tool(
                CallToolRequestParams::new("capabilities_batch_call")
                    .with_arguments(args.as_object().unwrap().clone()),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        let value = response.structured_content.unwrap();
        client.cancel().await.unwrap();
        server.cancel().await.unwrap();
        value
    }

    async fn assert_entries(&self, response: &Value, status: UndoEntryStatus) {
        let entries = self.state.undo_ledger.as_ref().unwrap().entries().await;
        assert_eq!(entries.len(), 2);
        let expected_batch = response["request_id"]
            .as_str()
            .unwrap_or_else(|| response["trace_id"].as_str().unwrap());
        for entry in entries {
            assert_eq!(entry.status, status);
            assert_eq!(entry.batch_id.as_deref(), Some(expected_batch));
            assert_eq!(entry.tenant_id, "default");
            assert_eq!(entry.work_item_id, None);
        }
    }
}

impl Drop for BatchFixture {
    fn drop(&mut self) {
        self.worker.abort();
    }
}

fn batch_args(option: Option<Value>, fail: bool) -> Value {
    let mut args = json!({"steps":[
        {"id":"one","capability_id":"mock.mutate","args":{"value":1},"continue_on_error":true},
        {"id":"two","capability_id":"mock.mutate","args":{"value":2},"continue_on_error":true}
    ]});
    if fail {
        args["steps"].as_array_mut().unwrap().extend([
            json!({"id":"fail","capability_id":"mock.fail","args":{},"continue_on_error":true}),
            json!({"id":"later","capability_id":"mock.read","args":{}}),
        ]);
    }
    if let Some(option) = option {
        args["autoRollbackOnFailure"] = option;
    }
    args
}

#[tokio::test]
async fn mcp_batch_failure_rolls_back_in_reverse_order_with_either_identity() {
    for request_id in [None, Some("mcp-rollback")] {
        let fixture = BatchFixture::new(false);
        let mut args = batch_args(Some(json!(true)), true);
        if let Some(id) = request_id {
            args["request_id"] = json!(id);
        }
        let response = fixture.request(args).await;
        assert_eq!(response["ok"], false);
        assert_eq!(response["results"].as_array().unwrap().len(), 3);
        let outcomes = response["rollback_results"].as_array().unwrap();
        assert_eq!(outcomes.len(), 2);
        assert!(outcomes.iter().all(|outcome| outcome["ok"] == true));
        assert_eq!(*fixture.value.lock().await, json!(0));
        assert_eq!(
            *fixture.calls.lock().await,
            ["read", "mutate", "read", "mutate", "fail", "restore", "restore"]
        );
        let entries = fixture.state.undo_ledger.as_ref().unwrap().entries().await;
        assert_eq!(outcomes[0]["undo_id"], entries[1].undo_id);
        assert_eq!(outcomes[1]["undo_id"], entries[0].undo_id);
        fixture
            .assert_entries(&response, UndoEntryStatus::Reverted)
            .await;
    }
}

#[tokio::test]
async fn mcp_batch_without_rollback_preserves_continue_on_error() {
    for option in [None, Some(json!(false))] {
        for continue_on_error in [false, true] {
            let fixture = BatchFixture::new(false);
            let mut args = batch_args(option.clone(), true);
            args["steps"][2]["continue_on_error"] = json!(continue_on_error);
            let response = fixture.request(args).await;
            assert_eq!(response["ok"], false);
            assert_eq!(response["rollback_results"], json!([]));
            assert_eq!(
                response["results"].as_array().unwrap().len(),
                if continue_on_error { 4 } else { 3 }
            );
            assert_eq!(*fixture.value.lock().await, json!(2));
            assert!(!fixture
                .calls
                .lock()
                .await
                .iter()
                .any(|name| name == "restore"));
            fixture
                .assert_entries(&response, UndoEntryStatus::Armed)
                .await;
        }
    }
}

#[tokio::test]
async fn mcp_batch_success_keeps_inverses_armed() {
    let fixture = BatchFixture::new(false);
    let response = fixture.request(batch_args(Some(json!(true)), false)).await;
    assert_eq!(response["ok"], true);
    assert_eq!(response["rollback_results"], json!([]));
    assert_eq!(*fixture.value.lock().await, json!(2));
    fixture
        .assert_entries(&response, UndoEntryStatus::Armed)
        .await;
}

#[tokio::test]
async fn mcp_batch_rejects_invalid_rollback_options_before_dispatch() {
    for option in [json!("true"), json!(1), Value::Null] {
        let fixture = BatchFixture::new(false);
        let response = fixture.request(batch_args(Some(option), true)).await;
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "INVALID_ARGS");
        assert!(fixture.calls.lock().await.is_empty());
        assert!(fixture
            .state
            .undo_ledger
            .as_ref()
            .unwrap()
            .entries()
            .await
            .is_empty());
    }
}

#[tokio::test]
async fn mcp_batch_rollback_respects_inverse_policy() {
    let fixture = BatchFixture::new(true);
    let response = fixture.request(batch_args(Some(json!(true)), true)).await;
    assert_eq!(response["ok"], false);
    assert_eq!(response["results"][2]["ok"], false);
    assert!(response["results"][2]["error"]
        .as_str()
        .unwrap()
        .contains("planned failure"));
    let outcomes = response["rollback_results"].as_array().unwrap();
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|outcome| outcome["ok"] == false
        && outcome["error"]
            .as_str()
            .unwrap()
            .contains("denied by current policy")));
    assert_eq!(*fixture.value.lock().await, json!(2));
    assert!(!fixture
        .calls
        .lock()
        .await
        .iter()
        .any(|name| name == "restore"));
}
