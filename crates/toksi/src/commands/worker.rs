// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/worker.rs
// created: 2026-11-24

//! Multi-call child worker entrypoint: `toksi _worker <role> [--port <p>] [--pipe <id>]`.

#![deny(unsafe_code)]

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::watch;

use crate::db::{open_default_db, SupervisorWorkerRecord};
use crate::ipc::server::{serve_worker_ipc, DefaultWorkerIpcHandler};
use crate::ipc::transport::{ipc_endpoint_for_role, supervisor_pids_dir, WorkerRole};
use crate::util::logger::L;
use crate::util::paths::write_file_atomic;

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct WorkerLaunchArgs {
    pub role: WorkerRole,
    pub port_override: Option<u16>,
    pub pipe_override: Option<String>,
    pub ipc_only: bool,
}

pub fn parse_worker_args(args: &[String]) -> Result<WorkerLaunchArgs, String> {
    let raw_role = args
        .first()
        .ok_or_else(|| "Usage: toksi _worker <role> [--port <p>] [--pipe <id>] [--ipc-only]".to_string())?;
    let role: WorkerRole = raw_role.parse()?;
    let mut port_override = None;
    let mut pipe_override = None;
    let mut ipc_only = false;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--port" if i + 1 < args.len() => {
                port_override = args[i + 1].parse::<u16>().ok();
                i += 2;
            }
            "--pipe" if i + 1 < args.len() => {
                pipe_override = Some(args[i + 1].clone());
                i += 2;
            }
            "--ipc-only" => {
                ipc_only = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    Ok(WorkerLaunchArgs {
        role,
        port_override,
        pipe_override,
        ipc_only,
    })
}

