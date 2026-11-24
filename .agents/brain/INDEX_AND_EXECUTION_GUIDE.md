# Toksi Master Execution Index & Brain Guide (Wave 20 / v7.0)

This directory (`d:/AI/neo/.agents/brain/`) holds the modular, non-overlapping implementation phases (**49** total markdown guides: `INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_20_6`) for the **Toksi Super-Intelligence & Zero-Regression Runtime Consolidation (Wave 20 / v7.0)** spanning all **493 unique mandated vaults** in `inspect-vault/*` (`294` unique Wave 18 `<deep-research>` vaults + `138` Wave 19 additions + `61` Wave 20 additions across `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, and `<skills>` `Navegos/skills`, covering Sections `2.1`–`2.17`, the **11-Pillar AI/ML Lifecycle**, **Distributed Cluster Stack `src/cluster/`**, and **35-component `<toksi-siagent>` Super Intelligence Agent Architecture**), all **30+ local/Rust inference engines**, and all **65/91+ AI coding agents / runtimes / protocols / security scanners**. Every phase prompt starts by reading ALL **12** canonical workspace documents + companion router TOML — including canonical `<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}` — so parallel or sequential agents maintain identical structural awareness.

## Mandatory 8-State Progress Taxonomy

Every master plan document, roadmap table, and `.agents/brain/PHASE_*.md` header carries explicit verified status badges from the 8-state vocabulary:

| State Badge | Meaning |
|---|---|
| `[DEEP-RESEARCH-DONE]` | All mandated vaults/specs for this scope inspected at source level on disk. |
| `[PLAN-DONE]` | Complete technical blueprint and worker prompt authored and synchronized. |
| `[PLAN-IMPLEMENTED]` | Target Rust implementation / tests exist cleanly on disk for core functionality. |
| `[PLAN-FIXED]` | Known bugs, compile/lint issues, or drift identified and repaired with proof. |
| `[MISSING-DEEP-RESEARCH]` | Referenced vaults still require source-level deep inspection before implementation. |
| `[PLAN-NOT-DONE]` | Planned work or E2E verification partially implemented or not yet completed. |
| `[PLAN-UNIMPLEMENTED]` | Plan exists; core runtime/infra implementation has not started yet. |
| `[PLAN-NEEDS-FIX]` | Implemented or active files fail Wave 19–20 invariants and require targeted remediation. |

## Mandatory Global Pre-Reads (Copy into Every Session)
```text
Read these 12 canonical workspace files (plus the companion router TOML) FIRST before editing any code:
- crates/toksi/CONSOLIDATION_PLAN.md (Wave 20 / v7.0, all 493 unique vaults, 11 AI/ML Pillars, Distributed Cluster Stack, 35-component <toksi-siagent>, & 65/91+ ecosystems)
- crates/toksi/PLAN.md (Wave 20 / v7.0 architectural blueprint, 8-port multi-process _worker topology, detached :9100, ~/.toksi/services/*.db, & 29 slash commands)
- crates/toksi/research/RESEARCHPLAN.md (Sections 0–13: Wave 20 blueprints, 3-repo topology, ctxmode/aimemory renames, _worker IPC, Docker/Compose/sbxenv tests, 933-dir skills hygiene, 22 modes, 10-tab UI, cluster stack, OTel)
- crates/toksi/research/docker_cli_compose_yaml_full_reference.md
- crates/toksi/research/docker_sandboxes_sbx_cli_yaml_reference.md
- crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
- crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture.md
- crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
- crates/toksi/research/toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md
- crates/toksi/research/toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml
- .toksi/projectmem/plan.md
- .toksi/projectmem/PROJECT_MAP.md
- .toksi/projectmem/summary.md
- .agents/brain/PHASE_<N>_<NAME>.md (your active phase specification)
```

---

## Wave 20 / v7.0 Canonical Invariants
- **493 Unique Mandated Vaults & 3-Repository Ecosystem**: `294` Waves 1–18 + `138` Wave 19 + `61` Wave 20 additions (`26` Sec `2.15` K8s/Kubeflow/NVCRE/`NVSentinel`/`aicr`/Spark + `26` Sec `2.16` datasets/`liteparse`/`hyperflare-inference`/`kvpress`/`bitsandbytes`/PAIR/`semantic-router`/`NeMo-Agent-Toolkit*`/`NeMo-Relay-Plugins`/`elements`/`llama_index`/`haystack`/`prompt-ops`/`aider`/`vscode` + `9` Sec `2.17` `opentelemetry-*`/`AutoFA`/`deepeval`/`ragas`/`PurpleLlama`) = `493` total deep-researched vaults (`[DEEP-RESEARCH-DONE]`).
- **Real `<toksi-siagent>` Subprocess, 22 Orchestrator Modes & 29 Slash Commands (`PHASE_20_1`)**: Eliminate local echo/mock stubs in `src/agent_runtime/loop_engine.rs:173-176, 249-251`; route through `toksi _worker siagent` + IPC; implement all 22 Orchestrator Modes across 7 Cognitive Domains (`[toksi_siagent_router.*]`); support `Max|XHigh|High|Medium|Low` efforts (default `Medium`) + strict per-model `(temperature, top_p, top_k)` compatibility filtering; expose all **29** slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/mode`, `/sampling`, `/router`, `/projectmem`, `/ctxmode`, `/codegraph`, `/icm`, `/aimemory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, `/workforce`, `/spec`, `/dataset`, `/train`, `/eval`, `/compress`, `/serve`, `/cluster`, `/otel`, `/voice`).
- **`ctxmode` & `aimemory` Renames (`PHASE_19_1`)**: Eliminate legacy `context-mode` (`303` refs) and `ai-memory` (`67` refs) in `crates/toksi/src/`; standardize strictly on **`ctxmode`** & **`aimemory`** (`toksi ctxmode`, `toksi aimemory`, `/ctxmode`, `/aimemory`, `ctxmode-hook <agent>`, `aimemory-hook <agent>`, MCP keys `ctxmode`/`aimemory`, `<repo>/.toksi/{ctxmode,aimemory}/`) with lossless auto-migration.
- **Dedicated Multi-Process Supervisor, Detached `:9100`, Per-Service DBs, Auto `/v1`, Dual `agy`/ADC & 10-Tab UI (`PHASE_19_2`, `PHASE_20_2`)**: Split `daemon.rs` tasks into `toksi _worker <role>` subprocesses over Named Pipes/UDS Direct-Wire IPC; remove `:9100` from `toksi daemon start`; add `toksi daemon heal|dashboard|service <name> <action>`; move telemetry into `~/.toksi/services/*.db`; guarantee UI NEVER spawns `opencode` or external agent CLIs; auto-complete `/v1`; coexist standard `agy` + `AGY_ADC_AUTH=true` Vertex ADC; deliver **10-Tab UI** (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`).
- **Live-Safe Docker/Compose & `sbxenv.yaml`/Kit v3 (`PHASE_19_3`, `PHASE_20_3`)**: Ship `docker/Dockerfile.test` + `docker/compose.test.yaml` + declarative `sbxenv.yaml`/Kit v3 validator/runner (`toksi test --docker|--sandbox`, `toksi sandbox compose|sbxenv`) honoring `.toksi/test-baseline.json`.
- **Zero-Pollution Repo Hygiene (`PHASE_19_4`)**: Audit & clean all **933** items (**922** dirs) in `<neo>/.agents/skills/` + `<neo>/agents/` + `<neo>/rules/`, stage vetted skills outside `crates/toksi/`, sync to `Navegos/skills` and zero-plaintext blind shards (`BLAKE3` + `sparse::DFA`) to `Navegos/advisory`.
- **Distributed Cluster Stack (`PHASE_20_5`) & OpenTelemetry Native LLMOps (`PHASE_20_6`)**: Build `src/cluster/` + `toksi cluster` + `src/telemetry/otel.rs` (`opentelemetry-rust` `gen_ai.*` OTLP).

---

## Complete 48-Phase Execution Catalog (Waves 1–20 — 49 Files Total)

### Wave 1–14: Pure Rust Foundation & Zero-Downloader Hardening
1. `PHASE_01_RTK_FILTER_ENGINE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Port RTK TOML filters, TEE recovery, `toksi gain`, SQLite history.
2. `PHASE_02_PROJECTMEM_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — JSONL append-only ledger + derived `summary.md` + strict `<repo>/.toksi/projectmem/` isolation.
3. `PHASE_03_CONTEXTMODE_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — FTS5 BM25 sandbox analyzer (`PHASE_19_1` renames `303` `context-mode` refs -> `ctxmode` + `<repo>/.toksi/ctxmode/`).
4. `PHASE_04_ICM_AND_AIMEMORY.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Hierarchical SQLite memory + hybrid BM25/vector engine (`aimemory` rename in `PHASE_19_1`).
5. `PHASE_05_CODEGRAPH_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — 45-language Tree-sitter AST structural indexer, call graphs & blast-radius trace.
6. `PHASE_06_IN_TREE_RULES_AND_SKILLS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Externalized skills/rules staged outside `crates/toksi/` synced to `<skills>` (`PHASE_19_4` cleans `<neo>/.agents/skills/`).
7. `PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Pure-Rust Headroom AST/TOON proxy (`:8787`), effort scaling, and open BYOK routing.
8. `PHASE_08_DOWNLOADER_PURGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Eliminate third-party npm/pip/curl installers; 100% pure-Rust single binary.
9. `PHASE_09_LEGACY_HEALING_UNHIJACK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Un-hijack corrupted hooks/settings & restore pristine agent binaries.
10. `PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Append-only `<enforce>` rules & reversible env export shims.
11. `PHASE_11_AGENT_DOCTOR_SELF_HEALING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Self-healing `toksi doctor` CLI & automatic WAL/FTS5/legacy-dotdir recovery.
12. `PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Currently 9 tabs & coupled to `daemon.rs`; upgrade to decoupled 10-tab + `~/.toksi/services/*.db` in `PHASE_20_2`.
13. `PHASE_13_E2E_VERIFICATION_BENCHMARKS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Unit suites pass; isolated Docker/Compose/sandbox E2E moves to `PHASE_19_3`/`20_3`.
14. `PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Native OS Keyring, SQLite `toksi.db`, and Google Cloud ADC bridge (`PHASE_20_1`/`20_2` expand tables).

### Wave 15–16: Enterprise Topology, Vault Connectors, TUI & Cockpit
15.1. `PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — `#![deny(unsafe_code)]` (`0` unsafe) live; needs `PHASE_19_4` root/skills hygiene sweep.
15.2. `PHASE_15_2_UNIFIED_5_PORT_DAEMON.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Currently single-process Tokio tasks (`daemon.rs:491-588`); split to `_worker` & detach dashboard in `PHASE_19_2`/`20_2`.
15.3. `PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Universal quota intelligence with zero paid-tier restrictions.
15.4. `PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Core protocol bridges present; `ANP` (`:9098`), `QUIC`, and live-wire parity open (`PHASE_19_5`).
15.5. `PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Token savings attribution (`tokview`), effort-tiered accounting, reconnect self-healing.
15.6. `PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Vault indexing & local graph bridges active.
16.1. `PHASE_16_1_MOUSE_CAPABLE_TUI.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Base Ratatui shell live; upgrade to decoupled 10-tab (`Activity Room` + `Quotas` + `System Controls`) in `PHASE_20_2`.
16.2. `PHASE_16_2_WEB_COCKPIT_DASHBOARD.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Base `:9100` Cockpit live; decouple from `daemon`, remove `"opencode"` stub, expand to 10 tabs in `PHASE_20_2`.
16.3. `PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Universal `.agents/` & vendor layouts; full runtime & README parity follows Waves 19–20.

### Wave 17: Unified Runtime, TOML Config, 45-Language Tree-sitter & Direct Wire
17.1. `PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Canonical `config.toml` & open BYOK catalog live; router/cluster sections added in `PHASE_20_1`/`20_5`.
17.2. `PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Multi-model DB catalogs, `plugins_registry`, `gemini:antigravity` keyring, Headroom pass live.
17.3. `PHASE_17_3_TREESITTER_41_LANGS_AND_REPO_ISOLATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Complete 45-language `langs.rs` (`language_vue()`, `language_scss()`, `language_dockerfile()`) + `.git`-root scoping.
17.4. `PHASE_17_4_VAULT_ECOSYSTEM_CONNECTORS_AND_HOOK_CLEANUP.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — `commands/{index,ctx,runmcp}.rs` fixed; `ctxmode`/`aimemory` hook rename (`PHASE_19_1`) & subprocess IPC (`PHASE_19_2`) required.
17.5. `PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — `loop_engine.rs` still local stub; expand `16` -> `29` slash commands & wire `_worker siagent` in `PHASE_20_1`.
17.6. `PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — SSE usage accounting, reconnect heal & hook dispatch live.
17.7. `PHASE_17_7_ASYNC_9TAB_UI_INCREMENTAL_TESTS_AND_DOCS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — `35` baseline suites tracked; Docker/Compose/sbx live-safe harness pending `PHASE_19_3`/`20_3`.

### Wave 18 (v5.2): 294 Vaults, 35-Component `<toksi-siagent>`, Plugins, Security & Checkpoints
18.1. `PHASE_18_1_29_NEW_VAULTS_RESEARCH_AND_SYNTHESIS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Synthesized Waves 1–18 vaults across Sections `2.1`–`2.10`.
18.2. `PHASE_18_2_ENFORCE_SKILLS_HEALER_AND_41_LANG_SECURITY_ANALYZER.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — `<enforce>`, `src/plugins/`, Zero-Plaintext Blind Engine, Native Validator, `skilldoctor`/`agentskills`, 45-lang SAST.
18.3. `PHASE_18_3_GOOGLE_ADC_EFFORT_OPTIMIZATION_AND_PLAN_CHECKPOINT.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — `AGY_ADC_AUTH=true`, effort routing, SQLite checkpoints live; dual `agy`/ADC coexistence in `PHASE_20_2`.
18.4. `PHASE_18_4_LEGACY_PROJECTMEM_REPAIR_AND_MASTER_DOC_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Hygiene checks & `.toksi/projectmem/*` migration fixed.
18.5. `PHASE_18_5_AGENT_SOURCE_UNIFICATION_ADR_SANDBOX_AND_KILL_SWITCH.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — `src/security/adr.rs`, 5-backend `sandbox.rs`, `src/commands/guard.rs` live.
18.6. `PHASE_18_6_35_COMPONENT_SIAGENT_AND_INFERENCE_COMPUTE_STACK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — 35 `<toksi-siagent>` components scaffolded; real 22-mode & cluster execution wired in Wave 20.

### Wave 19 (v6.0): Renames, Multi-Process `_worker` IPC, Live-Safe Tests, Hygiene & 138 Vaults
19.1. `PHASE_19_1_CTXMODE_TOOL_RENAME_EVERYWHERE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Rename all `303` `context-mode` & `67` `ai-memory` refs to **`ctxmode`** & **`aimemory`** (`toksi ctxmode|aimemory`, `/ctxmode`, `/aimemory`, `ctxmode-hook`, `aimemory-hook`, `<repo>/.toksi/{ctxmode,aimemory}/`) with lossless auto-migration.
19.2. `PHASE_19_2_DEDICATED_MULTIPROCESS_SUPERVISOR_AND_IPC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Split `commands/daemon.rs` tasks into dedicated `toksi _worker <role>` subprocesses over Named Pipe / UDS Direct-Wire IPC.
19.3. `PHASE_19_3_LIVE_SAFE_SANDBOXED_AND_DOCKER_TEST_ISOLATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Ship `docker/Dockerfile.test` and `toksi test --docker|--sandbox` host-safe isolation.
19.4. `PHASE_19_4_ZERO_POLLUTION_HYGIENE_AND_SKILLS_CLEANUP.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Audit & clean `<neo>/.agents/skills/` (**933** items / **922** dirs), stage vetted packs outside `crates/toksi/`, sync `<skills>` + `<advisory>`.
19.5. `PHASE_19_5_SI_AGENTIC_STACK_WORKFORCE_ANP_AND_SECURITY_VAULTS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Wire Workforce (`/workforce`), `OpenSpec` (`/spec`), `ANP` + `QUIC` (`:9098`), ATR + OpenAI Guardrails (`91` Sec `2.11`/`2.12`/`2.14` vaults).
19.6. `PHASE_19_6_TRAINING_SERVING_MULTIMODAL_PLANE_AND_8STATE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Build `src/training/`, `toksi train|serve|eval`, `/train`, `/serve`, `/voice` (`47` Sec `2.10`/`2.13` vaults).

### Wave 20 (v7.0): Real `_worker siagent`, 22 Modes, 29 Slash Cmds, 10-Tab UI, Compose/sbxenv, 11 AI/ML Pillars, Cluster Stack & OpenTelemetry
20.1. `PHASE_20_1_REAL_SIAGENT_SUBPROCESS_22_MODE_ROUTER_AND_29_SLASH_CMDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Kill `loop_engine.rs` stubs, run `_worker siagent`, wire 22 Orchestrator Modes (`[toksi_siagent_router.*]`), 5 efforts (`Low..Max`), sampling compatibility filter, and **29** slash commands.
20.2. `PHASE_20_2_DAEMON_HEAL_SERVICE_PER_SERVICE_DBS_AND_10TAB_UI.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Detach `:9100`, add `daemon heal|dashboard|service`, separate `~/.toksi/services/*.db`, auto `/v1`, dual `agy`/ADC, remove `"opencode"` stub, deliver **10-Tab UI**.
20.3. `PHASE_20_3_DOCKER_COMPOSE_VIRTUALIZATION_AND_SBXENV_KIT_V3.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Build `docker/compose.test.yaml` + pure-Rust `ComposeSpec`, `sbxenv.yaml` (v1), and Kit v3 (`schemaVersion: "3"`) sandbox engine.
20.4. `PHASE_20_4_11_PILLAR_AIML_LIFECYCLE_AND_WAVE20_VAULTS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Integrate `liteparse`, `hyperflare-inference`, `NeMo-Relay-Plugins`, `kvpress`, `bitsandbytes`, `Megatron-Energon`, `ossie*`, `auto-ontology`, `semantic-router`, `AutoFA`, `deepeval`, `ragas`, `PurpleLlama` (`toksi dataset|compress|eval`).
20.5. `PHASE_20_5_DISTRIBUTED_CLUSTER_STACK_AND_AI_FACTORY_MESH.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Build `src/cluster/` + `toksi cluster` unifying K8s/Minikube/Kompose/Kops/Helm/Autoscaler/NPD, NVCRE/`NVSentinel`/`aicr`/Knavigator/DeepOps, Kubeflow/`Run`, Spark/`cudf-spark`/`cuPhoton`, and `Personal-AI-Router` LAN mesh.
20.6. `PHASE_20_6_OPENTELEMETRY_RUST_LLMOPS_AND_49FILE_8STATE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Embed `opentelemetry-rust` OTLP + `opentelemetry-python-genai` `gen_ai.*` semantic spans + `opentelemetry-collector`/`operator` configs + full 49-file 8-state sync.

---

## Global Verification Rules
Every phase agent MUST finish with:
1. `cargo check --workspace` — zero errors.
2. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
3. Focused phase integration test (`cargo test --test <phase_test>`) + incremental update to `.toksi/test-baseline.json` (never breaking or touching live host `toksi` processes or `~/.toksi/toksi.db`).
4. Strict `<repo>/.toksi/` repo-only isolation check + zero stale `.projectmem/` pre-read refs + zero unadapted foreign scripts/skills/actions in `<neo>` + zero unrequested background daemons.
