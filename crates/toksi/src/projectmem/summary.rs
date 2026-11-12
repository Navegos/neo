// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/summary.rs
// created: 2026-10-27
// lastModified: 2026-11-12

#![deny(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use regex::Regex;

use super::error::Result;
use super::schema::{superseded_ids, Event};
use super::store::{
    atomic_write_file, issues_dir, project_map_path, read_events, summary_path,
};
use crate::util::paths::normalize_path;

pub const RECENT_FIXED_ISSUES: usize = 10;
pub const RECENT_DECISIONS: usize = 15;
pub const OPEN_IN_FULL: usize = 20;
pub use super::store::{RECORD_CHARS, TITLE_CHARS};

pub fn clip_record(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= RECORD_CHARS {
        return collapsed;
    }
    let head_limit = RECORD_CHARS.saturating_sub(3);
    let head = &collapsed[..head_limit];
    let half = RECORD_CHARS / 2 + 1;
    let boundary = head.rfind(' ').filter(|&idx| idx >= half);
    let clipped_head = match boundary {
        Some(idx) => &head[..idx],
        None => head,
    };
    let trimmed = clipped_head.trim_end_matches(|c| c == ' ' || c == ',' || c == ';' || c == ':');
    format!("{}...", trimmed)
}

pub fn loc_str(location: Option<&str>) -> String {
    let collapsed = location.unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        String::new()
    } else {
        format!(" [{}]", normalize_path(&collapsed))
    }
}

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
    let lower = text.to_lowercase();
    let replaced = if let Ok(re) = Regex::new(r"[^a-z0-9]+") {
        re.replace_all(&lower, "-").to_string()
    } else {
        lower.replace(' ', "-")
    };
    let trimmed = replaced.trim_matches('-');
    let truncated = if trimmed.len() > 48 {
        &trimmed[..48]
    } else {
        trimmed
    };
    let final_slug = truncated.trim_matches('-');
    if final_slug.is_empty() {
        "issue".to_string()
    } else {
        final_slug.to_string()
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

pub fn extract_project_purpose_from_readme(root: Option<&Path>) -> Option<String> {
    let root_path = match root {
        Some(r) => r.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };

    for name in ["README.md", "readme.md", "README.txt", "README"] {
        let p = root_path.join(name);
        if p.exists() {
            if let Ok(content) = fs::read_to_string(&p) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty()
                        || trimmed.starts_with('#')
                        || trimmed.starts_with("![")
                        || trimmed.starts_with("[!")
                        || trimmed.starts_with("<")
                        || looks_like_placeholder(trimmed)
                    {
                        continue;
                    }
                    let snippet: String = trimmed.chars().take(150).collect();
                    return Some(snippet);
                }
            }
        }
    }
    None
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
    let issue_loc = loc_str(issue.location.as_deref());
    if let Some(f) = fix {
        let fixed_loc = if !issue_loc.is_empty() {
            issue_loc
        } else {
            loc_str(f.location.as_deref())
        };
        return vec![format!(
            "- [DONE] #{} {}{fixed_loc} (fixed)",
            issue_id,
            clip_record(&issue.summary)
        )];
    }

    let mut lines = vec![format!(
        "- [OPEN] #{} {}{issue_loc} (open)",
        issue_id,
        clip_record(&issue.summary)
    )];

    let lessons: Vec<&Event> = issue_events
        .iter()
        .filter(|e| e.event_type == "attempt" && (e.outcome.as_deref() == Some("failed") || e.outcome.as_deref() == Some("partial")))
        .collect();

    let start_idx = lessons.len().saturating_sub(3);
    for lesson in &lessons[start_idx..] {
        let tag = match lesson.outcome.as_deref() {
            Some("partial") => "Partial attempt",
            _ => "Failed attempt",
        };
        lines.push(format!(
            "  - {}: {}{}",
            tag,
            clip_record(&lesson.summary),
            loc_str(lesson.location.as_deref())
        ));
    }

    lines
}

pub fn infer_file_mentions(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(re) = Regex::new(r"(?:^|[^\w/.-])([\w./-]+\.[A-Za-z0-9]+(?::\d+)?)") {
        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(1) {
                out.push(normalize_path(m.as_str()));
            }
        }
    }
    out
}

