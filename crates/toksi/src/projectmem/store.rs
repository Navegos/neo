// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/store.rs
// created: 2026-10-27
// lastModified: 2026-11-12

#![deny(unsafe_code)]

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use fs2::FileExt;
use serde::{Deserialize, Serialize};

use super::error::{ProjectMemError, Result};
use super::redaction::redact_event_fields;
use super::schema::{auto_resolve_location, Event};

pub const TOKSI_MEM_SUBDIR: &str = ".toksi/projectmem";
pub const LEGACY_MEM_DIR: &str = ".projectmem";
pub const SUMMARY_FILE: &str = "summary.md";
pub const EVENTS_FILE: &str = "events.jsonl";
pub const CONFIG_FILE: &str = "config.toml";
pub const ISSUES_DIR: &str = "issues";
pub const AI_INSTRUCTIONS_FILE: &str = "AI_INSTRUCTIONS.md";
pub const INSTRUCTIONS_BACKUP_FILE: &str = "AI_INSTRUCTIONS.md.bak";
pub const PROJECT_MAP_FILE: &str = "PROJECT_MAP.md";
pub const PLAN_FILE: &str = "plan.md";
pub const CURRENT_ISSUE_MARKER: &str = ".current_issue";
pub const LAST_FAILURE_FILE: &str = ".last_failure.json";
pub const STRUCTURE_FILE: &str = "structure.json";

pub const INSTRUCTIONS_VERSION: u32 = 2;
pub const TITLE_CHARS: usize = 140;
pub const RECORD_CHARS: usize = 300;
pub const GLOBAL_MEMORY_HEADING: &str = "## Global Memory — Inherited Knowledge";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastFailureTrace {
    pub command: String,
    pub root_cause: String,
    pub raw_output: String,
    pub timestamp: String,
}

pub fn write_last_failure_trace(trace: &LastFailureTrace, root: Option<&Path>) -> Result<()> {
    let mem_dir = match discover_mem_dir(root) {
        Some(d) => d,
        None => return Ok(()),
    };
    let path = mem_dir.join(LAST_FAILURE_FILE);
    let json = serde_json::to_string_pretty(trace)?;
    atomic_write_file(&path, &json)
}

