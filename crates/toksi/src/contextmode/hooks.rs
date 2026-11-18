// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/hooks.rs
// created: 2026-11-13

use std::path::{Path, PathBuf};
use serde_json::{json, Value};

use crate::util::paths::normalize_path;
use super::compaction::{pick_recall, render_state};
use super::filter::{get_replace_trigger_bytes, process_sandbox_output, replace_handle_line_full};
use super::fts::ContentStore;
use super::journal::{canonical_key, NewJournalRow};
use super::policy::{decide, CanonicalHookEvent, Decision, ToolCallInput};
use super::security::is_own_file;
use super::store::find_repo_root_and_subproject;

pub const GEMINI_REPLACE_LEAD: &str =
    "the tool ran and succeeded; context-mode indexed its full output, and this is the part that answers the call:\n";

/// Entrypoint for native hook execution: `toksi context-hook <agent>`
pub fn run_context_hook(args: &[String], input: &[u8]) -> i32 {
    let agent_name = resolve_agent(args);

    if input.is_empty() {
        return 0;
    }

    let raw_val: Value = match serde_json::from_slice(input) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    let payload = match raw_val.as_object() {
        Some(map) => map,
        None => return 0,
    };

    let cwd_str = payload
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| normalize_path(&p.to_string_lossy()))
                .unwrap_or_else(|_| ".".to_string())
        });

    let cwd = PathBuf::from(&cwd_str);

    // Strict repository root resolution: stops at .git root
    let (repo_root, _subproject_id) = match find_repo_root_and_subproject(Some(&cwd)) {
        Ok(res) => res,
        Err(_) => {
            // Failsafe: if outside repo, never block host agent
            return 0;
        }
    };

    let session_id = payload
        .get("session_id")
        .or_else(|| payload.get("sessionId"))
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let event_name = extract_event_name(payload);
    let worktree = normalize_path(&repo_root.to_string_lossy());

    let db_path = repo_root.join(".toksi").join("context").join("context.db");
    let mut store = match ContentStore::open(&db_path) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let canonical_event = map_to_canonical_event(payload, &event_name, &agent_name, &session_id, &cwd_str);

    match canonical_event.event.as_str() {
        "PreToolUse" => {
            let rules: Vec<String> = Vec::new(); // user rules loaded if present
            let decision = decide(&canonical_event, &agent_name, &rules, &repo_root);
            match decision {
                Decision::Deny { reason, rewritten } => {
                    let out = encode_deny(&agent_name, &reason, rewritten.as_deref());
                    if !out.is_empty() {
                        print!("{}", out);
                    }
                    0
                }
                _ => 0,
            }
        }
        "PostToolUse" => {
            let tool = canonical_event.tool.as_ref();
            let tool_name = tool.map(|t| t.name.as_str()).unwrap_or("tool");
            let canonical_tool = tool.map(|t| t.canonical_name.as_str()).unwrap_or(tool_name);

            let (output_text, is_error) = extract_tool_response(payload);
            let bytes_len = output_text.len();

            let cmd_or_key = tool.and_then(|t| t.command.clone()).unwrap_or_default();
            let key = if !cmd_or_key.is_empty() {
                Some(canonical_key(canonical_tool, &cmd_or_key))
            } else {
                tool.and_then(|t| t.file_path.clone())
            };

            let kind = if is_error {
                "toolerr"
            } else if canonical_tool == "Read" || canonical_tool == "read_file" {
                "edit"
            } else {
                "cmd"
            };

            let new_row = NewJournalRow {
                session_id: session_id.clone(),
                worktree: worktree.clone(),
                agent: agent_name.clone(),
                kind: kind.to_string(),
                tool: Some(tool_name.to_string()),
                key: key.clone(),
                path: tool.and_then(|t| t.file_path.clone()),
                ok: Some(if is_error { 0 } else { 1 }),
                exit_code: if is_error { Some(1) } else { Some(0) },
                bytes: bytes_len,
                body: output_text.clone(),
                turn: canonical_event.turn.unwrap_or(0),
                sandbox_bytes: bytes_len,
                returned_bytes: bytes_len,
            };

            let _ = store.append_journal_event(&new_row);

            // Bypass replacement for agent's own rule / instruction files
            if let Some(target_file) = tool.and_then(|t| t.file_path.as_ref())
                && is_own_file(&agent_name, Path::new(target_file))
            {
                return 0;
            }

            // #1404: Skill load results reach the model whole.
            if canonical_tool == "Skill" {
                return 0;
            }

            // #1392: omp read with an inline line range is a bounded read, not replaced.
            if agent_name == "omp"
                && canonical_tool == "Read"
                && let Some(target_file) = tool.and_then(|t| t.file_path.as_deref())
                && has_omp_line_range(target_file)
            {
                return 0;
            }

            let trigger_bytes = get_replace_trigger_bytes();
            if bytes_len > trigger_bytes {
                let source_label = format!("{}:{}", tool_name, key.as_deref().unwrap_or("output"));
                let _ = store.index(&output_text, &source_label, Some("prose"), Some(&session_id));

                // #1402: A plugin mod's MCP call (toolu_plugin_) gets its output whole after indexing.
                let call_id = payload
                    .get("tool_use_id")
                    .or_else(|| payload.get("call_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if call_id.starts_with("toolu_plugin_") && tool_name.starts_with("mcp__") {
                    return 0;
                }

                let handle_line = replace_handle_line_full(
                    &cmd_or_key,
                    &source_label,
                    bytes_len,
                    "",
                    "",
                );

                let preview = process_sandbox_output(Some(&cmd_or_key), &output_text, 30, 2048);
                let replaced = format!("{}\n\n{}", preview, handle_line);

                let out = encode_replace(&agent_name, &replaced);
                if !out.is_empty() {
                    print!("{}", out);
                }
            }
            0
        }
        "UserPromptSubmit" => {
            let prompt_text = canonical_event.prompt.clone().unwrap_or_default();
            let new_row = NewJournalRow {
                session_id: session_id.clone(),
                worktree: worktree.clone(),
                agent: agent_name.clone(),
                kind: "prompt".to_string(),
                tool: None,
                key: None,
                path: None,
                ok: Some(1),
                exit_code: None,
                bytes: prompt_text.len(),
                body: prompt_text.clone(),
                turn: canonical_event.turn.unwrap_or(0),
                sandbox_bytes: 0,
                returned_bytes: prompt_text.len(),
            };

            let _ = store.append_journal_event(&new_row);

            // Turn-boundary recall if session was compacted (#1394: bounded by compaction seq)
            let compact_flag = format!("compacted:{}", session_id);
            if let Some(boundary) = store.meta_get(&compact_flag) {
                let max_seq = parse_compacted_max_seq(&boundary);
                if let Ok(past_rows) = store.get_recall_rows_bounded(&session_id, &worktree, 50, max_seq) {
                    let recall_text = pick_recall(&past_rows, &prompt_text);
                    if !recall_text.is_empty() {
                        let out = encode_hint(&agent_name, "UserPromptSubmit", &recall_text);
                        if !out.is_empty() {
                            print!("{}", out);
                        }
                    }
                }
            }
            0
        }
        "SessionStart" => {
            let pending_compact_key = format!("pending_compact:{}", session_id);
            let source = payload.get("source").and_then(|v| v.as_str()).unwrap_or("");
            let is_compact_start = source == "compact" || store.meta_take(&pending_compact_key).is_some();
            if is_compact_start {
                // #1396: Key SessionStart(compact) state delivery per compaction using newest journal seq.
                let last_seq = store.last_session_seq(&session_id);
                let compact_once_key = format!("compact_state:{}:{}", session_id, last_seq);
                if store.meta_get(&compact_once_key).is_none()
                    && let Ok(events) = store.get_session_events(&session_id, None)
                {
                    let state_block = render_state(&events, 0, "");
                    if !state_block.is_empty() {
                        let _ = store.meta_set(&compact_once_key, "1");
                        let out = encode_hint(&agent_name, "SessionStart", &state_block);
                        if !out.is_empty() {
                            print!("{}", out);
                        }
                    }
                }
            }
            0
        }
        "PreCompact" => {
            let pending_compact_key = format!("pending_compact:{}", session_id);
            let compacted_key = format!("compacted:{}", session_id);
            let last_seq = store.last_session_seq(&session_id);
            let _ = store.meta_set(&pending_compact_key, "1");
            // #1394: Store newest journal seq at compaction boundary so post-compact rows aren't recalled.
            let _ = store.meta_set(&compacted_key, &format!("seq:{}", last_seq));

            // Take pre-compact state snapshot
            if let Ok(events) = store.get_session_events(&session_id, None) {
                let snapshot = render_state(&events, 0, "");
                let snap_key = format!("snapshot:{}", session_id);
                let _ = store.meta_set(&snap_key, &snapshot);
            }
            0
        }
        "Usage" => {
            let new_row = NewJournalRow {
                session_id: session_id.clone(),
                worktree: worktree.clone(),
                agent: agent_name.clone(),
                kind: "cost".to_string(),
                tool: None,
                key: None,
                path: None,
                ok: Some(1),
                exit_code: None,
                bytes: 0,
                body: String::new(),
                turn: canonical_event.turn.unwrap_or(0),
                sandbox_bytes: 0,
                returned_bytes: 0,
            };
            let _ = store.append_journal_event(&new_row);
            0
        }
        _ => 0,
    }
}

