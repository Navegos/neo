// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/tier_check.rs
// created: 2026-11-09
// lastModified: 2026-11-13

//! Open multi-instance BYOK eligibility & read-only quota telemetry.
//! Zero paid-tier restrictions: free, trial, hobby, OAuth, subscription, and PAYG
//! credentials are all eligible whenever explicitly configured by the user.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
}
