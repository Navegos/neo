// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/widgets/button.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Widget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Success,
    Warning,
    Danger,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: &'a str,
    style: ButtonStyle,
    selected: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            style: ButtonStyle::Neutral,
            selected: false,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl<'a> Widget for Button<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        let (fg, bg) = match (self.style, self.selected) {
            (ButtonStyle::Primary, false) => (Color::White, Color::Rgb(20, 60, 120)),
            (ButtonStyle::Primary, true) => (Color::Black, Color::Cyan),
            (ButtonStyle::Success, false) => (Color::White, Color::Rgb(20, 100, 40)),
            (ButtonStyle::Success, true) => (Color::Black, Color::Green),
            (ButtonStyle::Warning, false) => (Color::Black, Color::Yellow),
            (ButtonStyle::Warning, true) => (Color::White, Color::Rgb(180, 100, 0)),
            (ButtonStyle::Danger, false) => (Color::White, Color::Rgb(140, 20, 20)),
            (ButtonStyle::Danger, true) => (Color::White, Color::Red),
            (ButtonStyle::Neutral, false) => (Color::Gray, Color::Rgb(40, 40, 40)),
            (ButtonStyle::Neutral, true) => (Color::White, Color::Rgb(70, 70, 70)),
        };

        let style = Style::default()
            .fg(fg)
            .bg(bg)
            .add_modifier(if self.selected { Modifier::BOLD } else { Modifier::empty() });

        let text = format!(" [ {} ] ", self.label);
        let span = Span::styled(text, style);
        buf.set_span(area.x, area.y, &span, area.width);
    }
}
