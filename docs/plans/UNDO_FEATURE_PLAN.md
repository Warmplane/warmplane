## Architectural Thesis & Staged Implementation Proposal: Externalized Reversibility and Temporal Composability in Warmplane

**Target Codebase:** `Warmplane/warmplane` (v0.32.0+)
**Objective:** Replace high-friction Human-in-the-Loop (HITL) approval prompts for non-catastrophic mutations with an out-of-band, runtime-enforced Undo and Hold-Window architecture.

## 1. Thesis Statement & Theoretical Foundation

Traditional agent governance relies on blocking approval gates (`requireApproval`) that pause execution before every state-mutating tool call. As agent autonomy scales, approval prompts induce severe reviewer fatigue, serialize fast workflows behind human response latency, and fail to remedy mistakes that were accidentally approved. Conversely, giving an LLM an `undo` tool inside its own context window is an anti-pattern: an agent that misread a situation or suffered a prompt injection cannot be trusted to recognize its error or execute its own cleanup.

Safety and velocity can be reconciled by shifting governance from pre-execution permission to post-execution reversibility enforced strictly outside the agent's context window. By synthesizing three foundational paradigms directly into Warmplane's Rust control plane, we can make recovery a deterministic property of the proxy harness rather than a prompt instruction:

* **Revertible Effects & Temporal Composability:** Every mutating context transformation $\Gamma \to \Gamma \times (\Gamma \to \Gamma)$ is paired at birth with an idempotent inverse closure accumulated on a Last-In, First-Out (LIFO) stack owned by the runtime (`work_item_id` fiber), while pure reads execute free of accumulator overhead.
* **The Four-Class Reversibility Taxonomy & Decay:** Every capability is classified before invocation as `reversible` (exact inverse exists), `compensable` (mitigated via a follow-up action like a refund, subject to daily/window caps), `delayable` (withheld in a pre-emission queue before crossing the external system boundary), or `irreversible` (requires a blocking HITL gate), with explicit Time-To-Live (TTL) expiration rules reflecting that reversibility decays as downstream observers read the mutated state.
* **Evidentiary Composability & Human-Readable Capsules:** Every forward mutation, pre-state snapshot, and inverse execution is recorded in a tamper-evident, SHA-256 hash-chained ledger over content-addressed artifacts, surfaced to operators via a structured, out-of-band Action Capsule feed rather than buried in ephemeral chat transcripts.



## 2. Architectural Mapping to Warmplane (v0.32.0 Baseline)

Warmplane already mediates every MCP tool call across stdio, Streamable HTTP/SSE, REST, and embedded Rust consumers via `ControlPlaneHandle`. Implementing externalized undo requires extending five existing subsystems without breaking MCP protocol compatibility:

| Warmplane Subsystem | Current v0.32.0 Capability | Reversibility Extension |
| --- | --- | --- |
| **Config & Aliases** | Polymorphic `AliasTarget` overrides tool summaries, signatures, and retry classes | Add `ReversibilityContract` defining class, inverse target, interpolation map, pre-snapshot hook, TTL, and caps |
| **Persistent State** | Crash-resilient `AtomicFile<T>` (`0o600`) for `approvals.json`, `tasks.json`, and `idempotency.json` | Add `undo_ledger.json` and a content-addressed snapshot blob store (`art_<sha256>`) in the `0o700` state directory |
| **Async Tasks** | SEP-2663 task state machine with `TaskWaitSender` and TTL expiration | Add `PendingEmission` hold-window state for `delayable` tools with cooperative cancel before upstream dispatch |
| **WORM Audit Store** | Monotonic SHA-256 hash-chained `AuditEventType` log with HMAC signing | Add `EmissionHeld`, `EmissionCancelled`, `UndoRegistered`, `UndoExecuted`, and `UndoFailed` event types |
| **Operator Surfaces** | REST `/v1/tasks`, CLI `warmplane tasks`, Control Deck `/ui`, and HMAC ChatOps cards | Add `/v1/undo/*` REST routes, `warmplane undo` CLI, Control Deck Action Feed drawer, and post-action Undo webhooks

 |

## 3. Core Data Models & Schema Specifications

### 3.1. Declarative Reversibility Contract (`mcp_servers.json`)

Extend `AliasTarget` in `src/config.rs` and `docs/config.schema.json` to accept an optional `reversibility` block. Ensure `McpConfig::sanitize_secrets` redacts any sensitive static headers or tokens inside the block.

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
        "inverse": {
          "capability": "linear.update_issue",
          "args": {
            "id": "${args.id}",
            "priority": "${snapshot}"
          }
        }
      }
    },
    "billing.refund_customer": {
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
    }
  }
}

