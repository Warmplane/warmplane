# TypeScript Integrators Guide

This guide explains how to drive a running Warmplane daemon from TypeScript over its HTTP REST API. It includes a minimal `WarmplaneClient` class you can drop into your project and build on.

---

## Prerequisites

- A running Warmplane daemon with at least one upstream MCP server configured.
- Node.js 18+ or Bun 1.0+ (both ship with native `fetch`).
- No external dependencies required — the client uses the built-in `fetch` API.

For MCP client installation, Warmplane supports a spawned stdio facade or a daemon-hosted Streamable HTTP `/mcp` facade. HTTP requires `mcpHttpServer` in the daemon configuration.

Start the daemon in a separate terminal:

```bash
warmplane daemon --config mcp_servers.json
```

---

## WarmplaneClient

A thin typed wrapper over `fetch`. No frameworks, no build step — just enough structure to avoid repeating URL construction and envelope parsing.

```typescript
// warmplane-client.ts

/** Standard Warmplane response envelope. */
interface Envelope<T = unknown> {
  ok: boolean;
  request_id: string | null;
  trace_id: string | null;
  data: T | null;
  error: EnvelopeError | null;
  retry: RetryInfo | null;
}

/** Error details within a response envelope. */
interface EnvelopeError {
  code: string;
  message: string;
  retryable: boolean;
}

/** Retry classification metadata. */
interface RetryInfo {
  classification: "safe" | "idempotent" | "unsafe" | null;
  upstream_execution_state: "not_started" | "completed" | "unknown" | null;
}

/** Compact capability entry from the catalog index. */
interface CapabilityEntry {
  id: string;
  summary: string;
  server: string;
  tags: string[];
  signature?: string;
  mode?: string;
}

/** Search engine runtime info and feature status. */
interface SearchEngineInfo {
  semantic_enabled: boolean;
  vector_backend?: string;
  rrf_k: number;
}

/** Catalog listing response body. */
interface CatalogResponse {
  version: string;
  catalog_version: string;
  capabilities: CapabilityEntry[];
  ttl_ms?: number;
  cache_scope?: string;
  search_engine?: SearchEngineInfo;
}

/** Search result entry with relevance score. */
interface SearchResult {
  id: string;
  summary: string;
  server: string;
  tags: string[];
  signature?: string;
  score: number;
  match_types: string[];
}

/** Search response body. */
interface SearchResponse {
  version: string;
  catalog_version: string;
  capabilities: SearchResult[];
}

/** Batch call step definition. */
interface BatchStep {
  id: string;
  capability_id: string;
  args: Record<string, unknown>;
  continue_on_error?: boolean;
}

/** Request body for a chained batch call. */
interface BatchCallRequest {
  steps: BatchStep[];
  request_id?: string;
  autoRollbackOnFailure?: boolean;
}

/** Outcome reported for one inverse attempt during automatic rollback. */
interface RollbackResult {
  undo_id: string;
  operation_id: string | null;
  capability: string;
  ok: boolean;
  error: string | null;
  error_code?: string;
  output?: unknown;
  audit_event_id?: string;
  audit_hash?: string;
  audit_error?: string;
  forced_expired: boolean;
  entry_status?: "armed" | "executing" | "reverted" | "expired" | "failed";
}

/** Result envelope for a batch call. */
interface BatchCallResponse {
  ok: boolean;
  request_id: string | null;
  trace_id: string;
  results: Array<Record<string, unknown>>;
  total_duration_us: number;
  rollback_results: RollbackResult[];
}

/** Task receipt returned for asynchronous execution and held emissions. */
interface TaskReceipt {
  taskId: string;
  status: "working" | "input_required" | "completed" | "failed" | "cancelled";
  requestId?: string;
  phase?: "pending_emission" | "dispatching";
  releaseAtMs?: number;
  cancellable?: boolean;
  result?: unknown;
  error?: unknown;
}

/** Context distillation options for limiting tool output. */
interface DistillOptions {
  /** JSONPath selector (e.g. `$.items[*].name`). */
  jsonpath?: string;
  /** Maximum output lines. */
  limitLines?: number;
  /** Maximum output bytes. */
  truncateBytes?: number;
}

/** Warmplane API error thrown on non-ok envelopes. */
class WarmplaneError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly retryable: boolean,
  ) {
    super(`[${code}] ${message}`);
    this.name = "WarmplaneError";
  }
}

/**
 * Minimal Warmplane HTTP API client.
 *
 * Wraps `fetch` with a base URL pointing at a running daemon.
 * All methods return parsed response envelopes or throw typed errors.
 */
class WarmplaneClient {
  private readonly baseUrl: string;
  private cachedEtag: string | null = null;

  constructor(baseUrl = "http://127.0.0.1:9090") {
    this.baseUrl = baseUrl.replace(/\/+$/, "");
  }

  // ------------------------------------------------------------------
  // Internal helpers
  // ------------------------------------------------------------------

  private async get<T>(path: string, headers?: HeadersInit): Promise<Response> {
    return fetch(`${this.baseUrl}${path}`, { headers });
  }

  private async postJson<T>(path: string, body: unknown, headers?: HeadersInit): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json", ...headers },
      body: JSON.stringify(body),
    });
    return res.json() as Promise<T>;
  }

  private assertOk<T>(envelope: Envelope<T>): Envelope<T> {
    if (!envelope.ok && envelope.error) {
      throw new WarmplaneError(
        envelope.error.code,
        envelope.error.message,
        envelope.error.retryable,
      );
    }
    return envelope;
  }

  // ------------------------------------------------------------------
  // Catalog discovery
  // ------------------------------------------------------------------

  /**
   * Lists all capabilities from the compact catalog index.
   *
   * Performs conditional revalidation using `If-None-Match` when a
   * cached ETag is available. Returns `null` on 304 Not Modified.
   */
  async listCapabilities(): Promise<CatalogResponse | null> {
    const headers: Record<string, string> = {};
    if (this.cachedEtag) {
      headers["If-None-Match"] = this.cachedEtag;
    }

    const res = await this.get("/v1/capabilities", headers);

    if (res.status === 304) return null;

    const etag = res.headers.get("etag");
    if (etag) this.cachedEtag = etag;

    return res.json() as Promise<CatalogResponse>;
  }

  /** Fetches the full JSON Schema for a single capability. */
  async describeCapability(id: string): Promise<unknown> {
    const res = await this.get(`/v1/capabilities/${encodeURIComponent(id)}`);
    return res.json();
  }

  // ------------------------------------------------------------------
  // Tool execution
  // ------------------------------------------------------------------

  /**
   * Calls a capability tool and returns the response envelope.
   *
   * Optionally pass an `idempotencyKey` for safe retries of mutating
   * operations.
   * Reversible and compensable behavior is configured on the daemon's
   * capability alias; it does not require extra request fields. When a
   * compensable daily limit is reached, the daemon routes the call through
   * its existing approval flow.
   */
  async callTool(
    capabilityId: string,
    args: Record<string, unknown>,
    idempotencyKey?: string,
  ): Promise<Envelope> {
    const headers: Record<string, string> = { "Content-Type": "application/json" };
    if (idempotencyKey) {
      headers["Idempotency-Key"] = idempotencyKey;
    }

    const res = await fetch(`${this.baseUrl}/v1/tools/call`, {
      method: "POST",
      headers,
      body: JSON.stringify({ capability_id: capabilityId, args }),
    });

    const envelope: Envelope = await res.json();
    return this.assertOk(envelope);
  }

  /**
   * Calls a capability with context distillation modifiers.
   *
   * Use `jsonpath` to select specific fields, `limitLines` to cap
   * output length, and `truncateBytes` for a hard byte budget.
   */
  async callToolDistilled(
    capabilityId: string,
    args: Record<string, unknown>,
    options: DistillOptions,
  ): Promise<Envelope> {
    const distilledArgs = { ...args };
    if (options.jsonpath) distilledArgs._jsonpath = options.jsonpath;
    if (options.limitLines) distilledArgs._limit_lines = options.limitLines;
    if (options.truncateBytes) distilledArgs._truncate_bytes = options.truncateBytes;

    return this.callTool(capabilityId, distilledArgs);
  }

  /**
   * Executes a chained batch of tool steps in a single round-trip.
   *
   * Steps can reference outputs from earlier steps using `$step_id.field`
   * variable interpolation syntax. Configured reversible and compensable
   * contracts are applied to each successful step by the daemon.
   */
  async batchCall(
    steps: BatchStep[],
    autoRollbackOnFailure = false,
  ): Promise<BatchCallResponse> {
    return this.postJson("/v1/tools/batch_call", {
      steps,
      autoRollbackOnFailure,
    });
  }

  // ------------------------------------------------------------------
  // Search
  // ------------------------------------------------------------------

  /** Searches capabilities using hybrid lexical + semantic ranking. */
  async searchCapabilities(
    query: string,
    limit = 8,
    serverIds: string[] = [],
    tags: string[] = [],
  ): Promise<SearchResponse> {
    return this.postJson<SearchResponse>("/v1/capabilities/search", {
      query,
      limit,
      server_ids: serverIds,
      tags,
    });
  }

  // ------------------------------------------------------------------
  // Resources & Prompts
  // ------------------------------------------------------------------

  /** Lists all registered resources. */
  async listResources(): Promise<unknown> {
    const res = await this.get("/v1/resources");
    return res.json();
  }

  /** Reads an upstream resource by ID. */
  async readResource(resourceId: string): Promise<unknown> {
    return this.postJson("/v1/resources/read", { resource_id: resourceId });
  }

  /** Lists all registered prompt templates. */
  async listPrompts(): Promise<unknown> {
    const res = await this.get("/v1/prompts");
    return res.json();
  }

  /** Renders a prompt template with arguments. */
  async getPrompt(promptId: string, args: Record<string, unknown> = {}): Promise<unknown> {
    return this.postJson("/v1/prompts/get", { prompt_id: promptId, arguments: args });
  }

  // ------------------------------------------------------------------
  // SSE streaming
  // ------------------------------------------------------------------

  /**
   * Connects to the catalog mutation SSE stream.
   *
   * Yields parsed event data objects as they arrive. The caller should
   * iterate with `for await` and handle reconnection as needed.
   */
  async *watchCatalogUpdates(): AsyncGenerator<{ event: string; data: unknown }> {
    const res = await this.get("/v1/resources/updates");
    if (!res.body) return;

    const reader = res.body.pipeThrough(new TextDecoderStream()).getReader();
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += value;

      const parts = buffer.split("\n\n");
      buffer = parts.pop() ?? "";

      for (const part of parts) {
        const lines = part.split("\n");
        let event = "message";
        let data = "";

        for (const line of lines) {
          if (line.startsWith("event:")) event = line.slice(6).trim();
          else if (line.startsWith("data:")) data += line.slice(5).trim();
        }

        if (data) {
          try {
            yield { event, data: JSON.parse(data) };
          } catch {
            yield { event, data };
          }
        }
      }
    }
  }
}

export {
  WarmplaneClient,
  WarmplaneError,
  type Envelope,
  type EnvelopeError,
  type RetryInfo,
  type CapabilityEntry,
  type CatalogResponse,
  type SearchResult,
  type SearchResponse,
  type BatchStep,
  type DistillOptions,
};
```

