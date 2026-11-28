# Phase 22.1: `<toksi-siagent>` Chat Double-Keystroke Fix, Gateway `502` / Headroom-Central Routing, Line Editor & Git Engineer

> **Status (Wave 22 / v9.0 Audit):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```markdown
Implement Phase 22.1 across `crates/toksi/src/agent_runtime/chat_ui.rs`, `loop_engine.rs`, `src/gateway/server.rs`, and `src/tools/git_engineer.rs`.

Mandatory Pre-Read Set:
- `crates/toksi/research/RESEARCHPLAN.md` (Sections 1.3.4, 4.2, 5.2, 11, 12)
- `crates/toksi/CONSOLIDATION_PLAN.md` & `crates/toksi/PLAN.md` & `crates/toksi/README.md`
- `crates/toksi/research/WAVE22_MASTER_EXECUTION_PROMPT.md`
- `crates/toksi/research/{docker_cli_compose_yaml_full_reference,docker_sandboxes_sbx_cli_yaml_reference,folders_structures_default_knowledge_used_by_the_known_cli_agents_ides,full_ai_ecosystem_agent_runtimes_local_inference_engines_protocols_gateways_and_quota_architecture,open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator,toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping}.md`
- `.toksi/projectmem/plan.md`, `.toksi/projectmem/PROJECT_MAP.md`, `.toksi/projectmem/summary.md`
- `.agents/rules/ascii-only.md`
- `.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`

Multi-Agent Resume Rule: Claim disjoint worker leases in `.toksi/projectmem/worker_leases.json` and flush progress to `orchestration_journal.jsonl` before long test runs or on network/429/quota interruption. Never touch live `~/.toksi/`.

Targets:
1. `crates/toksi/src/agent_runtime/chat_ui.rs`: Filter `crossterm::event::Event::Key(k)` strictly on `k.kind == KeyEventKind::Press` to eliminate Windows double-keystroke duplication; implement `Ctrl+A`, `Ctrl+E`, `Ctrl+W`, `Ctrl+K`, `Home`, `End`, `Left`, `Right`, `Delete`, cursor positioning, multi-line paste, scrollback, explicit turn states (`Idle | Thinking | Streaming | ExecutingTool | WaitingApproval | Paused`), Pause/Resume, Edit-While-Working, and Abort controls.
2. `crates/toksi/src/agent_runtime/loop_engine.rs`: Remove Headroom bypass (`loop_engine.rs:198-205`) and hardcoded synthetic `"gpt-6.2-astra"` fallback; resolve configured/default profile model from `ToksiConfig` / `toksi.db` and route all turns centrally through Headroom (`:8787`).
3. `crates/toksi/src/gateway/server.rs`: Upgrade `resolve_upstream` (`lines 582-617, 678`) from exact `p.model_id == model` matching to canonical longest-prefix + provider/family fallback before failing cleanly with structured provider diagnostics instead of opaque HTTP `502`.
4. `crates/toksi/src/tools/git_engineer.rs`: Implement Git-Engineer context detector parsing `.git/config`, current branch, remotes (`<neo>`, `<advisory>`, `<skills>`), and dirty status for `<toksi-siagent>` context injection and commit/PR workflows.

Verification:
- Unit & integration tests (isolated `TempDir` + `:0`) proving single keystroke per press, full line-editor motions, Headroom `:8787` routing without synthetic model HTTP 502s, and Git-Engineer `.git/config` parsing.
- Run `cargo check -p toksi`, `cargo clippy -p toksi --tests -- -D warnings`, and pure-ASCII check (0 bytes > 0x7F).
```

## Technical Specifications & Verification Criteria
- **Zero-Panic Policy**: Zero `.unwrap()` / `.expect()` / `panic!()` in `chat_ui.rs`, `loop_engine.rs`, `server.rs`, and `git_engineer.rs`.
- **ASCII Policy**: 100% pure 7-bit ASCII (`0x00`-`0x7F`), `LF` line endings.
- **Verification**: `cargo test -p toksi chat_ui` and `cargo test -p toksi gateway` pass with 0 warnings.