```

### 3.2. Rust Domain Types (`src/undo/types.rs`)

* **`ReversibilityClass`**: Enum of `Reversible`, `Compensable`, `Delayable`, and `Irreversible` (defaulting to standard policy evaluation when omitted).


* **`UndoEntry`**: Struct persisted in `AtomicFile<UndoLedger>` containing `undo_id` (`undo_<timestamp>_<seq>`), `operation_id`, `work_item_id`, `tenant_id`, `actor_id`, `forward_capability`, `reversibility_class`, `resolved_inverse_capability`, `resolved_inverse_args` (`serde_json::Value`), `snapshot_artifact_hash` (`Option<String>`), `created_at_ms`, `expires_at_ms`, and `status` (`Armed`, `Executing`, `Reverted`, `Expired`, `Failed`).


* **`GuardArmed` Atomic Flag**: Mirroring Algorithm 1 of the Cordis specification, each `UndoEntry` transitions atomically from `Armed` to `Executing` via compare-and-swap so concurrent operator clicks or fiber unwinds execute the inverse at most once.



## 4. Staged Implementation Plan

Each stage is self-contained, testable via `cargo test`, and maintains 100% adherence to Warmplane's pragmatic Rust guidelines (`// Rust guideline compliant YYYY-MM-DD`) and CI drift gates.

### Stage 1: Delayable Hold-Window Engine ("Withholding Emissions")

**Goal:** Persist outbound emissions in a shared cancellation window before submitting them to an upstream server. Existing `requireApproval` policy remains authoritative and resolves first; the hold begins with the approved arguments.

* **Configuration and effective policy (`src/config.rs`, `src/daemon/policy.rs`):** support `delayable` aliases, profile/global `holdWindowSecs`, validation on load/reload/API update, canonical alias propagation, and conflict detection. Resolve duration as profile override → alias value → global default → 30 seconds. A duration setting does not classify a capability as delayable.
* **Shared coordinator and task receipts (`src/emissions.rs`, `src/tasks.rs`, `src/daemon/state.rs`):** route REST, embedded, MCP stdio/SSE, and batch calls through one coordinator. Keep SEP-2663 task statuses unchanged; expose `phase`, `releaseAtMs`, `cancellable`, and `requestId` on receipts. Persist approved arguments, canonical server/tool mapping, request/context and idempotency correlation, and authorization provenance. Never persist bearer credentials.
* **Atomic transitions and recovery (`src/storage.rs`, daemon startup/shutdown):** persist acceptance before returning, and persist the dispatch claim before enqueueing `CallTool`. Use private temporary files, file sync, atomic rename, and directory sync. Serialize cancellation and dispatch claims through one gate. Resume future and overdue pending holds only after current policy, role, profile, capability mapping, and server checks. Fail recovered dispatching tasks as `EMISSION_OUTCOME_UNKNOWN`; never resend them. Keep pending holds in memory when persistence is disabled, and document that these holds are lost on process exit.
* **Transport behavior and idempotency:** async calls return task receipts. Synchronous calls wait when `holdWindowMs < toolTimeoutMs`; at equality they return a receipt. Dropped connections do not cancel accepted work. Route task, MCP task, and operation cancellation through the coordinator; repeated emission cancellation is idempotent, while post-claim cancellation returns a too-late conflict. Deduplicated calls reuse the original task and deadline.
* **Batch behavior:** execute each held step through the coordinator and await its result before resolving later references. Include approval, hold, and execution in the 60-second budget; cancel on budget expiry if dispatch is still pending. Persist batch/step correlation and recover only the current emission, never the entire batch continuation.
* **Audit and documentation:** add `EmissionHeld` and `EmissionCancelled` filters, redact held and released payloads, and correlate release `ToolExecution` events with task IDs. Update user/integrator guides, Warmplane skill, schema, OpenAPI, whitepaper, and this plan.
* **Verification:** cover cancellation at 500 ms of a two-second hold (zero upstream submissions), uncancelled exactly-once submission, approval and modified arguments, policy/profile/alias restrictions, cancellation races, all entry points, idempotency, restart recovery, persistence failure and permissions, batch budgets, and audit redaction/hash-chain integrity. Run repository-required Rust and UI checks.





### Stage 2: Pre-State Snapshots & Single-Step Inverse Execution (`Reversible` & `Compensable`)

