# WORM Audit Trail & SIEM Telemetry Export — Implementation Plan

This document details the architectural design, data models, storage engine, cryptographic integrity verification, SIEM telemetry export pipeline, compliance reporting APIs, and Control Deck UI for Warmplane's enterprise Write-Once-Read-Many (WORM) audit logging system.

---

## 1. Objectives & Compliance Goals

1. **Non-Repudiation & Tamper Evidence (WORM):**
   - Provide an immutable, write-once-read-many local audit log for all capability tool executions, approval actions, policy violations, sampling calls, and configuration changes.
   - Use cryptographic hash chaining (Merkle / SHA-256 linear chain) where each audit entry signs `hash(prev_hash + entry_data)` to detect any database tampering or retroactive modification.
2. **Standardized Compliance Fields (SOC2, ISO 27001, HIPAA, FedRAMP):**
   - Record comprehensive metadata per event: `audit_id`, `timestamp_ns`, `event_type`, `trace_id`, `request_id`, `actor_id`, `work_item_id`, `client_ip`, `target_resource` / `capability_id`, `sanitized_args`, `execution_latency_us`, `status` (`success`, `failed`, `denied`, `intercepted_hitl`), `error_code`, `operator_id`, and `merkle_hash`.
3. **Automated PII/Credential Redaction:**
   - Enforce policy-driven sanitization (`redact_keys`) prior to persisting audit records or streaming to external SIEM collectors.
4. **Multi-Target SIEM & Observability Export:**
   - **OpenTelemetry (OTel / OTLP gRPC & HTTP):** Export structured audit spans and log records to OTel collectors.
   - **Splunk HEC (HTTP Event Collector):** High-throughput JSON event ingestion.
   - **Datadog / Generic Webhook / AWS CloudWatch Logs:** Configurable HTTP/HTTPS JSON telemetry sinks with retry queues and backpressure management.
   - **Stdout / JSON Lines File:** Streamable local audit sink for containerized environments (Kubernetes FluentBit / Promtail).
5. **High Performance, Zero-Overhead Async Logging:**
   - Decouple audit record ingestion from the critical path of tool execution using an asynchronous, bounded lock-free channel and background batch flusher.
6. **Search, Query & Compliance Export API:**
   - Paginated HTTP query endpoints with filters for time range, `event_type`, `actor_id`, `capability_id`, and `status`.
   - Compliance export formats: JSONL, CSV, and Cryptographic Proof Verification report.
7. **Control Deck Audit & Compliance Explorer:**
   - Dedicated **Audit Trail** view in the web UI featuring real-time event streaming, filtering, full event inspection with parameter diffs, integrity verification badge, and 1-click export.

---

## 2. Architecture & Data Flow

```
                                  WARMPLANE DAEMON
 ┌──────────────────────────────────────────────────────────────────────────────────┐
 │                                                                                  │
 │   Agent / Caller               HITL Engine                Config / Admin         │
 │   POST /v1/tools/call       POST /v1/approvals/*       POST /v1/config/*         │
 │            │                         │                         │                 │
 │            ▼                         ▼                         ▼                 │
 │   ┌──────────────────────────────────────────────────────────────────────────┐   │
 │   │                       AuditEvent Interceptor Hook                        │   │
 │   │       - Redact sensitive keys (PolicyConfig.redactKeys)                  │   │
 │   │       - Capture latency, caller headers, trace/request/actor context     │   │
 │   └────────────────────────────────────┬─────────────────────────────────────┘   │
 │                                        │ (Non-blocking async send)               │
 │                                        ▼                                         │
 │   ┌──────────────────────────────────────────────────────────────────────────┐   │
 │   │               Bounded Ring Channel (tokio::sync::mpsc)                   │   │
 │   └────────────────────────────────────┬─────────────────────────────────────┘   │
 │                                        │                                         │
 │                                        ▼                                         │
 │   ┌──────────────────────────────────────────────────────────────────────────┐   │
 │   │                     Audit Log Batch Worker Task                          │   │
 │   │  - Linear SHA-256 Hash Chain Calculation: H_n = SHA256(H_(n-1) || record)│   │
 │   └─────────────┬────────────────────────────────────────────┬───────────────┘   │
 │                 │                                            │                   │
 │                 ▼                                            ▼                   │
 │   ┌───────────────────────────┐                ┌───────────────────────────┐     │
 │   │    Append-Only Storage    │                │    SIEM Exporter Dispatch │     │
 │   │    (SQLite / rusqlite)    │                │  (OTLP / Splunk / Datadog)│     │
 │   │  - WAL Mode               │                │  - Exponential Backoff    │     │
 │   │  - Strict INSERT ONLY     │                │  - Batching & Flush Queue │     │
 │   │  - Merkle / Chain Hashes  │                └─────────────┬─────────────┘     │
 │   └─────────────┬─────────────┘                              │                   │
 └─────────────────┼────────────────────────────────────────────┼───────────────────┘
                   │                                            │
                   ▼                                            ▼
         ┌───────────────────┐                     ┌───────────────────────────┐
         │ Control Deck UI & │                     │ Enterprise SIEM / OTel    │
         │ Compliance Export │                     │ Splunk / Datadog / OTLP   │
         └───────────────────┘                     └───────────────────────────┘
```

