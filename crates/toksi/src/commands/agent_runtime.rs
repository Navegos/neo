// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/agent_runtime.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! CLI handlers for `toksi chat`, `toksi run`, `toksi exec`, `toksi swarm`,
//! and `toksi agent chat|run|exec|swarm|resume`.

#![deny(unsafe_code)]

use std::path::PathBuf;
use std::time::Duration;

use crate::agent_runtime::AgentRuntimeEngine;
use crate::util::colors::C;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonConnectionMode {
    DirectWireIpc,
    NetworkGateway,
    Standalone,
}

impl DaemonConnectionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectWireIpc => "direct-wire-ipc",
            Self::NetworkGateway => "network-gateway",
            Self::Standalone => "standalone",
        }
    }
}

/// Probes `WorkerRole::NetworkManager` DirectWire IPC first (150ms timeout),
/// then checks the loopback gateway listener before falling back to standalone.
pub fn detect_daemon_connection() -> DaemonConnectionMode {
    let ep = crate::ipc::WorkerRole::NetworkManager.ipc_endpoint();
    let ping = crate::ipc::IpcFrame::new(crate::ipc::IpcOpcode::Ping, 1, Vec::new());
    if crate::ipc::client::send_frame_blocking(&ep, &ping, Duration::from_millis(150)).is_ok() {
        return DaemonConnectionMode::DirectWireIpc;
    }
    let ports = crate::config::resolved_ports();
    let addr = format!("127.0.0.1:{}", ports.ai_gateway);
    if let Ok(sock) = addr.parse::<std::net::SocketAddr>()
        && std::net::TcpStream::connect_timeout(&sock, Duration::from_millis(100)).is_ok()
    {
        return DaemonConnectionMode::NetworkGateway;
    }
    DaemonConnectionMode::Standalone
}

fn workspace_from_args(args: &[String]) -> (PathBuf, Vec<String>) {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--workspace" || args[i] == "-w") && i + 1 < args.len() {
            root = PathBuf::from(&args[i + 1]);
            i += 2;
            continue;
        }
        if let Some(val) = args[i].strip_prefix("--workspace=") {
            root = PathBuf::from(val);
            i += 1;
            continue;
        }
        if args[i] == "--effort" && i + 1 < args.len() {
            let _ = crate::agent_runtime::global_slash_engine()
                .execute(&format!("/effort {}", args[i + 1]), "toksi-cli");
            i += 2;
            continue;
        }
        if let Some(val) = args[i].strip_prefix("--effort=") {
            let _ = crate::agent_runtime::global_slash_engine()
                .execute(&format!("/effort {}", val), "toksi-cli");
            i += 1;
            continue;
        }
        rest.push(args[i].clone());
        i += 1;
    }
    (root, rest)
}

pub fn run_chat_cli(args: &[String]) -> i32 {
    let (root, rest) = workspace_from_args(args);
    let mut force_plain = false;
    let mut mode_override: Option<crate::agent_runtime::mode_router::OrchestratorMode> = None;
    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--plain" | "--no-tui" => {
                force_plain = true;
                i += 1;
            }
            "--mode" if i + 1 < rest.len() => {
                mode_override = crate::agent_runtime::chat_ui::ChatUiState::resolve_mode_alias(&rest[i + 1]);
                i += 2;
            }
            s if s.starts_with("--mode=") => {
                let val = s.trim_start_matches("--mode=");
                mode_override = crate::agent_runtime::chat_ui::ChatUiState::resolve_mode_alias(val);
                i += 1;
            }
            _ => i += 1,
        }
    }
    let mut engine = AgentRuntimeEngine::new(root, "toksi-cli");
    if let Some(m) = mode_override {
        engine.active_mode = m;
    }
    engine.run_chat_repl_with_options(force_plain)
}

pub fn run_runtime_run_cli(args: &[String]) -> i32 {
    let (root, rest) = workspace_from_args(args);
    let prompt = rest.join(" ").trim().to_string();
    if prompt.is_empty() {
        eprintln!("Usage: toksi run \"<prompt>\" [--workspace <path>]");
        return 1;
    }
    let mut engine = AgentRuntimeEngine::new(root, "toksi-cli");
    match engine.run_single_shot(&prompt) {
        Ok(rep) => {
            println!("{}", rep.output);
            0
        }
        Err(e) => {
            eprintln!("{} {}", C.red("Error:"), e);
            1
        }
    }
}

pub fn run_runtime_exec_cli(args: &[String]) -> i32 {
    let (root, rest) = workspace_from_args(args);
    let cmd = rest.join(" ").trim().to_string();
    if cmd.is_empty() {
        eprintln!("Usage: toksi exec \"<command>\" [--workspace <path>]");
        return 1;
    }
    let mut engine = AgentRuntimeEngine::new(root, "toksi-sandbox");
    match engine.run_sandboxed_exec(&cmd) {
        Ok(rep) => {
            println!("{}", rep.output);
            0
        }
        Err(e) => {
            eprintln!("{} {}", C.red("Sandbox Error:"), e);
            1
        }
    }
}

pub fn run_runtime_swarm_cli(args: &[String]) -> i32 {
    let (root, rest) = workspace_from_args(args);
    let goal = rest.join(" ").trim().to_string();
    if goal.is_empty() {
        eprintln!("Usage: toksi swarm \"<goal>\" [--workspace <path>]");
        return 1;
    }
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Tokio runtime init error: {}", e);
            return 1;
        }
    };
    let mut engine = AgentRuntimeEngine::new(root, "toksi-swarm");
    match rt.block_on(engine.run_swarm(&goal)) {
        Ok(rep) => {
            println!("{}", rep.output);
            0
        }
        Err(e) => {
            eprintln!("{} {}", C.red("Swarm Error:"), e);
            1
        }
    }
}

pub fn run_runtime_resume_cli(args: &[String]) -> i32 {
    let (root, rest) = workspace_from_args(args);
    let plan_id = rest.first().map(|s| s.as_str()).unwrap_or("");
    if plan_id.is_empty() {
        let out = crate::agent_runtime::global_slash_engine()
            .execute("/resume", "toksi-cli")
            .output;
        println!("{}", out);
        return 0;
    }
    let mut engine = AgentRuntimeEngine::new(root, "toksi-cli");
    match engine.resume_plan(plan_id) {
        Ok(msg) => {
            println!("{}", msg);
            0
        }
        Err(e) => {
            eprintln!("{} {}", C.red("Resume Error:"), e);
            1
        }
    }
}
