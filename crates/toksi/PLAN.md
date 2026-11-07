# toksi — System Architecture & Implementation Plan

High-performance headroom proxy, token optimizer, native AI gateway, and agent wire integration for AI coding agents.

## 1. System Overview

`toksi` is an intelligent headroom proxy, token compression engine, native AI gateway, and agent routing platform. It transparently sits between local AI coding agents (such as Antigravity, Claude Code, GitHub Copilot, Cursor, Codex, OpenCode, and others) and 39+ upstream LLM providers or in-process MCP servers.

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
├── agents/         # Native configuration, detection, hooks, and MCP wiring for 17 AI coding agents
├── aimemory/       # Open Knowledge Format (OKF v0.2) wiki store and multi-agent handoffs
├── codegraph/      # Tree-sitter AST dependency, call-path, and blast-radius engine
├── commands/       # CLI command implementations (daemon, auth, db, tui, dashboard, doctor, init, etc.)
├── contextmode/    # Polyglot sandbox runner, SQLite FTS5 search, and HTML fetcher (tool: context)
├── core/           # Common data structures, registries, and configuration models
├── dashboard/      # Embedded zero-dependency Web Telemetry Cockpit on port 9100
├── db/             # Persistent SQLite WAL configuration database stored in ~/.toksi/toksi.db
├── env_injector/   # Non-destructive OS environment injection (HKCU\Environment on Windows)
├── gateway/        # 39+ BYOK providers, smart CLI bridges, load balancer, dual-tier semantic cache
├── headroom/       # High-performance streaming proxy engine, MCP pruner, cache aligner, copilot supervisor
├── icm/            # Instant Context Memory SQLite FTS5 BM25 knowledge engine (~/.toksi/icm/memory.db)
├── keyring/        # Cross-platform secure credential manager (advapi32.dll, Keychain, Secret Service)
├── projectmem/     # Pure Rust append-only JSONL event store, summary, precheck, and MCP server
├── rtk/            # 100+ command output filters and zero-allocation ANSI stripper
├── rules/          # In-tree coding guidelines (Karpathy, Caveman, Ponytail) & universal sync
├── telemetry/      # Ground-truth session token accounting, latency tracker, Prometheus exporter
├── tools/          # Legacy tool adapters and facades
├── tui/            # Full-featured interactive terminal user interface with ratatui mouse capture
├── util/           # Forward-slash / path normalization, exec, JSONC/TOML parsers
├── lib.rs          # Reusable library API
└── main.rs         # Standalone toksi CLI executable with comprehensive usage help
```

## 4. System Conventions & Disciplines

1. **Forward-Slash Path Standard**: Universal forward slashes (`/`) across Windows and Unix for Git Bash / MSYS2 compatibility. Exception: For JavaScript/TypeScript/Python/Node.js hook/MCP JSON5/TOML configurations requiring cmd execution, preserve the default escaped double backslashes (`C:\\Programs\\neo\\bin\\toksi.exe`).
2. **User Home Database Location**: Default configuration database stored in `~/.toksi/toksi.db` (`users/.toksi`), with automatic startup migration from legacy `LOCALAPPDATA/toksi`.
3. **Workspace Root Consolidation**:
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
   - Workspace agent dirs (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`) only exist in the primary repo root.
4. **Tool Hook & MCP Naming Normalization**:
   - MCP tool names: `projectmem`, `context`, `codegraph`.
   - Hook command syntax standard: `<tool>-hook <agent>` (e.g. `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`).
   - Agent identity taxonomy: `antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude-cli;claude-desktop`, etc.
   - MCP instruction directories: `context-mode` -> `context`.
5. **Smart Background Launch for Local Agent CLIs**:
   - `AntigravityCli` (`agy.exe`), `CopilotCli` (`copilot.exe`), `ClaudeCli` (`claude`), `CursorBridge` (`cursor`), `GrokCli` (`grok.exe`) register as first-class BYOK providers.
   - Auto-checks OS Keyring / session tokens and smart-launches in detached background mode when authenticated and chat endpoint reachable.
6. **Strict Paid Quota Enforcement**:
   - Mandate: all BYOK providers must have an active paid commercial tier or subscription quota. Free accounts are disqualified from active gateway rotation.
   - Graceful Inform & Zero-Fail Policy: emits non-blocking informative status (`[INFO] Provider 'X' is on Free tier...`), never panics, bugs, or crashes, and smoothly falls back to paid provider or direct agent execution.
   - Gateway engages when `active_paid_providers >= 1`. If zero, graceful passthrough.
