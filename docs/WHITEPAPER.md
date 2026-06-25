# Warmplane Whitepaper

## Warmplane: A Local Control Plane for Deterministic, Token-Efficient MCP Operations

### Abstract

Modern Model Context Protocol (MCP) deployments increasingly suffer from a structural inefficiency: repeated transmission and processing of large capability surfaces, even when only a small subset of tools/resources/prompts are used per task. This paper introduces **Warmplane**, a local control plane that maintains persistent upstream MCP sessions while exposing a compact, deterministic, policy-governed interface to clients.

Warmplane separates backend protocol richness from frontend interaction cost by presenting index-first capability discovery and on-demand schema expansion. In measured scenarios from the project evaluation harness, this approach reduced token footprint by **58.1%–58.2%** in a public filesystem control suite and **95.6%–95.8%** in an authenticated GitHub Copilot MCP suite. These improvements were achieved without sacrificing MCP compatibility or multi-transport interoperability.

We present the system architecture, transport model, policy/governance controls, error determinism model, and empirical evaluation methodology. We also discuss enterprise implications for latency, cost, reliability, and auditability, and outline future research directions for adaptive schema compaction and workload-aware orchestration.

## 1. Introduction

### 1.1 Problem Statement

As organizations operationalize AI agents, tool connectivity moves from demonstration to infrastructure. MCP has become a useful substrate for standardizing tool/resource/prompt access. However, in direct MCP client-server patterns, agent loops frequently overpay in two dimensions:

1. **Context overhead**: large metadata payloads are delivered repeatedly.
2. **Control fragmentation**: policy, retries, and error handling are inconsistently implemented across clients.

The result is avoidable token spend, higher startup latency, and reduced operational predictability.

### 1.2 Thesis

The central thesis of this whitepaper is:

> Agent systems should treat MCP integration as a two-plane architecture: rich protocol backend + compact control-plane frontend.

Warmplane implements this thesis by:

- keeping upstream MCP sessions warm and stateful,
- exposing a compact, stable interface for tools/resources/prompts,
- normalizing invocation and error envelopes,
- centralizing policy and redaction controls.

### 1.3 Contributions

This paper contributes:

1. A practical architecture for MCP session persistence plus compact interaction surfaces.
2. A deterministic execution model across CLI, HTTP, and MCP-native client modes.
3. A reproducible token-efficiency evaluation harness and measured baselines.
4. A governance model suitable for enterprise policy, observability, and risk controls.

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

Warmplane consists of four major components:

1. **Upstream Session Layer**
   - Connects to multiple MCP upstreams.
   - Supports stdio and streamable HTTP/SSE transports.
   - Keeps negotiated sessions persistent.

2. **Registry Layer**
   - Builds in-memory registries for capabilities, resources, prompts.
   - Applies alias mapping (`capabilityAliases`, `resourceAliases`, `promptAliases`).

3. **Policy and Envelope Layer**
   - Enforces allow/deny patterns across capability types.
   - Applies payload redaction keys in logs.
   - Standardizes response envelopes and error codes.

4. **Access Modes**
   - HTTP `/v1` facade.
   - CLI facade.
   - MCP server mode exposing lightweight synthetic tools and native resources/prompts methods.

### 3.2 Transport Model

Per upstream server, transport is inferred by strict configuration:

- `command` => stdio transport
- `url` => streamable HTTP transport

Exactly one selector must be set; ambiguous entries fail fast at startup.

For HTTP/SSE upstreams, Warmplane supports:

- `protocolVersion` header control,
- `allowStateless` behavior,
- custom headers,
- authentication schemas: static credentials (`bearer`, `basic`) and dynamic `oauth2` (OAuth 2.1 / OIDC) flows featuring PKCE (`S256`), dynamic server discovery (RFC 9728 & RFC 8414), scope accumulation during step-up challenges, and silent token refreshing via rotated refresh tokens.

### 3.3 Deterministic Call Model

Warmplane normalizes execution/read/get results to a stable envelope:

- `ok`
- `request_id`
- `trace_id`
- `data`
- `error`

Error classes are explicit and bounded (e.g., `UPSTREAM_TIMEOUT`, `UPSTREAM_ERROR`, `SERVER_UNREACHABLE`).

