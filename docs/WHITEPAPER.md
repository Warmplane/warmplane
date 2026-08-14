# Warmplane Whitepaper

## Warmplane: A Local Control Plane for Deterministic, Token-Efficient MCP Operations

### Abstract

Modern Model Context Protocol (MCP) deployments increasingly suffer from a structural inefficiency: repeated transmission and processing of large capability surfaces, even when only a small subset of tools/resources/prompts are used per task. This paper introduces **Warmplane**, a local control plane that maintains persistent upstream MCP sessions while exposing a compact, deterministic, policy-governed interface to clients.

Warmplane separates backend protocol richness from frontend interaction cost by presenting index-first capability discovery, hybrid lexical/vector search, SHA-256 catalog cache validation (`304 Not Modified`), and on-demand schema expansion. In measured scenarios from the project evaluation harness, this approach reduced token footprint by **58.1%–58.2%** in a public filesystem control suite and **95.6%–95.8%** in an authenticated GitHub Copilot MCP suite. These improvements were achieved while introducing deterministic execution governance, including request context propagation (`operation_id`, `actor_id`, `grant_id`), idempotency deduplication (`Idempotency-Key`), safe/unsafe retry classification, and active operation cancellation.

We present the system architecture, transport model, policy/governance controls, error determinism model, hybrid capability search engine, catalog versioning model, and empirical evaluation methodology. We also discuss enterprise implications for latency, cost, reliability, security, and auditability, and outline future research directions for adaptive schema compaction and workload-aware orchestration.

## 1. Introduction

### 1.1 Problem Statement

As organizations operationalize AI agents, tool connectivity moves from demonstration to infrastructure. MCP has become a useful substrate for standardizing tool/resource/prompt access. However, in direct MCP client-server patterns, agent loops frequently overpay in two dimensions:

1. **Context overhead**: large metadata payloads are delivered repeatedly.
2. **Control fragmentation**: policy, context tracking, retries, and error handling are inconsistently implemented across clients.

The result is avoidable token spend, higher startup latency, and reduced operational predictability.

### 1.2 Thesis

The central thesis of this whitepaper is:

> Agent systems should treat MCP integration as a two-plane architecture: rich protocol backend + compact control-plane frontend.

Warmplane implements this thesis by:

- keeping upstream MCP sessions warm and stateful,
- exposing a compact, stable interface for tools/resources/prompts alongside hybrid lexical/semantic search,
- providing zero-token catalog revalidation via SHA-256 state digests and change event streams,
- normalizing invocation, request context, and error envelopes with explicit retry governance,
- centralizing policy, redaction, idempotency deduplication, and operation cancellation controls.

### 1.3 Contributions

This paper contributes:

1. A practical architecture for MCP session persistence plus compact interaction surfaces.
2. A deterministic execution model across CLI, HTTP, and MCP-native client modes.
3. A reproducible token-efficiency evaluation harness and measured baselines.
4. A governance model suitable for enterprise policy, observability, and risk controls.
5. A hybrid BM25 and ONNX vector search engine for sub-linear capability discovery over dense catalogs.
6. A deterministic catalog digest model for conditional cache revalidation (`304 Not Modified`) and cursor-based event feeds.
7. An execution governance framework providing multi-tenant request context propagation, idempotency deduplication, safe retry safety classification, and active in-flight operation cancellation.

## 2. Background and Motivation

### 2.1 The “Description Tax” in Tool Calling

In many agent stacks, the dominant cost is not tool execution but tool description overhead. Before a model can execute a single operation, it may receive broad catalogs of full schemas for tools that are never invoked.

This “description tax” creates three compounding effects:

- **token inflation** in prompt context,
- **planning noise** from irrelevant capability details,
- **latency drag** due to repeated schema transfer and processing.

### 2.2 Why Direct Connectivity Alone Is Insufficient

Direct MCP connectivity maximizes compatibility, but at scale it can underperform operationally when each client independently handles:

- transport differences,
- error semantics,
- policy interpretation,
- retries and timeouts,
- schema filtering and caching.

Warmplane addresses this by introducing a local control plane that standardizes these concerns once.

## 3. System Architecture

### 3.1 Architectural Overview

Warmplane consists of five major components:

1. **Upstream Session Layer**
   - Connects to multiple MCP upstreams.
   - Supports stdio and streamable HTTP/SSE transports.
   - Keeps negotiated sessions persistent.

