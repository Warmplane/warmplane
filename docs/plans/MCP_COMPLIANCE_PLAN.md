# MCP 2026-07-28 Compliance & `rmcp` 3.x Migration Plan

> **Target Spec Version:** `2026-07-28`  
> **Target SDK Version:** `rmcp = "3.1.2"`  
> **Status:** Draft / Planned  
> **Scope:** Full protocol compliance across proxy runtime, HTTP v1 API, stdio facade, and client-to-upstream connections.

---

## 1. Executive Summary & Architectural Impact

The MCP `2026-07-28` specification shifts MCP from a stateful, connection-oriented protocol to a **stateless request/response core** with:
- **No Handshakes or Session IDs:** Complete elimination of `initialize` / `notifications/initialized` handshakes and `Mcp-Session-Id` headers.
- **Self-Describing Requests:** Every request carries its protocol version, client capabilities, and client identity within `_meta`.
- **Discovery RPC:** Up-front version selection and capability negotiation via `server/discover`.
- **Multi Round-Trip Requests (MRTR):** Elimination of server-initiated streams (`sampling/createMessage`, `elicitation/create`, `roots/list`) in favor of `resultType: "input_required"` interim replies and client replays.
- **Cacheable Results:** Mandatory cache hints (`ttlMs`, `cacheScope`) and deterministic ordering for tool, resource, and prompt catalogs.
- **Unified Change Feed:** Replaced fragmented subscription endpoints with `subscriptions/listen` POST streams.
- **Tasks Extension:** Tasks relocated out of core into `io.modelcontextprotocol/tasks` with poll/update mechanisms.

Warmplane acts as a **Local Control Plane and Proxy Facade**. Upgrading to `rmcp = "3.1.2"` and `2026-07-28` aligns Warmplane with the modern stateless ecosystem, unlocks native support for prompt caching, and simplifies downstream agent integrations.

---

## 2. Dependency Upgrade & Feature Flags

### Cargo.toml Changes
- Upgrade `rmcp` dependency:
  ```toml
  rmcp = { version = "3.1.2", features = ["client", "server", "transport-child-process", "transport-io", "transport-streamable-http-client-reqwest"] }
  ```
- Verify feature alignment (`transport-streamable-http-server` if hosting HTTP MCP endpoints natively).

---

## 3. Detailed Change-by-Change Implementation Plan

### Phase 1: Protocol Constants, Configuration & Version Negotiation
- [x] **Update Defaults (`src/daemon.rs`, `src/config.rs`):**
  - Change `DEFAULT_MCP_PROTOCOL_VERSION` to `"2026-07-28"`.
  - Maintain backward compatibility check to accept `"2025-11-25"`.
- [x] **Error Handling:**
  - Define standard protocol error constants:
    - `HeaderMismatch`: `-32020`
    - `MissingRequiredClientCapability`: `-32021`
    - `UnsupportedProtocolVersion`: `-32022`
    - Resource Not Found aligned to `-32602` (`Invalid Params`).
  - Return `-32022` if an incoming request provides an unsupported `protocolVersion`.

### Phase 2: Upstream Client Connections & Routing Headers
- [x] **Streamable HTTP Client Headers (`src/daemon.rs`):**
  - On every outgoing HTTP POST request to an upstream MCP server, inject standard headers:
    - `Mcp-Protocol-Version`: `"2026-07-28"`
- [x] **Stateless `_meta` Envelope & Builder Upgrades (`src/daemon.rs`):**
  - Use rmcp 3.x request builders (`CallToolRequestParams::new()`, `ReadResourceRequestParams::new()`, `GetPromptRequestParams::new()`).

### Phase 3: Server Facade (`rmcp` Stdio & HTTP)
- [x] **Implement `server/discover` RPC (`src/mcp_server.rs`):**
  - Expose server identity (`warmplane`), supported versions (`["2026-07-28", "2025-11-25"]`), and consolidated capabilities via `rmcp 3.x` default discover handler.
- [x] **Deterministic Sorting for Tools & Catalogs (`src/mcp_server.rs`, `src/http_v1.rs`):**
  - Sort tools, resources, and prompts deterministically (alphabetical by ID) to maximize upstream prompt cache hit rates for LLM clients.
- [x] **Cacheable Result Envelopes (`ttlMs` & `cacheScope`):**
  - Added `ttl_ms` (300,000ms) and `cache_scope` (`"public"`) fields to catalog listings (`/v1/capabilities`, `/v1/resources`, `/v1/prompts`).
- [x] **Envelope `resultType` Standardization:**
  - Use `CallToolResponse::Complete(...)`, `ReadResourceResponse::Complete(...)`, `GetPromptResponse::Complete(...)` envelopes.

### Phase 4: Multi Round-Trip Requests (MRTR) Support
- [x] **Interim Response Handling (`src/daemon.rs`, `src/http_v1.rs`, `src/mcp_server.rs`):**
  - Updated request payloads with optional `input_responses` (`BTreeMap<String, Value>`) and `request_state` (`String`).
  - Added MRTR argument propagation through daemon actor channels and `rmcp 3.x` request builders (`with_input_responses`, `with_request_state`).
  - Stdio facade and HTTP REST endpoints transparently accept and return MRTR envelopes.

### Phase 5: Event Subscriptions & Change Feeds
- [x] **`subscriptions/listen` Change Feed Tool (`src/mcp_server.rs`):**
  - Implemented `subscriptions_listen` tool in facade server mirroring the `/v1/catalog/events` feed.
  - Returns `catalog_version`, `cursor`, and list of `events`.
- [x] **Real-Time Resource Updates (`src/http_v1.rs`):**
  - SSE stream `/v1/resources/updates` notifies clients of resource content mutations.

### Phase 6: OAuth 2.0 & Identity Hardening
- [x] **RFC 9207 & SEP-2468 Issuer Validation (`src/oauth2.rs`):**
  - Verify `iss` query parameter against discovery issuer without normalization.
  - Strict AS endpoint discovery (RFC 8414 / RFC 9728) with host verification.
- [x] **Full Compliance & Test Suite Verification:**
  - Strict compiler and clippy checks (`cargo clippy -- -D warnings`).
  - Complete test suite passing (`cargo test`).

---

## 4. Verification & Validation Plan

1. **Compilation & Guideline Check:**
   - `cargo check --all-targets`
   - `cargo clippy -- -D warnings`
   - `cargo fmt -- --check`
2. **Automated Unit & Integration Tests:**
   - `cargo test` covering catalog versioning, ETag caching, MRTR roundtrips, and HTTP routing headers.
3. **Spec Compliance Test Suite:**
   - Test stdio facade against MCP official validator / client tests.
   - Validate header emission on Streamable HTTP transport proxies.
