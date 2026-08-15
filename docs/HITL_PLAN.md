# Human-in-the-Loop (HITL) Approval Engine — Implementation Plan

This document details the architectural design, data models, protocol flow, webhook authentication, error handling, argument modification, and implementation plan for adding Human-in-the-Loop (HITL) capability approval workflows to Warmplane.

---

## 1. Objectives & Requirements

1. **Policy-Driven Interception:** Configure specific capabilities or wildcard patterns (e.g. `*.delete_*`, `docker.run_container`, `db.execute_write`) to require operator approval before execution.
2. **Non-Blocking Asynchronous Suspension:**
   - Synchronous HTTP callers (`POST /v1/tools/call`) can hold the connection up to a configurable timeout while awaiting approval, OR receive a `202 Accepted` with an `approval_ticket_id` to poll / await via SSE.
   - Streaming SSE feed emits live `approval.requested`, `approval.granted`, `approval.rejected`, and `approval.expired` events.
3. **Structured Agent Feedback:** When rejected, return structured agent-digestible feedback (`OPERATION_REJECTED_BY_OPERATOR` with operator reasoning) so LLM planning loops can self-correct.
4. **Parameter Tweaking / Argument Modification:** Allow operators to approve calls with sanitized/modified parameters (e.g. adding row limits or fixing flags) without rejecting the agent's workflow.
5. **Atomic Idempotency & Replay Protection:** Guarantee exact-once upstream tool execution even under race conditions or duplicate webhook callbacks.
6. **Control Deck Review Queue:** A dedicated **Approvals** tab in the Control Deck UI displaying pending approval requests, caller context (`actor_id`, `work_item_id`), capability arguments, diffs, and parameter editing with 1-click **Approve** and **Reject (with feedback)** actions.
7. **Secure Webhook & Notification Integrations:** Outbound HTTP webhooks (e.g. to Slack, Discord, PagerDuty, or internal security portals) dispatched when an approval is requested, with HMAC-SHA256 signature verification and bearer authentication headers.
8. **Security & Auditability:** Immutable audit logging of who approved/rejected/modified the request, rationale, and timestamps.

---

## 2. Architecture & Data Flow

```
Agent / Client             Warmplane Daemon                 Control Deck / Webhook
      │                           │                                    │
      │  1. POST /v1/tools/call   │                                    │
      ├──────────────────────────►│                                    │
      │                           │ 2. Check Policy (Match HITL rule) │
      │                           ├─────────────────────────────────┐  │
      │                           │ Creates PendingApproval Ticket  │  │
      │                           │◄────────────────────────────────┘  │
      │                           │                                    │
      │                           │ 3. Broadcast SSE & Signed Webhook  │
      │                           ├───────────────────────────────────►│
      │                           │    (HMAC-SHA256 Signature Header)  │
      │                           │                                    │
      │ (A) Synchronous Wait OR   │                                    │
      │ (B) HTTP 202 Pending      │                                    │
      │                           │                                    │
      │                           │ 4. Operator: Approve/Modify/Reject │
      │                           │◄───────────────────────────────────┤
      │                           │                                    │
      │                           │ 5. Atomic State Transition (CAS)   │
      │                           │    Execute Upstream Tool (if OK)   │
      │                           ├─────────────────┐                  │
      │                           │ Call MCP Client │                  │
      │                           │◄────────────────┘                  │
      │                           │                                    │
      │  6. Return Tool Envelope  │                                    │
      │◄──────────────────────────┤                                    │
```

---

## 3. Data Models & Configuration

### 3.1 Policy Configuration (`mcp_servers.json`)

Extend `PolicyConfig` to define approval rules, timeout thresholds, and enterprise webhook authentication settings:

```json
{
  "mcpServers": { ... },
  "policy": {
    "allow": ["*"],
    "deny": ["*.destroy_cluster"],
    "redactKeys": ["password", "token", "secret"],
    "requireApproval": [
      "docker.run_container",
      "docker.remove_*",
      "filesystem.write_*",
      "filesystem.edit_*",
      "db.execute_mutation",
      "github.create_pull_request"
    ],
    "approvalTimeoutSecs": 300,
    "webhook": {
      "url": "https://secops.corp.internal/api/v1/mcp-approvals",
      "secret": "whsec_...",
      "secretEnv": "WARMPLANE_WEBHOOK_SECRET",
      "authHeader": "Bearer eyJhbGciOi...",
      "headers": {
        "X-Environment": "production"
      }
    }
  }
}
```

### 3.2 Webhook Authentication Details

When an approval is created, expired, or completed, Warmplane dispatches a structured webhook event:

* **Headers Added:**
  * `Content-Type: application/json`
  * `X-Warmplane-Timestamp: <unix_epoch_seconds>`
  * `X-Warmplane-Signature: sha256=<hex_encoded_hmac_sha256_of_payload_and_timestamp>` (if `secret` or `secretEnv` is configured)
  * `Authorization: <authHeader>` (if configured)
  * Custom headers from `headers` map
* **HMAC Signature Formula:**
  `HMAC-SHA256(secret, "${timestamp}.${raw_json_body}")`

### 3.3 Rust In-Memory State (`AppState`)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: Option<String>,
    #[serde(rename = "secretEnv")]
    pub secret_env: Option<String>,
    #[serde(rename = "authHeader")]
    pub auth_header: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