2. **Registry and Search Layer**
   - Builds in-memory registries for capabilities, resources, prompts.
   - Applies alias mapping (`capabilityAliases`, `resourceAliases`, `promptAliases`).
   - Implements **Hybrid Capability Search** (v0.3) combining BM25 lexical scoring with optional FastEmbed ONNX vector embeddings.
   - Computes SHA-256 state digests for **Catalog Versioning & Change Event Feeds** (v0.4).
   - Generates deterministic catalog listings with `ttl_ms` and `cache_scope` hints (v0.9).

3. **Policy, Governance, and Envelope Layer**
   - Enforces allow/deny patterns across capability types.
   - Applies payload redaction keys in logs.
   - Standardizes response envelopes with **Request Context** (`operation_id`, `actor_id`, `grant_id`, `work_item_id`) (v0.5) and **Retry Governance** (`safe|unsafe|idempotent` classification) (v0.6).
   - Supports **Multi Round-Trip Requests (MRTR)** (v0.9) with `input_responses` and `request_state` propagation for interactive approvals and missing input elicitation.

4. **Idempotency and Operations Manager**
   - Deduplicates concurrent or replayed invocations via `Idempotency-Key` (v0.6).
   - Manages active task handle lifetimes and provides in-flight operation cancellation (`POST /v1/operations/:id/cancel`) (v0.6).

5. **Access Modes**
   - HTTP `/v1` facade (exposing capabilities, hybrid search, catalog events, operation cancellation, SSE resource updates, argument completion, sampling, resources, and prompts).
   - CLI facade (`warmplane search-capabilities`, `list-catalog-events`, `cancel-operation`, etc.).
   - MCP server mode exposing lightweight synthetic tools (`subscriptions_listen`, `completion_complete`, etc.) and native resources/prompts methods (v0.9).

### 3.2 Transport Model

Per upstream server, transport is inferred by strict configuration:

- `command` => stdio transport
- `url` => streamable HTTP transport

Exactly one selector must be set; ambiguous entries fail fast at startup.

For HTTP/SSE upstreams, Warmplane supports:

- `protocolVersion` header control (defaults to `"2026-07-28"` with backward compatibility for `"2025-11-25"`),
- `allowStateless` behavior,
- custom headers,
- authentication schemas: static credentials (`bearer`, `basic`) and dynamic `oauth2` (OAuth 2.1 / OIDC) flows featuring PKCE (`S256`), dynamic server discovery (RFC 9728 & RFC 8414), RFC 9207 / SEP-2468 `iss` matching, scope accumulation during step-up challenges, and silent token refreshing via rotated refresh tokens.

### 3.3 Deterministic Call and Governance Model

Warmplane normalizes execution/read/get results to a stable envelope:

- `ok`: boolean success flag
- `request_id`: unique server-generated request identifier
- `trace_id`: OpenTelemetry trace correlation identifier
- `context`: structured request context (`operation_id`, `work_item_id`, `actor_id`, `grant_id`)
- `retry`: execution governance metadata (`classification`: `safe` | `unsafe` | `idempotent`, `state`: `fresh` | `deduplicated` | `replayed`)
- `data`: payload output upon success
- `error`: structured error classification envelope upon failure

Error classes are explicit and bounded (e.g., `UPSTREAM_TIMEOUT`, `UPSTREAM_ERROR`, `SERVER_UNREACHABLE`, `OPERATION_CANCELLED`, `POLICY_DENIED`).

When an orchestrator issues concurrent requests with an identical `Idempotency-Key` (or `X-Idempotency-Key` header), Warmplane joins the secondary requests to the active in-flight worker, returning a single cached execution outcome to all callers without re-executing the upstream capability.

### 3.4 Observability Architecture

Warmplane implements observability as a first-class control-plane concern rather than an afterthought:

1. **Structured audit logs**
   - JSON-formatted runtime logs across daemon lifecycle and capability/resource/prompt operations.
   - Event records include contextual fields (e.g., server ID, capability/resource/prompt IDs, trace IDs, `actor_id`, `grant_id`, `operation_id`).

2. **OpenTelemetry tracing**
   - Optional OTLP export path for enterprise trace backends.
   - Service metadata can be overridden with deployment-specific naming.
   - Local structured logs remain enabled even when OTEL export is active.

3. **Cross-surface correlation**
   - HTTP execution envelopes and logs expose matching `trace_id` and `RequestContext` attributes.
   - HTTP header fallbacks (`X-Request-ID`, `X-Operation-ID`, `X-Actor-ID`, `X-Grant-ID`) ensure trace context propagation even across non-native HTTP clients.