**Goal:** Pair every mutating tool call with its inverse closure at execution time, capture required pre-state outside the LLM, and enforce daily/window caps on compensable actions.

* **Content-Addressed Snapshot Store (`src/storage.rs`, `src/undo/store.rs`):**
* Create a content-addressed artifact store inside `StateDirectory` (`state_dir/artifacts/art_<sha256>.json`) with `0o600` file permissions.


* Implement `AtomicFile<UndoLedger>` (`undo_ledger.json`) to persist the active undo stack across daemon restarts.




* **Pre-Execution Snapshot & Interpolation (`src/engine/mod.rs`):**
* Extract the `${...}` reference resolver from `POST /v1/tools/batch_call` into a shared interpolation module (`src/engine/interpolation.rs`) supporting `${args.<path>}`, `${result.<path>}`, and `${snapshot.<path>}`.


* Before dispatching a `reversible` or `compensable` tool call, if `preSnapshot` is configured, invoke the read capability against the upstream server, apply optional `captureJsonPath` distillation, and write the output to the content-addressed artifact store. Fail closed (abort the forward mutation) if the pre-snapshot fails.


* After the forward capability succeeds, interpolate `inverse.args` using the original `args`, upstream `result`, and saved `snapshot`, and push an `Armed` `UndoEntry` with `expires_at_ms = now + (ttlSecs * 1000)` onto `UndoLedger`.




* **Compensable Budget Caps (`src/policy.rs`, `src/undo/mod.rs`):**
* Track rolling 24-hour invocation counts per `(tenant_id, profile, capability)` for `compensable` tools.


* When a `compensable` call exceeds `dailyLimit`, automatically escalate the call into Warmplane's existing fail-closed HITL approval interceptor (`src/http_v1/tasks_api.rs`) rather than executing immediately.




* **Stage 2 Verification:**
* Unit test the interpolation resolver across nested JSON payloads, arrays, and missing keys.
* Write an embedded engine test (`tests/embedded_tests.rs`) where a mock file/DB update automatically snapshots prior state, executes the mutation, and records a valid, interpolated `UndoEntry` in `undo_ledger.json`.





### Stage 3: LIFO Fiber Accumulator & Transactional Batch Unwind

**Goal:** Mechanize BEAM-Claw and Cordis temporal composability across multi-step agent turns (`work_item_id`) and batch pipelines (`batch_call`) so partial failures or turn-level rollbacks unwind cleanly in reverse order.

* **Work-Item Fiber Grouping (`src/undo/fiber.rs`):**
* Group `UndoEntry` records by `RequestContext.work_item_id` (and `batch_id` for `POST /v1/tools/batch_call`) into a logical revertible fiber.


* Implement `unwind_fiber(work_item_id)` which pops all `Armed`, unexpired `UndoEntry` items for that fiber in strict LIFO order.




* **Idempotent Inverse Dispatch & Automatic Rollback (`src/engine/mod.rs`):**
* Pass deterministic idempotency keys (`idk_<sha256(undo_id)>`) when dispatching inverse tool calls through `ControlPlaneHandle` so interrupted unwinds can be safely retried without duplicate side effects.


* Add an `autoRollbackOnFailure: bool` option to `POST /v1/tools/batch_call`. If step $k$ of a batch pipeline fails, Warmplane automatically invokes `unwind_fiber` for steps $k-1 \dots 0$ and returns a structured envelope detailing both the step failure and the per-step rollback outcomes.




* **WORM Ledger Chaining (`src/audit/store.rs`):**
* Log every inverse execution as `AuditEventType::UndoExecuted` (or `AuditEventType::UndoFailed`) carrying the parent `operation_id` and `snapshot_artifact_hash` inside the 20-field SHA-256 + HMAC-SHA256 hash chain.




* **Stage 3 Verification:**
* Build a 3-step `batch_call` test where Step 1 and Step 2 mutate state and Step 3 fails; verify that Step 2's inverse runs before Step 1's inverse, restoring the mock server to its exact initial state, and that attempting to unwind the same fiber a second time is a no-op.





### Stage 4: Out-of-Band Operator Plane (REST API, CLI & Security Boundary)

**Goal:** Expose deterministic undo controls to human operators and external orchestrators while keeping the undo trigger inaccessible to prompt-injected LLMs by default.

* **Strict MCP Surface Isolation (`src/mcp_server.rs`, `src/rbac/`):**
* Do **not** expose `undo_execute` on the default MCP stdio/SSE tool catalog (`tools/list`) so an LLM cannot be socially engineered or prompt-injected into suppressing or abusing the safety mechanism.


