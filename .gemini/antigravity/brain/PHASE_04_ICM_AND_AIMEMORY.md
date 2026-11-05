# Phase 04: ICM & ai-memory Integration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_04_ICM_AND_AIMEMORY.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/icm, neo/inspect-vault/ai-memory as updated please rerun

Execute Phase 04: ICM & ai-memory Integration.
Sources:
  - neo/inspect-vault/icm (Rust) -> crates/toksi/src/icm/
  - neo/inspect-vault/ai-memory (Rust, v3.2.8) -> crates/toksi/src/aimemory/

Requirements:
1. From ICM: Embed single-file SQLite database (~/.toksi/memory.db) with bundled FTS5 BM25 search. Guarantee zero LLM token spend on store and recall with sub-10ms latency.
2. From ai-memory: Embed claim-once multi-agent handoff state machine and tiered L0 (TOC) / L1 (Abstracts) / L2 (Full bodies) progressive disclosure packer.
3. Strictly enforce the Zero-LLM token rule: no background LLM re-summarization loops.
4. Coordinate with crates/toksi/src/projectmem/ and prepare hooks for headroom dynamic memory budgeting.
5. Ensure zero compiler warnings and passing unit tests.
```

