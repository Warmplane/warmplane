RFC-0042: Warmplane Next-Generation Product Roadmap — From Tool Connectivity to Autonomous Governance and Stateful Recourse

Document Identifier: RFC-0042
Document Title: Warmplane Next-Generation Product Roadmap: Autonomous Governance, Dynamic Reranking, and Distributed State Recourse
Author: Warmplane Systems Architecture and Engineering Group
Document Status: Proposed / Review
Target Release Horizon: v0.34.0 – v1.0.0
Initial Specification Date: November 2026

1. Context, Motivation, and Core Architectural Thesis

The initial wave of Model Context Protocol (MCP) implementations established foundational protocol interoperability across heterogeneous environments. By standardizing client-server exchanges over JSON-RPC 2.0—utilizing local standard input/output (stdio) and remote Streamable HTTP—the protocol enabled large language models to interact with external tools, contextual resources, and prompt templates through a unified specification. However, deploying autonomous agents in production environments has demonstrated that raw connectivity alone does not create enterprise-grade infrastructure. Direct client-to-server topologies expose agent workflows to severe economic, reliability, and security bottlenecks, including prompt token bloat, description overhead, unmanaged failure cascades, un-idempotent mutations, and an absence of stateful recourse.

In direct integration architectures, agent frameworks repeatedly ingest expansive capability definitions on every conversational turn, regardless of whether those tools are relevant to the immediate task. Empirical evaluations indicate that this "Description Tax" constitutes the primary component of operational token consumption, crowding context windows and causing model attention to deteriorate. Concurrently, standard reverse proxies and API gateway extensions treat MCP traffic merely as an additional HTTP or JSON-RPC stream. These generic layers remain payload-blind to tool-level mutations, side effects, and state drift, offering no native mechanisms to intercept, hold, or reverse destructive operations.

Warmplane resolves these operational challenges by implementing a two-plane architecture. Under this model, full protocol richness and session state are preserved on the backend, while a compact, deterministic, policy-governed facade is exposed to the language model. This document establishes the strategic architectural roadmap for Warmplane as it evolves from a high-efficiency persistent session multiplexer into an enterprise-wide control plane for autonomous tool execution.

The architectural thesis guiding this specification rests upon three core requirements:

1. Interface Efficiency Over Eager Exposition: Language models must receive concise, high-signal capability indexes during initial turns, expanding into detailed parameter schemas only on demand.
2. Deterministic Execution Over Blind Mutation: Tool execution must provide crash-resilient idempotency, sub-microsecond state revalidation, and tamper-evident cryptographic auditability for all side effects.
3. Stateful Recourse Over Passive Observation: The runtime must govern not merely the initial dispatch of an action, but its temporal hold, execution safety, and subsequent reversal or compensation when workflows fail.

2. Current State Retrospective: The v0.33.0 Baseline

Warmplane v0.33.0 established a robust runtime foundation implemented in pure Rust, offering low-latency session persistence, policy enforcement, and reversible execution mechanics across three unified deployment interfaces: a native MCP stdio proxy, a background HTTP REST daemon with an embedded Control Deck web dashboard, and an in-process embedded Rust engine (EmbeddedWarmplane).

Performance Profile and Token Optimization Metrics

By decoupling upstream schema complexity from frontend agent interactions, Warmplane converts the economic profile of multi-turn agent loops from linear metadata accumulation into an asymptotic zero-cost model for non-mutating turns. The evaluation harness validates significant prompt token reductions across distinct upstream environments:

