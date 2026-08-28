# Native MCP Stdio Usage Reference

## 1. Client Configurations

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