---

## 3. Data Schema & Cryptographic Integrity

### 3.1 Audit Event Record Schema

Each audit event conforms to a strict canonical structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique monotonic event ID (e.g. `aud_01J...` ULID / UUIDv7).
    pub id: String,
    /// Unix timestamp in nanoseconds.
    pub timestamp_ns: u64,
    /// Categorical event type:
    /// `tool_execution`, `tool_intercepted`, `approval_granted`,
    /// `approval_rejected`, `policy_violation`, `config_mutation`, `sampling_call`
    pub event_type: AuditEventType,
    /// Distributed tracing context.
    pub trace_id: String,
    pub request_id: Option<String>,
    pub actor_id: Option<String>,
    pub work_item_id: Option<String>,
    pub client_ip: Option<String>,
    /// Target entity details.
    pub server_id: Option<String>,
    pub capability_id: Option<String>,
    pub resource_uri: Option<String>,
    /// Sanitized parameters and return status.
    pub sanitized_args: Option<serde_json::Value>,
    pub execution_latency_us: Option<u64>,
    pub status: AuditEventStatus, // Success, Failed, Denied, Intercepted
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    /// HITL / Operator metadata if applicable.
    pub operator_id: Option<String>,
    pub approval_ticket_id: Option<String>,
    /// Cryptographic Tamper-Evidence Chain.
    pub prev_hash: String,
    pub hash: String,
}
```

### 3.2 Cryptographic Hash Chain (WORM Guarantee)

1. **Genesis Record:** The first record in the audit log uses a constant genesis seed `0000000000000000000000000000000000000000000000000000000000000000` as `prev_hash`.
2. **Canonical Hash Computation:**
   $$\text{Hash}_n = \text{SHA-256}\Big(\text{Hash}_{n-1} \parallel \text{id} \parallel \text{timestamp\_ns} \parallel \text{event\_type} \parallel \text{trace\_id} \parallel \text{actor\_id} \parallel \text{capability\_id} \parallel \text{canonical\_json}(\text{sanitized\_args}) \parallel \text{status}\Big)$$
3. **Verification Endpoint:**
   `GET /v1/audit/verify` reads the sequential log and verifies that $\forall i > 0, \text{prev\_hash}_i == \text{hash}_{i-1}$ and $\text{recomputed\_hash}_i == \text{hash}_i$. If any row was modified or deleted, verification fails immediately pinpointing the exact corrupted row index.

---

## 4. Configuration Schema (`mcp_servers.json`)

Extend `McpConfig` with an optional `audit` block:

```json
{
  "mcpServers": { ... },
  "policy": { ... },
  "audit": {
    "enabled": true,
    "dbPath": "warmplane_audit.db",
    "retentionDays": 90,
    "bufferCapacity": 10000,
    "flushIntervalMs": 500,
    "siem": {
      "targets": [
        {
          "type": "otlp",
          "endpoint": "http://127.0.0.1:4317",
          "protocol": "grpc",
          "headers": { "x-api-key": "${SIEM_API_KEY}" }
        },
        {
          "type": "splunk_hec",
          "url": "https://splunk.corp.internal:8088/services/collector",
          "token": "${SPLUNK_HEC_TOKEN}",
          "index": "ai_audit_events",
          "source": "warmplane-daemon"
        },
        {
          "type": "webhook",
          "url": "https://siem.corp.internal/v1/ingest",
          "authHeader": "Bearer ${DATADOG_API_KEY}"
        }
      ]
    }
  }
}
```

---

## 5. Storage Layer & High-Throughput Ingestion

### 5.1 Embedded SQLite WORM Store
- Uses `rusqlite` or `sqlx-sqlite` with `journal_mode = WAL` and `synchronous = NORMAL` for sub-millisecond writes.
- Enforces strict append-only constraints:
  - SQLite table with `id`, `timestamp_ns`, `event_type`, `trace_id`, `actor_id`, `capability_id`, `payload_json`, `status`, `prev_hash`, `hash`.
  - Database schema contains NO `UPDATE` or `DELETE` triggers; application layer only exposes `INSERT` and indexed `SELECT` operations.

### 5.2 Decoupled Actor Architecture
- **In-Memory Bounded Ring Buffer:** Workers publish `AuditEvent` into a `tokio::sync::mpsc::channel(10_000)`.
- **Background Flusher Worker:**
  - Batches writes up to 200 events or every 500ms into a single SQLite transaction.
  - Computes linear hash chain in memory before transaction commit.
  - Fans out batched events to active SIEM exporters concurrently.

---

## 6. HTTP API Specification

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/v1/audit/events` | Paginated query with filters (`start_time`, `end_time`, `actor_id`, `capability_id`, `event_type`, `status`, `limit`, `cursor`). |
| `GET` | `/v1/audit/events/:id` | Retrieve single audit event with full payload and cryptographic verification proofs. |
| `GET` | `/v1/audit/verify` | Verify linear hash chain integrity across all or a specified time slice of audit records. |
| `GET` | `/v1/audit/export` | Stream raw audit log in `jsonl` or `csv` format with `Content-Disposition: attachment`. |
| `GET` | `/v1/audit/stats` | Aggregate summary statistics (total events, calls by status, top capabilities, policy violation counts). |

