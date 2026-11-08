# toksi — System Architecture & Implementation Plan (v3.4)

High-performance headroom proxy, token optimizer, native AI gateway, multi-protocol agent wire integration, and telemetry cockpit for AI coding agents.

## 1. System Overview

`toksi` is an intelligent headroom proxy, token compression engine, native AI gateway, and multi-protocol agent routing platform. It transparently sits between local AI coding assistants (such as Antigravity, Claude Code, GitHub Copilot, Cursor, Codex, OpenCode, and others) and 39+ upstream LLM providers or in-process multi-protocol tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`).

`toksi` slashes token consumption by 30-70% through intelligent prompt compression, tool output condensing (RTK), code graph indexing, and structured cross-session project memory.

## 2. Dedicated 5-Port Network Topology

To eliminate TCP backpressure and socket/thread contention between ultra-low-latency LLM streaming and rich browser interfaces:
- **Port 8787**: Transparent Streaming Headroom Proxy (Google Antigravity, Grok)
- **Port 8789**: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
- **Port 8790**: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
- **Port 9099**: Enterprise AI Gateway & Drop-in SDKs (`/v1/chat/completions`, `/v1/messages`, `/metrics`)
- **Port 9100**: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)

## 3. Architecture & Core Subsystems

```
crates/toksi/src/
├── agents/         # Native configuration, detection, hooks, and Open Agents (.agents/) wiring
├── aimemory/       # Open Knowledge Format (OKF v0.2) wiki store, multi-agent handoffs, mem0
├── codegraph/      # Tree-sitter AST dependency, petgraph blast-radius, SCIP, ast-grep, repomix
├── commands/       # CLI command implementations (daemon, auth, db, agent, tui, dashboard, doctor, init, etc.)
├── contextmode/    # Polyglot sandbox runner, SQLite FTS5 search with .gitignore purge, and HTML fetcher (tool: context)
├── core/           # Common data structures, registries, and configuration models
├── dashboard/      # Embedded zero-dependency Web Telemetry Cockpit on port 9100
├── db/             # Persistent SQLite WAL configuration database stored in ~/.toksi/toksi.db
├── env_injector/   # Non-destructive OS environment injection (HKCU\Environment on Windows)
├── gateway/        # 39+ BYOK providers, smart CLI bridges, load balancer, dual-tier semantic cache, auto-reconnect
├── headroom/       # High-performance streaming proxy engine, MCP pruner, cache aligner, copilot supervisor
├── icm/            # Instant Context Memory SQLite FTS5 BM25 knowledge engine (~/.toksi/icm/memory.db)
├── keyring/        # Cross-platform secure credential manager (advapi32.dll, Keychain, Secret Service)
├── projectmem/     # Pure Rust append-only JSONL event store, summary, precheck, and multi-protocol server
├── rtk/            # 100+ command output filters and zero-allocation ANSI stripper
├── rules/          # In-tree coding guidelines (Karpathy, Caveman, Ponytail) & universal sync
├── telemetry/      # Ground-truth session token accounting, TokenTracker, latency tracker, Prometheus exporter
├── tools/          # Multi-protocol tool adapters (MCP, A2A, ACP, stdio, SSE, HTTP)
├── tui/            # Full-featured interactive terminal user interface with ratatui mouse capture
├── util/           # Forward-slash / path normalization, PATH validation mandate, exec, JSONC/TOML parsers
├── lib.rs          # Reusable library API
└── main.rs         # Standalone toksi CLI executable with comprehensive usage help
```

## 4. System Conventions & Disciplines

1. **Forward-Slash Path Standard**: Universal forward slashes (`/`) across Windows and Unix for Git Bash / MSYS2 compatibility. Exception: For JavaScript/TypeScript/Python/Node.js hook/MCP JSON5/TOML configurations requiring cmd execution, preserve the default escaped double backslashes (`C:\\Programs\\neo\\bin\\toksi.exe`).
2. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**:
   - For all needed binaries/commands (`bash`, `cmd`, `toksi`, `agy.exe`, `copilot.exe`, `claude`, `cursor`, `grok`, tools, etc.), Toksi checks `PATH`.
   - Never encode hardcoded full paths everywhere.
   - If not found in `PATH`, log: `[ERROR] Required executable '<cmd>' not found in system PATH. Please add '<cmd>' to your PATH environment variable and retry.`
3. **User Home Database Location**: Default configuration database stored in `~/.toksi/toksi.db` (`users/.toksi`), NOT in `localdata/toksi`, with automatic startup migration from legacy `LOCALAPPDATA/toksi`.
4. **Workspace Root Consolidation**:
   - Only ONE `.toksi` folder exists in the first (root) workspace directory.
   - Any subfolder `.toksi` directories are automatically discovered, merged into `<root>/.toksi/` (with safe SQLite DB merging), and deleted.
   - Tool directories live strictly inside `.toksi/<tool>`:
     * `.toksi/codegraph`
     * `.toksi/headroom`
     * `.toksi/projectmem`
     * `.toksi/context`
     * `.toksi/icm`
     * `.toksi/ai-memory`
     * `.toksi/rtk`
   - Workspace agent dirs (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.agents`) only exist in the primary repo root.
