# Warmplane Control Deck: Web UI & Mission Console Plan

> **Goal:** Design and specify an embedded, zero-dependency, ultra-fast Web UI ("Warmplane Control Deck") running directly within the Warmplane Rust daemon (`http://127.0.0.1:9090/ui` and `/`).
> **Vision:** Deliver the "Swagger + Postman + Grafana" for Model Context Protocol (MCP) sessions—combining visual configuration management, live tool execution & MRTR playgrounds, real-time token savings analytics, and policy sandboxing into a gorgeous Bento-grid interface.
> **Status:** Proposed Architectural Plan & Specification  
> **Target Version:** `v0.11.0`

---

## 1. Executive Summary & Design Philosophy

Warmplane already maintains persistent in-memory connections to upstream MCP servers, live telemetry, hybrid search vectors, and normalized execution envelopes. Exposing this through a browser-accessible web console turns Warmplane from a headless proxy into a **visual control plane for agent developers and platform engineers**.

### Key Architectural Principles
1. **Zero External Runtime Dependencies:** The entire UI is built as an ultra-lean SPA (Vanilla TS/CSS or Preact + Vite) and embedded into the Warmplane Rust binary via `rust-embed` or `include_str!`. Zero Node.js or external web servers required at runtime.
2. **Information-Dense Cockpit (2026 UI Standard):** Bento-grid layouts, dark-mode amber/ember aesthetic matching Warmplane's design system, tactile depth with liquid-glass accents, and sub-millisecond local responsiveness.
3. **Dual-Channel Live Sync:**
   - **REST API (`/v1/config/...`)**: Transactional reads, writes, and imports using the config engine from `v0.10.0`.
   - **Real-Time SSE Streams (`/v1/catalog/events`, `/v1/resources/updates`)**: Live server status pulses, upstream latency indicators, and live tool execution streams without polling.

---

## 2. Information Architecture & Core Modules

The Web UI is structured into five cohesive workspaces accessible via a sleek sidebar navigation:

```
Warmplane Control Deck
├── 1. 🎛️ Overview & Bento Deck (Cockpit)
│   ├── Token Savings & ETag Hit Rate Gauges
│   ├── Active Upstream Session Grid (Health, Pings, Protocol Versions)
│   └── Live Event & Request Stream (SSE Feed)
│
├── 2. 🔌 Servers & Upstream Hub (Configuration Management)
│   ├── Visual Server Cards (Stdio processes & Remote HTTP/SSE)
│   ├── 1-Click "Add Server" Modal (Presets: GitHub, Postgres, Filesystem, Linear, Context7)
│   ├── Ecosystem Importer (Claude Desktop, Cursor, Zed discovery & merge preview)
│   └── Live Connection Probe & Discover Diagnostic
│
├── 3. 🧪 Capability Explorer & Interactive Playground (The "MCP Postman")
│   ├── Search & Filter Catalog (Lexical + Vector search visualizer)
│   ├── Dynamic JSON Schema Form Generator (Auto-generates input fields)
│   ├── Live "Execute Tool" with Normalized Envelope Visualizer
│   └── MRTR Interactive Flow Simulator (Elicitation & Approval dialogs)
│
├── 4. 🛡️ Policy & Governance Sandbox
│   ├── Visual Allow/Deny Matrix by Server Namespace
│   ├── Sensitive Key Redaction Tag Editor
│   └── "Policy Simulator": Test hypothetical agent queries against rules
│
└── 5. 🏷️ Facades & Alias Studio
    ├── Fast alias mappings (`git-commit` -> `github.create_commit`)
    └── Token savings calculator per alias
```

---

## 3. Detailed Workspace Specifications

### 3.1 Overview & Bento Dashboard

The landing cockpit provides immediate situational awareness for running MCP sessions.

```
+-----------------------------------------------------------------------------------+
|  WARMPLANE CONTROL DECK   [● DAEMON RUNNING :9090]   [v0.10.0]       [⚙ Settings] |
+-----------------------------------------------------------------------------------+
| [ METRIC BENTO 1 ]         | [ METRIC BENTO 2 ]         | [ METRIC BENTO 3 ]      |
| 🔥 95.8% Token Reduction   | ⚡ 4.2ms Avg Dispatch Lat. | 💾 0 Tokens (ETag Hit)  |
| 1.2M Tokens Saved Today    | 4 Upstreams Warm & Hot     | 84.1% 304 Cache Rate    |
+----------------------------+----------------------------+-------------------------+
| [ ACTIVE SERVERS GRID ]                                                           |
|  ● github (stdio: npx)      ● sqlite (stdio: npx)      ● context7 (http: sse)     |
|    24 Tools · 18ms latency    8 Tools · 1.2ms latency    12 Tools · 42ms latency  |
+-----------------------------------------------------------------------------------+
| [ REAL-TIME REQUEST STREAM (SSE) ]                                                |
|  16:34:02  POST /v1/tools/call  → db.query            200 OK (0.8ms)  [Idempotent]|
|  16:34:10  GET  /v1/capabilities                       304 Not Mod.    [ETag Hit]  |
|  16:34:18  POST /v1/tools/call  → github.get_issue    200 OK (22ms)   [MRTR Flow] |
+-----------------------------------------------------------------------------------+
```

