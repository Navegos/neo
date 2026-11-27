# Phase 17.7: Async 10-Tab Mouse TUI (PHASE_20_2 Upgraded), Live Web Cockpit (`:9100`), Incremental `.toksi/test-baseline.json` Harness, CLI `--help`, & Full Doc Sync

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 11 & 13.5--13.6), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_17_7_ASYNC_9TAB_UI_INCREMENTAL_TESTS_AND_DOCS.md first.

Execute Phase 17.7: Final 10-Tab UI Parity (PHASE_20_2 Upgraded), Incremental Test Engine, & Complete Documentation Verification.
Targets:
  - crates/toksi/src/tui/{mod.rs, app.rs, views/*, widgets/*}
  - crates/toksi/src/dashboard/{mod.rs, api.rs, state.rs, sse.rs, assets/{index.html, style.css, app.js}}
  - crates/toksi/src/commands/test_runner.rs (or scripts/ + .toksi/test-baseline.json)
  - crates/toksi/tests/phase14..phase17_7*.rs
  - crates/toksi/src/main.rs (`help_text()`), crates/toksi/README.md

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly; zero `.unwrap()`/`.expect()`/`panic!()`.
2. Exact 10-Tab Parity Across BOTH TUI & Web Cockpit (`< 5ms` non-blocking response via `watch::Receiver<DashboardSnapshot>`):
   1. `Savings` *(renamed from `Waterfall & Savings`)*
   2. `The War Room`
   3. `AI Agent Fleet & Quotas` *(renamed from `Agent Fleet (17)`)*
   4. `Memory Decision DAG`
   5. `CodeGraph Explorer`
   6. `Sandbox Inspector` *(typo `Insperctor` fixed)*
   7. `System Controls`
   8. `AI Gateway & Quotas`
   9. `Keyring & Database`
3. Zero Fake UI Data & Zero Paid-Tier Badges: Completely purge `init_waterfall_samples()`, `0.38/0.42/0.20` layer shares, `68.0%`, `1428 KB` vacuum stub, `:8788 BYOK` mislabel, fake `429` rows, fake 6-symbol CodeGraph stub, fake heatmap vec, raw `<AGENT>_API_KEY` Keyring stub, and `[PAID]`/`[DISQUALIFIED-FREE]` badges.
4. Never Weaken Tests: Incremental test skipping ONLY bypasses already-confirmed passing suites when their `Cargo.lock` + `watched_paths` BLAKE3 tree hash is unchanged; `--all`/`--force` re-runs everything cleanly.

Requirements:
1. Finalize Async 10-Tab Mouse TUI (PHASE_20_2 Upgraded) (`src/tui/`) & Futuristic Web Cockpit (`src/dashboard/`):
   - Configured-only visibility in `Savings`, `AI Gateway & Quotas`, and `Keyring & Database` (`0` metrics when idle).
   - `AI Agent Fleet & Quotas` lists all supported Toksi & 65/90+ adapters, greys out binaries missing from `PATH` with remediation hints, provides `Direct-Wire`/`Config-Wire`/`Unwire` actions, and displays read-only plan + `hour|day|week|month|year` quota gauges.
   - `Memory Decision DAG`: wire `GET /api/memory/dag`, clickable `.heatmap-cell` modal drilldown (`GET /api/memory/heatmap/:cell_id` showing `issue_id`, `file:line`, attempts, resolution/retry actions), and monorepo subproject filter.
   - `CodeGraph Explorer`: live `symbols.db` stats (`41+` langs), search/impact explorer, `ast-grep`/SCIP/Repomix preview, static security flaws + `skill-scanner` SARIF/CEL, **`Navegos/advisory` O(1) prefix search + `publish-pr` status**, `.ignore` purge action.
   - `Sandbox Inspector`: live OS sandbox backend (`native` / `sandbox-runtime` / `sbx`), `< 2KB` projected stdout, FTS5 search bar, URL cache monitor, **ADR quarantine events**, `CommandGuard` offensive-policy + kill-switch audit log, `defenseclaw` fail-closed guardrails & audit evidence.
   - `System Controls`: live `~/.toksi/config.toml` editor (`[repositories]`, `[plugins]`, `[security]`), `daemon|proxy|dashboard` `start|stop|restart` controls, interactive Plugin Manager (`/api/plugins`), **External `<skills>` Sync Manager** (`Navegos/skills` + user skills dir), per-agent granular component installer (`hooks|mcp|skills|rules|instructions|configs` for `user`-auto vs explicit `repo` opt-in), live logs.
   - `AI Gateway & Quotas`: CRUD over multiple `<byok, model, effort>` profiles per provider, custom `base_url` editor, zero-cost `/v1/models` refresh, per-agent & per-`<byok, model, effort>` Dynamic Complexity Routing & Load Balancer selector, Headroom bypass monitor.
   - `Keyring & Database`: configured-only Keyring (`gemini:antigravity` normalized), live key editor, SQLite table browser (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`), working `Export` (`POST /api/db/export`), `Import` (`POST /api/db/import` with `overwrite`), Backup/Restore, legacy nested `<repo>` repair.
2. Ship `.toksi/test-baseline.json` Incremental Test Skip Harness + Integration Suites `phase14`--`phase18_5`:
   - Recompute BLAKE3 over `Cargo.lock` + suite `watched_paths`; print `[SKIP-UNCHANGED] <suite>` for clean untouched suites while running dirty/failed suites and always running `cargo check --workspace` & `cargo clippy --workspace --all-targets -- -D warnings`.
   - Add integration suites covering `phase14`--`phase18_6` across all **495 mandated vaults** and **35-component `<toksi-siagent>`** (**43** `.agents/brain/*.md` guides, config TOML incl. `[repositories]`/`[plugins]`/`[security]`, free BYOK, multi-profile `<byok, model, effort>`, `plugins_registry`/`blind_signatures`/`advisory_embargoes`/`adr_audit_events`, Keyring `gemini:antigravity` & `AGY_ADC_AUTH=true`, `live_zone` + `/byok/` Headroom + effort scaling across all 7 entrypoints, 41+ Tree-sitter & repo-only isolation, `TokenEconomizer`/`agent-opt`/`ctx-hook`/`runmcp`, `DirectWireBus` + **16** slash cmds incl. `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, Zero-Plaintext Blind Engine + Validator PRs + ADR + 5-backend Sandbox + `CommandGuard` kill-switch, `opa`/`cedar`/`spire`/`Guardrails`/`llm-guard`, `temporal`/`restate`/`langgraph`, `Qdrant`/`Cognee`/`Graphiti`, local/Rust/cluster inference `mistral.rs`/`candle`/`ollama`/`vllm`/`sglang`/`vmon`/`cuda-rust`, gateway/ConnectRPC/reconnect, 10-Tab API/SSE, `<enforce>`/`skilldoctor`/`agentskills` healer/41-lang SAST, and `.toksi/projectmem/plan_checkpoints/`).
3. Finalize `main.rs` `--help` & `crates/toksi/README.md`, then execute final full build + clippy + tests.
4. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p toksi` (and verify subsequent incremental run logs `[SKIP-UNCHANGED]`).
```

---

## Technical Specifications & Details

### 1. Verification Criteria
- `init_waterfall_samples` and fake API constants (`1428`, `0.38`, `68.0`, `[DISQUALIFIED-FREE]`) have **0** matches across `crates/toksi/src/`.
- Both `crates/toksi/src/tui/` and `crates/toksi/src/dashboard/assets/index.html` render the exact 9 named tabs identically.
- Second incremental test run with no source edits logs `[SKIP-UNCHANGED]` for all green suites; editing 1 file in a watched module re-triggers only the affected suite(s).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** `.toksi/test-baseline.json` tracks `35` suites; `PHASE_19_3` adds `docker/Dockerfile.test`, `toksi test --docker|--sandbox`, and `TOKSI_HOME`/`:0` isolation.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
