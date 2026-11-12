// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/redaction.rs
// created: 2026-10-27
// lastModified: 2026-11-12

#![deny(unsafe_code)]

use std::borrow::Cow;
use std::env;
use std::sync::LazyLock;
use regex::Regex;
use super::schema::Event;

struct PatternDef {
    name: &'static str,
    regex: Regex,
}

static PATTERNS: LazyLock<Vec<PatternDef>> = LazyLock::new(|| {
    let raw = [
        ("openai_key", r"\bsk-[A-Za-z0-9_-]{40,}\b"),
        ("github_pat", r"\bgithub_pat_[A-Za-z0-9_]{82}\b"),
        ("github_token", r"\bgh[pousr]_[A-Za-z0-9]{36}\b"),
        ("aws_access_key", r"\bAKIA[0-9A-Z]{16}\b"),
        ("google_api_key", r"\bAIza[0-9A-Za-z_-]{35}\b"),
        ("slack_token", r"\bxox[abprs]-[A-Za-z0-9-]{10,}\b"),
        ("stripe_key", r"\b(?:sk|pk|rk)_(?:live|test)_[A-Za-z0-9]{24,}\b"),
        ("jwt", r"\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{10,}\b"),
        ("bearer_token", r"(?i)\bbearer\s+[A-Za-z0-9._~+/=\-]{20,}"),
        ("private_key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    ];

    let mut patterns = Vec::with_capacity(raw.len());
    for (name, pat) in raw {
        if let Ok(regex) = Regex::new(pat) {
            patterns.push(PatternDef { name, regex });
        }
    }
    patterns
});

pub fn is_redaction_enabled() -> bool {
    let val = env::var("PROJECTMEM_NO_REDACT").unwrap_or_default().trim().to_lowercase();
    val != "1" && val != "true" && val != "yes"
}

/// Zero-allocation secret redactor using Cow: borrows text directly if no secrets match.
pub fn redact_cow<'a>(text: &'a str) -> (Cow<'a, str>, Vec<String>) {
    if text.is_empty() {
        return (Cow::Borrowed(text), Vec::new());
    }

    // Quick check if any pattern matches before allocating
    let mut any_match = false;
    for p in PATTERNS.iter() {
        if p.regex.is_match(text) {
            any_match = true;
            break;
        }
    }

    if !any_match {
        return (Cow::Borrowed(text), Vec::new());
    }

    let mut out = text.to_string();
    let mut fired = Vec::new();

    for p in PATTERNS.iter() {
        let repl = format!("[REDACTED:{}]", p.name);
        let matches: Vec<_> = p.regex.find_iter(&out).collect();
        if !matches.is_empty() {
            for _ in &matches {
                fired.push(p.name.to_string());
            }
            out = p.regex.replace_all(&out, repl.as_str()).to_string();
        }
    }

    (Cow::Owned(out), fired)
}

pub fn redact(text: &str) -> (String, Vec<String>) {
    let (cow, fired) = redact_cow(text);
    (cow.into_owned(), fired)
}

pub fn redact_event_fields(event: &mut Event) -> Vec<String> {
    if !is_redaction_enabled() {
        return Vec::new();
    }

    let mut all_fired = Vec::new();

    let (s, f1) = redact(&event.summary);
    event.summary = s;
    all_fired.extend(f1);

    if let Some(ref n) = event.notes {
        let (s, f2) = redact(n);
        event.notes = Some(s);
        all_fired.extend(f2);
    }

    if let Some(ref l) = event.location {
        let (s, f3) = redact(l);
        event.location = Some(s);
        all_fired.extend(f3);
    }

    if let Some(ref c) = event.command {
        let (s, f4) = redact(c);
        event.command = Some(s);
        all_fired.extend(f4);
    }

    all_fired
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_alloc_borrow() {
        let clean = "This is a clean string without any tokens or keys.";
        let (cow, fired) = redact_cow(clean);
        assert!(matches!(cow, Cow::Borrowed(_)));
        assert!(fired.is_empty());
    }

    #[test]
    fn test_redact_patterns() {
        let raw = "sk-1234567890abcdef1234567890abcdef12345678 and AKIAIOSFODNN7EXAMPLE";
        let (redacted, fired) = redact(raw);
        assert!(redacted.contains("[REDACTED:openai_key]"));
        assert!(redacted.contains("[REDACTED:aws_access_key]"));
        assert_eq!(fired.len(), 2);
    }
}
