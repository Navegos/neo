// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/tui.rs

use crate::tui::run_tui_with_tab;

pub fn run_tui_cli(argv: &[String]) -> i32 {
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: tokless tui [monitor|agents|memory|graph|headroom|logs]");
        return 0;
    }

    let initial_tab = match argv.first().map(|s| s.to_lowercase()).as_deref() {
        Some("1") | Some("monitor") | Some("top") => 0,
        Some("2") | Some("agents") | Some("fleet") | Some("wire") => 1,
        Some("3") | Some("memory") | Some("mem") | Some("projectmem") => 2,
        Some("4") | Some("graph") | Some("codegraph") | Some("ast") => 3,
        Some("5") | Some("headroom") | Some("byok") | Some("cache") => 4,
        Some("6") | Some("logs") | Some("diag") => 5,
        _ => 0,
    };

    run_tui_with_tab(initial_tab)
}
