# Rust Integrators Guide

This guide explains how to drive a running Warmplane daemon from Rust code over its HTTP REST API. It includes a minimal `WarmplaneClient` you can drop into your project and build on.

---

## Prerequisites

- A running Warmplane daemon with at least one upstream MCP server configured.
- Rust toolchain (1.80+).
- The following crates in your `Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.13", features = ["json", "rustls"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

Start the daemon in a separate terminal:

```bash
warmplane daemon --config mcp_servers.json
```

---

## WarmplaneClient

A thin typed wrapper over `reqwest`. No framework, no macros — just enough structure to avoid repeating URL construction and envelope parsing.

```rust
use anyhow::{bail, Context, Result};
use reqwest::{header, Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Minimal Warmplane HTTP API client.
///
/// Wraps a `reqwest::Client` with a base URL pointing at a running daemon.
/// All methods return parsed response envelopes or structured errors.
pub struct WarmplaneClient {
    client: Client,
    base_url: String,
    /// Cached catalog ETag for conditional revalidation.
    cached_etag: std::sync::Mutex<Option<String>>,
}

/// Standard Warmplane response envelope.
#[derive(Debug, Deserialize)]
pub struct Envelope {
    pub ok: bool,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub data: Option<Value>,
    pub error: Option<EnvelopeError>,
    pub retry: Option<RetryInfo>,
}

/// Error details within a response envelope.
#[derive(Debug, Deserialize)]
pub struct EnvelopeError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

/// Retry classification metadata.
#[derive(Debug, Deserialize)]
pub struct RetryInfo {
    pub classification: Option<String>,
    pub upstream_execution_state: Option<String>,
}

/// Compact capability entry from the catalog index.
#[derive(Debug, Deserialize)]
pub struct CapabilityEntry {
    pub id: String,
    pub summary: String,
    pub server: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub mode: Option<String>,
}

/// Catalog listing response body.
#[derive(Debug, Deserialize)]
pub struct CatalogResponse {
    pub version: String,
    pub catalog_version: String,
    pub capabilities: Vec<CapabilityEntry>,
    pub ttl_ms: Option<u64>,
    pub cache_scope: Option<String>,
}

/// Search result entry with relevance score.
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub summary: String,
    pub server: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub score: f64,
    #[serde(default)]
    pub match_types: Vec<String>,
}

/// Search response body.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub version: String,
    pub catalog_version: String,
    pub capabilities: Vec<SearchResult>,
}

