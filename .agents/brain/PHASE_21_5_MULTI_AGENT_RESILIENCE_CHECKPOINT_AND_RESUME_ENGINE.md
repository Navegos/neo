# Phase 21.5 — Multi-Agent Orchestration Resilience, Fault/Quota Checkpoint & Cross-Chat Resume Engine

> **Status (Wave 21 / v8.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read ALL 15 canonical workspace files FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. crates/toksi/README.md
5. .toksi/projectmem/plan.md
6. .toksi/projectmem/PROJECT_MAP.md
7. .toksi/projectmem/summary.md
8. crates/toksi/research/docker_cli_compose_yaml_full_reference.md
9. crates/toksi/research/docker_sandboxes_sbx_cli_yaml_reference.md
10. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
11. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture.md
12. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
13. crates/toksi/research/toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md
14. crates/toksi/research/toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml
15. .agents/brain/INDEX_AND_EXECUTION_GUIDE.md
Plus .agents/brain/PHASE_21_5_MULTI_AGENT_RESILIENCE_CHECKPOINT_AND_RESUME_ENGINE.md.

Multi-Agent Resume Rule: Check `.toksi/projectmem/plan_checkpoints/` and `.toksi/test-baseline.json` before starting; keep parallel edits strictly disjoint and flush checkpoint state before exiting on any error, network drop, or quota limit.

Execute Phase 21.5: Build Bulletproof Multi-Agent Orchestration Resilience & Cross-Chat Resume Engine (`src/agent_runtime/orchestration_resume.rs`).
Sources: neo/inspect-vault/{temporal,restate,langgraph,chevalier,brat,projectmem,icm,ai-memory,context-mode,codegraph}
Targets:
  - crates/toksi/src/agent_runtime/{orchestration_resume.rs,loop_engine.rs,orchestrator.rs,mod.rs}
  - crates/toksi/src/projectmem/plan_checkpoint.rs
  - crates/toksi/src/commands/agent_runtime.rs

Requirements:
1. Implement `OrchestrationJournal` and `WorkerLeaseRegistry` in `src/agent_runtime/orchestration_resume.rs` persisted strictly inside `<repo>/.toksi/projectmem/`:
   - `orchestration_journal.jsonl`: `fs2`-locked write-ahead log of worker spawn, lease claim, step completion, failure classification (`network_drop`, `http_429`, `quota_exhausted`, `tool_error`, `user_interrupt`), and BYOK failover events.
   - `worker_leases.json`: disjoint file-path ownership lock table guaranteeing parallel subagents/swarm workers never collide on the same target file.
   - Atomic `plan_checkpoints/<plan_id>.json`: includes `dirty_files_blake3`, `active_phase`, `completed_steps`, `pending_steps`, and `active_byok_profile`.
2. Implement 5-store pre-failure atomic flush (`projectmem`, `icm`, `aimemory`, `ctxmode`, `codegraph`) triggered automatically on `StopFailure`, network drop, HTTP 429, context-limit `CompactPre`, or quota exhaustion, paired with L0/L1/L2 BYOK failover rotation (`gateway/cascade.rs`).
3. Upgrade `AgentRuntimeEngine::resume_plan()`, `toksi agent resume [<plan_id>]`, and `/resume` to validate `git status` + BLAKE3 hashes, release stale dead-worker leases, skip already-green `.toksi/test-baseline.json` suites, and resume strictly from the first uncompleted step across chat restarts.
4. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and multi-agent fault-injection / resume tests.
```

## Technical Specifications & Verification Criteria
- Simulated network drop, HTTP 429, and worker error cleanly persist `orchestration_journal.jsonl` + `plan_checkpoints/<id>.json` and resume from exact pending step in a new engine instance.
- Overlapping file-lease claims across two concurrent workers fail fast before any file write occurs.
