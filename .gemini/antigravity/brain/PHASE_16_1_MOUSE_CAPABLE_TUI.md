# Phase 16.1: Mouse-Capable Interactive TUI

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.1: Mouse-Capable Interactive TUI.
Targets:
  - crates/toksi/src/tui/mod.rs
  - crates/toksi/src/tui/app.rs
  - crates/toksi/src/tui/views/mod.rs
  - crates/toksi/src/tui/views/monitor.rs
  - crates/toksi/src/tui/views/agents.rs
  - crates/toksi/src/tui/views/gateway.rs
  - crates/toksi/src/tui/widgets/mod.rs
  - crates/toksi/src/commands/tui.rs

Requirements:
1. Mouse Capture Enablement:
   - In `tui/mod.rs`, enable mouse capture during terminal initialization:
     `execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)?;`
   - Ensure `TerminalGuard::drop` cleanly restores terminal state:
     `let _ = execute!(stdout(), Show, LeaveAlternateScreen, DisableMouseCapture);`
2. Interactive Mouse Event Handling in `tui/app.rs`:
   - Handle `Event::Mouse(MouseEvent)`:
     * Tab Switching: Clicking the tab bar detects column bounds and switches active tab (Monitor, Agents, Gateway, Memory, CodeGraph, Headroom, Logs).
     * Row Selection: Clicking a row in the agent table or provider list selects that entity and shows its detail inspector.
     * Action Buttons: Clicking UI action buttons (e.g. `[Wire]`, `[Unwire]`, `[Precheck]`, `[Test Provider]`, `[Sync Rules]`) triggers the corresponding action.
     * Mouse Wheel: Scrolling mouse wheel up/down scrolls the active list, log stream, or detail panel.
3. Aesthetic Visual Polish:
   - High-contrast, clean dark theme styling with Ratatui blocks, sparklines, gauges, and tables.
   - Display real connected agent status, live token savings, and provider health.
4. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Terminal Mouse Setup in `tui/mod.rs`
```rust
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)?;

    let mut guard = TerminalGuard;
    ...
}
```

### 2. Mouse Click Dispatch in `tui/app.rs`
```rust
match event {
    Event::Mouse(mouse_event) => {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                app.handle_mouse_click(mouse_event.column, mouse_event.row);
            }
            MouseEventKind::ScrollDown => {
                app.scroll_down();
            }
            MouseEventKind::ScrollUp => {
                app.scroll_up();
            }
            _ => {}
        }
    }
    _ => {}
}
```

