# projectmem - neo

_Last updated: 2026-10-30_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- [DONE] #0015 Execute Phase 7: Headroom Deep Integration (compression, cache alignment, adaptive memory governor, BYOK proxy unification) [crates/tokless/src/headroom/] -> Phase 7 complete: Full native headroom deep integration with lossless log compression, AST code pruner, JSON minifier, 1024-token prompt cache alignment, adaptive memory governor (L0/L1/L2), and BYOK streaming proxy. (fixed)
- [DONE] #0014 Phase 6: Port and deep integrate headroom into crates/tokless/src/headroom [crates/tokless/src/headroom] -> Phase 6 complete: Ported and deeply integrated headroom into pure Rust in crates/tokless/src/headroom with native reverse proxy, log deduplication, AST code compression, JSON pruner, 1024-token prompt cache alignment, dynamic memory governor, dropped Python headroom-py, zero compiler warnings, 172 passing tests [crates/tokless/src/headroom] (fixed)
- [DONE] #0013 ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash [crates/tokless/src/icm/store.rs:77] -> Add UNIQUE constraint directly on summary_hash column in memories table (fixed)
- [DONE] #0012 Commands doctor/init/proxy parity gaps: doctor missing per-agent tool/proxy/instruction rows and proxy port checks; init missing excluded-tool unwire and persisted selection load [crates/tokless/src/commands/doctor.rs] -> Implemented doctor per-agent tool rows and Ports overview, init excluded tools unwire and selection persistence, proxy lifecycle lock and rollback transaction (fixed)
- [DONE] #0011 OpenCode v1/v2 proxy & BYOK parity missing: ConfigureOpenCodeProxyV2, RemoveOpenCodeProxyV2, OpenCodeProxyV2Wired, retrieve tool stash, and proxy_agent opencode2 spec [crates/tokless/src/agents/opencode.rs] -> Verified and integrated OpenCode v1 and v2 proxy and BYOK parity with complete specs and stash handling (fixed)
- [DONE] #0010 MCP adapters and opencode v2 regressions: opencode2 MCP writes v1 root shape instead of mcp.servers, codex MCP block parser missing, cursor MCP permissions missing [crates/tokless/src/agents/mcp_adapters.rs] -> Verified and wired MCP adapters for opencode2 with mcp.servers, codex TOML block parser, and cursor MCP permissions (fixed)
- [DONE] #0009 ProjectMem tool missing pjm context + intent sections + global gotchas, kilo/opencode2 extensions, before_agent_start/transform hooks, and git worktree post-commit hook [crates/tokless/src/tools/projectmem.rs] -> Projectmem tool equipped with session push, kilo/opencode2 extensions, and rule files refresh (fixed)
- [DONE] #0008 CodeGraph tool missing antigravity/droid/pi hooks and package purge, status --json health check, codegraph init vs index --quiet, and opencode autoindexjs unwire [crates/tokless/src/tools/codegraph.rs] -> CodeGraph tool equipped with hooks for antigravity, droid, pi, and opencode/opencode2 autoindex unwiring (fixed)
- [DONE] #0007 Context-Mode tool missing minNode=22, opencode cache postinstall, codex 7-event & workspace cleanup, and agent-specific hooks for droid/antigravity/copilot/pi/cursor [crates/tokless/src/tools/contextmode.rs] -> Context-mode tool equipped with minNode 22 check, opencode cache postinstall, and codex 7-event/workspace cleanup (fixed)
- [DONE] #0006 Headroom tool verify_for always returns Some(true), missing 7 agents in build_headroom, and missing proxy_readiness_for and lifecycle lock wiring [crates/tokless/src/tools/headroom.rs] -> Headroom tool wired for all 17 agents with proxy_readiness_for check and lifecycle locks (fixed)
- ...and 5 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...12 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
- Executed Phase 1: Full port and embed neo\inspect-vault\rtk into crates/tokless/src/rtk with 100+ filters and zero-allocation ANSI stripper [crates/tokless/src/rtk]
- Phase 2 complete: Ported Python projectmem to pure Rust in crates/tokless/src/projectmem with schema, store locking, redaction, summary generator, precheck, heatmap, in-process MCP server, and tokless mem CLI with zero warnings and 86 passing tests [crates/tokless/src/projectmem]
- Execute Phase 3: Port TypeScript context-mode to pure Rust in crates/tokless/src/contextmode with rusqlite FTS5, sandbox runner, RTK filtering, and in-process MCP server
- Complete Phase 4: Port and embed ICM SQLite FTS5 BM25 store and ai-memory tiered claim-once state machine into tokless with zero warnings and 104 passing tests
- Execute Phase 5: Port and embed codegraph into crates/tokless/src/codegraph with tree-sitter, ignore walker, petgraph AST blast-radius, incremental cache, and in-process MCP server
- Phase 5 complete: Ported codegraph to pure Rust in crates/tokless/src/codegraph with tree-sitter parsers, ignore walker, petgraph AST call-graph/blast-radius, incremental SQLite cache, in-process MCP server, and tokless graph CLI with zero compiler warnings and 135 passing tests
- Phase 6: Ported headroom to native Rust in crates/tokless/src/headroom with native HTTP/SSE proxy, lossless log deduplication, AST code compression, JSON pruner, 1024-token cache alignment, dynamic memory governor, unified BYOK gateway, dropped Python headroom-py, 172 passing tests [crates/tokless/src/headroom]
- Phase 7 complete: Verified all 17 agent harnesses, in-process MCP multiplexer, unified tokless CLI subcommands, proxy streaming compression, and token benchmarks with 241 passed workspace tests and zero compiler warnings [crates/tokless/tests/phase7_system_verification.rs]
- Port ProjectMem to pure safe Rust in crates/tokless/src/projectmem/ with LockFileEx, fcntl, README generator, zero-alloc redaction, RTK trace connector, and tokless mem CLI
- Phase 3 complete: Port Context-Mode to pure Rust in crates/tokless/src/contextmode with rusqlite FTS5 BM25 search, recursive directory indexing, async reqwest fetcher, sandbox subprocess runner with RTK output filtering, and tokless ctx <eval|search|index> CLI [crates/tokless/src/contextmode]
- Phase 4: Embedded ICM SQLite FTS5 BM25 memory store and ai-memory claim-once multi-agent handoff state machine across 17+ agents with progressive L0/L1/L2 disclosure and zero LLM token spend [crates/tokless/src/icm/store.rs:1]
- Phase 5: Ported CodeGraph to pure safe Rust engine in crates/tokless/src/codegraph with embedded Tree-sitter parsers, petgraph blast-radius engine, incremental SQLite cache, in-process MCP server, projectmem coordinate auto-resolution, and tokless graph CLI [crates/tokless/src/codegraph/mod.rs:1]
- Phase 6: In-tree rules and skills integration with compile-time include_str! assets for Karpathy principles, Caveman, and Ponytail, eliminating external clone links [crates/tokless/src/rules/]
- Phase 7: Integrated pure Rust Headroom engine into crates/tokless/src/headroom/ with byte-for-byte critical log preservation, Tree-sitter code compression, structural JSON minification, 1024-token prompt cache alignment, adaptive L0/L1/L2 memory scaling, and BYOK reverse proxy. Dropped Python headroom-py completely.
- Phase 8: Purged runtime downloaders (npminstall, nodewin, nodelinux, uv, projectmem, gitwin); rewired agent and tool harnesses to native in-tree Rust modules; enforced system detection policy via which::which; 0 compiler warnings, 224 tests passing [crates/tokless/src/util/mod.rs:1]

