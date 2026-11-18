// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/views/warroom.rs
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

pub fn render_warroom_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // TokenSpeed & GPU Telemetry KPIs
            Constraint::Length(7), // Cascade Failover Stages
            Constraint::Min(6),    // Rate Limits & Context Watchdog
            Constraint::Length(3), // Action Buttons
        ])
        .split(area);

    // 1. TokenSpeed & GPU Telemetry
    let wr = &app.warroom;
    let kpi_lines = vec![
        Line::from(vec![
            Span::styled(" TokenSpeed TTFT: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2} ms", wr.ttft_ms), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("   │   "),
            Span::styled("TPOT: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2} ms", wr.tpot_ms), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("   │   "),
            Span::styled("OpenInference Spans: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", wr.openinference_spans), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" GPU (vmon/cuda-rust): ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.1}%", wr.gpu_utilization_pct), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("   │   "),
            Span::styled("VRAM: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} / {} MB", wr.gpu_memory_used_bytes / (1024 * 1024), wr.gpu_memory_total_bytes / (1024 * 1024)),
                Style::default().fg(Color::White),
            ),
            Span::raw("   │   "),
            Span::styled("Active In-Flight: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", wr.active_requests), Style::default().fg(Color::Cyan)),
        ]),
    ];
    let kpi_block = Block::default()
        .title(" The War Room — TokenSpeed, GPU & OpenInference Telemetry ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Paragraph::new(kpi_lines).block(kpi_block), chunks[0]);

    // 2. Cascade Failover Stages
    let byok_port = crate::config::resolved_ports().byok_gateway;
    let gw_port = crate::config::resolved_ports().ai_gateway;
    let cascade_rows = vec![
        Row::new(vec![
            Cell::from("L0"),
            Cell::from(format!("Enterprise AI Gateway (:{})", gw_port)),
            Cell::from(if app.monitor.daemon_up { "ACTIVE" } else { "STANDBY" }),
            Cell::from("HeadroomEffortCompressor"),
        ]),
        Row::new(vec![
            Cell::from("L1"),
            Cell::from(format!("BYOK & Vertex ADC Gateway (:{})", byok_port)),
            Cell::from(if !app.headroom.routes.is_empty() { "READY" } else { "UNCONFIGURED" }),
            Cell::from("Multi-Instance <byok, model, effort>"),
        ]),
        Row::new(vec![
            Cell::from("L2"),
            Cell::from("Local Degraded Mode (ICM / AiMemory SQLite)"),
            Cell::from("STANDBY"),
            Cell::from("Zero-LLM FTS5 Recall"),
        ]),
    ];
    let cascade_table = Table::new(
        cascade_rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(45),
            Constraint::Length(16),
            Constraint::Percentage(35),
        ],
    )
    .header(
        Row::new(vec!["Tier", "Endpoint", "State", "Pipeline"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .title(" Failover Cascade & Circuit Breaker ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(cascade_table, chunks[1]);

    // 3. Context Watchdog & Rate-Limit Status
    let watchdog_lines = vec![
        Line::from(vec![
            Span::styled(" Context Watchdog: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} / 200000 tokens", wr.input_tokens), Style::default().fg(Color::White)),
            Span::raw("   │   "),
            Span::styled("Pruning Tier: ", Style::default().fg(Color::DarkGray)),
            Span::styled(wr.pruning_tier.clone(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled(" Cache Hits (Direct / Semantic / Miss): ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} / {} / {}", wr.cache_hits_direct, wr.cache_hits_semantic, wr.cache_misses),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];
    let watchdog_block = Block::default()
        .title(" Context Watchdog & Cache Hit Ratio ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(watchdog_lines).block(watchdog_block), chunks[2]);

    // 4. Action Bar
    let btn_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    let btn_sub = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Length(20),
            Constraint::Min(10),
        ])
        .split(btn_block.inner(chunks[3]));
    Button::new("Refresh Snapshot (r)")
        .style(ButtonStyle::Primary)
        .render(btn_sub[0], f.buffer_mut());
    Button::new("Reset Circuit")
        .style(ButtonStyle::Warning)
        .render(btn_sub[1], f.buffer_mut());
    f.render_widget(btn_block, chunks[3]);
}
