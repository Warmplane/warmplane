# CLI Command Cheatsheet

## 1. Daemon Lifecycle
- Start default: `warmplane daemon`
- Start custom: `warmplane daemon --config <path> --port <port>`
- Hot-reload running daemon: `warmplane reload`
- Validate config: `warmplane validate-config --config <path>`

## 2. Server Management
- Restart upstream: `warmplane server restart <server-id>`

## 3. Client Sync (17 Ecosystems)
- Scan: `warmplane client scan`
- Attach: `warmplane client attach <client-id> [--profile <name>]`
- Detach: `warmplane client detach <client-id>`

## 4. Secrets Vault
- Set: `warmplane secret set <KEY>`
- Get: `warmplane secret get <KEY>`
- List: `warmplane secret list`
- Delete: `warmplane secret delete <KEY>`

## 5. Standalone Proxy
- Stdio proxy: `warmplane mcp-server --config <path> [--profile <name>]`
- HTTP/SSE proxy: `warmplane mcp-http-server --config <path> --bind 127.0.0.1 --port 9191`

## 6. Undo Operations
- List: `warmplane undo list [--work-item-id <id>] [--status armed]`
- Show: `warmplane undo show <undo-id>`
- Execute: `warmplane undo run <undo-id>`
- Roll back a work item: `warmplane undo rollback <work-item-id>`
- Force past expiry only with the elevated `forceExpired` permission and `--reason <text>`: add `--force-expired --reason "incident reference"`.
- Supply daemon bearer credentials with `WARMPLANE_OPERATOR_TOKEN`; static `authToken` from the config is used when that environment variable is unset.