---

## 7. SIEM Exporter Subsystem

1. **OpenTelemetry Log / Event Integration:**
   - Convert `AuditEvent` to standard OpenTelemetry LogRecords / Spans with semantic attributes (`gen_ai.system`, `gen_ai.tool.name`, `user.id`, `event.status`).
2. **Splunk HEC Exporter:**
   - JSON-formatted HEC payload with `time` (epoch float), `event` (structured audit payload), `index`, and `sourcetype="warmplane:audit"`.
3. **Generic Webhook / Datadog:**
   - Batch HTTP POST exporter with circuit breaking, in-memory retry queue (up to 1,000 failed events), and exponential backoff.

---

## 8. Control Deck Web UI: Audit Explorer

Add a dedicated **Audit & Compliance** tab to the Control Deck:
1. **Live Event Stream & Search Bar:** Filter by actor, tool name, date range, or status with instant debounced updates.
2. **Cryptographic Proof Badge:** Real-time indicator showing "Chain Verified: 100% Tamper Free" with a button to run `/v1/audit/verify`.
3. **Event Inspector Drawer:** Detailed JSON viewer displaying redacted arguments, execution timings, operator approval notes, and SHA-256 chain hashes.
4. **Export Action Button:** Instant download of `warmplane_audit_<timestamp>.csv` or `.jsonl` for compliance audits.

---

## 9. Phased Execution Roadmap

### Phase 1: Core Audit Engine & Hash Chain Storage
- [x] Create `src/audit/` module (`models.rs`, `store.rs`, `chain.rs`, `worker.rs`).
- [x] Implement append-only storage with linear SHA-256 hash chaining.
- [x] Add async batching worker with bounded mpsc channel.
- [x] Implement integrity verification algorithm (`verify_chain`).
- [x] Write thorough unit & integration tests for tampering detection and append-only constraints.

### Phase 2: Interceptor Integration & Redaction
- [x] Hook audit event emission into `/v1/tools/call`, HITL approval lifecycle (`approve`, `reject`, `expire`), and policy rejection paths.
- [x] Ensure arguments and returns pass through `redact_keys` sanitization before channel dispatch.
- [x] Benchmark overhead to ensure <10µs latency impact on tool calls.

### Phase 3: HTTP API & Compliance Export
- [x] Implement `/v1/audit/events`, `/v1/audit/events/:id`, `/v1/audit/verify`, `/v1/audit/export`, and `/v1/audit/stats` in `src/http_v1/audit_api.rs`.
- [x] Support JSONL and CSV streaming downloads.
- [x] Add OpenAPI specification documentation for audit endpoints.

### Phase 4: SIEM Exporter Pipelines
- [x] Implement OTLP, Splunk HEC, and HTTP Webhook telemetry forwarders with batching and retry buffers.
- [x] Add configuration parsing in `src/config.rs` for `audit.siem`.

### Phase 5: Control Deck UI (Audit Explorer)
- [ ] Create `ui/src/components/audit.ts` with responsive filtering, stats cards, and expandable event rows.
- [ ] Add integrity verification UI indicator and CSV/JSONL export triggers.
- [ ] Integrate into `ui/src/main.ts` navigation and state management.
- [ ] Run full UI build (`bun run build:ui`) and verify end-to-end.
