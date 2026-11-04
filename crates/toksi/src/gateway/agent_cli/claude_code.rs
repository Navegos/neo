// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/claude_code.rs
// created: 2026-10-31
// lastModified: 2026-11-04

use serde_json::Value;
use crate::keyring::get_agent_token;

pub fn read_claude_code_session() -> Option<String> {
    // 1. Check OS Keyring first
    if let Some(env) = get_agent_token("claude") {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.access_token);
        }
    }

    // 2. Check ~/.claude.json
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".claude.json");
        if path.exists() {
            if let Ok(raw) = std::fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                    if let Some(s) = val.get("primaryApiKey")
                        .and_then(Value::as_str)
                        .or_else(|| val.get("oauthToken").and_then(Value::as_str))
                        .or_else(|| val.get("sessionKey").and_then(Value::as_str))
                    {
                        if !s.is_empty() {
                            return Some(s.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: check environment variable
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }
    }

    None
}
