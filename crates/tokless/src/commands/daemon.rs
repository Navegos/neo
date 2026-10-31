// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/daemon.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::commands::doctor_stats::process_alive;
use crate::headroom::proxy::{proxy_running, start_proxy};
use crate::util::colors::*;
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::proxy_routing_enabled;
use crate::util::logger::L;
use crate::util::paths::{ensure_dir, read_file_safe, tokless_abs_strict, write_file_atomic};

fn daemon_pid_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("daemon.pid")
        .to_string_lossy()
        .to_string()
}

fn daemon_stop_file() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("daemon.stop")
        .to_string_lossy()
        .to_string()
}

pub fn is_daemon_running() -> bool {
    let pid_str = read_file_safe(daemon_pid_file()).unwrap_or_default().trim().to_string();
    if pid_str.is_empty() {
        return false;
    }
    let Ok(pid) = pid_str.parse::<u32>() else {
        return false;
    };
    process_alive(pid)
}

pub fn run_daemon_cli(argv: &[String]) -> i32 {
    let subcmd = argv.first().map(|s| s.as_str()).unwrap_or("status");
    match subcmd {
        "start" => daemon_start(),
        "stop" => daemon_stop(),
        "status" => daemon_status(),
        "run" | "__run-foreground" => daemon_run_foreground(),
        "restart" => {
            daemon_stop();
            sleep(Duration::from_millis(500));
            daemon_start()
        }
        _ => {
            println!(
                "{} — tokless background supervisor\n\n\
                {}\n  tokless daemon start    Start daemon supervisor in background\n\
                  tokless daemon stop     Stop daemon supervisor\n\
                  tokless daemon status   Check daemon and proxy health\n\
                  tokless daemon restart  Restart supervisor",
                C.bold("tokless daemon"),
                C.bold("Usage:")
            );
            1
        }
    }
}

fn daemon_status() -> i32 {
    let running = is_daemon_running();
    let hr_running = proxy_running();
    let routing = proxy_routing_enabled();

    L.raw(&format!(
        "  {} Daemon supervisor: {}",
        if running { C.green(Sym::CHECK) } else { C.yellow(Sym::WARN) },
        if running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Headroom proxy (8787): {}",
        if hr_running { C.green(Sym::CHECK) } else { C.gray("•") },
        if hr_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Proxy routing: {}",
        if routing { C.cyan("enabled") } else { C.dim("disabled") },
        if routing { "(agents routed through Headroom)" } else { "(agents direct)" }
    ));
    0
}

fn daemon_start() -> i32 {
    if is_daemon_running() {
        L.raw(&format!("  {} Tokless daemon already running", C.green(Sym::CHECK)));
        return 0;
    }

    // Self-healing: restore legacy hijacked binaries and purge shims
    let healing = crate::env_injector::heal_legacy_hijacks();
    if !healing.is_clean() {
        L.info(&format!(
            "Daemon preflight: restored {} hijacked binaries, removed {} shims",
            healing.restored_binaries.len(),
            healing.removed_shims.len()
        ));
    }

    // Connect non-destructive environment injection so environment is ready before VS Code or IDEs launch
    if let Err(e) = crate::env_injector::inject_proxy_env() {
        L.warn(&format!("Environment injection warning: {}", e));
    }

    let _ = fs::remove_file(daemon_stop_file());
    let exe = tokless_abs_strict();
    if exe.is_empty() {
        L.err("Cannot start daemon: tokless binary path unknown");
        return 1;
    }

    let mut cmd = Command::new(&exe);
    cmd.args(["daemon", "__run-foreground"]);
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
        Ok(_) => {
            sleep(Duration::from_millis(600));
            if is_daemon_running() {
                L.raw(&format!("  {} Tokless daemon supervisor started", C.green(Sym::CHECK)));
                0
            } else {
                L.warn("Daemon spawned but status verification pending");
                0
            }
        }
        Err(e) => {
            L.err(&format!("Failed spawning daemon: {}", e));
            1
        }
    }
}

fn daemon_stop() -> i32 {
    // Purge all Tokless-owned environment variables upon shutdown
    let _ = crate::env_injector::teardown_proxy_env();

    if !is_daemon_running() {
        L.raw(&format!("  {} Tokless daemon is not running", C.dim("•")));
        return 0;
    }

    let _ = write_file_atomic(daemon_stop_file(), "stop");
    let pid_str = read_file_safe(daemon_pid_file()).unwrap_or_default().trim().to_string();
    if let Ok(pid) = pid_str.parse::<u32>() {
        let _ = crate::headroom::process_identity::kill_process(pid);
    }

    let _ = fs::remove_file(daemon_pid_file());
    let _ = fs::remove_file(daemon_stop_file());

    L.raw(&format!("  {} Tokless daemon supervisor stopped", C.green(Sym::CHECK)));
    0
}

fn daemon_run_foreground() -> i32 {
    let pid = std::process::id();
    let _ = ensure_dir(&headroom_paths_resolved().root);
    if let Err(e) = write_file_atomic(daemon_pid_file(), &pid.to_string()) {
        L.err(&format!("Failed writing daemon.pid: {}", e));
        return 1;
    }

    // Register teardown handlers (SIGINT, SIGTERM, Ctrl+C) and RAII guard
    crate::env_injector::register_teardown_handlers();
    let mut teardown_guard = crate::env_injector::TeardownGuard::new();
    let _ = crate::env_injector::inject_proxy_env();

    loop {
        if Path::new(&daemon_stop_file()).exists() {
            let _ = fs::remove_file(daemon_pid_file());
            let _ = fs::remove_file(daemon_stop_file());
            break;
        }

        // Keep Headroom proxy active if routing is enabled
        if proxy_routing_enabled() && !proxy_running() {
            let _ = start_proxy();
        }

        sleep(Duration::from_secs(2));
    }

    let _ = crate::env_injector::teardown_proxy_env();
    teardown_guard.disarm();

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_status_and_files() {
        assert!(daemon_pid_file().contains("daemon.pid"));
        assert!(daemon_stop_file().contains("daemon.stop"));
        assert_eq!(daemon_status(), 0);
    }
}
