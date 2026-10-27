// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/precheck.rs
// created: 2026-10-27

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::schema::{location_to_file, superseded_ids, Event};
use super::store::{
    append_event, read_events, require_mem_dir, MEM_DIR,
};

pub const HIGH_CHURN_THRESHOLD: usize = 4;
pub const FAILED_ATTEMPT_BLOCK_COUNT: usize = 3;
pub const RECENT_DAYS: i64 = 30;
pub const SNOOZE_MARKER: &str = "precheck.snooze";

pub const SEVERITY_INFO: &str = "info";
pub const SEVERITY_WARN: &str = "warn";
pub const SEVERITY_BLOCK: &str = "block";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecheckWarning {
    pub file: String,
    pub severity: String,
    #[serde(rename = "type")]
    pub warning_type: String,
    pub title: String,
    pub details: Vec<String>,
}

pub fn is_memory_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized.contains(MEM_DIR) || normalized.starts_with(".projectmem")
}

pub fn parse_snooze_duration(text: &str) -> Result<ChronoDuration, String> {
    let re = Regex::new(r"(?i)^(\d+)\s*([mhd])$").unwrap();
    let caps = re
        .captures(text.trim())
        .ok_or_else(|| format!("Invalid duration '{}' — use forms like 30m, 2h, or 1d.", text))?;

    let val: i64 = caps[1]
        .parse()
        .map_err(|_| "Invalid number in duration".to_string())?;
    if val <= 0 {
        return Err("Snooze duration must be positive.".to_string());
    }

    match &caps[2].to_lowercase()[..] {
        "m" => Ok(ChronoDuration::minutes(val)),
        "h" => Ok(ChronoDuration::hours(val)),
        "d" => Ok(ChronoDuration::days(val)),
        _ => Err("Invalid duration unit".to_string()),
    }
}

pub fn snooze_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(SNOOZE_MARKER))
}

pub fn active_snooze(root: Option<&Path>) -> Option<DateTime<Utc>> {
    let path = snooze_path(root).ok()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let expiry = DateTime::parse_from_rfc3339(content.trim())
        .ok()?
        .with_timezone(&Utc);

    if expiry <= Utc::now() {
        let _ = fs::remove_file(path);
        return None;
    }
    Some(expiry)
}

pub fn set_snooze(duration_text: &str, root: Option<&Path>) -> Result<DateTime<Utc>, String> {
    let delta = parse_snooze_duration(duration_text)?;
    let expiry = Utc::now() + delta;

    let path = snooze_path(root)?;
    fs::write(&path, expiry.to_rfc3339()).map_err(|e| e.to_string())?;

    let mut note = Event::new(
        "note",
        &format!("precheck warnings snoozed for {} (until {})", duration_text, expiry.to_rfc3339()),
    );
    let _ = append_event(&mut note, root);

    Ok(expiry)
}

pub fn clear_snooze(root: Option<&Path>) -> bool {
    if let Ok(path) = snooze_path(root) {
        if path.exists() {
            return fs::remove_file(path).is_ok();
        }
    }
    false
}

pub fn get_staged_files(root: &Path) -> Vec<String> {
    let mut cmd = Command::new("git");
    cmd.args(["diff", "--name-only", "--cached"]);
    cmd.current_dir(root);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|l| l.trim().replace('\\', "/"))
                .filter(|l| !l.is_empty())
                .collect();
        }
    }
    Vec::new()
}

pub fn get_working_tree_files(root: &Path) -> Vec<String> {
    let mut files = HashSet::new();
    let mut cmd = Command::new("git");
    cmd.args(["diff", "--name-only"]);
    cmd.current_dir(root);
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            for l in String::from_utf8_lossy(&output.stdout).lines() {
                let trimmed = l.trim().replace('\\', "/");
                if !trimmed.is_empty() {
                    files.insert(trimmed);
                }
            }
        }
    }

    let mut status_cmd = Command::new("git");
    status_cmd.args(["status", "--porcelain"]);
    status_cmd.current_dir(root);
    if let Ok(output) = status_cmd.output() {
        if output.status.success() {
            for l in String::from_utf8_lossy(&output.stdout).lines() {
                if l.len() > 3 {
                    let path_part = l[3..].trim().replace('\\', "/");
                    if !path_part.is_empty() {
                        files.insert(path_part);
                    }
                }
            }
        }
    }
    files.into_iter().collect()
}

