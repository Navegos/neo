// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/daemon.rs
// created: 2026-10-27
// lastModified: 2026-11-16

#![deny(unsafe_code)]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use thiserror::Error;

use crate::commands::doctor_stats::process_alive;
use crate::headroom::proxy::proxy_running;
use crate::util::colors::*;
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::proxy_routing_enabled;
use crate::util::logger::L;
use crate::util::paths::{ensure_dir, read_file_safe, toksi_abs_strict, which, write_file_atomic};

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("Daemon configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("Port validation error: {0}")]
    PortValidation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonServicePlan {
    pub selected_agents: Vec<String>,
    pub explicit_agents: bool,
    pub headroom_proxy: bool,
    pub grok_oauth_proxy: bool,
    pub copilot_vscode_proxy: bool,
    pub copilot_cli_proxy: bool,
    pub protocol_bus: bool,
    pub byok_gateway: bool,
    pub ai_gateway: bool,
    pub dashboard: bool,
}

fn parse_agents_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_daemon_args(argv: &[String]) -> (String, Option<Vec<String>>, bool) {
    let mut subcmd = String::new();
    let mut agents: Option<Vec<String>> = None;
    let mut help = false;
    let mut i = 0;
    while i < argv.len() {
        let a = &argv[i];
        if a == "--help" || a == "-h" || a == "help" {
            help = true;
        } else if (a == "--agents" || a == "--agent") && i + 1 < argv.len() {
            let parsed = parse_agents_csv(&argv[i + 1]);
            agents = Some(parsed);
            i += 1;
        } else if let Some(raw) = a
            .strip_prefix("--agents=")
            .or_else(|| a.strip_prefix("--agent="))
        {
            agents = Some(parse_agents_csv(raw));
        } else if subcmd.is_empty() && !a.starts_with('-') {
            subcmd = a.clone();
        }
        i += 1;
    }
    if subcmd.is_empty() {
        subcmd = "status".to_string();
    }
    (subcmd, agents, help)
}

pub fn resolve_daemon_plan(explicit: Option<&[String]>) -> DaemonServicePlan {
    let (agents, has_explicit) = match explicit {
        Some(list) if !list.is_empty() => (
            list.iter()
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>(),
            true,
        ),
        _ => {
            let cfg_agents = crate::config::resolve_default_agents();
            if cfg_agents.is_empty() {
                (Vec::new(), false)
            } else {
                (cfg_agents, true)
            }
        }
    };

    if has_explicit {
        let all = agents.iter().any(|a| a == "all" || a == "*");
        let has_grok = all || agents.iter().any(|a| matches!(a.as_str(), "grok" | "grok-cli" | "grok-ide"));
        let has_copilot_vscode = all
            || agents
                .iter()
                .any(|a| matches!(a.as_str(), "copilot" | "copilot-vscode" | "copilot-ide"));
        let has_copilot_cli = all
            || agents
                .iter()
                .any(|a| matches!(a.as_str(), "copilot" | "copilot-cli"));
        let has_byok = all
            || agents.iter().any(|a| a == "byok")
            || crate::headroom::byok_gateway::byok_routes_configured();
        let has_headroom = all
            || agents.iter().any(|a| {
                !matches!(
                    a.as_str(),
                    "grok" | "grok-cli" | "grok-ide" | "copilot" | "copilot-vscode" | "copilot-ide" | "copilot-cli"
                )
            });

        return DaemonServicePlan {
            selected_agents: agents,
            explicit_agents: true,
            headroom_proxy: has_headroom || proxy_routing_enabled(),
            grok_oauth_proxy: has_grok,
            copilot_vscode_proxy: has_copilot_vscode,
            copilot_cli_proxy: has_copilot_cli,
            protocol_bus: true,
            byok_gateway: has_byok,
            ai_gateway: true,
            dashboard: false,
        };
    }

    let grok_configured = crate::agents::grok::grok_proxy_applicable();
    let copilot_vscode_configured = crate::agents::copilot::copilot_vscode_proxy_wired()
        || crate::agents::proxy_provider::is_vscode_extension_installed("github.copilot-chat");
    let copilot_cli_configured =
        crate::agents::copilot::copilot_proxy_wired() || !which("copilot").is_empty();
    let byok_configured = crate::headroom::byok_gateway::byok_routes_configured();

    DaemonServicePlan {
        selected_agents: Vec::new(),
        explicit_agents: false,
        headroom_proxy: true,
        grok_oauth_proxy: grok_configured,
        copilot_vscode_proxy: copilot_vscode_configured,
        copilot_cli_proxy: copilot_cli_configured,
        protocol_bus: true,
        byok_gateway: byok_configured,
        ai_gateway: true,
        dashboard: false,
    }
}

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

fn daemon_help_text() -> String {
    format!(
        "{} — toksi unified 8-port multi-process supervisor (`toksi _worker <role>`)\n\n\
        {}\n  toksi daemon start [--agents <list>]          Start multi-process supervisor in background (detached cockpit)\n\
          toksi daemon stop                             Gracefully drain & reap all workers and clean env\n\
          toksi daemon status                           Check daemon and 8-port worker health\n\
          toksi daemon restart [--agents <list>]        Restart supervisor and workers\n\
          toksi daemon heal [--agents <list>]           Probe & restart unhealthy/dead workers\n\
          toksi daemon dashboard [--port <p>]           Launch detached Web Cockpit (:9100)\n\
          toksi daemon service <role> <start|stop|restart|heal|status>\n",
        C.bold("toksi daemon"),
        C.bold("Usage:")
    )
}

pub fn daemon_help_text_for_test() -> String {
    daemon_help_text()
}

pub fn plan_worker_roles(plan: &DaemonServicePlan) -> Vec<crate::ipc::WorkerRole> {
    use crate::ipc::WorkerRole;
    let mut roles = Vec::new();
    if plan.headroom_proxy {
        roles.push(WorkerRole::Port8787);
    }
    if plan.grok_oauth_proxy {
        roles.push(WorkerRole::Port8788);
    }
    if plan.copilot_vscode_proxy {
        roles.push(WorkerRole::Port8789);
    }
    if plan.copilot_cli_proxy {
        roles.push(WorkerRole::Port8790);
    }
    if plan.protocol_bus {
        roles.push(WorkerRole::Port9098);
    }
    if plan.ai_gateway {
        roles.push(WorkerRole::Port9099);
    }
    if plan.dashboard {
        roles.push(WorkerRole::Port9100);
    }
    if plan.byok_gateway {
        roles.push(WorkerRole::Port18787);
    }
    roles.push(WorkerRole::Siagent);
    roles.extend_from_slice(WorkerRole::TOOL_WORKERS);
    roles
}

pub fn spawn_worker_subprocess(
    role: crate::ipc::WorkerRole,
    port_override: Option<u16>,
    pipe_override: Option<&str>,
) -> std::io::Result<std::process::Child> {
    let exe = toksi_abs_strict();
    let exe_path = if exe.is_empty() {
        std::env::current_exe()?
    } else {
        std::path::PathBuf::from(exe)
    };
    let endpoint = pipe_override
        .map(ToString::to_string)
        .unwrap_or_else(|| role.ipc_endpoint());
    let mut cmd = Command::new(&exe_path);
    cmd.args(["_worker", role.as_str(), "--pipe", &endpoint]);
    if let Some(p) = port_override {
        cmd.args(["--port", &p.to_string()]);
    }
    crate::env_injector::apply_env_to_command(&mut cmd);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn()
}

pub fn daemon_service_action(role_name: &str, action: &str) -> Result<serde_json::Value, String> {
    use crate::ipc::{IpcFrame, IpcOpcode, WorkerRole};
    let role = role_name.parse::<WorkerRole>()?;
    let endpoint = role.ipc_endpoint();
    let pid_file = crate::ipc::supervisor_pids_dir().join(format!("{}.pid", role.as_str()));

    let worker_alive = || -> bool {
        let frame = IpcFrame::new(IpcOpcode::Ping, 1, Vec::new());
        if crate::ipc::client::send_frame_blocking(&endpoint, &frame, Duration::from_millis(250)).is_ok() {
            return true;
        }
        fs::read_to_string(&pid_file)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .is_some_and(process_alive)
    };

    let act = action.trim().to_ascii_lowercase();
    match act.as_str() {
        "status" => Ok(serde_json::json!({
            "role": role.as_str(),
            "endpoint": endpoint,
            "alive": worker_alive(),
        })),
        "stop" => {
            let drain = IpcFrame::new(IpcOpcode::GracefulDrain, 0, Vec::new());
            let _ = crate::ipc::client::send_frame_blocking(&endpoint, &drain, Duration::from_millis(150));
            if let Ok(raw) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = raw.trim().parse::<u32>() {
                    let _ = crate::headroom::process_identity::kill_process(pid);
                }
                let _ = fs::remove_file(&pid_file);
            }
            if let Ok(db) = crate::db::open_default_db() {
                let _ = db.mark_worker_stopped(role.as_str());
            }
            Ok(serde_json::json!({
                "role": role.as_str(),
                "action": "stop",
                "stopped": true
            }))
        }
        "start" | "restart" | "heal" => {
            if act == "heal" && worker_alive() {
                return Ok(serde_json::json!({
                    "role": role.as_str(),
                    "action": "heal",
                    "restarted": false,
                    "alive": true
                }));
            }
            if act == "restart" || act == "heal" {
                let _ = fs::read_to_string(&pid_file)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .map(crate::headroom::process_identity::kill_process);
            }
            let _ = ensure_dir(crate::ipc::supervisor_pids_dir());
            let child = spawn_worker_subprocess(role, None, Some(&endpoint))
                .map_err(|e| format!("Failed spawning worker '{}': {e}", role.as_str()))?;
            let cpid = child.id();
            let _ = write_file_atomic(&pid_file, &cpid.to_string());
            if let Ok(mut db) = crate::db::open_default_db() {
                let _ = db.upsert_supervisor_worker(&crate::db::SupervisorWorkerRecord {
                    worker_role: role.as_str().to_string(),
                    pid: Some(i64::from(cpid)),
                    ipc_endpoint: endpoint.clone(),
                    status: "running".to_string(),
                    restart_count: if act == "start" { 0 } else { 1 },
                    last_heartbeat: chrono::Utc::now().timestamp(),
                });
            }
            std::mem::forget(child);
            Ok(serde_json::json!({
                "role": role.as_str(),
                "action": act,
                "pid": cpid,
                "restarted": true
            }))
        }
        other => Err(format!("Unsupported service action '{other}' (expected start|stop|restart|heal|status)")),
    }
}