---

## Usage Examples

### 1. Discover and list capabilities

```typescript
const wp = new WarmplaneClient("http://127.0.0.1:9090");

// First call fetches the full catalog
const catalog = await wp.listCapabilities();
if (catalog) {
  console.log(`Catalog version: ${catalog.catalog_version}`);
  for (const cap of catalog.capabilities) {
    console.log(`  ${cap.id} — ${cap.summary}`);
  }
}

// Second call uses ETag — returns null on 304
const cached = await wp.listCapabilities();
if (cached === null) {
  console.log("Catalog unchanged (304 Not Modified)");
}
```

### 2. Describe a capability and call it

For an OpenAPI source, discover the configured `source.operationId` or alias through the same tool catalog. Use the described input schema to supply distinct `path` and `query` objects, and read JSON from `structuredContent`. For a configured write, preserve the idempotency key when checking an uncertain execution. See the [OpenAPI source configuration](USER-GUIDE.md#openapi-http-sources).

Trust administrators can generate a contract using `POST /v1/config/reversibility/draft`. Send the returned capability revisions and one-use confirmation token with the unchanged draft to `/v1/config/reversibility/record` after operator confirmation. Activation uses `/v1/config/reversibility/activate` with the review token from a successful recording, the draft, its revisions, and the alias. Keep the same profile across the three requests. See the [REST contract](openapi.yaml).

```typescript
const wp = new WarmplaneClient();

// Fetch full schema to understand required arguments
const schema = await wp.describeCapability("filesystem.read_file");
console.log("Schema:", JSON.stringify(schema, null, 2));

// Execute the tool
const result = await wp.callTool("filesystem.read_file", {
  path: "/tmp/example.txt",
});
console.log("Result:", JSON.stringify(result.data, null, 2));
```

### Delayable Emissions

For a delayable alias, send `async_task: true` to receive a task receipt without waiting for the hold window. A synchronous request waits only when the configured hold is shorter than `toolTimeoutMs`; otherwise it also returns a task. Duration precedence is profile policy, alias `holdWindowSecs`, global policy, then 30 seconds. Approval runs before the countdown and approved argument edits are preserved.

```typescript
const response = await fetch("http://127.0.0.1:9090/v1/tools/call", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    capability_id: "mail.send",
    args: { to: "person@example.com", body: "Hello" },
    async_task: true,
  }),
});
const body = await response.json();
const task = body.task as TaskReceipt;
console.log(task.phase, task.releaseAtMs, task.cancellable);

const cancelled = await fetch(
  `http://127.0.0.1:9090/v1/tasks/${encodeURIComponent(task.taskId)}/cancel`,
  { method: "POST", headers: { "Content-Type": "application/json" }, body: "{}" },
);
if (cancelled.status === 409) {
  // Dispatch has been claimed; the upstream outcome may be pending.
}
```

Task status can be polled with `GET /v1/tasks/{taskId}`. Cancellation can also use `POST /v1/operations/{requestId}/cancel`; it succeeds only before dispatch claim. Pending holds resume after restart and policy/profile revalidation. A task recovered in `dispatching` fails with `EMISSION_OUTCOME_UNKNOWN` and is not resent automatically. Batch steps wait for held results within the shared 60-second batch budget.

### Operator Undo

The daemon provides `GET /v1/undo`, `GET /v1/undo/{undo_id}`, `POST /v1/undo/{undo_id}/execute`, and `POST /v1/undo/work-items/{work_item_id}/rollback`. Configure per-role `undoPermissions` in RBAC. Entry summaries may include filtered inverse field names, but omit inverse arguments and snapshot contents; cross-tenant entry IDs return `404`. Setting `forceExpired: true` additionally requires the `forceExpired` permission and a non-empty `reason`; successful execution returns the committed audit event ID and hash.

`GET /v1/resources/updates` also emits named `action_capsule` invalidations with only `{ "refresh": true }`; fetch `/v1/undo` and `/v1/tasks` to update the current tenant's feed. Internal snapshot and drift-check capabilities must be allowed without approval. A pre-snapshot policy failure prevents the forward call, a post-state capture failure reports inverse registration failure after the effect, and a blocked pre-undo check leaves the entry armed. Optional `preUndoCheck` blocks inverse dispatch when the upstream state hash has drifted. ChatOps cards are opt-in through `policy.webhook.events` and submit a target-scoped action token to `/v1/webhooks/actions`; force-expired undo is unavailable through that route.

`POST /v1/undo/test` accepts `{ "alias": "..." }` for an alias configured with `testTarget.server`, fixed `testTarget.args`, and `preSnapshot`. It runs the cycle against that test server and returns whether the observed state matched its original snapshot, without returning snapshot or tool output values.

Lists use `GET /v1/undo?offset=0&limit=50` and return `{ok, result: {entries, total, offset, limit}}`. The maximum page size is 200. The Action Feed displays pages of 50 and retains the selected page on refresh. Work-item rollback covers eligible entries across pages, with reverse registration order breaking equal creation timestamps. Automatic rollback rechecks expiry at each claim.

ChatOps callbacks acknowledge valid tokens immediately and execute in the background. Inspect the entry or task afterward for completion and drift failures.

### Configure Alias Reversibility

Reversibility contracts are attached to capability aliases. They can be edited in the Control Deck's **Facade & Alias Studio** or through `POST /v1/config/alias`:

```json
{
  "kind": "tool",
  "alias": "issue.update",
  "target": "issues.update",
  "reversibility": {
    "class": "reversible",
    "ttlSecs": 86400,
    "preSnapshot": {
      "capability": "issues.read",
      "args": {"id": "${args.id}"},
      "captureJsonPath": "$.priority"
    },
    "preUndoCheck": {
      "capability": "issues.read",
      "args": {"id": "${args.id}"},
      "captureJsonPath": "$.priority"
    },
    "inverse": {
      "capability": "issues.update",
      "args": {"id": "${args.id}", "priority": "${snapshot}"}
    }
  }
}
```

When updating an existing alias, omitting `reversibility` preserves its current contract; sending `"reversibility": null` removes it. The daemon validates class-specific fields and argument object shapes before saving.

If task persistence is disabled, held emissions remain in memory and are lost when the daemon exits. Configure persistent state to recover them after restart.

### 3. Context distillation — limit large outputs

```typescript
const wp = new WarmplaneClient();

