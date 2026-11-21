# Toksi Master Execution Index & Brain Guide (Wave 19 / v6.0)

This directory (`d:/AI/neo/.agents/brain/`) holds the modular, non-overlapping implementation phases (**43** total markdown guides: `INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_19_6`) for the **Toksi Super-Intelligence & Zero-Regression Runtime Consolidation (Wave 19 / v6.0)** spanning all **432 unique mandated vaults** in `inspect-vault/*` (`294` unique Wave 18 `<deep-research>` vaults + `138` Wave 19 additions/reconciliations across `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, and `<skills>` `Navegos/skills`, covering Sections `2.1`–`2.14` and the **35-component `<toksi-siagent>` Super Intelligence Agent Architecture**), all **26+ local/Rust inference engines**, and all **65 / 90+ AI coding agents / runtimes / local inference engines / security scanners**. Every phase prompt starts by reading the global architectural documents — including canonical `<repo>/.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}` — so parallel or sequential agents maintain identical structural awareness.

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
| `[PLAN-NEEDS-FIX]` | Implemented or active files fail Wave 19 invariants and require targeted remediation. |

## Mandatory Global Pre-Reads (Copy into Every Session)
```text
Read these master architectural files FIRST before editing any code:
- crates/toksi/CONSOLIDATION_PLAN.md (Wave 19 / v6.0, all 432 unique vaults, 35-component <toksi-siagent>, & 65/90+ ecosystems)
- crates/toksi/PLAN.md (Wave 19 / v6.0 architectural blueprint, 8-port multi-process _worker topology, & 21 slash commands)
- crates/toksi/research/RESEARCHPLAN.md (Sections 0–13: Wave 19 blueprints, 3-repo topology, ctxmode rename, _worker IPC, Docker/sandbox tests, 922-dir skills hygiene, ADR, sandbox, kill-switch, ADC, effort, security)
- crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
- crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
- crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
- crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
- .toksi/projectmem/plan.md
- .toksi/projectmem/PROJECT_MAP.md
- .toksi/projectmem/summary.md
- .agents/brain/PHASE_<N>_<NAME>.md (your active phase specification)
```

---

## Wave 19 / v6.0 Canonical Invariants
- **432 Unique Mandated Vaults & 3-Repository Ecosystem**: `36` Waves 1–17 vaults + `258` Wave 18 vaults (`294` Waves 1–18 incl. `xla` + companion repos `<advisory>` `Navegos/advisory` & `<skills>` `Navegos/skills` + `agents-structures`) + `138` Wave 19 additions/reconciliations (`56` Sec `2.11` pure-Rust/swarm/spec/voice agents, `28` Sec `2.12` Rust systems/IPC/storage/Vertex SDK, `46` Sec `2.13` + `xla` Sec `2.10` training/quantization/ONNX/TRT/serving/MLOps, `7` Sec `2.14` ATR/Guardrails/upskill/skills packs) = `432` total deep-researched vaults (`[DEEP-RESEARCH-DONE]`).
- **`ctxmode` Tool Rename Everywhere (`PHASE_19_1`)**: Eliminate all `316` legacy `context-mode` references and conflicting `context` tool names in `crates/toksi/src/`; standardize strictly on **`ctxmode`** (`toksi ctxmode`, `/ctxmode`, `ctxmode-hook <agent>`, MCP key `"ctxmode"`, `<repo>/.toksi/ctxmode/`), auto-migrating legacy `.context-mode/` and `<repo>/.toksi/context/` SQLite stores cleanly.
- **Dedicated Multi-Process Supervisor & Direct-Wire IPC (`PHASE_19_2`)**: Split `commands/daemon.rs` in-process Tokio tasks into isolated OS subprocesses (`toksi _worker <role>`) per Network Port (`8787`, `8788`, `8789`, `8790`, `9098`, `9099`, `9100`, `18787`), `<toksi-siagent>` (`toksi _worker siagent`), and each core tool (`codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`) bridged over Windows Named Pipes / Unix Domain Sockets + `DirectWireBus`, with watchdog restart and `CommandGuard` kill-switch fan-out.
- **Live-Safe Sandboxed & Docker Test Isolation (`PHASE_19_3`)**: `cargo test` unit suites isolate `TOKSI_HOME` tempdirs and `:0` ephemeral ports; `toksi test --docker|--sandbox` executes full daemon/port/hook E2E suites inside `docker/Dockerfile.test` (`docker/docker-compose.test.yml`) or sandbox backends without disturbing live host `toksi` services, `~/.toksi/toksi.db`, or OS env/keyring, while honoring `.toksi/test-baseline.json`.
- **Zero-Pollution Adapt-Only Repo Hygiene (`PHASE_19_4`)**: Audit all **922** directories in `<neo>/.agents/skills/` plus `<neo>/agents/` (`28` files), `<neo>/rules/` (`24` files), and crate dirs; strip unadapted foreign scripts, bash helpers, and GitHub Actions YAML; never embed skills/rules inside `crates/toksi/`; grade vetted skills via `skilldoctor` + `agentskills` + `upskill` + `skill-scanner` (`<enforce>`) and sync to `Navegos/skills` plus zero-plaintext shards to `Navegos/advisory`.
- **35-Component `<toksi-siagent>`, ANP, Workforce & 8-Port Universal Headroom Pass (`PHASE_19_5`, `PHASE_19_6`)**: Wire all 35 `<toksi-siagent>` components off the back via `DirectWireBus` across all 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098` incl. DID-WBA `ANP`, `:9099`, `:18787`) through `HeadroomEffortCompressor` (`Low`/`Medium`/`High`), unified `src/plugins/`, ATR + `openai-guardrails-python` + `model_spec`, and `toksi train | serve | eval` + `/voice` multimodal plane.
- **21 Universal Slash Commands (`< 5ms`)**: `/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/ctxmode`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, `/workforce`, `/spec`, `/train`, `/serve`, `/voice`.
- **Zero-Plaintext Blind Engine, Native Git/PR Validator, ADR, 5-Backend Sandbox, Kill-Switch & Single Memory Root**: Keyed `BLAKE3` + `sparse::DFA` (`src/security/`), `DisclosureGate` + 12-DB `UpstreamDeduplicator` + `src/validator/`, `src/security/adr.rs` + `src/agent_runtime/sandbox.rs`, `src/commands/guard.rs`, `AGY_ADC_AUTH=true` Google Cloud ADC, zero paid-tier BYOK restrictions, forward-slash `/` paths, Rust Edition 2024 `#![deny(unsafe_code)]`, and canonical `<repo>/.toksi/<tool>/` memory root.

