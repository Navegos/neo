// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/store.rs
// created: 2026-10-27

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use regex::Regex;

use super::redaction::redact_event_fields;
use super::schema::{normalize_timestamp, Event};

pub const MEM_DIR: &str = ".projectmem";
pub const SUMMARY_FILE: &str = "summary.md";
pub const EVENTS_FILE: &str = "events.jsonl";
pub const CONFIG_FILE: &str = "config.toml";
pub const ISSUES_DIR: &str = "issues";
pub const AI_INSTRUCTIONS_FILE: &str = "AI_INSTRUCTIONS.md";
pub const INSTRUCTIONS_BACKUP_FILE: &str = "AI_INSTRUCTIONS.md.bak";
pub const PROJECT_MAP_FILE: &str = "PROJECT_MAP.md";
pub const PLAN_FILE: &str = "plan.md";
pub const CURRENT_ISSUE_MARKER: &str = ".current_issue";
pub const INSTRUCTIONS_VERSION: u32 = 1;

pub struct FileLock {
    file: File,
    #[cfg(windows)]
    locked: bool,
}

impl FileLock {
    pub fn acquire_exclusive(file: File) -> Self {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::{LockFileEx, LOCKFILE_EXCLUSIVE_LOCK};
            use windows_sys::Win32::System::IO::OVERLAPPED;

            let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
            let start = Instant::now();
            let mut locked = false;
            while start.elapsed() < Duration::from_secs(5) {
                let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
                let ok = unsafe {
                    LockFileEx(
                        handle,
                        LOCKFILE_EXCLUSIVE_LOCK,
                        0,
                        u32::MAX,
                        u32::MAX,
                        &mut overlapped,
                    )
                };
                if ok != 0 {
                    locked = true;
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Self { file, locked }
        }

        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let mut fl = libc::flock {
                l_type: libc::F_WRLCK as libc::c_short,
                l_whence: libc::SEEK_SET as libc::c_short,
                l_start: 0,
                l_len: 0,
                l_pid: 0,
            };
            unsafe {
                libc::fcntl(file.as_raw_fd(), libc::F_SETLKW, &mut fl);
            }
            Self { file }
        }

        #[cfg(not(any(windows, unix)))]
        {
            Self { file }
        }
    }

    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            if self.locked {
                use std::os::windows::io::AsRawHandle;
                use windows_sys::Win32::Storage::FileSystem::UnlockFileEx;
                use windows_sys::Win32::System::IO::OVERLAPPED;
                let handle = self.file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
                let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
                unsafe {
                    UnlockFileEx(handle, 0, u32::MAX, u32::MAX, &mut overlapped);
                }
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let mut fl = libc::flock {
                l_type: libc::F_UNLCK as libc::c_short,
                l_whence: libc::SEEK_SET as libc::c_short,
                l_start: 0,
                l_len: 0,
                l_pid: 0,
            };
            unsafe {
                libc::fcntl(self.file.as_raw_fd(), libc::F_SETLK, &mut fl);
            }
        }
    }
}

pub fn mem_path(root: Option<&Path>) -> PathBuf {
    match root {
        Some(r) => r.join(MEM_DIR),
        None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(MEM_DIR),
    }
}

pub fn is_project_mem_dir(candidate: &Path) -> bool {
    candidate.is_dir() && candidate.join(CONFIG_FILE).exists()
}

pub fn discover_mem_dir(start: Option<&Path>) -> Option<PathBuf> {
    let cur = match start {
        Some(s) => s.to_path_buf(),
        None => env::current_dir().ok()?,
    };
    let mut current: &Path = &cur;
    loop {
        let candidate = current.join(MEM_DIR);
        if is_project_mem_dir(&candidate) {
            return Some(candidate);
        }
        match current.parent() {
            Some(p) => current = p,
            None => break,
        }
    }
    None
}

