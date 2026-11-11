# Phase 17.1: `~/.toksi/config.toml`, CLI Process Controls, Edition 2024 `#![deny(unsafe_code)]`, & Zero Paid-Tier Restrictions

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 12–13), .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_17_1_CONFIG_TOML_CLI_AND_FREE_BYOK.md first.

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
4. Verify: `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi`.
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
