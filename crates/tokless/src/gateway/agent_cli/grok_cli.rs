// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/agent_cli/grok_cli.rs

use serde_json::Value;

pub fn read_grok_cli_session() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(".grok").join("config.json");
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(path).ok()?;
    let val: Value = serde_json::from_str(&raw).ok()?;
    val.get("api_key")
        .or_else(|| val.get("session_token"))
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}
