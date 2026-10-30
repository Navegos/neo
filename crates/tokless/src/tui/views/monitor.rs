// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/views/monitor.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table},
    Frame,
};

use crate::tui::app::App;

pub fn render_monitor_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Metric Cards
            Constraint::Min(10),   // Waterfall Table
        ])
        .split(area);

    // Top metrics: 4 columns
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    // Card 1: Tokens Saved
    let tokens_card = Block::default()
        .title(" Tokens Saved ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let tokens_text = vec![
        Line::from(vec![
            Span::styled(
                format!("{:>12}", format_count(app.monitor.tokens_saved_lifetime)),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" total"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:>12}", format_count(app.monitor.tokens_saved_today)),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" today"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{:>12}", format_count(app.monitor.tokens_saved_7d)),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" 7 days"),
        ]),
    ];
    f.render_widget(Paragraph::new(tokens_text).block(tokens_card), top_cols[0]);

    // Card 2: Dollar Savings
    let dollar_card = Block::default()
        .title(" Dollar Savings (USD) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let dollar_text = vec![
        Line::from(vec![
            Span::styled(
                format!("  ${:.2}", app.monitor.cost_saved_usd),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" saved"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  {} calls", format_count(app.monitor.total_calls)),
                Style::default().fg(Color::White),
            ),
            Span::raw(" processed"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  ~${:.4}", if app.monitor.total_calls > 0 { app.monitor.cost_saved_usd / app.monitor.total_calls as f64 } else { 0.0 }),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" / call"),
        ]),
    ];
    f.render_widget(Paragraph::new(dollar_text).block(dollar_card), top_cols[1]);

    // Card 3: Compression Ratio & Sparkline
    let _comp_card = Block::default()
        .title(" Compression Rate ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let comp_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(top_cols[2]);

    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Magenta).bg(Color::DarkGray))
        .percent(app.monitor.compression_pct.clamp(0.0, 100.0) as u16)
        .label(format!("{:.1}% reduced", app.monitor.compression_pct));
    f.render_widget(gauge, comp_chunks[0]);

    let sparkline = Sparkline::default()
        .block(Block::default().title(" History "))
        .style(Style::default().fg(Color::Cyan))
        .data(&app.monitor.sparkline_data);
    f.render_widget(sparkline, comp_chunks[1]);
    f.render_widget(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)), top_cols[2]);

    // Card 4: Cache Warmth & Services
    let cache_card = Block::default()
        .title(" Prompt Cache & Services ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let cache_text = vec![
        Line::from(vec![
            Span::raw("Alignment: "),
            Span::styled(
                format!("{:.1}% ", app.monitor.cache_warmth_pct),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("(~90% prompt discount)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("Headroom:  "),
            if app.monitor.proxy_up {
                Span::styled("● ACTIVE (8787)", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ STOPPED", Style::default().fg(Color::Red))
            },
        ]),
        Line::from(vec![
            Span::raw("Daemon:    "),
            if app.monitor.daemon_up {
                Span::styled("● RUNNING", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ STANDBY", Style::default().fg(Color::DarkGray))
            },
        ]),
    ];
    f.render_widget(Paragraph::new(cache_text).block(cache_card), top_cols[3]);

    // Lower: Waterfall Table
    let waterfall_block = Block::default()
        .title(" [Tab 1] Live Token Compression Waterfall & Real-Time Stream ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let header_row = Row::new(vec![
        Cell::from("Time").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Agent").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Endpoint / Model").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Raw ──▶ Compressed").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Savings").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Pipeline Transformations").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Saved $").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .monitor
        .waterfall
        .iter()
        .rev()
        .take(area.height.saturating_sub(11) as usize)
        .map(|item| {
            let agent_color = match item.agent.as_str() {
                "Claude Code" | "Claude" => Color::Magenta,
                "Cursor" => Color::Cyan,
                "Antigravity" => Color::Blue,
                "OpenCode" => Color::Green,
                "Codex" => Color::Yellow,
                _ => Color::White,
            };

            Row::new(vec![
                Cell::from(Span::styled(&item.time, Style::default().fg(Color::DarkGray))),
                Cell::from(Span::styled(&item.agent, Style::default().fg(agent_color).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(&item.endpoint, Style::default().fg(Color::White))),
                Cell::from(Span::styled(
                    format!("{}k ──▶ {}k", item.raw_tokens as f64 / 1000.0, item.compressed_tokens as f64 / 1000.0),
                    Style::default().fg(Color::Cyan),
                )),
                Cell::from(Span::styled(
                    format!("-{:.1}%", item.savings_pct),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(&item.pipeline, Style::default().fg(Color::Gray))),
                Cell::from(Span::styled(format!("+${:.3}", item.saved_usd), Style::default().fg(Color::Green))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(9),
        Constraint::Length(14),
        Constraint::Length(26),
        Constraint::Length(22),
        Constraint::Length(10),
        Constraint::Min(25),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(waterfall_block);

    f.render_widget(table, chunks[1]);
}

fn format_count(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
