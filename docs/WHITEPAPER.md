# Warmplane Whitepaper

## Warmplane: A Local Control Plane for Deterministic, Token-Efficient MCP Operations

### Abstract

Modern Model Context Protocol (MCP) deployments increasingly suffer from a structural inefficiency: repeated transmission and processing of large capability surfaces, even when only a small subset of tools, resources, and prompts are used per task. This paper introduces **Warmplane**, a local control plane that maintains persistent upstream MCP sessions while exposing a compact, deterministic, policy-governed interface to clients.

Warmplane separates backend protocol richness from frontend interaction cost by presenting index-first capability discovery, hybrid lexical/vector search, SHA-256 catalog cache validation (`304 Not Modified`), and on-demand schema expansion. In measured scenarios from the project evaluation harness, this approach reduced token footprint by **58.1%–58.2%** in a public filesystem control suite and **95.6%–95.8%** in an authenticated GitHub Copilot MCP suite. Micro-benchmarks demonstrate sub-microsecond facade overheads, including **50.4 ns** cached catalog validation and **159.8 ns** idempotency lookups. These improvements were achieved while introducing deterministic execution governance, including Human-in-the-Loop (HITL) approval gates, cryptographic Write-Once-Read-Many (WORM) audit logging, SIEM streaming (Splunk HEC, Webhooks), request context propagation (`operation_id`, `actor_id`, `grant_id`), idempotency deduplication (`Idempotency-Key`), safe/unsafe retry classification, and active operation cancellation.

We present the system architecture, transport model, policy and governance controls, error determinism model, hybrid capability search engine, catalog versioning model, cryptographic audit subsystem, and empirical evaluation methodology. We also discuss enterprise implications for latency, cost, reliability, security, and auditability, and outline future research directions for adaptive schema compaction and workload-aware orchestration.

---

## 1. Introduction

### 1.1 Problem Statement

As organizations operationalize AI agents, tool connectivity moves from demonstration to infrastructure. MCP has become a useful substrate for standardizing tool, resource, and prompt access. However, in direct MCP client-server patterns, agent loops frequently overpay in two dimensions:

1. **Context overhead**: Large metadata payloads are delivered repeatedly.
2. **Control fragmentation**: Policy enforcement, Human-in-the-Loop governance, context tracking, audit trails, retries, and error handling are inconsistently implemented across clients.

The result is avoidable token spend, higher startup latency, security exposure, compliance vulnerability, and reduced operational predictability.

### 1.2 Thesis

The central thesis of this whitepaper is:

> Agent systems should treat MCP integration as a two-plane architecture: rich protocol backend + compact control-plane frontend.

Warmplane implements this thesis by:

- Keeping upstream MCP sessions warm and stateful.
- Exposing a compact, stable interface for tools, resources, and prompts alongside hybrid lexical/semantic search.
- Providing zero-token catalog revalidation via SHA-256 state digests and change event streams.
- Normalizing invocation, request context, and error envelopes with explicit retry governance.
- Centralizing policy, PII redaction, Human-in-the-Loop approval workflows, idempotency deduplication, and operation cancellation controls.
- Enforcing non-repudiable WORM audit logging with SHA-256 linear hash chaining and real-time SIEM streaming.

### 1.3 Contributions

This paper contributes:

1. A practical architecture for MCP session persistence and compact interaction surfaces.
2. A deterministic execution model across Web UI, CLI, HTTP REST, and MCP-native client modes.
3. A Human-in-the-Loop governance model providing non-blocking suspension, parameter modification, and signed webhook notifications for sensitive capability execution.
4. A Write-Once-Read-Many (WORM) cryptographic audit log architecture guaranteeing tamper-evident traceability and seamless SIEM forwarding.
5. In-flight context distillation (`_jsonpath`, `_limit_lines`, `_truncate_bytes`) and multi-step chained batch execution with reference interpolation (`$step.field`).
6. A reproducible token-efficiency evaluation harness and measured baselines across real-world workloads.
7. Micro-benchmark profiling demonstrating sub-microsecond control-plane overheads.
8. A hybrid BM25 and ONNX vector search engine for sub-linear capability discovery over dense catalogs across HTTP and MCP facade interfaces.
9. A deterministic catalog digest model for conditional cache revalidation (`304 Not Modified`) and cursor-based event feeds.
10. An execution governance framework providing multi-tenant request context propagation, idempotency deduplication, retry safety classification, and active in-flight operation cancellation.

