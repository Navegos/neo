// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/adc.rs
// created: 2026-11-18
// lastModified: 2026-11-21

//! Google Cloud Application Default Credentials (`AGY_ADC_AUTH=true`),
//! pure-Rust OAuth2 refresh-token / service-account RS256 JWT exchange,
//! 60-second-skew in-memory token cache, and SPIRE SVID keyring helpers.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::{STANDARD as B64_STD, URL_SAFE_NO_PAD as B64_URL};
use base64::Engine;
use ring::rand::SystemRandom;
use ring::signature::{RsaKeyPair, RSA_PKCS1_SHA256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zeroize::Zeroize;

use super::envelope::{parse_credential_envelope, OAuthEnvelope};
use super::{delete_credential, get_credential, set_credential};

pub const DEFAULT_GCP_LOCATION: &str = "us-central1";
pub const GOOGLE_OAUTH2_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const VERTEX_CLOUD_PLATFORM_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
pub const ADC_EXPIRY_SKEW_SECS: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdcStatus {
    pub enabled: bool,
    pub gcp_project: Option<String>,
    pub gcp_location: String,
    pub adc_path: String,
    pub adc_present: bool,
    pub credential_kind: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdcCredentialKind {
    AuthorizedUser {
        client_id: String,
        client_secret: String,
        refresh_token: String,
        quota_project_id: Option<String>,
    },
    ServiceAccount {
        client_email: String,
        private_key: String,
        token_uri: String,
        project_id: Option<String>,
    },
}

impl AdcCredentialKind {
    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::AuthorizedUser { .. } => "authorized_user",
            Self::ServiceAccount { .. } => "service_account",
        }
    }

    pub fn project_hint(&self) -> Option<&str> {
        match self {
            Self::AuthorizedUser {
                quota_project_id, ..
            } => quota_project_id.as_deref(),
            Self::ServiceAccount { project_id, .. } => project_id.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAdcToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at_unix: i64,
    pub source_kind: String,
}

impl CachedAdcToken {
    pub fn is_valid_at(&self, now_unix: i64) -> bool {
        !self.access_token.is_empty() && (self.expires_at_unix - ADC_EXPIRY_SKEW_SECS) > now_unix
    }
}

impl Zeroize for CachedAdcToken {
    fn zeroize(&mut self) {
        self.access_token.zeroize();
        self.token_type.zeroize();
    }
}

static ADC_TOKEN_CACHE: OnceLock<RwLock<HashMap<String, CachedAdcToken>>> = OnceLock::new();

fn token_cache() -> &'static RwLock<HashMap<String, CachedAdcToken>> {
    ADC_TOKEN_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn clear_adc_token_cache() {
    if let Ok(mut guard) = token_cache().write() {
        for (_, mut tok) in guard.drain() {
            tok.zeroize();
        }
    }
}

pub fn put_cached_adc_token(cache_key: &str, token: CachedAdcToken) {
    if let Ok(mut guard) = token_cache().write() {
        guard.insert(cache_key.to_string(), token);
    }
}

pub fn get_cached_adc_token(cache_key: &str, now_unix: i64) -> Option<CachedAdcToken> {
    let guard = token_cache().read().ok()?;
    let entry = guard.get(cache_key)?;
    if entry.is_valid_at(now_unix) {
        Some(entry.clone())
    } else {
        None
    }
}

fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
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
        return matches!(
            trimmed.to_ascii_lowercase().as_str(),
            "true" | "1" | "yes"
        );
    }
    let cfg = crate::config::load_toksi_toml_config();
    if cfg.auth.gcp_adc.agy_adc_auth || !cfg.auth.gcp_adc.gcp_project.trim().is_empty() {
        return true;
    }
    if let Some(settings_path) = vscode_settings_path() {
        if let Ok(raw) = std::fs::read_to_string(&settings_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&raw) {
                if val
                    .get("agy.adcAuth")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return true;
                }
            }
        }
    }
    false
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
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let trimmed = xdg.trim();
            if !trimmed.is_empty() {
                return Some(
                    PathBuf::from(crate::util::paths::normalize_path(trimmed))
                        .join("gcloud")
                        .join("application_default_credentials.json"),
                );
            }
        }
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

