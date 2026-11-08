# Phase 10: Non-Destructive Environment Injection & Lifecycle Teardown

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 10: Non-Destructive Environment Injection & Lifecycle Teardown.
Targets:
  - crates/toksi/src/env_injector/windows.rs
  - crates/toksi/src/env_injector/unix.rs
  - crates/toksi/src/env_injector/teardown.rs
  - crates/toksi/src/env_injector/mod.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): Safe Rust enforced crate-wide. The ONLY permissible exceptions are strictly isolated Win32 FFI leaf calls in `windows.rs` (advapi32.dll, user32.dll) accompanied by explicit `// SAFETY:` rationale.
2. Zero Warnings Policy: Compile cleanly under `RUSTFLAGS="-D warnings"`.
3. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
4. Universal Forward-Slash (/) Standard: Use normalize_path() for all environment paths.
5. Non-Destructive Invariant: Injected variables must be cleanly unset upon process exit or signal capture via RAII TeardownGuard.

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
4. Integrate with `toksi daemon start` and `toksi daemon stop`.
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
- Unsafe FFI blocks in `windows.rs` must have explicit safety documentation validating pointer validity and null termination.
- Zero leftover proxy environment variables after daemon termination.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- env_injector` passes 100%.
- Daemon exit cleans up all registered environment keys from `HKCU\Environment`.