---

## 2. Background and Motivation

### 2.1 The “Description Tax” in Tool Calling

In many agent stacks, the dominant cost is not tool execution but tool description overhead. Before a model can execute a single operation, it may receive broad catalogs of full schemas for tools that are never invoked.

This “description tax” creates three compounding effects:

- **Token inflation** in prompt context.
- **Planning noise** from irrelevant capability details.
- **Latency drag** due to repeated schema transfer and processing.

### 2.2 Why Direct Connectivity Alone Is Insufficient

Direct MCP connectivity maximizes compatibility, but at scale it can underperform operationally when each client independently handles:

- Transport differences and connection lifecycles.
- Error semantics and unhandled crashes.
- Policy interpretation and authorization drift.
- Retries, idempotency, and timeouts.
- Schema filtering, searching, and caching.

Warmplane addresses this by introducing a local control plane that standardizes these concerns once.

---

## 3. System Architecture

```
                 ┌──────────────────────────────────────────────┐
                 │       AI Clients & Agent Orchestrators       │
                 │  (Claude Desktop, Cursor, Web, REST Gateways)│
                 └──────────────────────┬───────────────────────┘
                                        │
                 ┌──────────────────────▼───────────────────────┐
                 │             WARMPLANE CONTROL PLANE          │
                 │                                              │
                 │  ┌────────────────────┐ ┌─────────────────┐  │
                 │  │ Compact HTTP / CLI │ │ Stdio MCP Proxy │  │
                 │  │ + Web Control Deck │ │ Lightweight     │  │
                 │  └─────────┬──────────┘ └────────┬────────┘  │
                 │            │                     │           │
                 │  ┌─────────▼─────────────────────▼────────┐  │
                 │  │     Governance & Policy Pipeline       │  │
                 │  │  • Allow/Deny Gates   • HITL Approvals │  │
                 │  │  • Redaction Filter   • Idempotency    │  │
                 │  │  • Request Context    • Retry Tags     │  │
                 │  └───────────────────┬────────────────────┘  │
                 │                      │                       │
                 │  ┌───────────────────▼────────────────────┐  │
                 │  │    Registry & Search Subsystem         │  │
                 │  │  • Hybrid Search (BM25 + FastEmbed)    │  │
                 │  │  • ETag Catalog Digest (SHA-256)       │  │
                 │  │  • Cursor Change Feeds                 │  │
                 │  └───────────────────┬────────────────────┘  │
                 └──────────────────────┼───────────────────────┘
                                        │ Managed Persistent Sessions
                 ┌──────────────────────▼───────────────────────┐
                 │        Upstream MCP Servers (Warm & Ready)   │
                 │   [GitHub]   [Postgres]   [Filesystem]...    │
                 └──────────────────────────────────────────────┘
```

### 3.1 Architectural Overview

Warmplane consists of five major components:

1. **Upstream Session Layer**
   - Connects to multiple MCP upstreams over stdio and streamable HTTP/SSE transports.
   - Keeps negotiated sessions persistent to avoid cold-start handshakes.
   - Supports dynamic server mounting, unmounting, and zero-downtime hot-reloading (`POST /v1/config/reload`).

2. **Registry and Search Layer**
   - Builds in-memory registries for capabilities, resources, and prompts.
   - Applies alias mapping (`capabilityAliases`, `resourceAliases`, `promptAliases`).
   - Implements **Hybrid Capability Search** combining BM25 lexical scoring with optional FastEmbed ONNX vector embeddings.
   - Computes SHA-256 state digests for **Catalog Versioning and Change Event Feeds**.
   - Generates deterministic catalog listings with `ttl_ms` and `cache_scope` hints.

