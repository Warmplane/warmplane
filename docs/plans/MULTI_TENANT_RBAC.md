# Multi-Tenant Role-Based Access Control (RBAC) & Catalog Partitioning — Implementation Plan

This document details the architectural design, security model, configuration schema, runtime token verification, catalog filtering logic, and phased implementation plan for adding Multi-Tenant RBAC & Catalog Partitioning to Warmplane.

---

## 1. Objectives & Core Principles

1. **Deterministic Catalog Partitioning (`Least Privilege by Default`):**
   - Callers discover only capabilities, resources, and prompts matching their authorized role/grant scopes.
   - `GET /v1/capabilities`, `GET /v1/resources`, `GET /v1/prompts`, and hybrid search (`POST /v1/capabilities/search`) dynamically prune items invisible to the caller's role.
   - Capability detail requests (`GET /v1/capabilities/:id`) and executions (`POST /v1/tools/call`) return `403 FORBIDDEN` / `CAPABILITY_UNAUTHORIZED` if the capability is outside the caller's allowed scope.

2. **Scoped Authentication Tokens & Claims:**
   - Support both static API tokens mapped to roles in `mcp_servers.json` (for local scripts, daemons, CI runners) and cryptographic JWT/OIDC Bearer tokens (for enterprise SSO/IDP integrations like Okta, Entra ID, Auth0).
   - Resolve tenant identity (`tenant_id`), caller role (`role`), identity (`actor_id`), and explicit grant scopes (`scopes`).

3. **Per-Role Policy Isolation & HITL Customization:**
   - Define distinct `allow`, `deny`, and `require_approval` rules per role (e.g. `analyst` can run `db.query` directly; `devops` requires HITL approval for `docker.remove_*`; `intern` is denied mutating tools entirely).

4. **WORM Audit & Non-Repudiation Binding:**
   - Automatically bind verified caller metadata (`tenant_id`, `actor_id`, `role`, `grant_id`) into every WORM audit record and SIEM event stream.

5. **Zero Performance Degradation:**
   - Fast-path cached role resolution and sub-millisecond in-memory wildcard scope evaluation without disk or external network bottlenecks on every tool call.

---

## 2. Architectural Design & Request Flow

```
                ┌──────────────────────────────────────────────────┐
                │        Callers (CI Runner, Agent, Senior Eng)    │
                └─────────────────────────┬────────────────────────┘
                                          │
                                          │ Authorization: Bearer <Token/JWT>
                                          │ OR X-Warmplane-Key: <Key>
                                          ▼
                ┌──────────────────────────────────────────────────┐
                │          Security & RBAC Guard Middleware        │
                │                                                  │
                │  1. Authenticate Token (Static Map or JWT Verify)│
                │  2. Resolve TenantContext:                       │
                │     • tenant_id: "acme-corp"                     │
                │     • role: "data-analyst"                       │
                │     • actor_id: "agent-007"                      │
                │     • effective_policy: PolicyConfig             │
                └─────────────────────────┬────────────────────────┘
                                          │
                                          │ Injects TenantContext into Request Extensions
                                          ▼
         ┌────────────────────────────────┬────────────────────────────────┐
         │                                │                                │
         ▼                                ▼                                ▼
┌──────────────────┐            ┌──────────────────┐            ┌──────────────────┐
│ Catalog Endpoints│            │ Tool Executions  │            │ WORM Audit Trail │
│ • /capabilities  │            │ • /tools/call    │            │ Records:         │
│ • /resources     │            │ • /batch_call    │            │ • tenant_id      │
│ • /search        │            │                  │            │ • actor_id       │
│                  │            │ Evaluates role   │            │ • role           │
│ Prunes items not │            │ allow/deny &     │            │ • grant_id       │
│ allowed by role  │            │ HITL rules       │            │ • status         │
└──────────────────┘            └──────────────────┘            └──────────────────┘
```

---

## 3. Data Models & Configuration Schema

### 3.1 `mcp_servers.json` Configuration (`RbacConfig`)

Extend `McpConfig` with an optional `rbac` block:

