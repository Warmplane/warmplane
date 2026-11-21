# We built an undo button for AI agents

An agent changes the wrong issue, moves the wrong file, or queues an email you wish you had reviewed. An approval prompt helps before execution. Afterward, you need a recovery path.

Quaspar's [Agents Need an Undo Button, Not More Approval Prompts](https://quaspar.com/resources/agents-need-an-undo-button) argues for designing oversight around whether an action can be reversed. That is a useful question to ask at the tool boundary: what would it take to undo this particular call, and when would that stop being possible?

We built support for that approach into Warmplane v0.33.0. Warmplane is a local control plane between AI clients and upstream Model Context Protocol (MCP) servers. It can hold configured sends before submission, capture pre-state for configured mutations, and give operators a way to invoke an inverse or roll back a work item.

The runtime owns those records and checks. An agent does not have to remember how to undo its work, and a human does not have to persuade the model that a correction is necessary. Existing policy and approval gates still apply.

## Put the recovery path in the runtime

An agent can make a mistake without recognizing it. It can also encounter malicious instructions in the material it reads. Asking it to maintain its own undo plan leaves the recovery path dependent on the same conversation that produced the action.

Warmplane records recovery information outside that conversation. For an alias with a configured contract, it can:

- Run a configured snapshot read before the forward mutation.
- Register an inverse capability and its resolved arguments after a successful call.
- Expose operator controls through the Control Deck, CLI, and REST API.

These are opt-in contracts. Warmplane does not infer an inverse for an arbitrary MCP tool. Operators must define the contract, test its assumptions, and ensure that relevant calls pass through Warmplane.

## Four classes of reversibility

A cancellation window for an email and an inverse for a database update solve different problems. Warmplane supports four classes on detailed `capabilityAliases` in `mcp_servers.json`.

The tool names, arguments, and response fields below are illustrative. Adapt them to the schemas exposed by your upstream MCP servers. Merge this excerpt into a configuration containing those servers; the `linear-test` server must be a dedicated test environment.

```json
{
  "capabilityAliases": {
    "tickets.set_priority": {
      "target": "linear.update_issue",
      "summary": "Update issue priority",
      "reversibility": {
        "class": "reversible",
        "ttlSecs": 3600,
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
          "args": { "id": "UNDO-TEST-1", "priority": "high" }
        },
        "inverse": {
          "capability": "linear.update_issue",
          "args": {
            "id": "${args.id}",
            "priority": "${snapshot}"
          }
        }
      }
    },
    "billing.charge_customer": {
      "target": "stripe.create_charge",
      "reversibility": {
        "class": "compensable",
        "ttlSecs": 86400,
        "dailyLimit": 5,
        "inverse": {
          "capability": "stripe.create_refund",
          "args": { "charge": "${result.id}" }
        }
      }
    },
    "mail.send": {
      "target": "gmail.send_message",
      "reversibility": {
        "class": "delayable",
        "holdWindowSecs": 30
      }
    },
    "database.drop_table": {
      "target": "db.drop_table",
      "reversibility": {
        "class": "irreversible"
      }
    }
  },
  "policy": {
    "requireApproval": ["database.drop_table"]
  }
}
```

### Reversible: capture state and configure an inverse

For `tickets.set_priority`, Warmplane reads the issue before the mutation and captures `$.priority`. It stores the snapshot by content hash, then uses `${snapshot}` to resolve the inverse arguments after the forward call succeeds. The resulting undo entry has a one-hour window in this example. Snapshot files use private permissions on Unix.

A failed pre-snapshot prevents the forward mutation. There is a separate failure boundary after execution: if inverse registration or the configured post-state capture fails, the forward effect may already have happened. Warmplane reports the registration failure; an operator must reconcile the upstream state rather than assume an undo entry is available.

The example also configures `preUndoCheck`. Warmplane records a hash of the observed post-execution state and compares it with a fresh read before undo. A mismatch blocks the inverse and leaves the entry armed for review. This checks only the configured observation, and it is not an atomic compare-and-swap with the external system. An upstream can still change between the check and the inverse call.

Snapshot and drift-check reads must be available in the active profile and permitted without approval by the effective policy.

### Compensable: correct an effect that cannot be erased

A refund compensates for a charge, but does not erase the original transaction or every consequence it caused. For `billing.charge_customer`, the inverse template takes the charge identifier from `${result.id}` and supplies it to the configured refund tool.

`dailyLimit` tracks invocations over a rolling 24-hour period per tenant, profile, and capability. With a limit of five, calls beyond the available budget go through human approval. Calls within the budget still obey any existing approval rules. The cap is an invocation count, not a monetary spending limit.

### Delayable: cancel before submission

For `mail.send`, Warmplane holds the forward payload before submitting it upstream. Any required approval completes first. During the hold, an operator can cancel the send from the UI or API.

The hold duration resolves from profile policy, then the alias, then global policy, then a 30-second default. The alias in this example requests 30 seconds. Asynchronous calls return a task receipt. Synchronous calls wait when the hold is shorter than the tool timeout; longer holds return a receipt instead. Batch steps wait for the held result within the batch execution budget.

Cancellation succeeds only before dispatch is claimed. Once submission begins, Warmplane cannot promise to stop it or retrieve a delivered message. With task persistence enabled, pending holds can recover after restart and revalidate authorization before release. An interrupted dispatch is treated as an unknown outcome and is not automatically resent.

### Irreversible: configure the approval boundary explicitly

The `irreversible` class declares that a tool has no configured recovery path. It does not automatically require approval. The example separately places `database.drop_table` in `policy.requireApproval`.

