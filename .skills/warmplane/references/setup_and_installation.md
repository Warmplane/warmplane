# Installation & Setup Reference

## 1. Install Methods

- **Homebrew (macOS/Linux)**:
  `brew tap warmplane/tap && brew install warmplane`
  Update: `brew update && brew upgrade warmplane`

- **Cargo (crates.io)**:
  `cargo install warmplane`
  With ONNX semantic search: `cargo install warmplane --features semantic-search`

- **Pre-built Release Binaries**:
  Download platform executable from GitHub Releases (`x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`).
  Make executable: `chmod +x warmplane && sudo mv warmplane /usr/local/bin/`

- **Build from Source**:
  `git clone https://github.com/Warmplane/warmplane.git && cd warmplane && cargo install --path .`

## 2. Verification Commands
```bash
warmplane --version
warmplane validate-config --config mcp_servers.json
warmplane client scan
```

## 3. Minimal Config Bootstrap
Write `mcp_servers.json`:
```json
{
  "port": 9090,
  "mcpServers": {
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"]
    }
  }
}
```
Run daemon: `warmplane daemon --config mcp_servers.json`
Control Deck UI available at `http://127.0.0.1:9090`.
