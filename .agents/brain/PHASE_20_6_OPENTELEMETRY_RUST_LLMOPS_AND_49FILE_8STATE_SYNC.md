# Phase 20.6 â€” OpenTelemetry Native LLMOps (`opentelemetry-rust`) & 49-File 8-State Master Sync

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
Plus .agents/brain/PHASE_20_6_OPENTELEMETRY_RUST_LLMOPS_AND_49FILE_8STATE_SYNC.md.

Execute Phase 20.6: OpenTelemetry Native LLMOps (`opentelemetry-rust`, `opentelemetry-collector`, `opentelemetry-operator`, `opentelemetry-python*`), Full Integration Suites & 49-File 8-State Master Synchronization.
Sources (Section 2.17): `opentelemetry-rust`, `opentelemetry-collector`, `opentelemetry-operator`, `opentelemetry-python`, `opentelemetry-python-genai`.
Targets:
  - crates/toksi/src/telemetry/otel.rs & crates/toksi/src/cluster/otel_pipeline.rs
  - crates/toksi/src/commands/otel.rs & crates/toksi/src/main.rs
  - crates/toksi/src/dashboard/{api.rs,server.rs} (`/api/otel/status`)
  - crates/toksi/tests/{phase19_1_to_19_6_verification.rs,phase20_1_to_20_6_verification.rs}
  - .toksi/test-baseline.json (refresh BLAKE3 hashes for all test suites)
  - All 6 canonical plan targets (`RESEARCHPLAN.md`, `CONSOLIDATION_PLAN.md`, `PLAN.md`, `README.md`, `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md,events.jsonl}`) + all 49 `.agents/brain/*.md` guides.

Requirements:
1. Implement pure-Rust OTLP/HTTP JSON span/metric exporter (`gen_ai.system`, `gen_ai.operation.name`, `gen_ai.request.model`, `gen_ai.request.temperature`, `gen_ai.request.top_p`, `gen_ai.usage.*`, `toksi.effort`, `toksi.mode`, `toksi.headroom.bytes_saved`) buffering to `~/.toksi/services/otel.db` with non-blocking fail-open export, plus `opentelemetry-collector` YAML & `opentelemetry-operator` CRD generators.
2. Expose `toksi otel status|traces|export|collector-config|operator-crd` CLI, `/otel` slash command, and `GET /api/otel/status`.
3. Add comprehensive live-safe integration test suites `phase19_1_to_19_6_verification.rs` and `phase20_1_to_20_6_verification.rs`, refresh `.toksi/test-baseline.json`, and run full workspace checks (`cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`).
4. Synchronize ALL 6 canonical plan files and ALL 49 `.agents/brain/*.md` guides using the exact verified **8-State Progress Taxonomy**.
```

## Technical Specifications & Verification Criteria
- Full `cargo test --workspace` passes with zero warnings (`-D warnings`), `0` `unsafe` blocks in `crates/toksi/src/`, zero `.unwrap()` in non-test production code, and 100% 55-file 8-state documentation parity.