fn git_recent_changes(file_path: &str, days: i64, root: Option<&Path>) -> Option<usize> {
    let mut cmd = Command::new("git");
    let since = format!("{} days ago", days);
    cmd.args(["log", "--oneline", &format!("--since={}", since), "--", file_path]);
    if let Some(r) = root {
        cmd.current_dir(r);
    }
    let output = cmd.output().ok()?;
    if output.status.success() {
        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();
        return Some(count);
    }
    None
}

pub fn events_for_file(file_path: &str, events: &[Event]) -> Vec<Event> {
    let retired = superseded_ids(events);
    let norm_target = file_path.replace('\\', "/");

    let mut out = Vec::new();
    for e in events {
        if retired.contains(&e.id) {
            continue;
        }

        if e.files.iter().any(|f| f.replace('\\', "/") == norm_target) {
            out.push(e.clone());
            continue;
        }

        if let Some(loc_file) = location_to_file(e.location.as_deref()) {
            if loc_file.replace('\\', "/") == norm_target {
                out.push(e.clone());
                continue;
            }
        }

        if e.summary.contains(file_path) || e.summary.contains(&norm_target) {
            out.push(e.clone());
        }
    }
    out
}

pub fn analyze_files(files: &[String], events: &[Event], root: Option<&Path>) -> Vec<PrecheckWarning> {
    let cutoff = Utc::now() - ChronoDuration::days(RECENT_DAYS);
    let mut warnings = Vec::new();

    let clean_files: Vec<&String> = files.iter().filter(|f| !is_memory_path(f)).collect();

    for file_path in clean_files {
        let file_events = events_for_file(file_path.as_str(), events);
        if file_events.is_empty() {
            continue;
        }

        let mut recent = Vec::new();
        for e in &file_events {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&e.timestamp) {
                if dt.with_timezone(&Utc) >= cutoff {
                    recent.push(e.clone());
                    continue;
                }
            }
            recent.push(e.clone());
        }

        // Check 1: Failed attempts
        let failed_attempts: Vec<&Event> = recent
            .iter()
            .filter(|e| e.event_type == "attempt" && e.outcome.as_deref() == Some("failed"))
            .collect();

        if !failed_attempts.is_empty() {
            let count = failed_attempts.len();
            let severity = if count >= FAILED_ATTEMPT_BLOCK_COUNT {
                SEVERITY_BLOCK
            } else {
                SEVERITY_WARN
            };

            let mut details = Vec::new();
            let start = count.saturating_sub(3);
            for attempt in failed_attempts[start..].iter().rev() {
                let s_trunc: String = attempt.summary.chars().take(90).collect();
                details.push(format!("x {}", s_trunc));
            }
            if count > 3 {
                details.push(format!("  ... and {} more (pjm search --failed-only)", count - 3));
            }

            warnings.push(PrecheckWarning {
                file: file_path.to_string(),
                severity: severity.to_string(),
                warning_type: "failed_attempts".to_string(),
                title: format!("What already failed here ({} attempt{}):", count, if count != 1 { "s" } else { "" }),
                details,
            });
        }

        // Check 2: Open unresolved issues
        let closed_issue_ids: HashSet<String> = events
            .iter()
            .filter(|e| e.event_type == "fix" && e.issue_id.is_some())
            .filter_map(|e| e.issue_id.clone())
            .collect();

        let open_issues: Vec<&Event> = file_events
            .iter()
            .filter(|e| {
                e.event_type == "issue"
                    && e.issue_id
                        .as_ref()
                        .map(|iid| !closed_issue_ids.contains(iid))
                        .unwrap_or(false)
            })
            .collect();

        if !open_issues.is_empty() {
            let details: Vec<String> = open_issues
                .iter()
                .take(3)
                .map(|iss| {
                    let s_trunc: String = iss.summary.chars().take(80).collect();
                    format!("#{}: {}", iss.issue_id.as_deref().unwrap_or("?"), s_trunc)
                })
                .collect();

            warnings.push(PrecheckWarning {
                file: file_path.to_string(),
                severity: SEVERITY_WARN.to_string(),
                warning_type: "open_issues".to_string(),
                title: format!("{} unresolved issue{} on this file", open_issues.len(), if open_issues.len() != 1 { "s" } else { "" }),
                details,
            });
        }

        // Check 3: High churn
        let churn_count = git_recent_changes(file_path.as_str(), RECENT_DAYS, root).unwrap_or_else(|| {
            recent.iter().filter(|e| e.git_commit.is_some()).count()
        });

        if churn_count >= HIGH_CHURN_THRESHOLD {
            warnings.push(PrecheckWarning {
                file: file_path.to_string(),
                severity: SEVERITY_WARN.to_string(),
                warning_type: "high_churn".to_string(),
                title: format!("HIGH CHURN: {} changes in last {} days", churn_count, RECENT_DAYS),
                details: vec!["May indicate unresolved architectural issue".to_string()],
            });
        }

        // Check 4: Recent reverts
        let reverts: Vec<&Event> = recent
            .iter()
            .filter(|e| {
                e.event_type == "attempt"
                    && e.outcome.as_deref() == Some("failed")
                    && e.capture_source.as_deref() == Some("git_post_revert")
            })
            .collect();

        if let Some(last_revert) = reverts.last() {
            let msg = last_revert
                .git_message
                .as_deref()
                .unwrap_or(&last_revert.summary);
            let s_trunc: String = msg.chars().take(80).collect();
            warnings.push(PrecheckWarning {
                file: file_path.to_string(),
                severity: SEVERITY_WARN.to_string(),
                warning_type: "recent_revert".to_string(),
                title: "Recent revert affected this file".to_string(),
                details: vec![format!("Reverted: {}", s_trunc)],
            });
        }

        // Check 5: Recent decisions
        let decisions: Vec<&Event> = recent
            .iter()
            .filter(|e| e.event_type == "decision")
            .collect();

        if let Some(last_dec) = decisions.last() {
            let s_trunc: String = last_dec.summary.chars().take(100).collect();
            warnings.push(PrecheckWarning {
                file: file_path.to_string(),
                severity: SEVERITY_INFO.to_string(),
                warning_type: "relevant_decision".to_string(),
                title: "Recent decision affects this file".to_string(),
                details: vec![s_trunc],
            });
        }
    }

    warnings
}

