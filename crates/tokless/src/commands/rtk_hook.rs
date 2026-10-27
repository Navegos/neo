// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/rtk_hook.rs
// created: 2026-10-05
// lastModified: 2026-10-27

use std::env;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;

use crate::util::jsonc::{stringify_json, try_parse_jsonc, try_parse_jsonc_map, OrderedMap, Value};
use crate::util::paths::{copilot_paths_resolved, resolve_rtk_bin};

pub fn run_rtk_rewrite() -> i32 {
    let args: Vec<String> = env::args().skip(2).collect();
    let mut actual_args = &args[..];
    for (i, a) in args.iter().enumerate() {
        if a == "--" {
            actual_args = &args[i + 1..];
            break;
        }
    }
    let cmd_line = actual_args.join(" ").trim().to_string();
    let (out, changed) = rtk_rewrite(&cmd_line);
    if !changed {
        return 1;
    }
    println!("{}", out);
    0
}

pub fn is_already_rtk(cmd_line: &str) -> bool {
    segment_starts_with_rtk(cmd_line.trim()) && shell_command_segments(cmd_line).len() <= 1
}

pub fn rtk_rewrite_if_needed(cmd_line: &str) -> (String, bool) {
    rtk_rewrite(cmd_line)
}

pub fn rtk_rewrite(cmd_line: &str) -> (String, bool) {
    if cmd_line.is_empty() || rtk_unsafe_find(cmd_line) || has_shell_pipe(cmd_line) {
        return (String::new(), false);
    }
    let (native_rewritten, changed) = crate::rtk::rtk_rewrite(cmd_line);
    if changed && !native_rewritten.is_empty() {
        return (native_rewritten, true);
    }

    let rtk_path = resolve_rtk_bin();
    if rtk_path.is_empty() {
        return (String::new(), false);
    }
    let output = match Command::new(&rtk_path).args(["rewrite", cmd_line]).output() {
        Ok(o) => o,
        Err(_) => return (String::new(), false),
    };
    if !output.status.success() && output.status.code() != Some(3) {
        return (String::new(), false);
    }
    let new_cmd = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if new_cmd.is_empty() || new_cmd == cmd_line {
        return (String::new(), false);
    }
    (new_cmd, true)
}

fn has_shell_pipe(line: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    for c in line.chars() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '|' if !in_single && !in_double => return true,
            _ => {}
        }
    }
    false
}

pub fn rtk_unsafe_find(cmd_line: &str) -> bool {
    let unsupported = [
        "-not", "!", "-or", "-o", "-and", "-a", "-exec", "-execdir", "-delete", "-print0",
        "-newer", "-perm", "-size", "-mtime", "-mmin", "-atime", "-amin", "-ctime", "-cmin",
        "-empty", "-link", "-regex", "-iregex",
    ];
    for part in shell_command_segments(cmd_line) {
        let tokens = shell_tokens(&part);
        if tokens.len() < 2 || tokens[0] != "find" {
            continue;
        }
        for token in &tokens[1..] {
            if unsupported.contains(&token.as_str()) {
                return true;
            }
        }
    }
    false
}

fn shell_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' if !in_single => {
                if i + 1 < chars.len() {
                    buf.push(chars[i + 1]);
                    i += 1;
                }
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' | '\n' if !in_single && !in_double => {
                if !buf.is_empty() {
                    out.push(std::mem::take(&mut buf));
                }
            }
            _ => buf.push(c),
        }
        i += 1;
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

pub fn shell_command_segments(line: &str) -> Vec<String> {
    let mut segs = Vec::new();
    let mut seg = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    let mut flush = |seg_buf: &mut String| {
        let trimmed = seg_buf.trim().to_string();
        if !trimmed.is_empty() {
            segs.push(trimmed);
        }
        seg_buf.clear();
    };

    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' if !in_single => {
                seg.push(c);
                if i + 1 < chars.len() {
                    seg.push(chars[i + 1]);
                    i += 1;
                }
            }
            '\'' if !in_double => {
                in_single = !in_single;
                seg.push(c);
            }
            '"' if !in_single => {
                in_double = !in_double;
                seg.push(c);
            }
            '&' if !in_single && !in_double && i + 1 < chars.len() && chars[i + 1] == '&' => {
                flush(&mut seg);
                i += 1;
            }
            '|' if !in_single && !in_double && i + 1 < chars.len() && chars[i + 1] == '|' => {
                flush(&mut seg);
                i += 1;
            }
            '|' | ';' if !in_single && !in_double => {
                flush(&mut seg);
            }
            _ => seg.push(c),
        }
        i += 1;
    }
    flush(&mut seg);
    segs
}

