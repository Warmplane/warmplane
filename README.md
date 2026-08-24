# Warmplane
[![Latest Release](https://img.shields.io/github/v/release/Warmplane/warmplane)](https://github.com/Warmplane/warmplane/releases/latest) [![crates.io](https://img.shields.io/crates/v/warmplane.svg)](https://crates.io/crates/warmplane) [![docs.rs](https://docs.rs/warmplane/badge.svg)](https://docs.rs/warmplane) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/Warmplane/warmplane)
[![Rust Guideline Checks](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml/badge.svg?branch=main&job=compliance-check)](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml) [![Security Audit](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml/badge.svg?branch=main&job=security-audit)](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml)

**Security controls:** [![WORM + HMAC audit](https://img.shields.io/badge/WORM%20%2B%20HMAC-audit-4c1.svg)](docs/OBSERVABILITY.md) [![HITL + policy](https://img.shields.io/badge/HITL%20%2B%20policy-controls-4c1.svg)](docs/ENTERPRISE_FEATURES.md) [![OAuth2 + PKCE](https://img.shields.io/badge/OAuth2%20%2B%20PKCE-protected-4c1.svg)](docs/research/MCP_AUTHORIZATION.md) [![Secret redaction](https://img.shields.io/badge/secret-redaction-4c1.svg)](docs/ENTERPRISE_FEATURES.md) [![SIEM + OTLP](https://img.shields.io/badge/SIEM%20%2B%20OTLP-observable-4c1.svg)](docs/OBSERVABILITY.md)

> **The Local control plane that keeps MCP sessions warm with compact capability/resource/prompt facades.**
> 
> v0.23.0 — [Changelog](#changelog) · [User Guide](docs/USER-GUIDE.md) · [Performance](docs/PERFORMANCE.md) · [Whitepaper](docs/WHITEPAPER.md) · [OpenAPI](docs/openapi.yaml)


Warmplane runs multiple upstream MCP servers behind one local process, keeps those sessions persistent, and exposes a compact, policy-aware surface for tools, resources, and prompts — accessible via HTTP, CLI, and MCP-native clients.

---

## Quick Start

**1. Build**

```bash
cargo install --path .

# Optional: local ONNX vector embeddings (FastEmbed)
cargo install --path . --features semantic-search
```

**2. Configure Upstream Servers**

Manage servers interactively or import from existing tools:

```bash
# Interactive setup wizard
warmplane server add

# Or add non-interactively
warmplane server add filesystem --command npx --arg "-y" --arg "@modelcontextprotocol/server-filesystem" --arg "/tmp"
warmplane server add context7 --url "https://mcp.context7.ai/sse" --bearer-env "CONTEXT7_API_KEY"

# Or import directly from Claude Desktop / Cursor
warmplane config import
```

Or manually create `mcp_servers.json`:

```json
{
  "port": 9090,
  "toolTimeoutMs": 15000,
  "capabilityAliases": { "sqlite.read_query": "db.query" },
  "resourceAliases":  { "filesystem.file:///tmp/readme.txt": "fs.readme" },
  "promptAliases":    { "github.code_review": "prompt.code-review" },
  "policy": {
    "allow": ["db.*", "fs.*", "prompt.*"],
    "deny":  ["fs.secret"],
    "redactKeys": ["token", "api_key", "password"]
  },
  "profiles": {
    "coding": {
      "servers": ["filesystem", "sqlite"],
      "description": "Local coding and data inspection tools"
    }
  },
  "mcpServers": {
    "sqlite":     { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-sqlite", "./test.db"] },
    "filesystem": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"] }
  }
}
```

**3. Validate, then start**

```bash
warmplane server list
warmplane validate-config --config mcp_servers.json
warmplane daemon --config mcp_servers.json
```

---

## Run Modes

All three modes share the same backend state, aliases, policy checks, and timeout behaviour.

### HTTP Daemon

```bash
warmplane daemon --config mcp_servers.json
# Serves /v1/... on the configured port (default 9090)
```

Key endpoints:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/capabilities` | Compact capability index |
| `POST` | `/v1/capabilities/search` | Hybrid lexical + semantic search |
| `POST` | `/v1/tools/call` | Normalized execution envelope (supports idempotency keys, `_jsonpath`, `_limit_lines`, `_truncate_bytes`) |
| `POST` | `/v1/tools/batch_call` | Chained multi-step execution with `$step.field` reference interpolation |
| `GET` | `/v1/idempotency/records` | List cached idempotency execution records and replay counts |
| `GET` | `/v1/idempotency/records/:key` | Inspect single cached idempotency record |
| `GET` | `/v1/resources` | Resource index |
| `POST` | `/v1/resources/read` | Read resource |
| `GET` | `/v1/prompts` | Prompt index |
| `POST` | `/v1/prompts/get` | Render prompt |
| `GET` | `/v1/catalog/events` | Catalog change event feed |
| `POST` | `/v1/operations/:id/cancel` | Cancel an in-flight operation |

### MCP Server (stdio)

```bash
warmplane mcp-server --config mcp_servers.json
```

Point any MCP-native client at this process. It exposes lightweight facade tools (`capabilities_list`, `capability_search`, `capability_describe`, `capability_call`, `capabilities_batch_call`, `resource_read`, `prompt_get`, …) alongside native `resources/*` and `prompts/*` methods.

Claude Desktop / Cursor config:

```json
{
  "mcpServers": {
    "warmplane": {
      "command": "warmplane",
      "args": ["mcp-server", "--config", "mcp_servers.json"]
    }
  }
}
```

### MCP Server (HTTP/SSE)

Exposes the same facade over Streamable HTTP/SSE so remote clients — CI pipelines, multi-host agent clusters, or remote desktop clients — can connect without a local process:

```bash
# Local-only (default, no auth required)
warmplane mcp-http-server --config mcp_servers.json

# Network-accessible (requires authToken in config)
warmplane mcp-http-server --config mcp_servers.json --bind 0.0.0.0 --port 9191

# Profile-restricted
warmplane mcp-http-server --config mcp_servers.json --profile coding
```

Connect from any MCP HTTP client:

```json
{
  "mcpServers": {
    "warmplane": {
      "url": "http://localhost:9191/mcp"
    }
  }
}
```

Alternatively, add an `mcpHttpServer` block to `mcp_servers.json` and the **daemon will co-host both servers** in one process:

```json
{
  "mcpHttpServer": { "port": 9191 },
  "mcpServers": { ... }
}
```

See [§4.7 of the User Guide](docs/USER-GUIDE.md#47-mcp-httparse-server-configuration-mcphttpserver) for the full config reference.

### CLI Configuration & Operations

```bash
# Interactive server setup wizard
warmplane server add

# Import settings from Claude Desktop or Cursor
warmplane config import

# Hot-reload in-memory workers from mcp_servers.json without restarting
warmplane reload

# Inspect & test servers
warmplane server list
warmplane server test github

# Ecosystem config import (Claude Desktop, Cursor, Zed)
warmplane config import

# Aliases and Policies
warmplane config alias set tool git-commit github.create_commit
warmplane config policy allow "github.*" "fetch.*"

# Capability and Execution CLI (supports --profile <name>)
warmplane list-capabilities --profile coding
warmplane search-capabilities "triage logs" --limit 5 --profile coding
warmplane describe-capability db.query --profile coding

warmplane call-capability db.query \
  --params '{"query":"SELECT 1"}' \
  --request-id req-101 --actor-id user-7 \
  --idempotency-key op-20-run-1 \
  --profile coding

warmplane read-resource fs.readme --profile coding
warmplane get-prompt prompt.code-review --arguments '{"code":"fn main() {}"}' --profile coding

warmplane list-catalog-events --after evt_3
warmplane cancel-operation req-101
```

---

## Performance Highlights

Warmplane is engineered with pure Rust zero-cost abstractions, keeping agent loops snappy:

- **50.4 ns** ETag Cache Validation (`If-None-Match` $\rightarrow$ `304 Not Modified`)
- **159.8 ns** Idempotent Cache-Hit Deduplication
- **1.58 µs** SHA-256 Incremental Catalog Version Hashing ($N=10$)
- **15.9 µs** Filtered Hybrid Capability Search ($N=50$)
- **372.1 µs** In-Memory Zero-Allocation Lexical Tag Search across 1,000 Tools

👉 See the complete benchmarks and profiling methodology in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

---

## Feature Overview

| Feature | Since | Summary |
|---------|-------|---------|
| **Exactly-Once Idempotency & Replay Ledger** | v0.23.0 | Deterministic auto-key derivation (`idk_<sha256>`), `X-Warmplane-Deduplicated: true` header caching, replay count tracking, WORM audit trail linking (`idempotency_key`, `is_replay`), and `/v1/idempotency/records` inspection APIs |
| **Embedded Rust Library Engine** | v0.23.0 | Direct in-process library interface (`EmbeddedWarmplane`, `ControlPlaneHandle`) with strongly typed response envelopes (`Envelope<T>`), direct tool/resource/prompt execution, and graceful cancellation on caller's Tokio runtime |
| **MCP HTTP/SSE Server Mode** | v0.22.0 | Streamable HTTP/SSE MCP server (`mcp-http-server`) for remote network clients; daemon co-hosting via `mcpHttpServer` config block; profile restriction, auth enforcement on non-loopback bind, graceful shared-state shutdown |
| **Signal Handling & Graceful Teardown** | v0.21.0 | Immediate signal cancellation (`CancellationToken`), instant SSE stream termination, clean stdio child orphan protection (`kill_on_drop`), and bounded drain safety timeouts |
| **Named Server Constellations (Profiles)** | v0.21.0 | Task-relevant server constellation slicing (`profiles`), dynamic per-request selection (`X-Warmplane-Profile` / `?profile=`), profile-partitioned ETag caching (`-p:<profile_id>`), and stdio MCP proxy filtering (`--profile`) |
| **Multi-Tenant RBAC & Catalog Partitioning** | v0.20.0 | Tenant-scoped authorization (`TenantContext`), static token & HMAC-SHA256 JWT auth, per-role policy overrides, dynamic catalog/search pruning, and WORM audit identity binding |
| **Client-Delegated MCP Sampling** | v0.19.0 | Reverse RPC sampling delegation (`sampling/createMessage`), synchronous long-polling or async ticket lifecycle, persistent `sampling.json` storage, and self-healing HTTP/SSE supervisor |
| **Persistent State & Graceful Teardown** | v0.18.0 | Restart-resilient atomic storage (`approvals`, `idempotency`, `oauth`, `catalog_events`), SIGINT/SIGTERM handling, audit batch drain, Bun CI UI gate & E2E suite |
| **360° MCP Explorer & Execution Controls** | v0.17.0 | Resources explorer & live reader, prompt template studio, in-flight cancellation, visual batch pipeline builder, WORM multi-field search & pagination |
| **Security Hardening & WORM Integrity** | v0.16.0 | API token gate, OAuth proxy guard, HMAC audit verification, sliding-window backoff, resource caps |
| **Fault Tolerance & Supervision** | v0.15.0 | Supervisor restart loop, configurable circuit breakers, and degraded boot |
| **Agent Enrichment Suite** | v0.14.0 | Facade search, context distillation (`_jsonpath`/`_limit_lines`), and multi-step batch execution |
| **HITL Approval Engine** | v0.13.0 | Operator gate approval engine, suspension, argument editing, and HMAC webhook dispatch |
| **WORM Audit & SIEM** | v0.12.0 | Append-only SHA-256 hash-chained audit logging, verification API, and Splunk/Webhook SIEM export |
| **Control Deck Web UI** | v0.11.0 | Standalone embedded web dashboard for servers, testing playground, policy & telemetry |
| **Dynamic Hot-Reloading** | v0.11.0 | Zero-downtime upstream mounting/unmounting, explicit `warmplane reload` & `/v1/config/reload` |
| **CLI Config & Interactive Setup** | v0.10.0 | `warmplane server` & `warmplane config` wizards, atomic JSON writes |
| **Ecosystem Import** | v0.10.0 | 1-click import from Claude Desktop, Cursor, and Zed settings |
| **MCP 2026-07-28 & MRTR**| v0.9 | Full spec compliance, `rmcp 3.1.2`, cache hints, Multi Round-Trip Requests |
| **Subscriptions Feed** | v0.9 | `subscriptions_listen` tool & `/v1/resources/updates` SSE feed |
| **Semantic Vector Search**| v0.8.0 | FastEmbed ONNX embedding pipeline with cosine ranking |
| **Pragmatic Rust Architecture** | v0.7.0 | Builder patterns (`M-INIT-BUILDER`), robust panic safety, structured logging |
| **Idempotency & Retry** | v0.6.0 | `Idempotency-Key` deduplication and structured `"retry"` metadata envelopes |
| **Cancellation** | v0.6.0 | `POST /v1/operations/:id/cancel` / `cancel-operation` CLI |
| **Request context** | v0.5.0 | `operation_id`, `actor_id`, `grant_id` in envelopes + HTTP header fallback |
| **Catalog versioning** | v0.4.0 | SHA-256 `ETag`, `If-None-Match` → `304`, change event feed |
| **Hybrid search** | v0.3.0 | BM25 lexical + vector search with filters |
| **Alias registry** | v0.1.0 | Short stable aliases over upstream capability IDs |
| **Compact indexes** | v0.1.0 | Lazy, token-efficient catalog — detail only on demand |
| **Policy profiles** | v0.1.0 | Allow/deny lists, redact keys, role-scoped exposure |
| **Normalized envelopes** | v0.1.0 | Consistent result, timeout, and error format across all modes |
| **OTLP traces** | v0.1.0 | OpenTelemetry export, `trace_id` reflected in envelopes |

---

## Changelog

### v0.23.0 — Embedded Rust Engine, In-Process Control Plane & Facade Adapter Refactoring
- **In-Process Embedded Rust Library Engine (`EmbeddedWarmplane`, `ControlPlaneHandle`):** Exposed Warmplane as a pure in-process library without HTTP, daemon child process, or JSON-RPC serialization overhead. Callers spawn the engine directly on their own Tokio runtime via `EmbeddedWarmplane::start(config)` or `EmbeddedWarmplane::start_from_path(path)` and interact via typed `ControlPlaneHandle` methods (`list_capabilities`, `describe_capability`, `search_capabilities`, `call_capability`, `batch_call`, `read_resource`, `get_prompt`, `health_status`).
- **Strongly Typed Generic Envelopes & Error Models:** Added `Envelope<T>`, `WarmplaneError`, `CapabilitySummary`, `CapabilityDetail`, `ExecutionOptions`, `ReadResourceOptions`, `GetPromptOptions`, and `EngineHealthStatus` in `warmplane::engine::types`, preserving structured diagnostics (`request_id`, `trace_id`, `retry`, `operator`) for programmatic orchestration.
- **MCP Stdio Server Facade Adapter Refactoring:** Refactored `mcp_server.rs` to delegate all tool call execution, search, descriptions, batch execution, and resource/prompt dispatch directly to `ControlPlaneHandle`, eliminating duplicated logic and unifying execution pipelines.
- **Embedded Engine Integration Test Suite:** Added dedicated integration tests in `tests/embedded_tests.rs` covering embedded lifecycle startup, degraded server handling, health status inspection, and graceful cancellation.


### v0.22.0 — Streamable HTTP/SSE MCP Transport, Interactive Playground & UI Polish
- **Streamable HTTP/SSE MCP Server Transport (`mcp-http-server`):** Built standalone and daemon-co-hosted HTTP/SSE MCP server endpoints (`/mcp/sse`, `/mcp/messages`), allowing remote AI agents and IDEs (Cursor, Windsurf, Claude Desktop) to connect over standard HTTP/SSE networks with automatic keep-alives and zero client drift.
- **Daemon Co-hosting & Configuration (`mcpHttpServer`):** Added first-class configuration support (`mcpHttpServer`) enabling automatic background initialization of HTTP/SSE MCP server instances directly alongside the core `/v1` HTTP daemon.
- **Profile Restriction & Bound Auth Gate:** Integrated profile restriction (`profile`) on the MCP server transport, strictly isolating tools and resources exposed to remote clients. Automatically enforced bearer token authentication when binding to public non-loopback network interfaces (`0.0.0.0` / external IPs).
- **Interactive MCP Playground Ergonomics:** Added sample template injection, dynamic format switching, schema-driven argument generation, and live parameter validation in the Web Control Deck MCP Playground.
- **UI Transitions, Collision Checks & Polish:** Deduplicated page headers across dashboard tabs, introduced buttery-smooth CSS cubic-bezier transitions, animated modal backdrops with `prefers-reduced-motion` accessibility support, and added collision guards with automatic unique server ID derivation.

### v0.21.0 — Named Server Constellations (Profiles), Signal Lifecycle & Integrator Ecosystem
- **Named Server Constellations (Profiles):** Added first-class profile support (`ProfileConfig`) allowing task-relevant subsets of upstream MCP servers to be grouped into named constellations (e.g. `coding`, `research`, `data_science`).
- **Dynamic Profile Selection & Scoped ETag Partitioning:** Supported per-request profile scoping via `X-Warmplane-Profile` headers and `?profile=` query parameters. Catalog endpoints (`/v1/capabilities`, `/v1/resources`, `/v1/prompts`) and search (`/v1/capabilities/search`) dynamically prune items outside the active profile and maintain deterministic profile-partitioned ETags (`sha256:...-p:<profile_id>`).
- **MCP Facade Stdio Profile Filtering:** Added `--profile <name>` CLI option to `warmplane mcp-server` and `warmplane list-capabilities`, providing agent hosts with a strictly scoped MCP tool and resource surface.
- **Web Control Deck Profile Hub:** Added interactive visual profile manager in the Web UI dashboard with 1-click active constellation switching, server toggles, and live catalog filtering.
- **Signal Handling & Process Teardown Hardening:** Integrated immediate `tokio_util::sync::CancellationToken` dispatch on `SIGINT` (Ctrl-C) / `SIGTERM`, unblocked SSE stream draining, added child process orphan prevention via `kill_on_drop(true)`, and enforced a 3-second bounded safety shutdown timeout.
- **Developer & Integrator Ecosystem Guides:** Published official [Rust Integrators Guide](docs/RUST_INTEGRATORS_GUIDE.md), [TypeScript Integrators Guide](docs/TYPESCRIPT_INTEGRATORS_GUIDE.md), and [Idempotency Architecture Editorial](docs/EDITORIAL.md).

### v0.20.0 — Multi-Tenant RBAC & Deterministic Catalog Partitioning
- **Multi-Tenant RBAC Engine (`src/rbac`):** Built role-based access control engine with support for static API tokens, token-to-role mappings in configuration (`rbac.tokens`), and cryptographic HMAC-SHA256 symmetric JWT signature verification with configurable secret key (`rbac.jwt_secret`).
- **Deterministic Catalog Partitioning & Scope Pruning:** Restructured `/v1/capabilities`, `/v1/resources`, `/v1/prompts`, and `/v1/capabilities/search` to dynamically filter items by caller role and effective policy. Unauthorized items are completely invisible in catalog listings and search queries.
- **Tenant Context Injection & Non-Repudiation Audit:** Injected resolved `TenantContext` (`tenant_id`, `role`, `actor_id`, `grant_id`, `effective_policy`) into request extensions via RBAC guard middleware (`src/rbac/middleware.rs`). Bound verified tenant/actor metadata automatically into WORM audit events.
- **Role Policy Overrides & HITL Delegation:** Supported fine-grained role definitions (`RolePolicyConfig`) with custom `allow`, `deny`, `require_approval`, and `redact_keys` rules, overriding or intersecting with base system policy.
- **Integration Test Suite:** Added dedicated RBAC integration tests (`tests/rbac_integration.rs`) covering token authentication, search filtering, catalog isolation, policy boundary enforcement, and multi-tenant audit verification.

### v0.19.0 — Client-Delegated MCP Sampling, HTTP/SSE Supervisor Loop & Hardening
- **Client-Delegated MCP Sampling (`sampling/createMessage`):** Implemented client-delegated LLM completion reverse RPC handling (`src/sampling.rs`). Upstream servers or agents submit sampling requests, generating tracked tickets (`samp_<timestamp>_<seq>`) with synchronous long-polling or asynchronous lifecycle endpoints (`POST /v1/sampling/create_message`, `GET /v1/sampling/requests`, `GET /v1/sampling/requests/:id`, `POST /v1/sampling/requests/:id/respond`).
- **Persistent Sampling State:** Added atomic, restart-resilient disk storage (`sampling.json`) via `AtomicFile<HashMap<String, PendingSamplingRequest>>` with automated expiration reaper tasks.
- **Self-Healing Streamable HTTP/SSE Supervisor:** Integrated remote HTTP/SSE MCP servers into the supervisor loop with automated reconnection backoff, catalog reconciliation, and degraded boot status reporting.
- **Security & DoS Hardening:** Capped candidate capability embeddings for semantic vector search to `MAX_VECTOR_SEARCH_CANDIDATES = 250`; bounded audit export queries to `MAX_IN_MEMORY_AUDIT_EVENTS` (20,000); enforced Host header and loopback Origin checks on OAuth proxy requests.
- **CI Gating & TypeScript Typechecking:** Added automated TypeScript typechecking (`tsc --noEmit`) to Web UI CI pipeline and restricted push triggers to `main` to eliminate duplicate runs.
- **Pragmatic Rust Compliance:** Achieved 100% adherence across all 51 source files with standard compliance headers (`// Rust guideline compliant YYYY-MM-DD`).

### v0.18.0 — Persistent State Subsystem, Graceful Teardown, CI UI Automation & E2E Test Suite
- **Persistent State Subsystem:** Added atomic, restart-resilient disk storage (`AtomicFile<T>` and `StateDirectory`) for Human-in-the-Loop pending approvals (`approvals.json`), idempotent execution records (`idempotency.json`), OAuth2 tokens (`oauth_tokens.json`), and catalog mutation events (`catalog_events.json`). Added `state` block in `McpConfig` and `warmplane config state show/set` CLI commands.
- **Graceful Signal Handling & Subsystem Teardown:** Added robust `SIGINT` (Ctrl+C) and `SIGTERM` signal capture on Unix and Windows, integrated graceful HTTP server draining, async audit worker flushes (`AuditWorkerMsg::FlushAndShutdown`), and clean stdio subprocess process termination on shutdown.
- **Pragmatic Rust Compliance:** Achieved 100% adherence across all 51 source files with standard compliance headers (`// Rust guideline compliant YYYY-MM-DD`).
- **CI Automated Web UI Build & Drift Gate:** Integrated Bun into GitHub Actions CI (`ci.yml`) and release pipelines (`release-artifacts.yml`), with automated build steps and strict `git diff --exit-code ui/dist/index.html` drift detection.
- **Comprehensive End-to-End Integration Suite:** Implemented dedicated E2E test harness (`tests/e2e_tests.rs`) exercising stdio MCP protocol handshakes, live TCP SSE streaming, config hot-reloading, mock OAuth2 RFC 8414 provider round-trips with silent 401 token refresh, and supervisor recovery.

### v0.17.0 — 360° MCP Explorer, Visual Batch Pipeline Builder & WORM Audit Pagination
- **WORM Audit Multi-Field Search & Pagination:** Added case-insensitive substring search across 11 metadata fields, outcome status and server filters, offset/limit pagination slicing (`/v1/audit/events`), and context-aware CSV and JSONL export downloads (`/v1/audit/export`).
- **MCP Resources Explorer & Live Content Reader:** Added 360° resource discovery browser with protocol scheme badges (`file://`, `postgres://`, `github://`, `memory://`, `sqlite://`, etc.), metadata/MIME viewer, and live content reader supporting context distillation (`_jsonpath`, `_limit_lines`, `_truncate_bytes`).
- **MCP Prompt Template Studio:** Added dynamic prompt template browsing and argument form generation with `REQUIRED` validation badges, rendering resolved system/user prompt envelopes (`/v1/prompts/get`).
- **In-Flight Operation Cancellation:** Added interactive UI execution cancellation controls with live execution timers and instant cooperative abort (`POST /v1/operations/:id/cancel`).
- **Visual Multi-Step Batch Pipeline Builder:** Added interactive modal pipeline editor for chaining tools with parameter variable interpolation (`${steps[0].result.id}`) and per-node fault tolerance (`POST /v1/tools/batch_call`).
- **Realtime SSE State Synchronization:** Connected `/v1/resources/updates` SSE stream to automatically refresh resources, prompts, and catalog feeds on the Control Deck.

### v0.16.0 — Enterprise Security Hardening, Audit Cryptographic Integrity & Lifecycle Resilience
- **API Token Authentication & Middleware Guarding:** Added `--auth-token` CLI parameter, `McpConfig.auth_token` configuration field, and `WARMPLANE_AUTH_TOKEN` environment variable support to securely protect daemon endpoints with Bearer/X-Warmplane-Key authentication.
- **OAuth Proxy Security & Secret Masking:** Hardened the OAuth proxy listener with Host validation and cross-origin browser blocking; masked `state` and PKCE `code_challenge` secrets in log output.
- **Cryptographic WORM Audit Integrity & HMAC Signing:** Included all 20 persisted metadata fields in hash chain digests (`work_item_id`, `client_ip`, `resource_uri`, `execution_latency_us`, `error_message`); added HMAC-SHA256 keyed digest calculations and checkpoint generation for external cryptographic anchoring.
- **Secret Sanitization & Redaction:** Enhanced case-insensitive redaction matching with built-in default sensitive key list (`token`, `secret`, `password`, `key`, `authorization`, etc.); restricted external HITL webhook events to only transmit `sanitized_args`; sanitized secrets before printing in CLI commands.
- **Supervisor & Circuit Breaker Coordination:** Automatically reset circuit breakers on successful supervisor reconnection; added single-flight probe limits in `HalfOpen` circuit breaker state; implemented 60-second sliding-window restart backoff; pruned removed items during catalog reconciliation; cleaned up circuit breakers on server unmount.
- **Resource Caps & Safety Controls:** Enforced `MAX_BATCH_STEPS = 50` and `DEFAULT_BATCH_TIMEOUT_MS = 60_000` execution budget; capped wildcard JSONPath output expansion to 10,000 items and search results to 100; returned `POLICY_DENIED` (HTTP 403) and 404 for unknown operation cancellation; sanitized CSV audit exports against formula injection.
- **CI Quality Gates & Dependency Audits:** Integrated native `cargo-audit` scanning in GitHub Actions CI; updated dependencies resolving all reported advisories.
- **CLI Version Flag:** Enabled standard `--version` and `-V` flags in the `warmplane` binary parser.

### v0.15.0 — Fault Tolerance, Boot Resilience & Control Deck Feature Parity
- **Boot Resilience & Degraded Startup:** Graceful daemon boot when upstream servers (such as Docker, remote SSE endpoints, or unconfigured tools) fail or timeout. Failed servers are flagged as `degraded` without crashing the daemon or blocking other healthy upstreams.
- **Process Supervisor & Circuit Breakers:** Per-server and global circuit breakers (`failureThreshold`, `cooldownMs`, `autoRestart`, `maxRestarts`) with state tracking (`CLOSED`, `OPEN`, `HALF-OPEN`) and exponential backoff restart supervision.
- **Full Web UI Feature Parity:** 
  - Server Hub: Server card edit workflow (`✏️ Edit`), live circuit breaker telemetry badges, and server resilience indicators.
  - Template Wizard: Fault tolerance and supervisor configuration accordion directly inside 1-click curated server setup.
  - Interactive Playground: Context distillation controls (`_jsonpath`, `_limit_lines`, `_truncate_bytes`) and execution latency measurements.
  - Responsive Bento-Grid overview cards with real-time health indicator dots (`connected` 🟢, `degraded` 🟡, `error` 🔴).
- **CLI Resilience Configuration:** `warmplane config resilience set` and `warmplane config resilience show` for headless circuit breaker management.

### v0.14.0 — Agent Enrichment Suite: Facade Search, Context Distillation & Multi-Step Batching
- **MCP Facade Hybrid Search:** Exposed `capability_search` tool over MCP stdio facade with keyword, tag, server ID, and execution mode filters.
- **Context Distillation & Truncation:** Added `_jsonpath`, `_limit_lines`, and `_truncate_bytes` modifiers to `/v1/tools/call` and `capability_call` to protect LLM context windows from oversized outputs.
- **Multi-Step Chained Batch Calls:** Added `POST /v1/tools/batch_call` and `capabilities_batch_call` for single-roundtrip dependent tool executions with `$step.field` reference interpolation.

### v0.13.0 — Human-in-the-Loop (HITL) Approval Engine & Signed Webhooks
- **Approval Interceptor:** Policy-driven suspension for sensitive capability calls (`requireApproval`).
- **Operator REST API:** Endpoints to list (`/v1/approvals`), inspect, approve with modified arguments (`/v1/approvals/:id/approve`), or reject (`/v1/approvals/:id/reject`).
- **Signed HMAC Webhooks:** Real-time webhook dispatch with SHA-256 HMAC signature headers on approval lifecycle events.
- **Configurable TTL Expiration:** Automatic timeout expiration (`approvalTimeoutSecs`) returning structured cancellation envelopes.

### v0.12.0 — Cryptographic WORM Audit Log & SIEM Streaming
- Append-only linear SHA-256 hash chaining over tool execution and HITL decision events.
- Audit verification and export endpoints (`/v1/audit/...`), Splunk HEC and generic Webhook ingestion.

### v0.11.0 — Control Deck Web UI & Dynamic Upstream Hot-Reloading
- **Control Deck Web UI:** Embedded zero-dependency web dashboard at `/ui` and `/` with live telemetry, server manager, interactive tool playground, policy/redaction manager, and alias registry.
- **Zero-Downtime Dynamic Upstream Mounting:** Dynamically mount and unmount stdio, HTTP, and OAuth2 upstream workers via REST/UI/CLI without restarting the daemon process.
- **Dynamic Catalogs & ETags:** Concurrency-safe state management with automatic SHA256 ETag recomputation and SSE resource notifications on server changes.
- **Explicit Config Hot-Reload:** Added `warmplane reload` CLI command and `POST /v1/config/reload` endpoint to cleanly reconcile in-memory workers against manual edits to `mcp_servers.json`.

### v0.10.0 — CLI Configuration Management & Ecosystem Importers
- Added `warmplane server` (`add`, `remove`, `list`, `get`, `test`) commands with interactive `inquire` wizards and flag-driven headless automation.
- Added `warmplane config` (`init`, `show`, `import`, `alias`, `policy`) for safe, transactional configuration mutations.
- Added auto-discovery and import from Claude Desktop, Cursor, and Zed settings.
- Added atomic configuration file writes (`fs::rename`) preventing JSON corruption.

### v0.9.0 — MCP 2026-07-28 Spec Compliance, MRTR & Subscriptions
- Upgraded to official MCP Rust SDK `rmcp 3.1.2` and `reqwest 0.13`.
- Set default protocol version to `"2026-07-28"` with backward compatibility for `"2025-11-25"`.
- Added Multi Round-Trip Requests (MRTR) support (`input_responses` and `request_state`) across HTTP REST, stdio facade, and upstream workers.
- Added cache hints (`ttl_ms: 300000`, `cache_scope: "public"`) and deterministic alphabetical sorting on catalog listings.
- Added `subscriptions_listen` tool to stdio facade and `/v1/resources/updates` SSE stream for real-time notifications.
- Validated RFC 9207 / SEP-2468 `iss` callback verification for OAuth 2.0 flows.

### v0.8.0 — Semantic Vector Embeddings & FastEmbed ONNX Pipeline
- Integrated FastEmbed ONNX embedding pipeline with dense cosine vector similarity under optional `--features semantic-search`.
- Hybrid reciprocal rank fusion combining BM25 keyword matching and dense vector search.

### v0.7.0 — Pragmatic Rust Modernization & Builder Patterns
Full adoption of Microsoft's Pragmatic Rust Guidelines (`AGENTS.md`). Implemented `Builder Pattern` (`M-INIT-BUILDER`) for core state (`AppStateBuilder`), search filters (`SearchFilterBuilder`), and request context (`RequestContextBuilder`). Enhanced error safety (`M-PANIC-IS-STOP`), structured logging (`M-LOG-STRUCTURED`), canonical documentation (`M-CANONICAL-DOCS`), and flexible trait interop (`M-IMPL-ASREF`).

### v0.6.0 — Idempotency, Cancellation & Retry Metadata
Pass `Idempotency-Key` / `X-Idempotency-Key` to deduplicate concurrent tool calls. Abort any in-flight request via cancel endpoint or CLI. Every response envelope now includes structured `"retry"` metadata (`classification` + `state`) for orchestrator-aware retry logic.

### v0.5.0 — Request Context & Correlation
Structured `RequestContext` (`operation_id`, `work_item_id`, `actor_id`, `grant_id`) threaded through all execution envelopes and tracing spans. HTTP header fallback (`X-Request-ID`, `X-Operation-ID`, `X-Actor-ID`, `X-Grant-ID`).

### v0.4.0 — Catalog Versioning & Cache Validation
SHA-256 catalog version. `ETag` headers on all catalog reads, `If-None-Match` conditional requests returning `304 Not Modified`. `GET /v1/catalog/events` change feed with cursor-based pagination.

### v0.3.0 — Hybrid Search
`POST /v1/capabilities/search` with BM25 scoring, optional FastEmbed vector embeddings, tag/server-ID filters, and ranked results.
`POST /v1/capabilities/search` with BM25 scoring, optional FastEmbed vector embeddings, tag/server-ID filters, and ranked results.

---

## Docs

| Document | Description |
|----------|-------------|
| [docs/USER-GUIDE.md](docs/USER-GUIDE.md) | Complete usage guide: config, modes, all CLI commands, auth, policy |
| [docs/PERFORMANCE.md](docs/PERFORMANCE.md) | Benchmark methodology, latency percentiles, and profiling results |
| [docs/openapi.yaml](docs/openapi.yaml) | OpenAPI 3.1 spec |
| [docs/config.schema.json](docs/config.schema.json) | JSON Schema for `mcp_servers.json` |
| [docs/OBSERVABILITY.md](docs/OBSERVABILITY.md) | Structured logs, OTLP config, trace correlation |
| [docs/INSTALL.md](docs/INSTALL.md) | Build variants, distribution notes |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment runbook |
