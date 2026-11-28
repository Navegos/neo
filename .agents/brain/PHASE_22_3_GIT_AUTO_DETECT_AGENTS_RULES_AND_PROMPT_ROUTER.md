# Phase 22.3: Dynamic `.git/config` Repo Auto-Detection (`No neo_dir`), `<neo>/agents/rules/` Consolidation & Non-Destructive Prompt Router

> **Status (Wave 22 / v9.0 Audit):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```markdown
Implement Phase 22.3 across `crates/toksi/src/config/mod.rs`, `src/rules/skills_repo.rs`, `src/rules/sync.rs`, `crates/toksi/full_example_config.toml`, `src/agent_runtime/prompt_router.rs`, and `src/commands/refactor_router.rs`.

Mandatory Pre-Read Set:
- `crates/toksi/research/RESEARCHPLAN.md` (Sections 1.3.4, 4.3, 5.3, 9.6, 12)
- `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md` & `crates/toksi/README.md`
- `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`
- `crates/toksi/research/{docker_cli_compose_yaml_full_reference,docker_sandboxes_sbx_cli_yaml_reference,folders_structures_default_knowledge_used_by_the_known_cli_agents_ides,full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture,open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator,toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping}.md`
- `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`
- `.agents/rules/ascii-only.md`
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`

Multi-Agent Resume Rule: Never test against real `~/.toksi/config.toml` or `<neo>/.git/` writes; isolate every test in `TempDir` with fixture `.git/config` files and checkpoint progress in `orchestration_journal.jsonl`.

Targets:
1. Remove hardcoded `neo_dir` from `src/config/mod.rs` (`[repositories]`), `src/rules/skills_repo.rs`, and `crates/toksi/full_example_config.toml` (parse legacy `neo_dir` only as deprecated alias warning if present in user TOML, or remove cleanly per config contract with clear migration hint).
2. Implement automatic `.git/config` remote/root discovery (`Navegos/neo`, `Navegos/advisory`, `Navegos/skills` + arbitrary workspace repos) without hardcoded OS paths.
3. Point all rules sync, lookup, and validation paths in `src/rules/` to `<neo>/agents/rules/` and `<neo>/agents/agent_instructions.md` instead of nonexistent `<neo>/rules/`.
4. Implement `src/agent_runtime/prompt_router.rs` + `src/commands/refactor_router.rs` (`toksi refactor-router`) classifying monolithic prompts into Core Identity, Decision Matrices, Technical Specs, Tool Schemas, Examples, and Verification Gate without deleting domain detail.

Verification:
- Tests verifying `neo_dir`-free config parsing, `.git/config` remote auto-detection, `<neo>/agents/rules/` discovery, and lossless `toksi refactor-router` section extraction.
- Run `cargo check -p toksi`, `cargo clippy -p toksi --tests -- -D warnings`, and pure-ASCII check (0 bytes > 0x7F).
```

## Technical Specifications & Verification Criteria
- **Zero Hardcoded Local Paths**: Zero `C:/` or `d:/AI/neo` production defaults; 100% `.git/config` + `dirs::home_dir()` discovery.
- **Verification**: `cargo test -p toksi rules` and `cargo test -p toksi config` pass with 0 warnings.
