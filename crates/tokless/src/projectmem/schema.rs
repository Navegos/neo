// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/schema.rs
// created: 2026-10-27

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const VALID_EVENT_TYPES: &[&str] = &[
    "issue",
    "hypothesis",
    "attempt",
    "fix",
    "decision",
    "note",
];

pub const VALID_OUTCOMES: &[&str] = &["worked", "failed", "partial"];

pub const VALID_CAPTURE_SOURCES: &[&str] = &[
    "git_post_commit",
    "git_post_revert",
    "git_post_merge",
    "churn_detector",
    "ci_parser",
];

pub const VALID_CONFIDENCE_LEVELS: &[&str] = &["high", "medium", "low"];

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn default_event_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let counter = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let hex_part = format!("{:010x}{:06x}{:04x}", now.as_secs(), (now.subsec_nanos() >> 8) & 0xffffff, counter & 0xffff);
    format!("evt_{}", &hex_part[..20])
}

pub fn utc_now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn normalize_timestamp(ts: Option<&str>) -> String {
    let raw = match ts {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return utc_now_iso(),
    };

    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return dt.with_timezone(&Utc).format("%Y-%m-%dT%H:%M:%SZ").to_string();
    }
    // Accept Git's `%ai` format: "YYYY-MM-DD HH:MM:SS ±HHMM"
    if let Ok(dt) = DateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S %z") {
        return dt.with_timezone(&Utc).format("%Y-%m-%dT%H:%M:%SZ").to_string();
    }
    // If it cannot be parsed, return as-is
    raw.to_string()
}

