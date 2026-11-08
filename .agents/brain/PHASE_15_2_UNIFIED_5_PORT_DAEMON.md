# Phase 15.2: Unified 5-Port Daemon & Copilot Supervision

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

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

    let (tx, rx) = std::sync::mpsc::channel();
    let r1 = tx.clone();
    let r2 = tx.clone();
    let r3 = tx.clone();
    let r4 = tx.clone();
    let r5 = tx.clone();

    // 1. Headroom streaming proxy (:8787)
    std::thread::spawn(move || {
        let _ = r1.send(run_proxy_listener(8787));
    });

    // 2. Copilot VS Code proxy (:8789)
    std::thread::spawn(move || {
        let _ = r2.send(run_copilot_proxy(8789));
    });

    // 3. Copilot CLI proxy (:8790)
    std::thread::spawn(move || {
        let _ = r3.send(run_copilot_proxy(8790));
    });

    // 4. AI Gateway & /metrics (:9099)
    std::thread::spawn(move || {
        let _ = r4.send(run_gateway_listener(9099));
    });

    // 5. Web Telemetry Cockpit (:9100)
    std::thread::spawn(move || {
        let _ = r5.send(run_dashboard_listener(9100));
    });

    match rx.recv() {
        Ok(code) => code,
        Err(_) => 0,
    }
}
```
