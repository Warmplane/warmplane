// Rust guideline compliant 2026-08-14

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use warmplane::{
    catalog::CatalogEventStore,
    daemon::{compute_catalog_version, CapabilityMeta, PromptMeta, ResourceMeta},
};

fn generate_synthetic_catalog(
    count: usize,
) -> (
    HashMap<String, CapabilityMeta>,
    HashMap<String, ResourceMeta>,
    HashMap<String, PromptMeta>,
) {
    let mut caps = HashMap::with_capacity(count);
    let mut res = HashMap::with_capacity(count);
    let mut prompts = HashMap::with_capacity(count);

    for i in 0..count {
        let server = format!("srv_{}", i % 4);
        let cap_id = format!("{}_cap_{}", server, i);
        caps.insert(
            cap_id,
            CapabilityMeta {
                server: server.clone(),
                tool: format!("tool_{}", i),
                summary: format!("Summary for tool {}", i),
                description: format!("Description for tool {}", i),
                input_schema: json!({"type": "object"}),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                examples: vec![],
            },
        );

        let res_id = format!("{}_res_{}", server, i);
        res.insert(
            res_id.clone(),
            ResourceMeta {
                server: server.clone(),
                uri: format!("file:///path/to/resource_{}.txt", i),
                name: format!("Resource {}", i),
                description: Some(format!("Resource description {}", i)),
                mime_type: Some("text/plain".to_string()),
                tags: vec!["resource".to_string()],
            },
        );

        let prompt_id = format!("{}_prompt_{}", server, i);
        prompts.insert(
            prompt_id,
            PromptMeta {
                server,
                name: format!("prompt_template_{}", i),
                title: Some(format!("Prompt Title {}", i)),
                description: Some(format!("Prompt description {}", i)),
                arguments: vec![json!({"name": "input", "required": true})],
                tags: vec!["prompt".to_string()],
            },
        );
    }

    (caps, res, prompts)
}

fn bench_catalog_version_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("catalog_versioning");
    let sizes = [10, 50, 200, 500];

    for &size in &sizes {
        let (caps, res, prompts) = generate_synthetic_catalog(size);

        group.bench_with_input(
            BenchmarkId::new("compute_catalog_version", size),
            &(caps, res, prompts),
            |b, (c, r, p)| {
                b.iter(|| {
                    compute_catalog_version(black_box(c), black_box(r), black_box(p));
                });
            },
        );
    }
    group.finish();
}

fn bench_catalog_event_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("catalog_event_store");

    group.bench_function("record_event_sequential", |b| {
        let store = CatalogEventStore::new();
        let mut i = 0;
        b.iter(|| {
            i += 1;
            store.record_with_detail(
                black_box("capability"),
                black_box(format!("cap_{}", i)),
                black_box("added"),
                black_box(Some("Capability dynamically registered")),
            );
        });
    });

    group.bench_function("get_events_after_cursor_scan", |b| {
        let store = CatalogEventStore::new();
        for i in 1..=500 {
            store.record_with_detail("capability", format!("cap_{}", i), "added", None::<String>);
        }
        // Test pagination starting from event 250
        let cursor = "evt_250";
        b.iter(|| {
            store.get_events_after(black_box(Some(cursor)));
        });
    });

    group.bench_function("concurrent_record_events", |b| {
        let store = Arc::new(CatalogEventStore::new());
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| {
                let s = store.clone();
                async move {
                    let mut handles = Vec::with_capacity(8);
                    for t in 0..8 {
                        let st = s.clone();
                        handles.push(tokio::spawn(async move {
                            for i in 0..20 {
                                st.record("capability", format!("t{}_cap_{}", t, i), "updated");
                            }
                        }));
                    }
                    for h in handles {
                        let _ = h.await;
                    }
                }
            });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_catalog_version_computation,
    bench_catalog_event_store
);
criterion_main!(benches);
