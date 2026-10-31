// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/agent_cli/gemini_agy.rs

use serde_json::Value;

pub fn read_gemini_adc_session() -> Option<String> {
    // 1. Check ~/.gemini/
    if let Some(home) = dirs::home_dir() {
        let gemini_token_path = home.join(".gemini").join("oauth_credentials.json");
        if gemini_token_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(gemini_token_path) {
                if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                    if let Some(tok) = val.get("access_token").and_then(Value::as_str) {
                        return Some(tok.to_string());
                    }
                }
            }
        }

        // 2. Check gcloud application_default_credentials.json
        let adc_path = if cfg!(windows) {
            dirs::config_dir().map(|c| c.join("gcloud").join("application_default_credentials.json"))
        } else {
            home.join(".config").join("gcloud").join("application_default_credentials.json").into()
        };

        if let Some(path) = adc_path {
            if path.exists() {
                if let Ok(raw) = std::fs::read_to_string(path) {
                    if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                        if let Some(tok) = val.get("client_secret").and_then(Value::as_str) {
                            return Some(tok.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