## Notes
- New feature: feat(icm): implement in-memory ICM store with CRUD operations and FTS5 support [crates/tokless/src/aimemory/handoff.rs]
- New feature: feat: Introduce content compression module with unified detection and compression for code, JSON, and logs [crates/tokless/src/headroom/adaptive_mem.rs]
- New feature: feat: add proxy agent specification for 'grok', enhance CLI help text, and implement system verification tests for all a [crates/tokless/src/agents/proxy_agent.rs]
- New feature: feat: update project summaries, add CLI test issues, enhance command tracking, and improve redaction functionality [crates/tokless/src/projectmem/cli.rs]
- New feature: feat: implement directory indexing, enhance CLI commands, and add async fetch capabilities [crates/tokless/src/commands/ctx.rs]
- New feature: feat: enhance ICM with multi-agent handoff capabilities, including offer, claim, resolve, and cancel functionalities; ad [crates/tokless/src/aimemory/handoff.rs]
- New feature: feat: enhance projectmem functionality with auto-resolve location features, improve location parsing, and add new tests [crates/tokless/src/codegraph/mcp.rs]
- Phase 6 complete: In-tree rules & skills integration with embedded markdown assets via include_str! (principles, caveman, ponytail), zero external repo clone links, patch/inject helpers for CLAUDE.md, .cursor/rules/, AGENTS.md, 222 tests passing with 0 warnings [crates/tokless/src/rules/mod.rs:1]
- New feature: Add embedded rules for coding guidelines: Caveman, Ponytail, and Principles [crates/tokless/src/lib.rs]
- New feature: feat: implement Phase 7 headroom deep integration with compression, cache alignment, and adaptive memory governor; add s [crates/tokless/src/headroom/adaptive_mem.rs]

## Key files
- `.editorconfig`
- `.gitattributes`
- `.gitignore`
- `LICENSE`
- `README.md`
- `crates/tokless/src/agents/hooks.rs`
- `Cargo.toml`
- `crates/.gitkeep`
- `crates/tokless/Cargo.toml`
- `crates/tokless/src/agents/mod.rs`
- `crates/tokless/src/agents/proxy_agent.rs`
- `crates/tokless/src/agents/proxy_protocol.rs`
- `crates/tokless/src/commands/codex_perm_hook.rs`
- `crates/archinfo/build.rs`
- `crates/archinfo/.codegraph/.gitignore`
- `crates/archinfo/.gitattributes`
- `crates/archinfo/.github/workflows/release.yml`
- `crates/archinfo/.gitignore`
- `crates/archinfo/Cargo.lock`
- `crates/archinfo/Cargo.toml`

## Open questions
- None logged yet.
