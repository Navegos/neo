// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/staleness.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

use super::schema::{location_to_file, normalize_timestamp, superseded_ids, Event};
use super::store::find_repo_root;
use crate::util::paths::normalize_path;

pub const STALE_COMMIT_THRESHOLD: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleEvent {
    pub event_id: String,
    pub event_type: String,
    pub summary: String,
    pub file: String,
    pub commits_since: i64,
}

pub fn commit_times(
    file_path: &str,
    since_iso: Option<&str>,
    root: Option<&Path>,
) -> Option<Vec<String>> {
    let repo_root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| find_repo_root(root));

    let mut cmd = Command::new("git");
    cmd.args(["log", "--format=%cI"]);
    if let Some(since) = since_iso {
        if !since.trim().is_empty() {
            cmd.arg(format!("--since={}", since));
        }
    }
    cmd.args(["--", file_path]);
    cmd.current_dir(repo_root);

    let output = cmd.output().ok()?;
    if output.status.success() {
        let times: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| normalize_timestamp(Some(l.trim())))
            .filter(|l| !l.is_empty())
            .collect();
        return Some(times);
    }
    None
}

pub fn commits_touching_since(
    file_path: &str,
    since_iso: &str,
    root: Option<&Path>,
) -> Option<usize> {
    let times = commit_times(file_path, Some(since_iso), root)?;
    let since_zulu = normalize_timestamp(Some(since_iso));
    let count = times.iter().filter(|&t| t > &since_zulu).count();
    Some(count)
}

pub fn find_stale_events(
    events: &[Event],
    root: Option<&Path>,
    threshold: usize,
    only_files: Option<&HashSet<String>>,
) -> Vec<StaleEvent> {
    let repo_root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| find_repo_root(root));

    let retired = superseded_ids(events);
    let mut candidates: Vec<(&Event, String)> = Vec::new();

    for event in events {
        if event.event_type != "decision" && event.event_type != "fix" && event.event_type != "note" {
            continue;
        }
        if retired.contains(&event.id) {
            continue;
        }
        if event.auto_captured && (event.event_type == "fix" || event.event_type == "note") {
            continue;
        }

        let file_path = match location_to_file(event.location.as_deref()) {
            Some(f) => normalize_path(&f),
            None => continue,
        };

        if file_path.is_empty() || file_path.starts_with(".toksi/") || file_path.starts_with(".projectmem/") {
            continue;
        }

        if let Some(filter) = only_files {
            if !filter.contains(&file_path) {
                continue;
            }
        }

        candidates.push((event, file_path));
    }

    let mut oldest_by_file: HashMap<String, String> = HashMap::new();
    for (event, file_path) in &candidates {
        let ts = normalize_timestamp(Some(&event.timestamp));
        oldest_by_file
            .entry(file_path.clone())
            .and_modify(|existing| {
                if ts < *existing {
                    *existing = ts.clone();
                }
            })
            .or_insert(ts);
    }

    let mut history: HashMap<String, Option<Vec<String>>> = HashMap::new();
    for (_, file_path) in &candidates {
        if history.contains_key(file_path) {
            continue;
        }
        let full_path = repo_root.join(file_path);
        if !full_path.exists() {
            history.insert(file_path.clone(), None);
            continue;
        }
        let oldest = oldest_by_file.get(file_path).map(|s| s.as_str());
        let times = commit_times(file_path, oldest, Some(&repo_root));
        history.insert(file_path.clone(), times);
    }

    let mut flagged = Vec::new();
    for (event, file_path) in candidates {
        let full_path = repo_root.join(&file_path);
        if !full_path.exists() {
            flagged.push(StaleEvent {
                event_id: event.id.clone(),
                event_type: event.event_type.clone(),
                summary: event.summary.clone(),
                file: file_path,
                commits_since: -1,
            });
            continue;
        }

        let times = match history.get(&file_path) {
            Some(Some(t)) => t,
            _ => continue,
        };

        let logged = normalize_timestamp(Some(&event.timestamp));
        let count = times.iter().filter(|&t| t > &logged).count();

        if count >= threshold {
            flagged.push(StaleEvent {
                event_id: event.id.clone(),
                event_type: event.event_type.clone(),
                summary: event.summary.clone(),
                file: file_path,
                commits_since: count as i64,
            });
        }
    }

    flagged
}

pub fn stale_label(item: &StaleEvent) -> String {
    let reason = if item.commits_since == -1 {
        format!("cited file {} no longer exists", item.file)
    } else {
        format!("predates {} commits to {}", item.commits_since, item.file)
    };

    let summary_trunc: String = item.summary.chars().take(70).collect();
    format!(
        "{} [{}] \"{}\" — {}. Confirm, or retire with: toksi mem decision \"...\" --supersedes {}",
        item.event_type, item.event_id, summary_trunc, reason, item.event_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stale_label() {
        let item = StaleEvent {
            event_id: "evt_123".into(),
            event_type: "decision".into(),
            summary: "Use bcrypt rounds=12".into(),
            file: "src/auth.rs".into(),
            commits_since: 5,
        };
        let label = stale_label(&item);
        assert!(label.contains("predates 5 commits to src/auth.rs"));
        assert!(label.contains("evt_123"));
    }
}
