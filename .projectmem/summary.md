# projectmem - neo

_Last updated: 2026-11-13_

## Project purpose
--------------------------------------------------------------------------------
Enterprise-grade native Safe Rust (`#![deny(unsafe_code)]`) token optimization, intelligent multi-port proxying, native coding CLI/swarm runtime (`toksi chat|run|exec|swarm`), multi-protocol agent wire integration (`MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`, `DirectWireBus`), polyglot AST/SCIP code intelligence, and 9-tab telemetry cockpits connecting **18+ registered & 65/90+ ecosystem AI coding assistants** with **22 local engines** and multi-instance `<byok, model, effort>` upstream providers.

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
- Phase 06 rerun: In-Tree Rules & Skills Integration verified with embedded Karpathy principles, Caveman ASD-STE100, and Ponytail build discipline compiled via include_str! and zero warnings [crates/toksi/src/rules/mod.rs:1]
- Phase 07 rerun: Headroom Deep Integration & Gateway Catalog verified with 39+ in-tree provider catalog, 8-class failure cascade, dynamic load balancer, task complexity router, virtual key governance, dual-tier semantic cache (SHA-256 + Cosine Similarity >= 0.92), AST code/whitespace pruning... [crates/toksi/src/headroom/mod.rs:1]
- Phase 07: Headroom Deep Integration & Gateway Catalog verified. Dropped headroom-py; implemented core token compression (log deduplication with byte-for-byte error/fatal/panic survival, AST code pruner, JSON minifier, dynamic MCP schema pruner, 1024-token prompt cache alignment, adaptive memory... [crates/toksi/src/headroom/mod.rs:1]
- Phase 10 executed: non-destructive environment injection via Windows HKCU\Environment registry direct Win32 APIs, broadcast WM_SETTINGCHANGE via SendMessageTimeoutW, Unix session env via systemd/launchctl/environment.d, and RAII TeardownGuard with process signal handlers (SIGINT, SIGTERM... [crates/toksi/src/env_injector/mod.rs:1]
- Phase 10: Non-destructive user environment management verified across Windows (direct HKCU\Environment registry + WM_SETTINGCHANGE SendMessageTimeoutW broadcast) and Unix (systemd/launchctl/environment.d). Connected to toksi daemon start and graceful teardown guard. Explicitly unsets... [crates/toksi/src/env_injector/mod.rs:1]
- Phase 11: Agent Doctor & Self-Healing completed. Upgraded toksi doctor and toksi doctor --fix to audit all 17 agents, purge broken legacy MCP runners across TOML/YAML/JSON, sync in-tree rules, and report environment and process proxies.
- Phase 12: TUI, Prometheus Telemetry & Web Dashboard refreshed with upstream bifrost dropped-request metric isolation, 7-tab interactive TUI with Governance & Quotas, and embedded SPA dashboard [crates/toksi/src/telemetry/mod.rs:1]
- Refactor: Refactor context mode components for improved readability and functionality [crates/toksi/src/contextmode/auto_memory.rs]
- Refactor: Refactor codegraph hook commands and improve database path resolution [crates/toksi/src/agents/antigravity.rs]
- Refactor: refactor: streamline tool input handling and enhance trace command logging [crates/toksi/src/commands/rtk_hook.rs]
- Refactor: Refactor context mode CLI to support 'execute' command and enhance test coverage [crates/toksi/src/contextmode/cli.rs]
- Wave 17 v4.0 Master Architecture: 36 inspect-vault engines, genuine #![deny(unsafe_code)] via crates/toksi-sys and crates/toksi-grammars leaf crates, 41+ Tree-sitter languages, ~/.toksi/config.toml, native coding CLI src/agent_runtime/, DirectWireBus, and 9-tab UI parity [crates/toksi/research/RESEARCHPLAN.md:1228]
- Workspace-only brain prompts inside <repo>/.agents/brain/ (31 files: INDEX_AND_EXECUTION_GUIDE.md plus PHASE_01 through PHASE_17_7), retiring user-home plan paths [.agents/brain/INDEX_AND_EXECUTION_GUIDE.md:1]
- Strict <repo>/.toksi/<tool> repo-only storage for codegraph, projectmem, context, icm, and ai-memory (never ~/.toksi/<tool> or nested crates); walk to .git root with monorepo subproject_id and dynamic .gitignore/.ignore purge [crates/toksi/src/commands/index.rs:1]
- Unified 7-port topology (:8787 Headroom, :8788 Grok OAuth, :8789/:8790 Copilot, :18787 BYOK, :9099 Gateway, :9100 Cockpit), zero paid-tier restrictions (FreeTierDisqualified removed), multi-instance <byok, model, effort> SQLite profiles, and 65/90+ Categories 1-5 + 22 local engines [crates/toksi/CONSOLIDATION_PLAN.md:1]

## Notes
- New feature: feat(dashboard): Enhance dashboard with AI Gateway and Keyring management [crates/toksi/src/dashboard/api.rs]
- New feature: feat: introduce Agent Manager for lifecycle management of agents [crates/toksi/src/agents/manager.rs]
- New feature: Add comprehensive reference for local inference engines with API details [crates/toksi/research/RESEARCHPLAN.txt]
- New feature: feat: add RTK slash command handling and tracking features [crates/toksi/src/commands/rtk.rs]
- New feature: feat: Implement project structure generation and saving [crates/toksi/src/projectmem/cli.rs]
- New feature: feat: Enhance context-mode with slash command handling and policy enforcement [crates/toksi/src/commands/ctx.rs]
- New feature: feat(toksi): implement Phase 05 CodeGraph pure Rust engine and 41+ polyglot extractors [crates/toksi-grammars/build.rs]
- New feature: feat: implement DirectWire communication for CodeGraph [crates/toksi/src/agents/antigravity.rs]
- New feature: feat(rules): add caveman v4 and ponytail modes with selective agent sync [crates/toksi/src/agents/manager.rs]
- New feature: feat(gateway): implement Phase 07 proxy architecture, providers, and headroom compression [crates/toksi/src/commands/daemon.rs]

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
