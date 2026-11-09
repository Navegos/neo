// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/grok.rs
// created: 2026-11-09

use std::process::{Child, Command, Stdio};
use std::time::Duration;
use serde_json::Value;
use crate::keyring::get_agent_token;
use crate::util::env_path::check_executable_in_path;

pub fn read_grok_session() -> Option<String> {
    // 1. Check OS Keyring first
    if let Some(env) = get_agent_token("grok") {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.access_token);
        }
    }
    if let Some(env) = get_agent_token("xai") {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.access_token);
        }
    }

    // 2. Check local config ~/.grok/config.json
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".grok").join("config.json");
        if path.exists() {
            if let Ok(raw) = std::fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                    if let Some(tok) = val.get("api_key")
                        .or_else(|| val.get("session_token"))
                        .and_then(Value::as_str)
                    {
                        if !tok.is_empty() {
                            return Some(tok.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: check environment variable
    if let Ok(k) = std::env::var("XAI_API_KEY").or_else(|_| std::env::var("GROK_API_KEY")) {
        if !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }
    }

    None
}

/// Spawns the local xAI Grok CLI bridge process in detached background mode.
pub fn launch_grok_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "grok.exe" } else { "grok" };
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
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }

    cmd.spawn().map_err(|e| format!("Failed to spawn Grok CLI bridge: {}", e))
}

/// Probes Grok CLI bridge endpoint reachability.
pub fn verify_grok_health(port: u16, timeout: Duration) -> bool {
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
