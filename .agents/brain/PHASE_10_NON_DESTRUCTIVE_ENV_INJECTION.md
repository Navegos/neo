# Phase 10: Non-Destructive Environment Injection & Lifecycle Teardown

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 10: Non-Destructive Environment Injection & Lifecycle Teardown.
Targets:
  - crates/toksi/src/env_injector/windows.rs
  - crates/toksi/src/env_injector/unix.rs
  - crates/toksi/src/env_injector/teardown.rs
  - crates/toksi/src/env_injector/mod.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly across `crates/toksi/src/`. Zero `unsafe` in `env_injector/{unix,windows,teardown}.rs` (eliminate ALL `std::env::set_var`/`remove_var` calls using `Command::env()`/`env_remove()` and isolate Windows Registry/Broadcast Win32 FFI (`advapi32`, `user32`) into leaf crate `crates/toksi-sys` with explicit `// SAFETY:` proofs). Replace `SetConsoleCtrlHandler`/`libc::signal` with `tokio::signal` + `ctrlc`.
2. Zero Warnings Policy: Compile cleanly under `RUSTFLAGS="-D warnings"`.
3. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors.
4. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all environment paths (keeping `\\` solely inside explicit `cmd.exe` hook strings) and respect `~/.toksi/config.toml` `[ports]` overrides (`:8787`, `:8788`, `:8789`, `:8790`, `:18787`, `:9099`, `:9100`).
5. Non-Destructive & Auth Bypass Invariant: Injected variables must be cleanly unset upon process exit or signal capture via RAII `TeardownGuard`; verify `PATH` executables before spawning (`Rule of Test and Inform User`); set `GOOGLE_GEMINI_BASE_URL` cleanly and NEVER inject `CLOUD_CODE_URL` so auth/quota checks pass untouched.

Requirements:
1. Windows Implementation (`windows.rs`):
   - Direct Win32 Registry manipulation under `HKCU\Environment` without spawning PowerShell or `setx.exe`.
   - Broadcast `WM_SETTINGCHANGE` via `SendMessageTimeoutW` with `HWND_BROADCAST` to notify running desktop processes (e.g. VS Code, terminals) without reboot.
2. Unix Implementation (`unix.rs`):
   - Linux: `systemd --user set-environment` / `~/.config/environment.d/` integration.
   - macOS: `launchctl setenv` and user session export.
3. RAII Teardown Guard (`teardown.rs`):
   - Stores original environment snapshots.
   - Restores original variables or deletes injected proxy keys on normal exit, `SIGINT`, `SIGTERM`, `SIGHUP`, or panic hook trigger.
4. Integrate with `toksi daemon start` and `toksi daemon stop` (plus `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, `AGY_ADC_AUTH=true` Google Cloud ADC pass-through, and **35-component `<toksi-siagent>`** across **495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, and externalized zero-bloat `<skills>` & `<advisory>`).
5. Ensure 0 compiler warnings and passing unit tests.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Windows Direct Registry Client** (`src/env_injector/windows.rs`): Uses Win32 `RegOpenKeyExW`, `RegSetValueExW`, and `RegDeleteValueW` on `HKEY_CURRENT_USER\Environment`.
- **Broadcast System Event**: Calls `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment", SMTO_ABORTIFHUNG, 3000, ...)` to inform active shells.
- **Unix Environment Injector** (`src/env_injector/unix.rs`): Interfaces with user-space service managers (`systemd --user` on Linux, `launchctl` on macOS).
- **RAII Teardown Guard** (`src/env_injector/teardown.rs`): Guarantees that proxy environment overrides (e.g. `HTTP_PROXY`, `OPENAI_BASE_URL`, `ANTIGRAVITY_BASE_URL`) never outlive the daemon.

### 2. Disciplinary Standards & Invariants
- Unsafe Win32 Registry/Broadcast FFI blocks live isolated inside `crates/toksi-sys` with explicit safety documentation validating pointer validity and null termination (`crates/toksi/src/env_injector/windows.rs` stays `#![deny(unsafe_code)]`).
- Zero leftover proxy environment variables after daemon termination.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- env_injector` passes 100%.
- Daemon exit cleans up all registered environment keys from `HKCU\Environment`.

---

## Wave 18 (v5.2) Synchronization Addendum
- Enforce `CommandGuard` immediate bypass kill-switch teardown (`src/commands/guard.rs`), `AGY_ADC_AUTH=true` ADC pass-through (`PHASE_18_3`), and `:9098` Protocol Bus export across **495 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, and canonical `<repo>/.toksi/projectmem/` with externalized `<skills>` & `<advisory>`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
