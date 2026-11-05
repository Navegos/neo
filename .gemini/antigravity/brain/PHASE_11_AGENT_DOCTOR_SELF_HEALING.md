# Phase 11: Agent Doctor & Self-Healing Parity

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_11_AGENT_DOCTOR_SELF_HEALING.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 11: Agent Doctor & Self-Healing Parity.
Target: crates/toksi/src/commands/doctor.rs, crates/toksi/src/commands/init.rs, crates/toksi/src/agents/

Requirements:
1. Upgrade `toksi doctor` and `toksi doctor --fix`:
   - Audit all 17 agent configurations (Claude Code, Cursor, Codex, OpenCode, Antigravity, Copilot, Cline, Droid, Grok, Kilo, OMP, Pi, Amp, Kimi, ZCode, DSH).
   - Verify all rules and instructions point to the native compiled in-tree rules (Karpathy guidelines, Caveman, Ponytail).
   - Verify all MCP server entries point to in-process native Toksi MCP handlers; automatically unwire and purge broken legacy entries.
   - Check and report environment proxy status.
2. Ensure `toksi init` idempotently writes clean configs and repairs broken settings across all detected agents.
3. Ensure zero compiler warnings and passing unit tests.
```