fn daemon_heal(explicit_agents: Option<&[String]>) -> i32 {
    let plan = resolve_daemon_plan(explicit_agents);
    let mut healed = 0usize;
    let mut healthy = 0usize;
    for role in plan_worker_roles(&plan) {
        match daemon_service_action(role.as_str(), "heal") {
            Ok(val) => {
                if val.get("restarted").and_then(|v| v.as_bool()) == Some(true) {
                    healed += 1;
                } else {
                    healthy += 1;
                }
            }
            Err(e) => L.warn(&format!("heal '{}': {}", role.as_str(), e)),
        }
    }
    println!("Daemon heal complete: healthy={} restarted={}", healthy, healed);
    0
}

pub fn run_daemon_cli(argv: &[String]) -> i32 {
    let (subcmd, agents_opt, help) = parse_daemon_args(argv);
    if help {
        println!("{}", daemon_help_text());
        return 0;
    }
    match subcmd.as_str() {
        "start" => daemon_start(agents_opt.as_deref()),
        "stop" => daemon_stop(),
        "status" => daemon_status(),
        "heal" => daemon_heal(agents_opt.as_deref()),
        "dashboard" => crate::dashboard::run_dashboard_cli(argv.get(1..).unwrap_or(&[])),
        "service" => {
            let pos: Vec<&String> = argv.iter().skip(1).filter(|a| !a.starts_with('-')).collect();
            let (Some(name), Some(action)) = (pos.first(), pos.get(1)) else {
                eprintln!("Usage: toksi daemon service <role> <start|stop|restart|heal|status>");
                return 1;
            };
            match daemon_service_action(name, action) {
                Ok(v) => {
                    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
                    0
                }
                Err(e) => {
                    eprintln!("service error: {e}");
                    1
                }
            }
        }
        "run" | "__run-foreground" => daemon_run_foreground(agents_opt.as_deref()),
        "restart" => {
            daemon_stop();
            sleep(Duration::from_millis(500));
            daemon_start(agents_opt.as_deref())
        }
        _ => {
            println!("{}", daemon_help_text());
            1
        }
    }
}