---

## Complete 42-Phase Execution Catalog (Waves 1–19 — 43 Files Total)

### Wave 1–14: Pure Rust Foundation & Zero-Downloader Hardening
1. `PHASE_01_RTK_FILTER_ENGINE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Port RTK TOML filters, TEE recovery, `toksi gain`, SQLite history.
2. `PHASE_02_PROJECTMEM_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — JSONL append-only ledger + derived `summary.md` + strict `<repo>/.toksi/projectmem/` isolation & legacy `.projectmem/` auto-migration.
3. `PHASE_03_CONTEXTMODE_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — FTS5 BM25 sandbox analyzer & multi-language chunking (currently uses `context-mode` in `316` places; `PHASE_19_1` renames all tool/MCP/CLI/dir surfaces to `ctxmode` + `<repo>/.toksi/ctxmode/`).
4. `PHASE_04_ICM_AND_AIMEMORY.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Hierarchical SQLite memory + hybrid BM25/vector engine.
5. `PHASE_05_CODEGRAPH_PURE_RUST.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Multi-language AST structural indexer, call graphs & blast-radius trace.
6. `PHASE_06_IN_TREE_RULES_AND_SKILLS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Externalized skills/rules staged outside `crates/toksi/` synced to `<skills>` + `<enforce>` + `skilldoctor`/`agentskills` (`PHASE_19_4` audits & cleans the `922` unadapted dirs in `<neo>/.agents/skills/`).
7. `PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Pure-Rust Headroom AST/TOON proxy (`:8787`), dynamic effort scaling, and free-tier BYOK model routing.
8. `PHASE_08_DOWNLOADER_PURGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Eliminate third-party npm/pip/curl installers; 100% pure-Rust single binary.
9. `PHASE_09_LEGACY_HEALING_UNHIJACK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Un-hijack corrupted hooks/settings (`~/.claude/`, `~/.codex/`, `~/.gemini/`, `.antigravity/`) & quarantine malicious plugins.
10. `PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Append-only safe agent rule markers (`<enforce>`) & reversible env export shims.
11. `PHASE_11_AGENT_DOCTOR_SELF_HEALING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Self-healing `toksi doctor` CLI & automatic SQLite WAL/FT5/legacy-dotdir recovery.
12. `PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Initial 6-tab Ratatui TUI & Prometheus `/metrics` (`:9099`).
13. `PHASE_13_E2E_VERIFICATION_BENCHMARKS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Unit tests pass; live-safe Docker/sandbox full E2E verification completed in `PHASE_19_3`.
14. `PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Native OS Keychain/DPAPI, SQLite `byok_profiles` (`auth_mode`, `gcp_project`, `gcp_location`), `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, and Google Cloud ADC bridge.

### Wave 15–16: Enterprise Topology, Vault Connectors, TUI & Cockpit
15.1. `PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — `<repo>/.toksi/` consolidation, `#![deny(unsafe_code)]`, forward-slash `/` paths; `922` `.agents/skills/` hygiene cleanup finalized in `PHASE_19_4`.
15.2. `PHASE_15_2_UNIFIED_5_PORT_DAEMON.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Unified 8-port daemon currently running in-process Tokio tasks (`commands/daemon.rs`); refactored to dedicated OS subprocesses (`toksi _worker <role>`) in `PHASE_19_2`.
15.3. `PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Universal quota intelligence (`antigravity`, `copilot-chat`, `codex`, `claude`, `zai`, `gemini`) with zero paid-tier restrictions.
15.4. `PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — MCP/A2A/ACP/SSE protocol bus (`:9098`), OTLP/OpenTelemetry, headless CI guardrails; extended with `ANP` (`did_wba`) & `QUIC` in `PHASE_19_5`.
15.5. `PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Token savings attribution (`tokview`), effort-tiered accounting, offline/429 self-healing.
15.6. `PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Zero-download parity across OpenViking, Graphiti, LEANN, PageIndex, Context7, Gitingest, Qdrant/Vector bridges.
16.1. `PHASE_16_1_MOUSE_CAPABLE_TUI.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Full mouse-driven 9-tab Ratatui TUI (Tab 1..Tab 9); async worker/workforce/training live telemetry refinements pending Wave 19.
16.2. `PHASE_16_2_WEB_COCKPIT_DASHBOARD.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Detached HTMX + Alpine Web Cockpit (`:9100`) mirroring all 9 tabs; `_worker`, `/workforce`, `/train`, `/serve` Cockpit panels pending Wave 19.
16.3. `PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Universal 65/90+ agent hooks, `toksi plugin`, **21** slash commands (`/byok`..`/voice`), and Wave 19 README/CLI parity.

