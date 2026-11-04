// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/gemini_agy.rs
// created: 2026-10-31
// lastModified: 2026-11-04

use serde_json::Value;
use crate::keyring::get_agent_token;

pub fn read_gemini_adc_session() -> Option<String> {
    // 1. Check native OS Keyring first (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    if let Some(env) = get_agent_token("antigravity") {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.access_token);
        }
    }
    if let Some(env) = get_agent_token("gemini") {
        if !env.is_expired(30) && !env.access_token.is_empty() {
            return Some(env.access_token);
        }
    }

    // 2. Check ~/.gemini/oauth_credentials.json
    if let Some(home) = dirs::home_dir() {
        let gemini_token_path = home.join(".gemini").join("oauth_credentials.json");
        if gemini_token_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(gemini_token_path) {
                if let Some(env) = crate::keyring::parse_credential_envelope(&raw) {
                    if !env.is_expired(30) && !env.access_token.is_empty() {
                        return Some(env.access_token);
                    }
                }
                if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                    if let Some(tok) = val.get("access_token").or_else(|| val.get("token")).and_then(Value::as_str) {
                        return Some(tok.to_string());
                    }
                }
            }
        }

        // 3. Check gcloud application_default_credentials.json
        let adc_path = if cfg!(windows) {
            dirs::config_dir().map(|c| c.join("gcloud").join("application_default_credentials.json"))
        } else {
            home.join(".config").join("gcloud").join("application_default_credentials.json").into()
        };

        if let Some(path) = adc_path {
            if path.exists() {
                if let Ok(raw) = std::fs::read_to_string(path) {
                    if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                        if let Some(tok) = val.get("client_secret").or_else(|| val.get("access_token")).and_then(Value::as_str) {
                            return Some(tok.to_string());
                        }
                    }
                }
            }
        }
    }

    // 4. Fallback: check environment variables
    if let Ok(k) = std::env::var("GEMINI_API_KEY") {
        if !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }
    }
    if let Ok(k) = std::env::var("GOOGLE_API_KEY") {
        if !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }
    }

    None
}
