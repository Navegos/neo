# Phase 22.5: Separate Service Port Isolation, Copilot -> Headroom Consolidation, Multi-Surface SAST, Plugin Mounts, Bidirectional BYOK & 502 Vault Count

> **Status (Wave 22 / v9.0 Audit):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```markdown
Implement Phase 22.5 across `crates/toksi/src/ipc/network_manager.rs`, `src/headroom/`, `src/gateway/`, `src/plugins/`, `src/security/`, `src/db/`, `src/config/`, and vault registry constants (`CONSOLIDATED_VAULT_COUNT`).

Mandatory Pre-Read Set:
- `crates/toksi/research/RESEARCHPLAN.md` (Sections 1.3.4, 2.0, 2.18, 4.2, 5.3, 12)
- `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md` & `crates/toksi/README.md`
- `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`
- `crates/toksi/research/{docker_cli_compose_yaml_full_reference,docker_sandboxes_sbx_cli_yaml_reference,folders_structures_default_knowledge_used_by_the_known_cli_agents_ides,full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture,open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator,toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping}.md`
- `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`
- `.agents/rules/ascii-only.md`
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`

Multi-Agent Resume Rule: Separate worker leases for `network_manager/headroom`, `plugins/security`, and `db/config/vault_registry`; never share live SQLite handles across tests (use `TempDir` DBs). Flush after each subsystem passes `cargo test`.

Targets:
1. Separate Service Port Enforcement & Copilot Consolidation: Ensure every daemon service binds a distinct non-overlapping local port managed by `NetworkService`, while consolidating Copilot proxy listeners (`:8789` / `:8790` compatibility routes) centrally into Headroom (`:8787`) so all Copilot and `<toksi-siagent>` turns flow through `HeadroomEffortCompressor`.
2. Multi-Surface SAST Wiring: Wire unified SAST execution across standalone CLI (`toksi security scan`), MCP tool calls (`:9098`), and `<toksi-siagent>` pre/post tool execution hooks (`loop_engine.rs`).
3. Unified Plugin System Expansion: Extend `src/plugins/` (`ToksiPluginComponent`) to support runtime plugin mounts across CLI subcommands, TUI views, Cockpit `:9100` panels, SAST analyzers, and `<toksi-siagent>` slash/tool hooks under `<enforce>`.
4. Bidirectional DB <-> TOML BYOK Sync: Synchronize `[byok.*]` / `[[byok_profiles]]` edits between `~/.toksi/config.toml` and `byok_profiles` in `~/.toksi/toksi.db` atomically without overriding encrypted keyring secrets.
5. Promote `CONSOLIDATED_VAULT_COUNT` / `total_mandated_vault_count()` from `495` to `502` incorporating all 7 Wave 22 vaults (`ratatui`, `ratatui-image`, `tui-realm`, `iocraft`, `r3bl-open-core`, `MojibakeLab`, `python-ftfy`).

Verification:
- Integration tests proving collision-free separate service ports, Copilot -> `:8787` consolidation, standalone/MCP/`<toksi-siagent>` SAST parity, plugin mount dispatch, bidirectional DB/TOML BYOK sync, and `total_mandated_vault_count() == 502`.
- Run `cargo check -p toksi`, `cargo clippy -p toksi --tests -- -D warnings`, and pure-ASCII check.
```

## Technical Specifications & Verification Criteria
- **Zero Regressions**: Preserve 493 Wave 20 and 495 Wave 21 vault invariants in historical changelogs/tests while asserting `total_mandated_vault_count() == 502`.
- **Verification**: `cargo test -p toksi phase22` (or subsystem suite) passes cleanly with 0 warnings.
