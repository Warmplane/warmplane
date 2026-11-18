# Stage 1: Durable, Cancellable Hold Windows

## Summary and agreed behavior

Implement a shared hold-window engine for delayable capabilities across REST, embedded Rust, MCP stdio/SSE, and batch execution.

Agreed behavior:

- Preserve explicit `requireApproval` rules. Approval completes first; the cancellation window starts afterward, using the approved arguments.
- Async calls immediately return a task receipt. Synchronous calls wait for short holds and return a receipt for longer holds.
- Persist held emissions and resume unsent work after restart. Release overdue holds after policy revalidation.
- Never automatically resend a call whose dispatch began before a crash.
- Apply hold windows within batch steps.

Snapshots, inverse execution, undo ledgers, new undo commands, and Action Feed UI remain later-stage work.

## Repository grounding

- REST execution in `src/http_v1/execute.rs` and embedded execution in `src/engine/mod.rs` independently implement policy, approvals, idempotency, and upstream dispatch. MCP tool calls use the embedded engine.
- `src/batch_executor.rs` sends directly to upstream workers, so changing single-call execution alone would leave a bypass.
- `TaskRegistry::cancel_task` currently updates status and removes input waiters; it does not cancel an executing worker. Completion and failure methods can overwrite terminal states.
- `OperationRegistry` tracks cancellation tokens by request ID. Its endpoint currently reports success when a token is cancelled, without confirming that dispatch was prevented.
- Task persistence logs write failures instead of returning them. `AtomicFile` provides atomic replacement, but its current permission handling covers selected filenames and does not provide fsync durability.
- The draft references `src/policy.rs`; actual policy handling lives in `src/daemon/policy.rs` and `src/rbac/engine.rs`.

These findings make a shared dispatch gate and durable state transitions prerequisites.

## Implementation changes

### 1. Configuration and effective policy

- Extend detailed `AliasTarget` with optional `ReversibilityConfig`. Stage 1 accepts `class: "delayable"` with optional `holdWindowSecs`; reject unsupported classes rather than silently promising unimplemented behavior.
- Add optional `holdWindowSecs` to `PolicyConfig`, usable globally and in profiles. Resolve duration as: profile override → alias value → global policy default → 30 seconds. Policy duration alone does not classify a capability as delayable.
- Reject zero durations and values that overflow millisecond/deadline conversion. Validate on initial load, reload, and configuration API writes before activating changes.
- Preserve allow/deny, tenant restrictions, profile server boundaries, approval, and redaction. Do not modify `compute_effective_policy` to bypass approval.
- Resolve contracts by canonical server/tool identity and propagate them to aliases and passthrough tools. Reject conflicting contracts for the same target, preventing a second alias from bypassing the hold.
- Update configuration schema and alias-update request handling so editing an alias preserves its reversibility settings.

### 2. Shared hold coordinator and task model

Introduce a hold coordinator owned by `AppState`, used by all four execution paths.

Keep SEP-2663 `TaskStatus` values unchanged. Add optional emission metadata to task records and responses:

- `phase`: `pending_emission` or `dispatching`
- `releaseAtMs`: absolute deadline
- `cancellable`: whether cancellation can still prevent dispatch

Persist an internal execution record containing the approved payload, resolved server/tool, original capability, request/context identifiers, profile and authenticated authorization provenance, idempotency key, deadline, and dispatch phase. Never persist bearer credentials.

Use these transitions:

| Current state | Event | Result |
|---|---|---|
| Approval required | Approval granted | Persist hold and start full window |
| Pending emission | Cancellation wins | Persist `Cancelled`; never send |
| Pending emission | Deadline reached | Revalidate, durably claim dispatch, then send |
| Dispatching | Upstream success/error | Persist `Completed`/`Failed` |
| Dispatching | Cancellation requested | Reject prevention claim; dispatch may have occurred |
| Dispatching on startup | Recovery | Fail with `EMISSION_OUTCOME_UNKNOWN`; never resend |

- Serialize cancellation and dispatch claiming through one coordinator lock. A `CancellationToken` wakes the timer; the locked state transition decides the winner.
- Use Tokio monotonic timers during a process lifetime and persisted wall-clock deadlines for recovery.
- Persist accepted holds before returning a receipt. Persist the dispatch claim before enqueueing `ServerMsg::CallTool`.
- Surface persistence failures and prevent dispatch when the required write fails.
- Make terminal task transitions conditional so late workers cannot overwrite cancellation or failure.
- Generate task IDs with collision detection, replacing the current small random suffix assumption.

### 3. Persistence, startup, and shutdown