/// Reads GCP project and location following the Wave 18.3 resolution order:
/// 1. `TOKSI_GCP_PROJECT` / `GOOGLE_CLOUD_PROJECT` / `GCP_PROJECT`
/// 2. `~/.toksi/config.toml` `[auth.gcp_adc]`
/// 3. VS Code `settings.json` (`"gcp.project"`, `"gcp.location"`)
/// 4. Default location `us-central1`
pub fn read_vscode_gcp_settings() -> (Option<String>, String) {
    let mut project: Option<String> = std::env::var("TOKSI_GCP_PROJECT")
        .ok()
        .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
        .or_else(|| std::env::var("GCP_PROJECT").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut location: Option<String> = std::env::var("TOKSI_GCP_LOCATION")
        .ok()
        .or_else(|| std::env::var("GOOGLE_CLOUD_LOCATION").ok())
        .or_else(|| std::env::var("GCP_LOCATION").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let cfg = crate::config::load_toksi_toml_config();
    if project.is_none() {
        let cfg_proj = cfg.auth.gcp_adc.gcp_project.trim();
        if !cfg_proj.is_empty() {
            project = Some(cfg_proj.to_string());
        }
    }
    if location.is_none() {
        let cfg_loc = cfg.auth.gcp_adc.gcp_location.trim();
        if !cfg_loc.is_empty() {
            location = Some(cfg_loc.to_string());
        }
    }

    if project.is_none() || location.is_none() {
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
    }

    (
        project,
        location.unwrap_or_else(|| DEFAULT_GCP_LOCATION.to_string()),
    )
}

pub fn parse_adc_credentials_json(raw: &str) -> Result<AdcCredentialKind, String> {
    let val: Value =
        serde_json::from_str(raw).map_err(|e| format!("Invalid ADC JSON: {}", e))?;
    let cred_type = val
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    match cred_type {
        "authorized_user" => {
            let client_id = val
                .get("client_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "ADC authorized_user missing client_id".to_string())?
                .to_string();
            let client_secret = val
                .get("client_secret")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "ADC authorized_user missing client_secret".to_string())?
                .to_string();
            let refresh_token = val
                .get("refresh_token")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "ADC authorized_user missing refresh_token".to_string())?
                .to_string();
            let quota_project_id = val
                .get("quota_project_id")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            Ok(AdcCredentialKind::AuthorizedUser {
                client_id,
                client_secret,
                refresh_token,
                quota_project_id,
            })
        }
        "service_account" => {
            let client_email = val
                .get("client_email")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "ADC service_account missing client_email".to_string())?
                .to_string();
            let private_key = val
                .get("private_key")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "ADC service_account missing private_key".to_string())?
                .to_string();
            let token_uri = val
                .get("token_uri")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(GOOGLE_OAUTH2_TOKEN_URL)
                .to_string();
            let project_id = val
                .get("project_id")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            Ok(AdcCredentialKind::ServiceAccount {
                client_email,
                private_key,
                token_uri,
                project_id,
            })
        }
        other => Err(format!(
            "Unsupported ADC credential type '{}' (expected authorized_user or service_account)",
            other
        )),
    }
}

pub fn parse_adc_credentials_file(path: &Path) -> Result<AdcCredentialKind, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        format!(
            "Failed to read ADC file '{}': {}",
            crate::util::paths::normalize_path(&path.to_string_lossy()),
            e
        )
    })?;
    parse_adc_credentials_json(&raw)
}

fn decode_pkcs8_pem(pem: &str) -> Result<Vec<u8>, String> {
    let mut b64 = String::with_capacity(pem.len());
    let mut in_body = false;
    for line in pem.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-----BEGIN ") {
            in_body = true;
            continue;
        }
        if trimmed.starts_with("-----END ") {
            break;
        }
        if in_body && !trimmed.is_empty() {
            b64.push_str(trimmed);
        }
    }
    if b64.is_empty() {
        return Err("Service account PEM missing PKCS#8 body".to_string());
    }
    B64_STD
        .decode(b64.as_bytes())
        .map_err(|e| format!("Invalid PEM base64: {}", e))
}

