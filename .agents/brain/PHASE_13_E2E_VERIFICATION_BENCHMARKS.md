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
1. Execute full workspace test suite across all consolidated subsystems:
   - RTK command output filters
   - ProjectMem JSONL append, locking, and summary generation
   - Context-Mode sandbox execution and FTS5 search
   - ICM BM25 search and ai-memory handoffs
   - CodeGraph Tree-sitter parsing and petgraph blast radius
   - Headroom lossless compression and dynamic MCP schema pruner
   - Enterprise Gateway 39+ provider routing and failover cascades
   - Non-destructive environment lifecycle and teardown
2. Verify token reduction benchmarks:
   - Prove 30-70% token savings on realistic agent trace payloads.
   - Prove zero latency regression on streaming SSE responses (<10 µs proxy overhead).
3. Ensure 100% test pass rate with 0 compiler warnings.
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