An undo contract does not remove an approval gate from another class, either. Decide which calls need approval and which have a tested recovery path as separate parts of the policy.

## Roll back a work item

A workflow can succeed several times before a later step fails. Warmplane can retain the inverses for those successful configured calls and execute eligible entries newest first. Equal timestamps are ordered by reverse persisted registration order.

For calls you want to group into a work item, supply the same `work_item_id` in their request context. An operator can then run:

```bash
warmplane undo rollback <work-item-id>
```

A `POST /v1/tools/batch_call` request has a separate batch identity derived from its request ID. Setting `autoRollbackOnFailure` to `true` stops at the first failed step and attempts registered inverses for earlier successful steps in reverse order. It does not require a work-item ID. Work-item grouping and batch rollback are distinct scopes.

Each inverse passes the current authorization, policy, profile, and expiry checks. Automatic rollback cannot run an inverse that requires approval; blocked and failed entries are reported individually. Ledger records remain available after execution so operators can inspect what succeeded and what did not.

Warmplane derives a deterministic idempotency key for each inverse to support deduplication. That does not guarantee exactly-once side effects in every upstream system. If a process stops while an inverse is executing, its outcome may be unknown. Warmplane does not automatically redispatch that entry; inspect upstream state and the audit trail before reconciling it.

This has a connection to the runtime-owned inverses discussed in Shi, Zhang, and Cui's [A Programming Paradigm for Spatiotemporal Composability](https://arxiv.org/abs/2608.25512). External MCP calls add their own constraints: an inverse is another tool call, compensation can fail, and a remote side effect may be impossible to restore exactly. A work-item rollback is not a distributed transaction.

## Operator controls without another model turn

The default MCP tool catalog does not expose `undo_execute`. Operators can use the recovery controls independently of the model:

- **Action Feed & Undo in the Control Deck:** inspect undo history and held sends, view capability and actor/profile provenance, check remaining windows, undo an action, or roll back a work item. Cards show safe inverse field names; they omit snapshot values and inverse arguments. SSE invalidations trigger REST refreshes of the feed.
- **ChatOps:** opt into `undo.action_available` or `emission.held` webhook events and configure the signing secret, callback URL, and action role. Slack, Discord, and Teams cards can then carry signed Undo Action or Cancel Send buttons. The callback acknowledges the request and runs it in the background; inspect the Action Feed for the outcome.
- **CLI, REST, and embedded Rust:** inspect entries with `warmplane undo list` and `warmplane undo show <undo-id>`, execute one with `warmplane undo run <undo-id>`, or use the corresponding API operations. Embedded applications can use `ControlPlaneHandle` directly.

MCP agent self-undo is available as an explicit opt-in. With an active profile, both that profile and the caller's RBAC role must enable `allowAgentSelfUndo`. The runtime still owns the ledger and enforces access checks; exposing the tool does not give the agent arbitrary inverse arguments or snapshot contents.

Held-send transitions and undo outcomes are recorded in the hash-chained audit trail. HMAC protection is optional, and durable audit storage depends on configuration. Snapshots live in a separate content-addressed store, with hashes linking them to recovery records. These records help inspect execution and detect recorded-history changes; they do not prove every consequence in an external system.

## Test the recovery path and understand its limits

In Facade & Alias Studio, configure the contract and use **Test saved contract**, or run:

```bash
warmplane undo test tickets.set_priority
```

This performs real forward and inverse calls against the configured test server. It requires `testTarget.server`, fixed `testTarget.args`, and a `preSnapshot` read. The test server must expose the corresponding tool names and be isolated from production.

Warmplane compares the final observed state with the initial snapshot. A passing test is evidence that the configured observation was restored for those inputs. It does not prove equivalence across every external effect or every possible argument.

Information disclosure remains outside undo's reach. Deleting a reply does not make its recipient forget a private key. Warmplane's policy and profile controls restrict tool access, while `redactKeys` and credential sanitization mask supported fields on specific logging, response, and configuration surfaces. They are not a general secret detector or an outbound content filter.

## Get started with a shared runtime

These examples target Warmplane v0.33.0 or later. Install a suitable [release](https://github.com/Warmplane/warmplane/releases), or build from a checkout containing undo support:

```bash
git clone https://github.com/Warmplane/warmplane.git
cd warmplane
cargo install --path . --locked
warmplane --version

# Import upstream definitions from an existing client.
warmplane config import
```

Before starting the daemon:

1. Configure the required upstream servers and capability contracts. You can edit contracts in **Facade & Alias Studio** after startup as well.
2. Configure persistent state if held sends must survive restart. Reversible and compensable execution requires the persistent undo ledger; pre-state capture requires snapshot storage.
3. Add the following block to the same `mcp_servers.json` used by the daemon:

   ```json
   "mcpHttpServer": {
     "port": 9191
   }
   ```

Start the daemon and open the Control Deck at `http://localhost:9090/ui`:

```bash
warmplane daemon --config mcp_servers.json --port 9090
```

Connect the agent to the daemon's MCP endpoint at `http://127.0.0.1:9191/mcp`, using the client's HTTP transport configuration. Importing upstream definitions does not make that connection for you. A separately launched `warmplane mcp-server` process owns a separate runtime; sharing a state directory does not synchronize its live state with the operator UI.

Use the [undo configuration and operator guide](../USER-GUIDE.md#evaluation-rules) for permissions, policy precedence, persistence, and contract testing.

A recovery contract should say whether an action can be held, reversed, or compensated, under what conditions, and who can trigger it. Warmplane gives operators a place to define those rules and act on them without another model turn.
