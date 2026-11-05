# Phase 09: Legacy Healing & Binary Un-Hijack

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_09_LEGACY_HEALING_UNHIJACK.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 09: Legacy Healing & Binary Un-Hijack.
Target: crates/toksi/src/env_injector/heal.rs, crates/toksi/src/agents/{antigravity.rs, grok.rs, copilot.rs}

Requirements:
1. Purge destructive binary hijacking from Toksi:
   - Stop renaming agy.exe to agy.real.exe or agy to agy.real.
   - Stop renaming grok.exe to grok.real.exe or copilot to copilot.real.
   - Delete all shim batch wrappers (agy.cmd, grok.cmd, shell wrappers) that collide with IDE extensions.
2. Build automated self-healing engine in `crates/toksi/src/env_injector/heal.rs`:
   - Detect any .real or .real.exe binaries left behind by legacy tools.
   - Atomically restore them to their canonical binary filenames (agy.exe, grok.exe, copilot).
   - Remove orphaned state files (antigravity-cli-proxy.json, grok-cli-proxy.json, etc.).
3. Automatically invoke this healing routine inside toksi init, toksi doctor --fix, and toksi daemon start.
4. Ensure zero compiler warnings and passing tests.
```

