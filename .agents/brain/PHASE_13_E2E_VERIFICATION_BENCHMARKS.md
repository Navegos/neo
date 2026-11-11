# Phase 13: End-to-End System Verification & Benchmarks

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_13_E2E_VERIFICATION_BENCHMARKS.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 13: End-to-End System Verification & Benchmarks.
Targets:
  - crates/toksi/tests/e2e_tests.rs
  - crates/toksi/tests/benchmarks.rs
  - crates/toksi/src/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Universal Forward-Slash (/) Standard: Use normalize_path() across all test fixtures and assertions.
4. Cross-Platform Verification: Tests must pass uniformly on Windows, Linux, and macOS.
5. High-Performance Benchmarks: Verify sub-10ms memory recall, <10 µs gateway proxy overhead, and 30-70% token savings.

Requirements:
1. Execute full incremental workspace test suite across all consolidated subsystems (`phase1..phase17_7`):
   - RTK command output filters & SQLite `Tracker` gain/tee/discover
   - ProjectMem JSONL append, safe `File::lock()`/`fs2`, repo-only storage, and clickable heatmap
   - Context-Mode sandbox execution, `ctx-hook` policy/journal/compaction, and FTS5 `.ignore` purge
   - ICM BM25 search (`memory.db`), missing 23 MCP tools, and `ai-memory` handoffs
   - CodeGraph 41+ Tree-sitter parsing, monorepo subproject partitioning, `ast-grep`, `scip`, `jcodemunch`, `repomix`
   - Headroom `live_zone` cache preservation, `CcrStore`, `TokenEconomizer`, and mandatory `/byok/` compression pass
   - Enterprise Gateway (`:9099`/`:18787`) 65/90+ ecosystem & 22 local engines, `<byok, model, effort>` routing, zero paid-tier gates
   - Native `toksi` Coding CLI/Swarm (`src/agent_runtime/`), `DirectWireBus`, 10 `/...` slash commands, and non-destructive env teardown
2. Incremental Test Skip Policy (`.toksi/test-baseline.json`):
   - Compute BLAKE3 digest over `Cargo.lock` + each suite's `watched_paths`; log `[SKIP-UNCHANGED] <suite>` for already-confirmed passing suites whose code paths and dependencies are unmodified (`--all`/`--force` runs everything). Never strip or weaken assertions!
   - Always run `cargo check --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
3. Verify token reduction & latency benchmarks (`>30%` savings, `<10 µs` proxy overhead, `<5ms` slash/TUI latency).
4. Ensure 100% relevant test pass rate with 0 compiler warnings.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Integration Test Harness** (`tests/e2e_tests.rs`): Tests multi-tool interactions (e.g. context-mode piping to RTK, projectmem referencing codegraph AST symbols).
- **Headroom Compression Verification**: Asserts that compression engines preserve exact error lines and produce valid JSON bodies.
- **Latency & Throughput Benchmarks** (`tests/benchmarks.rs`): Measures token parsing overhead, cache lookup latency, and FTS5 recall performance.

### 2. Disciplinary Standards & Invariants
- Tests must clean up all temporary directories and mock databases created in `target/tmp`.
- No lingering spawned processes or background threads after test execution.

### 3. Verification Criteria
- `cargo test -p toksi -- --nocapture` passes 100% with 0 failures.
- Token reduction asserts confirm >30% savings on sample agent traces.
- All 10/10 e2e integration suites succeed.
