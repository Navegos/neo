# Phase 15.6: Advanced Vault Intelligence Engines Consolidation

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/TokenTracker, neo/inspect-vault/jcodemunch-mcp, neo/inspect-vault/repomix, neo/inspect-vault/ast-grep, neo/inspect-vault/scip, neo/inspect-vault/mem0, neo/inspect-vault/chroma, neo/inspect-vault/litellm is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and engine capabilities into consolidated toksi.

Execute Phase 15.6: Advanced Vault Intelligence Engines Consolidation.
Sources:
  - neo/inspect-vault/TokenTracker -> crates/toksi/src/telemetry/tokentracker.rs
  - neo/inspect-vault/jcodemunch-mcp -> crates/toksi/src/codegraph/jcodemunch.rs
  - neo/inspect-vault/repomix -> crates/toksi/src/codegraph/repomix.rs
  - neo/inspect-vault/ast-grep -> crates/toksi/src/codegraph/ast_grep.rs
  - neo/inspect-vault/scip -> crates/toksi/src/codegraph/scip.rs
  - neo/inspect-vault/mem0 -> crates/toksi/src/aimemory/mem0_bridge.rs
  - neo/inspect-vault/chroma -> crates/toksi/src/gateway/semantic_cache.rs
  - neo/inspect-vault/litellm -> crates/toksi/src/gateway/normalizer/litellm.rs
Targets:
  - crates/toksi/src/telemetry/tokentracker.rs
  - crates/toksi/src/codegraph/
  - crates/toksi/src/aimemory/
  - crates/toksi/src/gateway/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. 100% Native Rust Implementation: Complete elimination of external language runtimes (Node.js, Python, TypeScript).
4. Universal Forward-Slash (/) Standard: Use normalize_path() for all file trees, symbols, and SCIP documents.
5. Dynamic Ignore Purging: Respect .gitignore and .ignore rules; dynamically purge vector caches and FTS5 records.

Requirements:
1. Ingest & Consolidate Reference Vaults into Pure Rust:
   - **TokenTracker** (`neo/inspect-vault/TokenTracker`): Ground-truth real-time session prompt/completion metrics, quota tracker, and byte-to-token ratio calculators.
   - **jcodemunch-mcp** (`neo/inspect-vault/jcodemunch-mcp`): AST outline navigation, semantic symbol chunking, token-efficient code extraction for prompts.
   - **Repomix** (`neo/inspect-vault/repomix`): Full codebase packager, directory tree generator with exact token calculations and .gitignore filtering.
   - **ast-grep** (`neo/inspect-vault/ast-grep`): Structural code search and AST pattern matching rules across polyglot trees.
   - **scip** (`neo/inspect-vault/scip`): SCIP protocol code intelligence indexer for precise cross-file definition & reference graphs.
   - **mem0** (`neo/inspect-vault/mem0`): Personalized long-term entity-fact memory extractor.
   - **Chroma** (`neo/inspect-vault/chroma`): Embedding vector similarity indexing and dual-tier semantic cache backing store.
   - **LiteLLM** (`neo/inspect-vault/litellm`): 100+ model naming aliases, standard request/response mapping, token billing rates, and drop-in compatibility.
2. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **AST Outline Chunking** (`src/codegraph/jcodemunch.rs`): Extracts function signatures and class summaries without pulling full bodies into context.
- **Codebase Packager** (`src/codegraph/repomix.rs`): Packages relevant files into structured XML/markdown context payloads with accurate token counts.
- **SCIP Graph Indexer** (`src/codegraph/scip.rs`): Generates and queries SCIP symbol graphs for exact cross-file symbol definition lookups.
- **LiteLLM Model Compatibility Matrix** (`src/gateway/normalizer/litellm.rs`): Maps 100+ model names to provider-specific endpoints and token pricing tiers.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- codegraph` and `cargo test -p toksi -- telemetry` pass 100%.
- Structural AST searches using ast-grep patterns match targeted functions accurately.
