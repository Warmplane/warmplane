# Deployment and Operations Runbook

This runbook provides baseline production patterns for Warmplane.

## 1) Environment Variables

Common runtime variables:

- `RUST_LOG=info,warmplane=debug`
- `WARMPLANE_OTEL_ENABLED=true`
- `OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317`
- `WARMPLANE_SERVICE_NAME=warmplane-prod`

Use env-backed secrets for upstream auth (`tokenEnv`, `passwordEnv`), or use the standard `oauth2` flow for dynamic bearer tokens. Note that OAuth2 configuration requires loopback redirect capability on the running environment.

## 2) systemd Service

Example `/etc/systemd/system/warmplane.service`:

```ini
[Unit]
Description=Warmplane MCP Control Plane
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/warmplane daemon --config /etc/warmplane/mcp_servers.json
Restart=always
RestartSec=2
Environment=RUST_LOG=info
Environment=WARMPLANE_OTEL_ENABLED=true
Environment=OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
Environment=WARMPLANE_SERVICE_NAME=warmplane-prod
User=warmplane
Group=warmplane
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

Enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now warmplane
sudo systemctl status warmplane
```

## 3) Docker Pattern

Example `Dockerfile`:

```dockerfile
FROM rust:1.85 as builder
WORKDIR /src
COPY . .
# Standard build (BM25 Lexical + Fuzzy search)
RUN cargo build --release
# Alternatively, build with embedded ONNX vector embeddings:
# RUN cargo build --release --features semantic-search

FROM debian:bookworm-slim
RUN useradd -m -u 10001 warmplane
COPY --from=builder /src/target/release/warmplane /usr/local/bin/warmplane
USER warmplane
ENTRYPOINT ["warmplane"]
```

Run:

```bash
docker run --rm \
  -p 9090:9090 \
  -p 9191:9191 \
  -e RUST_LOG=info \
  -e WARMPLANE_OTEL_ENABLED=true \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317 \
  -v $(pwd)/mcp_servers.json:/app/mcp_servers.json:ro \
  warmplane:latest daemon --config /app/mcp_servers.json
# Port 9090 = REST control-plane API
# Port 9191 = Streamable HTTP/SSE MCP facade (when mcpHttpServer block is configured)
```

## 4) Kubernetes Pattern

Minimal deployment concepts:

- ConfigMap for non-secret config
- Secret for auth tokens/passwords
- Readiness probe: `GET /v1/capabilities`
- Resource requests/limits set from observed workload

Readiness probe example:

```yaml
readinessProbe:
  httpGet:
    path: /v1/capabilities
    port: 9090
  initialDelaySeconds: 3
  periodSeconds: 5
```

## 5) Baseline SLOs

Recommended initial targets:

- Availability: 99.9% for `/v1/capabilities` and `/v1/tools/call`
- p95 latency for `/v1/tools/call`: environment-specific, start with < 1s for fast upstreams
- Error budget tracking on `UPSTREAM_TIMEOUT` and `UPSTREAM_ERROR`

## 6) Alerting Recommendations

Page-level alerts:

- sustained daemon unavailability
- high timeout ratio (`UPSTREAM_TIMEOUT`)
- high upstream error ratio (`UPSTREAM_ERROR`)
- sudden drop in capability count at startup (possible upstream failure)

Ticket-level alerts:

- policy deny spikes
- trace export failures
- repeated auth failures to HTTP upstreams

## 7) Operational Checklist

Before production rollout:

1. Validate config in CI: `warmplane validate-config --config mcp_servers.json`
2. Smoke test MCP stdio server mode: `./scripts/smoke_mcp_server.sh`
3. If using MCP HTTP/SSE facade: verify `warmplane mcp-http-server --config mcp_servers.json --port 9191` starts and responds on `http://127.0.0.1:9191/mcp`
4. Verify OTEL export path in staging
5. Confirm redaction list covers secrets
6. Load test representative tool/resource/prompt mixes

## 8) Incident Triage Flow

1. Capture `trace_id` from client response envelope.
2. Find trace in observability backend.
3. Correlate structured logs for same trace/context fields.
4. Classify failure as policy, transport, timeout, or upstream application error.
5. Apply runbook action (config fix, timeout tuning, upstream remediation).
