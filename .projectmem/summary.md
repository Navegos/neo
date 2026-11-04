# projectmem - neo

_Last updated: 2026-11-03_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- [DONE] #0029 RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json [crates/toksi/src/agents/antigravity.rs:210 (antigravity_hooks_file)] (fixed)
- [DONE] #0028 VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream [crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)] (fixed)
- [DONE] #0027 toksi proxy reports model unavailable in VS Code antigravity extension while terminal works [crates/toksi/src/main.rs:252 (run)] (fixed)
- [DONE] #0026 VS Code MCP spawn and proxy wiring parity for Copilot and Antigravity streaming [crates/toksi/src/agents/copilot.rs:2478 (detect_copilot_proxy)] (fixed)
- [DONE] #0025 HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400 [crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)] (fixed)
- [DONE] #0024 Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth [crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)] (fixed)
- [DONE] #0023 Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)] (fixed)
- [DONE] #0022 Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)] (fixed)
- [DONE] #0021 Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk [crates/toksi/src/commands/rtk_hook.rs:52 (rtk_rewrite)] (fixed)
- [DONE] #0020 Headroom proxy startup fails identity verification when routing disabled [crates/toksi/src/headroom/proxy.rs:498 (run_proxy_foreground)] (fixed)
- ...and 19 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...34 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
- Phase 5: CodeGraph pure Rust engine integrated with multi-language Tree-sitter parsers, petgraph blast-radius traversal, SQLite symbols cache, in-process MCP server, and projectmem coordinate resolution; 281 tests passing with 0 warnings [crates/tokless/src/codegraph/mod.rs]
- Phase 6: In-Tree Rules & Skills Integration complete with embedded Karpathy principles, Caveman ASD-STE100, and Ponytail discipline; zero clone links, config patchers for CLAUDE.md/AGENTS.md/.cursor, CLI subcommands, 310 tests passing with 0 warnings [crates/tokless/src/rules/mod.rs]
- Phase 7: Headroom Deep Integration verified with core compression (log, code, json), 1024-token cache alignment, adaptive memory governor (L0/L1/L2), and BYOK streaming proxy; 0 warnings, all tests passing [crates/tokless/src/headroom/mod.rs]
- Phase 8: Complete runtime downloader purge from rtk.rs; enforce native in-tree compilation and system PATH detection via which; verified with phase8 test suite (5/5 passing, 0 compiler warnings) [crates/tokless/src/tools/rtk.rs:109]
- Phase 9: Purged destructive binary hijacking from antigravity, grok, and copilot; implemented self-healing engine in crates/tokless/src/env_injector/heal.rs to restore .real/.real.exe binaries and purge shims/state files across tokless init, tokless doctor --fix, and tokless daemon start
- Phase 10: Non-destructive environment injection via native Windows HKCU\\Environment APIs and broadcast WM_SETTINGCHANGE, Unix environment.d/systemd, teardown guard on daemon stop and Ctrl+C [crates/tokless/src/env_injector/]
- Phase 12: Interactive 6-tab TUI in crossterm/ratatui with Live Monitor, Wire Manager, Memory Explorer, CodeGraph Visualizer, Headroom BYOK & Cache Inspector, Diagnostic Logs; zero-dependency Telemetry Web Dashboard on :9099 with embedded SPA and SSE [crates/tokless/src/tui/]
- Phase 13: End-to-End System Verification & Benchmarks verified across all 17 agent harnesses, unified CLI, proxy compression, non-destructive env injection, zero external runtime deps, and benchmarks (395 tests passing, 0 warnings) [crates/tokless/tests/phase13_e2e_verification.rs]
- Phase 7: Implemented Native Enterprise AI Gateway and Headroom deep integration with 36 provider clients, local session reuse bridges, dynamic failover cascade, multi-key pools, load balancer, task complexity router, virtual key governance, dual-tier semantic cache, and dynamic MCP schema... [crates/tokless/src/gateway/]
- Refactor: Rename crate tokless to toksi across workspace, packages, binaries, APIs, metrics, and agent harnesses
- Phase 11: Resolved duplicate definitions from tokless-to-toksi rename, verified Phase 11 agent doctor & self-healing parity; 241 unit tests and 8 verification tests passing with 0 warnings [crates/toksi/src/]
- Refactor: refactor(toksi): rename tokless to toksi and enforce Phase 11 self-healing parity [crates/nbuild/src/cli.rs]
- Refactor: Refactor proxy handling and telemetry integration [crates/toksi/src/agents/antigravity.rs]
- Use only toksi paths, purge tokless aliases from discovery and adapters, keep tokless checks strictly for self-healing and data migration
- Refactor: Refactor headroom proxy and utility functions; enhance binary resolution and add CORS support [crates/toksi/src/agents/antigravity.rs]

## Notes
- New feature: feat(env_injector): implement non-destructive environment injection and teardown (Phase 10) [crates/tokless/src/agents/antigravity.rs]
- New feature: feat(tokless): implement Phase 11 agent doctor and self-healing parity [crates/tokless/src/agents/mcp_adapters.rs]
- New feature: feat(tokless): implement Phase 12 TUI and telemetry web dashboard verification [crates/tokless/src/dashboard/api.rs]
- New feature: feat(tokless): implement Phase 13 end-to-end system verification and benchmarks [crates/tokless/tests/phase13_e2e_verification.rs]
- New feature: feat: update project documentation for Phase 13 end-to-end system verification and benchmarks [crates/tokless/CONSOLIDATION_PLAN.md]
- New feature: feat(tokless): implement Phase 7 headroom deep integration and native enterprise AI gateway [crates/tokless/src/gateway/agent_cli/claude_code.rs]
- New feature: feat(tokless): implement Phase 12 native Prometheus telemetry subsystem and metrics endpoint" -m "- Add in-tree model ca [crates/tokless/src/dashboard/mod.rs]
- New feature: feat: enhance proxy detection and routing for Antigravity and Copilot, improve query handling in BYOK gateway [crates/toksi/src/agents/antigravity.rs]
- New feature: feat: implement Transfer-Encoding chunked decoding in HTTP request parser and add corresponding tests [crates/toksi/src/headroom/byok_gateway.rs]
- New feature: feat: Add VS Code Antigravity extension connection fixes and RTK hook enhancements [crates/toksi/src/agents/antigravity.rs]

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
