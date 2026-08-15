# Warmplane CLI Configuration Management Plan

> **Goal:** Provide a seamless, ergonomic CLI management interface for Warmplane configuration (`mcp_servers.json`), eliminating manual JSON editing, ensuring schema safety, supporting interactive guided flows, and aligning with MCP ecosystem conventions (`claude mcp`, `cursor`, `opencode`).
> **Status:** Proposed Design / Implementation Plan  
> **Target Branch:** `features/cli-configuration`

---

## 1. Motivation & Ecosystem Alignment

### Current Pain Points
- Users must hand-craft or edit JSON (`mcp_servers.json`) with deeply nested structures:
  - Stdio process configuration (command, args array, env key-values)
  - HTTP / SSE / Streamable HTTP endpoints and headers
  - Complex authentication blocks (`Bearer`, `Basic`, `OAuth2` with client ID, auth server URL, scopes, metadata URL)
  - Capability, resource, and prompt aliases
  - Security policies (`allow`, `deny`, `redactKeys`)
- Manual editing is error-prone: unescaped characters, missing commas, invalid schema types, and typos in server names.
- Round-tripping or synchronizing configurations from other tools requires manual copy-pasting.

### Ecosystem Alignment
The CLI should feel familiar to developers using standard tooling across the MCP and developer tools landscape:
- **`claude mcp add / remove / list`** (Claude CLI pattern)
- **`gh config / gh auth`** (GitHub CLI intuitive interactive & flag-based experience)
- **`docker / kubectl`** (hierarchical noun-verb structure)

Warmplane will support both:
1. **Scriptable / Headless execution:** Fast, full flag support for CI/CD, automation scripts, and LLM agent orchestration.
2. **Interactive TUI / Prompting:** Rich guided prompts (using crates like `inquire` or `dialoguer`) when flags are omitted or `--interactive` is specified.

---

## 2. Command Surface Architecture

The CLI interface introduces the `warmplane server` and `warmplane config` command namespaces.

```
warmplane
├── server
│   ├── add <NAME> [FLAGS]           # Add a new stdio or HTTP upstream server (interactive if args omitted)
│   ├── remove <NAME>                # Remove an upstream server (alias: rm, delete)
│   ├── list                         # List configured servers (tabular or JSON)
│   ├── get <NAME>                   # Show detailed server configuration
│   ├── edit <NAME>                  # Interactive edit or selective flag updates
│   ├── test <NAME>                  # Validate upstream reachability and handshake
│   └── enable / disable <NAME>      # Toggle servers without deleting definition
│
├── config
│   ├── init                         # Create a default or guided mcp_servers.json
│   ├── show                         # Print current merged configuration
│   ├── get <KEY>                    # Get top-level setting (e.g. port, toolTimeoutMs)
│   ├── set <KEY> <VALUE>            # Set top-level setting
│   ├── alias
│   │   ├── set <TYPE> <ALIAS> <TARGET>   # e.g., set tool git-commit github.create_commit
│   │   ├── remove <TYPE> <ALIAS>
│   │   └── list
│   ├── policy
│   │   ├── allow <PATTERN...>       # Add allowed capability pattern
│   │   ├── deny <PATTERN...>        # Add denied capability pattern
│   │   ├── redact <KEY...>          # Add sensitive payload redaction key
│   │   └── show
│   ├── import                       # Import from Claude Desktop, Cursor, or Zed configs
│   └── export                       # Export clean config to file or stdout
```

---

## 3. Detailed Command Specs & Usage Examples

### 3.1 `warmplane server add`

Supports both command-line flags and interactive prompts.

#### A. Stdio Servers (Command-based)
```bash
# Non-interactive CLI
warmplane server add github \
  --command npx \
  --args "-y,@modelcontextprotocol/server-github" \
  --env "GITHUB_PERSONAL_ACCESS_TOKEN=ghp_xxx"

# Arguments with custom separation or repeated flags
warmplane server add filesystem \
  --command "npx" \
  --arg "-y" \
  --arg "@modelcontextprotocol/server-filesystem" \
  --arg "/Users/origo/projects"
```

#### B. HTTP / SSE / Streamable HTTP Remote Servers
```bash
# Streamable HTTP / SSE with Bearer auth token env
warmplane server add context7 \
  --url "https://mcp.context7.ai/sse" \
  --bearer-env "CONTEXT7_API_KEY"

# Direct bearer token (warns on raw secrets in config vs env)
warmplane server add remote-tools \
  --url "https://tools.example.com/mcp" \
  --bearer-token "secret_token_123"

# Basic auth
warmplane server add internal-db \
  --url "https://db.internal.net/mcp" \
  --username "admin" \
  --password-env "DB_MCP_PASS"

# OAuth2 PKCE
warmplane server add linear \
  --url "https://mcp.linear.app/sse" \
  --oauth2 \
  --client-id "lin_client_123" \
  --auth-server "https://linear.app/oauth/authorize" \
  --scope "read,write,issues:create"
```

#### C. Interactive Flow (`warmplane server add` with no flags or `-i / --interactive`)
```
? Server identifier (name): github
? Server transport type:
  > Stdio (Local executable / sub-process)
    HTTP / SSE (Remote endpoint)
? Executable command: npx
? Command arguments (comma or space separated): -y @modelcontextprotocol/server-github
? Add environment variables? Yes
  ? Key: GITHUB_PERSONAL_ACCESS_TOKEN
  ? Value (or $ENV_VAR): $GITHUB_TOKEN
  ? Add another variable? No
? Validate and test connection now? [Y/n] Y
✔ Validating connection to "github"... Connected! Found 24 tools.
✔ Added server "github" to mcp_servers.json
```

