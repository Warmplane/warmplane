# Warmplane User Guide

Warmplane is a local control plane that maintains persistent Model Context Protocol (MCP) sessions.

It aggregates multiple upstream MCP servers behind a single runtime. It provides a compact, deterministic interface for tools, resources, and prompts over HTTP REST APIs, CLI commands, and stdio MCP server proxying.

This guide provides configuration references, deployment procedures, API specifications, and operational guidance for enterprise platform engineers and software integrators.

---

## 1. System Overview

Warmplane optimizes AI agent tool execution across three core technical metrics:

- **Startup latency**: Eliminates per-call handshake overhead by maintaining persistent, warm connections to upstream MCP servers.
- **Payload footprint**: Reduces LLM context token usage by serving lightweight catalog indexes first and full schema details on demand.
- **System determinism**: Standardizes request envelopes, response structures, and error classifications across heterogeneous upstream tools.

### Client Interfaces

Warmplane exposes three client interfaces backed by shared daemon state:

1. **HTTP REST API (`/v1/...`)**: Low-overhead HTTP JSON API for web applications, microservices, and orchestration gateways.
2. **MCP Stdio Server Mode (`mcp-server`)**: Standard MCP stdio interface for direct integration with MCP-native AI clients.
3. **CLI Facade (`warmplane <command>`)**: Command-line interface for administrative scripting, health checks, and manual debugging.

---

## 2. Installation and Build

### Prerequisites

- Rust toolchain (version 1.80 or later)
- Cargo package manager

