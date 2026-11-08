# neo — plan

> Editable **intent** file: ideas + plans — what we *mean to do*.
> This is NOT the event log. `events.jsonl` -> `summary.md` records what
> *happened*; this file records what we *intend*. The AI reads it at
> session start and edits it directly (like `PROJECT_MAP.md`): add ideas
> and plans, check items off, move done work down to Shipped. Plans are
> never logged as events.

## Ideas
_Loose thoughts, not yet committed to._
- Distributed build remote execution using NativeLink REAPI worker daemon.
- Automatic translation of GitHub Actions `.github/workflows/*.yml` directly into local runnable pipelines.
- AI Agent toolchain auto-provisioning: agent queries missing header/tool -> nbuild auto-fetches and injects into environment.

## Active plans
_What we're working toward now. Use `- [ ]` / `- [x]` checklists._

### toksi — Native Rust Consolidation & Single-Daemon Cockpit (v3.4)
> Master blueprint: `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md`.
> Modular Brain Execution Index: `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`
- [x] Phase 1: RTK Engine Integration (100+ filters, ANSI stripper, `toksi rtk`)
- [x] Phase 2: ProjectMem Pure Rust Port (schema, JSONL store, summary, precheck, MCP)
- [x] Phase 3: Context-Mode Pure Rust Port (sandbox runner, FTS5 chunker, `ctx_*` tools)
- [x] Phase 4: ICM & ai-memory Integration (SQLite FTS5, multi-agent handoff, tiered briefs)
- [x] Phase 5: CodeGraph Pure Rust Engine (Tree-sitter, petgraph, symbols cache)
- [x] Phase 6: In-Tree Rules & Skills (Karpathy, Caveman, Ponytail embedded via `include_str!`)
- [x] Phase 7: Headroom Deep Integration & Gateway Catalog (39 providers, semantic cache)
- [x] Phase 8: Downloader Purge (zero npm/pip/node/python dependency)
- [x] Phase 9: Legacy Healing & Binary Un-Hijack (`heal.rs`)
- [x] Phase 10: Non-Destructive Environment Injection (`windows.rs`, `WM_SETTINGCHANGE`)
- [x] Phase 11: Agent Doctor & Self-Healing Parity (`doctor --fix`, 17 agent harnesses)
- [x] Phase 12: TUI, Prometheus Telemetry & Web Dashboard (`/metrics`, TUI, dashboard)
- [x] Phase 13: End-to-End System Verification (10/10 e2e tests passing)
- [x] Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration
- [ ] Phase 15.1: Workspace Topology, Forward-Slash Normalization, Standards & PATH Validation Mandate:
  > Plan 15.1: `.agents/brain/PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md`
  - [ ] Implement `normalize_path` with universal forward slashes `/` across all paths (Windows & Unix), preserving escaped `\\` in JSON5/TOML configs for cmd hooks.json commands
  - [ ] Implement PATH verification mandate ("rule of test and inform user"): check executable in PATH before launch, inform user cleanly if missing, never hardcode absolute paths
  - [ ] Relocate default SQLite database to `~/.toksi/toksi.db` (`users/.toksi`) with startup migration from legacy paths
  - [ ] Single root `.toksi` consolidation: discover subfolder `.toksi` dirs, merge rows into root `.toksi/toksi.db`, and delete legacy instances
  - [ ] Tool layout under `.toksi/<tool>`: `.toksi/{codegraph, headroom, projectmem, context, icm, ai-memory, rtk}`
  - [ ] Root-only agent folders: `.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.agents` strictly in first workspace root
  - [ ] Standardize hook syntax (`<tool>-hook <agent>`) and MCP naming (`projectmem`, `context`, `codegraph`)
  - [ ] Map MCP settings to skills path: `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp` -> `skills/<tool>`
