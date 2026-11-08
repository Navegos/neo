# Phase 16.1: High-Performance Mouse-Capable Interactive TUI

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.1: High-Performance Mouse-Capable Interactive TUI.
Targets:
  - crates/toksi/src/tui/mod.rs
  - crates/toksi/src/tui/app.rs
  - crates/toksi/src/tui/views/mod.rs
  - crates/toksi/src/tui/views/monitor.rs
  - crates/toksi/src/tui/views/agents.rs
  - crates/toksi/src/tui/views/memory.rs
  - crates/toksi/src/tui/views/graph.rs
  - crates/toksi/src/tui/views/headroom.rs
  - crates/toksi/src/tui/views/governance.rs
  - crates/toksi/src/tui/views/logs.rs
  - crates/toksi/src/tui/widgets/button.rs
  - crates/toksi/src/commands/tui.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Mouse-Capable Interactive TUI: Enable crossterm `EnableMouseCapture`. Support tab clicking, entity row selection, action buttons, scrollable views.
4. Non-Blocking Event Loop: Permanently eliminate terminal hangs, input freezes, and tab switching deadlocks.
5. Realistic Real-Time Telemetry: True token consumption, ground-truth savings, and active BYOK quota gauges. When zero agents are running, stats read exactly 0.

Requirements:
1. Mouse-Enabled Interactive TUI:
   - Enable mouse capture in Crossterm (`EnableMouseCapture` on start, `DisableMouseCapture` on exit).
   - Route mouse click events in `app.rs`:
     * Clicking tabs instantly switches views (Monitor, Agents, Memory, Graph, Headroom, Governance, Logs).
     * Clicking table rows selects agents, providers, or memory items for inspection.
     * Clicking action buttons triggers operations (Sync, Fix, Health, Start, Stop).
     * Mouse wheel smoothly scrolls logs and inspection panels.
2. Fix Terminal Hangs & Input Locking:
   - Replace blocking event calls with non-blocking async event polling in the TUI event loop (`crossterm::event::poll(Duration::from_millis(50))`).
   - Decouple terminal render cycle from background telemetry collectors.
   - Ensure keyboard navigation between tabs and buttons never hangs or locks up.
3. Realistic Real-Time Telemetry:
   - Show true token consumption, ground-truth savings, and active BYOK quota gauges.
   - Display active paid commercial tier badges (`[PAID]`, `[DISQUALIFIED-FREE]`).
4. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
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
