// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/dashboard/mod.rs

pub mod api;
pub mod assets;
pub mod server;
#[cfg(test)]
pub mod tests;

pub use api::*;
pub use assets::*;
pub use server::*;

use crate::util::colors::*;
use crate::util::logger::L;

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

pub fn run_dashboard_cli(argv: &[String]) -> i32 {
    let mut port: u16 = 9099;
    let mut open = false;

    let mut i = 0;
    while i < argv.len() {
        let arg = &argv[i];
        if arg == "--help" || arg == "-h" {
            println!("Usage: tokless dashboard [--port <port>] [--open]");
            return 0;
        } else if arg == "--port" || arg == "-p" {
            if i + 1 < argv.len() {
                if let Ok(p) = argv[i + 1].parse::<u16>() {
                    port = p;
                }
                i += 1;
            }
        } else if arg.starts_with("--port=") {
            if let Ok(p) = arg["--port=".len()..].parse::<u16>() {
                port = p;
            }
        } else if arg == "--open" || arg == "-o" {
            open = true;
        }
        i += 1;
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            L.err(&format!("Failed to initialize Tokio runtime: {}", e));
            return 1;
        }
    };

    let url = format!("http://127.0.0.1:{}", port);
    println!(
        "\n  {} {}\n  {} {}\n  {} {}\n\n  {}",
        C.cyan("tokless"),
        C.bold("Telemetry & Control Cockpit (Phase 12)"),
        C.dim("Local Dashboard:"),
        C.cyan(&url),
        C.dim("SSE Stream:"),
        C.cyan(&format!("{}/api/events", url)),
        C.dim("Press Ctrl+C to stop dashboard server...")
    );

    if open {
        open_browser(&url);
    }

    let server = DashboardServer::new("127.0.0.1", port);
    if let Err(e) = rt.block_on(server.run()) {
        L.err(&format!("Dashboard server terminated: {}", e));
        return 1;
    }

    0
}

