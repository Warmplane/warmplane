# Wishlist for application integrators

## Purpose

Warmplane already provides a compact facade over multiple Model Context
Protocol (MCP) servers. It keeps upstream sessions warm, normalizes errors,
applies policy, and exposes stable aliases for capabilities, resources, and
prompts.

This wishlist describes additions that would make Warmplane easier to embed in
an agent application. The application owns its users, tasks, authorization,
approval workflow, and audit records. Warmplane remains the MCP control plane.

The requests do not assume a particular agent framework. They apply to local
desktop applications, server-side agent runtimes, and multi-user products.

## Priorities

| Priority | Request | Why it matters |
| --- | --- | --- |
| P0 | Capability search | Large capability catalogs need a compact way to find the relevant tool. |
| P0 | Catalog versions | Clients must safely cache indexes and descriptions. |
| P0 | Request context and correlation | An application needs to connect a Warmplane call to its own durable operation record. |
| P1 | Idempotency, cancellation, and retry metadata | External writes need reliable recovery after timeouts or process restarts. |
| P1 | Result shaping | Raw tool output can be too large or sensitive for an agent prompt. |
| P1 | Health and authentication state | An application needs actionable connection status without seeing credentials. |
| P2 | Temporary policy overlays | An application may grant a narrow, short-lived capability without changing global configuration. |
| P2 | Events and namespaces | These features improve recovery and isolation in larger deployments. |

## P0: Capability search

### Need

`list` and exact-ID `describe` work well for small registries. They become
hard to use when a client has hundreds of tools from several servers. Sending a
large list to a model recreates the description-tax problem that Warmplane is
designed to solve.

### Request

Add a capability search endpoint and matching CLI command. It should search
only capability metadata, not execute tools or inspect tool results.

Suggested endpoint:

```text
POST /v1/capabilities/search
```

Suggested request:

```json
{
  "query": "triage production errors",
  "limit": 8,
  "server_ids": ["github", "observability"],
  "tags": ["read"],
  "modes": ["read"]
}
```

Suggested response:

```json
{
  "version": "v1",
  "catalog_version": "sha256:...",
  "capabilities": [
    {
      "id": "observability.logs.search",
      "summary": "Search structured application logs.",
      "server": "observability",
      "tags": ["logs", "read"],
      "mode": "read",
      "score": 0.91,
      "match": ["semantic", "tag"]
    }
  ]
}
```

### Search behavior

Use hybrid retrieval:

- lexical matching for exact IDs, aliases, product names, acronyms, and tags;
- semantic matching for intent expressed in natural language; and
- deterministic filters for server, tag, risk mode, and result limit.

Return compact entries only. A client calls `describe` for the selected ID.
The response should state why an item matched without exposing model internals.

### Why this belongs in Warmplane

Warmplane owns the canonical normalized registry and alias map. Searching that
registry prevents every client from rebuilding the same index. Applications
can still apply their own authorization filter before showing results or before
asking Warmplane to execute a selected capability.

### Notes

- Search should be optional and should degrade to lexical matching when a
  semantic index is disabled or unavailable.
- Search must never widen the configured Warmplane policy.
- Search ranking is advisory. Exact capability IDs remain authoritative.

## P0: Catalog versions and cache validation

### Need

An agent application should cache a compact catalog and individual capability
descriptions. It needs to know when aliases, schemas, policies, or upstream
availability make that cache stale.

### Request

Expose a stable `catalog_version` on every capability, resource, and prompt
list response. Include it on `describe`, `read`, `get`, and execution envelopes
where applicable. The value may be an opaque hash or monotonic revision.

Support HTTP conditional reads with `ETag` and `If-None-Match` for list and
describe endpoints. Return `304 Not Modified` when the effective public
catalog has not changed.

Optionally add a compact change feed:

```text
GET /v1/catalog/events?after={cursor}
```

Events should identify the changed object type and ID. They should not include
full schemas by default.

### Why it matters

Without a version, a client cannot distinguish a valid cached description from
one that references a renamed tool, changed input schema, removed permission,
or unavailable upstream. Versioning reduces repeated metadata transfer while
making retries and diagnostics more reliable.

## P0: Request context and correlation

### Need

An application often records each external operation in its own database. The
record may include the requesting user or agent, a work item, an approval, and
an application-side idempotency key. Warmplane already returns a `trace_id`,
but a client also needs a reliable way to attach its operation ID to logs and
responses.

### Request

Accept an optional, opaque client context on call, read, and prompt-get
requests. Warmplane should not interpret it for authorization.

Suggested shape:

```json
{
  "id": "repo.issue.search",
  "params": {"query": "is:open label:bug"},
  "request_id": "client-request-123",
  "context": {
    "operation_id": "op-123",
    "work_item_id": "work-456",
    "actor_id": "agent-789",
    "grant_id": "grant-abc"
  }
}
```

Echo a validated, size-bounded context in the execution envelope and structured
logs. Preserve it as OpenTelemetry attributes when tracing is enabled.

### Requirements

- Treat context as untrusted metadata.
- Enforce a documented size limit.
- Redact configured context keys in logs.
- Do not place context in upstream tool arguments unless explicitly mapped.
- Return `request_id`, `trace_id`, and the accepted context on success and
  error envelopes.

### Why it matters

