# Phase 20.1 — Real `<toksi-siagent>` Subprocess, 22-Mode Router, 5 Efforts, Sampling Matrix & 29 Slash Commands

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

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
Plus .agents/brain/PHASE_20_1_REAL_SIAGENT_SUBPROCESS_22_MODE_ROUTER_AND_29_SLASH_CMDS.md.

Execute Phase 20.1: Kill Local Stubs & Wire Real `<toksi-siagent>` Subprocess (`toksi _worker siagent`), 22-Mode Orchestrator Router, 5 Effort Tiers, Sampling Compatibility Matrix & 29 Slash Commands.
Targets:
  - crates/toksi/src/agent_runtime/{loop_engine.rs,modes.rs,orchestrator.rs,slash.rs}
  - crates/toksi/src/commands/{agent_runtime.rs,orchestrator.rs}
  - crates/toksi/src/gateway/{effort_router.rs,sampling.rs,proxy.rs}
  - crates/toksi/src/headroom/effort.rs
  - crates/toksi/src/config/mod.rs (`[toksi_siagent_router.*]`)
  - crates/toksi/src/db/{schema.rs,operations.rs} (`orchestrator_mode_routes`)
  - crates/toksi/src/main.rs
  - crates/toksi/tests/phase20_1_to_20_6_verification.rs

Requirements:
1. Eliminate local string-echo/mock-swarm stubs in `crates/toksi/src/agent_runtime/loop_engine.rs:173-176, 249-251`. Dispatch `toksi chat|run|exec|swarm` into `toksi _worker siagent` via `src/ipc/` Direct-Wire IPC and real Gateway HTTP/SSE inference (`:9099` / `:18787` / custom `base_url`), failing explicitly when no live gateway or BYOK provider is reachable.
2. Implement all 22 Orchestrator Modes across 7 Cognitive Domains (`[toksi_siagent_router.*]` user-overrideable in `~/.toksi/config.toml` and `orchestrator_mode_routes` in `~/.toksi/toksi.db`) matching `toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md` and `toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml`.
3. Expand `EffortLevel` to 5 tiers (`Low | Medium | High | XHigh | Max`, default `Medium`) across all 8 ports, `<toksi-siagent>`, and training/inference/fine-tuning.
4. Implement strict per-provider/model `(temperature, top_p, top_k)` sampling parameter compatibility filtering (`gateway/sampling.rs`) per `full_ai_ecosystem_..._protocols_gateways_and_quota_architecture.md`, silently omitting unsupported parameters rather than failing upstream APIs.
5. Expand `CORE_SLASH_COMMANDS` to all 29 unified `< 5ms` slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/mode`, `/sampling`, `/router`, `/projectmem`, `/ctxmode`, `/codegraph`, `/icm`, `/aimemory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, `/workforce`, `/spec`, `/dataset`, `/train`, `/eval`, `/compress`, `/serve`, `/cluster`, `/otel`, `/voice`), keeping `/context` and `/ai-memory` aliases functional.
6. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and focused runtime/gateway tests.
```

## Technical Specifications & Verification Criteria
- **Zero Mock Stubs:** `AgentRuntimeEngine::run_single_shot()` and `run_swarm()` dispatch real compressed/filtered requests through IPC/Gateway HTTP endpoints.
- **22 Orchestrator Modes across 7 Cognitive Domains:** Meta-Cognition & Governance (4), Memory & World Modeling (3), Discovery & Epistemics (4), Pedagogy & Knowledge Transfer (2), Strategy & Architecture (2), Autonomous Execution (3), Verification & Resilience (4).
- **29 Slash Commands:** Exact 29-command registry in `src/agent_runtime/slash.rs`.
