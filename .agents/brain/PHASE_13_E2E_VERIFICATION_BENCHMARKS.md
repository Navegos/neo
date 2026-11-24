# Phase 13: End-to-End System Verification & Benchmarks

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_13_E2E_VERIFICATION_BENCHMARKS.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

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
   - Native `toksi` Coding CLI/Swarm (`src/agent_runtime/`), `DirectWireBus`, Unified Plugin System (`src/plugins/`, `defenseclaw` + `skill-scanner` fail-closed gate), ADR + Isolated Agent Sandbox (`sbx`/`sandbox-runtime`/3-OS), `CommandGuard` kill-switch, Zero-Plaintext Blind Engine + `src/validator/`, externalized `<skills>` sync, **16** `/...` slash commands (`/byok`..`/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`), and non-destructive env teardown
2. Incremental Test Skip Policy (`.toksi/test-baseline.json`):
   - Compute BLAKE3 digest over `Cargo.lock` + each suite's `watched_paths`; log `[SKIP-UNCHANGED] <suite>` for already-confirmed passing suites whose code paths and dependencies are unmodified (`--all`/`--force` runs everything). Never strip or weaken assertions!
   - Always run `cargo check --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
3. Verify token reduction & latency benchmarks (`>30%` savings, `<10 µs` proxy overhead, `<5ms` slash/TUI latency) across **35-component `<toksi-siagent>`** (**432 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, externalized zero-bloat `<skills>` & `<advisory>`).
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

---

## Wave 18 (v5.2) Synchronization Addendum
- Pair with `PHASE_18_4`: assert zero stale `.projectmem/` pre-read refs, zero legacy `crates/tokless/*` paths in docs, all **43** `.agents/brain/*.md` files present (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_19_6`), **432** mandated vaults (`36` + `259` Wave 18 across Sections `2.1`–`2.10` incl. `opencode`, `cve-*`, `osv-schema`, `skilldoctor`, `sandbox-runtime`, `OpenShell`, `E2B`, `opa`, `cedar`, `spire`, `Guardrails`, `llm-guard`, `temporal`, `restate`, `qdrant`, `cognee`, `graphiti`, `defenseclaw`, `skill-scanner`, `<advisory>`, `<skills>`), **35-component `<toksi-siagent>`**, zero embedded skills/rules inside `crates/toksi/`, 8-port daemon coverage (`:9098` Protocol Bus), **16** slash commands (`/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`), `src/plugins/` Unified Plugin System, `src/validator/`, `src/security/adr.rs`, `src/commands/guard.rs`, `plugins_registry` + zero-plaintext SQLite DDL, and clean incremental updates to `<repo>/.toksi/test-baseline.json`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** Unit verification passes across `35` `.toksi/test-baseline.json` suites, while live-safe isolated container/sandbox full E2E verification moves to `PHASE_19_3` (`docker/Dockerfile.test`, `toksi test --docker|--sandbox`).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
