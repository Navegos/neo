# Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_2_WEB_COCKPIT_DASHBOARD.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard.
Targets:
  - crates/toksi/src/dashboard/mod.rs
  - crates/toksi/src/dashboard/api.rs
  - crates/toksi/src/dashboard/state.rs
  - crates/toksi/src/dashboard/sse.rs
  - crates/toksi/src/dashboard/assets/index.html
  - crates/toksi/src/dashboard/assets/style.css
  - crates/toksi/src/dashboard/assets/app.js
  - crates/toksi/src/commands/dashboard.rs

Requirements:
1. Background Web Cockpit on Port 9100:
   - Non-blocking background service running on `http://127.0.0.1:9100` (`toksi dashboard`).
   - Modern glassmorphic dark UI embedded via `include_str!`.
2. Telemetry & Control Capabilities:
   - Real-time 17-agent observation grid with live ground-truth token savings, connected provider, model, latency breakdown, upstream quota gauges, and paid tier status badges.
   - Interactive Gateway & Load Balancer controls (toggle providers, adjust weights/priorities, switch algorithms, complexity tiers).
   - OS Keyring and SQLite DB management tools (Export, Import, Backup, Restore).
3. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Dashboard Embedded Server in `dashboard/mod.rs`
- Serves `assets/index.html`, `assets/style.css`, and `assets/app.js` using `include_str!`.
- Provides SSE metrics stream at `GET /api/events` and REST API endpoints at `/api/stats`, `/api/providers`, `/api/agents`.
- Runs on a dedicated runtime thread on `127.0.0.1:9100`, fully decoupled from Gateway port `:9099`.
