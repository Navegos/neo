// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/mcp.rs
// created: 2026-10-27

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use serde_json::{json, Value};

use super::heatmap::calculate_score;
use super::precheck::analyze_files;
use super::schema::{resolve_event_ref, Event};
use super::store::{
    active_issue_marker, ai_instructions_path, append_event, clear_current_issue,
    current_issue_id, get_git_commit, issue_exists, issues_dir, latest_open_issue_within,
    next_issue_id, normalize_issue_id, plan_path, project_map_path, read_events, require_mem_dir,
    summary_path, write_current_issue,
};
use super::summary::regenerate_summary;

pub fn resolve_project_root(project_arg: Option<&str>) -> Result<PathBuf, String> {
    if let Some(p) = project_arg {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Ok(pb);
        }
    }
    let mem = require_mem_dir(None)?;
    Ok(mem.parent().unwrap_or(&mem).to_path_buf())
}

pub fn tool_get_instructions(root: &Path) -> String {
    if let Ok(p) = ai_instructions_path(Some(root)) {
        if p.exists() {
            return fs::read_to_string(p).unwrap_or_else(|_| "Failed to read instructions.".into());
        }
    }
    "No instructions found. Run `tokless mem init` first.".to_string()
}

pub fn tool_get_summary(root: &Path) -> String {
    if let Ok(p) = summary_path(Some(root)) {
        if p.exists() {
            return fs::read_to_string(p).unwrap_or_else(|_| "Failed to read summary.".into());
        }
    }
    "No summary found. Run `tokless mem init` first.".to_string()
}

pub fn tool_get_project_map(root: &Path) -> String {
    if let Ok(p) = project_map_path(Some(root)) {
        if p.exists() {
            return fs::read_to_string(p).unwrap_or_else(|_| "Failed to read project map.".into());
        }
    }
    "No project map found.".to_string()
}

pub fn tool_get_plan(root: &Path) -> String {
    if let Ok(p) = plan_path(Some(root)) {
        if p.exists() {
            return fs::read_to_string(p).unwrap_or_else(|_| "Failed to read plan.".into());
        }
    }
    "No plan found.".to_string()
}

pub fn tool_precheck_file(root: &Path, file_path: &str) -> String {
    if super::precheck::is_memory_path(file_path) {
        return format!(
            "{}: this is a projectmem memory file. It is regenerated from the event log, so do not edit it; record the change with add_note / add_decision / log_issue / record_fix instead.",
            file_path
        );
    }
    let events = match read_events(Some(root)) {
        Ok(evs) => evs,
        Err(e) => return format!("Failed to read memory: {}", e),
    };
    let warnings = analyze_files(&[file_path.to_string()], &events, Some(root));
    if warnings.is_empty() {
        return format!("{}: no warnings. Safe to modify.", file_path);
    }

    let mut lines = vec![format!("projectmem precheck: {}", file_path)];
    for w in warnings {
        lines.push(format!("  [{}] {}", w.severity.to_uppercase(), w.title));
        for d in w.details {
            lines.push(format!("    - {}", d));
        }
    }
    lines.join("\n")
}

pub fn tool_get_issue(root: &Path, issue_id: &str) -> String {
    let idir = match issues_dir(Some(root)) {
        Ok(d) => d,
        Err(e) => return e,
    };
    let norm = normalize_issue_id(Some(issue_id)).unwrap_or_else(|| issue_id.to_string());
    if let Ok(entries) = fs::read_dir(&idir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}-", norm)) && name.ends_with(".md") {
                return fs::read_to_string(entry.path()).unwrap_or_else(|_| "Failed to read issue file.".into());
            }
        }
    }
    format!("No issue found with ID {}.", issue_id)
}

pub fn tool_search_events(root: &Path, query: &str, limit: usize) -> String {
    let events = match read_events(Some(root)) {
        Ok(evs) => evs,
        Err(e) => return format!("Failed to read memory: {}", e),
    };
    let q = query.to_lowercase();
    let mut matches = Vec::new();
    for e in &events {
        if e.summary.to_lowercase().contains(&q)
            || e.notes.as_ref().map(|n| n.to_lowercase().contains(&q)).unwrap_or(false)
        {
            matches.push(e.clone());
        }
    }

    let max_len = limit.clamp(1, 100);
    let start = matches.len().saturating_sub(max_len);
    let slice = &matches[start..];

    if slice.is_empty() {
        return format!("No events match '{}'.", query);
    }

    let mut lines = vec![format!("Found {} match(es) for '{}':", matches.len(), query)];
    for e in slice {
        let outcome = match e.outcome {
            Some(ref o) => format!(" ({})", o),
            None => String::new(),
        };
        let loc = match e.location {
            Some(ref l) => format!(" @ {}", l),
            None => String::new(),
        };
        lines.push(format!("  [{}{}] {}{}", e.event_type, outcome, e.summary, loc));
    }
    lines.join("\n")
}

