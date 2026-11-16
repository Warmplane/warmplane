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
        "holdWindowSecs": 15,
        "redactKeys": ["password", "token"]
      }
    }
  },
  "capabilityAliases": {
    "query": "sqlite.query",
    "set-priority": {
      "target": "linear.update_issue",
      "reversibility": {
        "class": "reversible",
        "ttlSecs": 86400,
        "preSnapshot": {
          "capability": "linear.get_issue",
          "args": { "id": "${args.id}" },
          "captureJsonPath": "$.priority"
        },
        "inverse": {
          "capability": "linear.update_issue",
          "args": { "id": "${args.id}", "priority": "${snapshot}" }
        }
      }
    },
    "search": {
      "target": "semble-rs.search",
      "summary": "Search codebase with semantic or BM25 ranking. Pass absolute repo path."
    }
  },
  "policy": { "requireApproval": ["*.drop*"], "holdWindowSecs": 30, "redactKeys": ["secret", "key"] },
  "chatops": {
    "webhooks": [{ "platform": "slack", "url": "https://hooks.slack.com/...", "signingSecret": "keychain://SLACK_SECRET" }]
  }
}
```

## Secret URIs
- `keychain://<KEY>`: Native OS Keychain
- `op://<vault>/<item>/<field>`: 1Password CLI
- `env://<VAR>`: Process environment variable

## Reversibility Contracts

Detailed capability aliases may set `reversibility.class` to `delayable`, `reversible`, `compensable`, or `irreversible`. Reversible and compensable contracts require an `inverse` capability and object-valued `args`; an optional `preSnapshot` performs a policy-checked upstream read before the forward call. `captureJsonPath` narrows the captured value. Template references support `${args.path}`, `${result.path}`, and `${snapshot.path}`; exact references preserve JSON values. Snapshots are content addressed in the Warmplane state directory, and inverse records are persisted in the undo ledger. Compensable contracts require a positive `dailyLimit`; the rolling limit is scoped by tenant, profile, and capability, and over-limit calls enter human approval.
