# Phase 20.3 --- Docker CLI/Compose OS Virtualization & `sbxenv.yaml`/Kit v3 Sandbox Engine

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]

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
Plus .agents/brain/PHASE_20_3_DOCKER_COMPOSE_VIRTUALIZATION_AND_SBXENV_KIT_V3.md.

Execute Phase 20.3: Docker CLI/Compose OS Virtualization (`compose.test.yaml`) & `sbxenv.yaml`/Kit v3 Declarative Sandbox Engine.
Targets:
  - docker/compose.test.yaml
  - crates/toksi/src/agent_runtime/sandbox.rs
  - crates/toksi/src/commands/{sandbox.rs,test_runner.rs}
  - crates/toksi/tests/phase20_1_to_20_6_verification.rs

Requirements:
1. Author `docker/compose.test.yaml` paired with `docker/Dockerfile.test` providing isolated bridge networking, tmpfs `TOKSI_HOME`, resource limits, and zero host `~/.toksi` mounts.
2. Implement pure-Rust `ComposeSpec` parser/validator (`parse_compose_yaml`, `validate_compose_spec`) in `src/agent_runtime/sandbox.rs` honoring `docker_cli_compose_yaml_full_reference.md` and enforcing SSRF/privileged-mount safety.
3. Implement pure-Rust `SbxEnvManifest` (`schemaVersion: "1"`) and `SbxKitV3Manifest` (`schemaVersion: "3"`) parsers, validators, and `sbx` CLI builders honoring `docker_sandboxes_sbx_cli_yaml_reference.md`.
4. Expose `toksi sandbox compose <validate|up|down|test>` and `toksi sandbox sbxenv <validate|plan>` in `src/commands/sandbox.rs`.
5. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and sandbox/Compose/Kit v3 unit & integration tests.
```

## Technical Specifications & Verification Criteria
- `docker/compose.test.yaml` parses cleanly via `parse_compose_yaml` and passes `validate_compose_spec`.
- `sbxenv.yaml` v1 and Kit v3 (`schemaVersion: "3"`) schemas validate with fail-closed SSRF and privilege checks.
