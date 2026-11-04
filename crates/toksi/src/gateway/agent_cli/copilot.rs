// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/copilot.rs
// created: 2026-10-31
// lastModified: 2026-11-04

use std::process::Command;
use crate::keyring::get_agent_token;

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
