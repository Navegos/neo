# Phase 16.1: Mouse-Capable Interactive TUI

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_1_MOUSE_CAPABLE_TUI.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.1: Mouse-Capable Interactive TUI.
Targets:
  - crates/toksi/src/tui/mod.rs
  - crates/toksi/src/tui/app.rs
  - crates/toksi/src/tui/views/*
  - crates/toksi/src/tui/widgets/*
  - crates/toksi/src/commands/tui.rs

Requirements:
1. Mouse-Enabled Interactive TUI:
   - Enable mouse capture in Crossterm (`EnableMouseCapture` on start, `DisableMouseCapture` on exit).
   - Route mouse click events in `app.rs`: clicking tabs switches tabs, clicking rows selects agents/providers, mouse wheel scrolls logs.
2. Fix Terminal Hangs & Input Locking:
   - Replace blocking event calls with non-blocking async event polling in the TUI event loop.
   - Ensure switching between tabs (Monitor, Agents, Memory, Graph, Headroom, Logs) and clicking buttons never hangs or locks keyboard input.
3. Realistic Real-Time Telemetry:
   - Show true token consumption, ground-truth savings, and active BYOK quota gauges. When zero agents are running, stats read 0.
4. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Crossterm Mouse Capture in `tui/mod.rs`
```rust
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, std::io::Error> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<(), std::io::Error> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()
}
```

### 2. Non-Blocking Event Polling in `tui/mod.rs`
```rust
if crossterm::event::poll(std::time::Duration::from_millis(50))? {
    match crossterm::event::read()? {
        Event::Key(key) => app.handle_key(key),
        Event::Mouse(mouse) => app.handle_mouse(mouse),
        _ => {}
    }
}
```