const result = await wp.callToolDistilled(
  "sqlite.read_query",
  { query: "SELECT * FROM users" },
  {
    jsonpath: "$.records[*].email",  // JSONPath filter
    limitLines: 10,                   // Max 10 lines
  },
);

console.log(JSON.stringify(result.data, null, 2));
```

### 4. Multi-step chained batch call

```typescript
const wp = new WarmplaneClient();

const result = await wp.batchCall([
  {
    id: "step1",
    capability_id: "db.get_customer",
    args: { customer_id: "cust_123" },
  },
  {
    id: "step2",
    capability_id: "stripe.get_invoice",
    args: { invoice_id: "$step1.latest_invoice_id" },
    continue_on_error: false,
  },
]);

console.log(JSON.stringify(result, null, 2));
```

### 5. Search capabilities with hybrid ranking

```typescript
const wp = new WarmplaneClient();

const results = await wp.searchCapabilities("find production error logs", 5);

for (const cap of results.capabilities) {
  console.log(`[${cap.score.toFixed(2)}] ${cap.id} — ${cap.summary}`);
}
```

### 6. Idempotent tool call with retry safety

```typescript
const wp = new WarmplaneClient();
const key = "payment-charge-ord-42";

// First call executes upstream
const r1 = await wp.callTool(
  "payments.charge",
  { amount: 100, currency: "USD" },
  key,
);
console.log("First:", r1.retry);

