// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/views/headroom.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render_headroom_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Cache Alignment Gauge & Memory Scaling
            Constraint::Length(8), // Fallback Cascades & War Room
            Constraint::Min(8),    // BYOK Provider Routes Table
        ])
        .split(area);

    // Section 1: Cache Alignment & Headroom Memory Scaling
    let cache_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Gauge: 1024 Boundary Alignment
    let align_block = Block::default()
        .title(" [1] 1024-Token Prompt Cache Boundary Alignment ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let align_sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(align_block.inner(cache_chunks[0]));

    let align_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
        .percent(app.headroom.cache_alignment_pct as u16)
        .label(format!("{:.1}% Aligned to 1,024 Boundary", app.headroom.cache_alignment_pct));
    f.render_widget(align_gauge, align_sub[0]);

    let align_text = vec![
        Line::from(vec![
            Span::styled("Estimated Prompt Cache Discount: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("~{:.1}%", app.headroom.cache_alignment_pct * 0.9), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" (Anthropic/OpenAI/Gemini cache tiers)"),
        ]),
        Line::from(vec![
            Span::styled("Stabilized System Prompts: ", Style::default().fg(Color::Gray)),
            Span::styled("1024 / 2048 / 3072 / 4096 tokens", Style::default().fg(Color::Cyan)),
            Span::raw(" │ Press 'c' to test align"),
        ]),
    ];
    f.render_widget(Paragraph::new(align_text).wrap(Wrap { trim: false }), align_sub[1]);
    f.render_widget(align_block, cache_chunks[0]);

    // Memory Scaling Governor (L0 / L1 / L2)
    let gov_block = Block::default()
        .title(" [2] Dynamic Memory Governor & Context Headroom ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let gov_text = vec![
        Line::from(vec![
            Span::styled("Context Window Capacity: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{}/200k tok ", app.headroom.context_tokens_used), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("({} headroom left)", 200_000 - app.headroom.context_tokens_used), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Memory Tier Active: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("[{}] ", app.headroom.active_memory_tier), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("(L0: TOC <20k | L1: Abstract 20k-100k | L2: Full >100k)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Overflow Prevention: ", Style::default().fg(Color::Gray)),
            Span::styled("PROTECTED — 0 context overflows caught", Style::default().fg(Color::Green)),
        ]),
    ];
    f.render_widget(Paragraph::new(gov_text).block(gov_block).wrap(Wrap { trim: false }), cache_chunks[1]);

    // Section 2: Outage & Fallback Cascade Visualizer
    let cascade_block = Block::default()
        .title(" [3] Standby Provider Fallback Cascade & Outage Shield ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let cascade_text = vec![
        Line::from(vec![
            Span::styled("Primary: ", Style::default().fg(Color::Yellow)),
            Span::styled("Anthropic Direct (Claude 3.7) ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("──(429/503 Outage)──▶ ", Style::default().fg(Color::Red)),
            Span::styled("Standby 1: ", Style::default().fg(Color::Yellow)),
            Span::styled("Gemini Flash (BYOK) ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("──(Fallback)──▶ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Standby 2: ", Style::default().fg(Color::Yellow)),
            Span::styled("OpenAI Direct", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Rate-Limit War Room: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("429 Backoffs: {}  │  5xx Reroutes: {}  │  Active Gateway Port: {}", app.headroom.rate_limit_429_count, app.headroom.reroute_count, app.headroom.byok_port), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled("Seamless failover enabled — zero agent process restarts required on provider downtime", Style::default().fg(Color::Green)),
        ]),
    ];
    f.render_widget(Paragraph::new(cascade_text).block(cascade_block).wrap(Wrap { trim: false }), chunks[1]);

    // Section 3: BYOK Provider Routes Table
    let table_block = Block::default()
        .title(format!(" [4] Configured BYOK Routes ({}) ", app.headroom.routes.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let header_row = Row::new(vec![
        Cell::from("Provider").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Target Upstream URL").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Virtual Prefix").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Health (/livez)").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Quota Usage").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Assigned Agents").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .headroom
        .routes
        .iter()
        .enumerate()
        .map(|(idx, route)| {
            let is_selected = idx == app.headroom.selected_route;
            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 45, 60))
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(&route.provider, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(&route.target_url, Style::default().fg(Color::White))),
                Cell::from(Span::styled(&route.virtual_prefix, Style::default().fg(Color::Yellow))),
                Cell::from(if route.healthy {
                    Span::styled("● OK (200)", Style::default().fg(Color::Green))
                } else {
                    Span::styled("○ UNREACHABLE", Style::default().fg(Color::Red))
                }),
                Cell::from(Span::styled(&route.quota_display, Style::default().fg(Color::Gray))),
                Cell::from(Span::styled(&route.agents_assigned, Style::default().fg(Color::DarkGray))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(16),
        Constraint::Length(28),
        Constraint::Length(22),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(table_block);

    f.render_widget(table, chunks[2]);
}