pub fn collect_files(events: &[Event]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut files = Vec::new();
    for e in events {
        for explicit in &e.files {
            let norm = normalize_path(explicit);
            if !seen.contains(&norm) {
                seen.insert(norm.clone());
                files.push(norm);
            }
        }
        for inferred in infer_file_mentions(&e.summary) {
            let norm = normalize_path(&inferred);
            if !seen.contains(&norm) {
                seen.insert(norm.clone());
                files.push(norm);
            }
        }
        if let Some(ref loc) = e.location {
            if let Some(f) = super::schema::location_to_file(Some(loc)) {
                let norm = normalize_path(&f);
                if !seen.contains(&norm) {
                    seen.insert(norm.clone());
                    files.push(norm);
                }
            }
        }
    }
    files
}

pub fn build_summary(events: &[Event], project_name: &str, purpose: Option<&str>) -> String {
    let default_purpose = format!(
        "Replace this placeholder with a concise description of what this project does, who it serves, and the main technologies."
    );
    let purpose = purpose.unwrap_or(&default_purpose);

    let issue_groups = group_issue_events(events);
    let mut open_issues = Vec::new();
    let mut fixed_issues = Vec::new();

    for (issue_id, issue_events) in &issue_groups {
        let issue_event = issue_events.iter().find(|e| e.event_type == "issue");
        if let Some(issue) = issue_event {
            let fix = fix_for(issue_events);
            if fix.is_some() {
                fixed_issues.push((issue_id.clone(), issue.clone(), issue_events.clone(), fix));
            } else {
                open_issues.push((issue_id.clone(), issue.clone(), issue_events.clone(), None));
            }
        }
    }

    open_issues.sort_by(|a, b| issue_sort_key(&b.0).cmp(&issue_sort_key(&a.0)));
    fixed_issues.sort_by(|a, b| issue_sort_key(&b.0).cmp(&issue_sort_key(&a.0)));

    let mut issue_sections = Vec::new();
    for (id, issue, evs, fix) in open_issues.iter().take(OPEN_IN_FULL) {
        issue_sections.extend(issue_lines(id, issue, evs, fix.as_ref()));
    }
    if open_issues.len() > OPEN_IN_FULL {
        issue_sections.push(format!(
            "- ...and {} older open issues not shown (`toksi mem search`, or `get_issue(issue_id)` via MCP)",
            open_issues.len() - OPEN_IN_FULL
        ));
    }

    for (id, issue, evs, fix) in fixed_issues.iter().take(RECENT_FIXED_ISSUES) {
        issue_sections.extend(issue_lines(id, issue, evs, fix.as_ref()));
    }
    if fixed_issues.len() > RECENT_FIXED_ISSUES {
        issue_sections.push(format!(
            "- ...and {} older fixed issues not shown (`toksi mem search`, or `get_issue(issue_id)` via MCP)",
            fixed_issues.len() - RECENT_FIXED_ISSUES
        ));
    }

    let retired = superseded_ids(events);
    let all_decisions: Vec<&Event> = events
        .iter()
        .filter(|e| e.event_type == "decision" && !retired.contains(&e.id))
        .collect();

    let decision_total = all_decisions.len();
    let mut decision_lines = Vec::new();
    if decision_total > RECENT_DECISIONS {
        decision_lines.push(format!(
            "- ...{} earlier decisions not shown (`toksi mem search`, or `search_events(query)` via MCP)",
            decision_total - RECENT_DECISIONS
        ));
    }
    let start_dec = decision_total.saturating_sub(RECENT_DECISIONS);
    for d in &all_decisions[start_dec..] {
        decision_lines.push(format!(
            "- {}{}",
            clip_record(&d.summary),
            loc_str(d.location.as_deref())
        ));
    }

    let all_notes: Vec<&Event> = events.iter().filter(|e| e.event_type == "note").collect();
    let note_total = all_notes.len();
    let mut note_lines = Vec::new();
    if note_total > 10 {
        note_lines.push(format!(
            "- ...{} earlier notes not shown (`toksi mem search`, or `search_events(query)` via MCP)",
            note_total - 10
        ));
    }
    let start_note = note_total.saturating_sub(10);
    for n in &all_notes[start_note..] {
        note_lines.push(format!(
            "- {}{}",
            clip_record(&n.summary),
            loc_str(n.location.as_deref())
        ));
    }

    let all_files = collect_files(events);
    let mut file_lines = Vec::new();
    for f in all_files.iter().take(20) {
        file_lines.push(format!("- `{}`", f));
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();

    let issues_content = if issue_sections.is_empty() {
        "- None logged yet.".to_string()
    } else {
        issue_sections.join("\n")
    };

    let decisions_content = if decision_lines.is_empty() {
        "- None logged yet.".to_string()
    } else {
        decision_lines.join("\n")
    };

    let notes_content = if note_lines.is_empty() {
        "- None logged yet.".to_string()
    } else {
        note_lines.join("\n")
    };

    let files_content = if file_lines.is_empty() {
        "- None logged yet.".to_string()
    } else {
        file_lines.join("\n")
    };

    format!(
        "# projectmem - {project_name}\n\n\
        _Last updated: {today}_\n\n\
        ## Project purpose\n\
        --------------------------------------------------------------------------------\n\
        {purpose}\n\n\
        ## Recent issues\n\
        {issues_content}\n\n\
        ## Decisions\n\
        {decisions_content}\n\n\
        ## Notes\n\
        {notes_content}\n\n\
        ## Key files\n\
        {files_content}\n\n\
        ## Open questions\n\
        - None logged yet.\n"
    )
}

pub fn build_summary_for_root(events: &[Event], root: Option<&Path>) -> String {
    let repo_root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| super::store::find_repo_root(root));
    let project_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    let existing_summary = summary_path(root)
        .ok()
        .and_then(|p| fs::read_to_string(p).ok());

    let map_purpose = project_map_path(root)
        .ok()
        .and_then(|p| extract_project_purpose_from_map(&p));

    let readme_purpose = extract_project_purpose_from_readme(Some(&repo_root));

    let preserved_purpose = existing_summary
        .as_deref()
        .and_then(extract_project_purpose);

    let default_purpose = format!(
        "Replace this placeholder with a concise description of what this project does, who it serves, and the main technologies."
    );

    let purpose = map_purpose
        .or(readme_purpose)
        .or(preserved_purpose)
        .unwrap_or(default_purpose);

    build_summary(events, project_name, Some(&purpose))
}

