# Phase 2 — OpenAPI onboarding and reversibility authoring

**Status** Proposed. **Baseline** Warmplane v0.35.0. **First release cut** Read-only OpenAPI onboarding. Assign release numbers after the pilot.

Phase 2 lets an operator expose selected internal HTTP operations as Warmplane capabilities without maintaining an MCP wrapper. Generated capabilities inherit the trust controls shipped in v0.35.0. Later cuts add remote spec loading, writes, reversibility drafts, and evidence capture on a dedicated test target. See the [v0.35.0 changelog](../../README.md#v0350--trust-controls-for-existing-integrations) and [Phase 1 plan](TRUST_CONTROLS.md).

## Lexical audit

| Term | Meaning |
| --- | --- |
| Source | An operator-configured API with a Warmplane server ID, a spec location, a request base URL, credentials, and an operation allowlist. Profiles select sources using the existing server identity model. |
| Spec endpoint | The HTTPS URL Warmplane reads to obtain a remote specification. Its destination and credentials are configured independently of those used to invoke the API. |
| Operation | One HTTP method and path template in the spec. The first cut selects it by an exact, unique `operationId`. |
| Capability ID | The existing server-qualified tool ID derived from the source ID and the selected `operationId`. It identifies the operation across document reordering and metadata edits. An explicit rename or move between sources changes the ID. |
| Definition revision | The canonical metadata and HTTP request/response contract reviewed by an operator. A capability ID can remain stable while its definition revision changes. |
| Draft | A proposed instance of the existing reversibility contract, tied to the source and capability revisions used to generate it. It may be edited and reviewed before an authorized configuration change activates it. |
| Test target | A separately configured non-production source with its own credentials and fixed or explicitly reviewed test arguments. The recorder uses this source for every probe. |

## Logic and state translation

### Checkpoint 2A — Local read-only sources

| Surface | Initial contract |
| --- | --- |
| Configuration | Add an OpenAPI variant under the existing source registry so a source occupies one server ID and uses normal profile membership. An OpenAPI entry and MCP command/URL fields are mutually exclusive. Require a local spec path, explicit base URL, nonempty operation allowlist, and an explicit auth choice. Resolve relative spec paths from the configuration directory. Start with no auth and environment-backed bearer auth. Validate the configured choice against each selected operation's effective security requirements. |
| Spec input | Accept local JSON and YAML documents in OpenAPI 3.0.x and 3.1.x. For selected operations, resolve only bounded, acyclic same-document JSON Pointer references. Reject unsupported or ambiguous constructs affecting request construction, authentication, or selected success response types before activation. Unselected operations do not produce capabilities. |
| Selection and IDs | Allow explicit `GET` operations whose owners confirm read-only semantics. Require unique IDs that fit existing tool naming rules. Reject name collisions and invalid IDs without silently rewriting them. Treat a missing allowlisted operation as a configuration error. |
| Arguments | Support required scalar `simple` path parameters and required or optional scalar `form` query parameters. Apply operation-level parameter overrides by location and name. Use distinct `path` and `query` argument objects, validate declared types and supported constraints, and reject unknown fields. Omit absent optional values. Defer request bodies, header and cookie arguments, collections, and unsupported serialization. |
| Destination | Use the operator's base URL, including its path prefix. Spec server declarations, examples, and request arguments cannot choose the destination or supply credentials. Require HTTPS for deployment. Permit explicit loopback HTTP for the local fixture. Disable redirects, use normal TLS verification, and enforce the configured destination and path prefix after URL construction. Private internal origins must be explicitly configured. |
| Response | Accept a declared successful status with a JSON body, or a declared `204` with no body. Map JSON to the existing structured tool result and use the normal policy and result shaping path. Map other statuses, malformed bodies, unexpected media types, and transport failures to safe upstream errors. Keep raw headers, request URLs with values, and upstream error bodies out of client diagnostics and logs. |
| Reload and removal | Compile and validate a complete candidate before replacing the active source. A failed reload reports the source and reason and retains its previous revision. A valid edit activates a single revision for the worker and catalog, invalidates affected views, and re-evaluates pins. Explicit source removal withdraws its capabilities and prevents queued calls from dispatching against it. At startup, an invalid source is unavailable and reported to the operator. |

The first pilot is a fixture-backed internal inventory API exposing `getAsset` and `listAssets` under a base path. Provide equivalent 3.0 and 3.1 specs, one local reference, a bearer credential, a filtered query, and a result containing a synthetic sensitive field. Demonstrate configuration, discovery, invocation, pin review, sanitization, and removal. Follow with one real internal API to validate that the supported subset is useful. The OpenAPI documents define the source format and serialization semantics. [OpenAPI 3.0](https://spec.openapis.org/oas/v3.0.4.html), [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.1.html)

#### Implementation boundaries

- Compile immutable request plans in the configuration and source-loading path. Reuse [daemon lifecycle](../../src/daemon/lifecycle.rs) for registration and removal. Stage OpenAPI candidates before publishing worker or catalog state. The existing MCP reload updates policy and profiles separately, so require consistent publication for the OpenAPI source revision and test authorization again at dispatch.
- Register generated [capability metadata](../../src/daemon/types.rs) and signatures through the same catalog used by REST, MCP, CLI, and embedded clients. Put HTTP request construction behind the upstream worker boundary. Reuse the [engine](../../src/engine/mod.rs) for RBAC, profiles, approvals, idempotency, and batches. Route successful results through Phase 1 sanitization before task storage, replay, interpolation, and delivery.
- Extend [tool definition pins](../../src/tool_pins.rs) with a source-specific canonical definition that covers the method, destination and path template, argument and response contracts, and metadata shown to the operator. Keep credential values out of digests and review views. Credential rotation within the same configured identity can preserve pins. Changing the configured credential binding or destination requires review. Preserve existing MCP digest compatibility. A validated local revision remains the current observation until reload or removal. Reuse persistent approvals and the final worker check.
- Give operators a Control Deck source status view with the source type, safe load diagnostics, operation count, and existing pin review. Use local configuration for authoring in 2A. Keep credential references, paths, and diagnostics within the existing administrative boundary, and omit secret values from UI/API responses. Verify CLI configuration and the JSON schema recognize the source variant.

| Given | When | Expected |
| --- | --- | --- |
| Equivalent specs with reordered keys or distinct whitespace | The source reloads | IDs and canonical pin digests match. A changed schema, destination, or serialization rule invalidates the affected pin. |
| A selected operation is missing, collides, uses unsupported serialization, or contains an external or cyclic reference | An operator activates a candidate | The candidate fails with a safe diagnostic. The existing revision, if any, remains callable under current policy. Spec loading performs zero outbound requests. |
| A caller lacks permission, has the wrong profile, or presents a stale pin | The caller invokes directly, through an alias, or as a batch step | The fixture sees zero HTTP requests. A held request prepared against a changed definition cannot dispatch after release. |
| A caller supplies a missing, extra, incompatible, or escaping argument | Warmplane validates the request | It rejects invalid input before sending. Valid values produce the exact expected method, encoded path, and query and stay within the base path. |
| The fixture returns a redirect, auth failure, oversized payload, or timeout | The adapter processes the response | It emits a bounded diagnostic with no secret or raw body. A redirect receiver sees zero requests. |
| A successful HTTP response contains the sensitive fixture field | Delivery occurs synchronously, through a task or batch, or on replay | The policy applies to every supported delivery path. Raw protected values are absent from audit, SIEM, and tracing. |
| An operator removes the source or rolls back to v0.35.0 | The operator restores config and reloads or restarts | Removed capabilities and pending dispatches are unavailable. Restoring the previous config keeps existing MCP sources usable. Exercise removal in both daemon and embedded modes. |

### Checkpoint 2B — Operator-configured remote specs

Build on 2A's compiler and read-only invocation contract. The remote pilot serves the same inventory spec over HTTPS from a registry separate from the invocation API. Checkpoints 2B and 2C can ship independently once 2A is ready. Keep remote sources read-only until the combined fetch, pin, and write behavior has its own acceptance tests.

| Surface | Remote spec contract |
| --- | --- |
| Configuration | An operator chooses either a local path or an exact HTTPS spec URL per source. Require a DNS hostname, reject URL user information, query strings, and fragments, and keep the invocation base URL and operation allowlist explicit. Use the existing administrative configuration boundary. Discovery and invocation accept no spec URL. |
| Fetch and credentials | Use a dedicated HTTP client to issue a single `GET` per attempt. Use normal TLS certificate and hostname verification. Disable redirects, implicit proxies, cookies, and automatic retries. Support an explicit unauthenticated option or an environment-backed bearer credential dedicated to spec fetches. Invocation credentials and fetch credentials stay on their respective clients. |
| Network policy | Check all returned IPv4 and IPv6 addresses against the spec endpoint's egress policy before connecting. Permit globally reachable unicast by default and private internal addresses only within operator-configured CIDRs for that endpoint. Deny loopback, link-local, unspecified, multicast, and provider metadata destinations even when a private range is allowed. Normalize equivalent address forms and constrain the actual connection to the checked addresses while retaining TLS verification for the configured hostname. Reject mixed allowed and denied DNS answers. |
| Response and references | Accept `200` with a supported OpenAPI JSON or YAML media type. Apply the same parser and operation validation as 2A under the shared input budgets. Enforce the byte cap while reading and require identity content encoding for the first cut. Resolve same-document references only. For selected operations, reject external references before activation, even when the reference points to the same origin. Spec-defined server URLs and document metadata cannot cause a fetch or change the configured invocation destination. |
| Trigger | Fetch at startup and on explicit administrative reload. Use one in-flight fetch per source. Carry the source configuration generation through fetch and compile, and discard results superseded by a later configuration change or removal. Use a full bounded fetch on each attempt. Polling, conditional requests, and persisted fallback are later options. |
| Tool pins | Require persistent pin storage and pin enforcement for remote sources. A validated update publishes its worker and catalog revision together. New or changed definitions enter quarantine immediately. Unchanged canonical definitions retain approvals within the same source configuration. Revoke approval when a validated revision removes or changes a definition, even if a later fetch restores the old digest. When a configuration change withdraws an active revision, require fresh review for the replacement. |
| Operator status | Show the last fetch outcome, last successful validation time, active revision, and whether a prior revision is still serving. Audit the source ID, initiating actor or startup, result category, and revision. Keep credentials, raw document contents, response bodies, and rejected URLs out of diagnostics and audit. Render imported metadata as text in review views. |

Remote fetch failures obey the following state transitions. Egress rules apply on every fetch, and a fetch failure never retries from a different URL. [OWASP SSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)

| Given | When | Expected |
| --- | --- | --- |
| No active revision exists | Startup fetch, parse, or validation fails | The source stays unavailable. No capabilities appear. A subsequent successful reload can register it. |
| An active revision exists and source configuration is unchanged | A reload fails | Keep the previous validated in-memory revision, subject to current authorization and pin policy. Mark it as serving after a failed refresh. An operator can remove or disable it immediately. |
| An active revision exists | The operator changes the spec URL or its egress or credential binding, or changes invocation destination, invocation credential binding, or the operation allowlist | Withdraw the old revision from discovery and new dispatch before processing the replacement. A failed replacement remains unavailable. An older in-flight fetch cannot restore it. Calls already sent retain their existing outcome and audit semantics. |
| A valid spec changes a selected definition | Reload publishes the new revision | The fixture sees zero dispatches for that operation until review, including held calls prepared against the earlier definition. An unchanged spec or an edit confined to unselected operations retains existing approved pins. |
| A DNS answer contains a disallowed address, a redirect names a second endpoint, or a selected operation uses an external reference | Warmplane processes the fetch or compiles the document | A disallowed DNS answer blocks the initial connection. A redirect or reference results in zero requests to its target. Use fixture destinations and a controlled resolver to assert request counts and connection addresses. |
| A source is fetching | The operator removes the source or starts a replacement under a new configuration generation | Cancellation or discard prevents the result from republishing a withdrawn revision. Restart without a reachable spec host leaves the source unavailable. |

**Release gate.** Serve both fixture formats from the HTTPS registry with distinct fetch and invocation credentials. Verify no cross-use of credentials, a private origin with explicit CIDR approval, DNS policy changes between refreshes, TLS failure, forbidden redirects, byte and deadline limits, and diagnostic redaction. Demonstrate source removal and recovery by switching explicitly to a reviewed local spec. In daemon and embedded modes, show a successful reload, an unchanged pin, and a changed operation blocked until review.

### Checkpoint 2C — Explicitly enabled writes

- Add selected `POST`, `PUT`, `PATCH`, and `DELETE` operations with explicit per-operation mutation enablement. Support bounded JSON bodies and declared status/content mappings. Perform the same pre-activation validation and pin review as 2A. Method and spec metadata cannot opt an operation into automatic retries, replay, or reversibility.
- Use a separate authorization policy for the pilot's write operations and apply normal approvals, holds, and idempotency. Bind queued work to the reviewed revision. Treat a timeout or connection loss after possible submission as an uncertain outcome. Require a proven upstream idempotency contract before enabling automatic retries of submitted writes.
- Verify one reversible fixture mutation through REST and MCP with a shared execution path. Test duplicate keys, late completions, credential failure, restart during dispatch, unmount, and pin changes while work is pending. The release gate is zero duplicate upstream writes in replay and restart scenarios covered by the fixture. Remove or disable writes independently of the read-only source.

### Checkpoint 2D — Reversibility drafts

- Generate an existing [reversibility contract](../../src/config.rs) from explicit forward, pre-snapshot, and inverse capability selections and operator-supplied field mappings. Use the existing template syntax and [undo engine](../../src/engine/undo.rs). Support OpenAPI and MCP capabilities through the same editor and validators. Start with drafts for restoring previous state and leave current compensation limits and approval checks in force for compensable actions.
- Validate target identity, required fields, literal types, and template sources against the selected revisions. Surface unknown guarantees for dynamic values and require a checked test result before suggesting activation. Require expiry and a drift check for generated restore contracts. Re-resolve revisions at review and save. A changed revision invalidates the review and requires a fresh decision. A draft or export does not write live configuration. Saving requires the existing authenticated configuration authorization and a separate operator action.
- Gate the cut on a pilot that restores pre-state, blocks an inverse after intervening drift, reports a rejected inverse, and leaves live configuration byte-identical during draft generation, editing, and export. Cover a hand-authored contract and a generated draft through the same validation path.

### Checkpoint 2E — Control Deck recorder

- Extend the [reversibility editor](../../ui/src/components/reversibility-editor.ts) and reuse the existing undo test target and authorization model. Resolve every probe against the dedicated test source. Before submission show the target, operation, and redacted arguments and require explicit operator confirmation. Require a distinct source and credential binding and verify the recorded target is configured for test use before the first probe.
- Record a pre-state read, one governed forward invocation, and a post-state read. Retain only the field paths and sanitized evidence required for the draft. Treat denied or failed reads and uncertain writes as incomplete recordings. Never repeat an uncertain forward invocation automatically. Expose the failure and fixture cleanup outcome to the operator.
- Gate the cut on a successful run that produces a schema-checked JSON draft with no live config write. Verify that production targets, stale references, denied actions, and unconfirmed probes cannot trigger a call. An authenticated save uses the same review and validation as 2D.

## Quantification of constraints

These are proposed pilot limits. Exercise the boundary and one value beyond it. Measure validation and dispatch latency against the fixture and publish p50/p95 and payload sizes before setting a release latency target.

| Limit | Proposed value and behavior |
| --- | --- |
| Source input | 2 MiB per spec, 100 selected operations per source, and 64 levels of decoded nesting. Enforce a separate 10,000-node expanded reference budget per operation so shared references cannot expand without bound. |
| Remote fetch | Same 2 MiB spec limit, enforced while streaming. Use a 3-second connection timeout and 10-second overall fetch deadline including DNS, TLS, headers, and body. Limit parallel remote fetches across sources to four. |
| HTTP adapter request | 8 KiB encoded request target, 3-second connection timeout, and a 10-second total adapter call deadline within the existing execution deadline. Reject an oversized target before sending. |
| HTTP adapter response | 1 MiB decoded per response, enforced while reading. Keep the existing sanitizer's independent aggregate byte and depth limits when policy is enabled. |
| Release exercise | Run the existing Rust and UI checks and Slop Gate. Add integration evidence from a fixture request counter for pre-dispatch denials, identical-spec reloads, pin changes, and write replay. Verify the supported modes and migration with a local small MCP deployment alongside the new source. |

For each cut, update the Warmplane skill, User Guide, and relevant integrator guides, configuration schema, and API contracts. Update the architecture and Whitepaper when the source or recorder changes the trust boundary. Document disabling the new behavior, downgrade steps, and the fate of queued work.

## The inquisition — Decisions for checkpoint review

1. Must the first real pilot require authentication beyond one environment-backed bearer credential? Default to one supported credential and defer expansion unless the pilot requires it.
2. May a valid reload automatically activate a changed destination when pinning is disabled? Proposed default is to require an explicit configuration action for destination or credential binding changes and to test the effect on prepared calls.
3. May a production and test source share the same origin under different prefixes? Proposed default is a distinct origin for recorder probes until isolation can be verified independently.
4. May the first write pilot retry a request after possible submission? Proposed default is to require a confirmed upstream idempotency contract before enabling that behavior.
5. Must the first remote pilot resume after a restart while the spec host is unavailable? Proposed default is to require a successful fetch after restart. An operator can explicitly switch to a reviewed local spec during an outage.

## Inventory pilot

Run the [synthetic inventory API](../../tests/fixtures/openapi/inventory_pilot.py) on loopback with an isolated configuration. The fixture serves `getAsset` and `listAssets`, requires a synthetic bearer credential, and includes a field for testing result sanitization.

```sh
export WARMPLANE_INVENTORY_PILOT_TOKEN=synthetic-inventory-bearer
pilot_dir=$(mktemp -d)
python3 tests/fixtures/openapi/inventory_pilot.py "$pilot_dir" &
inventory_process=$!
trap 'kill "$inventory_process" 2>/dev/null || true' EXIT
while [ ! -s "$pilot_dir/mcp_servers.json" ]; do sleep 1; done
warmplane daemon --config "$pilot_dir/mcp_servers.json" --port 9296
```

Open Control Deck at `http://127.0.0.1:9296/ui`, connect through **Trust access** with the fixture credential `inventory-pilot-admin`, and approve both inventory definitions in **Tool definition pins**. In the Playground, call `inventory.getAsset` with `{"path":{"assetId":"A42"}}` and `inventory.listAssets` with `{"query":{"tag":"stock"}}`. Both responses should show `[REDACTED]` for the protected field.

The macOS arm64 debug-build pilot used a 2,192-byte spec, 40 unique authenticated reads, and 12 full reloads. Timing measured the complete local REST round trip, including durable local state where used.

| Pilot operation | Samples | p50 | p95 |
| --- | ---: | ---: | ---: |
| Authenticated asset read | 40 | 11.95 ms | 13.15 ms |
| Spec reload with unchanged definitions | 12 | 3.25 ms | 6.47 ms |

The pilot exercised CLI configuration validation, quarantine and approval, source authentication, query serialization, result sanitization, stable pins over reload, recovery from an invalid refresh, and source withdrawal. The [Rust integration fixtures](../../tests/openapi_sources.rs) exercise embedded calls, mutation replay and restart, and isolated contract recording using independent synthetic inventory instances.
