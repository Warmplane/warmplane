# Server Constellations ("Profiles") — Implementation Plan

This document details the design, configuration schema, request resolution semantics, catalog partitioning logic, and phased implementation plan for adding **Profiles** (named server constellations) to Warmplane.

---

## 1. Motivation & Core Concept

Currently, a Warmplane daemon connects to all upstream MCP servers defined in `mcpServers` and exposes the complete aggregated catalog across all endpoints.

While **RBAC** handles *who* is authorized to call *what* (identity and role boundaries), **Profiles** address *what constellation of tools is relevant to a specific task or agent context* (task-based view selection).

### Key Properties
1. **Task-Relevant Partitioning**: A single running daemon can serve a coding agent (`["github", "filesystem", "sqlite"]`), a support agent (`["zendesk", "slack"]`), and a data agent (`["postgres"]`) without running separate daemon processes or writing complex RBAC deny rules.
2. **Keep-Warm Persistence**: All configured upstream servers remain warm and connected in the background. Profiles do not alter server lifecycle; they act as a deterministic pre-filter on discovery, search, and invocation surfaces.
3. **Orthogonal to RBAC**: Profiles compose cleanly with RBAC and policy evaluation:
   $$\text{Available Capabilities} = \text{Full Catalog} \cap \text{Profile Servers} \cap \text{RBAC Role Scopes} \cap \text{Global Policy}$$
4. **Per-Request Agility**: HTTP clients can select or switch profiles dynamically per-request using headers or query parameters without session renegotiation.

---

## 2. Architecture & Request Resolution

### Request Resolution Pipeline

```
                                  Client Request
                                         │
                                         ▼
                 ┌──────────────────────────────────────────────┐
                 │          Profile Resolution Layer            │
                 │                                              │
                 │  Extract Profile ID:                         │
                 │  1. Header: `X-Warmplane-Profile`            │
                 │  2. Query Param: `?profile=`                 │
                 │  3. CLI / Stdio Flag: `--profile`            │
                 │  (Default: None / Unrestricted View)         │
                 └───────────────────────┬──────────────────────┘
                                         │ Injects `ProfileContext`
                                         ▼
                 ┌──────────────────────────────────────────────┐
                 │       RBAC & Authentication Middleware       │
                 │                                              │
                 │  Resolve `TenantContext` & Role Scopes       │
                 └───────────────────────┬──────────────────────┘
                                         │
                     ┌───────────────────┴───────────────────┐
                     ▼                                       ▼
        ┌─────────────────────────┐             ┌─────────────────────────┐
        │  Catalog & Search APIs  │             │   Execution Handlers    │
        │                         │             │                         │
        │ • /v1/capabilities      │             │ • /v1/tools/call        │
        │ • /v1/resources         │             │ • /v1/tools/batch_call  │
        │ • /v1/prompts           │             │ • /v1/resources/read    │
        │ • /v1/capabilities/search│            │ • /v1/prompts/get       │
        │                         │             │                         │
        │ Prune items from        │             │ Validate target server  │
        │ servers not in active   │             │ is in active profile    │
        │ profile whitelist       │             │ (returns 404 / 403)     │
        └─────────────────────────┘             └─────────────────────────┘
```

---

## 3. Configuration Schema

Extend `mcp_servers.json` (`McpConfig`) with a top-level `profiles` mapping.

```json
{
  "port": 9090,
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"]
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    },
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite", "./data.db"]
    },
    "zendesk": {
      "url": "https://mcp.zendesk.internal/sse"
    },
    "slack": {
      "url": "https://mcp.slack.internal/sse"
    }
  },
  "profiles": {
    "coding": {
      "servers": ["github", "filesystem", "sqlite"],
      "description": "Software engineering and code review tools"
    },
    "support": {
      "servers": ["zendesk", "slack"],
      "description": "Customer support and messaging tools"
    },
    "triage": {
      "servers": ["github", "slack"],
      "description": "Issue triage and alert notifications"
    }
  }
}
```

### Config Validation Rules
- **V1**: When loading configuration, every server name listed in `profiles.<name>.servers` must exist in `mcpServers`. Validation fails fast at startup if an unknown server ID is referenced.
- **V2**: Profile names must be valid non-empty identifiers (alphanumeric, hyphens, underscores).

---

## 4. Detailed Design & Interface Semantics

### 4.1 Profile Selection by Interface

#### 1. HTTP REST API
- **Header**: `X-Warmplane-Profile: <profile_id>` (Preferred for programmatic clients / SDKs)
- **Query Parameter**: `?profile=<profile_id>` (Convenient for quick curls / browser testing)
- **Precedence**: Header takes precedence over query parameter if both are present.
- **Unknown Profile Behavior**: If a client specifies a profile name not present in configuration, return `404 Not Found` with structured error:
  ```json
  {
    "ok": false,
    "error": {
      "code": "PROFILE_NOT_FOUND",
      "message": "Profile 'devops' is not defined in configuration",
      "retryable": false
    }
  }
  ```
- **Unspecified Profile**: If no profile is requested, no profile filtering is applied (full catalog view, subject only to RBAC/policy). No `defaultProfile` is required or recommended.

#### 2. Stdio MCP Server Mode (`warmplane mcp-server`)
- Add CLI flag: `warmplane mcp-server --config mcp_servers.json --profile coding`
- The entire stdio session operates strictly within the specified profile's constellation.

#### 3. CLI Commands
- Support `--profile <name>` on catalog query and invocation commands:
  - `warmplane list-capabilities --profile coding`
  - `warmplane search-capabilities "find errors" --profile support`
  - `warmplane call-capability sqlite.read_query --profile coding ...`

---

### 4.2 Profile-Scoped SHA-256 Catalog Versioning & ETags

