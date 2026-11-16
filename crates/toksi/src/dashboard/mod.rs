// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/mod.rs
// created: 2026-10-30
// lastModified: 2026-11-16

#![deny(unsafe_code)]

pub mod api;
pub mod assets;
pub mod server;
pub mod sse;
pub mod state;
#[cfg(test)]
pub mod tests;

pub use api::*;
pub use assets::*;
pub use server::*;
pub use sse::*;
pub use state::*;

use std::path::Path;

use crate::util::colors::*;
use crate::util::headroom::headroom_paths_resolved;
use crate::util::logger::L;
use crate::util::paths::{ensure_dir, read_file_safe, write_file_atomic};

pub const DEFAULT_DASHBOARD_PORT: u16 = 9100;

pub fn dashboard_port() -> u16 {
    crate::config::resolved_ports().dashboard
}

fn dashboard_pid_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("dashboard.pid")
        .to_string_lossy()
        .to_string()
}

pub fn stop_dashboard_process() -> bool {
    let pid_file = dashboard_pid_file();
    let Some(raw) = read_file_safe(&pid_file) else {
        return false;
    };
    let Ok(pid) = raw.trim().parse::<u32>() else {
        let _ = std::fs::remove_file(&pid_file);
        return false;
    };
    let killed = crate::headroom::process_identity::kill_process(pid).is_ok();
    let _ = std::fs::remove_file(&pid_file);
    killed
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
    let mut port: u16 = dashboard_port();
    let mut open = false;
    let mut foreground = false;
    let mut action = "start";

    let mut i = 0;
    while i < argv.len() {
        let arg = &argv[i];
        if arg == "--help" || arg == "-h" {
            println!("Usage: toksi dashboard [start|stop|restart|status] [--port <port>] [--open] [--foreground]");
            println!("Runs the Web Telemetry Cockpit on Port {} (config.toml [ports].dashboard) in the background by default.", port);
            return 0;
        } else if matches!(arg.as_str(), "start" | "stop" | "restart" | "status") {
            action = match arg.as_str() {
                "stop" => "stop",
                "restart" => "restart",
                "status" => "status",
                _ => "start",
            };
        } else if arg == "--port" || arg == "-p" {
            if i + 1 < argv.len() {
                if let Ok(p) = argv[i + 1].parse::<u16>()
                    && p > 0
                {
                    port = p;
                }
                i += 1;
            }
        } else if let Some(raw_p) = arg.strip_prefix("--port=") {
            if let Ok(p) = raw_p.parse::<u16>()
                && p > 0
            {
                port = p;
            }
        } else if arg == "--open" || arg == "-o" {
            open = true;
        } else if arg == "--foreground" || arg == "-f" || arg == "__run-foreground" {
            foreground = true;
        }
        i += 1;
    }

    if action == "status" {
        let running = dashboard_live_z(port, std::time::Duration::from_millis(500));
        println!(
            "  {} Web Telemetry Cockpit (:{}) is {}",
            if running { C.green(Sym::CHECK) } else { C.yellow(Sym::WARN) },
            port,
            if running { "running" } else { "stopped" }
        );
        return 0;
    }

    if action == "stop" {
        if stop_dashboard_process() || !dashboard_live_z(port, std::time::Duration::from_millis(300)) {
            println!("  {} Web Telemetry Cockpit stopped", C.green(Sym::CHECK));
            return 0;
        }
        L.warn(&format!("Dashboard on :{} may be supervised by `toksi daemon`; run `toksi daemon stop` if still active", port));
        return 0;
    }

    if action == "restart" {
        let _ = stop_dashboard_process();
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    let url = format!("http://127.0.0.1:{}", port);

    // If already running
    if dashboard_live_z(port, std::time::Duration::from_millis(500)) {
        println!(
            "\n  {} {}\n  {} {}\n",
            C.cyan("toksi"),
            C.bold("Web Cockpit already running in background"),
            C.dim("Local Dashboard:"),
            C.cyan(&url)
        );
        if open {
            open_browser(&url);
        }
        return 0;
    }

    // Default to background spawning unless --foreground requested
    if !foreground {
        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => std::path::PathBuf::from("toksi"),
        };
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(["dashboard", "--foreground", &format!("--port={}", port)]);
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }

        match cmd.spawn() {
            Ok(child) => {
                let _ = ensure_dir(&headroom_paths_resolved().root);
                let _ = write_file_atomic(dashboard_pid_file(), &child.id().to_string());
                std::thread::sleep(std::time::Duration::from_millis(600));
                println!(
                    "\n  {} {}\n  {} {}\n  {} {}\n  {} {}\n",
                    C.cyan("toksi"),
                    C.bold("Enterprise Web Telemetry Cockpit (Background)"),
                    C.dim("Local Dashboard:"),
                    C.cyan(&url),
                    C.dim("Prometheus Metrics:"),
                    C.cyan(&format!("{}/metrics", url)),
                    C.dim("SSE Live Stream:"),
                    C.cyan(&format!("{}/api/events", url))
                );

                if open {
                    open_browser(&url);
                }
                return 0;
            }
            Err(e) => {
                L.warn(&format!("Could not spawn background process, running in foreground: {}", e));
            }
        }
    }

    let _ = ensure_dir(&headroom_paths_resolved().root);
    let _ = write_file_atomic(dashboard_pid_file(), &std::process::id().to_string());

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("toksi-dashboard-cockpit")
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            L.err(&format!("Failed to initialize Tokio runtime: {}", e));
            return 1;
        }
    };

    println!(
        "\n  {} {}\n  {} {}\n  {} {}\n  {} {}\n\n  {}",
        C.cyan("toksi"),
        C.bold("Telemetry & Control Cockpit (Phase 16.2)"),
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

    let (shutdown_tx, _) = tokio::sync::watch::channel(false);
    let tx_sig = shutdown_tx.clone();
    let mut server = DashboardServer::new("127.0.0.1", port);
    server.shutdown_tx = Some(shutdown_tx);
    let res = rt.block_on(async move {
        tokio::spawn(async move {
            let _ = crate::env_injector::wait_for_shutdown_signal().await;
            let _ = tx_sig.send(true);
        });
        server.run().await
    });
    let _ = std::fs::remove_file(dashboard_pid_file());
    if let Err(e) = res {
        L.err(&format!("Dashboard server terminated: {}", e));
        return 1;
    }

    0
}