pub fn location_to_file(location: Option<&str>) -> Option<String> {
    let raw = location?.trim().trim_matches(|c| c == '"' || c == '\'');
    if raw.is_empty() {
        return None;
    }

    let mut loc = raw.to_string();
    let mut drive = String::new();

    // Check Windows drive letter: C:\ or C:/
    let chars: Vec<char> = loc.chars().collect();
    if chars.len() >= 3 && chars[0].is_ascii_alphabetic() && chars[1] == ':' && (chars[2] == '\\' || chars[2] == '/') {
        drive = loc[..2].to_string();
        loc = loc[2..].to_string();
    }

    // Strip trailing line/col suffix :123 or :123:45
    let line_re = Regex::new(r":\d+(?::\d+)?$").unwrap();
    let stripped = line_re.replace(&loc, "").to_string();
    if stripped.is_empty() {
        return None;
    }

    let candidate = format!("{}{}", drive, stripped);
    if stripped.contains('/') || stripped.contains('\\') {
        return Some(candidate);
    }

    // No separator: only recognized source suffixes count as a file
    if let Some(pos) = stripped.rfind('.') {
        let suffix = stripped[pos + 1..].to_lowercase();
        const SOURCE_SUFFIXES: &[&str] = &[
            "py", "pyi", "js", "jsx", "ts", "tsx", "mjs", "cjs", "rs", "go", "rb", "php",
            "java", "kt", "kts", "swift", "c", "h", "cc", "cpp", "hpp", "cs", "m", "mm",
            "sh", "bash", "zsh", "ps1", "sql", "html", "css", "scss", "sass", "vue",
            "svelte", "json", "yaml", "yml", "toml", "ini", "cfg", "conf", "xml", "md",
            "rst", "txt", "lock", "dockerfile", "gradle", "tf", "proto", "graphql", "ex",
            "exs", "erl", "hs", "scala", "clj", "lua", "r", "jl", "dart", "zig", "nim",
        ];
        if SOURCE_SUFFIXES.contains(&suffix.as_str()) {
            return Some(candidate);
        }
    }

    None
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attempt {
    pub summary: String,
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fix {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl From<Issue> for Event {
    fn from(i: Issue) -> Self {
        let mut ev = Event::new("issue", &i.summary);
        ev.location = i.location;
        ev
    }
}

impl From<Attempt> for Event {
    fn from(a: Attempt) -> Self {
        let mut ev = Event::new("attempt", &a.summary);
        ev.outcome = Some(a.outcome);
        ev.issue_id = a.issue_id;
        ev.location = a.location;
        ev.command = a.command;
        ev
    }
}

impl From<Fix> for Event {
    fn from(f: Fix) -> Self {
        let mut ev = Event::new("fix", &f.summary);
        ev.issue_id = f.issue_id;
        ev.location = f.location;
        ev
    }
}

impl From<Decision> for Event {
    fn from(d: Decision) -> Self {
        let mut ev = Event::new("decision", &d.summary);
        ev.supersedes = d.supersedes;
        ev.location = d.location;
        ev
    }
}

impl From<Note> for Event {
    fn from(n: Note) -> Self {
        let mut ev = Event::new("note", &n.summary);
        ev.location = n.location;
        ev
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(default = "default_event_id")]
    pub id: String,
    #[serde(default = "utc_now_iso")]
    pub timestamp: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto_captured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_confidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
}

impl Event {
    pub fn new(event_type: &str, summary: &str) -> Self {
        Self {
            id: default_event_id(),
            timestamp: utc_now_iso(),
            event_type: event_type.to_string(),
            issue_id: None,
            summary: summary.trim().to_string(),
            outcome: None,
            files: Vec::new(),
            command: None,
            notes: None,
            git_commit: None,
            location: None,
            auto_captured: false,
            capture_source: None,
            capture_confidence: None,
            git_message: None,
            supersedes: None,
        }
    }

    pub fn issue(summary: &str, location: Option<&str>) -> Self {
        let mut ev = Self::new("issue", summary);
        ev.location = location.map(|s| s.to_string());
        ev
    }

    pub fn attempt(
        summary: &str,
        outcome: &str,
        issue_id: Option<&str>,
        location: Option<&str>,
        command: Option<&str>,
    ) -> Self {
        let mut ev = Self::new("attempt", summary);
        ev.outcome = Some(outcome.to_string());
        ev.issue_id = issue_id.map(|s| s.to_string());
        ev.location = location.map(|s| s.to_string());
        ev.command = command.map(|s| s.to_string());
        ev
    }

    pub fn fix(summary: &str, issue_id: Option<&str>, location: Option<&str>) -> Self {
        let mut ev = Self::new("fix", summary);
        ev.issue_id = issue_id.map(|s| s.to_string());
        ev.location = location.map(|s| s.to_string());
        ev
    }

    pub fn decision(summary: &str, supersedes: Option<&str>, location: Option<&str>) -> Self {
        let mut ev = Self::new("decision", summary);
        ev.supersedes = supersedes.map(|s| s.to_string());
        ev.location = location.map(|s| s.to_string());
        ev
    }

    pub fn note(summary: &str, location: Option<&str>) -> Self {
        let mut ev = Self::new("note", summary);
        ev.location = location.map(|s| s.to_string());
        ev
    }

    pub fn validate(&self) -> Result<(), String> {
        if !VALID_EVENT_TYPES.contains(&self.event_type.as_str()) {
            return Err(format!("Unsupported event type: {}", self.event_type));
        }
        if let Some(ref o) = self.outcome {
            if !VALID_OUTCOMES.contains(&o.as_str()) {
                return Err(format!("Unsupported outcome: {}", o));
            }
        }
        if let Some(ref src) = self.capture_source {
            if !VALID_CAPTURE_SOURCES.contains(&src.as_str()) {
                return Err(format!("Unsupported capture source: {}", src));
            }
        }
        if let Some(ref conf) = self.capture_confidence {
            if !VALID_CONFIDENCE_LEVELS.contains(&conf.as_str()) {
                return Err(format!("Unsupported confidence level: {}", conf));
            }
        }
        if self.summary.trim().is_empty() {
            return Err("Event summary cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn superseded_ids(events: &[Event]) -> HashSet<String> {
    events
        .iter()
        .filter_map(|e| e.supersedes.as_ref())
        .filter(|s| !s.is_empty())
        .cloned()
        .collect()
}

pub fn resolve_event_ref(events: &[Event], ref_str: &str) -> Result<Event, String> {
    let needle = ref_str.trim();
    if needle.is_empty() {
        return Err("Empty event reference".to_string());
    }

    let exact: Vec<&Event> = events.iter().filter(|e| e.id == needle).collect();
    if exact.len() == 1 {
        return Ok(exact[0].clone());
    }

    let bare = needle.strip_prefix("evt_").unwrap_or(needle);
    let candidates: Vec<&Event> = events
        .iter()
        .filter(|e| {
            let e_bare = e.id.strip_prefix("evt_").unwrap_or(&e.id);
            e_bare.starts_with(bare)
        })
        .collect();

    if candidates.is_empty() {
        return Err(format!(
            "No event matches '{}'. Use `pjm search <text>` to find the event id (shown as evt_...).",
            ref_str
        ));
    }
    if candidates.len() > 1 {
        return Err(format!(
            "Event reference '{}' is ambiguous ({} matches) — use more characters of the id.",
            ref_str,
            candidates.len()
        ));
    }

    Ok(candidates[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_to_file() {
        assert_eq!(location_to_file(Some("src/auth.py")), Some("src/auth.py".into()));
        assert_eq!(location_to_file(Some("src/auth.py:42")), Some("src/auth.py".into()));
        assert_eq!(location_to_file(Some("C:\\src\\foo.rs:10:5")), Some("C:\\src\\foo.rs".into()));
        assert_eq!(location_to_file(Some("foo.rs")), Some("foo.rs".into()));
        assert_eq!(location_to_file(Some("AuthService.validate")), None);
        assert_eq!(location_to_file(Some("")), None);
    }

    #[test]
    fn test_event_validation_and_id() {
        let mut evt = Event::new("issue", "Test issue");
        assert!(evt.id.starts_with("evt_"));
        assert_eq!(evt.id.len(), 24);
        assert!(evt.validate().is_ok());

        evt.event_type = "invalid_type".into();
        assert!(evt.validate().is_err());
    }

    #[test]
    fn test_resolve_event_ref() {
        let mut e1 = Event::new("decision", "first");
        e1.id = "evt_abc12345678901234567".into();
        let mut e2 = Event::new("decision", "second");
        e2.id = "evt_def98765432109876543".into();

        let events = vec![e1.clone(), e2];
        assert_eq!(resolve_event_ref(&events, "evt_abc12345678901234567").unwrap().summary, "first");
        assert_eq!(resolve_event_ref(&events, "abc12").unwrap().summary, "first");
        assert!(resolve_event_ref(&events, "nonexistent").is_err());
    }
}

