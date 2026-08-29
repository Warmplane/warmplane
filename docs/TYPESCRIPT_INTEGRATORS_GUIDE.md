# TypeScript Integrators Guide

This guide explains how to drive a running Warmplane daemon from TypeScript over its HTTP REST API. It includes a minimal `WarmplaneClient` class you can drop into your project and build on.

---

## Prerequisites

- A running Warmplane daemon with at least one upstream MCP server configured.
- Node.js 18+ or Bun 1.0+ (both ship with native `fetch`).
- No external dependencies required — the client uses the built-in `fetch` API.

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
   * variable interpolation syntax.
   */
  async batchCall(steps: BatchStep[]): Promise<unknown> {
    return this.postJson("/v1/tools/batch_call", { steps });
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
