// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/mod.rs
// created: 2026-10-30
// lastModified: 2026-11-08

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

pub const DEFAULT_DASHBOARD_PORT: u16 = 9100;

pub fn dashboard_port() -> u16 {
    if let Ok(raw) = std::env::var("TOKSI_DASHBOARD_PORT") {
        if let Ok(p) = raw.trim().parse::<u16>() {
            if p > 0 {
                return p;
            }
        }
    }
    DEFAULT_DASHBOARD_PORT
}

pub fn dashboard_live_z(port: u16, timeout: std::time::Duration) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    use std::io::{Read, Write};
    let req = format!("GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut resp = [0u8; 256];
    let Ok(n) = stream.read(&mut resp) else {
        return false;
    };
    let s = String::from_utf8_lossy(&resp[..n]);
    s.starts_with("HTTP/1.1 200") || s.starts_with("HTTP/1.0 200")
}

pub fn is_dashboard_running() -> bool {
    dashboard_live_z(dashboard_port(), std::time::Duration::from_millis(500))
}

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
    let mut port: u16 = DEFAULT_DASHBOARD_PORT;
    let mut open = false;

    let mut i = 0;
    while i < argv.len() {
        let arg = &argv[i];
        if arg == "--help" || arg == "-h" {
            println!("Usage: toksi dashboard [--port <port>] [--open]");
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
        "\n  {} {}\n  {} {}\n  {} {}\n  {} {}\n\n  {}",
        C.cyan("toksi"),
        C.bold("Telemetry & Control Cockpit (Phase 12)"),
        C.dim("Local Dashboard:"),
        C.cyan(&url),
        C.dim("Prometheus Metrics:"),
        C.cyan(&format!("{}/metrics", url)),
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

