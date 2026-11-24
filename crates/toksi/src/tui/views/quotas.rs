// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/views/quotas.rs
// created: 2026-11-24
// lastModified: 2026-11-24

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render_quotas_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(7),
        ])
        .split(area);

    let summary_block = Block::default()
        .title(" [Tab 4] Quotas & Rate-Limit Telemetry (hour/day/week/month/year) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let summary_lines = vec![
        Line::from(vec![
            Span::styled("Tracked Quota Windows: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} ", app.headroom.quota_windows.len()),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::styled("│ BYOK Port: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(":{}", app.headroom.byok_port),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("429 Backoffs: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} ", app.headroom.rate_limit_429_count),
                Style::default().fg(Color::White),
            ),
            Span::styled("│ 5xx Reroutes: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} ", app.headroom.reroute_count),
                Style::default().fg(Color::White),
            ),
            Span::styled("│ Press [r] to refresh quota windows", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    f.render_widget(
        Paragraph::new(summary_lines).block(summary_block).wrap(Wrap { trim: false }),
        chunks[0],
    );

    let rows: Vec<Row> = if app.headroom.quota_windows.is_empty() {
        vec![Row::new(vec![
            Cell::from("-"),
            Cell::from("No quota windows recorded yet"),
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("-"),
        ])]
    } else {
        app.headroom
            .quota_windows
            .iter()
            .map(|q| {
                let req = match (q.requests_remaining, q.requests_limit) {
                    (Some(rem), Some(lim)) => format!("{}/{}", rem, lim),
                    _ => "-".to_string(),
                };
                let tok = match (q.tokens_remaining, q.tokens_limit) {
                    (Some(rem), Some(lim)) => format!("{}/{}", rem, lim),
                    _ => "-".to_string(),
                };
                Row::new(vec![
                    Cell::from(q.provider_id.clone()),
                    Cell::from(q.profile_id.clone()),
                    Cell::from(q.window_kind.clone()),
                    Cell::from(req),
                    Cell::from(tok),
                    Cell::from(q.plan_label.clone()),
                ])
            })
            .collect()
    };

    let header = Row::new(vec![
        Cell::from("Provider").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Profile").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Window").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Requests").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Tokens").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Plan").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(22),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Live Provider Quota Windows ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );
    f.render_widget(table, chunks[1]);

    let route_lines: Vec<Line> = app
        .headroom
        .routes
        .iter()
        .take(5)
        .map(|r| {
            Line::from(vec![
                Span::styled(format!("{} ", r.provider), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} ", r.tier_badge), Style::default().fg(Color::Green)),
                Span::styled(r.quota_display.clone(), Style::default().fg(Color::Gray)),
            ])
        })
        .collect();

    let routes_block = Block::default()
        .title(" Configured Route Tiers ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(
        Paragraph::new(route_lines).block(routes_block).wrap(Wrap { trim: false }),
        chunks[2],
    );
}
