# toksi — System Architecture & Implementation Plan

High-performance headroom proxy, token optimizer, and agent wire integration for AI coding agents.

## 1. System Overview

`toksi` is an intelligent headroom proxy, token compression engine, and agent routing platform. It transparently sits between local AI coding agents (such as Antigravity, Claude Code, GitHub Copilot, Cursor, Codex, and others) and model backends or MCP servers.

`toksi` slashes token consumption by 30-70% through intelligent prompt compression, tool output condensing (RTK), code graph indexing, and structured cross-session project memory.

## 2. Architecture & Core Subsystems

```
crates/toksi/src/
├── agents/         # Agent configurations, detection, hooks, and proxy protocols
├── commands/       # CLI command implementations (init, doctor, proxy, runmcp, mcp-proxy, info, disable)
├── core/           # Common data structures, registries, and configuration models
├── headroom/       # High-performance local proxy engine, router, watchdog, and BYOK gateway
├── tools/          # Token optimization filters (RTK, Caveman, Ponytail, CodeGraph, ContextMode, ProjectMem)
├── util/           # Cross-platform utilities, process lifecycle, routing state, JSONC/TOML parsers
├── lib.rs          # Reusable library API
└── main.rs         # Standalone toksi CLI executable
```

### 1. Headroom Proxy & Gateway (`src/headroom`)
- **Local Transparent Proxy**: Intercepts HTTP/SSE streaming requests between agent clients and upstream model providers (Anthropic, OpenAI, Google Gemini, OpenRouter, Groq).
- **BYOK (Bring Your Own Key) Gateway**: Securely routes requests using user-configured API keys without leaking secrets into project files or transcripts.
- **Routing Coordinator & Store**: Manages dynamic model routing, fallback cascades, load shedding, and quota tracking.
- **Watchdog & Process Identity**: Monitors agent daemon processes, automatic resurrection on failure, and health watchdog.

### 2. Supported Agent Profiles (`src/agents`)
Native configuration, automatic hook injection, and proxy protocol handling for 17 AI coding agents:
1. **Antigravity** (Google AGY CLI & IDE)
2. **Claude Code** (Anthropic CLI)
3. **Cline** (VS Code / JetBrains extension)
4. **Codex** (OpenAI Codex CLI)
5. **GitHub Copilot** (VS Code / Visual Studio)
6. **Cursor** (AI Code Editor)
7. **Droid** (Factory Droid agent)
8. **Grok** (xAI Grok agent)
9. **Kilo** (Kilo code assistant)
10. **OMP** (Open Multi-Agent Platform)
11. **OpenCode** (Open-source agent runner v1)
12. **OpenCode2** (Open-source agent runner v2)
13. **Pi** (Pi agent rust harness)
14. **Amp** (Amp agent)
15. **Kimi Code** (Kimi Code agent)
16. **ZCode** (ZCode agent)
17. **DeepSeek Harness** (`dsh`)

### 3. Token Compression & Headroom Tools (`src/tools`)
- **RTK (Runtime Token Kernel)**: Condenses CLI command output, strips redundant build noise, pagination traces, and whitespace while preserving essential error lines and exit codes.
- **Caveman Compressor**: Translates system prompts and agent instructions into terse ASD-STE100 Simplified Technical English, reducing token footprint without losing precision.
- **Ponytail Discipline**: Injects strict build guidelines enforcing YAGNI, surgical edits, and minimal runnable checks.
- **CodeGraph**: Out-of-context repository indexing, symbol navigation, and call-graph blast radius computation via MCP.
- **ContextMode**: Sandbox code execution (`ctx_execute`, `ctx_batch_execute`, `ctx_search`) to keep large analysis out of the LLM context window.
- **ProjectMem**: Append-only structured event log tracking issues, attempts, fixes, and architectural decisions across sessions in `.projectmem/`.

## 3. CLI Subcommands

- `toksi init`: Auto-detects installed coding agents and injects proxy hooks and instructions.
- `toksi doctor`: Probes agent configurations, proxy connectivity, token savings metrics, and PATH health.
- `toksi proxy`: Starts the background headroom proxy server and BYOK routing engine.
- `toksi runmcp`: Wraps MCP servers with streaming headroom optimization and token metrics.
- `toksi mcp-proxy`: Bridges standard MCP JSON-RPC transports with headroom caching and filtering.
- `toksi info`: Displays active agent hooks, proxy ports, and token compression statistics.
- `toksi disable`: Reverts agent configurations to unproxied native state.
- `toksi update`: Self-updates `toksi` binary from GitHub releases or builds from source.

## 4. Crates.io Publishing & Release Lifecycle

1. **Autonomous Entity on Crates.io**:
   - Package name: `toksi`.
   - Independent semver releases (`v0.1.0`, ...).
   - Functions both as a command-line tool (`toksi`) and a library crate.
2. **Release Pipeline**:
   - Cross-compiled across Windows (`x86_64`, `aarch64`), Linux (`x86_64`, `aarch64`, `riscv64`), and macOS (`aarch64`).
   - Binary packages compressed with Zstandard (`toksi-{version}-{target}.tar.zst`).
   - Packaged and published independently to crates.io via `cargo publish -p toksi`.

