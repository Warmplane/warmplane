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
- `GET /v1/tasks/{id}` / `POST /v1/tasks/{id}/input` (`{"input": "approved"}`)

## 4. Secrets Vault
- `GET /v1/secrets`
- `POST /v1/secrets` (`{"key": "DB_PASSWORD", "value": "secret"}`)
- `DELETE /v1/secrets/{key}`
