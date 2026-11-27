# Phase 21.3 --- External 13-IDE Plugin Host Protocol (`toksi ide-bridge` / `:9098`)

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
Plus .agents/brain/PHASE_21_3_EXTERNAL_13_IDE_PLUGIN_HOST_PROTOCOL.md.

Multi-Agent Resume Rule: Check `.toksi/projectmem/plan_checkpoints/` and `.toksi/test-baseline.json` before starting; keep parallel edits strictly disjoint and flush checkpoint state before exiting on any error, network drop, or quota limit.

Execute Phase 21.3: Standardize External 13-IDE Plugin Host Protocol (`src/tools/ide_plugin_host.rs`, `toksi ide-bridge`).
Sources: neo/inspect-vault/{agent-client-protocol,vscode,cline,Zoo-Code,continue}
Targets:
  - crates/toksi/src/tools/{ide_plugin_host.rs,multi_protocol.rs,mod.rs}
  - crates/toksi/src/commands/mod.rs
  - crates/toksi/src/main.rs

Requirements:
1. Define `IdePluginKind` and standardized capability handshake contracts in `src/tools/ide_plugin_host.rs` for all 13 external IDEs (`RESEARCHPLAN.txt:646`): `vscode`, `visual-studio`, `jetbrains`, `android-studio`, `zed`, `xcode`, `eclipse`, `cursor`, `windsurf`, `apache-netbeans`, `kiro`, `qoder`, and `lapce` (built as separate extensions outside `crates/toksi/`).
2. Expose `toksi ide-bridge --ide <id> [--transport stdio|pipe|uds|sse|http]` and `:9098/ide/*` JSON-RPC / ACP / MCP endpoints streaming `<toksi-siagent>` chat turns, 22-mode selection, diff/patch previews, 45-language CodeGraph/SAST diagnostics, ProjectMem checkpoints (`/resume`), and `/byok`/`/model`/`/effort` controls.
3. Guarantee zero IDE extension binaries or Node/JVM packages are embedded inside `crates/toksi/`, and verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and focused IDE host protocol tests.
```

## Technical Specifications & Verification Criteria
- All 13 external IDE targets resolve typed capability manifests via `IdePluginHost`.
- Zero binary bloat inside `crates/toksi/` (extensions remain external consumers of `:9098` / `toksi ide-bridge`).
