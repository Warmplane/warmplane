# Warmplane Extension Plan & Integrator Specification

## Executive Summary

Warmplane serves as a compact, high-performance local control plane and HTTP bridge for Model Context Protocol (MCP) servers. By maintaining persistent ("warm") upstream connections, applying security policies, and providing stable resource/capability facades, Warmplane eliminates session overhead and simplifies MCP integration for AI agent applications.

This plan addresses the requirements outlined in [`docs/INTEGRATOR_WISHLIST.md`](file:///Users/origo/src/warmplane/docs/INTEGRATOR_WISHLIST.md). It outlines the architecture, evaluation, technical designs, and implementation roadmap required to transform Warmplane into an embedding-ready, highly observable MCP control plane without compromising its core principle: remaining a lightweight control facade rather than an application-level agent runtime.

---

## Architectural Principles & Non-Goals

### Core Philosophy
1. **Facade Scoping**: Warmplane handles MCP protocol state, session warming, credential management, catalog normalization, capability execution, and policy enforcement.
2. **Application Autonomy**: The integrating application retains ownership of user identity, task authorization, approval workflows, long-term operational audit logs, and prompt orchestration.
3. **Deterministic Governance**: Upstream tool discovery, filtering, and execution are predictable, auditable, and strictly governed by Warmplane policy.
4. **Zero-Lock-in & Graceful Degradation**: Features like semantic search degrade seamlessly to lexical search when embeddings are disabled or models are unavailable.

### Explicit Non-Goals
* Warmplane will **not** build agent orchestration loops, task planners, or multi-step execution graphs.
* Warmplane will **not** manage application user permissions, databases, or end-user identity providers.
* Warmplane will **not** store long-term business execution logs beyond ephemeral operational history needed for deduplication, trace correlation, and metrics.

---

## Integrator Wishlist Analysis & Requirements

| Priority | Feature Area | Key Requirement | Architectural Solution |
| :--- | :--- | :--- | :--- |
| **P0** | **Capability Search** | Efficient discovery across hundreds of MCP capabilities without sending full catalogs to LLMs. | `POST /v1/capabilities/search` endpoint and CLI matching with hybrid lexical + semantic search and deterministic filtering. |
| **P0** | **Catalog Versions** | Cache validation and change tracking for capability/resource/prompt catalogs. | Monotonic/SHA-256 `catalog_version`, `ETag` / `If-None-Match` HTTP validation, and optional event feed (`GET /v1/catalog/events`). |
| **P0** | **Request Context** | Trace correlation between application operations and Warmplane executions. | Pass-through untrusted `context` object, validated size limits, OTEL trace attribute mapping, and log redaction. |
| **P1** | **Operations Metadata** | Idempotent writes, cancellation, and execution state recovery. | `idempotency_key`, `deadline_ms`, `POST /v1/operations/{request_id}/cancel`, and explicit `retry` classification envelopes. |
| **P1** | **Result Shaping** | Context window optimization and sensitive data filtering on tool outputs. | Result controls including max payload bytes, JSONPath allowlist/denylist, truncation markers, and auto-redaction. |
| **P1** | **Health & Auth Signals** | Actionable status of upstream servers and credentials without exposing secrets. | `GET /v1/health/upstreams` exposing server state, OAuth token validity (`valid`, `expires_soon`, `auth_required`), and connection latency. |
| **P2** | **Policy Overlays** | Narrow, short-lived permission grants for specific agent steps. | Per-request policy overlay parameter that can strictly *narrow* (never widen) static policy rules. |
| **P2** | **Events & Namespaces** | Operational recovery and multi-tenant isolation. | Lifecycle event cursors and namespace configuration isolation for multi-environment deployments. |

---

## Semantic Search Architectural Research & Strategy

### Objective
Provide natural language query resolution over registered MCP capability metadata (IDs, names, summaries, descriptions, input schemas, and tags) while keeping boot times low, memory footprint minimal, and requiring zero external server dependencies (e.g., Qdrant, Pinecone).

### Evaluated Options for Semantic Search in Rust

```
+-----------------------------------------------------------------------------------+
|                            Capability Search Architecture                          |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|   Query: "triage production errors"                                               |
|      |                                                                            |
|      +-----------------------+-----------------------+                            |
|      |                       |                       |                            |
|      v                       v                       v                            |
|  [Lexical Filter]    [BM25 / Fuzzy Token]   [FastEmbed ONNX CPU]                  |
|  (Server, Tag, Mode) (ID, Alias, Acronym)   (All-MiniLM-L6-v2 Embed)             |
|      |                       |                       |                            |
|      |                       +-----------+-----------+                            |
|      |                                   |                                        |
|      v                                   v                                        |
|  Deterministic                Reciprocal Rank Fusion                              |
|  Boolean Filter                 (RRF Hybrid Scoring)                              |
|      |                                   |                                        |
|      +-------------------+---------------+                                        |
|                          |                                                        |
|                          v                                                        |
|             Ranked Capability Summaries                                          |
+-----------------------------------------------------------------------------------+
```

#### Option A: Pure Lexical / Keyword Matching (Baseline)
* **Mechanism**: Substring, token matching, and BM25 / trigram scoring over capability fields.
* **Pros**: Zero dependencies, 0ms startup time, ultra-low memory overhead (~100KB).
* **Cons**: Fails on natural language queries where intent does not share exact vocabulary (e.g., query "fix bug" vs tool summary "patch code flaw").

#### Option B: Embedded ONNX Runtime (`fastembed-rs` + In-Memory Flat Vector Store) **(Recommended)**
* **Mechanism**: Uses `fastembed` (wrapping ONNX Runtime) with quantized `all-MiniLM-L6-v2` (23MB model) to compute 384-dimensional embeddings for capability metadata upon catalog refresh. Search queries compute a single 384-dim query vector and perform SIMD-accelerated brute-force cosine similarity against the in-memory array of capability vectors.
* **Performance Benchmark**: For catalogs up to 10,000 tools:
  * Index build time: ~50ms for 500 capabilities.
  * Search latency: < 2ms CPU inference + < 0.1ms vector dot-product.
  * Memory footprint: ~30MB (ONNX model weights + vector cache).
* **Pros**: Completely local, self-contained, high accuracy for intent matching, zero external network dependency or API keys.
* **Cons**: Adds small binary overhead (`ort` static/dynamic linking) and ~25MB memory footprint.

#### Option C: Remote Embedding API Client (OpenAI / Ollama / Custom HTTP Endpoint)
* **Mechanism**: Warmplane forwards capability text and search queries to an external embedding API.
* **Pros**: Zero binary size increase, allows high-capacity models (e.g., `text-embedding-3-small`).
* **Cons**: Introduces external network dependency, latency (~100-300ms), API cost, and privacy concerns (sending catalog metadata out-of-process).

#### Option D: Hybrid Retrieval (Lexical + Semantic with Reciprocal Rank Fusion)
* **Mechanism**: Combine Lexical BM25 rank ($R_{lex}$) and Semantic Cosine rank ($R_{sem}$) using Reciprocal Rank Fusion:
  $$Score(d) = \frac{w_{lex}}{k + R_{lex}(d)} + \frac{w_{sem}}{k + R_{sem}(d)}$$
* **Pros**: Handles both exact technical identifiers (e.g., `github.issue_123`) and fuzzy intent ("find open bugs").

### Recommended Hybrid Search Architecture
Warmplane will implement **Option D** using an optional, feature-gated embedded ONNX pipeline:

1. **Compilation Gate**: Compile semantic search conditionally via `cfg(feature = "semantic-search")` using `fastembed`.
2. **Runtime Degradation**:
   * If `search.semantic.enabled = false` in `mcp_servers.json` or model downloading is skipped, search automatically falls back to pure BM25/Fuzzy lexical ranking.
   * Capability matching tags indicate match provenance: `["exact_id"]`, `["tag"]`, `["lexical"]`, or `["semantic"]`.
3. **In-Memory Flat Vector Index**: Because MCP server catalogs contain hundreds (not millions) of capabilities, an in-memory `Vec<(String, Vec<f32>)>` with standard Rust SIMD cosine similarity is faster and vastly simpler than introducing a heavy vector DB extension (such as SQLite-vec or LanceDB).

---

## Codebase Exploration & Target Reorganization

### Current Architecture Assessment
Currently, Warmplane codebase comprises:
* [`src/daemon.rs`](file:///Users/origo/src/warmplane/src/daemon.rs): Manages `AppState`, MCP upstream server initialization via stdio/HTTP transport, actor loops, and Axum routing.
* [`src/http_v1.rs`](file:///Users/origo/src/warmplane/src/http_v1.rs): Axum handlers for listing, describing, reading, getting, and calling capabilities/resources/prompts.
* [`src/config.rs`](file:///Users/origo/src/warmplane/src/config.rs): Serde structures for `McpConfig`, `ServerConfig`, `PolicyConfig`, and validation logic.
* [`src/oauth2.rs`](file:///Users/origo/src/warmplane/src/oauth2.rs): Dynamic OAuth2 PKCE flow, discovery, token cache, and proxy bridge.
* [`src/models.rs`](file:///Users/origo/src/warmplane/src/models.rs): Clap CLI commands.

### Proposed Codebase Reorganization
To support the extension suite cleanly, the code will be structured into dedicated domain modules:

```text
src/
├── catalog/           # Catalog versioning, SHA256 hashing, ETag generation, change feed
│   ├── mod.rs
│   └── events.rs
├── search/            # Capability search engine
│   ├── mod.rs
│   ├── lexical.rs     # BM25 / Fuzzy token matching engine
│   ├── vector.rs      # FastEmbed / SIMD vector dot product
│   └── hybrid.rs      # RRF fusion logic
├── context/           # Request context validation, size bounds, trace context propagation
│   └── mod.rs
├── operations/        # Operation tracker, idempotency store, cancellation channels
│   ├── mod.rs
│   └── store.rs
├── shaping/           # Result truncation, JSONPath allow/deny, redact filter
│   └── mod.rs
├── health/            # Upstream monitor & OAuth auth state inspector
│   └── mod.rs
├── policy/            # Static policy + temporary request-bound overlay evaluator
│   └── mod.rs
├── config.rs          # Extended configuration structures
├── daemon.rs          # Axum router setup & server actor supervisor
├── http_v1.rs         # V1 API handlers
└── models.rs          # CLI definitions
```

---

## Detailed Specifications & Technical Design

### 1. Catalog Versioning & Cache Validation (P0)

#### SHA-256 Catalog Hash Calculation
Whenever upstream servers complete listing capabilities, resources, or prompts, `CatalogState` computes a deterministic SHA-256 digest over the canonical JSON representation of all registered items and active policy rules:

$$\text{catalog\_version} = \text{"sha256:"} + \text{Hex}(\text{SHA256}(\text{CanonicalCatalogJson}))$$

#### HTTP Headers & Conditional Reads
* All `/v1/capabilities*`, `/v1/resources*`, `/v1/prompts*` responses include:
  * Header: `ETag: "sha256:..."`
  * Body Envelope Field: `"catalog_version": "sha256:..."`
* GET handlers process `If-None-Match` HTTP header. If matches current `catalog_version`, return HTTP `304 Not Modified` with empty body.

#### Change Feed API (`GET /v1/catalog/events`)
* Endpoint: `GET /v1/catalog/events?after={cursor}`
* Response:
```json
{
  "catalog_version": "sha256:9f8a...",
  "cursor": "evt_104",
  "events": [
    {
      "id": "evt_104",
      "timestamp": "2026-08-12T21:30:00Z",
      "object_type": "capability",
      "object_id": "github.issues.create",
      "change_type": "updated"
    }
  ]
}
```

---

### 2. Request Context & Correlation (P0)

#### Context Payload Schema
Add an optional `context` object and `request_id` to `CallCapabilityRequest`, `ReadResourceRequest`, and `GetPromptRequest`:

```json
{
  "id": "github.issues.search",
  "params": {"query": "is:open"},
  "request_id": "req-client-8819",
  "context": {
    "operation_id": "op-4412",
    "work_item_id": "task-990",
    "actor_id": "agent-user-12",
    "grant_id": "grant-771"
  }
}
```

#### Validation & Telemetry Mapping
* **Size Enforcement**: `serde_json::to_vec(&context).len() <= 4096` bytes. Exceeding returns HTTP 400 (`CONTEXT_TOO_LARGE`).
* **OpenTelemetry Attributes**: Context key-values automatically mapped to span attributes: `app.context.<key> = <value>`.
* **Redaction Integration**: Log outputs sanitize context fields against configured `policy.redact_keys`.
* **Upstream Isolation**: `context` is strictly captured for logging/tracing and returned in execution envelopes—it is **never** injected into upstream MCP tool parameters unless explicitly declared in parameter mappings.

---

### 3. Capability Search API & CLI (P0)

#### Search Endpoint
`POST /v1/capabilities/search`

#### Request Payload
```json
{
  "query": "triage production errors",
  "limit": 8,
  "server_ids": ["github", "observability"],
  "tags": ["read"],
  "modes": ["read"]
}
```

#### Response Payload
```json
{
  "version": "v1",
  "catalog_version": "sha256:9f8a...",
  "capabilities": [
    {
      "id": "observability.logs.search",
      "summary": "Search structured application logs.",
      "server": "observability",
      "tags": ["logs", "read"],
      "mode": "read",
      "score": 0.91,
      "match": ["semantic", "tag"]
    }
  ]
}
```

#### CLI Integration
```bash
warmplane search-capabilities "triage production errors" --server github --limit 5
```

---

### 4. Idempotency, Cancellation & Retry Metadata (P1)

#### Execution Envelope Extension
All execution responses (`/v1/tools/call`, `/v1/resources/read`, `/v1/prompts/get`) are enriched with execution state classification:

```json
{
  "ok": false,
  "request_id": "req-123",
  "trace_id": "trace-456",
  "data": null,
  "error": {
    "code": "UPSTREAM_TIMEOUT",
    "message": "Tool call timed out after 15000ms",
    "retryable": true
  },
  "retry": {
    "classification": "unsafe",
    "upstream_execution_state": "unknown"
  }
}
```

#### Operations Tracking & Idempotency Store
* In-memory bounded cache mapping `idempotency_key` -> `OperationRecord` for a configurable deduplication TTL (default 1 hour).
* If a duplicate `idempotency_key` is submitted while an operation is in progress, Warmplane waits on the existing execution result. If completed, returns cached result envelope.

#### Cancellation Endpoint
`POST /v1/operations/{request_id}/cancel`
* Signals the underlying `tokio::task` / MCP actor channel to abort the active upstream request.

---

### 5. Result Shaping & Data Handling (P1)

#### Per-Capability & Per-Request Result Controls
Extend `mcp_servers.json` configuration and call request payloads with shaping options:

```json
{
  "result_shaping": {
    "max_bytes": 16384,
    "jsonpath_include": ["$.items[*].id", "$.items[*].title"],
    "jsonpath_exclude": ["$.items[*].raw_payload"],
    "truncation_strategy": "json_array_truncate"
  }
}
```

#### Enriched Envelope Output
```json
{
  "ok": true,
  "data": {"items": [{"id": 1, "title": "Error log"}]},
  "result_meta": {
    "truncated": true,
    "original_bytes": 45120,
    "returned_bytes": 1240,
    "redacted_fields": ["user_email"]
  }
}
```

---

### 6. Upstream Health & Auth Signals (P1)

#### Health Status Endpoint
`GET /v1/health/upstreams`

```json
{
  "version": "v1",
  "catalog_version": "sha256:9f8a...",
  "upstreams": [
    {
      "server_id": "github",
      "transport": "streamable_http",
      "connection_state": "ready",
      "auth_state": "valid",
      "auth_type": "oauth2",
      "last_connected_at": "2026-08-12T20:00:00Z",
      "token_expires_at": "2026-08-12T23:00:00Z",
      "rolling_avg_latency_ms": 42.5,
      "recent_error_count": 0
    },
    {
      "server_id": "jira",
      "transport": "stdio",
      "connection_state": "authentication_required",
      "auth_state": "expired",
      "auth_type": "basic",
      "last_connected_at": "2026-08-12T18:00:00Z",
      "last_error": "401 Unauthorized"
    }
  ]
}
```

---

### 7. Temporary Policy Overlays (P2)

#### Request-Bound Narrowing Overlay
Call payloads may pass a `policy_overlay`:

```json
{
  "capability_id": "github.issues.create",
  "args": {"title": "Test"},
  "policy_overlay": {
    "grant_id": "grant-temp-901",
    "allow_capabilities": ["github.issues.create"],
    "expires_at": "2026-08-12T22:00:00Z"
  }
}
```

#### Evaluation Rules
1. Static Warmplane policy (`policy.allow` / `policy.deny`) is evaluated first. If static policy denies, request is blocked.
2. Overlay is evaluated second: capability MUST be explicitly present in `overlay.allow_capabilities`.
3. Overlay **cannot** override a static deny rule or allow a capability omitted from static allowlists.
4. Response envelope includes `policy_hash: "sha256:..."` for audit validation.

---

### 8. Lifecycle Events & Namespaces (P2)

#### Multi-Tenant Namespace Configuration
`mcp_servers.json` supports namespace partitioning:

```json
{
  "namespaces": {
    "staging": {
      "mcpServers": { ... },
      "policy": { ... }
    },
    "production": {
      "mcpServers": { ... },
      "policy": { ... }
    }
  }
}
```

Routes accept optional `X-Warmplane-Namespace` HTTP header or URL prefix `/v1/n/:namespace/capabilities`.

---

## Phased Delivery Roadmap

```text
Phase 1: Catalog Versioning, Caching & Request Context (P0)
   ├── Implement SHA256 catalog hashing & ETag middleware
   ├── Support HTTP conditional GETs (304 Not Modified)
   └── Add request context parsing, OTEL span integration & log redaction

Phase 2: Capability Search & Semantic Indexing (P0)
   ├── Implement BM25 / Fuzzy Lexical search engine
   ├── Implement optional FastEmbed / ONNX vector embedding engine
   ├── Implement RRF Hybrid Fusion ranker
   └── Add POST /v1/capabilities/search & CLI command

Phase 3: Operations Reliability, Idempotency & Result Shaping (P1)
   ├── Implement operation tracker, idempotency store & cancellation endpoint
   ├── Enrich execution envelopes with retry state classification
   └── Implement result shaping (byte capping, JSONPath filters & result metadata)

Phase 4: Health Monitoring, Policy Overlays & Namespaces (P1 / P2)
   ├── Implement GET /v1/health/upstreams & auth inspector
   ├── Implement request-bound policy overlay evaluator
   └── Implement lifecycle change feed & namespace partitioning
```

---

## Verification & Validation Plan

### Automated Test Suites
1. **Unit Tests**:
   * Catalog hashing determinism and ETag comparison.
   * Search rank accuracy (BM25 vs Vector vs RRF hybrid scoring).
   * Context size validation and redaction filtering.
   * Policy overlay narrowing invariants (verify overlay cannot bypass static deny).
   * JSONPath result shaping and byte truncation correctness.
2. **Integration Tests**:
   * Axum endpoint HTTP 304 behavior on matching `If-None-Match`.
   * Cancellation token propagation during mock long-running tool calls.
   * Idempotency replay verification under concurrent duplicate requests.

### Performance & Memory Benchmarks
* Benchmark search latency under catalog sizes of 100, 500, and 2,000 capabilities.
* Verify memory footprint with semantic search enabled (< 40MB total RSS).
