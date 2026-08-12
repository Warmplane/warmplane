# Warmplane User Guide

Warmplane is the local control plane that keeps MCP sessions warm.

It runs multiple upstream MCP servers behind one local runtime and exposes one compact, deterministic interaction surface for tools, resources, and prompts.

This guide covers setup, configuration, run modes, APIs, operations, and troubleshooting.

## 1. What Warmplane Does

Warmplane is designed to improve three measurable outcomes:

- startup latency: avoids repeated cold-start/handshake overhead by maintaining upstream sessions
- payload size: exposes compact indexes first, details on demand
- determinism: normalizes call/read/get envelopes and error classes

Warmplane supports three client interaction modes over the same backend state:

1. HTTP facade (`/v1/...`)
2. CLI facade commands
3. MCP server mode (`mcp-server`) for MCP-native clients

## 2. Installation

### Build from source

```bash
cargo build --release
```

Binary path:

```bash
./target/release/warmplane
```

### Dev run

```bash
cargo run -- daemon --config mcp_servers.json
```

## 3. Quick Start

1. Create `mcp_servers.json`.
2. Start daemon mode.
3. Verify capability listing.

Minimal example:

```json
{
  "port": 9090,
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

Start daemon:

```bash
cargo run --release -- daemon --config mcp_servers.json
```

Check capabilities:

```bash
curl -s http://127.0.0.1:9090/v1/capabilities | jq
```

## 4. Configuration Reference

Top-level fields:

- `port` (optional): HTTP daemon port, default `9090`
- `toolTimeoutMs` (optional): upstream timeout per call/read/get, default `15000`
- `capabilityAliases` (optional): map `<server>.<tool>` to exported capability ID
- `resourceAliases` (optional): map `<server>.<resource-uri>` to exported resource ID
- `promptAliases` (optional): map `<server>.<prompt-name>` to exported prompt ID
- `policy` (optional): allow/deny/redaction controls
- `mcpServers` (required): upstream server definitions

### 4.1 Upstream transport selection

Per `mcpServers.<id>`, set exactly one of:

- `command` for stdio
- `url` for streamable HTTP

If both or neither are set, Warmplane fails fast with validation errors.

### 4.2 Stdio upstream config

Fields:

- `command` (required)
- `args` (optional)
- `env` (optional key/value)

Example:

```json
"sqlite": {
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-sqlite", "./test.db"],
  "env": {
    "NODE_ENV": "production"
  }
}
```

### 4.3 HTTP/SSE upstream config

Fields:

- `url` (required)
- `protocolVersion` (optional, default `2025-11-25`)
- `allowStateless` (optional)
- `headers` (optional key/value)
- `auth` (optional)

Example:

```json
"github": {
  "url": "https://api.githubcopilot.com/mcp/",
  "protocolVersion": "2025-11-25",
  "allowStateless": true,
  "headers": {
    "X-Tenant": "acme"
  },
  "auth": {
    "type": "bearer",
    "tokenEnv": "GITHUB_MCP_PAT"
  }
}
```

### 4.4 Auth schema

Bearer:

```json
{
  "type": "bearer",
  "tokenEnv": "MCP_TOKEN"
}
```

Basic:

```json
{
  "type": "basic",
  "username": "svc-user",
  "passwordEnv": "MCP_PASSWORD"
}
```

OAuth2 (OAuth 2.1 / OIDC):

```json
{
  "type": "oauth2",
  "clientId": "client-id-here",
  "authorizationServerUrl": "https://auth.example.com",
  "scopes": ["read", "write"],
  "clientMetadataUrl": "https://example.com/metadata.json"
}
```

Rules:

- bearer: exactly one of `token` or `tokenEnv`
- basic: exactly one of `password` or `passwordEnv`
- oauth2: requires non-empty `clientId` and `authorizationServerUrl`. Optionally takes `scopes` and `clientMetadataUrl`.

When OAuth2 is configured, Warmplane automatically spins up a local background proxy server on an ephemeral loopback port to:
- Probe and discover authorization servers (RFC 9728 & RFC 8414).
- Execute the cryptographically bound Authorization Code flow with PKCE (`S256`).
- Handle loopback redirection callbacks and validate the expected issuer parameter (RFC 9207 / SEP-2468).
- Accumulate scopes dynamically during step-up challenges (`403 Forbidden` with `insufficient_scope`).
- Silently refresh access tokens using refresh tokens (`offline_access`).

### 4.5 Policy

Policy applies consistently across tools/resources/prompts.

Example:

```json
"policy": {
  "allow": ["db.*", "fs.*", "prompt.*"],
  "deny": ["fs.secret", "db.delete"],
  "redactKeys": ["token", "api_key", "password"]
}
```

Semantics:

- `deny` takes precedence over `allow`
- if `allow` is empty, default allow
- wildcard suffix supported: `prefix*` and `*`

## 5. Run Modes

## 5.1 HTTP daemon mode

Start:

```bash
warmplane daemon --config mcp_servers.json
```

Dev equivalent:

```bash
cargo run -- daemon --config mcp_servers.json
```

Default bind: `127.0.0.1:<port>`.

### HTTP endpoints

- `GET /v1/capabilities` (supports `If-None-Match` header -> `304 Not Modified`, returns `ETag`)
- `POST /v1/capabilities/search`
- `GET /v1/capabilities/:id` (supports `If-None-Match` header -> `304 Not Modified`, returns `ETag`)
- `POST /v1/tools/call`
- `GET /v1/resources` (supports `If-None-Match` header -> `304 Not Modified`, returns `ETag`)
- `POST /v1/resources/read`
- `GET /v1/prompts` (supports `If-None-Match` header -> `304 Not Modified`, returns `ETag`)
- `POST /v1/prompts/get`
- `GET /v1/catalog/events` (catalog change feed: `?after={cursor}`)

#### Capability Search (`POST /v1/capabilities/search`)

Request payload:
```json
{
  "query": "triage production errors",
  "limit": 5,
  "server_ids": ["observability"],
  "tags": ["logs"],
  "modes": ["read"]
}
```

Response payload:
```json
{
  "version": "v1",
  "catalog_version": "sha256:8f2a1b...",
  "capabilities": [
    {
      "id": "obs.logs.search",
      "summary": "Search structured application logs.",
      "server": "observability",
      "tags": ["logs", "read"],
      "mode": "read",
      "score": 0.91,
      "match_types": ["lexical", "semantic", "tag"]
    }
  ]
}
```

## 5.2 MCP server mode (stdio)

Start:

```bash
warmplane mcp-server --config mcp_servers.json
```

This exposes a compact MCP facade to MCP-native clients.

Synthetic tool surface:

- `capabilities_list`
- `capability_describe`
- `capability_call`
- `resources_list`
- `resource_read`
- `prompts_list`
- `prompt_get`

Native MCP methods also exposed:

- resources: `resources/list`, `resources/read`
- prompts: `prompts/list`, `prompts/get`

## 5.3 CLI facade mode

Capabilities:

```bash
warmplane list-capabilities --config mcp_servers.json
warmplane search-capabilities "triage logs" --limit 5 --config mcp_servers.json
warmplane describe-capability db.query --config mcp_servers.json
warmplane call-capability db.query --params '{"query":"SELECT 1"}' --config mcp_servers.json
```

Resources:

```bash
warmplane list-resources --config mcp_servers.json
warmplane read-resource fs.readme --config mcp_servers.json
```

Prompts:

```bash
warmplane list-prompts --config mcp_servers.json
warmplane get-prompt prompt.code-review --arguments '{"code":"fn main() {}"}' --config mcp_servers.json
```

## 6. HTTP API Semantics

List endpoints return versioned payloads:

- `{ "version": "v1", "capabilities": [...] }`
- `{ "version": "v1", "resources": [...] }`
- `{ "version": "v1", "prompts": [...] }`

Execution/read/get endpoints return normalized envelopes:

```json
{
  "ok": true,
  "request_id": "optional-client-id",
  "trace_id": "server-trace-id",
  "data": {},
  "error": null
}
```

Error envelope shape is stable (`ok: false`, `error.code`, `error.message`) and uses these codes:

- `TOOL_NOT_FOUND`
- `RESOURCE_NOT_FOUND`
- `PROMPT_NOT_FOUND`
- `SERVER_UNREACHABLE`
- `INVALID_ARGS`
- `UPSTREAM_TIMEOUT`
- `UPSTREAM_ERROR`
- `INTERNAL_ERROR`

## 7. Aliasing Strategy

Aliases decouple client contracts from upstream naming drift.

Recommended pattern:

- Use domain-centric IDs (example: `payments.charge.create`, `repo.issue.open`)
- Avoid embedding environment details in public IDs
- Treat alias ID changes as API versioning events

## 8. Token Efficiency Evaluation

Warmplane includes a dedicated evaluation harness:

- path: `eval/token-efficiency/`
- outputs:
  - `eval/token-efficiency/output/summary.json`
  - `eval/token-efficiency/output/report.md`

Run:

```bash
cargo run --manifest-path eval/token-efficiency/Cargo.toml -- \
  --suite-dir eval/token-efficiency/suites \
  --out-dir eval/token-efficiency/output