impl WebhookConfig {
    pub fn resolve_secret(&self) -> Option<String> {
        if let Some(ref env_var) = self.secret_env {
            if let Ok(val) = std::env::var(env_var) {
                return Some(val);
            }
        }
        self.secret.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved {
        operator: String,
        timestamp: u64,
        modified_args: Option<serde_json::Value>,
    },
    Rejected {
        operator: String,
        reason: Option<String>,
        timestamp: u64,
    },
    Expired {
        timestamp: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,                  // e.g. "appr-01J9X..."
    pub capability_id: String,
    pub server_id: String,
    pub args: serde_json::Value,
    pub sanitized_args: serde_json::Value,
    pub request_id: Option<String>,
    pub context: Option<RequestContext>,
    pub created_at: u64,
    pub expires_at: u64,
    #[serde(flatten)]
    pub status: ApprovalStatus,
}

pub enum ApprovalResolution {
    Approved { modified_args: Option<serde_json::Value> },
    Rejected { reason: Option<String>, operator: String },
    Expired,
}

pub struct ApprovalRegistry {
    pub pending: RwLock<HashMap<String, PendingApproval>>,
    pub wait_channels: RwLock<HashMap<String, tokio::sync::oneshot::Sender<ApprovalResolution>>>,
    pub webhook: Option<WebhookConfig>,
}
```

---

## 4. API Surface & Endpoints

### 4.1 Execution Interception (`POST /v1/tools/call`)
- When `policy.requires_approval(&capability_id)` matches:
  - Create `PendingApproval` ticket and trigger signed webhook.
  - If client specifies header `Prefer: respond-async`, return `HTTP 202 Accepted`:
    ```json
    {
      "status": "pending_approval",
      "approval_id": "appr-77a1bc",
      "capability_id": "docker.run_container",
      "message": "Execution suspended pending human operator approval",
      "expires_at": 1755193400
    }
    ```
  - Otherwise, hold the async request on a `oneshot::channel` until approved, rejected, or expired.
- **Rejection Return Structure:**
  ```json
  {
    "ok": false,
    "error": {
      "code": "OPERATION_REJECTED_BY_OPERATOR",
      "message": "Human operator rejected execution: Destructive query on production schema",
      "operator": "origo@warmplane.io"
    }
  }
  ```

### 4.2 Pending Approvals API
- `GET /v1/approvals` — List all active pending approvals and recent history.
- `GET /v1/approvals/:id` — Get detailed context and payload for a specific approval.
- `POST /v1/approvals/:id/approve` — Approve execution (with optional parameter edits):
  ```json
  {
    "operator": "origo@warmplane.io",
    "modified_args": { "limit": 100 }
  }
  ```
- `POST /v1/approvals/:id/reject` — Reject execution:
  ```json
  {
    "operator": "origo@warmplane.io",
    "reason": "Destructive query on production schema"
  }
  ```

---

## 5. Control Deck UI Component: Approvals Queue

A dedicated **Approvals** tab in the Control Deck UI (`ui/src/components/approvals.ts`):

1. **Active Pending Badge:** Real-time badge in sidebar navigation showing current pending count (`Approvals (3)`).
2. **Approval Card View:**
   - Visual indicator of risk level (amber for mutation, red for deletion).
   - Sanitized arguments inspector with editable JSON area for parameter tweaking.
   - Caller metadata (`actor_id`, `operation_id`, client IP, time elapsed).
3. **Action Triggers:**
   - **Approve Button (Green):** Dispatches approval and triggers instant tool run.
   - **Approve with Edits Button:** Sends modified arguments.
   - **Reject Button (Red):** Opens modal to provide optional reason returned directly to the agent's LLM context.

---

## 6. Phased Implementation Steps

### Phase 1: Core Policy & Approval Registry (Backend)
1. Update `PolicyConfig`, `Policy`, and add `WebhookConfig` in `src/config.rs` & `src/daemon.rs` with `require_approval`, `approval_timeout_secs`, and `webhook` authentication settings.
2. Implement `ApprovalRegistry` in `src/approvals.rs` with thread-safe pending queue, HMAC signing helper, atomic state transitions, and `tokio::sync::oneshot` channels.
3. Update `handle_call_capability` in `src/http_v1.rs` to intercept matching capabilities, suspend execution, and resume with modified arguments or structured rejection messages.

### Phase 2: Approval Management Endpoints & SSE Events
1. Add `/v1/approvals`, `/v1/approvals/:id/approve`, and `/v1/approvals/:id/reject` endpoints in `src/http_v1.rs`.
2. Emit SSE events to `/v1/resources/updates` and `/v1/catalog/events` when approval tickets transition state.
3. Add asynchronous webhook dispatch worker with HMAC-SHA256 and Bearer auth support in `src/approvals.rs`.

### Phase 3: Control Deck UI Integration (Frontend)
1. Add `approvals` tab to `ui/src/state.ts` and `ui/src/main.ts`.
2. Build `ui/src/components/approvals.ts` with real-time pending list, countdown timers, argument modification editor, and Approve/Reject actions.
3. Add approval rule and webhook configuration management section into the **Security Governance & Redaction** tab.

### Phase 4: CLI & Automated Testing
1. Add CLI commands: `warmplane approvals list`, `warmplane approve <ID>`, and `warmplane reject <ID>`.
2. Write integration tests in `tests/hitl_tests.rs` covering wildcard matching, argument modifications, rejection reason propagation, HMAC-SHA256 signature verification, synchronous blocking, asynchronous polling, and timeout expiration.
