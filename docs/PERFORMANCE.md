# Performance & Benchmark Guide

Warmplane is designed as an ultra-low-overhead, memory-safe local control plane and proxy for Model Context Protocol (MCP) sessions.

This document details the performance characteristics, micro-benchmark results, system implications, and optimization strategies of Warmplane across its core subsystems.

---

## 🚀 Key Takeaways

1. **Sub-Microsecond Facade Overheads:**
   - Zero-copy ETag-cached catalog validation (`If-None-Match` $\rightarrow$ `304 Not Modified`) takes **$50.4\ \text{ns}$** (a **$66.3\%$ speedup** from baseline), enabling high-frequency client polling with essentially zero CPU or allocation overhead.
   - Header mismatch detection takes **$27.4\ \text{ns}$** (a **$77.5\%$ speedup**), and absent header checks take **$1.98\ \text{ns}$**.
2. **Deterministic Catalog Hashing:**
   - Incremental SHA-256 catalog ETag version hashing (`compute_catalog_version`) takes **$1.58\ \mu\text{s}$** for 10 capabilities and **$126\ \mu\text{s}$** for 500 capabilities.
3. **High-Throughput Idempotency:**
   - Cached deduplication hit lookup runs in **$159.8\ \text{ns}$**.
   - Concurrent contention across 16 parallel async workers completing 800 operations takes **$550\ \mu\text{s}$ total** ($<0.7\ \mu\text{s}$ per operation, an **$83.1\%$ improvement** over naive lock retention).
4. **Optimized Capability Search:**
   - Zero-allocation lexical search scans 1,000 capabilities across exact matches, tags, and multi-token fuzzy descriptions in **$371\ \mu\text{s}$ to $591\ \mu\text{s}$** (a **$36\%$–$56\%$ speedup**).
   - Reciprocal Rank Fusion (RRF) hybrid search with pre-allocated vector scores finishes in **$15.9\ \mu\text{s}$** (filtered, $N=50$) and **$1.40\ \text{ms}$** (unfiltered, $N=1,000$).

---

## 📊 Benchmark Results

