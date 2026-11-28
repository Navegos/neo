# Phase 22.2: `r3bl-open-core` (`r3bl_tui`) TUI Migration & `winit`-Style Mouse/Keyboard Ergonomics

> **Status (Wave 22 / v9.0 Audit):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```markdown
Implement Phase 22.2 across `crates/toksi/Cargo.toml`, `crates/toksi/src/tui/`, `src/agent_runtime/chat_ui.rs`, `src/commands/top.rs`, and `src/commands/rtk.rs`.

Mandatory Pre-Read Set:
- `crates/toksi/research/RESEARCHPLAN.md` (Sections 2.18, 11, 12)
- `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md` & `crates/toksi/README.md`
- `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`
- `crates/toksi/research/{docker_cli_compose_yaml_full_reference,docker_sandboxes_sbx_cli_yaml_reference,folders_structures_default_knowledge_used_by_the_known_cli_agents_ides,full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture,open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator,toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping}.md`
- `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`
- `.agents/rules/ascii-only.md`
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`
- `inspect-vault/{r3bl-open-core,ratatui,ratatui-image,tui-realm,iocraft}`

Multi-Agent Resume Rule: Lock `crates/toksi/Cargo.toml` under coordinator lease; update `.toksi/projectmem/orchestration_journal.jsonl` after each migrated surface (`chat`, `top`, `gain`, `tui`).

Targets:
1. Adopt `r3bl-open-core` (`r3bl_tui`) patterns/components plus `ratatui-image`, `tui-realm`, and `iocraft` capabilities for all interactive terminal UIs (`toksi chat`, `toksi gain`, `toksi top`, `toksi tui`).
2. Implement `winit`-inspired input normalization: strict `Press` gating, `Repeat` opt-in for navigation keys only, `Release` ignored, debounced mouse drag/scroll, and real-time mouse-hover focus highlights across tabs, lists, buttons, and collapsible tool cards.
3. Preserve exact **10-Tab TUI** layout (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`) with `--plain` non-TTY fallback and ASCII-safe borders.

Verification:
- Compile check + non-blocking async event-loop tests for `tui`, `chat_ui`, `top`, and `rtk gain` with simulated `Press`/`Release`/`MouseEventKind::Moved` inputs.
- Run `cargo check -p toksi`, `cargo clippy -p toksi --tests -- -D warnings`, and pure-ASCII check (0 bytes > 0x7F).
```

## Technical Specifications & Verification Criteria
- **Zero Synthetic UI Stubs**: Zero `"opencode"` or mock subprocess spawning in TUI views.
- **ASCII Policy**: 100% pure 7-bit ASCII (`0x00`-`0x7F`), `LF` line endings.
- **Verification**: `cargo test -p toksi tui` passes with 0 warnings.
