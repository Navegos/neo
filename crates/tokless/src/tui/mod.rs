// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/mod.rs

pub mod app;
pub mod views;
pub mod widgets;

#[cfg(test)]
mod tests;

use std::io::stdout;
use std::time::Duration;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

pub use app::App;
use views::*;
use widgets::{render_footer, render_header, render_tab_bar};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
    }
}

pub fn run_tui() -> i32 {
    run_tui_with_tab(0)
}

pub fn run_tui_with_tab(initial_tab: usize) -> i32 {
    let is_tty = crossterm::tty::IsTty::is_tty(&stdout());
    if !is_tty {
        eprintln!("tokless tui requires an interactive terminal (TTY)");
        return 1;
    }

    if enable_raw_mode().is_err() {
        eprintln!("Failed to enable terminal raw mode");
        return 1;
    }

    let _guard = TerminalGuard;
    let mut stdout = stdout();
    if execute!(stdout, EnterAlternateScreen, Hide).is_err() {
        return 1;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to initialize ratatui terminal backend: {}", e);
            return 1;
        }
    };

    let mut app = App::new();
    app.current_tab = initial_tab.min(app.tab_names.len().saturating_sub(1));

    let frame_duration = Duration::from_millis(16); // ~60fps target

    while !app.should_quit {
        if let Err(e) = terminal.draw(|f| render_ui(f, &mut app)) {
            eprintln!("Terminal draw error: {}", e);
            break;
        }

        // 60fps non-blocking event polling
        if event::poll(frame_duration).unwrap_or(false) {
            if let Ok(ev) = event::read() {
                match ev {
                    Event::Key(key) => {
                        // On Windows, ignore Release events to avoid double actions
                        if key.kind == KeyEventKind::Press {
                            app.handle_key(key);
                        }
                    }
                    Event::Resize(_, _) => {
                        // Terminal resize is automatically redrawn next loop iteration
                    }
                    _ => {}
                }
            }
        }

        app.tick();
    }

    0
}

fn render_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Top Status Bar
            Constraint::Length(2), // Navigation Tabs
            Constraint::Min(10),   // Active Tab Viewport
            Constraint::Length(2), // Bottom Keybindings Footer
        ])
        .split(size);

    render_header(f, chunks[0], app);
    render_tab_bar(f, chunks[1], app);

    match app.current_tab {
        0 => render_monitor_view(f, chunks[2], app),
        1 => render_agents_view(f, chunks[2], app),
        2 => render_memory_view(f, chunks[2], app),
        3 => render_graph_view(f, chunks[2], app),
        4 => render_headroom_view(f, chunks[2], app),
        5 => render_logs_view(f, chunks[2], app),
        _ => {}
    }

    render_footer(f, chunks[3], app);
}
