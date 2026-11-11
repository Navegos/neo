# Phase 16.1: High-Performance 9-Tab Async Mouse-Capable TUI

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.1: High-Performance 9-Tab Async Mouse-Capable Interactive TUI.
Targets:
  - crates/toksi/src/tui/mod.rs
  - crates/toksi/src/tui/app.rs
  - crates/toksi/src/tui/views/mod.rs
  - crates/toksi/src/tui/views/* (align exact 9 tabs matching Web Cockpit)
  - crates/toksi/src/tui/widgets/button.rs
  - crates/toksi/src/commands/tui.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors.
3. Mouse-Capable 9-Tab Interactive TUI: Enable crossterm `EnableMouseCapture`. Align exact 9 views matching the Web Cockpit (`Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`).
4. Non-Blocking 60 FPS Event Loop: Receive lock-free snapshots via `tokio::sync::watch::Receiver<DashboardSnapshot>`; never run synchronous SQLite/HTTP calls inside the render/event loop so keys (`1..9`, `Tab`, `Shift+Tab`, arrows) and mouse clicks respond in `< 5ms` with zero hangs.
5. Authentic Telemetry & Zero Paid-Tier Gates: True token consumption (`0` when idle; delete `init_waterfall_samples()` fake data), configured-only visibility in Savings/Gateway/Keyring, PATH availability grey-out in Fleet, clickable heatmap detail popup, multi-instance `<byok, model, effort>` routing controls, normalized Keyring (`gemini:antigravity`), DB export/import controls, and ZERO `[PAID]`/`[DISQUALIFIED-FREE]` badges.

Requirements:
1. Mouse-Enabled 9-Tab Interactive TUI:
   - Enable mouse capture in Crossterm (`EnableMouseCapture` on start, `DisableMouseCapture` on exit).
   - Route mouse click events in `app.rs`:
     * Clicking tabs instantly switches all 9 views (`Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`).
     * Clicking table rows selects configured agents, `<byok, model, effort>` profiles, or memory DAG nodes/heatmap cells for drilldown inspection.
     * Clicking action buttons triggers operations (`Direct-Wire`, `Config-Wire`, `Unwire`, `Fix`, `Refresh Models`, `Export/Import DB`, `Daemon Start/Stop`).
     * Mouse wheel smoothly scrolls logs and inspection panels.
2. Fix Terminal Hangs & Input Locking:
   - Replace blocking event/DB calls with `crossterm::event::poll(Duration::from_millis(16))` reading background-collected `watch::Receiver<DashboardSnapshot>`.
   - Ensure keyboard navigation (`1..9`, `Tab`, arrows) and mouse clicks never hang or lock up.
3. Futuristic Telemetry UI & Clean Configured Data:
   - High-contrast cyber-slate/cyan/emerald/amber aesthetic matching Web Cockpit.
   - Show true configured token savings, `direct-wired` vs `config-wired` badges, PATH availability, read-only plan & `hour..year` quota windows without `[PAID]`/`[DISQUALIFIED-FREE]` restrictions.
4. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Mouse Capture Lifecycle** (`src/tui/mod.rs`): Initializes terminal in alternate screen with raw mode and `EnableMouseCapture`. Clean exit hook guarantees terminal restoration.
- **Event Dispatcher & Click Hit-Testing** (`src/tui/app.rs`): Maps screen coordinates `(x, y)` to UI element hit-boxes (tabs, table rows, buttons) to trigger actions.
- **Asynchronous Loop Architecture**: Uses `tokio::select!` interleaving terminal input events, background metric updates, and periodic redraws.

### 2. Disciplinary Standards & Invariants
- Terminal state must be restored cleanly even on abnormal termination or panic via custom panic hook.
- CPU usage of the idle TUI must remain <0.5%.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- tui` passes 100%.
- TUI launches cleanly with `toksi tui` and responds to mouse clicks and arrow keys without freezing.
