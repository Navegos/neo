# Phase 17.4: Deep Vault Parity, 65/90+ Ecosystem & 22 Local Engines, & Hook/MCP Cleanup

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 10 & 13.3), crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_17_4_VAULT_ECOSYSTEM_CONNECTORS_AND_HOOK_CLEANUP.md first.

Execute Phase 17.4: Deep Vault Parity (`TokenEconomizer`, `agent-opt`, `TokenTracker`, `rtk`, `icm`, `ai-memory`, `projectmem`, `caveman` v4, `ponytail`), 65/90+ Ecosystem & 22 Local Engines, & Hook/MCP Naming Cleanup.
Sources:
  - neo/inspect-vault/{TokenEconomizer, agent-opt, TokenTracker, rtk, context-mode, icm, projectmem, ai-memory, caveman, ponytail, agents-structures, goose, grok-build, gemini-cli}
Targets:
  - crates/toksi/src/commands/{ctx.rs, runmcp.rs}
  - crates/toksi/src/headroom/compression/economizer.rs
  - crates/toksi/src/agents/{opt.rs, mod.rs, manager.rs, hooks.rs, mcp_adapters.rs, open_agents.rs}
  - crates/toksi/src/{telemetry/tokentracker.rs, rtk/*, icm/mcp.rs, projectmem/{mcp.rs, store.rs}, aimemory/*, rules/*}
  - crates/toksi/src/gateway/providers/ (Categories 1–5 + 22 Local Inference Engines)
  - crates/toksi/src/tools/autoindex.rs, crates/toksi/src/util/{exec.rs, paths.rs}

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly; zero panics or `.unwrap()`.
2. Zero Stub Passthroughs: Eliminate silent no-op hooks and broken MCP routing.
3. Exact Hook, MCP & Skill Standardization:
   - Enforce `<tool>-hook <agent>` everywhere.
   - Normalized MCP identifiers ONLY: `projectmem`, `context`, `codegraph`, `icm`, `aimemory`, `rtk`.
   - Split CLI vs IDE adapters (`antigravity-cli` vs `antigravity-ide`, `copilot-cli` vs `copilot-vscode`, etc.).
   - Map MCP skill directories `.{gemini,copilot,claude,agents}/mcp` -> `skills/<tool>`.

Requirements:
1. Fix Critical Broken Entrypoints:
   - Fix `commands/ctx.rs:417-434` (`run_ctx_hook` passthrough `return 0` stub) by porting `context-mode` `core/policy/*`, `core/hooks/dispatch`, `core/journal/*`, and `core/compaction/*`.
   - Fix `commands/runmcp.rs:167-174` (`aimemory` / `handoff` misrouted to `icm::mcp` -> route to `aimemory::mcp`).
2. Complete Deep Vault Parity Across Remaining Missing Features & Bugs:
   - **`TokenEconomizer`** (`economizer.rs`): Fix empty-text stub bugs; implement 3-stage pipeline (Whitespace/Comments -> AST Compression -> LLMLingua Entropy Pruning), `EnvFactsDistiller`, `ToolCompact` (11 formats), `LoopGuard`.
   - **`agent-opt`** (`agents/opt.rs`): Replace randomized `DefaultHasher` with cryptographic `SHA-256` 1-hop context deduplication + AST `get_symbol_outline`/`read_symbol_slice` + semantic oscillation detector (`max_repetitions: 3`) + `trim_context`.
   - **`TokenTracker`**: Port 47 AI tool usage parsers + Responses SSE + `cache_creation_input_tokens` + zero-cost `limits-cache`.
   - **`rtk`**: Port SQLite `Tracker` (`rtk gain` JSON/history), `tee` failure saver, `discover` missed-savings scanner, `learn` TOML/correction detector.
   - **`projectmem`**: Add 3 missing MCP tools (`precheck_file`, `list_projects`, `current_project`) + `staleness`/`structure`.
   - **`icm`**: Add ~23 missing MCP tools (`memoir_*`, `feedback_*`, `transcript_*`, `icm_pin`/`forget`/`consolidate`), `temporal` decay, `fusion` RRF.
   - **`caveman` v4 & `ponytail`**: Stage v4 companion modes (`/ultracave`, `/megacave`, `caveman-commit`, `caveman-review`, `caveman-compress`, `shrink`, `browse`, `.blocks`) + `ponytail` modes/statusline externally outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`, synced to `<skills>` `Navegos/skills` with zero binary bloat, healed/graded via `skilldoctor` + `agentskills`).
3. Register Full 65/90+ Categories 1–5 Ecosystem & 22 Local Inference Engines:
   - Wire all Category 1–5 connectors, webhooks, PR/review MCP/A2A hooks, router headers (`x-portkey-*`, `cf-aig-*`, `Helicone-*`), AND 22 local inference engines (`Ollama`, `vLLM`, `llama.cpp`, `LM Studio`, `SGLang`, `TGI`, `Exo`, `MLC`, etc.) into `gateway/providers/` + `agents/` + `PATH`/endpoint discovery.
4. Reconcile `agents-structures/` & Purge Legacy Remnants:
   - Fix `agents/mcp_adapters.rs:992` (`"openagents"` + `.agents/mcp`), `agents/hooks.rs` (`antigravity-cli` vs `antigravity-ide`), `.grok/hooks/*.json`, `.copilot/hooks/`, `.github/copilot/`, `.gemini/{settings.json, commands/*.toml}`, `.claude/.mcp.json`.
   - Purge legacy strings/helpers: `toksi-projectmem`, `toksi-codegraph`, `toksi-codegraph-index`, `agy-hook codegraph-index`, `rtk hook cursor` (`main.rs:340`), `tokless_*` (`paths.rs`), `resolve_npm_binary` (`util/exec.rs`), `toksi-codegraph-init.js` (`tools/autoindex.rs`), `util/projectmem.rs`.
5. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi` honoring **295 mandated vaults**, **35-component `<toksi-siagent>`**, **37** `.agents/brain/*.md` guides (`PHASE_01`..`PHASE_18_6`), `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized zero-bloat `<skills>` & `<advisory>`.
```

---

## Technical Specifications & Details

### 1. Verification Criteria
- `rg "toksi-projectmem|toksi-codegraph|agy-hook codegraph-index|rtk hook cursor|tokless_|toksi-codegraph-init|resolve_npm_binary" crates/toksi/src` returns **0** hits.
- `runmcp aimemory` serves `aimemory::mcp` tools (not `icm`); `ctx-hook` enforces real `policy/*` & `journal/*` instead of returning `0` silently.
- `projectmem` exposes all 16 MCP tools, `icm` exposes all 28 MCP tools, and `TokenEconomizer` produces real deterministic compression.

---

## Wave 18 (v5.2) Synchronization Addendum
- Expand audited reference scope from **36** to all **295 unique mandated vaults** (`+259` Wave 18 new & companion vaults across Sections `2.1`–`2.10` incl. `opencode`, `temporal`, `restate`, `qdrant`, `cognee`, `graphiti`, `opa`, `cedar`, `spire`, `Guardrails`, `llm-guard`, `E2B`, `OpenShell`, `cve-*`, `osv-schema`, `skilldoctor`, `sandbox-runtime`, `defenseclaw`, `skill-scanner`, 26 local/Rust inference engines, 48 vLLM/DeepSeek/SGLang vaults, 41 HF/Optimum/NeMo vaults, 51 PyTorch/TRT/Triton/GPUStack/CUDA/Slurm vaults, `<advisory>`, `<skills>` synthesized in `PHASE_18_1`–`PHASE_18_6`) into **35-component `<toksi-siagent>`** and **37** `.agents/brain/*.md` guides (`shared 7-entrypoint HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`).
- Enforce `<enforce>` security delimiters, `defenseclaw` fail-closed hookguard + `skill-scanner` multi-engine/CEL scan, `skilldoctor` + `agentskills` YAML frontmatter healing + deduplication (`PHASE_18_2`), externalized `<skills>` staging (zero binary bloat), Toksi Unified Plugin System (`src/plugins/`) hook/MCP mounts, and OpenCode `.opencode/hooks.json` unification (`PHASE_18_5`) across all synced rule files, `SKILL.md` assets, plugins, and agent hook catalogs.
