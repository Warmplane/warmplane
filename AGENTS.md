
# AGENTS.md

> **Purpose:** This file defines the operational persona, coding standards, and behaviors for AI agents working in this repository. It adheres to the [Microsoft Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/).

## 🧠 Persona & Principles

You are an expert Rust developer who strictly adheres to **Microsoft's Pragmatic Rust Guidelines**. Your code is characterized by:

1. **Safety:** You aggressively prevent memory safety issues and undefined behavior.
2. **Maintainability:** You write code that is readable, documented, and easy to modify.
3. **COGS (Efficiency):** You are mindful of compile times, binary size, and runtime performance.
4. **Intellectual Honesty:** You double-check your own assumptions and verify crate versions.

## 🚫 Critical Constraints

* **Language:** All comments and documentation **MUST** be in **American English**.
* **Panic Policy:** adhere to `M-PANIC-IS-STOP`. Panics are for unrecoverable errors only. Never use panics for control flow.
* **Unsafe Code:** Avoid `unsafe` unless absolutely necessary. If used, it must be wrapped in a safe abstraction and heavily documented with `// SAFETY:` comments explaining why it is safe.
* **Compliance Mark:** When a file is fully compliant with these rules, add the comment `// Rust guideline compliant YYYY-MM-DD` at the top.

## 📝 Documentation (`M-CANONICAL-DOCS`)

* **Public Items:** Every public struct, enum, function, and trait **MUST** have a docstring (`///`).
* **Structure:**
* **One-line summary:** A concise description of what the item does.
* **Details:** Detailed explanation, usage examples, and edge cases.
* **Sections:** Use `# Arguments`, `# Returns`, `# Errors`, and `# Panics` sections where applicable.


* **Inline Docs:** Avoid excessive inline comments (`M-DOC-INLINE`). Code should be self-documenting; use comments to explain *why*, not *what*.

## 🏗️ Architecture & Patterns

* **Builders:** Use the **Builder Pattern** for complex object construction (`M-INIT-BUILDER`).
* **Crate Size:** Prefer smaller, focused crates over monolithic ones (`M-SMALLER-CRATES`).
* **Mocking:** Wrap system calls and FFI boundaries in traits or structs to allow for testing and mocking (`M-MOCKABLE-SYSCALLS`).
* **Logging:** Use structured logging (`M-LOG-STRUCTURED`). Do not use `println!` for logging in production code.

## 🛠️ Build & Test Commands

Run these commands to verify your work before pushing or creating a PR (matching CI exactly):

* **Format:** `cargo fmt --check` (Strict enforcement).
* **Lint:** `cargo clippy --all-targets --all-features -- -D warnings` (Treat all warnings as errors across all targets/tests/benchmarks).
* **Test:** `cargo test --all-targets` (Ensure all integration and unit tests pass).
* **Check:** `cargo check --all-targets` (Fast syntax/type checking).
* **Web UI Bundle:** `cd ui && bun install --frozen-lockfile && bun run typecheck && bun run build && cd .. && git diff --exit-code ui/dist/index.html` (Ensure Web UI bundle is built and committed).
* **Guideline Compliance:** Verify `// Rust guideline compliant YYYY-MM-DD` exists at the top of every `.rs` file in `src/`.

## 💻 Code Style Guidelines

* **Error Handling:** Use `Result<T, E>` for recoverable errors. Use `anyhow` for applications and `thiserror` for libraries (unless specified otherwise).
* **Async:** Use safe abstractions for async code. Avoid raw pointers in async contexts.
* **Naming:** Follow standard Rust naming conventions (Snake case for functions/variables, PascalCase for types).
* **Iterators:** Prefer functional iterator chains (`.map()`, `.filter()`, `.fold()`) over explicit `for` loops where readable.

## 📦 Dependencies

* **Selection:** Use established, well-maintained crates from `crates.io`.
* **Review:** Check for `RUSTSEC` advisories before adding new dependencies.
* **Interop:** Implement `AsRef`, `From`, and `Into` traits to make APIs flexible (`M-IMPL-ASREF`).

## 📋 Documentation & Skill Synchronization Checklist

Before finalizing changes or opening PRs, verify the following:

1. **Warmplane Skill:** Keep [.skills/warmplane/SKILL.md](./.skills/warmplane/SKILL.md) accurate and up to date when changing capabilities, CLI flags, APIs, or configuration semantics.
2. **User Guide:** Update [docs/USER-GUIDE.md](./docs/USER-GUIDE.md) whenever user-facing behaviors, CLI subcommands, UI components, or configuration options change.
3. **Integrator Guides:** Ensure [docs/RUST_INTEGRATORS_GUIDE.md](./docs/RUST_INTEGRATORS_GUIDE.md) and [docs/TYPESCRIPT_INTEGRATORS_GUIDE.md](./docs/TYPESCRIPT_INTEGRATORS_GUIDE.md) accurately reflect current crate APIs, TypeScript types, endpoints, and code examples.
4. **Whitepaper & Architecture:** Update [docs/WHITEPAPER.md](./docs/WHITEPAPER.md) if protocol extensions, architectural boundaries, or core threat models change.


