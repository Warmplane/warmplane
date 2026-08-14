# Warmplane API Spec

## HTTP Facade API (`/v1`)

Base:

- Daemon binds to `127.0.0.1:<port>`.
- Default port: `9090`.

Endpoints:

- `GET /v1/capabilities`: List compact capabilities index (returns `ttl_ms` and `cache_scope` hints)
- `POST /v1/capabilities/search`: Hybrid search over capabilities
- `GET /v1/capabilities/:id`: Full capability JSON schema definition
- `POST /v1/tools/call`: Invoke tool execution (supports MRTR `input_responses` and `request_state`)
- `GET /v1/resources`: List compact resource index (returns `ttl_ms` and `cache_scope` hints)
- `POST /v1/resources/read`: Read resource contents (supports MRTR)
- `GET /v1/prompts`: List compact prompt index (returns `ttl_ms` and `cache_scope` hints)
- `POST /v1/prompts/get`: Render prompt template (supports MRTR)
- `GET /v1/catalog/events`: Change feed with cursor pagination
- `GET /v1/resources/updates`: SSE stream for real-time resource updates
- `POST /v1/completion/complete`: Prompt/resource argument autocompletion
- `POST /v1/sampling/create_message`: Sampling delegation
- `POST /v1/operations/:id/cancel`: Cancel active in-flight operation

Response semantics:

- list endpoints return `{ "version": "v1", "ttl_ms": 300000, "cache_scope": "public", ... }` payloads
- execution/read/get endpoints return normalized envelope:
  - `ok`, `request_id`, `trace_id`, `context`, `retry`, `data`, `error`

Error codes:

- `TOOL_NOT_FOUND`
- `RESOURCE_NOT_FOUND`
- `PROMPT_NOT_FOUND`
- `SERVER_UNREACHABLE`
- `INVALID_ARGS`
- `UPSTREAM_TIMEOUT`
- `UPSTREAM_ERROR`
- `OPERATION_CANCELLED`
- `INTERNAL_ERROR`

## MCP Server Facade (stdio)

Run:

```bash
warmplane mcp-server --config mcp_servers.json
```

Exposed MCP tools (lightweight schemas):

- `capabilities_list`
- `capability_describe`
- `capability_call`
- `resources_list`
- `resource_read`
- `prompts_list`
- `prompt_get`
- `completion_complete`
- `subscriptions_listen`

These tools return the same lightweight JSON payloads used by CLI/HTTP facade semantics.

Also exposed natively via MCP methods:

- discovery: `server/discover`
- resources: `resources/list`, `resources/read`
- prompts: `prompts/list`, `prompts/get`

## Policy + Aliases

Configured in `mcp_servers.json`:

- `capabilityAliases`: `<server>.<tool>` -> capability ID
- `resourceAliases`: `<server>.<resource-uri>` -> resource ID
- `promptAliases`: `<server>.<prompt-name>` -> prompt ID
- `policy.allow` / `policy.deny`: ID pattern gates across tools/resources/prompts
- `policy.redactKeys`: redaction keys for logged payloads

## Upstream Transport Config

`mcpServers.<id>` must define exactly one transport selector:

- `command`: stdio transport
- `url`: streamable HTTP transport (JSON + SSE)

If both or neither are present, startup fails with config validation errors.

### stdio fields

- `command` (required for stdio)
- `args` (optional)
- `env` (optional)

### HTTP/SSE fields

- `url` (required for HTTP/SSE)
- `protocolVersion` (optional, default `"2026-07-28"`)
- `allowStateless` (optional; defaults to rmcp transport behavior)
- `headers` (optional map of static request headers)
- `auth` (optional):
  - bearer: `{ "type": "bearer", "token" | "tokenEnv" }`
  - basic: `{ "type": "basic", "username", "password" | "passwordEnv" }`
  - oauth2: `{ "type": "oauth2", "clientId", "authorizationServerUrl", "scopes", "clientMetadataUrl" }`

For bearer/basic auth, exactly one secret source must be set.