fn daemon_status() -> i32 {
    let ports = crate::config::resolved_ports();
    let running = is_daemon_running();
    let hr_running = proxy_running();
    let grok_running = crate::headroom::grok_daemon::grok_oauth_proxy_running();
    let copilot_running = crate::headroom::copilot_daemon::copilot_proxy_running();
    let copilot_cli_running = crate::headroom::copilot_daemon::copilot_cli_proxy_running();
    let bus_running = crate::tools::multi_protocol::is_protocol_bus_running();
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
        "  {} Unified Protocol Bus ({}): {}",
        if bus_running { C.green(Sym::CHECK) } else { C.gray("•") },
        ports.protocol_bus,
        if bus_running { "running" } else { "stopped" }
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

fn daemon_start(explicit_agents: Option<&[String]>) -> i32 {
    let ports = crate::config::resolved_ports();
    if let Err(e) = ports.validate() {
        L.err(&format!("Cannot start daemon: {}", e));
        return 1;
    }

    if is_daemon_running() {
        let _ = crate::env_injector::inject_proxy_env_for_ports(&ports);
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

    // Rule of Test and Inform User: verify toksi executable in PATH before spawning
    if let Err(msg) = crate::util::env_path::check_executable_in_path("toksi") {
        L.warn(&msg);
    }

    let _ = fs::remove_file(daemon_stop_file());
    let exe = toksi_abs_strict();
    if exe.is_empty() {
        L.err("Cannot start daemon: toksi binary path unknown");
        return 1;
    }

    // Non-destructive environment injection so environment is ready before IDEs launch
    if let Err(e) = crate::env_injector::inject_proxy_env_for_ports(&ports) {
        L.warn(&format!("Environment injection warning: {}", e));
    }

    let mut cmd = Command::new(&exe);
    cmd.args(["daemon", "__run-foreground"]);
    if let Some(list) = explicit_agents
        && !list.is_empty()
    {
        cmd.args(["--agents", &list.join(",")]);
    }
    crate::env_injector::apply_env_to_command(&mut cmd);
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
            let _ = crate::env_injector::teardown_proxy_env();
            L.err(&format!("Failed spawning daemon: {}", e));
            1
        }
    }
}

