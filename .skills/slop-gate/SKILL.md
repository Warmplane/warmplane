---
name: slop-gate
description: Install and run Slop Gate for deterministic Rust code-quality checks in an agentic repository workflow. Use when an agent needs to create a baseline, evaluate commit-to-commit changes, scan a revision or working tree, or interpret gate results; do not use for non-Rust projects.
---

# Slop Gate

Use Slop Gate to detect newly introduced Rust function growth, near-clones,
lint suppressions, unsafe surface, and production/build dependency surface.
The `index` and `check` commands evaluate Git revisions. The `scan` command can
also inspect uncommitted working-tree edits.

## Prerequisite

Require `cargo` and a Git repository with the revisions to compare:

```sh
cargo --version
git rev-parse --show-toplevel
```

## Install

If `slop-gate` is not already available, install the published crate:

```sh
cargo install slop-gate --locked
slop-gate --version
```

Installation may need network access. Do not reinstall on every invocation;
reuse the installed binary when it is present.

## Agent workflow

1. Check for `.slop-gate.toml` at the repository root. Respect its rule
   severities, thresholds, and suppressions. A missing policy uses
   warning-only defaults. Do not silently change policy to make a check pass.
2. Select immutable commit IDs for `BASE` and `HEAD`. Resolve symbolic refs
   before invoking the tool when the workflow needs reproducibility:

   ```sh
   BASE="$(git rev-parse origin/main)"
   HEAD="$(git rev-parse HEAD)"
   ```

3. Build or reuse an artifact for the exact base commit and current policy:

   ```sh
   mkdir -p .slop-gate
   slop-gate index \
     --ref "$BASE" \
     --output .slop-gate/main.json
   ```

   The artifact is bound to both the commit and policy. Rebuild it if either
   changes. Keep `.slop-gate/` out of commits unless the repository explicitly
   chooses to version artifacts.

4. Evaluate introduced debt using JSON for machine interpretation:

   ```sh
   slop-gate check \
     --base "$BASE" \
     --head "$HEAD" \
     --index .slop-gate/main.json \
     --format json
   ```

   Preserve stdout as the report and capture stderr separately for diagnostics.
   Use `--format sarif` when uploading findings to a code-scanning service and
   `--format human` when presenting a concise developer-facing report.

5. For a repository-wide audit of one revision, use:

   ```sh
   slop-gate scan --ref "$HEAD" --format json
   ```

## Configuration reference

The repository policy is strict: use schema version `1`, the exact rule table
names below, and only `off`, `warn`, or `error` severities. Unknown keys and
invalid values are operational failures (status `2`).

```toml
version = 1

[rules.function_mass]
severity = "warn"
new_function_limit = 80.0
delta_limit = 20.0

[rules.near_clone]
severity = "warn"
minimum_sloc = 8
minimum_tokens = 40
similarity_threshold = 0.85
max_candidates = 64

[rules.lint_suppression]
severity = "warn"

[rules.unsafe_surface]
severity = "warn"

[rules.dependency_surface]
severity = "warn"

[[suppressions]]
rule = "near-clone"
path = "src/compat.rs"
line = 42
reason = "Protocol compatibility requires this implementation."
```

Suppression `path` values are repository-relative. Omit `line` to suppress a
rule for the whole path; otherwise the path and line must match exactly. Use
suppressions sparingly and never add one merely to make an agent check pass.
Changing policy changes the artifact fingerprint, so rebuild the index after a
policy change. See the repository [configuration documentation](../../README.md)
for the complete policy explanation.

## Result handling

Interpret the process status explicitly:

| Status | Meaning | Agent action |
| --- | --- | --- |
| `0` | No finding at `error` severity. | Continue. Review warning findings if useful. |
| `1` | At least one unsuppressed `error` finding. | Report the findings and address or obtain an explicit policy decision. |
| `2` | Configuration, Git, artifact, output, or analysis failure. | Stop the gate workflow, report the diagnostic, and fix the underlying problem. |

Warnings are findings, not failures. Do not promote warnings to errors in the
agent or reinterpret status `2` as a code-quality result.

## Operational constraints

- Run `check` only against committed revisions. Commit intended changes before
  evaluating them; do not claim that an uncommitted tree passed.
- Use `scan --working-tree` for local edits. It includes untracked non-ignored
  Rust files and respects `.gitignore` unless `--no-ignore` is supplied.
- Use the artifact matching `BASE`; a stale or policy-mismatched artifact must
  be rebuilt rather than bypassed.
- Treat malformed baseline Rust, malformed configuration, malformed manifests,
  missing revisions, and Git failures as operational failures.
- Changed head Rust syntax errors are reported as advisory `analysis-error`
  findings; surface them to the user instead of hiding them.
- The supported analysis language is Rust. `dependency-surface-growth` covers
  direct production and build dependencies, not dev dependencies, lockfiles,
  workspace declarations alone, or transitive resolution changes.
- Do not modify `.slop-gate.toml`, suppress findings, or change severity without
  the user's direction or an explicit repository policy.
