# Phase 19.1 --- Mandatory `ctxmode` Tool Rename Everywhere (`context-mode` / `context` -> `ctxmode`)

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read ALL 10 canonical workspace files FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. .toksi/projectmem/plan.md
5. .toksi/projectmem/PROJECT_MAP.md
6. .toksi/projectmem/summary.md
7. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
8. crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
9. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
10. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
Plus .agents/brain/PHASE_19_1_CTXMODE_TOOL_RENAME_EVERYWHERE.md.

Execute Phase 19.1: Mandatory `ctxmode` Tool Rename Everywhere (RESEARCHPLAN.txt:578).
Targets (316 `context-mode` refs and 0 `ctxmode` refs verified on disk):
  - crates/toksi/src/tools/contextmode.rs (129 refs)
  - crates/toksi/src/contextmode/{mod.rs,cli.rs,fts.rs,hooks.rs,mcp.rs,tests.rs,filter.rs,sandbox.rs}
  - crates/toksi/src/commands/{ctx.rs,runmcp.rs,index.rs,init.rs,doctor.rs,doctor_probe.rs,doctor_stats.rs}
  - crates/toksi/src/agents/{manager.rs,mcp_adapters.rs,hooks.rs,copilot.rs,antigravity.rs,grok.rs,claude.rs,proxy_provider.rs,open_agents.rs}
  - crates/toksi/src/tools/multi_protocol.rs (`ToolTarget::Ctxmode`), crates/toksi/src/core/mod.rs
  - crates/toksi/src/agent_runtime/slash.rs (`/ctxmode` + deprecated `/context` fallback notice)
  - crates/toksi/src/util/{instructions.rs,versions.rs,toml.rs,paths.rs}
  - crates/toksi/src/main.rs (`toksi ctxmode`), crates/toksi/src/tui/*, crates/toksi/src/dashboard/*
  - crates/toksi/tests/phase19_1_ctxmode_rename_verification.rs (NEW)

Requirements:
1. Rename EVERY tool-facing `context-mode` and conflicting bare `context` reference to **`ctxmode`** across CLI subcommands (`toksi ctxmode`, preserving `toksi ctx` alias with deprecation hint if desired), in-chat slash commands (`/ctxmode`, keeping temporary `/context` redirect notice), hook syntax (`ctxmode-hook <agent>`), MCP server key (`ctxmode`), skill directory (`skills/ctxmode/`), and repo workspace store (`<repo>/.toksi/ctxmode/`).
2. Implement transactional, non-destructive auto-migration in `src/contextmode/fts.rs` & `src/commands/doctor.rs`: detect legacy `<repo>/.context-mode/` and `<repo>/.toksi/context/`, merge SQLite FTS5 chunks/journals/stats row-by-row into `<repo>/.toksi/ctxmode/`, and delete legacy folders cleanly without data loss.
3. Update all agent MCP generators and config adapters (`agents/*.rs`, `tools/contextmode.rs`) to write `"ctxmode"` MCP keys, `ctxmode-hook <agent>`, and `skills/ctxmode/` while scrubbing legacy `context-mode` / `toksi-context-mode` / bare `context` entries during `toksi doctor --fix`.
4. Ensure forward-slash `/` paths, Edition 2024 `#![deny(unsafe_code)]`, zero `.unwrap()` in non-test production code, zero stale `.projectmem/` refs, and update `.toksi/test-baseline.json` plus all 6 canonical plan destinations upon completion.
5. Verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p toksi --test phase19_1_ctxmode_rename_verification --test phase3_contextmode_verification`.
```

---

## Technical Specifications & Details

### 1. Ground-Truth Scope Audit (Verified on Disk)
- Live codebase scan: `316` occurrences of `context-mode` and `0` occurrences of `ctxmode` in `crates/toksi/src/`.
- Primary hotspots: `tools/contextmode.rs` (129), `agents/manager.rs` (16), `agents/mcp_adapters.rs` (15), `agents/copilot.rs` (14), `commands/doctor_probe.rs` (12), `agents/antigravity.rs` (9), `util/instructions.rs` (9), `util/versions.rs` (9), `main.rs` (7), `agents/hooks.rs` (7), `agents/grok.rs` (6), `contextmode/mcp.rs` (6).

### 2. Migration & Storage Contracts
- Canonical workspace store: `<repo>/.toksi/ctxmode/` (DB `ctxmode.db` or migrated `context.db` inside `<repo>/.toksi/ctxmode/`).
- Canonical hook syntax: `ctxmode-hook <agent>`.
- Canonical MCP server ID: `ctxmode`.
- Canonical skill folder: `skills/ctxmode/`.
- Canonical CLI & slash command: `toksi ctxmode` and `/ctxmode`.

## Verification Criteria
- `rg -i "context-mode" crates/toksi/src/` returns `0` matches outside explicit legacy-migration/cleanup match arms.
- `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `phase19_1_ctxmode_rename_verification` pass 100%.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
