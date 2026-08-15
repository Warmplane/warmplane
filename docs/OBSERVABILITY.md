# Observability Guide

Warmplane emits structured JSON logs by default, supports distributed trace correlation, and exports distributed traces via OpenTelemetry (OTLP).

This guide covers logging defaults, request correlation, Human-in-the-Loop (HITL) audit logging, idempotency tracking, operation cancellation, catalog change feeds, and OpenTelemetry collector setup.

---

## 1. Logging and Context Defaults

Warmplane uses the `tracing` framework and emits machine-parseable JSON logs by default.

### Key Capabilities

- **Structured event names**: Stable event identifiers for daemon lifecycle, upstream server mounts, capability execution, catalog versioning, and approval workflows.
- **Contextual audit fields**:
  - `server_id`: Identifier of the upstream MCP server.
  - `capability_id`, `resource_id`, `prompt_id`: Unique target identifiers.
  - `request_id`: Server-generated or client-supplied unique request identifier.
  - `operation_id`, `work_item_id`, `actor_id`, `grant_id`: Multi-tenant request tracing attributes.
  - `idempotency_key`, `retry_classification`: Deduplication keys and retry safety markers (`safe`, `unsafe`, `idempotent`).
  - `ticket_id`, `operator`, `approval_status`: HITL governance metadata.
- **Trace correlation**: Propagates `trace_id` across HTTP response envelopes, stdio logs, and OTLP spans.

### Verbosity Controls

Configure logging levels with the `RUST_LOG` environment variable:

```bash
export RUST_LOG=info,warmplane=debug
```

---

## 2. Request Correlation and Propagation

All execution endpoints (`/v1/tools/call`, `/v1/resources/read`, `/v1/prompts/get`) automatically capture and propagate request context.

### Context Sources

Warmplane resolves request context from request payload fields or HTTP headers (in order of precedence):

1. **Request payload attributes**:
   - `request_id`
   - `context.operation_id`
   - `context.work_item_id`
   - `context.actor_id`
   - `context.grant_id`

2. **HTTP correlation headers** (fallback when payload fields are omitted):
   - `X-Request-ID`
   - `X-Operation-ID`
   - `X-Work-Item-ID`
   - `X-Actor-ID`
   - `X-Grant-ID`

These attributes are injected into active `tracing` spans and forwarded in OTLP trace context. This provides full request lineage across the agent orchestrator, Warmplane daemon, and upstream MCP servers.

---

## 3. Human-in-the-Loop (HITL) and Governance Observability

Warmplane provides dedicated observability for security policy enforcement and interactive capability approvals.

### 3.1 Approval Lifecycle Audit Events

When a capability matches `policy.requireApproval` patterns, Warmplane pauses execution and emits structured audit logs:

- `approval_requested`: Emitted when an approval ticket is created, recording `ticket_id`, `capability_id`, `server_id`, `sanitized_args`, and `request_id`.
- `approval_approved`: Emitted when an operator approves a ticket, recording `ticket_id`, `operator`, and whether arguments were modified (`modified_args`).
- `approval_rejected`: Emitted when an operator rejects a ticket, recording `ticket_id`, `operator`, and rejection `reason`.
- `approval_expired`: Emitted when a ticket exceeds `policy.approvalTimeoutSecs`.

### 3.2 Webhook Dispatch Observability

If `policy.webhook` is configured:

- Warmplane signs outbound webhook requests with HMAC-SHA256 in the `X-Warmplane-Signature-256` header.
- Webhook dispatch attempts, response status codes, and network retries emit structured `webhook_dispatch` events.

---

## 4. Idempotency, Cancellation, and Catalog Events

### 4.1 Idempotency and Deduplication Tracking

When requests provide an `Idempotency-Key` header or payload field:

- In-flight execution logs record `idempotency_key` and state (`in_progress` vs deduplicated cache hit).
- Secondary callers subscribing to in-flight tasks log deduplication cache hits.

### 4.2 Operation Cancellation