### Wave 17: Unified Runtime, TOML Config, 45-Language Tree-sitter & Direct Wire
17.1. `PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Canonical `~/.toksi/config.toml` + `toksi config/byok/auth` CLI + free tier defaults (`AGY_ADC_AUTH` toggle).
17.2. `PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Multi-model DB catalogs + `plugins_registry`/`blind_signatures`/`advisory_embargoes`/`adr_audit_events`, fallback chains, Headroom single-pass AST/TOON pipeline.
17.3. `PHASE_17_3_TREESITTER_41_LANGS_AND_REPO_ISOLATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Complete 45-language `langs.rs` support (`language_vue()`, `language_scss()`, `language_dockerfile()` live) + strict `.git`-root repo hash isolation.
17.4. `PHASE_17_4_VAULT_ECOSYSTEM_CONNECTORS_AND_HOOK_CLEANUP.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Ecosystem connectors live; `ctxmode` hook rename (`PHASE_19_1`) & subprocess Direct-Wire IPC (`PHASE_19_2`) required.
17.5. `PHASE_17_5_AGENT_RUNTIME_DIRECT_WIRE_AND_SLASH_CMDS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Native Rust agent runtime (`DirectWireClient`), `src/plugins/` mounts, `< 5ms` **16** core slash commands live; Wave 19 expands to **21** (`/ctxmode`, `/workforce`, `/spec`, `/train`, `/serve`, `/voice`).
17.6. `PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Streaming SSE usage parsing, 429 exponential backoff/failover, gateway telemetry.
17.7. `PHASE_17_7_ASYNC_9TAB_UI_INCREMENTAL_TESTS_AND_DOCS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — Async non-blocking 9-tab TUI & `:9100` Cockpit + `.toksi/test-baseline.json` (`35` suites tracked); isolated Docker/sandbox test runner wired in `PHASE_19_3`.

### Wave 18 (v5.2): 258+ Companion Vaults (294 Waves 1–18), 35-Component `<toksi-siagent>`, Unified Plugins, Zero-Plaintext Engine, Validator, `<skills>`, ADR/Sandbox/Kill-Switch, ADC, Effort & Checkpoints
18.1. `PHASE_18_1_29_NEW_VAULTS_RESEARCH_AND_SYNTHESIS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Synthesize all Wave 18 vaults across Sections `2.1`–`2.10` (`294` Waves 1–18 vaults incl. `xla`, `RouteLLM`, `phoenix`, `olla`, `tokenspeed`, `tokenizers`, `NeMo-*`, `cognee`, `graphiti`, `qdrant`, `milvus`, `letta`, `opencode`, `temporal`, `restate`, `opa`, `cedar`, `spire`, `defenseclaw`, `skill-scanner`, `<advisory>`, `<skills>`).
18.2. `PHASE_18_2_ENFORCE_SKILLS_HEALER_AND_41_LANG_SECURITY_ANALYZER.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — `<enforce>` across 3 repos, `src/plugins/`, Zero-Plaintext Blind Signature Engine (`src/security/`), Native Git/PR Validator (`src/validator/`), `skilldoctor`/`agentskills`, and 45-language AST + OSV analyzer.
18.3. `PHASE_18_3_GOOGLE_ADC_EFFORT_OPTIMIZATION_AND_PLAN_CHECKPOINT.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Google Cloud ADC (`AGY_ADC_AUTH=true`), Low/Medium/High `/effort`, and atomic `.toksi/projectmem/plan_checkpoints/` pause/resume (`/resume`).
18.4. `PHASE_18_4_LEGACY_PROJECTMEM_REPAIR_AND_MASTER_DOC_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Canonical `<repo>/.toksi/projectmem/` enforcement, doc hygiene test suite, and workspace verification.
18.5. `PHASE_18_5_AGENT_SOURCE_UNIFICATION_ADR_SANDBOX_AND_KILL_SWITCH.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Unify agent command/rule surfaces into `<toksi-siagent>`, ADR (`src/security/adr.rs`), 5-backend Sandbox (`src/agent_runtime/sandbox.rs`), and `CommandGuard` Kill-Switch (`src/commands/guard.rs`).
18.6. `PHASE_18_6_35_COMPONENT_SIAGENT_AND_INFERENCE_COMPUTE_STACK.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — 35 `<toksi-siagent>` components wired via `DirectWireBus`, 26 local/Rust engines, and vLLM/DeepSeek/SGLang/HF/NeMo/CUDA telemetry.

### Wave 19 (v6.0): `ctxmode` Rename, Dedicated Multi-Process `_worker` IPC, Live-Safe Docker Tests, Zero-Pollution Cleanup, 138 Wave 19 Vaults, Workforce/ANP/Training/Serving & 43-File 8-State Sync
19.1. `PHASE_19_1_CTXMODE_TOOL_RENAME_EVERYWHERE.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Rename all `316` `context-mode` & conflicting `context` tool/MCP/hook/slash/dir references to **`ctxmode`** (`toksi ctxmode`, `/ctxmode`, `ctxmode-hook <agent>`, `<repo>/.toksi/ctxmode/`) with zero-loss SQLite migration.
19.2. `PHASE_19_2_DEDICATED_MULTIPROCESS_SUPERVISOR_AND_IPC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Split `commands/daemon.rs` tasks into dedicated OS worker subprocesses (`toksi _worker <role>`) per Port (`8787`..`18787`), `<toksi-siagent>`, and each core tool over Named Pipe / UDS `DirectWireBus` (`pingora`/`iggy`/`quinn`/`rpmalloc-rs`).
19.3. `PHASE_19_3_LIVE_SAFE_SANDBOXED_AND_DOCKER_TEST_ISOLATION.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Ship `docker/Dockerfile.test`, `docker/docker-compose.test.yml`, `toksi test --docker|--sandbox`, and ephemeral `TOKSI_HOME`/`:0` port isolation so tests never touch live host services or `~/.toksi/toksi.db`.
19.4. `PHASE_19_4_ZERO_POLLUTION_HYGIENE_AND_SKILLS_CLEANUP.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — Audit & clean all **922** directories in `<neo>/.agents/skills/` (plus `<neo>/agents/`, `<neo>/rules/`), strip unadapted foreign scripts/actions, adapt & grade via `skilldoctor`/`agentskills`/`upskill`/`skill-scanner`, and sync to `Navegos/skills` + `Navegos/advisory`.
19.5. `PHASE_19_5_SI_AGENTIC_STACK_WORKFORCE_ANP_AND_SECURITY_VAULTS.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Implement Wave 19 pure-Rust agents, Workforce (`OMA`/`Swarms`/`ChatDev`), `OpenSpec` (`/spec`), `ANP` DID-WBA (`:9098`), Rust systems (`vertex_rust_sdk`, `lancedb`, `arrow-rs`, `tiktoken`, `BLAKE3`, `pmat`), and `ATR` + `openai-guardrails-python` + `model_spec` (`56` Sec `2.11` + `28` Sec `2.12` + `7` Sec `2.14` vaults).
19.6. `PHASE_19_6_TRAINING_SERVING_MULTIMODAL_PLANE_AND_8STATE_SYNC.md` `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — Implement `src/training/`, `toksi train|serve|eval`, `/train`, `/serve`, `/voice` (`oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `safetensors`, `ggml`, `whisper*`, `sd.cpp`, `cudarc`, `triton`, `DeepSpeed`, `unsloth*`, `Automodel`, `nanotron`, `onnx*`, `Olive`, `lighteval`, `langwatch`, `wandb`, `clearml*`, `livekit*`, `moly-ai`, `robius`, `robrix`) + 43-file 8-state master sync.

---

## Global Verification Rules
Every phase agent MUST finish with:
1. `cargo check --workspace` — zero errors.
2. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
3. Focused phase integration test (`cargo test --test <phase_test>`) + incremental update to `.toksi/test-baseline.json` (never breaking or touching live host `toksi` processes or `~/.toksi/toksi.db`).
4. Strict `<repo>/.toksi/` repo-only isolation check + zero stale `.projectmem/` pre-read refs + zero unadapted foreign scripts/skills/actions in `<neo>` + zero unrequested background daemons.
