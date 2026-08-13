# Observability Guide

Warmplane emits structured JSON logs by default, supports distributed trace correlation, and exports distributed traces via OpenTelemetry (OTLP).

This guide covers practical setup, request correlation features (introduced in v0.5.0), idempotency/cancellation observability (v0.6.0), catalog change feeds (v0.4.0), and operations.

---

## 1) Logging & Context Defaults

Warmplane uses `tracing` + JSON output by default.

### Key Properties

- Machine-parseable log lines (JSON)
- Stable event names for startup, capability execution, catalog versioning, and operation lifecycle
- Contextual fields for audit:
  - `server_id`, `capability_id`, `resource_id`, `prompt_id`
  - **Request Context (v0.5.0)**: `request_id`, `operation_id`, `work_item_id`, `actor_id`, `grant_id`
  - **Idempotency & Retry (v0.6.0)**: `idempotency_key`, retry classification (`safe` | `unsafe` | `idempotent`), retry state
- `trace_id` correlation across HTTP response envelopes, stdio logs, and OTLP spans

### Verbosity Controls

```bash
export RUST_LOG=info,warmplane=debug
```

---

## 2) Request Correlation & Propagation (v0.5.0+)

All execution endpoints (`/v1/tools/call`, `/v1/resources/read`, `/v1/prompts/get`) automatically capture request context from:

1. Request body fields (`request_id`, `context.operation_id`, `context.actor_id`, etc.)
2. HTTP correlation headers (when body fields are omitted):
   - `X-Request-ID`
   - `X-Operation-ID`
   - `X-Work-Item-ID`
   - `X-Actor-ID`
   - `X-Grant-ID`

These attributes are injected into `tracing` spans and exported in OTLP trace context, enabling end-to-end tracing across orchestrator, Warmplane, and upstream MCP servers.

---

## 3) Idempotency, Cancellation & Catalog Observability (v0.4.0–v0.6.0)

### 3.1 Idempotency & Deduplication (v0.6.0)
When duplicate or concurrent execution requests pass an `Idempotency-Key` or `X-Idempotency-Key`:
- In-flight execution logs record `idempotency_key` and state (`in_progress` vs deduplicated cache hit).
- Duplicated callers receive identical execution response envelopes without duplicating upstream calls.

### 3.2 Operation Cancellation (v0.6.0)
- Operation registration and cancellation events emit explicit structured audit events with `request_id` and `cancellation_reason`.
- In-flight cancellations via `POST /v1/operations/:id/cancel` log `operation_cancelled` events.

### 3.3 Catalog Versioning & Change Events (v0.4.0)
- SHA-256 catalog checksums are recorded on catalog updates.
- `GET /v1/catalog/events` emits structured change events (`evt_1`, `evt_2`, …) capturing incremental catalog updates for event consumers.

---

## 4) OpenTelemetry Export

OTEL export is optional and controlled by environment variables.

### Enable OTEL

```bash
export WARMPLANE_OTEL_ENABLED=true
```

### Collector Endpoints

Set collector endpoint (preferred):

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
```

Fallback endpoint (if `OTEL_EXPORTER_OTLP_ENDPOINT` is unset):

```bash
export WARMPLANE_OTEL_ENDPOINT=http://127.0.0.1:4317
```

Set service name:

```bash
export WARMPLANE_SERVICE_NAME=warmplane-prod
```

Run:

```bash
warmplane daemon --config mcp_servers.json
```

---

## 5) Recommended Production Baseline

- Ingest structured JSON logs into your SIEM / Loki / ELK pipeline.
- Export OTLP traces to a central collector (e.g. Tempo, Jaeger, Datadog).
- Standardize `WARMPLANE_SERVICE_NAME` by environment (`warmplane-dev`, `warmplane-staging`, `warmplane-prod`).
- Pass HTTP correlation headers (`X-Request-ID`, `X-Operation-ID`, `X-Actor-ID`) from your API gateway or agent orchestrator to track full request lineages.

---

## 6) Collector Patterns

### 6.1 Local OpenTelemetry Collector

`otel-collector-config.yaml` example:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
      http:

processors:
  batch:

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

Run collector and point Warmplane to `http://127.0.0.1:4317`.

### 6.2 Grafana / Tempo Pipeline

- **Warmplane** -> OTLP collector -> Tempo
- **Logs** -> Loki (or equivalent)
- Correlate logs and traces by `trace_id`, `request_id`, and `operation_id`.

---

## 7) Correlation Triage Workflow

When an HTTP response or execution error returns a `trace_id` or `request_id`:

1. Look up `trace_id` in your distributed tracing backend (Tempo/Jaeger).
2. Filter structured logs by `request_id` or `operation_id`.
3. Inspect upstream server responses, idempotency deduplication status, and retry classification (`safe` vs `unsafe`).

---

## 8) Security and Compliance Notes

- Prefer env-backed secrets for upstream auth (`tokenEnv`, `passwordEnv`).
- Keep daemon on localhost unless fronted by an authenticated reverse proxy or API gateway.
- Utilize policy deny/allow rules for write actions by default.
- Header redacting cleans key fields (`token`, `api_key`, `password`) automatically from logs and response envelopes.
