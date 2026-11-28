# Phase 17.1: `~/.toksi/config.toml`, CLI Process Controls, Edition 2024 `#![deny(unsafe_code)]`, & Zero Paid-Tier Restrictions

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-NEEDS-FIX]

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 12--13), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md first.

Execute Phase 17.1: `~/.toksi/config.toml`, CLI Process Controls, Edition 2024 `#![deny(unsafe_code)]`, & Zero Paid-Tier Restrictions.
Targets:
  - Cargo.toml (workspace member `crates/toksi-sys`)
  - crates/toksi-sys/{Cargo.toml, src/lib.rs}
  - crates/toksi/src/{lib.rs, main.rs}
  - crates/toksi/src/config/mod.rs
  - crates/toksi/src/commands/{config.rs, daemon.rs, proxy.rs, dashboard.rs, doctor_stats.rs, runmcp.rs, mod.rs}
  - crates/toksi/src/gateway/agent_cli/{mod,tier_check,antigravity,claude,codex,copilot,cursor,grok}.rs
  - crates/toksi/src/util/{paths,env_path,routing_lock,exec,pathsetup,prompt}.rs
  - crates/toksi/src/{agents/{antigravity,proxy_provider}.rs, env_injector/{unix,windows,teardown}.rs, headroom/{autostart,process_identity}.rs, projectmem/store.rs, codegraph/kernel/stack.rs}
  - crates/toksi/src/dashboard/{api.rs, assets/app.js}

Disciplinary & Portability Requirements:
1. Genuine Edition 2024 `#![deny(unsafe_code)]`: Enforce `#![deny(unsafe_code)]` at top of `crates/toksi/src/lib.rs` and `crates/toksi/src/main.rs`. Eliminate ALL 23 `unsafe` sites in `crates/toksi/src/`:
   - Delete `std::env::set_var` / `remove_var` in `agents/antigravity.rs:1725,1748,1803`, `commands/proxy.rs:393-423`, `commands/runmcp.rs:139-171`, `env_injector/{unix.rs:36,160,186, windows.rs}`, `util/{exec.rs:196, pathsetup.rs:79}` -> store runtime overrides in `OnceLock<RwLock<ToksiTomlConfig>>` and apply per-child via `Command::env()`/`Command::env_remove()`.
   - Replace `LockFileEx`/`OVERLAPPED`/`fcntl`/`flock` in `projectmem/store.rs:84-157` and `util/routing_lock.rs:21-84` with safe `std::fs::File::lock()` / `fs2::FileExt`.
   - Replace Unix `pre_exec(setsid)` in `commands/doctor_stats.rs:500,580` and `gateway/agent_cli/*.rs` with safe `CommandExt::process_group(0)` or `crates/toksi-sys`.
   - Replace `libc::signal` / `SetConsoleCtrlHandler` with `tokio::signal` + `ctrlc`; replace raw Win32/sysctl process checks (`headroom/process_identity.rs`) with `sysinfo`; move unavoidable Win32 Registry/Credential/Console/Job-Object FFI into `crates/toksi-sys` with explicit `// SAFETY:` proofs.
2. Zero Panics Policy: Prohibit `.unwrap()`, `.expect()`, `panic!()` in runtime paths.
3. Universal Forward-Slash (`/`) & PATH Mandate: Convert paths via `normalize_path()` and verify external binaries cleanly via `check_executable_in_path` (missing binaries grey out cleanly in UI/CLI with remediation hints rather than fatal aborts).
4. Zero Paid-Tier Restrictions: Completely delete `FreeTierDisqualified`, `ACTIVE_PAID_PROVIDERS`, and `[PAID]`/`[DISQUALIFIED-FREE]` gates across gateway, database, TUI, and Web UI while keeping read-only plan badges and `quota_windows`.