pub fn run_worker_cli(args: &[String]) -> i32 {
    let parsed = match parse_worker_args(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };

    let endpoint = parsed
        .pipe_override
        .clone()
        .unwrap_or_else(|| ipc_endpoint_for_role(parsed.role.as_str()));
    let pid = std::process::id();
    let pid_path = supervisor_pids_dir().join(format!("{}.pid", parsed.role.as_str()));
    let _ = write_file_atomic(&pid_path, &pid.to_string());

    if let Ok(mut db) = open_default_db() {
        let _ = db.upsert_supervisor_worker(&SupervisorWorkerRecord {
            worker_role: parsed.role.as_str().to_string(),
            pid: Some(i64::from(pid)),
            ipc_endpoint: endpoint.clone(),
            status: "running".to_string(),
            restart_count: 0,
            last_heartbeat: now_epoch(),
        });
    }

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .thread_name(format!("toksi-worker-{}", parsed.role.as_str()))
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            L.err(&format!("Worker {} failed building Tokio runtime: {e}", parsed.role.as_str()));
            return 1;
        }
    };

    let role = parsed.role;
    let port_opt = parsed.port_override;
    let ep_clone = endpoint.clone();

    let exit_code = rt.block_on(async move {
        let (kill_tx, kill_rx) = watch::channel(false);
        let handler = Arc::new(DefaultWorkerIpcHandler::new(role, kill_tx.clone()));
        let ipc_task = tokio::spawn(async move {
            let _ = serve_worker_ipc(&ep_clone, handler, kill_rx).await;
        });

        let ports = crate::config::resolved_ports();
        let target_port = port_opt.or(match role {
            WorkerRole::Port8787 => Some(ports.headroom_proxy),
            WorkerRole::Port8788 => Some(ports.grok_oauth_proxy),
            WorkerRole::Port8789 => Some(ports.copilot_vscode_proxy),
            WorkerRole::Port8790 => Some(ports.copilot_cli_proxy),
            WorkerRole::Port9098 => Some(ports.protocol_bus),
            WorkerRole::Port9099 => Some(ports.ai_gateway),
            WorkerRole::Port9100 => Some(ports.dashboard),
            WorkerRole::Port18787 => Some(ports.byok_gateway),
            _ => None,
        });

        let mut gw_handle = None;
        let mut dash_handle = None;

        let ipc_only = parsed.ipc_only;
        let service_task = match role {
            WorkerRole::NetworkManager => {
                let nm_rx = kill_tx.subscribe();
                let plan = crate::commands::daemon::resolve_daemon_plan(None);
                Some(tokio::spawn(async move {
                    let _ = crate::ipc::network_manager::run_network_manager(plan, nm_rx).await;
                }))
            }
            WorkerRole::Port8787 if !ipc_only => Some(tokio::spawn(async move {
                let p = target_port.unwrap_or(8787);
                let _ = crate::headroom::proxy::run_native_headroom_proxy(p).await;
            })),
            WorkerRole::Port8788 if !ipc_only => Some(tokio::spawn(async move {
                let p = target_port.unwrap_or(8788);
                let _ = crate::headroom::grok_daemon::run_native_grok_proxy(p).await;
            })),
            WorkerRole::Port8789 if !ipc_only => Some(tokio::spawn(async move {
                let p = target_port.unwrap_or(8789);
                let _ = crate::headroom::copilot_daemon::run_native_copilot_proxy(p, false).await;
            })),
            WorkerRole::Port8790 if !ipc_only => Some(tokio::spawn(async move {
                let p = target_port.unwrap_or(8790);
                let _ = crate::headroom::copilot_daemon::run_native_copilot_proxy(p, true).await;
            })),
            WorkerRole::Port9098 if !ipc_only => Some(tokio::spawn(async move {
                let p = target_port.unwrap_or(9098);
                let _ = crate::tools::multi_protocol::run_protocol_bus_server(p).await;
            })),
            WorkerRole::Port18787 if !ipc_only => Some(tokio::spawn(async move {
                let p = target_port.unwrap_or(18787);
                let _ = crate::headroom::byok_gateway::run_byok_gateway_serve_on_port(p, true).await;
            })),
            WorkerRole::Port9099 if !ipc_only => {
                let p = target_port.unwrap_or(9099);
                if let Ok(h) = crate::gateway::server::spawn_isolated_gateway("127.0.0.1", p) {
                    gw_handle = Some(h);
                }
                None
            }
            WorkerRole::Port9100 => {
                let p = target_port.unwrap_or(9100);
                if let Ok(h) = crate::dashboard::spawn_isolated_dashboard("127.0.0.1", p) {
                    dash_handle = Some(h);
                }
                None
            }
            WorkerRole::Port8787
            | WorkerRole::Port8788
            | WorkerRole::Port8789
            | WorkerRole::Port8790
            | WorkerRole::Port9098
            | WorkerRole::Port9099
            | WorkerRole::Port18787
            | WorkerRole::Siagent
            | WorkerRole::Codegraph
            | WorkerRole::Ctxmode
            | WorkerRole::Projectmem
            | WorkerRole::Icm
            | WorkerRole::Aimemory
            | WorkerRole::Rtk
            | WorkerRole::Security => None,
        };

        let mut shutdown_watch = kill_tx.subscribe();
        loop {
            if *shutdown_watch.borrow() {
                break;
            }
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    if let Ok(db) = open_default_db() {
                        let _ = db.update_worker_heartbeat(role.as_str(), now_epoch());
                    }
                }
                res = shutdown_watch.changed() => {
                    if res.is_err() || *shutdown_watch.borrow() {
                        break;
                    }
                }
                sig = crate::env_injector::wait_for_shutdown_signal() => {
                    L.debug(&format!("Worker {} received shutdown signal: {sig}", role.as_str()));
                    let _ = kill_tx.send(true);
                    break;
                }
            }
        }

        if let Some(t) = service_task {
            t.abort();
        }
        if let Some(mut h) = gw_handle {
            h.shutdown();
        }
        if let Some(mut h) = dash_handle {
            h.shutdown();
        }
        ipc_task.abort();
        0
    });

    let _ = std::fs::remove_file(&pid_path);
    if let Ok(db) = open_default_db() {
        let _ = db.mark_worker_stopped(parsed.role.as_str());
    }
    exit_code
}
