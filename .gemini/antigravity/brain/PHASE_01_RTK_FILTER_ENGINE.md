# Phase 01: RTK Engine Integration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_01_RTK_FILTER_ENGINE.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/rtk as updated please rerun

Execute Phase 01: RTK Engine Integration.
Source: neo/inspect-vault/rtk
Target: crates/toksi/src/rtk/

Requirements:
1. Full port and embed of RTK's 100+ command-specific output filters (git, cargo, test runners, system tools) and zero-allocation ANSI stripper into crates/toksi/src/rtk/.
2. Adapt RTK to serve as the shared in-process filter engine for:
   - crates/toksi/src/contextmode/ (filtering sandbox outputs)
   - crates/toksi/src/projectmem/ (extracting root-cause failure traces)
   - crates/toksi/src/headroom/ (pre-filtering tool outputs)
3. Expose the native CLI command `toksi rtk <command...>`.
4. Ensure 100% functional parity, zero missing filter code, zero external binary spawns, 0 compiler warnings, and passing unit tests.
```

