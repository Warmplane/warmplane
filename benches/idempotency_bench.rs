// Rust guideline compliant 2026-08-14

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use warmplane::idempotency::IdempotencyStore;

fn bench_idempotency_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("idempotency_store");

    group.bench_function("single_key_lifecycle", |b| {
        let store = IdempotencyStore::new(Duration::from_secs(60));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut counter = 0u64;

        b.to_async(&rt).iter(|| {
            counter += 1;
            let key = format!("req_idem_key_{}", counter);
            let s = &store;
            async move {
                let _ = s.check_or_start(black_box(&key)).await;
                s.complete(black_box(&key), black_box(json!({"status": "ok"})))
                    .await;
                let _ = s.check_or_start(black_box(&key)).await;
            }
        });
    });

    group.bench_function("cache_hit_deduplication", |b| {
        let store = IdempotencyStore::new(Duration::from_secs(60));
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            let key = "cached_key";
            let _ = store.check_or_start(key).await;
            store
                .complete(key, json!({"status": "cached_result"}))
                .await;
        });

        b.to_async(&rt).iter(|| {
            let s = &store;
            async move { s.check_or_start(black_box("cached_key")).await }
        });
    });

    let concurrency_levels = [2, 8, 16];
    for &concurrency in &concurrency_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_contention", concurrency),
            &concurrency,
            |b, &threads| {
                let store = Arc::new(IdempotencyStore::new(Duration::from_secs(60)));
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(&rt).iter(|| {
                    let st = store.clone();
                    async move {
                        let mut handles = Vec::with_capacity(threads);
                        for t in 0..threads {
                            let s = st.clone();
                            handles.push(tokio::spawn(async move {
                                for i in 0..50 {
                                    let key = format!("concurrent_t{}_{}", t, i);
                                    let _ = s.check_or_start(&key).await;
                                    s.complete(&key, json!({"res": i})).await;
                                }
                            }));
                        }
                        for h in handles {
                            let _ = h.await;
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_idempotency_store);
criterion_main!(benches);
