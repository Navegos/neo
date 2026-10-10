use std::path::Path;
use std::fs;
use crate::util::paths::*;
use crate::util::jsonc::{try_parse_jsonc, stringify_json, OrderedMap, Value};

fn tokless_persisted_cmd(args: &[&str]) -> String {
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return String::new();
    }
    persisted_tokless_command(&exe, args)
}

fn parse_jsonc_object(raw: &str) -> OrderedMap {
    match try_parse_jsonc(raw) {
        Some(Value::Object(m)) => m,
        _ => OrderedMap::new(),
    }
}

// --- Claude ---

pub fn allow_claude_bash_pattern(pattern: &str) {
    let p = claude_code_paths();
    let raw = read_file_safe(&p.settings).unwrap_or_default();
    let mut cfg = parse_jsonc_object(&raw);

    let perms = match cfg.get_mut("permissions") {
        Some(Value::Object(m)) => m,
        _ => {
            cfg.insert("permissions".into(), Value::Object(OrderedMap::new()));
            match cfg.get_mut("permissions") {
                Some(Value::Object(m)) => m,
                _ => unreachable!(),
            }
        }
    };

    let mut allow_list = match perms.get("allow") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };

    for v in &allow_list {
        if let Value::String(s) = v {
            if s == pattern {
                return;
            }
        }
    }
    allow_list.push(Value::String(pattern.to_string()));
    perms.insert("allow".into(), Value::Array(allow_list));

    let _ = ensure_dir(&p.dir);
    let _ = write_file(&p.settings, &stringify_json(&Value::Object(cfg)));
}

pub fn disallow_claude_bash_pattern(pattern: &str) {
    let p = claude_code_paths();
    let raw = match read_file_safe(&p.settings) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let perms = match cfg.get_mut("permissions") {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let mut allow_list = match perms.get("allow") {
        Some(Value::Array(a)) => a.clone(),
        _ => return,
    };
    let before = allow_list.len();
    allow_list.retain(|v| {
        if let Value::String(s) = v {
            s != pattern
        } else {
            true
        }
    });
    if allow_list.len() == before {
        return;
    }
    if allow_list.is_empty() {
        perms.remove("allow");
    } else {
        perms.insert("allow".into(), Value::Array(allow_list));
    }
    let _ = write_file(&p.settings, &stringify_json(&Value::Object(cfg)));
}

// --- Droid ---

fn droid_hooks_path() -> String {
    Path::new(&home()).join(".factory").join("hooks.json").to_string_lossy().to_string()
}

pub fn install_droid_rtk_hook() -> bool {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "droid"]);
    if cmd.is_empty() {
        return false;
    }
    let p = droid_hooks_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let raw = read_file_safe(&p).unwrap_or_default();
    let mut cfg = parse_jsonc_object(&raw);

    let mut pre_tool_use = match cfg.get("PreToolUse") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };

    let mut found = false;
    for group in &mut pre_tool_use {
        if let Value::Object(gm) = group {
            if let Some(Value::String(m)) = gm.get("matcher") {
                if m == "Execute" {
                    let mut hooks = OrderedMap::new();
                    hooks.insert("type".into(), Value::String("command".into()));
                    hooks.insert("command".into(), Value::String(cmd.clone()));
                    gm.insert("hooks".into(), Value::Array(vec![Value::Object(hooks)]));
                    found = true;
                    break;
                }
            }
        }
    }
    if !found {
        let mut gm = OrderedMap::new();
        gm.insert("matcher".into(), Value::String("Execute".into()));
        let mut hm = OrderedMap::new();
        hm.insert("type".into(), Value::String("command".into()));
        hm.insert("command".into(), Value::String(cmd.clone()));
        gm.insert("hooks".into(), Value::Array(vec![Value::Object(hm)]));
        pre_tool_use.push(Value::Object(gm));
    }
    cfg.insert("PreToolUse".into(), Value::Array(pre_tool_use));
    write_file(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn has_droid_rtk_hook() -> bool {
    let p = droid_hooks_path();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return false,
    };
    raw.contains("rtk-hook") && raw.contains("droid")
}