impl WarmplaneClient {
    /// Creates a new client pointing at the given daemon base URL.
    ///
    /// # Example
    /// ```
    /// let client = WarmplaneClient::new("http://127.0.0.1:9090");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            cached_etag: std::sync::Mutex::new(None),
        }
    }

    // ------------------------------------------------------------------
    // Catalog discovery
    // ------------------------------------------------------------------

    /// Lists all capabilities from the compact catalog index.
    ///
    /// Performs conditional revalidation using `If-None-Match` when a
    /// cached ETag is available. Returns `Ok(None)` on 304 Not Modified.
    pub async fn list_capabilities(&self) -> Result<Option<CatalogResponse>> {
        let url = format!("{}/v1/capabilities", self.base_url);
        let mut req = self.client.get(&url);

        if let Some(etag) = self.cached_etag.lock().unwrap().as_ref() {
            req = req.header(header::IF_NONE_MATCH, etag.as_str());
        }

        let res = req.send().await.context("GET /v1/capabilities")?;

        if res.status() == StatusCode::NOT_MODIFIED {
            return Ok(None);
        }

        // Capture ETag for future revalidation
        if let Some(etag) = res.headers().get(header::ETAG) {
            if let Ok(val) = etag.to_str() {
                *self.cached_etag.lock().unwrap() = Some(val.to_string());
            }
        }

        let catalog: CatalogResponse = res.json().await?;
        Ok(Some(catalog))
    }

    /// Fetches the full JSON Schema for a single capability.
    pub async fn describe_capability(&self, id: &str) -> Result<Value> {
        let url = format!("{}/v1/capabilities/{}", self.base_url, id);
        let res: Value = self.client.get(&url).send().await?.json().await?;
        Ok(res)
    }

    // ------------------------------------------------------------------
    // Tool execution
    // ------------------------------------------------------------------

    /// Calls a capability tool and returns the response envelope.
    ///
    /// Optionally pass an `idempotency_key` for safe retries of mutating
    /// operations.
    pub async fn call_tool(
        &self,
        capability_id: &str,
        args: Value,
        idempotency_key: Option<&str>,
    ) -> Result<Envelope> {
        let url = format!("{}/v1/tools/call", self.base_url);

        let mut body = serde_json::json!({
            "capability_id": capability_id,
            "args": args,
        });

        let mut req = self.client.post(&url).json(&body);

        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }

        let envelope: Envelope = req.send().await?.json().await?;

        if !envelope.ok {
            if let Some(ref err) = envelope.error {
                bail!("[{}] {}", err.code, err.message);
            }
        }

        Ok(envelope)
    }

    /// Calls a capability with context distillation modifiers.
    ///
    /// Use `jsonpath` to select specific fields, `limit_lines` to cap
    /// output length, and `truncate_bytes` for a hard byte budget.
    pub async fn call_tool_distilled(
        &self,
        capability_id: &str,
        mut args: Value,
        jsonpath: Option<&str>,
        limit_lines: Option<usize>,
        truncate_bytes: Option<usize>,
    ) -> Result<Envelope> {
        if let Some(jp) = jsonpath {
            args["_jsonpath"] = Value::String(jp.to_string());
        }
        if let Some(ll) = limit_lines {
            args["_limit_lines"] = Value::Number(ll.into());
        }
        if let Some(tb) = truncate_bytes {
            args["_truncate_bytes"] = Value::Number(tb.into());
        }
        self.call_tool(capability_id, args, None).await
    }

    /// Executes a chained batch of tool steps in a single round-trip.
    ///
    /// Steps can reference outputs from earlier steps using `$step_id.field`
    /// variable interpolation syntax.
    pub async fn batch_call(&self, steps: Vec<Value>) -> Result<Value> {
        let url = format!("{}/v1/tools/batch_call", self.base_url);
        let body = serde_json::json!({ "steps": steps });
        let res: Value = self.client.post(&url).json(&body).send().await?.json().await?;
        Ok(res)
    }

    // ------------------------------------------------------------------
    // Search
    // ------------------------------------------------------------------

    /// Searches capabilities using hybrid lexical + semantic ranking.
    pub async fn search_capabilities(
        &self,
        query: &str,
        limit: usize,
        server_ids: &[&str],
        tags: &[&str],
    ) -> Result<SearchResponse> {
        let url = format!("{}/v1/capabilities/search", self.base_url);
        let body = serde_json::json!({
            "query": query,
            "limit": limit,
            "server_ids": server_ids,
            "tags": tags,
        });
        let res: SearchResponse = self.client.post(&url).json(&body).send().await?.json().await?;
        Ok(res)
    }

    // ------------------------------------------------------------------
    // Resources & Prompts
    // ------------------------------------------------------------------

    /// Lists all registered resources.
    pub async fn list_resources(&self) -> Result<Value> {
        let url = format!("{}/v1/resources", self.base_url);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    /// Reads an upstream resource by ID.
    pub async fn read_resource(&self, resource_id: &str) -> Result<Value> {
        let url = format!("{}/v1/resources/read", self.base_url);
        let body = serde_json::json!({ "resource_id": resource_id });
        Ok(self.client.post(&url).json(&body).send().await?.json().await?)
    }

    /// Lists all registered prompt templates.
    pub async fn list_prompts(&self) -> Result<Value> {
        let url = format!("{}/v1/prompts", self.base_url);
        Ok(self.client.get(&url).send().await?.json().await?)
    }

    /// Renders a prompt template with arguments.
    pub async fn get_prompt(&self, prompt_id: &str, arguments: Value) -> Result<Value> {
        let url = format!("{}/v1/prompts/get", self.base_url);
        let body = serde_json::json!({
            "prompt_id": prompt_id,
            "arguments": arguments,
        });
        Ok(self.client.post(&url).json(&body).send().await?.json().await?)
    }
}
```

---

## Usage Examples

### 1. Discover and list capabilities

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wp = WarmplaneClient::new("http://127.0.0.1:9090");

    // First call fetches the full catalog
    let catalog = wp.list_capabilities().await?
        .expect("should return catalog on first call");

    println!("Catalog version: {}", catalog.catalog_version);
    for cap in &catalog.capabilities {
        println!("  {} — {}", cap.id, cap.summary);
    }

    // Second call uses ETag — returns None on 304
    if wp.list_capabilities().await?.is_none() {
        println!("Catalog unchanged (304 Not Modified)");
    }

    Ok(())
}
```

