---
name: warmplane
description: >-
  Expert guide for Warmplane, a local Model Context Protocol (MCP) control plane and facade.
  Use when configuring, discovering, calling, or managing MCP tools, servers, profiles,
  secrets vault, or 1-click client integrations.
---

# Warmplane Skill

Warmplane aggregates multiple upstream MCP servers behind a single runtime. It keeps upstream connections warm in the background, compresses tool metadata to save context tokens, and exposes deterministic interfaces over **native MCP stdio**, **HTTP REST API**, and **CLI**.

## 1. Quick Install & Setup
- **Homebrew (macOS/Linux)**: `brew tap warmplane/tap && brew install warmplane`
- **Cargo (crates.io)**: `cargo install warmplane` (add `--features semantic-search` for ONNX vector search)
- **Pre-built Binary**: Download release binary from GitHub Releases
- **Source**: `git clone https://github.com/Warmplane/warmplane.git && cd warmplane && cargo install --path .`
- See [Setup Reference](./references/setup_and_installation.md) for verification and build flags.

## 2. Operational Model
- **Tier 1 (Native MCP Stdio - Primary Mode)**: You connect to `warmplane` as a single MCP server over stdio. The client spawns `warmplane mcp-server` as a child process (no background daemon needed). Tools appear namespaced as `<server>.<tool>` or top-level passthrough aliases with zero startup delay. See [MCP Stdio Reference](./references/mcp_stdio_usage.md).
- **Tier 2 (HTTP Daemon & Web UI - Centralized Mode)**: Start `warmplane daemon` to host the Web UI, REST endpoints, and optional Streamable HTTP/SSE facade (`http://127.0.0.1:9191/sse`) for multi-client connections.
- **1-click client connections**: The Web UI can attach clients over stdio (client-launched `warmplane mcp-server`) or HTTP (a URL to the already-running daemon's `/mcp` endpoint). HTTP requires `mcpHttpServer` to be configured and the daemon restarted; it does not launch the daemon automatically.
- **Tier 3 (REST & CLI - Orchestration)**: Use for progressive tool discovery, health checks, hot-reloading (`warmplane reload`), secrets vault, and Human-in-the-Loop (HITL) gates. See [REST API Reference](./references/http_rest_api.md) and [CLI Cheatsheet](./references/cli_cheatsheet.md).

## 3. Workflows

### W1: Connect & Call over Stdio MCP
Configure your client (e.g. `.cursor/mcp.json`, `opencode.json`):
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
Call tools directly via standard MCP JSON-RPC `tools/call` on `<server>.<tool>`.

### W2: Progressive Tool Discovery (Native MCP)
In large catalogs, avoid loading all schemas into context. Discover on demand:
1. Search tools: Call MCP tool `capability_search` with `{"query": "database query", "limit": 5}`.
2. Inspect schema: Call MCP tool `capability_describe` with `{"id": "postgres.query"}`.
3. Execute: Call the discovered tool directly via MCP `tools/call`.

### W3: Restart Failing Upstream Servers
If an upstream process crashes or reports `Circuit: OPEN`:
- CLI: `warmplane server restart <server-id>`
- REST: `POST http://127.0.0.1:9090/v1/config/servers/{server_id}/restart`

### W4: Hold and Cancel Delayable Emissions
For aliases configured with `reversibility.class: "delayable"`, Warmplane starts the hold after any required approval. The duration precedence is profile policy, alias `holdWindowSecs`, global policy, then 30 seconds. Async calls return a task receipt (`phase`, `releaseAtMs`, `cancellable`); synchronous calls wait only when the hold is shorter than `toolTimeoutMs`. Cancel with `/v1/tasks/{taskId}/cancel`, `/v1/operations/{requestId}/cancel`, MCP `tasks/cancel`, or the embedded `ControlPlaneHandle::cancel_task` API. A `409 EMISSION_DISPATCHING` means dispatch was durably claimed. Pending calls resume after restart and policy/profile revalidation; calls recovered in `dispatching` fail as `EMISSION_OUTCOME_UNKNOWN` and are never automatically resent. Batch steps wait for the held result within the batch budget.

When task persistence is disabled, accepted holds exist only in memory and are lost when the process exits. Persistent state is required for restart recovery.

For operators and agents to share live undo state, use the daemon with `mcpHttpServer` enabled and connect agents to `/mcp`. Separate stdio processes own separate runtime state.

### W5: Inspect and Run Undo Entries
Operators can list and inspect safe undo summaries with `GET /v1/undo` and `GET /v1/undo/{undo_id}`, execute an entry with `POST /v1/undo/{undo_id}/execute`, or unwind a work item in reverse order with `POST /v1/undo/work-items/{work_item_id}/rollback`. Configure role `undoPermissions` for these operations. Expired entries require the dedicated `forceExpired` permission and a non-empty reason, which is included in the committed audit event. MCP `undo_execute` is exposed only when both the RBAC role and active profile opt in with `allowAgentSelfUndo`; agents can target a known `undo_id` or the newest armed entry for a `work_item_id`.

The Control Deck Action Feed provides Previous/Next navigation through undo history in pages of 50 and retains the selected page on live refresh. Work-item rollback includes eligible entries on other pages. Internal snapshot and drift-check capabilities must be allowed without approval by the effective policy. The feed consumes `action_capsule` invalidations from `/v1/resources/updates` and refreshes tenant-scoped undo and task lists from REST. SSE invalidations contain no tenant or action data. Feed cards show filtered inverse field names, but omit inverse arguments and snapshot contents. An optional `preUndoCheck` reads and hashes the expected post-execution state before dispatch; drift blocks the inverse and leaves the entry armed for review. Configure all four reversibility classes and their class-specific fields in **Facade & Alias Studio**; JSON argument templates must be objects. Configure `reversibility.testTarget.server` as a dedicated MCP test server and fixed `args`; `preSnapshot` is required. `warmplane undo test <alias>` and the Control Deck (via the Playground or **Test saved contract** in the Alias Studio) run the saved forward/inverse cycle against that server and compare the final state to the original snapshot. Signed ChatOps Undo and Cancel Send cards are sent only when the corresponding event is explicitly listed in `policy.webhook.events`; configure `secret`/`secretEnv`, `callbackUrl`, and `actionRole`. The `/v1/webhooks/actions` token is target-, tenant-, role-, action-, and expiry-bound. ChatOps cannot force an expired undo.

## 4. Reference Map
- [Setup & Install Guide](./references/setup_and_installation.md): Binaries, package managers, and verification.
- [MCP Stdio Usage](./references/mcp_stdio_usage.md): 1-Click client config for 17 IDEs/agents and stdio tools.
- [HTTP REST API](./references/http_rest_api.md): Complete `/v1/...` endpoint reference.
- [CLI Cheatsheet](./references/cli_cheatsheet.md): Shell commands for daemon, 1-click sync, and vault.
- [Configuration Schema](./references/configuration_schema.md): `mcp_servers.json` spec, dynamic secrets (`keychain://`, `op://`, `env://`), profiles, resilience, and reversibility contracts.
- [Troubleshooting](./references/error_resolution.md): Circuit breakers, policy denials, and cascade cleanup.
