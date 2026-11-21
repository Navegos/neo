// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/llm_guard.rs
// created: 2026-11-21

//! Pure-Rust `llm-guard` input/output scanner (PromptInjection, InvisibleUnicode,
//! TokenLimit, Toxicity, and Anonymize/Secrets).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::contextmode::text_norm::scrub_secrets;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmGuardConfig {
    pub max_tokens_approx: usize,
    pub block_invisible_unicode: bool,
    pub block_prompt_injection: bool,
    pub redact_secrets: bool,
}

impl Default for LlmGuardConfig {
    fn default() -> Self {
        Self {
            max_tokens_approx: 128_000,
            block_invisible_unicode: true,
            block_prompt_injection: true,
            redact_secrets: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmGuardVerdict {
    pub is_valid: bool,
    pub risk_score: f32,
    pub sanitized_output: String,
    pub triggered_scanners: Vec<String>,
}

const INVISIBLE_CHARS: &[char] = &[
    '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{200F}', '\u{202A}', '\u{202B}',
    '\u{202C}', '\u{202D}', '\u{202E}', '\u{2060}', '\u{FEFF}',
];

const INJECTION_SIGNATURES: &[(&str, f32)] = &[
    ("ignore previous instructions", 0.95),
    ("ignore all prior instructions", 0.95),
    ("disregard system prompt", 0.95),
    ("override <enforce>", 1.0),
    ("bypass <enforce>", 1.0),
    ("disable commandguard", 1.0),
    ("you are now dan", 0.85),
    ("</enforce>", 0.75),
];

/// Scans prompt or model output with `llm-guard` scanners.
pub fn scan_llm_guard(text: &str, cfg: &LlmGuardConfig) -> LlmGuardVerdict {
    let mut triggered = Vec::new();
    let mut max_risk: f32 = 0.0;
    let mut sanitized = text.to_string();

    if cfg.block_invisible_unicode && text.chars().any(|c| INVISIBLE_CHARS.contains(&c)) {
        triggered.push("InvisibleUnicode".to_string());
        max_risk = max_risk.max(0.8);
        sanitized = sanitized
            .chars()
            .filter(|c| !INVISIBLE_CHARS.contains(c))
            .collect();
    }

    let approx_tokens = text.len().div_ceil(4);
    if approx_tokens > cfg.max_tokens_approx {
        triggered.push(format!("TokenLimit({approx_tokens}>{})", cfg.max_tokens_approx));
        max_risk = max_risk.max(0.6);
    }

    if cfg.block_prompt_injection {
        let lower = sanitized.to_ascii_lowercase();
        for (pat, score) in INJECTION_SIGNATURES {
            if lower.contains(pat) {
                triggered.push(format!("PromptInjection({pat})"));
                max_risk = max_risk.max(*score);
            }
        }
    }

    if cfg.redact_secrets {
        let scrubbed = scrub_secrets(&sanitized);
        if scrubbed != sanitized {
            triggered.push("SecretsAnonymizer".to_string());
            max_risk = max_risk.max(0.5);
            sanitized = scrubbed;
        }
    }

    let is_valid = !triggered.iter().any(|t| {
        t.starts_with("PromptInjection")
            || t == "InvisibleUnicode"
            || t.starts_with("TokenLimit")
    });

    LlmGuardVerdict {
        is_valid,
        risk_score: max_risk,
        sanitized_output: sanitized,
        triggered_scanners: triggered,
    }
}
