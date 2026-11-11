# Phase 03: Context-Mode Pure Rust Port

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_03_CONTEXTMODE_PURE_RUST.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

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
3. Multi-Protocol Support: Expose `context` tools over `MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`, and `DirectWireBus`.
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