- Extend the existing task persistence model rather than introducing the Stage 2 undo ledger. Default missing fields when reading older task files.
- Add durable storage support for emission transitions: private temporary files, file sync, atomic rename, and directory sync where supported. Enforce `0o600` on emission-bearing files and `0o700` on their state directory.
- Start recovery once state, policy, profiles, and upstream initialization are available.
- Revalidate pending work against current authorization, profile membership, capability mapping, and server configuration before release. Fail closed if provenance cannot be resolved, permission was revoked, or the target changed.
- Resume future deadlines for the remaining duration; release overdue unsent holds immediately after validation. A configured but unavailable server produces a terminal failure; do not introduce automatic mutation retries.
- On graceful shutdown, stop claiming new dispatches, preserve pending holds, finish bounded in-flight work, and flush storage and audit workers.
- With persistence disabled, support holds in memory and explicitly document that restart recovery is unavailable.

### 4. Transport behavior, cancellation, and idempotency

- Route delayable execution through the same coordinator after existing approval handling. Extract reusable upstream dispatch/result handling needed by REST, embedded, and batch paths.
- For `async_task: true` or `Prefer: respond-async`, return the existing task receipt shape with countdown metadata; REST returns `202`.
- For synchronous calls, wait when `holdWindowMs < toolTimeoutMs`; otherwise immediately return a task receipt. At equality, return a receipt.
- Keep the upstream execution timeout separate from the hold duration. Do not expire held tasks using the approval timeout.
- A dropped synchronous connection does not cancel an accepted hold. Explicit task or operation cancellation remains available.
- Route task cancellation, MCP `tasks/cancel`, and operation cancellation to the same coordinator. Preserve operation endpoint lookup by request ID and include that identifier in receipts.
- Return a specific too-late conflict for held emissions already claimed for dispatch. Repeated cancellation of an already-cancelled emission succeeds without another audit event.
- Authorize emission task inspection and cancellation using stored ownership and current tenant/profile policy. An opaque task ID must not grant access.
- Associate each held execution with its idempotency key. Duplicates reuse the task and original deadline; they never create another timer. Return current receipts while pending and the terminal outcome afterward.
- Retain cancelled outcomes for the existing idempotency retention period. Sending again requires a new key.

### 5. Batch behavior

- Execute delayable steps through the coordinator and await their actual result before resolving later `$step.field` references.
- Include approval, hold, and execution time in the existing 60-second batch budget.
- If the batch deadline or cancellation occurs during a hold, cancel that hold before returning. If dispatch already began, report that its outcome may be pending rather than claiming prevention.
- Preserve `continue_on_error`, interpolation, and step-result behavior.
- Persist batch/request/step correlation on held tasks. After restart, recover the accepted current emission only; do not replay completed steps or start subsequent steps. Durable continuation of the whole batch remains outside Stage 1.

### 6. Audit and documentation

- Add `EmissionHeld` and `EmissionCancelled`, including audit API filter parsing. Emit each once per accepted transition.
- Continue using `ToolExecution` for released outcomes. Include task correlation in sanitized event metadata and preserve the existing hash-chain representation.
- Apply redaction consistently to held, cancelled, failed, and replayed events.
- Update the user guide, Warmplane skill, Rust and TypeScript integrator guides, configuration schema, and OpenAPI task/receipt definitions.
- Update the whitepaper and undo plan to describe approval preservation, durable recovery, batch behavior, and the unknown-outcome boundary.
- Add public Rust documentation and dated compliance markers to new or fully reviewed Rust files. No frontend implementation is required.

## Test and acceptance plan

Use the existing channel-backed mock upstream pattern in `tests/tasks_tests.rs`, with deterministic clock control for state-machine races.

- **Required delivery tests:** a two-second hold cancelled at 500 ms produces zero upstream messages; an uncancelled hold produces exactly one.
- **Policy:** deny remains authoritative; approval precedes the timer; approved argument edits are sent; profile duration precedence works; aliases cannot bypass the contract.
- **Concurrency:** cancellation and expiry race through one winner; terminal states remain terminal; repeated cancellation emits one event.
- **Interfaces:** REST async/header paths, synchronous short/long holds, embedded calls, MCP facade and passthrough calls expose equivalent behavior.
- **Idempotency:** duplicates before release, after completion, after cancellation, and after restart share one execution.
- **Recovery:** future and overdue unsent holds resume; interrupted dispatch is never replayed; revoked permission or changed targets block release.
- **Storage:** failed acceptance/claim writes prevent dispatch; old task files load; private permissions and durable transition behavior are verified.
- **Batch:** later steps wait for released results; budget expiry cancels pending emission; restart does not replay earlier steps or continue later ones.
- **Audit:** held/cancelled/released events remain redacted and pass hash-chain verification.

Run all repository-required checks: formatting, Clippy with all targets/features, tests, type checking, Rust compliance markers, and the frozen-lockfile UI typecheck/build/drift check.

The delivery guarantee is precise: a successful pre-dispatch cancellation prevents upstream submission. Crash recovery avoids automatic resend after a durable dispatch claim; it cannot guarantee exactly-once effects inside an arbitrary upstream system.