fn daemon_stop() -> i32 {
    // Purge all Toksi-owned environment variables and restore snapshots upon shutdown
    if let Err(e) = crate::env_injector::teardown_proxy_env()
        && !e.is_access_denied()
    {
        L.warn(&format!("Environment teardown warning: {}", e));
    }

    // Auto-heal any lingering hijacked binaries, shims, symlinks, or orphaned state files
    let report = crate::env_injector::heal_legacy_hijacks();
    if !report.is_clean() {
        L.raw(&format!(
            "  {} Healed {} hijacked binary/shim artifact(s) on daemon stop",
            C.green(Sym::CHECK),
            report.total_healed()
        ));
    }

    // Verify executable presence in PATH
    if crate::util::env_path::check_executable_in_path("toksi").is_ok() {
        L.debug("Verified toksi binary in PATH during daemon stop");
    }

    if !is_daemon_running() {
        L.raw(&format!("  {} Toksi daemon is not running", C.dim("•")));
        return 0;
    }

    let _ = write_file_atomic(daemon_stop_file(), "stop");
    let pid_str = read_file_safe(daemon_pid_file()).unwrap_or_default().trim().to_string();
    if let Ok(pid) = pid_str.parse::<u32>() {
        for _ in 0..10 {
            if !process_alive(pid) {
                break;
            }
            sleep(Duration::from_millis(100));
        }
        if process_alive(pid) {
            let _ = crate::headroom::process_identity::kill_process(pid);
        }
    }

    for role in crate::ipc::WorkerRole::ALL {
        let drain = crate::ipc::IpcFrame::new(crate::ipc::IpcOpcode::GracefulDrain, 0, Vec::new());
        let ep = role.ipc_endpoint();
        let _ = crate::ipc::client::send_frame_blocking(&ep, &drain, Duration::from_millis(100));
        let wpid_path = crate::ipc::supervisor_pids_dir().join(format!("{}.pid", role.as_str()));
        if let Ok(raw) = fs::read_to_string(&wpid_path) {
            if let Ok(wpid) = raw.trim().parse::<u32>() {
                let _ = crate::headroom::process_identity::kill_process(wpid);
            }
            let _ = fs::remove_file(wpid_path);
        }
    }

    let _ = fs::remove_file(daemon_pid_file());
    let _ = fs::remove_file(daemon_stop_file());

    L.raw(&format!("  {} Toksi daemon supervisor stopped", C.green(Sym::CHECK)));
    0
}

