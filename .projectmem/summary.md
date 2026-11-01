# projectmem - neo

_Last updated: 2026-11-01_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- [DONE] #0022 Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)] -> Unified embedded tool resolution across Windows, Linux, and Mac: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom (fixed)
  - Partial attempt: Update doctor_stats, versions, doctor_probe, projectmem, and rtk tool to resolve in-tree toksi binary for embedded tools [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)]
- [DONE] #0021 Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk [crates/toksi/src/commands/rtk_hook.rs:52 (rtk_rewrite)] -> Purged external rtk binary fallback from rtk_rewrite; unsupported commands stay unmutated, supported commands rewrite to toksi rtk (fixed)
  - Partial attempt: Remove legacy external rtk binary fallback in rtk_rewrite and support toksi rtk prefix [crates/toksi/src/commands/rtk_hook.rs:43 (rtk_rewrite)]
- [DONE] #0020 Headroom proxy startup fails identity verification when routing disabled [crates/toksi/src/headroom/proxy.rs:498 (run_proxy_foreground)] -> Prevent premature exit in run_proxy_foreground by removing routing_enabled guard on serve mode, enable routing preference on start_proxy, and improve Windows identity verification fallback/timeouts (fixed)
- [DONE] #0019 Multiple issues in toksi: proxy startup fail, TUI hangs, agent wiring, dashboard telemetry [crates/toksi] -> Fixed proxy serve args and parser, decoupled TUI probe thread, unified antigravity 4-directory topology, purged tokless MCP runners, wired GEMINI.md in-tree rules [crates/toksi] (fixed)
- [DONE] #0018 Phase 12: Implement Native Prometheus Telemetry Subsystem, cost tracking, latency breakdown, and /metrics exposition [crates/tokless/src/telemetry/mod.rs:1] -> Phase 12 complete: Prometheus metrics subsystem, cost tracking, latency breakdown, /metrics endpoint, TUI, and embedded Web Dashboard verified (5 tests passing, 0 warnings) [crates/tokless/src/telemetry/mod.rs:1] (fixed)
- [DONE] #0017 Phase 11: Agent Doctor & Self-Healing Parity across all 17 agents [crates/tokless/src/commands/doctor.rs:31] -> Phase 11 verified: all 17 agent configurations audited, in-tree rules verified, legacy runners purged, environment status checked; 331 tests passing with 0 warnings (fixed)
- [DONE] #0016 Phase 9: Legacy healing and binary un-hijack [crates/tokless/CONSOLIDATION_PLAN.md:404] -> Phase 9 complete: purged binary hijacking, automated self-healing engine restores .real binaries and deletes shims/state files, 232 unit tests and 5 phase 9 verification tests passing with 0 warnings (fixed)
- [DONE] #0015 Execute Phase 7: Headroom Deep Integration (compression, cache alignment, adaptive memory governor, BYOK proxy unification) [crates/tokless/src/headroom/] -> Phase 7 complete: Full native headroom deep integration with lossless log compression, AST code pruner, JSON minifier, 1024-token prompt cache alignment, adaptive memory governor (L0/L1/L2), and BYOK streaming proxy. (fixed)
- [DONE] #0014 Phase 6: Port and deep integrate headroom into crates/tokless/src/headroom [crates/tokless/src/headroom] -> Phase 6 complete: Ported and deeply integrated headroom into pure Rust in crates/tokless/src/headroom with native reverse proxy, log deduplication, AST code compression, JSON pruner, 1024-token prompt cache alignment, dynamic memory governor, dropped Python headroom-py, zero compiler warnings, 172 passing tests [crates/tokless/src/headroom] (fixed)
- [DONE] #0013 ON CONFLICT clause does not match any PRIMARY KEY or UNIQUE constraint on memories.summary_hash [crates/tokless/src/icm/store.rs:77] -> Add UNIQUE constraint directly on summary_hash column in memories table (fixed)
- ...and 12 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...31 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
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
- Phase 13: End-to-End System Verification & Benchmarks verified across all 17 agent harnesses, unified CLI, proxy compression, non-destructive env injection, zero external runtime deps, and benchmarks (395 tests passing, 0 warnings) [crates/tokless/tests/phase13_e2e_verification.rs]
- Phase 7: Implemented Native Enterprise AI Gateway and Headroom deep integration with 36 provider clients, local session reuse bridges, dynamic failover cascade, multi-key pools, load balancer, task complexity router, virtual key governance, dual-tier semantic cache, and dynamic MCP schema pruner; 407 workspace tests passing with 0 warnings [crates/tokless/src/gateway/]
- Refactor: Rename crate tokless to toksi across workspace, packages, binaries, APIs, metrics, and agent harnesses
- Phase 11: Resolved duplicate definitions from tokless-to-toksi rename, verified Phase 11 agent doctor & self-healing parity; 241 unit tests and 8 verification tests passing with 0 warnings [crates/toksi/src/]
- Refactor: refactor(toksi): rename tokless to toksi and enforce Phase 11 self-healing parity [crates/nbuild/src/cli.rs]

## Notes
- New feature: feat: Update project summary and enhance verification tests for Phase 7 headroom integration [crates/tokless/tests/phase3_contextmode_verification.rs]
- New feature: feat: Update project summary, enhance CLI test issue tracking, and add Phase 8 downloader purge verification tests [crates/tokless/src/tools/rtk.rs]
- New feature: feat: implement self-healing for legacy hijacked binaries and shims [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(env_injector): implement non-destructive environment injection and teardown (Phase 10) [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(tokless): implement Phase 11 agent doctor and self-healing parity [crates/tokless/src/agents/mcp_adapters.rs]
- New feature: feat(tokless): implement Phase 12 TUI and telemetry web dashboard verification [crates/tokless/src/dashboard/api.rs]
- New feature: feat(tokless): implement Phase 13 end-to-end system verification and benchmarks [crates/tokless/tests/phase13_e2e_verification.rs]
- New feature: feat: update project documentation for Phase 13 end-to-end system verification and benchmarks [crates/tokless/CONSOLIDATION_PLAN.md]
- New feature: feat(tokless): implement Phase 7 headroom deep integration and native enterprise AI gateway [crates/tokless/src/gateway/agent_cli/claude_code.rs]
- New feature: feat(tokless): implement Phase 12 native Prometheus telemetry subsystem and metrics endpoint" -m "- Add in-tree model ca [crates/tokless/src/dashboard/mod.rs]

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
