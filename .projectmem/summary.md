# projectmem - neo

_Last updated: 2026-11-04_

## Project purpose
--------------------------------------------------------------------------------
Enterprise-grade native Rust tooling, token-optimization, and headroom gateway pipeline for AI coding assistants. Transparently sits between 17 coding agents and 39+ upstream LLM providers to cut token spend by 30-70%, enforce latency/cost/concurrency quotas, and provide zero-latency memory, AST code intelligence, and real-time telemetry across dedicated isolated ports.

## Recent issues
- [OPEN] #0034 Bypass non-LLM authentication and ConnectRPC management endpoints directly without intercepting headers or proxying, capturing only LLM processing in and out [crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)] (open)
- [OPEN] #0033 Transparent TCP tunnel or non-LLM passthrough for Antigravity auth/eligibility, proxy only LLM generation requests [crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)] (open)
- [DONE] #0032 Antigravity eligibility check returns 404 not found Google upstream service endpoint not found [crates/toksi/src/headroom/proxy.rs:672 (serve_headroom_connection)] (fixed)
- [DONE] #0031 Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing [crates/toksi/src/headroom/proxy.rs:625 (serve_headroom_connection)] (fixed)
- [DONE] #0030 Antigravity auth and eligibility check fails with proto syntax error line 1:1 invalid value due to ConnectRPC route discrimination, wrong upstream routing, and HTTP chunking of protobuf responses [crates/toksi/src/headroom/proxy.rs:619 (serve_headroom_connection)] (fixed)
- [DONE] #0029 RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json [crates/toksi/src/agents/antigravity.rs:210 (antigravity_hooks_file)] (fixed)
- [DONE] #0028 VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream [crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)] (fixed)
- [DONE] #0027 toksi proxy reports model unavailable in VS Code antigravity extension while terminal works [crates/toksi/src/main.rs:252 (run)] (fixed)
- [DONE] #0026 VS Code MCP spawn and proxy wiring parity for Copilot and Antigravity streaming [crates/toksi/src/agents/copilot.rs:2478 (detect_copilot_proxy)] (fixed)
- [DONE] #0025 HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400 [crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)] (fixed)
- [DONE] #0024 Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth [crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)] (fixed)
- [DONE] #0023 Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)] (fixed)
- ...and 22 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...38 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
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
- Consolidated toksi master architecture v3.0: unified single daemon launch (toksi daemon start), cross-platform forward-slash (/) paths standard for Git Bash compatibility, default SQLite config storage in ~/.toksi/toksi.db, mouse-capable TUI (ratatui/crossterm) and Web UI Telemetry Cockpit... [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Toksi master architecture v3.1: 5-port dedicated topology (:8787 Headroom, :8789 Copilot VSCode, :8790 Copilot CLI, :9099 AI Gateway, :9100 Background Dashboard Cockpit) eliminating port contention; expanded 39+ BYOK providers including Bifrost, SageMaker, Bedrock Mantle and smart background... [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Enforce first-class BYOK provider registration for 5 Local Agent CLIs (agy.exe, copilot.exe, claude, cursor, grok) with smart background launch and chat endpoint verification; mandate active paid quota/subscription for all BYOK providers (disqualifying free-tier accounts from gateway rotation... [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Refactor: Refactor code structure for improved readability and maintainability [crates/toksi/.toksi/memory.db]

## Notes
- New feature: feat: update project documentation for Phase 13 end-to-end system verification and benchmarks [crates/tokless/CONSOLIDATION_PLAN.md]
- New feature: feat(tokless): implement Phase 7 headroom deep integration and native enterprise AI gateway [crates/tokless/src/gateway/agent_cli/claude_code.rs]
- New feature: feat(tokless): implement Phase 12 native Prometheus telemetry subsystem and metrics endpoint" -m "- Add in-tree model ca [crates/tokless/src/dashboard/mod.rs]
- New feature: feat: enhance proxy detection and routing for Antigravity and Copilot, improve query handling in BYOK gateway [crates/toksi/src/agents/antigravity.rs]
- New feature: feat: implement Transfer-Encoding chunked decoding in HTTP request parser and add corresponding tests [crates/toksi/src/headroom/byok_gateway.rs]
- New feature: feat: Add VS Code Antigravity extension connection fixes and RTK hook enhancements [crates/toksi/src/agents/antigravity.rs]
- New feature: feat(keyring): Implement OS keyring support with OAuth envelope handling [crates/toksi/src/commands/auth.rs]
- New feature: feat: Enhance Antigravity auth and eligibility checks; implement route discrimination for AICode and improve keyring tar [crates/toksi/src/gateway/agent_cli/gemini_agy.rs]
- New feature: feat: Implement bypass for non-LLM authentication and ConnectRPC management endpoints; enhance error handling and loggin [crates/toksi/src/agents/antigravity.rs]
- New feature: feat: Update project documentation and fix CLI test assertions; enhance command filtering logic [crates/toksi/src/rtk/engine.rs]

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
