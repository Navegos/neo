# Phase 15.2: Unified 5-Port Daemon & Copilot Supervision

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.2: Unified 5-Port Daemon & Copilot Supervision.
Targets:
  - crates/toksi/src/commands/daemon.rs
  - crates/toksi/src/headroom/copilot_daemon.rs
  - crates/toksi/src/agents/copilot.rs
  - crates/toksi/src/headroom/proxy.rs
  - crates/toksi/src/gateway/mod.rs
  - crates/toksi/src/dashboard/mod.rs

Requirements:
1. Unified 5-Port Daemon Supervision (`toksi daemon start`):
   - Supervise all 5 dedicated endpoints simultaneously in the background:
     * Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
     * Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
     * Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
     * Port 9099: Enterprise AI Gateway & Drop-in SDKs (`/v1/chat/completions`, `/v1/messages`, `/metrics`)
     * Port 9100: Web Telemetry & Control Dashboard Cockpit (non-blocking background service)
   - Zero multi-command manual steps. One daemon launch prepares the entire system for all 17 AI coding assistants.
2. Network Port Separation for Maximum Latency Performance:
   - Port 9099 strictly dedicated to high-throughput streaming LLM inference, SSE token forwarding, and protocol translation (<10 µs latency overhead).
   - Port 9100 dedicated to Web Dashboard Cockpit, SSE metrics broadcasting, static assets, and SQLite DB backup/export.
   - Eliminates socket buffer and thread pool contention between browser clients and AI completion streams.
3. Copilot Wiring & Dual-Port Isolation:
   - Ensure both Port 8789 (VS Code proxy) and Port 8790 (CLI proxy) are launched, supervised, and verified.
   - `validate_copilot_proxy_ports()` guarantees ports never collide.
   - Live health checks (`/livez`) for both Copilot proxies.
4. Process Lifecycle & Non-Destructive Environment Cleanup:
   - Upgrade `is_daemon_running()`, `daemon_status()`, and `daemon_stop()`.
   - `daemon_stop()` terminates all 5 background listeners cleanly and removes Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast).
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. 5-Port Supervisor in `daemon.rs`
```rust
pub fn daemon_run_foreground() -> i32 {
    let pid = std::process::id();
    let _ = ensure_dir(&headroom_paths_resolved().root);
    let _ = write_file_atomic(daemon_pid_file(), &pid.to_string());

    crate::env_injector::register_teardown_handlers();
    let mut teardown_guard = crate::env_injector::TeardownGuard::new();
    let _ = crate::env_injector::inject_proxy_env();

    // Ensure all 5 endpoints are running:
    // 1. Port 8787: Headroom Streaming Proxy
    // 2. Port 8789: Copilot VS Code Proxy
    // 3. Port 8790: Copilot CLI Proxy
    // 4. Port 9099: AI Gateway & Drop-in SDKs
    // 5. Port 9100: Web Telemetry Cockpit
    ...
}
```

### 2. Dual-Port Copilot Supervisor in `copilot_daemon.rs`
- Add `ensure_copilot_cli_proxy_up()` for port 8790 alongside `ensure_copilot_proxy_up()` for port 8789.
- Distinct PID and log files:
  * `copilot-vscode.pid` & `copilot-vscode.log` (Port 8789)
  * `copilot-cli.pid` & `copilot-cli.log` (Port 8790)

