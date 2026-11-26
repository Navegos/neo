// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/codex.rs
// created: 2026-11-09

use std::process::Child;
use std::time::Duration;
use serde_json::Value;
use crate::keyring::get_agent_token;

pub fn read_codex_session() -> Option<String> {
    // 1. Check OS Keyring first
    if let Some(mut env) = get_agent_token("codex")
        && !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.take_access_token());
        }
    if let Some(mut env) = get_agent_token("openai")
        && !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.take_access_token());
        }

    // 2. Check local ~/.codex/auth.json
    if let Some(home) = dirs::home_dir() {
        let auth_path = home.join(".codex").join("auth.json");
        if auth_path.exists()
            && let Ok(raw) = std::fs::read_to_string(auth_path)
                && let Ok(val) = serde_json::from_str::<Value>(&raw)
                    && let Some(s) = val.get("tokens")
                        .and_then(|t| t.get("access_token").or_else(|| t.get("id_token")))
                        .and_then(Value::as_str)
                        .or_else(|| val.get("api_key").and_then(Value::as_str))
                        && !s.is_empty() {
                            return Some(s.to_string());
                        }
    }

    // 3. Fallback: OPENAI_API_KEY environment variable
    if let Ok(k) = std::env::var("OPENAI_API_KEY")
        && !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }

    None
}

/// Spawns the local OpenAI Codex CLI bridge in detached background mode.
pub fn launch_codex_bridge(port: u16) -> Result<Child, String> {
    let bin_name = if cfg!(windows) { "codex.exe" } else { "codex" };
    let port_str = port.to_string();
    super::launch_detached_bridge(bin_name, &["serve", "--port", &port_str], "Codex")
}

/// Probes Codex endpoint /livez reachability.
pub fn verify_codex_health(port: u16, timeout: Duration) -> bool {
    super::probe_local_livez(port, timeout)
}
