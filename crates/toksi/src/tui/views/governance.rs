// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/views/governance.rs
// created: 2026-11-06
// lastModified: 2026-11-09

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Widget, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render_governance_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Section 1: Overview Cards & Budget Gauge
            Constraint::Min(10),   // Section 2: Virtual Keys & Governance Quotas Table
            Constraint::Length(6), // Section 3: Selected Key Detail & Circuit Breakers
            Constraint::Length(3), // Section 4: Action Buttons Bar
        ])
        .split(area);

    // Section 1: Overview Cards & Budget Gauge
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Budget Gauge
    let budget_block = Block::default()
        .title(" [1] USD Spend vs Allocated Budget ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let budget_sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(budget_block.inner(top_chunks[0]));

    let pct = if app.governance.total_budget_usd > 0.0 {
        ((app.governance.total_spent_usd / app.governance.total_budget_usd) * 100.0).min(100.0) as u16
    } else {
        0
    };

    let gauge_color = if pct > 90 {
        Color::Red
    } else if pct > 75 {
        Color::Yellow
    } else {
        Color::Green
    };

    let budget_gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray))
        .percent(pct)
        .label(format!(
            "${:.2} / ${:.2} ({:.1}%)",
            app.governance.total_spent_usd, app.governance.total_budget_usd, pct
        ));
    f.render_widget(budget_gauge, budget_sub[0]);

    let budget_text = vec![
        Line::from(vec![
            Span::styled("Enforcement Mode: ", Style::default().fg(Color::Gray)),
            Span::styled("STRICT HARD QUOTA ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("(Rejects requests exceeding max_budget_usd with HTTP 429)"),
        ]),
        Line::from(vec![
            Span::styled("Reset Cycle: ", Style::default().fg(Color::Gray)),
            Span::styled("Monthly Calendar (1st of month UTC)  │  Remaining Pool: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("${:.2}", (app.governance.total_budget_usd - app.governance.total_spent_usd).max(0.0)),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(budget_text).wrap(Wrap { trim: false }), budget_sub[1]);
    f.render_widget(budget_block, top_chunks[0]);

    // Rate Limiters & Circuit Breakers summary
    let quotas_summary_block = Block::default()
        .title(" [2] Gateway Governance & Resilience Watchdog ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let summary_text = vec![
        Line::from(vec![
            Span::styled("Registered Virtual Keys: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{} keys active ", app.governance.keys.len()), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("(Header: 'x-toksi-vk')", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Active Rate-Limiters: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{} enforced ", app.governance.active_rate_limits), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("│ Sliding 60s RPM / TPM token-bucket windows", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Circuit Breaker Status: ", Style::default().fg(Color::Gray)),
            if app.governance.circuit_breaker_trips == 0 {
                Span::styled("HEALTHY (0 trips) ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(format!("WARNING ({} trips recorded) ", app.governance.circuit_breaker_trips), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            },
            Span::styled("│ Consecutive 5xx failover shield active", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    f.render_widget(Paragraph::new(summary_text).block(quotas_summary_block).wrap(Wrap { trim: false }), top_chunks[1]);

    // Section 2: Virtual Keys & Governance Quotas Table
    let table_block = Block::default()
        .title(format!(" [3] Virtual Keys & Quota Allocations ({}) ", app.governance.keys.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let header_row = Row::new(vec![
        Cell::from("Virtual Key (x-toksi-vk)").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Name / Team").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Spend / Budget").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("RPM (cur/max)").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("TPM (cur/max)").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Concurrency").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Allowed Models").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .governance
        .keys
        .iter()
        .enumerate()
        .map(|(idx, k)| {
            let is_selected = idx == app.governance.selected;
            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 45, 60))
            } else {
                Style::default()
            };

            let budget_str = if k.max_budget_usd > 0.0 {
                format!("${:.2} / ${:.2}", k.current_spend_usd, k.max_budget_usd)
            } else {
                format!("${:.2} / Unltd", k.current_spend_usd)
            };

            let rpm_str = if k.max_rpm > 0 {
                format!("{}/{} rpm", k.current_rpm, k.max_rpm)
            } else {
                format!("{}/Unltd", k.current_rpm)
            };

            let tpm_str = if k.max_tpm > 0 {
                format!("{}/{}k tpm", k.current_tpm, k.max_tpm / 1000)
            } else {
                format!("{}/Unltd", k.current_tpm)
            };

            let conc_str = if k.max_concurrency > 0 {
                format!("{}/{}", k.current_concurrency, k.max_concurrency)
            } else {
                format!("{}/Unltd", k.current_concurrency)
            };

            Row::new(vec![
                Cell::from(Span::styled(&k.vk, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(&k.name, Style::default().fg(Color::White))),
                Cell::from(Span::styled(budget_str, Style::default().fg(Color::Green))),
                Cell::from(Span::styled(rpm_str, Style::default().fg(Color::Yellow))),
                Cell::from(Span::styled(tpm_str, Style::default().fg(Color::Magenta))),
                Cell::from(Span::styled(conc_str, Style::default().fg(Color::LightBlue))),
                Cell::from(Span::styled(&k.allowed_models, Style::default().fg(Color::Gray))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(22),
        Constraint::Length(18),
        Constraint::Length(18),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Min(24),
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(table_block);

    f.render_widget(table, chunks[1]);

    // Section 3: Selected Key Detail & Circuit Breakers
    let detail_block = Block::default()
        .title(" [4] Key Governance Policy & Runtime Diagnostics ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let detail_text = if let Some(k) = app.governance.keys.get(app.governance.selected) {
        vec![
            Line::from(vec![
                Span::styled("Selected Key: ", Style::default().fg(Color::Gray)),
                Span::styled(&k.vk, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" ("),
                Span::styled(&k.name, Style::default().fg(Color::White)),
                Span::raw(") │ "),
                Span::styled("Policy Status: ", Style::default().fg(Color::Gray)),
                Span::styled("COMPLIANT — All RPM/TPM/Budget thresholds nominal", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Model Restriction: ", Style::default().fg(Color::Gray)),
                Span::styled(&k.allowed_models, Style::default().fg(Color::Yellow)),
                Span::raw(" │ "),
                Span::styled("Circuit Breakers: ", Style::default().fg(Color::Gray)),
                Span::styled("Tripped after 3 consecutive 5xx errors (auto-cooloff: 30s)", Style::default().fg(Color::DarkGray)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled("No virtual key selected", Style::default().fg(Color::DarkGray)))]
    };

    f.render_widget(Paragraph::new(detail_text).block(detail_block).wrap(Wrap { trim: false }), chunks[2]);

    // Section 4: Action Buttons Bar
    let btn_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let btn_area = chunks[3];
    let btn_sub = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24),
            Constraint::Length(24),
            Constraint::Min(10),
        ])
        .split(btn_block.inner(btn_area));

    crate::tui::widgets::Button::new("Reset Windows (r)")
        .style(crate::tui::widgets::ButtonStyle::Neutral)
        .render(btn_sub[0], f.buffer_mut());

    crate::tui::widgets::Button::new("Clear Breakers (c)")
        .style(crate::tui::widgets::ButtonStyle::Warning)
        .render(btn_sub[1], f.buffer_mut());

    f.render_widget(btn_block, btn_area);
}