- [ ] Phase 15.2: Unified 5-Port Daemon & Copilot Supervision:
  > Plan 15.2: `.agents/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md`
  - [ ] Supervise all 5 dedicated endpoints simultaneously in `toksi daemon start`:
    * Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
    * Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
    * Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
    * Port 9099: Enterprise AI Gateway & Drop-in SDKs (`/v1/chat/completions`, `/v1/messages`, `/metrics`)
    * Port 9100: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)
  - [ ] Guarantee port latency isolation: port 9099 strictly for streaming inference; port 9100 for web cockpit
  - [ ] Verify Copilot dual-port wiring (8789 & 8790) with health probes (`/livez`)
  - [ ] Teardown Windows `HKCU\Environment` variables cleanly on daemon shutdown
- [ ] Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement:
  > Plan 15.3: `.agents/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md`
  - [ ] Register 5 Local Agent CLIs as first-class peer BYOK providers (`agy.exe`, `copilot.exe`, `claude`, `cursor`, `grok.exe`)
  - [ ] Smart background launch in detached mode when authenticated via OS Keyring/session tokens; verify chat endpoint reachability
  - [ ] Add Bifrost provider client into `gateway/providers/aggregators/bifrost.rs`
  - [ ] Enforce strict paid quota mandate: disqualify free accounts from active rotation; emit non-blocking inform (`[INFO] Provider 'X' is on Free tier...`), never crash or panic, fall back gracefully
  - [ ] Gateway activation guard: engage when `active_paid_providers >= 1`; graceful passthrough if zero
- [ ] Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging:
  > Plan 15.4: `.agents/brain/PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md`
  - [ ] Expose multi-protocol interfaces (MCP, A2A, ACP, stdio, SSE, HTTP) across all tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`)
  - [ ] Integrate `.gitignore` and `.ignore` support in all indexing walkers
  - [ ] Dynamic vector & cache purging: when ignore rules are added/modified, automatically purge previously indexed records from FTS5 BM25 search and vector embedding cache
- [ ] Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing:
  > Plan 15.5: `.agents/brain/PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md`
  - [ ] Extract real prompt/completion/cached tokens from live SSE response streams and upstream rate-limit headers; zero virtual math (counter reads 0 when idle)
  - [ ] Query provider quotas via free agent AI API channels without token cost
  - [ ] Implement network reconnection auto-healing watchdog in `gateway/reconnect.rs` to eliminate dormant states
  - [ ] Universal rules/skills sync across 17 agents; update CLI usage help in `main.rs`
- [ ] Phase 15.6: Advanced Vault Intelligence Engines Consolidation:
  > Plan 15.6: `.agents/brain/PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md`
  - [ ] Consolidate remaining reference vaults in pure Rust: `TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`
- [ ] Phase 16.1: High-Performance Mouse-Capable Interactive TUI:
  > Plan 16.1: `.agents/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md`
  - [ ] Enable crossterm `EnableMouseCapture`: tab clicks, row selection, action buttons, scrollable views
  - [ ] Replace blocking event polling with non-blocking async loop to permanently fix terminal hangs and keyboard input freezes
- [ ] Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard:
  > Plan 16.2: `.agents/brain/PHASE_16_2_WEB_COCKPIT_DASHBOARD.md`
  - [ ] Non-blocking background Web Cockpit on `:9100` (`toksi dashboard`)
  - [ ] 17-agent live observation grid, ground-truth token savings, upstream quota gauges, gateway/loadbalancer controls, DB export
- [ ] Phase 16.3: Open Agents Directory Protocols & README Verification:
  > Plan 16.3: `.agents/brain/PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md`
  - [ ] Support open and vendor agent layouts (`.agents/`, `.gemini/`, `.copilot/`, `.claude/`, `.codex/`, `.cursor/`, `.grok/`)
  - [ ] Auto-install to user directory (`~/.<agent>/`) if selected; workspace repository install is strictly optional and user-selected
  - [ ] Full rewrite/expansion of `crates/toksi/README.md`; 0 compiler warnings, 100% tests pass

## Shipped
- [x] Initial toksi proxy, headroom, and command dispatcher
- [x] Phases 1-14 implementation and e2e verification
