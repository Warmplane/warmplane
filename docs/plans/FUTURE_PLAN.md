# Warmplane future work

**Status** Proposed roadmap. **Baseline** Warmplane v0.34.0.

Warmplane keeps upstream MCP sessions warm and gives agents a compact, policy-governed interface. The next product goal is to make that same interface practical for internal APIs, large tool catalogs, and teams that need to review or recover from tool actions. This plan orders work by dependency and defines evidence for deciding when a change is ready to ship. Release numbers, dates, and performance budgets belong in the individual checkpoint plans.

## What already ships

| Foundation | How future work uses it |
| --- | --- |
| RBAC | Static tokens and JWT verification resolve tenant and role for the authenticated HTTP surfaces. Reuse caller context, catalog filtering, role policy, and audit attribution. Verify parity with each interface's supported identity model as scope expands. |
| Profiles | Named server constellations narrow discovery and execution while upstream sessions stay warm. Profiles contribute policy and use scoped catalog validators. An optional profile provides task scope. A mandatory restriction belongs in authenticated policy or deployment configuration. |
| Compact discovery | Index-first discovery, on-demand descriptions, and hybrid lexical/vector search provide the baseline for retrieval improvements. |
| Governed execution | Approval, idempotency claims, retry classification, cancellation, task receipts, and audit export provide the common execution path. |
| Stateful recourse | Holds, snapshots, declared inverses and compensation, drift checks, and work-item rollback provide the execution model for new authoring tools. |

Tenant and role describe who is acting. Profile describes the requested server subset and task policy. An operation is an upstream action exposed as a capability. A fingerprint identifies an approved revision of that capability's definition. A reversibility contract is an operator-approved declaration of how to delay, undo, or compensate an action. A shortlist is an ordered view of capabilities eligible for discovery under the current authorization and profile.