fn resolve_agent(args: &[String]) -> String {
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--agent" || args[i] == "-a") && i + 1 < args.len() {
            return args[i + 1].to_lowercase();
        }
        i += 1;
    }
    if let Some(first) = args.first() {
        let low = first.to_lowercase();
        if !low.starts_with('-') {
            return low;
        }
    }
    "generic".to_string()
}

fn extract_event_name(payload: &serde_json::Map<String, Value>) -> String {
    if let Some(n) = payload.get("hook_event_name").and_then(|v| v.as_str()) {
        return n.to_string();
    }
    if let Some(n) = payload.get("event").and_then(|v| v.as_str()) {
        return n.to_string();
    }
    "PreToolUse".to_string()
}

fn map_to_canonical_event(
    payload: &serde_json::Map<String, Value>,
    event_name: &str,
    agent: &str,
    session_id: &str,
    cwd: &str,
) -> CanonicalHookEvent {
    let canonical_event_str = match event_name {
        "BeforeTool" | "PreToolUse" => "PreToolUse",
        "AfterTool" | "PostToolUse" => "PostToolUse",
        "BeforeAgent" | "UserPromptSubmit" => "UserPromptSubmit",
        "SessionStart" => "SessionStart",
        "PreCompress" | "PreCompact" => "PreCompact",
        "Usage" | "step-finish" => "Usage",
        other => other,
    };

    let prompt = payload.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string());
    let turn = payload.get("turn").and_then(|v| v.as_u64()).map(|n| n as usize);

    let tool = if canonical_event_str == "PreToolUse" || canonical_event_str == "PostToolUse" {
        let name = payload.get("tool_name").and_then(|v| v.as_str()).unwrap_or("tool").to_string();
        let canonical_name = map_tool_to_canonical(&name);
        let input_obj = payload.get("tool_input").and_then(|v| v.as_object());

        let command = input_obj.and_then(|o| o.get("command").and_then(|v| v.as_str())).map(|s| s.to_string());
        let file_path = input_obj.and_then(|o| {
            o.get("path")
                .or_else(|| o.get("file_path"))
                .or_else(|| o.get("file"))
                .and_then(|v| v.as_str())
        }).map(|s| s.to_string());
        let url = input_obj.and_then(|o| o.get("url").and_then(|v| v.as_str())).map(|s| s.to_string());
        let description = input_obj.and_then(|o| o.get("description").and_then(|v| v.as_str())).map(|s| s.to_string());
        let language = input_obj.and_then(|o| o.get("language").and_then(|v| v.as_str())).map(|s| s.to_string());

        Some(ToolCallInput {
            name,
            canonical_name,
            command,
            file_path,
            url,
            description,
            language,
            raw_json: payload.get("tool_input").cloned(),
        })
    } else {
        None
    };

    CanonicalHookEvent {
        event: canonical_event_str.to_string(),
        agent: agent.to_string(),
        session_id: session_id.to_string(),
        cwd: cwd.to_string(),
        prompt,
        tool,
        turn,
        is_subagent: payload.get("agent_id").is_some(),
        agent_type: payload.get("agent_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
    }
}

