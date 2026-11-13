// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/antigravity.rs
// created: 2026-11-09

use std::process::{Child, Command, Stdio};
use std::time::Duration;
use crate::util::env_path::check_executable_in_path;

pub fn read_antigravity_session() -> Option<String> {
    crate::gateway::agent_cli::gemini_agy::read_gemini_adc_session()
}

/// Spawns the local Antigravity CLI bridge (agy.exe / agy) in detached background mode.
/// Windows: uses CREATE_NO_WINDOW & DETACHED_PROCESS to prevent console flash.
/// Unix: uses setsid() to run completely detached from the parent terminal.
pub fn launch_antigravity_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "agy.exe" } else { "agy" };
    if let Err(e) = check_executable_in_path(bin_name) {
        return Err(e);
    }

    let mut cmd = Command::new(bin_name);
    cmd.args(["serve", "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn().map_err(|e| format!("Failed to spawn Antigravity CLI bridge: {}", e))
}

/// Health check: probes ConnectRPC / /livez endpoint for reachability.
pub fn verify_antigravity_health(port: u16, timeout: Duration) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let Ok(mut stream) = std::net::TcpStream::connect_timeout(&addr, timeout) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    use std::io::{Read, Write};
    let req = format!("GET /livez HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
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
