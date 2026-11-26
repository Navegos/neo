# Phase 20.2 â€” `daemon heal|dashboard|service`, Detached `:9100`, Per-Service DBs, Auto `/v1`, Dual `agy`/ADC & 10-Tab UI

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
Plus .agents/brain/PHASE_20_2_DAEMON_HEAL_SERVICE_PER_SERVICE_DBS_AND_10TAB_UI.md.

Execute Phase 20.2: Detach `:9100` Cockpit, Add `toksi daemon heal|dashboard|service`, Separate Per-Service DBs, Auto `/v1`, Dual `agy`/ADC & Deliver 10-Tab UI.
Targets:
  - crates/toksi/src/commands/daemon.rs
  - crates/toksi/src/db/{mod.rs,schema.rs,operations.rs,byok.rs,service_dbs.rs}
  - crates/toksi/src/util/byok.rs
  - crates/toksi/src/keyring/adc.rs & crates/toksi/src/gateway/vertex_adc.rs
  - crates/toksi/src/tui/{app.rs,mod.rs,views/*}
  - crates/toksi/src/dashboard/{api.rs,server.rs,assets/{index.html,app.js,style.css}}
  - crates/toksi/tests/{phase12_tui_dashboard_verification.rs,phase15_2_unified_8port_daemon_verification.rs,phase16_ui_and_open_agents_verification.rs,phase17_7_ui_parity_and_test_baseline_verification.rs}

Requirements:
1. Detach `:9100` Web Cockpit (`DaemonServicePlan::dashboard = false` on `toksi daemon start`; launch only via `toksi dashboard` or `toksi daemon dashboard`). Implement `toksi daemon heal` and `toksi daemon service <name> <start|stop|restart|heal|status>`.
2. Separate per-service telemetry DBs (`~/.toksi/services/{gateway,headroom,quotas,siagent,cluster,otel}.db`) from `~/.toksi/toksi.db` configuration state; extend `byok_profiles` with `models_json`, `efforts_json`, `temperature`, `top_p`, `top_k`.
3. Auto-complete `/v1` base URLs cleanly (`normalize_provider_base_url`); support concurrent standard `agy` CLI OAuth/subscription and `AGY_ADC_AUTH=true` Vertex ADC without overwriting either.
4. Remove the hardcoded `"opencode" | "opencode2" => ("DeepSeek", "deepseek-r1", "[BYOK]")` stub in `dashboard/api.rs` and guarantee TUI/Web-UI never spawn `opencode` or external agent CLIs.
5. Upgrade TUI (`src/tui/`) and Web Cockpit (`:9100`) to exact 10-Tab parity (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`), wired to `GET /api/quotas`, `GET /api/activity`, `GET/PUT /api/orchestrator/modes`, `GET/POST /api/daemon/service/:name/:action`, `GET/POST /api/cluster/*`, `GET /api/otel/status`.
6. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and TUI/dashboard/daemon verification suites.
```

## Technical Specifications & Verification Criteria
- `resolve_daemon_plan()` always sets `dashboard = false`.
- `App::tab_names` and Web Cockpit nav expose all 10 decoupled tabs (`Quotas` separated from `AI Agent Fleet` and `AI Gateway`).
