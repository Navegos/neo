// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/cascade.rs
// created: 2026-10-31
// lastModified: 2026-10-31

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_failure_8_categories() {
        let headers = HashMap::new();

        // 1. RateLimit (429)
        assert_eq!(classify_failure(429, &headers, "rate limit exceeded"), FailureClass::RateLimit);

        // 2. Quota (402 or 429 with quota message)
        assert_eq!(classify_failure(402, &headers, "Payment required"), FailureClass::Quota);
        assert_eq!(classify_failure(429, &headers, "insufficient_quota: out of credits"), FailureClass::Quota);

        // 3. Credential (401 or 403 invalid key)
        assert_eq!(classify_failure(401, &headers, "Unauthorized"), FailureClass::Credential);
        assert_eq!(classify_failure(403, &headers, "invalid_api_key provided"), FailureClass::Credential);

        // 4. ModelGone (404 with decommissioned or model not found)
        assert_eq!(classify_failure(404, &headers, "model_not_found: gpt-3.5 is decommissioned"), FailureClass::ModelGone);

        // 5. RegionBlocked (403 region not supported)
        assert_eq!(classify_failure(403, &headers, "location not supported in your country"), FailureClass::RegionBlocked);

        // 6. ModelAccess (403 default)
        assert_eq!(classify_failure(403, &headers, "Forbidden"), FailureClass::ModelAccess);

        // 7. Transient (500, 502, 503, 504, 529)
        assert_eq!(classify_failure(503, &headers, "Service Unavailable"), FailureClass::Transient);
        assert_eq!(classify_failure(529, &headers, "Overloaded"), FailureClass::Transient);

        // 8. Timeout (408)
        assert_eq!(classify_failure(408, &headers, "Request Timeout"), FailureClass::Timeout);
    }

    #[test]
    fn test_failure_class_behavior_flags() {
        assert!(FailureClass::Credential.is_permanent_key_failure());
        assert!(FailureClass::Quota.is_permanent_key_failure());
        assert!(FailureClass::ModelAccess.is_permanent_key_failure());
        assert!(!FailureClass::RateLimit.is_permanent_key_failure());

        assert!(FailureClass::RateLimit.is_retryable());
        assert!(FailureClass::Transient.is_retryable());
        assert!(FailureClass::Timeout.is_retryable());
        assert!(!FailureClass::Credential.is_retryable());
    }

    #[test]
    fn test_parse_retry_after() {
        let mut headers = HashMap::new();
        headers.insert("retry-after".to_string(), "5".to_string());
        assert_eq!(parse_retry_after(&headers), Duration::from_secs(5));

        let mut headers_ms = HashMap::new();
        headers_ms.insert("retry-after-ms".to_string(), "250".to_string());
        assert_eq!(parse_retry_after(&headers_ms), Duration::from_millis(250));

        let empty = HashMap::new();
        assert_eq!(parse_retry_after(&empty), Duration::from_millis(1500));
    }
}

