# Toksi Master Execution Index & Brain Guide (Wave 22 / v9.0)

This directory (`d:/AI/neo/.agents/brain/`) holds the modular, non-overlapping implementation phases (**61** total markdown guides: `INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_22_6`) for the **Toksi Super-Intelligence & Zero-Regression Runtime Consolidation (Wave 22 / v9.0)** spanning all **502 unique mandated vaults** in `inspect-vault/*` (`294` unique Wave 18 `<deep-research>` vaults + `138` Wave 19 additions + `61` Wave 20 additions = `493` Wave 20 + `2` Wave 21 = `495` Wave 21 + `7` Wave 22: `ratatui`, `ratatui-image`, `tui-realm`, `iocraft`, `r3bl-open-core`, `MojibakeLab`, `python-ftfy` across `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, and `<skills>` `Navegos/skills`, covering Sections `2.1`--`2.18`, the **11-Pillar AI/ML Lifecycle**, **Distributed Cluster Stack `src/cluster/`**, **Single `NetworkService` Network Manager + Headroom-Central `<toksi-siagent>`**, **`r3bl-open-core` (`r3bl_tui`) Chat & TUI migration**, **13-IDE External Plugin Host**, **Dynamic `.git/config` Repo Discovery + `<neo>/agents/rules/` + Pure 7-Bit ASCII & `MojibakeLab`/`python-ftfy` Auto-Healer**, **Multi-Agent Orchestration Resilience & Cross-Chat Resume**, and **35-component `<toksi-siagent>` Super Intelligence Agent Architecture**), all **30+ local/Rust inference engines**, and all **65/91+ AI coding agents / runtimes / protocols / security scanners**. Every phase prompt starts by reading ALL **16** canonical workspace documents --- including `crates/toksi/README.md`, `.agents/rules/ascii-only.md`, `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`, and canonical `<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}` --- and follows the Multi-Agent Resume Protocol so parallel or sequential agents maintain identical structural awareness across network drops, errors, or quota limits.

## Mandatory 12-State Progress Taxonomy

Every master plan document, roadmap table, and `.agents/brain/PHASE_*.md` header carries explicit verified status badges from the 12-state vocabulary across three orthogonal axes (Deep Research, Plan & Docs, Source Code Execution):

| State Badge | Axis | Meaning |
|---|---|---|
| `[DEEP-RESEARCH-DONE]` | Deep Research | All mandated vaults/specs for this scope inspected at source level on disk. |
| `[MISSING-DEEP-RESEARCH]` | Deep Research | Referenced vaults still require source-level deep inspection before implementation. |
| `[PLAN-DONE]` | Planning | Complete technical blueprint and worker prompt authored and synchronized. |
| `[PLAN-NOT-DONE]` | Planning | Plan document or phase specification incomplete or not yet authored. |
| `[PLAN-IMPLEMENTED]` | Planning | Planned architecture has corresponding runtime wiring and tests recorded in plans. |
| `[PLAN-UNIMPLEMENTED]` | Planning | Plan exists cleanly; runtime implementation has not started yet. |
| `[PLAN-FIXED]` | Planning | Plan drift, legacy paths, or specification gaps identified and repaired. |
| `[PLAN-NEEDS-FIX]` | Planning | Plan text itself requires targeted correction or alignment. |
| `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` | Source Code | Real production code and tests exist, compile cleanly, and pass all Wave 22 invariants. |
| `[SOURCE-CODE-INCOMPLETE]` | Source Code | Core source compiles and partially runs, but requested Wave 22 features/sub-requirements remain open. |
| `[SOURCE-CODE-NOT-IMPLEMENTED]` | Source Code | New Wave 22 module/fix is planned only and has not been written in `crates/toksi/src/` yet. |
| `[SOURCE-CODE-NEEDS-FIX]` | Source Code | Existing source code has verified runtime/policy bugs (`chat_ui.rs` double-keystroke, `loop_engine.rs` bypass/502, `skills_repo.rs` `<neo>/rules`, hardcoded `neo_dir`, non-ASCII bytes in `118` `.rs` files). |

## Mandatory Global Pre-Reads (Copy into Every Session)
```text
Read these 16 canonical workspace files FIRST before editing any code:
- crates/toksi/research/RESEARCHPLAN.md (Wave 22 / v9.0 Sections 0-14: 502 vaults, 12-State Taxonomy, NetworkService manager, Headroom-central routing, r3bl_tui, Pure-ASCII ftfy healer, Multi-Agent Resume)
- crates/toksi/CONSOLIDATION_PLAN.md (Wave 22 / v9.0 master plan, all 502 unique vaults, 11 AI/ML Pillars, Cluster Stack, 35-component <toksi-siagent>, & 65/91+ ecosystems)
- crates/toksi/PLAN.md (Wave 22 / v9.0 architectural blueprint, NetworkService + _worker topology, detached :9100, ~/.toksi/services/*.db, & 29 slash commands)
- crates/toksi/README.md (Complete CLI, config, agent, bridge, and verification guide)
- crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md (Universal Cross-Chat Run & Resume Master Prompt + Multi-Agent Resilience Protocol)
- crates/toksi/research/docker_cli_compose_yaml_full_reference.md
- crates/toksi/research/docker_sandboxes_sbx_cli_yaml_reference.md
- crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
- crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture.md
- crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
- crates/toksi/research/toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md
- .agents/rules/ascii-only.md (Strict pure 7-bit ASCII 0x00-0x7F, LF, zero BOM policy)
- .toksi/projectmem/plan.md
- .toksi/projectmem/PROJECT_MAP.md
- .toksi/projectmem/summary.md
- .agents/brain/PHASE_<N>_<NAME>.md (your active phase specification)
```

---

## Wave 20, Wave 21 & Wave 22 Canonical Invariants
- **502 Unique Mandated Vaults & 3-Repository Ecosystem**: `294` Waves 1--18 + `138` Wave 19 + `61` Wave 20 = `493` Wave 20 + `2` Wave 21 (`Zoo-Code`, `agent-orchestrator`) = `495` Wave 21 + **`7` Wave 22 (`ratatui`, `ratatui-image`, `tui-realm`, `iocraft`, `r3bl-open-core`, `MojibakeLab`, `python-ftfy`)** = **`502` total deep-researched vaults (`[DEEP-RESEARCH-DONE]`)**.
- **Real `<toksi-siagent>` Subprocess, 22 Orchestrator Modes & 29 Slash Commands (`PHASE_20_1` + `PHASE_22_1`)**: Eliminate local stubs and Headroom bypasses (`loop_engine.rs:198-205` synthetic `"gpt-6.2-astra"` -> HTTP `502`); route centrally through Headroom (`:8787`) -> Gateway (`:9099`, canonical longest-prefix/provider+family matching); expose all 22 Orchestrator Modes across 7 Cognitive Domains (`[toksi_siagent_router.*]`), `Max|XHigh|High|Medium|Low` efforts (default `Medium`), sampling filter, and **29** slash commands.
- **Wave 22 UI, Input & Git Engineering (`PHASE_22_1`, `PHASE_22_2`)**: Fix Windows `crossterm` double-keystroke (`KeyEventKind::Press`), add full line-editor ergonomics + Pause/Resume/Edit-While-Working + `src/tools/git_engineer.rs`, and migrate `toksi chat|gain|top|tui` toward `r3bl-open-core` (`r3bl_tui`) + `ratatui-image`/`tui-realm`/`iocraft` with `winit`-style press gating and mouse hover.
- **Dynamic `.git/config` Repo Discovery (`No neo_dir`), `<neo>/agents/rules/` & Prompt Router (`PHASE_22_3`)**: Eliminate hardcoded `neo_dir`, stage external rules strictly in `<neo>/agents/rules/` + `<neo>/agents/agent_instructions.md`, and provide non-destructive prompt classification/routing (`toksi refactor-router`).
- **Strict Pure 7-Bit ASCII & `MojibakeLab`/`python-ftfy` Auto-Healer (`PHASE_22_4`)**: Zero non-ASCII bytes (`0x00`-`0x7F` only), `LF`, zero BOM in enforced paths + `toksi heal-encoding` + remediation of `118` `.rs` files in `crates/toksi/src/`.
- **Separate Service Ports, Copilot -> `:8787` Consolidation, Multi-Surface SAST, Plugin Mounts & Bidirectional BYOK (`PHASE_22_5`)**: Enforce collision-free separate service ports, consolidate Copilot `:8789`/`:8790` compatibility onto `:8787`, wire standalone/MCP/`<toksi-siagent>` SAST, expand `src/plugins/` mounts, sync DB <-> TOML BYOK atomically, and set `CONSOLIDATED_VAULT_COUNT = 502`.

---

## Complete 60-Phase Execution Catalog (Waves 1--22 --- 61 Files Total)

### Wave 1--14: Pure Rust Foundation & Zero-Downloader Hardening
1. `PHASE_01_RTK_FILTER_ENGINE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Port RTK TOML filters, TEE recovery, `toksi gain`, SQLite history.
2. `PHASE_02_PROJECTMEM_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- JSONL append-only ledger + derived `summary.md` + strict `<repo>/.toksi/projectmem/` isolation.
3. `PHASE_03_CONTEXTMODE_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- FTS5 BM25 sandbox analyzer (`PHASE_19_1` renamed `context-mode` refs -> `ctxmode` + `<repo>/.toksi/ctxmode/`).
4. `PHASE_04_ICM_AND_AIMEMORY.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Hierarchical SQLite memory + hybrid BM25/vector engine (`aimemory` rename completed in `PHASE_19_1`).
5. `PHASE_05_CODEGRAPH_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- 45-language Tree-sitter AST structural indexer, call graphs & blast-radius trace.
6. `PHASE_06_IN_TREE_RULES_AND_SKILLS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- Externalized skills/rules staged outside `crates/toksi/` synced to `<skills>`; Wave 22 moves `rules/` lookups to `<neo>/agents/rules/` + `<neo>/agents/agent_instructions.md`.
7. `PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Pure-Rust Headroom AST/TOON proxy (`:8787`), effort scaling, and open BYOK routing.
8. `PHASE_08_DOWNLOADER_PURGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Eliminate third-party npm/pip/curl installers; 100% pure-Rust single binary.
9. `PHASE_09_LEGACY_HEALING_UNHIJACK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Un-hijack corrupted hooks/settings & restore pristine agent binaries.
10. `PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Append-only `<enforce>` rules & reversible env export shims.
11. `PHASE_11_AGENT_DOCTOR_SELF_HEALING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Self-healing `toksi doctor` CLI & automatic WAL/FTS5/legacy-dotdir recovery.
12. `PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- 10-tab UI + `~/.toksi/services/*.db`; Wave 22 upgrades `top`/`gain`/`tui` to `r3bl-open-core` + mouse hover (`PHASE_22_2`).
13. `PHASE_13_E2E_VERIFICATION_BENCHMARKS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Unit/integration suites + isolated Docker/Compose/sandbox E2E.
14. `PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Native OS Keyring, SQLite `toksi.db`, and Google Cloud ADC bridge.

### Wave 15--16: Enterprise Topology, Vault Connectors, TUI & Cockpit
15.1. `PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- `#![deny(unsafe_code)]`; Wave 22 removes hardcoded `neo_dir` and non-ASCII bytes across `118` `.rs` files (`PHASE_22_3`/`22_4`).
15.2. `PHASE_15_2_UNIFIED_5_PORT_DAEMON.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Dedicated `_worker` subprocesses + detached `:9100` dashboard.
15.3. `PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Universal quota intelligence with zero paid-tier restrictions.
15.4. `PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Full MCP/A2A/ACP/`ANP` (`:9098`)/`QUIC` live-wire protocol parity.
15.5. `PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Token savings attribution (`tokview`), effort-tiered accounting, reconnect self-healing.
15.6. `PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Vault indexing & local graph bridges active.
16.1. `PHASE_16_1_MOUSE_CAPABLE_TUI.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- 10-tab mouse TUI; Wave 22 adds `r3bl_tui` + mouse hover + press gating (`PHASE_22_2`).
16.2. `PHASE_16_2_WEB_COCKPIT_DASHBOARD.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- Detached `:9100` 10-tab Cockpit; Wave 22 expands plugin UI panel mounts (`PHASE_22_5`).
16.3. `PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Universal `.agents/` & vendor layouts + README parity.

### Wave 17: Unified Runtime, TOML Config, 45-Language Tree-sitter & Direct Wire
17.1. `PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- Canonical `config.toml`; Wave 22 removes `neo_dir` + adds bidirectional DB/TOML BYOK sync (`PHASE_22_3`/`22_5`).
17.2. `PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Multi-model DB catalogs, `plugins_registry`, `gemini:antigravity` keyring, Headroom pass live.
17.3. `PHASE_17_3_TREESITTER_41_LANGS_AND_REPO_ISOLATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Complete 45-language `langs.rs` + `.git`-root scoping.
17.4. `PHASE_17_4_VAULT_ECOSYSTEM_CONNECTORS_AND_HOOK_CLEANUP.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `commands/{index,ctx,runmcp}.rs` + `ctxmode`/`aimemory` hook rename.
17.5. `PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Real `_worker siagent` + **29** slash commands wired.
17.6. `PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- SSE usage accounting, reconnect heal & hook dispatch live.
17.7. `PHASE_17_7_ASYNC_9TAB_UI_INCREMENTAL_TESTS_AND_DOCS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Baseline suites tracked in `.toksi/test-baseline.json`.

### Wave 18 (v5.2): 294 Vaults, 35-Component `<toksi-siagent>`, Plugins, Security & Checkpoints
18.1. `PHASE_18_1_29_NEW_VAULTS_RESEARCH_AND_SYNTHESIS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Synthesized Waves 1--18 vaults across Sections `2.1`--`2.10`.
18.2. `PHASE_18_2_ENFORCE_SKILLS_HEALER_AND_41_LANG_SECURITY_ANALYZER.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- `<enforce>`, `src/plugins/`, 45-lang SAST; Wave 22 adds `<neo>/agents/rules/` + multi-surface SAST (`PHASE_22_3`/`22_5`).
18.3. `PHASE_18_3_GOOGLE_ADC_EFFORT_OPTIMIZATION_AND_PLAN_CHECKPOINT.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `AGY_ADC_AUTH=true`, effort routing, SQLite checkpoints.
18.4. `PHASE_18_4_LEGACY_PROJECTMEM_REPAIR_AND_MASTER_DOC_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Hygiene checks & `.toksi/projectmem/*` migration fixed.
18.5. `PHASE_18_5_AGENT_SOURCE_UNIFICATION_ADR_SANDBOX_AND_KILL_SWITCH.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `src/security/adr.rs`, 5-backend `sandbox.rs`, `src/commands/guard.rs` live.
18.6. `PHASE_18_6_35_COMPONENT_SIAGENT_AND_INFERENCE_COMPUTE_STACK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- 35 `<toksi-siagent>` components scaffolded.

### Wave 19 (v6.0): Renames, Multi-Process `_worker` IPC, Live-Safe Tests, Hygiene & 138 Vaults
19.1. `PHASE_19_1_CTXMODE_TOOL_RENAME_EVERYWHERE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Canonical `ctxmode` & `aimemory` rename + migration.
19.2. `PHASE_19_2_DEDICATED_MULTIPROCESS_SUPERVISOR_AND_IPC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-INCOMPLETE]` --- `_worker` subprocesses over Direct-Wire IPC; Wave 22 completes distinct per-service ports + Copilot -> `:8787` (`PHASE_22_5`).
19.3. `PHASE_19_3_LIVE_SAFE_SANDBOXED_AND_DOCKER_TEST_ISOLATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `docker/Dockerfile.test` and `toksi test --docker|--sandbox` isolation.
19.4. `PHASE_19_4_ZERO_POLLUTION_HYGIENE_AND_SKILLS_CLEANUP.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- `<neo>/.agents/skills/` hygiene; Wave 22 removes stale `<neo>/rules` lookup (`PHASE_22_3`).
19.5. `PHASE_19_5_SI_AGENTIC_STACK_WORKFORCE_ANP_AND_SECURITY_VAULTS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Workforce, `OpenSpec`, `ANP` + `QUIC`, ATR + OpenAI Guardrails.
19.6. `PHASE_19_6_TRAINING_SERVING_MULTIMODAL_PLANE_AND_8STATE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `src/training/`, `toksi train|serve|eval`, `/train`, `/serve`, `/voice`.

### Wave 20 (v7.0): Real `_worker siagent`, 22 Modes, 29 Slash Cmds, 10-Tab UI, Compose/sbxenv, 11 AI/ML Pillars, Cluster Stack & OpenTelemetry
20.1. `PHASE_20_1_REAL_SIAGENT_SUBPROCESS_22_MODE_ROUTER_AND_29_SLASH_CMDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- Real `_worker siagent` & 22 modes; Wave 22 fixes `loop_engine.rs:198-205` `"gpt-6.2-astra"` Headroom bypass & HTTP `502` (`PHASE_22_1`).
20.2. `PHASE_20_2_DAEMON_HEAL_SERVICE_PER_SERVICE_DBS_AND_10TAB_UI.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- Detached `:9100`, per-service DBs, 10-Tab UI; Wave 22 completes separate service ports + bidirectional DB/TOML BYOK (`PHASE_22_5`).
20.3. `PHASE_20_3_DOCKER_COMPOSE_VIRTUALIZATION_AND_SBXENV_KIT_V3.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `docker/compose.test.yaml`, `sbxenv.yaml`, Kit v3 engine.
20.4. `PHASE_20_4_11_PILLAR_AIML_LIFECYCLE_AND_WAVE20_VAULTS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- 11-Pillar AI/ML Lifecycle & `toksi dataset|compress|eval`.
20.5. `PHASE_20_5_DISTRIBUTED_CLUSTER_STACK_AND_AI_FACTORY_MESH.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `src/cluster/` + `toksi cluster`.
20.6. `PHASE_20_6_OPENTELEMETRY_RUST_LLMOPS_AND_49FILE_8STATE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `opentelemetry-rust` `gen_ai.*` OTLP + collector/operator configs.

### Wave 21 (v8.0): `NetworkService` Manager, Codex/Grok Chat TUI, 13-IDE Host, `495` Vaults & Multi-Agent Resume
21.1. `PHASE_21_1_NETWORK_SERVICE_MANAGER_AND_SIAGENT_DECOUPLING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-INCOMPLETE]` --- Single `toksi _worker network-manager`; Wave 22 enforces separate per-service ports + Copilot -> `:8787` consolidation (`PHASE_22_5`).
21.2. `PHASE_21_2_CODEX_GROK_STYLE_SIAGENT_CHAT_TUI_AND_CLI_BRIDGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-NEEDS-FIX]` --- Chat TUI & CLI bridge; Wave 22 fixes Windows `KeyEventKind::Press` double-keystroke + line editor + Pause/Resume (`PHASE_22_1`).
21.3. `PHASE_21_3_EXTERNAL_13_IDE_PLUGIN_HOST_PROTOCOL.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Standardized `:9098`/IPC host contract (`toksi ide-bridge`) for 13 external IDEs.
21.4. `PHASE_21_4_ZOOCODE_AGENT_ORCHESTRATOR_AND_11_PILLAR_DEEP_INTEGRATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-INCOMPLETE]` --- `Zoo-Code` + `agent-orchestrator` (`495` vaults); Wave 22 promotes `CONSOLIDATED_VAULT_COUNT` to `502` (`PHASE_22_5`).
21.5. `PHASE_21_5_MULTI_AGENT_RESILIENCE_CHECKPOINT_AND_RESUME_ENGINE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- Write-ahead `orchestration_journal.jsonl`, disjoint `worker_leases.json`, 5-store atomic flush, `/resume`.
21.6. `PHASE_21_6_WAVE21_VERIFICATION_AND_55FILE_8STATE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]` --- `phase21_1_to_21_6_verification.rs`, `.toksi/test-baseline.json`, 55-file sync.

### Wave 22 (v9.0): `r3bl_tui`, `502` Vaults, Headroom-Central `<toksi-siagent>`, `<neo>/agents/rules/`, `.git/config` Auto-Detect, Pure-ASCII `MojibakeLab`/`ftfy` Healer & 12-State Audit
22.1. `PHASE_22_1_SIAGENT_CHAT_FIXES_502_ROUTING_AND_GIT_ENGINEER.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- Fix `chat_ui.rs` double-keystroke (`KeyEventKind::Press`), fix `loop_engine.rs` Headroom bypass (`"gpt-6.2-astra"` -> `502`), upgrade Gateway matching, add full line editor + Pause/Resume + `git_engineer.rs`.
22.2. `PHASE_22_2_R3BL_OPEN_CORE_TUI_MIGRATION_AND_MOUSE_HOVER.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- Migrate `toksi chat|gain|top|tui` to `r3bl-open-core` (`r3bl_tui`) + `ratatui-image`/`tui-realm`/`iocraft` + `winit`-style press gating & mouse hover.
22.3. `PHASE_22_3_GIT_AUTO_DETECT_AGENTS_RULES_AND_PROMPT_ROUTER.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- Remove hardcoded `neo_dir`, add `.git/config` dynamic discovery, target `<neo>/agents/rules/` + `<neo>/agents/agent_instructions.md`, and build `toksi refactor-router`.
22.4. `PHASE_22_4_PURE_ASCII_MOJIBAKE_FTFY_HEALER_AND_SOURCE_REMEDIATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- Build `src/security/ascii_guard.rs` + `src/util/{mojibake,ftfy}.rs` + `toksi heal-encoding`, and clean non-ASCII bytes across `118` `.rs` files in `crates/toksi/src/`.
22.5. `PHASE_22_5_SEPARATE_SERVICES_PLUGINS_STANDARDS_AND_502_VAULTS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- Enforce separate service ports, Copilot -> `:8787` consolidation, multi-surface SAST, expanded plugin mounts, bidirectional DB/TOML BYOK sync, and `502` vault count.
22.6. `PHASE_22_6_WAVE22_VERIFICATION_12STATE_AUDIT_AND_61FILE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- Author `phase22_1_to_22_6_verification.rs`, refresh `.toksi/test-baseline.json`, pass pure-ASCII + clippy + test gates, and promote **61** `.agents/brain/*.md` guides to 12-State completion.

---

## Global Verification Rules
Every phase agent MUST finish with:
1. `cargo check --workspace` --- zero errors.
2. `cargo clippy --workspace --all-targets -- -D warnings` --- zero warnings.
3. Focused phase integration test (`cargo test --test <phase_test>`) + incremental update to `.toksi/test-baseline.json` (never breaking or touching live host `toksi` processes or `~/.toksi/toksi.db`).
4. Strict `<repo>/.toksi/` repo-only isolation check + zero stale bare `.projectmem/` pre-read refs + zero non-ASCII bytes (`[^\x00-\x7F]`) or CRLF/BOM in enforced paths + zero unadapted foreign scripts/skills/actions in `<neo>` + zero unrequested background daemons.