This design provides auditable and machine-parsable operational evidence while preserving deterministic API behavior.

### 3.5 Hybrid Capability Discovery Architecture

In high-density environments featuring hundreds of upstream tools, even compact index listings can accumulate token context. Warmplane introduces a **Hybrid Capability Search Engine** (`POST /v1/capabilities/search`):

- **Lexical BM25 Scoring**: Evaluates keyword frequency across tool names, aliases, and description tokens.
- **Semantic ONNX Vector Embeddings**: Uses local FastEmbed vector embeddings to evaluate semantic similarity against natural language intent.
- **Structured Metadata Filtering**: Filters query candidates by tag facets, upstream server IDs, and policy permissions.

Agents query this search surface to discover top-$k$ candidate tools without ingesting full schema catalogs.

### 3.6 Catalog Digest, Conditional Cache Validation, and Change Feed

To eliminate redundant metadata pulls entirely when catalog schemas have not changed, Warmplane implements a **Deterministic Catalog Digest Model**:

1. **State Digest Hashing**: Computes a SHA-256 hash over the combined capability, resource, and prompt registry state.
2. **HTTP `ETag` & Conditional Revalidation**: Catalog endpoints (`/v1/capabilities`, `/v1/resources`, `/v1/prompts`) emit an `ETag` header containing the catalog digest. Subsequent requests providing `If-None-Match: <etag>` receive an empty `304 Not Modified` response (0 tokens spent on context).
3. **Cursor-Based Change Feed (`/v1/catalog/events`)**: Long-running agents subscribe to change feed event streams (`capability_added`, `capability_removed`, `capability_updated`) using cursor pagination to track upstream catalog mutation incrementally.

## 4. Formalizing the Efficiency Hypothesis

Let:

- $R_t$ = raw token cost of direct MCP metadata surfaces per cycle,
- $F_t$ = facade token cost per cycle,
- $S_t = R_t - F_t$ = token savings,
- $\eta = S_t / R_t$ = fractional savings.

In direct mode, repeated loop cost over $n$ turns approximates:

$$
C_{raw}(n) = n \cdot R_t
$$

In index-first facade mode with one on-demand expansion cost $ D $:

$$
C_{facade}(n) = n \cdot I + D
$$

where $I$ is compact index cost.

When $R_t \gg I $, savings scale with loop length:

$$
\eta(n) = 1 - \frac{nI + D}{nR_t}
$$

As $n \to \infty $, $\eta(n) \to 1 - I/R_t $.

### 4.1 Search-Augmented and Cache-Validated Efficiency

Warmplane v0.3+ and v0.4+ extend this model in two dimensions:

1. **Search-Augmented Discovery ($Q_k$)**: In ultra-dense tool catalogs, rather than ingesting full index $I$, the agent issues a query returning top-$k$ candidate schema details ($Q_k \ll I \ll R_t$). The facade cost per query cycle becomes:

$$
C_{search}(n) = n \cdot Q_k + D
$$

2. **Zero-Token Conditional Revalidation ($V = 0$)**: When catalog state is unchanged, an agent validating state via `If-None-Match: <etag>` receives HTTP `304 Not Modified`, yielding:

$$
C_{cached}(n) = 0 \text{ tokens}
$$

Thus, total context spend across $n$ turns with zero schema changes collapses to only execution payloads, driving asymptotic savings $\eta(n) \to 100\%$ for non-mutating turns.

Warmplane’s measured results fit this behavior: large $R_t/I$ and $R_t/Q_k$ ratios produce very high sustained savings.

## 5. Evaluation Methodology

### 5.1 Harness

Warmplane includes a reproducible harness at `eval/token-efficiency/`.

For each suite:

1. Measure **raw** payload footprints:
   - `tools/list`
   - `resources/list`
   - `prompts/list`
2. Measure **facade** payload footprints:
   - `/v1/capabilities`
   - `/v1/resources`
   - `/v1/prompts`
   - one capability describe request
3. Tokenize with `cl100k_base` (via `tiktoken-rs`).
4. Compute scenario outcomes:
   - discovery pull,
   - 5-turn tool loop,
   - 10-turn mixed loop.

### 5.2 Scenarios

- **Discovery**: one complete metadata pull.
- **5-turn loop**: repeated tool interaction with lazy detail acquisition.
- **10-turn mixed**: tools every turn, resources every 2 turns, prompts every 3 turns.

### 5.3 Limitations of Current Runs

