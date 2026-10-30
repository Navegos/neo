// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/views/logs.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, LogSource};

pub fn render_logs_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter bar
            Constraint::Min(10),   // Logs viewport
        ])
        .split(area);

    let sources = [
        (LogSource::All, " [1: All Logs] "),
        (LogSource::Proxy, " [2: Headroom Proxy] "),
        (LogSource::Daemon, " [3: Daemon] "),
        (LogSource::Watchdog, " [4: Watchdog] "),
    ];

    let mut header_spans = vec![
        Span::styled("Log Source: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ];

    for (src, label) in sources {
        if app.logs.source == src {
            header_spans.push(Span::styled(label, Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)));
        } else {
            header_spans.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
        }
        header_spans.push(Span::raw(" "));
    }

    header_spans.push(Span::raw("│ "));
    header_spans.push(Span::styled("Follow [f]: ", Style::default().fg(Color::Yellow)));
    if app.logs.follow {
        header_spans.push(Span::styled("● ON (Auto-Scroll)", Style::default().fg(Color::Green)));
    } else {
        header_spans.push(Span::styled("○ PAUSED", Style::default().fg(Color::DarkGray)));
    }

    header_spans.push(Span::raw(" │ "));
    header_spans.push(Span::styled("Clear [c]  │  Scroll [Up/Down/PgUp/PgDn]", Style::default().fg(Color::DarkGray)));

    f.render_widget(Paragraph::new(Line::from(header_spans)).block(Block::default().borders(Borders::BOTTOM)), chunks[0]);

    // Viewport
    let filtered_logs = app.logs.filtered_logs();
    let total_lines = filtered_logs.len();

    let display_lines: Vec<Line> = filtered_logs
        .iter()
        .map(|entry| {
            let level_style = match entry.level.to_uppercase().as_str() {
                "ERROR" | "ERR" | "FATAL" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                "WARN" | "WARNING" => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                "INFO" => Style::default().fg(Color::Cyan),
                "DEBUG" | "TRACE" => Style::default().fg(Color::DarkGray),
                _ => Style::default().fg(Color::White),
            };

            let src_badge = match entry.source {
                LogSource::Proxy => Span::styled("[PROXY] ", Style::default().fg(Color::Magenta)),
                LogSource::Daemon => Span::styled("[DAEMON] ", Style::default().fg(Color::Green)),
                LogSource::Watchdog => Span::styled("[WATCHDOG] ", Style::default().fg(Color::Yellow)),
                _ => Span::raw(""),
            };

            Line::from(vec![
                Span::styled(&entry.timestamp, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                src_badge,
                Span::styled(format!("[{}] ", entry.level), level_style),
                Span::styled(&entry.message, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let available_height = chunks[1].height.saturating_sub(2) as usize;
    let scroll = if app.logs.follow {
        total_lines.saturating_sub(available_height) as u16
    } else {
        app.logs.scroll_offset as u16
    };

    let log_block = Block::default()
        .title(format!(
            " [Tab 6] Real-Time Diagnostic Stream ({} events) ",
            total_lines
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(display_lines)
        .block(log_block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, chunks[1]);
}
