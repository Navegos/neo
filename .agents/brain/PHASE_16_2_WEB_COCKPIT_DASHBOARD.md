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
4. Futuristic Cyberpunk Glassmorphic UI (Exact 9 Tabs): `Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`.
5. Authentic Telemetry & Zero Synthetic Stubs: Delete fake waterfall ratios (`0.38/0.42/0.20`), hardcoded `68.0%`, fake agent latencies, `50/50`/`0.0005` gateway stubs, fake `1428 KB` vacuum savings, fake `:8788 BYOK` label, fake 6-symbol CodeGraph list, fake heatmap vec, and `[PAID]`/`[DISQUALIFIED-FREE]` badges.

Requirements:
1. Isolated Background Web Cockpit on Port `9100` (or `~/.toksi/config.toml` `[ports].dashboard`):
   - Detached background execution by default via `toksi dashboard start|stop|restart` (`--foreground` stays attached; `--open` launches browser).
   - Single-page application assets (HTML/CSS/JS) embedded at compile-time via `include_str!`.
2. Full Interactive Telemetry & Control Capabilities Across All 9 Tabs:
   - **`Savings`**: Configured-only agents & `<byok, model, effort>` individual + overall token/USD savings (`0` at rest) and per-tool attribution (`rtk`, `headroom`, `mcp_pruner`, `ccr`, `cache_align`, `context`, `economizer`).
   - **`The War Room`**: Live per-agent tool activity, real 429 backoff/failover cascade state, `ArcSwap` reconnect watchdog logs.
   - **`AI Agent Fleet & Quotas`**: All supported agents across CLIs/IDEs (65/90+ catalog); grey out binaries missing from `PATH` with remediation hint; one-click `Direct-Wire`/`Config-Wire`/`Unwire`; read-only plan & `hour|day|week|month|year` quota gauges (zero paywall blocks).
   - **`Memory Decision DAG`**: Real `projectmem` decision/issue/attempt/fix lineage + **clickable `.heatmap-cell` modal drilldown** (`GET /api/memory/heatmap/:cell_id`) + monorepo subproject filter.
   - **`CodeGraph Explorer`**: Live `symbols.db` stats (`41+` Tree-sitter languages), caller/impact explorer, `ast-grep`/SCIP/Repomix preview, `.ignore` purge action.
   - **`Sandbox Inspector`**: Real OS sandbox telemetry, `< 2KB` projected stdout, FTS5 search test bar, URL cache monitor.
   - **`System Controls`**: Live `~/.toksi/config.toml` editor, `daemon|proxy|dashboard` controls, granular per-agent component installer (`hooks|mcp|skills|rules|instructions|configs`) for user-auto vs explicit-repo opt-in, live logs.
   - **`AI Gateway & Quotas`**: Multi-instance `<byok, model, effort>` CRUD (multiple profiles per BYOK provider with editable custom `base_url` + zero-cost `/v1/models` refresh), per-agent & per-`<byok, model, effort>` Dynamic Complexity Routing & Load Balancer algorithm selector, Headroom bypass monitor.
   - **`Keyring & Database`**: Configured-only OS Keyring entries (normalized `gemini:antigravity` alias), SQLite DB Export (`POST /api/db/export`), Import (`POST /api/db/import` with `overwrite`), Backup/Restore, legacy nested `<repo>` repair.
3. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
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
