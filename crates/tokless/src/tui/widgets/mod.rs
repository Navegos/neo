// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/widgets/mod.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::util::version::tokless_version;

/// Renders top status banner with version, daemon, and proxy health indicators.
pub fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let proxy_status = if app.monitor.proxy_up {
        Span::styled(" [Proxy: UP 8787] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [Proxy: DOWN] ", Style::default().fg(Color::Red))
    };

    let daemon_status = if app.monitor.daemon_up {
        Span::styled(" [Daemon: OK] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [Daemon: OFF] ", Style::default().fg(Color::DarkGray))
    };

    let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let header_line = Line::from(vec![
        Span::styled("⚡ TOKLESS ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("v{} ", tokless_version()), Style::default().fg(Color::DarkGray)),
        Span::raw("│ "),
        proxy_status,
        daemon_status,
        Span::raw("│ "),
        Span::styled(format!("cwd: {} ", app.cwd_display), Style::default().fg(Color::DarkGray)),
        Span::raw("│ "),
        Span::styled(now_str, Style::default().fg(Color::White)),
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(header_line).block(block);
    f.render_widget(paragraph, area);
}

/// Renders the 6 navigation tabs.
pub fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = app
        .tab_names
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let key = format!("[{}] ", idx + 1);
            if idx == app.current_tab {
                Line::from(vec![
                    Span::styled(key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(*name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(key, Style::default().fg(Color::DarkGray)),
                    Span::styled(*name, Style::default().fg(Color::Gray)),
                ])
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(app.current_tab)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));

    f.render_widget(tabs, area);
}

/// Renders bottom keybinding helper line.
pub fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let mut keys = vec![
        Span::styled("Tab/1-6", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(":Tab  "),
    ];

    match app.current_tab {
        0 => {
            keys.push(Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Refresh  "));
            keys.push(Span::styled("u", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Proxy Up  "));
            keys.push(Span::styled("d", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Proxy Down  "));
        }
        1 => {
            keys.push(Span::styled("Space", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Wire/Unwire  "));
            keys.push(Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Inspect MCP  "));
            keys.push(Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Scan  "));
        }
        2 => {
            keys.push(Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Search  "));
            keys.push(Span::styled("p", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Run Precheck  "));
            keys.push(Span::styled("f", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Filter  "));
            keys.push(Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Detail  "));
        }
        3 => {
            keys.push(Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Search Symbol  "));
            keys.push(Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Blast Radius  "));
            keys.push(Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Toggle Tree  "));
            keys.push(Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Reindex  "));
        }
        4 => {
            keys.push(Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Reload Routes  "));
            keys.push(Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Cache Align Test  "));
        }
        5 => {
            keys.push(Span::styled("1-4", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Source  "));
            keys.push(Span::styled("f", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Follow  "));
            keys.push(Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            keys.push(Span::raw(":Clear  "));
        }
        _ => {}
    }

    keys.push(Span::styled("q/Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    keys.push(Span::raw(":Quit"));

    if let Some(ref msg) = app.status_message {
        keys.push(Span::raw(" │ "));
        keys.push(Span::styled(msg, Style::default().fg(Color::Green)));
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(Line::from(keys)).block(block);
    f.render_widget(paragraph, area);
}

/// Helper to render a centered modal dialog box.
pub fn render_modal(
    f: &mut Frame,
    title: &str,
    lines: Vec<Line>,
    scroll: u16,
    percent_x: u16,
    percent_y: u16,
) {
    let size = f.area();
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(size);

    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1];

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            format!(" {} (Press Esc to Close) ", title),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
