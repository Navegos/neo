// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/redaction.rs
// created: 2026-10-27

use std::env;
use std::sync::LazyLock;
use regex::Regex;
use super::schema::Event;

struct PatternDef {
    name: &'static str,
    regex: Regex,
}

static PATTERNS: LazyLock<Vec<PatternDef>> = LazyLock::new(|| {
    vec![
        PatternDef {
            name: "openai_key",
            regex: Regex::new(r"\bsk-[A-Za-z0-9_-]{40,}\b").unwrap(),
        },
        PatternDef {
            name: "github_pat",
            regex: Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{82}\b").unwrap(),
        },
        PatternDef {
            name: "github_token",
            regex: Regex::new(r"\bgh[pousr]_[A-Za-z0-9]{36}\b").unwrap(),
        },
        PatternDef {
            name: "aws_access_key",
            regex: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
        },
        PatternDef {
            name: "google_api_key",
            regex: Regex::new(r"\bAIza[0-9A-Za-z_-]{35}\b").unwrap(),
        },
        PatternDef {
            name: "slack_token",
            regex: Regex::new(r"\bxox[abprs]-[A-Za-z0-9-]{10,}\b").unwrap(),
        },
        PatternDef {
            name: "stripe_key",
            regex: Regex::new(r"\b(?:sk|pk|rk)_(?:live|test)_[A-Za-z0-9]{24,}\b").unwrap(),
        },
        PatternDef {
            name: "jwt",
            regex: Regex::new(r"\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{10,}\b").unwrap(),
        },
        PatternDef {
            name: "bearer_token",
            regex: Regex::new(r"\b[Bb]earer\s+[A-Za-z0-9._~+/=\-]{20,}").unwrap(),
        },
        PatternDef {
            name: "private_key",
            regex: Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").unwrap(),
        },
    ]
});

pub fn is_redaction_enabled() -> bool {
    let val = env::var("PROJECTMEM_NO_REDACT").unwrap_or_default().trim().to_lowercase();
    val != "1" && val != "true" && val != "yes"
}

pub fn redact(text: &str) -> (String, Vec<String>) {
    if text.is_empty() {
        return (text.to_string(), Vec::new());
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

    (out, fired)
}

pub fn redact_event_fields(event: &mut Event) -> Vec<String> {
    if !is_redaction_enabled() {
        return Vec::new();
    }

    let mut all_fired = Vec::new();

    let (s, f) = redact(&event.summary);
    if !f.is_empty() {
        event.summary = s;
        all_fired.extend(f);
    }

    if let Some(ref notes) = event.notes {
        let (s, f) = redact(notes);
        if !f.is_empty() {
            event.notes = Some(s);
            all_fired.extend(f);
        }
    }

    if let Some(ref cmd) = event.command {
        let (s, f) = redact(cmd);
        if !f.is_empty() {
            event.command = Some(s);
            all_fired.extend(f);
        }
    }

    if let Some(ref gm) = event.git_message {
        let (s, f) = redact(gm);
        if !f.is_empty() {
            event.git_message = Some(s);
            all_fired.extend(f);
        }
    }

    if let Some(ref loc) = event.location {
        let (s, f) = redact(loc);
        if !f.is_empty() {
            event.location = Some(s);
            all_fired.extend(f);
        }
    }

    all_fired
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_patterns() {
        let raw = "Tried fixing with sk-1234567890123456789012345678901234567890 and Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let (redacted, fired) = redact(raw);
        assert!(redacted.contains("[REDACTED:openai_key]"));
        assert!(redacted.contains("[REDACTED:bearer_token]") || redacted.contains("[REDACTED:jwt]"));
        assert!(!fired.is_empty());
    }

    #[test]
    fn test_redact_event_fields() {
        let mut evt = Event::new("issue", "Bug with AKIA1234567890123456 key");
        let fired = redact_event_fields(&mut evt);
        assert_eq!(fired, vec!["aws_access_key"]);
        assert_eq!(evt.summary, "Bug with [REDACTED:aws_access_key] key");
    }
}

