# Installation and Distribution

This document covers practical install paths for Warmplane and release packaging expectations.

## 1) Build from Source

```bash
git clone https://github.com/warmplane/warmplane.git
cd warmplane
cargo build --release
./target/release/warmplane --help
```

## 2) Local Development Install

```bash
cargo install --path .
warmplane --help
```

## 3) Release Artifacts (Recommended)

For each release, publish binaries for:

- Linux x86_64 (gnu)
- Linux aarch64 (gnu)
- Linux x86_64 (musl)
- Linux aarch64 (musl)
- macOS x86_64
- macOS aarch64

Suggested naming:

- `warmplane-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `warmplane-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz`
- `warmplane-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`
- `warmplane-vX.Y.Z-aarch64-unknown-linux-musl.tar.gz`
- `warmplane-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `warmplane-vX.Y.Z-aarch64-apple-darwin.tar.gz`

Each archive should include:

- `warmplane` binary
- `LICENSE`
- `README.md`
- `docs/config.schema.json`
- `docs/openapi.yaml`

## 4) MCPB Bundles

Tagged releases also publish platform-specific `.mcpb` bundles for MCP clients that support the MCPB format. Each bundle contains the Warmplane binary, the MCPB manifest, documentation, and an empty `mcp_servers.json` configuration:

```text
warmplane-<version>-<target>.mcpb
├── manifest.json
├── server/warmplane
├── config/mcp_servers.json
├── README.md
└── INSTALL.md
```

The manifest starts Warmplane with the bundled configuration:

```text
warmplane mcp-server --config <bundle>/config/mcp_servers.json
```

The bundled configuration has no upstream servers. Copy it to a writable location and add upstream MCP servers before using the bundle for real workloads. Never place credentials directly in the bundle. Release workflows also publish `server.json` with the GitHub Release URLs and SHA-256 hashes required by the Official MCP Registry.

## 5) Homebrew (Tap Strategy)

Recommended approach:

- create tap repo: `mrorigo/homebrew-warmplane`
- formula: `warmplane.rb`
- formula installs prebuilt binaries per platform

User flow:

```bash
brew tap mrorigo/warmplane
brew install warmplane
```

## 6) Integrity and Provenance

For each release publish:

- SHA256 checksums file
- signed checksums (or release attestation)
- changelog delta

Example:

```bash
shasum -a 256 warmplane-vX.Y.Z-*.tar.gz > checksums.txt
```

## 7) Version Compatibility Policy

- Keep `/v1` HTTP API stable within major version.
- Treat alias contract changes as versioned API changes.
- Document breaking changes in release notes.

## 8) Post-Install Validation

Validate binary and config quickly:

```bash
warmplane validate-config --config mcp_servers.json
warmplane daemon --config mcp_servers.json
curl -sf http://127.0.0.1:9090/v1/capabilities >/dev/null
```