pub fn tool_get_score(root: &Path) -> String {
    let events = match read_events(Some(root)) {
        Ok(evs) => evs,
        Err(e) => return format!("Failed to read memory: {}", e),
    };
    let res = calculate_score(&events, None);
    format!(
        "projectmem Prevention Score: {} ({}/100)\n  Failed approaches on record: {}\n  Decisions documented: {}\n  Fixes with context: {}\n  Debugging hours saved: ~{:.1}h\n  Tokens saved: {}\n  Estimated USD saved: ${:.2}",
        res.grade,
        res.score,
        res.components.failed_approaches,
        res.components.decisions_documented,
        res.components.fixes_with_context,
        res.debugging_hours_saved,
        res.tokens_saved,
        res.usd_saved
    )
}

pub fn tool_get_context(root: &Path, _tokens: usize, focus: Option<&str>) -> String {
    let events = match read_events(Some(root)) {
        Ok(evs) => evs,
        Err(e) => return format!("Failed to read memory: {}", e),
    };
    let score = calculate_score(&events, None);
    let mut lines = vec![
        "## projectmem context".to_string(),
        String::new(),
        format!("Score: {}/100 ({}) | Tokens Saved: {}", score.score, score.grade, score.tokens_saved),
        String::new(),
    ];

    if let Some(f) = focus {
        lines.push(format!("Focus area: `{}`", f));
        lines.push(String::new());
    }

    lines.push("### Recent Decisions".to_string());
    let decisions: Vec<&Event> = events.iter().filter(|e| e.event_type == "decision").collect();
    if decisions.is_empty() {
        lines.push("- None logged yet.".to_string());
    } else {
        for d in decisions.iter().rev().take(5) {
            lines.push(format!("- {}", d.summary));
        }
    }

    lines.push(String::new());
    lines.push("### Recent Fixes".to_string());
    let fixes: Vec<&Event> = events.iter().filter(|e| e.event_type == "fix").collect();
    if fixes.is_empty() {
        lines.push("- None logged yet.".to_string());
    } else {
        for f in fixes.iter().rev().take(5) {
            lines.push(format!("- Fixed: {}", f.summary));
        }
    }

    lines.join("\n")
}

pub fn tool_log_issue(root: &Path, summary: &str, location: Option<&str>) -> Result<String, String> {
    let events = read_events(Some(root))?;
    let issue_id = next_issue_id(&events);

    let mut event = Event::new("issue", summary);
    event.issue_id = Some(issue_id.clone());
    event.git_commit = get_git_commit(Some(root));
    event.location = location.map(|s| s.to_string());

    append_event(&mut event, Some(root))?;
    write_current_issue(&issue_id, Some(root));
    let _ = regenerate_summary(Some(root));

    let proj_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    Ok(format!("Logged issue #{} → {}: {}", issue_id, proj_name, summary))
}

