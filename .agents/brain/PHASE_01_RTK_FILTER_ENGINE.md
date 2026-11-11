# Phase 01: RTK Filter Engine Integration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_01_RTK_FILTER_ENGINE.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/rtk is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and filter additions into consolidated toksi.

Execute Phase 01: RTK Filter Engine Integration.
Source: neo/inspect-vault/rtk
Target: crates/toksi/src/rtk/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly in `crates/toksi` (zero `unsafe` in `rtk/`). Zero compiler warnings and zero Clippy lints (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors and `anyhow::Result` with `.context()` in CLI.
3. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all paths.
4. PATH Validation Mandate: Validate executables in system `PATH` before executing external commands (`rtk` never spawns Node/Python).
5. In-process multi-protocol exposure: `MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`, plus `DirectWireBus` (`direct-wired`) alongside `config-wired`.
6. Repo-Only Tracking Scope: Project-level `rtk` tracking artifacts live strictly inside `<repo>/.toksi/rtk/` (respecting `.gitignore`/`.ignore` and monorepo `subproject_id`), never in user home project dirs or `<repo>/.git/`.

Requirements:
1. Full pure Rust port of RTK's 100+ command-specific output filters (`git`, `cargo`, test runners, linters, `docker`, `npm`) and zero-allocation ANSI stripper into `crates/toksi/src/rtk/`, plus Wave 17 parity modules (`tracker` SQLite `rtk gain` JSON/history, `tee` raw failure recovery, `discover` missed-savings scanner, `learn` CLI correction detector, and `/rtk` slash command dispatch).
2. Adapt RTK to serve as the shared in-process filter engine for:
   - `crates/toksi/src/contextmode/` (filtering sandbox outputs)
   - `crates/toksi/src/projectmem/` (extracting root-cause failure traces)
   - `crates/toksi/src/headroom/` (pre-filtering tool outputs)
3. Expose the native CLI command `toksi rtk <command...>`, including `toksi rtk gain`, `toksi rtk discover`, `toksi rtk learn`.
4. Implement standard tool hook syntax: `rtk-hook <agent>` (e.g., `rtk-hook agy`, `rtk-hook antigravity-cli`, `rtk-hook copilot`, `rtk-hook cursor`). Purge legacy aliases such as `rtk hook cursor` (`main.rs:340`).
5. Ensure 100% functional parity, zero missing filter code, zero external binary spawns, 0 compiler warnings, and targeted `cargo test -p toksi` pass (or `.toksi/test-baseline.json` skip if unchanged).
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Zero-Allocation ANSI Stripper** (`src/rtk/ansi.rs`): High-throughput streaming state machine stripping SGR sequences and color codes with zero memory reallocations.
- **Filter Registry & Engine** (`src/rtk/engine.rs`): Subsystem parsing command lines, dispatching to specialized domain filters (`git`, `cargo`, `docker`, `npm`, `pytest`, `ruff`, `nextest`).
- **Hook Dispatcher** (`src/rtk/hook.rs`): Standardized `<tool>-hook <agent>` handler executing command lines through the in-process filter and emitting token-condensed stdout.
- **Multi-Protocol Bridge** (`src/rtk/mod.rs`): Exposing filter capabilities over MCP (`rtk_filter`), A2A, ACP, stdio, and HTTP.

### 2. Disciplinary Standards & Invariants
- Enforce forward-slash `/` paths in all diagnostic and command outputs.
- Retain root-cause failure traces (panic, error, fatal) byte-for-byte; compress repetitive and informational lines.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- rtk` passes 100%.
- `toksi rtk cargo test` slashes output tokens by >60% while preserving test failures.
