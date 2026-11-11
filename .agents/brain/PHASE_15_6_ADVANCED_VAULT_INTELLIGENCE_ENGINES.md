# Phase 15.6: Advanced Vault Intelligence Engines Consolidation

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Always treat all 36 reference vaults in neo/inspect-vault/* — especially TokenTracker, jcodemunch-mcp, repomix, ast-grep, scip, mem0, chroma, litellm, TokenEconomizer, agent-opt, plus Wave 17 agentic vaults (hermes-agent, openclaw, openclaw-rs, mofa, mofa-engine, mofaclaw, codex, crustly, crush, goose, grok-build, gemini-cli, crewAI) — as active code to inspect, fix bugs in, and wire cleanly into consolidated toksi.

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
  - neo/inspect-vault/TokenEconomizer -> crates/toksi/src/headroom/compression/economizer.rs
  - neo/inspect-vault/agent-opt -> crates/toksi/src/agents/opt.rs
  - Wave 17 agentic & router vaults -> crates/toksi/src/agent_runtime/, crates/toksi/src/gateway/router.rs (`PHASE_17_5` / `PHASE_17_6`)
Targets:
  - crates/toksi/src/telemetry/tokentracker.rs
  - crates/toksi/src/codegraph/
  - crates/toksi/src/aimemory/
  - crates/toksi/src/gateway/
  - crates/toksi/src/headroom/compression/
  - crates/toksi/src/agents/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors.
3. 100% Native Rust Implementation: Complete elimination of external language runtimes (Node.js, Python, TypeScript).
4. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all file trees, symbols, and SCIP documents.
5. Repo-Only & Dynamic Ignore Purging: Store all workspace tool artifacts strictly inside `<repo>/.toksi/<tool>` (respecting `.gitignore`/`.ignore`/`.git/` boundaries & monorepo subprojects); dynamically purge vector caches and FTS5 records.

Requirements:
1. Ingest & Wire All Vaults End-to-End in Pure Rust (Eliminating Verified Stub Bugs):
   - **TokenTracker**: 47 AI tool usage parsers, `cache_creation_input_tokens`, Responses SSE usage, zero-cost `limits-cache`.
   - **jcodemunch-mcp**: Wire directly into `codegraph/mod.rs` + CLI + MCP using O(1) SQLite `byte_offset..byte_len` slices.
   - **Repomix**: Wire directly into `codegraph/mod.rs` + A2A pack export with secretlint redaction, BPE counter, `.gitignore`/`.repomixignore`.
   - **ast-grep**: Replace heuristic line matcher with real Tree-sitter CST metavariable engine (`$VAR`, `$$$`) + `MatchStrictness`.
   - **scip**: Replace stub with real Protobuf wire encoding + descriptor AST extraction across all 41+ languages.
   - **mem0**: Personalized long-term entity-fact memory synced with `icm`/`ai-memory` inside `<repo>/.toksi/`.
   - **Chroma**: Upgrade `semantic_cache.rs` from linear JSON scan to ANN HNSW (`instant-distance`/`usearch`) + SPANN disk spill (`foyer`) and wire live `store()`/`lookup_semantic()` in `headroom/proxy.rs`.
   - **LiteLLM**: 100+ model naming aliases, `/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/embeddings` mapping, billing catalog across 65/90+ ecosystem + 22 local engines.
   - **TokenEconomizer**: Fix empty-text stub bugs; implement 3-stage compression + `EnvFactsDistiller` + `ToolCompact` (11 formats) + `LoopGuard`.
   - **agent-opt**: Fix randomized `DefaultHasher` bug using `SHA-256` 1-hop context dedup, AST outline slicing, and semantic oscillation detector (`max_repetitions: 3`).
2. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **AST Outline Chunking** (`src/codegraph/jcodemunch.rs`): Extracts function signatures and class summaries without pulling full bodies into context.
- **Codebase Packager** (`src/codegraph/repomix.rs`): Packages relevant files into structured XML/markdown context payloads with accurate token counts.
- **SCIP Graph Indexer** (`src/codegraph/scip.rs`): Generates and queries SCIP symbol graphs for exact cross-file symbol definition lookups.
- **LiteLLM Model Compatibility Matrix** (`src/gateway/normalizer/litellm.rs`): Maps 100+ model names to provider-specific endpoints and token pricing tiers.
- **Token Economizer** (`src/headroom/compression/economizer.rs`): Eliminates repetitive instructions and strips schema overhead.
- **Agent Optimizer** (`src/agents/opt.rs`): Prunes intermediate redundant execution turns and bounds latency.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- codegraph` and `cargo test -p toksi -- telemetry` pass 100%.
- Structural AST searches using ast-grep patterns match targeted functions accurately.