See the [changelog](../../README.md#changelog), [User Guide](../USER-GUIDE.md), [RBAC design history](implemented/MULTI_TENANT_RBAC.md), and [Profiles design history](implemented/PROFILES_PLAN.md).

## Product priorities

1. **Connect internal APIs under the same controls as MCP servers.** OpenAPI onboarding should take a configured spec and an operation allowlist, with no wrapper service to maintain.
2. **Keep tool output and metadata within operator policy.** Operators should be able to approve changes to a tool's definition and set what parts of its output may reach the downstream agent.
3. **Find the right tools in large catalogs.** Improve the existing authorized search path and measure whether a smaller candidate set helps agents pick the right capability.
4. **Make recovery easier to configure and verify.** Give operators a way to draft and test recourse contracts before approving them for a live target.
5. **Keep execution local while coordinating fleet policy and evidence.** Start with policy distribution, approvals, and audit collection across named workstations. Expand execution topology only after testing ownership and failure semantics.

The market draft makes a case for owning these workflows at the tool boundary. The product hypothesis is that teams will choose Warmplane when onboarding, selection, execution, and recovery work together in a self-hosted runtime. Validate that hypothesis with an internal API onboarding pilot, a large-catalog evaluation, and a fleet pilot before committing to fleet scale or a 1.0 scope. [Market analysis and original roadmap](archive/FUTURE_DRAFT.md)

## Architecture rules

- Derive caller identity from trusted authentication. Apply profile membership and the effective authorization policy to discovery, detail, search, and dispatch. Test equivalent decisions for capability aliases, native tools, batches, resources, and prompts where supported. Rank only eligible candidates.
- Reuse the existing execution path for new sources and delayed actions. Revalidate current authorization, approval requirements, and any required pinned definition at dispatch, including after a hold or approval. Scope replay, task access, and audit views to the appropriate caller boundary.
- Derive catalog validators and cursors from the effective view and revision. Scope cached results to the identity, policy, profile, and query inputs that affect visibility. A changed view must never validate a stale representation from a different scope.
- Preserve existing stdio, HTTP, and embedded entry points and on-disk compatibility. Gate new behaviors through explicit configuration. Document compatibility per transport and negotiated MCP revision.
- Protect secrets at output and storage boundaries. Emit canonical audit events with verified provenance, decision, and outcome. Keep sensitive result content out of audit records and SIEM exports. Record uncertainty when an upstream side effect may have occurred.

## Work and checkpoint criteria

These are candidate release cuts. Each can ship once its dependencies and acceptance criteria are met. Retrieval evaluation can start alongside the trust work. The fleet pilot depends on the trust baseline and can proceed independently of the OpenAPI and retrieval work when its local MCP use case is ready.

### 1. Trust controls for existing integrations

See the [Trust Controls Plan](TRUST_CONTROLS.md) for implementation decisions, migration, and acceptance criteria.

- Add opt-in result sanitization for structured fields and text patterns. Specify the supported output types, rule composition, and payload limits. Apply the configured rules before downstream delivery and enforce them on task results, replay, errors, and batch-derived data at their respective disclosure boundaries. Define how protected operational state used by snapshots and inverse templates is stored and accessed. Consider local model-based detection after measuring deterministic rules.
- Add administrator-approved fingerprints over canonical, execution-relevant capability definitions, including descriptions and schemas. Compare the currently observed definition with the approved revision during refresh and at dispatch. Quarantine changed or unverified capabilities under pinning policy until an authorized operator approves the revision. Invalidate affected catalog and search views, and reuse signed webhooks for change notifications.
- Extend existing ChatOps action callbacks where approval receipts or delivery feedback are missing. Scope receipts to the action and verified tenant, with expiry and replay handling. Evaluate Slack and Teams adapters against the same callback contract.

**Ready when** tests cover definition changes during holds and approvals, authorization or profile changes on replay, malformed or oversized results, and sensitive fixture data across synchronous, batch, and async delivery. Sanitization failure under enforcing policy withholds the affected result and records the outcome of the upstream invocation when known. A pinning or pre-snapshot failure detected before dispatch blocks the upstream call.

### 2. OpenAPI onboarding and reversibility authoring

- Ingest operator-selected OpenAPI 3.0 and 3.1 operations as a capability source. Start with explicitly allowlisted, read-only operations from local specs. Add controlled remote spec ingestion and explicitly enabled mutations as follow-on work. Generate stable IDs and compact signatures from operation metadata. Document supported path/query/body serialization, content types, references, and response mapping. Address unsupported input during validation, before activation.
- Keep configured base URLs and credentials under operator control. Keep credentials out of schemas and downstream results. Define reference resolution, outbound network policy, redirects, size limits, timeouts, and spec reload behavior before enabling remote ingestion. Use normal authorization, pinning, audit, retry, and output policy for generated capabilities. Expose the existing output shaping options where the returned type supports them.
- Generate **draft** inverse and pre-snapshot templates from explicitly matched operations and field mappings. Treat HTTP method and naming conventions as hints. Require an operator to verify semantics, target identity, permissions, and drift handling. Keep irreversible operations explicit. Retain compensation limits and approval checks for approved compensable actions.
- Add a Control Deck recorder after the draft format and review flow are proven. It captures a pre-state read, an explicitly authorized forward action, and the observed result against a configured test target. Present a schema-checked JSON or YAML draft for review. Activation requires a separate authenticated approval. Keep declarations usable with existing MCP capabilities as well.

**Ready when** a sample internal API works without a wrapper and tests cover stable IDs, allowlist and parameter handling, authentication failures, incompatible spec updates, and zero upstream dispatch on authorization denial. A recourse pilot demonstrates approved reversal, drift blocking, failed inverse reporting, and draft isolation from live configuration. Writes, template generation, and the recorder can each be reviewed as separate cuts after the read-only importer.

### 3. Query-aware discovery

- Extend explicit query-aware discovery through REST and the MCP capability search tool. Compare lexical, dense, and hybrid retrieval using the existing index and on-demand definitions. Use a configurable shortlist size and deterministic tie-breaking. Preserve explicit search and describe access to eligible tools outside the shortlist. Define a fallback for missing embeddings and empty or low-confidence results.
- Add an opt-in host integration only where the host deliberately supplies a query. Standard MCP `tools/list` retains its authorization-scoped catalog behavior. Its request carries no conversation turn, and the 2026-07-28 revision constrains tool lists from varying as a side effect of other requests. Keep query-based results separate from standard catalog caching and pagination. [MCP tools specification, 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) · [MCP tools specification, 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
- Evaluate chunked execution envelopes separately. Keep the existing response path until a streaming design specifies framing, sanitization before delivery, cancellation, terminal status, and replay.

**Ready when** a reproducible evaluation reports top-10 hit rate, end-to-end top-1 selection accuracy, schema tokens, and p50/p95 latency against current Warmplane search. Test filtering before ranking and changes to eligibility between discovery and execution. Set release thresholds from that baseline and the target hardware. HumanMCP reports 87.6% top-10 hit rate and 76% downstream selection accuracy for its dense retriever configuration on 2,000 tools. Use the study as a comparison point and include Warmplane-specific tasks and existing token-efficiency suites. [HumanMCP evaluation](https://arxiv.org/html/2602.23367v1#S5.SS3) · [Warmplane token-efficiency harness](../../eval/token-efficiency/README.md)

### 4. Hub-and-spoke pilot

- Introduce a self-hosted Hub for enrolled Spoke identities, versioned policy, approval coordination, and audit intake. Use authenticated outbound connections from workstation Spokes. Spokes own local upstream sessions and execution state. Keep local execution credentials at the Spoke.
- Specify enrollment, rotation, revocation, policy expiry, and offline behavior. Bind approval decisions to tenant, target, arguments, policy revision, and expiry. Revalidate at the Spoke before dispatch. Define when a lost Hub connection blocks work and how pending work is reconciled after reconnect.
- Preserve ordered, verifiable audit streams per Spoke. Bind each stream to the enrolled identity and use authenticated checkpoints to verify across retention boundaries. The Hub verifies continuity, deduplicates replays, and flags gaps, forks, or stale streams. The aggregated view correlates events across Spokes without asserting global event order.
- Evaluate clustered Hub storage and cross-node execution only after the pilot. Any movable execution ownership needs durable claims, fencing, replay protection, and reconciliation for uncertain upstream outcomes. Publish compliance evidence mappings after applicable controls and evidence have been reviewed.

**Ready when** the pilot demonstrates onboarding without inbound workstation ports, policy revocation and expiry, tenant-scoped evidence, and gap/fork detection through disconnect and reconnect. Verify approval replay handling and unchanged ownership of in-flight mutations during Hub outages. Use pilot measurements to set connection-count, event throughput, and recovery targets before proposing production scale.

## Release verification

| Given | When | Expected |
| --- | --- | --- |
| A caller cannot discover a capability under the active role, profile, or policy | The caller searches, describes, invokes by ID, or reaches it in a batch | Discovery omits it and dispatch is blocked. |
| A caller presents a validator or cursor from a different eligible catalog view | The request resolves to the current view | The validator cannot produce a `304`. The cursor is rejected and the caller can restart pagination. |
| A pinned capability changes after approval | A held or queued invocation is released | It cannot dispatch under the stale approval. Its blocked status is visible to authorized operators. |
| An upstream mutation completes and its result cannot be sanitized | The client polls or retries | The client receives a policy-safe outcome and the result filter failure alone cannot cause a new upstream mutation. |
| A reviewed contract has a drift check and intervening state has changed | An inverse is requested | The inverse is blocked and the operator can see the failure. |
| The Hub observes a gap or conflicting event in a Spoke's audit sequence | It verifies intake | The stream exposes an integrity issue to authorized operators. |

For each checkpoint, record the interface and configuration changes, supported deployment modes, tests and benchmark fixtures, migration and rollback procedure, and behavior of outstanding work during upgrade. Run the repository's release checks and update the Warmplane skill and relevant user and integrator guides as behavior changes.

## Decisions for checkpoint planning

- Which internal API and supported OpenAPI subset form the first onboarding fixture? Is remote spec ingestion required in that release?
- Which output kinds and policy rule types are supported at launch? How should existing cached results be handled when sanitization becomes stricter?
- Who can approve a changed definition, and can a policy require every capability on a source to be pinned before that source is usable?
- Which clients can pass task context explicitly, and what query handling and latency budgets apply to each?
- Which test-target verification is required before the recorder can invoke a probe? Which contract changes require reapproval?
- Can a disconnected Spoke execute under a still-valid cached policy, and which capabilities require a live Hub decision? What can a Spoke operator observe versus a Hub operator?
- Which adoption and operational results justify moving the fleet pilot, streaming, or compliance evidence packs into a named release?
