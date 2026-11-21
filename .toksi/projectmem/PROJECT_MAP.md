# Toksi Project Map (Wave 19 / v6.0)

- **Repository Root**: `d:/AI/neo` (Three-Repository Ecosystem: `<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory` at `inspect-vault/advisory`, `<skills>` `Navegos/skills` at `inspect-vault/skills`)
- **Primary Rust Crate**: `crates/toksi` (`crates/toksi/Cargo.toml` — `456` `.rs` files, `#![deny(unsafe_code)]` with `0` unsafe blocks; leaf FFI/grammar crates `crates/toksi-sys`, `crates/toksi-grammars`)
- **Externalized Zero-Bloat Staging & Hygiene Audit Target**: `<neo>/agents/` (`28` files), `<neo>/rules/` (`24` files) outside `crates/toksi/` synced to `<skills>`; `<neo>/.agents/skills/` (`922` unadapted dirs queued for `PHASE_19_4` audit/cleanup)
- **Architecture & Phase Specs**:
  - `crates/toksi/CONSOLIDATION_PLAN.md` (Wave 19 / v6.0 Master Plan — **432** unique vaults across Sections `2.1`–`2.14`, 35-component `<toksi-siagent>`, 42 phases + 8-state roadmap, ready prompts `01`..`19.6`)
  - `crates/toksi/PLAN.md` (Wave 19 / v6.0 Architectural Blueprint, Dedicated Multi-Process `_worker` Topology & `21` slash commands)
  - `crates/toksi/research/RESEARCHPLAN.md` (Wave 19 Technical Blueprints: **432** vaults `[DEEP-RESEARCH-DONE]`, `<toksi-siagent>`, 3 repos, `ctxmode`, `_worker` IPC, `Dockerfile.test` isolation, `922`-dir cleanup, Workforce/`ANP`/`ATR`/`Training`/`Serving`)
  - `crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md` (65/90+ ecosystems, Categories 1–5)
  - `crates/toksi/research/docker_sandboxes_sbx_cli_reference.md` (`sbx` sandbox CLI parity)
  - `crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md` (`.agents` & vendor directory schemas + Category 1–3 hooks)
  - `crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md` (Zero-plaintext `BLAKE3` + `sparse::DFA`, `DisclosureGate`, 12-DB `UpstreamDeduplicator`, `CommandGuard`, & `Navegos/advisory` 3-tree shards)
  - `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md` + 42 `PHASE_*.md` files (**43** total markdown guides covering `PHASE_01`..`PHASE_19_6` with 8-State Progress Taxonomy badges)
  - `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl,plan_checkpoints/}` (Canonical workspace memory root)