fn daemon_run_foreground(explicit_agents: Option<&[String]>) -> i32 {
    let pid = std::process::id();
    let _ = ensure_dir(&headroom_paths_resolved().root);
    if let Err(e) = write_file_atomic(daemon_pid_file(), &pid.to_string()) {
        L.err(&format!("Failed writing daemon.pid: {}", e));
        return 1;
    }

    let plan = resolve_daemon_plan(explicit_agents);
    let ports = crate::config::resolved_ports();

    // Register teardown handlers (SIGINT, SIGTERM, Ctrl+C) and RAII guard
    crate::env_injector::register_teardown_handlers();
    let mut teardown_guard = crate::env_injector::TeardownGuard::new();
    let _ = crate::env_injector::inject_proxy_env_for_ports(&ports);

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .thread_name("toksi-daemon-supervisor")
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            L.err(&format!("Failed to build Tokio runtime: {}", e));
            let _ = fs::remove_file(daemon_pid_file());
            let _ = crate::env_injector::teardown_proxy_env();
            teardown_guard.disarm();
            return 1;
        }
    };

    let run_res = rt.block_on(run_supervised_daemon_async(plan));
    if let Err(e) = run_res {
        L.err(&format!("Daemon supervisor encountered error: {}", e));
    }

    for role in crate::ipc::WorkerRole::ALL {
        let drain = crate::ipc::IpcFrame::new(crate::ipc::IpcOpcode::GracefulDrain, 0, Vec::new());
        let ep = role.ipc_endpoint();
        let _ = crate::ipc::client::send_frame_blocking(&ep, &drain, Duration::from_millis(100));
        let wpid_path = crate::ipc::supervisor_pids_dir().join(format!("{}.pid", role.as_str()));
        if let Ok(raw) = fs::read_to_string(&wpid_path) {
            if let Ok(wpid) = raw.trim().parse::<u32>() {
                let _ = crate::headroom::process_identity::kill_process(wpid);
            }
            let _ = fs::remove_file(wpid_path);
        }
    }

    let _ = fs::remove_file(daemon_pid_file());
    let _ = fs::remove_file(daemon_stop_file());
    let _ = crate::env_injector::teardown_proxy_env();
    teardown_guard.disarm();

    0
}

