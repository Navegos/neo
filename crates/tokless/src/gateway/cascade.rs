// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/cascade.rs

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureClass {
    Credential,
    Quota,
    RateLimit,
    ModelAccess,
    ModelGone,
    RegionBlocked,
    Transient,
    Timeout,
    Unknown,
}

impl FailureClass {
    /// Determines whether the failure should permanently rotate the active key immediately without retry
    pub fn is_permanent_key_failure(&self) -> bool {
        matches!(self, Self::Credential | Self::Quota | Self::ModelAccess)
    }

    /// Determines whether retry with backoff on the same or sibling key is allowed
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimit | Self::Transient | Self::Timeout)
    }
}

const RATE_LIMIT_PATTERNS: &[&str] = &[
    "rate limit", "rate_limit", "ratelimit", "too many requests", "request limit",
    "throttled", "throttling", "rate exceeded", "limit exceeded", "requests per",
    "rpm exceeded", "tpm exceeded", "tokens per minute", "requests per minute",
    "api rate limit", "concurrent requests limit",
];

const QUOTA_PATTERNS: &[&str] = &[
    "quota exceeded", "quota_exceeded", "credit balance", "spend limit",
    "api usage limits", "prepayment credits", "insufficient_quota", "out of credits",
    "billing", "payment required",
];

const CREDENTIAL_PATTERNS: &[&str] = &[
    "invalid_api_key", "unauthorized", "invalid key", "authentication error",
    "incorrect api key", "bad api key", "invalid authentication", "api key not found",
];

const REGION_PATTERNS: &[&str] = &[
    "region not supported", "location not supported", "country not supported",
    "not available in your region", "geoblocked", "geographic restriction",
];

const MODEL_GONE_PATTERNS: &[&str] = &[
    "model_not_found", "model not found", "does not exist", "decommissioned",
];

pub fn classify_failure(status: u16, headers: &HashMap<String, String>, body: &str) -> FailureClass {
    let lower_body = body.to_lowercase();

    // Check specific headers
    if status == 429 {
        if QUOTA_PATTERNS.iter().any(|p| lower_body.contains(p)) {
            return FailureClass::Quota;
        }
        return FailureClass::RateLimit;
    }

    if status == 401 {
        return FailureClass::Credential;
    }

    if status == 402 {
        return FailureClass::Quota;
    }

    if status == 408 {
        return FailureClass::Timeout;
    }

    if status == 404 && MODEL_GONE_PATTERNS.iter().any(|p| lower_body.contains(p)) {
        return FailureClass::ModelGone;
    }

    if status == 403 {
        if REGION_PATTERNS.iter().any(|p| lower_body.contains(p)) {
            return FailureClass::RegionBlocked;
        }
        if QUOTA_PATTERNS.iter().any(|p| lower_body.contains(p)) {
            return FailureClass::Quota;
        }
        if CREDENTIAL_PATTERNS.iter().any(|p| lower_body.contains(p)) {
            return FailureClass::Credential;
        }
        return FailureClass::ModelAccess;
    }

    if status == 529 || (status >= 500 && status <= 504) {
        return FailureClass::Transient;
    }

    // Pattern matches against response body
    if RATE_LIMIT_PATTERNS.iter().any(|p| lower_body.contains(p)) {
        return FailureClass::RateLimit;
    }

    if QUOTA_PATTERNS.iter().any(|p| lower_body.contains(p)) {
        return FailureClass::Quota;
    }

    if CREDENTIAL_PATTERNS.iter().any(|p| lower_body.contains(p)) {
        return FailureClass::Credential;
    }

    // Check headers for rate limit indications
    for (k, _) in headers {
        let lk = k.to_lowercase();
        if lk.contains("retry-after") || lk.contains("ratelimit") {
            return FailureClass::RateLimit;
        }
    }

    FailureClass::Unknown
}

/// Extracts suggested cooldown delay from Retry-After headers if available
pub fn parse_retry_after(headers: &HashMap<String, String>) -> Duration {
    for (k, v) in headers {
        let lk = k.to_lowercase();
        if lk == "retry-after" {
            if let Ok(sec) = v.parse::<u64>() {
                return Duration::from_secs(sec);
            }
        }
        if lk == "retry-after-ms" {
            if let Ok(ms) = v.parse::<u64>() {
                return Duration::from_millis(ms);
            }
        }
    }
    // Default jittered backoff for rate limits
    Duration::from_millis(1500)
}

