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

## Disciplinary Coding Standards & Conventions Mandate (For All Agent Workers)
- **Safe Rust (Edition 2024)**: `#![deny(unsafe_code)]` crate-wide. Unsafe permitted exclusively in isolated OS FFI leaf modules (`src/keyring/os_windows.rs`, `src/env_injector/windows.rs`) with mandatory `// SAFETY:` proofs.
- **Zero Warnings Policy**: Code must compile cleanly under `RUSTFLAGS="-D warnings"` with zero Clippy lints (`cargo clippy -p toksi --all-targets -- -D warnings`).
- **Strict Error Handling**: Prohibit `.unwrap()`, `.expect()`, `panic!()`, `unreachable!()` in production. Typed errors via `thiserror` in libraries; `anyhow::Result` with `.context()` in CLI.
- **Continuous Reference Vault Re-Audit**: All 23 reference vaults in `neo/inspect-vault/*` (`tokless`, `bifrost`, `headroom`, `rtk`, `context-mode`, `codegraph`, `projectmem`, `ai-memory`, `icm`, `ponytail`, `caveman`, `andrej-karpathy-skills`, `TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `litellm`, `TokenEconomizer`, `agent-opt`, `agents-structures`) are ALWAYS treated as new/updated code to inspect, convert, and import, verifying against missing features, addons, updates, new agents, and bugfixes.
- **Cross-Platform Portability (Windows, Linux, macOS)**:
  * Universal forward-slash `/` paths standard (`normalize_path(&str)`). Escaped `\\` allowed ONLY in `cmd.exe` commands inside JSON5/TOML configs.
  * Filesystem case-insensitivity handling: normalize search terms before lookup.
  * User home DB storage (`~/.toksi/toksi.db`) with automatic migration from legacy paths.
  * Cross-platform file locking (`fs2`) for JSONL appends and SQLite WAL checkpoints.
  * OS Keyring: `advapi32.dll` (Windows), Keychain (macOS), Secret Service (Linux).
  * Environment: `HKCU\Environment` + `WM_SETTINGCHANGE` (Windows) vs systemd/launchctl (Unix). RAII `TeardownGuard`.
  * Detached processes: `CREATE_NO_WINDOW`/`DETACHED_PROCESS` (Windows) vs `setsid()` (Unix).
  * Sockets: Bind strictly to `127.0.0.1` / `::1`, set `SO_REUSEADDR`.
  * Line endings: Normalize `\r\n` to `\n` across all parsers, JSONL, and token accounting.

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
- [x] Phase 15.1: Workspace Topology, Forward-Slash Normalization, Standards & PATH Validation Mandate:
  > Plan 15.1: `.agents/brain/PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md`
  - [x] Implement `normalize_path(p: &str) -> String` in `crates/toksi/src/util/paths.rs` converting all `\` to `/` across all OSes (Windows, Linux, macOS). Handle drive letters (`C:/`), UNC paths (`//server`), and tildes. Escaped `\\` allowed only for cmd.exe commands in JSON5/TOML configs.
  - [x] Implement `check_executable_in_path(cmd: &str) -> Result<(), String>` in `crates/toksi/src/util/env_path.rs`. Validate required binaries (`bash`, `cmd`, `toksi`, `agy.exe`, `copilot.exe`, `claude`, `cursor`, `grok`, tools) exist in `PATH`. Never hardcode absolute paths. On missing binary, log clean remediation instruction.
  - [x] Relocate default SQLite database to `~/.toksi/toksi.db` (`dirs::home_dir()`), NOT in `localdata/toksi`. Implement automatic startup migration from legacy `LOCALAPPDATA/toksi`.
  - [x] Single root `.toksi` consolidation: discover subfolder `.toksi` directories (e.g. `crates/toksi/.toksi`), merge table rows into root `.toksi/toksi.db`, `memory.db`, `symbols.db` without data loss, and delete nested subfolder instances.
  - [x] Consolidate tool layout strictly under `.toksi/<tool>`: `.toksi/{codegraph, headroom, projectmem, context, icm, ai-memory, rtk}`. Automatically migrate loose legacy folders (`.codegraph`, `.projectmem`, `.context-mode`, etc.) into `.toksi/<tool>`.
  - [x] Enforce root-only workspace agent folders: `.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.gemini`, `.github`, `.agents` strictly in the primary workspace root.
  - [x] Standardize tool hook syntax: `<tool>-hook <agent>` (e.g. `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`).
  - [x] Normalize MCP tool identifiers: `toksi-projectmem` -> `projectmem`, `toksi-context-mode` -> `context`, `toksi-codegraph-index` -> `codegraph`. Map MCP settings to `skills/<tool>`.
  - [x] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [x] Phase 15.2: Unified 5-Port Daemon & Copilot Supervision:
  > Plan 15.2: `.agents/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md`
  - [x] Supervise all 5 dedicated endpoints simultaneously in `toksi daemon start`:
    * Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
    * Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
    * Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
    * Port 9099: Enterprise AI Gateway & Drop-in SDKs (`/v1/chat/completions`, `/v1/messages`, `/metrics`)
    * Port 9100: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)
  - [x] Guarantee port latency isolation: port 9099 strictly dedicated to streaming inference (<10 µs latency overhead); port 9100 strictly dedicated to web dashboard cockpit.
  - [x] Verify Copilot dual-port wiring (8789 & 8790) with health probes (`/livez`) and collision prevention.
  - [x] Ensure non-destructive environment lifecycle: clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix session env on daemon shutdown.
  - [x] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement:
  > Plan 15.3: `.agents/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md`
  - [ ] Securely register 6 Local Agent CLIs as first-class peer BYOK providers (`agy.exe`, `copilot.exe`, `claude`, `codex`, `cursor`, `grok.exe`).
  - [ ] Implement smart background launch in detached mode (`CREATE_NO_WINDOW`/`DETACHED_PROCESS` on Windows, `setsid()` on Unix) when authenticated via OS Keyring/session tokens; verify chat endpoint reachability (`/livez` / ConnectRPC).
  - [ ] Integrate Bifrost provider client into `gateway/providers/aggregators/bifrost.rs` with health probes, routing, and load balancing.
  - [ ] Enforce strict paid commercial quota mandate: disqualify free accounts (unbilled AI Studio trial keys, expired credits, free Copilot trials) from active rotation.
  - [ ] Graceful Inform & Zero-Fail Policy: never crash, panic, or fail; emit informative notice (`[INFO] Provider 'X' is on Free tier... Bypassing from active pool without error.`) and smoothly fall back to next paid provider or direct agent execution.
  - [ ] Gateway activation guard: engage when `active_paid_providers >= 1`; graceful passthrough if zero.
  - [ ] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging:
  > Plan 15.4: `.agents/brain/PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md`
  - [ ] Expose unified multi-protocol interfaces (MCP, A2A, ACP, stdio, SSE, HTTP) across all tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`).
  - [ ] Integrate ripgrep-grade ignore walker respecting `.gitignore` and `.ignore` files during AST and document indexing.
  - [ ] Dynamic vector & cache purging: when ignore rules are added/modified, automatically purge previously indexed records from SQLite FTS5 search indexes and vector embedding caches.
  - [ ] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing:
  > Plan 15.5: `.agents/brain/PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md`
  - [ ] Extract real prompt, completion, and cached tokens directly from live SSE response streams and upstream rate-limit headers (`x-ratelimit-remaining-tokens`, `anthropic-ratelimit-requests-remaining`). Prohibit virtual/synthetic math (counter reads 0 when idle).
  - [ ] Query provider quotas via free agent AI API channels without token cost.
  - [ ] Implement network reconnection auto-healing watchdog in `gateway/reconnect.rs` to detect interface toggles, timeouts, and VPN drops, auto-refreshing connection pools without dormant lockout.
  - [ ] Universal rules/skills sync across all 17 agents; comprehensive update of CLI usage help in `crates/toksi/src/main.rs`.
  - [ ] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 15.6: Advanced Vault Intelligence Engines Consolidation:
  > Plan 15.6: `.agents/brain/PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md`
  - [ ] Consolidate remaining reference vaults in pure Rust:
    * `TokenTracker` -> `crates/toksi/src/telemetry/tokentracker.rs`
    * `jcodemunch-mcp` -> `crates/toksi/src/codegraph/jcodemunch.rs`
    * `repomix` -> `crates/toksi/src/codegraph/repomix.rs`
    * `ast-grep` -> `crates/toksi/src/codegraph/ast_grep.rs`
    * `scip` -> `crates/toksi/src/codegraph/scip.rs`
    * `mem0` -> `crates/toksi/src/aimemory/mem0_bridge.rs`
    * `chroma` -> `crates/toksi/src/gateway/semantic_cache.rs`
    * `litellm` -> `crates/toksi/src/gateway/normalizer/litellm.rs`
    * `TokenEconomizer` -> `crates/toksi/src/headroom/compression/economizer.rs`
    * `agent-opt` -> `crates/toksi/src/agents/opt.rs`
  - [ ] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 16.1: High-Performance Mouse-Capable Interactive TUI:
  > Plan 16.1: `.agents/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md`
  - [ ] Enable crossterm `EnableMouseCapture`: interactive tab clicks, entity row selection, action buttons, scrollable inspection panels.
  - [ ] Replace blocking event polling with non-blocking async loop to permanently fix terminal hangs and keyboard input freezes.
  - [ ] Realistic real-time telemetry: true token consumption, ground-truth savings, and active BYOK quota gauges.
  - [ ] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard:
  > Plan 16.2: `.agents/brain/PHASE_16_2_WEB_COCKPIT_DASHBOARD.md`
  - [ ] Non-blocking background Web Cockpit on Port 9100 (`toksi dashboard`).
  - [ ] Real-time 17-agent observation grid, ground-truth token savings, upstream quota gauges, provider toggle/weights/latency controls, DB backup/export.
  - [ ] Verify: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.
- [ ] Phase 16.3: Open Agents Directory Protocols & README Verification:
  > Plan 16.3: `.agents/brain/PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md`
  - [ ] Support open and vendor agent layouts (`.agents/`, `.gemini/`, `.copilot/`, `.claude/`, `.codex/`, `.cursor/`, `.grok/`).
  - [ ] Auto-install to user directory (`~/.<agent>/`) if selected; workspace repository install is strictly optional and user-selected.
  - [ ] Full rewrite/expansion of `crates/toksi/README.md`; 0 compiler warnings, 100% tests pass.

## Shipped
- [x] Initial toksi proxy, headroom, and command dispatcher
- [x] Phases 1-14 implementation and e2e verification