Evaluation Suite	Workload Profile	Raw MCP Consumption	Warmplane Facade Consumption	Measured Token Savings
Public Filesystem (No-Auth Control)	Discovery Pull	2,552 tokens	1,066 tokens	58.2%
Public Filesystem (No-Auth Control)	5-Turn Tool Loop	12,760 tokens	5,349 tokens	58.1%
Public Filesystem (No-Auth Control)	10-Turn Mixed Loop	25,520 tokens	10,669 tokens	58.2%
GitHub Copilot MCP (Authenticated)	Discovery Pull	54,715 tokens	2,386 tokens	95.6%
GitHub Copilot MCP (Authenticated)	5-Turn Tool Loop	260,005 tokens	11,173 tokens	95.7%
GitHub Copilot MCP (Authenticated)	10-Turn Mixed Loop	547,150 tokens	22,895 tokens	95.8%

These efficiency gains are supported by micro-benchmarks captured via Criterion on Apple Silicon, verifying that comprehensive architectural governance introduces minimal system overhead:

* 50.4 nanoseconds: Catalog If-None-Match cache revalidation returning HTTP 304 Not Modified through zero-copy quote-trim validation.
* 1.98 nanoseconds: Direct header map lookup miss when conditional validation headers are absent.
* 159.8 nanoseconds: Exactly-once idempotent cache-hit deduplication.
* 550.7 microseconds: Contention handling across 16 threads executing 800 concurrent operations.
* 126.0 microseconds: Incremental SHA-256 version hashing across 500 capabilities.
* 372.1 microseconds: Zero-allocation lexical tag scanning across 1,000 tool definitions.
* 15.9 microseconds: Filtered hybrid capability search combining BM25 scoring and FastEmbed reciprocal rank fusion.

Enterprise Compliance and Stateful Recourse

Warmplane v0.33.0 introduced critical compliance and governance primitives that distinguish it from standard routing layers:

* Cryptographic WORM Audit Trails: An append-only ledger employing linear SHA-256 cryptographic hash chaining where each event digest incorporates the preceding event hash and a canonical JSON representation. Real-time verification (GET /v1/audit/verify) traverses the chain in linear time, while an asynchronous queue streams batched events to Splunk HTTP Event Collector (HEC) and HTTP webhooks.
* Reversible AI and Work-Item Rollback: Capability aliases declare execution reversibility contracts across four formal classes:
  * Delayable: Holds approved calls in a durable coordinator across a configurable holdWindowSecs (defaulting to 30 seconds), allowing operators to abort executions before upstream transmission.
  * Reversible: Captures pre-execution state (preSnapshot) as SHA-256 content-addressed artifacts and applies deterministic inverse parameter templates.
  * Compensable: Executes post-action semantic compensation for non-invertible operations, bound by a mandatory rolling 24-hour dailyLimit that routes volume overages to Human-in-the-Loop review.
  * Irreversible: Explicitly flags non-reversible operations to prevent agents or operators from assuming rollback safety.
* Pre-Execution Drift Detection (preUndoCheck): Stores expected post-execution state hashes upon forward completion and re-reads upstream resources immediately prior to inverse execution. If external mutations cause a hash mismatch, Warmplane aborts the inverse with UNDO_STATE_DRIFT, preserving the armed entry for administrative triage.
* Batch Rollback Mechanics: In multi-step batch executions (POST /v1/tools/batch_call), setting autoRollbackOnFailure: true halts the pipeline upon encountering a failed step and automatically invokes inverse compensation for preceding successful steps in reverse chronological order.

3. The Empirical Imperative: Lessons from the HumanMCP Benchmark

The technical initiatives defined in this roadmap are guided directly by empirical findings from the HumanMCP benchmark (arXiv:2602.23367). The study evaluated efficiency-optimized models—including Google Gemini 2.0 Flash, OpenAI GPT-4o-mini, and Anthropic Claude 3.5 Haiku—across realistic, persona-conditioned queries to measure tool retrieval and selection accuracy as catalog search spaces expand.

The benchmark established three critical conclusions that dictate our engineering priorities:

The 100-Tool Cliff and the 2,000-Tool Collapse

