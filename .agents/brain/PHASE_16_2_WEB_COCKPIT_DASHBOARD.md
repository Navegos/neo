# Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_2_WEB_COCKPIT_DASHBOARD.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/bifrost is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new UI components, control panels, and dashboard widgets into consolidated toksi.

Execute Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard.
Source: neo/inspect-vault/bifrost (dashboard UI assets, controls)
Targets:
  - crates/toksi/src/dashboard/mod.rs
  - crates/toksi/src/dashboard/api.rs
  - crates/toksi/src/dashboard/state.rs
  - crates/toksi/src/dashboard/sse.rs
  - crates/toksi/src/dashboard/assets/index.html
  - crates/toksi/src/dashboard/assets/style.css
  - crates/toksi/src/dashboard/assets/app.js
  - crates/toksi/src/commands/dashboard.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Dedicated Port Isolation:
   - Port 9100: Dedicated strictly to Web Dashboard Cockpit.
   - Non-blocking background execution: `toksi dashboard` runs in the background without locking the terminal window.
   - Zero socket and thread contention with LLM inference streams on Port 9099.
4. Beautiful Dark Glassmorphic UI: High aesthetic finish, responsive grid, real-time micro-interactions.
5. Realistic Telemetry: True token savings, upstream quota gauges, provider routing status.

Requirements:
1. Isolated Background Web Cockpit on Port 9100:
   - Embedded local HTTP & SSE server running on `http://127.0.0.1:9100` (`toksi dashboard`).
   - Run in the background by default (or `--foreground`). Open browser automatically with `--open`.
   - Single-page application assets (HTML/CSS/JS) embedded at compile-time via `include_str!`.
2. Telemetry & Control Capabilities:
   - Real-time 17-agent live observation grid displaying connected provider, model, latency breakdown, ground-truth token savings, and paid quota status.
   - Live upstream quota gauges (remaining tokens, reset countdowns) parsed from rate-limit headers.
   - Interactive Gateway & Load Balancer controls (toggle providers, adjust weights/priorities, switch algorithms, set complexity tiers).
   - OS Keyring and SQLite DB management tools (Export, Import, Backup, Restore).
3. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Web Cockpit Listener** (`src/dashboard/mod.rs`): Dedicated Axum/Hyper listener on `127.0.0.1:9100`.
- **Real-Time SSE Feed** (`src/dashboard/sse.rs`): Broadcasts metrics snapshots every 500ms to connected browsers.
- **REST Control API** (`src/dashboard/api.rs`): Endpoints for `/api/status`, `/api/providers`, `/api/agents`, `/api/memory`, `/api/graph`, and `/api/control`.
- **Embedded Glassmorphic UI** (`src/dashboard/assets/`): Zero-dependency vanilla JS and modern CSS delivering instant load times and fluid animations.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- dashboard` passes 100%.
- `toksi dashboard --open` launches the server in the background and prints the local dashboard URL.
