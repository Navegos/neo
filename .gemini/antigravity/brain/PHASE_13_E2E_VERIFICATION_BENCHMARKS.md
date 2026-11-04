# Phase 13: End-to-End System Verification & Benchmarks

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, .projectmem/plan.md, and .gemini/antigravity/brain/PHASE_13_E2E_VERIFICATION_BENCHMARKS.md first to understand the global master architecture and cross-tool coupling.

Execute Phase 13: End-to-End System Verification & Benchmarks.

Requirements:
1. Perform end-to-end integration tests across all 17 supported agent harnesses (Claude Code, Cursor, Codex, OpenCode, Antigravity, Copilot, Cline, Droid, Grok, Kilo, OMP, Pi, Amp, Kimi, ZCode, DSH).
2. Verify all unified CLI commands: `toksi mem`, `toksi graph`, `toksi ctx`, `toksi rtk`, `toksi proxy`, `toksi doctor`, `toksi tui`, `toksi dashboard`.
3. Verify streaming proxy compression and in-process MCP multiplexer with live simulated agent requests.
4. Verify non-destructive environment injection and clean shutdown (confirming no .real binary renames and clean user environment teardown).
5. Run `cargo test --workspace` and verify:
   - 0 compiler warnings
   - 0 test failures
   - 0 external runtime dependencies (no npm, no pip, no node, no python required)
6. Run benchmarks measuring token reduction ratio (target 40–80% overall savings) and latency overhead (<10ms).
```