pub fn require_mem_dir(root: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(r) = root {
        let path = mem_path(Some(r));
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("No .projectmem directory found in {}. Run `tokless mem init`.", r.display()));
    }

    if let Ok(env_root) = env::var("PROJECTMEM_ROOT") {
        if !env_root.trim().is_empty() {
            let path = PathBuf::from(env_root.trim()).join(MEM_DIR);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_path = mem_path(Some(&cwd));
    if is_project_mem_dir(&cwd_path) {
        return Ok(cwd_path);
    }

    if let Some(found) = discover_mem_dir(Some(&cwd)) {
        return Ok(found);
    }

    Err(format!(
        "No .projectmem directory found in {} or any parent. Run `tokless mem init` to create one.",
        cwd.display()
    ))
}

pub fn events_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(EVENTS_FILE))
}

pub fn summary_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(SUMMARY_FILE))
}

pub fn ai_instructions_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(AI_INSTRUCTIONS_FILE))
}

pub fn project_map_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(PROJECT_MAP_FILE))
}

pub fn plan_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(PLAN_FILE))
}

pub fn issues_dir(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(ISSUES_DIR))
}

pub fn current_issue_marker_path(root: Option<&Path>) -> Result<PathBuf, String> {
    Ok(require_mem_dir(root)?.join(CURRENT_ISSUE_MARKER))
}

pub fn read_events(root: Option<&Path>) -> Result<Vec<Event>, String> {
    let path = events_path(root)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    let mut reader = BufReader::new(file);

    let mut events = Vec::new();
    let mut line = String::new();
    let mut line_no = 0;

    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        line_no += 1;
        let mut trimmed = line.trim();
        if trimmed.starts_with('\u{FEFF}') {
            trimmed = &trimmed['\u{FEFF}'.len_utf8()..];
        }
        if !trimmed.is_empty() {
            match serde_json::from_str::<Event>(trimmed) {
                Ok(mut ev) => {
                    ev.timestamp = normalize_timestamp(Some(&ev.timestamp));
                    events.push(ev);
                }
                Err(e) => {
                    return Err(format!("Invalid event at {}:{}: {}", path.display(), line_no, e));
                }
            }
        }
        line.clear();
    }

    Ok(events)
}

