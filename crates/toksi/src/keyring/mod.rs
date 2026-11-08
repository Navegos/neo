// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/mod.rs
// created: 2026-11-04
// lastModified: 2026-11-04

pub mod envelope;
pub mod os_linux;
pub mod os_macos;
pub mod os_windows;

pub use envelope::{decode_base64, encode_base64, parse_credential_envelope, OAuthEnvelope};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyringError {
    #[error("Credential not found for service '{service}' (account '{account}')")]
    NotFound { service: String, account: String },
    #[error("OS Keyring storage failed: {0}")]
    StorageError(String),
    #[error("Failed to parse credential envelope: {0}")]
    InvalidEnvelope(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyringStatus {
    pub agent: String,
    pub service: String,
    pub account: String,
    pub backend: String,
    pub has_credential: bool,
    pub is_expired: bool,
    pub expires_at: Option<i64>,
    pub token_preview: Option<String>,
}

pub fn current_backend_name() -> &'static str {
    if cfg!(windows) {
        "Windows Credential Manager"
    } else if cfg!(target_os = "macos") {
        "macOS Keychain"
    } else if cfg!(target_os = "linux") {
        "Linux Secret Service"
    } else {
        "Unknown Keyring Backend"
    }
}

fn vault_path() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("TOKSI_DATA_DIR") {
        if !dir.is_empty() {
            return std::path::PathBuf::from(dir).join("vault.json");
        }
    }
    let h = crate::util::paths::home();
    if !h.is_empty() {
        return std::path::PathBuf::from(h).join(".toksi").join("vault.json");
    }
    std::path::PathBuf::from(crate::util::paths::toksi_data_dir()).join("vault.json")
}

fn read_from_vault(key: &str) -> Option<String> {
    let p = vault_path();
    let raw = std::fs::read_to_string(p).ok()?;
    let val: serde_json::Value = serde_json::from_str(&raw).ok()?;
    val.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn write_to_vault(key: &str, secret: &str) -> bool {
    let p = vault_path();
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut map: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .and_then(|v: serde_json::Value| v.as_object().cloned())
        .unwrap_or_default();
    map.insert(key.to_string(), serde_json::Value::String(secret.to_string()));
    if let Ok(serialized) = serde_json::to_string_pretty(&map) {
        crate::util::paths::write_file_atomic(&p, &serialized).is_ok()
    } else {
        false
    }
}

fn delete_from_vault(key: &str) -> bool {
    let p = vault_path();
    let Ok(raw) = std::fs::read_to_string(&p) else {
        return true;
    };
    let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return true;
    };
    if let Some(obj) = val.as_object_mut() {
        obj.remove(key);
        if let Ok(serialized) = serde_json::to_string_pretty(&obj) {
            let _ = crate::util::paths::write_file_atomic(&p, &serialized);
        }
    }
    true
}

pub fn get_credential(service: &str, account: &str) -> Option<String> {
    let os_result = if cfg!(windows) {
        let combined = format!("{}:{}", service, account);
        os_windows::read_credential(&combined)
            .or_else(|| os_windows::read_credential(service))
            .or_else(|| os_windows::read_credential(account))
    } else if cfg!(target_os = "macos") {
        os_macos::read_keychain(service, account)
    } else if cfg!(target_os = "linux") {
        os_linux::read_secret_service(service, account)
    } else {
        None
    };

    if os_result.is_some() {
        return os_result;
    }

    // Fallback: check persistent local vault
    let combined = format!("{}:{}", service, account);
    read_from_vault(&combined)
        .or_else(|| read_from_vault(service))
        .or_else(|| read_from_vault(account))
}

pub fn set_credential(service: &str, account: &str, secret: &str) -> bool {
    let combined = format!("{}:{}", service, account);
    let os_ok = if cfg!(windows) {
        os_windows::write_credential(&combined, Some(account), secret)
            || os_windows::write_credential(service, Some(account), secret)
    } else if cfg!(target_os = "macos") {
        os_macos::write_keychain(service, account, secret)
    } else if cfg!(target_os = "linux") {
        os_linux::write_secret_service(service, account, secret)
    } else {
        false
    };

    if os_ok {
        true
    } else {
        // Fallback: store in local persistent vault (e.g. sandbox container where LSASS write is blocked)
        write_to_vault(&combined, secret)
    }
}