pub fn read_last_failure_trace(root: Option<&Path>) -> Option<LastFailureTrace> {
    let mem_dir = discover_mem_dir(root)?;
    let path = mem_dir.join(LAST_FAILURE_FILE);
    if !path.exists() {
        return None;
    }
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub struct FileLock {
    file: File,
}

impl FileLock {
    pub fn acquire_exclusive(file: File) -> Result<Self> {
        file.lock_exclusive()
            .map_err(|e| ProjectMemError::Lock(format!("Failed to lock file: {}", e)))?;
        Ok(Self { file })
    }

    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

static MEM_ROOT_OVERRIDE: std::sync::RwLock<Option<PathBuf>> = std::sync::RwLock::new(None);

pub fn set_mem_root_override(p: Option<PathBuf>) {
    match MEM_ROOT_OVERRIDE.write() {
        Ok(mut lock) => *lock = p,
        Err(poisoned) => *poisoned.into_inner() = p,
    }
}

pub fn find_repo_root(start: Option<&Path>) -> PathBuf {
    if let Some(s) = start {
        if is_project_mem_dir(s) {
            return s.parent().unwrap_or(s).to_path_buf();
        }
        if is_project_mem_dir(&s.join(TOKSI_MEM_SUBDIR)) || is_project_mem_dir(&s.join(LEGACY_MEM_DIR)) {
            return s.to_path_buf();
        }
        let canonical = s.canonicalize().unwrap_or_else(|_| s.to_path_buf());
        let mut current = canonical.as_path();
        loop {
            if current.join(".git").exists() {
                return current.to_path_buf();
            }
            if is_project_mem_dir(&current.join(TOKSI_MEM_SUBDIR)) || is_project_mem_dir(&current.join(LEGACY_MEM_DIR)) {
                return current.to_path_buf();
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }
        return canonical;
    }

    let override_root = match MEM_ROOT_OVERRIDE.read() {
        Ok(lock) => lock.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    if let Some(p) = override_root {
        return p;
    }
    if let Ok(env_root) = env::var("PROJECTMEM_ROOT") {
        let trimmed = env_root.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.exists() {
                return p;
            }
        }
    }

    let initial = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let canonical = initial.canonicalize().unwrap_or(initial);
    let mut current = canonical.as_path();

    // Walk up looking for .git (repository root)
    loop {
        if current.join(".git").exists() {
            return current.to_path_buf();
        }
        if is_project_mem_dir(&current.join(TOKSI_MEM_SUBDIR)) || is_project_mem_dir(&current.join(LEGACY_MEM_DIR)) {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    canonical
}

pub fn mem_path(root: Option<&Path>) -> PathBuf {
    let repo_root = find_repo_root(root);
    let toksi_mem = repo_root.join(TOKSI_MEM_SUBDIR);
    if toksi_mem.is_dir() {
        return toksi_mem;
    }
    // Check legacy .projectmem
    let legacy = repo_root.join(LEGACY_MEM_DIR);
    if legacy.is_dir() {
        return legacy;
    }
    // Default to strict .toksi/projectmem
    toksi_mem
}

pub fn is_project_mem_dir(candidate: &Path) -> bool {
    candidate.is_dir() && (candidate.join(CONFIG_FILE).exists() || candidate.join(EVENTS_FILE).exists())
}

pub fn discover_mem_dir(start: Option<&Path>) -> Option<PathBuf> {
    if let Some(s) = start {
        if is_project_mem_dir(s) {
            return Some(s.to_path_buf());
        }
        let toksi_mem = s.join(TOKSI_MEM_SUBDIR);
        if is_project_mem_dir(&toksi_mem) {
            return Some(toksi_mem);
        }
        let legacy = s.join(LEGACY_MEM_DIR);
        if is_project_mem_dir(&legacy) {
            let _ = migrate_legacy_mem_dir(&legacy, &toksi_mem);
            if is_project_mem_dir(&toksi_mem) {
                return Some(toksi_mem);
            }
            return Some(legacy);
        }
    } else {
        let override_root = match MEM_ROOT_OVERRIDE.read() {
            Ok(lock) => lock.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        if let Some(ref p) = override_root {
            let toksi_mem = p.join(TOKSI_MEM_SUBDIR);
            if is_project_mem_dir(&toksi_mem) {
                return Some(toksi_mem);
            }
            let legacy = p.join(LEGACY_MEM_DIR);
            if is_project_mem_dir(&legacy) {
                return Some(legacy);
            }
        }
    }

    let repo_root = find_repo_root(start);
    let toksi_mem = repo_root.join(TOKSI_MEM_SUBDIR);
    if is_project_mem_dir(&toksi_mem) {
        return Some(toksi_mem);
    }
    let legacy = repo_root.join(LEGACY_MEM_DIR);
    if is_project_mem_dir(&legacy) {
        // Auto-migrate legacy to canonical .toksi/projectmem
        let _ = migrate_legacy_mem_dir(&legacy, &toksi_mem);
        if is_project_mem_dir(&toksi_mem) {
            return Some(toksi_mem);
        }
        return Some(legacy);
    }

    None
}

pub fn require_mem_dir(root: Option<&Path>) -> Result<PathBuf> {
    if let Some(d) = discover_mem_dir(root) {
        return Ok(d);
    }
    let repo_root = find_repo_root(root);
    let toksi_mem = repo_root.join(TOKSI_MEM_SUBDIR);
    if is_project_mem_dir(&toksi_mem) {
        return Ok(toksi_mem);
    }
    let legacy = repo_root.join(LEGACY_MEM_DIR);
    if is_project_mem_dir(&legacy) {
        return Ok(legacy);
    }

    Err(ProjectMemError::NotFound(format!(
        "No project memory directory found in {} or any parent. Run `toksi mem init`.",
        repo_root.display()
    )))
}

pub fn events_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(EVENTS_FILE))
}

pub fn summary_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(SUMMARY_FILE))
}

pub fn ai_instructions_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(AI_INSTRUCTIONS_FILE))
}

