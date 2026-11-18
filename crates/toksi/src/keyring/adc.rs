// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/adc.rs
// created: 2026-11-18
// lastModified: 2026-11-18

//! Google Cloud Application Default Credentials (`AGY_ADC_AUTH=true`) and SPIRE SVID keyring helpers.

#![deny(unsafe_code)]

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zeroize::Zeroize;

use super::envelope::{parse_credential_envelope, OAuthEnvelope};
use super::{delete_credential, get_credential, set_credential};

pub const DEFAULT_GCP_LOCATION: &str = "us-central1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdcStatus {
    pub enabled: bool,
    pub gcp_project: Option<String>,
    pub gcp_location: String,
    pub adc_path: String,
    pub adc_present: bool,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpireSvidSecret {
    pub spiffe_id: String,
    pub token: String,
}

impl Zeroize for SpireSvidSecret {
    fn zeroize(&mut self) {
        self.spiffe_id.zeroize();
        self.token.zeroize();
    }
}

impl Drop for SpireSvidSecret {
    fn drop(&mut self) {
        self.zeroize();
    }
}

pub fn is_agy_adc_auth_enabled() -> bool {
    if let Ok(val) = std::env::var("AGY_ADC_AUTH") {
        let trimmed = val.trim();
        return trimmed.eq_ignore_ascii_case("true") || trimmed == "1";
    }
    let cfg = crate::config::load_toksi_toml_config();
    !cfg.auth.gcp_adc.gcp_project.trim().is_empty()
}

pub fn default_adc_credentials_path() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(crate::util::paths::normalize_path(trimmed)));
        }
    }
    if cfg!(windows) {
        dirs::config_dir().map(|c| c.join("gcloud").join("application_default_credentials.json"))
    } else {
        dirs::home_dir().map(|h| {
            h.join(".config")
                .join("gcloud")
                .join("application_default_credentials.json")
        })
    }
}

fn vscode_settings_path() -> Option<PathBuf> {
    if cfg!(windows) {
        dirs::config_dir().map(|c| c.join("Code").join("User").join("settings.json"))
    } else if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| {
            h.join("Library")
                .join("Application Support")
                .join("Code")
                .join("User")
                .join("settings.json")
        })
    } else {
        dirs::home_dir().map(|h| h.join(".config").join("Code").join("User").join("settings.json"))
    }
}

pub fn read_vscode_gcp_settings() -> (Option<String>, String) {
    let mut project: Option<String> = std::env::var("GOOGLE_CLOUD_PROJECT")
        .ok()
        .or_else(|| std::env::var("GCP_PROJECT").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut location: Option<String> = std::env::var("GOOGLE_CLOUD_LOCATION")
        .ok()
        .or_else(|| std::env::var("GCP_LOCATION").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(settings_path) = vscode_settings_path() {
        if let Ok(raw) = std::fs::read_to_string(&settings_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                if project.is_none() {
                    project = val
                        .get("gcp.project")
                        .and_then(Value::as_str)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }
                if location.is_none() {
                    location = val
                        .get("gcp.location")
                        .and_then(Value::as_str)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }
            }
        }
    }

    if project.is_none() {
        let cfg = crate::config::load_toksi_toml_config();
        let cfg_proj = cfg.auth.gcp_adc.gcp_project.trim();
        if !cfg_proj.is_empty() {
            project = Some(cfg_proj.to_string());
        }
    }

    (
        project,
        location.unwrap_or_else(|| DEFAULT_GCP_LOCATION.to_string()),
    )
}

pub fn inspect_adc_status() -> AdcStatus {
    let enabled = is_agy_adc_auth_enabled();
    let (gcp_project, gcp_location) = read_vscode_gcp_settings();
    let adc_path_buf = default_adc_credentials_path();
    let adc_path = adc_path_buf
        .as_ref()
        .map(|p| crate::util::paths::normalize_path(&p.to_string_lossy()))
        .unwrap_or_default();

    let adc_present = adc_path_buf
        .as_ref()
        .map(|p| {
            p.is_file()
                && std::fs::read_to_string(p)
                    .ok()
                    .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
                    .is_some()
        })
        .unwrap_or(false);

    let diagnostic = if !adc_present {
        let proj = gcp_project.as_deref().unwrap_or("<GCP_PROJECT>");
        Some(format!(
            "ADC credentials missing at '{}'. Run: gcloud auth application-default login --project {}",
            adc_path, proj
        ))
    } else {
        None
    };

    AdcStatus {
        enabled,
        gcp_project,
        gcp_location,
        adc_path,
        adc_present,
        diagnostic,
    }
}

pub fn store_spire_svid(spiffe_id: &str, svid_token: &str) -> bool {
    let clean_id = spiffe_id.trim();
    if clean_id.is_empty() || svid_token.trim().is_empty() {
        return false;
    }
    set_credential("spiffe", clean_id, svid_token)
}

pub fn read_spire_svid(spiffe_id: &str) -> Option<OAuthEnvelope> {
    let clean_id = spiffe_id.trim();
    if clean_id.is_empty() {
        return None;
    }
    let mut raw = get_credential("spiffe", clean_id)?;
    let env = parse_credential_envelope(&raw);
    raw.zeroize();
    env
}

pub fn delete_spire_svid(spiffe_id: &str) -> bool {
    let clean_id = spiffe_id.trim();
    if clean_id.is_empty() {
        return false;
    }
    delete_credential("spiffe", clean_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adc_status_diagnostic_format() {
        let status = inspect_adc_status();
        assert!(!status.gcp_location.is_empty());
        if !status.adc_present {
            let diag = status.diagnostic.as_deref().unwrap_or("");
            assert!(diag.contains("gcloud auth application-default login --project"));
        }
    }
}