This deterministic contract is central to robust orchestration loops and reliable fallback behavior.

### 3.4 Observability Architecture

Warmplane implements observability as a first-class control-plane concern rather than an afterthought:

1. **Structured audit logs**
   - JSON-formatted runtime logs across daemon lifecycle and capability/resource/prompt operations.
   - Event records include contextual fields (e.g., server ID, capability/resource/prompt IDs, trace IDs).

2. **OpenTelemetry tracing**
   - Optional OTLP export path for enterprise trace backends.
   - Service metadata can be overridden with deployment-specific naming.
   - Local structured logs remain enabled even when OTEL export is active.

3. **Cross-surface correlation**
   - HTTP execution envelopes expose `trace_id`.
   - The same identifier can be used to correlate user-facing responses, logs, and distributed traces.

This design provides auditable and machine-parsable operational evidence while preserving deterministic API behavior.

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

Warmplane’s measured results fit this behavior: large $R_t/I$ ratios produce very high sustained savings.

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

Token reductions in the 58%–96% range materially affect operating budgets for high-frequency agent workflows, especially where context windows are repeatedly consumed by metadata.

### 7.2 Latency and Time-to-Action

Persistent upstream sessions reduce cold-start behavior and repeated negotiations. Compact indexes reduce frontend payload parsing overhead, shortening time to first useful action.

### 7.3 Reliability and Determinism

Normalized envelopes and bounded error classes simplify:

- retry policy,
- fallback branches,
- incident diagnosis,
- automation confidence.

### 7.4 Governance and Risk Posture

Centralized policy and redaction controls enable consistent enforcement across heterogeneous upstream servers and clients.

For regulated environments, this reduces policy drift and improves auditability.

### 7.5 Observability and Compliance Operations

Enterprise operations require verifiable evidence chains for agent actions. Warmplane supports this with:

- structured JSON logs suitable for SIEM ingestion,
- trace export into existing OTEL pipelines,
- stable error classes and trace-linked envelopes for post-incident reconstruction.

This reduces mean-time-to-understand during incidents and supports policy/compliance reviews with concrete telemetry artifacts.

## 8. Security and Trust Boundaries

Warmplane does not eliminate upstream risk; it concentrates control points. 

By upgrading to standard OAuth 2.1 authentication patterns, Warmplane achieves major architectural security bounds:

- **Mitigation of Confused Deputy Attacks**: Employs Resource Indicators (RFC 8707) to bound issued access tokens to the exact canonical target server URI, preventing token replay attacks across services.
- **Defending against Authorization Mix-Ups**: Implements exact string-match issuer parameter verification (RFC 9207 / SEP-2468) during loopback redirects to eliminate session hijack risks.
- **Securing Public Clients (PKCE)**: Uses S256 PKCE verification to safeguard temporary redirect authorization codes from interception.
- **Egress Filtering and SSRF Defenses**: Discovery and token requests utilize strict hostname and scheme validations to avoid server-side request forgery (SSRF).

Recommended deployment posture:

- bind local interfaces conservatively,
- use env-backed secrets or OAuth2 dynamic verification,
- enforce deny-by-default for destructive operations,
- separate read/write policy profiles by workload class,
- audit call paths via trace IDs.

For HTTP/SSE upstreams, protocol versioning and auth headers should be explicit, verified, and monitored.

## 9. Positioning Against Alternatives

Warmplane differs from generic “gateway/proxy/router/mesh” framing by emphasizing two explicit properties:

1. **Warm sessions** (persistent upstream state)
2. **Control-plane compactness** (index-first, lazy detail expansion)

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

1. **Adaptive Schema Compression**
   - Dynamic index detail levels based on prior turn usage.

2. **Profile-Aware Prompting Contracts**
   - Distinct compact surfaces for planner, executor, and auditor roles.

3. **Cross-Provider Token Model Calibration**
   - Evaluate savings under multiple tokenizer regimes.

4. **Queueing and Concurrency Analysis**
   - Quantify warm-session behavior under high parallel workloads.

5. **Policy Explainability**
   - Formal proofs or machine-checkable traces for allow/deny decisions.

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