Requirements:
1. Implement `crates/toksi/src/config/mod.rs` & `toksi config [show|path|set|edit|reload]` reading `~/.toksi/config.toml` (auto-creating commented default if absent) with `[ports]`, `default_agents`, `direct_wire`, `[test] incremental`, `[byok.<id>] base_url/model/effort/enabled`.
2. Upgrade CLI process commands:
   - `toksi daemon start|stop|restart --agents <list>` (PID/socket supervision honoring `config.toml` `[ports]` & configured agents).
   - `toksi proxy start|stop|restart --agents <list>`.
   - `toksi dashboard start|stop|restart [--foreground] [--open]` (detaches in background by default!).
3. Purge all `FreeTierDisqualified` / `ACTIVE_PAID_PROVIDERS` checks from `tier_check.rs`, `dashboard/api.rs`, `dashboard/assets/app.js`, and `tui/`.
4. Verify: `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi` across **495 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides (`PHASE_01`..`PHASE_19_6`), `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized zero-bloat `<skills>` & `<advisory>`.
```

---

## Technical Specifications & Details

### 1. `~/.toksi/config.toml` Reference Schema (`src/config/mod.rs`)
```toml
[general]
default_agents = ["claude", "codex", "antigravity-cli"]
direct_wire = true
auto_install_user_hooks = true
auto_install_repo_hooks = false

[ports]
headroom = 8787
grok_oauth = 8788
copilot_vscode = 8789
copilot_cli = 8790
byok_gateway = 18787
gateway = 9099
dashboard = 9100

[test]
incremental = true
baseline_path = ".toksi/test-baseline.json"
```
Supports dynamic `[[byok.profiles]]` overrides merged cleanly with `~/.toksi/toksi.db` (`byok_profiles` table).

### 2. Verification Criteria
- `rg "unsafe\s*\{" crates/toksi/src` returns **0** hits; `crates/toksi/src/{lib,main}.rs` both carry `#![deny(unsafe_code)]`.
- `rg "FreeTierDisqualified|DISQUALIFIED-FREE|ACTIVE_PAID_PROVIDERS" crates/toksi/src` returns **0** hits.
- `toksi config show`, `toksi daemon start --agents claude`, `toksi dashboard start`, and `toksi dashboard stop` work cleanly.

---

## Wave 18 (v5.2) Synchronization Addendum
- Extend `~/.toksi/config.toml` with `[repositories]` (`advisory_dir`, `advisory_remote`, `advisory_pages_url`, `skills_dir`, `skills_remote`, `skills_pages_url`, `user_skills_dir`, `stage_dir_outside_crates`), `[ports].protocol_bus = 9098`, `[auth.gcp_adc]` (`agy_adc_auth`, `gcp_project`, `gcp_location`), `[optimization]` (`adapt_to_thinking_effort`, `low_effort_compression`, `medium_effort_compression`, `high_effort_compression`), `[security]` (`enforce_in_repo_skills = true`, `auto_heal_skill_frontmatter = true`, `static_analyzer_enabled = true`, `defenseclaw_fail_closed = true`, `skill_scanner_policy = "balanced"`, `policy_engine = "opa_cedar"`, `iam_attestation = "spiffe_spire"`, `kill_all_services_on_bypass = true`, `adr_enabled = true`, `zero_plaintext_blind_signatures = true`, `sandbox_backend = "auto"`), `[plugins]` (`enabled = true`, `plugin_dirs = ["~/.toksi/plugins", "<repo>/.toksi/plugins", "<repo>/.agents/plugins"]`, `enforce_security_scan = true`, `fail_closed_hooks = true`, `protocols = ["mcp", "a2a", "acp", "stdio", "sse", "http"]`, `allow_agent_to_agent = true`), and `[checkpoint]` (`auto_save_on_429`, `auto_save_milestones`).
- Pair directly with `PHASE_18_2` (`src/plugins/`, `src/validator/`, zero-plaintext engine, `defenseclaw`, `skill-scanner`), `PHASE_18_3` (`AGY_ADC_AUTH=true` + `"gcp.project"` / `"gcp.location"`), `PHASE_18_5` (`adr.rs`, `sandbox.rs`, `guard.rs` kill-switch), and `PHASE_18_6` (**35-component `<toksi-siagent>`** across **495 mandated vaults** & **43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, canonical `<repo>/.toksi/projectmem/`).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