/// Pure-Rust RS256 JWT assertion builder for Google service-account OAuth2 exchange.
pub fn build_service_account_jwt(
    client_email: &str,
    private_key_pem: &str,
    token_uri: &str,
    scope: &str,
    now_unix: i64,
) -> Result<String, String> {
    let der = decode_pkcs8_pem(private_key_pem)?;
    let key_pair = RsaKeyPair::from_pkcs8(&der)
        .map_err(|e| format!("Invalid RS256 PKCS#8 private key: {:?}", e))?;

    let header = B64_URL.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
    let claims_json = json!({
        "iss": client_email,
        "scope": scope,
        "aud": token_uri,
        "iat": now_unix,
        "exp": now_unix + 3600,
    });
    let claims = B64_URL.encode(
        serde_json::to_vec(&claims_json)
            .map_err(|e| format!("JWT claims serialize error: {}", e))?,
    );
    let signing_input = format!("{}.{}", header, claims);

    let mut sig = vec![0u8; key_pair.public().modulus_len()];
    let rng = SystemRandom::new();
    key_pair
        .sign(&RSA_PKCS1_SHA256, &rng, signing_input.as_bytes(), &mut sig)
        .map_err(|e| format!("RS256 JWT signing failed: {:?}", e))?;

    Ok(format!("{}.{}", signing_input, B64_URL.encode(&sig)))
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    token_type: Option<String>,
}

/// Exchanges an ADC credential (`authorized_user` refresh token or `service_account` RS256 JWT)
/// for a short-lived Google OAuth2 bearer token via `https://oauth2.googleapis.com/token`.
pub async fn exchange_adc_token_async(cred: &AdcCredentialKind) -> Result<CachedAdcToken, String> {
    let client = crate::gateway::reconnect::auto_healing_async_client();
    let now = now_unix_secs();

    let (token_url, form_params): (&str, Vec<(&str, String)>) = match cred {
        AdcCredentialKind::AuthorizedUser {
            client_id,
            client_secret,
            refresh_token,
            ..
        } => (
            GOOGLE_OAUTH2_TOKEN_URL,
            vec![
                ("client_id", client_id.clone()),
                ("client_secret", client_secret.clone()),
                ("refresh_token", refresh_token.clone()),
                ("grant_type", "refresh_token".to_string()),
            ],
        ),
        AdcCredentialKind::ServiceAccount {
            client_email,
            private_key,
            token_uri,
            ..
        } => {
            let assertion = build_service_account_jwt(
                client_email,
                private_key,
                token_uri,
                VERTEX_CLOUD_PLATFORM_SCOPE,
                now,
            )?;
            (
                token_uri.as_str(),
                vec![
                    (
                        "grant_type",
                        "urn:ietf:params:oauth:grant-type:jwt-bearer".to_string(),
                    ),
                    ("assertion", assertion),
                ],
            )
        }
    };

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in &form_params {
        serializer.append_pair(k, v);
    }
    let encoded_body = serializer.finish();

    let resp = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(encoded_body)
        .send()
        .await
        .map_err(|e| format!("ADC token exchange request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("ADC token exchange HTTP {}: {}", status, body));
    }

    let parsed: OAuthTokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("ADC token response parse error: {}", e))?;

    let expires_in = parsed.expires_in.unwrap_or(3600).max(120);
    Ok(CachedAdcToken {
        access_token: parsed.access_token,
        token_type: parsed.token_type.unwrap_or_else(|| "Bearer".to_string()),
        expires_at_unix: now + expires_in,
        source_kind: cred.kind_label().to_string(),
    })
}

/// Returns a cached or freshly exchanged Google Cloud ADC OAuth2 bearer token
/// (cached in-memory with 60s expiry skew; zero shelling out per request).
pub async fn get_or_refresh_adc_token(explicit_path: Option<&Path>) -> Result<CachedAdcToken, String> {
    let path_buf = explicit_path
        .map(Path::to_path_buf)
        .or_else(default_adc_credentials_path)
        .ok_or_else(|| {
            "ADC credentials path could not be resolved. Run: gcloud auth application-default login --project <GCP_PROJECT> and toksi auth adc --project <id> --location us-central1".to_string()
        })?;

    let cache_key = crate::util::paths::normalize_path(&path_buf.to_string_lossy());
    let now = now_unix_secs();
    if let Some(cached) = get_cached_adc_token(&cache_key, now) {
        return Ok(cached);
    }

    let cred = parse_adc_credentials_file(&path_buf)?;
    let fresh = exchange_adc_token_async(&cred).await?;
    put_cached_adc_token(&cache_key, fresh.clone());
    Ok(fresh)
}

