// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/text_norm.rs
// created: 2026-11-18

//! Deterministic WFST-style text normalization (NeMo-text-processing parity)
//! and PII/secret pre-scrubbing for Context-Mode indexing and sandbox output.

use std::sync::LazyLock;
use regex::Regex;

static SECRET_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    let specs: &[(&str, &str)] = &[
        (r"sk-[A-Za-z0-9]{20,}", "[REDACTED_API_KEY]"),
        (r"ghp_[A-Za-z0-9]{30,}", "[REDACTED_GH_TOKEN]"),
        (r"github_pat_[A-Za-z0-9_]{30,}", "[REDACTED_GH_PAT]"),
        (r"AIza[0-9A-Za-z\-_]{35}", "[REDACTED_GCP_KEY]"),
        (r"AKIA[0-9A-Z]{16}", "[REDACTED_AWS_KEY]"),
        (r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*", "Bearer [REDACTED_TOKEN]"),
        (
            r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
            "[REDACTED_PRIVATE_KEY]",
        ),
    ];
    specs
        .iter()
        .filter_map(|(pat, rep)| Regex::new(pat).ok().map(|re| (re, *rep)))
        .collect()
});

static MULTI_BLANK_LINES: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\n{3,}").ok());

/// Deterministic text normalization:
/// - Normalizes `\r\n` and `\r` to `\n`
/// - Strips zero-width Unicode characters (`\u{200B}`, `\u{200C}`, `\u{200D}`, `\u{FEFF}`)
/// - Normalizes smart quotes and non-breaking spaces to ASCII equivalents
/// - Collapses runs of 3+ blank lines to 2 newlines
pub fn normalize_text(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    let _ = chars.next();
                }
                out.push('\n');
            }
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
            '\u{00A0}' => out.push(' '),
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201C}' | '\u{201D}' => out.push('"'),
            '\u{2013}' | '\u{2014}' => out.push('-'),
            _ => out.push(ch),
        }
    }

    if let Some(re) = MULTI_BLANK_LINES.as_ref() {
        re.replace_all(&out, "\n\n").into_owned()
    } else {
        out
    }
}

/// Scrubs common secret/credential tokens from text before indexing or returning to model context.
pub fn scrub_secrets(input: &str) -> String {
    let mut current = input.to_string();
    for (re, replacement) in SECRET_PATTERNS.iter() {
        if re.is_match(&current) {
            current = re.replace_all(&current, *replacement).into_owned();
        }
    }
    current
}

/// Combined deterministic normalization and secret pre-scrubbing pipeline.
pub fn normalize_and_scrub(input: &str) -> String {
    let normalized = normalize_text(input);
    scrub_secrets(&normalized)
}

/// WFST-style unit/whitespace canonicalization (`NeMo-text-processing` parity).
pub fn wfst_canonicalize(input: &str) -> String {
    let norm = normalize_and_scrub(input);
    norm.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Deterministic 64-bit MinHash-style shingle fingerprint (`Curator` / `DALI` dedup parity).
pub fn minhash_fingerprint(input: &str, shingle_words: usize) -> u64 {
    let canon = wfst_canonicalize(input).to_ascii_lowercase();
    let words: Vec<&str> = canon.split_whitespace().collect();
    if words.is_empty() {
        return 0;
    }
    let width = shingle_words.max(1).min(words.len());
    let mut min_hash = u64::MAX;
    for win in words.windows(width) {
        let mut h: u64 = 0xcbf29ce484222325;
        for w in win {
            for b in w.as_bytes() {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x100000001b3);
            }
            h ^= 0x20;
            h = h.wrapping_mul(0x100000001b3);
        }
        if h < min_hash {
            min_hash = h;
        }
    }
    min_hash
}
