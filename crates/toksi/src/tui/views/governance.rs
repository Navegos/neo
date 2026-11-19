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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render_governance_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Section 1: Keyring & DB Summary Cards
            Constraint::Min(10),   // Section 2: Configured Keyring Entries Table
            Constraint::Length(5), // Section 3: Wave 18 DB Tables & ADC Details
            Constraint::Length(3), // Section 4: Action Buttons Bar
        ])
        .split(area);

    // Section 1: Overview Cards
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Card 1: OS Keyring & Google Cloud ADC
    let keyring_block = Block::default()
        .title(" [1] Normalized Keyring & Google Cloud ADC ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let keyring_text = vec![
        Line::from(vec![
            Span::styled("Configured Credentials: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} active", app.governance.keyring_entries.len()),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (gemini:antigravity normalized)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Google Cloud ADC (AGY_ADC_AUTH): ", Style::default().fg(Color::Gray)),
            if app.governance.gcp_adc_active {
                Span::styled("ENABLED ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("STANDBY ", Style::default().fg(Color::DarkGray))
            },
            Span::styled(
                format!("({} / {})", if app.governance.gcp_project.is_empty() { "unset" } else { &app.governance.gcp_project }, app.governance.gcp_location),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tier Policy: ", Style::default().fg(Color::Gray)),
            Span::styled("OPEN (Zero paid-tier restrictions)", Style::default().fg(Color::Green)),
        ]),
    ];
    f.render_widget(Paragraph::new(keyring_text).block(keyring_block).wrap(Wrap { trim: false }), top_chunks[0]);

    // Card 2: SQLite Database Stats
    let db_block = Block::default()
        .title(" [2] Persistent SQLite Store (~/.toksi/toksi.db) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let st = &app.governance.db_stats;
    let db_text = vec![
        Line::from(vec![
            Span::styled("DB Path: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.governance.db_path, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Profiles/Quotas: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} byok_profiles │ {} quota_windows │ {} providers", st.byok_profiles, st.quota_windows, st.providers),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Wave 18 Tables: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} plugins │ {} blind_sig │ {} embargoes │ {} adr", st.plugins, st.blind_signatures, st.advisory_embargoes, st.adr_events),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(db_text).block(db_block).wrap(Wrap { trim: false }), top_chunks[1]);

    // Section 2: Configured Keyring Entries Table
    let table_block = Block::default()
        .title(format!(" [3] Configured Keyring Entries ({}) ", app.governance.keyring_entries.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let header_row = Row::new(vec![
        Cell::from("Agent").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Normalized Service").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Account").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Keyring Status").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Source").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .governance
        .keyring_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = idx == app.governance.selected;
            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 45, 60))
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(&entry.agent, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(&entry.service, Style::default().fg(Color::White))),
                Cell::from(Span::styled(&entry.account, Style::default().fg(Color::Yellow))),
                Cell::from(if entry.present {
                    Span::styled("● PRESENT", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("○ CONFIGURED", Style::default().fg(Color::Cyan))
                }),
                Cell::from(Span::styled(&entry.source, Style::default().fg(Color::Gray))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(18),
        Constraint::Length(26),
        Constraint::Length(22),
        Constraint::Length(16),
        Constraint::Min(24),
    ];

    let table = Table::new(rows, widths).header(header_row).block(table_block);
    f.render_widget(table, chunks[1]);

    // Section 3: Selected Entry Detail
    let detail_block = Block::default()
        .title(" [4] Keyring & Database Backup Diagnostics ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let detail_text = if let Some(entry) = app.governance.keyring_entries.get(app.governance.selected) {
        vec![Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::Gray)),
            Span::styled(&entry.service, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" │ "),
            Span::styled("Account: ", Style::default().fg(Color::Gray)),
            Span::styled(&entry.account, Style::default().fg(Color::Yellow)),
            Span::raw(" │ "),
            Span::styled("Backup File: ", Style::default().fg(Color::Gray)),
            Span::styled("~/.toksi/toksi-backup.json", Style::default().fg(Color::Green)),
        ])]
    } else {
        vec![Line::from(Span::styled(
            "No configured keyring entries detected. Use Export/Import DB to manage ~/.toksi/toksi.db.",
            Style::default().fg(Color::DarkGray),
        ))]
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
            Constraint::Length(22),
            Constraint::Length(22),
            Constraint::Length(18),
            Constraint::Min(10),
        ])
        .split(btn_block.inner(btn_area));

    crate::tui::widgets::Button::new("Export DB (e)")
        .style(crate::tui::widgets::ButtonStyle::Primary)
        .render(btn_sub[0], f.buffer_mut());

    crate::tui::widgets::Button::new("Import DB (i)")
        .style(crate::tui::widgets::ButtonStyle::Success)
        .render(btn_sub[1], f.buffer_mut());

    crate::tui::widgets::Button::new("Refresh (r)")
        .style(crate::tui::widgets::ButtonStyle::Neutral)
        .render(btn_sub[2], f.buffer_mut());

    f.render_widget(btn_block, btn_area);
}
