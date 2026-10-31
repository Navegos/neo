# projectmem - neo

_Last updated: 2026-10-31_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- [DONE] #0017 Phase 11: Agent Doctor & Self-Healing Parity across all 17 agents [crates/tokless/src/commands/doctor.rs:31] -> Phase 11 verified: all 17 agent configurations audited, in-tree rules verified, legacy runners purged, environment status checked; 331 tests passing with 0 warnings (fixed)
- [DONE] #0016 Phase 9: Legacy healing and binary un-hijack [crates/tokless/CONSOLIDATION_PLAN.md:404] -> Phase 9 complete: purged binary hijacking, automated self-healing engine restores .real binaries and deletes shims/state files, 232 unit tests and 5 phase 9 verification tests passing with 0 warnings (fixed)
- [DONE] #0015 Execute Phase 7: Headroom Deep Integration (compression, cache alignment, adaptive memory governor, BYOK proxy unification) [crates/tokless/src/headroom/] -> Phase 7 complete: Full native headroom deep integration with lossless log compression, AST code pruner, JSON minifier, 1024-token prompt cache alignment, adaptive memory governor (L0/L1/L2), and BYOK streaming proxy. (fixed)
- [DONE] #0014 Phase 6: Port and deep integrate headroom into crates/tokless/src/headroom [crates/tokless/src/headroom] -> Phase 6 complete: Ported and deeply integrated headroom into pure Rust in crates/tokless/src/headroom with native reverse proxy, log deduplication, AST code compression, JSON pruner, 1024-token prompt cache alignment, dynamic memory governor, dropped Python headroom-py, zero compiler warnings, 172 passing tests [crates/tokless/src/headroom] (fixed)
- [DONE] #0013 ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash [crates/tokless/src/icm/store.rs:77] -> Add UNIQUE constraint directly on summary_hash column in memories table (fixed)
- [DONE] #0012 Commands doctor/init/proxy parity gaps: doctor missing per-agent tool/proxy/instruction rows and proxy port checks; init missing excluded-tool unwire and persisted selection load [crates/tokless/src/commands/doctor.rs] -> Implemented doctor per-agent tool rows and Ports overview, init excluded tools unwire and selection persistence, proxy lifecycle lock and rollback transaction (fixed)
- [DONE] #0011 OpenCode v1/v2 proxy & BYOK parity missing: ConfigureOpenCodeProxyV2, RemoveOpenCodeProxyV2, OpenCodeProxyV2Wired, retrieve tool stash, and proxy_agent opencode2 spec [crates/tokless/src/agents/opencode.rs] -> Verified and integrated OpenCode v1 and v2 proxy and BYOK parity with complete specs and stash handling (fixed)
- [DONE] #0010 MCP adapters and opencode v2 regressions: opencode2 MCP writes v1 root shape instead of mcp.servers, codex MCP block parser missing, cursor MCP permissions missing [crates/tokless/src/agents/mcp_adapters.rs] -> Verified and wired MCP adapters for opencode2 with mcp.servers, codex TOML block parser, and cursor MCP permissions (fixed)
- [DONE] #0009 ProjectMem tool missing pjm context + intent sections + global gotchas, kilo/opencode2 extensions, before_agent_start/transform hooks, and git worktree post-commit hook [crates/tokless/src/tools/projectmem.rs] -> Projectmem tool equipped with session push, kilo/opencode2 extensions, and rule files refresh (fixed)
- [DONE] #0008 CodeGraph tool missing antigravity/droid/pi hooks and package purge, status --json health check, codegraph init vs index --quiet, and opencode autoindexjs unwire [crates/tokless/src/tools/codegraph.rs] -> CodeGraph tool equipped with hooks for antigravity, droid, pi, and opencode/opencode2 autoindex unwiring (fixed)
- ...and 7 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...26 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
- Phase 8: Purged runtime downloaders (npminstall, nodewin, nodelinux, uv, projectmem, gitwin); rewired agent and tool harnesses to native in-tree Rust modules; enforced system detection policy via which::which; 0 compiler warnings, 224 tests passing [crates/tokless/src/util/mod.rs:1]
- Phase 9: Implemented interactive multi-tab TUI in crates/tokless/src/tui using ratatui and crossterm with 6 views, 60fps rendering, live waterfall, agent wire manager, memory explorer, codegraph inspector, and headroom gauges [crates/tokless/src/tui/mod.rs]
- Phase 10: Implement zero-dependency embedded Telemetry & Control Web Dashboard with pure Rust HTTP/SSE server and SPA embedded via include_str! [crates/tokless/src/dashboard/mod.rs:1]
- Phase 11: End-to-End System Verification & Benchmarks across all 17 agent harnesses, unified CLI, in-process MCP multiplexer, and streaming compression [crates/tokless/tests/phase11_e2e_verification.rs:1]
- Phase 1: Verified and hardened RTK filter engine integration with 100+ command profiles, zero-alloc ANSI stripper, cross-tool adapters (contextmode, projectmem, headroom), and phase1 integration test suite (255 tests passing, 0 warnings) [crates/tokless/src/rtk/]
- Phase 2: ProjectMem pure Rust port completed with schema, store, deterministic summary, precheck, redaction, heatmap, global memory and stack detection, in-process MCP server, and RTK auto-trace extraction; 274 tests passing with 0 warnings
- Phase 3: ContextMode pure Rust port verified with isolated sandbox runner, in-memory RTK pipe, SQLite FTS5 BM25 chunker, async fetcher, in-process ctx_* MCP server, CLI, and integration test suite; 325 tests passing with 0 warnings [crates/tokless/src/contextmode/mod.rs:1]
- Phase 4: ICM & ai-memory integration complete with embedded SQLite FTS5 BM25 store, sub-10ms recall latency, claim-once multi-agent handoffs across 17+ agents, tiered progressive disclosure packer, projectmem sync, CLI commands (tokless icm, tokless handoff, tokless mem sync), and in-process MCP server dispatch; 281 tests passing with 0 warnings [crates/tokless/src/icm/mod.rs:1]
- Phase 5: CodeGraph pure Rust engine integrated with multi-language Tree-sitter parsers, petgraph blast-radius traversal, SQLite symbols cache, in-process MCP server, and projectmem coordinate resolution; 281 tests passing with 0 warnings [crates/tokless/src/codegraph/mod.rs]
- Phase 6: In-Tree Rules & Skills Integration complete with embedded Karpathy principles, Caveman ASD-STE100, and Ponytail discipline; zero clone links, config patchers for CLAUDE.md/AGENTS.md/.cursor, CLI subcommands, 310 tests passing with 0 warnings [crates/tokless/src/rules/mod.rs]
- Phase 7: Headroom Deep Integration verified with core compression (log, code, json), 1024-token cache alignment, adaptive memory governor (L0/L1/L2), and BYOK streaming proxy; 0 warnings, all tests passing [crates/tokless/src/headroom/mod.rs]
- Phase 8: Complete runtime downloader purge from rtk.rs; enforce native in-tree compilation and system PATH detection via which; verified with phase8 test suite (5/5 passing, 0 compiler warnings) [crates/tokless/src/tools/rtk.rs:109]
- Phase 9: Purged destructive binary hijacking from antigravity, grok, and copilot; implemented self-healing engine in crates/tokless/src/env_injector/heal.rs to restore .real/.real.exe binaries and purge shims/state files across tokless init, tokless doctor --fix, and tokless daemon start
- Phase 10: Non-destructive environment injection via native Windows HKCU\\Environment APIs and broadcast WM_SETTINGCHANGE, Unix environment.d/systemd, teardown guard on daemon stop and Ctrl+C [crates/tokless/src/env_injector/]
- Phase 12: Interactive 6-tab TUI in crossterm/ratatui with Live Monitor, Wire Manager, Memory Explorer, CodeGraph Visualizer, Headroom BYOK & Cache Inspector, Diagnostic Logs; zero-dependency Telemetry Web Dashboard on :9099 with embedded SPA and SSE [crates/tokless/src/tui/]

## Notes
- New feature: feat: Enhance project memory management and global gotchas functionality [crates/tokless/src/main.rs]
- New feature: feat(tests): add phase 3 context mode verification tests and enhance project summaries [crates/tokless/tests/phase3_contextmode_verification.rs]
- New feature: feat: Add Instant Context Memory (ICM) CLI and integrate with existing commands [crates/tokless/src/aimemory/cli.rs]
- New feature: feat: Enhance codegraph functionality with health checks and multi-language parsing tests [crates/tokless/src/tools/codegraph.rs]
- New feature: feat: Implement in-tree rules and skills management with CLI integration and verification tests [crates/tokless/src/main.rs]
- New feature: feat: Update project summary and enhance verification tests for Phase 7 headroom integration [crates/tokless/tests/phase3_contextmode_verification.rs]
- New feature: feat: Update project summary, enhance CLI test issue tracking, and add Phase 8 downloader purge verification tests [crates/tokless/src/tools/rtk.rs]
- New feature: feat: implement self-healing for legacy hijacked binaries and shims [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(env_injector): implement non-destructive environment injection and teardown (Phase 10) [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(tokless): implement Phase 11 agent doctor and self-healing parity [crates/tokless/src/agents/mcp_adapters.rs]

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