## Crate Module Map (`crates/toksi/src/` & Planned Wave 19 Additions)
| Directory / Module | Role |
|---|---|
| `src/main.rs`, `src/lib.rs`, `src/commands/` (`daemon.rs`, `worker.rs` [Wave 19.2], `test_runner.rs` [Wave 19.3], `train.rs`/`serve.rs`/`eval.rs` [Wave 19.6], `guard.rs`, `compile_blind.rs`, `advisory.rs`, `skills.rs`, `sandbox.rs`, `plugin.rs`, `security.rs`, `ctx.rs`) | CLI entrypoint, Dedicated Multi-Process Supervisor (`toksi _worker <role>`), `CommandGuard` kill-switch, live-safe `--docker`/`--sandbox` test runner, & **16** current core -> **21** Wave 19 `< 5ms` slash commands (`/byok`..`/resume`, `/ctxmode`, `/workforce`, `/spec`, `/train`, `/serve`, `/voice`) |
| `src/ipc/` *(planned `PHASE_19_2`)* | Dedicated Named Pipe (`//./pipe/toksi-*`) & Unix Domain Socket (`~/.toksi/run/*.sock`) Direct-Wire IPC + `iggy` binary framing + `quinn` QUIC transport connecting supervisor, port workers (`8787..18787`), `<toksi-siagent>`, and core tool subprocesses |
| `src/config/`, `src/util/ports.rs` | Canonical `~/.toksi/config.toml` parser (`[repositories]`, `[plugins]`, `[security]`, `[supervisor]`/`[testing]` Wave 19), port allocation & `AGY_ADC_AUTH=true` config |
| `src/keyring/` (`adc.rs`) | OS Keychain/DPAPI encrypted secrets & Google Cloud ADC (`AGY_ADC_AUTH=true`) token exchange |
| `src/db/` | Unified `~/.toksi/toksi.db` (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, `supervisor_workers` [Wave 19.2]) & repo `<repo>/.toksi/` SQLite |
| `src/headroom/` (`effort.rs`) | Pure-Rust AST/TOON & `HeadroomEffortCompressor` (`Low`/`Medium`/`High`, `live_zone`, `CcrStore`, `SmartCrusher`, `TokenEconomizer`, `agent-opt`, `mcp_pruner`, `chroma`/`lance`) shared across all 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`) |
| `src/gateway/` (`providers/local/`, `effort_router.rs`, `vertex_adc.rs`, `gemini_wire.rs`, `proxy.rs`) | Dedicated port workers `:8788..:8790`, `:9098` Bus (`MCP`/`A2A`/`ACP`/`ANP`/`QUIC`), `:9099` Gateway, `:18787` BYOK/Vertex ADC (`vertex_rust_sdk`), Pingora connection pooling, & 30+ local/Rust engines (`oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `whisper.cpp`, `sd.cpp`, `triton`, `DeepSpeed-MII`) |
| `src/training/` *(planned `PHASE_19_6`)* | `toksi train \| eval` orchestration (`unsloth*`, `Automodel`, `nanotron`, `Emerging-Optimizers`, `Olive`, `onnx*`, `intel-neural-compressor`, `digestai`, `lighteval`, `clearml*`, `wandb`) + `/train` & `/eval` telemetry |
| `src/rtk/` | High-speed TOML filter & TEE recovery engine |
| `src/projectmem/` (`plan_checkpoint.rs`, `wiki_sync.rs`) | JSONL events, derived `summary.md`, `plan.md` sync & atomic `plan_checkpoints/<plan_id>.json` resume |
| `src/contextmode/` *(Wave 19.1 `ctxmode`)* | FTS5 BM25 sandbox analyzer & chunking engine (`316` legacy `context-mode` refs -> `ctxmode`, `<repo>/.toksi/ctxmode/` w/ auto-migration from `.context-mode/` & `<repo>/.toksi/context/`) |
| `src/icm/`, `src/aimemory/` (`lance_store.rs`, `qdrant_store.rs`, `cognee_graph.rs`, `temporal_graph.rs`) | Hierarchical memory + `funes` discrimination recall, hybrid BM25/vector & Lance (`0.42.0-beta.20` + `arrow-rs`) / Qdrant stores (`<repo>/.toksi/ai-memory/`) |
| `src/codegraph/` (`kernel/langs.rs`, `openwiki.rs`, `repomix.rs`, `scip.rs`, `ast_grep.rs`) | 45-language Tree-sitter indexer (`language_vue()`, `language_scss()`, `language_dockerfile()` live; `.git`-root scoped in `commands/index.rs`), `pmat` TDG complexity, call graph & OpenWiki synthesizer |
| `src/rules/` (`skills_repo.rs`, `skill_validator.rs`, `skills_catalog.rs`, `sync.rs`) | Externalized `<skills>` sync (`Navegos/skills` + `~/.toksi/user-skills`), `skilldoctor` + `agentskills` + `upskill` [Wave 19] healer/evaluator, & `<neo>/.agents/skills/` 922-dir hygiene cleaner (`PHASE_19_4`) |
| `src/security/` & `src/validator/` | OPA (`policy_opa`), Cedar (`policy_cedar`), SPIRE (`iam_attestation`), `Guardrails`/`llm-guard`/`openai-guardrails-python` [Wave 19.5], `ATR` 19,699-rule conformance [Wave 19.5], `model_spec` hierarchy, Zero-Plaintext `BLAKE3` (`1.8.7`)+`sparse::DFA`, `adr.rs`, `defenseclaw`, `skill-scanner`, 45-lang SAST, OSV/EPSS, & `Navegos/advisory` PR validator |
| `src/plugins/` | Toksi Unified Plugin System loading 7 mount surfaces (`MCP`/`A2A`/`ACP`/`ANP`/`stdio`/`SSE`/`HTTP`/`QUIC`) under `<enforce>` |
| `src/agent_runtime/` (`swarm/`, `acp.rs`, `sandbox.rs`, `workflow_state.rs`, `durable_journal.rs`, `world_model.rs`, `orchestrator.rs`, `deep_planning.rs`, `modes.rs`) | Pure-Rust 35-component `<toksi-siagent>` + Wave 19 Workforce (`OMA`, `swarms-rs`, `ChatDev`, `deer-flow`, `openworker`, `OpenSpec` `/spec`, `adk-rust`, `rig`, `maki`, `steer`, `chevalier`, `agentrix-os`, `ironclaw`, `brat`, `livekit*` `/voice`) over Direct-Wire IPC |
| `src/telemetry/`, `src/tui/`, `src/dashboard/` | Token/effort attribution (`tokview`), `langwatch`/`phoenix`/`openinference` traces, `cudarc`/`vmon` GPU probes, 9-tab mouse-capable Ratatui TUI, `:9100` Web Cockpit |
| `docker/` *(planned `PHASE_19_3`)* | `docker/Dockerfile.test` & `docker/docker-compose.test.yml` for live-safe isolated container test runs (`toksi test --docker`) |

## Key Verification Paths
- `crates/toksi/tests/` — **35** integration suites tracked in `<repo>/.toksi/test-baseline.json` (`phase1`..`phase18_6`), `benchmarks.rs`, `e2e_tests.rs`, `keyring_db_byok_verification.rs`.
- `inspect-vault/` — **432** unique mandated upstream research repositories & companion repos (`294` Waves 1–18 + `138` Wave 19 across Sections `2.1`–`2.14`) — all `[DEEP-RESEARCH-DONE]`.
