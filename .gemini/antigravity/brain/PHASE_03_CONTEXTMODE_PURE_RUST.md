# Phase 03: Context-Mode Pure Rust Port

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_03_CONTEXTMODE_PURE_RUST.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/context-mode as updated please rerun

Execute Phase 03: Context-Mode Pure Rust Port.
Source: neo/inspect-vault/context-mode (TypeScript)
Target: crates/toksi/src/contextmode/

Requirements:
1. Full port of Context-Mode from TypeScript to pure Rust in crates/toksi/src/contextmode/:
   - sandbox.rs (isolated subprocess runner with strict CPU timeout and memory bounds)
   - filter.rs (output slicing, head/tail truncation, 98% context reduction)
   - fts.rs (SQLite FTS5 document chunker & BM25 search)
   - fetch.rs (async reqwest HTTP fetcher with clean markdown extraction)
   - mcp.rs (in-process ctx_* MCP tools: ctx_execute, ctx_execute_file, ctx_batch_execute, ctx_index, ctx_search, ctx_fetch_and_index)
2. Pipe all sandbox execution stdout directly through crates/toksi/src/rtk/ filters in-memory.
3. Drop Node.js and TypeScript runtime dependencies completely.
4. Wire into `toksi ctx <eval|search|index>` CLI.
5. Ensure 100% functional parity, zero missing code, 0 compiler warnings, and passing unit tests.
```

