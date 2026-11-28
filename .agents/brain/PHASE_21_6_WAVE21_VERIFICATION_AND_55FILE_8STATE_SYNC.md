# Phase 21.6 --- Wave 21 Integration Verification & 55-File 8-State Master Synchronization

> **Status (Wave 21 / v8.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]

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
Plus .agents/brain/PHASE_21_6_WAVE21_VERIFICATION_AND_55FILE_8STATE_SYNC.md.

Multi-Agent Resume Rule: Check `.toksi/projectmem/plan_checkpoints/` and `.toksi/test-baseline.json` before starting; keep parallel edits strictly disjoint and flush checkpoint state before exiting on any error, network drop, or quota limit.

Execute Phase 21.6: Comprehensive Wave 21 Verification Suite & 55-File 8-State Master Synchronization.
Targets:
  - crates/toksi/tests/phase21_1_to_21_6_verification.rs
  - .toksi/test-baseline.json
  - All 6 master docs (`RESEARCHPLAN.md`, `CONSOLIDATION_PLAN.md`, `PLAN.md`, `README.md`, `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}`) + all 55 `.agents/brain/*.md` guides

Requirements:
1. Author `crates/toksi/tests/phase21_1_to_21_6_verification.rs` (ephemeral `TOKSI_HOME` & `:0` ports, zero host interference) verifying:
   - Single `NetworkService` (`_worker network-manager`) socket ownership + DirectWire routing + `daemon start` excluding `WorkerRole::Siagent`.
   - Codex/Grok-Build Chat TUI controller state machine + in-chat CLI command execution.
   - External 13-IDE plugin host manifest & JSON-RPC handshake (`vscode`..`lapce`).
   - `Zoo-Code` (`Semble`, `DCG`, `.roomodes`), `agent-orchestrator` (git-worktree Kanban lanes), & `total_mandated_vault_count() == 495`.
   - Multi-agent `OrchestrationJournal`, disjoint `WorkerLeaseRegistry`, 5-store pre-failure flush, and cross-chat `/resume`.
2. Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (zero warnings, zero relaxed assertions), refresh `.toksi/test-baseline.json`, and synchronize all 55 `.agents/brain/*.md` files & 6 master docs with verified `[PLAN-IMPLEMENTED]` / `[PLAN-FIXED]` 8-state badges.
```

## Technical Specifications & Verification Criteria
- Full `cargo test --workspace` passes with zero warnings (`-D warnings`), `0` `unsafe` blocks in `crates/toksi/src/`, zero `.unwrap()` in production paths, and 100% 55-file 8-state documentation parity.
