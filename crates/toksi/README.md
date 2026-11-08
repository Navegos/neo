# Toksi: Native Consolidation Master Engine (v3.4)

[![Crates.io](https://img.shields.io/crates/v/toksi.svg)](https://crates.io/crates/toksi)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**Toksi** is a 100% Native Safe Rust consolidation of token optimization, intelligent proxying, multi-protocol agent wire integration, code intelligence, and telemetry cockpits for AI coding assistants.

Toksi transparently sits between **17 AI coding agents** (Google Antigravity, GitHub Copilot, Claude Code, Cursor, OpenAI Codex, Grok, Cline, OpenCode, etc.) and **39+ upstream LLM providers** or local agent CLI bridges. Toksi slashes token spend by **30% to 70%** through lossless log deduplication, dynamic MCP tool schema pruning, Tree-sitter code graph indexing, runtime token filtering (RTK), and structured cross-session project memory—all with zero external package manager dependencies (no Node.js, Python, npm, or pip runtime required).

---

## 1. Master Architecture & 5-Port Network Topology

Toksi operates a dedicated, isolated multi-port network architecture to guarantee **zero socket and thread pool contention** between ultra-low-latency streaming LLM inference and rich browser dashboards:

```
[ AI Coding Assistants / IDEs / Local CLIs / Browsers ]
  │           │              │             │            │
  │ :8787     │ :8789        │ :8790       │ :9099      │ :9100
  ▼           ▼              ▼             ▼            ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        toksi daemon start                              │
│                                                                        │
│ ┌───────────────────────────┐       ┌────────────────────────────────┐ │
│ │ Port 8787: Headroom Proxy │       │ Port 9099: Enterprise Gateway  │ │
│ │ - Streaming HTTP/SSE      │       │ - /v1/chat/completions         │ │
│ │ - Antigravity & Grok      │       │ - /v1/messages, /v1beta/*      │ │
│ │ - MCP Schema Pruner       │       │ - Prometheus /metrics          │ │
│ │ - 1024-Token Cache Align  │       │ - 39+ BYOK Providers           │ │
│ └───────────────────────────┘       │ - Local Agent CLI Bridges      │ │
│                                     │ - Dynamic Load Balancer        │ │
│ ┌───────────────────────────┐       │ - 8-Mode Failover Cascade      │ │
│ │ Port 8789: Copilot VSCode │       │ - Dual-Tier Semantic Cache     │ │
│ │ - Isolated extension proxy│       └────────────────────────────────┘ │
│ └───────────────────────────┘                                          │
│                                     ┌────────────────────────────────┐ │
│ ┌───────────────────────────┐       │ Port 9100: Web Telemetry UI    │ │
│ │ Port 8790: Copilot CLI    │       │ - Background Web Cockpit       │ │
│ │ - Dedicated CLI proxy     │       │ - 17-Agent Observation Grid    │ │
│ └───────────────────────────┘       │ - Ground-Truth Token Gauges    │ │
│                                     │ - Load Balancer Controls       │ │
│                                     │ - Keyring & DB Admin           │ │
│                                     └────────────────────────────────┘ │
│                                                                        │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ Native Rust In-Process Engines                                     │ │
│ │ • RTK (100+ Command Filters)    • ProjectMem (JSONL Event Store)   │ │
│ │ • ContextMode (Sandbox & FTS5)  • ICM (Sub-10ms Zero-Token Memory) │ │
│ │ • CodeGraph (Tree-sitter/SCIP)  • ai-memory (OKF v0.2 Wiki Store)  │ │
│ │ • Rules (Karpathy, Caveman, Ponytail embedded compile-time)        │ │
│ │ • Multi-Protocol Engine (MCP, A2A, ACP, stdio, SSE, HTTP)          │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### Dedicated Ports:
* **Port 8787**: Transparent Streaming Headroom Proxy (Google Antigravity, Grok).
* **Port 8789**: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`).
* **Port 8790**: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`).
* **Port 9099**: Enterprise AI Gateway & Drop-in SDK Endpoints (`/v1/chat/completions`, `/v1/messages`, `/v1beta/models/*`, `/metrics`).
* **Port 9100**: Zero-Dependency Real-Time Web Telemetry Cockpit Dashboard (non-blocking background service).

---

## 2. Core Disciplines & Standards

1. **Universal Forward-Slash (`/`) Path Standard**:
   - Universal forward slashes `/` across Windows, Linux, and macOS (`C:/Users/name/.toksi/toksi.db`).
   - Completely prevents Git Bash, MSYS2, and POSIX shell escaping errors.
   - *Escaping Exception*: For Node.js/Python hooks requiring Windows Command Prompt `cmd.exe` execution in JSON5/TOML configs, use default escaped double backslashes: `C:\\Programs\\neo\\bin\\toksi.exe`.
2. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**:
   - All tools, compilers, shells, and agent CLIs (`bash`, `cmd`, `toksi`, `agy.exe`, `copilot.exe`, `claude`, `cursor`, `grok`) must exist in system `PATH`.
   - Never hardcode absolute executable paths everywhere; maintain 100% portability.
   - If missing, Toksi logs:
     ```text
     [ERROR] Required executable '<name>' not found in system PATH.
     [ACTION] Please add the directory containing '<name>' to your PATH environment variable and retry.
     ```
3. **Database Location (`~/.toksi/toksi.db`)**:
   - Persistent SQLite database stored in user home (`~/.toksi/toksi.db` or `C:/Users/<User>/.toksi/toksi.db`), **NOT in `localdata/toksi`**.
   - Automatic startup migration imports legacy configurations from `LOCALAPPDATA/toksi`.
4. **Workspace Root Consolidation**:
   - **Only the first (root) workspace directory contains `.toksi/`**.
   - Subfolder `.toksi` directories are automatically discovered, merged row-by-row into root `.toksi/toksi.db`, and deleted.
   - Tool directories live strictly under `.toksi/<tool>`:
     * `.toksi/codegraph/` (Tree-sitter AST symbol cache, scip index, and graph)
     * `.toksi/headroom/` (Daemon PID, logs, and proxy state)
     * `.toksi/projectmem/` (Atomic JSONL event store, summary, plan, project map)
     * `.toksi/context/` (Context-mode FTS5 database and sandbox scratch)
     * `.toksi/icm/` (Instant Context Memory SQLite FTS5 database)
     * `.toksi/ai-memory/` (OKF v0.2 wiki store and handoff states)
     * `.toksi/rtk/` (RTK command filter stats and cache)
   - Workspace agent directories (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.gemini`, `.agents`) strictly live in the workspace root.
5. **Tool Hook & MCP Naming Conventions**:
   - Standard Hook Syntax: `<tool>-hook <agent>` (e.g., `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`).
   - Clean MCP Names: `projectmem`, `context`, `codegraph`.
   - MCP Settings to Skills Mapping: Settings saved under `.gemini/mcp/`, `.copilot/mcp/`, etc. are automatically wired under `skills/<tool>`.
   - Agent Taxonomy: Distinguishes CLI from IDE extensions: `antigravity-cli;agy` vs `antigravity-ide;antigravity`, `copilot-cli` vs `copilot-vscode`, `claude-cli` vs `claude-desktop`, `cursor-cli` vs `cursor-ide`, `grok-cli` vs `grok-ide`.
6. **Multi-Protocol Standardization Across All Tools**:
   - All tools and agent connectors support: **MCP**, **A2A**, **ACP**, **stdio**, **SSE**, and **HTTP**.
7. **Dynamic `.gitignore` & `.ignore` Cache Purging**:
   - Codebase indexing strictly obeys `.gitignore` and `.ignore` files.
   - When ignore rules change, Toksi dynamically purges matching cached symbols and vector embeddings from SQLite databases.
8. **Network Reconnect Auto-Healing Watchdog**:
   - Automatically detects interface disconnects, VPN toggles, and socket timeouts.
   - Flushes stale connection pools without requiring agent or daemon restart, eliminating dormant lockout states.

---

## 3. Supported BYOK Providers & Smart Local Agent CLI Bridges

### 39+ Supported BYOK Upstream Providers:
1. **Frontier & Cloud**: Anthropic, Azure OpenAI, AWS Bedrock, AWS Bedrock Mantle, AWS SageMaker, Databricks, Google Gemini, OpenAI, Google Vertex AI.
2. **Accelerators & Fast Inference**: Cerebras, DeepSeek, Fireworks AI, Groq, Nebius AI, Parasail, Together AI, Wafer.
3. **Aggregators & Proxies**: Bifrost, Cloudflare AI Gateway, Hugging Face, OpenRouter, Replicate.
4. **Specialized & Enterprise**: BytePlus, Cohere, MiniMax, Mistral AI, Perplexity AI, Sarvam AI, TypeSafe, xAI Grok.
5. **Multimodal**: ElevenLabs, Runware, Runway ML.
6. **Local Engines**: llama.cpp, Ollama, SGLang, vLLM.
7. **Ecosystem**: GitHub Copilot, OpenCode.

### First-Class Local Agent CLI Providers:
Toksi securely integrates the 5 primary local agent CLIs as first-class peer BYOK providers:
1. `AntigravityCli` (`agy.exe`)
2. `CopilotCli` (`copilot.exe`)
3. `ClaudeCli` (`claude`)
4. `CursorBridge` (`cursor`)
5. `GrokCli` (`grok.exe`)

*Smart Background Launch*: Automatically queries OS Keyring credentials and local session tokens (`~/.gemini/`, `~/.claude.json`, `gh auth token`). When authenticated, Toksi smart-launches the background bridge and validates chat reachability (`/livez` or ConnectRPC).

### Strict Paid Quota Enforcement (Zero Free Accounts for BYOK):
* **Commercial Paid Quota Mandate**: All BYOK providers must have an active paid commercial tier or subscription quota. Free accounts and trial keys are strictly disqualified from active gateway rotation.
* **Graceful Inform & Zero-Fail Policy**:
  - Detecting a free-tier account **never crashes, panics, or causes a breaking error**.
  - Logs: `[INFO] Provider '<Name>' is on a Free Tier (unbilled/rate-limited). Paid quota required for BYOK gateway pool. Bypassing from active pool without error.`
  - Gracefully falls back to the next paid provider in the cascade or direct local agent execution.
* **Activation Guard**:
  - Gateway engages when `active_paid_providers >= 1`.
  - If zero providers are configured, requests pass through gracefully without breaking host applications.

---

## 4. Ground-Truth Session Token Accounting

* **Zero Synthetic or Virtual Tokens**: When no agent or gateway is active, the Token Savings counter reads **exactly 0**.
* **Payload Extraction**: Prompt, completion, and cached tokens are extracted directly from live SSE completion chunks (`usage.prompt_tokens`, `usage.completion_tokens`, `cache_read_input_tokens`, Google `usageMetadata`).
* **Real Savings**: Measured as exact byte delta from Headroom compression and token weight of pruned MCP schemas.
* **Upstream Quota Telemetry**: Extracted from upstream HTTP response headers (`x-ratelimit-remaining-tokens`, `anthropic-ratelimit-requests-remaining`) and free agent AI API channels at zero cost.

---

## 5. Supported Agent Directory Protocols

Toksi reads, writes, and synchronizes the directory standards for all 17 AI coding assistants:

| Agent Ecosystem | User Global Path | Workspace Root Path | Key Configuration Files |
|---|---|---|---|
| **OpenAgents Standard** | `~/.agents/` | `<root>/.agents/` | `mcp.json`, `models.json`, `agents.md`, `skills/`, `tasks/`, `memories/` |
| **Google Antigravity** | `~/.gemini/` | `<root>/.gemini/` | `hooks.json`, `mcp_config.json`, `settings.json`, `rules/`, `skills/`, `GEMINI.md` |
| **GitHub Copilot** | `~/.copilot/`, `~/.github/` | `<root>/.github/` | `copilot-instructions.md`, `mcp-config.json`, `providers.json`, `skills/` |
| **Claude Code** | `~/.claude/` | `<root>/.claude/` | `CLAUDE.md`, `settings.json`, `.mcp.json`, `rules/`, `skills/`, `agents/` |
| **OpenAI Codex** | `~/.codex/` | `<root>/.codex/` | `config.toml`, `AGENTS.md`, `hooks.json`, `agents/`, `rules/`, `auth.json` |
| **Cursor** | `~/.cursor/` | `<root>/.cursor/` | `rules/*.mdc`, `skills/`, `agents/`, `hooks.json`, `mcp.json` |
| **xAI Grok** | `~/.grok/` | `<root>/.grok/` | `config.toml`, `mcp_credentials.json`, `skills/`, `hooks/*.json`, `agents/` |
| **Cline / OpenCode / Others** | `~/.<agent>/` | `<root>/.<agent>/` | `AGENTS.md`, `rules.md`, `mcp.json` |

*User-Level vs Workspace Install Policy*:
- Auto-installed to user directory (`~/.<agent>/`) if selected.
- Workspace repository installation is **NEVER automatic**; it is only installed into `<root>/` if the user explicitly triggers it.

---

## 6. Quick Start & CLI Reference

### Universal Daemon Launch
Launch all 5 dedicated endpoints simultaneously in the background:
```bash
toksi daemon start
```
Check status or terminate cleanly:
```bash
toksi daemon status
toksi daemon stop
```

### Interactive Terminal User Interface (TUI)
Launch the mouse-capable interactive terminal control center:
```bash
toksi tui
# or
toksi top
```
- Click tabs to switch views instantly (Monitor, Agents, Memory, Graph, Headroom, Logs).
- Click rows to inspect agents or providers.
- Click action buttons (Sync, Fix, Health, Start, Stop).
- Mouse wheel smoothly scrolls inspection logs.
- Asynchronous non-blocking event loop guarantees **zero keyboard freezes or input deadlocks**.

### Background Web Telemetry Cockpit
Launch the dedicated web dashboard on port 9100:
```bash
# Launch in background and open default browser
toksi dashboard --open

# Launch in foreground
toksi dashboard --foreground --port 9100
```

### Agent Management
Inspect, health-check, install, or fix agent configurations:
```bash
# Audit all 17 agent configurations and verify PATH executables
toksi doctor

# Auto-heal broken routes, sync rules, and consolidate workspace .toksi
toksi doctor --fix

# Manage agent harnesses
toksi agent install antigravity
toksi agent health copilot
toksi agent fix claude
```

### Credential & Keyring Management
Manage BYOK provider keys securely via OS Keyring:
```bash
# View configured credentials and health status
toksi auth status

# Set provider API key
toksi auth set anthropic --key sk-ant-...

# Sync keys from local agent configuration files
toksi auth sync
```

### Database Operations
Inspect, backup, or migrate the SQLite configuration store:
```bash
# Check SQLite DB status
toksi db status

# Export database to JSON
toksi db export --file backup.json

# Restore database from JSON
toksi db import --file backup.json
```

### In-Process Tools via CLI
```bash
# Query Instant Context Memory
toksi mem recall "JWT auth middleware"

# Explore AST Code Graph and blast radius
toksi graph explore "AuthService.validateToken"

# Run command with RTK token filtering
toksi rtk -- cargo test
```

---

## 7. Build & Installation

### Requirements:
- Safe Rust 1.85+ (Edition 2024)
- Cargo

### Compilation:
```bash
git clone https://github.com/Navegos/neo.git
cd neo
cargo build --release -p toksi
```
The resulting binary is located at `target/release/toksi` (`target/release/toksi.exe` on Windows). Ensure this directory is added to your system `PATH`.

---

## 8. License

Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