```json
{
  "port": 9090,
  "rbac": {
    "enabled": true,
    "defaultRole": "anonymous",
    "jwt": {
      "issuer": "https://auth.acme.corp/",
      "audience": "warmplane-control-plane",
      "jwksUrl": "https://auth.acme.corp/.well-known/jwks.json",
      "roleClaim": "https://warmplane.io/role",
      "tenantClaim": "https://warmplane.io/tenant_id"
    },
    "tokens": {
      "wp_live_admin_secret_key": {
        "role": "admin",
        "tenantId": "org-core",
        "actorId": "admin-automation",
        "description": "Cluster management automation key"
      },
      "wp_live_analyst_key": {
        "role": "analyst",
        "tenantId": "org-analytics",
        "actorId": "bi-agent",
        "description": "Read-only analytics agent key"
      }
    },
    "roles": {
      "admin": {
        "description": "Full access to all capabilities, servers, and configurations",
        "allow": ["*"],
        "deny": [],
        "requireApproval": []
      },
      "analyst": {
        "description": "Read-only database queries and analytics context",
        "allow": ["db.query", "db.explain", "fs.read_*", "prompt.analytics_*"],
        "deny": ["*.write_*", "*.delete_*", "*.drop_*", "docker.*"],
        "requireApproval": ["db.query_large_dataset"],
        "redactKeys": ["password", "ssn", "credit_card", "secret"]
      },
      "devops": {
        "description": "Infrastructure operations with mandatory HITL approval on mutations",
        "allow": ["docker.*", "k8s.*", "fs.*", "github.*"],
        "deny": ["*.destroy_cluster"],
        "requireApproval": ["docker.remove_*", "docker.kill_*", "k8s.delete_*"]
      },
      "anonymous": {
        "description": "Unauthenticated or unprivileged fallback role",
        "allow": ["prompt.public_*"],
        "deny": ["*"]
      }
    }
  }
}
```

### 3.2 Internal Rust Types (`src/rbac/models.rs`)

```rust
// Rust guideline compliant 2026-08-19

//! Multi-tenant RBAC models and security claim definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for multi-tenant Role-Based Access Control.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct RbacConfig {
    /// Whether RBAC enforcement is active.
    #[serde(default)]
    pub enabled: bool,
    /// Default fallback role if no valid credentials supplied.
    #[serde(default = "default_fallback_role")]
    pub default_role: String,
    /// Optional JWT verification configuration for OIDC/SSO integrations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
    /// Map of static API tokens to role assignments.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tokens: HashMap<String, TokenAssignment>,
    /// Role definitions configuring specific capability policies.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub roles: HashMap<String, RolePolicyConfig>,
}

fn default_fallback_role() -> String {
    "anonymous".to_string()
}

/// Token binding assignment.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TokenAssignment {
    /// Assigned role name.
    pub role: String,
    /// Tenant or organization identifier.
    #[serde(default, rename = "tenantId", skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// Optional fixed actor identifier for audit trail.
    #[serde(default, rename = "actorId", skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    /// Human readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Role capability and execution policy rules.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct RolePolicyConfig {
    /// Description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Capability patterns allowed.
    #[serde(default)]
    pub allow: Vec<String>,
    /// Capability patterns denied.
    #[serde(default)]
    pub deny: Vec<String>,
    /// Capability patterns requiring operator approval.
    #[serde(default, rename = "requireApproval")]
    pub require_approval: Vec<String>,
    /// Sensitive keys to redact for this role.
    #[serde(default, rename = "redactKeys")]
    pub redact_keys: Vec<String>,
}

/// Verified tenant caller context attached to HTTP request extensions.
#[derive(Clone, Debug, PartialEq)]
pub struct TenantContext {
    pub tenant_id: String,
    pub role: String,
    pub actor_id: Option<String>,
    pub grant_id: Option<String>,
    pub effective_policy: crate::daemon::Policy,
}
```

---

## 4. Catalog Partitioning & Execution Enforcement

### 4.1 Capability & Resource Catalog Filtering
When `handle_list_capabilities`, `handle_list_resources`, or `handle_list_prompts` execute:
1. Extract `TenantContext` from Axum request extensions (fallback to configured `default_role` or global policy).
2. Filter the in-memory map using `tenant_context.effective_policy.allows(&capability_id)`.
3. Compute partition-aware cache headers (`ETag: "cat_<version>_<role>"` and `Vary: Authorization, X-Warmplane-Key`).

```rust
// Catalog filtering snippet
let caps_guard = state.capabilities.read().await;
let capabilities: Vec<_> = caps_guard
    .iter()
    .filter(|(id, _)| tenant_context.effective_policy.allows(id))
    .map(|(id, meta)| json!({
        "id": id,
        "summary": meta.summary,
        "server": meta.server,
        "tool": meta.tool,
        "tags": meta.tags,
    }))
    .collect();
```