fn map_tool_to_canonical(name: &str) -> String {
    match name {
        "run_shell_command" | "execute_command" | "Bash" | "PowerShell" => "Bash".to_string(),
        "read_file" | "Read" | "view" | "read" => "Read".to_string(),
        "web_fetch" | "WebFetch" => "WebFetch".to_string(),
        "grep_search" | "Grep" => "Grep".to_string(),
        "skill" | "Skill" => "Skill".to_string(),
        other => {
            if let Some(stripped) = other.strip_prefix("mcp_context-mode_") {
                stripped.to_string()
            } else if let Some(stripped) = other.strip_prefix("mcp__plugin_context-mode_context-mode__") {
                stripped.to_string()
            } else {
                other.to_string()
            }
        }
    }
}

/// #1394: Parses the compaction seq boundary from `compacted:<sid>` ("seq:<N>" or legacy "1").
fn parse_compacted_max_seq(raw: &str) -> i64 {
    if let Some(rest) = raw.strip_prefix("seq:")
        && let Ok(n) = rest.trim().parse::<i64>()
    {
        return n;
    }
    i64::MAX
}

/// #1392: Detects OMP inline read line ranges like `file.rs:10-20`, `file.rs:10+5`, `file.rs:-20`,
/// `file.rs:10,20`, or `file.rs:10-20:raw`, without misclassifying Windows drive letters or bare `:raw`.
pub(crate) fn has_omp_line_range(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    let without_raw = trimmed.strip_suffix(":raw").unwrap_or(trimmed);
    let Some((prefix, suffix)) = without_raw.rsplit_once(':') else {
        return false;
    };
    // Avoid Windows drive letter like "C:" or empty prefix
    if prefix.len() <= 1 || suffix.is_empty() {
        return false;
    }
    // Must contain only digits, '-', '+', ',' and at least one digit
    let has_digit = suffix.chars().any(|c| c.is_ascii_digit());
    let valid_chars = suffix
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == ',');
    has_digit && valid_chars
}

