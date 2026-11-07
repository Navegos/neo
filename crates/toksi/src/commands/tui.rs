// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/tui.rs
// created: 2026-10-30
// lastModified: 2026-11-06

use crate::tui::run_tui_with_tab;

pub fn run_tui_cli(argv: &[String]) -> i32 {
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: toksi tui [monitor|agents|gateway|memory|graph|governance|logs]");
        return 0;
    }

    let initial_tab = match argv.first().map(|s| s.to_lowercase()).as_deref() {
        Some("1") | Some("monitor") | Some("top") | Some("waterfall") => 0,
        Some("2") | Some("agents") | Some("fleet") | Some("wire") => 1,
        Some("3") | Some("gateway") | Some("byok") | Some("keys") | Some("headroom") => 2,
        Some("4") | Some("memory") | Some("mem") | Some("projectmem") => 3,
        Some("5") | Some("graph") | Some("codegraph") | Some("ast") => 4,
        Some("6") | Some("governance") | Some("quotas") | Some("vk") | Some("budgets") => 5,
        Some("7") | Some("logs") | Some("diag") => 6,
        _ => 0,
    };

    run_tui_with_tab(initial_tab)
}
