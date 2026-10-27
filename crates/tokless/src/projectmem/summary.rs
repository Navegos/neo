// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/summary.rs
// created: 2026-10-27

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use regex::Regex;

use super::schema::{superseded_ids, Event};
use super::store::{
    atomic_write_file, issues_dir, project_map_path, read_events, summary_path,
};

pub const RECENT_FIXED_ISSUES: usize = 10;
pub const RECENT_DECISIONS: usize = 15;
pub const OPEN_IN_FULL: usize = 20;

const PLACEHOLDER_PHRASES: &[&str] = &[
    "Not described yet.",
    "Short description of what the project does.",
    "Replace this placeholder",
    "Status: not created yet",
    "This file should be created by the first AI assistant",
];

pub fn looks_like_placeholder(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    PLACEHOLDER_PHRASES.iter().any(|phrase| trimmed.contains(phrase))
}

pub fn slugify(text: &str) -> String {
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let lower = text.to_lowercase();
    let replaced = re.replace_all(&lower, "-").to_string();
    let trimmed = replaced.trim_matches('-');
    let mut truncated = if trimmed.len() > 48 {
        &trimmed[..48]
    } else {
        trimmed
    };
    truncated = truncated.trim_matches('-');
    if truncated.is_empty() {
        "issue".to_string()
    } else {
        truncated.to_string()
    }
}

pub fn extract_project_purpose_from_map(map_path: &Path) -> Option<String> {
    if !map_path.exists() {
        return None;
    }
    let content = fs::read_to_string(map_path).ok()?;
    let marker = "## Project purpose";
    let start = content.find(marker)?;
    let body = &content[start + marker.len()..];
    let next_section = body.find("\n## ");
    let section_text = match next_section {
        Some(pos) => &body[..pos],
        None => body,
    };
    let trimmed = section_text.trim();
    if !looks_like_placeholder(trimmed) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub fn extract_project_purpose(summary: &str) -> Option<String> {
    let start = summary
        .find("## Project purpose")
        .or_else(|| summary.find("## What this project is"))?;
    let marker_len = if summary[start..].starts_with("## Project purpose") {
        "## Project purpose".len()
    } else {
        "## What this project is".len()
    };
    let body = &summary[start + marker_len..];
    let next_section = body.find("\n## ");
    let section_text = match next_section {
        Some(pos) => &body[..pos],
        None => body,
    };
    let trimmed = section_text.trim();
    if !looks_like_placeholder(trimmed) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub fn group_issue_events(events: &[Event]) -> HashMap<String, Vec<Event>> {
    let mut map: HashMap<String, Vec<Event>> = HashMap::new();
    for e in events {
        if let Some(ref iid) = e.issue_id {
            map.entry(iid.clone()).or_default().push(e.clone());
        }
    }
    map
}

fn fix_for(issue_events: &[Event]) -> Option<Event> {
    issue_events.iter().rev().find(|e| e.event_type == "fix").cloned()
}

fn issue_sort_key(issue_id: &str) -> (i64, String) {
    let num = issue_id.parse::<i64>().unwrap_or(-1);
    (num, issue_id.to_string())
}

fn issue_lines(
    issue_id: &str,
    issue: &Event,
    issue_events: &[Event],
    fix: Option<&Event>,
) -> Vec<String> {
    let status = if fix.is_some() { "fixed" } else { "open" };
    let marker = if fix.is_some() { "DONE" } else { "OPEN" };
    let issue_loc = match issue.location {
        Some(ref l) if !l.trim().is_empty() => format!(" [{}]", l.trim()),
        _ => String::new(),
    };
    let outcome = match fix {
        Some(f) => {
            let loc = match f.location {
                Some(ref l) if !l.trim().is_empty() => format!(" [{}]", l.trim()),
                _ => String::new(),
            };
            format!(" -> {}{}", f.summary, loc)
        }
        None => String::new(),
    };

    let mut lines = vec![format!(
        "- [{}] #{} {}{}{} ({})",
        marker, issue_id, issue.summary, issue_loc, outcome, status
    )];

    let lessons: Vec<&Event> = issue_events
        .iter()
        .filter(|e| e.event_type == "attempt" && (e.outcome.as_deref() == Some("failed") || e.outcome.as_deref() == Some("partial")))
        .collect();

    let start_idx = lessons.len().saturating_sub(3);
    for lesson in &lessons[start_idx..] {
        let loc = match lesson.location {
            Some(ref l) if !l.trim().is_empty() => format!(" [{}]", l.trim()),
            _ => String::new(),
        };
        let tag = match lesson.outcome.as_deref() {
            Some("partial") => "Partial attempt",
            _ => "Failed attempt",
        };
        lines.push(format!("  - {}: {}{}", tag, lesson.summary, loc));
    }

    lines
}

pub fn infer_file_mentions(text: &str) -> Vec<String> {
    let re = Regex::new(r"(?:^|[^\w/.-])([\w./-]+\.[A-Za-z0-9]+(?::\d+)?)").unwrap();
    let mut out = Vec::new();
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            out.push(m.as_str().to_string());
        }
    }
    out
}

