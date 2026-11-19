# Native MCP Client Integration Reference

> **Note on Process Execution**: When configuring `warmplane mcp-server` via `command`, your client launches Warmplane directly as an independent child process over standard I/O (`stdin`/`stdout`). You do **not** need `warmplane daemon` running in the background.
> Alternatively, if you run `warmplane daemon` 24/7 with `mcpHttpServer` enabled, you can connect your client to the daemon's Streamable HTTP/SSE endpoint (`http://127.0.0.1:9191/mcp`).

## 1. Client Configurations (Stdio Child Process)

Configure Warmplane in your client configuration file:

- **Standard Clients (Cursor, Claude Code, Antigravity, LibreChat, VS Code, DeepSeek)**:
  `~/.cursor/mcp.json` / `~/.claude.json` / `~/.gemini/config/mcp_config.json`:
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

- **OpenCode**: `~/.config/opencode/opencode.json`:
  ```json
  {
    "mcp": {
      "warmplane": {
        "type": "local",
        "command": "warmplane",
        "args": ["mcp-server", "--config", "mcp_servers.json"],
        "enabled": true
      }
    }
  }
  ```

- **Zed**: `~/.config/zed/settings.json`:
  ```json
  {
    "context_servers": {
      "warmplane": {
        "command": { "path": "warmplane", "args": ["mcp-server", "--config", "mcp_servers.json"] }
      }
    }
  }
  ```

## 2. 1-Click Sync CLI
```bash
warmplane client scan
warmplane client attach claude-desktop --profile coding
warmplane client attach cursor
warmplane client detach claude-desktop
```

## 3. Protocol Operations

### 3.1 Calling Upstream Tools
Execute namespaced tool `<server>.<tool>` via standard MCP `tools/call`:
```json
{
  "name": "sqlite.query",
  "arguments": { "query": "SELECT * FROM users LIMIT 5;" }
}
```

### 3.2 Built-in MCP Discovery & Control Tools
- `capability_search`: `{"query": "database", "limit": 5}`
- `capability_describe`: `{"id": "postgres.query"}`
- `capabilities_batch_call`: Sequential execution steps with output reference interpolation
- `task_get` / `task_update` / `task_cancel`: Inspect and resume HITL tasks


## Undo and operator controls

Capability aliases apply their configured reversibility contracts to MCP tool calls. `capabilities_batch_call` accepts the optional boolean `autoRollbackOnFailure` over both stdio and HTTP MCP. It defaults to `false`; other types, including `null`, are rejected before execution. When enabled, the batch stops at its first failure even if a step has `continue_on_error: true`, then attempts registered inverses for earlier successful steps in reverse order. The response retains the original forward failure (`ok: false`) and reports inverse outcomes in `rollback_results`. Current policy, profile, and approval checks can block inverses, so rollback is not an atomic transaction.

For example, call the MCP tool with these arguments, replacing capability IDs and arguments with configured aliases:

```json
{
  "request_id": "priority-batch-42",
  "autoRollbackOnFailure": true,
  "steps": [
    {
      "id": "first",
      "capability_id": "tickets.set_priority",
      "args": {"id": "ISSUE-1", "priority": "high"}
    },
    {
      "id": "second",
      "capability_id": "tickets.set_priority",
      "args": {"id": "ISSUE-2", "priority": "high"}
    }
  ]
}
```

The batch identity is `request_id`, or the generated trace ID if omitted. A `context.work_item_id` is not required. The option applies only to this batch request; separate tool calls inside a Claude Code Mode script are not automatically grouped or rolled back. Automatic batch rollback does not require enabling the separate agent self-undo tool.

The `undo_execute` tool is hidden by default. With RBAC enabled, its role must opt in using `allowAgentSelfUndo`; an active profile must also opt in. It accepts either `undo_id` or `work_item_id` to undo the newest armed entry.

For a shared Action Feed and operator REST/CLI controls, connect agents to the daemon's configured MCP HTTP `/mcp` endpoint. A standalone stdio process owns a separate in-memory ledger and event bus. Running separate processes against the same state files does not synchronize their live state.
