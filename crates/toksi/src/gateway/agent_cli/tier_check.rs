// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/tier_check.rs
// created: 2026-11-09
// lastModified: 2026-11-13

//! Open multi-instance BYOK eligibility & read-only quota telemetry.
//! Zero paid-tier restrictions: free, trial, hobby, OAuth, subscription, and PAYG
//! credentials are all eligible whenever explicitly configured by the user.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::db::operations::ToksiConfigDb;
use crate::db::schema::{ByokProfileRecord, ProviderModelRecord, QuotaWindowRecord};
use crate::util::logger::L;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TierCheckError {
    #[error("Database error during credential validation: {0}")]
    DbError(String),
    #[error("Provider '{0}' has no valid API key or credential configured")]
    MissingCredential(String),
    #[error("Network error during quota check: {0}")]
    NetworkError(String),
}

/// Non-blocking credential classification used strictly as an informational label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountTier {
    Configured,
    OAuthSubscription,
    PaygApiKey,
    FreeOrTrial,
    Unknown,
}

impl AccountTier {
    pub fn label(self) -> &'static str {
        match self {
            Self::Configured => "[BYOK]",
            Self::OAuthSubscription => "[OAUTH]",
            Self::PaygApiKey => "[API-KEY]",
            Self::FreeOrTrial => "[FREE-TIER]",
            Self::Unknown => "[UNCONFIGURED]",
        }
    }
}

/// Read-only quota telemetry window reported from upstream response headers or `/quota`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuotaWindowKind {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl QuotaWindowKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }
}

/// Configured-only gateway routing check: engages whenever >= 1 enabled BYOK profile or route exists.
pub fn is_gateway_routing_active() -> bool {
    !crate::headroom::byok_gateway::configured_byok_routes().is_empty()
}

