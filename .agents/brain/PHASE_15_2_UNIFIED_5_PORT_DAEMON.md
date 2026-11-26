# Phase 15.2: Unified 8-Port Daemon, `config.toml`, & Copilot Supervision

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_15_2_UNIFIED_5_PORT_DAEMON.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.2: Unified 8-Port Daemon, `~/.toksi/config.toml`, & Copilot Supervision.
Targets:
  - crates/toksi/src/config/mod.rs
  - crates/toksi/src/commands/daemon.rs
  - crates/toksi/src/commands/proxy.rs
  - crates/toksi/src/headroom/{copilot_daemon.rs, grok_daemon.rs, byok_gateway.rs}
  - crates/toksi/src/gateway/mod.rs
  - crates/toksi/src/dashboard/mod.rs
  - crates/toksi/src/env_injector/teardown.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly (move `SetConsoleCtrlHandler`/`libc::signal` to `tokio::signal`/`ctrlc` & Win32 Registry/Broadcast to `crates/toksi-sys`). Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors and `anyhow::Result` with `.context()` in CLI.
3. Unified 8-Port Isolation Guarantee (all overridable via `~/.toksi/config.toml` `[ports]`, adding `:9098` Protocol Bus in Wave 18):
   - Port `8787`: Streaming Headroom Proxy (`headroom/proxy.rs` â€” Antigravity, Claude, Codex, Cursor, BYOKs)
   - Port `8788`: Grok OAuth Daemon (`grok_daemon.rs` â€” NOT BYOK!)
   - Port `8789`: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
   - Port `8790`: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
   - Port `9098`: Unified Protocol Bus (`MCP`, `A2A` v1.0, `ACP`, `stdio`/`SSE`/`HTTP`, shared `HeadroomEffortCompressor` payload compression, Unified Plugin System endpoints under `<enforce>`, `plan_checkpoints`, & security events)
   - Port `18787`: BYOK Multi-Profile & Vertex ADC Gateway (`byok_gateway.rs` â€” supervised by daemon, routed through shared `HeadroomEffortCompressor` `Low`/`Medium`/`High`)
   - Port `9099`: Enterprise AI Gateway & Drop-in SDKs + `/metrics` (Dedicated inference thread pool, <10 Âµs latency overhead, executing full `HeadroomEffortCompressor` `Low`/`Medium`/`High` before cache/failover)
   - Port `9100`: Detached Web Dashboard Cockpit (Isolated non-blocking background service, zero TCP backpressure on inference)
4. Non-Destructive Environment Lifecycle: Clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix session env on daemon shutdown (never inject `CLOUD_CODE_URL`).

Requirements:
1. Master Daemon & Proxy Control (`toksi daemon start|stop|restart --agents <list>`, `toksi proxy start|stop|restart --agents <list>`):
   - Read port overrides, custom API URLs, and default `--agents` from `~/.toksi/config.toml`.
   - Supervise all 8 dedicated endpoints simultaneously (`:8787..:8790, :9098, :9099, :9100, :18787`) in a single background process for user-configured/selected agents ONLY (skipping unconfigured agents without blocking).
   - Non-blocking asynchronous task supervision using Tokio tasks.
2. Copilot Dual-Port Real Proxy Supervision:
   - Manage Port `8789` for Copilot VS Code extension and Port `8790` for Copilot CLI.
   - Replace `"copilot proxy ready"` & `"wrap"` stubs (`copilot_daemon.rs`) with real upstream OAuth/SSE proxying, Headroom compression, live `/livez` probes, and port collision prevention.
3. Thread & Socket Isolation:
   - Guarantee complete socket buffer and thread isolation between Port `9099` (LLM streaming inference) and Port `9100` (browser web cockpit).
4. Environment Teardown:
   - Cleanly uninject all proxy environment variables on daemon exit or `SIGINT`/`SIGTERM`.
   - Unify **35-component `<toksi-siagent>`** out of the box ("off the back") via `DirectWireBus` and `:8787`/`:9098`/`:9099`/`:18787` across all **495 mandated vaults**, **43** `.agents/brain/*.md` guides (`PHASE_01`..`PHASE_19_6`), zero paid-tier restrictions, externalized zero-bloat `<skills>` & `<advisory>`, and canonical `<repo>/.toksi/projectmem/`.
5. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Daemon Master Supervisor** (`src/commands/daemon.rs`): Spawns independent Tokio listener tasks for configured ports `8787`, `8788` (Grok), `8789`, `8790`, `9098` (Protocol Bus), `18787` (BYOK), `9099`, and `9100`, filtered by `--agents` / `config.toml`.
- **Port Latency Isolation Rationale**: Web dashboards require serving HTML, CSS, JS, and high-frequency SSE UI streams. Placing the dashboard on Port `9100` prevents slow browser clients from starving the Tokio reactor threads processing LLM tokens on Port `9099`.
- **Copilot Dual-Port Manager** (`src/headroom/copilot_daemon.rs`): Directs VS Code requests to `8789` and CLI tool requests to `8790`, handling real OAuth token refresh and SSE upstream framing transparently.

### 2. Verification Criteria
- `cargo check -p toksi` and `cargo clippy --all-targets -- -D warnings` pass with 0 warnings.
- `cargo test -p toksi -- daemon` passes 100%.
- Simultaneous HTTP requests to `:9099/healthz` and `:9100/api/status` return 200 OK without mutual interference.

---

## Wave 18 (v5.2) Synchronization Addendum
- Ensure `CommandGuard` bypass kill-switch (`src/commands/guard.rs`) cleanly shuts down all 8 daemon ports (`:8787..:8790, :9098, :9099, :9100, :18787`), while ALL 7 inference/protocol ports enforce the shared `HeadroomEffortCompressor` across **495 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized `<skills>` & `<advisory>`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`
- **Wave 19â€“20 Reconciliation:** Remediated (`PHASE_19_2`/`20_2`): split network ports (`8787`â€“`8790`, `9098`, `9099`, `18787`), `<toksi-siagent>`, and core tools into dedicated `toksi _worker <role>` subprocesses over Named Pipe / UDS IPC with `:9100` detached from `daemon start`.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
