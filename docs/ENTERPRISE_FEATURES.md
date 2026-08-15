# Warmplane Enterprise Roadmap & Strategic Capabilities

This document outlines high-value enterprise features designed to establish Warmplane as the leading production-grade control plane for AI Agent & Model Context Protocol (MCP) ecosystems.

---

## 1. Executive Summary

As enterprises deploy autonomous agents in production environments, unmanaged MCP connectivity introduces significant operational, compliance, and security risks. Warmplane bridges local execution performance with centralized enterprise governance.

```
       ┌────────────────────────────────────────────────────────┐
       │                   Autonomous LLM Agents                │
       └───────────────────────────┬────────────────────────────┘
                                   │ HTTP / MCP Facade
                                   ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      WARMPLANE ENTERPRISE PLANE                        │
 │                                                                        │
 │  ┌─────────────────┐   ┌─────────────────┐   ┌──────────────────────┐  │
 │  │ 1. HITL Gate    │   │ 2. Scoped RBAC  │   │ 3. WORM Audit Logger │  │
 │  │ Interactive Call│   │ Multi-Tenant    │   │ Append-Only SQLite & │  │
 │  │ Approval Flow   │   │ Tokens & OIDC   │   │ SIEM / OTel Export   │  │
 │  └─────────────────┘   └─────────────────┘   └──────────────────────┘  │
 │  ┌─────────────────┐   ┌─────────────────┐   ┌──────────────────────┐  │
 │  │ 4. Resilience   │   │ 5. Token Caps   │   │ 6. Guardrails        │  │
 │  │ Circuit Breaker │   │ FinOps Budgeting│   │ PII Redaction &      │  │
 │  │ Process Watcher │   │ & Call Quotas   │   │ Strict Deny Policies │  │
 │  └─────────────────┘   └─────────────────┘   └──────────────────────┘  │
 └─────────────────────────────────┬──────────────────────────────────────┘
                                   │ Managed Sub-Processes / Network
                                   ▼
        ┌───────────────────────────────────────────────────────┐
        │        Upstream MCP Servers (Stdio / SSE / HTTP)      │
        │   [GitHub]   [Postgres]   [Filesystem]   [Docker]...  │
        └───────────────────────────────────────────────────────┘
```

---

## 2. Core Strategic Pillars

### 2.1 Human-in-the-Loop (HITL) Interceptor Engine
* **Problem:** Autonomous models executing mutating or destructive operations (e.g. dropping database tables, deleting Kubernetes namespaces, submitting PRs, triggering financial workflows) without human oversight.
* **Solution:** A policy-driven suspension engine that intercepts matching capability invocations, puts them in a pending state, alerts human operators via webhook/UI, and resumes or rejects execution based on cryptographically signed operator decisions.
* **Detailed Blueprint:** See [HITL Implementation Plan](HITL_PLAN.md).

---

### 2.2 Scoped Multi-Tenant RBAC & OIDC Integration
* **Problem:** Shared daemon instances expose identical capability surfaces to all callers regardless of permission tier (e.g. CI runner vs. intern agent vs. senior engineer).
* **Solution:**
  * **Role-Based Token Scopes (`X-Warmplane-Key` / JWT Bearer):** Define tenant/agent roles (`admin`, `read-only`, `devops`, `analyst`).
  * **Catalog Partitioning:** Dynamic catalog pruning where `GET /v1/capabilities` and `GET /v1/resources` automatically filter items based on the caller's verified claims.
  * **Enterprise SSO / OIDC:** Seamless integration with Okta, Microsoft Entra ID, and Auth0 for Control Deck authentication and API authorization.

---

### 2.3 Write-Once-Read-Many (WORM) Audit Logging & SIEM Export
* **Problem:** Compliance frameworks (SOC2 Type II, ISO 27001, HIPAA, FedRAMP) require non-repudiable logs of every tool invocation and sensitive data transfer.
* **Solution:**
  * **Append-Only SQLite / Parquet Log:** Records `request_id`, `actor_id`, `grant_id`, `timestamp`, `capability_id`, `sanitized_args`, `execution_latency_us`, and `status`.
  * **Native SIEM Streaming:** Structured telemetry pipeline exporting audit events over OpenTelemetry (OTLP gRPC), Splunk HEC, Datadog, or AWS CloudWatch.
  * **Compliance UI:** Searchable timeline viewer in the Control Deck with JSON/CSV export capabilities.
  * **Detailed Blueprint:** See [WORM Audit & SIEM Implementation Plan](plans/WORM_SIEM_PLAN.md).

---

### 2.4 Upstream Resiliency, Circuit Breaking & Process Supervision
* **Problem:** Fragile third-party MCP servers hang, crash, rate-limit, or leak memory, degrading or deadlocking LLM reasoning loops.
* **Solution:**
  * **Process Supervisor:** Automatic health check pings (`ping` / `list_tools`) and supervisor restarts with exponential backoff for crashed stdio children.
  * **Circuit Breakers:** Configurable failure thresholds (e.g. 3 consecutive timeouts trips the circuit for 30s) returning fast-fail errors so agents can adapt rather than hang.
  * **Automatic Fallbacks:** Declarative fallback chaining (e.g. query local mirror if primary DB connection fails).

---

### 2.5 FinOps Token Budgeting & Payload Safety Caps
* **Problem:** Massive MCP resource responses (e.g. a 20MB log file or unfiltered DB dump) blow out context windows, spike LLM inference bills, and cause out-of-memory errors.
* **Solution:**
  * **Hard Payload Truncation:** Configurable response size caps (e.g., max 50KB per tool return) with automated structural summarization.
  * **Rate Limiting & Cost Tracking:** Rate limits by `actor_id` (e.g., max 60 calls/min) and live calculation of prompt token savings achieved via Warmplane's compact facades.

---

## 3. Prioritized Implementation Roadmap

| Milestone | Feature Pillar | Status | Impact | Complexity | Target Audience |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **Phase 1** | **Human-in-the-Loop (HITL) Workflows** | ✅ Released (v0.8.0) | 🟢 Critical | Medium | Security & Compliance Officers |
| **Phase 2** | **WORM Audit Trail & SIEM Export** | ✅ Released (v0.12.0) | 🟢 High | Low-Medium | SecOps / Enterprise Auditors |
| **Phase 3** | **Multi-Tenant RBAC & OIDC Auth** | 🔄 Planned | 🟡 High | Medium | Platform Engineers & SREs |
| **Phase 4** | **Circuit Breaking & Supervision** | 🔄 Planned | 🟡 High | Medium | AI Platform Teams |
| **Phase 5** | **FinOps Budgeting & Quota Caps** | 🔄 Planned | 🔵 Medium | Low | FinOps & Engineering Leads |