pub fn project_map_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(PROJECT_MAP_FILE))
}

pub fn plan_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(PLAN_FILE))
}

pub fn issues_dir(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(ISSUES_DIR))
}

pub fn structure_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(STRUCTURE_FILE))
}

pub fn migrate_legacy_mem_dir(legacy: &Path, target: &Path) -> Result<()> {
    if !legacy.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(target)?;
    fs::create_dir_all(target.join(ISSUES_DIR))?;

    for item in &[
        CONFIG_FILE,
        EVENTS_FILE,
        SUMMARY_FILE,
        PROJECT_MAP_FILE,
        PLAN_FILE,
        AI_INSTRUCTIONS_FILE,
        INSTRUCTIONS_BACKUP_FILE,
        CURRENT_ISSUE_MARKER,
        LAST_FAILURE_FILE,
        STRUCTURE_FILE,
    ] {
        let src = legacy.join(item);
        let dst = target.join(item);
        if src.exists() && (!dst.exists() || dst.metadata()?.len() == 0) {
            let _ = fs::copy(&src, &dst);
        }
    }

    let src_issues = legacy.join(ISSUES_DIR);
    let dst_issues = target.join(ISSUES_DIR);
    if src_issues.is_dir() {
        if let Ok(entries) = fs::read_dir(src_issues) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let dest = dst_issues.join(entry.file_name());
                    if !dest.exists() {
                        let _ = fs::copy(&p, &dest);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn sync_to_legacy_mem_dir(source: &Path, legacy: &Path) -> Result<()> {
    if !legacy.is_dir() {
        return Ok(());
    }
    for item in &[
        CONFIG_FILE,
        EVENTS_FILE,
        SUMMARY_FILE,
        PROJECT_MAP_FILE,
        PLAN_FILE,
        AI_INSTRUCTIONS_FILE,
        INSTRUCTIONS_BACKUP_FILE,
        CURRENT_ISSUE_MARKER,
        LAST_FAILURE_FILE,
        STRUCTURE_FILE,
    ] {
        let src = source.join(item);
        let dst = legacy.join(item);
        if src.exists() {
            let _ = fs::copy(&src, &dst);
        }
    }
    Ok(())
}

pub fn extract_instructions_version(content: &str) -> u32 {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<!--") && trimmed.contains("projectmem-instructions:") {
            if let Some(pos) = trimmed.find("projectmem-instructions:") {
                let rest = &trimmed[pos + "projectmem-instructions:".len()..];
                let num_str: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(v) = num_str.parse::<u32>() {
                    return v;
                }
            }
        }
    }
    0
}

pub fn initialize(root: Option<&Path>, project_purpose: Option<&str>) -> Result<PathBuf> {
    let repo_root = match root {
        Some(r) => r.to_path_buf(),
        None => find_repo_root(None),
    };
    let project_dir = repo_root.join(TOKSI_MEM_SUBDIR);

    // If legacy .projectmem exists, migrate it
    let legacy = repo_root.join(LEGACY_MEM_DIR);
    if legacy.is_dir() {
        let _ = migrate_legacy_mem_dir(&legacy, &project_dir);
    }

    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join(ISSUES_DIR))?;

    let events_file = project_dir.join(EVENTS_FILE);
    if !events_file.exists() {
        File::create(&events_file)?;
    }

    let config_file = project_dir.join(CONFIG_FILE);
    if !config_file.exists() {
        let purpose = project_purpose.unwrap_or("").trim();
        let content = format!(
            "recent_days = 30\nproject_description = \"{}\"\n",
            purpose.replace('"', "\\\"")
        );
        fs::write(&config_file, content)?;
    }

    let summary_file = project_dir.join(SUMMARY_FILE);
    if !summary_file.exists() {
        fs::write(&summary_file, initial_summary(&repo_root, project_purpose))?;
    }

    let instructions_file = project_dir.join(AI_INSTRUCTIONS_FILE);
    if instructions_file.exists() {
        let existing = fs::read_to_string(&instructions_file).unwrap_or_default();
        let ver = extract_instructions_version(&existing);
        if ver < INSTRUCTIONS_VERSION {
            let bak = instructions_file.with_extension("md.bak");
            let _ = fs::write(&bak, &existing);
            fs::write(&instructions_file, ai_instructions())?;
        }
    } else {
        fs::write(&instructions_file, ai_instructions())?;
    }

    let project_map_file = project_dir.join(PROJECT_MAP_FILE);
    if !project_map_file.exists() {
        fs::write(&project_map_file, initial_project_map(&repo_root))?;
    }

    let plan_file = project_dir.join(PLAN_FILE);
    if !plan_file.exists() {
        fs::write(&plan_file, initial_plan(&repo_root))?;
    }

    // Mirror to root .projectmem if .projectmem already exists
    if legacy.is_dir() {
        let _ = sync_to_legacy_mem_dir(&project_dir, &legacy);
    }

    ensure_gitignore_entry(&repo_root);
    let _ = super::global::register_project(&repo_root);

    Ok(project_dir)
}

