# Toksi Project Map (Wave 21 / v8.0)

- **Repository Root**: `d:/AI/neo` (Three-Repository Ecosystem: `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory` at `inspect-vault/advisory`, `<skills>` `Navegos/skills` at `inspect-vault/skills`)
- **Primary Rust Crate**: `crates/toksi` (`crates/toksi/Cargo.toml` — `497` `src/**/*.rs` / `541` total `.rs` files, `#![deny(unsafe_code)]` with `0` unsafe blocks; leaf FFI/grammar crates `crates/toksi-sys`, `crates/toksi-grammars`)
- **Externalized Zero-Bloat Staging & Hygiene Audit Target**: `<neo>/agents/`, `<neo>/rules/` outside `crates/toksi/` synced to `<skills>`; `<neo>/.agents/skills/` cleaned & audited under `PHASE_19_4`
- **Architecture & Phase Specs**:
  - `crates/toksi/CONSOLIDATION_PLAN.md` (Wave 21 / v8.0 Master Plan — **495** unique vaults [`493` Wave 20 + `2` Wave 21: `Zoo-Code`, `agent-orchestrator`] across Sections `2.1`–`2.17`, 35-component `<toksi-siagent>`, 11 AI/ML Pillars, Distributed Cluster Stack, `NetworkService` Manager, Codex/Grok Chat TUI, 13-IDE Plugin Host, Multi-Agent Resume, 54 phases + 8-state roadmap)
  - `crates/toksi/PLAN.md` (Wave 21 / v8.0 Architectural Blueprint, `NetworkService` + `_worker` IPC Topology, `daemon heal|dashboard|service`, `~/.toksi/services/*.db`, 22 modes & `29` slash commands)
  - `crates/toksi/README.md` (Wave 21 / v8.0 Technical Documentation & CLI/TUI/Cockpit/Bridge Usage Reference)
  - `crates/toksi/research/RESEARCHPLAN.md` (Wave 21 Technical Blueprints: **495** vaults `[DEEP-RESEARCH-DONE]`, `<toksi-siagent>`, 3 repos, `ctxmode`/`aimemory` renames, `NetworkService` + DirectWire IPC, Codex/Grok Chat TUI, 13-IDE host protocol, `Dockerfile.test`/`compose.test.yaml`/`sbxenv.yaml`, skills hygiene, 22 modes, 10-tab UI, cluster stack, OTel, Multi-Agent Resilience & Cross-Chat Resume)
  - `crates/toksi/research/docker_cli_compose_yaml_full_reference.md` & `docker_sandboxes_sbx_cli_yaml_reference.md`
  - `crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture.md` (65/91+ ecosystems, Categories 1–5, sampling compatibility matrix)
  - `crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md` (`.agents` & vendor directory schemas + Category 1–3 hooks)
  - `crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md` (Zero-plaintext `BLAKE3` + `sparse::DFA`, `DisclosureGate`, 12-DB `UpstreamDeduplicator`, `CommandGuard`, & `Navegos/advisory`)
  - `crates/toksi/research/toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md` + `toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml` (22 Orchestrator Modes across 7 Cognitive Domains)
  - `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` + 54 `PHASE_*.md` files (**55** total markdown guides covering `PHASE_01`..`PHASE_21_6` with 8-State Progress Taxonomy badges)
  - `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/}` (Canonical workspace memory root)