// Second call with same key returns cached result — no re-execution
const r2 = await wp.callTool(
  "payments.charge",
  { amount: 100, currency: "USD" },
  key,
);
console.log("Second:", r2.retry);
```

### 7. Watch catalog updates via SSE

```typescript
const wp = new WarmplaneClient();

for await (const event of wp.watchCatalogUpdates()) {
  console.log(`[${event.event}]`, JSON.stringify(event.data, null, 2));
}
```

---

## Error Handling

All methods that return envelopes throw `WarmplaneError` when `ok` is `false`. The error carries a typed `code` you can match against:

```typescript
import { WarmplaneError } from "./warmplane-client";

try {
  await wp.callTool("nonexistent.tool", {});
} catch (err) {
  if (err instanceof WarmplaneError) {
    switch (err.code) {
      case "TOOL_NOT_FOUND":
        console.error("Tool does not exist — check capability ID");
        break;
      case "POLICY_DENIED":
        console.error("Blocked by security policy");
        break;
      case "APPROVAL_PENDING":
        console.error("Awaiting operator approval");
        break;
      case "CIRCUIT_OPEN":
        console.error("Upstream circuit breaker tripped — retry later");
        break;
      default:
        console.error(`Unexpected error: ${err.message}`);
    }

    if (err.retryable) {
      console.log("This error is transient — safe to retry");
    }
  }
}
```

### Standard Error Codes

| Code | HTTP | Meaning |
|:---|:---:|:---|
| `TOOL_NOT_FOUND` | 404 | Capability ID does not exist or is policy-blocked |
| `INVALID_ARGS` | 400 | Arguments failed JSON schema validation |
| `POLICY_DENIED` | 403 | Blocked by deny rule |
| `APPROVAL_PENDING` | 202 | Intercepted by HITL — awaiting operator decision |
| `CIRCUIT_OPEN` | 503 | Upstream circuit breaker tripped |
| `UPSTREAM_TIMEOUT` | 504 | Upstream exceeded `toolTimeoutMs` |
| `RESULT_WITHHELD` | 502 | The result policy withheld a live or stored payload |
| `TOOL_DEFINITION_UNAPPROVED` | 403 | The current tool definition lacks an applicable approval |

With `policy.resultSanitization` enabled, `RESULT_WITHHELD` can carry a completed upstream execution state. Keep the original idempotency key if you check a withheld mutation again. Task status and ledger metadata can remain visible while the stored result is withheld after a policy change.

The pinned tool catalog's ETag changes when tool visibility changes. If an invocation is refused after a definition changes, refresh the catalog and submit a new request after an operator approves the current definition.

Under RBAC, authenticate approval and task input requests with a token for the ticket tenant and a role with `resolveApprovals`. Send `{"inputResponses":{"hitl_approval":{"approved":true}}}` for a linked task. If the task was created in a profile, include that profile in the `X-Warmplane-Profile` header on task input requests. The `receipt` describes the persisted decision. Read the task separately for execution status.

See the [User Guide](USER-GUIDE.md) for the complete error code table and the [Whitepaper](WHITEPAPER.md) for architectural context.

---

## Runtime Compatibility

The client uses only standard Web APIs (`fetch`, `ReadableStream`, `TextDecoderStream`) and works out of the box with:

- **Bun** ≥ 1.0
- **Node.js** ≥ 18 (native fetch)
- **Deno** ≥ 1.28
- **Browsers** (for dashboard or admin UI integrations)

No polyfills, no build step, no bundler configuration required.

---

## Next Steps

- **Typed error codes**: Extend `WarmplaneError.code` to a string union type for exhaustive switch matching.
- **Retry middleware**: Wrap `callTool` with automatic retry logic that respects `retryable` and `classification` fields.
- **HITL integration**: Poll `GET /v1/approvals` to build operator approval workflows.
- **Audit queries**: Use `GET /v1/audit/events` to pull execution history for compliance reporting.

### Audit verification

`GET /v1/audit/verify` returns `report.is_valid` and `report.total_records`.
When `report.first_retained_index` is present, it is the zero-based start of the
verified in-memory suffix. The number of retained records is
`total_records - first_retained_index`. Persisted audit logs are verified from disk.

When an upstream result arrives after task expiry, task polling and idempotent
replay of that result report `TASK_EXPIRED`. A hold receipt may report uncertain persistence
in `statusMessage`; the hold has been accepted and can resume after restart if
its record survives.