pub fn initial_summary(root: &Path, purpose: Option<&str>) -> String {
    let project_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    let default_purpose = "Replace this placeholder with a concise description of what this project does, who it serves, and the main technologies.";
    let purpose_text = purpose.unwrap_or(default_purpose);
    format!(
        "# projectmem - {project_name}\n\n\
        _Last updated: never (placeholder — populate via `toksi mem decision` / `toksi mem note` or MCP tools)._\n\n\
        ## Project purpose\n\
        {purpose_text}\n\n\
        ## How to use this memory\n\
        AI assistants and human contributors should read this file before making changes.\n\n\
        ## Current issues\n\
        - None logged yet.\n\n\
        ## Recent fixes\n\
        - None logged yet.\n\n\
        ## Decisions\n\
        - None logged yet.\n\n\
        ## Known gotchas\n\
        - None logged yet.\n\n\
        ## Key files\n\
        - None logged yet.\n\n\
        ## Open questions\n\
        - None logged yet.\n"
    )
}

pub fn initial_project_map(root: &Path) -> String {
    let project_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    format!(
        "# Project Map - {project_name}\n\n\
        Status: not created yet.\n\n\
        ## Project purpose\n\
        Not described yet.\n\n\
        ## Structure\n\n\
        ## Relationships\n"
    )
}

pub fn initial_plan(root: &Path) -> String {
    let project_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    format!(
        "# {project_name} — plan\n\n\
        > Editable **intent** file: ideas + plans — what we *mean to do*.\n\
        > `events.jsonl` -> `summary.md` records what *happened*; this file records what we *intend*.\n\n\
        ## Ideas\n\
        _Loose thoughts, not yet committed to._\n\n\
        ## Active plans\n\
        _What we're working toward now. Use `- [ ]` / `- [x]` checklists._\n\n\
        ## Next\n\
        _Queued, but not started._\n\n\
        ## Someday / maybe\n\n\
        ## Shipped\n\
        _Move completed plans here so the top stays about the future._\n"
    )
}

pub fn ai_instructions() -> String {
    format!(
        "# projectmem AI Instructions\n\n\
        <!-- projectmem-instructions: {INSTRUCTIONS_VERSION} -->\n\n\
        These instructions are MANDATORY for all AI coding agents working in this project.\n\n\
        ## Start of every session\n\
        1. Read `.toksi/projectmem/summary.md` and `.toksi/projectmem/PROJECT_MAP.md` (or call MCP `get_summary()`).\n\
        2. Identify your mode (Setup Mode vs Maintenance Mode).\n\
        3. Work from map and memory, do not re-scan the entire repo.\n\n\
        ## MANDATORY Triggers\n\
        - Bug, error, or unexpected behavior -> `log_issue(summary, location)`\n\
        - Fix attempt FAILED -> `record_attempt(summary, outcome=\"failed\")`\n\
        - Fix attempt PARTIAL -> `record_attempt(summary, outcome=\"partial\")`\n\
        - Fix attempt WORKED -> `record_attempt(summary, outcome=\"worked\")`\n\
        - Fix confirmed -> `record_fix(summary)`\n\
        - Architectural or design decision -> `add_decision(summary)`\n\
        - Gotcha / setup detail / constraint -> `add_note(summary)`\n\n\
        ## Record Length Guidance\n\
        Keep an issue title under {TITLE_CHARS} characters and an attempt, fix, note or decision under {RECORD_CHARS}. summary.md clips longer records.\n"
    )
}

