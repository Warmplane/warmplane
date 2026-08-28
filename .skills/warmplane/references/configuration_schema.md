# Configuration Schema Reference (`mcp_servers.json`)

## Canonical Schema
```json
{
  "port": 9090,
  "mcpServers": {
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite"],
      "resilience": { "failureThreshold": 3, "cooldownMs": 30000, "autoRestart": true }
    },
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "PG_PASSWORD": "keychain://POSTGRES_PASSWORD",
        "API_TOKEN": "op://vault/service/token",
        "ENV_VAR": "env://HOST_VAR"
      }
    },
    "remote": {
      "url": "https://mcp.internal.net/sse",
      "auth": { "type": "bearer", "tokenEnv": "TOKEN_ENV" }
    }
  },
  "profiles": {
    "coding": {
      "servers": ["sqlite", "postgres"],
      "policy": {
        "allow": ["sqlite.query", "postgres.*"],
        "deny": ["sqlite.delete*"],
        "requireApproval": ["postgres.mutation*"],
        "redactKeys": ["password", "token"]
      }
    }
  },
  "capabilityAliases": { "query": "sqlite.query" },
  "policy": { "requireApproval": ["*.drop*"], "redactKeys": ["secret", "key"] },
  "chatops": {
    "webhooks": [{ "platform": "slack", "url": "https://hooks.slack.com/...", "signingSecret": "keychain://SLACK_SECRET" }]
  }
}
```

## Secret URIs
- `keychain://<KEY>`: Native OS Keychain
- `op://<vault>/<item>/<field>`: 1Password CLI
- `env://<VAR>`: Process environment variable
