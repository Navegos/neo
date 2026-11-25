# Phase 12: TUI, Prometheus Telemetry & Web Dashboard

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

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
   - Interactive **10-tab** (Quotas separated, Activity Room renamed) mouse-capable TUI built on `ratatui` + `crossterm::event::EnableMouseCapture`.
   - Exact 9 views matching Web Cockpit verbatim: `Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`.
   - Fix terminal hangs: non-blocking async `watch::Receiver` event loop (`< 5ms` navigation).
3. Web Telemetry Cockpit (`crates/toksi/src/dashboard/`):
   - Detached background HTTP server on `127.0.0.1:9100` (`toksi dashboard start|stop|restart [--foreground]`).
   - Bundled futuristic SPA assets (HTML/CSS/JS) embedded via `include_str!` with exact 10 tabs (PHASE_20_2).
   - Live SSE metrics feed broadcasting authentic token savings (configured-only agents & `<byok, model, effort>` profiles), clickable `.heatmap-cell` modal, `skill-scanner` CEL/SARIF + `Navegos/advisory` O(1) prefix search (Tab 5), `defenseclaw` fail-closed hookguard & audit evidence + ADR/sandbox/kill-switch events (Tab 6), interactive Plugin Manager `/api/plugins` + External `<skills>` sync (Tab 7), OS Keyring (`gemini:antigravity` normalized), and real DB Export/Import/Backup/Restore including `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events` (Tab 9).
4. Wire `tokview`, `phoenix` OpenInference spans, `tokenspeed` TTFT/TPOT, `vmon`/`cuda-rust` GPU telemetry across **35-component `<toksi-siagent>`** (**495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, externalized zero-bloat `<skills>` & `<advisory>`), and ensure 0 compiler warnings and targeted `cargo test` pass.
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

---

## Wave 18 (v5.2) Synchronization Addendum
- Surface `tokview` + `langfuse` + `phoenix` OpenInference traces, `tokenspeed` latency percentiles, `vmon` + `cuda-rust` GPU telemetry, `opa`/`cedar`/`spire` attestation checks, and `temporal`/`restate`/`langgraph` checkpoints across all 9 TUI & `:9100` Web Cockpit tabs for **35-component `<toksi-siagent>`** (**495 mandated vaults**, **43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, externalized `<skills>` & `<advisory>`).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19–20 Reconciliation:** Synchronized to Wave 20 / v7.0 (`493` deep-researched vaults, `49` `.agents/brain/*.md` guides, dedicated `toksi _worker <role>` subprocesses, `ctxmode`/`aimemory` renames, `docker/Dockerfile.test` + `docker/compose.test.yaml`, separated `~/.toksi/services/*.db`, detached `:9100` **10-Tab** UI, `toksi cluster`, `src/telemetry/otel.rs`, and **29** slash commands).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