pub fn ensure_gitignore_entry(root: &Path) {
    if !root.join(".git").exists() && !root.join(".gitignore").exists() {
        return;
    }
    let gitignore = root.join(".gitignore");
    let entries = [
        format!("{}/{}", TOKSI_MEM_SUBDIR, EVENTS_FILE),
        format!("{}/watch.pid", TOKSI_MEM_SUBDIR),
        format!("{}/watch.log", TOKSI_MEM_SUBDIR),
        format!("{}/{}", TOKSI_MEM_SUBDIR, STRUCTURE_FILE),
        format!("{}/{}", TOKSI_MEM_SUBDIR, LAST_FAILURE_FILE),
        format!("{}/{}", LEGACY_MEM_DIR, EVENTS_FILE),
        format!("{}/{}", LEGACY_MEM_DIR, STRUCTURE_FILE),
    ];

    let existing = fs::read_to_string(&gitignore).unwrap_or_default();
    let existing_lines: Vec<&str> = existing.lines().map(|l| l.trim()).collect();
    let mut to_add = Vec::new();

    for e in &entries {
        if !existing_lines.contains(&e.as_str()) {
            to_add.push(e.clone());
        }
    }

    if to_add.is_empty() {
        return;
    }

    let prefix = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };

    let mut file = match OpenOptions::new().create(true).append(true).open(&gitignore) {
        Ok(f) => f,
        Err(_) => return,
    };

    let _ = write!(file, "{}{}\n", prefix, to_add.join("\n"));
}

pub fn read_events(root: Option<&Path>) -> Result<Vec<Event>> {
    let path = events_path(root)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);

    let mut events = Vec::new();
    let mut line_number = 0;
    let mut line_buf = String::new();

    while reader.read_line(&mut line_buf)? > 0 {
        line_number += 1;
        let mut trimmed = line_buf.trim();
        if trimmed.starts_with('\u{feff}') {
            trimmed = &trimmed['\u{feff}'.len_utf8()..];
        }
        if !trimmed.is_empty() {
            match serde_json::from_str::<Event>(trimmed) {
                Ok(ev) => events.push(ev),
                Err(e) => {
                    return Err(ProjectMemError::Validation(format!(
                        "Invalid event at {}:{}: {}",
                        path.display(),
                        line_number,
                        e
                    )));
                }
            }
        }
        line_buf.clear();
    }

    Ok(events)
}

pub fn append_event(event: &mut Event, root: Option<&Path>) -> Result<()> {
    let _ = redact_event_fields(event);

    let repo_root = find_repo_root(root);
    if let Some(ref loc) = event.location {
        event.location = Some(auto_resolve_location(&repo_root, loc));
    }

    let p = events_path(root)?;
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .append(true)
        .open(&p)?;

    let mut lock = FileLock::acquire_exclusive(file)?;
    let json = serde_json::to_string(event)?;
    writeln!(lock.file_mut(), "{}", json)?;
    lock.file_mut().flush()?;
    drop(lock);

    // If root .projectmem also exists, mirror event for backwards compatibility
    let repo_root = find_repo_root(root);
    let legacy_events = repo_root.join(LEGACY_MEM_DIR).join(EVENTS_FILE);
    if legacy_events.exists() && legacy_events != p {
        if let Ok(file_leg) = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .open(&legacy_events)
        {
            if let Ok(mut lock_leg) = FileLock::acquire_exclusive(file_leg) {
                let _ = writeln!(lock_leg.file_mut(), "{}", json);
                let _ = lock_leg.file_mut().flush();
            }
        }
    }

    // Auto-promote library mentions
    if (event.event_type == "attempt" || event.event_type == "decision" || event.event_type == "note")
        && !event.summary.is_empty()
    {
        let proj_name = repo_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        let stack = super::global::detect_stack(&repo_root);
        let mut libs = stack.libraries;
        libs.extend(stack.tags);
        super::global::auto_promote_event(
            &event.summary,
            &event.event_type,
            proj_name,
            event.location.as_deref(),
            Some(&libs),
            event.outcome.as_deref(),
        );
    }

    Ok(())
}