pub fn first_raw_token(s: &str) -> String {
    let mut buf = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for c in s.chars() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' | '\n' if !in_single && !in_double => return buf,
            _ => buf.push(c),
        }
    }
    buf
}

pub fn segment_binary(seg: &str) -> String {
    let trimmed = seg.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut bin = first_raw_token(trimmed);
    if bin.len() >= 2
        && ((bin.starts_with('"') && bin.ends_with('"'))
            || (bin.starts_with('\'') && bin.ends_with('\'')))
    {
        bin = bin[1..bin.len() - 1].to_string();
    }
    if let Some(idx) = bin.rfind(['/', '\\']) {
        bin = bin[idx + 1..].to_string();
    }
    let lower = bin.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

pub fn segment_starts_with_rtk(seg: &str) -> bool {
    segment_binary(seg) == "rtk"
}

pub fn command_uses_rtk(cmd_line: &str) -> bool {
    shell_command_segments(cmd_line)
        .iter()
        .any(|seg| segment_starts_with_rtk(seg))
}

pub fn copilot_rtk_decide(cmd_line: &str) -> (String, bool, bool) {
    let (new_cmd, changed) = rtk_rewrite(cmd_line);
    if changed {
        return (new_cmd, true, true);
    }
    if command_uses_rtk(cmd_line) {
        return (String::new(), false, true);
    }
    (String::new(), false, false)
}

pub fn cline_rewrite_commands(cmd_line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for seg in shell_command_segments(cmd_line) {
        let (rewritten, changed) = rtk_rewrite(&seg);
        if changed && !rewritten.is_empty() {
            out.push(rewritten);
        }
    }
    out
}

pub fn cline_rewrite_commands_input(tool_input: &OrderedMap) -> Option<Value> {
    match tool_input.get("commands")? {
        Value::String(commands) => {
            let (new_cmd, did_change, approve) = copilot_rtk_decide(commands);
            if approve && did_change && !new_cmd.is_empty() && &new_cmd != commands {
                Some(Value::String(new_cmd))
            } else {
                None
            }
        }
        Value::Array(commands) => {
            let mut updated = Vec::with_capacity(commands.len());
            let mut changed = false;
            for item in commands {
                if let Value::String(cmd_line) = item {
                    let (new_cmd, did_change, approve) = copilot_rtk_decide(cmd_line);
                    if approve && did_change && !new_cmd.is_empty() && &new_cmd != cmd_line {
                        updated.push(Value::String(new_cmd));
                        changed = true;
                        continue;
                    }
                }
                updated.push(item.clone());
            }
            if changed {
                Some(Value::Array(updated))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn cline_tool_input(req: &OrderedMap) -> Option<(String, OrderedMap)> {
    if let Some(Value::Object(tc)) = req.get("tool_call") {
        if let Some(name) = tc.get("name").and_then(|v| v.as_str())
            && !name.is_empty()
        {
            let input = tc
                .get("input")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            return Some((name.to_string(), input));
        }
    }
    if let Some(Value::Object(pt)) = req.get("preToolUse") {
        if let Some(name) = pt.get("toolName").and_then(|v| v.as_str())
            && !name.is_empty()
        {
            let params = pt
                .get("parameters")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            return Some((name.to_string(), params));
        }
    }
    None
}

pub fn cline_shell_tool(name: &str) -> bool {
    matches!(name, "execute_command" | "run_commands")
}

pub fn copilot_rtk_trace_cmd(cmd_line: &str) {
    let trace_path = Path::new(&copilot_paths_resolved().dir).join("tokless-rtk.log");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("{ts} | {cmd_line}\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(trace_path) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn copilot_rtk_trace(stage: &str, cmd_line: &str, rewritten: &str, changed: bool) {
    if let Ok(trace_path) = env::var("TOKLESS_COPILOT_RTK_TRACE") {
        if !trace_path.is_empty() {
            if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&trace_path) {
                let entry = serde_json::json!({
                    "stage": stage,
                    "command": cmd_line,
                    "rewritten": rewritten,
                    "changed": changed,
                });
                let _ = writeln!(f, "{}", entry);
            }
        }
    }
}

fn copilot_shell_tool(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "bash" | "powershell" | "shell" | "runterminalcommand"
    )
}

fn copilot_tool_name(req: &OrderedMap) -> String {
    for key in ["toolName", "tool_name"] {
        if let Some(s) = req.get(key).and_then(|v| v.as_str())
            && !s.is_empty()
        {
            return s.to_string();
        }
    }
    String::new()
}

fn parse_tool_args_map(val: Option<&Value>) -> Option<OrderedMap> {
    match val? {
        Value::String(s) if !s.is_empty() => try_parse_jsonc_map(s),
        Value::Object(m) => Some(m.clone()),
        _ => None,
    }
}

fn copilot_command(req: &OrderedMap) -> String {
    if let Some(args) = parse_tool_args_map(req.get("toolArgs"))
        && let Some(c) = args.get("command").and_then(|v| v.as_str())
        && !c.is_empty()
    {
        return c.to_string();
    }
    if let Some(ti) = req.get("tool_input").and_then(|v| v.as_object())
        && let Some(c) = ti.get("command").and_then(|v| v.as_str())
    {
        return c.to_string();
    }
    String::new()
}

pub fn copilot_rtk_post(raw: &str) -> i32 {
    if let Some(cfg) = try_parse_jsonc_map(raw) {
        let snake = cfg.contains_key("tool_result");
        return copilot_rtk_post_req(&cfg, snake);
    }
    0
}

fn copilot_rtk_post_req(req: &OrderedMap, snake: bool) -> i32 {
    let tool_name = if snake {
        req.get("tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        copilot_tool_name(req)
    };
    if !copilot_shell_tool(&tool_name) {
        return 0;
    }
    let cmd_line = if snake {
        req.get("tool_input")
            .and_then(|v| v.as_object())
            .and_then(|ti| ti.get("command"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        let mut c = parse_tool_args_map(req.get("toolArgs"))
            .and_then(|m| m.get("command").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_default();
        if c.is_empty() {
            c = copilot_command(req);
        }
        c
    };
    if cmd_line.is_empty() {
        return 0;
    }
    eprintln!("rtk: {cmd_line}");
    copilot_rtk_trace_cmd(&cmd_line);
    copilot_rtk_trace("postToolUse", &cmd_line, "", false);
    0
}

// --- Agent-specific hook entry points ---

pub fn run_rtk_hook() -> i32 {
    // Antigravity PreToolUse hook
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_call = match cfg.get_mut("toolCall") {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    if tool_call.get("name") != Some(&Value::String("run_command".into())) {
        return 0;
    }
    let args = match tool_call.get_mut("args") {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let cmd_line = match args.get("CommandLine") {
        Some(Value::String(s)) => s.clone(),
        _ => return 0,
    };

    let mut final_cmd = cmd_line.clone();
    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    if changed {
        final_cmd = new_cmd;
    }
    if final_cmd == cmd_line && !command_uses_rtk(&cmd_line) {
        return 0;
    }
    args.insert("CommandLine".into(), Value::String(final_cmd));

    let mut resp = OrderedMap::new();
    resp.insert("decision".into(), Value::String("allow".into()));
    resp.insert("allowTool".into(), Value::Bool(true));
    resp.insert("overwrite".into(), Value::Object(args.clone()));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_claude() -> i32 {
    run_rtk_hook_codex()
}

pub fn run_rtk_hook_omp() -> i32 {
    run_rtk_hook_codex()
}

pub fn run_rtk_hook_codex() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = match cfg.get("tool_name") {
        Some(Value::String(s)) => s.as_str(),
        _ => return 0,
    };
    if !is_bash_like_tool_name(tool_name) {
        return 0;
    }
    let tool_input = match cfg.get("tool_input") {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return 0,
    };

    let mut updated = tool_input.clone();
    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    if changed {
        updated.insert("command".into(), Value::String(new_cmd));
    }

    let mut hook_out = OrderedMap::new();
    hook_out.insert("hookEventName".into(), Value::String("PreToolUse".into()));
    hook_out.insert("permissionDecision".into(), Value::String("allow".into()));
    hook_out.insert("updatedInput".into(), Value::Object(updated));

    let mut resp = OrderedMap::new();
    resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_droid() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = match cfg.get("tool_name") {
        Some(Value::String(s)) => s.as_str(),
        _ => return 0,
    };
    if tool_name != "Execute" {
        return 0;
    }
    let tool_input = match cfg.get("tool_input") {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let (cmd_key, cmd_line) = if let Some(Value::String(c)) = tool_input.get("command")
        && !c.is_empty()
    {
        ("command", c.clone())
    } else if let Some(Value::String(c)) = tool_input.get("CommandLine")
        && !c.is_empty()
    {
        ("CommandLine", c.clone())
    } else {
        return 0;
    };

    let mut updated = tool_input.clone();
    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    if changed {
        updated.insert(cmd_key.into(), Value::String(new_cmd));
    }

    let mut hook_out = OrderedMap::new();
    hook_out.insert("hookEventName".into(), Value::String("PreToolUse".into()));
    hook_out.insert("permissionDecision".into(), Value::String("allow".into()));
    hook_out.insert("updatedInput".into(), Value::Object(updated));

    let mut resp = OrderedMap::new();
    resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_cursor() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = match cfg.get("tool_name") {
        Some(Value::String(s)) => s.as_str(),
        _ => return 0,
    };
    if tool_name != "Shell" {
        return 0;
    }
    let tool_input = match cfg.get("tool_input") {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return 0,
    };

    let mut updated = tool_input.clone();
    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    if changed {
        updated.insert("command".into(), Value::String(new_cmd));
    }

    let mut resp = OrderedMap::new();
    resp.insert("permission".into(), Value::String("allow".into()));
    resp.insert("updated_input".into(), Value::Object(updated));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_cline() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => {
            println!("{{}}");
            return 0;
        }
    };
    let (_tool_name, tool_input) = match cline_tool_input(&cfg) {
        Some(pair) if cline_shell_tool(&pair.0) => pair,
        _ => {
            println!("{{}}");
            return 0;
        }
    };
    if let Some(commands) = cline_rewrite_commands_input(&tool_input) {
        let mut updated = tool_input;
        updated.insert("commands".into(), commands);
        let mut resp = OrderedMap::new();
        resp.insert("overrideInput".into(), Value::Object(updated));
        println!("{}", stringify_json(&Value::Object(resp)));
        return 0;
    }
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            println!("{{}}");
            return 0;
        }
    };
    let (new_cmd, changed, approve) = copilot_rtk_decide(&cmd_line);
    if !approve || !changed || new_cmd.is_empty() || new_cmd == cmd_line {
        println!("{{}}");
        return 0;
    }

    let mut updated = tool_input;
    updated.insert("command".into(), Value::String(new_cmd));
    let mut resp = OrderedMap::new();
    resp.insert("overrideInput".into(), Value::Object(updated));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_grok() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = cfg
        .get("toolName")
        .or_else(|| cfg.get("tool_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let is_shell = tool_name.eq_ignore_ascii_case("run_terminal_cmd")
        || tool_name.eq_ignore_ascii_case("run_terminal_command")
        || tool_name.eq_ignore_ascii_case("bash");

    if !is_shell {
        return 0;
    }

    let tool_input = match cfg.get("toolInput").or_else(|| cfg.get("tool_input")) {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return 0,
    };

    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    if !changed || new_cmd.is_empty() || new_cmd == cmd_line {
        return 0;
    }

    let mut updated = tool_input.clone();
    updated.insert("command".into(), Value::String(new_cmd));

    let mut hook_out = OrderedMap::new();
    hook_out.insert("updatedInput".into(), Value::Object(updated));

    let mut resp = OrderedMap::new();
    resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_copilot() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => return 0,
    };

    if cfg.contains_key("toolResult") {
        return copilot_rtk_post_req(&cfg, false);
    }
    if cfg.contains_key("tool_result") {
        return copilot_rtk_post_req(&cfg, true);
    }

    // VS Code Chat / dual-fire PreToolUse (snake_case tool_name).
    if let Some(tool_name) = cfg.get("tool_name").and_then(|v| v.as_str()) {
        if tool_name.is_empty() || !copilot_shell_tool(tool_name) {
            return 0;
        }
        let tool_input = cfg
            .get("tool_input")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let cmd_line = match tool_input.get("command").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => return 0,
        };
        let (new_cmd, changed, approve) = copilot_rtk_decide(&cmd_line);
        copilot_rtk_trace("preToolUse", &cmd_line, &new_cmd, changed);
        if !approve {
            return 0;
        }
        let mut out_body = OrderedMap::new();
        out_body.insert("hookEventName".into(), Value::String("PreToolUse".into()));
        out_body.insert("permissionDecision".into(), Value::String("allow".into()));
        out_body.insert(
            "permissionDecisionReason".into(),
            Value::String("RTK auto-approved".into()),
        );
        let effective = if changed {
            copilot_rtk_trace_cmd(&new_cmd);
            new_cmd
        } else {
            if command_uses_rtk(&cmd_line) {
                copilot_rtk_trace_cmd(&cmd_line);
            }
            cmd_line
        };
        if changed {
            let mut updated = tool_input;
            updated.insert("command".into(), Value::String(effective));
            out_body.insert("updatedInput".into(), Value::Object(updated));
        }
        let mut resp = OrderedMap::new();
        resp.insert("hookSpecificOutput".into(), Value::Object(out_body));
        println!("{}", stringify_json(&Value::Object(resp)));
        return 0;
    }

    // Copilot CLI preToolUse (camelCase).
    let tool_name = copilot_tool_name(&cfg);
    if !copilot_shell_tool(&tool_name) {
        return 0;
    }
    let args = parse_tool_args_map(cfg.get("toolArgs")).unwrap_or_default();
    let mut cmd_line = args
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if cmd_line.is_empty() {
        cmd_line = copilot_command(&cfg);
    }
    if cmd_line.is_empty() {
        return 0;
    }
    let (new_cmd, changed, approve) = copilot_rtk_decide(&cmd_line);
    copilot_rtk_trace("preToolUse", &cmd_line, &new_cmd, changed);
    if !approve {
        return 0;
    }
    let mut resp = OrderedMap::new();
    resp.insert("permissionDecision".into(), Value::String("allow".into()));
    resp.insert(
        "permissionDecisionReason".into(),
        Value::String("RTK auto-approved".into()),
    );
    let effective = if changed {
        copilot_rtk_trace_cmd(&new_cmd);
        new_cmd
    } else {
        if command_uses_rtk(&cmd_line) {
            copilot_rtk_trace_cmd(&cmd_line);
        }
        cmd_line
    };
    if changed {
        let mut modified = args;
        modified.insert("command".into(), Value::String(effective));
        resp.insert("modifiedArgs".into(), Value::Object(modified));
    }
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

fn is_bash_like_tool_name(name: &str) -> bool {
    matches!(name.trim().to_lowercase().as_str(), "bash" | "shell")
}

fn command_has_raw_git(cmd_line: &str) -> bool {
    for seg in shell_command_segments(cmd_line) {
        if segment_starts_with_rtk(&seg) {
            continue;
        }
        if segment_binary(&seg) == "git" {
            return true;
        }
    }
    false
}

pub fn run_rtk_hook_dsh() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc_map(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = match cfg.get("tool_name").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return 0,
    };
    if !is_bash_like_tool_name(tool_name) {
        return 0;
    }
    let tool_input = match cfg.get("tool_input").and_then(|v| v.as_object()) {
        Some(m) => m,
        None => return 0,
    };
    let cmd_line = match tool_input.get("command").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return 0,
    };
    if cmd_line.is_empty() || !command_has_raw_git(cmd_line) {
        return 0;
    }

    let mut hook_out = OrderedMap::new();
    hook_out.insert("hookEventName".into(), Value::String("PreToolUse".into()));
    hook_out.insert("permissionDecision".into(), Value::String("deny".into()));
    hook_out.insert(
        "permissionDecisionReason".into(),
        Value::String("raw git is filtered by tokless: use `rtk git ...` instead".into()),
    );

    let mut resp = OrderedMap::new();
    resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

pub fn run_rtk_hook_cmd_kimi() -> i32 {
    run_rtk_hook_dsh()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_already_rtk_and_command_uses_rtk() {
        assert!(is_already_rtk("rtk git status"));
        assert!(!is_already_rtk("rtk git status && git diff"));
        assert!(command_uses_rtk("rtk git status && git diff"));
        assert!(command_uses_rtk("\"/usr/local/bin/rtk.exe\" git status"));
        assert!(!command_uses_rtk("git status"));
    }

    #[test]
    fn test_cline_tool_input_parsing() {
        let req1 = try_parse_jsonc_map(
            r#"{"tool_call":{"name":"execute_command","input":{"command":"git status"}}}"#,
        )
        .unwrap();
        let (name1, input1) = cline_tool_input(&req1).unwrap();
        assert_eq!(name1, "execute_command");
        assert!(cline_shell_tool(&name1));
        assert_eq!(
            input1.get("command").and_then(|v| v.as_str()),
            Some("git status")
        );

        let req2 = try_parse_jsonc_map(
            r#"{"preToolUse":{"toolName":"run_commands","parameters":{"commands":["rtk git status"]}}}"#,
        )
        .unwrap();
        let (name2, input2) = cline_tool_input(&req2).unwrap();
        assert_eq!(name2, "run_commands");
        assert!(cline_shell_tool(&name2));
        assert!(input2.contains_key("commands"));
    }
}
