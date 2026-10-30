// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tui/tests.rs

use super::app::*;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn press_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

#[test]
fn test_app_initialization_and_tabs() {
    let app = App::new();
    assert_eq!(app.current_tab, 0);
    assert_eq!(app.tab_names.len(), 6);
    assert!(!app.should_quit);
    assert!(!app.monitor.waterfall.is_empty());
}

#[test]
fn test_tab_navigation_keys() {
    let mut app = App::new();

    // Tab forwards
    app.handle_key(press_key(KeyCode::Tab));
    assert_eq!(app.current_tab, 1);

    app.handle_key(press_key(KeyCode::Tab));
    assert_eq!(app.current_tab, 2);

    // Number keys direct navigation
    app.handle_key(press_key(KeyCode::Char('5')));
    assert_eq!(app.current_tab, 4);

    app.handle_key(press_key(KeyCode::Char('1')));
    assert_eq!(app.current_tab, 0);

    // Tab backwards
    app.handle_key(press_key(KeyCode::BackTab));
    assert_eq!(app.current_tab, 5);
}

#[test]
fn test_quit_key() {
    let mut app = App::new();
    assert!(!app.should_quit);
    app.handle_key(press_key(KeyCode::Char('q')));
    assert!(app.should_quit);
}

#[test]
fn test_memory_filter_and_search() {
    let mut app = App::new();
    app.current_tab = 2; // Memory tab

    let total = app.memory.items.len();
    assert!(total > 0);

    // Filter toggle
    app.handle_key(press_key(KeyCode::Char('f')));
    assert_eq!(app.memory.filter, MemoryFilter::Issues);

    // Search query
    app.memory.search_query = "codegraph".to_string();
    app.memory.filter = MemoryFilter::All;
    let matching = app.memory.filtered_events();
    for item in matching {
        assert!(
            item.summary.to_lowercase().contains("codegraph")
                || item.location.to_lowercase().contains("codegraph")
        );
    }
}

#[test]
fn test_graph_symbol_filter() {
    let mut app = App::new();
    app.current_tab = 3; // Graph tab

    app.graph.search_query = "top".to_string();
    let filtered = app.graph.filtered_symbols();
    assert!(!filtered.is_empty());
    for s in filtered {
        assert!(s.name.to_lowercase().contains("top") || s.file.to_lowercase().contains("top"));
    }
}

#[test]
fn test_agent_selection_and_modal() {
    let mut app = App::new();
    app.current_tab = 1; // Agents tab

    if !app.agents.items.is_empty() {
        app.handle_key(press_key(KeyCode::Char('j'))); // Move down
        assert!(app.agents.selected <= app.agents.items.len() - 1);

        // Open MCP config modal
        app.handle_key(press_key(KeyCode::Enter));
        assert!(app.agents.inspect_mcp.is_some());

        // Close modal with Esc
        app.handle_key(press_key(KeyCode::Esc));
        assert!(app.agents.inspect_mcp.is_none());
    }
}
