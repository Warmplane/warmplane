// Rust guideline compliant 2026-08-14

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::collections::HashMap;
use warmplane::{
    daemon::{CapabilityMeta, Policy},
    search::{
        hybrid::{HybridSearchEngine, SearchFilter},
        lexical::score_lexical,
    },
};

fn generate_synthetic_capabilities(count: usize) -> HashMap<String, CapabilityMeta> {
    let mut map = HashMap::with_capacity(count);
    let sample_tags = [
        vec!["read".to_string(), "filesystem".to_string()],
        vec!["write".to_string(), "git".to_string()],
        vec!["execute".to_string(), "terminal".to_string()],
        vec![
            "database".to_string(),
            "sql".to_string(),
            "read".to_string(),
        ],
        vec![
            "network".to_string(),
            "http".to_string(),
            "fetch".to_string(),
        ],
    ];

    for i in 0..count {
        let server = format!("server_{}", i % 5);
        let tool = format!("tool_action_{}", i);
        let id = format!("{}__{}", server, tool);
        let tags = sample_tags[i % sample_tags.len()].clone();
        let summary = format!(
            "Executes action number {} on downstream server {}",
            i, server
        );
        let description = format!(
            "Comprehensive tool description for operation {}. Supports querying, filtering, and executing commands across {} clusters.",
            i, server
        );
        let input_schema = json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "limit": { "type": "integer" },
                "filter": { "type": "string" }
            },
            "required": ["id"]
        });

        map.insert(
            id,
            CapabilityMeta {
                server,
                tool,
                summary,
                description,
                input_schema,
                tags,
                examples: vec![],
            },
        );
    }
    map
}

fn bench_lexical_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexical_search");
    let sizes = [10, 100, 500, 1000];

    for &size in &sizes {
        let caps_map = generate_synthetic_capabilities(size);
        let caps_vec: Vec<(String, CapabilityMeta)> = caps_map.into_iter().collect();

        group.bench_with_input(
            BenchmarkId::new("exact_id_match", size),
            &caps_vec,
            |b, items| {
                let target_id = &items[items.len() / 2].0;
                b.iter(|| {
                    score_lexical(black_box(target_id), black_box(items));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("tag_match", size),
            &caps_vec,
            |b, items| {
                b.iter(|| {
                    score_lexical(black_box("database"), black_box(items));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("fuzzy_description_overlap", size),
            &caps_vec,
            |b, items| {
                b.iter(|| {
                    score_lexical(
                        black_box("comprehensive querying filtering clusters"),
                        black_box(items),
                    );
                });
            },
        );
    }
    group.finish();
}

fn bench_hybrid_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_search");
    let engine = HybridSearchEngine::new();
    let policy = Policy::default();
    let sizes = [50, 200, 1000];

    for &size in &sizes {
        let caps = generate_synthetic_capabilities(size);
        let filter_all = SearchFilter::default();
        let filter_server = SearchFilter::builder().server_id("server_1").build();

        group.bench_with_input(
            BenchmarkId::new("unfiltered_limit_8", size),
            &caps,
            |b, items| {
                b.iter(|| {
                    engine.search(
                        black_box("terminal command execution"),
                        black_box(8),
                        black_box(&filter_all),
                        black_box(items),
                        black_box(&policy),
                    );
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("filtered_by_server_limit_8", size),
            &caps,
            |b, items| {
                b.iter(|| {
                    engine.search(
                        black_box("action query"),
                        black_box(8),
                        black_box(&filter_server),
                        black_box(items),
                        black_box(&policy),
                    );
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_lexical_queries, bench_hybrid_search);
criterion_main!(benches);