pub fn tool_record_attempt(
    root: &Path,
    summary: &str,
    outcome: Option<&str>,
    location: Option<&str>,
    issue_id_arg: Option<&str>,
    command: Option<&str>,
    notes: Option<&str>,
) -> Result<String, String> {
    let events = read_events(Some(root))?;
    let mut target_issue_id: Option<String> = None;

    if let Some(arg) = issue_id_arg {
        if let Some(norm) = normalize_issue_id(Some(arg)) {
            if issue_exists(&events, &norm) {
                target_issue_id = Some(norm);
            } else {
                return Err(format!("Issue #{} was not found.", arg));
            }
        }
    }

    if target_issue_id.is_none() {
        target_issue_id = active_issue_marker(&events, Some(root))
            .or_else(|| latest_open_issue_within(&events, 5));
    }

    if target_issue_id.is_none() {
        // Auto-create parent issue
        let new_id = next_issue_id(&events);
        let mut auto_issue = Event::new("issue", summary);
        auto_issue.issue_id = Some(new_id.clone());
        auto_issue.git_commit = get_git_commit(Some(root));
        auto_issue.location = location.map(|s| s.to_string());
        append_event(&mut auto_issue, Some(root))?;
        write_current_issue(&new_id, Some(root));
        target_issue_id = Some(new_id);
    }

    let iid = target_issue_id.unwrap();
    let out = outcome.unwrap_or("failed");

    // RTK root cause extraction: refine summary if raw error output or command is provided
    let cmd_str = command.unwrap_or("");
    let (final_summary, extracted_notes) = if out == "failed" {
        if let Some(root_cause) = crate::rtk::extract_root_cause(cmd_str, summary) {
            if summary.lines().count() > 1 {
                (format!("Failed: {}", root_cause), Some(summary.to_string()))
            } else {
                (summary.to_string(), None)
            }
        } else {
            (summary.to_string(), None)
        }
    } else {
        (summary.to_string(), None)
    };

    let mut event = Event::new("attempt", &final_summary);
    event.issue_id = Some(iid.clone());
    event.outcome = Some(out.to_string());
    event.git_commit = get_git_commit(Some(root));
    event.location = location.map(|s| s.to_string());
    event.command = command.map(|s| s.to_string());
    event.notes = notes.map(|s| s.to_string()).or(extracted_notes);

    append_event(&mut event, Some(root))?;
    let _ = regenerate_summary(Some(root));

    let proj_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    Ok(format!("Recorded {} attempt on #{} → {}: {}", out, iid, proj_name, final_summary))
}

pub fn tool_record_fix(
    root: &Path,
    summary: &str,
    location: Option<&str>,
    issue_id_arg: Option<&str>,
) -> Result<String, String> {
    let events = read_events(Some(root))?;
    let target_issue_id = if let Some(arg) = issue_id_arg {
        let norm = normalize_issue_id(Some(arg)).ok_or_else(|| "Invalid issue id".to_string())?;
        if !issue_exists(&events, &norm) {
            return Err(format!("Issue #{} was not found.", norm));
        }
        norm
    } else {
        active_issue_marker(&events, Some(root))
            .or_else(|| current_issue_id(&events))
            .ok_or_else(|| "No open issue found. Run `tokless mem log <text>` first.".to_string())?
    };

    let mut event = Event::new("fix", summary);
    event.issue_id = Some(target_issue_id.clone());
    event.git_commit = get_git_commit(Some(root));
    event.location = location.map(|s| s.to_string());

    append_event(&mut event, Some(root))?;

    if let Some(active) = active_issue_marker(&events, Some(root)) {
        if active == target_issue_id {
            clear_current_issue(Some(root));
        }
    }

    let _ = regenerate_summary(Some(root));
    let proj_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    Ok(format!("Fixed issue #{} → {}: {}", target_issue_id, proj_name, summary))
}

pub fn tool_add_decision(
    root: &Path,
    summary: &str,
    location: Option<&str>,
    supersedes: Option<&str>,
) -> Result<String, String> {
    let events = read_events(Some(root))?;
    let mut superseded_id = None;
    if let Some(ref_str) = supersedes {
        let old = resolve_event_ref(&events, ref_str)?;
        superseded_id = Some(old.id);
    }

    let mut event = Event::new("decision", summary);
    event.git_commit = get_git_commit(Some(root));
    event.location = location.map(|s| s.to_string());
    event.supersedes = superseded_id.clone();

    append_event(&mut event, Some(root))?;
    let _ = regenerate_summary(Some(root));

    let proj_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    if let Some(sup) = superseded_id {
        Ok(format!("Recorded decision → {} (supersedes {}): {}", proj_name, sup, summary))
    } else {
        Ok(format!("Recorded decision → {}: {}", proj_name, summary))
    }
}

pub fn tool_add_note(root: &Path, summary: &str, location: Option<&str>) -> Result<String, String> {
    let mut event = Event::new("note", summary);
    event.git_commit = get_git_commit(Some(root));
    event.location = location.map(|s| s.to_string());

    append_event(&mut event, Some(root))?;
    let _ = regenerate_summary(Some(root));

    let proj_name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
    Ok(format!("Recorded note → {}: {}", proj_name, summary))
}