7. **Ground-Truth Session Token Accounting**:
   - Parses real usage tokens directly from live SSE response streams and upstream response headers. Zero synthetic/virtual guesses.

## 5. Supported Agent Profiles (`src/agents`)
Native configuration, automatic hook injection, and proxy protocol handling for 17 AI coding agents:
1. **Antigravity** (Google AGY CLI & IDE)
2. **Claude Code** (Anthropic CLI)
3. **Cline** (VS Code / JetBrains extension)
4. **Codex** (OpenAI Codex CLI)
5. **GitHub Copilot** (VS Code / Visual Studio & CLI)
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

## 6. Complete CLI Subcommands

- `toksi daemon [start|stop|status|restart]`: Supervises all 5 ports simultaneously in background.
- `toksi init`: Auto-detects installed coding agents, consolidates root `.toksi`, and injects proxy hooks and instructions.
- `toksi doctor [--fix]`: Probes agent configurations, proxy connectivity, token savings metrics, and PATH health.
- `toksi proxy`: Starts the background headroom proxy server and BYOK routing engine.
- `toksi auth [status|get|set|delete|sync]`: OS Keyring credential management.
- `toksi db [status|export|import|backup|restore]`: SQLite config database management.
- `toksi tui` / `toksi top`: Interactive mouse-capable terminal control center.
- `toksi dashboard [--port 9100] [--open]`: Background Web Telemetry & Control Cockpit.
- `toksi rules [sync|show|check]`: Synchronizes coding guidelines across workspace and user config homes.
- `toksi mem`: Native ProjectMem CLI and in-process MCP server.
- `toksi ctx`: Native ContextMode CLI and in-process MCP server.
- `toksi graph`: Native CodeGraph AST explorer and in-process MCP server.
- `toksi icm`: Instant Context Memory CLI and in-process MCP server.
- `toksi rtk`: RTK CLI output filter engine.
- `toksi info`: Displays active agent hooks, proxy ports, and token compression statistics.
- `toksi disable`: Reverts agent configurations to unproxied native state.
- `toksi update`: Self-updates `toksi` binary.

## 7. 12 Inspect-Vault Engines Audit & Feature Parity

| Vault Engine | Target Subsystem in `crates/toksi` | Key Features & Bugfixes Integrated |
|---|---|---|
| `inspect-vault/tokless` | `src/agents/`, `src/headroom/` | 17 agent adapters, proxy headers, JSONC parsing, environment injection. |
| `inspect-vault/bifrost` | `src/gateway/` | 39+ BYOK providers (including Bifrost aggregator, SageMaker, Bedrock Mantle), load balancing, failover cascades. |
| `inspect-vault/headroom` | `src/headroom/` | Dynamic MCP schema pruner, lossless deduplication, AST compression, Copilot dual-port (8789/8790). |
| `inspect-vault/rtk` | `src/rtk/` | 100+ command output filters, zero-allocation ANSI stripper, streaming interceptor. |
| `inspect-vault/context-mode` | `src/contextmode/` | Polyglot sandbox runner, SQLite FTS5 search index, async HTTP fetcher, `ctx_*` MCP tools. |
| `inspect-vault/codegraph` | `src/codegraph/` | Pure Tree-sitter parsers (Rust, TS/JS, Python, Go, C/C++), petgraph blast radius, symbol cache. |
| `inspect-vault/projectmem` | `src/projectmem/` | Atomic JSONL event store, deterministic zero-LLM summary, git staged diff precheck, in-process MCP. |
| `inspect-vault/ai-memory` | `src/aimemory/` | OKF v0.2 wiki store, claim-once multi-agent handoffs, progressive L0/L1/L2 disclosure. |
| `inspect-vault/icm` | `src/icm/` | SQLite FTS5 BM25 search, sub-10ms zero-token recall (`~/.toksi/icm/memory.db`). |
| `inspect-vault/caveman` | `src/rules/caveman.rs` | ASD-STE100 primitive English compressor; rule embedded via `include_str!`. |
| `inspect-vault/ponytail` | `src/rules/ponytail.rs` | Lazy senior developer build discipline ladder; rule embedded via `include_str!`. |
| `inspect-vault/andrej-karpathy-skills` | `src/rules/principles.rs` | Core engineering principles (think, simplify, surgical, verify); embedded via `include_str!`. |

