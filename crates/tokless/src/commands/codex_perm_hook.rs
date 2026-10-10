use std::io::{self, Read};
use crate::util::jsonc::{try_parse_jsonc, stringify_json, OrderedMap, Value};
use crate::util::paths::is_win;

const CODEX_PERM_ALLOWLIST: &[&str] = &[
    "rtk", "tokless", "git", "cd", "ls",
    "node", "npm", "npx",
    "context-mode", "codegraph",
    "cat", "head", "tail",
    "grep", "find", "pwd",
    "which", "echo", "true", "false",
    "bash",
];

pub fn run_codex_perm_hook() -> i32 {
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
    let command = cfg.get("tool_input").and_then(|v| {
        if let Value::Object(m) = v {
            m.get("command").and_then(|c| {
                if let Value::String(s) = c { Some(s.as_str()) } else { None }
            })
        } else {
            None
        }
    }).unwrap_or("");

    if !codex_perm_allow(tool_name, command) {
        return 0;
    }

    let mut decision = OrderedMap::new();
    decision.insert("behavior".into(), Value::String("allow".into()));

    let mut hook_out = OrderedMap::new();
    hook_out.insert("hookEventName".into(), Value::String("PermissionRequest".into()));
    hook_out.insert("decision".into(), Value::Object(decision));

    let mut resp = OrderedMap::new();
    resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
    println!("{}", stringify_json(&Value::Object(resp)));
    0
}

fn codex_perm_allow(tool_name: &str, command: &str) -> bool {
    if tool_name == "apply_patch" {
        return true;
    }
    if tool_name.starts_with("ctx_") || tool_name.starts_with("codegraph_") {
        return true;
    }
    if tool_name != "Bash" {
        return false;
    }
    let cmd = command.trim();
    if cmd.is_empty() {
        return false;
    }
    let tok = strip_path(first_token(cmd));
    if tok == "bash" || tok == "sh" {
        return bash_inner_script_all_allowed(cmd);
    }
    CODEX_PERM_ALLOWLIST.contains(&tok.as_str())
}

fn first_token(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

fn strip_path(tok: &str) -> String {
    let mut clean = tok;
    if let Some(idx) = clean.rfind('/') {
        clean = &clean[idx + 1..];
    }
    if is_win() {
        if let Some(idx) = clean.rfind('\\') {
            clean = &clean[idx + 1..];
        }
        clean = clean.strip_suffix(".exe").unwrap_or(clean);
        clean = clean.strip_suffix(".cmd").unwrap_or(clean);
        clean = clean.strip_suffix(".bat").unwrap_or(clean);
    }
    clean.to_string()
}

fn bash_inner_script_all_allowed(cmd: &str) -> bool {
    let mut rest = cmd;
    for flag in &["-lc", "-c"] {
        if let Some(idx) = rest.find(flag) {
            rest = rest[idx + flag.len()..].trim();
            break;
        }
    }
    if rest.is_empty() {
        return false;
    }
    if (rest.starts_with('"') && rest.ends_with('"'))
        || (rest.starts_with('\'') && rest.ends_with('\'')) {
        rest = &rest[1..rest.len() - 1];
    }
    let segments = split_script(rest);
    for seg in segments {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            continue;
        }
        let tok = strip_path(first_token(trimmed));
        if !CODEX_PERM_ALLOWLIST.contains(&tok.as_str()) {
            return false;
        }
    }
    true
}

fn split_script(s: &str) -> Vec<String> {
    let mut replaced = s.replace("||", "\0");
    replaced = replaced.replace("&&", "\0");
    replaced = replaced.replace('|', "\0");
    replaced = replaced.replace(';', "\0");
    replaced.split('\0').map(|part| part.to_string()).collect()
}
