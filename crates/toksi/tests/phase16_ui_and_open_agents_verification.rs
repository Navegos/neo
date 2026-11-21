// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase16_ui_and_open_agents_verification.rs

#![deny(unsafe_code)]

use toksi::dashboard::assets::{APP_JS, INDEX_HTML};
use toksi::tui::app::App;

#[test]
fn test_phase16_exact_9_tab_parity_and_zero_fake_data() {
    let app = App::new();
    let expected = [
        "Savings",
        "The War Room",
        "AI Agent Fleet & Quotas",
        "Memory Decision DAG",
        "CodeGraph Explorer",
        "Sandbox Inspector",
        "System Controls",
        "AI Gateway & Quotas",
        "Keyring & Database",
    ];
    assert_eq!(app.tab_names, expected);

    for name in expected {
        let html_name = name.replace('&', "&amp;");
        assert!(
            INDEX_HTML.contains(name) || INDEX_HTML.contains(&html_name),
            "index.html missing tab: {}",
            name
        );
    }

    assert!(!INDEX_HTML.contains("Waterfall & Savings"));
    assert!(!INDEX_HTML.contains("Agent Fleet (17)"));
    assert!(!INDEX_HTML.contains("Insperctor"));
    assert!(!INDEX_HTML.contains("[DISQUALIFIED-FREE]"));
    assert!(!APP_JS.contains("Disqualified Free"));
    assert!(APP_JS.contains("openHeatmapDrilldown"));
}
