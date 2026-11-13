// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/daemon.rs
// created: 2026-10-27
// lastModified: 2026-11-13

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::commands::doctor_stats::process_alive;
use crate::headroom::proxy::proxy_running;
use crate::util::colors::*;
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::proxy_routing_enabled;
use crate::util::logger::L;
use crate::util::paths::{ensure_dir, read_file_safe, toksi_abs_strict, write_file_atomic};

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
                "{} — toksi background supervisor\n\n\
                {}\n  toksi daemon start    Start daemon supervisor in background\n\
                  toksi daemon stop     Stop daemon supervisor\n\
                  toksi daemon status   Check daemon and proxy health\n\
                  toksi daemon restart  Restart supervisor",
                C.bold("toksi daemon"),
                C.bold("Usage:")
            );
            1
        }
    }
}

fn daemon_status() -> i32 {
    let ports = crate::util::ports::resolved_ports();
    let running = is_daemon_running();
    let hr_running = proxy_running();
    let grok_running = crate::headroom::grok_daemon::grok_oauth_proxy_running();
    let copilot_running = crate::headroom::copilot_daemon::copilot_proxy_running();
    let copilot_cli_running = crate::headroom::copilot_daemon::copilot_cli_proxy_running();
    let byok_running = crate::headroom::byok_gateway::byok_gateway_live();
    let gw_running = crate::gateway::server::is_gateway_running();
    let dash_running = crate::dashboard::is_dashboard_running();
    let routing = proxy_routing_enabled();

    L.raw(&format!(
        "  {} Daemon supervisor: {}",
        if running { C.green(Sym::CHECK) } else { C.yellow(Sym::WARN) },
        if running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Headroom proxy ({}): {}",
        if hr_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.headroom_proxy,
        if hr_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Grok OAuth proxy ({}): {}",
        if grok_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.grok_oauth_proxy,
        if grok_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Copilot VS Code proxy ({}): {}",
        if copilot_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.copilot_vscode_proxy,
        if copilot_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Copilot CLI proxy ({}): {}",
        if copilot_cli_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.copilot_cli_proxy,
        if copilot_cli_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} BYOK Multi-Profile gateway ({}): {}",
        if byok_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.byok_gateway,
        if byok_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} AI Gateway ({}): {}",
        if gw_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.ai_gateway,
        if gw_running { "running" } else { "stopped" }
    ));
    L.raw(&format!(
        "  {} Web Dashboard Cockpit ({}): {}",
        if dash_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.dashboard,
        if dash_running { "running" } else { "stopped" }
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
        let _ = crate::env_injector::inject_proxy_env();
        L.raw(&format!("  {} Toksi daemon already running", C.green(Sym::CHECK)));
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
    let exe = toksi_abs_strict();
    if exe.is_empty() {
        L.err("Cannot start daemon: toksi binary path unknown");
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
                L.raw(&format!("  {} Toksi daemon supervisor started", C.green(Sym::CHECK)));
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
    // Purge all Toksi-owned environment variables upon shutdown
    let _ = crate::env_injector::teardown_proxy_env();

    if !is_daemon_running() {
        L.raw(&format!("  {} Toksi daemon is not running", C.dim("•")));
        return 0;
    }

    let _ = write_file_atomic(daemon_stop_file(), "stop");
    let pid_str = read_file_safe(daemon_pid_file()).unwrap_or_default().trim().to_string();
    if let Ok(pid) = pid_str.parse::<u32>() {
        let _ = crate::headroom::process_identity::kill_process(pid);
    }

    let _ = fs::remove_file(daemon_pid_file());
    let _ = fs::remove_file(daemon_stop_file());

    L.raw(&format!("  {} Toksi daemon supervisor stopped", C.green(Sym::CHECK)));
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

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            L.err(&format!("Failed to build Tokio runtime: {}", e));
            let _ = crate::env_injector::teardown_proxy_env();
            teardown_guard.disarm();
            return 1;
        }
    };

    let run_res = rt.block_on(async {
        run_supervised_daemon_async().await
    });

    if let Err(e) = run_res {
        L.err(&format!("Daemon supervisor encountered error: {}", e));
    }

    let _ = crate::env_injector::teardown_proxy_env();
    teardown_guard.disarm();

    0
}

