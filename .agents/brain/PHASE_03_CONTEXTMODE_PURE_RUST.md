# Phase 03: Context-Mode Pure Rust Port

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_03_CONTEXTMODE_PURE_RUST.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/context-mode is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and tool additions into consolidated toksi.

Execute Phase 03: Context-Mode Pure Rust Port.
Source: neo/inspect-vault/context-mode (TypeScript)
Target: crates/toksi/src/contextmode/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly in `crates/toksi`. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors and `anyhow::Result` with `.context()` in CLI.
3. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all sandbox paths, working directories, and FTS5 paths.
4. PATH Validation Mandate: Validate executables (`bash`, `cmd`, `python`, etc.) in system `PATH` before executing in sandbox.
5. Strict `<repo>/.toksi/context/` Repo-Only Consolidation: Store workspace FTS5 index, journals, and compaction state ONLY inside workspace root `<repo>/.toksi/context/` (NEVER in `~/.toksi/context/`, NEVER outside a `.git` repository root, NEVER inside `<repo>/.git/`). Respect `.gitignore` & `.ignore` dynamically and track monorepo `subproject_id`.
6. Standardized Naming: Tool identifier is `context` (purge `toksi-context-mode` & `context-mode` MCP aliases). Hook syntax is `context-hook <agent>`. MCP skill dir is `skills/context/`.

Requirements:
1. Full pure Rust port of `context-mode` from TypeScript into `crates/toksi/src/contextmode/`:
   - `sandbox.rs` (isolated subprocess runner with strict CPU timeout, memory bounds, `< 2KB` projected stdout, OS sandbox integration)
   - `filter.rs` (output slicing, head/tail truncation, 98% context reduction)
   - `fts.rs` (SQLite FTS5 document chunker & BM25 search with `.gitignore`/`.ignore` dynamic purging and `.git/` exclusion)
   - `fetch.rs` (async reqwest HTTP fetcher with clean markdown extraction, SSRF per-hop guard with RFC private ranges)
   - `policy/` + `journal/` + `compaction/` (fix `commands/ctx.rs:417-434` `run_ctx_hook` passthrough `0` stub by dispatching real pre/post tool policies, session event journal, and pre-compact snapshot!)
   - `mcp.rs` (multi-protocol `ctx_*` MCP suite: `ctx_execute`, `ctx_execute_file`, `ctx_batch_execute`, `ctx_index`, `ctx_search`, `ctx_fetch_and_index`, plus `/context` slash command dispatch).
2. Pipe all sandbox stdout/stderr directly through `crates/toksi/src/rtk/` filters in-memory.
3. Multi-Protocol Support: Expose `context` tools over `MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`, and `DirectWireBus` for **35-component `<toksi-siagent>`** (routing through the shared 7-entrypoint `HeadroomEffortCompressor`, across **495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, and externalized zero-bloat `<skills>` & `<advisory>`).
4. Expose the native CLI command `toksi ctx ...`.
5. Ensure zero Node.js runtime dependency, 0 compiler warnings, and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Sandbox Process Runner** (`src/contextmode/sandbox.rs`): Executes shell or language commands in isolated sub-processes with CPU timeouts (`tokio::time::timeout`) and memory bounds.
- **SSRF Guard & Fetcher** (`src/contextmode/fetch.rs`): Async HTTP client enforcing strict SSRF validation across all redirects (blocking 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 127.0.0.0/8, 169.254.0.0/16, 100.64.0.0/10, 198.18.0.0/15, 240.0.0.0/4).
- **SQLite FTS5 Chunker & BM25 Engine** (`src/contextmode/fts.rs`): Chunks documents, indexes into FTS5, and dynamically purges records matching `.gitignore` patterns.
- **RTK Filter Pipe**: Sandbox outputs pipe through in-process RTK filters to eliminate token bloat before entering LLM context.

### 2. Disciplinary Standards & Invariants
- All output paths must use universal forward slashes `/`.
- Ensure non-blocking execution inside `tokio::task::spawn_blocking` for CPU-heavy parsing and SQLite transactions.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- contextmode` passes 100%.
- `toksi ctx execute --cmd "echo hello"` pipes through RTK and returns in <10ms.

---

## Wave 18 (v5.2) Synchronization Addendum
- Integrate `NeMo-text-processing` WFST text normalization (`src/contextmode/text_norm.rs`), `sage` domain allowlisting, `adr.rs` + `sandbox.rs` isolation, and shared 7-entrypoint `HeadroomEffortCompressor` output budgets (`Low`/`Medium`/`High`) across `<toksi-siagent>` (35 components, 495 mandated vaults, 43 `.agents/brain/*.md` guides, canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, externalized `<skills>` & `<advisory>`).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`
- **Wave 19--20 Reconciliation:** Remediated (`PHASE_19_1`): CLI, MCP, slash (`/ctxmode`), hook (`ctxmode-hook <agent>`), and dir (`<repo>/.toksi/ctxmode/`) surfaces standardized to **`ctxmode`** with lossless SQLite migration.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