* Allow explicit opt-in via RBAC (`allowAgentSelfUndo: true` on a role/profile) only for specialized recovery sub-agents.




* **HTTP REST Endpoints (`src/http_v1/undo_api.rs`):**
* `GET /v1/undo`: List active, expired, and reverted `UndoEntry` capsules filtered by `tenant_id`, `work_item_id`, or `status`.


* `POST /v1/undo/:undo_id/execute`: Trigger an immediate single-action rollback, returning the upstream inverse output and updated WORM audit digest.


* `POST /v1/undo/work-items/:work_item_id/rollback`: Unwind an entire agent turn/session in LIFO order.


* Update `docs/openapi.yaml` and `ControlPlaneHandle` (`list_undo_entries`, `execute_undo`, `rollback_work_item`) for full parity across REST and embedded Rust callers.




* **CLI Commands (`src/cli.rs`):**
* Add `warmplane undo list [--work-item <id>]`, `warmplane undo show <undo_id>`, `warmplane undo run <undo_id>`, and `warmplane undo rollback <work_item_id>`.




* **Stage 4 Verification:**
* Add RBAC integration tests in `tests/rbac_integration.rs` verifying that Cross-Tenant undo attempts return `403 POLICY_DENIED` and that expired TTL entries return `409 UNDO_WINDOW_EXPIRED` unless forced by an operator.





### Stage 5: Control Deck Action Capsules UI & Non-Blocking ChatOps Receipts

**Goal:** Shift human attention from reading pre-execution prompts to monitoring a live, low-friction feed of reversible Action Capsules.

* **Control Deck Action Feed (`ui/src/components/undo.ts`, `ui/src/main.ts`):**
* Add an **Action Feed & Undo** view to the embedded Web UI (`/ui`) synchronized in real time via the `/v1/resources/updates` SSE stream.


* Render each mutation as a self-contained **Capsule** card displaying: actor/profile provenance, capability alias, a human-readable diff between `snapshot` and `args`, a live countdown progress bar for `delayable` hold windows (`Cancel Now`) or `reversible` TTL decay (`Undo (4m 12s left)`), and 1-click **Rollback Turn** controls grouped by `work_item_id`.


* Ensure all rendered strings pass through Warmplane's consolidated HTML escape helper (`ui/` security standard) and verify `ui/dist/index.html` passes the CI Bun build drift check.




* **Bidirectional ChatOps Undo Cards (`src/chatops/`, `src/http_v1/webhooks_api.rs`):**
* Extend the Slack (Block Kit), Discord (Embeds), and Microsoft Teams (Adaptive Cards) dispatchers.


* When a `reversible`, `compensable`, or `delayable` action occurs, dispatch a non-blocking notification card containing an interactive **Undo Action** (or **Cancel Send**) button signed with Warmplane's mandatory constant-time HMAC-SHA256 webhook signature (`subtle::ConstantTimeEq`).




* **Stage 5 Verification:**
* E2E test (`tests/e2e_tests.rs`) exercising a full lifecycle: agent calls a `reversible` tool over MCP stdio $\rightarrow$ Warmplane snapshots state, executes upstream, and emits an SSE event $\rightarrow$ operator invokes the HMAC-signed webhook callback $\rightarrow$ Warmplane executes the inverse and records the hash-chained `UndoExecuted` audit event.





## 5. Risk Matrix & Boundary Limitations

Even a formal revertible-effect runtime cannot reverse every physical reality. The implementing agent must enforce the following guardrails in code and documentation:

* **Information Leakage Across the System Boundary:** If an agent reads a secret resource and sends it in a reply, deleting the reply later does not un-share the secret. Warmplane's existing `redactKeys` secret sanitization and per-profile resource RBAC (`v0.32.0`) must continue to run *before* any `reversible` or `delayable` execution.


* **Untested Inverse Contracts:** A tool labeled `reversible` whose inverse fails at runtime is worse than an unlabelled tool. Add a `warmplane undo test <alias>` CLI command (and Playground button) that executes a dry-run forward + snapshot + inverse cycle against a test target and verifies observational equivalence ($\simeq$).


* **State Drift Before Expiration:** If an external actor mutates the upstream resource during the `ttlSecs` window, blindly restoring the pre-snapshot may clobber concurrent human edits. Support an optional `preUndoCheck` hash assertion on `ReversibilityContract` that compares the current upstream state against the post-execution state before applying the inverse, prompting the operator if concurrent drift is detected.
