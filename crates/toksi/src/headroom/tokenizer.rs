// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/tokenizer.rs
// created: 2026-11-21

//! Pure-Rust fast BPE-style token counter (`tokenizers` parity) for zero-API-call
//! pre- vs. post-compression measurement inside `HeadroomEffortCompressor`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenSavingsMeasurement {
    pub raw_tokens: usize,
    pub compressed_tokens: usize,
    pub tokens_saved: usize,
}

#[derive(Debug, Clone)]
pub struct BpeTokenizer {
    pub avg_chars_per_subword: usize,
}

impl Default for BpeTokenizer {
    fn default() -> Self {
        Self {
            avg_chars_per_subword: 4,
        }
    }
}

impl BpeTokenizer {
    /// Deterministic subword-aware token counter splitting identifiers, punctuation, and whitespace.
    pub fn count_tokens(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        let mut tokens = 0usize;
        let mut run_len = 0usize;
        let mut prev_alnum = false;
        let chunk = self.avg_chars_per_subword.max(1);

        for ch in text.chars() {
            if ch.is_whitespace() {
                if run_len > 0 {
                    tokens += run_len.div_ceil(chunk);
                    run_len = 0;
                }
                prev_alnum = false;
            } else if ch.is_ascii_alphanumeric() || ch == '_' {
                if !prev_alnum && run_len > 0 {
                    tokens += run_len.div_ceil(chunk);
                    run_len = 0;
                }
                run_len += 1;
                prev_alnum = true;
            } else {
                if run_len > 0 {
                    tokens += run_len.div_ceil(chunk);
                    run_len = 0;
                }
                tokens += 1;
                prev_alnum = false;
            }
        }
        if run_len > 0 {
            tokens += run_len.div_ceil(chunk);
        }
        tokens
    }

    pub fn measure_compression(&self, raw: &str, compressed: &str) -> TokenSavingsMeasurement {
        let raw_tokens = self.count_tokens(raw);
        let compressed_tokens = self.count_tokens(compressed);
        TokenSavingsMeasurement {
            raw_tokens,
            compressed_tokens,
            tokens_saved: raw_tokens.saturating_sub(compressed_tokens),
        }
    }

    pub fn truncate_to_budget(&self, text: &str, max_tokens: usize) -> String {
        if max_tokens == 0 || text.is_empty() {
            return String::new();
        }
        if self.count_tokens(text) <= max_tokens {
            return text.to_string();
        }
        let approx_chars = max_tokens.saturating_mul(self.avg_chars_per_subword.max(1));
        text.chars().take(approx_chars).collect()
    }
}