As context windows scale from 10 to 100 tools, all evaluated models experience an average accuracy degradation of approximately 10%. Gemini 2.0 Flash drops from 98.4% to 88.2%, GPT-4o-mini declines from 98.2% to 88.2%, and Claude 3.5 Haiku drops from 97.8% to 88.6%.

When the candidate pool is expanded to 2,000 tools—a realistic footprint for enterprise environments connecting multiple database, code repository, and customer management systems—Gemini’s selection accuracy collapses to 65.0%. As demonstrated in the scaling evaluations, intermediate thresholds reveal a continuous downward trajectory: 87.4% accuracy at 500 tools, declining to 78.4% at 1,000 tools, before dropping sharply to 65.0% at 2,000 tools. High-noise distractors and dense schema descriptions overwhelm the model's planning attention, inducing severe context drift.

The Failure of Naive Keyword Matching

Crucially, the benchmark demonstrated that naive keyword retrieval actively degrades selection performance compared to raw context processing. When researchers employed BM25 to filter the 2,000-tool catalog down to a top-10 candidate list, the low retrieval hit rate (61.11%) introduced noisy, incomplete candidate sets. Consequently, Gemini's downstream selection accuracy dropped to 59.0%—substantially worse than the unassisted 65.0% baseline.

The Semantic Retrieval Recovery

In contrast, dense semantic retrieval utilizing a SentenceTransformer achieved an 87.6% Top-10 Hit Rate, lifting Gemini's final accuracy back to 76.0% (an immediate 11 percentage point recovery over raw context processing). TF-IDF achieved an intermediate 75.15% hit rate and 68.0% accuracy.

These empirical realities confirm that exposing large tool catalogs to LLMs is fundamentally anti-scalable. The solution is neither asking the LLM to process a 2,000-tool prompt nor relying on brittle keyword matching. Instead, the control plane must autonomously maintain a dense, semantic indexing layer that dynamically injects a tailored, low-noise candidate menu into the agent's immediate context.

4. Competitive Landscape and Strategic Positioning

The enterprise infrastructure ecosystem has bifurcated into AI Gateways (managing model routing, prompt security, and LLM inference) and MCP Gateways (governing agent-to-tool execution). Within this second category, distinct operational philosophies have emerged across the industry.

Competitive Dimension	Enterprise API Gateways (Kong AI MCP Proxy)	Managed SaaS Action Hubs (Composio, Zapier)	Dedicated Governance Proxies (MCP Manager, Lunar.dev)	High-Performance Network Data Planes (AgentGateway, Envoy)	Warmplane Control Plane
Primary Architecture	Reverse proxy plugin extending existing L7 API gateway	Multi-tenant cloud SaaS with pre-built connectors	Workstation and cloud proxy for MCP traffic	Line-rate L7/L8 network proxy binary (C++ / Go)	Pure Rust persistent daemon, embedded engine, and local control plane
Context Strategy	Eager-loads full OpenAPI/JSON-RPC schemas into context	Hosted execution; static schema injection	Static catalog allowlisting	Raw wire-format body parsing and forwarding	Lazy "index first, detail on demand" facade (~58%–96% token savings)
Stateful Recourse	Stateless; retries based on HTTP error codes	Custom webhooks on task failure	Informational read/write annotations	L4/L7 circuit breaking and connection timeouts	Formal Reversibility Contracts (Hold, Snapshot, Compensate, Drift-Checked Undo)
Audit Verification	Standard access logs; optional SIEM forwarders	CSV/JSON platform export	Centralized audit logs with trace correlation	OpenTelemetry gRPC distributed traces	Linear SHA-256 hash-chained WORM logs with mathematical verification
Execution Locality	Centralized Kubernetes clusters or cloud	Managed third-party cloud runtime	Workstation Docker containers with WireGuard	Edge proxies, service mesh sidecars	Hybrid: Embedded Rust, local developer daemon, and cluster control plane

Structural Differentiators

Warmplane maintains distinct technical advantages over competing approaches:

