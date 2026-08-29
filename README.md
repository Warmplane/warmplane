# Warmplane
[![Latest Release](https://img.shields.io/github/v/release/Warmplane/warmplane)](https://github.com/Warmplane/warmplane/releases/latest) [![crates.io](https://img.shields.io/crates/v/warmplane.svg)](https://crates.io/crates/warmplane) [![docs.rs](https://docs.rs/warmplane/badge.svg)](https://docs.rs/warmplane) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/Warmplane/warmplane)
[![Rust Guideline Checks](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml/badge.svg?branch=main&job=compliance-check)](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml) [![Security Audit](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml/badge.svg?branch=main&job=security-audit)](https://github.com/Warmplane/warmplane/actions/workflows/ci.yml)

**Security controls:** [![WORM + HMAC audit](https://img.shields.io/badge/WORM%20%2B%20HMAC-audit-4c1.svg)](docs/OBSERVABILITY.md) [![HITL + policy](https://img.shields.io/badge/HITL%20%2B%20policy-controls-4c1.svg)](docs/ENTERPRISE_FEATURES.md) [![OAuth2 + PKCE](https://img.shields.io/badge/OAuth2%20%2B%20PKCE-protected-4c1.svg)](docs/research/MCP_AUTHORIZATION.md) [![Secret redaction](https://img.shields.io/badge/secret-redaction-4c1.svg)](docs/ENTERPRISE_FEATURES.md) [![SIEM + OTLP](https://img.shields.io/badge/SIEM%20%2B%20OTLP-observable-4c1.svg)](docs/OBSERVABILITY.md)

> **The local control plane that keeps Model Context Protocol (MCP) sessions warm with compact capability facades, policy governance, and deterministic execution.**
> 
> v0.27.0 — [Changelog](#changelog) · [User Guide](docs/USER-GUIDE.md) · [Agent Skill](.skills/warmplane/SKILL.md) · [Performance](docs/PERFORMANCE.md) · [Whitepaper](docs/WHITEPAPER.md) · [OpenAPI](docs/openapi.yaml)

---

## ⚡ What is Warmplane?

Warmplane is a local control plane and reverse proxy for AI tool calling. It maintains persistent, warm connections to multiple upstream MCP servers and multiplexes them behind a single, governed interface.

### The Problems Warmplane Solves

1. **Context Window Token Bloat**: Sending massive JSON schemas for dozens of tools on every turn wastes tens of thousands of prompt tokens. Warmplane provides a compact catalog index (cutting payload size by 58–96%), on-demand schema discovery, and SHA-256 ETag caching.
2. **Duplicate Invocations & Retries**: When network hiccups occur, naive agents retry blind mutations. Warmplane provides crash-resilient, exactly-once idempotency deduplication (`idk_<sha256>`) and explicit retry classifications (`safe`, `idempotent`, `unsafe`).
3. **Ungoverned Execution & Security**: Connecting agents directly to live infrastructure risks unauthorized operations. Warmplane enforces multi-tenant RBAC, per-profile server constellations, secret redaction, and Human-in-the-Loop (HITL) approval gates.
4. **Cascading Hangs & Flakiness**: Slow or crashed upstream processes freeze agent loops. Warmplane monitors health with sub-microsecond circuit breakers and self-healing process supervision.

---

## 🚀 Quick Start

### 1. Installation

**Homebrew (macOS & Linux):**
```bash
brew tap warmplane/tap
brew install warmplane
```

**Cargo (crates.io):**
```bash
cargo install warmplane

# Optional: with local ONNX vector search (FastEmbed)
cargo install warmplane --features semantic-search
```

**Build from Source:**
```bash
git clone https://github.com/Warmplane/warmplane.git
cd warmplane
cargo install --path . --features semantic-search
```

### 2. Configure Upstream Servers

Add servers interactively or import from existing AI tools:

```bash
# Interactive setup wizard
warmplane server add

# Or non-interactively
warmplane server add filesystem --command npx --arg "-y" --arg "@modelcontextprotocol/server-filesystem" --arg "/tmp"
warmplane server add context7 --url "https://mcp.context7.ai/sse" --bearer-env "CONTEXT7_API_KEY"

# Or 1-click import from Claude Desktop, Cursor, OpenCode, Zed
warmplane config import
```

Or configure `mcp_servers.json`:

```json
{
  "port": 9090,
  "toolTimeoutMs": 15000,
  "capabilityAliases": {
    "db.query": "sqlite.read_query",
    "search": {
      "target": "semble-rs.search",
      "summary": "Search codebase using semantic or BM25 ranking. Pass absolute repo path."
    }
  },
  "policy": {
    "allow": ["db.*", "fs.*", "search"],
    "deny": ["fs.delete*"],
    "requireApproval": ["db.mutation*"],
    "redactKeys": ["token", "password", "api_key"]
  },
  "profiles": {
    "coding": {
      "servers": ["filesystem", "sqlite"],
      "description": "Local engineering and exploration tools"
    }
  },
  "mcpServers": {
    "sqlite": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-sqlite", "./test.db"] },
    "filesystem": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"] }
  }
}
```

### 3. Start Warmplane

```bash
# Start background daemon & Web Control Deck on http://127.0.0.1:9090
warmplane daemon

# Or expose Warmplane as a stdio MCP server for Claude Desktop or Cursor
warmplane mcp-server
```

---

## 🔌 Client Interfaces

Warmplane exposes three primary access models sharing the same unified core state, policy gates, and telemetry:

### 1. Native MCP Stdio Proxy (`warmplane mcp-server`)
Point any MCP-native desktop client (Claude Desktop, Cursor, Zed, Windsurf) directly to Warmplane:
```json
{
  "mcpServers": {
    "warmplane": {
      "command": "warmplane",
      "args": ["mcp-server", "--config", "mcp_servers.json", "--profile", "coding"]
    }
  }
}
```

### 2. HTTP REST Control Plane (`warmplane daemon`)
Full-featured HTTP JSON API for gateways, web apps, and backend services:
- `GET /v1/capabilities`: Compact capability catalog index
- `POST /v1/capabilities/search`: Hybrid lexical + semantic capability search
- `POST /v1/tools/call`: Normalized execution envelope with context distillation (`_jsonpath`, `_limit_lines`) and idempotency keys
- `POST /v1/tools/batch_call`: Chained multi-step execution with `$step.field` parameter interpolation
- `GET /v1/tasks` & `POST /v1/tasks/:id/update`: SEP-2663 async task lifecycle & HITL review
- `GET /ui`: Embedded standalone Web Control Deck

### 3. In-Process Embedded Engine (`EmbeddedWarmplane`)
Direct in-process Rust library for zero-overhead agent execution without HTTP child processes or network hops:
```rust
use warmplane::{EmbeddedWarmplane, engine::ExecutionOptions};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cp, _token) = EmbeddedWarmplane::start_from_path("mcp_servers.json").await?;
    let res = cp.call_capability(
        "filesystem.read_file",
        json!({ "path": "/tmp/test.txt" }),
        ExecutionOptions::default().with_request_id("req-1"),
    ).await;
    println!("Output: {:?}", res.data);
    Ok(())
}
```

---

## 🤖 Teach Your AI Agent Warmplane (Agent Skill)

Warmplane includes an official **Agent Skill** (`.skills/warmplane/`) adhering to the [`agentskills.io`](https://agentskills.io) open standard. Point your coding agent (Claude Code, Google Antigravity, Cursor, OpenCode, Codex) directly to this repository:

```bash
# Install Warmplane Skill into Claude Code
claude skill install Warmplane/warmplane

# Or copy into your agent workspace
mkdir -p .agents/skills/warmplane && cp -r .skills/warmplane/* .agents/skills/warmplane/
```

- 📖 [`.skills/warmplane/SKILL.md`](.skills/warmplane/SKILL.md) — Core prompt triggers and standard agent workflows
- 🔌 [`references/mcp_stdio_usage.md`](.skills/warmplane/references/mcp_stdio_usage.md) — 1-Click client configs (17 IDEs) & MCP facade tools
- ⚙️ [`references/configuration_schema.md`](.skills/warmplane/references/configuration_schema.md) — `mcp_servers.json` schema & dynamic secrets
- 🛠️ [`references/cli_cheatsheet.md`](.skills/warmplane/references/cli_cheatsheet.md) — Terminal commands for daemon, sync, and vault
- 🚑 [`references/error_resolution.md`](.skills/warmplane/references/error_resolution.md) — Circuit breakers, policy denials, and recovery

---

## 📊 Performance Highlights

Warmplane is engineered in pure Rust with zero-cost abstractions:

- **50.4 ns**: ETag Cache Validation (`If-None-Match` $\rightarrow$ `304 Not Modified`)
- **159.8 ns**: Idempotent Cache-Hit Deduplication
- **1.58 µs**: SHA-256 Incremental Catalog Version Hashing ($N=10$)
- **15.9 µs**: Filtered Hybrid Capability Search ($N=50$)
- **372.1 µs**: Zero-Allocation Lexical Tag Search across 1,000 Tools

👉 See complete benchmarks and methodology in [docs/PERFORMANCE.md](docs/PERFORMANCE.md).

---

## 🛡️ Core Capabilities Matrix

| Capability | Since | Description |
|---|---|---|
| **Custom Alias Descriptions & Signatures** | v0.27.0 | Polymorphic docstring overrides (`AliasTarget`), compact LLM signatures (`tool(req, [opt])`), bidirectional alias resolution |
| **1-Click AI Client Injector & Sync** | v0.26.0 | Bidirectional MCP adapter engine for Claude Desktop, OpenCode, Claude Code, Cursor, Zed, Windsurf, Cline |
| **Native OS Keychain Vault** | v0.26.0 | Secure OS Keychain storage and dynamic secret URI resolution (`keychain://`, `op://`, `env://`) |
| **Actionable ChatOps Webhooks** | v0.26.0 | Bidirectional Slack, Discord, and Microsoft Teams approval cards with HMAC-SHA256 signatures |
| **Per-Profile Governance & Constellations** | v0.26.0 | Fine-grained per-profile policy rules, constellation boundary badges, and live filter metrics |
| **Control Deck Tasks & HITL UI** | v0.25.0 | Live Tasks & Approvals hub, MRTR input resolution forms, Playground async toggle, and embedded task API |
| **SEP-2663 Tasks Extension** | v0.24.0 | Non-blocking `io.modelcontextprotocol/tasks` capability, unified HITL state machine, REST & CLI commands |
| **In-Process Embedded Rust Engine** | v0.23.0 | `EmbeddedWarmplane` & `ControlPlaneHandle` for zero-overhead library integration |
| **Streamable HTTP/SSE MCP Transport** | v0.22.0 | Co-hosted `/mcp/sse` MCP transport for remote agent connectivity |
| **Named Server Constellations** | v0.21.0 | Profile grouping (`profiles`) with scoped ETag partitioning and stdio filtering |
| **Multi-Tenant RBAC** | v0.20.0 | Role-based token access, deterministic catalog partitioning, tenant context propagation |
| **Client-Delegated MCP Sampling** | v0.19.0 | Reverse RPC sampling (`sampling/createMessage`) with ticket tracking & long-polling |
| **Persistent State Subsystem** | v0.18.0 | Atomic restart-resilient disk storage (`AtomicFile<T>`) for approvals, idempotency, and OAuth2 tokens |
| **Signal Handling & Graceful Teardown** | v0.18.0 | Robust `SIGINT`/`SIGTERM` handling, async worker flushes, child process orphan prevention |
| **MCP Resource & Prompt Studio** | v0.17.0 | 360° resource explorer, prompt template renderer with dynamic forms, and SSE syncing |
| **Multi-Step Batch Pipelines** | v0.17.0 | Visual pipeline editor with reference parameter interpolation (`POST /v1/tools/batch_call`) |
| **Enterprise Security & Auth** | v0.16.0 | Token-based middleware protection, WORM audit HMAC verification, and secret masking |
| **Fault Tolerance & Supervision** | v0.15.0 | Degraded startup, per-server circuit breakers, exponential backoff restart supervision |
| **Agent Enrichment Suite** | v0.14.0 | Facade search, context distillation (`_jsonpath`/`_limit_lines`), and multi-step batch execution |
| **HITL Approval Engine** | v0.13.0 | Operator gate approval engine, suspension, argument editing, and HMAC webhook dispatch |
| **WORM Audit & SIEM** | v0.12.0 | Append-only SHA-256 hash-chained audit logging, verification API, and Splunk/Webhook SIEM export |
| **Control Deck Web UI** | v0.11.0 | Standalone embedded web dashboard for servers, testing playground, policy & telemetry |
| **Dynamic Hot-Reloading** | v0.11.0 | Zero-downtime upstream mounting/unmounting, explicit `warmplane reload` & `/v1/config/reload` |

---

## Changelog

### v0.27.0 — Custom Alias Descriptions, Compact LLM Tool Signatures & Task Inspector
- **Custom Alias Descriptions & Docstring Overrides (`src/config.rs`, `src/supervisor.rs`):** Upgraded alias configuration model to support polymorphic definitions (`AliasTarget`). Aliases can be simple target strings (`"alias": "server.tool"`) or detailed objects (`"alias": { "target": "server.tool", "summary": "...", "description": "..." }`), enabling platform engineers and developers to repair or improve poorly-described upstream tools for zero-shot LLM ergonomics without upstream source changes.
- **Compact LLM Tool Signatures (`src/supervisor.rs`, `src/daemon/types.rs`, `src/engine/types.rs`):** Derived deterministic, compact parameter signatures (`tool_name(req1, [opt1], [opt2])`) from JSON Schemas (accounting for required fields vs nullable/optional properties). Surfaced across MCP `capabilities_list`, catalog search, and Web UI index summaries.
- **Bidirectional Alias Resolution (`src/supervisor.rs`):** Resolved mapping mismatch where supervisory discovery checks target equality against configured alias keys, ensuring canonical targets are promoted seamlessly to client interfaces.
- **Rich Task Inspector Modal & Dual Controls (`ui/src/components/tasks.ts`):** Enhanced Tasks & Approvals UI with dedicated inspector modal, live state viewers, formatted JSON payload inspections, and dual inspect/cancel action controls.
- **Server Template Missing Secret Warnings (`ui/src/components/servers.ts`, `src/vault/`):** Added live `(Missing Keys)` warning badges and status indicators across server cards and diagnostics when required template environment variables or Keychain secrets are unconfigured.
- **Live Alias Configuration API & UI:** Added custom summary inputs to the Control Deck Aliases tab (`ui/src/components/aliases.ts`) and CLI (`warmplane config alias set --summary ...`) with automated live hot-reloading reconciliation on disk mutation.

### v0.26.1 — MCP Stdio Stream Isolation & Logging Fix
- **MCP Stdio Stream Isolation (`src/telemetry.rs`):** Configured `tracing_subscriber::fmt::layer()` to write to `stderr` (`.with_writer(std::io::stderr)`). Prevents runtime structured JSON logs and span diagnostics from polluting `stdout`.
- **Upstream Process Stderr Inheritance (`src/supervisor.rs`):** Upstream stdio child processes now explicitly inherit Warmplane's standard error (`cmd.stderr(std::process::Stdio::inherit())`). Prevents upstream startup banners (e.g. Memory and Filesystem server banners) from leaking into stdio JSON-RPC sessions.
- **Client Protocol Reliability:** Resolves JSON-RPC initialization failure (`invalid message version tag ""; expected "2.0"`) when running Warmplane in stdio server mode (`warmplane mcp-server`) with AI agents and IDEs.

### v0.26.0 — 1-Click AI Client Sync, Native Secrets Vault, ChatOps & Profile Governance
- **1-Click AI Client Injector & Ecosystem Sync (`src/client_sync.rs`):** Zero-configuration bidirectional MCP adapter engine. Detects, injects, and detaches Warmplane proxy configurations with profile binding across Claude Desktop (macOS, Linux, Windows), OpenCode, Claude Code CLI (`CLAUDE_CONFIG_DIR`), Cursor (Global & Workspace), Zed Editor (`context_servers`), Windsurf, and Roo Code / Cline.
- **100% Agent Config Import Parity (`src/config_import.rs`):** Unified external config discovery with dialect-aware parsers (`StandardMcpServers`, OpenCode `mcp`, Zed `context_servers`) and self-proxy protection.
- **Native OS Keychain Vault & Dynamic Secrets (`src/vault/`):** Added secure OS-level credential management (`warmplane secret set/get/delete`) and dynamic runtime secret expansion (`keychain://`, `op://`, `env://`) in environment variables with masked logs.
- **Actionable ChatOps & Bidirectional Webhooks (`src/chatops/`):** Rich interactive approval cards for Slack (Block Kit), Discord (Embeds), and Microsoft Teams (Adaptive Cards) with HMAC-SHA256 signature verification.
- **Per-Profile Governance Policies (`src/policy.rs`, `ui/src/components/policy.ts`):** Fine-grained per-profile `allow`, `deny`, and `requireApproval` rules overriding or scoping global policies.
- **Constellation Boundaries & Dynamic Visibility (`ui/src/components/servers.ts`):** Visual constellation badges (`✔ IN CONSTELLATION`, `🚫 EXCLUDED FROM PROFILE`), auto-derived `<server>.*` implicit policy denials, and 1-click membership toggles.
- **Server Diagnostics & 1-Click Restart:** Added live error diagnostics modals, server restart endpoint (`POST /v1/config/servers/:id/restart`), and automated smoke testing suite for all 25 MCP server templates (`scripts/test-templates.ts`).
- **Dynamic Catalog ETag Fingerprinting & Layout Stabilization:** Profile-aware fingerprint hashing (`sha256:...-p:<profile_id>:<hash>`) ensuring immediate ETag invalidation and playground catalog re-population. Stabilized viewport layouts with continuous scrollbar gutter reservation.

### v0.25.2 — Official MCP Registry Metadata & MCPB Packaging Format
- **MCPB Distribution Format (`packaging/mcpb/`, `.github/workflows/release-artifacts.yml`):** Added automated build and packaging of platform-specific Model Context Protocol Bundles (`.mcpb`) containing standalone binaries, bootstrap configurations, and standardized manifests (`manifest_version: "0.3"`).
- **Official MCP Registry Metadata (`server.json`):** Release workflows now automatically generate canonical registry metadata adhering to the official `server.schema.json` specification (`io.github.warmplane/warmplane`) with multi-arch SHA-256 package digests.
- **Homebrew Tap (`Warmplane/homebrew-tap`):** Configured official tap distribution with prebuilt macOS and Linux formula (`brew tap warmplane/tap && brew install warmplane`).

### v0.25.1 — Asynchronous Task Completion State Machine Fix
- **Asynchronous Task Finalization (`src/engine/mod.rs`, `src/http_v1/execute.rs`):** Resolved regression where asynchronous capability executions (`async_task: true` or `Prefer: respond-async`) and tasks resumed after Human-in-the-Loop (HITL) input responses remained indefinitely in `TaskStatus::Working`. Background workers now reliably record terminal state (`TaskStatus::Completed` with `result`, or `TaskStatus::Failed` with structured error) directly into `TaskRegistry`.
- **Embedded Task Lifecycle Tests (`tests/embedded_tests.rs`, `tests/tasks_tests.rs`):** Added comprehensive automated integration tests verifying that `get_task` and `list_tasks` observe terminal `completed` status with upstream payload following approval submissions and direct asynchronous calls.

### v0.25.0 — Control Deck Tasks & HITL UI, In-Process Embedded Task API
- **Control Deck Tasks & Approvals Hub (`ui/src/components/tasks.ts`):** Upgraded the review queue into a unified Tasks & Approvals dashboard (`data-tab="tasks"`) with live status KPIs (`input_required`, `working`, `completed`, `cancelled/failed`), interactive action cards with inlined MRTR input resolution forms (booleans, JSON editors, text fields), TTL countdown timers, and cooperative cancellation controls.
- **MCP Playground Async Execution Mode:** Added **"⚡ Async Task Mode"** toggle in the tool testing playground and an interactive `202 Accepted` task card preview with 1-click navigation to the Tasks & Approvals review deck.
- **Embedded Rust Task Management API (`ControlPlaneHandle`):** Exposed direct task management methods on the in-process `ControlPlaneHandle` (`list_tasks`, `get_task`, `update_task`, `cancel_task`), allowing embedding applications to manage asynchronous SEP-2663 tasks without HTTP or JSON-RPC serialization overhead.
- **Overview Cockpit & Badge Integration:** Added **"Tasks & HITL State"** telemetry card in the Overview Cockpit and linked real-time sidebar badges to outstanding `input_required` tasks.

### v0.24.0 — SEP-2663 Tasks Extension, Unified HITL Execution & MCP Roadmap Alignment
- **SEP-2663 Tasks Extension Implementation (`src/tasks.rs`):** Implemented the official `io.modelcontextprotocol/tasks` capability with atomic persistent storage (`tasks.json`), safe TTL expiration detection, and oneshot input response channels (`TaskWaitSender` / `TaskWaitReceiver`).
- **Unified HITL Approval & Asynchronous Task State Machine:** Unified Human-in-the-Loop approval workflows and asynchronous tool executions directly onto the SEP-2663 lifecycle. Tool executions requiring operator gate approval or requested with `async_task: true` / `Prefer: respond-async` return `202 Accepted` with `resultType: "task"` and `status: "input_required"`, inlining MRTR `inputRequests`.
- **HTTP REST Task API (`/v1/tasks/*`):** Added comprehensive REST endpoints for task inspection and control: `GET /v1/tasks` (list), `GET /v1/tasks/:id` (poll status/result), `POST /v1/tasks/:id/update` (submit `inputResponses` to wake up worker), and `POST /v1/tasks/:id/cancel` (cooperative cancellation).
- **MCP Facade Server Integration:** Exposed `task_get`, `task_update`, and `task_cancel` tools on the MCP facade server for direct agent orchestration.
- **CLI Tasks Subcommands & Async Execution Flag:** Added `warmplane tasks list`, `warmplane tasks get <id>`, `warmplane tasks update <id> -r '<json>'`, `warmplane tasks cancel <id>`, and the `--async-task` flag to `warmplane call-capability`.
- **MCP Protocol Roadmap Strategic Analysis:** Published `docs/MCP_ROADMAP_REPORT.md` analyzing the official MCP roadmap and Warmplane's architectural alignment across stateless transports, sessionless routing, and progressive capability discovery.

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
