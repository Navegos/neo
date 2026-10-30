// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/views/graph.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render_graph_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar & stats
            Constraint::Min(10),   // Split between Symbols and Blast-Radius
        ])
        .split(area);

    // Search header
    let mut search_spans = vec![
        Span::styled("CodeGraph AST Explorer [/]: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ];

    if app.graph.search_active {
        search_spans.push(Span::styled(
            format!("{}█", app.graph.search_query),
            Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 40)),
        ));
    } else if app.graph.search_query.is_empty() {
        search_spans.push(Span::styled("(Type to search symbols... e.g. 'run', 'proxy', 'parse')", Style::default().fg(Color::DarkGray)));
    } else {
        search_spans.push(Span::styled(&app.graph.search_query, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
    }

    search_spans.push(Span::raw(" │ "));
    search_spans.push(Span::styled(
        format!("Indexed: {} symbols ({} files)", app.graph.total_symbols, app.graph.total_files),
        Style::default().fg(Color::Gray),
    ));
    search_spans.push(Span::raw(" │ "));
    search_spans.push(Span::styled("Press 'b' for Blast-Radius Tree  │  'r' to Reindex", Style::default().fg(Color::DarkGray)));

    f.render_widget(Paragraph::new(Line::from(search_spans)).block(Block::default().borders(Borders::BOTTOM)), chunks[0]);

    // Split: Left = Symbol Table (50%), Right = Call Paths & Blast Radius (50%)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left: Symbol Table
    let header_row = Row::new(vec![
        Cell::from("Symbol").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Kind").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("File Location").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Line").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .graph
        .filtered_symbols()
        .iter()
        .enumerate()
        .map(|(idx, sym)| {
            let is_selected = idx == app.graph.selected;

            let kind_color = match sym.kind.as_str() {
                "function" | "fn" => Color::Cyan,
                "struct" => Color::Green,
                "enum" => Color::Yellow,
                "trait" => Color::Magenta,
                "method" => Color::LightBlue,
                "class" => Color::Green,
                _ => Color::White,
            };

            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 45, 60))
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(
                    &sym.name,
                    if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                )),
                Cell::from(Span::styled(&sym.kind, Style::default().fg(kind_color))),
                Cell::from(Span::styled(&sym.file, Style::default().fg(Color::DarkGray))),
                Cell::from(Span::styled(sym.line.to_string(), Style::default().fg(Color::Gray))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(35),
        Constraint::Length(12),
        Constraint::Percentage(40),
        Constraint::Length(8),
    ];

    let sym_table = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .title(" [Tab 4] AST Symbol Index ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(sym_table, cols[0]);

    // Right: Callers, Callees & Blast Radius Tree
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(cols[1]);

    // Callers / Callees
    let call_block = Block::default()
        .title(" Call Path & Direct Edges ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let mut call_lines = Vec::new();
    if let Some(selected_sym) = app.graph.selected_symbol() {
        call_lines.push(Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&selected_sym.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" in "),
            Span::styled(format!("{}:{}", selected_sym.file, selected_sym.line), Style::default().fg(Color::Green)),
        ]));
        call_lines.push(Line::raw(""));

        call_lines.push(Line::from(Span::styled("Inbound Callers:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        if app.graph.active_callers.is_empty() {
            call_lines.push(Line::from(Span::styled("  (Root caller / entry point / no internal callers)", Style::default().fg(Color::DarkGray))));
        } else {
            for caller in &app.graph.active_callers {
                call_lines.push(Line::from(vec![
                    Span::styled("  ▲ ", Style::default().fg(Color::Cyan)),
                    Span::styled(&caller.name, Style::default().fg(Color::White)),
                    Span::styled(format!(" ({}:{})", caller.file, caller.line), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        call_lines.push(Line::raw(""));
        call_lines.push(Line::from(Span::styled("Outbound Dependencies (Callees):", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
        if app.graph.active_callees.is_empty() {
            call_lines.push(Line::from(Span::styled("  (Leaf node / no further dependencies)", Style::default().fg(Color::DarkGray))));
        } else {
            for callee in &app.graph.active_callees {
                call_lines.push(Line::from(vec![
                    Span::styled("  ▼ ", Style::default().fg(Color::Magenta)),
                    Span::styled(&callee.name, Style::default().fg(Color::White)),
                    Span::styled(format!(" ({}:{})", callee.file, callee.line), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
    } else {
        call_lines.push(Line::from(Span::styled("Select a symbol to inspect call path.", Style::default().fg(Color::DarkGray))));
    }
    f.render_widget(Paragraph::new(call_lines).block(call_block).wrap(Wrap { trim: false }), right_chunks[0]);

    // Lower: Blast-Radius Impact Simulator
    let blast_block = Block::default()
        .title(" Petgraph Blast-Radius Impact Tree ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let mut blast_lines = Vec::new();
    if let Some(ref report) = app.graph.active_blast_radius {
        blast_lines.push(Line::from(vec![
            Span::styled("Blast Radius: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} dependent symbols across {} files", report.total_impacted, report.impacted_files.len()), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        blast_lines.push(Line::from(vec![
            Span::styled("Max Impact Depth: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{} layers", report.max_depth), Style::default().fg(Color::Cyan)),
            Span::raw(" │ Affected Tests: "),
            Span::styled(format!("{} test suites", report.affected_tests.len()), Style::default().fg(Color::Green)),
        ]));
        blast_lines.push(Line::raw(""));
        blast_lines.push(Line::from(Span::styled("Impact Tree:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));

        for (idx, item) in report.tree_items.iter().enumerate() {
            let prefix = if idx == report.tree_items.len() - 1 { "└── " } else { "├── " };
            let indent = "    ".repeat(item.depth.saturating_sub(1));
            blast_lines.push(Line::from(vec![
                Span::styled(format!("{}{}", indent, prefix), Style::default().fg(Color::DarkGray)),
                Span::styled(&item.name, Style::default().fg(Color::White)),
                Span::styled(format!(" [{}] (depth {})", item.file, item.depth), Style::default().fg(Color::DarkGray)),
            ]));
        }
    } else {
        blast_lines.push(Line::from(Span::styled(
            "Press Enter on a symbol to simulate full blast radius impact.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    f.render_widget(Paragraph::new(blast_lines).block(blast_block).wrap(Wrap { trim: false }), right_chunks[1]);
}