All benchmarks are measured using [Criterion.rs](https://github.com/bheisler/criterion.rs) with 100 samples across warmup periods on Apple Silicon (M-series).

### 1. Capability Search Subsystem (`search_bench`)

Measures query scoring across exact ID matches, tag filtering, and multi-token fuzzy description overlap against synthetic catalogs ($N = 10, 100, 500, 1000$).

| Workload | Catalog Size ($N$) | Latency ($p_{50}$) | Throughput / Scaling Notes |
| :--- | :--- | :--- | :--- |
| **Exact ID Match** | 10 | **$1.17\ \mu\text{s}$** | Fast ASCII equality path |
| | 100 | **$61.3\ \mu\text{s}$** | In-place ASCII comparison |
| | 500 | **$347.7\ \mu\text{s}$** | In-place ASCII comparison |
| | 1,000 | **$591.8\ \mu\text{s}$** | ~1.7M candidate evaluations/sec |
| **Tag Match** | 10 | **$1.15\ \mu\text{s}$** | Short-circuit tag match |
| | 100 | **$37.4\ \mu\text{s}$** | Short-circuit tag match |
| | 500 | **$184.4\ \mu\text{s}$** | Short-circuit tag match |
| | 1,000 | **$372.1\ \mu\text{s}$** | ~2.7M candidate evaluations/sec |
| **Fuzzy Description Overlap** | 10 | **$9.88\ \mu\text{s}$** | Zero-copy streaming token matching |
| | 100 | **$52.6\ \mu\text{s}$** | Zero-copy streaming token matching |
| | 500 | **$262.6\ \mu\text{s}$** | Zero-copy streaming token matching |
| | 1,000 | **$525.0\ \mu\text{s}$** | ~1.9M candidate evaluations/sec |
| **Hybrid Search (RRF Filtered)** | 50 | **$15.9\ \mu\text{s}$** | Server ID filter + RRF fusion |
| | 200 | **$59.2\ \mu\text{s}$** | Server ID filter + RRF fusion |
| | 1,000 | **$305.8\ \mu\text{s}$** | Server ID filter + RRF fusion |
| **Hybrid Search (RRF Unfiltered)** | 50 | **$63.4\ \mu\text{s}$** | Full candidate set + RRF fusion |
| | 200 | **$268.4\ \mu\text{s}$** | Full candidate set + RRF fusion |
| | 1,000 | **$1.41\ \text{ms}$** | Full candidate set + RRF fusion |

---

### 2. HTTP Facade & Serialization (`facade_bench`)

Measures HTTP header parsing, JSON response payload mapping, and configuration deserialization.

| Benchmark Target | Workload / Scale | Latency ($p_{50}$) | Details |
| :--- | :--- | :--- | :--- |
| **`check_if_none_match` (304)** | ETag Match | **$50.4\ \text{ns}$** | Zero-copy quote-trim validation (**66.3% faster**) |
| **`check_if_none_match` (Mismatch)** | ETag Mismatch | **$27.4\ \text{ns}$** | Non-matching quote-trim (**77.5% faster**) |
| **`check_if_none_match` (Absent)**| Header Absent | **$1.98\ \text{ns}$** | Direct HeaderMap lookup miss (**33.1% faster**) |
| **`serialize_capabilities_list`**| 10 capabilities | **$8.05\ \mu\text{s}$** | Map to compact facade + sort + JSON |
| | 100 capabilities | **$85.0\ \mu\text{s}$** | Map to compact facade + sort + JSON |
| | 500 capabilities | **$499.4\ \mu\text{s}$** | Map to compact facade + sort + JSON |
| | 1,000 capabilities | **$1.05\ \text{ms}$** | Map to compact facade + sort + JSON |
| **`deserialize_mcp_config`** | 50 server config JSON | **$28.3\ \mu\text{s}$** | Full serde deserialization |

---

### 3. Idempotency Deduplication Store (`idempotency_bench`)

Measures key checking, lock contention, and cache lookup performance for idempotent operations.

| Benchmark Target | Concurrency / Scale | Latency ($p_{50}$) | Details |
| :--- | :--- | :--- | :--- |
| **`single_key_lifecycle`** | 1 thread | **$2.46\ \mu\text{s}$** | Check $\rightarrow$ Insert $\rightarrow$ Complete $\rightarrow$ Verify (**41.1% faster**) |
| **`cache_hit_deduplication`** | 1 thread | **$159.8\ \text{ns}$** | Immediate return of cached value |
| **`concurrent_contention`** | 2 threads (100 ops) | **$51.3\ \mu\text{s}$** | Amortized eviction under concurrency (**49.9% faster**) |
| | 8 threads (400 ops) | **$273.1\ \mu\text{s}$** | Amortized eviction under concurrency (**71.2% faster**) |
| | 16 threads (800 ops) | **$550.7\ \mu\text{s}$** | Amortized eviction under concurrency (**83.1% faster**) |

---

### 4. Catalog & Versioning Subsystem (`catalog_bench`)

Measures SHA-256 version computation for cache ETags and high-concurrency event store recording.

| Benchmark Target | Catalog Size ($N$) | Latency ($p_{50}$) | Details |
| :--- | :--- | :--- | :--- |
| **`compute_catalog_version`** | 10 items | **$1.58\ \mu\text{s}$** | Keys sorted + SHA256 hashed |
| | 50 items | **$9.63\ \mu\text{s}$** | Keys sorted + SHA256 hashed |
| | 200 items | **$42.0\ \mu\text{s}$** | Keys sorted + SHA256 hashed |
| | 500 items | **$126.0\ \mu\text{s}$** | Keys sorted + SHA256 hashed |
| **`record_event_sequential`** | Single event | **$227.8\ \text{ns}$** | In-memory append with timestamp |
| **`get_events_after`** | 500 events (from 250) | **$27.9\ \mu\text{s}$** | Cursor scan + slice clone |
| **`concurrent_record_events`** | 8 threads (160 events)| **$66.3\ \mu\text{s}$** | Concurrent thread safety under write lock |

---

## 🛠️ Running the Benchmarks

To execute the Criterion benchmark suites locally:

```bash
# Run all benchmark targets
cargo bench

# Run a specific benchmark suite
cargo bench --bench search_bench
cargo bench --bench catalog_bench
cargo bench --bench idempotency_bench
cargo bench --bench facade_bench

# Run a specific test filter
cargo bench --bench search_bench -- "tag_match"
```

Criterion generates interactive HTML reports, throughput plots, and regression metrics in `target/criterion/`.