When caching catalogs via `ETag` and `If-None-Match`, different profiles see different subsets of tools, so they must produce distinct ETags.

#### ETag Calculation
Given base catalog SHA-256 digest $H_{\text{raw}}$:
- Unscoped (All servers): `ETag: "sha256:<hash>"`
- Profile Scoped: `ETag: "sha256:<hash>-p:<profile_id>"` or a deterministic SHA-256 hash computed over the profile-filtered sorted capability keys.

This guarantees:
1. Two requests with different `X-Warmplane-Profile` headers never falsely share a `304 Not Modified` payload.
2. Fast-path quote-trim validation overhead remains sub-microsecond.

---

### 4.3 Catalog & Search API Behavior

1. **`GET /v1/capabilities`**: Filters the returned `capabilities` list so that `cap.server` is in `profile.servers`.
2. **`GET /v1/capabilities/:id`**: If capability exists on the daemon but its originating server is not part of the active profile, returns `404 TOOL_NOT_FOUND` (or `403 CAPABILITY_UNAUTHORIZED`).
3. **`POST /v1/capabilities/search`**:
   - The hybrid BM25 / vector search engine filters candidate keys against `profile.servers` before scoring and ranking.
4. **`GET /v1/resources` & `POST /v1/resources/read`**:
   - Filters resources by `resource.server \in profile.servers`.
5. **`GET /v1/prompts` & `POST /v1/prompts/get`**:
   - Filters prompts by `prompt.server \in profile.servers`.

---

### 4.4 Tool Execution Guard

When `POST /v1/tools/call` or `POST /v1/tools/batch_call` is executed with an active profile:
1. Warmplane resolves `capability_id` to its backing server $S$.
2. If $S \notin \text{profile.servers}$, execution is blocked immediately before invoking policy, HITL, or upstream dispatch:
   ```json
   {
     "ok": false,
     "error": {
       "code": "TOOL_NOT_IN_PROFILE",
       "message": "Capability 'zendesk.create_ticket' belongs to server 'zendesk' which is not in active profile 'coding'",
       "retryable": false
     }
   }
   ```
3. For batch calls (`/v1/tools/batch_call`), all individual steps are checked against the active profile.

---

## 5. Rust Implementation Details

### 5.1 Data Models (`src/config.rs` & `src/models.rs`)

```rust
/// Profile configuration defining a named server constellation.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
pub struct ProfileConfig {
    /// Whitelist of upstream server identifiers included in this constellation.
    pub servers: Vec<String>,
    /// Optional human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// In McpConfig:
pub struct McpConfig {
    // ... existing fields ...
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub profiles: HashMap<String, ProfileConfig>,
}
```

### 5.2 Context Model (`src/context.rs` or `src/http_v1/types.rs`)

```rust
/// Active profile context resolved from incoming request headers or flags.
#[derive(Clone, Debug, Default)]
pub struct ProfileContext {
    /// Profile identifier, or None if unfiltered.
    pub profile_id: Option<String>,
    /// Set of allowed server IDs for O(1) lookup.
    pub allowed_servers: Option<std::collections::HashSet<String>>,
}

impl ProfileContext {
    /// Checks if a server identifier is allowed under this profile.
    pub fn is_server_allowed(&self, server: &str) -> bool {
        match &self.allowed_servers {
            Some(set) => set.contains(server),
            None => true,
        }
    }
}
```

### 5.3 Axum Profile Extractor (`src/http_v1/helpers.rs`)

Implement an Axum extractor or middleware that reads `X-Warmplane-Profile` or `?profile=`, validates against `state.profiles`, and places `ProfileContext` in request extensions.

---

## 6. Implementation Phases

| Phase | Description | Deliverables |
|:---|:---|:---|
| **Phase 1: Config & Validation** | Add `ProfileConfig` to `McpConfig` with strict startup validation. | Config parsing tests, error handling for unknown server IDs. |
| **Phase 2: HTTP Layer & Extractor** | Implement `ProfileContext` extractor for headers/query params with `PROFILE_NOT_FOUND` error handling. | Unit tests for profile header & query parsing. |
| **Phase 3: Catalog & Search Partitioning** | Update `/v1/capabilities`, `/v1/resources`, `/v1/prompts`, search, and ETag calculation to respect `ProfileContext`. | ETag partition tests, catalog filter tests. |
| **Phase 4: Tool Execution Enforcement** | Enforce profile membership check in `/v1/tools/call` and `/v1/tools/batch_call`. | `TOOL_NOT_IN_PROFILE` integration tests. |
| **Phase 5: MCP Stdio & CLI Integration** | Add `--profile` flag to `warmplane mcp-server`, `list-capabilities`, `search-capabilities`, etc. | Stdio smoke tests with profile selection. |
| **Phase 6: Docs & Integrator Guides** | Update User Guide, OpenAPI spec, and Integrators Guides (Rust & TS) with profile usage examples. | Documentation updates. |

---

## 7. Verification & Test Plan

1. **Config Validation**: Ensure defining a profile with a non-existent server name produces a clear startup error.
2. **Catalog Pruning**: Configure servers `A`, `B`, `C` and profile `p1 = ["A", "B"]`. Verify `GET /v1/capabilities` with `X-Warmplane-Profile: p1` returns only capabilities from `A` and `B`.
3. **ETag Divergence**: Verify `GET /v1/capabilities` with `X-Warmplane-Profile: p1` vs no profile header yield different `ETag` checksums and independent `304` lifecycles.
4. **Execution Enforcement**: Verify calling a tool from server `C` while `X-Warmplane-Profile: p1` is active returns `404` or `403` with `TOOL_NOT_IN_PROFILE`.
5. **Stdio Isolation**: Run `warmplane mcp-server --profile p1` and verify `capabilities_list` via JSON-RPC only yields items from `A` and `B`.
