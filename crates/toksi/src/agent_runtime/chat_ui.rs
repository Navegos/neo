// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/chat_ui.rs
// created: 2026-11-25

//! Codex CLI (`codex-rs/tui`) + Grok-Build CLI grade streaming `ratatui` Chat TUI
//! with collapsible tool-call cards, inline diff/patch preview, HITL approval modal,
//! live `<byok, model, effort, mode>` status bar across all 22 Orchestrator Modes,
//! and full in-chat `toksi` CLI + 29-slash-command execution.

#![deny(unsafe_code)]

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::commands::agent_runtime::{detect_daemon_connection, DaemonConnectionMode};
use crate::commands::guard::{CommandGuard, GuardDecision};
use crate::headroom::effort::EffortLevel;

use super::loop_engine::AgentRuntimeEngine;
use super::mode_router::{resolve_mode_spec, OrchestratorMode, ALL_ORCHESTRATOR_MODES};
use super::slash::global_slash_engine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatMessageRole {
    User,
    Assistant,
    Tool,
    System,
    Diff,
}

impl ChatMessageRole {
    pub const fn label(self) -> &'static str {
        match self {
            Self::User => "YOU",
            Self::Assistant => "SIAGENT",
            Self::Tool => "TOOL",
            Self::System => "SYSTEM",
            Self::Diff => "DIFF",
        }
    }

    pub const fn color(self) -> Color {
        match self {
            Self::User => Color::Cyan,
            Self::Assistant => Color::Green,
            Self::Tool => Color::Yellow,
            Self::System => Color::Gray,
            Self::Diff => Color::Magenta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatEntry {
    pub role: ChatMessageRole,
    pub title: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallCard {
    pub id: String,
    pub tool_name: String,
    pub summary: String,
    pub output: String,
    pub status: String,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffPreviewPane {
    pub file_path: String,
    pub hunks: Vec<String>,
    pub approved: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitlApprovalModal {
    pub request_id: String,
    pub action: String,
    pub command_preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatUiAction {
    SubmitInput(String),
    Quit,
    CycleMode,
    ToggleDiffPane,
    ResolveHitl { request_id: String, approved: bool },
}

pub struct ChatUiState {
    pub session_id: String,
    pub workspace_root: PathBuf,
    pub connection_mode: DaemonConnectionMode,
    pub active_mode: OrchestratorMode,
    pub active_effort: EffortLevel,
    pub active_byok: String,
    pub active_model: String,
    pub messages: Vec<ChatEntry>,
    pub tool_cards: Vec<ToolCallCard>,
    pub diff_preview: Option<DiffPreviewPane>,
    pub approval_modal: Option<HitlApprovalModal>,
    pub input_buffer: String,
    pub scroll_offset: u16,
    pub streaming_active: bool,
    pub show_diff_pane: bool,
    pub should_quit: bool,
    pub status_notice: String,
}

impl ChatUiState {
    pub fn new(session_id: impl Into<String>, workspace_root: PathBuf) -> Self {
        let mode = OrchestratorMode::WriteCodeBuild;
        let spec = resolve_mode_spec(mode);
        let conn = detect_daemon_connection();
        Self {
            session_id: session_id.into(),
            workspace_root,
            connection_mode: conn,
            active_mode: mode,
            active_effort: spec.reasoning_effort,
            active_byok: "auto".to_string(),
            active_model: spec.primary_model,
            messages: vec![ChatEntry {
                role: ChatMessageRole::System,
                title: "toksi-siagent".to_string(),
                content: "Ready. Enter prompts, 29 slash commands (/mode, /byok, /effort, /resume), or `toksi <cmd>` CLI calls.".to_string(),
                timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            }],
            tool_cards: Vec::new(),
            diff_preview: None,
            approval_modal: None,
            input_buffer: String::new(),
            scroll_offset: 0,
            streaming_active: false,
            show_diff_pane: false,
            should_quit: false,
            status_notice: "Idle".to_string(),
        }
    }

    pub fn resolve_mode_alias(raw: &str) -> Option<OrchestratorMode> {
        if let Some(m) = OrchestratorMode::from_str_loose(raw) {
            return Some(m);
        }
        let key = raw.trim().to_ascii_lowercase().replace('-', "_");
        match key.as_str() {
            "coding" => Some(OrchestratorMode::WriteCodeBuild),
            "conversation" => Some(OrchestratorMode::ChatInterface),
            "orchestration" | "swarm" => Some(OrchestratorMode::OrchestrateRoute),
            "planning" => Some(OrchestratorMode::PlanDecompose),
            "analysis" => Some(OrchestratorMode::SynthesizeAbstract),
            "advisory" | "security" => Some(OrchestratorMode::AuditRedTeam),
            "run" | "exec" => Some(OrchestratorMode::ActOperate),
            _ => None,
        }
    }

    pub fn set_mode(&mut self, raw: &str) -> Result<OrchestratorMode, String> {
        let mode = Self::resolve_mode_alias(raw)
            .ok_or_else(|| format!("Unknown orchestrator mode '{raw}'"))?;
        let spec = resolve_mode_spec(mode);
        self.active_mode = mode;
        self.active_effort = spec.reasoning_effort;
        self.active_model = spec.primary_model;
        self.status_notice = format!("Mode -> {}", mode.as_str());
        Ok(mode)
    }

    pub fn cycle_mode(&mut self) -> OrchestratorMode {
        let idx = ALL_ORCHESTRATOR_MODES
            .iter()
            .position(|m| *m == self.active_mode)
            .unwrap_or(0);
        let next = ALL_ORCHESTRATOR_MODES[(idx + 1) % ALL_ORCHESTRATOR_MODES.len()];
        let _ = self.set_mode(next.as_str());
        next
    }

    pub fn push_message(
        &mut self,
        role: ChatMessageRole,
        title: impl Into<String>,
        content: impl Into<String>,
    ) {
        self.messages.push(ChatEntry {
            role,
            title: title.into(),
            content: content.into(),
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        });
    }

    pub fn append_stream_chunk(&mut self, chunk: &str) {
        if chunk.is_empty() {
            return;
        }
        self.streaming_active = true;
        if let Some(last) = self.messages.last_mut()
            && last.role == ChatMessageRole::Assistant
        {
            last.content.push_str(chunk);
            return;
        }
        self.push_message(ChatMessageRole::Assistant, "siagent", chunk);
    }

    pub fn add_tool_card(
        &mut self,
        id: impl Into<String>,
        tool_name: impl Into<String>,
        summary: impl Into<String>,
        output: impl Into<String>,
        status: impl Into<String>,
    ) {
        self.tool_cards.push(ToolCallCard {
            id: id.into(),
            tool_name: tool_name.into(),
            summary: summary.into(),
            output: output.into(),
            status: status.into(),
            collapsed: true,
        });
    }

    pub fn toggle_tool_card(&mut self, idx: usize) {
        if let Some(card) = self.tool_cards.get_mut(idx) {
            card.collapsed = !card.collapsed;
        }
    }

    pub fn open_diff_preview(&mut self, file_path: impl Into<String>, hunks: Vec<String>) {
        self.diff_preview = Some(DiffPreviewPane {
            file_path: file_path.into(),
            hunks,
            approved: None,
        });
        self.show_diff_pane = true;
    }

    pub fn open_hitl_modal(
        &mut self,
        request_id: impl Into<String>,
        action: impl Into<String>,
        command_preview: impl Into<String>,
    ) {
        self.approval_modal = Some(HitlApprovalModal {
            request_id: request_id.into(),
            action: action.into(),
            command_preview: command_preview.into(),
        });
    }

    pub fn resolve_hitl_modal(&mut self, approved: bool) -> Option<String> {
        let modal = self.approval_modal.take()?;
        if let Some(diff) = self.diff_preview.as_mut() {
            diff.approved = Some(approved);
        }
        self.status_notice = if approved {
            format!("HITL approved: {}", modal.action)
        } else {
            format!("HITL denied: {}", modal.action)
        };
        Some(modal.request_id)
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
    }

    pub fn scroll_down(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<ChatUiAction> {
        if let Some(modal) = self.approval_modal.clone() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.resolve_hitl_modal(true);
                    return Some(ChatUiAction::ResolveHitl {
                        request_id: modal.request_id,
                        approved: true,
                    });
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.resolve_hitl_modal(false);
                    return Some(ChatUiAction::ResolveHitl {
                        request_id: modal.request_id,
                        approved: false,
                    });
                }
                _ => return None,
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('d')) {
            self.should_quit = true;
            return Some(ChatUiAction::Quit);
        }

        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
                Some(ChatUiAction::Quit)
            }
            KeyCode::F(2) => {
                self.cycle_mode();
                Some(ChatUiAction::CycleMode)
            }
            KeyCode::F(3) => {
                self.show_diff_pane = !self.show_diff_pane;
                Some(ChatUiAction::ToggleDiffPane)
            }
            KeyCode::Tab => {
                if !self.tool_cards.is_empty() {
                    self.toggle_tool_card(self.tool_cards.len() - 1);
                }
                None
            }
            KeyCode::Up => {
                self.scroll_up(1);
                None
            }
            KeyCode::Down => {
                self.scroll_down(1);
                None
            }
            KeyCode::PageUp => {
                self.scroll_up(5);
                None
            }
            KeyCode::PageDown => {
                self.scroll_down(5);
                None
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                None
            }
            KeyCode::Enter => {
                let text = self.input_buffer.trim().to_string();
                self.input_buffer.clear();
                if text.is_empty() {
                    return None;
                }
                if text == "/exit" || text == "/quit" {
                    self.should_quit = true;
                    return Some(ChatUiAction::Quit);
                }
                Some(ChatUiAction::SubmitInput(text))
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                None
            }
            _ => None,
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_up(3),
            MouseEventKind::ScrollDown => self.scroll_down(3),
            MouseEventKind::Down(MouseButton::Left) if !self.tool_cards.is_empty() => {
                self.toggle_tool_card(self.tool_cards.len() - 1);
            }
            _ => {}
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        let status_line = Line::from(vec![
            Span::styled(
                " toksi-siagent ",
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                " mode={} | effort={} | byok={}/{} | conn={} | {}",
                self.active_mode.as_str(),
                self.active_effort.as_str(),
                self.active_byok,
                self.active_model,
                self.connection_mode.as_str(),
                self.status_notice
            )),
        ]);
        let header = Paragraph::new(status_line)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(header, chunks[0]);

        if self.show_diff_pane && self.diff_preview.is_some() {
            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
                .split(chunks[1]);
            self.render_transcript(frame, body[0]);
            self.render_diff_pane(frame, body[1]);
        } else {
            self.render_transcript(frame, chunks[1]);
        }

        let input_title = if self.streaming_active {
            "Composer (streaming...)"
        } else {
            "Composer (prompt | /slash | toksi <cmd>)"
        };
        let input = Paragraph::new(format!("toksi> {}", self.input_buffer))
            .block(Block::default().borders(Borders::ALL).title(input_title));
        frame.render_widget(input, chunks[2]);

        let hints = Paragraph::new(
            " F2:CycleMode(22) | F3:DiffPane | Tab:ToggleToolCard | Up/Down/Mouse:Scroll | Ctrl+C:Quit ",
        )
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hints, chunks[3]);

        if let Some(modal) = &self.approval_modal {
            let popup = centered_rect(70, 35, area);
            frame.render_widget(Clear, popup);
            let modal_text = format!(
                "Action: {}\nPreview: {}\n\nPress [Y/Enter] to approve or [N/Esc] to deny.",
                modal.action, modal.command_preview
            );
            let box_widget = Paragraph::new(modal_text)
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .title(format!("HITL Approval ({})", modal.request_id)),
                );
            frame.render_widget(box_widget, popup);
        }
    }

    fn render_transcript(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line<'static>> = Vec::new();
        for entry in &self.messages {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{} {}] ", entry.timestamp, entry.role.label()),
                    Style::default().fg(entry.role.color()).add_modifier(Modifier::BOLD),
                ),
                Span::raw(entry.title.clone()),
            ]));
            for l in entry.content.lines() {
                lines.push(Line::from(format!("  {l}")));
            }
        }
        for card in &self.tool_cards {
            let arrow = if card.collapsed { "+" } else { "-" };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{arrow} TOOL {} ({})] ", card.tool_name, card.status),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(card.summary.clone()),
            ]));
            if !card.collapsed {
                for l in card.output.lines() {
                    lines.push(Line::from(format!("    {l}")));
                }
            }
        }

        let total_lines = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        let visible_h = area.height.saturating_sub(2);
        let max_scroll = total_lines.saturating_sub(visible_h);
        let top = max_scroll.saturating_sub(self.scroll_offset.min(max_scroll));

        let transcript = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((top, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Transcript (session={})", self.session_id)),
            );
        frame.render_widget(transcript, area);
    }

    fn render_diff_pane(&self, frame: &mut Frame, area: Rect) {
        let Some(diff) = &self.diff_preview else {
            return;
        };
        let status = match diff.approved {
            Some(true) => "APPROVED",
            Some(false) => "DENIED",
            None => "PENDING",
        };
        let mut body = format!("File: {}\nStatus: {status}\n\n", diff.file_path);
        for h in &diff.hunks {
            body.push_str(h);
            body.push('\n');
        }
        let pane = Paragraph::new(body)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Diff / Patch Review"));
        frame.render_widget(pane, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}

fn split_shell_words(raw: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), ch) if ch == q => quote = None,
            (Some(_), '\\') => {
                if let Some(next) = chars.next() {
                    cur.push(next);
                }
            }
            (Some(_), ch) => cur.push(ch),
            (None, '"' | '\'') => quote = Some(c),
            (None, ch) if ch.is_whitespace() => {
                if !cur.is_empty() {
                    args.push(std::mem::take(&mut cur));
                }
            }
            (None, ch) => cur.push(ch),
        }
    }
    if quote.is_some() {
        return Err("Unclosed quote in CLI command".to_string());
    }
    if !cur.is_empty() {
        args.push(cur);
    }
    Ok(args)
}