pub fn sync_issue_files(events: &[Event], root: Option<&Path>) -> Result<()> {
    let idir = match issues_dir(root) {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };
    fs::create_dir_all(&idir)?;

    let groups = group_issue_events(events);
    for (iid, evs) in groups {
        let issue = match evs.iter().find(|e| e.event_type == "issue") {
            Some(i) => i,
            None => continue,
        };
        let slug = slugify(&issue.summary);
        let filename = format!("{}-{}.md", iid, slug);
        let path = idir.join(&filename);

        let mut lines = vec![
            format!("# Issue #{} - {}", iid, issue.summary),
            String::new(),
            format!("**Created:** {}", issue.timestamp),
        ];
        if let Some(ref l) = issue.location {
            lines.push(format!("**Location:** `{}`", normalize_path(l)));
        }
        if let Some(ref c) = issue.git_commit {
            lines.push(format!("**Commit:** `{}`", c));
        }

        let fix = fix_for(&evs);
        if let Some(ref f) = fix {
            lines.push(format!("**Status:** Resolved (fixed by: {})", f.summary));
        } else {
            lines.push("**Status:** Open".to_string());
        }

        lines.push(String::new());
        lines.push("## History".to_string());
        lines.push(String::new());

        for e in &evs {
            match e.event_type.as_str() {
                "issue" => lines.push(format!("- [{}] Opened: {}", e.timestamp, e.summary)),
                "attempt" => {
                    let out = e.outcome.as_deref().unwrap_or("failed");
                    let cmd_str = e.command.as_deref().map(|c| format!(" (cmd: `{}`)", c)).unwrap_or_default();
                    lines.push(format!("- [{}] Attempt ({}): {}{}", e.timestamp, out, e.summary, cmd_str));
                    if let Some(ref n) = e.notes {
                        lines.push(format!("  - Details: {}", clip_record(n)));
                    }
                }
                "fix" => lines.push(format!("- [{}] Fix confirmed: {}", e.timestamp, e.summary)),
                _ => {}
            }
        }

        lines.push(String::new());
        let _ = atomic_write_file(&path, &lines.join("\n"));
    }

    Ok(())
}

pub fn regenerate_summary(root: Option<&Path>) -> Result<PathBuf> {
    let events = read_events(root)?;
    let content = build_summary_for_root(&events, root);
    let p = summary_path(root)?;
    atomic_write_file(&p, &content)?;

    // Mirror to root .projectmem if .projectmem exists
    let repo_root = root
        .map(|r| r.to_path_buf())
        .unwrap_or_else(|| super::store::find_repo_root(root));
    let legacy_summary = repo_root.join(super::store::LEGACY_MEM_DIR).join(super::store::SUMMARY_FILE);
    if legacy_summary.exists() && legacy_summary != p {
        let _ = atomic_write_file(&legacy_summary, &content);
    }

    let _ = sync_issue_files(&events, root);
    Ok(p)
}