fn extract_tool_response(payload: &serde_json::Map<String, Value>) -> (String, bool) {
    if let Some(resp) = payload.get("tool_response") {
        if let Some(s) = resp.as_str() {
            return (s.to_string(), false);
        }
        if let Some(obj) = resp.as_object() {
            let is_err = obj.get("error").is_some() || obj.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
            if let Some(llm) = obj.get("llmContent").and_then(|v| v.as_str()) {
                return (llm.to_string(), is_err);
            }
            if let Some(stdout) = obj.get("stdout").and_then(|v| v.as_str()) {
                let stderr = obj.get("stderr").and_then(|v| v.as_str()).unwrap_or("");
                let combined = if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };
                return (combined, is_err);
            }
            return (serde_json::to_string_pretty(resp).unwrap_or_default(), is_err);
        }
    }
    (String::new(), false)
}

fn encode_deny(agent: &str, reason: &str, rewritten: Option<&str>) -> String {
    let effective_reason = if let Some(rw) = rewritten {
        format!("{}\n{}", rw, reason)
    } else {
        reason.to_string()
    };

    match agent {
        "gemini" | "agy" | "antigravity" => {
            json!({
                "decision": "deny",
                "reason": effective_reason
            }).to_string()
        }
        "claude" | "claude-code" | "codex" => {
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": effective_reason
                }
            }).to_string()
        }
        _ => {
            json!({
                "decision": "deny",
                "reason": effective_reason
            }).to_string()
        }
    }
}

fn encode_replace(agent: &str, replaced: &str) -> String {
    match agent {
        "gemini" | "agy" | "antigravity" => {
            json!({
                "decision": "deny",
                "reason": format!("{}{}", GEMINI_REPLACE_LEAD, replaced)
            }).to_string()
        }
        "claude" | "claude-code" | "codex" => {
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "PostToolUse",
                    "updatedToolOutput": replaced
                }
            }).to_string()
        }
        _ => {
            json!({
                "updatedToolOutput": replaced
            }).to_string()
        }
    }
}

fn encode_hint(agent: &str, event_name: &str, hint_text: &str) -> String {
    match agent {
        "gemini" | "agy" | "antigravity" => {
            json!({
                "hookSpecificOutput": {
                    "additionalContext": hint_text
                }
            }).to_string()
        }
        "claude" | "claude-code" | "codex" => {
            json!({
                "hookSpecificOutput": {
                    "hookEventName": event_name,
                    "additionalContext": hint_text
                }
            }).to_string()
        }
        _ => {
            json!({
                "additionalContext": hint_text
            }).to_string()
        }
    }
}