pub fn remove_droid_rtk_hook() {
    let p = droid_hooks_path();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    if let Some(Value::Array(pre_tool_use)) = cfg.get_mut("PreToolUse") {
        pre_tool_use.retain(|group| {
            if let Value::Object(gm) = group {
                if let Some(Value::String(m)) = gm.get("matcher") {
                    return m != "Execute";
                }
            }
            true
        });
    }
    let _ = write_file(&p, &stringify_json(&Value::Object(cfg)));
}

// --- Antigravity ---

pub fn install_antigravity_rtk_hook() -> bool {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "agy"]);
    if cmd.is_empty() {
        return false;
    }
    let p = Path::new(&home()).join(".gemini").join("config").join("hooks.json").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let raw = read_file_safe(&p).unwrap_or_default();
    let mut cfg = parse_jsonc_object(&raw);

    let mut rtk_group = OrderedMap::new();
    let mut entry = OrderedMap::new();
    entry.insert("matcher".into(), Value::String("run_command".into()));
    let mut hook_cfg = OrderedMap::new();
    hook_cfg.insert("type".into(), Value::String("command".into()));
    hook_cfg.insert("command".into(), Value::String(cmd));
    entry.insert("hooks".into(), Value::Array(vec![Value::Object(hook_cfg)]));
    rtk_group.insert("PreToolUse".into(), Value::Array(vec![Value::Object(entry)]));

    cfg.insert("rtk".into(), Value::Object(rtk_group));
    allow_antigravity_entry("command(rtk)");
    allow_antigravity_entry("command(rtk )");
    write_file(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn has_antigravity_rtk_hook() -> bool {
    let p = Path::new(&home()).join(".gemini").join("config").join("hooks.json").to_string_lossy().to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return false,
    };
    raw.contains("rtk-hook") && raw.contains("agy")
}

pub fn remove_antigravity_rtk_hook() {
    let p = Path::new(&home()).join(".gemini").join("config").join("hooks.json").to_string_lossy().to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    cfg.remove("rtk");
    if cfg.is_empty() {
        let _ = fs::remove_file(&p);
    } else {
        let _ = write_file(&p, &stringify_json(&Value::Object(cfg)));
    }
}

pub fn allow_antigravity_entry(entry: &str) {
    let p = antigravity_paths_resolved().settings;
    let raw = read_file_safe(&p).unwrap_or_default();
    let mut cfg = parse_jsonc_object(&raw);

    let perms = match cfg.get_mut("permissions") {
        Some(Value::Object(m)) => m,
        _ => {
            cfg.insert("permissions".into(), Value::Object(OrderedMap::new()));
            match cfg.get_mut("permissions") {
                Some(Value::Object(m)) => m,
                _ => unreachable!(),
            }
        }
    };
    let mut allow_list = match perms.get("allow") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    for item in &allow_list {
        if let Value::String(s) = item {
            if s == entry {
                return;
            }
        }
    }
    allow_list.push(Value::String(entry.to_string()));
    perms.insert("allow".into(), Value::Array(allow_list));
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let _ = write_file(&p, &stringify_json(&Value::Object(cfg)));
}

pub fn remove_antigravity_entry(entry: &str) {
    let p = antigravity_paths_resolved().settings;
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let perms = match cfg.get_mut("permissions") {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    if let Some(Value::Array(allow_list)) = perms.get_mut("allow") {
        allow_list.retain(|v| {
            if let Value::String(s) = v {
                s != entry
            } else {
                true
            }
        });
    }
    let _ = write_file(&p, &stringify_json(&Value::Object(cfg)));
}

// --- Codex ---

pub fn install_codex_rtk_hook() -> bool {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "codex"]);
    if cmd.is_empty() {
        return false;
    }
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let raw = read_file_safe(&p).unwrap_or_default();
    let mut cfg = parse_jsonc_object(&raw);

    let hooks = match cfg.get_mut("hooks") {
        Some(Value::Object(m)) => m,
        _ => {
            cfg.insert("hooks".into(), Value::Object(OrderedMap::new()));
            match cfg.get_mut("hooks") {
                Some(Value::Object(m)) => m,
                _ => unreachable!(),
            }
        }
    };

    let mut pre_arr = match hooks.get("PreToolUse") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };

    let mut gm = OrderedMap::new();
    gm.insert("matcher".into(), Value::String("Bash".into()));
    let mut hm = OrderedMap::new();
    hm.insert("type".into(), Value::String("command".into()));
    hm.insert("command".into(), Value::String(cmd));
    hm.insert("timeout".into(), Value::from(10));
    gm.insert("hooks".into(), Value::Array(vec![Value::Object(hm)]));
    pre_arr.push(Value::Object(gm));

    hooks.insert("PreToolUse".into(), Value::Array(pre_arr));
    write_file(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn has_codex_rtk_hook() -> bool {
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return false,
    };
    raw.contains("rtk-hook") && raw.contains("codex")
}