5. **Tool Hook & MCP Naming Normalization**:
   - MCP tool names: `projectmem`, `context`, `codegraph`.
   - Hook command syntax standard: `<tool>-hook <agent>` (e.g. `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`).
   - Agent identity taxonomy: `antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude-cli;claude-desktop`, etc.
   - MCP instruction directories: `context-mode` -> `context`.
   - MCP settings to skills mapping: `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp` mapped to `skills/<tool>`.
6. **Smart Background Launch for Local Agent CLIs**:
   - `AntigravityCli` (`agy.exe`), `CopilotCli` (`copilot.exe`), `ClaudeCli` (`claude`), `CursorBridge` (`cursor`), `GrokCli` (`grok.exe`) register as first-class BYOK providers.
   - Auto-checks OS Keyring / session tokens and smart-launches in detached background mode when authenticated and chat endpoint reachable.
7. **Strict Paid Quota Enforcement**:
   - Mandate: all BYOK providers must have an active paid commercial tier or subscription quota. Free accounts are disqualified from active gateway rotation.
   - Graceful Inform & Zero-Fail Policy: emits non-blocking informative status (`[INFO] Provider 'X' is on Free tier. Paid account required for BYOK rotation; bypassing gracefully`), zero crashes or panics, and smoothly falls back to paid provider or direct agent execution.
   - Gateway engages when `active_paid_providers >= 1`. If zero, graceful passthrough.
8. **Ground-Truth Session Token Accounting**:
   - Parses real usage tokens directly from live SSE response streams and upstream response headers. Zero synthetic/virtual guesses.
9. **Multi-Protocol Architecture**:
   - All tools and agent connectors support MCP, A2A, ACP, stdio, SSE, HTTP.
10. **.gitignore & .ignore Optimization with Dynamic Purging**:
    - Respects ignore rules during AST and document indexing, and automatically purges existing index entries when ignore rules change.
11. **User Auto-Install vs Selective Workspace Installation**:
    - Auto-installs agent hooks, MCPs, rules, skills, configs to user directory (`~/.<agent>/`) if selected. Workspace repository install is never automatic; requires explicit user action.

## 5. Supported Agent Profiles & Open Directory Standards (`src/agents`)
Native configuration, automatic hook injection, and proxy protocol handling for 17 AI coding agents:
1. **OpenAgents Standard** (`.agents/`)
2. **Antigravity** (Google AGY CLI & IDE, `.gemini/`)
3. **GitHub Copilot** (VS Code / Visual Studio & CLI, `.copilot/`, `.github/`)
4. **Claude Code** (Anthropic CLI, `.claude/`)
5. **OpenAI Codex** (`.codex/`)
6. **Cursor** (`.cursor/`)
7. **Grok** (xAI Grok agent, `.grok/`)
8. **Cline** (VS Code / JetBrains extension)
9. **OpenCode** (Open-source agent runner v1)
10. **OpenCode2** (Open-source agent runner v2)
11. **Droid** (Factory Droid agent)
12. **Pi** (Pi agent rust harness)
13. **OMP** (Open Multi-Agent Platform)
14. **Kilo** (Kilo code assistant)
15. **Amp** (Amp agent)
16. **Kimi Code** (Kimi Code agent)
17. **ZCode / DSH** (ZCode and DeepSeek Harness)

## 6. Execution Roadmap (Phases 15.1 – 16.3)
- **Phase 15.1**: Workspace Topology, Path Normalization, Root `.toksi` Consolidation, Tool Layout, Hook/MCP Naming & PATH Validation Mandate.
- **Phase 15.2**: Unified 5-Port Daemon (:8787, :8789, :8790, :9099, :9100), Copilot Dual-Port Wiring & Latency Isolation.
- **Phase 15.3**: Local Agent CLIs as First-Class BYOK, Smart Launch, Paid Quota Enforcement, Graceful Fallback & Bifrost Integration.
- **Phase 15.4**: Multi-Protocol Architecture (MCP, A2A, ACP, stdio, SSE, HTTP) & Dynamic .gitignore/.ignore Index Purging.
- **Phase 15.5**: Ground-Truth Token Accounting, Zero-Math Metrics, Network Reconnect Auto-Healing Watchdog & Rules Sync.
- **Phase 15.6**: Advanced Vault Intelligence Engines (`TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`).
- **Phase 16.1**: High-Performance Mouse-Capable Interactive TUI (`EnableMouseCapture`, fix input locks & tab freezes).
- **Phase 16.2**: Enterprise Background Web Telemetry Cockpit Dashboard (:9100, live observation grid, quotas, load balancer).
- **Phase 16.3**: Open Agents Directory Protocols, Selective User vs Workspace Install & Complete Technical README Verification.
