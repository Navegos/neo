// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/views/agents.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::App;
use crate::tui::widgets::render_modal;

pub fn render_agents_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instructions & summary
            Constraint::Min(10),   // Agent Table
        ])
        .split(area);

    let wired_count = app.agents.items.iter().filter(|a| a.wired).count();
    let installed_count = app.agents.items.iter().filter(|a| a.installed).count();

    let info_text = Line::from(vec![
        Span::styled("Fleet Status: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{} / 17 Wired ", wired_count), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled(format!("{} Detected ", installed_count), Style::default().fg(Color::Cyan)),
        Span::raw("│ "),
        Span::styled("Press [Space] to Wire/Unwire  │  [Enter] to Inspect MCP Config", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(info_text).block(Block::default().borders(Borders::BOTTOM)), chunks[0]);

    let header_row = Row::new(vec![
        Cell::from("Wiring Status").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("ID").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Agent Name").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("System Detection").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Active Port").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Integrated Tool Stack").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .agents
        .items
        .iter()
        .enumerate()
        .map(|(idx, agent)| {
            let is_selected = idx == app.agents.selected;

            let (wire_badge, wire_style) = if agent.wired {
                (" [● WIRED] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                (" [○ unwired] ", Style::default().fg(Color::DarkGray))
            };

            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 45, 60))
            } else {
                Style::default()
            };

            let detect_badge = if agent.installed {
                Span::styled(format!("✓ {}", agent.detect_source), Style::default().fg(Color::Cyan))
            } else {
                Span::styled("not found", Style::default().fg(Color::DarkGray))
            };

            Row::new(vec![
                Cell::from(Span::styled(wire_badge, wire_style)),
                Cell::from(Span::styled(&agent.id, Style::default().fg(Color::Yellow))),
                Cell::from(Span::styled(
                    &agent.label,
                    if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                )),
                Cell::from(detect_badge),
                Cell::from(Span::styled(&agent.port, Style::default().fg(Color::Gray))),
                Cell::from(Span::styled(&agent.tools_summary, Style::default().fg(Color::DarkGray))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(22),
        Constraint::Length(22),
        Constraint::Length(14),
        Constraint::Min(30),
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .title(" [Tab 2] 17 AI Coding Agents Fleet Matrix ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(table, chunks[1]);

    // Render MCP Config Modal if open
    if let Some(ref mcp_info) = app.agents.inspect_mcp {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Agent: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&mcp_info.agent_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" ("),
            Span::styled(&mcp_info.agent_id, Style::default().fg(Color::Cyan)),
            Span::raw(")"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Config Path: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&mcp_info.config_path, Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Config Status: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            if mcp_info.exists {
                Span::styled("Found on disk", Style::default().fg(Color::Green))
            } else {
                Span::styled("File not created yet", Style::default().fg(Color::Red))
            },
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "Registered MCP Servers & Configuration Content:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::raw("--------------------------------------------------------------------------------"));

        for raw_line in mcp_info.content.lines() {
            lines.push(Line::from(Span::styled(raw_line, Style::default().fg(Color::White))));
        }

        render_modal(
            f,
            &format!("MCP Configuration: {}", mcp_info.agent_label),
            lines,
            mcp_info.scroll,
            80,
            75,
        );
    }
}
