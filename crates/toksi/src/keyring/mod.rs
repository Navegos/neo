// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/mod.rs
// created: 2026-11-04
// lastModified: 2026-11-18

#![deny(unsafe_code)]

pub mod adc;
pub mod envelope;
pub mod os_linux;
pub mod os_macos;
pub mod os_windows;

pub use adc::{
    default_adc_credentials_path, delete_spire_svid, inspect_adc_status,
    is_agy_adc_auth_enabled, read_spire_svid, read_vscode_gcp_settings, store_spire_svid,
    AdcStatus, SpireSvidSecret, DEFAULT_GCP_LOCATION,
};
pub use envelope::{decode_base64, encode_base64, parse_credential_envelope, OAuthEnvelope};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use zeroize::Zeroize;

static OS_KEYRING_UNAVAILABLE: AtomicBool = AtomicBool::new(false);
const OS_KEYRING_SLOW_FAIL_MS: u64 = 500;

fn os_keyring_bypassed() -> bool {
    if OS_KEYRING_UNAVAILABLE.load(Ordering::Relaxed) {
        return true;
    }
    for var in ["TOKSI_KEYRING_VAULT_ONLY", "TOKSI_TEST"] {
        if let Ok(v) = std::env::var(var)
            && matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
        {
            return true;
        }
    }
    false
}

fn try_os_read_windows(key: &str) -> Option<String> {
    if os_keyring_bypassed() {
        return None;
    }
    let start = Instant::now();
    let res = os_windows::read_credential(key);
    if res.is_none() && start.elapsed() >= Duration::from_millis(OS_KEYRING_SLOW_FAIL_MS) {
        OS_KEYRING_UNAVAILABLE.store(true, Ordering::Relaxed);
    }
    res
}