---

### 3.2 Server Hub & Ecosystem Importers (`/ui/servers`)

A visual management suite replacing manual JSON manipulation with instant feedback.

#### Features & User Experience:
* **Server Health Cards:** Display transport type (`stdio` / `http`), command arguments or URL, protocol version (`2026-07-28`), authentication badge (`Bearer (env)`, `OAuth2 PKCE`), and process PID / HTTP status.
* **Add Server Wizard:**
  - **Quick Presets:** Instant configuration templates for popular servers (`@modelcontextprotocol/server-filesystem`, `server-github`, `server-postgres`, `server-sqlite`, `server-brave-search`, `context7`).
  - **Custom Form:** Switch between Stdio and Remote HTTP/SSE. Form validates environment variable presence before saving.
* **Ecosystem Sync Modal:**
  - Automatically scans `~/Library/Application Support/Claude/claude_desktop_config.json`, `~/.cursor/mcp.json`, and `~/.config/zed/settings.json`.
  - Visual side-by-side diff: highlights newly discovered servers vs existing servers.
  - "Import Selected" with one click.
* **"Test Probe" Button:**
  - Dispatches an asynchronous `server/discover` or execution check to verify reachability without restarting the daemon.

---

### 3.3 Interactive Capability Playground ("MCP Postman") (`/ui/playground`)

Allows developers to test and debug MCP tools directly in the browser.

```
+------------------------------------+-----------------------------------------------+
| SEARCH & CATALOG (34 items)        | CAPABILITY: db.read_query                     |
| [ 🔍 search or filter...         ] | Namespace: sqlite · Protocol: 2026-07-28      |
+------------------------------------+-----------------------------------------------+
| ● db.read_query (sqlite)           | Summary: Execute a read-only SQL query.       |
| ● db.write_query (sqlite)          |                                               |
| ● github.create_issue (github)     | ARGUMENTS (Auto-generated Schema Form):       |
| ● github.get_file (github)         | ┌───────────────────────────────────────────┐ |
| ● fs.read_file (filesystem)        | │ query: [ SELECT * FROM users LIMIT 5    ] │ |
|                                    | └───────────────────────────────────────────┘ |
| Filters:                           |                                               |
| [x] Tools  [ ] Resources  [ ] Prom | Context (Optional):                           |
| Server: [ All Servers    ▼ ]       | Request ID: [ req-dev-01 ]  Actor: [ origo ]  |
|                                    | [ ⚡ EXECUTE CAPABILITY ]                      |
|                                    |                                               |
|                                    | EXECUTION RESPONSE (200 OK · 1.4ms):          |
|                                    | {                                             |
|                                    |   "ok": true,                                 |
|                                    |   "request_id": "req-dev-01",                 |
|                                    |   "data": { "rows": [...] },                  |
|                                    |   "retry": { "classification": "safe" }       |
|                                    | }                                             |
+------------------------------------+-----------------------------------------------+
```

#### Multi Round-Trip Request (MRTR) Simulation:
- If a tool returns `input_required` (e.g. human confirmation or OAuth step-up), the UI displays an interactive dialog: *"This tool requires confirmation: Deploy to production? [Confirm / Reject]"*.
- Upon user input, the playground automatically sends the replay request with `input_responses` and `request_state`.

---

### 3.4 Security & Policy Sandbox (`/ui/policy`)

* **Visual Toggle Grid:** Allows administrators to toggle capability access (`allow` / `deny`) with visual switches per tool or wildcard prefix (`github.*`).
* **Redaction Manager:** Tag chip input for adding sensitive key patterns (e.g., `client_secret`, `private_key`, `access_token`).
* **Interactive Policy Tester:** Input a test capability name and mock payload to verify whether Warmplane permits, denies, or redacts the call before deploying to production.

---

