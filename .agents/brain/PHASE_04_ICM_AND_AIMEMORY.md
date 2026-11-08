# Phase 04: ICM & ai-memory Integration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_04_ICM_AND_AIMEMORY.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/icm and neo/inspect-vault/ai-memory is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and schema extensions into consolidated toksi.

Execute Phase 04: ICM & ai-memory Integration.
Sources:
  - neo/inspect-vault/icm (Rust) -> crates/toksi/src/icm/
  - neo/inspect-vault/ai-memory (Rust, v3.2.8) -> crates/toksi/src/aimemory/
Targets:
  - crates/toksi/src/icm/
  - crates/toksi/src/aimemory/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Universal Forward-Slash (/) Standard: Use normalize_path() for all wiki pages, file coordinates, and DB paths.
4. Directory Consolidation: Place storage in .toksi/icm/ (~/.toksi/icm/memory.db) and .toksi/ai-memory/ (~/.toksi/ai-memory/wiki/) in user home and workspace root.
5. Multi-Protocol Support: Expose ICM and ai-memory tools over MCP, A2A, ACP, stdio, SSE, and HTTP.
6. Standardized Naming: Tool identifiers are `icm` and `aimemory`. Hook syntax is `icm-hook <agent>`.

Requirements:
1. From ICM:
   - Single-file SQLite database (~/.toksi/icm/memory.db) with bundled FTS5 BM25 search.
   - Robust query sanitization stripping FTS5 control characters to prevent syntax panics.
   - Sub-10ms zero-token recall; strictly enforce the Zero-LLM token rule (no background LLM summarization loops).
2. From ai-memory:
   - Open Knowledge Format (OKF v0.2) markdown wiki store and schema.
   - Claim-once multi-agent handoff state machine with atomic file lock transitions (`pending` -> `claimed` -> `completed`).
   - Progressive disclosure packer: Tiered L0 (Table of Contents), L1 (Abstracts), L2 (Full markdown bodies) based on headroom context budget.
3. Prepare integration hooks for Headroom adaptive memory governor (`crates/toksi/src/headroom/adaptive_mem.rs`).
4. Ensure 0 compiler warnings and passing unit tests.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **ICM Local BM25 Engine** (`src/icm/store.rs`): SQLite WAL storage indexing topics, memories, and facts. Sanitize search tokens using `sanitize_fts5_query()` to prevent syntax errors.
- **OKF v0.2 Wiki Store** (`src/aimemory/wiki.rs`): Markdown-based persistent memory structured into topics with frontmatter metadata.
- **Claim-Once Multi-Agent Handoff** (`src/aimemory/handoff.rs`): State machine coordinating task transfers between agents (e.g. Architect -> Developer -> Tester) without message duplication.
- **Tiered Progressive Disclosure** (`src/aimemory/tiered.rs`): Formats memory briefs at L0/L1/L2 granularity to fit exact available LLM token headroom.

### 2. Disciplinary Standards & Invariants
- All file references and storage paths must use forward slashes `/`.
- Zero LLM tokens spent during indexing or retrieval.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- icm` and `cargo test -p toksi -- aimemory` pass 100%.
- Search queries with special characters (e.g. `query: "foo:bar*"`) return cleanly without FTS5 errors.