1. Against Enterprise API Gateways (Kong): Kong excels at translating existing REST services into MCP tools via OpenAPI schemas. However, it treats MCP as static HTTP routes, lacks managed credential isolation from the LLM, does not mitigate prompt bloat, documents no rug-pull protection for changed tool docstrings, and explicitly marks AI Guardrails as unsupported on MCP traffic. Warmplane provides native token optimization, deterministic envelopes, and active reversibility.
2. Against Managed Action Platforms (Composio): Composio provides extensive SaaS catalog breadth with over 1,000 connectors. However, it operates as a proprietary, multi-tenant cloud platform requiring teams to entrust corporate credentials to an external provider. Warmplane delivers a local, self-hosted runtime that keeps credentials within corporate perimeters while governing local stdio processes.
3. Against Specialized Workstation Governance (MCP Manager): MCP Manager provides workstation governance through Docker containers and WireGuard tunnels with Presidio PII filtering. However, it is delivered as a closed, US-hosted SaaS without VPC or self-hosted builds, and lacks stateful rollback mechanics. Warmplane provides native reversibility contracts, sub-microsecond in-memory caching, and open-source deployment flexibility.
4. Against L7/L8 Network Proxies (AgentGateway, Envoy): Envoy and AgentGateway deliver line-rate body parsing, token-aware rate limiting, and connection pooling. However, they operate strictly at the network transport layer, offering no application-level capability facades, parameter signatures, or pre-snapshot state tracking. Warmplane acts as the Layer 8/9 application control plane sitting above or alongside these network proxies.

5. Core Architectural Initiatives

To build upon its architectural advantages, Warmplane will execute five major technical initiatives across the v0.34.0 to v1.0.0 release cycle.

Initiative 1: Zero-Code OpenAPI / REST-to-MCP Ingestion Engine

Context and Problem Statement

Enterprise adoption is frequently delayed because organizations maintain thousands of internal microservices documented via OpenAPI 3.0/3.1 specifications, but possess few native MCP servers. Requiring platform teams to write, deploy, and maintain custom Node.js or Python MCP wrapper servers introduces excessive operational maintenance and slows deployment velocity.

Technical Architecture and Implementation

Warmplane will introduce native, zero-code OpenAPI ingestion directly within the mcpServers configuration subsystem:

{
  "mcpServers": {
    "billing-service": {
      "type": "openapi",
      "spec": "https://api.internal.corp/billing/v3/openapi.json",
      "baseUrl": "https://api.internal.corp/billing/v3",
      "auth": {
        "type": "bearer",
        "tokenEnv": "INTERNAL_BILLING_JWT"
      },
      "includeOperations": ["getInvoice", "createCreditMemo", "voidTransaction"],
      "compactSignatures": true
    }
  }
}


* Dynamic In-Memory Synthesis: The supervisor parses the target OpenAPI document during startup, dynamically instantiating internal worker actors that map declared operationId items into MCP capabilities.
* Deterministic Parameter Mapping: Outbound capability calls map JSON-RPC arguments into path parameters, query strings, and request bodies according to the OpenAPI schema.
* Automatic Compact Signatures: Generates concise call signatures (e.g., createCreditMemo(customer_id, amount, [reason])) derived directly from schema required-field arrays.
* Context Distillation Compatibility: Ingested endpoints natively inherit Warmplane's execution modifiers (_jsonpath, _limit_lines, _truncate_bytes), allowing agents to query large REST endpoints without overflowing context windows.

Initiative 2: Autonomous Reversibility Contract Synthesis and Interactive Recorder

Context and Problem Statement

Warmplane's v0.33.0 Reversible AI engine provides unmatched stateful recourse, but manually drafting snapshot JSONPaths, inverse argument templates, and drift checks in mcp_servers.json creates configuration overhead for platform engineers.

Technical Architecture and Implementation

Warmplane will automate reversibility contract authoring through static inference and interactive test recording:

