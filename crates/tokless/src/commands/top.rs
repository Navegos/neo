// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/top.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::env;
use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::commands::daemon::is_daemon_running;
use crate::commands::doctor_stats::doctor_tool_stats;
use crate::core::list_agents;
use crate::headroom::proxy::{ensure_proxy_up, proxy_running, stop_proxy};
use crate::util::colors::*;
use crate::util::headroom_proxy::proxy_routing_enabled;
use crate::util::version::tokless_version;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
    }
}

pub fn run_top(once: bool) -> i32 {
    let is_tty = crossterm::tty::IsTty::is_tty(&stdout());
    if once || !is_tty {
        // Fallback or explicit one-shot: render single frame
        render_frame();
        return 0;
    }

    if enable_raw_mode().is_err() {
        render_frame();
        return 0;
    }

    let _guard = TerminalGuard;
    let _ = execute!(stdout(), EnterAlternateScreen, Hide);

    loop {
        render_frame();

        if event::poll(Duration::from_millis(1000)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => {}
                    KeyCode::Char('u') => {
                        let _ = ensure_proxy_up();
                    }
                    KeyCode::Char('d') => {
                        let _ = stop_proxy();
                    }
                    _ => {}
                }
            }
        }
    }

    0
}

fn render_frame() {
    let mut out = stdout();
    let _ = execute!(out, MoveTo(0, 0));

    let cwd = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into());

    let hr_active = proxy_running();
    let daemon_active = is_daemon_running();
    let routing = proxy_routing_enabled();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut buf = String::new();
    buf.push_str(&format!(
        "{} {}  {}  {}\n\n",
        C.bold(&C.cyan("tokless top")),
        C.gray(&format!("v{}", tokless_version())),
        C.dim(&format!("cwd: {}", cwd)),
        C.dim(&now)
    ));

    // Proxies & Supervisor
    buf.push_str(&format!("{}\n", C.bold("--- Services & Routing ---")));
    buf.push_str(&format!(
        "  Headroom Proxy (8787):    {}   Proxy Routing: {}\n",
        if hr_active { C.green("ACTIVE") } else { C.red("STOPPED") },
        if routing { C.cyan("ENABLED") } else { C.dim("DIRECT") }
    ));
    buf.push_str(&format!(
        "  Supervisor Daemon:        {}   BYOK Gateway (28786): {}\n\n",
        if daemon_active { C.green("RUNNING") } else { C.dim("IDLE") },
        if hr_active { C.green("ONLINE") } else { C.dim("OFFLINE") }
    ));

    // Performance & Savings
    buf.push_str(&format!("{}\n", C.bold("--- Savings & Warmth ---")));
    let stats = doctor_tool_stats(&cwd, true);
    if let Some(hr_rows) = stats.get("headroom") {
        for row in hr_rows {
            if row.contains("warmth:") || row.contains("lifetime:") || row.contains("today:") {
                buf.push_str(&format!("  {}\n", row.trim()));
            }
        }
        buf.push_str("\n");
    } else {
        buf.push_str("  Headroom statistics unavailable.\n\n");
    }

    if let Some(pm_rows) = stats.get("projectmem") {
        buf.push_str(&format!("{}\n", C.bold("--- Project Memory ---")));
        for row in pm_rows {
            buf.push_str(&format!("  {}\n", row.trim()));
        }
        buf.push_str("\n");
    }

    // Detected Agents
    buf.push_str(&format!("{}\n", C.bold("--- Detected Agents ---")));
    let agents = list_agents();
    let mut installed_agents = Vec::new();
    for a in &agents {
        if a.detect().installed {
            installed_agents.push(a);
        }
    }
    if installed_agents.is_empty() {
        buf.push_str("  No compatible agents detected.\n");
    } else {
        let names: Vec<String> = installed_agents
            .iter()
            .map(|a| format!("{} {}", C.green("✔"), a.label))
            .collect();
        for chunk in names.chunks(3) {
            buf.push_str(&format!("  {}\n", chunk.join("   |   ")));
        }
    }

    buf.push_str("\n");
    buf.push_str(&format!(
        "{}\n",
        C.dim("Keys: [q] Quit   [r] Refresh   [u] Proxy Up   [d] Proxy Down")
    ));

    let _ = out.write_all(buf.as_bytes());
    let _ = out.flush();
}