### 2. Describe a capability and call it

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wp = WarmplaneClient::new("http://127.0.0.1:9090");

    // Fetch full schema to understand required arguments
    let schema = wp.describe_capability("filesystem.read_file").await?;
    println!("Schema:\n{}", serde_json::to_string_pretty(&schema)?);

    // Execute the tool
    let result = wp.call_tool(
        "filesystem.read_file",
        serde_json::json!({ "path": "/tmp/example.txt" }),
        None,
    ).await?;

    println!("Result: {}", serde_json::to_string_pretty(&result.data)?);
    Ok(())
}
```

### 3. Context distillation — limit large outputs

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wp = WarmplaneClient::new("http://127.0.0.1:9090");

    let result = wp.call_tool_distilled(
        "sqlite.read_query",
        serde_json::json!({ "query": "SELECT * FROM users" }),
        Some("$.records[*].email"),  // JSONPath filter
        Some(10),                     // Max 10 lines
        None,
    ).await?;

    println!("{}", serde_json::to_string_pretty(&result.data)?);
    Ok(())
}
```

### 4. Multi-step chained batch call

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wp = WarmplaneClient::new("http://127.0.0.1:9090");

    let result = wp.batch_call(vec![
        serde_json::json!({
            "id": "step1",
            "capability_id": "db.get_customer",
            "args": { "customer_id": "cust_123" }
        }),
        serde_json::json!({
            "id": "step2",
            "capability_id": "stripe.get_invoice",
            "args": { "invoice_id": "$step1.latest_invoice_id" },
            "continue_on_error": false
        }),
    ]).await?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
```

### 5. Search capabilities with hybrid ranking

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wp = WarmplaneClient::new("http://127.0.0.1:9090");

    let results = wp.search_capabilities(
        "find production error logs",
        5,
        &[],     // all servers
        &[],     // all tags
    ).await?;

    for cap in &results.capabilities {
        println!("[{:.2}] {} — {}", cap.score, cap.id, cap.summary);
    }
    Ok(())
}
```

### 6. Idempotent tool call with retry safety

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wp = WarmplaneClient::new("http://127.0.0.1:9090");
    let key = "payment-charge-ord-42";

    // First call executes upstream
    let r1 = wp.call_tool(
        "payments.charge",
        serde_json::json!({ "amount": 100, "currency": "USD" }),
        Some(key),
    ).await?;
    println!("First:  {:?}", r1.retry);

    // Second call with same key returns cached result — no re-execution
    let r2 = wp.call_tool(
        "payments.charge",
        serde_json::json!({ "amount": 100, "currency": "USD" }),
        Some(key),
    ).await?;
    println!("Second: {:?}", r2.retry);

    Ok(())
}
```

---

## Response Envelope

All tool execution and resource read responses follow the standard envelope:

```json
{
  "ok": true,
  "request_id": "req-994812",
  "trace_id": "8f2a1b3c4d5e6f7a",
  "data": { "result": "..." },
  "error": null,
  "retry": {
    "classification": "safe",
    "upstream_execution_state": "completed"
  }
}
```

When `ok` is `false`, inspect `error.code` against the [standard error codes](USER-GUIDE.md#67-standard-error-codes) for programmatic handling:

| Code | HTTP | Meaning |
|:---|:---:|:---|
| `TOOL_NOT_FOUND` | 404 | Capability ID does not exist or is policy-blocked |
| `INVALID_ARGS` | 400 | Arguments failed JSON schema validation |
| `POLICY_DENIED` | 403 | Blocked by deny rule |
| `APPROVAL_PENDING` | 202 | Intercepted by HITL — awaiting operator decision |
| `CIRCUIT_OPEN` | 503 | Upstream circuit breaker tripped |
| `UPSTREAM_TIMEOUT` | 504 | Upstream exceeded `toolTimeoutMs` |

See the [User Guide](USER-GUIDE.md) for the complete error code table and the [Whitepaper](WHITEPAPER.md) for architectural context.

---

## Next Steps

- **ETag caching**: The `WarmplaneClient` above caches one ETag for the capabilities list. Extend this pattern to resources and prompts endpoints.
- **SSE streaming**: Connect to `GET /v1/resources/updates` for real-time catalog mutation events using `reqwest`'s streaming API.
- **HITL integration**: Poll `GET /v1/approvals` to build operator approval workflows into your application.
- **Audit queries**: Use `GET /v1/audit/events` to pull execution history for compliance reporting.
