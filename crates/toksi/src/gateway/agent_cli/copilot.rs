// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/copilot.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::process::{Child, Command, Stdio};
use std::time::Duration;
use crate::keyring::get_agent_token;
use crate::util::env_path::check_executable_in_path;

pub fn read_copilot_session() -> Option<String> {
    // 1. Check OS Keyring first
    if let Some(env) = get_agent_token("copilot") {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.access_token);
        }
    }

    // 2. Try executing gh auth token
    if let Ok(out) = Command::new("gh").args(["auth", "token"]).output() {
        if out.status.success() {
            let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    // 3. Fallback: check config file ~/.config/gh/hosts.yml
    if let Some(config_dir) = dirs::config_dir() {
        let hosts_path = config_dir.join("gh").join("hosts.yml");
        if hosts_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(hosts_path) {
                for line in raw.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("oauth_token:") {
                        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let tok = parts[1].trim().to_string();
                            if !tok.is_empty() {
                                return Some(tok);
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Fallback: check environment variable
    if let Ok(k) = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN")) {
        if !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }
    }

    None
}

/// Spawns the local GitHub Copilot CLI bridge process in detached background mode.
pub fn launch_copilot_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "copilot.exe" } else { "copilot" };
    if let Err(e) = check_executable_in_path(bin_name) {
        return Err(e);
    }

    let mut cmd = Command::new(bin_name);
    cmd.args(["--proxy-port", &port.to_string()])
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

    cmd.spawn().map_err(|e| format!("Failed to spawn Copilot CLI bridge: {}", e))
}

/// Probes Copilot endpoint /livez reachability.
pub fn verify_copilot_health(port: u16, timeout: Duration) -> bool {
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