1. Inferred CRUD Inverses: When fronting REST microservices or relational databases, Warmplane automatically derives inverse contract templates:
  * A creation operation (POST /resources) is paired with an inverse deletion (DELETE /resources/${result.id}).
  * A modification operation (PUT /resources/{id}) is automatically paired with a pre-execution snapshot (GET /resources/${args.id}) and an inverse modification restoring ${snapshot}.
2. Interactive Contract Recorder in Control Deck:
  * Platform engineers toggle "Record Reversibility Contract" within the MCP Playground.
  * Warmplane executes a safe probe invocation against a configured testTarget server.
  * The runtime records the pre-state read, the forward invocation, and the resulting state, auto-generating a validated, schema-checked YAML/JSON contract draft for immediate administrative promotion.

Initiative 3: Tool-Result Sanitization and Fingerprinted Description Pinning

Context and Problem Statement

While Warmplane currently redacts outbound log events, sensitive personally identifiable information (PII) returned by upstream tools can flow back into downstream LLM context windows. Additionally, malicious or compromised third-party servers can quietly modify tool descriptions between turns—an attack vector known as tool poisoning or rug-pull updates—inducing agents to execute unintended operations.

Technical Architecture and Implementation

1. Inline Response Sanitization Engine:
  * A high-throughput inline scrubber intercepts tool outputs before returning them to the agent.
  * Configured under policy.resultSanitization, it applies local pattern matching or lightweight ONNX models to scrub Social Security numbers, credit card numbers, and authorization tokens.
  * Sanitized occurrences emit structured result_sanitized records to the WORM audit trail while leaving operational data intact.
2. SHA-256 Tool Description Pinning:
  * Upstream tool docstrings and parameter schemas are hashed into SHA-256 fingerprints upon initial administrator approval.
  * If an upstream server mutates a docstring during a runtime discovery poll or notifications/tools/list_changed event, Warmplane trips a TOOL_DESCRIPTION_MUTATED policy lock.
  * The modified tool is immediately disabled, a signed notification is dispatched via ChatOps webhooks, and the mutation event is committed to the tamper-evident audit ledger.

Initiative 4: Transparent Just-in-Time (JIT) Semantic Tool Reranking

Context and Problem Statement

The HumanMCP benchmark established that when candidate tool catalogs expand into hundreds or thousands of definitions, model selection accuracy collapses unless tools are filtered down to a tight, semantically relevant shortlist. Requiring the agent to self-discover tools via capability_search introduces multi-turn latency and relies on the model knowing what terms to query.

Technical Architecture and Implementation

Warmplane will introduce transparent Just-in-Time semantic tool reranking directly within its proxy layer:

1. Request Interception: When an agent issues a standard MCP tools/list request, Warmplane inspects the user query or the trailing conversation turn.
2. FastEmbed ONNX Vector Filtering: The runtime executes sub-millisecond dense cosine similarity scoring across the full capability catalog, matching the query against pre-computed tool embeddings.
3. Dynamic Shortlist Injection: Instead of returning an overwhelming catalog of 2,000 schemas, Warmplane dynamically returns only the top 10 relevant candidate schemas in the tools/list response.
4. Context Shielding: Downstream language models receive a compact, high-relevance tool surface, bypassing the long-context degradation curve and achieving the 76.0% accuracy threshold validated by the HumanMCP benchmark without requiring client-side modifications.

Initiative 5: Distributed Enterprise Topology (Hub-and-Spoke Mesh)

Context and Problem Statement

Enterprises face operational friction between centralized cloud infrastructure and local developer environments. Engineers run local MCP servers on their workstations (via Cursor, Claude Code, or VS Code), creating unmonitored "shadow MCP" setups. Simultaneously, centralized agents run in VPC clusters. Organizations need unified policy, approval, and audit visibility across both environments without opening inbound workstation firewall ports.

Technical Architecture and Implementation

