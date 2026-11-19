// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/tui.rs
// created: 2026-10-30
// lastModified: 2026-11-06

use crate::tui::run_tui_with_tab;

pub fn run_tui_cli(argv: &[String]) -> i32 {
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Usage: toksi tui [savings|warroom|agents|memory|graph|sandbox|controls|gateway|keyring]"
        );
        return 0;
    }

    let initial_tab = match argv.first().map(|s| s.to_lowercase()).as_deref() {
        Some("1") | Some("savings") | Some("monitor") | Some("top") | Some("waterfall") => 0,
        Some("2") | Some("warroom") | Some("war-room") | Some("telemetry") => 1,
        Some("3") | Some("agents") | Some("fleet") | Some("wire") => 2,
        Some("4") | Some("memory") | Some("mem") | Some("dag") | Some("projectmem") => 3,
        Some("5") | Some("graph") | Some("codegraph") | Some("ast") => 4,
        Some("6") | Some("sandbox") | Some("security") | Some("defenseclaw") => 5,
        Some("7") | Some("controls") | Some("system") | Some("logs") | Some("diag") => 6,
        Some("8") | Some("gateway") | Some("byok") | Some("headroom") | Some("quotas") => 7,
        Some("9") | Some("keyring") | Some("db") | Some("database") | Some("governance")
        | Some("keys") | Some("vk") => 8,
        _ => 0,
    };

    run_tui_with_tab(initial_tab)
}
