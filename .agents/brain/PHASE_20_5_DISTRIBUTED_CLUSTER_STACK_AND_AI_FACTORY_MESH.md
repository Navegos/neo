# Phase 20.5 — Toksi Distributed Cluster Stack (`src/cluster/` & `toksi cluster` — 26 Sec 2.15 Vaults)

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
Plus .agents/brain/PHASE_20_5_DISTRIBUTED_CLUSTER_STACK_AND_AI_FACTORY_MESH.md.

Execute Phase 20.5: Build Toksi Distributed Cluster Stack (`src/cluster/` & `toksi cluster`) Unifying All 26 Section 2.15 Kubernetes, Kubeflow, AI-Factory, Spark & LAN PAIR Vaults.
Sources (all verified `[DEEP-RESEARCH-DONE]` in RESEARCHPLAN.md Section 2.15):
  `kubernetes`, `minikube`, `kompose`, `kops`, `helm`, `autoscaler`, `node-problem-detector`, `cluster-readiness-engine` (NVCRE), `NVSentinel`, `aicr`, `knavigator`, `AI-Factory-Operations-Agent`, `deepops`, Kubeflow (`hub`, `pipelines`, `trainer`, `sdk`, `kale`, `spark-operator`, `dashboard`, `notebooks`, `semantic-operator`, archival `katib` HPO ref), `Run` (`nemo_run`), `spark`, `cudf-spark`, plus `Personal-AI-Router` (`PAIR` LAN mesh).
Targets:
  - crates/toksi/src/cluster/{mod.rs,topology.rs,kubernetes.rs,readiness.rs,kubeflow.rs,spark.rs,lan_pair.rs}
  - crates/toksi/src/commands/cluster.rs
  - crates/toksi/src/agent_runtime/cluster_exec.rs & crates/toksi/src/gateway/router.rs
  - crates/toksi/src/db/{schema.rs,operations.rs,service_dbs.rs} (`cluster_nodes`, `~/.toksi/services/cluster.db`)
  - crates/toksi/src/dashboard/{api.rs,server.rs} (`/api/cluster/*`)
  - crates/toksi/src/main.rs & crates/toksi/src/lib.rs

Requirements:
1. Build pure-Rust `src/cluster/` supporting `Local | LanPair | Kubernetes | Slurm | Ssh` backends persisted in `toksi.db` `cluster_nodes` and `~/.toksi/services/cluster.db`.
2. Implement NVIDIA AI Factory operations: NVCRE readiness gates (`readiness.rs`), `NVSentinel` auto-quarantine state machine (`healthy|warning|faulted`), `aicr` cluster recipe manifests, `knavigator`/`autoscaler`/`node-problem-detector`, `kompose` Compose-to-K8s translator, Kubeflow `TrainJob`/Pipelines/ModelRegistry specs, `Run` (`nemo_run`), `spark`/`cudf-spark` job specs, and `Personal-AI-Router` LAN GPU mesh failover.
3. Expose `toksi cluster status|nodes|readiness|nvsentinel|recipe|kubeflow|spark|lan|kompose` CLI and `/cluster` slash command + `/api/cluster/*` endpoints.
4. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and cluster unit/integration tests.
```

## Technical Specifications & Verification Criteria
- Zero unsafe code; `PATH`-checked external `kubectl`/`helm`/`minikube` invocation only with graceful diagnostics when absent.
- `cluster_nodes` persisted in `ToksiDbDump` export/import roundtrips.
