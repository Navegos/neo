# Phase 20.4 â€” 11-Pillar AI/ML Lifecycle & Wave 20 Rust/RAG/Compression/Eval/Security Vaults

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read ALL 12 canonical workspace files (plus the companion router TOML) FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. .toksi/projectmem/plan.md
5. .toksi/projectmem/PROJECT_MAP.md
6. .toksi/projectmem/summary.md
7. crates/toksi/research/docker_cli_compose_yaml_full_reference.md
8. crates/toksi/research/docker_sandboxes_sbx_cli_yaml_reference.md
9. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
10. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture.md
11. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
12. crates/toksi/research/toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md
13. crates/toksi/research/toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml
Plus .agents/brain/PHASE_20_4_11_PILLAR_AIML_LIFECYCLE_AND_WAVE20_VAULTS.md.

Execute Phase 20.4: 11-Pillar AI/ML Lifecycle & Wave 20 Rust, Dataset, RAG, Compression, Eval & Security Vaults (26 Sec 2.16 + 4 Sec 2.17 non-OTel Vaults).
Sources (all verified `[DEEP-RESEARCH-DONE]` in RESEARCHPLAN.md Sections 2.16 & 2.17):
  `ossie`, `ossie-converters`, `auto-ontology`, `structured-data-models`, `Megatron-Energon`, `cuPhoton`, `hyperflare-inference`, `kvpress`, `bitsandbytes`, `Personal-AI-Router`, `semantic-router`, `NeMo-Agent-Toolkit`, `NeMo-Agent-Toolkit-UI`, `NeMo-Relay-Plugins`, `elements`, `llama_index`, `llama-agents`, `liteparse`, `haystack`, `prompt-ops`, `llama`, `llama-models`, `ogx`, `ogx-client-go`, `vscode`, `aider`, `AutoFA`, `deepeval`, `ragas`, `PurpleLlama`.
Targets:
  - crates/toksi/src/training/{dataset.rs,compress.rs}
  - crates/toksi/src/telemetry/evals.rs (`DeepEvalMetric`, `RagasScorecard`, `AutoFaFailureClusterer`)
  - crates/toksi/src/gateway/{router.rs,local_engines.rs} (`SemanticRouter`, `hyperflare-inference`, `SECTION_2_15_TO_2_17_VAULTS`)
  - crates/toksi/src/plugins/manifest.rs (`relay-plugin.toml` + SHA-256 checksum verification)
  - crates/toksi/src/aimemory/retriever.rs & crates/toksi/src/security/purple_llama.rs
  - crates/toksi/src/commands/{dataset.rs,compress.rs,eval.rs}
  - crates/toksi/src/main.rs

Requirements:
1. Build Pillar 1 Dataset & Ingestion contracts (`Megatron-Energon`, `structured-data-models`, `ossie*`, `auto-ontology`, `liteparse`) exposed via `toksi dataset inspect|validate|ingest|ontology` and `/dataset`.
2. Build Pillar 6 Eval & AutoFA failure-clustering contracts (`deepeval`, `ragas`, `AutoFA`, `lighteval`) exposed via `toksi eval` and `/eval`.
3. Build Pillar 7 Compression (`kvpress` eviction policies, `bitsandbytes` QLoRA/int8) exposed via `toksi compress plan|kvpress|quant` and `/compress`.
4. Integrate `NeMo-Relay-Plugins` (`relay-plugin.toml`), `semantic-router`, `prompt-ops`, `hyperflare-inference` (SM120/SM121), `llama_index`/`llama-agents`/`haystack`/`NeMo-Agent-Toolkit`/`aider`/`vscode`/`elements`, and defensive `PurpleLlama` (`LlamaGuard` S1-S14 + `CodeShield` + `LlamaFirewall`) under `<enforce>`.
5. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and 11-pillar unit/integration tests.
```

## Technical Specifications & Verification Criteria
- All 493 unique mandated vaults indexed cleanly across `gateway/local_engines.rs` / `agent_runtime/fleet.rs`.
- `toksi dataset`, `toksi compress`, `toksi eval` CLI subcommands and slash handlers `/dataset`, `/compress`, `/eval` work end-to-end.
