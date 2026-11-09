// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/tier_check.rs
// created: 2026-11-09

use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TierCheckError {
    #[error("Database error during tier validation: {0}")]
    DbError(String),
    #[error("Provider '{0}' has no valid API key or credential configured")]
    MissingCredential(String),
    #[error("Network error during tier validation: {0}")]
    NetworkError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountTier {
    PaidCommercial,
    FreeTierDisqualified,
    Unknown,
}

static ACTIVE_PAID_PROVIDERS: AtomicUsize = AtomicUsize::new(0);

/// Return the count of currently verified active paid BYOK providers.
pub fn active_paid_provider_count() -> usize {
    ACTIVE_PAID_PROVIDERS.load(Ordering::Relaxed)
}

/// Set the count of currently verified active paid BYOK providers.
pub fn set_active_paid_provider_count(count: usize) {
    ACTIVE_PAID_PROVIDERS.store(count, Ordering::Relaxed)
}

/// Safety guard check: should gateway active LLM routing engage?
/// Only engages when at least 1 healthy paid commercial BYOK provider is verified.
pub fn is_gateway_routing_active() -> bool {
    active_paid_provider_count() > 0
}

/// Determine whether a provider key or credential indicates an active paid commercial tier.
/// Enforces zero panics and the Graceful Inform & Zero-Fail Policy:
/// Free-tier accounts (unbilled AI Studio trial keys, expired credits, free Copilot trials)
/// are disqualified from active rotation with a clear informative notice without erroring or crashing.
pub fn inspect_provider_tier(provider_name: &str, key_or_token: &str) -> AccountTier {
    let trimmed = key_or_token.trim();
    if trimmed.is_empty() {
        return AccountTier::Unknown;
    }

    let is_free = is_known_free_tier(provider_name, trimmed);
    if is_free {
        crate::util::logger::L.info(&format!(
            "[INFO] Provider '{}' is on a Free Tier (unbilled/rate-limited). Paid quota required for BYOK gateway pool. Bypassing from active pool without error.",
            provider_name
        ));
        AccountTier::FreeTierDisqualified
    } else {
        AccountTier::PaidCommercial
    }
}

/// Checks heuristics and metadata for known free, trial, or unbilled keys.
fn is_known_free_tier(provider_name: &str, key_or_token: &str) -> bool {
    let lower_prov = provider_name.to_lowercase();
    let lower_key = key_or_token.to_lowercase();

    // Explicit test keys
    if lower_key == "test"
        || lower_key.contains("free-tier")
        || lower_key.contains("trial")
        || lower_key.contains("unbilled")
    {
        return true;
    }

    // Google / Gemini AI Studio free tier heuristics
    if lower_prov.contains("gemini")
        || lower_prov.contains("antigravity")
        || lower_prov.contains("agy")
    {
        if lower_key.starts_with("aizasy-free") || lower_key.ends_with("-free") {
            return true;
        }
    }

    // GitHub Copilot free tier trial heuristics
    if lower_prov.contains("copilot") {
        if lower_key.contains("copilot-free") || lower_key.starts_with("gh_free_") {
            return true;
        }
    }

    // Anthropic / Claude trial keys
    if lower_prov.contains("claude") || lower_prov.contains("anthropic") {
        if lower_key.contains("trial-") || lower_key.contains("-free") {
            return true;
        }
    }

    // OpenAI / Codex trial keys
    if lower_prov.contains("openai") || lower_prov.contains("codex") {
        if lower_key.contains("trial") || lower_key.starts_with("sess-free-") {
            return true;
        }
    }

    // Grok trial keys
    if lower_prov.contains("grok") || lower_prov.contains("xai") {
        if lower_key.contains("free") || lower_key.starts_with("xai-trial-") {
            return true;
        }
    }

    false
}

/// Validates provider tier and updates SQLite provider_keys database status.
pub fn validate_and_update_key_tier(
    db: &mut crate::db::operations::ToksiConfigDb,
    provider_id: &str,
    key_alias: &str,
    key_or_token: &str,
) -> Result<AccountTier, TierCheckError> {
    let tier = inspect_provider_tier(provider_id, key_or_token);
    let status_str = match tier {
        AccountTier::PaidCommercial => "ok",
        AccountTier::FreeTierDisqualified => "FreeTierDisqualified",
        AccountTier::Unknown => "unknown",
    };

    // Update in database if present
    if let Ok(keys) = db.list_keys_for_provider(provider_id) {
        for mut k in keys {
            if k.key_alias == key_alias || k.secret_ref == key_or_token {
                k.last_status = status_str.to_string();
                if tier == AccountTier::FreeTierDisqualified {
                    k.is_active = false;
                }
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
    fn test_tier_check_paid_vs_free() {
        let paid = inspect_provider_tier("OpenAI", "sk-proj-live-paid-1234567890");
        assert_eq!(paid, AccountTier::PaidCommercial);

        let free = inspect_provider_tier("Gemini", "AIzaSy-free-trial-key");
        assert_eq!(free, AccountTier::FreeTierDisqualified);

        let free_copilot = inspect_provider_tier("Copilot", "gh_free_trial_token");
        assert_eq!(free_copilot, AccountTier::FreeTierDisqualified);
    }

    #[test]
    fn test_gateway_routing_active_guard() {
        set_active_paid_provider_count(0);
        assert!(!is_gateway_routing_active());

        set_active_paid_provider_count(2);
        assert!(is_gateway_routing_active());

        set_active_paid_provider_count(0);
    }
}