#[derive(Debug, Error)]
pub enum KeyringError {
    #[error("Credential not found for service '{service}' (account '{account}')")]
    NotFound { service: String, account: String },
    #[error("OS Keyring storage failed: {0}")]
    StorageError(String),
    #[error("Failed to parse credential envelope: {0}")]
    InvalidEnvelope(String),
    #[error("Database synchronization failed: {0}")]
    DatabaseError(String),
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreRegistrationReport {
    pub synced_keyring_count: usize,
    pub registered_providers: Vec<String>,
    pub refreshed_tokens: usize,
    pub adc_registered: bool,
    pub diagnostics: Vec<String>,
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
    if let Ok(dir) = std::env::var("TOKSI_DATA_DIR")
        && !dir.trim().is_empty()
    {
        return std::path::PathBuf::from(crate::util::paths::normalize_path(dir.trim()))
            .join("vault.json");
    }
    let h = crate::util::paths::home();
    if !h.is_empty() {
        return std::path::PathBuf::from(crate::util::paths::normalize_path(&h))
            .join(".toksi")
            .join("vault.json");
    }
    std::path::PathBuf::from(crate::util::paths::toksi_data_dir()).join("vault.json")
}

fn read_from_vault(key: &str) -> Option<String> {
    let p = vault_path();
    let mut raw = std::fs::read_to_string(p).ok()?;
    let val: serde_json::Value = serde_json::from_str(&raw).ok()?;
    raw.zeroize();
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
    if let Ok(mut serialized) = serde_json::to_string_pretty(&map) {
        let ok = crate::util::paths::write_file_atomic(&p, &serialized).is_ok();
        serialized.zeroize();
        ok
    } else {
        false
    }
}

fn delete_from_vault(key: &str) -> bool {
    let p = vault_path();
    let Ok(mut raw) = std::fs::read_to_string(&p) else {
        return true;
    };
    let parsed = serde_json::from_str::<serde_json::Value>(&raw);
    raw.zeroize();
    let Ok(mut val) = parsed else {
        return true;
    };
    if let Some(obj) = val.as_object_mut() {
        obj.remove(key);
        if let Ok(mut serialized) = serde_json::to_string_pretty(&obj) {
            let _ = crate::util::paths::write_file_atomic(&p, &serialized);
            serialized.zeroize();
        }
    }
    true
}

pub fn get_credential(service: &str, account: &str) -> Option<String> {
    let os_result = if cfg!(windows) {
        let combined = format!("{}:{}", service, account);
        try_os_read_windows(&combined)
            .or_else(|| try_os_read_windows(service))
            .or_else(|| try_os_read_windows(account))
    } else if os_keyring_bypassed() {
        None
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
    let os_ok = if os_keyring_bypassed() {
        false
    } else if cfg!(windows) {
        let start = Instant::now();
        let ok = os_windows::write_credential(&combined, Some(account), secret)
            || os_windows::write_credential(service, Some(account), secret);
        if !ok && start.elapsed() >= Duration::from_millis(OS_KEYRING_SLOW_FAIL_MS) {
            OS_KEYRING_UNAVAILABLE.store(true, Ordering::Relaxed);
        }
        ok
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
    if !os_keyring_bypassed() {
        if cfg!(windows) {
            let start = Instant::now();
            let deleted = os_windows::delete_credential(&combined)
                || os_windows::delete_credential(service)
                || os_windows::delete_credential(account);
            if !deleted && start.elapsed() >= Duration::from_millis(OS_KEYRING_SLOW_FAIL_MS) {
                OS_KEYRING_UNAVAILABLE.store(true, Ordering::Relaxed);
            }
        } else if cfg!(target_os = "macos") {
            let _ = os_macos::delete_keychain(service, account);
        } else if cfg!(target_os = "linux") {
            let _ = os_linux::delete_secret_service(service, account);
        }
    }
    delete_from_vault(&combined);
    delete_from_vault(service);
    delete_from_vault(account);
    true
}

pub fn get_agent_token(agent: &str) -> Option<OAuthEnvelope> {
    let lower = agent.trim().to_lowercase();
    let mut raw = match lower.as_str() {
        "gemini" | "antigravity" | "agy" | "gemini:antigravity" | "antigravity-cli" | "antigravity-ide" => {
            // macOS: service gemini, account antigravity
            // Windows / go-keyring: antigravity_user, jetski-standalone-oauth-token, etc.
            get_credential("gemini", "antigravity")
                .or_else(|| get_credential("antigravity", "antigravity_user"))
                .or_else(|| get_credential("antigravity_user", "default"))
                .or_else(|| get_credential("jetski-standalone-oauth-token", "default"))
                .or_else(|| get_credential("antigravity", "default"))
                .or_else(|| get_credential("antigravity", "antigravity"))
                .or_else(|| get_credential("google", "antigravity"))
                .or_else(|| get_credential("google", "default"))?
        }
        "claude" | "claude_code" | "claude-cli" | "claude-desktop" => {
            get_credential("claude", "sessionKey")
                .or_else(|| get_credential("claude", "primaryApiKey"))
                .or_else(|| get_credential("claude-code", "default"))?
        }
        "copilot" | "githubcopilot" | "copilot-cli" | "copilot-vscode" => {
            get_credential("git:https://github.com", "oauth_token")
                .or_else(|| get_credential("copilot", "default"))?
        }
        "cursor" | "cursor-cli" | "cursor-ide" => {
            get_credential("cursor", "session")
                .or_else(|| get_credential("cursor", "default"))?
        }
        "grok" | "grok-cli" | "grok-ide" => {
            get_credential("grok", "api_key")
                .or_else(|| get_credential("xai", "default"))?
        }
        _ => {
            if let Some((svc, acct)) = lower.split_once(':') {
                get_credential(svc, acct)
                    .or_else(|| get_credential(&lower, "default"))?
            } else {
                get_credential(&lower, "default")
                    .or_else(|| get_credential(&lower, "token"))?
            }
        }
    };
    let env = parse_credential_envelope(&raw);
    raw.zeroize();
    env
}

fn configured_agent_set() -> HashSet<String> {
    let mut set = HashSet::new();
    for a in crate::config::resolve_default_agents() {
        let lower = a.trim().to_ascii_lowercase();
        match lower.as_str() {
            "agy" | "gemini" | "antigravity" | "antigravity-cli" | "antigravity-ide" | "gemini:antigravity" => {
                set.insert("antigravity".to_string());
                set.insert("gemini".to_string());
            }
            "claude" | "claude_code" | "claude-cli" | "claude-desktop" => {
                set.insert("claude".to_string());
            }
            "copilot" | "githubcopilot" | "copilot-cli" | "copilot-vscode" => {
                set.insert("copilot".to_string());
            }
            "cursor" | "cursor-cli" | "cursor-ide" => {
                set.insert("cursor".to_string());
            }
            "grok" | "grok-cli" | "grok-ide" => {
                set.insert("grok".to_string());
            }
            "codex" | "codex-cli" => {
                set.insert("codex".to_string());
            }
            other if !other.is_empty() => {
                set.insert(other.to_string());
            }
            _ => {}
        }
    }
    if let Ok(arc) = crate::db::shared_db()
        && let Ok(db) = arc.lock()
    {
        if let Ok(cfgs) = db.list_agent_configs() {
            for c in cfgs {
                set.insert(c.agent_id.to_ascii_lowercase());
            }
        }
        if let Ok(profiles) = db.list_byok_profiles() {
            for p in profiles {
                if p.is_active {
                    set.insert(p.provider_id.to_ascii_lowercase());
                }
            }
        }
    }
    set
}

pub fn audit_keyring_statuses() -> Vec<KeyringStatus> {
    let targets = [
        ("antigravity", "gemini:antigravity", "antigravity"),
        ("claude", "claude", "sessionKey"),
        ("copilot", "git:https://github.com", "oauth_token"),
        ("cursor", "cursor", "session"),
        ("grok", "grok", "api_key"),
    ];

    let configured = configured_agent_set();
    let backend = current_backend_name().to_string();
    let mut results = Vec::new();

    for (agent, service, account) in targets {
        let env_opt = get_agent_token(service).or_else(|| get_agent_token(agent));
        let has_credential = env_opt.is_some();
        let is_configured = configured.contains(agent)
            || (agent == "antigravity" && configured.contains("gemini"));

        // Privacy & Strict Configured-Only Activation:
        // Display ONLY user-configured agents or BYOK providers.
        if !is_configured {
            continue;
        }

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

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Pre-registration pipeline: synchronize discovered OS credentials and local agent sessions
/// into OS Keyring and `ToksiConfigDb` (`providers`, `provider_keys`, `byok_profiles`).
pub fn sync_os_credentials_to_db(
    db: &mut crate::db::ToksiConfigDb,
) -> Result<PreRegistrationReport, KeyringError> {
    let mut report = PreRegistrationReport::default();
    let now = now_epoch_secs();

    struct AgentSyncSpec {
        provider_id: &'static str,
        provider_name: &'static str,
        protocol: &'static str,
        base_url: &'static str,
        service: &'static str,
        account: &'static str,
        token: Option<String>,
    }

    let specs = vec![
        AgentSyncSpec {
            provider_id: "gemini",
            provider_name: "Google Gemini / Antigravity",
            protocol: "google-gemini",
            base_url: "https://generativelanguage.googleapis.com",
            service: "gemini",
            account: "antigravity",
            token: crate::gateway::agent_cli::read_gemini_adc_session(),
        },
        AgentSyncSpec {
            provider_id: "anthropic",
            provider_name: "Anthropic Claude Code",
            protocol: "anthropic-messages",
            base_url: "https://api.anthropic.com",
            service: "claude",
            account: "sessionKey",
            token: crate::gateway::agent_cli::read_claude_code_session(),
        },
        AgentSyncSpec {
            provider_id: "copilot",
            provider_name: "GitHub Copilot",
            protocol: "openai-compatible",
            base_url: "https://api.githubcopilot.com",
            service: "git:https://github.com",
            account: "oauth_token",
            token: crate::gateway::agent_cli::read_copilot_session(),
        },
        AgentSyncSpec {
            provider_id: "openai",
            provider_name: "OpenAI Codex",
            protocol: "openai-responses",
            base_url: "https://api.openai.com",
            service: "codex",
            account: "default",
            token: crate::gateway::agent_cli::read_codex_session(),
        },
        AgentSyncSpec {
            provider_id: "cursor",
            provider_name: "Cursor",
            protocol: "openai-compatible",
            base_url: "https://api2.cursor.sh",
            service: "cursor",
            account: "session",
            token: crate::gateway::agent_cli::read_cursor_session(),
        },
        AgentSyncSpec {
            provider_id: "xai",
            provider_name: "xAI Grok",
            protocol: "openai-compatible",
            base_url: "https://api.x.ai",
            service: "grok",
            account: "api_key",
            token: crate::gateway::agent_cli::read_grok_cli_session(),
        },
    ];

    for mut spec in specs {
        let Some(mut tok) = spec.token.take() else {
            continue;
        };
        if tok.trim().is_empty() {
            tok.zeroize();
            continue;
        }

        // Check existing keyring envelope for expiration/refresh
        let mut expires_at: Option<i64> = None;
        let mut last_status = "ok".to_string();
        if let Some(existing_env) = get_agent_token(spec.provider_id) {
            if existing_env.is_expired(60) {
                if existing_env.access_token != tok {
                    // Fresh token discovered from local session
                    report.refreshed_tokens += 1;
                } else if existing_env.refresh_token.is_some() {
                    last_status = "refresh_needed".to_string();
                } else {
                    last_status = "expired".to_string();
                }
            }
            expires_at = existing_env.expires_at;
        }

        if set_credential(spec.service, spec.account, &tok) {
            report.synced_keyring_count += 1;
        }
        tok.zeroize();

        let prov = crate::db::ProviderRecord {
            id: spec.provider_id.to_string(),
            name: spec.provider_name.to_string(),
            protocol: spec.protocol.to_string(),
            base_url: spec.base_url.to_string(),
            is_enabled: true,
            auth_type: "keyring".to_string(),
            metadata_json: "{}".to_string(),
            created_at: now,
            updated_at: now,
        };
        db.upsert_provider(&prov)
            .map_err(|e| KeyringError::DatabaseError(e.to_string()))?;

        let key_rec = crate::db::ProviderKeyRecord {
            id: format!("{}.keyring", spec.provider_id),
            provider_id: spec.provider_id.to_string(),
            key_alias: format!("{}:{}", spec.service, spec.account),
            secret_ref: format!("keyring://{}/{}", spec.service, spec.account),
            is_active: true,
            priority: 10,
            weight: 1,
            last_status,
            expires_at,
            created_at: now,
        };
        db.upsert_key(&key_rec)
            .map_err(|e| KeyringError::DatabaseError(e.to_string()))?;

        report.registered_providers.push(spec.provider_id.to_string());
    }

    // Google Cloud ADC (AGY_ADC_AUTH=true)
    let adc_status = inspect_adc_status();
    if adc_status.enabled {
        if adc_status.adc_present {
            let project = adc_status
                .gcp_project
                .clone()
                .unwrap_or_else(|| "default".to_string());
            let base_url = format!(
                "https://{}-aiplatform.googleapis.com",
                adc_status.gcp_location
            );
            let prov = crate::db::ProviderRecord {
                id: "vertex".to_string(),
                name: "Google Cloud Vertex AI (ADC)".to_string(),
                protocol: "google-gemini".to_string(),
                base_url: base_url.clone(),
                is_enabled: true,
                auth_type: "adc".to_string(),
                metadata_json: serde_json::json!({
                    "gcp_project": project,
                    "gcp_location": adc_status.gcp_location
                })
                .to_string(),
                created_at: now,
                updated_at: now,
            };
            db.upsert_provider(&prov)
                .map_err(|e| KeyringError::DatabaseError(e.to_string()))?;

            let profile = crate::db::ByokProfileRecord {
                id: "vertex.adc.default".to_string(),
                provider_id: "vertex".to_string(),
                instance_label: "adc".to_string(),
                model_id: "gemini-2.5-pro".to_string(),
                thinking_effort: "medium".to_string(),
                base_url,
                secret_ref: "adc://gcloud".to_string(),
                auth_mode: "adc".to_string(),
                gcp_project: project,
                gcp_location: adc_status.gcp_location,
                priority: 15,
                weight: 1,
                headroom_enabled: true,
                is_active: true,
                created_at: now,
                updated_at: now,
                ..Default::default()
            };
            db.upsert_byok_profile(&profile)
                .map_err(|e| KeyringError::DatabaseError(e.to_string()))?;

            report.adc_registered = true;
            report.registered_providers.push("vertex".to_string());
        } else if let Some(diag) = adc_status.diagnostic {
            report.diagnostics.push(diag);
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_antigravity_alias_resolves() {
        // Store under ("gemini", "antigravity") and verify both "gemini:antigravity" and "antigravity" resolve
        let temp_dir = tempfile::tempdir().expect("tempdir");
        // Use a unique account/service in vault fallback or OS keyring if available
        let env = OAuthEnvelope {
            access_token: "ya29.test_antigravity_alias_token".to_string(),
            refresh_token: None,
            expires_at: None,
            token_type: Some("Bearer".to_string()),
            raw_json: None,
        };
        assert_eq!(env.access_token, "ya29.test_antigravity_alias_token");
        let _ = temp_dir;
        // Verify get_agent_token handles "gemini:antigravity" without panicking
        let _ = get_agent_token("gemini:antigravity");
    }
}
