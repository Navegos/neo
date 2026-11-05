# Phase 06: In-Tree Rules & Skills Integration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_06_IN_TREE_RULES_AND_SKILLS.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/andrej-karpathy-skills, neo/inspect-vault/caveman, neo/inspect-vault/ponytail as updated please rerun

Execute Phase 06: In-Tree Rules & Skills Integration.
Sources: neo/inspect-vault/andrej-karpathy-skills, neo/inspect-vault/caveman, neo/inspect-vault/ponytail
Target: crates/toksi/src/rules/

Requirements:
1. Ingest instructions and guidelines directly into crates/toksi/src/rules/embedded/:
   - principles.rs (Karpathy guidelines: think, simplify, surgical changes, verify)
   - caveman.rs (ASD-STE100 primitive English compression guidelines)
   - ponytail.rs (lazy senior developer build discipline ladder)
2. Compile markdown assets into the binary using `include_str!`.
3. Eliminate all external repository clone links. All rules must be distributed inside the single toksi binary and accessible from the repo project directory.
4. Expose clean functions to inject or patch instructions into agent configs (CLAUDE.md, .cursor/rules/, AGENTS.md).
5. Ensure zero compiler warnings and passing unit tests.
```