pub fn atomic_write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn next_issue_id(events: &[Event]) -> String {
    let mut max_id: u32 = 0;
    for e in events {
        if e.event_type == "issue" {
            if let Some(ref iid) = e.issue_id {
                let digits: String = iid.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = digits.parse::<u32>() {
                    if num > max_id {
                        max_id = num;
                    }
                }
            }
        }
    }
    format!("{:04}", max_id + 1)
}

pub fn current_issue_id(events: &[Event]) -> Option<String> {
    let closed: std::collections::HashSet<String> = events
        .iter()
        .filter(|e| e.event_type == "fix" && e.issue_id.is_some())
        .filter_map(|e| e.issue_id.clone())
        .collect();

    for e in events.iter().rev() {
        if e.event_type == "issue" {
            if let Some(ref iid) = e.issue_id {
                if !closed.contains(iid) {
                    return Some(iid.clone());
                }
            }
        }
    }
    None
}

pub fn current_issue_marker_path(root: Option<&Path>) -> Result<PathBuf> {
    Ok(require_mem_dir(root)?.join(CURRENT_ISSUE_MARKER))
}

pub fn write_current_issue(issue_id: &str, root: Option<&Path>) {
    if let Ok(p) = current_issue_marker_path(root) {
        let _ = fs::write(p, issue_id.trim());
    }
}

pub fn read_current_issue(root: Option<&Path>) -> Option<String> {
    let p = current_issue_marker_path(root).ok()?;
    if !p.exists() {
        return None;
    }
    let s = fs::read_to_string(p).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn normalize_issue_id(issue_id: Option<&str>) -> Option<String> {
    let raw = issue_id?.trim().trim_start_matches('#');
    if raw.is_empty() {
        return None;
    }
    if raw.chars().all(|c| c.is_ascii_digit()) {
        let num: u32 = raw.parse().ok()?;
        Some(format!("{:04}", num))
    } else {
        Some(raw.to_string())
    }
}

pub fn issue_exists(events: &[Event], issue_id: &str) -> bool {
    let norm = normalize_issue_id(Some(issue_id)).unwrap_or_else(|| issue_id.to_string());
    events.iter().any(|e| {
        e.event_type == "issue"
            && e.issue_id
                .as_deref()
                .map(|id| normalize_issue_id(Some(id)).unwrap_or_else(|| id.to_string()) == norm)
                .unwrap_or(false)
    })
}

pub fn active_issue_marker(events: &[Event], root: Option<&Path>) -> Option<String> {
    let marker = read_current_issue(root)?;
    if issue_exists(events, &marker) {
        Some(marker)
    } else {
        clear_current_issue(root);
        None
    }
}

pub fn clear_current_issue(root: Option<&Path>) {
    if let Ok(p) = current_issue_marker_path(root) {
        let _ = fs::remove_file(p);
    }
}

pub fn latest_open_issue_within(events: &[Event], minutes: i64) -> Option<String> {
    let closed: std::collections::HashSet<String> = events
        .iter()
        .filter(|e| e.event_type == "fix" && e.issue_id.is_some())
        .filter_map(|e| e.issue_id.clone())
        .collect();

    let cutoff = chrono::Utc::now() - chrono::Duration::minutes(minutes);

    for e in events.iter().rev() {
        if e.event_type == "issue" {
            if let Some(ref iid) = e.issue_id {
                if !closed.contains(iid) {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&e.timestamp) {
                        if dt.with_timezone(&chrono::Utc) >= cutoff {
                            return Some(iid.clone());
                        }
                    }
                    return None;
                }
            }
        }
    }
    None
}

pub fn get_git_commit(root: Option<&Path>) -> Option<String> {
    let r = root.map(|p| p.to_path_buf()).unwrap_or_else(|| find_repo_root(root));
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(r)
        .output()
        .ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}
