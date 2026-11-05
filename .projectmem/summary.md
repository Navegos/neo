# projectmem - neo

_Last updated: 2026-11-05_

## Project purpose
--------------------------------------------------------------------------------
Enterprise-grade native Rust tooling, token-optimization, and headroom gateway pipeline for AI coding assistants. Transparently sits between 17 coding agents and 39+ upstream LLM providers to cut token spend by 30-70%, enforce latency/cost/concurrency quotas, and provide zero-latency memory, AST code intelligence, and real-time telemetry across dedicated isolated ports.

## Recent issues
- [OPEN] #0035 Phase 08: Purge external runtime downloaders and rewire agent harnesses to in-tree modules [crates/toksi/src/util/] (open)
- [OPEN] #0034 Bypass non-LLM authentication and ConnectRPC management endpoints directly without intercepting headers or proxying, capturing only LLM processing in and out [crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)] (open)
- [OPEN] #0033 Transparent TCP tunnel or non-LLM passthrough for Antigravity auth/eligibility, proxy only LLM generation requests [crates/toksi/src/headroom/proxy.rs:615 (serve_headroom_connection)] (open)
- [DONE] #0032 Antigravity eligibility check returns 404 not found Google upstream service endpoint not found [crates/toksi/src/headroom/proxy.rs:672 (serve_headroom_connection)] -> Automatic sibling upstream fallback between aicode.googleapis.com and cloudcode-pa.googleapis.com on 404, eliminating 404 endpoint not found during Antigravity eligibility checks (fixed)
- [DONE] #0031 Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing [crates/toksi/src/headroom/proxy.rs:625 (serve_headroom_connection)] -> Preserve binary Protocol Buffer framing in and out, translate upstream 502/HTML errors to structured ConnectRPC error JSON (eliminating proto syntax error), and route all Cloud Code/Jetski RPCs to aicode.googleapis.com (fixed)
- [DONE] #0030 Antigravity auth and eligibility check fails with proto syntax error line 1:1 invalid value due to ConnectRPC route discrimination, wrong upstream routing, and HTTP chunking of protobuf responses [crates/toksi/src/headroom/proxy.rs:619 (serve_headroom_connection)] -> Expanded route discrimination for ConnectRPC and AICode service prefixes, routed aicode endpoints to aicode.googleapis.com, prevented false chunked transfer-encoding on unary protobuf/ConnectRPC responses, and expanded keyring targets for antigravity_user and jetski-standalone-oauth-token (fixed)
- [DONE] #0029 RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json [crates/toksi/src/agents/antigravity.rs:210 (antigravity_hooks_file)] -> Added ctx-hook and context-mode aliases in toksi CLI, installed fresh binary to C:/Programs/neo/bin/toksi.exe (fixed)
- [DONE] #0028 VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream [crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)] -> Eliminated false chunking of HTTP/2 responses lacking Content-Length that broke protobuf JSON parsing with invalid value []. Wired GatewayProxyOrchestrator semantic cache into proxy loop. [crates/toksi/src/headroom/byok_gateway.rs:1640 (forward_streaming_to_byok_target)] (fixed)
- [DONE] #0027 toksi proxy reports model unavailable in VS Code antigravity extension while terminal works [crates/toksi/src/main.rs:252 (run)] -> Proxy routes /v1beta and gemini endpoints to generativelanguage even when x-goog-api-client has antigravity header. Added CORS preflight 204 handler and fixed /v1 path preservation in byok_target_path. Tested by test_non_llm_uri_and_target_resolution. (fixed)
- [DONE] #0026 VS Code MCP spawn and proxy wiring parity for Copilot and Antigravity streaming [crates/toksi/src/agents/copilot.rs:2478 (detect_copilot_proxy)] -> Resolved Copilot proxy startup, VS Code MCP proxy boot, and Antigravity streaming timeouts in Toksi (fixed)
- [DONE] #0025 HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400 [crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)] -> Add Transfer-Encoding chunked decoding loop to read_chunked_body, stripping chunk size headers (e.g. 10103\r\n) and reassembling valid JSON body with Content-Length (fixed)
- [DONE] #0024 Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth [crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)] -> Headroom proxy detects non-LLM requests, bypasses prompt modifications, preserves query strings, and routes Cloud Code endpoints to https://cloudcode-pa.googleapis.com (fixed)
- [DONE] #0023 Fix proxy wiring detection for Antigravity and Copilot on Windows across proxy status, dashboard, and TUI [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)] -> Fixed Antigravity & Copilot Headroom proxy routing and status detection across dashboard, TUI, and CLI (fixed)
  - Partial attempt: Check Windows registry env for Antigravity, enable Copilot CLI provider config wired check, and update TUI/Dashboard to reflect proxy routing state [crates/toksi/src/agents/antigravity.rs:1949 (antigravity_proxy_wired)]
