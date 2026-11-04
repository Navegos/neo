# Phase 02: ProjectMem Pure Rust Port

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, .projectmem/plan.md, and .gemini/antigravity/brain/PHASE_02_PROJECTMEM_PURE_RUST.md first to understand the global master architecture and cross-tool coupling.

Execute Phase 02: ProjectMem Pure Rust Port.
Source: neo/inspect-vault/projectmem (~v0.3.4, Python)
Target: crates/toksi/src/projectmem/

Requirements:
1. Full port of ProjectMem from Python to pure safe Rust in crates/toksi/src/projectmem/:
   - schema.rs (Event, Issue, Attempt, Fix, Decision, Note)
   - store.rs (atomic append to .projectmem/events.jsonl, Windows LockFileEx and Unix fcntl locking, atomic markdown updates)
   - summary.rs (deterministic zero-LLM summary generator from README, active issues, fixes, decision DAG, and gotchas)
   - precheck.rs (git staged diff scanner vs gotchas and historical failure locations)
   - redaction.rs (zero-allocation regex secret/token sanitizer)
   - heatmap.rs (failure clustering & ROI metrics)
   - mcp.rs (in-process MCP handlers: log_issue, record_attempt, record_fix, add_decision, add_note)
2. Connect to crates/toksi/src/rtk/ so failed test/build commands auto-populate failure traces in record_attempt.
3. Wire into `toksi mem <init|log|attempt|fix|decision|note|summary|precheck|stats>` CLI.
4. Drop all Python, pip, and uvx dependencies.
5. Ensure 100% functional parity with Python projectmem, zero missing code, 0 compiler warnings, and passing unit tests.
```