Not all external suites are always measurable without credentials. This paper only treats concrete measured results as empirical evidence and clearly labels control-vs-authenticated scope.

## 6. Empirical Results

### 6.1 Authenticated GitHub Copilot MCP Suite

Measured from `eval/token-efficiency/output/report.md`:

- **Discovery**:
  - Raw: `54,715` tokens
  - Facade: `2,386` tokens
  - Savings: `52,329` (`95.6%`)

- **5-turn tool loop**:
  - Raw: `260,005`
  - Facade: `11,173`
  - Savings: `248,832` (`95.7%`)

- **10-turn mixed loop**:
  - Raw: `547,150`
  - Facade: `22,895`
  - Savings: `524,255` (`95.8%`)

### 6.2 Public Filesystem Control Suite

Measured from same report:

- **Discovery**:
  - Raw: `2,552`
  - Facade: `1,066`
  - Savings: `1,486` (`58.2%`)

- **5-turn tool loop**:
  - Raw: `12,760`
  - Facade: `5,349`
  - Savings: `7,411` (`58.1%`)

- **10-turn mixed loop**:
  - Raw: `25,520`
  - Facade: `10,669`
  - Savings: `14,851` (`58.2%`)

### 6.3 Interpretation

Two key observations:

1. Savings are robust across both control and high-density enterprise-style tool surfaces.
2. Higher raw schema density drives disproportionately larger gains in compact-plane architectures.

This validates the hypothesis that description-heavy ecosystems benefit most from index-first control planes.

## 7. Enterprise Implications

### 7.1 Cost Engineering

Token reductions in the 58%–96% range materially affect operating budgets for high-frequency agent workflows, especially where context windows are repeatedly consumed by metadata. Hybrid capability search and SHA-256 ETag caching reduce context ingestion further toward asymptotic minimums.

### 7.2 Latency and Time-to-Action

Persistent upstream sessions reduce cold-start behavior and repeated negotiations. Compact indexes reduce frontend payload parsing overhead, shortening time to first useful action.

### 7.3 Reliability, Retry & Idempotency Governance

Normalized envelopes and explicit execution governance simplify agent retry policies:

- **Idempotency Deduplication**: Passing an `Idempotency-Key` ensures concurrent or retried identical requests attach to the same background execution worker, preventing duplicate side-effects (e.g. double database writes or duplicate API charges).
- **Retry Safety Classification**: Every response envelope contains explicit `"retry"` metadata (`classification`: `safe` | `unsafe` | `idempotent`), empowering orchestrators to retry network blips safely while halting dangerous duplicate side-effecting operations.
- **In-flight Operation Cancellation**: Malfunctioning or runaway agent loops can be aborted immediately via `POST /v1/operations/:id/cancel`.

### 7.4 Governance and Risk Posture

Centralized policy and redaction controls enable consistent enforcement across heterogeneous upstream servers and clients.

For regulated environments, this reduces policy drift and improves auditability.

### 7.5 Observability, Auditability & Tenant Context

Enterprise operations require verifiable evidence chains for agent actions across complex multi-tenant environments. Warmplane supports this with:

- structured JSON logs suitable for SIEM ingestion, carrying explicit `actor_id`, `grant_id`, `work_item_id`, and `operation_id` attributes,
- trace export into existing OTEL pipelines linked across HTTP headers and execution envelopes,
- stable error classes and trace-linked envelopes for post-incident reconstruction.

This reduces mean-time-to-understand during incidents and supports policy/compliance reviews with concrete telemetry artifacts.

## 8. Security and Trust Boundaries

Warmplane does not eliminate upstream risk; it concentrates control points. 

By upgrading to standard OAuth 2.1 authentication patterns and multi-tenant request context headers, Warmplane achieves major architectural security bounds:

- **Mitigation of Confused Deputy Attacks**: Employs Resource Indicators (RFC 8707) to bound issued access tokens to the exact canonical target server URI, preventing token replay attacks across services.
- **Defending against Authorization Mix-Ups**: Implements exact string-match issuer parameter verification (RFC 9207 / SEP-2468) during loopback redirects to eliminate session hijack risks.
- **Securing Public Clients (PKCE)**: Uses S256 PKCE verification to safeguard temporary redirect authorization codes from interception.
- **Tenant Context Isolation**: Threads `actor_id` and `grant_id` context through execution policies and audit trails, preventing cross-tenant request spoofing.
- **Idempotency Key Scope Boundaries**: Scopes deduplication state by caller identity and target operation to prevent cache pollution attacks across tenants.
- **Egress Filtering and SSRF Defenses**: Discovery and token requests utilize strict hostname and scheme validations to avoid server-side request forgery (SSRF).