- ...and 22 older fixed issues not shown (`pjm search`, or `get_issue(issue_id)` via MCP)

## Decisions
- ...50 earlier decisions not shown (`pjm search`, or `search_events(query)` via MCP)
- Toksi master architecture v3.1: 5-port dedicated topology (:8787 Headroom, :8789 Copilot VSCode, :8790 Copilot CLI, :9099 AI Gateway, :9100 Background Dashboard Cockpit) eliminating port contention; expanded 39+ BYOK providers including Bifrost, SageMaker, Bedrock Mantle and smart background Agent CLI bridges; gateway activation guard requiring >=1 provider without crashing when using uncredentialed CLI agents; ground-truth session token accounting from live SSE streams and rate-limit headers; full rules/skills/instructions synchronization across agent workspaces and home config dirs [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Enforce first-class BYOK provider registration for 5 Local Agent CLIs (agy.exe, copilot.exe, claude, cursor, grok) with smart background launch and chat endpoint verification; mandate active paid quota/subscription for all BYOK providers (disqualifying free-tier accounts from gateway rotation pool with non-blocking inform and zero-fail/zero-bug fallback) [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Refactor: Refactor code structure for improved readability and maintainability [crates/toksi/.toksi/memory.db]
- Phase 03: Completed full native pure Rust port of Context-Mode with 100% parity: isolated PolyglotSandbox with CPU timeout and memory bounds, Windows mvn rewrite, NO_COLOR/FORCE_COLOR, CTX_FILE, background detached execution; filter.rs with OUTPUT_CAP and EXEC_PREVIEW_BYTES slice/handle line; fts.rs SQLite FTS5 chunker and BM25 search; fetch.rs async reqwest fetcher with SSRF guard and markdown extraction; mcp.rs in-process ctx_* tools (ctx_execute, ctx_execute_file, ctx_batch_execute, ctx_batch, ctx_index with url, ctx_search with auto-memory, ctx_fetch_and_index, ctx_resume, ctx_stats, ctx_doctor, ctx_purge) and toksi ctx CLI. 27 unit tests and 7 integration tests passing with 0 warnings. [crates/toksi/src/contextmode/mod.rs:1]
- Prioritize ~/.toksi/memory.db, expose recall_tiered_entries headroom hook, and support memory_handoff_accept in ICM
- Phase 05: CodeGraph Pure Rust Engine refreshed with upstream inspect-vault updates (C/C++ member type refs, Python parameterized base, Go uppercase const/var exports, JS/TS subscript exports) [crates/toksi/src/codegraph/mod.rs:1]
- Phase 07: Headroom Deep Integration & Gateway Catalog refreshed with 39+ providers (added BedrockMantle, SageMaker, Bifrost), byte-for-byte critical log survival, AST-aware code compression, and dual-tier semantic cache [crates/toksi/src/gateway/providers/mod.rs]
- Master architecture v3.3: unified 5-port daemon (:8787,:8789,:8790,:9099,:9100), root .toksi workspace consolidation, normalized hooks (<tool>-hook <agent>) and MCP names, 39+ BYOK providers with Bifrost and smart CLI bridges, strict paid quota enforcement with zero-fail fallback, ground-truth token accounting, forward-slash paths, mouse TUI and background Web UI [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Master architecture v3.3 plan approved: unified 5-port daemon (:8787,:8789,:8790,:9099,:9100), root .toksi workspace consolidation, normalized hooks (<tool>-hook <agent>) and MCP names (projectmem, context, codegraph), 39+ BYOK providers with Bifrost and smart CLI bridges, strict paid quota enforcement with zero-fail fallback, ground-truth token accounting, forward-slash paths, mouse TUI and background Web UI [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Consolidated Toksi v3.3 Master Architecture & Execution Plan across 5 dedicated ports, root .toksi workspace topology, normalized tool hooks (<tool>-hook <agent>) and MCP names, 39+ BYOK providers with smart background CLI bridges, strict paid quota enforcement with zero-fail fallback, ground-truth session token accounting, forward-slash paths, mouse TUI, and background Web UI cockpit [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Toksi v3.3 Disciplinary Coding Standards and Conventions Consolidation [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Modularized Phase 15 and 16 into discrete ready-to-run subphase prompt files in .gemini/antigravity/brain/*.md for token-isolated per-chat execution [crates/toksi/CONSOLIDATION_PLAN.md:1]
- Phase 03 refresh: verified full Context-Mode parity in pure Rust with strict CPU/memory limits, in-memory RTK pipe, SQLite FTS5 BM25 search, SSRF per-hop enforcement, and in-process ctx_* MCP suite [crates/toksi/src/contextmode/mod.rs:1]
- Phase 04 rerun: hardened ICM FTS5 BM25 token sanitization with operator stripping, prioritized ~/.toksi/memory.db, added memory_handoff_accept MCP alias, and added extractive leading_prose fallback for tiered progressive disclosure [crates/toksi/src/icm/store.rs:593 (sanitize_fts5_query)]
- Phase 05: CodeGraph Pure Rust Engine executed and verified with upstream inspect-vault parity, dual-path .toksi/codegraph/symbols.db and .codegraph/symbols.db support, petgraph blast-radius, and file-suffix coordinate resolution [crates/toksi/src/codegraph/mod.rs:1]

## Notes
- gotcha: BYOK gateway requires active commercial paid tier; free or trial accounts bypass rotation pool with non-blocking notice without crashing [crates/toksi/src/gateway/agent_cli/tier_check.rs:1]
- gotcha: universal forward-slash / path normalization required for Git Bash and cross-platform compatibility; isolated port 9099 (AI Gateway) and port 9100 (Web Dashboard) prevents SSE socket contention [crates/toksi/src/util/paths.rs:1]
- New feature: feat(plan): detail phase 15-16 execution specs and wire aimemory mcp [crates/toksi/src/util/mcpspawn.rs]
- New feature: feat(rtk): support human-readable git status in output filter [crates/toksi/src/rtk/filters/git.rs]
- feat(rtk): refresh RTK filter engine from inspect-vault with git ls-tree/ls-files fold, cargo nextest retry filtering, enhanced ruff JSON/text filters, and expanded root-cause extractions [crates/toksi/src/rtk/filters/git.rs:25 (filter_git)]
- New feature: feat(rtk): enhance git, nextest, and ruff filters with root-cause adapters [crates/toksi/src/rtk/filters/build.rs]
- New feature: feat(projectmem): align edge cases with v0.3.5 and sync phase prompt docs [crates/toksi/src/projectmem/mcp.rs]
- feat(contextmode): enhance SSRF guard with RFC ranges (shared 100.64.0.0/10, bench 198.18.0.0/15, broadcast 240.0.0.0/4, NAT64, 6to4) and per-hop redirect validation [crates/toksi/src/contextmode/fetch.rs:80 (ssrf_guard)]
- New feature: feat(contextmode): harden SSRF guard, add ctx_batch/ctx_upgrade tools, and update tests [crates/toksi/src/contextmode/fetch.rs]
- New feature: feat(aimemory,icm): add handoff alias, fallback paths, and robust fts5 sanitization" -m "- aimemory: support .toksi/ai-m [crates/toksi/src/aimemory/mod.rs]

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