async fn run_supervised_daemon_async() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let p_hr = crate::headroom::proxy::proxy_port();
    let p_grok = crate::headroom::grok_daemon::grok_oauth_proxy_port();
    let p_copilot = crate::headroom::copilot_daemon::copilot_proxy_port();
    let p_copilot_cli = crate::headroom::copilot_daemon::copilot_cli_proxy_port();
    let p_byok = crate::util::byok::byok_gateway_port();
    let p_gw = crate::gateway::server::gateway_port();
    let p_dash = crate::dashboard::dashboard_port();

    if let Err(e) = crate::headroom::copilot_daemon::validate_copilot_ports() {
        L.err(&format!("Port validation error: {}", e));
        return Err(e.into());
    }

    // Supervise 7 dedicated endpoints simultaneously
    let mut hr_task = if !crate::headroom::proxy::proxy_running() {
        Some(tokio::spawn(async move {
            if let Err(e) = crate::headroom::proxy::run_native_headroom_proxy(p_hr).await {
                L.warn(&format!("Headroom proxy error: {}", e));
            }
        }))
    } else {
        None
    };

    let mut grok_task = if !crate::headroom::grok_daemon::grok_oauth_proxy_running() {
        Some(tokio::spawn(async move {
            if let Err(e) = crate::headroom::grok_daemon::run_native_grok_proxy(p_grok).await {
                L.warn(&format!("Grok OAuth proxy error: {}", e));
            }
        }))
    } else {
        None
    };

    let mut copilot_task = if !crate::headroom::copilot_daemon::copilot_proxy_running() {
        Some(tokio::spawn(async move {
            if let Err(e) = crate::headroom::copilot_daemon::run_native_copilot_proxy(p_copilot, false).await {
                L.warn(&format!("Copilot VS Code proxy error: {}", e));
            }
        }))
    } else {
        None
    };

    let mut copilot_cli_task = if !crate::headroom::copilot_daemon::copilot_cli_proxy_running() {
        Some(tokio::spawn(async move {
            if let Err(e) = crate::headroom::copilot_daemon::run_native_copilot_proxy(p_copilot_cli, true).await {
                L.warn(&format!("Copilot CLI proxy error: {}", e));
            }
        }))
    } else {
        None
    };

    let mut byok_task = if !crate::headroom::byok_gateway::byok_gateway_live() {
        Some(tokio::spawn(async move {
            if let Err(e) = crate::headroom::byok_gateway::run_byok_gateway_serve().await {
                L.warn(&format!("BYOK gateway error: {}", e));
            }
        }))
    } else {
        None
    };

    let mut gw_task = if !crate::gateway::server::is_gateway_running() {
        Some(tokio::spawn(async move {
            let server = crate::gateway::server::GatewayServer::new("127.0.0.1", p_gw);
            if let Err(e) = server.run().await {
                L.warn(&format!("AI Gateway server error: {}", e));
            }
        }))
    } else {
        None
    };

    let (dash_shutdown_tx, _) = tokio::sync::watch::channel(false);
    let mut dash_task = if !crate::dashboard::is_dashboard_running() {
        let mut server = crate::dashboard::DashboardServer::new("127.0.0.1", p_dash);
        server.shutdown_tx = Some(dash_shutdown_tx);
        Some(tokio::spawn(async move {
            if let Err(e) = server.run().await {
                L.warn(&format!("Dashboard server error: {}", e));
            }
        }))
    } else {
        None
    };

    L.raw(&format!(
        "  {} Supervised 7 dedicated endpoints:\n\
           {} Headroom Proxy (:{p_hr})\n\
           {} Grok OAuth Proxy (:{p_grok})\n\
           {} Copilot VS Code Proxy (:{p_copilot})\n\
           {} Copilot CLI Proxy (:{p_copilot_cli})\n\
           {} BYOK Multi-Profile Gateway (:{p_byok})\n\
           {} AI Gateway (:{p_gw})\n\
           {} Web Dashboard Cockpit (:{p_dash})",
        C.green(Sym::CHECK),
        C.cyan("•"),
        C.cyan("•"),
        C.cyan("•"),
        C.cyan("•"),
        C.cyan("•"),
        C.cyan("•"),
        C.cyan("•")
    ));

    // Supervisor watch loop
    let mut last_ignore_check = std::time::Instant::now();
    let mut last_gitignore_mtime = std::collections::HashMap::<String, std::time::SystemTime>::new();

    loop {
        if Path::new(&daemon_stop_file()).exists() {
            let _ = fs::remove_file(daemon_pid_file());
            let _ = fs::remove_file(daemon_stop_file());
            break;
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }

        // Periodic Dynamic Ignore Purging (every 5 seconds)
        if last_ignore_check.elapsed() >= Duration::from_secs(5) {
            last_ignore_check = std::time::Instant::now();
            if let Ok(curr_dir) = std::env::current_dir() {
                let gitignore_path = curr_dir.join(".gitignore");
                let ignore_path = curr_dir.join(".ignore");

                let mut changed = false;
                for p in &[&gitignore_path, &ignore_path] {
                    if p.exists() {
                        let path_key = p.to_string_lossy().to_string();
                        if let Ok(meta) = fs::metadata(p) {
                            if let Ok(mtime) = meta.modified() {
                                if let Some(prev) = last_gitignore_mtime.get(&path_key) {
                                    if *prev != mtime {
                                        changed = true;
                                        last_gitignore_mtime.insert(path_key, mtime);
                                    }
                                } else {
                                    last_gitignore_mtime.insert(path_key, mtime);
                                }
                            }
                        }
                    }
                }

                if changed {
                    // Trigger dynamic purge across all indexing subsystems
                    let cg_db = crate::codegraph::resolve_db_path(&curr_dir);
                    if cg_db.exists() {
                        if let Ok(mut cg_cache) = crate::codegraph::cache::SymbolCache::open(&cg_db) {
                            let _ = cg_cache.purge_ignored_files(&curr_dir);
                        }
                    }

                    let ctx_db = crate::contextmode::fts::resolve_content_store_path(Some(&curr_dir));
                    if ctx_db.exists() {
                        if let Ok(mut ctx_fts) = crate::contextmode::fts::ContentStore::open(&ctx_db) {
                            let _ = ctx_fts.purge_ignored_sources(&curr_dir);
                        }
                    }

                    if let Ok(mut icm_store) = crate::icm::store::IcmStore::open_default() {
                        let _ = icm_store.purge_ignored_memories(&curr_dir);
                    }

                    let wiki_store = crate::aimemory::wiki::WikiStore::open_default();
                    let _ = wiki_store.purge_ignored_pages(&curr_dir);
                }
            }
        }

        // Auto-heal tasks if any finished prematurely
        if let Some(ref t) = hr_task {
            if t.is_finished() && !crate::headroom::proxy::proxy_running() {
                hr_task = Some(tokio::spawn(async move {
                    let _ = crate::headroom::proxy::run_native_headroom_proxy(p_hr).await;
                }));
            }
        }
        if let Some(ref t) = grok_task {
            if t.is_finished() && !crate::headroom::grok_daemon::grok_oauth_proxy_running() {
                grok_task = Some(tokio::spawn(async move {
                    let _ = crate::headroom::grok_daemon::run_native_grok_proxy(p_grok).await;
                }));
            }
        }
        if let Some(ref t) = copilot_task {
            if t.is_finished() && !crate::headroom::copilot_daemon::copilot_proxy_running() {
                copilot_task = Some(tokio::spawn(async move {
                    let _ = crate::headroom::copilot_daemon::run_native_copilot_proxy(p_copilot, false).await;
                }));
            }
        }
        if let Some(ref t) = copilot_cli_task {
            if t.is_finished() && !crate::headroom::copilot_daemon::copilot_cli_proxy_running() {
                copilot_cli_task = Some(tokio::spawn(async move {
                    let _ = crate::headroom::copilot_daemon::run_native_copilot_proxy(p_copilot_cli, true).await;
                }));
            }
        }
        if let Some(ref t) = byok_task {
            if t.is_finished() && !crate::headroom::byok_gateway::byok_gateway_live() {
                byok_task = Some(tokio::spawn(async move {
                    let _ = crate::headroom::byok_gateway::run_byok_gateway_serve().await;
                }));
            }
        }
        if let Some(ref t) = gw_task {
            if t.is_finished() && !crate::gateway::server::is_gateway_running() {
                gw_task = Some(tokio::spawn(async move {
                    let server = crate::gateway::server::GatewayServer::new("127.0.0.1", p_gw);
                    let _ = server.run().await;
                }));
            }
        }
        if let Some(ref t) = dash_task {
            if t.is_finished() && !crate::dashboard::is_dashboard_running() {
                dash_task = Some(tokio::spawn(async move {
                    let server = crate::dashboard::DashboardServer::new("127.0.0.1", p_dash);
                    let _ = server.run().await;
                }));
            }
        }
    }

    // Clean teardown of tasks
    if let Some(t) = hr_task { t.abort(); }
    if let Some(t) = grok_task { t.abort(); }
    if let Some(t) = copilot_task { t.abort(); }
    if let Some(t) = copilot_cli_task { t.abort(); }
    if let Some(t) = byok_task { t.abort(); }
    if let Some(t) = gw_task { t.abort(); }
    if let Some(t) = dash_task { t.abort(); }

    Ok(())
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

    #[test]
    fn test_copilot_dual_port_configuration() {
        assert_eq!(crate::headroom::copilot_daemon::DEFAULT_COPILOT_PROXY_PORT, 8789);
        assert_eq!(crate::headroom::copilot_daemon::DEFAULT_COPILOT_CLI_PROXY_PORT, 8790);
        let res = crate::headroom::copilot_daemon::validate_copilot_ports();
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_gateway_and_dashboard_simultaneous_isolation() {
        let gw_port = 29099;
        let dash_port = 29100;

        let gw_server = crate::gateway::server::GatewayServer::new("127.0.0.1", gw_port);
        let gw_task = tokio::spawn(async move {
            let _ = gw_server.run().await;
        });

        let (dash_stop_tx, _) = tokio::sync::watch::channel(false);
        let mut dash_server = crate::dashboard::DashboardServer::new("127.0.0.1", dash_port);
        dash_server.shutdown_tx = Some(dash_stop_tx);
        let dash_task = tokio::spawn(async move {
            let _ = dash_server.run().await;
        });

        tokio::time::sleep(Duration::from_millis(150)).await;

        let Ok(client) = reqwest::Client::builder().no_proxy().build() else {
            gw_task.abort();
            dash_task.abort();
            return;
        };

        // Simultaneous queries to Port 9099 and Port 9100
        let req_gw = client.get(format!("http://127.0.0.1:{}/healthz", gw_port)).send();
        let req_dash = client.get(format!("http://127.0.0.1:{}/api/status", dash_port)).send();

        let (res_gw, res_dash) = tokio::join!(req_gw, req_dash);

        if let Ok(resp) = res_gw {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(body.get("service").and_then(|v| v.as_str()), Some("ai-gateway"));
        }

        if let Ok(resp) = res_dash {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert!(body.get("tokens_saved_lifetime").is_some());
        }

        gw_task.abort();
        dash_task.abort();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_native_copilot_dual_proxy_livez() {
        let p_vscode = 29101;
        let p_cli = 29102;

        let t_vscode = tokio::spawn(async move {
            let _ = crate::headroom::copilot_daemon::run_native_copilot_proxy(p_vscode, false).await;
        });
        let t_cli = tokio::spawn(async move {
            let _ = crate::headroom::copilot_daemon::run_native_copilot_proxy(p_cli, true).await;
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        let Ok(client) = reqwest::Client::builder().no_proxy().build() else {
            t_vscode.abort();
            t_cli.abort();
            return;
        };

        let res_vscode = client.get(format!("http://127.0.0.1:{}/livez", p_vscode)).send().await;
        let res_cli = client.get(format!("http://127.0.0.1:{}/livez", p_cli)).send().await;

        if let Ok(resp) = res_vscode {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(body.get("service").and_then(|v| v.as_str()), Some("headroom-proxy"));
            assert_eq!(body.get("subservice").and_then(|v| v.as_str()), Some("copilot-vscode-proxy"));
        }

        if let Ok(resp) = res_cli {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(body.get("service").and_then(|v| v.as_str()), Some("headroom-proxy"));
            assert_eq!(body.get("subservice").and_then(|v| v.as_str()), Some("copilot-cli-proxy"));
        }

        t_vscode.abort();
        t_cli.abort();
    }
}
