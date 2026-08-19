# Your AI Agent Just Charged That Customer Twice

## The idempotency problem nobody talks about in AI tool calling

Here is a scenario that will happen to you if it hasn't already:

Your AI agent is calling a payment API. The network hiccups. The HTTP request times out. The agent retries. The customer gets charged twice.

Or maybe it's not payments. Maybe it's a database write. A deployment trigger. A Slack message to your CEO. The agent doesn't know the first call actually went through. It only knows it didn't get a response. So it tries again. And now you have a problem.

This is not a hypothetical. This is what happens when you connect AI agents directly to tools without thinking about execution governance.

---

## The "it works on my laptop" era of AI tool calling

Right now, most MCP setups look like this: an AI client connects directly to a bunch of MCP servers. Each connection is independent. Each call is fire-and-forget. There is no shared state, no deduplication, no record of what happened.

This works fine for demos. It works fine when you're reading files or searching logs. Read-only operations are forgiving.

But the moment your agent starts *doing things* (creating records, sending emails, deploying code, moving money) you need answers to questions that direct MCP connectivity doesn't even ask:

1. **Did this call already happen?** If the agent retries, will it cause a duplicate?
2. **Is it safe to retry?** Was this a read or a write? Did the upstream actually execute it?
3. **Who approved this?** Did a human sign off before the agent ran `DROP TABLE`?
4. **What exactly happened?** Can you prove it, six months later, to an auditor?

These are not edge cases. These are table stakes for any system that does real work.

---

## Idempotency is not optional

Idempotency means: if you send the same request twice, you get the same result, and the side effect only happens once. It's a solved problem in payment processing, message queues, and distributed systems. But in AI tool calling, almost nobody implements it.

Warmplane does. Here's what it looks like:

```bash
curl -X POST http://127.0.0.1:9090/v1/tools/call \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: charge-order-42" \
  -d '{
    "capability_id": "payments.charge",
    "args": {"amount": 100, "currency": "USD"}
  }'
```

The first time this request arrives, Warmplane executes it upstream and stores the result. The second time (same key), it returns the stored result without touching the upstream server. The customer gets charged once.

This works across retries, across agent restarts, across daemon restarts. The idempotency records are atomically persisted to disk. They survive crashes.

And every response tells you exactly what happened:

```json
{
  "ok": true,
  "data": { "charge_id": "ch_abc123" },
  "retry": {
    "classification": "idempotent",
    "upstream_execution_state": "completed"
  }
}
```

That `retry` block is not decoration. It tells the agent (and you): this was a mutating operation, the upstream executed it, and it's safe to use this result. Compare that to a raw MCP response, which gives you... the tool output. Nothing about whether it's safe to retry. Nothing about whether the upstream actually ran.

---

## The real cost of "just connect directly"

Let's zoom out from idempotency. Here's what a typical 10-tool MCP setup costs you when each client connects directly:

**Token waste.** Every agent loop, the LLM receives full JSON schemas for all 10 tools. Most of them are irrelevant to the current task. In benchmarks against a GitHub Copilot MCP server, this overhead is 54,000+ tokens per discovery pull. Warmplane's compact index cuts that to 2,400 tokens. That's a 95.6% reduction.

**No policy enforcement.** There's no central place to say "agents can read but not delete" or "this tool requires human approval." Each client implements its own rules (or doesn't).

**No audit trail.** When something goes wrong at 2 AM, you're grepping through agent logs across five different systems trying to figure out what tool was called, with what arguments, by which agent, and whether it succeeded.

**No circuit breaking.** When an upstream server hangs, every agent calling it hangs too. There's no fast-fail, no cooldown, no automatic recovery.

Warmplane solves all of these with a single local process that sits between your agents and your MCP servers.

---

## What Warmplane actually is

Warmplane is a local control plane. It keeps persistent, warm connections to your upstream MCP servers and exposes a compact, governed interface to clients.

Think of it as a reverse proxy for AI tool calling, but one that actually understands what it's proxying.

It gives you:

- **Compact catalog indexes** that cut token usage by 58-96%, with SHA-256 ETag caching for zero-token revalidation on unchanged catalogs.
- **Idempotency deduplication** with crash-safe disk persistence.
- **Retry classification** (`safe` / `idempotent` / `unsafe`) so agents know what they can safely retry.
- **Human-in-the-loop approval gates** where sensitive operations suspend and wait for an operator to approve or reject, with HMAC-signed webhook notifications.
- **Cryptographic WORM audit logging** with SHA-256 hash chaining, so you can prove the entire execution history hasn't been tampered with. Streams to Splunk and webhooks in real time.
- **Circuit breakers** that fast-fail at 50 nanoseconds when an upstream is down, instead of hanging for 15 seconds.
- **Context distillation** that lets you JSONPath-filter, line-limit, and byte-truncate tool outputs before they eat your context window.
- **Batch execution** that chains multiple tool calls in a single round-trip with variable interpolation between steps.

All of this behind a simple HTTP API, a CLI, a web dashboard, and an MCP stdio proxy that works with Claude Desktop, Cursor, and Zed.

---

## The "house-guest benchmark"

We keep coming back to this analogy: connecting an AI agent directly to raw MCP servers is like inviting a house guest who, every time they want a glass of water, asks you to describe every room in the house, every appliance in the kitchen, every item in every drawer, and then finally says "I'd like some water please."

Warmplane is the host who says: "Here's what's available" (compact index). "Want details on something specific?" (on-demand schema). "Here you go" (governed execution with a receipt).

The guest gets their water. The host knows what happened. Nobody had to describe the entire house.

---

## Getting started

Warmplane is a single Rust binary. It auto-discovers your existing MCP servers from Claude Desktop, Cursor, and Zed:

```bash
cargo install --path .
warmplane config import
```

Then add Warmplane as the single MCP server in your AI client config:

```json
{
  "mcpServers": {
    "warmplane": {
      "command": "warmplane",
      "args": ["mcp-server", "--config", "/path/to/mcp_servers.json"]
    }
  }
}
```

Same tools, same workflows. Now with governance, compact catalogs, and idempotency. Zero changes to your upstream servers.

---

## The bottom line

AI agents are graduating from "clever demos" to "systems that do real work." That transition requires the same infrastructure guarantees we expect from any production system: idempotency, audit trails, access control, circuit breaking, and operational visibility.

You can build all of that yourself, per-client, per-tool, per-agent. Or you can put a control plane in front of your MCP servers and get it for free.

That's Warmplane.

[GitHub](https://github.com/Warmplane/warmplane) | [User Guide](USER-GUIDE.md) | [Whitepaper](WHITEPAPER.md) | [Rust Integration](RUST_INTEGRATORS_GUIDE.md) | [TypeScript Integration](TYPESCRIPT_INTEGRATORS_GUIDE.md)
