# Phase 10: Non-Destructive Environment Injection & Lifecycle Teardown

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, .projectmem/plan.md, and .gemini/antigravity/brain/PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md first to understand the global master architecture and cross-tool coupling.

Execute Phase 10: Non-Destructive Environment Injection & Lifecycle Teardown.
Target: crates/toksi/src/env_injector/{windows.rs, unix.rs, teardown.rs}, crates/toksi/src/commands/daemon.rs

Requirements:
1. Implement non-destructive user environment management without modifying any agent binaries:
   - On Windows (windows.rs): Manage HKEY_CURRENT_USER\Environment directly via Windows APIs.
     Inject proxy environment variables (GOOGLE_GEMINI_BASE_URL, CLOUD_CODE_URL, etc.) with TOKSI_OWNED=1 markers.
     Broadcast WM_SETTINGCHANGE via SendMessageTimeoutW so new VS Code and terminal windows pick up the proxy immediately without restart.
   - On Linux/macOS (unix.rs): Inject variables into user session environment.
2. Implement lifecycle teardown guard (teardown.rs):
   - Hook into toksi daemon stop, toksi proxy down, and process termination handlers (SIGINT, SIGTERM, Ctrl+C).
   - Automatically purge all Toksi-owned environment variables upon shutdown.
   - Broadcast the environment update so the OS returns to its clean default state with zero leftovers.
3. Connect environment injection to toksi daemon start so the environment is ready before VS Code or IDEs launch.
4. Ensure zero compiler warnings and passing unit tests.
```

