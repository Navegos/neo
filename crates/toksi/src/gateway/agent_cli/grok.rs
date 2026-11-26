// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/grok.rs
// created: 2026-11-09

use std::process::Child;
use std::time::Duration;
use serde_json::Value;
use crate::keyring::get_agent_token;

pub fn read_grok_session() -> Option<String> {
    // 1. Check OS Keyring first
    if let Some(mut env) = get_agent_token("grok")
        && !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.take_access_token());
        }
    if let Some(mut env) = get_agent_token("xai")
        && !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.take_access_token());
        }

    // 2. Check local config ~/.grok/config.json
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".grok").join("config.json");
        if path.exists()
            && let Ok(raw) = std::fs::read_to_string(path)
                && let Ok(val) = serde_json::from_str::<Value>(&raw)
                    && let Some(tok) = val.get("api_key")
                        .or_else(|| val.get("session_token"))
                        .and_then(Value::as_str)
                        && !tok.is_empty() {
                            return Some(tok.to_string());
                        }
    }

    // 3. Fallback: check environment variable
    if let Ok(k) = std::env::var("XAI_API_KEY").or_else(|_| std::env::var("GROK_API_KEY"))
        && !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }

    None
}

/// Spawns the local xAI Grok CLI bridge process in detached background mode.
pub fn launch_grok_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "grok.exe" } else { "grok" };
    let port_str = port.to_string();
    super::launch_detached_bridge(bin_name, &["serve", "--port", &port_str], "Grok")
}

/// Probes Grok CLI bridge endpoint reachability.
pub fn verify_grok_health(port: u16, timeout: Duration) -> bool {
    super::probe_local_livez(port, timeout)
}
