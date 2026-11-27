# Phase 21.1 --- Single `NetworkService` Manager (`toksi _worker network-manager`) & On-Demand `<toksi-siagent>` Decoupling

> **Status (Wave 21 / v8.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

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
Plus .agents/brain/PHASE_21_1_NETWORK_SERVICE_MANAGER_AND_SIAGENT_DECOUPLING.md.

Multi-Agent Resume Rule: Check `.toksi/projectmem/plan_checkpoints/` and `.toksi/test-baseline.json` before starting; keep parallel edits strictly disjoint and flush checkpoint state before exiting on any error, network drop, or quota limit.

Execute Phase 21.1: Single Dedicated `NetworkService` Network Manager & On-Demand `<toksi-siagent>` Decoupling.
Targets:
  - crates/toksi/src/ipc/{mod.rs,transport.rs,server.rs,network_manager.rs}
  - crates/toksi/src/commands/{daemon.rs,worker.rs,agent_runtime.rs}
  - crates/toksi/src/agent_runtime/loop_engine.rs

Requirements:
1. Eliminate Windows `WSAEADDRINUSE` (`os error 10048`) port collisions (`RESEARCHPLAN.txt:635`) by introducing `WorkerRole::NetworkManager` (`toksi _worker network-manager`) in `src/ipc/network_manager.rs` as the ONLY daemon process binding TCP listeners (`:8787`, `:8788`, `:8789`, `:8790`, `:9098`, `:9099`, `:18787`; `:9100` remains detached for `toksi dashboard`).
2. Dispatch inbound HTTP/SSE/MCP/A2A/ACP/ANP/QUIC frames from `network-manager` over Named Pipe / UDS DirectWire IPC (`src/ipc/`) to isolated service and tool worker processes (`headroom`, `gateway`, `byok-gateway`, `protocol-bus`, `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`, `cluster`).
3. Remove `WorkerRole::Siagent` from `commands/daemon.rs::plan_worker_roles()` (`RESEARCHPLAN.txt:646`) so `toksi daemon start` never spawns `<toksi-siagent>`. Ensure `toksi chat|run|exec|swarm|orchestrator` launch `<toksi-siagent>` on demand in its own dedicated process and connect automatically via DirectWire IPC (or network protocol fallback when required) to running daemon services.
4. Preserve `#![deny(unsafe_code)]`, forward-slash `/` paths, zero `.unwrap()`, and verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and focused IPC/daemon tests.
```

## Technical Specifications & Verification Criteria
- `plan_worker_roles()` spawns `WorkerRole::NetworkManager` + service/tool workers without `WorkerRole::Siagent`.
- Single `TcpListener` bind ownership per port enforced inside `network_manager.rs`.
- `<toksi-siagent>` runs cleanly on demand and connects over Named Pipe / UDS IPC or gateway HTTP endpoints.
