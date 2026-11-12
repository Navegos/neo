# Project Map - neo

Status: synchronized for **Toksi Native Consolidation Master Architecture Plan (Wave 17 / v4.0)**.

## Project purpose
--------------------------------------------------------------------------------
Enterprise-grade native Safe Rust (`#![deny(unsafe_code)]`) token optimization, intelligent multi-port proxying, native coding CLI/swarm runtime (`toksi chat|run|exec|swarm`), multi-protocol agent wire integration (`MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`, `DirectWireBus`), polyglot AST/SCIP code intelligence, and 9-tab telemetry cockpits connecting **18+ registered & 65/90+ ecosystem AI coding assistants** with **22 local engines** and multi-instance `<byok, model, effort>` upstream providers.

## Disciplinary Standards & Coding Conventions
- **Genuine Safe Rust (Edition 2024)**: `#![deny(unsafe_code)]` enforced strictly at `crates/toksi/src/{lib,main}.rs`. All internal `set_var`, `LockFileEx`, and `pre_exec(setsid)` unsafe blocks eliminated; unavoidable platform FFI (`advapi32`, `user32`, Job Objects, console) lives isolated in `crates/toksi-sys`, and any legacy C-ABI Tree-sitter shims live in `crates/toksi-grammars`.
- **Zero Warnings**: Must compile cleanly under `RUSTFLAGS="-D warnings"` and `cargo clippy --workspace --all-targets -- -D warnings`.
- **Strict Error Handling**: Zero panics, zero `.unwrap()` or `.expect()` in production. Typed errors via `thiserror` in libraries; `anyhow::Result` with `.context()` in CLI.
- **Async Concurrency**: Tokio work-stealing reactor + lock-free `tokio::sync::watch::Receiver<DashboardSnapshot>` for UI (<5ms response) and `spawn_blocking` for SQLite/disk transactions.
- **Continuous Reference Vault Re-Audit (36 Reference Vaults)**: `tokless`, `bifrost`, `headroom`, `rtk`, `context-mode`, `codegraph`, `projectmem`, `ai-memory`, `icm`, `ponytail`, `caveman`, `andrej-karpathy-skills`, `TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `litellm`, `TokenEconomizer`, `agent-opt`, `agents-structures`, plus Wave 17 vaults `hermes-agent`, `openclaw`, `openclaw-rs`, `mofa`, `mofa-engine`, `mofaclaw`, `codex`, `crustly`, `crush`, `goose`, `grok-build`, `gemini-cli`, `crewAI`.

## Cross-Platform Portability & Storage Topology
- **Path convention**: Universal forward slashes (`/`) via `src/util/paths.rs::normalize_path()`; escaped `\\` allowed only in Windows `cmd.exe` hook strings.
- **PATH validation**: Non-blocking `check_executable_in_path` (`src/util/env_path.rs`); missing binaries appear greyed out cleanly in `AI Agent Fleet & Quotas` with remediation hints.
- **User Home Config & DB (`~/.toksi/`)**: Declarative `~/.toksi/config.toml` (`src/config/mod.rs`) and persistent SQLite WAL database `~/.toksi/toksi.db` (`byok_profiles` `<byok, model, effort>`, `agent_profile_bindings`, `quota_windows`, `provider_keys`, `provider_models`) with full `export_dump`/`import_dump` (`--overwrite`).
- **Strict `<repo>/.toksi/<tool>` Repo-Only Storage**: Workspace `codegraph`, `projectmem`, `context`, `icm`, `ai-memory`, and `rtk` tracking live STRICTLY in root `<repo>/.toksi/<tool>/` (NEVER in `~/.toksi/<tool>/`, NEVER outside `.git` repo roots, NEVER inside `.git/` or nested crate subfolders). Monorepo crates (`<repo>/crates/project1..N/`) partition cleanly by `subproject_id`.
- **OS Keyring & Env Injection**: `crates/toksi-sys` Windows Credential Manager (normalized `gemini:antigravity` alias), macOS Keychain, Linux Secret Service; `HKCU\Environment` + `WM_SETTINGCHANGE` (never injecting `CLOUD_CODE_URL`), RAII `TeardownGuard` via `tokio::signal`/`ctrlc`.
- **Unified 7-Port Network Topology** (overridable via `~/.toksi/config.toml` `[ports]`):
  * `:8787` Headroom Streaming Proxy (`live_zone` cache surgery, `CcrStore`, `SmartCrusher`, mandatory `/byok/` pass, ConnectRPC HTTP/2, 10 `/...` slash cmds)
  * `:8788` Grok OAuth Daemon (`grok_daemon.rs` — distinct from BYOK)
  * `:8789` GitHub Copilot VS Code Proxy (`copilot_daemon.rs`)
  * `:8790` GitHub Copilot CLI Proxy (`copilot_daemon.rs`)
  * `:18787` Multi-Profile BYOK Gateway (`byok_gateway.rs` — supervised by daemon)
  * `:9099` Enterprise AI Gateway & Drop-in SDKs + `/metrics` (`mofa-engine` 7-D router + `LoadBalancer` + `cascade` + `Chroma` ANN cache)
  * `:9100` Detached 9-Tab Web Telemetry Cockpit (`toksi dashboard start|stop|restart`)

## Structure
- `crates/toksi-sys`: Leaf platform FFI crate (Win32 Registry, Broadcast, Credential Manager, Console/Job Objects with `// SAFETY:` proofs).
- `crates/toksi-grammars`: Leaf Tree-sitter C-ABI wrapper crate keeping `crates/toksi` `#![deny(unsafe_code)]` across 41+ languages.
- `crates/toksi`: Native Rust workspace core:
  * `config/`: `~/.toksi/config.toml` loader, `[ports]`, `default_agents`, `direct_wire`, `[test] incremental`, custom `base_url` overrides.
  * `agent_runtime/`: Native `toksi chat|run|exec|swarm` coding CLI fusing `hermes-agent`, `openclaw`, `openclaw-rs`, `mofa`, `mofaclaw`, `crewAI`, `codex`, `crustly`, `crush`, `goose`, `grok-build`, `gemini-cli` + 10 instant `< 5ms` slash commands.
  * `gateway/`: 65/90+ Categories 1–5 + 22 local inference engines, zero paid-tier restrictions (`FreeTierDisqualified` removed), multi-instance `<byok, model, effort>` routing, `ArcSwap` reconnect watchdog, `Chroma` ANN + `foyer` semantic cache.
  * `headroom/`: Pure-Rust `headroom-core` (`live_zone`, `CcrStore`, `SmartCrusher`, `SearchCompressor`, `DiffCompressor`, `AuthMode`), `TokenEconomizer`, mandatory `/byok/` pass, HTTP/2 ConnectRPC passthrough (`#0033`/`#0034`).
  * `keyring/` & `db/`: Configured-only Keyring (`gemini:antigravity` fixed) + SQLite `byok_profiles` + full `export`/`import`.
  * `telemetry/`, `tui/`, `dashboard/`: Ground-truth SSE/Responses accounting (`0` idle), rate-limit `quota_windows` (`hour..year`), exact **9-tab parity** (`Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`), clickable heatmap modal, zero synthetic stubs.
  * `agents/` & `tools/`: 18+ built-in + 65/90+ ecosystem harnesses (`antigravity-cli|ide` split, `"openagents"`, `.agents/mcp`, `<tool>-hook <agent>`), `DirectWireBus`, real `A2aTaskStore` v1.0 & `SessionActorQueue` ACP v2.0.
  * `rtk/`, `projectmem/`, `icm/`, `aimemory/`, `codegraph/`, `contextmode/`, `rules/`: 100+ RTK filters + SQLite `Tracker`, 16-tool ProjectMem, 28-tool ICM + OKF wiki, 41+ Tree-sitter + real `ast-grep`/`scip`/`jcodemunch`/`repomix`, real `ctx-hook` policy/journal/compaction, `.gitignore`/`.ignore` purge, compile-time Karpathy/Caveman v4/Ponytail rules.
- `crates/nbuild`: Universal toolchain manager, local GitHub Actions workflow runner, package engine, and remote worker runner for AI agents.
- `crates/archinfo`: Hardware CPU feature detection, ISA mapping, and target triple platform analysis.
- `inspect-vault/*`: All 36 upstream reference vaults audited for Wave 17 consolidation.

## Key Files & Artifacts
- `crates/toksi/research/RESEARCHPLAN.md`: Master Wave 17 scientific & technical blueprint (100% of `RESEARCHPLAN.txt` preserved + verified Section 13 contracts)
- `crates/toksi/CONSOLIDATION_PLAN.md`: Full architectural & worker execution specification (Phases 1–17.7)
- `crates/toksi/PLAN.md`: Concise chronological phase tracker
- `crates/toksi/README.md`: Wave 17 user & operator manual
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` & `.agents/brain/PHASE_01`..`PHASE_17_7`: Complete modular worker prompt suite (`31` files)
- `Cargo.toml`, `crates/toksi/Cargo.toml`, `crates/toksi/src/lib.rs`, `crates/toksi/src/main.rs`