### 4.2 Hybrid Search Pruning (`handle_search_capabilities`)
Pass `tenant_context.effective_policy` directly to the `HybridSearchEngine` so candidate embeddings and lexical matches outside the tenant's authorization boundary are discarded before ranking and score truncation.

### 4.3 Execution Boundary Enforcement (`handle_call_capability`)
1. Before checking circuit breakers or dispatching to upstream MCP workers, test:
   ```rust
   if !tenant_context.effective_policy.allows(&capability_id) {
       return error_envelope(
           StatusCode::FORBIDDEN,
           &trace_id,
           "CAPABILITY_UNAUTHORIZED",
           &format!("Role '{}' is not authorized to execute capability '{}'", tenant_context.role, capability_id),
       );
   }
   ```
2. If `tenant_context.effective_policy.requires_approval(&capability_id)` evaluates to `true`, suspend the call into the HITL approval engine.
3. Automatically attach `tenant_id` and verified `role` to the WORM audit payload.

---

## 5. Security & Verification Plan

### 5.1 Threat Modeling & Guardrails
* **T1: Privilege Escalation via Forged Headers:**
  - `actor_id` and `grant_id` headers (`X-Warmplane-Actor`, `X-Warmplane-Grant`) are strictly overridden or bound by the verified token claims in `TenantContext`. Callers cannot spoof identities beyond what their token or JWT grants.
* **T2: Information Disclosure via Catalog Guessing:**
  - `GET /v1/capabilities/:id` returns `404 NOT_FOUND` (or `403 FORBIDDEN`) if the role is denied access, preventing enumeration of restricted enterprise tools.
* **T3: ETag Cache Poisoning Across Tenants:**
  - `ETag` headers incorporate the role identifier (e.g. `cat_v1_analyst`), preventing an unprivileged caller from reusing a 304 response intended for an admin.

### 5.2 Automated Test Strategy
* **Unit Tests (`src/rbac/tests.rs`):**
  - Token lookup and parsing.
  - Wildcard pattern matching per role.
  - Merging base policy with role-specific overrides.
* **Integration Tests (`tests/rbac_integration.rs`):**
  - Verify that `analyst` token sees only query capabilities on `GET /v1/capabilities`.
  - Verify that `devops` token triggers HITL suspension on mutating capabilities.
  - Verify that calling unauthorized capabilities directly returns `403 FORBIDDEN`.
  - Verify WORM audit logs contain `tenant_id` and `role`.

---

## 6. Phased Implementation Roadmap

### Phase 1: RBAC Data Models & Config Parser
- [ ] Create `src/rbac/mod.rs`, `src/rbac/models.rs`, and `src/rbac/engine.rs`.
- [ ] Extend `src/config.rs` to parse `RbacConfig` from `mcp_servers.json`.
- [ ] Add CLI validation commands (`warmplane validate-config`) checking for role circularity or missing references.

### Phase 2: Security Middleware & Token Verification
- [ ] Create `src/rbac/middleware.rs` to extract tokens, resolve `TenantContext`, and inject into Axum `Request.extensions`.
- [ ] Support static API key lookups in memory with constant-time comparison (`subtle::ConstantTimeEq`).
- [ ] Add JWT verification engine for OIDC bearer tokens.

### Phase 3: Catalog Partitioning
- [ ] Update `src/http_v1/catalog.rs` (`handle_list_capabilities`, `handle_describe_capability`, `handle_list_resources`, `handle_list_prompts`, `handle_catalog_events`).
- [ ] Update `src/search/hybrid.rs` to enforce role-scoped search filtering.
- [ ] Update `src/mcp_server.rs` stdio facade to respect caller's session role.

### Phase 4: Execution Gating & HITL Role Interception
- [ ] Update `src/http_v1/execution.rs` and `src/batch_executor.rs` to evaluate role permissions before dispatch.
- [ ] Pass role-specific `redact_keys` into sanitization pipeline.
- [ ] Record `tenant_id` and `role` into `AuditEvent` and WORM chain.

### Phase 5: Control Deck UI (Role Switcher & RBAC Matrix)
- [ ] Add RBAC status indicators and role simulator in Control Deck.
- [ ] Provide capability matrix visualization showing allowed/denied tools per role.
- [ ] Update documentation (`docs/USER-GUIDE.md`, `docs/ENTERPRISE_FEATURES.md`, `docs/openapi.yaml`).
