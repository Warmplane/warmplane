# HTTP REST API Reference

Daemon default: `http://127.0.0.1:9090`

## 1. Catalog & Execution
- `GET /v1/capabilities` (params: `?profile={name}`; supports `If-None-Match` ETag)
- `POST /v1/capabilities/search` (`{"query": "text", "limit": 8}`)
- `POST /v1/execute` (`{"capability_id": "sqlite.query", "args": {...}, "async_task": false}`)
- `GET /v1/resources` / `POST /v1/resources/read` (`{"uri": "file:///path"}`)
- `GET /v1/prompts` / `POST /v1/prompts/get` (`{"name": "review", "arguments": {...}}`)

## 2. Server & Profile Management
- `POST /v1/config/reload` (Hot-reloads from disk)
- `POST /v1/config/servers` (`{"name": "sqlite", "server": {"command": "npx", "args": [...]}}`)
- `DELETE /v1/config/servers/{id}` (Cascades profiles and aliases)
- `POST /v1/config/servers/{id}/restart` (Resets circuit breaker and restarts process)
- `POST /v1/config/profiles` (`{"name": "coding", "servers": ["sqlite"], "policy": {...}}`)
- `DELETE /v1/config/profiles/{id}`

## 3. Human-in-the-Loop & Tasks
- `GET /v1/approvals` / `POST /v1/approvals/{id}/approve` / `POST /v1/approvals/{id}/reject`
- `GET /v1/tasks` / `GET /v1/tasks/{id}` / `POST /v1/tasks/{id}/update` / `POST /v1/tasks/{id}/cancel`
- Delayable calls may return a task receipt containing `phase`, `releaseAtMs`, and `cancellable`. Cancel with the task endpoint above or `POST /v1/operations/{request_id}/cancel`. Cancellation after dispatch claim returns `409 EMISSION_DISPATCHING`.
- Hold duration precedence is profile policy, alias `holdWindowSecs`, global policy, then 30 seconds. Required approval resolves first. Pending holds resume after restart and current-policy revalidation; recovered dispatching tasks fail with `EMISSION_OUTCOME_UNKNOWN` and are never resent.

## 4. Secrets Vault
- `GET /v1/secrets`
- `POST /v1/secrets` (`{"key": "DB_PASSWORD", "value": "secret"}`)
- `DELETE /v1/secrets/{key}`

## 5. Undo Operations
- `GET /v1/undo` lists safe summaries for the authenticated tenant; optional query filters: `work_item_id`, `status`, `offset`, and `limit`.
- `GET /v1/undo/{undo_id}` shows one safe summary. Cross-tenant IDs are concealed with `404`.
- `POST /v1/undo/{undo_id}/execute` executes one inverse.
- `POST /v1/undo/work-items/{work_item_id}/rollback` executes eligible inverses in reverse order.
- Summaries never include inverse arguments or snapshot contents. With RBAC enabled, roles need `undoPermissions.list`, `.execute`, or `.rollback`; forcing expiry always requires the caller role's dedicated `.forceExpired` permission and JSON `{"forceExpired":true,"reason":"..."}`. The reason is included in the committed audit record.
