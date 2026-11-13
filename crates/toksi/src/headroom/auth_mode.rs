// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/auth_mode.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Authentication mode classifier ported from `headroom-core::auth_mode`.

/// Originating authentication billing/routing mode of an LLM API request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthMode {
    /// Pay-as-you-go API key (`x-api-key`, `x-goog-api-key`, `Authorization: Bearer sk-...`).
    Payg,
    /// OAuth token (`anthropic-oauth-*`, Google OAuth `ya29.*`, Copilot OAuth).
    OAuth,
    /// Seat/subscription session (`claude.ai`, session cookies).
    Subscription,
}

impl AuthMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Payg => "payg",
            Self::OAuth => "oauth",
            Self::Subscription => "subscription",
        }
    }

    /// OAuth and Subscription requests must never have upstream `cache_control` headers
    /// or system-prompt prefix mutations injected that could invalidate account cache state.
    pub const fn should_mutate_cache_control(self) -> bool {
        matches!(self, Self::Payg)
    }

    /// Live-zone tail compression preserves frozen prompt prefixes across all modes.
    pub const fn should_compress_live_zone(self) -> bool {
        true
    }
}

/// Classifies request auth mode from HTTP headers (case-insensitive).
pub fn classify_auth_mode(headers: &[(String, String)]) -> AuthMode {
    let mut has_api_key = false;
    let mut has_oauth = false;
    let mut has_sub = false;

    for (name, val) in headers {
        let lower = name.to_ascii_lowercase();
        let trimmed = val.trim();
        if trimmed.is_empty() {
            continue;
        }
        if lower == "x-api-key" || lower == "x-goog-api-key" || lower == "api-key" {
            has_api_key = true;
            continue;
        }
        if lower.starts_with("anthropic-oauth") || lower == "x-copilot-token" {
            has_oauth = true;
            continue;
        }
        if lower == "cookie" && trimmed.contains("sessionKey") {
            has_sub = true;
            continue;
        }
        if lower == "origin" || lower == "referer" {
            let vlow = trimmed.to_ascii_lowercase();
            if vlow.contains("claude.ai") || vlow.contains("chatgpt.com") {
                has_sub = true;
                continue;
            }
        }
        if lower == "authorization" {
            if let Some(token) = trimmed.strip_prefix("Bearer ").or_else(|| trimmed.strip_prefix("bearer ")) {
                let t = token.trim();
                if t.starts_with("ya29.") || t.starts_with("sk-ant-oat") || t.starts_with("ghu_") || t.starts_with("gho_") {
                    has_oauth = true;
                } else {
                    has_api_key = true;
                }
            }
        }
    }

    if has_sub {
        AuthMode::Subscription
    } else if has_oauth && !has_api_key {
        AuthMode::OAuth
    } else {
        AuthMode::Payg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_auth_mode_variants() {
        assert_eq!(
            classify_auth_mode(&[("x-api-key".into(), "sk-ant-api03-abc".into())]),
            AuthMode::Payg
        );
        assert_eq!(
            classify_auth_mode(&[("Authorization".into(), "Bearer ya29.test".into())]),
            AuthMode::OAuth
        );
        assert_eq!(
            classify_auth_mode(&[("Cookie".into(), "sessionKey=sk-ant-sid01".into())]),
            AuthMode::Subscription
        );
        assert!(!AuthMode::OAuth.should_mutate_cache_control());
        assert!(AuthMode::Payg.should_mutate_cache_control());
    }
}