3. **Policy, Governance, and Envelope Layer**
   - Enforces allow/deny patterns across capability types.
   - Evaluates Human-in-the-Loop (HITL) approval rules (`requireApproval`) and manages suspension states with crash-safe atomic disk persistence (`AtomicFile`), auto-expiring timeouts across daemon restarts.
   - Applies payload redaction keys in logs and trace spans.
   - Standardizes response envelopes with **Request Context** (`operation_id`, `actor_id`, `grant_id`, `work_item_id`) and **Retry Governance** (`safe|unsafe|idempotent` classification).
   - In-flight **Context Distillation & Truncation** (`_jsonpath`, `_limit_lines`, `_truncate_bytes`) reducing agent context token consumption.
   - Multi-step **Chained Batch Execution** (`POST /v1/tools/batch_call`, `capabilities_batch_call`) with `$step.field` reference interpolation.
   - Supports **Multi Round-Trip Requests (MRTR)** with `input_responses` and `request_state` propagation for interactive approvals and missing input elicitation.

4. **Idempotency and Operations Manager**
   - Deduplicates concurrent or replayed invocations via `Idempotency-Key` backed by atomic on-disk persistence and in-memory TTL caching.
   - Manages active task handle lifetimes and provides in-flight operation cancellation (`POST /v1/operations/:id/cancel`).

5. **WORM Audit & SIEM Subsystem**
   - Implements append-only storage with linear SHA-256 cryptographic hash chaining ($\text{hash}_i = \text{SHA256}(\text{prev\_hash}_i \mathbin{\Vert} \text{canonical\_payload}_i)$).
   - Guarantees non-repudiable audit trails across tool invocations, HITL decisions, policy violations, and configuration mutations.
   - Houses an asynchronous background batching queue with automatic SIEM dispatchers (Splunk HEC, HTTP Webhooks).
   - Exposes mathematical integrity verification (`/v1/audit/verify`) and streaming exports (`/v1/audit/export`).

6. **Access Modes**
   - Embedded Control Deck Web UI (`/ui` and `/`).
   - HTTP `/v1` facade (exposing capabilities, hybrid search, tool execution, batch calls, catalog events, approvals, WORM audit verification/export, operation cancellation, SSE resource updates, argument completion, sampling, resources, and prompts).
   - CLI facade (`warmplane server`, `config`, `approvals`, `search-capabilities`, `list-catalog-events`, `cancel-operation`).
   - MCP stdio server mode exposing lightweight synthetic tools (`capability_search`, `capabilities_batch_call`, `subscriptions_listen`, `completion_complete`, etc.) and native resources/prompts methods.

### 3.2 Transport Model

Per upstream server, transport is inferred by strict configuration:

- `command` $\rightarrow$ stdio transport
- `url` $\rightarrow$ streamable HTTP transport

Exactly one selector must be set; ambiguous entries fail fast at startup.

For HTTP/SSE upstreams, Warmplane supports:

- `protocolVersion` header control (defaults to `"2026-07-28"` with backward compatibility for `"2025-11-25"`).
- `allowStateless` behavior.
- Custom HTTP headers.
- Authentication schemas: static credentials (`bearer`, `basic`) and dynamic `oauth2` (OAuth 2.1 / OIDC) flows featuring PKCE (`S256`), dynamic server discovery (RFC 9728 & RFC 8414), RFC 9207 / SEP-2468 `iss` matching, scope accumulation during step-up challenges, and silent token refreshing via rotated refresh tokens.

### 3.3 Human-in-the-Loop (HITL) Execution Governance

Warmplane introduces an integrated HITL suspension engine:

1. **Interception**: Capabilities matching `policy.requireApproval` patterns are intercepted before upstream dispatch.
2. **Ticket Creation**: An approval ticket is created with a unique ID (e.g. `appr-1723668200-1`), capturing sanitized parameters and caller context, atomically persisted to disk.
3. **Webhook Notification**: Outbound HMAC-SHA256 signed webhook alerts (`X-Warmplane-Signature-256`) are dispatched to operator dashboards or chat bots.
4. **Resolution**: Operators approve (optionally modifying JSON arguments) or reject the ticket via the Control Deck Web UI, HTTP API, or CLI (`warmplane approvals approve`).
5. **Execution or Abortion**: Approved executions resume immediately with operator-supplied arguments; rejected or expired tickets return clean `APPROVAL_REJECTED` or `APPROVAL_TIMEOUT` error codes.

