# Project Map - neo

Status: updated for Toksi Native Consolidation Master Architecture Plan (v3.4).

## Project purpose
--------------------------------------------------------------------------------
Enterprise-grade native Rust tooling, token-optimization, multi-protocol agent wire integration, and headroom gateway pipeline for AI coding assistants. Transparently sits between 17 coding agents and 39+ upstream LLM providers to cut token spend by 30-70%, enforce latency/cost/concurrency quotas, and provide zero-latency memory, AST code intelligence, and real-time telemetry across dedicated isolated ports.

## Stack
- Language: Safe Rust (Edition 2024)
- Manifest: Cargo.toml (workspace)
- Crates: `crates/toksi`, `crates/nbuild`, `crates/archinfo`
- Path convention: Universal forward slashes (`/`) across Windows and Unix for Git Bash / POSIX compatibility (with escaped `\\` preserved in JSON5/TOML configs for Windows cmd hooks.json commands)
- PATH validation: Strict "Rule of test and inform user" — test required executables in system PATH, never hardcode absolute binary paths everywhere
- Storage: Default SQLite configuration database stored in user home (`~/.toksi/toksi.db`), not `localdata/toksi`
- Workspace Topology:
  * Only ONE `.toksi` directory in root workspace (`<root>/.toksi/`)
  * Subfolder `.toksi` directories auto-merged into `<root>/.toksi/` and deleted
  * Tool subdirectories consolidated under `.toksi/<tool>`: `.toksi/{codegraph, headroom, projectmem, context, icm, ai-memory, rtk}`
  * Workspace agent configuration directories (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.agents`) strictly located in first workspace root
- Tool Hook & MCP Conventions:
  * Hook syntax standard: `<tool>-hook <agent>` (e.g. `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`)
  * Normalized MCP tool names: `projectmem`, `context`, `codegraph`
  * Normalized MCP instruction directories: `context`
  * MCP settings to skills mapping: `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp` -> `skills/<tool>`
  * Agent taxonomy: `antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude-cli;claude-desktop`, etc.
- Multi-Protocol Standards:
  * MCP (Model Context Protocol), A2A (Agent-to-Agent), ACP (Agent Communication Protocol), stdio, SSE, HTTP
- Network Port Topology:
  * Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
  * Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
  * Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
  * Port 9099: Enterprise AI Gateway & Drop-in SDKs (`/v1/chat/completions`, `/v1/messages`, `/metrics`)
  * Port 9100: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)

## Structure
- `crates/toksi`: 100% Native Rust consolidation of:
  * `gateway`: 39+ BYOK providers (including `Bifrost`, `AWS SageMaker`, `AWS Bedrock Mantle`), first-class local Agent CLI providers (`AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CursorBridge`, `GrokCli`) with smart background launch and reachability verification, strict paid quota enforcement (disqualifying free-tier/trial accounts with non-blocking inform and zero-fail fallback), network reconnect auto-healing watchdog, 8-category failover cascade, dynamic load balancer, complexity tier router, dual-tier semantic cache, drop-in SDK endpoints (:9099).
  * `headroom`: Transparent HTTP/SSE streaming proxy (:8787), dual-port Copilot supervisor (:8789 and :8790), log deduplication, AST code/whitespace compression, JSON minifier, dynamic MCP schema pruner (saves 5k-15k tokens/turn), 1024-token prompt cache alignment, adaptive memory headroom scaling.
  * `keyring`: Cross-platform OS Keyring credential manager (Windows Credential Manager via `advapi32.dll`, macOS Keychain, Linux Secret Service) with base64 OAuth envelope parser.
  * `db`: Persistent SQLite WAL configuration database (`~/.toksi/toksi.db`) with online backup, JSON export/import, row-level table merge, and auto-migration from legacy paths.
  * `env_injector`: Non-destructive OS environment injection (`HKCU\Environment` on Windows with `WM_SETTINGCHANGE` broadcast; session env on Unix) and auto-healer for legacy shims.
  * `telemetry`: Ground-truth session metrics and rate-limit tracking directly from live SSE response streams, TokenTracker integration, zero virtual/synthetic math, async zero-latency non-blocking metrics collector, native Prometheus exposition (:9099/metrics), real-time USD cost and savings calculator.
  * `tui`: Full-featured interactive terminal user interface (`ratatui` + `crossterm`) with full mouse capture (tabs, tables, action buttons, scroll) and non-blocking event polling to eliminate hangs.
  * `dashboard`: Zero-dependency embedded Web UI Telemetry & Control Cockpit (:9100, background non-blocking) with 17-agent live observation grid, realistic token savings, provider routing, paid tier badges, and load balancer management.
  * `agents`: Native configs, hooks, and multi-protocol wiring for 17 AI coding assistants and Open Agents (`.agents/`, `.gemini/`, `.copilot/`, `.claude/`, `.codex/`, `.cursor/`, `.grok/`). User auto-install vs selective workspace install.
  * `rtk`: 100+ command output filters (git, cargo, test runners, linters, docker) and zero-allocation ANSI stripper.
  * `projectmem`: Safe Rust port of ProjectMem (~v0.3.4) with atomic JSONL store, zero-LLM summary, git staged diff precheck, and multi-protocol MCP/A2A/ACP server.
  * `icm`: Instant Context Memory with local SQLite FTS5 BM25 search and sub-10ms zero-token recall (`~/.toksi/icm/memory.db`).
  * `aimemory`: Open Knowledge Format (OKF v0.2) wiki store, claim-once multi-agent handoff state machine, progressive L0/L1/L2 disclosure, and mem0 long-term memory extraction.
  * `codegraph`: Pure Rust Tree-sitter parsers, SCIP code intelligence, ast-grep structural search, jcodemunch symbol chunking, Repomix codebase packager, petgraph dependency/blast-radius engine, and AST symbol cache with dynamic .gitignore/.ignore index purging.
  * `contextmode`: Subprocess sandbox runner with CPU/memory bounds, SQLite FTS5 search index with .gitignore purging, async HTTP fetcher, piped through RTK filters.
  * `rules`: In-tree compile-time embedded guidelines (`include_str!`): Karpathy principles, Caveman ASD-STE100 compressor, Ponytail lazy senior build discipline; universal synchronizer for `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md`.
- `crates/nbuild`: Universal toolchain manager, local GitHub Actions workflow runner, package engine, and remote worker runner for AI agents.
- `crates/archinfo`: Hardware CPU feature detection, ISA mapping, and target triple platform analysis.
- `inspect-vault/*`: Consolidated reference vaults (tokless, bifrost, headroom, rtk, context-mode, codegraph, projectmem, ai-memory, icm, ponytail, caveman, andrej-karpathy-skills, TokenTracker, jcodemunch-mcp, repomix, ast-grep, scip, mem0, chroma, agents-structures).