## 4. Technical Architecture & Rust Backend Integration

### 4.1 Backend Endpoints (Axum + `v0.10.0` Config Engine)

The following HTTP endpoints will be added to `src/daemon.rs` & `src/http_v1.rs`:

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/ui` or `/` | Serves the embedded Single-Page Application (HTML/CSS/JS). |
| `GET` | `/ui/assets/*` | Serves embedded CSS, JS, SVG icons, and fonts. |
| `GET` | `/v1/config` | Returns active `McpConfig` (servers, aliases, policies). |
| `POST` | `/v1/config/servers` | Adds or updates an upstream server definition. |
| `DELETE`| `/v1/config/servers/:id`| Removes a configured server. |
| `POST` | `/v1/config/servers/:id/test`| Dispatches connection probe to upstream server. |
| `GET` | `/v1/config/ecosystem` | Discovers available configs from Claude Desktop/Cursor/Zed. |
| `POST` | `/v1/config/import` | Imports selected servers into `mcp_servers.json`. |
| `POST` | `/v1/config/aliases` | Adds or removes capability, resource, and prompt aliases. |
| `POST` | `/v1/config/policy` | Updates allow, deny, and redaction policies. |
| `GET` | `/v1/telemetry/stats` | Aggregated metrics: token savings, requests, cache hit rate. |

### 4.2 Asset Embedding via `rust-embed`

```rust
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct Asset;

pub async fn ui_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let path = uri.path().trim_start_matches("/ui/").trim_start_matches('/');
    let file_path = if path.is_empty() { "index.html" } else { path };

    match Asset::get(file_path) {
        Some(content) => {
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => match Asset::get("index.html") {
            Some(content) => {
                ([(axum::http::header::CONTENT_TYPE, "text/html")], content.data).into_response()
            }
            None => (axum::http::StatusCode::NOT_FOUND, "UI asset not found").into_response(),
        },
    }
}
```

### 4.3 Frontend Stack: High-Performance Single-Page App

* **Framework:** Modern TypeScript + Vanilla Web Components / Lightweight Preact (~3kB).
* **Styling:** Curated OKLCH dark-mode theme, CSS Grid Bento layouts, custom glassmorphic cards, CSS animations for SSE live pulses.
* **Build System:** Vite (`bun run build` or `npm run build`) compiling down to static `/ui/dist/` bundle (< 150KB total gzipped).

---

## 5. Phased Implementation Roadmap

### Phase 1: Configuration & Telemetry REST APIs (`src/http_v1.rs`)
- [ ] Implement `GET /v1/config` and `POST /v1/config/servers` using the transactional `save_config` engine.
- [ ] Implement `GET /v1/config/ecosystem` and `POST /v1/config/import` using `src/config_import.rs`.
- [ ] Implement `POST /v1/config/servers/:id/test` live probing.
- [ ] Implement in-memory telemetry aggregator for token savings calculations and ETag hit ratios.

### Phase 2: Embedded UI Asset Pipeline (`rust-embed`)
- [ ] Set up `ui/` frontend workspace (Vite + TypeScript + vanilla CSS tokens).
- [ ] Add `rust-embed` and `mime_guess` dependencies in `Cargo.toml`.
- [ ] Wire Axum router in `src/daemon.rs` to serve `/ui` and redirect root `/` when accessed by browser `Accept: text/html`.

### Phase 3: Cockpit Bento Deck & Server Hub
- [ ] Build Bento-grid metrics dashboard with live SSE streaming counters.
- [ ] Build Server Management interface: Cards, health statuses, Add Server wizard with presets, and Ecosystem Import modal.

### Phase 4: Capability Explorer & MRTR Playground
- [ ] Build dynamic JSON Schema form renderer for capability tools.
- [ ] Build execution runner with JSON viewer for normalized execution envelopes.
- [ ] Build interactive MRTR dialog for multi round-trip approvals and input elicitation.

### Phase 5: Policy Sandbox & Alias Studio
- [ ] Build visual Allow/Deny matrix and sensitive key redaction manager.
- [ ] Build Policy Simulator for testing hypothetical agent queries.
- [ ] Final polishing, responsiveness checks, and automated integration test suite.

---

## 6. Verification & Quality Gates
- **Zero-Latency Serving:** Sub-millisecond initial asset load from memory.
- **Resilience:** Graceful handling when upstreams are offline or unreachable.
- **Compliance:** Strict adherence to Microsoft Pragmatic Rust guidelines, `M-CANONICAL-DOCS`, and `cargo clippy -- -D warnings`.