pub fn run_precheck(
    files: Option<&[String]>,
    working: bool,
    level: Option<&str>,
    quiet: bool,
    root: Option<&Path>,
) -> Result<Vec<PrecheckWarning>, String> {
    let target_root = match root {
        Some(r) => r.to_path_buf(),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    if active_snooze(Some(&target_root)).is_some() {
        if !quiet {
            println!("projectmem: warnings snoozed — `tokless mem precheck --unsnooze` to re-enable.");
        }
        return Ok(Vec::new());
    }

    let target_files = if let Some(fs) = files {
        fs.to_vec()
    } else if working {
        get_working_tree_files(&target_root)
    } else {
        let staged = get_staged_files(&target_root);
        if staged.is_empty() {
            get_working_tree_files(&target_root)
        } else {
            staged
        }
    };

    if target_files.is_empty() {
        if !quiet {
            println!("projectmem: No files to check.");
        }
        return Ok(Vec::new());
    }

    let events = read_events(Some(&target_root))?;
    let warnings = analyze_files(&target_files, &events, Some(&target_root));

    if warnings.is_empty() {
        if !quiet {
            println!("projectmem: no warnings — looking good!");
        }
        return Ok(warnings);
    }

    let min_severity = match level.unwrap_or(SEVERITY_WARN) {
        SEVERITY_INFO => 0,
        SEVERITY_BLOCK => 2,
        _ => 1,
    };

    for w in &warnings {
        let sev_level = match w.severity.as_str() {
            SEVERITY_BLOCK => 2,
            SEVERITY_WARN => 1,
            _ => 0,
        };
        if sev_level >= min_severity {
            let badge = match w.severity.as_str() {
                SEVERITY_BLOCK => "[BLOCK]",
                SEVERITY_WARN => "[WARN]",
                _ => "[INFO]",
            };
            println!("{} {} — {}", badge, w.file, w.title);
            for d in &w.details {
                println!("    - {}", d);
            }
        }
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snooze_duration_parsing() {
        assert_eq!(parse_snooze_duration("30m").unwrap(), ChronoDuration::minutes(30));
        assert_eq!(parse_snooze_duration("2h").unwrap(), ChronoDuration::hours(2));
        assert_eq!(parse_snooze_duration("1d").unwrap(), ChronoDuration::days(1));
        assert!(parse_snooze_duration("invalid").is_err());
    }

    #[test]
    fn test_analyze_files() {
        let mut e1 = Event::new("issue", "Bug on auth");
        e1.issue_id = Some("0001".into());
        e1.location = Some("src/auth.rs:10".into());

        let mut e2 = Event::new("attempt", "Failed JWT check");
        e2.issue_id = Some("0001".into());
        e2.outcome = Some("failed".into());
        e2.location = Some("src/auth.rs:10".into());

        let events = vec![e1, e2];
        let warnings = analyze_files(&["src/auth.rs".to_string()], &events, None);
        assert!(!warnings.is_empty());
        assert_eq!(warnings[0].warning_type, "failed_attempts");
    }
}
