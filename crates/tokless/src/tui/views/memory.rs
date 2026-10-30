// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/views/memory.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::tui::app::{App, MemoryFilter};
use crate::tui::widgets::render_modal;

pub fn render_memory_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar & filter pills
            Constraint::Min(10),   // Split between Table and Detail
        ])
        .split(area);

    // Search and filter header
    let filter_pills = [
        (MemoryFilter::All, " [1: All] "),
        (MemoryFilter::Issues, " [2: Issues] "),
        (MemoryFilter::Decisions, " [3: Decisions] "),
        (MemoryFilter::Gotchas, " [4: Gotchas/Notes] "),
        (MemoryFilter::Icm, " [5: ICM DB] "),
    ];

    let mut search_spans = vec![
        Span::styled("Filter: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ];

    for (f_type, label) in filter_pills {
        if app.memory.filter == f_type {
            search_spans.push(Span::styled(label, Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)));
        } else {
            search_spans.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
        }
        search_spans.push(Span::raw(" "));
    }

    search_spans.push(Span::raw("│  "));
    search_spans.push(Span::styled("Search [/]: ", Style::default().fg(Color::Yellow)));
    if app.memory.search_active {
        search_spans.push(Span::styled(
            format!("{}█", app.memory.search_query),
            Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 40)),
        ));
    } else if app.memory.search_query.is_empty() {
        search_spans.push(Span::styled("(Press / to search)", Style::default().fg(Color::DarkGray)));
    } else {
        search_spans.push(Span::styled(&app.memory.search_query, Style::default().fg(Color::Cyan)));
    }

    search_spans.push(Span::raw("  │  "));
    search_spans.push(Span::styled("Press 'p' to run Git Precheck", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));

    let search_block = Block::default().borders(Borders::BOTTOM);
    f.render_widget(Paragraph::new(Line::from(search_spans)).block(search_block), chunks[0]);

    // Body: Split horizontal: Event Table (65%) and Selected Detail (35%)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[1]);

    let filtered_events = app.memory.filtered_events();

    let header_row = Row::new(vec![
        Cell::from("ID").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Summary").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Location").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Source/Time").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = filtered_events
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == app.memory.selected;

            let type_color = match item.event_type.as_str() {
                "issue" => Color::Red,
                "fix" => Color::Green,
                "decision" => Color::Cyan,
                "note" | "gotcha" => Color::Yellow,
                "attempt" => Color::Magenta,
                "icm" => Color::LightBlue,
                _ => Color::White,
            };

            let row_style = if is_selected {
                Style::default().bg(Color::Rgb(30, 45, 60))
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(Span::styled(&item.id, Style::default().fg(Color::Yellow))),
                Cell::from(Span::styled(format!("[{}]", item.event_type), Style::default().fg(type_color).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(
                    &item.summary,
                    if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                )),
                Cell::from(Span::styled(&item.location, Style::default().fg(Color::Gray))),
                Cell::from(Span::styled(&item.timestamp, Style::default().fg(Color::DarkGray))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(12),
        Constraint::Min(25),
        Constraint::Length(22),
        Constraint::Length(14),
    ];

    let events_table = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .title(format!(
                    " [Tab 3] ProjectMem & ICM Memory Items ({}) ",
                    filtered_events.len()
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(events_table, body_chunks[0]);

    // Detail Pane
    let detail_block = Block::default()
        .title(" Selected Memory Detail ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(selected_item) = filtered_events.get(app.memory.selected) {
        let mut detail_lines = Vec::new();
        detail_lines.push(Line::from(vec![
            Span::styled("Event ID: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&selected_item.id, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" │ Type: "),
            Span::styled(&selected_item.event_type, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled("Timestamp: ", Style::default().fg(Color::Yellow)),
            Span::styled(&selected_item.timestamp, Style::default().fg(Color::Gray)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled("Coordinate: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if selected_item.location.is_empty() { "none".into() } else { selected_item.location.clone() },
                Style::default().fg(Color::Green),
            ),
        ]));
        detail_lines.push(Line::raw(""));
        detail_lines.push(Line::from(Span::styled(
            "Summary & Context:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        detail_lines.push(Line::from(Span::styled(&selected_item.summary, Style::default().fg(Color::White))));

        if !selected_item.details.is_empty() {
            detail_lines.push(Line::raw(""));
            detail_lines.push(Line::from(Span::styled("Full Record / Excerpt:", Style::default().fg(Color::DarkGray))));
            for line in selected_item.details.lines() {
                detail_lines.push(Line::from(Span::styled(line, Style::default().fg(Color::Gray))));
            }
        }

        let detail_p = Paragraph::new(detail_lines)
            .block(detail_block)
            .wrap(Wrap { trim: false });
        f.render_widget(detail_p, body_chunks[1]);
    } else {
        let empty_p = Paragraph::new(Line::from(Span::styled(
            "No memory events match current filter/search.",
            Style::default().fg(Color::DarkGray),
        )))
        .block(detail_block);
        f.render_widget(empty_p, body_chunks[1]);
    }

    // Precheck Result Modal if open
    if let Some(ref precheck) = app.memory.precheck_result {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Git Staged & Working Tree Diff Scanner", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Scanned Files: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{} files checked", precheck.files_scanned.len()), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Precheck Status: ", Style::default().fg(Color::Yellow)),
            if precheck.warnings.is_empty() {
                Span::styled("PASS — Zero memory regressions detected! Ready to commit.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(format!("WARNING — {} memory gotcha matches found!", precheck.warnings.len()), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            },
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled("Active Warnings & Past Failure Locations:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        lines.push(Line::raw("--------------------------------------------------------------------------------"));

        if precheck.warnings.is_empty() {
            lines.push(Line::from(Span::styled(
                "✓ No modified files intersect with recorded gotchas or past unresolved failures.",
                Style::default().fg(Color::Green),
            )));
        } else {
            for w in &precheck.warnings {
                lines.push(Line::from(vec![
                    Span::styled("• [WARNING] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(&w.file, Style::default().fg(Color::Yellow)),
                    Span::raw(": "),
                    Span::styled(&w.message, Style::default().fg(Color::White)),
                ]));
            }
        }

        render_modal(f, "ProjectMem Precheck Analysis", lines, precheck.scroll, 85, 75);
    }
}
