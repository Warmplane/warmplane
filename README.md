# Warmplane

`Warmplane` is the local control plane that keeps MCP sessions warm.

It runs multiple upstream MCP servers behind one local process, keeps those sessions persistent, and exposes a compact interaction surface for tools/resources/prompts. The goal is concrete and measurable: reduce startup latency, reduce payload size, and keep behavior deterministic.

## Why This Exists

Most agent stacks overpay in tokens and latency by eagerly surfacing large tool catalogs and detailed schemas that are never used.

`Warmplane` shifts that model to lazy, compact interaction:

- Discover compact indexes first.
- Fetch detail only when needed.
- Execute through normalized envelopes.

This improves:

- token efficiency
- time-to-first-useful-tool-call
- cross-client consistency
- observability and policy control

## What It Provides

One runtime, three access modes:

1. HTTP facade (`/v1/...`)
2. CLI facade commands
3. MCP facade server mode (`mcp-server`) for MCP-native clients

All three modes share the same backend state, aliases, policy checks, and timeout behavior.

## Core Facade Surface

### Capabilities
- list: compact capability index
- search: hybrid lexical + semantic capability search with filters
- describe: on-demand detail for one capability
- call: normalized execution envelope

### Resources
- list: compact resource index
- read: normalized read envelope

### Prompts
- list: compact prompt index
- get: normalized prompt rendering envelope

## Install

```bash
# Standard build (hybrid search with BM25 lexical engine)
cargo install --path .

# Build with local ONNX vector embeddings support (FastEmbed)
cargo install --path . --features semantic-search
```

`cargo install warmplane` is not available yet because the crate has not been published to crates.io.

## Validate Config

Validate and lint configuration before startup:

```bash
warmplane validate-config --config mcp_servers.json
```

Example success output:

```json
{"ok":true,"config":"mcp_servers.json","servers":3}
```

## Configuration

Create `mcp_servers.json`:

```json
{
  "port": 9090,
  "toolTimeoutMs": 15000,
  "capabilityAliases": {
    "sqlite.read_query": "db.query"
  },
  "resourceAliases": {
    "filesystem.file:///tmp/readme.txt": "fs.readme"
  },
  "promptAliases": {
    "github.code_review": "prompt.code-review"
  },
  "policy": {
    "allow": ["db.*", "fs.*", "prompt.*"],
    "deny": ["fs.secret"],
    "redactKeys": ["token", "api_key", "password"]
  },
  "mcpServers": {
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite", "./test.db"]
    },
    "remote_docs": {
      "url": "https://mcp.example.com/mcp",
      "protocolVersion": "2025-11-25",
      "allowStateless": false,
      "headers": {
        "X-Tenant": "acme"
      },
      "auth": {
        "type": "bearer",
        "tokenEnv": "REMOTE_DOCS_MCP_TOKEN"
      }
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

Per `mcpServers.<id>`, transport selection is strict and inferred:

- stdio upstream: set `command` (plus optional `args`, `env`)
- HTTP/SSE upstream: set `url` (plus optional `protocolVersion`, `allowStateless`, `headers`, `auth`)
- exactly one of `command` or `url` must be set

No legacy config fallback is supported.

For bearer/basic, exactly one direct secret (`token`/`password`) or env-backed secret (`tokenEnv`/`passwordEnv`) is required.

`auth.type = "oauth2"`:

```json
{
  "type": "oauth2",
  "clientId": "client-id-here",
  "authorizationServerUrl": "https://auth.example.com",
  "scopes": ["read", "write"],
  "clientMetadataUrl": "https://example.com/metadata.json"
}
```

For OAuth2 configurations:
- Both `clientId` and `authorizationServerUrl` are required.
- Warmplane spins up a local background proxy that handles server discovery (RFC 9728 & RFC 8414), PKCE code exchanges, callback redirect handling with issuer validation (RFC 9207 / SEP-2468), scope accumulation on step-ups (`403 insufficient_scope`), and silent access token refreshes.

## Run Modes

### 1) HTTP Daemon

```bash
warmplane daemon --config mcp_servers.json
```

Endpoints:

- `GET /v1/capabilities`
- `POST /v1/capabilities/search`
- `GET /v1/capabilities/:id`
- `POST /v1/tools/call`
- `GET /v1/resources`
- `POST /v1/resources/read`
- `GET /v1/prompts`
- `POST /v1/prompts/get`

### 2) MCP Server (stdio)

```bash
warmplane mcp-server --config mcp_servers.json
```

MCP clients can point directly to this process.

Synthetic lightweight tools exposed:

- `capabilities_list`
- `capability_describe`
- `capability_call`
- `resources_list`
- `resource_read`
- `prompts_list`
- `prompt_get`

Native MCP methods also supported:

- resources: `resources/list`, `resources/read`
- prompts: `prompts/list`, `prompts/get`

### 3) CLI Facade

```bash
# capabilities
warmplane list-capabilities
warmplane search-capabilities "triage logs" --limit 5
warmplane describe-capability db.query
warmplane call-capability db.query --params '{"query":"SELECT 1"}'

# resources
warmplane list-resources
warmplane read-resource fs.readme

# prompts
warmplane list-prompts
warmplane get-prompt prompt.code-review --arguments '{"code":"fn main() {}"}'

# catalog change events feed
warmplane list-catalog-events [--after evt_1]
```

## MCP Client Example

```json
{
  "mcpServers": {
    "fast-facade": {
      "command": "warmplane",
      "args": ["mcp-server", "--config", "mcp_servers.json"]
    }
  }
}
```

## Smoke Test

Run an end-to-end stdio MCP smoke test:

```bash
./scripts/smoke_mcp_server.sh
```

It validates:

- MCP `initialize`
- `tools/list` includes all synthetic lightweight facade tools
- `resources/list` and `prompts/list` return valid responses

## Design Notes

- Upstream MCP compatibility remains intact.
- Client-facing schemas are intentionally small and stable.
- Policy and aliasing are enforced consistently across modes.
- Timeout and error envelopes are normalized for deterministic orchestration.
- Runtime logs are structured JSON for auditability.
- OpenTelemetry trace export is supported via OTLP.

## Observability

Warmplane emits structured JSON logs by default (`tracing` + `tracing-subscriber`).

Example controls:

- `RUST_LOG=info,warmplane=debug` set verbosity
- `WARMPLANE_OTEL_ENABLED=true` enable OpenTelemetry export
- `OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317` set OTLP collector endpoint
- `WARMPLANE_OTEL_ENDPOINT=http://127.0.0.1:4317` fallback OTLP endpoint if `OTEL_EXPORTER_OTLP_ENDPOINT` is unset
- `WARMPLANE_SERVICE_NAME=warmplane-prod` override service name

Operational notes:

- Logs include structured request/capability/resource/prompt fields for audit trails.
- `trace_id` in execution envelopes can be correlated with logs and distributed traces.
- When OTEL is enabled, traces are exported via OTLP gRPC and local structured logs remain active.

For detailed request/response contracts, see [docs/spec.md](docs/spec.md).

Additional references:

- OpenAPI: [openapi.yaml](docs/openapi.yaml)
- Config schema: [config.schema.json](docs/config.schema.json)
- Install/distribution: [INSTALL.md](docs/INSTALL.md)
- Deployment runbook: [DEPLOYMENT.md](docs/DEPLOYMENT.md)
- Observability: [OBSERVABILITY.md](docs/OBSERVABILITY.md)