- In-flight operations cancelled via `POST /v1/operations/:id/cancel` or `warmplane cancel-operation` emit explicit `operation_cancelled` events with `request_id` and elapsed runtime.

### 4.3 Catalog Versioning and Mutation Feed

- SHA-256 catalog checksums are recomputed and logged whenever upstream server capabilities change.
- `GET /v1/catalog/events` emits structured change events (`capability_added`, `capability_removed`, `capability_updated`) for event stream consumers.

### 4.4 Dynamic Server Hot-Reload Events

Hot-reloading daemon state via `POST /v1/config/reload` or `warmplane reload` emits audit events:
- `config_reloaded`: Records added, updated, and removed server counts.
- `server_mounted` / `server_unmounted`: Records individual upstream connection lifecycle transitions.

---

## 5. OpenTelemetry (OTLP) Export

OpenTelemetry trace export is optional and controlled by environment variables.

### Environment Configuration

| Variable | Default | Description |
| :--- | :--- | :--- |
| `WARMPLANE_OTEL_ENABLED` | `false` | Enables OpenTelemetry trace export when set to `true`. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://127.0.0.1:4317` | Target OTLP gRPC collector endpoint (standard). |
| `WARMPLANE_OTEL_ENDPOINT` | `http://127.0.0.1:4317` | Fallback OTLP endpoint if `OTEL_EXPORTER_OTLP_ENDPOINT` is unset. |
| `WARMPLANE_SERVICE_NAME` | `warmplane` | Service identifier tag injected into emitted trace spans. |

### Example Startup

```bash
export WARMPLANE_OTEL_ENABLED=true
export OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector.internal:4317
export WARMPLANE_SERVICE_NAME=warmplane-prod

warmplane daemon --config mcp_servers.json
```

---

## 6. Collector Pipelines and Architecture

### 6.1 Local OpenTelemetry Collector

Sample `otel-collector-config.yaml`:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 256

exporters:
  logging:
    verbosity: normal

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [logging]
```

### 6.2 Enterprise Observability Stack

- **Warmplane Traces**: Exported via OTLP gRPC $\rightarrow$ OpenTelemetry Collector $\rightarrow$ Tempo / Jaeger / Datadog.
- **Warmplane Logs**: Ingested via stdout JSON $\rightarrow$ Vector / FluentBit $\rightarrow$ Loki / Elasticsearch.
- **Correlation**: Link logs and spans using `trace_id`, `request_id`, and `operation_id`.

---

## 7. Incident Triage Workflow

When debugging a failed capability execution:

1. **Extract Identifiers**: Note `trace_id`, `request_id`, and `error.code` from the HTTP response envelope or log line.
2. **Search Traces**: Locate the trace by `trace_id` in your distributed tracing tool (Tempo, Jaeger, Datadog) to inspect span durations and upstream latency.
3. **Filter Logs**: Query structured log files by `request_id` or `operation_id` to review argument sanitization, policy evaluations, and raw upstream protocol responses.
4. **Check Approval State**: If error code is `APPROVAL_PENDING`, `APPROVAL_TIMEOUT`, or `APPROVAL_REJECTED`, look up the ticket in `GET /v1/approvals/:id` or `warmplane approvals get <id>`.
5. **Verify Retry Governance**: Inspect the `"retry"` object (`classification`: `safe` | `unsafe` | `idempotent`) in the response envelope to determine whether client retries are safe.

---

## 8. Security and Redaction

- **Payload Redaction**: Keys configured under `policy.redactKeys` (e.g. `api_key`, `password`, `token`) are automatically masked in log events, tracing spans, and webhook payloads.
- **Secret Management**: Store upstream credentials in environment variables (`tokenEnv`, `passwordEnv`, `secretEnv`).
- **Localhost Binding**: Keep the daemon bound to `127.0.0.1` unless fronted by an authenticated reverse proxy or API gateway.
- **Webhook Security**: Always verify the `X-Warmplane-Signature-256` HMAC signature on receiving webhook endpoints.
