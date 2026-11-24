// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tui/tests.rs
// created: 2026-10-30
// lastModified: 2026-11-19

use super::app::*;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

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
    assert_eq!(app.tab_names.len(), 10);
    assert_eq!(app.tab_names[0], "Savings");
    assert_eq!(app.tab_names[1], "Activity Room");
    assert_eq!(app.tab_names[2], "AI Agent Fleet");
    assert_eq!(app.tab_names[3], "Quotas");
    assert_eq!(app.tab_names[8], "Keyring & Database");
    assert_eq!(app.tab_names[9], "System Controls");
    assert!(!app.should_quit);
    assert_eq!(app.monitor.tokens_saved_lifetime, 0);
    assert!(app.monitor.waterfall.is_empty());
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

    app.handle_key(press_key(KeyCode::Char('6')));
    assert_eq!(app.current_tab, 5);

    app.handle_key(press_key(KeyCode::Char('9')));
    assert_eq!(app.current_tab, 8);

    app.handle_key(press_key(KeyCode::Char('0')));
    assert_eq!(app.current_tab, 9);

    app.handle_key(press_key(KeyCode::Char('1')));
    assert_eq!(app.current_tab, 0);

    // Tab backwards wraps to 10th tab (index 9)
    app.handle_key(press_key(KeyCode::BackTab));
    assert_eq!(app.current_tab, 9);
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
    app.current_tab = 4; // Memory tab (Tab 5)

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
    app.current_tab = 5; // Graph tab (Tab 6)

    app.graph.search_query = "top".to_string();
    let filtered = app.graph.filtered_symbols();
    assert!(!filtered.is_empty());
    for s in filtered {
        assert!(s.name.to_lowercase().contains("top") || s.file.to_lowercase().contains("top"));
    }
}

#[test]
fn test_governance_navigation_and_actions() {
    let mut app = App::new();
    app.current_tab = 8; // Keyring & Database tab (Tab 9)

    assert_eq!(app.governance.selected, 0);
    assert!(app.governance.keys.is_empty());

    // Refresh Keyring & DB
    app.handle_key(press_key(KeyCode::Char('r')));
    assert!(app.status_message.is_some());

    // Clear trips
    app.governance.circuit_breaker_trips = 3;
    app.handle_key(press_key(KeyCode::Char('c')));
    assert_eq!(app.governance.circuit_breaker_trips, 0);
}

#[test]
fn test_agent_selection_and_modal() {
    let mut app = App::new();
    app.current_tab = 2; // Agents tab (Tab 3)

    if !app.agents.items.is_empty() {
        app.handle_key(press_key(KeyCode::Char('j'))); // Move down
        assert!(app.agents.selected < app.agents.items.len());

        // Open MCP config modal
        app.handle_key(press_key(KeyCode::Enter));
        assert!(app.agents.inspect_mcp.is_some());

        // Close modal with Esc
        app.handle_key(press_key(KeyCode::Esc));
        assert!(app.agents.inspect_mcp.is_none());
    }
}

#[test]
fn test_mouse_tab_switching() {
    let mut app = App::new();
    assert_eq!(app.current_tab, 0);

    let area = Rect::new(0, 0, 140, 40);

    // 140 / 10 = 14 cols per tab; col 20 -> tab 1; col 72 -> tab 5
    let mouse_tab2 = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 20,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(mouse_tab2, area);
    assert_eq!(app.current_tab, 1);

    let mouse_tab6 = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 72,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(mouse_tab6, area);
    assert_eq!(app.current_tab, 5);
}

#[test]
fn test_mouse_action_buttons() {
    let mut app = App::new();
    let area = Rect::new(0, 0, 120, 40);

    // Tab 0 Monitor: Proxy Up button at footer_start - 2, col 10
    app.current_tab = 0;
    app.monitor.proxy_up = false;
    let btn_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 36, // button row
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(btn_click, area);
    assert!(app.monitor.proxy_up);

    // Tab 0 Monitor: Proxy Down button at col 30
    let btn_down_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 30,
        row: 36,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(btn_down_click, area);
    assert!(!app.monitor.proxy_up);

    // Tab 4 Memory: Precheck button
    app.current_tab = 4;
    let precheck_click = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 36,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(precheck_click, area);
    assert!(app.memory.precheck_result.is_some());

    // Dismiss modal on outside click
    let modal_dismiss = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 1, // outside
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(modal_dismiss, area);
    assert!(app.memory.precheck_result.is_none());
}

#[test]
fn test_mouse_wheel_scroll() {
    let mut app = App::new();
    app.current_tab = 9; // System Controls / Logs
    app.logs.follow = true;
    app.logs.scroll_offset = 0;

    let area = Rect::new(0, 0, 120, 40);

    let scroll_down = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 50,
        row: 20,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(scroll_down, area);
    assert!(!app.logs.follow);
    assert_eq!(app.logs.scroll_offset, 3);

    let scroll_up = MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 50,
        row: 20,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_mouse(scroll_up, area);
    assert_eq!(app.logs.scroll_offset, 0);
}

#[test]
fn test_byok_tier_badges() {
    let app = App::new();
    for r in &app.headroom.routes {
        assert!(
            matches!(
                r.tier_badge.as_str(),
                "[BYOK]" | "[OAUTH]" | "[API-KEY]" | "[FREE-TIER]" | "[UNCONFIGURED]"
            ),
            "unexpected tier badge: {}",
            r.tier_badge
        );
        assert_ne!(r.tier_badge, "[PAID]");
        assert_ne!(r.tier_badge, "[DISQUALIFIED-FREE]");
    }
}

#[test]
fn test_memory_heatmap_popup_and_gateway_refresh() {
    let mut app = App::new();
    app.current_tab = 4; // Memory Decision DAG
    if !app.memory.heatmap.is_empty() {
        app.handle_key(press_key(KeyCode::Char('h')));
        assert!(app.memory.heatmap_popup.is_some());
        app.handle_key(press_key(KeyCode::Esc));
        assert!(app.memory.heatmap_popup.is_none());
    }

    app.current_tab = 7; // AI Gateway
    app.handle_key(press_key(KeyCode::Char('m')));
    assert!(app.status_message.is_some());
}