pub fn delete_credential(service: &str, account: &str) -> bool {
    let combined = format!("{}:{}", service, account);
    if cfg!(windows) {
        let _ = os_windows::delete_credential(&combined);
        let _ = os_windows::delete_credential(service);
        let _ = os_windows::delete_credential(account);
    } else if cfg!(target_os = "macos") {
        let _ = os_macos::delete_keychain(service, account);
    } else if cfg!(target_os = "linux") {
        let _ = os_linux::delete_secret_service(service, account);
    }
    delete_from_vault(&combined);
    delete_from_vault(service);
    delete_from_vault(account);
    true
}

pub fn get_agent_token(agent: &str) -> Option<OAuthEnvelope> {
    let lower = agent.trim().to_lowercase();
    match lower.as_str() {
        "gemini" | "antigravity" | "agy" => {
            // macOS: service gemini, account antigravity
            // Windows / go-keyring: antigravity_user, jetski-standalone-oauth-token, etc.
            let raw = get_credential("antigravity", "antigravity_user")
                .or_else(|| get_credential("antigravity_user", "default"))
                .or_else(|| get_credential("jetski-standalone-oauth-token", "default"))
                .or_else(|| get_credential("gemini", "antigravity"))
                .or_else(|| get_credential("antigravity", "default"))
                .or_else(|| get_credential("antigravity", "antigravity"))
                .or_else(|| get_credential("google", "antigravity"))
                .or_else(|| get_credential("google", "default"))?;
            parse_credential_envelope(&raw)
        }
        "claude" | "claude_code" => {
            let raw = get_credential("claude", "sessionKey")
                .or_else(|| get_credential("claude", "primaryApiKey"))
                .or_else(|| get_credential("claude-code", "default"))?;
            parse_credential_envelope(&raw)
        }
        "copilot" | "githubcopilot" => {
            let raw = get_credential("git:https://github.com", "oauth_token")
                .or_else(|| get_credential("copilot", "default"))?;
            parse_credential_envelope(&raw)
        }
        "cursor" => {
            let raw = get_credential("cursor", "session")
                .or_else(|| get_credential("cursor", "default"))?;
            parse_credential_envelope(&raw)
        }
        "grok" => {
            let raw = get_credential("grok", "api_key")
                .or_else(|| get_credential("xai", "default"))?;
            parse_credential_envelope(&raw)
        }
        _ => {
            let raw = get_credential(&lower, "default")
                .or_else(|| get_credential(&lower, "token"))?;
            parse_credential_envelope(&raw)
        }
    }
}

pub fn audit_keyring_statuses() -> Vec<KeyringStatus> {
    let targets = [
        ("antigravity", "gemini", "antigravity"),
        ("claude", "claude", "sessionKey"),
        ("copilot", "git:https://github.com", "oauth_token"),
        ("cursor", "cursor", "session"),
        ("grok", "grok", "api_key"),
    ];

    let backend = current_backend_name().to_string();
    let mut results = Vec::new();

    for (agent, service, account) in targets {
        let env_opt = get_agent_token(agent);
        let has_credential = env_opt.is_some();
        let is_expired = env_opt.as_ref().map(|e| e.is_expired(0)).unwrap_or(false);
        let expires_at = env_opt.as_ref().and_then(|e| e.expires_at);
        let token_preview = env_opt.as_ref().map(|e| {
            let tok = &e.access_token;
            if tok.len() > 10 {
                format!("{}...{}", &tok[..4], &tok[tok.len() - 4..])
            } else {
                "***".to_string()
            }
        });

        results.push(KeyringStatus {
            agent: agent.to_string(),
            service: service.to_string(),
            account: account.to_string(),
            backend: backend.clone(),
            has_credential,
            is_expired,
            expires_at,
            token_preview,
        });
    }

    results
}

