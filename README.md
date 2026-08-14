# Warmplane

> **The local control plane that keeps MCP sessions warm.**  
> v0.11.0 — [Changelog](#changelog) · [User Guide](docs/USER-GUIDE.md) · [Performance](docs/PERFORMANCE.md) · [Whitepaper](docs/WHITEPAPER.md) · [OpenAPI](docs/openapi.yaml) · [Spec](docs/spec.md)

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
| `GET` | `/v1/capabilities/:id` | On-demand capability detail |
| `POST` | `/v1/tools/call` | Normalized execution envelope |
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

Point any MCP-native client at this process. It exposes lightweight facade tools (`capabilities_list`, `capability_call`, `resource_read`, `prompt_get`, …) alongside native `resources/*` and `prompts/*` methods.

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

### CLI Configuration & Operations

```bash
# Interactive server setup wizard
warmplane server add

# Add stdio or HTTP upstream servers non-interactively
warmplane server add github --command npx --arg "-y" --arg "@modelcontextprotocol/server-github"
warmplane server add context7 --url "https://mcp.context7.ai/sse" --bearer-env "CONTEXT7_API_KEY"

# Inspect & test servers
warmplane server list
warmplane server test github

# Ecosystem config import (Claude Desktop, Cursor, Zed)
warmplane config import

# Aliases and Policies
warmplane config alias set tool git-commit github.create_commit
warmplane config policy allow "github.*" "fetch.*"

# Capability and Execution CLI
warmplane list-capabilities
warmplane search-capabilities "triage logs" --limit 5
warmplane describe-capability db.query

warmplane call-capability db.query \
  --params '{"query":"SELECT 1"}' \
  --request-id req-101 --actor-id user-7 \
  --idempotency-key op-20-run-1

warmplane read-resource fs.readme
warmplane get-prompt prompt.code-review --arguments '{"code":"fn main() {}"}'

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
- **371 µs** In-Memory Zero-Allocation Lexical Tag Search across 1,000 Tools

👉 See the complete benchmarks and profiling methodology in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

---

## Feature Overview

| Feature | Since | Summary |
|---------|-------|---------|
| **Control Deck Web UI** | v0.11.0 | Standalone embedded web dashboard for servers, testing playground, policy & telemetry |
| **Dynamic Hot-Reloading** | v0.11.0 | Zero-downtime upstream mounting/unmounting, explicit `warmplane reload` & `/v1/config/reload` |
| **CLI Config & Interactive Setup** | v0.10.0 | `warmplane server` & `warmplane config` wizards, atomic JSON writes |
| **Ecosystem Import** | v0.10.0 | 1-click import from Claude Desktop, Cursor, and Zed settings |
| **Alias registry** | v0.1 | Short stable aliases over upstream capability IDs |
| **Compact indexes** | v0.1 | Lazy, token-efficient catalog — detail only on demand |
| **Policy profiles** | v0.1 | Allow/deny lists, redact keys, role-scoped exposure |
| **Normalized envelopes** | v0.1 | Consistent result, timeout, and error format across all modes |
| **Hybrid search** | v0.3 | BM25 lexical + optional ONNX vector search with filters |
| **Catalog versioning** | v0.4 | SHA-256 `ETag`, `If-None-Match` → `304`, change event feed |
| **Request context** | v0.5 | `operation_id`, `actor_id`, `grant_id` in envelopes + HTTP header fallback |
| **Idempotency** | v0.6 | `Idempotency-Key` deduplication — concurrent duplicates share one result |
| **Cancellation** | v0.6 | `POST /v1/operations/:id/cancel` / `cancel-operation` CLI |
| **Retry metadata** | v0.6 | `"retry": { "classification": "safe\|unsafe\|idempotent", "state": "…" }` |
| **MCP 2026-07-28 & MRTR**| v0.9 | Full spec compliance, `rmcp 3.1.2`, cache hints, Multi Round-Trip Requests |
| **Subscriptions Feed** | v0.9 | `subscriptions_listen` tool & `/v1/resources/updates` SSE feed |
| **OTLP traces** | v0.1 | OpenTelemetry export, `trace_id` reflected in envelopes |

---

## Changelog

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

---

## Docs

| Document | Description |
|----------|-------------|
| [docs/USER-GUIDE.md](docs/USER-GUIDE.md) | Complete usage guide: config, modes, all CLI commands, auth, policy |
| [docs/PERFORMANCE.md](docs/PERFORMANCE.md) | Benchmark methodology, latency percentiles, and profiling results |
| [docs/spec.md](docs/spec.md) | HTTP request/response contracts |
| [docs/openapi.yaml](docs/openapi.yaml) | OpenAPI 3.1 spec |
| [docs/config.schema.json](docs/config.schema.json) | JSON Schema for `mcp_servers.json` |
| [docs/OBSERVABILITY.md](docs/OBSERVABILITY.md) | Structured logs, OTLP config, trace correlation |
| [docs/INSTALL.md](docs/INSTALL.md) | Build variants, distribution notes |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment runbook |
