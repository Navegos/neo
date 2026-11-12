// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/schema.rs
// created: 2026-10-27
// lastModified: 2026-11-12

#![deny(unsafe_code)]

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::error::{ProjectMemError, Result};
use crate::util::paths::normalize_path;

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
    let hex_part = format!(
        "{:010x}{:06x}{:04x}",
        now.as_secs(),
        (now.subsec_nanos() >> 8) & 0xffffff,
        counter & 0xffff
    );
    let prefix = if hex_part.len() >= 20 {
        &hex_part[..20]
    } else {
        &hex_part
    };
    format!("evt_{}", prefix)
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

    let base = if let Some(idx) = raw.find('(') {
        raw[..idx].trim()
    } else {
        raw
    };
    if base.is_empty() {
        return None;
    }

    let mut loc = normalize_path(base);
    let mut drive = String::new();

    // Check Windows drive letter: C:/
    let chars: Vec<char> = loc.chars().collect();
    if chars.len() >= 3 && chars[0].is_ascii_alphabetic() && chars[1] == ':' && chars[2] == '/' {
        drive = loc[..2].to_string();
        loc = loc[2..].to_string();
    }

    // Strip trailing line/col suffix :123 or :123:45
    let stripped = if let Ok(re) = Regex::new(r":\d+(?::\d+)?$") {
        re.replace(&loc, "").to_string()
    } else {
        loc.clone()
    };
    if stripped.is_empty() {
        return None;
    }

    let candidate = format!("{}{}", drive, stripped);
    if stripped.contains('/') {
        return Some(normalize_path(&candidate));
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

pub fn parse_location_coordinate(location: &str) -> Option<(String, u32)> {
    let raw = location.trim().trim_matches(|c| c == '"' || c == '\'');
    if raw.is_empty() {
        return None;
    }
    let base = if let Some(idx) = raw.find('(') {
        raw[..idx].trim()
    } else {
        raw
    };

    let re = Regex::new(r"^(.*?):(\d+)(?::\d+)?$").ok()?;
    let caps = re.captures(base)?;
    let file = normalize_path(caps.get(1)?.as_str().trim());
    let line: u32 = caps.get(2)?.as_str().parse().ok()?;
    if file.is_empty() {
        return None;
    }
    Some((file, line))
}

pub fn auto_resolve_location(root: &std::path::Path, location: &str) -> String {
    let raw = location.trim();
    if raw.is_empty() {
        return raw.to_string();
    }
    if raw.contains('(') && raw.ends_with(')') {
        return raw.to_string();
    }

    if let Some((file, line)) = parse_location_coordinate(raw) {
        if let Some(symbol) = crate::codegraph::resolve_coordinate(root, &file, line) {
            return format!("{}:{} ({})", file, line, symbol);
        }
        if let Some(symbol) = resolve_location_symbol(root, &file, line) {
            return format!("{}:{} ({})", file, line, symbol);
        }
    }
    raw.to_string()
}

pub fn resolve_location_symbol(root: &std::path::Path, file_path: &str, line: u32) -> Option<String> {
    if line == 0 {
        return None;
    }
    let p = root.join(file_path);
    if !p.exists() || !p.is_file() {
        return None;
    }

    let content = std::fs::read_to_string(&p).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let target_idx = (line as usize).saturating_sub(1).min(lines.len() - 1);
    let start_idx = target_idx.saturating_sub(50);

    let re_fn_rs = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)").ok()?;
    let re_struct_rs = Regex::new(r"(?:pub\s+)?(?:struct|enum|trait)\s+([a-zA-Z0-9_]+)").ok()?;
    let re_fn_py = Regex::new(r"def\s+([a-zA-Z0-9_]+)").ok()?;
    let re_class_py = Regex::new(r"class\s+([a-zA-Z0-9_]+)").ok()?;
    let re_fn_ts = Regex::new(r"(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)").ok()?;
    let re_class_ts = Regex::new(r"(?:export\s+)?class\s+([a-zA-Z0-9_]+)").ok()?;

    for i in (start_idx..=target_idx).rev() {
        let l = lines[i].trim();
        if let Some(caps) = re_fn_rs.captures(l) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
        if let Some(caps) = re_struct_rs.captures(l) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
        if let Some(caps) = re_fn_py.captures(l) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
        if let Some(caps) = re_class_py.captures(l) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
        if let Some(caps) = re_fn_ts.captures(l) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
        if let Some(caps) = re_class_ts.captures(l) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }

    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub summary: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,

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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subproject_id: Option<String>,

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
            summary: summary.trim().to_string(),
            issue_id: None,
            outcome: None,
            files: Vec::new(),
            command: None,
            notes: None,
            git_commit: None,
            location: None,
            subproject_id: None,
            auto_captured: false,
            capture_source: None,
            capture_confidence: None,
            git_message: None,
            supersedes: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !VALID_EVENT_TYPES.contains(&self.event_type.as_str()) {
            return Err(ProjectMemError::Validation(format!(
                "Unsupported event type: '{}'",
                self.event_type
            )));
        }

        if let Some(ref o) = self.outcome {
            if !VALID_OUTCOMES.contains(&o.as_str()) {
                return Err(ProjectMemError::Validation(format!(
                    "Unsupported outcome: '{}'",
                    o
                )));
            }
        }

        if let Some(ref s) = self.capture_source {
            if !VALID_CAPTURE_SOURCES.contains(&s.as_str()) {
                return Err(ProjectMemError::Validation(format!(
                    "Unsupported capture source: '{}'",
                    s
                )));
            }
        }

        if let Some(ref c) = self.capture_confidence {
            if !VALID_CONFIDENCE_LEVELS.contains(&c.as_str()) {
                return Err(ProjectMemError::Validation(format!(
                    "Unsupported confidence level: '{}'",
                    c
                )));
            }
        }

        if self.summary.trim().is_empty() {
            return Err(ProjectMemError::Validation(
                "Event summary cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    pub fn issue(summary: &str, location: Option<&str>) -> Self {
        let mut ev = Self::new("issue", summary);
        ev.location = location.map(|s| normalize_path(s));
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
        ev.location = location.map(|s| normalize_path(s));
        ev.command = command.map(|s| s.to_string());
        ev
    }

    pub fn fix(summary: &str, issue_id: Option<&str>, location: Option<&str>) -> Self {
        let mut ev = Self::new("fix", summary);
        ev.issue_id = issue_id.map(|s| s.to_string());
        ev.location = location.map(|s| normalize_path(s));
        ev
    }

    pub fn decision(summary: &str, location: Option<&str>, supersedes: Option<&str>) -> Self {
        let mut ev = Self::new("decision", summary);
        ev.location = location.map(|s| normalize_path(s));
        ev.supersedes = supersedes.map(|s| s.to_string());
        ev
    }

    pub fn note(summary: &str, location: Option<&str>) -> Self {
        let mut ev = Self::new("note", summary);
        ev.location = location.map(|s| normalize_path(s));
        ev
    }
}

/// Typed Issue projection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Issue {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl From<Issue> for Event {
    fn from(item: Issue) -> Self {
        let mut ev = Event::new("issue", &item.summary);
        ev.location = item.location.map(|s| normalize_path(&s));
        ev
    }
}

/// Typed Attempt projection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl From<Attempt> for Event {
    fn from(item: Attempt) -> Self {
        let mut ev = Event::new("attempt", &item.summary);
        ev.outcome = Some(item.outcome);
        ev.issue_id = item.issue_id;
        ev.location = item.location.map(|s| normalize_path(&s));
        ev.command = item.command;
        ev
    }
}

