# Phase 15.6: Advanced Vault Intelligence Engines Consolidation

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Always treat all 432 unique mandated vaults in neo/inspect-vault/* — the 36 Waves 1–17 vaults (including TokenTracker, jcodemunch-mcp, repomix, ast-grep, scip, mem0, chroma, litellm, TokenEconomizer, agent-opt, hermes-agent, openclaw, openclaw-rs, mofa, mofa-engine, mofaclaw, codex, crustly, crush, goose, grok-build, gemini-cli, crewAI) plus the 259 Wave 18 new & companion vaults bridged in PHASE_18_1..PHASE_18_6 across Sections 2.1–2.10 (including opencode, temporal, restate, qdrant, cognee, graphiti, milvus, letta, text-embeddings-inference, tokenizers, RouteLLM, phoenix, olla, tokenspeed, opa, cedar, spire, Guardrails, llm-guard, E2B, OpenShell, cve-*, osv-schema, skilldoctor, sandbox-runtime, defenseclaw, skill-scanner, 26 local/Rust inference engines, 48 vLLM/DeepSeek/SGLang vaults, 41 HF/Optimum/NeMo vaults, 51 PyTorch/TRT/Triton/GPUStack/CUDA/Slurm vaults, <advisory>, <skills>) — as active code to inspect, fix bugs in, and wire cleanly into consolidated 35-component `<toksi-siagent>` (**43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, externalized zero-bloat `<skills>` & `<advisory>`).

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

---

## Wave 18 (v5.2) Synchronization Addendum
- Incorporate all Wave 18 data/wiki/observability vaults (`lance`, `qdrant`, `milvus`, `cognee`, `graphiti`, `letta`, `langchain-mongodb`, `openwiki`, `tokview`, `langfuse`, `phoenix`, `tokenspeed`, `text-embeddings-inference`, `NeMo-Retriever`, `NeMo-text-processing`, `tokenizers`), security/CVE/validator/Policy-as-Code/IAM/governance engines (`opa`, `cedar`, `spire`, `Guardrails`, `llm-guard`, `safeagents`, `garak`, `Anonymizer`, `Safe-Synthesizer`, `cve-*`, `osv-schema`, `defenseclaw`, `skill-scanner`, `skilldoctor`, `SkillSpector`, `SkillEvaluator`, `<advisory>`, `<skills>`), Unified Plugin System (`src/plugins/`), ADR + 5-backend Sandbox + Kill-Switch (`adr.rs`, `sandbox.rs` w/ `sandbox-runtime`, `OpenShell`, `E2B`, `sbx`, `guard.rs`), agent frameworks (`opencode`, `temporal`, `restate`, `langgraph`, `langchain`, `langflow`, `deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `agent-control`, `agent-framework`, `dify`, `mastra`, `agno`, `SWE-agent`, `OpenHands`, `open-swe`, `EnterpriseBlueprints`, `kimi-code`, `loopwright`, `hermes-webui`), and inference/compute stacks (26 local/Rust + 48 vLLM/DeepSeek/SGLang + 41 HF/Optimum/NeMo + 51 PyTorch/TRT/Triton/GPUStack/CUDA/Slurm) into **35-component `<toksi-siagent>`** (`PHASE_18_1`–`PHASE_18_6`), maintaining 100% parity across all **495 mandated vaults** and **43** `.agents/brain/*.md` guides.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** Synchronized to Wave 19 / v6.0 (`432` deep-researched vaults, `43` `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, dedicated `toksi _worker <role>` subprocesses, `ctxmode` rename, `docker/Dockerfile.test` live-safe isolation, `922`-dir `.agents/skills/` hygiene, and `21` target slash commands).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