### 3.4 Cryptographic WORM Audit Trails and SIEM Forwarding

To satisfy enterprise compliance mandates (SOC2 Type II, ISO 27001, HIPAA), Warmplane implements a Write-Once-Read-Many (WORM) audit subsystem:

1. **Linear Cryptographic Hash Chaining**: Every log event record $R_i$ computes a SHA-256 hash over its canonical representation concatenated with the hash of the preceding record:
   $$\text{hash}_0 = \text{SHA256}(\text{GENESIS\_HASH} \mathbin{\Vert} \text{canonical}(R_0))$$
   $$\text{hash}_i = \text{SHA256}(\text{hash}_{i-1} \mathbin{\Vert} \text{canonical}(R_i)) \quad \forall i \ge 1$$
   Any modification, insertion, or truncation of stored events invalidates subsequent hashes in the chain.
2. **Deterministic Verification (`GET /v1/audit/verify`)**: Traverses the sequential log in $O(N)$ time, recomputing and verifying each link. Returns a detailed verification report identifying the exact record ID if corruption is detected.
3. **Asynchronous Batching**: Ingestion utilizes a non-blocking bounded queue (default capacity 10,000) flushed via background worker batches (flush interval 250ms, max batch size 100).
4. **Native SIEM Export**: Batched events stream in real time to external security hubs:
   - **Splunk HEC**: Formatted directly to Splunk HTTP Event Collector JSON payloads with event timestamps and index routing.
   - **HTTP Webhooks / Datadog**: Dispatched over HTTPS with configurable authorization headers and custom metadata tags.

---

## 4. Performance Profile and Micro-Benchmarks

Micro-benchmarks measured with Criterion on Apple Silicon demonstrate Warmplane's minimal resource overhead:

| Benchmark Target | Scale / Workload | Latency ($p_{50}$) | Speedup / Optimization |
| :--- | :--- | :--- | :--- |
| **Catalog `If-None-Match` Validation** | Match $\rightarrow$ `304 Not Modified` | **$50.4\ \text{ns}$** | Zero-copy quote-trim validation (**66.3% faster**) |
| **Catalog `If-None-Match` Check** | Header Absent | **$1.98\ \text{ns}$** | Direct `HeaderMap` lookup miss |
| **Idempotency Cache Hit** | Single thread | **$159.8\ \text{ns}$** | Immediate return of cached value |
| **Concurrent Idempotency Contention** | 16 threads (800 ops) | **$550.7\ \mu\text{s}$** | $<0.7\ \mu\text{s}$ per op (**83.1% faster** than lock retention) |
| **Catalog SHA-256 Versioning** | 500 capabilities | **$126.0\ \mu\text{s}$** | In-memory key sort + SHA-256 hash |
| **Lexical Search Scan** | 1,000 capabilities | **$372.1\ \mu\text{s}$** | Zero-allocation streaming match (~2.7M evals/sec) |
| **Hybrid Search (RRF Filtered)** | $N=50$ candidates | **$15.9\ \mu\text{s}$** | Server filter + Reciprocal Rank Fusion |
| **Hybrid Search (RRF Unfiltered)** | $N=1,000$ candidates | **$1.41\ \text{ms}$** | Full candidate scan + Reciprocal Rank Fusion |

---

## 5. Formalizing the Efficiency Hypothesis

Let:

- $R_t$ = raw token cost of direct MCP metadata surfaces per cycle,
- $F_t$ = facade token cost per cycle,
- $S_t = R_t - F_t$ = token savings,
- $\eta = S_t / R_t$ = fractional savings.

In direct mode, repeated loop cost over $n$ turns approximates:

$$
C_{raw}(n) = n \cdot R_t
$$

In index-first facade mode with one on-demand expansion cost $D$:

$$
C_{facade}(n) = n \cdot I + D
$$

where $I$ is compact index cost.

When $R_t \gg I$, savings scale with loop length:

$$
\eta(n) = 1 - \frac{nI + D}{nR_t}
$$

As $n \to \infty$, $\eta(n) \to 1 - I/R_t$.

### Search-Augmented and Cache-Validated Efficiency

Warmplane extends this model in two dimensions:

1. **Search-Augmented Discovery ($Q_k$)**: In ultra-dense tool catalogs, rather than ingesting full index $I$, the agent issues a query returning top-$k$ candidate schema details ($Q_k \ll I \ll R_t$):

$$
C_{search}(n) = n \cdot Q_k + D
$$

2. **Zero-Token Conditional Revalidation ($V = 0$)**: When catalog state is unchanged, an agent validating state via `If-None-Match: <etag>` receives HTTP `304 Not Modified`:

$$
C_{cached}(n) = 0 \text{ tokens}
$$

Thus, total context spend across $n$ turns with zero schema changes collapses to only execution payloads, driving asymptotic savings $\eta(n) \to 100\%$ for non-mutating turns.

---

## 6. Empirical Token Savings Results

Measured from the evaluation harness (`eval/token-efficiency/`):

### 6.1 Authenticated GitHub Copilot MCP Suite

- **Discovery Pull**:
  - Raw: `54,715` tokens
  - Facade: `2,386` tokens
  - Savings: `52,329` (**`95.6%`**)
- **5-Turn Tool Loop**:
  - Raw: `260,005` tokens
  - Facade: `11,173` tokens
  - Savings: `248,832` (**`95.7%`**)
- **10-Turn Mixed Loop**:
  - Raw: `547,150` tokens
  - Facade: `22,895` tokens
  - Savings: `524,255` (**`95.8%`**)

### 6.2 Public Filesystem Control Suite

- **Discovery Pull**:
  - Raw: `2,552` tokens
  - Facade: `1,066` tokens
  - Savings: `1,486` (**`58.2%`**)
- **5-Turn Tool Loop**:
  - Raw: `12,760` tokens
  - Facade: `5,349` tokens
  - Savings: `7,411` (**`58.1%`**)
- **10-Turn Mixed Loop**:
  - Raw: `25,520` tokens
  - Facade: `10,669` tokens
  - Savings: `14,851` (**`58.2%`**)

Higher raw schema density drives disproportionately larger gains in compact-plane architectures.

---

## 7. Enterprise Security and Trust Boundaries

Warmplane concentrates control points to enforce strict security invariants:

- **Mitigation of Confused Deputy Attacks**: Employs Resource Indicators (RFC 8707) to bound issued access tokens to the exact canonical target server URI.
- **Defending against Authorization Mix-Ups**: Implements exact string-match issuer parameter verification (RFC 9207 / SEP-2468) during loopback redirects.
- **Securing Public Clients (PKCE)**: Uses S256 PKCE verification to safeguard authorization codes from interception.
- **Tenant Context Isolation**: Threads `actor_id` and `grant_id` context through execution policies and audit trails.
- **Idempotency Key Scope Boundaries**: Scopes deduplication state by caller identity and target operation to prevent cache pollution attacks.
- **Egress Filtering and SSRF Defenses**: Discovery and token requests utilize strict hostname and scheme validations to avoid server-side request forgery.
- **HMAC Webhook Signatures**: Signs outbound approval notifications with HMAC-SHA256.
- **Cryptographic Audit Immutability**: Proves tamper-evidence via sequential SHA-256 chain links, protecting historical execution logs against post-hoc alteration or deletion.
- **SIEM Telemetry Boundary**: Enforces PII/credential redaction rules before dispatching audit batches to external collectors (Splunk HEC, HTTP Webhooks).

---

## 8. Conclusion and Future Directions

Warmplane demonstrates that MCP scale problems are solved through control-plane design rather than connectivity alone.

By maintaining persistent upstream sessions and exposing compact, deterministic, policy-governed interfaces, Warmplane delivers **58%–96% token savings**, sub-microsecond routing performance, and enterprise-grade security and HITL governance.

### Future Research Directions

1. **Streamable Tool Call Envelopes**: Extending normalized envelopes to support chunked streaming outputs.
2. **Profile-Aware Prompting Contracts**: Distinct compact surfaces for planner, executor, and auditor roles.
3. **Dynamic FinOps Token Caps**: Real-time spending quotas and automatic capability throttling.
4. **Automated Upstream Circuit Breaking**: Intelligent failover and health supervision for fragile MCP processes.