pub fn remove_codex_rtk_hook() {
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    if let Some(Value::Object(hooks)) = cfg.get_mut("hooks") {
        if let Some(Value::Array(pre_arr)) = hooks.get_mut("PreToolUse") {
            pre_arr.retain(|g| {
                if let Value::Object(gm) = g {
                    if let Some(Value::String(m)) = gm.get("matcher") {
                        return m != "Bash";
                    }
                }
                true
            });
        }
    }
    let _ = write_file(&p, &stringify_json(&Value::Object(cfg)));
}

// --- Cursor ---

pub fn install_cursor_rtk_hook() -> bool {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "cursor"]);
    if cmd.is_empty() {
        return false;
    }
    let p = Path::new(&home()).join(".cursor").join("hooks.json").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let raw = read_file_safe(&p).unwrap_or_default();
    let mut cfg = parse_jsonc_object(&raw);

    let mut hm = OrderedMap::new();
    hm.insert("command".into(), Value::String(cmd));
    cfg.insert("preToolUse".into(), Value::Object(hm));
    write_file(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn has_cursor_rtk_hook() -> bool {
    let p = Path::new(&home()).join(".cursor").join("hooks.json").to_string_lossy().to_string();
    let raw = match read_file_safe(&p) {
        Some(r) => r,
        None => return false,
    };
    raw.contains("rtk-hook") && raw.contains("cursor")
}

pub fn remove_cursor_rtk_hook() -> bool {
    let p = Path::new(&home()).join(".cursor").join("hooks.json").to_string_lossy().to_string();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    true
}

pub fn configure_cursor_rtk_permissions() -> bool {
    true
}

pub fn has_cursor_rtk_permissions() -> bool {
    true
}

pub fn remove_cursor_rtk_permissions() -> bool {
    true
}

pub struct CursorHookSnapshot {
    pub path: String,
    pub content: Option<String>,
}

impl CursorHookSnapshot {
    pub fn restore(&self) -> Result<(), std::io::Error> {
        if let Some(ref c) = self.content {
            fs::write(&self.path, c)?;
        } else if Path::new(&self.path).exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

pub fn snapshot_cursor_project_rules_hook() -> Result<CursorHookSnapshot, String> {
    let p = Path::new(&home()).join(".cursor").join("hooks.json").to_string_lossy().to_string();
    let content = read_file_safe(&p);
    Ok(CursorHookSnapshot { path: p, content })
}

pub fn install_cursor_project_rules_hook() -> bool {
    true
}

// --- Copilot ---

pub fn install_copilot_rtk_hook() -> bool {
    install_copilot_rtk_hook_safe().is_ok()
}

pub fn install_copilot_rtk_hook_safe() -> Result<(), String> {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "copilot"]);
    if cmd.is_empty() {
        return Err("no tokless command".into());
    }
    let p = Path::new(&copilot_paths_resolved().dir).join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let mut obj = OrderedMap::new();
    obj.insert("command".into(), Value::String(cmd));
    write_file(&p, &stringify_json(&Value::Object(obj))).map_err(|e| e.to_string())
}

pub fn install_copilot_ide_rtk_hook_safe() -> Result<(), String> {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "copilot"]);
    if cmd.is_empty() {
        return Err("no tokless command".into());
    }
    let p = Path::new(&ide_project_root()).join(".github").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let mut obj = OrderedMap::new();
    obj.insert("command".into(), Value::String(cmd));
    write_file(&p, &stringify_json(&Value::Object(obj))).map_err(|e| e.to_string())
}

pub fn has_copilot_rtk_hook() -> bool {
    let p = Path::new(&copilot_paths_resolved().dir).join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    exists(&p)
}