## 8. Chronological Implementation Phases

- **Phase 1-14**: Completed (RTK filters, ProjectMem, ContextMode, ICM, ai-memory, CodeGraph, in-tree rules, Headroom & Gateway, downloader purge, legacy healing, non-destructive env injection, agent doctor, telemetry/TUI, verification, OS Keyring & SQLite DB).
- **Phase 15**: Unified 5-Port Daemon, Paid Quota Enforcement, Smart Agent CLI Providers & Standards Normalization:
  * **Phase 15.1**: Workspace Topology & Standards Normalization (`.agents/brain/PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md`)
    - Forward-slash `/` paths standard (Git Bash friendly) across all OSes.
    - Default SQLite config DB in `~/.toksi/toksi.db` (`users/.toksi/toksi.db`).
    - Single root `.toksi` workspace consolidation with row-level SQLite merge.
    - Sub-tools strictly inside `.toksi/<tool>` (`codegraph`, `headroom`, `projectmem`, `context`, `icm`, `ai-memory`, `rtk`).
    - Root-only workspace agent configuration directories (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`).
    - Uniform tool hook syntax: `<tool>-hook <agent>` (e.g. `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`).
    - Normalized MCP tool names (`projectmem`, `context`, `codegraph`) and instruction dirs.
  * **Phase 15.2**: Unified 5-Port Daemon & Copilot Supervision (`.agents/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md`)
    - Single-command daemon launch (`toksi daemon start`) supervising all 5 ports simultaneously (:8787, :8789, :8790, :9099, :9100).
    - Network port performance isolation: Port 9099 dedicated to low-latency LLM inference streaming; Port 9100 dedicated to Web UI Cockpit.
    - Copilot dual-port isolation: Port 8789 (VS Code proxy) and Port 8790 (CLI proxy) launched and monitored without conflict.
  * **Phase 15.3**: Smart Agent CLI Providers & Paid Quota Enforcement (`.agents/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md`)
    - Local Agent CLI providers (`AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CursorBridge`, `GrokCli`) registered as first-class peer BYOK providers.
    - Smart detached background launch with OS Keyring / session tokens and `/livez` health probes.
    - Strict paid quota mandate: require active commercial paid tier or subscription; disqualify free-tier accounts from BYOK gateway pool.
    - Graceful inform & zero-fail policy: emit non-blocking info notice without crashing, panicking, or breaking sessions; smooth fallback to active paid provider or direct agent execution.
    - Zero-provider passthrough: routes cleanly if zero external providers or only local CLI agent configured.
  * **Phase 15.4**: Ground-Truth Token Accounting & In-Tree Rules Sync (`.agents/brain/PHASE_15_4_TOKEN_ACCOUNTING_AND_RULES_SYNC.md`)
    - Ground-truth session token accounting: parse real prompt/completion/cached tokens and upstream rate limits directly from live SSE streams. Zero virtual math.
    - Universal rules/skills/instructions synchronizer for all 17 agents across project workspace and user config homes.
    - Complete usage help text updated in `main.rs`.
- **Phase 16**: Mouse-Capable TUI & Enterprise Web UI Cockpit:
  * **Phase 16.1**: Mouse-Capable Interactive TUI (`.agents/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md`)
    - Crossterm `EnableMouseCapture` for tab clicks, table row selection, action buttons, and mouse-wheel scrolling in `toksi tui` / `top`.
  * **Phase 16.2**: High-Performance Web Telemetry Cockpit on Port 9100 (`.agents/brain/PHASE_16_2_WEB_COCKPIT_DASHBOARD.md`)
    - Non-blocking background Web UI on `http://127.0.0.1:9100` (`toksi dashboard`).
    - 17-agent live observation grid, ground-truth token savings, upstream quota gauges, provider toggle/weights, load balancer algorithms, and SQLite DB export/import.
  * **Phase 16.3**: Technical README Rewrite & Workspace Verification (`.agents/brain/PHASE_16_3_README_AND_VERIFICATION.md`)
    - Complete technical README in `crates/toksi/README.md` covering architecture, ports, setup, agents, providers, paid quotas, rules sync, and CLI reference.
    - Complete workspace verification with 0 compiler warnings and all tests passing.

