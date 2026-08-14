// Rust guideline compliant 2026-08-14

use axum::http::{header, HeaderMap, HeaderValue};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::collections::HashMap;
use warmplane::{config::McpConfig, daemon::CapabilityMeta, http_v1::check_if_none_match};

fn generate_synthetic_capabilities(count: usize) -> HashMap<String, CapabilityMeta> {
    let mut map = HashMap::with_capacity(count);
    for i in 0..count {
        let server = format!("server_{}", i % 5);
        let tool = format!("tool_name_{}", i);
        let id = format!("{}__{}", server, tool);
        map.insert(
            id,
            CapabilityMeta {
                server,
                tool,
                summary: format!("Summary text for capability tool {}", i),
                description: format!("Detailed description for capability tool {}", i),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "param1": { "type": "string" },
                        "param2": { "type": "integer" }
                    }
                }),
                tags: vec!["tag_a".to_string(), "tag_b".to_string()],
                examples: vec![],
            },
        );
    }
    map
}

fn bench_etag_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("facade_etag");
    let version = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let mut match_headers = HeaderMap::new();
    match_headers.insert(
        header::IF_NONE_MATCH,
        HeaderValue::from_str(&format!("\"{}\"", version)).unwrap(),
    );

    let mut mismatch_headers = HeaderMap::new();
    mismatch_headers.insert(
        header::IF_NONE_MATCH,
        HeaderValue::from_static("\"sha256:stale_etag_value\""),
    );

    let empty_headers = HeaderMap::new();

    group.bench_function("etag_match_304", |b| {
        b.iter(|| {
            check_if_none_match(black_box(&match_headers), black_box(version));
        });
    });

    group.bench_function("etag_mismatch", |b| {
        b.iter(|| {
            check_if_none_match(black_box(&mismatch_headers), black_box(version));
        });
    });

    group.bench_function("etag_header_absent", |b| {
        b.iter(|| {
            check_if_none_match(black_box(&empty_headers), black_box(version));
        });
    });

    group.finish();
}

fn bench_capabilities_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("facade_serialization");
    let sizes = [10, 100, 500, 1000];

    for &size in &sizes {
        let caps = generate_synthetic_capabilities(size);

        group.bench_with_input(
            BenchmarkId::new("serialize_capabilities_list", size),
            &caps,
            |b, items| {
                b.iter(|| {
                    let mut capabilities = items
                        .iter()
                        .map(|(id, meta)| {
                            json!({
                                "id": id,
                                "summary": &meta.summary,
                                "server": &meta.server,
                                "tool": &meta.tool,
                                "tags": &meta.tags,
                            })
                        })
                        .collect::<Vec<_>>();

                    capabilities.sort_by(|a, b| {
                        a.get("id")
                            .and_then(|v| v.as_str())
                            .cmp(&b.get("id").and_then(|v| v.as_str()))
                    });

                    black_box(json!({
                        "version": "v1",
                        "catalog_version": "sha256:test",
                        "ttl_ms": 300000,
                        "cache_scope": "public",
                        "capabilities": capabilities,
                    }));
                });
            },
        );
    }

    group.finish();
}

fn bench_config_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");

    // Generate large synthetic mcp_servers.json configuration string
    let mut servers_map = serde_json::Map::new();
    for i in 0..50 {
        servers_map.insert(
            format!("server_{}", i),
            json!({
                "command": "npx",
                "args": ["-y", format!("@modelcontextprotocol/server-{}", i)],
                "env": {
                    "API_KEY": "test_key_value",
                    "DEBUG": "true"
                },
                "protocolVersion": "2026-07-28",
                "allowStateless": true
            }),
        );
    }
    let config_json = json!({
        "port": 9090,
        "toolTimeoutMs": 15000,
        "capabilityAliases": {
            "fetch": "server_0__fetch",
            "git": "server_1__git"
        },
        "mcpServers": servers_map
    })
    .to_string();

    group.bench_function("deserialize_mcp_config_50_servers", |b| {
        b.iter(|| {
            let parsed: McpConfig = serde_json::from_str(black_box(&config_json)).unwrap();
            black_box(parsed);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_etag_validation,
    bench_capabilities_serialization,
    bench_config_deserialization
);
criterion_main!(benches);