/// Parses and executes `toksi <subcommand>` / `!<subcommand>` / `:cli <subcommand>`
/// from inside `<toksi-siagent>` chat, enforcing `CommandGuard` pre-exec security.
pub fn execute_in_chat_cli_command(
    workspace: &Path,
    input: &str,
) -> Option<Result<String, String>> {
    let trimmed = input.trim();
    let sub_cmd = if trimmed == "toksi" {
        "--help"
    } else if let Some(rest) = trimmed.strip_prefix("toksi ") {
        rest.trim()
    } else if let Some(rest) = trimmed.strip_prefix(":cli ") {
        rest.trim()
    } else {
        let rest = trimmed.strip_prefix('!')?;
        let r = rest.trim();
        r.strip_prefix("toksi ").unwrap_or(r)
    };

    if sub_cmd.is_empty() {
        return Some(Err("Empty CLI command".to_string()));
    }

    match CommandGuard::check_pre_exec("toksi-chat-tui", trimmed, Some(workspace)) {
        GuardDecision::Allow => {}
        GuardDecision::Deny { reason, .. } | GuardDecision::KillSwitchTriggered { reason, .. } => {
            return Some(Err(reason));
        }
    }

    let argv = match split_shell_words(sub_cmd) {
        Ok(v) if !v.is_empty() => v,
        Ok(_) => return Some(Err("Empty CLI command".to_string())),
        Err(e) => return Some(Err(e)),
    };

    if argv.len() == 1 && matches!(argv[0].as_str(), "version" | "--version" | "-v") {
        return Some(Ok(crate::util::version::toksi_version().to_string()));
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return Some(Err(format!("Failed locating toksi binary: {e}"))),
    };
    let output = match Command::new(&exe).args(&argv).current_dir(workspace).output() {
        Ok(o) => o,
        Err(e) => return Some(Err(format!("Failed spawning toksi CLI command: {e}"))),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let combined = match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => format!("(exit {})", output.status.code().unwrap_or(0)),
    };
    if output.status.success() {
        Some(Ok(combined))
    } else {
        Some(Err(format!(
            "toksi {} exited with {}: {}",
            argv.join(" "),
            output.status.code().unwrap_or(1),
            combined
        )))
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}

enum WorkerEvent {
    TurnFinished {
        prompt: String,
        role: ChatMessageRole,
        title: String,
        output: String,
        is_error: bool,
    },
}

fn dispatch_chat_turn_blocking(
    workspace: &Path,
    agent_id: &str,
    gateway_url: Option<String>,
    mode: OrchestratorMode,
    effort: EffortLevel,
    input: &str,
) -> WorkerEvent {
    if let Some(cli_res) = execute_in_chat_cli_command(workspace, input) {
        return match cli_res {
            Ok(out) => WorkerEvent::TurnFinished {
                prompt: input.to_string(),
                role: ChatMessageRole::Tool,
                title: "toksi-cli".to_string(),
                output: out,
                is_error: false,
            },
            Err(err) => WorkerEvent::TurnFinished {
                prompt: input.to_string(),
                role: ChatMessageRole::System,
                title: "cli-error".to_string(),
                output: err,
                is_error: true,
            },
        };
    }

    let slash = global_slash_engine().execute(input, agent_id);
    if slash.handled {
        return WorkerEvent::TurnFinished {
            prompt: input.to_string(),
            role: ChatMessageRole::System,
            title: "slash".to_string(),
            output: slash.output,
            is_error: false,
        };
    }

    let mut engine = AgentRuntimeEngine::new(workspace.to_path_buf(), agent_id)
        .with_mode_and_effort(mode, effort);
    if let Some(url) = gateway_url {
        engine = engine.with_gateway_url(url);
    }
    match engine.run_single_shot(input) {
        Ok(rep) => WorkerEvent::TurnFinished {
            prompt: input.to_string(),
            role: ChatMessageRole::Assistant,
            title: format!("siagent ({})", mode.as_str()),
            output: rep.output,
            is_error: false,
        },
        Err(err) => WorkerEvent::TurnFinished {
            prompt: input.to_string(),
            role: ChatMessageRole::System,
            title: "siagent-error".to_string(),
            output: err,
            is_error: true,
        },
    }
}

pub fn run_interactive_chat_tui(engine: &mut AgentRuntimeEngine) -> i32 {
    let mut terminal = match TerminalGuard::enter() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed starting chat TUI: {e}");
            return 1;
        }
    };
    let _guard = TerminalGuard;

    let mut state = ChatUiState::new(engine.session_id.clone(), engine.workspace_root.clone());
    state.active_mode = engine.active_mode;
    state.active_effort = engine.active_effort;

    let (tx, rx) = mpsc::channel::<WorkerEvent>();

    loop {
        if let Err(e) = terminal.draw(|f| state.render(f)) {
            eprintln!("Chat TUI draw error: {e}");
            return 1;
        }

        while let Ok(evt) = rx.try_recv() {
            match evt {
                WorkerEvent::TurnFinished {
                    prompt,
                    role,
                    title,
                    output,
                    is_error,
                } => {
                    state.streaming_active = false;
                    state.status_notice = if is_error {
                        "Last turn errored".to_string()
                    } else {
                        "Ready".to_string()
                    };
                    if role == ChatMessageRole::Tool {
                        state.add_tool_card(
                            format!("cmd-{}", state.tool_cards.len() + 1),
                            title.clone(),
                            prompt,
                            output.clone(),
                            if is_error { "error" } else { "ok" },
                        );
                    }
                    state.push_message(role, title, output);
                }
            }
        }

        if state.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50)).unwrap_or(false)
            && let Ok(ev) = event::read()
        {
            match ev {
                Event::Key(key) => {
                    if let Some(action) = state.handle_key_event(key) {
                        match action {
                            ChatUiAction::Quit => break,
                            ChatUiAction::CycleMode => {
                                engine.active_mode = state.active_mode;
                                engine.active_effort = state.active_effort;
                            }
                            ChatUiAction::ToggleDiffPane | ChatUiAction::ResolveHitl { .. } => {}
                            ChatUiAction::SubmitInput(text) => {
                                state.push_message(ChatMessageRole::User, "user", text.clone());
                                if let Some(rest) = text.strip_prefix("/mode ") {
                                    let _ = state.set_mode(rest);
                                    engine.active_mode = state.active_mode;
                                    engine.active_effort = state.active_effort;
                                } else if let Some(rest) = text.strip_prefix("/effort ")
                                    && let Some(eff) = EffortLevel::from_str_loose(rest)
                                {
                                    state.active_effort = eff;
                                    engine.active_effort = eff;
                                }
                                state.streaming_active = true;
                                state.status_notice = "Running...".to_string();
                                let tx_c = tx.clone();
                                let ws = engine.workspace_root.clone();
                                let aid = engine.agent_id.clone();
                                let gw = engine.gateway_base_url.clone();
                                let mode = state.active_mode;
                                let effort = state.active_effort;
                                std::thread::spawn(move || {
                                    let res =
                                        dispatch_chat_turn_blocking(&ws, &aid, gw, mode, effort, &text);
                                    let _ = tx_c.send(res);
                                });
                            }
                        }
                    }
                }
                Event::Mouse(m) => state.handle_mouse_event(m),
                _ => {}
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_chat_ui_state_22_modes_render_and_cli_bridge() {
        let mut state = ChatUiState::new("test-sess", PathBuf::from("."));
        assert_eq!(state.active_mode, OrchestratorMode::WriteCodeBuild);
        assert!(state.set_mode("coding").is_ok());
        assert_eq!(state.active_mode, OrchestratorMode::WriteCodeBuild);
        assert!(state.set_mode("research").is_ok());
        assert_eq!(state.active_mode, OrchestratorMode::ResearchGround);

        state.add_tool_card("t1", "codegraph", "find callers", "3 matches", "ok");
        assert!(state.tool_cards[0].collapsed);
        state.toggle_tool_card(0);
        assert!(!state.tool_cards[0].collapsed);

        state.open_diff_preview("src/lib.rs", vec!["@@ -1,2 +1,3 @@".to_string()]);
        state.open_hitl_modal("req-1", "apply_patch", "apply diff to src/lib.rs");
        let resolved = state.resolve_hitl_modal(true);
        assert_eq!(resolved.as_deref(), Some("req-1"));
        assert_eq!(
            state.diff_preview.as_ref().and_then(|d| d.approved),
            Some(true)
        );

        let backend = TestBackend::new(100, 30);
        let mut term = Terminal::new(backend).expect("test backend");
        term.draw(|f| state.render(f)).expect("render chat ui");

        let cli = execute_in_chat_cli_command(Path::new("."), "toksi version")
            .expect("cli matched")
            .expect("version succeeds");
        assert!(!cli.is_empty());

        let blocked = execute_in_chat_cli_command(Path::new("."), "toksi exec nmap -sV 127.0.0.1")
            .expect("cli matched");
        assert!(blocked.is_err());
    }
}
