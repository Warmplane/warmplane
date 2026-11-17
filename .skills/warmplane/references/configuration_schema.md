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
        "preUndoCheck": {
          "capability": "linear.get_issue",
          "args": { "id": "${args.id}" },
          "captureJsonPath": "$.priority"
        },
        "testTarget": {
          "server": "linear-test",
          "args": { "id": "UNDO-TEST-1" }
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

Detailed capability aliases may set `reversibility.class` to `delayable`, `reversible`, `compensable`, or `irreversible`. A delayable call starts its cancellable hold after any required approval and uses approved arguments. Its duration resolves as profile policy, alias `holdWindowSecs`, global policy, then 30 seconds. Async calls return a task receipt; synchronous calls wait only when the hold is shorter than `toolTimeoutMs`. Task receipts expose `phase`, `releaseAtMs`, and `cancellable`; cancellation through task or operation endpoints succeeds only before durable dispatch claim. Pending holds resume after restart and current-policy revalidation. A task recovered in `dispatching` fails with `EMISSION_OUTCOME_UNKNOWN` and is never resent automatically. Batch steps await release within the shared 60-second batch budget.

Reversible and compensable contracts require an `inverse` capability and object-valued `args`; an optional `preSnapshot` performs a policy-checked upstream read before the forward call. `captureJsonPath` narrows the captured value. Template references support `${args.path}`, `${result.path}`, and `${snapshot.path}`; exact references preserve JSON values. Snapshots are content addressed in the Warmplane state directory, and inverse records are persisted in the undo ledger. Compensable contracts require a positive `dailyLimit`; the rolling limit is scoped by tenant, profile, and capability, and over-limit calls enter human approval.

Optional `preUndoCheck` captures the expected post-state hash and blocks inverse dispatch if the state has drifted. `warmplane undo test <alias>` and the Playground require `testTarget.server`, fixed `testTarget.args`, and `preSnapshot`; they run forward and inverse calls only against the dedicated test server and report whether the final state matches the original snapshot.
