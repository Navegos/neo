# projectmem - neo

_Last updated: 2026-11-02_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- [DONE] #0025 HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400 [crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)] -> Add Transfer-Encoding chunked decoding loop to read_chunked_body, stripping chunk size headers (e.g. 10103\r\n) and reassembling valid JSON body with Content-Length (fixed)
- [DONE] #0024 Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth [crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)] -> Headroom proxy detects non-LLM requests, bypasses prompt modifications, preserves query strings, and routes Cloud Code endpoints to https://cloudcode-pa.googleapis.com (fixed)
- [DONE] #0023 Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)] -> Fixed Antigravity & Copilot Headroom proxy routing and status detection across dashboard, TUI, and CLI (fixed)
  - Partial attempt: Check Windows registry env for Antigravity, enable Copilot CLI provider config wired check, and update TUI/Dashboard to reflect proxy routing state [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)]
- [DONE] #0022 Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)] -> Unified embedded tool resolution across Windows, Linux, and Mac: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom (fixed)
  - Partial attempt: Update doctor_stats, versions, doctor_probe, projectmem, and rtk tool to resolve in-tree toksi binary for embedded tools [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)]
- [DONE] #0021 Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk [crates/toksi/src/commands/rtk_hook.rs:52 (rtk_rewrite)] -> Purged external rtk binary fallback from rtk_rewrite; unsupported commands stay unmutated, supported commands rewrite to toksi rtk (fixed)
  - Partial attempt: Remove legacy external rtk binary fallback in rtk_rewrite and support toksi rtk prefix [crates/toksi/src/commands/rtk_hook.rs:43 (rtk_rewrite)]
- [DONE] #0020 Headroom proxy startup fails identity verification when routing disabled [crates/toksi/src/headroom/proxy.rs:498 (run_proxy_foreground)] -> Prevent premature exit in run_proxy_foreground by removing routing_enabled guard on serve mode, enable routing preference on start_proxy, and improve Windows identity verification fallback/timeouts (fixed)
- [DONE] #0019 Multiple issues in toksi: proxy startup fail, TUI hangs, agent wiring, dashboard telemetry [crates/toksi] -> Fixed proxy serve args and parser, decoupled TUI probe thread, unified antigravity 4-directory topology, purged tokless MCP runners, wired GEMINI.md in-tree rules [crates/toksi] (fixed)
- [DONE] #0018 Phase 12: Implement Native Prometheus Telemetry Subsystem, cost tracking, latency breakdown, and /metrics exposition [crates/tokless/src/telemetry/mod.rs:1] -> Phase 12 complete: Prometheus metrics subsystem, cost tracking, latency breakdown, /metrics endpoint, TUI, and embedded Web Dashboard verified (5 tests passing, 0 warnings) [crates/tokless/src/telemetry/mod.rs:1] (fixed)
- [DONE] #0017 Phase 11: Agent Doctor & Self-Healing Parity across all 17 agents [crates/tokless/src/commands/doctor.rs:31] -> Phase 11 verified: all 17 agent configurations audited, in-tree rules verified, legacy runners purged, environment status checked; 331 tests passing with 0 warnings (fixed)
- [DONE] #0016 Phase 9: Legacy healing and binary un-hijack [crates/tokless/CONSOLIDATION_PLAN.md:404] -> Phase 9 complete: purged binary hijacking, automated self-healing engine restores .real binaries and deletes shims/state files, 232 unit tests and 5 phase 9 verification tests passing with 0 warnings (fixed)
- ...and 15 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...32 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
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
- Refactor: Refactor proxy handling and telemetry integration [crates/toksi/src/agents/antigravity.rs]

## Notes
- New feature: feat: Update project summary, enhance CLI test issue tracking, and add Phase 8 downloader purge verification tests [crates/tokless/src/tools/rtk.rs]
- New feature: feat: implement self-healing for legacy hijacked binaries and shims [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(env_injector): implement non-destructive environment injection and teardown (Phase 10) [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(tokless): implement Phase 11 agent doctor and self-healing parity [crates/tokless/src/agents/mcp_adapters.rs]
- New feature: feat(tokless): implement Phase 12 TUI and telemetry web dashboard verification [crates/tokless/src/dashboard/api.rs]
- New feature: feat(tokless): implement Phase 13 end-to-end system verification and benchmarks [crates/tokless/tests/phase13_e2e_verification.rs]
- New feature: feat: update project documentation for Phase 13 end-to-end system verification and benchmarks [crates/tokless/CONSOLIDATION_PLAN.md]
- New feature: feat(tokless): implement Phase 7 headroom deep integration and native enterprise AI gateway [crates/tokless/src/gateway/agent_cli/claude_code.rs]
- New feature: feat(tokless): implement Phase 12 native Prometheus telemetry subsystem and metrics endpoint" -m "- Add in-tree model ca [crates/tokless/src/dashboard/mod.rs]
- New feature: feat: enhance proxy detection and routing for Antigravity and Copilot, improve query handling in BYOK gateway [crates/toksi/src/agents/antigravity.rs]

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