This lets an application reconstruct a causal chain across its user interface,
its durable records, Warmplane logs, and upstream diagnostics without parsing
free-form text.

## P1: Idempotency, cancellation, and retry metadata

### Need

Network failures make the outcome of a write uncertain. A client may time out
after the upstream system has already created an issue, sent a message, or
changed a record. Blind retry can duplicate the side effect.

### Request

Add optional `idempotency_key` and `deadline_ms` fields to call requests.
Forward the idempotency key to an upstream server when that server supports
one. Otherwise retain it in Warmplane's operation records for a documented
deduplication window.

Add cancellation for in-flight calls. It may be an endpoint such as:

```text
POST /v1/operations/{request_id}/cancel
```

Return explicit retry metadata in every execution envelope:

```json
{
  "retry": {
    "classification": "safe | unsafe | unknown",
    "upstream_execution_state": "not_started | running | completed | unknown"
  }
}
```

### Why it matters

An application can then distinguish a failed read, which is usually safe to
retry, from a timed-out write whose result must be verified before replay.
Warmplane does not need to decide the application's approval policy. It should
provide enough execution evidence for the application to make a safe choice.

## P1: Result shaping and data handling

### Need

Tool results may be large, contain irrelevant fields, or include sensitive
data. An agent application often needs a concise, safe result for an agent
prompt and a separate full result for authorized inspection.

### Request

Extend per-capability configuration with optional result controls:

- maximum returned byte count;
- JSON-path allowlist or denylist;
- truncation strategy;
- sensitive-field redaction;
- compact summary fields; and
- content type and pagination hints.

Make the envelope explicit about shaping:

```json
{
  "ok": true,
  "data": {"items": []},
  "result_meta": {
    "truncated": false,
    "redacted_fields": ["customer.email"],
    "next_cursor": null
  }
}
```

### Why it matters

Warmplane's existing log redaction is useful. Result shaping extends the same
discipline to client payloads. It reduces context cost and lowers the chance
that a client accidentally stores sensitive upstream output in prompts, logs,
or long-lived memory.

## P1: Health, upstream state, and authentication signals

### Need

An application needs to explain why a capability is unavailable. A generic
`SERVER_UNREACHABLE` error is not enough for connection setup and operations.

### Request

Add a health endpoint with one compact record per configured upstream:

```text
GET /v1/health/upstreams
```

Each record should include:

- server ID and transport;
- connection state: `ready`, `connecting`, `reconnecting`, `unavailable`, or
  `authentication_required`;
- last successful connection time;
- catalog version and last refresh time;
- session age when relevant;
- recent error class and safe diagnostic message; and
- rolling latency and timeout counts.

For OAuth, expose credential state such as `valid`, `expires_soon`,
`authentication_required`, or `insufficient_scope`. Do not expose tokens,
authorization codes, refresh tokens, or sensitive endpoint data.

### Why it matters

This gives a client a clear recovery action: wait for reconnection, ask an
operator to authenticate, request a new scope, or disable a broken connection.

## P2: Temporary policy overlays

### Need

Warmplane's static allow/deny policy is a valuable baseline. Some applications
need a smaller grant for one operation or a short-lived workflow. Updating the
global configuration for that purpose is too broad and operationally awkward.

### Request

Add an optional request-bound policy overlay. It can only narrow the active
Warmplane policy. It cannot allow a capability that the configured policy
denies.

Suggested properties:

- explicit capability allowlist;
- optional argument constraints where safely expressible;
- expiry time;
- opaque grant ID for correlation; and
- an auditable policy hash in the result envelope.

### Why it matters

An application can offer a narrowly scoped approval without trusting a model
or a general client process with every configured capability. This remains
defense in depth. The application should still enforce its own authorization.

## P2: Lifecycle events and namespaces

### Events

Add optional structured events for operation lifecycle changes:

- accepted;
- policy denied;
- started;
- completed;
- failed;
- timed out; and
- cancellation requested or confirmed.

Events should carry `request_id`, `trace_id`, and accepted client context. A
cursor-based pull API is sufficient. Webhooks or server-sent events can be
optional transports.

These events help a client reconcile durable operation records after a restart
or lost request response.

### Namespaces

Add optional named namespaces or profiles. Each namespace should isolate its
upstream configuration, aliases, policy, catalog view, and trace labels.

This helps multi-user and multi-environment deployments avoid accidental
cross-use of credentials or capabilities. A local single-user installation can
continue to use one default namespace.

## Non-goals

This wishlist does not ask Warmplane to become an agent runtime.

Warmplane should not own:

- user-facing task workflows;
- approval user interfaces;
- application-specific role-based authorization;
- long-term business records; or
- model planning and prompt construction.

Its value is a compact, deterministic, observable MCP control plane that
applications can compose with their own trust and workflow models.

## Suggested delivery order

1. Add catalog versions and conditional reads.
2. Add request context and correlation support.
3. Add lexical capability search with filters.
4. Add semantic ranking as an optional index implementation.
5. Add idempotency, cancellation, and retry-state metadata.
6. Add result shaping and upstream health records.
7. Add policy overlays, lifecycle events, and namespaces when deployment needs
   justify them.

This order improves integration quality without delaying the existing compact
facade or requiring a full workflow product inside Warmplane.