pub fn collect_files(events: &[Event]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    for e in events {
        for explicit in &e.files {
            if !seen.contains(explicit) {
                seen.insert(explicit.clone());
                files.push(explicit.clone());
            }
        }
        for inferred in infer_file_mentions(&e.summary) {
            if !seen.contains(&inferred) {
                seen.insert(inferred.clone());
                files.push(inferred);
            }
        }
    }
    files
}

pub fn write_issue_files(events: &[Event], root: Option<&Path>) -> Result<(), String> {
    let idir = issues_dir(root)?;
    fs::create_dir_all(&idir).map_err(|e| e.to_string())?;

    let grouped = group_issue_events(events);
    for (issue_id, issue_events) in grouped {
        let issue = match issue_events.iter().find(|e| e.event_type == "issue") {
            Some(i) => i,
            None => continue,
        };

        let slug = slugify(&issue.summary);
        let file_path = idir.join(format!("{}-{}.md", issue_id, slug));

        let mut lines = vec![format!("# #{} {}", issue_id, issue.summary), String::new()];
        for e in &issue_events {
            let loc = match e.location {
                Some(ref l) if !l.trim().is_empty() => format!(" [{}]", l.trim()),
                _ => String::new(),
            };
            let mut detail = format!("- {} `{}`: {}{}", e.timestamp, e.event_type, e.summary, loc);
            if let Some(ref o) = e.outcome {
                detail.push_str(&format!(" ({})", o));
            }
            lines.push(detail);
        }
        lines.push(String::new());
        atomic_write_file(&file_path, &lines.join("\n"))?;
    }
    Ok(())
}