async fn run_supervised_daemon_async(
    plan: DaemonServicePlan,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::ipc::{IpcFrame, IpcOpcode, WorkerRole};

    crate::env_injector::register_teardown_handlers();

    let ports = crate::config::resolved_ports();
    ports.validate()?;
    if let Err(e) = crate::headroom::copilot_daemon::validate_copilot_ports() {
        L.err(&format!("Port validation error: {}", e));
        return Err(e.into());
    }

    let p_hr = ports.headroom_proxy;
    let p_grok = ports.grok_oauth_proxy;
    let p_copilot = ports.copilot_vscode_proxy;
    let p_copilot_cli = ports.copilot_cli_proxy;
    let p_bus = ports.protocol_bus;
    let p_byok = ports.byok_gateway;
    let p_gw = ports.ai_gateway;
    let p_dash = ports.dashboard;

    let port_for_role = |role: WorkerRole| -> Option<u16> {
        match role {
            WorkerRole::Port8787 => Some(p_hr),
            WorkerRole::Port8788 => Some(p_grok),
            WorkerRole::Port8789 => Some(p_copilot),
            WorkerRole::Port8790 => Some(p_copilot_cli),
            WorkerRole::Port9098 => Some(p_bus),
            WorkerRole::Port9099 => Some(p_gw),
            WorkerRole::Port9100 => Some(p_dash),
            WorkerRole::Port18787 => Some(p_byok),
            _ => None,
        }
    };

    struct ManagedWorker {
        role: WorkerRole,
        port: Option<u16>,
        endpoint: String,
        child: Option<std::process::Child>,
        restart_count: u32,
        window_started: std::time::Instant,
    }

    let cfg = crate::config::load_toksi_toml_config();
    let max_restarts = cfg.supervisor.max_restarts_per_min;
    let base_backoff_ms = cfg.supervisor.restart_backoff_ms.max(50);

    let _ = ensure_dir(crate::ipc::supervisor_pids_dir());
    let mut workers = Vec::new();
    for role in plan_worker_roles(&plan) {
        let port = port_for_role(role);
        let endpoint = role.ipc_endpoint();
        let child = match spawn_worker_subprocess(role, port, Some(&endpoint)) {
            Ok(c) => {
                let cpid = c.id();
                if let Ok(mut db) = crate::db::open_default_db() {
                    let _ = db.upsert_supervisor_worker(&crate::db::SupervisorWorkerRecord {
                        worker_role: role.as_str().to_string(),
                        pid: Some(i64::from(cpid)),
                        ipc_endpoint: endpoint.clone(),
                        status: "running".to_string(),
                        restart_count: 0,
                        last_heartbeat: chrono::Utc::now().timestamp(),
                    });
                }
                Some(c)
            }
            Err(e) => {
                L.warn(&format!("Failed spawning worker subprocess '{}': {}", role.as_str(), e));
                None
            }
        };
        workers.push(ManagedWorker {
            role,
            port,
            endpoint,
            child,
            restart_count: 0,
            window_started: std::time::Instant::now(),
        });
    }

    let reconnect_watchdog =
        crate::gateway::reconnect::start_global_watchdog(Duration::from_secs(10));

    let fmt_state = |enabled: bool| if enabled { "active" } else { "skipped (unconfigured)" };
    L.raw(&format!(
        "  {} Supervised 8-port multi-process control plane ({} worker processes):\n\
           {} Headroom Proxy (:{p_hr}) — {}\n\
           {} Grok OAuth Proxy (:{p_grok}) — {}\n\
           {} Copilot VS Code Proxy (:{p_copilot}) — {}\n\
           {} Copilot CLI Proxy (:{p_copilot_cli}) — {}\n\
           {} Unified Protocol Bus (:{p_bus}) — {}\n\
           {} BYOK Multi-Profile Gateway (:{p_byok}) — {}\n\
           {} AI Gateway (:{p_gw}) — {}\n\
           {} Web Dashboard Cockpit (:{p_dash}) — {}",
        C.green(Sym::CHECK),
        workers.len(),
        C.cyan("•"),
        fmt_state(plan.headroom_proxy),
        C.cyan("•"),
        fmt_state(plan.grok_oauth_proxy),
        C.cyan("•"),
        fmt_state(plan.copilot_vscode_proxy),
        C.cyan("•"),
        fmt_state(plan.copilot_cli_proxy),
        C.cyan("•"),
        fmt_state(plan.protocol_bus),
        C.cyan("•"),
        fmt_state(plan.byok_gateway),
        C.cyan("•"),
        fmt_state(plan.ai_gateway),
        C.cyan("•"),
        fmt_state(plan.dashboard),
    ));

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
            sig = crate::env_injector::wait_for_shutdown_signal() => {
                L.debug(&format!("Daemon received shutdown signal: {}", sig));
                break;
            }
        }

        // Periodic Dynamic Ignore Purging (every 5 seconds) offloaded from async reactor
        if last_ignore_check.elapsed() >= Duration::from_secs(5) {
            last_ignore_check = std::time::Instant::now();
            if let Ok(curr_dir) = std::env::current_dir() {
                let gitignore_path = curr_dir.join(".gitignore");
                let ignore_path = curr_dir.join(".ignore");

                let mut changed = false;
                for p in [&gitignore_path, &ignore_path] {
                    if p.exists()
                        && let Ok(meta) = fs::metadata(p)
                        && let Ok(mtime) = meta.modified()
                    {
                        let path_key = p.to_string_lossy().to_string();
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

                if changed {
                    let _ = tokio::task::spawn_blocking(move || {
                        let cg_db = crate::codegraph::resolve_db_path(&curr_dir);
                        if cg_db.exists()
                            && let Ok(mut cg_cache) = crate::codegraph::cache::SymbolCache::open(&cg_db)
                        {
                            let _ = cg_cache.purge_ignored_files(&curr_dir);
                        }

                        let ctx_db =
                            crate::contextmode::fts::resolve_content_store_path(Some(&curr_dir));
                        if ctx_db.exists()
                            && let Ok(mut ctx_fts) = crate::contextmode::fts::ContentStore::open(&ctx_db)
                        {
                            let _ = ctx_fts.purge_ignored_sources(&curr_dir);
                        }

                        if let Ok(mut icm_store) = crate::icm::store::IcmStore::open_default() {
                            let _ = icm_store.purge_ignored_memories(&curr_dir);
                        }

                        let wiki_store = crate::aimemory::wiki::WikiStore::open_default();
                        let _ = wiki_store.purge_ignored_pages(&curr_dir);

                        let _ = crate::gateway::semantic_cache::global_semantic_cache()
                            .purge_ignored_entries(&curr_dir);
                    })
                    .await;
                }
            }
        }

        // Supervise child worker OS processes with bounded exponential backoff restart
        for w in &mut workers {
            let needs_restart = match w.child.as_mut() {
                Some(child) => match child.try_wait() {
                    Ok(Some(_exit)) => true,
                    Ok(None) => false,
                    Err(_) => true,
                },
                None => true,
            };
            if !needs_restart {
                continue;
            }
            if w.window_started.elapsed() >= Duration::from_secs(60) {
                w.restart_count = 0;
                w.window_started = std::time::Instant::now();
            }
            if w.restart_count >= max_restarts {
                continue;
            }
            let shift = w.restart_count.min(4);
            let delay = Duration::from_millis(base_backoff_ms.saturating_mul(1u64 << shift));
            tokio::time::sleep(delay.min(Duration::from_secs(2))).await;
            w.restart_count += 1;
            if let Ok(next_child) = spawn_worker_subprocess(w.role, w.port, Some(&w.endpoint)) {
                let cpid = next_child.id();
                if let Ok(mut db) = crate::db::open_default_db() {
                    let _ = db.upsert_supervisor_worker(&crate::db::SupervisorWorkerRecord {
                        worker_role: w.role.as_str().to_string(),
                        pid: Some(i64::from(cpid)),
                        ipc_endpoint: w.endpoint.clone(),
                        status: "running".to_string(),
                        restart_count: i64::from(w.restart_count),
                        last_heartbeat: chrono::Utc::now().timestamp(),
                    });
                }
                w.child = Some(next_child);
            }
        }
    }

    reconnect_watchdog.abort();
    let drain_frame = IpcFrame::new(IpcOpcode::GracefulDrain, 0, Vec::new());
    for w in &mut workers {
        let _ = crate::ipc::client::send_frame_blocking(&w.endpoint, &drain_frame, Duration::from_millis(120));
        if let Some(mut child) = w.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Ok(db) = crate::db::open_default_db() {
            let _ = db.mark_worker_stopped(w.role.as_str());
        }
        let _ = fs::remove_file(crate::ipc::supervisor_pids_dir().join(format!("{}.pid", w.role.as_str())));
    }

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
    fn test_daemon_agents_plan_skips_unselected_agents() {
        let claude_only = vec!["claude".to_string()];
        let plan = resolve_daemon_plan(Some(&claude_only));
        assert!(plan.headroom_proxy);
        assert!(plan.protocol_bus);
        assert!(plan.ai_gateway);
        assert!(!plan.dashboard);
        assert!(!plan.grok_oauth_proxy);
        assert!(!plan.copilot_vscode_proxy);
        assert!(!plan.copilot_cli_proxy);

        let copilot_and_grok = vec!["copilot".to_string(), "grok".to_string()];
        let plan_cg = resolve_daemon_plan(Some(&copilot_and_grok));
        assert!(plan_cg.grok_oauth_proxy);
        assert!(plan_cg.copilot_vscode_proxy);
        assert!(plan_cg.copilot_cli_proxy);

        let all = vec!["all".to_string()];
        let plan_all = resolve_daemon_plan(Some(&all));
        assert!(plan_all.headroom_proxy);
        assert!(plan_all.grok_oauth_proxy);
        assert!(plan_all.copilot_vscode_proxy);
        assert!(plan_all.copilot_cli_proxy);
        assert!(plan_all.protocol_bus);
        assert!(plan_all.byok_gateway);
        assert!(plan_all.ai_gateway);
        assert!(!plan_all.dashboard);
    }

    #[test]
    fn test_parse_daemon_args() {
        let argv = vec![
            "start".to_string(),
            "--agents".to_string(),
            "claude,copilot".to_string(),
        ];
        let (cmd, agents, help) = parse_daemon_args(&argv);
        assert_eq!(cmd, "start");
        assert_eq!(
            agents,
            Some(vec!["claude".to_string(), "copilot".to_string()])
        );
        assert!(!help);
    }

    #[test]
    fn test_copilot_dual_port_configuration() {
        assert_eq!(crate::headroom::copilot_daemon::DEFAULT_COPILOT_PROXY_PORT, 8789);
        assert_eq!(
            crate::headroom::copilot_daemon::DEFAULT_COPILOT_CLI_PROXY_PORT,
            8790
        );
        let res = crate::headroom::copilot_daemon::validate_copilot_ports();
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_gateway_dashboard_and_protocol_bus_isolated() {
        let bus_port = 29098;
        let gw_port = 29099;
        let dash_port = 29100;

        let bus_task = tokio::spawn(async move {
            let _ = crate::tools::multi_protocol::run_protocol_bus_server(bus_port).await;
        });
        let mut gw_handle =
            match crate::gateway::server::spawn_isolated_gateway("127.0.0.1", gw_port) {
                Ok(h) => h,
                Err(_) => {
                    bus_task.abort();
                    return;
                }
            };
        let mut dash_handle =
            match crate::dashboard::spawn_isolated_dashboard("127.0.0.1", dash_port) {
                Ok(h) => h,
                Err(_) => {
                    gw_handle.shutdown();
                    bus_task.abort();
                    return;
                }
            };

        assert_ne!(gw_handle.thread_name, dash_handle.thread_name);
        tokio::time::sleep(Duration::from_millis(200)).await;

        let Ok(client) = reqwest::Client::builder().no_proxy().build() else {
            gw_handle.shutdown();
            dash_handle.shutdown();
            bus_task.abort();
            return;
        };

        let req_bus = client
            .get(format!("http://127.0.0.1:{}/livez", bus_port))
            .send();
        let req_gw = client
            .get(format!("http://127.0.0.1:{}/healthz", gw_port))
            .send();
        let req_dash = client
            .get(format!("http://127.0.0.1:{}/api/status", dash_port))
            .send();

        let (res_bus, res_gw, res_dash) = tokio::join!(req_bus, req_gw, req_dash);

        if let Ok(resp) = res_bus {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(
                body.get("service").and_then(|v| v.as_str()),
                Some("toksi-protocol-bus")
            );
        }

        if let Ok(resp) = res_gw {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(
                body.get("service").and_then(|v| v.as_str()),
                Some("ai-gateway")
            );
        }

        if let Ok(resp) = res_dash {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert!(body.get("tokens_saved_lifetime").is_some());
        }

        gw_handle.shutdown();
        dash_handle.shutdown();
        bus_task.abort();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_native_copilot_dual_proxy_livez() {
        let p_vscode = 29101;
        let p_cli = 29102;

        let t_vscode = tokio::spawn(async move {
            let _ =
                crate::headroom::copilot_daemon::run_native_copilot_proxy(p_vscode, false).await;
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

        let res_vscode = client
            .get(format!("http://127.0.0.1:{}/livez", p_vscode))
            .send()
            .await;
        let res_cli = client
            .get(format!("http://127.0.0.1:{}/livez", p_cli))
            .send()
            .await;

        if let Ok(resp) = res_vscode {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(
                body.get("service").and_then(|v| v.as_str()),
                Some("headroom-proxy")
            );
            assert_eq!(
                body.get("subservice").and_then(|v| v.as_str()),
                Some("copilot-vscode-proxy")
            );
        }

        if let Ok(resp) = res_cli {
            assert_eq!(resp.status().as_u16(), 200);
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(
                body.get("service").and_then(|v| v.as_str()),
                Some("headroom-proxy")
            );
            assert_eq!(
                body.get("subservice").and_then(|v| v.as_str()),
                Some("copilot-cli-proxy")
            );
        }

        t_vscode.abort();
        t_cli.abort();
    }
}
