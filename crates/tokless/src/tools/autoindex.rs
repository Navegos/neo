use std::fs;
use std::path::Path;
use crate::util::jsonc::{stringify_json, try_parse_jsonc, Value};
use crate::util::paths::{
    claude_code_paths, codex_paths_resolved, home, opencode_paths_resolved, read_file_safe,
    write_file_atomic,
};

/// Removes legacy SessionStart auto-index hooks from earlier Tokless versions.
pub fn unwire_auto_index(agent: &str) {
    match agent {
        "claude" => unwire_claude_auto_index(),
        "codex" => unwire_codex_auto_index(),
        "opencode" => unwire_opencode_auto_index(),
        "antigravity" => unwire_gemini_auto_index(),
        _ => {}
    }
}

pub fn unwire_claude_auto_index() {
    let p = claude_code_paths();
    let raw = match read_file_safe(&p.settings) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let hooks = match cfg.get_mut("hooks") {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let groups = match hooks.get("SessionStart") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return,
    };
    if !groups_contain_auto_index(&groups) {
        return;
    }
    let filtered = remove_auto_index_groups(groups);
    if filtered.is_empty() {
        hooks.remove("SessionStart");
    } else {
        hooks.insert("SessionStart".into(), Value::Array(filtered));
    }
    let _ = write_file_atomic(&p.settings, &stringify_json(&Value::Object(cfg)));
}

pub fn unwire_codex_auto_index() {
    let p = Path::new(&codex_paths_resolved().dir)
        .join("hooks.json")
        .to_string_lossy()
        .to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let hooks = match cfg.get_mut("hooks") {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let groups = match hooks.get("SessionStart") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return,
    };
    if !groups_contain_auto_index(&groups) {
        return;
    }
    let filtered = remove_auto_index_groups(groups);
    if filtered.is_empty() {
        hooks.remove("SessionStart");
    } else {
        hooks.insert("SessionStart".into(), Value::Array(filtered));
    }
    let _ = write_file_atomic(&p, &stringify_json(&Value::Object(cfg)));
}

pub fn unwire_opencode_auto_index() {
    let p = Path::new(&opencode_paths_resolved().plugins_dir).join("tokless-index.ts");
    if p.exists() {
        let _ = fs::remove_file(p);
    }
}

pub fn unwire_gemini_auto_index() {
    let p = Path::new(&home())
        .join(".gemini")
        .join("settings.json")
        .to_string_lossy()
        .to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let hooks = match cfg.get_mut("hooks") {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let groups = match hooks.get("SessionStart") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return,
    };
    if !groups_contain_auto_index(&groups) {
        return;
    }
    let filtered = remove_auto_index_groups(groups);
    if filtered.is_empty() {
        hooks.remove("SessionStart");
    } else {
        hooks.insert("SessionStart".into(), Value::Array(filtered));
    }
    let _ = write_file_atomic(&p, &stringify_json(&Value::Object(cfg)));
}

fn is_auto_index_cmd(cmd: &str) -> bool {
    cmd.contains("tokless") && cmd.contains("index") && cmd.contains("--auto")
}

pub fn groups_contain_auto_index(groups: &[Value]) -> bool {
    for g in groups {
        if let Value::Object(gm) = g {
            if let Some(Value::Array(hooks)) = gm.get("hooks") {
                for h in hooks {
                    if let Value::Object(hm) = h {
                        if let Some(Value::String(cmd)) = hm.get("command") {
                            if is_auto_index_cmd(cmd) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn remove_auto_index_groups(groups: Vec<Value>) -> Vec<Value> {
    let mut out = Vec::new();
    for g in groups {
        let mut gm = match g {
            Value::Object(m) => m,
            other => {
                out.push(other);
                continue;
            }
        };
        let hooks = match gm.get("hooks") {
            Some(Value::Array(arr)) => arr.clone(),
            _ => {
                out.push(Value::Object(gm));
                continue;
            }
        };
        let mut kept = Vec::new();
        for h in hooks {
            let drop_it = if let Value::Object(ref hm) = h {
                if let Some(Value::String(cmd)) = hm.get("command") {
                    is_auto_index_cmd(cmd)
                } else {
                    false
                }
            } else {
                false
            };
            if !drop_it {
                kept.push(h);
            }
        }
        if !kept.is_empty() {
            gm.insert("hooks".into(), Value::Array(kept));
            out.push(Value::Object(gm));
        }
    }
    out
}
