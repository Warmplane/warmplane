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
- [ ] **Update Defaults (`src/daemon.rs`, `src/config.rs`):**
  - Change `DEFAULT_MCP_PROTOCOL_VERSION` to `"2026-07-28"`.
  - Maintain backward compatibility check to accept `"2025-11-25"`.
- [ ] **Error Handling:**
  - Define standard protocol error constants:
    - `HeaderMismatch`: `-32020`
    - `MissingRequiredClientCapability`: `-32021`
    - `UnsupportedProtocolVersion`: `-32022`
    - Resource Not Found aligned to `-32602` (`Invalid Params`).
  - Return `-32022` if an incoming request provides an unsupported `protocolVersion`.

### Phase 2: Upstream Client Connections & Routing Headers
- [ ] **Streamable HTTP Client Headers (`src/daemon.rs`):**
  - On every outgoing HTTP POST request to an upstream MCP server, inject mandatory routing headers:
    - `Mcp-Method`: (e.g. `tools/call`, `tools/list`, `resources/read`, `prompts/get`)
    - `Mcp-Name`: (e.g. tool name, resource URI, prompt name when applicable)
    - `Mcp-Protocol-Version`: `"2026-07-28"`
  - Forward custom tool parameter headers if `x-mcp-header` is defined.
- [ ] **Stateless `_meta` Envelope Propagation:**
  - Populate `_meta["io.modelcontextprotocol/protocolVersion"] = "2026-07-28"`.
  - Populate `_meta["io.modelcontextprotocol/clientInfo"] = { "name": "warmplane", "version": "0.7.1" }`.
  - Populate `_meta["io.modelcontextprotocol/clientCapabilities"]`.
  - Propagate OpenTelemetry trace context keys (`traceparent`, `tracestate`, `baggage`) directly in `_meta`.

### Phase 3: Server Facade (`rmcp` Stdio & HTTP)
- [ ] **Implement `server/discover` RPC (`src/mcp_server.rs`):**
  - Expose server identity (`warmplane`), supported versions (`["2026-07-28", "2025-11-25"]`), and consolidated capabilities.
- [ ] **Deterministic Sorting for Tools & Catalogs (`src/mcp_server.rs`, `src/catalog/`):**
  - Sort `tools/list` deterministically (alphabetical by tool ID / name) to maximize upstream prompt cache hit rates for LLM clients.
- [ ] **Cacheable Result Envelopes (`ttlMs` & `cacheScope`):**
  - Add `ttlMs` and `cacheScope` (`"public"` / `"private"`) fields to catalog listings (`tools/list`, `resources/list`, `prompts/list`).
- [ ] **Envelope `resultType` Standardization:**
  - Guarantee `resultType: "complete"` is populated on all non-streaming results.

### Phase 4: Multi Round-Trip Requests (MRTR) Support
- [ ] **Interim Response Handling (`src/daemon.rs`, `src/http_v1.rs`, `src/mcp_server.rs`):**
  - Handle upstream responses returning `resultType: "input_required"` with `inputRequests` (e.g. user approval or missing parameter elicitation).
  - Forward `inputRequests` and `requestState` back to client / caller.
  - Support receiving client `inputResponses` on subsequent request retries and mapping them to upstream servers.

### Phase 5: Change Feeds & Subscriptions
- [ ] **Implement `subscriptions/listen` Handler:**
  - Add single long-lived POST SSE stream for change notifications (`toolsListChanged`, `promptsListChanged`, `resourcesListChanged`, `resourceSubscriptions`).
  - Attach `io.modelcontextprotocol/subscriptionId` to dispatched events.
  - Keep `/v1/catalog/events` HTTP API for REST clients while mapping internal events to `subscriptions/listen`.
- [ ] **Remove Deprecated Handlers:**
  - Clean up legacy `ping`, `logging/setLevel`, and `notifications/roots/list_changed` references.

### Phase 6: OAuth 2.0 & Identity Hardening
- [ ] **Client Credential Issuer Binding (`src/oauth2.rs`):**
  - Ensure stored OAuth tokens and credentials are keyed strictly by issuer URL (`DiscoveryMetadata.issuer`).
  - Validate RFC 9207 `iss` matching on OAuth callback (already partially implemented; verify against latest spec assertions).
  - Support Client ID Metadata Documents (CIMD) registration flow.

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
