# projectmem - neo

_Last updated: 2026-11-08_

## Project purpose
--------------------------------------------------------------------------------
Enterprise-grade native Rust tooling, token-optimization, multi-protocol agent wire integration, and headroom gateway pipeline for AI coding assistants. Transparently sits between 17 coding agents and 39+ upstream LLM providers to cut token spend by 30-70%, enforce latency/cost/concurrency quotas, and provide zero-latency memory, AST code intelligence, and real-time telemetry across dedicated isolated ports.

## Recent issues
- [OPEN] #0034 Bypass non-LLM authentication and ConnectRPC management endpoints directly without intercepting headers or proxying, capturing only LLM processing in and out [crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)] (open)
- [OPEN] #0033 Transparent TCP tunnel or non-LLM passthrough for Antigravity auth/eligibility, proxy only LLM generation requests [crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)] (open)
- [DONE] #0035 Phase 08: Purge external runtime downloaders and rewire agent harnesses to in-tree modules [crates/toksi/src/util/] (fixed)
- [DONE] #0032 Antigravity eligibility check returns 404 not found Google upstream service endpoint not found [crates/toksi/src/headroom/proxy.rs:672 (serve_headroom_connection)] (fixed)
- [DONE] #0031 Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing [crates/toksi/src/headroom/proxy.rs:625 (serve_headroom_connection)] (fixed)
- [DONE] #0030 Antigravity auth and eligibility check fails with proto syntax error line 1:1 invalid value due to ConnectRPC route discrimination, wrong upstream routing, and HTTP chunking of protobuf responses [crates/toksi/src/headroom/proxy.rs:619 (serve_headroom_connection)] (fixed)
- [DONE] #0029 RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json [crates/toksi/src/agents/antigravity.rs:210 (antigravity_hooks_file)] (fixed)
- [DONE] #0028 VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream [crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)] (fixed)
- [DONE] #0027 toksi proxy reports model unavailable in VS Code antigravity extension while terminal works [crates/toksi/src/main.rs:252 (run)] (fixed)
- [DONE] #0026 VS Code MCP spawn and proxy wiring parity for Copilot and Antigravity streaming [crates/toksi/src/agents/copilot.rs:2478 (detect_copilot_proxy)] (fixed)
- [DONE] #0025 HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400 [crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)] (fixed)
- [DONE] #0024 Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth [crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)] (fixed)
- ...and 23 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...62 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
- Phase 03 refresh: verified full Context-Mode parity in pure Rust with strict CPU/memory limits, in-memory RTK pipe, SQLite FTS5 BM25 search, SSRF per-hop enforcement, and in-process ctx_* MCP suite [crates/toksi/src/contextmode/mod.rs:1]
- Phase 04 rerun: hardened ICM FTS5 BM25 token sanitization with operator stripping, prioritized ~/.toksi/memory.db, added memory_handoff_accept MCP alias, and added extractive leading_prose fallback for tiered progressive disclosure [crates/toksi/src/icm/store.rs:593 (sanitize_fts5_query)]
- Phase 05: CodeGraph Pure Rust Engine executed and verified with upstream inspect-vault parity, dual-path .toksi/codegraph/symbols.db and .codegraph/symbols.db support, petgraph blast-radius, and file-suffix coordinate resolution [crates/toksi/src/codegraph/mod.rs:1]
- Phase 06 rerun: In-Tree Rules & Skills Integration verified with embedded Karpathy principles, Caveman ASD-STE100, and Ponytail build discipline compiled via include_str! and zero warnings [crates/toksi/src/rules/mod.rs:1]
- Phase 07 rerun: Headroom Deep Integration & Gateway Catalog verified with 39+ in-tree provider catalog, 8-class failure cascade, dynamic load balancer, task complexity router, virtual key governance, dual-tier semantic cache (SHA-256 + Cosine Similarity >= 0.92), AST code/whitespace pruning... [crates/toksi/src/headroom/mod.rs:1]
- Phase 07: Headroom Deep Integration & Gateway Catalog verified. Dropped headroom-py; implemented core token compression (log deduplication with byte-for-byte error/fatal/panic survival, AST code pruner, JSON minifier, dynamic MCP schema pruner, 1024-token prompt cache alignment, adaptive memory... [crates/toksi/src/headroom/mod.rs:1]
- Phase 10 executed: non-destructive environment injection via Windows HKCU\Environment registry direct Win32 APIs, broadcast WM_SETTINGCHANGE via SendMessageTimeoutW, Unix session env via systemd/launchctl/environment.d, and RAII TeardownGuard with process signal handlers (SIGINT, SIGTERM... [crates/toksi/src/env_injector/mod.rs:1]
- Phase 10: Non-destructive user environment management verified across Windows (direct HKCU\Environment registry + WM_SETTINGCHANGE SendMessageTimeoutW broadcast) and Unix (systemd/launchctl/environment.d). Connected to toksi daemon start and graceful teardown guard. Explicitly unsets... [crates/toksi/src/env_injector/mod.rs:1]
- Phase 11: Agent Doctor & Self-Healing completed. Upgraded toksi doctor and toksi doctor --fix to audit all 17 agents, purge broken legacy MCP runners across TOML/YAML/JSON, sync in-tree rules, and report environment and process proxies.
- Phase 12: TUI, Prometheus Telemetry & Web Dashboard refreshed with upstream bifrost dropped-request metric isolation, 7-tab interactive TUI with Governance & Quotas, and embedded SPA dashboard [crates/toksi/src/telemetry/mod.rs:1]
- Refactor: Refactor and consolidate documentation and architecture for Toksi v3.4 [crates/toksi/CONSOLIDATION_PLAN.md]
- Refactor: Refactor context mode components for improved readability and functionality [crates/toksi/src/contextmode/auto_memory.rs]
- Refactor: Refactor codegraph hook commands and improve database path resolution [crates/toksi/src/agents/antigravity.rs]
- Refactor: refactor: streamline tool input handling and enhance trace command logging [crates/toksi/src/commands/rtk_hook.rs]
- Refactor: Refactor context mode CLI to support 'execute' command and enhance test coverage [crates/toksi/src/contextmode/cli.rs]

## Notes
- New feature: feat(contextmode): harden SSRF guard, add ctx_batch/ctx_upgrade tools, and update tests [crates/toksi/src/contextmode/fetch.rs]
- New feature: feat(aimemory,icm): add handoff alias, fallback paths, and robust fts5 sanitization" -m "- aimemory: support .toksi/ai-m [crates/toksi/src/aimemory/mod.rs]
- New feature: feat(codegraph): support .toksi storage path, fallback spawn, and suffix resolution" -m "- codegraph: add resolve_db_pat [crates/toksi/src/codegraph/graph.rs]
- New feature: feat(env_injector): add launchctl support, unix signal handlers, and registry teardown guards [crates/toksi/src/env_injector/mod.rs]
- New feature: feat(doctor): add rules audit/sync, agent instruction probes, and multi-format MCP resolution [crates/toksi/src/agents/cursor.rs]
- New feature: feat(toksi): Enhance TUI with Governance Management and Metrics [crates/toksi/src/commands/tui.rs]
- New feature: feat(rtk): add JSON and log filtering capabilities, enhance command normalization [crates/toksi/src/commands/rtk.rs]
- New feature: Add agent instructions and consolidate phase prompts in documentation [AGENTS.md]
- New feature: feat: add ICM hook functionality and normalize path handling across modules [crates/toksi/src/aimemory/handoff.rs]
- New feature: Add new skills: Ponytail and Principles; enhance Caveman integration [crates/toksi/src/rules/embedded/caveman.rs]

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