pub fn format_missing_adc_guidance(gcp_project: Option<&str>, adc_path: &str) -> String {
    let proj = gcp_project
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("<GCP_PROJECT>");
    format!(
        "ADC credentials or project missing (path='{}'). Run: gcloud auth application-default login --project {} and toksi auth adc --project {} --location {}",
        adc_path, proj, proj, DEFAULT_GCP_LOCATION
    )
}

pub fn inspect_adc_status() -> AdcStatus {
    let enabled = is_agy_adc_auth_enabled();
    let (mut gcp_project, gcp_location) = read_vscode_gcp_settings();
    let adc_path_buf = default_adc_credentials_path();
    let adc_path = adc_path_buf
        .as_ref()
        .map(|p| crate::util::paths::normalize_path(&p.to_string_lossy()))
        .unwrap_or_default();

    let parsed_cred = adc_path_buf
        .as_ref()
        .filter(|p| p.is_file())
        .and_then(|p| parse_adc_credentials_file(p).ok());

    if gcp_project.is_none() {
        if let Some(hint) = parsed_cred.as_ref().and_then(|c| c.project_hint()) {
            gcp_project = Some(hint.to_string());
        }
    }

    let adc_present = parsed_cred.is_some();
    let credential_kind = parsed_cred.map(|c| c.kind_label().to_string());

    let diagnostic = if !adc_present || (enabled && gcp_project.is_none()) {
        Some(format_missing_adc_guidance(
            gcp_project.as_deref(),
            &adc_path,
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
        credential_kind,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DualAgyAuthSnapshot {
    pub standard_agy_available: bool,
    pub vertex_adc_available: bool,
    pub concurrent_ready: bool,
    pub standard_route_key: String,
    pub vertex_adc_route_key: String,
    pub adc_location: String,
}

/// Inspects standard `agy` CLI OAuth/keyring availability concurrently with
/// `AGY_ADC_AUTH=true` Vertex ADC under separate route keys without overwriting either.
pub fn inspect_dual_agy_auth() -> DualAgyAuthSnapshot {
    let standard_agy_available = get_credential("gemini", "antigravity").is_some()
        || get_credential("agy", "default").is_some();
    let adc = inspect_adc_status();
    let vertex_adc_available = adc.enabled && adc.adc_present && adc.gcp_project.is_some();
    DualAgyAuthSnapshot {
        standard_agy_available,
        vertex_adc_available,
        concurrent_ready: standard_agy_available && vertex_adc_available,
        standard_route_key: "gemini:antigravity".to_string(),
        vertex_adc_route_key: "vertex-adc:default".to_string(),
        adc_location: adc.gcp_location,
    }
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
            assert!(diag.contains("toksi auth adc --project"));
        }
    }

    #[test]
    fn test_parse_authorized_user_adc() {
        let raw = r#"{
            "type": "authorized_user",
            "client_id": "cid-123",
            "client_secret": "sec-456",
            "refresh_token": "1//refresh-abc",
            "quota_project_id": "my-gcp-proj"
        }"#;
        let cred = parse_adc_credentials_json(raw).expect("valid authorized_user");
        assert_eq!(cred.kind_label(), "authorized_user");
        assert_eq!(cred.project_hint(), Some("my-gcp-proj"));
    }

    #[test]
    fn test_adc_token_cache_respects_60s_skew() {
        clear_adc_token_cache();
        let now = 1_700_000_000;
        put_cached_adc_token(
            "test-key",
            CachedAdcToken {
                access_token: "ya29.test".to_string(),
                token_type: "Bearer".to_string(),
                expires_at_unix: now + 120,
                source_kind: "authorized_user".to_string(),
            },
        );
        assert!(get_cached_adc_token("test-key", now).is_some());
        assert!(get_cached_adc_token("test-key", now + 61).is_none());
        clear_adc_token_cache();
    }

    #[test]
    fn test_inspect_dual_agy_auth_distinct_routes() {
        let snap = inspect_dual_agy_auth();
        assert_eq!(snap.standard_route_key, "gemini:antigravity");
        assert_eq!(snap.vertex_adc_route_key, "vertex-adc:default");
        assert!(!snap.adc_location.is_empty());
        assert_eq!(
            snap.concurrent_ready,
            snap.standard_agy_available && snap.vertex_adc_available
        );
    }
}