Recommended deployment posture:

- bind local interfaces conservatively,
- use env-backed secrets or OAuth2 dynamic verification,
- enforce deny-by-default for destructive operations,
- separate read/write policy profiles by workload class,
- audit call paths via trace IDs and request context keys.

For HTTP/SSE upstreams, protocol versioning and auth headers should be explicit, verified, and monitored.

## 9. Positioning Against Alternatives

Warmplane differs from generic “gateway/proxy/router/mesh” framing by emphasizing two explicit properties:

1. **Warm sessions** (persistent upstream state)
2. **Control-plane compactness** (index-first, lazy detail expansion, hybrid search)

This pairing is what drives the empirical efficiency gains and deterministic behavior model.

## 10. Adoption Blueprint

### Phase 1: Sidecar Evaluation

- Deploy Warmplane adjacent to a subset of upstream MCP servers.
- Capture baseline token and latency measurements.
- Validate policy/redaction behavior.

### Phase 2: Contract Stabilization

- Introduce alias mappings as public capability IDs.
- Freeze client-facing IDs while allowing backend evolution.

### Phase 3: Enterprise Standardization

- Route all agent traffic through Warmplane profiles.
- Centralize timeout/error observability.
- Establish quarterly token-efficiency regression checks.

## 11. Limitations and Threats to Validity

- Results depend on upstream schema volume and shape.
- Some suites require credentials and controlled environments for full comparability.
- Tokenization model (`cl100k_base`) is representative but not universal across all providers.
- Throughput and latency under extreme concurrency were outside this initial token-focused study.

These do not negate findings but define scope.

## 12. Future Research Directions

1. **Streamable Tool Call Envelopes**
   - Extending normalized envelopes to support chunked streaming outputs for long-running capability operations.

2. **Profile-Aware Prompting Contracts**
   - Distinct compact surfaces for planner, executor, and auditor roles.

3. **Dynamic Policy Synthesis & Token Budget Enforcement**
   - Adaptive client-level token budget enforcement and dynamic allow/deny policy evaluation based on real-time agent spend.

4. **Cross-Provider Token Model Calibration**
   - Evaluate savings under multiple tokenizer regimes (e.g. tiktoken vs Llama/Claude tokenizers).

5. **Queueing and Concurrency Analysis**
   - Quantify warm-session behavior and idempotency deduplication under high parallel workloads.

## 13. Conclusion

Warmplane demonstrates that MCP scale problems are not solved by connectivity alone. They are solved by control-plane design.

By maintaining persistent upstream sessions and exposing compact, deterministic interaction surfaces, Warmplane yields measurable gains in token efficiency while improving operational behavior.

Empirical evaluation shows savings from approximately **58%** in control scenarios to **95%+** in high-density enterprise suites. For organizations scaling agent operations, this reframes tool architecture from “integration plumbing” to “economic and reliability infrastructure.”

The practical takeaway is simple:

> The cheapest token is the one never sent, and the most reliable tool call is the one executed through a deterministic control plane.

## Appendix A: Reproducibility

Run harness:

```bash
cargo run --manifest-path eval/token-efficiency/Cargo.toml -- \
  --suite-dir eval/token-efficiency/suites \
  --out-dir eval/token-efficiency/output
```

Primary artifacts:

- `eval/token-efficiency/output/summary.json`
- `eval/token-efficiency/output/report.md`
- `docs/research/TOKEN_EFFICIENCY_RESEARCH_REPORT.md`

## Appendix B: Terminology

- **Warm session**: an already established upstream MCP connection reused across requests.
- **Control plane**: the policy/contract layer that governs how clients access capabilities.
- **Data plane**: the execution path of actual tool/resource/prompt operations.
- **Index-first interface**: compact listing surface with deferred detail retrieval.
- **Hybrid search**: discovery engine combining BM25 lexical matching and FastEmbed vector embeddings.
- **Catalog digest**: SHA-256 state hash used for `ETag` conditional revalidation (`304 Not Modified`).
- **Request context**: structured execution tracking context (`operation_id`, `actor_id`, `grant_id`, `work_item_id`).
- **Idempotency key**: unique token passed by clients to deduplicate concurrent or replayed executions.
- **Retry classification**: structured classification (`safe`, `unsafe`, `idempotent`) returned in envelopes to guide orchestrator retry behavior.