```

This compares raw MCP payload footprints vs Warmplane facade footprints using `cl100k_base` tokenization.

## 9. Operations Guidance

### 9.0 Observability Defaults

Warmplane uses structured JSON logs (`tracing`) by default.

Useful environment controls:

- `RUST_LOG=info,warmplane=debug`
- `WARMPLANE_OTEL_ENABLED=true`
- `OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317`
- `WARMPLANE_OTEL_ENDPOINT=http://127.0.0.1:4317` (fallback if `OTEL_EXPORTER_OTLP_ENDPOINT` unset)
- `WARMPLANE_SERVICE_NAME=warmplane-prod`

When OTEL is enabled, Warmplane exports traces over OTLP gRPC and still emits structured local logs.

### 9.1 Startup and health

At startup Warmplane logs:

- upstream server boot progress
- registered tools/resources/prompts
- daemon listening address

A simple readiness probe:

```bash
curl -sf http://127.0.0.1:9090/v1/capabilities >/dev/null
```

### 9.2 Timeout tuning

Use `toolTimeoutMs` to set upper bounds for upstream operations.

- Too low: false timeouts for slower servers
- Too high: longer stalls before deterministic failure

Start with `15000` and adjust using observed p95/p99 upstream behavior.

### 9.3 Security posture

- Keep daemon bound to localhost unless you explicitly front it with a trusted reverse proxy.
- Prefer env-backed secrets (`tokenEnv`, `passwordEnv`) over inline secrets.
- Use `policy.deny` for destructive operations by default.

