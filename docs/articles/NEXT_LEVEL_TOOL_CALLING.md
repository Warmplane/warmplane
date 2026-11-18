# Next-Level Tool Calling

Most teams think tool calling costs come from execution.
In practice, a huge share comes from *description overhead*: sending large tool schemas and metadata before useful work starts.

That is where `Warmplane` changes the game.

## The Core Idea

Keep MCP richness in the backend, but present agents with a compact, lazy, stable facade:

- `GET /v1/capabilities`
- `GET /v1/capabilities/:id`
- `POST /v1/tools/call`
- plus compact resources/prompts surfaces

This moves agent interaction from “dump everything up front” to “index first, detail on demand.”

## What We Measured (Not Hypothetical)

From `eval/token-efficiency/output/report.md`:

### GitHub Copilot MCP (authenticated)

- Discovery scenario:
  - Raw: `54,715` tokens
  - Facade: `2,386` tokens
  - Savings: `52,329` tokens (`95.6%`)
- 5-turn tool loop:
  - Raw: `260,005`
  - Facade: `11,173`
  - Savings: `248,832` (`95.7%`)
- 10-turn mixed loop:
  - Raw: `547,150`
  - Facade: `22,895`
  - Savings: `524,255` (`95.8%`)

### Public filesystem MCP (no-auth control)

- Discovery:
  - Raw: `2,552`
  - Facade: `1,066`
  - Savings: `1,486` (`58.2%`)
- 5-turn tool loop:
  - Raw: `12,760`
  - Facade: `5,349`
  - Savings: `7,411` (`58.1%`)
- 10-turn mixed loop:
  - Raw: `25,520`
  - Facade: `10,669`
  - Savings: `14,851` (`58.2%`)

Interpretation: savings are not a narrow edge case. They range from strong (~58%) to extreme (~96%), depending on upstream schema density.

## Why Plain MCP Connectivity Loses at Scale

Direct connectivity is great for compatibility, but costly for repeated agent loops:

1. Agents repeatedly ingest oversized tool surfaces.
2. Planning context gets crowded with metadata.
3. Every model/provider integration has to cope with raw heterogeneity.

A compact facade avoids all three:

1. Much smaller default context payload.
2. Detail fetched only for selected capabilities.
3. One normalized envelope for calls, errors, and policy behavior.

## This Is Not Just a Token Story

Token savings are the headline. Operational gains are the moat:

- deterministic call/error envelopes
- centralized policy, redaction, and audit controls
- stable contract across CLI, HTTP, and MCP-native clients
- backend MCP server churn decoupled from agent prompt stability

In short: you get both lower spend and better systems engineering.

The “up to 96%” claim is grounded in measured scenarios above, not speculation.

## What “Next Level” Means

The first era of MCP proved protocol interoperability.
The next era is interface efficiency.

`Warmplane` makes that shift concrete:

- keep MCP on the backend for breadth
- expose a thin, high-signal capability plane to agents
- pay for detail only when it is actually needed

That is how you go from “tool connectivity” to “production-grade tool economics.”
