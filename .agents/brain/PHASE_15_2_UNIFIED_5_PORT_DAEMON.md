# Phase 15.2: Unified 5-Port Daemon & Copilot Supervision

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.2: Unified 5-Port Daemon & Copilot Supervision.
Targets:
  - crates/toksi/src/commands/daemon.rs
  - crates/toksi/src/headroom/copilot_daemon.rs
  - crates/toksi/src/gateway/mod.rs
  - crates/toksi/src/dashboard/mod.rs
  - crates/toksi/src/env_injector/teardown.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors and anyhow::Result with .context() in CLI.
3. Dedicated Port Isolation Guarantee:
   - Port 8787: Streaming Headroom Proxy (Antigravity, Grok)
   - Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
   - Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
   - Port 9099: Enterprise AI Gateway & Drop-in SDKs (Dedicated inference thread pool, <10 µs latency overhead)
   - Port 9100: Web Dashboard Cockpit (Non-blocking background service, zero TCP backpressure on inference)
4. Non-Destructive Environment Lifecycle: Clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix environment on daemon shutdown.

Requirements:
1. Master Daemon Supervisor (`toksi daemon start`):
   - Supervise all 5 dedicated endpoints simultaneously in a single background process.
   - Non-blocking asynchronous task supervision using Tokio tasks.
2. Copilot Dual-Port Supervision:
   - Manage Port 8789 for Copilot VS Code extension and Port 8790 for Copilot CLI.
   - Live `/livez` health probes and port collision prevention.
3. Thread & Socket Isolation:
   - Guarantee complete socket buffer and thread isolation between Port 9099 (LLM streaming inference) and Port 9100 (browser web cockpit).
4. Environment Teardown:
   - Cleanly uninject all proxy environment variables on daemon exit or SIGINT/SIGTERM.
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Daemon Master Supervisor** (`src/commands/daemon.rs`): Spawns independent Tokio listener tasks for Ports 8787, 8789, 8790, 9099, and 9100.
- **Port Latency Isolation Rationale**: Web dashboards require serving HTML, CSS, JS, and high-frequency SSE UI streams. Placing the dashboard on Port 9100 prevents slow browser clients from starving the Tokio reactor threads processing LLM tokens on Port 9099.
- **Copilot Dual-Port Manager** (`src/headroom/copilot_daemon.rs`): Directs VS Code requests to 8789 and CLI tool requests to 8790, handling their distinct header framing transparently.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- daemon` passes 100%.
- Simultaneous curl requests to :9099/healthz and :9100/api/status return 200 OK without mutual interference.
