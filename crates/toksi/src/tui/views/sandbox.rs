// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/views/sandbox.rs
// created: 2026-11-18

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
    Frame,
};

use crate::tui::app::App;
use crate::tui::widgets::{Button, ButtonStyle};

pub fn render_sandbox_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // DefenseClaw & CommandGuard Status
            Constraint::Min(8),    // 5-Backend Sandbox & ADR Audit Events
            Constraint::Length(3), // Action Buttons
        ])
        .split(area);

    let sb = &app.sandbox;
    let status_lines = vec![
        Line::from(vec![
            Span::styled(" DefenseClaw HookGuard: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if sb.fail_closed { "FAIL-CLOSED" } else { "PERMISSIVE" },
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   │   "),
            Span::styled("CommandGuard Kill-Switch: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if sb.kill_switch_armed { "ARMED" } else { "DISABLED" },
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   │   "),
            Span::styled("Active Backend: ", Style::default().fg(Color::DarkGray)),
            Span::styled(sb.active_backend.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(" Backends: ", Style::default().fg(Color::DarkGray)),
            Span::raw("3-OS Native | sandbox-runtime | OpenShell | E2B | Docker sbx (29-verb)"),
        ]),
    ];
    let status_block = Block::default()
        .title(" Sandbox Inspector — DefenseClaw, ADR & 5-Backend Isolation ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Paragraph::new(status_lines).block(status_block), chunks[0]);

    let rows: Vec<Row> = if sb.audit_events.is_empty() {
        vec![Row::new(vec![
            Cell::from("Idle"),
            Cell::from("defenseclaw"),
            Cell::from("No ADR/sandbox violation events recorded"),
            Cell::from("PASS"),
        ])]
    } else {
        sb.audit_events
            .iter()
            .map(|ev| {
                Row::new(vec![
                    Cell::from(ev.timestamp.clone()),
                    Cell::from(ev.source.clone()),
                    Cell::from(ev.summary.clone()),
                    Cell::from(ev.verdict.clone()),
                ])
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(16),
            Constraint::Percentage(55),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Timestamp", "Component", "Event / Audit Evidence", "Verdict"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .title(" ADR & HookGuard Audit Evidence ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(table, chunks[1]);

    let btn_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let btn_sub = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Length(26),
            Constraint::Min(10),
        ])
        .split(btn_block.inner(chunks[2]));
    Button::new("Run ContextMode Test")
        .style(ButtonStyle::Primary)
        .render(btn_sub[0], f.buffer_mut());
    Button::new("Toggle Fail-Closed (f)")
        .style(ButtonStyle::Warning)
        .render(btn_sub[1], f.buffer_mut());
    f.render_widget(btn_block, chunks[2]);
}