/// Typed Fix projection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fix {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl From<Fix> for Event {
    fn from(item: Fix) -> Self {
        let mut ev = Event::new("fix", &item.summary);
        ev.issue_id = item.issue_id;
        ev.location = item.location.map(|s| normalize_path(&s));
        ev
    }
}

/// Typed Decision projection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Decision {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
}

impl From<Decision> for Event {
    fn from(item: Decision) -> Self {
        let mut ev = Event::new("decision", &item.summary);
        ev.location = item.location.map(|s| normalize_path(&s));
        ev.supersedes = item.supersedes;
        ev
    }
}

/// Typed Note projection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Note {
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl From<Note> for Event {
    fn from(item: Note) -> Self {
        let mut ev = Event::new("note", &item.summary);
        ev.location = item.location.map(|s| normalize_path(&s));
        ev
    }
}

pub fn superseded_ids(events: &[Event]) -> HashSet<String> {
    events
        .iter()
        .filter_map(|e| e.supersedes.as_ref())
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .collect()
}

pub fn resolve_event_ref(events: &[Event], ref_str: &str) -> Result<Event> {
    let needle = ref_str.trim();
    if needle.is_empty() {
        return Err(ProjectMemError::EventRef(
            "Empty event reference".to_string(),
        ));
    }

    let exact: Vec<&Event> = events.iter().filter(|e| e.id == needle).collect();
    if exact.len() == 1 {
        return Ok((*exact[0]).clone());
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
        return Err(ProjectMemError::EventRef(format!(
            "No event matches '{}'. Use `toksi mem search <text>` to find the event id.",
            ref_str
        )));
    }

    if candidates.len() > 1 {
        return Err(ProjectMemError::EventRef(format!(
            "Event reference '{}' is ambiguous ({} matches) — use more characters of the id.",
            ref_str,
            candidates.len()
        )));
    }

    Ok((*candidates[0]).clone())
}