/// Inspect credential shape purely for informational UI badges — never disqualifies a key.
pub fn inspect_provider_tier(_provider_name: &str, key_or_token: &str) -> AccountTier {
    let trimmed = key_or_token.trim();
    if trimmed.is_empty() {
        return AccountTier::Unknown;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("sk-ant-oat") || lower.starts_with("ya29.") || trimmed.split('.').count() == 3 {
        AccountTier::OAuthSubscription
    } else if lower.contains("free") || lower.contains("trial") {
        AccountTier::FreeOrTrial
    } else if lower.starts_with("sk-") || lower.starts_with("aiza") || lower.starts_with("xai-") {
        AccountTier::PaygApiKey
    } else {
        AccountTier::Configured
    }
}

/// Records informational credential status into SQLite without deactivating free/trial keys.
pub fn validate_and_update_key_tier(
    db: &mut crate::db::operations::ToksiConfigDb,
    provider_id: &str,
    key_alias: &str,
    key_or_token: &str,
) -> Result<AccountTier, TierCheckError> {
    let tier = inspect_provider_tier(provider_id, key_or_token);
    let status_str = match tier {
        AccountTier::Unknown => "unconfigured",
        _ => "ok",
    };

    if let Ok(keys) = db.list_keys_for_provider(provider_id) {
        for mut k in keys {
            if k.key_alias == key_alias || k.secret_ref == key_or_token {
                k.last_status = status_str.to_string();
                if let Err(e) = db.upsert_key(&k) {
                    return Err(TierCheckError::DbError(e.to_string()));
                }
            }
        }
    }

    Ok(tier)
}

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn parse_i64_header(headers: &HashMap<String, String>, keys: &[&str]) -> Option<i64> {
    for key in keys {
        if let Some(val) = headers.get(*key) {
            let trimmed = val.trim();
            if let Ok(n) = trimmed.parse::<i64>() {
                return Some(n);
            }
        }
    }
    None
}

fn infer_window_kind(headers: &HashMap<String, String>) -> QuotaWindowKind {
    if let Some(raw) = headers
        .get("x-ratelimit-window")
        .or_else(|| headers.get("x-quota-window"))
    {
        match raw.trim().to_ascii_lowercase().as_str() {
            "hour" | "1h" | "hourly" => return QuotaWindowKind::Hour,
            "day" | "1d" | "24h" | "daily" => return QuotaWindowKind::Day,
            "week" | "1w" | "7d" | "weekly" => return QuotaWindowKind::Week,
            "month" | "1m" | "30d" | "monthly" => return QuotaWindowKind::Month,
            "year" | "1y" | "365d" | "yearly" | "annual" => return QuotaWindowKind::Year,
            _ => {}
        }
    }
    if let Some(reset) = headers
        .get("x-ratelimit-reset-requests")
        .or_else(|| headers.get("x-ratelimit-reset-tokens"))
    {
        let lower = reset.trim().to_ascii_lowercase();
        if lower.contains('d') {
            return QuotaWindowKind::Day;
        }
        if lower.contains('h') {
            return QuotaWindowKind::Hour;
        }
    }
    QuotaWindowKind::Day
}

/// Extracts optional read-only plan/tier label exposed in response headers.
pub fn extract_readonly_plan_label(headers: &HashMap<String, String>) -> Option<String> {
    for key in ["x-ratelimit-plan", "x-account-tier", "x-plan-tier", "openai-organization"] {
        if let Some(v) = headers.get(key) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Parses `x-ratelimit-*` and `anthropic-ratelimit-*` headers into `QuotaWindowRecord`s.
/// Purely read-only telemetry; never blocks or disqualifies any account tier.
pub fn parse_ratelimit_headers(
    provider_id: &str,
    instance_label: &str,
    headers: &HashMap<String, String>,
) -> Vec<QuotaWindowRecord> {
    let lower_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
        .collect();

    let limit_requests = parse_i64_header(
        &lower_map,
        &["x-ratelimit-limit-requests", "anthropic-ratelimit-requests-limit"],
    );
    let remaining_requests = parse_i64_header(
        &lower_map,
        &["x-ratelimit-remaining-requests", "anthropic-ratelimit-requests-remaining"],
    );
    let limit_tokens = parse_i64_header(
        &lower_map,
        &["x-ratelimit-limit-tokens", "anthropic-ratelimit-tokens-limit"],
    );
    let remaining_tokens = parse_i64_header(
        &lower_map,
        &["x-ratelimit-remaining-tokens", "anthropic-ratelimit-tokens-remaining"],
    );

    if limit_requests.is_none()
        && remaining_requests.is_none()
        && limit_tokens.is_none()
        && remaining_tokens.is_none()
    {
        return Vec::new();
    }

    let reset_at_epoch = parse_i64_header(
        &lower_map,
        &[
            "x-ratelimit-reset-requests",
            "x-ratelimit-reset-tokens",
            "anthropic-ratelimit-requests-reset",
            "anthropic-ratelimit-tokens-reset",
        ],
    );

    let window = infer_window_kind(&lower_map);
    let now = now_epoch_secs();
    let label = if instance_label.trim().is_empty() {
        "default"
    } else {
        instance_label.trim()
    };

    vec![QuotaWindowRecord {
        id: format!("{}:{}:{}", provider_id, label, window.as_str()),
        provider_id: provider_id.to_string(),
        instance_label: label.to_string(),
        window_kind: window.as_str().to_string(),
        limit_requests,
        remaining_requests,
        limit_tokens,
        remaining_tokens,
        reset_at_epoch,
        updated_at: now,
    }]
}

/// Persists parsed rate-limit header telemetry into SQLite `quota_windows`.
pub fn record_ratelimit_headers(
    db: &mut ToksiConfigDb,
    provider_id: &str,
    instance_label: &str,
    headers: &HashMap<String, String>,
) -> Result<Vec<QuotaWindowRecord>, TierCheckError> {
    let windows = parse_ratelimit_headers(provider_id, instance_label, headers);
    for w in &windows {
        if let Err(e) = db.upsert_quota_window(w) {
            return Err(TierCheckError::DbError(e.to_string()));
        }
    }
    Ok(windows)
}

/// Parses OpenAI/Bifrost (`data: [...]`) or Gemini/Ollama (`models: [...]`) `/v1/models` JSON.
pub fn parse_models_response(provider_id: &str, body: &Value) -> Vec<ProviderModelRecord> {
    let now = now_epoch_secs();
    let arr = body
        .get("data")
        .and_then(Value::as_array)
        .or_else(|| body.get("models").and_then(Value::as_array));

    let Some(items) = arr else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            let id = item
                .get("id")
                .or_else(|| item.get("name"))
                .and_then(Value::as_str)?
                .trim();
            if id.is_empty() {
                return None;
            }
            let display_name = item
                .get("display_name")
                .or_else(|| item.get("displayName"))
                .and_then(Value::as_str)
                .unwrap_or(id)
                .to_string();
            let context_window = item
                .get("context_window")
                .or_else(|| item.get("context_length"))
                .or_else(|| item.get("inputTokenLimit"))
                .and_then(Value::as_i64)
                .unwrap_or(128_000);
            let max_output_tokens = item
                .get("max_output_tokens")
                .or_else(|| item.get("outputTokenLimit"))
                .and_then(Value::as_i64)
                .unwrap_or(8_192);
            let supports_streaming = item
                .get("supports_streaming")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let supports_tools = item
                .get("supports_tools")
                .and_then(Value::as_bool)
                .unwrap_or(true);

            Some(ProviderModelRecord {
                id: format!("{}:{}", provider_id, id),
                provider_id: provider_id.to_string(),
                display_name,
                context_window,
                max_output_tokens,
                supports_streaming,
                supports_tools,
                is_default: false,
                created_at: now,
            })
        })
        .collect()
}

/// Zero-cost `/v1/models` discovery against an upstream provider or Bifrost endpoint.
/// On HTTP 429 or unbilled responses, logs a non-blocking warning and returns `Err` cleanly.
pub fn discover_models_zero_cost(
    base_url: &str,
    api_key: Option<&str>,
    provider_id: &str,
    timeout: Duration,
) -> Result<Vec<ProviderModelRecord>, TierCheckError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let models_url = if trimmed.ends_with("/v1") {
        format!("{}/models", trimmed)
    } else {
        format!("{}/v1/models", trimmed)
    };

    let is_loopback =
        models_url.starts_with("http://127.0.0.1") || models_url.starts_with("http://localhost");
    let mut builder = reqwest::blocking::Client::builder().timeout(timeout);
    if is_loopback {
        builder = builder.no_proxy();
    }
    let client = builder
        .build()
        .map_err(|e| TierCheckError::NetworkError(e.to_string()))?;

    let mut req = client.get(&models_url);
    if let Some(key) = api_key {
        let k = key.trim();
        if !k.is_empty() {
            req = req.bearer_auth(k);
        }
    }

    let resp = req
        .send()
        .map_err(|e| TierCheckError::NetworkError(e.to_string()))?;
    let status = resp.status().as_u16();
    if status == 429 || status == 402 {
        L.warn(&format!(
            "Zero-cost /v1/models discovery for '{}' hit HTTP {}; skipping cleanly",
            provider_id, status
        ));
        return Err(TierCheckError::NetworkError(format!(
            "HTTP {} during /v1/models discovery",
            status
        )));
    }
    if !resp.status().is_success() {
        L.debug(&format!(
            "Non-200 /v1/models response from '{}' ({}): HTTP {}",
            provider_id, models_url, status
        ));
        return Err(TierCheckError::NetworkError(format!(
            "HTTP {} from {}",
            status, models_url
        )));
    }

    let json_val: Value = resp
        .json()
        .map_err(|e| TierCheckError::NetworkError(e.to_string()))?;
    Ok(parse_models_response(provider_id, &json_val))
}