## Crate Module Map (`crates/toksi/src/` & Wave 19–21 Additions)
| Directory / Module | Role |
|---|---|
| `src/main.rs`, `src/lib.rs`, `src/commands/` (`daemon.rs`, `worker.rs`, `test_runner.rs`, `orchestrator.rs`, `train.rs`/`serve.rs`/`eval.rs`, `dataset.rs`/`compress.rs`, `cluster.rs`, `otel.rs`, `guard.rs`, `compile_blind.rs`, `advisory.rs`, `skills.rs`, `sandbox.rs`, `security.rs`, `plugin.rs`, `ctx.rs`) | CLI entrypoint, `NetworkService` Manager (`toksi _worker network-manager`) + `_worker` Supervisor (`daemon heal\|dashboard\|service`), `CommandGuard` kill-switch, `--docker`/`--sandbox` test runner, & **29** `< 5ms` slash commands (`/byok`..`/voice`) |
| `src/ipc/` (`network_manager.rs` planned `PHASE_21_1`) | Single-bind `NetworkService` Network Manager (prevents `os error 10048`) + Named Pipe (`//./pipe/toksi-*`) & UDS (`~/.toksi/run/*.sock`) Direct-Wire IPC connecting supervisor, service workers, cluster & core tool subprocesses |
| `src/config/`, `src/util/ports.rs` | Canonical `~/.toksi/config.toml` parser (`[repositories]`, `[plugins]`, `[security]`, `[supervisor]`, `[testing]`, `[auth.gcp_adc]`, `[toksi_siagent_router.*]`) |
| `src/keyring/` (`adc.rs`) | OS Keychain/DPAPI encrypted secrets & concurrent standard `agy` + Google Cloud ADC (`AGY_ADC_AUTH=true`) |
| `src/db/` (`service_dbs.rs`) | Master Config DB `~/.toksi/toksi.db` (`byok_profiles` w/ multi-model/effort & sampling params, `orchestrator_mode_routes`, `cluster_nodes`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, `supervisor_workers`) separated from `~/.toksi/services/{gateway,headroom,quotas,siagent,cluster,otel}.db` |
| `src/headroom/` (`effort.rs`), `src/gateway/` (`sampling.rs`, `effort_router.rs`, `router.rs`, `local_engines.rs`, `vertex_adc.rs`, `proxy.rs`) | Shared `HeadroomEffortCompressor` (`Max\|XHigh\|High\|Medium\|Low`), strict model sampling compatibility filtering, auto `/v1` base URLs, `SemanticRouter`, 30+ local/Rust engines (`hyperflare-inference`, `oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `mistral.rs`, `candle`), & failover |
| `src/training/` | `toksi train \| eval \| dataset \| compress` (`unsloth*`, `Automodel`, `nanotron`, `Megatron-Energon`, `structured-data-models`, `kvpress`, `bitsandbytes`, `Olive`, `onnx*`, `deepeval`, `ragas`, `lighteval`) |
| `src/cluster/` | Distributed Cluster Stack (`toksi cluster`): `kubernetes`, `minikube`, `kompose`, `kops`, `helm`, `autoscaler`, `node-problem-detector`, NVCRE/`NVSentinel`/`aicr`/Knavigator/DeepOps, Kubeflow/`Run`, Spark/`cudf-spark`/`cuPhoton`, `Personal-AI-Router` LAN mesh |
| `src/projectmem/`, `src/icm/`, `src/aimemory/`, `src/contextmode/`, `src/codegraph/`, `src/rtk/` | Repo-scoped `<repo>/.toksi/{projectmem,icm,aimemory,ctxmode,codegraph,rtk}/` (`ctxmode` & `aimemory` renames + `liteparse`/`ossie*`/`auto-ontology`/45-lang Tree-sitter) |
| `src/rules/`, `src/security/`, `src/validator/`, `src/plugins/` | Externalized `<skills>` + Zero-Pollution hygiene, `<enforce>`, `defenseclaw`, `skill-scanner`, `ATR`, `openai-guardrails-python`, `PurpleLlama`, OPA/Cedar/SPIRE, Zero-Plaintext Blind Engine, Native Validator, `NeMo-Relay-Plugins` (`relay-plugin.toml`) |
| `src/agent_runtime/` (`loop_engine.rs`, `chat_ui.rs` planned `21.2`, `orchestration_resume.rs` planned `21.5`, `modes.rs`, `orchestrator.rs`, `workforce.rs`, `openspec.rs`, `sandbox.rs`, `slash.rs`) | Real `<toksi-siagent>` execution (on-demand process `21.1`), Codex/Grok Chat TUI, 22 Orchestrator Modes, Workforce (`OMA`/`swarms-rs`/`agent-orchestrator`), `OpenSpec`, `ComposeSpec`/`sbxenv.yaml`/Kit v3, & **29** slash commands |
| `src/tools/ide_plugin_host.rs` *(planned `21.3`)*, `src/agents/zoo_code.rs` *(planned `21.4`)* | 13-IDE external plugin host protocol (`toksi ide-bridge` / `:9098`) & `Zoo-Code` (`Semble` + `DCG`) connector |
| `src/telemetry/` (`otel.rs`, `evals.rs`, `tokview.rs`), `src/tui/`, `src/dashboard/` | Native `opentelemetry-rust` OTLP (`gen_ai.*`), `AutoFA`, `deepeval`, `ragas`, decoupled **10-Tab** TUI & `:9100` Web Cockpit (zero hardcoded stubs or UI CLI spawning) |
| `docker/` | `docker/Dockerfile.test` & `docker/compose.test.yaml` for live-safe isolated container testing |

## Key Verification Paths
- `crates/toksi/tests/` — **43** integration suites (including `phase19_1_to_19_6_verification` & `phase20_1_to_20_6_verification`; Wave 21 `phase21_1_to_21_6_verification` planned) tracked in `<repo>/.toksi/test-baseline.json`.
- `inspect-vault/` — **495** unique mandated upstream research repositories & companion repos (`493` Wave 20 + `2` Wave 21 reconciled: `Zoo-Code`, `agent-orchestrator`) — all `[DEEP-RESEARCH-DONE]`.