pub fn mcp_tools_list() -> Value {
    json!([
        {
            "name": "get_instructions",
            "description": "Read the project's mandatory AI instructions at session start.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "get_summary",
            "description": "Read the project memory summary before answering questions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "get_project_map",
            "description": "Read PROJECT_MAP.md to understand the repo structure.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "get_plan",
            "description": "Read plan.md — the project's INTENT file (ideas + plans).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "precheck_file",
            "description": "Check a file's failure history BEFORE modifying it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Project-relative file path" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["file_path"]
            }
        },
        {
            "name": "get_issue",
            "description": "Read one specific issue's full history by ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "issue_id": { "type": "string", "description": "Zero-padded 4-digit issue ID" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["issue_id"]
            }
        },
        {
            "name": "search_events",
            "description": "Plain-text search across all logged events.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Case-insensitive substring" },
                    "limit": { "type": "integer", "description": "Max results (default 10)" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["query"]
            }
        },
        {
            "name": "get_score",
            "description": "Get the project's failure-prevention score.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "get_context",
            "description": "Generate a token-budgeted memory context block.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tokens": { "type": "integer", "description": "Target token budget" },
                    "focus": { "type": "string", "description": "Optional path prefix" },
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "get_global_gotchas",
            "description": "Query cross-project library gotchas.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "library": { "type": "string", "description": "Library filter" },
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        },
        {
            "name": "log_issue",
            "description": "Open a new issue. Returns the issue ID.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "summary": { "type": "string", "description": "One-line description of the bug" },
                    "location": { "type": "string", "description": "Optional file path or component" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["summary"]
            }
        },
        {
            "name": "record_attempt",
            "description": "Record a fix attempt on the current issue.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "summary": { "type": "string", "description": "One-line description of what was tried" },
                    "outcome": { "type": "string", "enum": ["worked", "failed", "partial"] },
                    "location": { "type": "string", "description": "Optional file path" },
                    "issue_id": { "type": "string", "description": "Optional issue ID" },
                    "command": { "type": "string", "description": "Optional command executed (e.g. cargo test, pytest)" },
                    "notes": { "type": "string", "description": "Optional failure trace or extra notes" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["summary"]
            }
        },
        {
            "name": "record_fix",
            "description": "Record a confirmed fix and close an issue.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "summary": { "type": "string", "description": "One-line description of confirmed fix" },
                    "location": { "type": "string", "description": "Optional file path" },
                    "issue_id": { "type": "string", "description": "Optional issue ID to close" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["summary"]
            }
        },
        {
            "name": "add_decision",
            "description": "Record an architectural or product decision permanently.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "summary": { "type": "string", "description": "Description of decision" },
                    "location": { "type": "string", "description": "Optional file path" },
                    "supersedes": { "type": "string", "description": "Optional event id being retired" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["summary"]
            }
        },
        {
            "name": "add_note",
            "description": "Record a gotcha, setup detail, or other durable context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "summary": { "type": "string", "description": "Description of gotcha or note" },
                    "location": { "type": "string", "description": "Optional file path or library" },
                    "project": { "type": "string", "description": "Optional project root" }
                },
                "required": ["summary"]
            }
        },
        {
            "name": "list_projects",
            "description": "List the projects this server can reach.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "current_project",
            "description": "Show which project a call would resolve to.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project": { "type": "string", "description": "Optional project root" }
                }
            }
        }
    ])
}