1. Warmplane Hub (Enterprise VPC):
  * A multi-tenant daemon deployed within the corporate VPC hosting the central Control Deck UI, global RBAC policies, Splunk SIEM export pipelines, and aggregated WORM audit ledgers.
  * Coordinates enterprise-wide approval queues and tracks compliance across all connected nodes.
2. Warmplane Spoke (Workstation Daemon):
  * A lightweight CLI mode (warmplane connect --hub https://warmplane.corp) running on developer laptops.
  * Manages local stdio processes (e.g., local Git, filesystem, and SQLite tools) and establishes an outbound, mutual-TLS or WireGuard tunnel to the Hub without opening inbound ports.
3. Unified Execution Lineage:
  * Developer IDEs connect to the local Spoke listener (:9191).
  * The Spoke evaluates policies synchronized from the Hub, prompts for approvals via local ChatOps cards, and asynchronously streams signed cryptographic WORM audit events to the central Hub.

6. Implementation Phases and Target Milestones

The proposed initiatives are organized into four sequential delivery milestones across the v0.34.0 to v1.0.0 horizon, ensuring backward compatibility, security validation, and progressive delivery:

Milestone	Target Horizon	Primary Architectural Deliverables	Core Success Metrics
Phase 1: Microservice Acceleration	v0.34.0 (Q4 2026)	• OpenAPI 3.0/3.1 ingestion engine
• Inferred CRUD inverse contract heuristics
• Interactive Reversibility Contract Recorder in Control Deck UI	• Zero-code ingestion of standard OpenAPI specs
• Sub-millisecond route generation
• 100% test coverage on generated inverse templates
Phase 2: Advanced Data Protection	v0.35.0 (Q1 2027)	• Inline tool-result PII scrubbing engine
• SHA-256 tool description pinning against rug pulls
• Asynchronous ChatOps approval receipts (Slack, Teams)	• Bounded PII scanning latency under 5ms
• Immediate fail-closed lock on modified tool docstrings
• Zero false-positive drops on authorized mutations
Phase 3: Autonomous Interface Optimization	v0.36.0 (Q2 2027)	• Transparent Just-in-Time (JIT) semantic tool reranking
• Dynamic Top-10 schema injection via FastEmbed ONNX
• Chunked streaming execution envelopes	• Retrieval accuracy exceeding 75% on 2,000-tool pools
• Reranking overhead under 2ms per turn
• Validated against HumanMCP benchmark
Phase 4: Enterprise Hub-and-Spoke Mesh	v1.0.0 (Q3 2027)	• Distributed Warmplane Hub cluster architecture
• Workstation Spoke dial-out tunnels
• SOC 2 Type II and HIPAA pre-packaged compliance audit packs	• Unified audit visibility across 1,000+ workstation Spokes
• Zero inbound firewall ports required on developer laptops
• Full mathematical chain integrity across distributed nodes

7. Security, Compliance, and Migration Governance

All features introduced in this roadmap are governed by strict security and backward-compatibility invariants:

1. Fail-Closed Execution Posture: If a PII scrubber, description-pinning check, or pre-snapshot read encounters an unhandled runtime error, Warmplane fails closed, halting the execution and emitting a POLICY_INTERCEPTION_FAILURE record to the audit log.
2. Cryptographic Chain Invariance: All new event types (such as OpenAPI route registration, description mutation locks, and inverse rollbacks) must adhere to canonical JSON serialization rules to maintain uninterrupted SHA-256 linear hash-chain integrity.
3. Protocol Backward Compatibility: Complete adherence to MCP version 2026-07-28 is maintained as the default, while configurable fallback support for legacy session-stable protocol version 2025-11-25 remains supported across all transport adapters.
4. Zero Upstream Modifications: All capabilities—including OpenAPI ingestion, PII scrubbing, dynamic reranking, and work-item rollback—operate entirely at the control-plane boundary without requiring code modifications to upstream microservices or third-party MCP servers.
