# Phase 21.2 --- Codex CLI + Grok-Build CLI Grade `<toksi-siagent>` Chat TUI & Full CLI Bridge

> **Status (Wave 21 / v8.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-NEEDS-FIX]

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
Plus .agents/brain/PHASE_21_2_CODEX_GROK_STYLE_SIAGENT_CHAT_TUI_AND_CLI_BRIDGE.md.

Multi-Agent Resume Rule: Check `.toksi/projectmem/plan_checkpoints/` and `.toksi/test-baseline.json` before starting; keep parallel edits strictly disjoint and flush checkpoint state before exiting on any error, network drop, or quota limit.

Execute Phase 21.2: Replace `loop_engine.rs` plain `stdin.read_line` REPL with a Codex CLI (`inspect-vault/codex/codex-rs`) + Grok-Build CLI (`inspect-vault/grok-build`) grade interactive streaming `ratatui` Chat TUI executing all `toksi` CLI commands.
Sources: neo/inspect-vault/{codex,grok-build,maki,rust-code,tidev}
Targets:
  - crates/toksi/src/agent_runtime/{chat_ui.rs,loop_engine.rs,slash.rs,mod.rs}
  - crates/toksi/src/commands/{agent_runtime.rs,orchestrator.rs}

Requirements:
1. Build `src/agent_runtime/chat_ui.rs` using non-blocking `ratatui` + `crossterm` (`EnableMouseCapture`) featuring: streaming markdown/token viewport, collapsible tool-call cards, inline diff/patch review pane (`apply_patch` / `HashlineEditTool`), HITL approval modal, live `<byok, model, effort, mode>` status bar across all 22 Orchestrator Modes (`chat|coding|conversation|orchestration|planning|analysis|advisory|security|run|exec|swarm|orchestrator|plan|study|research`), and mouse/keyboard scrolling.
2. Wire full in-chat CLI command bridge (`RESEARCHPLAN.txt:647`) so users can invoke ANY `toksi` subcommand (`doctor`, `daemon`, `security`, `advisory`, `skills`, `cluster`, `test`, `byok`, etc.) AND all 29 universal slash commands (`/byok`..`/voice`) directly from `toksi chat` without deadlocks or blocking the UI event loop.
3. Preserve headless `--plain` / non-TTY fallback for automated pipe invocations and verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and focused `chat_ui` tests.
```

## Technical Specifications & Verification Criteria
- `run_chat_repl()` launches `chat_ui::run_interactive_chat_tui()` when attached to a TTY and falls back cleanly on piped stdin.
- All 22 Orchestrator Modes, 29 slash commands, and full `toksi` CLI subcommands execute cleanly inside chat.