pub fn append_event(event: &mut Event, root: Option<&Path>) -> Result<(), String> {
    event.validate()?;
    let _ = redact_event_fields(event);

    let path = events_path(root)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;

    let mut lock = FileLock::acquire_exclusive(file);
    let serialized = serde_json::to_string(event).map_err(|e| e.to_string())?;
    writeln!(lock.file_mut(), "{}", serialized).map_err(|e| e.to_string())?;
    lock.file_mut().flush().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn next_issue_id(events: &[Event]) -> String {
    let mut max_id = 0;
    for e in events {
        if e.event_type == "issue" {
            if let Some(ref id_str) = e.issue_id {
                if let Ok(n) = id_str.parse::<u32>() {
                    if n > max_id {
                        max_id = n;
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
            if let Some(ref id) = e.issue_id {
                if !closed.contains(id) {
                    return Some(id.clone());
                }
            }
        }
    }
    None
}

pub fn write_current_issue(issue_id: &str, root: Option<&Path>) {
    if let Ok(path) = current_issue_marker_path(root) {
        let _ = fs::write(path, issue_id.trim());
    }
}

pub fn read_current_issue(root: Option<&Path>) -> Option<String> {
    let path = current_issue_marker_path(root).ok()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub fn clear_current_issue(root: Option<&Path>) {
    if let Ok(path) = current_issue_marker_path(root) {
        let _ = fs::remove_file(path);
    }
}

pub fn normalize_issue_id(issue_id: Option<&str>) -> Option<String> {
    let cleaned = issue_id?.trim().trim_start_matches('#');
    if cleaned.is_empty() {
        return None;
    }
    if let Ok(num) = cleaned.parse::<u32>() {
        return Some(format!("{:04}", num));
    }
    Some(cleaned.to_string())
}

pub fn issue_exists(events: &[Event], issue_id: &str) -> bool {
    events
        .iter()
        .any(|e| e.event_type == "issue" && e.issue_id.as_deref() == Some(issue_id))
}

pub fn active_issue_marker(events: &[Event], root: Option<&Path>) -> Option<String> {
    let marker = read_current_issue(root)?;
    if !issue_exists(events, &marker) {
        clear_current_issue(root);
        return None;
    }
    Some(marker)
}

pub fn latest_open_issue_within(events: &[Event], minutes: i64) -> Option<String> {
    let closed: std::collections::HashSet<String> = events
        .iter()
        .filter(|e| e.event_type == "fix" && e.issue_id.is_some())
        .filter_map(|e| e.issue_id.clone())
        .collect();

    let cutoff = Utc::now() - ChronoDuration::minutes(minutes);

    for e in events.iter().rev() {
        if e.event_type != "issue" {
            continue;
        }
        let Some(ref iid) = e.issue_id else { continue };
        if closed.contains(iid) {
            continue;
        }

        if let Ok(dt) = DateTime::parse_from_rfc3339(&e.timestamp) {
            if dt.with_timezone(&Utc) >= cutoff {
                return Some(iid.clone());
            }
            return None;
        }
    }
    None
}

pub fn get_git_commit(root: Option<&Path>) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "--short", "HEAD"]);
    if let Some(r) = root {
        cmd.current_dir(r);
    }
    let output = cmd.output().ok()?;
    if output.status.success() {
        let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !commit.is_empty() {
            return Some(commit);
        }
    }
    None
}

pub fn atomic_write_file(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let tmp_name = format!(
        ".tmp_{}_{}",
        std::process::id(),
        path.file_name().and_then(|s| s.to_str()).unwrap_or("file")
    );
    let tmp_path = parent.join(tmp_name);

    fs::write(&tmp_path, content).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(path);
        fs::rename(&tmp_path, path).map_err(|_| e.to_string())?;
    }
    Ok(())
}

pub fn initial_summary(project_name: &str) -> String {
    format!(
        "# projectmem - {project_name}\n\n\
        _Last updated: never (placeholder — populate via `pjm decision` / `pjm note` or the `add_decision` / `add_note` MCP tools)._\n\n\
        ## Project purpose\n\
        Replace this placeholder with a concise description of what this project does, who it serves, and the main technologies or runtime assumptions.\n\n\
        ## How to use this memory\n\
        AI assistants and human contributors should read this file before making changes. Keep it focused on durable context: current issues, decisions, failed attempts, gotchas, and files that matter.\n\n\
        **For AI assistants finding this placeholder:** you are in Setup Mode. Read README, package metadata, and obvious entry points, then call `add_decision` and `add_note` (MCP) — or `pjm decision` / `pjm note` (CLI) — to record what you learned. Each call appends an event and auto-regenerates this summary. **Do NOT edit this file directly** — it is derived from `events.jsonl`. See `.projectmem/AI_INSTRUCTIONS.md` for the full Setup Mode workflow.\n\n\
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

pub fn ai_instructions() -> String {
    format!(
        "# projectmem AI Instructions\n\n\
        <!-- projectmem-instructions: {INSTRUCTIONS_VERSION} -->\n\n\
        These instructions are MANDATORY for all AI coding agents working in this project. Failure to follow them means your work is incomplete and the audit trail is corrupted.\n\n\
        This file is stable operating guidance. Do not rewrite it unless the user asks or projectmem itself changes.\n\n\
        ## Start of every session\n\n\
        **Step 1 — Identify your mode by reading `.projectmem/summary.md` and `.projectmem/PROJECT_MAP.md`.**\n\n\
        - **Setup Mode** — `summary.md` and/or `PROJECT_MAP.md` still contain the **placeholder text** from `pjm init`.\n\
        - **Maintenance Mode** — `summary.md` AND `PROJECT_MAP.md` contain real project content.\n\n\
        **Step 2 — Read these files (or call the MCP equivalents):**\n\n\
        | File | MCP tool | Purpose |\n\
        | --- | --- | --- |\n\
        | `.projectmem/AI_INSTRUCTIONS.md` | `get_instructions()` | Workflow rules (this file) |\n\
        | `.projectmem/summary.md` | `get_summary()` | Distilled project memory (what HAPPENED) |\n\
        | `.projectmem/PROJECT_MAP.md` | `get_project_map()` | Structural layout |\n\
        | `.projectmem/plan.md` | `get_plan()` | Ideas + plans (what we INTEND to do) |\n\n\
        ## MANDATORY Triggers — You MUST act on these automatically\n\n\
        | Trigger | MCP tool | CLI command |\n\
        | --- | --- | --- |\n\
        | Bug, error, or unexpected behavior | `log_issue(summary, location)` | `pjm log \"<text>\" --at \"<file:line>\"` |\n\
        | Fix attempt FAILED | `record_attempt(summary, outcome=\"failed\")` | `pjm attempt \"<text>\" --failed --at \"<file:line>\"` |\n\
        | Fix attempt PARTIAL | `record_attempt(summary, outcome=\"partial\")` | `pjm attempt \"<text>\" --partial --at \"<file:line>\"` |\n\
        | Fix attempt WORKED | `record_attempt(summary, outcome=\"worked\")` | `pjm attempt \"<text>\" --worked --at \"<file:line>\"` |\n\
        | Fix confirmed — close the issue | `record_fix(summary)` | `pjm fix \"<text>\" --at \"<file:line>\"` |\n\
        | Architectural / design decision | `add_decision(summary)` | `pjm decision \"<text>\" --at \"<file:line>\"` |\n\
        | Gotcha / setup detail / constraint discovered | `add_note(summary)` | `pjm note \"<text>\" --at \"<file:line>\"` |\n\
        | Before finishing the session | `get_summary()` | `pjm show` |\n\n\
        All write tools auto-append to `events.jsonl` AND auto-regenerate `summary.md`.\n"
    )
}

pub fn initial_project_map(project_name: &str) -> String {
    format!(
        "# Project Map - {project_name}\n\n\
        Status: not created yet. Fill the sections below with the project's real structure and key relationships — the first AI session can do it (see `.projectmem/AI_INSTRUCTIONS.md`).\n\n\
        ## Project purpose\n\
        Not described yet.\n\n\
        ## Structure\n\n\
        ## Relationships\n"
    )
}

pub fn initial_plan(project_name: &str) -> String {
    format!(
        "# {project_name} — plan\n\n\
        > Editable **intent** file: ideas + plans — what we *mean to do*.\n\
        > This is NOT the event log. `events.jsonl` -> `summary.md` records what\n\
        > *happened*; this file records what we *intend*. The AI reads it at\n\
        > session start and edits it directly (like `PROJECT_MAP.md`): add ideas\n\
        > and plans, check items off, move done work down to Shipped. Plans are\n\
        > never logged as events.\n\n\
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

pub fn instructions_version(text: &str) -> u32 {
    let re = Regex::new(r"<!--\s*projectmem-instructions:\s*(\d+)\s*-->").unwrap();
    if let Some(caps) = re.captures(text) {
        if let Some(m) = caps.get(1) {
            if let Ok(v) = m.as_str().parse::<u32>() {
                return v;
            }
        }
    }
    0
}

pub fn refresh_ai_instructions(root: Option<&Path>) -> bool {
    let Ok(path) = ai_instructions_path(root) else { return false };
    if !path.exists() {
        return false;
    }
    let current = fs::read_to_string(&path).unwrap_or_default();
    if instructions_version(&current) >= INSTRUCTIONS_VERSION {
        return false;
    }

    let backup = path.with_file_name(INSTRUCTIONS_BACKUP_FILE);
    let _ = fs::write(&backup, &current);
    let fresh = ai_instructions();
    let _ = fs::write(&path, &fresh);
    if let Some(r) = root {
        ensure_gitignore_entry(r, &[&format!("{}/{}", MEM_DIR, INSTRUCTIONS_BACKUP_FILE)]);
    }
    true
}

pub fn ensure_gitignore_entry(root: &Path, extra: &[&str]) {
    let gitignore = root.join(".gitignore");
    let mut entries = vec![
        format!("{}/{}", MEM_DIR, EVENTS_FILE),
        format!("{}/watch.pid", MEM_DIR),
        format!("{}/watch.log", MEM_DIR),
        format!("{}/structure.json", MEM_DIR),
    ];
    for e in extra {
        entries.push(e.to_string());
    }

    let existing = fs::read_to_string(&gitignore).unwrap_or_default();
    let existing_lines: Vec<&str> = existing.lines().map(|l| l.trim()).collect();
    let mut new_entries = Vec::new();
    for e in &entries {
        if !existing_lines.contains(&e.as_str()) {
            new_entries.push(e.clone());
        }
    }

    if new_entries.is_empty() {
        return;
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    for e in new_entries {
        updated.push_str(&e);
        updated.push('\n');
    }
    let _ = fs::write(&gitignore, updated);
}

pub fn initialize(root: Option<&Path>, purpose: Option<&str>) -> Result<PathBuf, String> {
    let target = match root {
        Some(r) => r.to_path_buf(),
        None => env::current_dir().map_err(|e| e.to_string())?,
    };

    let dir = target.join(MEM_DIR);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join(ISSUES_DIR)).map_err(|e| e.to_string())?;

    let events = dir.join(EVENTS_FILE);
    if !events.exists() {
        fs::write(&events, "").map_err(|e| e.to_string())?;
    }

    let config = dir.join(CONFIG_FILE);
    if !config.exists() {
        let desc = purpose.unwrap_or("");
        let content = format!("recent_days = 30\nproject_description = \"{}\"\n", desc);
        fs::write(&config, content).map_err(|e| e.to_string())?;
    }

    let project_name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    let summary = dir.join(SUMMARY_FILE);
    if !summary.exists() {
        let mut sum_content = initial_summary(project_name);
        if let Some(p) = purpose {
            if !p.trim().is_empty() {
                sum_content = sum_content.replace(
                    "Replace this placeholder with a concise description of what this project does, who it serves, and the main technologies or runtime assumptions.",
                    p.trim(),
                );
            }
        }
        fs::write(&summary, sum_content).map_err(|e| e.to_string())?;
    }

    let instructions = dir.join(AI_INSTRUCTIONS_FILE);
    if !instructions.exists() {
        fs::write(&instructions, ai_instructions()).map_err(|e| e.to_string())?;
    }

    let project_map = dir.join(PROJECT_MAP_FILE);
    if !project_map.exists() {
        let mut map_content = initial_project_map(project_name);
        if let Some(p) = purpose {
            if !p.trim().is_empty() {
                map_content = map_content.replace("Not described yet.", p.trim());
            }
        }
        fs::write(&project_map, map_content).map_err(|e| e.to_string())?;
    }

    let plan = dir.join(PLAN_FILE);
    if !plan.exists() {
        fs::write(&plan, initial_plan(project_name)).map_err(|e| e.to_string())?;
    }

    ensure_gitignore_entry(&target, &[]);
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_initialize_and_store() {
        let tmp = tempdir().unwrap();
        let mem = initialize(Some(tmp.path()), Some("Testing store")).unwrap();
        assert!(mem.exists());
        assert!(mem.join(CONFIG_FILE).exists());
        assert!(mem.join(SUMMARY_FILE).exists());
        assert!(mem.join(EVENTS_FILE).exists());

        let mut ev = Event::new("issue", "First bug");
        ev.issue_id = Some("0001".into());
        append_event(&mut ev, Some(tmp.path())).unwrap();

        let events = read_events(Some(tmp.path())).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "First bug");
        assert_eq!(next_issue_id(&events), "0002");
    }
}
