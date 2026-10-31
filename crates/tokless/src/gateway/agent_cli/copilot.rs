// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/agent_cli/copilot.rs

use std::process::Command;

pub fn read_copilot_session() -> Option<String> {
    // Try executing gh auth token first
    if let Ok(out) = Command::new("gh").args(["auth", "token"]).output() {
        if out.status.success() {
            let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    // Fallback: check config file ~/.config/gh/hosts.yml
    let config_dir = dirs::config_dir()?;
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
    None
}