/// Persists discovered models into SQLite `provider_models`.
pub fn sync_discovered_models(
    db: &mut ToksiConfigDb,
    models: &[ProviderModelRecord],
) -> Result<usize, TierCheckError> {
    let mut count = 0;
    for m in models {
        if let Err(e) = db.upsert_provider_model(m) {
            return Err(TierCheckError::DbError(e.to_string()));
        }
        count += 1;
    }
    Ok(count)
}

/// Zero-fail 429/unbilled fallback selector: returns the highest-priority enabled
/// `<byok, model, effort>` profile not in `exhausted_ids`.
pub fn select_next_enabled_profile<'a>(
    profiles: &'a [ByokProfileRecord],
    exhausted_ids: &[String],
) -> Option<&'a ByokProfileRecord> {
    profiles
        .iter()
        .filter(|p| p.is_active && !exhausted_ids.iter().any(|ex| ex == &p.id))
        .max_by_key(|p| (p.priority, p.weight))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_tier_classification_never_disqualifies() {
        assert_eq!(
            inspect_provider_tier("OpenAI", "sk-proj-live-123456"),
            AccountTier::PaygApiKey
        );
        assert_eq!(
            inspect_provider_tier("Gemini", "AIzaSy-free-trial-key"),
            AccountTier::FreeOrTrial
        );
        assert_eq!(
            inspect_provider_tier("Anthropic", "sk-ant-oat01-token"),
            AccountTier::OAuthSubscription
        );
        assert_eq!(inspect_provider_tier("OpenAI", ""), AccountTier::Unknown);
    }

    #[test]
    fn test_parse_ratelimit_headers_populates_quota_windows() {
        let mut headers = HashMap::new();
        headers.insert("x-ratelimit-limit-requests".to_string(), "100".to_string());
        headers.insert("x-ratelimit-remaining-requests".to_string(), "42".to_string());
        headers.insert("x-ratelimit-limit-tokens".to_string(), "100000".to_string());
        headers.insert("x-ratelimit-remaining-tokens".to_string(), "85000".to_string());
        headers.insert("x-ratelimit-window".to_string(), "hour".to_string());
        headers.insert("x-ratelimit-plan".to_string(), "free-tier".to_string());

        let windows = parse_ratelimit_headers("openai", "personal", &headers);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].window_kind, "hour");
        assert_eq!(windows[0].limit_requests, Some(100));
        assert_eq!(windows[0].remaining_requests, Some(42));
        assert_eq!(windows[0].limit_tokens, Some(100000));
        assert_eq!(windows[0].remaining_tokens, Some(85000));
        assert_eq!(
            extract_readonly_plan_label(&headers),
            Some("free-tier".to_string())
        );
    }

    #[test]
    fn test_parse_models_response_and_profile_failover() {
        let body = serde_json::json!({
            "data": [
                {"id": "gpt-4o", "context_window": 128000, "max_output_tokens": 16384},
                {"id": "claude-3-7-sonnet", "supports_tools": true}
            ]
        });
        let models = parse_models_response("bifrost", &body);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "bifrost:gpt-4o");
        assert_eq!(models[0].context_window, 128000);

        let profiles = vec![
            ByokProfileRecord {
                id: "p1".to_string(),
                provider_id: "openai".to_string(),
                instance_label: "free".to_string(),
                model_id: "gpt-4o-mini".to_string(),
                thinking_effort: "low".to_string(),
                base_url: String::new(),
                secret_ref: "sk-1".to_string(),
                auth_mode: "api_key".to_string(),
                gcp_project: String::new(),
                gcp_location: "us-central1".to_string(),
                priority: 100,
                weight: 10,
                headroom_enabled: true,
                is_active: true,
                created_at: 0,
                updated_at: 0,
                ..Default::default()
            },
            ByokProfileRecord {
                id: "p2".to_string(),
                provider_id: "anthropic".to_string(),
                instance_label: "backup".to_string(),
                model_id: "claude-3-7-sonnet".to_string(),
                thinking_effort: "medium".to_string(),
                base_url: String::new(),
                secret_ref: "sk-2".to_string(),
                auth_mode: "api_key".to_string(),
                gcp_project: String::new(),
                gcp_location: "us-central1".to_string(),
                priority: 50,
                weight: 5,
                headroom_enabled: true,
                is_active: true,
                created_at: 0,
                updated_at: 0,
                ..Default::default()
            },
        ];
        let next = select_next_enabled_profile(&profiles, &["p1".to_string()]);
        assert_eq!(next.map(|p| p.id.as_str()), Some("p2"));
    }
}