pub fn execute_mcp_tool(name: &str, args: &Value) -> String {
    let project_arg = args.get("project").and_then(|v| v.as_str());
    let root = match resolve_project_root(project_arg) {
        Ok(r) => r,
        Err(e) => return format!("projectmem tool error: {}", e),
    };

    match name {
        "get_instructions" => tool_get_instructions(&root),
        "get_summary" => tool_get_summary(&root),
        "get_project_map" => tool_get_project_map(&root),
        "get_plan" => tool_get_plan(&root),
        "precheck_file" => {
            let fp = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            tool_precheck_file(&root, fp)
        }
        "get_issue" => {
            let id = args.get("issue_id").and_then(|v| v.as_str()).unwrap_or("");
            tool_get_issue(&root, id)
        }
        "search_events" => {
            let q = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            tool_search_events(&root, q, limit)
        }
        "get_score" => tool_get_score(&root),
        "get_context" => {
            let tokens = args.get("tokens").and_then(|v| v.as_u64()).unwrap_or(2000) as usize;
            let focus = args.get("focus").and_then(|v| v.as_str());
            tool_get_context(&root, tokens, focus)
        }
        "get_global_gotchas" => "No global gotchas found.".to_string(),
        "log_issue" => {
            let summary = args.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let loc = args.get("location").and_then(|v| v.as_str());
            tool_log_issue(&root, summary, loc).unwrap_or_else(|e| format!("projectmem tool error: {}", e))
        }
        "record_attempt" => {
            let summary = args.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let outcome = args.get("outcome").and_then(|v| v.as_str());
            let loc = args.get("location").and_then(|v| v.as_str());
            let iid = args.get("issue_id").and_then(|v| v.as_str());
            let cmd = args.get("command").and_then(|v| v.as_str());
            let notes = args.get("notes").and_then(|v| v.as_str());
            tool_record_attempt(&root, summary, outcome, loc, iid, cmd, notes)
                .unwrap_or_else(|e| format!("projectmem tool error: {}", e))
        }
        "record_fix" => {
            let summary = args.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let loc = args.get("location").and_then(|v| v.as_str());
            let iid = args.get("issue_id").and_then(|v| v.as_str());
            tool_record_fix(&root, summary, loc, iid)
                .unwrap_or_else(|e| format!("projectmem tool error: {}", e))
        }
        "add_decision" => {
            let summary = args.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let loc = args.get("location").and_then(|v| v.as_str());
            let sup = args.get("supersedes").and_then(|v| v.as_str());
            tool_add_decision(&root, summary, loc, sup)
                .unwrap_or_else(|e| format!("projectmem tool error: {}", e))
        }
        "add_note" => {
            let summary = args.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let loc = args.get("location").and_then(|v| v.as_str());
            tool_add_note(&root, summary, loc).unwrap_or_else(|e| format!("projectmem tool error: {}", e))
        }
        "list_projects" => {
            let name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
            format!("ACTIVE: {} — {}\n1 project(s)", name, root.display())
        }
        "current_project" => {
            let name = root.file_name().and_then(|s| s.to_str()).unwrap_or("project");
            format!("{} — {}", name, root.display())
        }
        _ => format!("projectmem tool error: Unknown tool '{}'", name),
    }
}

pub fn run_mcp_server() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    let mut line = String::new();
    while let Ok(n) = reader.read_line(&mut line) {
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                line.clear();
                continue;
            }
        };

        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");

        let resp: Option<Value> = match method {
            "initialize" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "projectmem",
                        "version": "0.3.4"
                    },
                    "capabilities": {
                        "tools": {}
                    },
                    "instructions": "You are connected to projectmem — a mandatory project memory + judgment layer."
                }
            })),
            "notifications/initialized" => None,
            "ping" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            })),
            "tools/list" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": mcp_tools_list()
                }
            })),
            "tools/call" => {
                let params = req.get("params");
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let args = params
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                let result_text = execute_mcp_tool(name, &args);
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": result_text
                            }
                        ]
                    }
                }))
            }
            _ => {
                if id.is_some() {
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("Method not found: {}", method)
                        }
                    }))
                } else {
                    None
                }
            }
        };

        if let Some(r) = resp {
            if let Ok(serialized) = serde_json::to_string(&r) {
                let _ = writeln!(writer, "{}", serialized);
                let _ = writer.flush();
            }
        }

        line.clear();
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectmem::store::initialize;
    use tempfile::tempdir;

    #[test]
    fn test_mcp_tools_execution() {
        let tmp = tempdir().unwrap();
        initialize(Some(tmp.path()), Some("MCP test")).unwrap();

        let root = tmp.path();
        let log_res = tool_log_issue(root, "Bug in MCP", Some("src/mcp.rs:10")).unwrap();
        assert!(log_res.contains("Logged issue #0001"));

        let att_res = tool_record_attempt(root, "Tried buffer check", Some("failed"), None, None, None, None).unwrap();
        assert!(att_res.contains("Recorded failed attempt on #0001"));

        let fix_res = tool_record_fix(root, "Fixed buffer handling", None, None).unwrap();
        assert!(fix_res.contains("Fixed issue #0001"));

        let dec_res = tool_add_decision(root, "Use fixed size buffer", None, None).unwrap();
        assert!(dec_res.contains("Recorded decision"));

        let note_res = tool_add_note(root, "gotcha: buffer flush required", None).unwrap();
        assert!(note_res.contains("Recorded note"));

        let sum = tool_get_summary(root);
        assert!(sum.contains("Fixed buffer handling"));
    }
}
