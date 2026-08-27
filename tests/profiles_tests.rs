// Rust guideline compliant 2026-08-27

//! Integration tests for Warmplane Named Profiles constellations (`M-CANONICAL-DOCS`).

use axum::http::StatusCode;
use serde_json::{json, Value};
use std::collections::HashMap;
use tempfile::NamedTempFile;
use tokio::net::TcpListener;

use warmplane::{
    config::{save_config, McpConfig, ProfileConfig, ServerConfig},
    daemon::{
        server::{build_router, initialize_state},
        CapabilityMeta, PromptMeta, ResourceMeta,
    },
};

#[tokio::test]
async fn test_profile_http_filtering_and_etag_caching() {
    let temp_config = NamedTempFile::new().unwrap();
    let config_path = temp_config.path().to_str().unwrap().to_string();

    let mut initial_config = McpConfig::default();
    initial_config.mcp_servers.insert(
        "server_a".to_string(),
        ServerConfig {
            command: Some("echo".to_string()),
            args: vec![],
            env: HashMap::new(),
            url: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            auth: None,
            resilience: None,
        },
    );
    initial_config.mcp_servers.insert(
        "server_b".to_string(),
        ServerConfig {
            command: Some("echo".to_string()),
            args: vec![],
            env: HashMap::new(),
            url: None,
            protocol_version: None,
            allow_stateless: None,
            headers: HashMap::new(),
            auth: None,
            resilience: None,
        },
    );
    initial_config.profiles.insert(
        "only_a".to_string(),
        ProfileConfig {
            servers: vec!["server_a".to_string()],
            description: Some("Profile with only server A".to_string()),
            policy: None,
        },
    );

    save_config(&config_path, &initial_config).unwrap();

    let state = initialize_state(initial_config, &config_path)
        .await
        .unwrap();

    // Populate capabilities, resources, and prompts
    {
        let mut caps = state.capabilities.write().await;
        caps.insert(
            "a.tool".to_string(),
            CapabilityMeta {
                server: "server_a".to_string(),
                tool: "tool_1".to_string(),
                summary: "Tool from server A".to_string(),
                description: "Description A".to_string(),
                input_schema: json!({"type": "object"}),
                tags: vec!["alpha".to_string()],
                examples: vec![],
            },
        );
        caps.insert(
            "b.tool".to_string(),
            CapabilityMeta {
                server: "server_b".to_string(),
                tool: "tool_2".to_string(),
                summary: "Tool from server B".to_string(),
                description: "Description B".to_string(),
                input_schema: json!({"type": "object"}),
                tags: vec!["beta".to_string()],
                examples: vec![],
            },
        );
    }
    {
        let mut res = state.resources.write().await;
        res.insert(
            "res.a".to_string(),
            ResourceMeta {
                server: "server_a".to_string(),
                uri: "file:///a.txt".to_string(),
                name: "resource_a".to_string(),
                description: Some("Resource A".to_string()),
                mime_type: Some("text/plain".to_string()),
                tags: vec![],
            },
        );
        res.insert(
            "res.b".to_string(),
            ResourceMeta {
                server: "server_b".to_string(),
                uri: "file:///b.txt".to_string(),
                name: "resource_b".to_string(),
                description: Some("Resource B".to_string()),
                mime_type: Some("text/plain".to_string()),
                tags: vec![],
            },
        );
    }
    {
        let mut prompts = state.prompts.write().await;
        prompts.insert(
            "prompt.a".to_string(),
            PromptMeta {
                server: "server_a".to_string(),
                name: "prompt_a".to_string(),
                title: Some("Prompt A".to_string()),
                description: Some("Prompt on server A".to_string()),
                arguments: vec![],
                tags: vec![],
            },
        );
        prompts.insert(
            "prompt.b".to_string(),
            PromptMeta {
                server: "server_b".to_string(),
                name: "prompt_b".to_string(),
                title: Some("Prompt B".to_string()),
                description: Some("Prompt on server B".to_string()),
                arguments: vec![],
                tags: vec![],
            },
        );
    }

    let app = build_router(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // 1. Unscoped / unrestricted capabilities list -> returns both server A and server B
    let all_resp = client
        .get(format!("{}/v1/capabilities", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(all_resp.status(), StatusCode::OK);
    let all_etag = all_resp
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let all_json: Value = all_resp.json().await.unwrap();
    assert_eq!(all_json["capabilities"].as_array().unwrap().len(), 2);

    // 2. Profile-scoped capabilities list via X-Warmplane-Profile header -> returns only server A
    let prof_resp = client
        .get(format!("{}/v1/capabilities", base_url))
        .header("x-warmplane-profile", "only_a")
        .send()
        .await
        .unwrap();
    assert_eq!(prof_resp.status(), StatusCode::OK);
    let prof_etag = prof_resp
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_ne!(all_etag, prof_etag);
    assert!(prof_etag.contains("-p:only_a"));
    let prof_json: Value = prof_resp.json().await.unwrap();
    let caps_arr = prof_json["capabilities"].as_array().unwrap();
    assert_eq!(caps_arr.len(), 1);
    assert_eq!(caps_arr[0]["id"], "a.tool");

    // 3. Conditional 304 Not Modified check with profile ETag
    let not_mod_resp = client
        .get(format!("{}/v1/capabilities", base_url))
        .header("x-warmplane-profile", "only_a")
        .header("if-none-match", &prof_etag)
        .send()
        .await
        .unwrap();
    assert_eq!(not_mod_resp.status(), StatusCode::NOT_MODIFIED);

    // 4. Query param profile selector: ?profile=only_a
    let query_resp = client
        .get(format!("{}/v1/capabilities?profile=only_a", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(query_resp.status(), StatusCode::OK);
    let query_json: Value = query_resp.json().await.unwrap();
    assert_eq!(query_json["capabilities"].as_array().unwrap().len(), 1);

    // 5. Unknown profile returns 404 PROFILE_NOT_FOUND
    let unknown_resp = client
        .get(format!("{}/v1/capabilities", base_url))
        .header("x-warmplane-profile", "non_existent")
        .send()
        .await
        .unwrap();
    assert_eq!(unknown_resp.status(), StatusCode::NOT_FOUND);
    let unknown_json: Value = unknown_resp.json().await.unwrap();
    assert_eq!(unknown_json["error"]["code"], "PROFILE_NOT_FOUND");

    // 6. Describe capability outside profile -> 404 TOOL_NOT_FOUND
    let desc_resp = client
        .get(format!("{}/v1/capabilities/b.tool", base_url))
        .header("x-warmplane-profile", "only_a")
        .send()
        .await
        .unwrap();
    assert_eq!(desc_resp.status(), StatusCode::NOT_FOUND);

    // 7. Search capabilities filtered to active profile
    let search_resp = client
        .post(format!("{}/v1/capabilities/search", base_url))
        .header("x-warmplane-profile", "only_a")
        .json(&json!({"query": "Tool"}))
        .send()
        .await
        .unwrap();
    assert_eq!(search_resp.status(), StatusCode::OK);
    let search_json: Value = search_resp.json().await.unwrap();
    let hits = search_json["capabilities"].as_array().unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["id"], "a.tool");

    // 8. Tool call to capability outside profile -> 403 TOOL_NOT_IN_PROFILE
    let call_resp = client
        .post(format!("{}/v1/tools/call", base_url))
        .header("x-warmplane-profile", "only_a")
        .json(&json!({
            "capability_id": "b.tool",
            "args": {}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(call_resp.status(), StatusCode::FORBIDDEN);
    let call_json: Value = call_resp.json().await.unwrap();
    assert_eq!(call_json["error"]["code"], "TOOL_NOT_IN_PROFILE");

    // 9. Batch call step outside profile -> step fails
    let batch_resp = client
        .post(format!("{}/v1/tools/batch_call", base_url))
        .header("x-warmplane-profile", "only_a")
        .json(&json!({
            "steps": [
                {
                    "id": "step1",
                    "capability_id": "b.tool",
                    "args": {},
                    "continue_on_error": false
                }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch_resp.status(), StatusCode::OK);
    let batch_json: Value = batch_resp.json().await.unwrap();
    assert_eq!(batch_json["ok"], false);
    assert!(batch_json["results"][0]["error"]
        .as_str()
        .unwrap()
        .contains("not in active profile"));

    // 10. List resources filtered to profile only_a
    let res_resp = client
        .get(format!("{}/v1/resources", base_url))
        .header("x-warmplane-profile", "only_a")
        .send()
        .await
        .unwrap();
    assert_eq!(res_resp.status(), StatusCode::OK);
    let res_json: Value = res_resp.json().await.unwrap();
    let res_arr = res_json["resources"].as_array().unwrap();
    assert_eq!(res_arr.len(), 1);
    assert_eq!(res_arr[0]["id"], "res.a");

    // 11. Read resource outside profile -> 404 RESOURCE_NOT_FOUND
    let read_resp = client
        .post(format!("{}/v1/resources/read", base_url))
        .header("x-warmplane-profile", "only_a")
        .json(&json!({
            "resource_id": "res.b"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(read_resp.status(), StatusCode::NOT_FOUND);

    // 12. List prompts filtered to profile only_a
    let prompts_resp = client
        .get(format!("{}/v1/prompts", base_url))
        .header("x-warmplane-profile", "only_a")
        .send()
        .await
        .unwrap();
    assert_eq!(prompts_resp.status(), StatusCode::OK);
    let prompts_json: Value = prompts_resp.json().await.unwrap();
    let prompts_arr = prompts_json["prompts"].as_array().unwrap();
    assert_eq!(prompts_arr.len(), 1);
    assert_eq!(prompts_arr[0]["id"], "prompt.a");

    // 13. Get prompt outside profile -> 404 PROMPT_NOT_FOUND
    let get_prompt_resp = client
        .post(format!("{}/v1/prompts/get", base_url))
        .header("x-warmplane-profile", "only_a")
        .json(&json!({
            "prompt_id": "prompt.b",
            "arguments": {}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(get_prompt_resp.status(), StatusCode::NOT_FOUND);
}
