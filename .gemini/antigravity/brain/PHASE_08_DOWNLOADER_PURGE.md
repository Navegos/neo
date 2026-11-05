# Phase 08: Downloader Purge & System Refactor

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_08_DOWNLOADER_PURGE.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 08: Downloader Purge & System Refactor.
Target: crates/toksi/src/util/ and tool wiring harnesses

Requirements:
1. Completely delete runtime downloader and installer modules:
   - crates/toksi/src/util/npminstall.rs
   - crates/toksi/src/util/nodewin.rs
   - crates/toksi/src/util/nodelinux.rs
   - crates/toksi/src/util/uv.rs
   - crates/toksi/src/util/projectmem.rs
2. Rewire all 17 agent harnesses (crates/toksi/src/agents/) and tool harnesses to call native in-tree Rust modules directly (projectmem, icm, aimemory, codegraph, contextmode, headroom, rtk, rules).
3. Enforce system detection policy: optional tools must only be detected on system PATH via `which::which`. Toksi never downloads or installs runtimes.
4. Clean up Cargo.toml, purge unused dependencies, remove dead code, and ensure 0 compiler warnings with all tests passing.
```

