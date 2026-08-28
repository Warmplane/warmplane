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
- **Tier 1 (Native MCP Stdio - Primary Mode)**: You connect to `warmplane` as a single MCP server over stdio. Tools appear namespaced as `<server>.<tool>` with zero startup delay. See [MCP Stdio Reference](./references/mcp_stdio_usage.md).
- **Tier 2 (REST & CLI - Orchestration)**: Use for progressive tool discovery, health checks, hot-reloading (`warmplane reload`), secrets vault, and Human-in-the-Loop (HITL) gates. See [REST API Reference](./references/http_rest_api.md) and [CLI Cheatsheet](./references/cli_cheatsheet.md).

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

## 4. Reference Map
- [Setup & Install Guide](./references/setup_and_installation.md): Binaries, package managers, and verification.
- [MCP Stdio Usage](./references/mcp_stdio_usage.md): 1-Click client config for 17 IDEs/agents and stdio tools.
- [HTTP REST API](./references/http_rest_api.md): Complete `/v1/...` endpoint reference.
- [CLI Cheatsheet](./references/cli_cheatsheet.md): Shell commands for daemon, 1-click sync, and vault.
- [Configuration Schema](./references/configuration_schema.md): `mcp_servers.json` spec, dynamic secrets (`keychain://`, `op://`, `env://`), profiles, and resilience.
- [Troubleshooting](./references/error_resolution.md): Circuit breakers, policy denials, and cascade cleanup.