pub fn has_copilot_ide_rtk_hook() -> bool {
    let p = Path::new(&ide_project_root()).join(".github").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    exists(&p)
}

pub fn remove_copilot_rtk_hook_safe() -> Result<(), String> {
    let p = Path::new(&copilot_paths_resolved().dir).join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    Ok(())
}

pub fn remove_copilot_ide_rtk_hook_safe() -> Result<(), String> {
    let p = Path::new(&ide_project_root()).join(".github").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    Ok(())
}

pub fn copilot_transaction_file_owned(path: &str) -> bool {
    path.contains("tokless")
}

pub fn copilot_shim_digest_path() -> String {
    Path::new(&copilot_paths_resolved().dir).join("tokless-digest.sha").to_string_lossy().to_string()
}

pub fn ide_project_root() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into())
}

pub fn write_copilot_project_file(path: &str, content: &str) -> Result<(), String> {
    write_file(path, content).map_err(|e| e.to_string())
}

pub fn clear_copilot_project_file(path: &str) -> Result<(), String> {
    if exists(path) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

pub fn copilot_proxy_port() -> u16 {
    18082
}

pub fn copilot_cli_proxy_port() -> u16 {
    18083
}

pub fn copilot_cli_proxy_applicable() -> bool {
    !which("copilot").is_empty()
}

pub fn copilot_vscode_proxy_applicable() -> bool {
    exists(&copilot_paths_resolved().dir)
}

// --- Grok ---

pub fn grok_instructions_file() -> String {
    Path::new(&home()).join(".grok").join("AGENTS.md").to_string_lossy().to_string()
}

pub fn install_grok_rtk_hook() -> Result<(), String> {
    let cmd = tokless_persisted_cmd(&["rtk-hook", "grok"]);
    if cmd.is_empty() {
        return Err("no tokless command".into());
    }
    let p = Path::new(&home()).join(".grok").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let mut obj = OrderedMap::new();
    obj.insert("command".into(), Value::String(cmd));
    write_file(&p, &stringify_json(&Value::Object(obj))).map_err(|e| e.to_string())
}

pub fn has_grok_rtk_hook() -> bool {
    let p = Path::new(&home()).join(".grok").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    exists(&p)
}

pub fn remove_grok_rtk_hook() {
    let p = Path::new(&home()).join(".grok").join("hooks").join("tokless-rtk.json").to_string_lossy().to_string();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
}

pub fn install_grok_codegraph_session_hook() -> Result<(), String> {
    Ok(())
}

pub fn has_grok_codegraph_session_hook() -> bool {
    true
}

pub fn remove_grok_codegraph_session_hook() {}

// --- Pi / OMP / Kilo / Cline ---

pub fn pi_agent_dir_resolved() -> String {
    Path::new(&home()).join(".pi").to_string_lossy().to_string()
}

pub fn has_pi_rtk_extension() -> bool {
    let p = Path::new(&pi_agent_dir_resolved()).join("extensions").join("rtk.ts").to_string_lossy().to_string();
    exists(&p)
}

pub fn pi_package_list() -> Vec<String> {
    vec!["@colbymchenry/codegraph".into(), "context-mode".into()]
}

pub fn pi_source_has(_src: &str) -> bool {
    false
}

pub fn pi_remove_source(_src: &str) -> bool {
    true
}

pub fn pi_update_packages() {}

pub fn omp_agent_dir_resolved() -> String {
    Path::new(&home()).join(".omp").to_string_lossy().to_string()
}

pub fn has_omp_rtk_extension() -> bool {
    let p = Path::new(&omp_agent_dir_resolved()).join("extensions").join("tokless-rtk.ts").to_string_lossy().to_string();
    exists(&p)
}

pub fn kilo_project_file(sub: &str, name: &str) -> String {
    Path::new(&kilo_paths_resolved().dir).join(sub).join(name).to_string_lossy().to_string()
}

pub fn cline_instructions_path() -> String {
    cline_paths_resolved().instructions
}

pub const PROJECTMEM_MCP_TOOL_NAMES: &[&str] = &[
    "log_issue", "record_attempt", "record_fix", "add_decision", "add_note",
];

