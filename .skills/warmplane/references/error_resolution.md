# Troubleshooting & Error Resolution

Quick diagnostic playbook for resolving runtime conditions.

## 1. Diagnostics & Remediation

- `503 Service Unavailable` / `Circuit: OPEN`
  - Cause: Upstream MCP process crashed repeatedly.
  - Action: Run `warmplane server restart <server-id>` or `POST /v1/config/servers/{id}/restart`.

- `403 Forbidden` / `POLICY_DENIED`
  - Cause: Blocked by global or profile `policy.deny` or omitted from `policy.allow`.
  - Action: Check profile policy in `mcp_servers.json` or switch profile.

- `202 Accepted` / `input_required`
  - Cause: Matched Human-in-the-Loop (`requireApproval`) policy rule.
  - Action: Poll `GET /v1/tasks/{task_id}` and submit approval via `POST /v1/tasks/{task_id}/input`.

- `304 Not Modified` on `/v1/capabilities`
  - Cause: ETag match.
  - Action: Reuse cached capability index without re-parsing.

- `400 Bad Request: Profile references unknown server`
  - Cause: Profile references a server ID absent from `mcpServers`.
  - Action: Add server to `mcpServers` or remove reference from `config.profiles`.

- Stdio Timeout / No Tools Visible
  - Cause: Non-existent binary or path.
  - Action: Run `warmplane client scan` and `warmplane validate-config --config <path>`.

## 2. Crash Recovery
1. View diagnostics: `GET /v1/config`
2. Restart upstream: `warmplane server restart <server-id>`
3. State resets failure counters and restores `Circuit: CLOSED`.
