use std::env;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;
use crate::util::jsonc::{try_parse_jsonc, stringify_json, OrderedMap, Value};
use crate::util::paths::resolve_rtk_bin;

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

pub fn rtk_rewrite(cmd_line: &str) -> (String, bool) {
    if cmd_line.is_empty() || rtk_unsafe_find(cmd_line) || has_shell_pipe(cmd_line) {
        return (String::new(), false);
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

fn rtk_unsafe_find(cmd_line: &str) -> bool {
    let unsupported = [
        "-not", "!", "-or", "-o", "-and", "-a",
        "-exec", "-execdir", "-delete", "-print0",
        "-newer", "-perm", "-size", "-mtime", "-mmin",
        "-atime", "-amin", "-ctime", "-cmin", "-empty",
        "-link", "-regex", "-iregex",
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

pub fn command_uses_rtk(cmd_line: &str) -> bool {
    for seg in shell_command_segments(cmd_line) {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            continue;
        }
        let first = trimmed.split_whitespace().next().unwrap_or("");
        let unquoted = first.trim_matches(|c| c == '"' || c == '\'');
        let bin = Path::new(unquoted).file_name().and_then(|f| f.to_str()).unwrap_or("");
        let lower = bin.to_lowercase();
        if lower == "rtk" || lower == "rtk.exe" {
            return true;
        }
    }
    false
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

pub fn run_rtk_hook_codex() -> i32 {
    let mut input = Vec::new();
    if io::stdin().read_to_end(&mut input).is_err() || input.is_empty() {
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = match cfg.get("tool_name") {
        Some(Value::String(s)) => s.as_str(),
        _ => return 0,
    };
    if tool_name != "Bash" && tool_name != "bash" {
        return 0;
    }
    let tool_input = match cfg.get("tool_input") {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) => s.clone(),
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
    let cfg = match try_parse_jsonc(&raw) {
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
    let (cmd_key, cmd_line) = if let Some(Value::String(c)) = tool_input.get("command") {
        ("command", c.clone())
    } else if let Some(Value::String(c)) = tool_input.get("CommandLine") {
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
    let cfg = match try_parse_jsonc(&raw) {
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
        Some(Value::String(s)) => s.clone(),
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
        println!("{{}}");
        return 0;
    }
    let raw = String::from_utf8_lossy(&input);
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => {
            println!("{{}}");
            return 0;
        }
    };
    let tool_input = match cfg.get("tool_input").or_else(|| cfg.get("input")) {
        Some(Value::Object(m)) => m,
        _ => {
            println!("{{}}");
            return 0;
        }
    };
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) => s.clone(),
        _ => {
            println!("{{}}");
            return 0;
        }
    };
    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    if !changed || new_cmd.is_empty() || new_cmd == cmd_line {
        println!("{{}}");
        return 0;
    }

    let mut updated = tool_input.clone();
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
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return 0,
    };
    let tool_name = cfg.get("toolName").or_else(|| cfg.get("tool_name")).and_then(|v| {
        if let Value::String(s) = v { Some(s.as_str()) } else { None }
    }).unwrap_or("");

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
        Some(Value::String(s)) => s.clone(),
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
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return 0,
    };

    let tool_name = cfg.get("tool_name").or_else(|| cfg.get("toolName")).and_then(|v| {
        if let Value::String(s) = v { Some(s.as_str()) } else { None }
    }).unwrap_or("");

    let lower = tool_name.to_lowercase();
    let is_shell = lower == "bash" || lower == "shell" || lower == "powershell" || lower == "runterminalcommand";
    if !is_shell {
        return 0;
    }

    let tool_input = match cfg.get("tool_input").or_else(|| cfg.get("toolInput")).or_else(|| cfg.get("toolArgs")) {
        Some(Value::Object(m)) => m,
        _ => return 0,
    };
    let cmd_line = match tool_input.get("command") {
        Some(Value::String(s)) => s.clone(),
        _ => return 0,
    };

    let (new_cmd, changed) = rtk_rewrite(&cmd_line);
    let mut out_body = OrderedMap::new();
    out_body.insert("hookEventName".into(), Value::String("PreToolUse".into()));
    out_body.insert("permissionDecision".into(), Value::String("allow".into()));
    out_body.insert("permissionDecisionReason".into(), Value::String("RTK auto-approved".into()));

    if changed {
        let mut updated = tool_input.clone();
        updated.insert("command".into(), Value::String(new_cmd));
        out_body.insert("updatedInput".into(), Value::Object(updated));
    }

    let mut resp = OrderedMap::new();
    resp.insert("hookSpecificOutput".into(), Value::Object(out_body));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}