### Build from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/Warmplane/warmplane.git
   cd warmplane
   ```

2. Compile the release binary:

   ```bash
   cargo build --release
   ```

3. Locate the compiled executable at `./target/release/warmplane`.

4. Install the binary to system path:

   ```bash
   cargo install --path .
   ```

---

## 3. Quick Start

Follow this procedure to deploy a local Warmplane daemon with a sample filesystem MCP server.

1. Create a configuration file named `mcp_servers.json`:

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

2. Start the daemon process:

   ```bash
   warmplane daemon --config mcp_servers.json
   ```

3. Verify catalog discovery from a separate terminal:

   ```bash
   curl -s http://127.0.0.1:9090/v1/capabilities | jq
   ```

   **Expected Result**: The endpoint returns HTTP 200 OK with a JSON array listing discovered tools under the `filesystem` namespace.

---

## 4. Configuration Reference

The configuration file (default: `mcp_servers.json`) controls upstream server connections, network ports, access control policies, and alias mappings.

### Top-Level Fields

| Field | Type | Required | Default | Description |
| :--- | :--- | :---: | :--- | :--- |
| `port` | Number | No | `9090` | TCP listening port for the HTTP daemon. |
| `toolTimeoutMs` | Number | No | `15000` | Upstream operation timeout in milliseconds. |
| `capabilityAliases` | Object | No | `{}` | Map of upstream tool identifiers (`<server>.<tool>`) to canonical public capability IDs. |
| `resourceAliases` | Object | No | `{}` | Map of upstream resource URIs to canonical public resource IDs. |
| `promptAliases` | Object | No | `{}` | Map of upstream prompt names to canonical public prompt IDs. |
| `policy` | Object | No | `null` | Access control rules and data redaction settings. |
| `mcpServers` | Object | Yes | N/A | Upstream server definitions keyed by server identifier string. |

---

### 4.1 Upstream Server Transports

Each entry in `mcpServers` defines an upstream server. Configure exactly one transport selector per server:

- `command`: Stdio process transport
- `url`: Streamable HTTP/SSE transport

Warmplane fails startup validation if a server definition specifies both or neither transport selectors.

#### Stdio Transport Configuration

| Field | Type | Required | Description |
| :--- | :--- | :---: | :--- |
| `command` | String | Yes | Path or binary name of the executable process. |
| `args` | Array | No | Command-line arguments passed to the executable. |
| `env` | Object | No | Key-value pairs of environment variables injected into the process. |

Example:

```json
"sqlite": {
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-sqlite", "./production.db"],
  "env": {
    "NODE_ENV": "production"
  }
}
```

#### HTTP / Streamable SSE Transport Configuration

| Field | Type | Required | Default | Description |
| :--- | :--- | :---: | :--- | :--- |
| `url` | String | Yes | N/A | Remote HTTP/SSE server endpoint URL. |
| `protocolVersion` | String | No | `"2025-11-25"` | MCP protocol version header string. |
| `allowStateless` | Boolean | No | `false` | Enables stateless HTTP execution for supported endpoints. |
| `headers` | Object | No | `{}` | Custom HTTP headers sent with every request. |
| `auth` | Object | No | `null` | Authentication configuration block. |

Example:

```json
"github": {
  "url": "https://api.githubcopilot.com/mcp/",
  "protocolVersion": "2025-11-25",
  "allowStateless": true,
  "headers": {
    "X-Tenant-ID": "enterprise-corp"
  },
  "auth": {
    "type": "bearer",
    "tokenEnv": "GITHUB_MCP_PAT"
  }
}
```

---

### 4.2 Authentication Protocols

Warmplane supports Bearer token, HTTP Basic, and enterprise OAuth2 / OpenID Connect (OIDC) authentication.

#### Bearer Token Authentication

Specify exactly one secret source (`token` or `tokenEnv`).

```json
{
  "type": "bearer",
  "tokenEnv": "UPSTREAM_API_TOKEN"
}
```

#### HTTP Basic Authentication

Specify `username` and exactly one password source (`password` or `passwordEnv`).

```json
{
  "type": "basic",
  "username": "service-account-warmplane",
  "passwordEnv": "UPSTREAM_BASIC_PASSWORD"
}
```

#### OAuth 2.1 / OIDC Authentication

Warmplane implements enterprise OAuth 2.1 authorization code flow with PKCE (`S256`).

| Field | Type | Required | Description |
| :--- | :--- | :---: | :--- |
| `clientId` | String | Yes | Registered OAuth2 client ID. |
| `authorizationServerUrl` | String | Yes | Base URL of the OAuth2 authorization server / identity provider. |
| `scopes` | Array | No | Requested OAuth2 scope strings (e.g. `["read", "write"]`). |
| `clientMetadataUrl` | String | No | Optional URL for OAuth2 client metadata discovery (RFC 7591). |

Example:

```json
{
  "type": "oauth2",
  "clientId": "warmplane-gateway-client",
  "authorizationServerUrl": "https://identity.enterprise.com/oauth2/v1",
  "scopes": ["mcp.read", "mcp.execute"],
  "clientMetadataUrl": "https://identity.enterprise.com/.well-known/oauth-authorization-server"
}
```

When an upstream server requires OAuth2 authentication, Warmplane executes the following automated lifecycle:

1. **Discovery**: Probes identity providers using RFC 9728 and RFC 8414 metadata discovery standards.
2. **PKCE Security**: Generates cryptographically random code verifiers and SHA-256 code challenges (`S256`).
3. **Loopback Redirection**: Launches a temporary loopback callback listener on `127.0.0.1` and validates state and issuer parameters (RFC 9207 / SEP-2468).
4. **Step-Up Authorization**: Captures `403 Forbidden` `insufficient_scope` challenges from upstream servers and triggers incremental scope accumulation.
5. **Token Renewal**: Automatically invokes refresh token flows (`offline_access`) to maintain non-blocking session continuity.

---

### 4.3 Policy and Access Control

The `policy` block enforces global security boundaries across all capabilities, resources, and prompts.

```json
"policy": {
  "allow": ["github.*", "obs.*", "db.read_*"],
  "deny": ["*.delete", "db.drop_*", "admin.*"],
  "redactKeys": ["password", "api_key", "secret", "private_key"]
}
```

#### Evaluation Rules

- **Deny Precedence**: Rules in `deny` take absolute precedence over `allow` rules. Matching a `deny` pattern blocks execution immediately.
- **Default Permissiveness**: If `allow` is empty or omitted, all non-denied items are permitted.
- **Wildcard Syntax**: Supports prefix wildcards (e.g. `github.*`) and global wildcards (`*`).
- **Data Redaction**: Fields in request and response payloads matching keys in `redactKeys` are masked in logs and tracing spans before emission.

---

## 5. Execution Modes

### 5.1 HTTP Daemon Mode

Start the background daemon process:

```bash
warmplane daemon --config mcp_servers.json --port 9090
```

By default, the daemon binds to `127.0.0.1:<port>`.

### 5.2 Stdio MCP Server Mode

Run Warmplane as a stdio MCP server for native integration with desktop AI clients (e.g. Claude Desktop, VS Code extensions):

```bash
warmplane mcp-server --config mcp_servers.json
```

#### Client Configuration Example (`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "warmplane": {
      "command": "warmplane",
      "args": ["mcp-server", "--config", "/etc/warmplane/mcp_servers.json"]
    }
  }
}
```

---

## 6. HTTP API Specification

All HTTP API endpoints return standard JSON response envelopes.

### Summary of Endpoints

| Method | Path | Description | ETag Support |
| :--- | :--- | :--- | :---: |
| `GET` | `/v1/capabilities` | List compact capability index. | Yes |
| `POST` | `/v1/capabilities/search` | Search capabilities via hybrid lexical/semantic ranking. | Yes |
| `GET` | `/v1/capabilities/:id` | Fetch full input JSON Schema for a specific capability. | Yes |
| `POST` | `/v1/tools/call` | Execute an upstream tool call. | No |
| `GET` | `/v1/resources` | List compact resource index. | Yes |
| `POST` | `/v1/resources/read` | Read content of an upstream resource URI. | No |
| `GET` | `/v1/prompts` | List compact prompt index. | Yes |
| `POST` | `/v1/prompts/get` | Render an upstream prompt template. | No |
| `GET` | `/v1/catalog/events` | Read catalog mutation change feed with cursor pagination. | Yes |
| `POST` | `/v1/operations/:id/cancel` | Cancel an in-flight async operation. | No |

---

### 6.1 Catalog Caching (`If-None-Match`)

Catalog read endpoints (`/v1/capabilities`, `/v1/capabilities/:id`, `/v1/resources`, `/v1/prompts`) return an `ETag` header containing the SHA-256 catalog checksum.

Pass the returned `ETag` in subsequent requests using the `If-None-Match` header. If the catalog has not changed, Warmplane returns `HTTP 304 Not Modified` with zero response body bytes.

---

### 6.2 Capability Search (`POST /v1/capabilities/search`)

Perform hybrid search combining BM25 lexical ranking and optional FastEmbed vector semantic search.

#### Request Body

```json
{
  "query": "search production error logs",
  "limit": 5,
  "server_ids": ["observability"],
  "tags": ["logs"],
  "modes": ["read"]
}
```

#### Response Body

```json
{
  "version": "v1",
  "catalog_version": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
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

---

### 6.3 Idempotency and Deduplication

To prevent duplicate execution during network retries or concurrent agent calls, pass an `Idempotency-Key` header or `idempotency_key` field in the payload envelope.

```bash
curl -X POST http://127.0.0.1:9090/v1/tools/call \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: exec-tx-994812" \
  -d '{
    "capability_id": "payments.charge",
    "args": {"amount": 100, "currency": "USD"}
  }'
```

#### Behavior Semantics

1. **First Request**: Executes the operation and stores the response in the deduplication cache.
2. **In-Flight Duplicate**: Subscribes to the active execution and waits for completion.
3. **Completed Duplicate**: Immediately returns the cached response payload without re-executing upstream.

---

### 6.4 Response Envelope & Retry Metadata

Tool execution, resource reading, and prompt fetching return normalized execution envelopes:

```json
{
  "ok": true,
  "request_id": "req-994812",
  "trace_id": "8f2a1b3c4d5e6f7a",
  "data": {
    "result": "success"
  },
  "error": null,
  "retry": {
    "classification": "safe",
    "upstream_execution_state": "completed"
  }
}
```

#### Retry Classification Schema

- `classification`:
  - `"safe"`: Read-only operation. Safe to retry automatically.
  - `"idempotent"`: Mutating operation with explicit idempotency handling. Safe to retry with same key.
  - `"unsafe"`: Non-idempotent mutation. Requires caution before retrying.
- `upstream_execution_state`:
  - `"not_started"`: Upstream operation was not invoked.
  - `"completed"`: Upstream operation finished successfully.
  - `"unknown"`: Request timed out or failed mid-stream. Upstream state is unverified.

---

### 6.5 Standard Error Codes

When `ok` is `false`, the envelope provides a structured error object (`error.code` and `error.message`):

| Error Code | HTTP Status | Cause | Resolution |
| :--- | :---: | :--- | :--- |
| `TOOL_NOT_FOUND` | 404 | Capability ID does not exist or is blocked by policy. | Verify capability ID or policy configuration. |
| `RESOURCE_NOT_FOUND` | 404 | Resource ID or URI not found. | Check registered resources list. |
| `PROMPT_NOT_FOUND` | 404 | Prompt template ID not found. | Check registered prompts list. |
| `INVALID_ARGS` | 400 | Request body failed JSON schema validation. | Correct invalid argument fields. |
| `SERVER_UNREACHABLE` | 502 | Upstream server process crashed or network disconnected. | Inspect upstream process status and network. |
| `UPSTREAM_TIMEOUT` | 504 | Upstream operation exceeded `toolTimeoutMs`. | Increase timeout or optimize upstream query. |
| `UPSTREAM_ERROR` | 500 | Upstream server returned a protocol error. | Inspect upstream server logs. |
| `INTERNAL_ERROR` | 500 | Internal daemon process error. | Review Warmplane daemon logs. |

---

## 7. Enterprise Operations and Observability

### 7.1 Health Probes

Use the `/v1/capabilities` endpoint for HTTP readiness and liveness checks:

```bash
curl -sf http://127.0.0.1:9090/v1/capabilities >/dev/null || exit 1
```

### 7.2 OpenTelemetry (OTLP) Tracing

Warmplane includes native OpenTelemetry tracing via OTLP gRPC export.

#### Environment Variables

| Variable | Default | Description |
| :--- | :--- | :--- |
| `RUST_LOG` | `info,warmplane=debug` | Log level filter string. |
| `WARMPLANE_OTEL_ENABLED` | `false` | Enables OpenTelemetry trace export when set to `true`. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://127.0.0.1:4317` | OTLP gRPC collector endpoint. |
| `WARMPLANE_SERVICE_NAME` | `warmplane` | Service identifier tag in emitted trace spans. |

Example startup with OpenTelemetry enabled:

```bash
export WARMPLANE_OTEL_ENABLED=true
export OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector.internal:4317
export WARMPLANE_SERVICE_NAME=warmplane-production

warmplane daemon --config /etc/warmplane/mcp_servers.json
```

---

## 8. Development and Regression Testing

Validate local changes using cargo test suites and integration smoke tests.

### Run Unit and Integration Tests

```bash
cargo test
```

### Run MCP Stdio Smoke Test

```bash
./scripts/smoke_mcp_server.sh
```

---

## 9. Reference Documentation

- [HTTP API Specification (`docs/spec.md`)](spec.md)
- [OpenAPI 3.1 Definition (`docs/openapi.yaml`)](openapi.yaml)
- [Observability Guide (`docs/OBSERVABILITY.md`)](OBSERVABILITY.md)
- [Deployment Runbook (`docs/DEPLOYMENT.md`)](DEPLOYMENT.md)
