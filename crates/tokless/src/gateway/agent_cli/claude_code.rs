// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/agent_cli/claude_code.rs

use serde_json::Value;

pub fn read_claude_code_session() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(".claude.json");
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(path).ok()?;
    let val: Value = serde_json::from_str(&raw).ok()?;
    val.get("primaryApiKey")
        .and_then(Value::as_str)
        .or_else(|| val.get("oauthToken").and_then(Value::as_str))
        .or_else(|| val.get("sessionKey").and_then(Value::as_str))
        .map(|s| s.to_string())
}
