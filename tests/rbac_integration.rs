// Rust guideline compliant 2026-08-19

//! Integration tests for Multi-Tenant Role-Based Access Control (RBAC) & Catalog Partitioning.

use axum::{
    body::to_bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;

use warmplane::{
    daemon::{AppState, CapabilityMeta, PromptMeta, ResourceMeta},
    http_v1::{
        catalog::{handle_list_capabilities, handle_search_capabilities},
        execute::handle_call_capability,
        types::{CallCapabilityRequest, SearchCapabilitiesRequest},
    },
    rbac::{JwtConfig, RbacConfig, RbacEngine, RolePolicyConfig, TokenAssignment},
};

fn create_test_rbac_state() -> AppState {
    let mut tokens = HashMap::new();
    tokens.insert(
        "admin_secret_token".to_string(),
        TokenAssignment {
            role: "admin".to_string(),
            tenant_id: Some("tenant-corp".to_string()),
            actor_id: Some("admin-bot".to_string()),
            description: Some("Admin full token".to_string()),
        },
    );
    tokens.insert(
        "analyst_secret_token".to_string(),
        TokenAssignment {
            role: "analyst".to_string(),
            tenant_id: Some("tenant-analytics".to_string()),
            actor_id: Some("analyst-bot".to_string()),
            description: Some("Analyst query token".to_string()),
        },
    );

    let mut roles = HashMap::new();
    roles.insert(
        "admin".to_string(),
        RolePolicyConfig {
            description: Some("Full administrator access".to_string()),
            allow: vec!["*".to_string()],
            deny: vec![],
            require_approval: vec![],
            redact_keys: vec![],
        },
    );
    roles.insert(
        "analyst".to_string(),
        RolePolicyConfig {
            description: Some("Data analyst restricted scope".to_string()),
            allow: vec!["db.query".to_string(), "fs.read_*".to_string(), "prompt.analytics_*".to_string(), "res.public_*".to_string()],
            deny: vec!["*.delete_*".to_string(), "docker.*".to_string()],
            require_approval: vec!["db.query_large".to_string()],
            redact_keys: vec!["password".to_string(), "ssn".to_string()],
        },
    );
    roles.insert(
        "anonymous".to_string(),
        RolePolicyConfig {
            description: Some("Unauthenticated public access".to_string()),
            allow: vec!["prompt.public_*".to_string()],
            deny: vec!["*".to_string()],
            require_approval: vec![],
            redact_keys: vec![],
        },
    );

    let rbac_config = RbacConfig {
        enabled: true,
        default_role: "anonymous".to_string(),
        jwt: Some(JwtConfig {
            secret: Some("symmetric_secret_key_12345".to_string()),
            ..Default::default()
        }),
        tokens,
        roles,
    };

    let mut capabilities = HashMap::new();
    capabilities.insert(
        "db.query".to_string(),
        CapabilityMeta {
            server: "db_srv".to_string(),
            tool: "query".to_string(),
            summary: "Execute read SQL query".to_string(),
            description: "Read SQL query".to_string(),
            input_schema: json!({}),
            tags: vec!["db".to_string(), "sql".to_string()],
            examples: vec![],
        },
    );
    capabilities.insert(
        "docker.run".to_string(),
        CapabilityMeta {
            server: "docker_srv".to_string(),
            tool: "run".to_string(),
            summary: "Run docker container".to_string(),
            description: "Run container".to_string(),
            input_schema: json!({}),
            tags: vec!["docker".to_string()],
            examples: vec![],
        },
    );
    capabilities.insert(
        "fs.delete_file".to_string(),
        CapabilityMeta {
            server: "fs_srv".to_string(),
            tool: "delete_file".to_string(),
            summary: "Delete a file from disk".to_string(),
            description: "Delete file".to_string(),
            input_schema: json!({}),
            tags: vec!["fs".to_string()],
            examples: vec![],
        },
    );

    let mut resources = HashMap::new();
    resources.insert(
        "res.public_readme".to_string(),
        ResourceMeta {
            server: "fs_srv".to_string(),
            uri: "file:///public/readme.txt".to_string(),
            name: "Public Readme".to_string(),
            description: None,
            mime_type: None,
            tags: vec![],
        },
    );
    resources.insert(
        "res.private_keys".to_string(),
        ResourceMeta {
            server: "fs_srv".to_string(),
            uri: "file:///private/id_rsa".to_string(),
            name: "Private Keys".to_string(),
            description: None,
            mime_type: None,
            tags: vec![],
        },
    );

    let mut prompts = HashMap::new();
    prompts.insert(
        "prompt.public_hello".to_string(),
        PromptMeta {
            server: "prompt_srv".to_string(),
            name: "hello".to_string(),
            title: None,
            description: None,
            arguments: vec![],
            tags: vec![],
        },
    );
    prompts.insert(
        "prompt.analytics_summary".to_string(),
        PromptMeta {
            server: "prompt_srv".to_string(),
            name: "summary".to_string(),
            title: None,
            description: None,
            arguments: vec![],
            tags: vec![],
        },
    );
    prompts.insert(
        "prompt.admin_reboot".to_string(),
        PromptMeta {
            server: "prompt_srv".to_string(),
            name: "reboot".to_string(),
            title: None,
            description: None,
            arguments: vec![],
            tags: vec![],
        },
    );

    let rbac_engine = RbacEngine::new(Some(rbac_config));

    AppState::builder()
        .capabilities(capabilities)
        .resources(resources)
        .prompts(prompts)
        .rbac_engine(rbac_engine)
        .catalog_version("sha256:test_cat_rbac")
        .build()
}

#[tokio::test]
async fn test_rbac_catalog_partitioning_by_role() {
    let state = create_test_rbac_state();
    let base_policy = state.policy.read().await.clone();

    // 1. Unauthenticated / anonymous caller
    let anon_ctx = state.rbac_engine.authenticate(None, &base_policy).unwrap();
    let res = handle_list_capabilities(
        State(state.clone()),
        axum::extract::Extension(Some(anon_ctx)),
        HeaderMap::new(),
    )
    .await
    .into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    let caps = payload["capabilities"].as_array().unwrap();
    assert_eq!(caps.len(), 0, "Anonymous role should see 0 capabilities");

    // 2. Analyst caller
    let analyst_ctx = state
        .rbac_engine
        .authenticate(Some("analyst_secret_token"), &base_policy)
        .unwrap();
    let res = handle_list_capabilities(
        State(state.clone()),
        axum::extract::Extension(Some(analyst_ctx.clone())),
        HeaderMap::new(),
    )
    .await
    .into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    let caps = payload["capabilities"].as_array().unwrap();
    assert_eq!(caps.len(), 1, "Analyst should see exactly db.query");
    assert_eq!(caps[0]["id"], "db.query");

    // 3. Admin caller
    let admin_ctx = state
        .rbac_engine
        .authenticate(Some("admin_secret_token"), &base_policy)
        .unwrap();
    let res = handle_list_capabilities(
        State(state.clone()),
        axum::extract::Extension(Some(admin_ctx)),
        HeaderMap::new(),
    )
    .await
    .into_response();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    let caps = payload["capabilities"].as_array().unwrap();
    assert_eq!(caps.len(), 3, "Admin should see all 3 capabilities");
}

#[tokio::test]
async fn test_rbac_search_pruning_by_role() {
    let state = create_test_rbac_state();
    let base_policy = state.policy.read().await.clone();

    let analyst_ctx = state
        .rbac_engine
        .authenticate(Some("analyst_secret_token"), &base_policy)
        .unwrap();

    let res = handle_search_capabilities(
        State(state.clone()),
        axum::extract::Extension(Some(analyst_ctx)),
        Json(SearchCapabilitiesRequest {
            query: Some("docker".to_string()),
            limit: 10,
            server_ids: vec![],
            tags: vec![],
            modes: vec![],
        }),
    )
    .await
    .into_response();

    assert_eq!(res.status(), StatusCode::OK);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    let results = payload["capabilities"].as_array().unwrap();
    assert_eq!(results.len(), 0, "Analyst search should prune docker capabilities");
}

#[tokio::test]
async fn test_rbac_execution_boundary_enforcement() {
    let state = create_test_rbac_state();
    let base_policy = state.policy.read().await.clone();

    let analyst_ctx = state
        .rbac_engine
        .authenticate(Some("analyst_secret_token"), &base_policy)
        .unwrap();

    // Analyst attempts unauthorized execution of docker.run
    let req = CallCapabilityRequest {
        capability_id: "docker.run".to_string(),
        args: json!({"image": "alpine"}),
        request_id: Some("req-unauth-test".to_string()),
        context: None,
        idempotency_key: None,
        input_responses: None,
        request_state: None,
    };

    let res = handle_call_capability(
        State(state.clone()),
        axum::extract::Extension(Some(analyst_ctx)),
        HeaderMap::new(),
        Json(req),
    )
    .await
    .into_response();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let payload: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "POLICY_DENIED");
}
