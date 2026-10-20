# projectmem - neo

_Last updated: 2026-10-20_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- [OPEN] #0002 Completed Phase 1 Performance & Memory Optimizations: scoped thread concurrent agent detection, zero-copy byte slice MCP frame scanning, and borrowed RouteRequest fast routing pass (open)
- [DONE] #0003 Process alive check on Windows falsely flags dead processes with exit code 259 (STILL_ACTIVE) as running [crates/tokless/src/agents/proxy_provider.rs:174] -> Use WaitForSingleObject with 0 timeout on Windows handle to detect still running processes without still_active 259 false-positives; eliminate unwrap in mcp_proxy; atomic write_file for instructions and versions (fixed)
- [DONE] #0001 CVT1100 duplicate resource error when linking nbuild tests on Windows [crates/nbuild/tests/test_basic.rs] -> Eliminated duplicate winres PE resource conflict across crates, all 75 tests pass (fixed)

## Decisions
- Port tokless Go codebase from inspect-vault/tokless to Rust in crates/tokless [crates/tokless]
- Go tokless ported to Rust in crates/tokless with zero compiler warnings and 17 unit tests passing [crates/tokless]
- Implement nbuild toolchain manager using dynamic releases API endpoints and asset hint matching for packages (cmake, ninja, llvm, zlib, libpng) [crates/nbuild]
- Implement multi-compression format ranking and stem hint resilience for *.tar.gz, *.tar.xz, *.tar.zst, *.zip, *.bz2, *.rar, *.7z with bsdtar and 7z fallback extraction [crates/nbuild]
- Plan Neo multi-crate distro package, doctor self-healing, interactive TUI, and in-use shadow-swap updates
- Vendor archinfo directly into nbuild and remove inter-crate dependency to resolve winres PE resource collisions [crates/nbuild/Cargo.toml]
- Ported missing agents (amp, dsh, kimi, zcode, opencode2) and BYOK providers from Go to Rust in crates/tokless
- Plan approved: Hardening (Windows Job Object orphan process prevention, Stale Routing Lock PID reclamation) and Features (tokless doctor --json / tokless stats command, Prompt Cache Warmth Gauge)
- Implemented Hardening & Bug Prevention (Windows detached process flags, Stale Routing Lock PID cleanup) and Features (tokless doctor --json, tokless stats alias, Prompt Cache Warmth Gauge) with 93 passing workspace tests

## Notes
- New feature: feat: Implement ForgeClient for managing releases from various git forges [crates/nbuild/src/agent/mod.rs]
- Implemented nbuild doctor self-healing, TOOLCHAINS_DIR environment & PATH setup, neo multi-crate distro package, TUI menu, and fs_ops [crates/nbuild/src/cli.rs:1]
- New feature: feat: Implement safe file and directory removal functions [crates/nbuild/src/cli.rs]
- Updated release workflow for multi-crate builds and individual crates.io publishing, added README, LICENSE, and PLAN for all crates [.github/workflows/release.yml:1]
- New feature: feat(tokless): Implement core functionality for headroom proxy and token optimization [crates/archinfo/build.rs]
- New feature: feat: Update release workflow and add new catalog entries [crates/nbuild/catalog/brotli.json]
- gotcha: tokless agents mcp_adapters supports 17 agents dispatching to core McpAdapter and selection store
- Phase 3 complete: Ported and verified tool harnesses (contextmode, codegraph, projectmem, rtk, caveman, ponytail, principles) for all missing agents with zero compiler warnings and 23 tests passing
- Phase 4 complete: Ported doctor_stats, background process management, and process_alive with zero compiler warnings and 27 passing tests
- Completed full port of tokless Go codebase to Rust (Phases 1-5): all 17 agents supported, MCP adapters, selection store, tool harnesses, doctor stats, process alive, 33 unit tests passing, zero warnings

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