---

### 3.2 `warmplane server list` & `warmplane server test`

```bash
# Formatted table
warmplane server list

# Output:
# NAME         TYPE    TARGET                                AUTH         STATUS
# github       stdio   npx -y @modelcontextprotocol/ser...   -            ENABLED
# context7     http    https://mcp.context7.ai/sse           Bearer (env) ENABLED
# internal-db  http    https://db.internal.net/mcp           Basic (env)  DISABLED

# JSON format for scripting
warmplane server list --json

# Test live reachability
warmplane server test github
# ✔ Spawned process `npx`
# ✔ Handshake successful (Protocol 2026-07-28)
# ✔ Capabilities discovered: 24 tools, 2 resources, 0 prompts
```

---

### 3.3 `warmplane config import` (Ecosystem Migration)

Seamlessly scan and import configurations from known locations:
- **Claude Desktop:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Cursor:** `~/.cursor/mcp.json` or `.cursor/mcp.json`
- **Zed:** `~/.config/zed/settings.json`

```bash
# Auto-detect sources
warmplane config import

# ? Detected MCP configurations:
#   [x] Claude Desktop (3 servers: github, fetch, memory)
#   [ ] Cursor Project (1 server: local-tools)
# ? Overwrite existing servers with the same name? No (skip)
# ✔ Imported 3 servers into mcp_servers.json
```

---

### 3.4 Aliases and Security Policy Management

```bash
# Fast tool aliasing
warmplane config alias set tool git-commit github.create_commit
warmplane config alias set resource company-handbook "handbook://internal/v1"
warmplane config alias set prompt code-review pr-reviewer.review_code

# Policy guardrails
warmplane config policy allow "github.*" "fetch.*"
warmplane config policy deny "filesystem.write_file" "filesystem.delete_file"
warmplane config policy redact "api_key" "secret" "password" "token"
```

---

## 4. Technical Design & Rust Architecture

### 4.1 Required Crates & Dependencies
- **`inquire`** (`0.7`): Fast, zero-config terminal prompts with autocompletion, selects, multi-selects, and custom validators.
- **`serde` / `serde_json`**: Already present. Add `Serialize` derives across `McpConfig`, `ServerConfig`, `AuthConfig`, and `PolicyConfig` to support two-way JSON writes.
- **`colored` / `console`** or `clap` styling: Clean terminal colors, checkmarks (`✔`), warnings (`⚠`), and error indicators.
- **`tabled`** (optional) or lightweight formatted columns for `server list`.

### 4.2 Module Organization in `src/`

```
src/
├── config.rs              # Updated: Serialize + Deserialization, atomic file saves, mutation methods
├── config/
│   ├── mod.rs             # Config manager engine (read, mutate, transactional write)
│   ├── import.rs          # Importers for Claude Desktop, Cursor, Zed
│   └── interactive.rs     # Interactive prompts and form wizards (inquire)
├── models.rs              # Updated Clap enum with ServerCommands and ConfigCommands
└── server_manager.rs      # Upstream server tester, health verification
```

### 4.3 Atomic & Safe File Persistence
To prevent corrupting `mcp_servers.json` on power failure or concurrent writes:
1. Load and parse existing JSON.
2. Apply memory modifications.
3. Validate invariants.
4. Write serialized JSON to a temporary file (`mcp_servers.json.tmp.<pid>`).
5. Atomically rename/replace the target configuration file using `std::fs::rename`.
6. Preserve formatting with pretty-printed 2-space indentation (`serde_json::to_string_pretty`).

---

## 5. Phased Implementation Roadmap

### Phase 1: Serialization & Core Config Mutation (`src/config.rs`)
- [ ] Add `#[derive(Serialize)]` to all config structs in `src/config.rs`.
- [ ] Implement atomic write helper `save_config(path: &str, config: &McpConfig) -> Result<()>`.
- [ ] Implement unit tests for round-tripping configs without losing data or fields.

### Phase 2: `warmplane server` CLI Commands (Headless)
- [ ] Extend `models.rs` and `main.rs` with `server add`, `server remove`, `server list`, `server get`.
- [ ] Implement command argument mapping for stdio options (`--command`, `--args`, `--env`).
- [ ] Implement command argument mapping for HTTP/SSE options (`--url`, `--headers`, auth flags).
- [ ] Add `--json` flag to `server list` and `server get`.

### Phase 3: Interactive Prompt Wizards (`inquire`)
- [ ] Implement interactive flow when `warmplane server add` is run without required flags or with `-i`.
- [ ] Interactive wizard for stdio command testing.
- [ ] Interactive wizard for OAuth2 PKCE setup and Bearer token env configuration.
- [ ] Interactive confirmation prompt before destructive actions (e.g. `warmplane server remove`).

### Phase 4: Verification & Ecosystem Importers
- [ ] Implement `warmplane server test <NAME>`: Spawns the stdio process or sends HTTP discovery to test upstream MCP protocol compatibility.
- [ ] Implement `warmplane config import`: Discover and import configs from Claude Desktop and Cursor.
- [ ] Implement `warmplane config alias` and `warmplane config policy` subcommands.

---

## 6. Verification & Quality Gates
- **Unit Tests:** Serialization/deserialization edge cases, invalid JSON error handling, atomic write verification.
- **Integration Tests:** CLI command execution (`cargo run -- server add ...`), asserting exit codes and config mutations.
- **Guidelines Compliance:** Follow `AGENTS.md` (Microsoft Pragmatic Rust Guidelines, `M-CANONICAL-DOCS`, no unhandled panics, American English).
