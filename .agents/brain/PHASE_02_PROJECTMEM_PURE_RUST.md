# Phase 02: ProjectMem Pure Rust Port

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_02_PROJECTMEM_PURE_RUST.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/projectmem is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and schema extensions into consolidated toksi.

Execute Phase 02: ProjectMem Pure Rust Port.
Source: neo/inspect-vault/projectmem (~v0.3.4, Python)
Target: crates/toksi/src/projectmem/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly in `crates/toksi`. Remove ALL raw `LockFileEx`/`OVERLAPPED`/`fcntl` `unsafe` blocks from `src/projectmem/store.rs` using safe `std::fs::File::lock()` / `fs2`. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors and `anyhow::Result` with `.context()` in CLI.
3. Cross-Platform File Locking: Safe `std::fs::File::lock()` / `fs2` locking on `<repo>/.toksi/projectmem/events.jsonl`.
4. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all file locations and AST references.
5. Strict `<repo>/.toksi/projectmem/` Repo-Only Consolidation: Store ALL workspace ProjectMem events, summaries, and index metadata strictly inside workspace root `<repo>/.toksi/projectmem/` (NEVER in `~/.toksi/projectmem/`, NEVER inside non-`.git` folders, NEVER inside `<repo>/.git/` or subfolder crates). Merge any nested `.projectmem/` or subfolder `.toksi/projectmem/` into root `<repo>/.toksi/projectmem/` and tag monorepo `subproject_id` (`<repo>/crates/*`).
6. Standardized Naming: Tool identifier is `projectmem` (purge legacy `toksi-projectmem` and `util/projectmem.rs`). Hook syntax is `projectmem-hook <agent>`.

Requirements:
1. Full pure safe Rust port of ProjectMem from Python into `crates/toksi/src/projectmem/`:
   - `schema.rs` (`Event`, `Issue`, `Attempt`, `Fix`, `Decision`, `Note`)
   - `store.rs` (safe locked append to `<repo>/.toksi/projectmem/events.jsonl`, atomic markdown generation, `staleness`/`structure` metadata)
   - `summary.rs` (deterministic zero-LLM summary generator from active issues, fixes, decision DAG, gotchas)
   - `precheck.rs` (git staged diff scanner vs gotchas and historical failure locations)
   - `redaction.rs` (zero-allocation regex secret and API key sanitizer)
   - `heatmap.rs` (failure clustering, ROI metrics, clickable cell drilldown payloads for UI `Memory Decision DAG` modal)
   - `global.rs` (repo-registry metadata index without duplicating workspace `events.jsonl` outside `<repo>/.toksi/projectmem/`)
   - `mcp.rs` (all 16 `projectmem` MCP tools including `precheck_file`, `list_projects`, `current_project`, `staleness`, `structure`, `heatmap`) + `/projectmem` slash command integration.
2. Support multi-protocol interfaces (`MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`) and `DirectWireBus`.
3. Expose the native CLI command `toksi mem ...`.
4. Ensure zero Python runtime dependency, zero `unsafe`, 0 compiler warnings, and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Schema & Events Model** (`src/projectmem/schema.rs`): Strict Serde mapping of all event variants (Issue, Attempt, Fix, Decision, Note).
- **Atomic Event Store** (`src/projectmem/store.rs`): Append-only JSONL with `fs2` locking, thread-safe memory writes, and automated markdown derivation (`summary.md`).
- **Deterministic Summary Generator** (`src/projectmem/summary.rs`): Generates human- and LLM-readable summaries without spending LLM tokens.
- **Git Staged Diff Precheck** (`src/projectmem/precheck.rs`): Scans `git diff --cached` for modified files matching known gotchas or prior bug locations.
- **Multi-Protocol MCP Server** (`src/projectmem/mcp.rs`): Serves `projectmem` tools over MCP stdio/SSE/HTTP and A2A wire protocols.

### 2. Disciplinary Standards & Invariants
- All file paths in event locations (e.g. `location: "file:line"`) must use forward slashes `/`.
- Events store must never leak credentials: run `redaction.rs` over all input summaries and notes.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- projectmem` passes 100%.
- `toksi mem log-issue "test"` appends to `.toksi/projectmem/events.jsonl` cleanly with file locking.