## 10. MCP Client Integration Example

Client config snippet:

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

## 11. Troubleshooting

### Error: ambiguous or invalid server transport

Cause:

- both `command` and `url` configured, or neither configured

Fix:

- set exactly one transport selector per server

### Error: auth configuration invalid

Cause:

- bearer/basic has both inline and env secret set, or neither set

Fix:

- set exactly one secret source

### Error: `SERVER_UNREACHABLE`

Cause:

- upstream process died / mailbox closed / HTTP target unavailable

Fix:

- verify upstream command/URL and credentials
- check startup logs for negotiation failure

### Error: `UPSTREAM_TIMEOUT`

Cause:

- upstream operation exceeded `toolTimeoutMs`

Fix:

- increase timeout or optimize upstream server behavior

## 12. Development and Validation

Main regression check:

```bash
cargo test
```

MCP smoke test:

```bash
./scripts/smoke_mcp_server.sh
```

## 13. Reference Docs

- API spec: [spec.md](docs/spec.md)
- Token research: [TOKEN_EFFICIENCY_RESEARCH_REPORT.md](docs/research/TOKEN_EFFICIENCY_RESEARCH_REPORT.md)
- Editorial: [NEXT_LEVEL_TOOL_CALLING.md](docs/NEXT_LEVEL_TOOL_CALLING.md)
- Narrative take: [TAKE_TWO.md](docs/TAKE_TWO.md)