pub fn build_summary(events: &[Event], project_name: &str, project_purpose: Option<&str>) -> String {
    let now = Utc::now().format("%Y-%m-%d").to_string();
    let issues = group_issue_events(events);
    let retired = superseded_ids(events);

    let decisions: Vec<&Event> = events
        .iter()
        .filter(|e| e.event_type == "decision" && !retired.contains(&e.id))
        .collect();

    let notes: Vec<&Event> = events
        .iter()
        .filter(|e| e.event_type == "note")
        .collect();

    let mut lines = vec![
        format!("# projectmem - {}", project_name),
        String::new(),
        format!("_Last updated: {}_", now),
        String::new(),
        "## Project purpose".to_string(),
        project_purpose
            .unwrap_or("Replace this placeholder with a concise description of what this project does, who it serves, and the main technologies or runtime assumptions.")
            .to_string(),
        String::new(),
        "## Recent issues".to_string(),
    ];

    let mut sorted_issue_ids: Vec<String> = issues.keys().cloned().collect();
    sorted_issue_ids.sort_by(|a, b| {
        let ka = issue_sort_key(a);
        let kb = issue_sort_key(b);
        kb.cmp(&ka)
    });

    struct GroupInfo<'a> {
        issue_id: String,
        issue: &'a Event,
        issue_events: &'a [Event],
        fix: Option<Event>,
    }

    let mut groups = Vec::new();
    for iid in sorted_issue_ids {
        if let Some(evs) = issues.get(&iid) {
            if let Some(iss) = evs.iter().find(|e| e.event_type == "issue") {
                let fx = fix_for(evs);
                groups.push(GroupInfo {
                    issue_id: iid,
                    issue: iss,
                    issue_events: evs,
                    fix: fx,
                });
            }
        }
    }

    if groups.is_empty() {
        lines.push("- No issues logged yet.".to_string());
    } else {
        let open_groups: Vec<&GroupInfo> = groups.iter().filter(|g| g.fix.is_none()).collect();
        let fixed_groups: Vec<&GroupInfo> = groups.iter().filter(|g| g.fix.is_some()).collect();

        for g in open_groups.iter().take(OPEN_IN_FULL) {
            lines.extend(issue_lines(&g.issue_id, g.issue, g.issue_events, g.fix.as_ref()));
        }

        if open_groups.len() > OPEN_IN_FULL {
            let more = &open_groups[OPEN_IN_FULL..];
            let ids: Vec<String> = more.iter().map(|g| format!("#{}", g.issue_id)).collect();
            let s_plural = if more.len() != 1 { "s" } else { "" };
            lines.push(format!(
                "- [OPEN] {} older open issue{}, by id: {} (`get_issue(issue_id)` for details)",
                more.len(), s_plural, ids.join(", ")
            ));
        }

        for g in fixed_groups.iter().take(RECENT_FIXED_ISSUES) {
            lines.extend(issue_lines(&g.issue_id, g.issue, g.issue_events, g.fix.as_ref()));
        }

        if fixed_groups.len() > RECENT_FIXED_ISSUES {
            let hidden = fixed_groups.len() - RECENT_FIXED_ISSUES;
            let s_plural = if hidden != 1 { "s" } else { "" };
            lines.push(format!(
                "- ...and {} older fixed issue{} not shown (`pjm search`, or `get_issue(issue_id)` via MCP)",
                hidden, s_plural
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Decisions".to_string());
    if !decisions.is_empty() {
        if decisions.len() > RECENT_DECISIONS {
            let hidden = decisions.len() - RECENT_DECISIONS;
            let s_plural = if hidden != 1 { "s" } else { "" };
            lines.push(format!(
                "- ...{} earlier decision{} not shown (`pjm search`, or `search_events(query)` via MCP)",
                hidden, s_plural
            ));
        }
        let start = decisions.len().saturating_sub(RECENT_DECISIONS);
        for dec in &decisions[start..] {
            let loc = match dec.location {
                Some(ref l) if !l.trim().is_empty() => format!(" [{}]", l.trim()),
                _ => String::new(),
            };
            lines.push(format!("- {}{}", dec.summary, loc));
        }
    } else {
        lines.push("- No decisions logged yet.".to_string());
    }

    lines.push(String::new());
    lines.push("## Notes".to_string());
    if !notes.is_empty() {
        let start = notes.len().saturating_sub(10);
        for n in &notes[start..] {
            let loc = match n.location {
                Some(ref l) if !l.trim().is_empty() => format!(" [{}]", l.trim()),
                _ => String::new(),
            };
            lines.push(format!("- {}{}", n.summary, loc));
        }
    } else {
        lines.push("- No notes logged yet.".to_string());
    }

    lines.push(String::new());
    lines.push("## Key files".to_string());
    let key_files = collect_files(events);
    if !key_files.is_empty() {
        for f in key_files.iter().take(20) {
            lines.push(format!("- `{}`", f));
        }
    } else {
        lines.push("- No key files logged yet.".to_string());
    }

    lines.push(String::new());
    lines.push("## Open questions".to_string());
    lines.push("- None logged yet.".to_string());
    lines.push(String::new());

    lines.join("\n")
}

pub fn regenerate_summary(root: Option<&Path>) -> Result<PathBuf, String> {
    let events = read_events(root)?;
    let sum_path = summary_path(root)?;

    let existing_content = fs::read_to_string(&sum_path).unwrap_or_default();
    let map_p = project_map_path(root).ok();
    let map_purpose = map_p.as_ref().and_then(|p| extract_project_purpose_from_map(p));
    let purpose = map_purpose.or_else(|| extract_project_purpose(&existing_content));

    let project_name = root
        .and_then(|r| r.file_name())
        .and_then(|s| s.to_str())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().and_then(|s| s.to_str().map(|s| s.to_string())))
                .map(|s| Box::leak(s.into_boxed_str()) as &str)
        })
        .unwrap_or("project");

    let content = build_summary(&events, project_name, purpose.as_deref());
    atomic_write_file(&sum_path, &content)?;
    write_issue_files(&events, root)?;

    Ok(sum_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectmem::store::initialize;
    use tempfile::tempdir;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Fix memory leak in auth middleware!"), "fix-memory-leak-in-auth-middleware");
        assert_eq!(slugify("---"), "issue");
    }

    #[test]
    fn test_summary_regeneration() {
        let tmp = tempdir().unwrap();
        initialize(Some(tmp.path()), Some("A great tool")).unwrap();

        let mut e1 = Event::new("issue", "Crashing on null pointer");
        e1.issue_id = Some("0001".into());
        e1.location = Some("src/main.rs:10".into());

        let mut e2 = Event::new("attempt", "Added unwrapped check");
        e2.issue_id = Some("0001".into());
        e2.outcome = Some("failed".into());

        let mut e3 = Event::new("fix", "Used if-let instead");
        e3.issue_id = Some("0001".into());

        let e4 = Event::new("decision", "Use if-let over match");

        let events = vec![e1, e2, e3, e4];
        let summary = build_summary(&events, "test_proj", Some("A great tool"));
        assert!(summary.contains("# projectmem - test_proj"));
        assert!(summary.contains("A great tool"));
        assert!(summary.contains("- [DONE] #0001 Crashing on null pointer [src/main.rs:10] -> Used if-let instead (fixed)"));
        assert!(summary.contains("- Failed attempt: Added unwrapped check"));
        assert!(summary.contains("- Use if-let over match"));
    }
}

