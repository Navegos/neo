# Toksi Consolidated Master Plan (Wave 20 / v7.0)

- **Target Crate**: `crates/toksi` (`d:/AI/neo`) across **3-repository ecosystem** (`<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, `<skills>` `Navegos/skills`)
- **Master Architecture**: `crates/toksi/CONSOLIDATION_PLAN.md`, `crates/toksi/PLAN.md`, `crates/toksi/research/RESEARCHPLAN.md`, `crates/toksi/research/docker_cli_compose_yaml_full_reference.md`, `crates/toksi/research/docker_sandboxes_sbx_cli_yaml_reference.md`, `crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md`, `crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture.md`, `crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md`, `crates/toksi/research/toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md`, `crates/toksi/research/toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml`
- **Audited Vaults**: **493** unique mandated `inspect-vault/*` repositories (`294` Waves 1–18 + `138` Wave 19 + `61` Wave 20 across Sections `2.1`–`2.17`) — all **493** `[DEEP-RESEARCH-DONE]` powering the **35-component `<toksi-siagent>`**, **11-Pillar AI/ML Lifecycle**, and **Distributed Cluster Stack (`src/cluster/`, `toksi cluster`)**
- **Daemon, Multi-Process Supervisor & Decoupled Cockpit**: **Dedicated Subprocess Supervisor (`toksi _worker <role>`) + Named Pipe/UDS `DirectWireBus` (`src/ipc/`)** (`toksi daemon start|stop|restart|heal|status|dashboard|service <name> <action>`); `:9100` Web Cockpit detached from `daemon start`; config in `~/.toksi/toksi.db`, per-service telemetry in `~/.toksi/services/*.db`; all 8 ports pass through `HeadroomEffortCompressor` (`Max|XHigh|High|Medium|Low`, default `Medium`) with per-model sampling filtering, auto `/v1` URL completion, and dual concurrent `agy` + `AGY_ADC_AUTH=true` Vertex ADC
- **Renames & Isolation Mandates**: `context-mode` / `context` -> **`ctxmode`** & `ai-memory` -> **`aimemory`** (`<repo>/.toksi/{ctxmode,aimemory}/` w/ auto-migration); `docker/Dockerfile.test` + `docker/compose.test.yaml` + `sbxenv.yaml`/Kit v3 (`toksi test --docker|--sandbox`); Zero-Pollution Cleanup of `<neo>/.agents/skills/` (**933** items / **922** dirs)
- **Target Universal Slash Commands (`< 5ms`) & 10-Tab UI**: **29 commands** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/mode`, `/sampling`, `/router`, `/projectmem`, `/ctxmode`, `/codegraph`, `/icm`, `/aimemory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, `/workforce`, `/spec`, `/dataset`, `/train`, `/eval`, `/compress`, `/serve`, `/cluster`, `/otel`, `/voice`); **10-Tab UI** (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`)

---

## Implementation Phases Checklist (8-State Progress Taxonomy)

### Waves 1–14: Core Pure-Rust Consolidation (`PHASE_01`..`PHASE_14`)
- [x] **Deep Research (Wave 20 Complete)**: All **493 unique vaults** (`inspect-vault/*` Sections `2.1`–`2.17`) source-inspected `[DEEP-RESEARCH-DONE]`.
- [x] **Phase 1, 2, 4, 5, 7, 8, 10, 11, 14**: Core pure-Rust implementations live `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`.
- [x] **Phase 9 (Healing)**: Compile/clippy zero-warning `#![deny(unsafe_code)]` baseline `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`.
- [x] **Phase 3 (`ctxmode` Rename), Phase 6 (`<skills>` Cleanup), Phase 12 (10-Tab/Service DB UI)**: `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — Completed in `PHASE_19_1`, `PHASE_19_4`, `PHASE_20_2`.
- [x] **Phase 13 (Full E2E)**: `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — Completed in `PHASE_19_3` & `PHASE_20_3`.

### Waves 15–18: Enterprise Topology, Protocols, Config, Plugins, Security & 35-Component `<toksi-siagent>`
- [x] **Phase 15.3, 15.5, 15.6, 17.1–17.3, 17.6, 18.1–18.6**: Implemented/Fixed baseline across gateway, 45-lang `langs.rs`, plugins, blind engine, validator, ADC, checkpoints, ADR, 5-backend sandbox, `CommandGuard`, and 35-comp `<toksi-siagent>` (`[PLAN-IMPLEMENTED]`/`[PLAN-FIXED]`).
- [x] **Phase 15.1, 15.2, 16.1, 16.2, 17.4, 17.5 (`[PLAN-FIXED]`) & Phase 15.4, 16.3, 17.7 (`[PLAN-IMPLEMENTED]`)**: Remediated and verified across Waves 19–20.

### Wave 19 (v6.0): Renames, `_worker` IPC, Docker Tests, Skills Hygiene, Workforce/`ANP`/ATR & Training/Serving (`PHASE_19_1`..`PHASE_19_6`)
- [x] **Phase 19.1 (`[PLAN-FIXED]`)**: `ctxmode` & `aimemory` canonical rename + SQLite auto-migration.
- [x] **Phase 19.2 (`[PLAN-IMPLEMENTED]`)**: `toksi _worker <role>` subprocesses + Named Pipe/UDS `src/ipc/` Direct-Wire IPC + `supervisor_workers` table.
- [x] **Phase 19.3 (`[PLAN-IMPLEMENTED]`)**: `docker/Dockerfile.test` + `toksi test --docker|--sandbox` + ephemeral `TOKSI_HOME`/`:0` isolation.
- [x] **Phase 19.4 (`[PLAN-FIXED]`)**: Zero-Pollution audit & cleanup of `<neo>/.agents/skills/` + externalized `<skills>` & `<advisory>` sync.
- [x] **Phase 19.5 (`[PLAN-IMPLEMENTED]`)**: Workforce (`/workforce`), `OpenSpec` (`/spec`), `ANP` + `QUIC` (`:9098`), ATR + OpenAI Guardrails + `model_spec` (`91` vaults).
- [x] **Phase 19.6 (`[PLAN-IMPLEMENTED]`)**: `src/training/` + `toksi train|serve|eval` + `/train`/`/serve`/`/voice` (`oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `safetensors`, `whisper*`, `sd.cpp`, `cudarc`, `unsloth*`, `Automodel`, `nanotron`, `onnx*`, `Olive`, `lighteval`, `langwatch`, `wandb`, `clearml*`, `livekit*`).

### Wave 20 (v7.0): Real `_worker siagent`, 22 Modes, 29 Slash Cmds, 10-Tab UI, Compose/`sbxenv`, 11 Pillars, Cluster Stack & OpenTelemetry (`PHASE_20_1`..`PHASE_20_6`)
- [x] **Phase 20.1 (`[PLAN-IMPLEMENTED]`)**: Killed `loop_engine.rs` mock stubs, wired `_worker siagent`, 22 Orchestrator Modes (`[toksi_siagent_router.*]`), 5 efforts (`Max..Low`), sampling compatibility matrix, and **29** slash commands.
- [x] **Phase 20.2 (`[PLAN-FIXED]`)**: Detached `:9100` (`daemon start`), added `daemon heal|dashboard|service`, separated `~/.toksi/services/*.db`, auto `/v1`, dual `agy`/ADC, removed `"opencode"` stub, delivered **10-Tab UI**.
- [x] **Phase 20.3 (`[PLAN-IMPLEMENTED]`)**: `docker/compose.test.yaml` + pure-Rust `ComposeSpec`, `sbxenv.yaml` v1 & Kit v3 sandbox engine.
- [x] **Phase 20.4 (`[PLAN-IMPLEMENTED]`)**: 11-Pillar AI/ML Lifecycle & Wave 20 vaults (`liteparse`, `hyperflare-inference`, `NeMo-Relay-Plugins`, `kvpress`, `bitsandbytes`, `Megatron-Energon`, `ossie*`, `auto-ontology`, `semantic-router`, `AutoFA`, `deepeval`, `ragas`, `PurpleLlama`; `toksi dataset|compress|eval`).
- [x] **Phase 20.5 (`[PLAN-IMPLEMENTED]`)**: Distributed Cluster Stack (`src/cluster/`, `toksi cluster`): K8s/Minikube/Kompose/Kops/Helm/Autoscaler/NPD, NVCRE/`NVSentinel`/`aicr`/Knavigator/DeepOps, Kubeflow/`Run`, Spark/`cudf-spark`/`cuPhoton`, `Personal-AI-Router` LAN mesh.
- [x] **Phase 20.6 (`[PLAN-IMPLEMENTED]`)**: OpenTelemetry Native LLMOps (`src/telemetry/otel.rs`, `opentelemetry-rust`, `opentelemetry-collector`, `opentelemetry-operator`, `opentelemetry-python-genai` `gen_ai.*`), integration suites & 49-file 8-state sync.
