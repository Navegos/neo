# Phase 12: TUI, Prometheus Telemetry & Web Dashboard

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/TokenTracker, neo/inspect-vault/bifrost is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and telemetry metrics into consolidated toksi.

Execute Phase 12: TUI, Prometheus Telemetry & Web Dashboard.
Sources:
  - neo/inspect-vault/TokenTracker
  - neo/inspect-vault/bifrost (dashboard assets, metrics)
Targets:
  - crates/toksi/src/telemetry/
  - crates/toksi/src/tui/
  - crates/toksi/src/dashboard/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Dedicated Port Isolation:
   - Port 9099: Dedicated strictly to LLM inference and Prometheus metrics exposition (`/metrics`).
   - Port 9100: Dedicated strictly to Web Dashboard Cockpit. Large browser downloads or open SSE tabs must NEVER introduce TCP backpressure on LLM streaming on 9099.
4. Ground-Truth Token Accounting: Real tokens parsed from live SSE streams. Strict prohibition of synthetic/virtual tokens (idle reads exactly 0).
5. Non-Blocking TUI Event Loop: Non-blocking event polling to prevent terminal hangs and input freezing.

Requirements:
1. Telemetry Subsystem (`crates/toksi/src/telemetry/`):
   - Native Prometheus `/metrics` exposition on Port `9099` (or `config.toml` `[ports].gateway`).
   - Lock-free, zero-latency async metrics collector channel (`tokio::sync::mpsc`) & `watch::Sender<DashboardSnapshot>`.
   - Ground-truth token tracking (`TokenTracker` 47 parsers): prompt tokens, completion tokens, `cache_creation_input_tokens`, Responses usage, cached tokens, and headroom compression savings (`0` when idle; zero synthetic `0.38/0.42/0.20`, `1428 KB`, or `50/50`/`0.0005` stubs).
   - Real-time USD cost and savings calculator using official model pricing catalogs.
2. Terminal UI (`crates/toksi/src/tui/`):
   - Interactive **9-tab** mouse-capable TUI built on `ratatui` + `crossterm::event::EnableMouseCapture`.
   - Exact 9 views matching Web Cockpit verbatim: `Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`.
   - Fix terminal hangs: non-blocking async `watch::Receiver` event loop (`< 5ms` navigation).
3. Web Telemetry Cockpit (`crates/toksi/src/dashboard/`):
   - Detached background HTTP server on `127.0.0.1:9100` (`toksi dashboard start|stop|restart [--foreground]`).
   - Bundled futuristic SPA assets (HTML/CSS/JS) embedded via `include_str!` with exact 9 tabs above.
   - Live SSE metrics feed broadcasting authentic token savings (configured-only agents & `<byok, model, effort>` profiles), clickable `.heatmap-cell` modal, OS Keyring (`gemini:antigravity` normalized), and real DB Export/Import/Backup/Restore.
4. Ensure 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Prometheus Text Exporter** (`src/telemetry/prometheus.rs`): Formats counters, gauges, and histograms (`toksi_prompt_tokens_total`, `toksi_savings_tokens_total`, `toksi_request_duration_seconds`) for scraping.
- **Lock-Free Collector** (`src/telemetry/collector.rs`): Worker task draining bounded channel events into atomic counters and rolling latency buffers without blocking the request pipeline.
- **TUI State Machine** (`src/tui/app.rs`): Renders responsive terminal layouts with tab navigation and auto-resizing.
- **Web Telemetry Server** (`src/dashboard/`): Lightweight Axum/Hyper listener on Port 9100 streaming SSE events to browser clients.

### 2. Disciplinary Standards & Invariants
- When no inference traffic is active, total token savings must read 0. Zero phantom token counting.
- Web assets must be self-contained; zero external CDN dependencies at runtime.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- telemetry` passes 100%.
- `curl http://127.0.0.1:9099/metrics` returns valid Prometheus text metrics.
