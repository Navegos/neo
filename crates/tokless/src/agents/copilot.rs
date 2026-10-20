// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/copilot.rs
// created: 2026-10-11
// lastModified: 2026-10-20

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::core::AgentManifest;
use crate::util::jsonc::{OrderedMap, Value, stringify_json};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;

use super::proxy_detection::{ProxyCapability, ProxyDetection, ProxyState, proxy_detection};
use super::proxy_provider::{
    ERR_NO_STABLE_TOKLESS, detect_vscode_agent_with, is_vscode_extension_installed,
    json_map_child, json_map_child_mut, sha256_hex, strings_to_values, tokless_command,
    tokless_command_ok, tokless_managed_command, try_parse_jsonc_map as try_parse_jsonc,
};

pub const DEFAULT_COPILOT_PROXY_PORT: u16 = 8789;
pub const DEFAULT_COPILOT_CLI_PROXY_PORT: u16 = 8790;

pub const COPILOT_VSCODE_MARKER_HEAD: &str = "// --- tokless:headroom copilot begin";
pub const COPILOT_VSCODE_MARKER_FOOT: &str = "// --- tokless:headroom copilot end";
pub const COPILOT_VSCODE_PROXY_KEY: &str = "github.copilot.advanced.debug.overrideProxyUrl";
pub const COPILOT_VSCODE_AUTH_KEY: &str = "github.copilot.advanced.debug.overrideAuthType";
pub const COPILOT_VSCODE_CAPI_KEY: &str = "github.copilot.advanced.debug.overrideCapiUrl";

static IDE_PROJECT_ROOT: OnceLock<Mutex<String>> = OnceLock::new();
static COPILOT_HOOK_WRITE_MU: OnceLock<Mutex<()>> = OnceLock::new();
static COPILOT_PROJECT_WRITE_MU: OnceLock<Mutex<()>> = OnceLock::new();
static COPILOT_PROXY_OPERATION_MU: OnceLock<Mutex<()>> = OnceLock::new();

fn ide_root_state() -> &'static Mutex<String> {
    IDE_PROJECT_ROOT.get_or_init(|| Mutex::new(String::new()))
}

fn hook_write_mu() -> &'static Mutex<()> {
    COPILOT_HOOK_WRITE_MU.get_or_init(|| Mutex::new(()))
}

fn project_write_mu() -> &'static Mutex<()> {
    COPILOT_PROJECT_WRITE_MU.get_or_init(|| Mutex::new(()))
}

fn proxy_op_mu() -> &'static Mutex<()> {
    COPILOT_PROXY_OPERATION_MU.get_or_init(|| Mutex::new(()))
}

pub fn copilot_manifest() -> AgentManifest {
    AgentManifest {
        id: "copilot".to_string(),
        label: "GitHub Copilot".to_string(),
        homepage: "https://github.com/github/copilot-cli".to_string(),
        cli_bin: "copilot".to_string(),
        config_dir: Some(Arc::new(|| paths::copilot_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_vscode_agent_with(
                "copilot",
                &paths::copilot_paths_resolved().dir,
                &copilot_known_bin_dirs(),
                "github.copilot-chat",
            )
        })),
    }
}

pub fn copilot_known_bin_dirs() -> Vec<String> {
    let mut dirs = vec![
        Path::new(&paths::home())
            .join(".local")
            .join("bin")
            .to_string_lossy()
            .into_owned(),
        "/usr/local/bin".to_string(),
    ];
    if paths::is_win()
        && let Ok(la) = std::env::var("LOCALAPPDATA")
        && !la.is_empty()
    {
        dirs.push(
            Path::new(&la)
                .join("Microsoft")
                .join("WinGet")
                .join("Links")
                .to_string_lossy()
                .into_owned(),
        );
    }
    dirs
}

fn copilot_hooks_file(name: &str) -> String {
    Path::new(&paths::copilot_paths_resolved().hooks_dir)
        .join(name)
        .to_string_lossy()
        .into_owned()
}

pub fn set_ide_project_root(p: &str) {
    if let Ok(mut guard) = ide_root_state().lock() {
        *guard = p.to_string();
    }
}

pub fn ide_project_root() -> String {
    ide_root()
}

pub fn ide_in_project() -> bool {
    ide_root() != "."
}

fn ide_root() -> String {
    if let Ok(guard) = ide_root_state().lock()
        && !guard.is_empty()
    {
        return guard.clone();
    }
    let Ok(mut dir) = std::env::current_dir() else {
        return ".".to_string();
    };
    loop {
        for marker in [".vscode", ".github", ".git"] {
            if paths::exists(dir.join(marker)) {
                return dir.to_string_lossy().into_owned();
            }
        }
        let Some(parent) = dir.parent().map(|p| p.to_path_buf()) else {
            return ".".to_string();
        };
        if parent == dir {
            return ".".to_string();
        }
        dir = parent;
    }
}

fn copilot_ide_hooks_dir() -> String {
    Path::new(&ide_root())
        .join(".github")
        .join("hooks")
        .to_string_lossy()
        .into_owned()
}

fn copilot_ide_hooks_file(name: &str) -> String {
    Path::new(&copilot_ide_hooks_dir())
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn copilot_ide_mcp_file() -> String {
    Path::new(&ide_root())
        .join(".vscode")
        .join("mcp.json")
        .to_string_lossy()
        .into_owned()
}

fn copilot_ide_instructions_file() -> String {
    Path::new(&ide_root())
        .join(".github")
        .join("copilot-instructions.md")
        .to_string_lossy()
        .into_owned()
}

pub fn install_copilot_context_mode_hook() {
    let _ = install_copilot_context_mode_hook_safe();
}

pub fn install_copilot_context_mode_hook_safe() -> Result<(), String> {
    let p = paths::copilot_paths_resolved();
    paths::ensure_dir(&p.hooks_dir).map_err(|e| e.to_string())?;

    let events = [
        ("preToolUse", "pretooluse"),
        ("postToolUse", "posttooluse"),
        ("sessionStart", "sessionstart"),
        ("userPromptSubmitted", "userpromptsubmit"),
        ("agentStop", "stop"),
        ("preCompact", "precompact"),
    ];
    let mut hooks = OrderedMap::new();
    for (event, token) in events {
        let mut h = OrderedMap::new();
        h.insert("type".to_string(), Value::String("command".to_string()));
        h.insert(
            "command".to_string(),
            Value::String(format!("context-mode hook copilot-cli {token}")),
        );
        hooks.insert(event.to_string(), Value::Array(vec![Value::Object(h)]));
    }

    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_hooks_file("context-mode.json"),
        &stringify_json(&Value::Object(root)),
        "context-mode hook copilot-cli",
    )
}

pub fn remove_copilot_context_mode_hook() {
    let _ = remove_copilot_context_mode_hook_safe();
}

pub fn remove_copilot_context_mode_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(
        &copilot_hooks_file("context-mode.json"),
        "context-mode hook copilot-cli",
    )
}

pub fn has_copilot_context_mode_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_hooks_file("context-mode.json")) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "context-mode hook copilot-cli")
}

pub fn install_copilot_ide_context_mode_hook() {
    let _ = install_copilot_ide_context_mode_hook_safe();
}

pub fn install_copilot_ide_context_mode_hook_safe() -> Result<(), String> {
    paths::ensure_dir(&copilot_ide_hooks_dir()).map_err(|e| e.to_string())?;
    let events = ["PreToolUse", "PostToolUse", "SessionStart", "Stop"];
    let tokens = ["pretooluse", "posttooluse", "sessionstart", "stop"];
    let mut hooks = OrderedMap::new();
    for (ev, tok) in events.iter().zip(tokens.iter()) {
        let mut h = OrderedMap::new();
        h.insert("type".to_string(), Value::String("command".to_string()));
        h.insert(
            "command".to_string(),
            Value::String(format!("context-mode hook copilot-vscode {tok}")),
        );
        h.insert("timeout".to_string(), Value::from(10));
        hooks.insert((*ev).to_string(), Value::Array(vec![Value::Object(h)]));
    }
    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_ide_hooks_file("context-mode.json"),
        &stringify_json(&Value::Object(root)),
        "context-mode hook copilot-vscode",
    )
}

pub fn remove_copilot_ide_context_mode_hook() {
    let _ = remove_copilot_ide_context_mode_hook_safe();
}

pub fn remove_copilot_ide_context_mode_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(
        &copilot_ide_hooks_file("context-mode.json"),
        "context-mode hook copilot-vscode",
    )
}

pub fn has_copilot_ide_context_mode_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_ide_hooks_file("context-mode.json")) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "context-mode hook copilot-vscode")
}

fn copilot_rtk_hook_command() -> String {
    tokless_command_ok(&["rtk-hook", "copilot"]).0
}

pub fn install_copilot_rtk_hook() -> bool {
    install_copilot_rtk_hook_safe().is_ok()
}

pub fn install_copilot_rtk_hook_safe() -> Result<(), String> {
    let p = paths::copilot_paths_resolved();
    paths::ensure_dir(&p.hooks_dir).map_err(|e| e.to_string())?;

    let cmd = copilot_rtk_hook_command();
    if cmd.is_empty() {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }

    let make_flat = || {
        let mut h = OrderedMap::new();
        h.insert("type".to_string(), Value::String("command".to_string()));
        h.insert("command".to_string(), Value::String(cmd.clone()));
        Value::Object(h)
    };

    let mut hooks = OrderedMap::new();
    hooks.insert("PreToolUse".to_string(), Value::Array(vec![make_flat()]));
    hooks.insert("preToolUse".to_string(), Value::Array(vec![make_flat()]));
    hooks.insert("PostToolUse".to_string(), Value::Array(vec![make_flat()]));
    hooks.insert("postToolUse".to_string(), Value::Array(vec![make_flat()]));

    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_hooks_file("tokless-rtk.json"),
        &stringify_json(&Value::Object(root)),
        "rtk-hook copilot",
    )?;
    ensure_copilot_rtk_command_approval_safe()
}

pub fn remove_copilot_rtk_hook() {
    let _ = remove_copilot_rtk_hook_safe();
}

pub fn remove_copilot_rtk_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(&copilot_hooks_file("tokless-rtk.json"), "rtk-hook copilot")
}

pub fn install_copilot_ide_rtk_hook() {
    let _ = install_copilot_ide_rtk_hook_safe();
}

pub fn install_copilot_ide_rtk_hook_safe() -> Result<(), String> {
    paths::ensure_dir(&copilot_ide_hooks_dir()).map_err(|e| e.to_string())?;
    let cmd = copilot_rtk_hook_command();
    let mut entry = OrderedMap::new();
    entry.insert("type".to_string(), Value::String("command".to_string()));
    entry.insert("command".to_string(), Value::String(cmd));

    let mut hooks = OrderedMap::new();
    hooks.insert(
        "PreToolUse".to_string(),
        Value::Array(vec![Value::Object(entry.clone())]),
    );
    hooks.insert(
        "PostToolUse".to_string(),
        Value::Array(vec![Value::Object(entry)]),
    );
    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_ide_hooks_file("tokless-rtk.json"),
        &stringify_json(&Value::Object(root)),
        "rtk-hook copilot",
    )
}

pub fn remove_copilot_ide_rtk_hook() {
    let _ = remove_copilot_ide_rtk_hook_safe();
}

pub fn remove_copilot_ide_rtk_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(
        &copilot_ide_hooks_file("tokless-rtk.json"),
        "rtk-hook copilot",
    )
}

pub fn has_copilot_ide_rtk_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_ide_hooks_file("tokless-rtk.json")) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "rtk-hook copilot")
}

fn copilot_permissions_file() -> String {
    Path::new(&paths::copilot_paths_resolved().dir)
        .join("permissions-config.json")
        .to_string_lossy()
        .into_owned()
}

pub fn ensure_copilot_rtk_command_approval() {
    let _ = ensure_copilot_rtk_command_approval_safe();
}

pub fn ensure_copilot_rtk_command_approval_safe() -> Result<(), String> {
    let path = copilot_permissions_file();
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.to_string()),
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => {
            if !raw.trim().is_empty() {
                return Err(format!("invalid Copilot permissions config {path}"));
            }
            OrderedMap::new()
        }
    };
    let Some(locs) = json_map_child_mut(&mut cfg, "locations") else {
        if cfg.contains_key("locations") {
            return Err(format!(
                "Copilot permissions config {path} has non-object locations"
            ));
        }
        return Ok(());
    };
    let mut changed = false;
    let keys: Vec<String> = locs.keys().cloned().collect();
    for key in keys {
        let Some(loc) = json_map_child_mut(locs, &key) else {
            continue;
        };
        let mut approvals = if let Some(v) = loc.get("tool_approvals") {
            let Some(arr) = v.as_array() else {
                return Err(format!(
                    "Copilot permissions config {path} has non-array tool_approvals for {key}"
                ));
            };
            arr.clone()
        } else {
            Vec::new()
        };
        if copilot_approvals_has_rtk(&approvals) {
            continue;
        }
        let mut entry = OrderedMap::new();
        entry.insert("kind".to_string(), Value::String("commands".to_string()));
        entry.insert(
            "commandIdentifiers".to_string(),
            Value::Array(vec![Value::String("rtk".to_string())]),
        );
        approvals.push(Value::Object(entry));
        loc.insert("tool_approvals".to_string(), Value::Array(approvals));
        changed = true;
    }
    if !changed {
        return Ok(());
    }
    write_copilot_file(&path, &stringify_json(&Value::Object(cfg)))
}

fn copilot_approvals_has_rtk(approvals: &[Value]) -> bool {
    for a in approvals {
        let Some(m) = a.as_object() else {
            continue;
        };
        if m.get("kind").and_then(|v| v.as_str()) != Some("commands") {
            continue;
        }
        if let Some(ids) = m.get("commandIdentifiers").and_then(|v| v.as_array())
            && ids.iter().any(|id| id.as_str() == Some("rtk"))
        {
            return true;
        }
    }
    false
}

pub fn install_copilot_codegraph_index_hook() {
    let _ = install_copilot_codegraph_index_hook_safe();
}

pub fn install_copilot_codegraph_index_hook_safe() -> Result<(), String> {
    let p = paths::copilot_paths_resolved();
    paths::ensure_dir(&p.hooks_dir).map_err(|e| e.to_string())?;
    let cmd = tokless_command(&["copilot-hook", "codegraph-index"]);
    if cmd.is_empty() {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }
    let mut hook = OrderedMap::new();
    hook.insert("type".to_string(), Value::String("command".to_string()));
    hook.insert("command".to_string(), Value::String(cmd));
    let mut hooks = OrderedMap::new();
    hooks.insert(
        "sessionStart".to_string(),
        Value::Array(vec![Value::Object(hook)]),
    );
    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_hooks_file("tokless-codegraph-index.json"),
        &stringify_json(&Value::Object(root)),
        "copilot-hook codegraph-index",
    )
}

pub fn remove_copilot_codegraph_index_hook() {
    let _ = remove_copilot_codegraph_index_hook_safe();
}

pub fn remove_copilot_codegraph_index_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(
        &copilot_hooks_file("tokless-codegraph-index.json"),
        "copilot-hook codegraph-index",
    )
}

pub fn install_copilot_ide_codegraph_index_hook() {
    let _ = install_copilot_ide_codegraph_index_hook_safe();
}

pub fn install_copilot_ide_codegraph_index_hook_safe() -> Result<(), String> {
    paths::ensure_dir(&copilot_ide_hooks_dir()).map_err(|e| e.to_string())?;
    let cmd = tokless_command(&["copilot-hook", "codegraph-index", "--vscode"]);
    if cmd.is_empty() {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }
    let mut entry = OrderedMap::new();
    entry.insert("type".to_string(), Value::String("command".to_string()));
    entry.insert("command".to_string(), Value::String(cmd));
    entry.insert("timeout".to_string(), Value::from(120));
    let mut hooks = OrderedMap::new();
    hooks.insert(
        "SessionStart".to_string(),
        Value::Array(vec![Value::Object(entry)]),
    );
    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_ide_hooks_file("tokless-codegraph-index.json"),
        &stringify_json(&Value::Object(root)),
        "copilot-hook codegraph-index",
    )
}

fn write_owned_copilot_hook(path: &str, content: &str, marker: &str) -> Result<(), String> {
    let _guard = hook_write_mu().lock().map_err(|e| e.to_string())?;
    write_owned_copilot_hook_locked(path, content, marker)
}

fn write_owned_copilot_hook_locked(path: &str, content: &str, marker: &str) -> Result<(), String> {
    if let Some(raw) = paths::read_file_safe(path) {
        let Some(cfg) = try_parse_jsonc(&raw) else {
            return Err(format!("refusing to overwrite foreign Copilot hook {path}"));
        };
        if !copilot_hook_owned(&cfg, marker) {
            return Err(format!("refusing to overwrite foreign Copilot hook {path}"));
        }
    }
    write_copilot_file(path, content)
}

fn file_mode_or(path: &str, default_mode: u32) -> Result<u32, String> {
    match fs::metadata(path) {
        Ok(_meta) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                Ok(_meta.permissions().mode() & 0o777)
            }
            #[cfg(not(unix))]
            {
                Ok(default_mode)
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(default_mode),
        Err(e) => Err(e.to_string()),
    }
}

fn write_copilot_file(path: &str, content: &str) -> Result<(), String> {
    let mode = file_mode_or(path, 0o644)?;
    write_copilot_file_mode(path, content, mode)
}

fn write_copilot_file_mode(path: &str, content: &str, mode: u32) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(format!("refusing to write through Copilot symlink {path}"));
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    paths::write_file_mode(path, content, mode).map_err(|e| e.to_string())
}

pub fn write_copilot_instruction_file(path: &str, content: &str) -> Result<(), String> {
    write_copilot_file(path, content)
}

pub fn write_copilot_project_file(path: &str, content: &str) -> Result<(), String> {
    write_copilot_vscode_file(path, content)
}

fn remove_owned_copilot_hook(path: &str, marker: &str) -> Result<(), String> {
    let _guard = hook_write_mu().lock().map_err(|e| e.to_string())?;
    remove_owned_copilot_hook_locked(path, marker)
}

fn remove_owned_copilot_hook_locked(path: &str, marker: &str) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(path)
        && meta.file_type().is_symlink()
    {
        return Err(format!("refusing to remove Copilot hook symlink {path}"));
    }
    let Some(raw) = paths::read_file_safe(path) else {
        return Ok(());
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return Err(format!("refusing to remove foreign Copilot hook {path}"));
    };
    if !copilot_hook_owned(&cfg, marker) {
        return Err(format!("refusing to remove foreign Copilot hook {path}"));
    }
    fs::remove_file(path).map_err(|e| e.to_string())
}

fn copilot_hook_owned(cfg: &OrderedMap, marker: &str) -> bool {
    for key in cfg.keys() {
        if key != "version" && key != "hooks" {
            return false;
        }
    }
    if cfg.get("version").and_then(|v| v.as_i64()) != Some(1) {
        return false;
    }
    let Some(hooks) = json_map_child(cfg, "hooks") else {
        return false;
    };
    let mut expected: HashSet<&str> = HashSet::new();
    let mut expected_timeout: i64 = -1;
    match marker {
        "context-mode hook copilot-cli" => {
            expected.extend([
                "preToolUse",
                "postToolUse",
                "sessionStart",
                "userPromptSubmitted",
                "agentStop",
                "preCompact",
            ]);
        }
        "context-mode hook copilot-vscode" => {
            expected.extend(["PreToolUse", "PostToolUse", "SessionStart", "Stop"]);
            expected_timeout = 10;
        }
        "rtk-hook copilot" => {
            if hooks.len() == 2 {
                expected.extend(["PreToolUse", "PostToolUse"]);
            } else {
                expected.extend(["PreToolUse", "preToolUse", "PostToolUse", "postToolUse"]);
            }
        }
        "projectmem-hook flat" => {
            if hooks.contains_key("SessionStart") {
                expected.insert("SessionStart");
                expected_timeout = 10;
            } else {
                expected.insert("sessionStart");
            }
        }
        "copilot-hook codegraph-index" => {
            if copilot_hook_command_marker(cfg).contains("--vscode") {
                expected.insert("SessionStart");
                expected_timeout = 120;
            } else {
                expected.insert("sessionStart");
            }
        }
        _ => return false,
    }
    if expected.len() != hooks.len() {
        return false;
    }
    for event in hooks.keys() {
        if !expected.contains(event.as_str()) {
            return false;
        }
        let Some(list) = hooks.get(event).and_then(|v| v.as_array()) else {
            return false;
        };
        if list.len() != 1 {
            return false;
        }
        let Some(entry) = list[0].as_object() else {
            return false;
        };
        if entry.len() != 2 && !(expected_timeout >= 0 && entry.len() == 3) {
            return false;
        }
        if entry.get("type").and_then(|v| v.as_str()) != Some("command") {
            return false;
        }
        let timeout_opt = entry.get("timeout").and_then(|v| v.as_i64());
        if (expected_timeout >= 0) != timeout_opt.is_some()
            || (timeout_opt.is_some() && timeout_opt != Some(expected_timeout))
        {
            return false;
        }
        let command_text = entry.get("command").and_then(|v| v.as_str()).unwrap_or("");
        if !copilot_hook_command_owned(command_text, marker) {
            return false;
        }
    }
    !hooks.is_empty()
}

fn copilot_hook_command_marker(cfg: &OrderedMap) -> String {
    let Some(hooks) = json_map_child(cfg, "hooks") else {
        return String::new();
    };
    for event in hooks.keys() {
        if let Some(list) = hooks.get(event).and_then(|v| v.as_array())
            && let Some(entry) = list.first().and_then(|v| v.as_object())
            && let Some(cmd) = entry.get("command").and_then(|v| v.as_str())
        {
            return cmd.to_string();
        }
    }
    String::new()
}

fn copilot_hook_command_owned(command: &str, marker: &str) -> bool {
    match marker {
        "context-mode hook copilot-cli" => has_exact_hook_suffix(
            command,
            marker,
            &[
                "pretooluse",
                "posttooluse",
                "sessionstart",
                "userpromptsubmit",
                "stop",
                "precompact",
            ],
        ),
        "context-mode hook copilot-vscode" => has_exact_hook_suffix(
            command,
            marker,
            &["pretooluse", "posttooluse", "sessionstart", "stop"],
        ),
        "rtk-hook copilot" => tokless_managed_command(command, &["rtk-hook", "copilot"]),
        "projectmem-hook flat" => tokless_managed_command(command, &["projectmem-hook", "flat"]),
        "copilot-hook codegraph-index" => {
            tokless_managed_command(command, &["copilot-hook", "codegraph-index"])
                || tokless_managed_command(
                    command,
                    &["copilot-hook", "codegraph-index", "--vscode"],
                )
        }
        _ => command == marker,
    }
}

fn has_exact_hook_suffix(command: &str, prefix: &str, suffixes: &[&str]) -> bool {
    let Some(rest) = command.strip_prefix(prefix) else {
        return false;
    };
    let Some(suffix) = rest.strip_prefix(' ') else {
        return false;
    };
    suffixes.contains(&suffix)
}

pub fn remove_copilot_ide_codegraph_index_hook() {
    let _ = remove_copilot_ide_codegraph_index_hook_safe();
}

pub fn remove_copilot_ide_codegraph_index_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(
        &copilot_ide_hooks_file("tokless-codegraph-index.json"),
        "copilot-hook codegraph-index",
    )
}

pub fn has_copilot_ide_codegraph_index_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_ide_hooks_file("tokless-codegraph-index.json"))
    else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "copilot-hook codegraph-index")
}

pub fn has_copilot_codegraph_index_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_hooks_file("tokless-codegraph-index.json"))
    else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "copilot-hook codegraph-index")
}

pub fn install_copilot_projectmem_hook_safe() -> Result<(), String> {
    let p = paths::copilot_paths_resolved();
    paths::ensure_dir(&p.hooks_dir).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(copilot_hooks_file("tokless-projectmem.json"));
    let projectmem_cmd = tokless_command(&["projectmem-hook", "flat"]);
    if projectmem_cmd.is_empty() {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }
    let mut hook = OrderedMap::new();
    hook.insert("type".to_string(), Value::String("command".to_string()));
    hook.insert("command".to_string(), Value::String(projectmem_cmd));
    let mut hooks = OrderedMap::new();
    hooks.insert(
        "sessionStart".to_string(),
        Value::Array(vec![Value::Object(hook)]),
    );
    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_hooks_file("projectmem-context.json"),
        &stringify_json(&Value::Object(root)),
        "projectmem-hook flat",
    )
}

pub fn remove_copilot_projectmem_hook_safe() -> Result<(), String> {
    let res = remove_owned_copilot_hook(
        &copilot_hooks_file("projectmem-context.json"),
        "projectmem-hook flat",
    );
    let _ = fs::remove_file(copilot_hooks_file("tokless-projectmem.json"));
    res
}

pub fn has_copilot_projectmem_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_hooks_file("projectmem-context.json")) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "projectmem-hook flat")
}

pub fn install_copilot_ide_projectmem_hook_safe() -> Result<(), String> {
    paths::ensure_dir(&copilot_ide_hooks_dir()).map_err(|e| e.to_string())?;
    let projectmem_cmd = tokless_command(&["projectmem-hook", "flat"]);
    if projectmem_cmd.is_empty() {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }
    let mut hook = OrderedMap::new();
    hook.insert("type".to_string(), Value::String("command".to_string()));
    hook.insert("command".to_string(), Value::String(projectmem_cmd));
    hook.insert("timeout".to_string(), Value::from(10));
    let mut hooks = OrderedMap::new();
    hooks.insert(
        "SessionStart".to_string(),
        Value::Array(vec![Value::Object(hook)]),
    );
    let mut root = OrderedMap::new();
    root.insert("version".to_string(), Value::from(1));
    root.insert("hooks".to_string(), Value::Object(hooks));
    write_owned_copilot_hook(
        &copilot_ide_hooks_file("projectmem-context.json"),
        &stringify_json(&Value::Object(root)),
        "projectmem-hook flat",
    )
}

pub fn remove_copilot_ide_projectmem_hook_safe() -> Result<(), String> {
    remove_owned_copilot_hook(
        &copilot_ide_hooks_file("projectmem-context.json"),
        "projectmem-hook flat",
    )
}

pub fn has_copilot_ide_projectmem_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_ide_hooks_file("projectmem-context.json"))
    else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "projectmem-hook flat")
}

pub fn has_copilot_rtk_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_hooks_file("tokless-rtk.json")) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    copilot_hook_owned(&cfg, "rtk-hook copilot")
}

pub fn configure_copilot_mcp(tool_id: &str) -> (bool, String) {
    match configure_copilot_mcp_safe(tool_id) {
        Ok((changed, file)) => (changed, file),
        Err(_) => (false, paths::copilot_paths_resolved().mcp_config),
    }
}

pub fn configure_copilot_mcp_safe(tool_id: &str) -> Result<(bool, String), String> {
    let p = paths::copilot_paths_resolved();
    paths::ensure_dir(&p.dir).map_err(|e| e.to_string())?;
    let raw = match fs::read_to_string(&p.mcp_config) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.to_string()),
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => {
            if !raw.is_empty() {
                return Err(format!("invalid Copilot MCP config {}", p.mcp_config));
            }
            OrderedMap::new()
        }
    };
    if cfg.get("mcpServers").is_some_and(|v| !v.is_object()) {
        return Err(format!(
            "Copilot MCP config {} has non-object mcpServers",
            p.mcp_config
        ));
    }
    if !cfg.contains_key("mcpServers") {
        cfg.insert("mcpServers".to_string(), Value::Object(OrderedMap::new()));
    }
    let desired = copilot_mcp_desired(tool_id, false);
    let servers = json_map_child_mut(&mut cfg, "mcpServers").unwrap();
    if let Some(existing) = servers.get(tool_id) {
        if copilot_mcp_equal(existing, &desired) {
            return Ok((false, p.mcp_config));
        }
        if !copilot_managed_mcp_entry(existing, tool_id, false) {
            return Err(format!(
                "refusing to overwrite foreign Copilot MCP entry {tool_id}"
            ));
        }
    }
    servers.insert(tool_id.to_string(), Value::Object(desired));
    write_copilot_file(&p.mcp_config, &stringify_json(&Value::Object(cfg)))?;
    Ok((true, p.mcp_config))
}

pub fn remove_copilot_mcp(tool_id: &str) -> bool {
    remove_copilot_mcp_safe(tool_id).unwrap_or(false)
}

pub fn remove_copilot_mcp_safe(tool_id: &str) -> Result<bool, String> {
    let p = paths::copilot_paths_resolved();
    let raw = match fs::read_to_string(&p.mcp_config) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return Err(format!("invalid Copilot MCP config {}", p.mcp_config));
    };
    if cfg.get("mcpServers").is_some_and(|v| !v.is_object()) {
        return Err(format!(
            "Copilot MCP config {} has non-object mcpServers",
            p.mcp_config
        ));
    }
    let Some(servers) = json_map_child_mut(&mut cfg, "mcpServers") else {
        return Ok(false);
    };
    let Some(existing) = servers.get(tool_id) else {
        return Ok(false);
    };
    if !copilot_managed_mcp_entry(existing, tool_id, false) {
        return Err(format!(
            "refusing to remove foreign Copilot MCP entry {tool_id}"
        ));
    }
    servers.shift_remove(tool_id);
    write_copilot_file(&p.mcp_config, &stringify_json(&Value::Object(cfg)))?;
    Ok(true)
}

pub fn copilot_mcp_has(tool_id: &str) -> bool {
    let p = paths::copilot_paths_resolved();
    let Some(raw) = paths::read_file_safe(&p.mcp_config) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    json_map_child(&cfg, "mcpServers").is_some_and(|sm| sm.contains_key(tool_id))
}

pub fn configure_copilot_ide_mcp(tool_id: &str) -> (bool, String) {
    match configure_copilot_ide_mcp_safe(tool_id) {
        Ok((changed, file)) => (changed, file),
        Err(_) => (false, copilot_ide_mcp_file()),
    }
}

pub fn configure_copilot_ide_mcp_safe(tool_id: &str) -> Result<(bool, String), String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    configure_copilot_ide_mcp_locked(tool_id)
}

fn configure_copilot_ide_mcp_locked(tool_id: &str) -> Result<(bool, String), String> {
    let path = copilot_ide_mcp_file();
    if let Some(parent) = Path::new(&path).parent() {
        paths::ensure_dir(parent).map_err(|e| e.to_string())?;
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.to_string()),
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => {
            if !raw.is_empty() {
                return Err(format!("invalid Copilot VS Code MCP config {path}"));
            }
            OrderedMap::new()
        }
    };
    if cfg.get("servers").is_some_and(|v| !v.is_object()) {
        return Err(format!("Copilot MCP config {path} has non-object servers"));
    }
    if !cfg.contains_key("servers") {
        cfg.insert("servers".to_string(), Value::Object(OrderedMap::new()));
    }
    let desired = copilot_mcp_desired(tool_id, true);
    let servers = json_map_child_mut(&mut cfg, "servers").unwrap();
    if let Some(existing) = servers.get(tool_id) {
        if copilot_mcp_equal(existing, &desired) {
            return Ok((false, path));
        }
        if !copilot_managed_mcp_entry(existing, tool_id, true) {
            return Err(format!(
                "refusing to overwrite foreign Copilot VS Code MCP entry {tool_id}"
            ));
        }
    }
    servers.insert(tool_id.to_string(), Value::Object(desired));
    write_copilot_file(&path, &stringify_json(&Value::Object(cfg)))?;
    Ok((true, path))
}

fn copilot_mcp_desired(tool_id: &str, ide: bool) -> OrderedMap {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("copilot", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut desired = OrderedMap::new();
    desired.insert(
        "type".to_string(),
        Value::String(if ide { "stdio" } else { "local" }.to_string()),
    );
    desired.insert("command".to_string(), Value::String(spawn.command));
    desired.insert(
        "args".to_string(),
        Value::Array(strings_to_values(&spawn.args)),
    );
    if !ide {
        desired.insert(
            "tools".to_string(),
            Value::Array(vec![Value::String("*".to_string())]),
        );
    }
    if tool_id == "context-mode" {
        let mut env = OrderedMap::new();
        env.insert(
            "CONTEXT_MODE_PLATFORM".to_string(),
            Value::String(if ide { "copilot-vscode" } else { "copilot-cli" }.to_string()),
        );
        env.insert(
            "CONTEXT_MODE_COPILOT_PLUGIN".to_string(),
            Value::String("1".to_string()),
        );
        desired.insert("env".to_string(), Value::Object(env));
    }
    desired
}

fn copilot_managed_mcp_entry(v: &Value, tool_id: &str, ide: bool) -> bool {
    copilot_mcp_equal(v, &copilot_mcp_desired(tool_id, ide))
}

fn copilot_mcp_equal(existing: &Value, desired: &OrderedMap) -> bool {
    let Some(em) = existing.as_object() else {
        return false;
    };
    if em.len() != desired.len() {
        return false;
    }
    for (key, want) in desired {
        let Some(have) = em.get(key) else {
            return false;
        };
        if stringify_json(have) != stringify_json(want) {
            return false;
        }
    }
    true
}

pub fn remove_copilot_ide_mcp(tool_id: &str) -> bool {
    remove_copilot_ide_mcp_safe(tool_id).unwrap_or(false)
}

pub fn remove_copilot_ide_mcp_safe(tool_id: &str) -> Result<bool, String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    remove_copilot_ide_mcp_locked(tool_id)
}

fn remove_copilot_ide_mcp_locked(tool_id: &str) -> Result<bool, String> {
    let path = copilot_ide_mcp_file();
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return Err(format!("invalid Copilot VS Code MCP config {path}"));
    };
    let Some(servers_val) = cfg.get("servers") else {
        return Ok(false);
    };
    if !servers_val.is_object() {
        return Err(format!("Copilot MCP config {path} has non-object servers"));
    }
    let servers = json_map_child_mut(&mut cfg, "servers").unwrap();
    let Some(existing) = servers.get(tool_id) else {
        return Ok(false);
    };
    if !copilot_managed_mcp_entry(existing, tool_id, true) {
        return Err(format!(
            "refusing to remove foreign Copilot VS Code MCP entry {tool_id}"
        ));
    }
    servers.shift_remove(tool_id);
    write_copilot_file(&path, &stringify_json(&Value::Object(cfg)))?;
    Ok(true)
}

pub fn copilot_mcp_bounded(tool_id: &str, ide: bool) -> bool {
    let (path, child) = if ide {
        (copilot_ide_mcp_file(), "servers")
    } else {
        (paths::copilot_paths_resolved().mcp_config, "mcpServers")
    };
    let Some(raw) = paths::read_file_safe(&path) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(sm) = json_map_child(&cfg, child) else {
        return false;
    };
    let Some(existing) = sm.get(tool_id) else {
        return false;
    };
    copilot_mcp_equal(existing, &copilot_mcp_desired(tool_id, ide))
}

pub fn copilot_projectmem_mcp_bounded(ide: bool) -> bool {
    copilot_mcp_bounded("projectmem", ide)
}

pub fn copilot_ide_mcp_has(tool_id: &str) -> bool {
    let Some(raw) = paths::read_file_safe(&copilot_ide_mcp_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    json_map_child(&cfg, "servers").is_some_and(|sm| sm.contains_key(tool_id))
}

pub fn copilot_transaction_file_owned(path: &str) -> bool {
    let Some(raw) = paths::read_file_safe(path) else {
        return false;
    };
    let base = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if base == "context-mode.json"
        || base == "tokless-codegraph-index.json"
        || base == "tokless-rtk.json"
    {
        let Some(cfg) = try_parse_jsonc(&raw) else {
            return false;
        };
        for marker in [
            "context-mode hook copilot-cli",
            "context-mode hook copilot-vscode",
            "copilot-hook codegraph-index",
            "rtk-hook copilot",
        ] {
            if copilot_hook_owned(&cfg, marker) {
                return true;
            }
        }
        return false;
    }
    if path == copilot_ide_mcp_file() || path == paths::copilot_paths_resolved().mcp_config {
        let Some(cfg) = try_parse_jsonc(&raw) else {
            return false;
        };
        let (key, ide) = if path == copilot_ide_mcp_file() {
            ("servers", true)
        } else {
            ("mcpServers", false)
        };
        let Some(entries) = json_map_child(&cfg, key) else {
            return false;
        };
        if entries.is_empty() || cfg.len() != 1 {
            return false;
        }
        for (tool_id, entry) in entries {
            if !copilot_managed_mcp_entry(entry, tool_id, ide) {
                return false;
            }
        }
        return true;
    }
    const BEGIN: &str = "# tokless:copilot-instructions begin";
    const END: &str = "# tokless:copilot-instructions end";
    if base == "copilot-instructions.md" {
        return raw.starts_with(&format!("{BEGIN}\n"))
            && raw.ends_with(&format!("{END}\n"))
            && raw.matches(BEGIN).count() == 1
            && raw.matches(END).count() == 1;
    }
    false
}

pub fn sync_copilot_ide_instructions() {
    let _ = sync_copilot_ide_instructions_safe();
}

pub fn sync_copilot_ide_instructions_safe() -> Result<(), String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    sync_copilot_ide_instructions_locked()
}

fn sync_copilot_ide_instructions_locked() -> Result<(), String> {
    let cli_path = paths::copilot_paths_resolved().instructions;
    let (body, ok) = match fs::read_to_string(&cli_path) {
        Ok(s) => (s, true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), false),
        Err(e) => return Err(e.to_string()),
    };
    let path = copilot_ide_instructions_file();
    const BEGIN: &str = "# tokless:copilot-instructions begin";
    const END: &str = "# tokless:copilot-instructions end";
    if !ok || body.trim().is_empty() {
        if let Some(existing) = paths::read_file_safe(&path)
            && let (Some(start), Some(finish_idx)) = (existing.find(BEGIN), existing.find(END))
            && finish_idx >= start
        {
            let finish = finish_idx + END.len();
            let remaining = format!("{}{}", &existing[..start], &existing[finish..]);
            if remaining.trim().is_empty() {
                return clear_copilot_project_file_locked(&path);
            }
            return write_copilot_vscode_file_locked(
                &path,
                &format!("{}\n", remaining.trim_end_matches('\n')),
            );
        }
        return Ok(());
    }
    if let Some(parent) = Path::new(&path).parent() {
        paths::ensure_dir(parent).map_err(|e| e.to_string())?;
    }
    let mut managed = format!("{BEGIN}\n{}\n{END}\n", body.trim_end_matches('\n'));
    if let Some(existing) = paths::read_file_safe(&path)
        && !existing.trim().is_empty()
    {
        let (Some(start), Some(finish_idx)) = (existing.find(BEGIN), existing.find(END)) else {
            return Err(format!(
                "refusing to overwrite foreign Copilot instructions {path}"
            ));
        };
        if finish_idx < start {
            return Err(format!(
                "refusing to overwrite foreign Copilot instructions {path}"
            ));
        }
        let finish = finish_idx + END.len();
        managed = format!("{}{}{}", &existing[..start], managed, &existing[finish..]);
    }
    write_copilot_vscode_file_locked(&path, &managed)
}

// --- copilot_cli.go ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CopilotProviderConfig {
    pub managed_by: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: String,
    pub model: String,
}

fn copilot_headroom_command() -> String {
    let hr = crate::util::headroom::headroom_paths_resolved();
    if !hr.bin.is_empty() && paths::exists(&hr.bin) {
        return hr.bin;
    }
    let b = paths::which("headroom");
    if !b.is_empty() {
        return b;
    }
    "headroom".to_string()
}

pub(crate) fn copilot_shim_path() -> String {
    let persisted = paths::tokless_persisted_abs();
    let mut dir = Path::new(&persisted)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    if dir == "." || dir.is_empty() {
        let dirs = paths::expected_bin_dirs();
        if let Some(first) = dirs.first() {
            dir = first.clone();
        } else {
            dir = Path::new(&paths::home())
                .join(".local")
                .join("bin")
                .to_string_lossy()
                .into_owned();
        }
    }
    let name = if paths::is_win() {
        "copilot.cmd"
    } else {
        "copilot"
    };
    Path::new(&dir).join(name).to_string_lossy().into_owned()
}

fn copilot_stashed_path() -> String {
    format!("{}.real", copilot_shim_path())
}

fn copilot_stash_marker_path() -> String {
    format!("{}.tokless", copilot_stashed_path())
}

pub fn copilot_shim_digest_path() -> String {
    format!("{}.digest", copilot_shim_path())
}

fn copilot_stash_marker(raw: &str) -> String {
    format!("{}\n", sha256_hex(raw.as_bytes()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn copilot_shim_text() -> String {
    let tokless = paths::tokless_persisted_abs();
    if paths::is_win() {
        return format!(
            "@echo off\r\nrem tokless:copilot-shim\r\nsetlocal\r\n\"{tokless}\" __copilot %*\r\nexit /b %ERRORLEVEL%\r\n"
        );
    }
    format!(
        "#!/bin/sh\n# tokless:copilot-shim\nexec {} __copilot \"$@\"\n",
        shell_quote(&tokless)
    )
}

fn copilot_shim_owned(raw: &str) -> bool {
    if raw == copilot_shim_text() {
        return true;
    }
    let Some(digest) = paths::read_file_safe(&copilot_shim_digest_path()) else {
        return false;
    };
    digest.trim() == copilot_stash_marker(raw).trim()
}

pub fn copilot_provider_config_path() -> String {
    Path::new(&paths::tokless_data_dir())
        .join("copilot.json")
        .to_string_lossy()
        .into_owned()
}

pub fn copilot_config_path() -> String {
    copilot_provider_config_path()
}

fn save_copilot_provider_config() -> Result<(), String> {
    let base_url = std::env::var("COPILOT_PROVIDER_BASE_URL")
        .unwrap_or_default()
        .trim()
        .to_string();
    if base_url.is_empty() {
        return Ok(());
    }
    let mut provider_type = std::env::var("COPILOT_PROVIDER_TYPE")
        .unwrap_or_default()
        .trim()
        .to_string();
    if provider_type.is_empty() {
        provider_type = "openai".to_string();
    }
    let model = std::env::var("COPILOT_PROVIDER_MODEL_ID")
        .unwrap_or_default()
        .trim()
        .to_string();
    let cfg = CopilotProviderConfig {
        managed_by: "tokless".to_string(),
        provider_type,
        base_url,
        model,
    };
    let b = serde_json::to_string(&cfg).map_err(|e| e.to_string())?;
    paths::ensure_dir(&paths::tokless_data_dir()).map_err(|e| e.to_string())?;
    let cfg_path = copilot_config_path();
    if let Some(raw) = paths::read_file_safe(&cfg_path) {
        let valid = serde_json::from_str::<CopilotProviderConfig>(&raw)
            .ok()
            .is_some_and(|existing| existing.managed_by == "tokless");
        if !valid {
            return Err(format!(
                "refusing to overwrite foreign Copilot provider metadata {cfg_path}"
            ));
        }
    }
    write_copilot_file_mode(&cfg_path, &format!("{b}\n"), 0o600)
}

fn load_copilot_provider_config() -> Option<CopilotProviderConfig> {
    let raw = paths::read_file_safe(&copilot_config_path())?;
    let cfg: CopilotProviderConfig = serde_json::from_str(&raw).ok()?;
    if cfg.managed_by != "tokless" || cfg.base_url.is_empty() {
        return None;
    }
    Some(cfg)
}

fn install_copilot_shim() -> Result<bool, String> {
    let path = copilot_shim_path();
    let want = copilot_shim_text();
    match fs::symlink_metadata(&path) {
        Ok(meta) => {
            if !meta.file_type().is_file() {
                return Err(format!(
                    "refusing to replace non-regular Copilot command {path}"
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    if let Some(raw) = paths::read_file_safe(&path) {
        if raw != want {
            if copilot_shim_owned(&raw) {
                let old_digest = paths::read_file_safe(&copilot_shim_digest_path());
                let mode = file_mode_or(&path, 0o755).unwrap_or(0o755);
                write_copilot_file_mode(&path, &want, mode)?;
                if let Err(err) = write_copilot_file_mode(
                    &copilot_shim_digest_path(),
                    &copilot_stash_marker(&want),
                    0o600,
                ) {
                    let _ = write_copilot_file_mode(&path, &raw, mode);
                    if let Some(d) = old_digest {
                        let _ = write_copilot_file_mode(&copilot_shim_digest_path(), &d, 0o600);
                    } else {
                        let _ = fs::remove_file(copilot_shim_digest_path());
                    }
                    return Err(err);
                }
                return Ok(true);
            }
            if paths::is_win() {
                return Err(format!(
                    "refusing to replace non-Tokless Copilot command {path}"
                ));
            }
            let stashed = copilot_stashed_path();
            if fs::metadata(&stashed).is_ok() {
                return Err(format!(
                    "refusing to replace non-Tokless Copilot command {path}: backup already exists"
                ));
            }
            fs::rename(&path, &stashed)
                .map_err(|e| format!("stash real Copilot command: {e}"))?;
            if let Err(err) = write_copilot_file_mode(
                &copilot_stash_marker_path(),
                &copilot_stash_marker(&raw),
                0o600,
            ) {
                let _ = fs::rename(&stashed, &path);
                return Err(format!("record real Copilot command: {err}"));
            }
            if let Err(err) = write_copilot_file_mode(
                &copilot_shim_digest_path(),
                &copilot_stash_marker(&want),
                0o600,
            ) {
                let _ = fs::rename(&stashed, &path);
                let _ = fs::remove_file(copilot_stash_marker_path());
                return Err(format!("record Copilot shim: {err}"));
            }
            if let Err(err) = write_copilot_file_mode(&path, &want, 0o755) {
                let _ = fs::rename(&stashed, &path);
                let _ = fs::remove_file(copilot_stash_marker_path());
                let _ = fs::remove_file(copilot_shim_digest_path());
                return Err(err);
            }
            return Ok(true);
        }
        return Ok(false);
    }
    if let Some(parent) = Path::new(&path).parent() {
        paths::ensure_dir(parent).map_err(|e| e.to_string())?;
    }
    let mode = if paths::is_win() { 0o644 } else { 0o755 };
    write_copilot_file_mode(&path, &want, mode)?;
    if let Err(err) = write_copilot_file_mode(
        &copilot_shim_digest_path(),
        &copilot_stash_marker(&want),
        0o600,
    ) {
        let _ = fs::remove_file(&path);
        return Err(err);
    }
    Ok(true)
}

fn remove_copilot_shim() -> bool {
    let path = copilot_shim_path();
    let Some(raw) = paths::read_file_safe(&path) else {
        return false;
    };
    if !copilot_shim_owned(&raw) {
        return false;
    }
    let shim_mode = file_mode_or(&path, 0o755).unwrap_or(0o755);
    if !paths::is_win() {
        let stashed = copilot_stashed_path();
        if fs::metadata(&stashed).is_ok() {
            let marker = paths::read_file_safe(&copilot_stash_marker_path());
            let real = paths::read_file_safe(&stashed);
            let is_reg = fs::symlink_metadata(&stashed)
                .ok()
                .is_some_and(|m| m.file_type().is_file());
            let (Some(marker_raw), Some(real_raw)) = (marker, real) else {
                return false;
            };
            if !is_reg || marker_raw.trim() != copilot_stash_marker(&real_raw).trim() {
                return false;
            }
            let digest_raw = paths::read_file_safe(&copilot_shim_digest_path());
            let tmp = format!("{path}.removing");
            if fs::rename(&path, &tmp).is_err() {
                return false;
            }
            if fs::rename(&stashed, &path).is_err() {
                let _ = fs::rename(&tmp, &path);
                return false;
            }
            if fs::remove_file(&tmp).is_err() {
                let _ = fs::rename(&path, &stashed);
                let _ = fs::rename(&tmp, &path);
                return false;
            }
            if remove_copilot_sidecars().is_err() {
                let _ = fs::rename(&path, &stashed);
                let _ = write_copilot_file_mode(&path, &raw, shim_mode);
                let _ = write_copilot_file_mode(&copilot_stash_marker_path(), &marker_raw, 0o600);
                if let Some(d) = digest_raw {
                    let _ = write_copilot_file_mode(&copilot_shim_digest_path(), &d, 0o600);
                }
                return false;
            }
            return true;
        }
    }
    if fs::remove_file(&path).is_err() {
        return false;
    }
    if let Err(err) = fs::remove_file(copilot_shim_digest_path())
        && err.kind() != std::io::ErrorKind::NotFound
    {
        let _ = write_copilot_file_mode(&path, &raw, shim_mode);
        return false;
    }
    true
}

fn remove_copilot_sidecars() -> Result<(), String> {
    if let Err(e) = fs::remove_file(copilot_stash_marker_path())
        && e.kind() != std::io::ErrorKind::NotFound
    {
        return Err(e.to_string());
    }
    if let Err(e) = fs::remove_file(copilot_shim_digest_path())
        && e.kind() != std::io::ErrorKind::NotFound
    {
        return Err(e.to_string());
    }
    Ok(())
}

fn copilot_shim_installed() -> bool {
    paths::read_file_safe(&copilot_shim_path()).is_some_and(|raw| copilot_shim_owned(&raw))
}

fn env_value(env: &HashMap<String, String>, key: &str) -> String {
    env.get(key).map(|s| s.trim().to_string()).unwrap_or_default()
}

fn copilot_command_env() -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();
    if let Some(cfg) = load_copilot_provider_config() {
        if env_value(&env, "COPILOT_PROVIDER_BASE_URL").is_empty() {
            env.insert("COPILOT_PROVIDER_BASE_URL".to_string(), cfg.base_url);
        }
        if env_value(&env, "COPILOT_PROVIDER_TYPE").is_empty() {
            env.insert("COPILOT_PROVIDER_TYPE".to_string(), cfg.provider_type);
        }
        if env_value(&env, "COPILOT_PROVIDER_MODEL_ID").is_empty() {
            env.insert("COPILOT_PROVIDER_MODEL_ID".to_string(), cfg.model);
        }
    }
    env
}

fn real_copilot_path(shim: &str, env: &HashMap<String, String>) -> String {
    let name = if paths::is_win() {
        "copilot.exe"
    } else {
        "copilot"
    };
    let shim_dir = Path::new(shim)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let is_regular = |p: &Path| fs::metadata(p).ok().is_some_and(|m| m.is_file());

    if !paths::is_win() {
        let stashed = copilot_stashed_path();
        if Path::new(&stashed).parent() == Some(shim_dir.as_path()) {
            let is_reg = fs::symlink_metadata(&stashed)
                .ok()
                .is_some_and(|m| m.file_type().is_file());
            if let (true, Some(real), Some(marker)) = (
                is_reg,
                paths::read_file_safe(&stashed),
                paths::read_file_safe(&copilot_stash_marker_path()),
            ) && marker.trim() == copilot_stash_marker(&real).trim()
            {
                return stashed;
            }
        }
    } else {
        let p = shim_dir.join(name);
        if p != Path::new(shim) && is_regular(&p) {
            return p.to_string_lossy().into_owned();
        }
    }
    let path_var = env.get("PATH").or_else(|| env.get("Path")).cloned().unwrap_or_default();
    let sep = if paths::is_win() { ';' } else { ':' };
    for dir in path_var.split(sep) {
        if dir.is_empty() || Path::new(dir) == shim_dir.as_path() {
            continue;
        }
        let p = Path::new(dir).join(name);
        if is_regular(&p) {
            return p.to_string_lossy().into_owned();
        }
        if paths::is_win() {
            for ext in [".cmd", ".bat"] {
                let p_ext = Path::new(dir).join(format!("copilot{ext}"));
                if is_regular(&p_ext) {
                    return p_ext.to_string_lossy().into_owned();
                }
            }
        }
    }
    String::new()
}

pub fn run_copilot_cli(args: &[String]) -> i32 {
    let mut env = copilot_command_env();
    let shim = copilot_shim_path();
    let real_copilot = real_copilot_path(&shim, &env);
    if real_copilot.is_empty() {
        eprintln!("tokless: real Copilot CLI not found on PATH");
        return 127;
    }
    let headroom = copilot_headroom_command();
    if headroom.is_empty() {
        eprintln!("tokless: Headroom binary not found; run tokless first");
        return 127;
    }

    let base = Path::new(&real_copilot)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut temp_dir_opt: Option<std::path::PathBuf> = None;
    let command_path = if base == "copilot" || (paths::is_win() && base == "copilot.exe") {
        real_copilot.clone()
    } else {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "tokless-copilot-{}-{nanos}",
            std::process::id()
        ));
        if let Err(err) = fs::create_dir_all(&dir) {
            eprintln!("tokless: prepare real Copilot CLI: {err}");
            return 1;
        }
        let Ok(raw_bytes) = fs::read(&real_copilot) else {
            let _ = fs::remove_dir_all(&dir);
            eprintln!("tokless: prepare real Copilot CLI: read failed");
            return 1;
        };
        let mut alias_name = "copilot".to_string();
        if paths::is_win()
            && let Some(ext) = Path::new(&real_copilot).extension().and_then(|s| s.to_str())
        {
            let ext_lower = ext.to_ascii_lowercase();
            if ext_lower == "exe" || ext_lower == "cmd" || ext_lower == "bat" {
                alias_name.push('.');
                alias_name.push_str(&ext_lower);
            }
        }
        let alias = dir.join(alias_name);
        if let Err(err) = fs::write(&alias, raw_bytes) {
            let _ = fs::remove_dir_all(&dir);
            eprintln!("tokless: prepare real Copilot CLI: {err}");
            return 1;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&alias, fs::Permissions::from_mode(0o755));
        }
        temp_dir_opt = Some(dir);
        alias.to_string_lossy().into_owned()
    };

    let sep = if paths::is_win() { ";" } else { ":" };
    let sep_char = if paths::is_win() { ';' } else { ':' };
    let shim_dir = Path::new(&shim).parent();
    let path_var = env.get("PATH").or_else(|| env.get("Path")).cloned().unwrap_or_default();
    let mut path_parts: Vec<String> = path_var
        .split(sep_char)
        .filter(|d| !d.is_empty() && Some(Path::new(*d)) != shim_dir)
        .map(|s| s.to_string())
        .collect();
    if command_path != real_copilot
        && let Some(cmd_dir) = Path::new(&command_path).parent()
    {
        path_parts.insert(0, cmd_dir.to_string_lossy().into_owned());
    }
    env.insert("PATH".to_string(), path_parts.join(sep));
    env.insert("HEADROOM_WRAP_SILENT".to_string(), "1".to_string());
    let base_url_val = env_value(&env, "COPILOT_PROVIDER_BASE_URL");
    env.insert("COPILOT_PROVIDER_BASE_URL".to_string(), base_url_val.clone());
    env.remove("GROK_MODELS_BASE_URL");

    let provider_key = env_value(&env, "COPILOT_PROVIDER_API_KEY");
    let bearer = env_value(&env, "COPILOT_PROVIDER_BEARER_TOKEN");
    let headers = env_value(&env, "COPILOT_PROVIDER_HEADERS");
    let mut provider_type = env_value(&env, "COPILOT_PROVIDER_TYPE");
    if provider_type.is_empty() {
        provider_type = "openai".to_string();
    }
    let mut wrap_args = vec![
        "wrap".to_string(),
        "copilot".to_string(),
        "--port".to_string(),
        copilot_cli_proxy_port().to_string(),
    ];
    if (!provider_key.is_empty() || !bearer.is_empty() || !headers.is_empty())
        && (!base_url_val.is_empty() || !headers.is_empty())
    {
        if provider_type != "anthropic" && provider_type != "openai" {
            if let Some(dir) = temp_dir_opt {
                let _ = fs::remove_dir_all(dir);
            }
            eprintln!(
                "Unsupported COPILOT_PROVIDER_TYPE: {provider_type} (expected anthropic or openai)"
            );
            return 2;
        }
        if env_value(&env, "COPILOT_MODEL").is_empty() {
            let model_id = env_value(&env, "COPILOT_PROVIDER_MODEL_ID");
            env.insert("COPILOT_MODEL".to_string(), model_id);
        }
        wrap_args.push("--provider-type".to_string());
        wrap_args.push(provider_type.clone());
        if provider_type == "anthropic" {
            env.remove("OPENAI_TARGET_API_URL");
            env.insert("ANTHROPIC_TARGET_API_URL".to_string(), base_url_val);
        } else {
            env.remove("ANTHROPIC_TARGET_API_URL");
            env.insert("OPENAI_TARGET_API_URL".to_string(), base_url_val);
        }
    } else {
        env.remove("OPENAI_TARGET_API_URL");
        env.remove("ANTHROPIC_TARGET_API_URL");
    }
    wrap_args.push("--".to_string());
    wrap_args.extend(args.iter().cloned());

    let status = Command::new(&headroom)
        .args(&wrap_args)
        .env_clear()
        .envs(&env)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    if let Some(dir) = temp_dir_opt {
        let _ = fs::remove_dir_all(dir);
    }

    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

// --- copilot_proxy.go ---

pub fn copilot_vscode_settings_file() -> String {
    let home = paths::home();
    if paths::is_win() {
        if let Ok(appdata) = std::env::var("APPDATA")
            && !appdata.is_empty()
        {
            return Path::new(&appdata)
                .join("Code")
                .join("User")
                .join("settings.json")
                .to_string_lossy()
                .into_owned();
        }
        return Path::new(&home)
            .join("AppData")
            .join("Roaming")
            .join("Code")
            .join("User")
            .join("settings.json")
            .to_string_lossy()
            .into_owned();
    }
    if cfg!(target_os = "macos") {
        return Path::new(&home)
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User")
            .join("settings.json")
            .to_string_lossy()
            .into_owned();
    }
    let config = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Path::new(&home).join(".config").to_string_lossy().into_owned());
    Path::new(&config)
        .join("Code")
        .join("User")
        .join("settings.json")
        .to_string_lossy()
        .into_owned()
}

pub fn copilot_proxy_port() -> u16 {
    if let Ok(raw) = std::env::var("TOKLESS_COPILOT_PROXY_PORT")
        && let Ok(port) = raw.trim().parse::<u16>()
        && port > 0
    {
        return port;
    }
    DEFAULT_COPILOT_PROXY_PORT
}

pub fn copilot_cli_proxy_port() -> u16 {
    if let Ok(raw) = std::env::var("TOKLESS_COPILOT_CLI_PROXY_PORT")
        && let Ok(port) = raw.trim().parse::<u16>()
        && port > 0
    {
        return port;
    }
    DEFAULT_COPILOT_CLI_PROXY_PORT
}

pub fn validate_copilot_proxy_ports() -> Result<(), String> {
    let p = copilot_proxy_port();
    if p == copilot_cli_proxy_port() {
        return Err(format!(
            "Copilot CLI and VS Code proxy ports must differ (both are {p})"
        ));
    }
    Ok(())
}

fn copilot_normalize_fence_block(block: &str) -> String {
    let replaced = block.replace("\r\n", "\n");
    format!("{}\n", replaced.strip_suffix('\n').unwrap_or(&replaced))
}

fn copilot_vscode_proxy_url() -> String {
    format!("http://127.0.0.1:{}", copilot_proxy_port())
}

fn write_copilot_vscode_file(path: &str, content: &str) -> Result<(), String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    write_copilot_vscode_file_locked(path, content)
}

fn write_copilot_vscode_file_locked(path: &str, content: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            let target = fs::canonicalize(path).map_err(|e| e.to_string())?;
            let current = fs::canonicalize(path).map_err(|e| e.to_string())?;
            if current != target {
                return Err(format!(
                    "Copilot VS Code symlink changed during write {path}"
                ));
            }
            return write_copilot_file(&target.to_string_lossy(), content);
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    write_copilot_file(path, content)
}

pub fn clear_copilot_project_file(path: &str) -> Result<(), String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    clear_copilot_project_file_locked(path)
}

fn clear_copilot_project_file_locked(path: &str) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(path)
        && meta.file_type().is_symlink()
    {
        let target = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let current = fs::canonicalize(path).map_err(|e| e.to_string())?;
        if current != target {
            return Err(format!(
                "Copilot project symlink changed during cleanup {path}"
            ));
        }
        return write_copilot_file(&target.to_string_lossy(), "");
    }
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn copilot_vscode_span(raw: &str) -> Option<(usize, usize)> {
    let mut start = raw.find(COPILOT_VSCODE_MARKER_HEAD)?;
    if raw.matches(COPILOT_VSCODE_MARKER_HEAD).count() != 1
        || raw.matches(COPILOT_VSCODE_MARKER_FOOT).count() != 1
    {
        return None;
    }
    let end_marker = raw[start..].find(COPILOT_VSCODE_MARKER_FOOT)?;
    let mut end = start + end_marker + COPILOT_VSCODE_MARKER_FOOT.len();
    if let Some(line_start) = raw[..start].rfind('\n') {
        start = line_start + 1;
    }
    if let Some(line_end) = raw[end..].find('\n') {
        end += line_end + 1;
    }
    Some((start, end))
}

fn copilot_vscode_block(comma_added: bool) -> String {
    let mut marker = COPILOT_VSCODE_MARKER_HEAD.to_string();
    if comma_added {
        marker.push_str(" (comma-added)");
    }
    let url = copilot_vscode_proxy_url();
    format!(
        "\t{marker}\n\t\"{COPILOT_VSCODE_PROXY_KEY}\": \"{url}\",\n\t\"{COPILOT_VSCODE_CAPI_KEY}\": \"{url}\"\n\t{COPILOT_VSCODE_MARKER_FOOT}"
    )
}

fn copilot_vscode_legacy_block(comma_added: bool) -> String {
    let mut marker = COPILOT_VSCODE_MARKER_HEAD.to_string();
    if comma_added {
        marker.push_str(" (comma-added)");
    }
    let url = copilot_vscode_proxy_url();
    format!(
        "\t{marker}\n\t\"{COPILOT_VSCODE_PROXY_KEY}\": \"{url}\",\n\t\"{COPILOT_VSCODE_AUTH_KEY}\": \"token\"\n\t{COPILOT_VSCODE_MARKER_FOOT}"
    )
}

fn jsonc_root_object_end(raw: &str) -> Option<usize> {
    let bytes = raw.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_line_comment {
            if c == b'\n' || c == b'\r' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_string = true,
            b'/' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    in_line_comment = true;
                    i += 1;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    in_block_comment = true;
                    i += 1;
                }
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn jsonc_last_token_end(raw: &str, limit: usize) -> usize {
    let bytes = raw.as_bytes();
    let limit = limit.min(bytes.len());
    let mut last = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut i = 0;
    while i < limit {
        let c = bytes[i];
        if in_line_comment {
            if c == b'\n' || c == b'\r' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && i + 1 < limit && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_string = false;
                last = i + 1;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_string = true,
            b'/' => {
                if i + 1 < limit && bytes[i + 1] == b'/' {
                    in_line_comment = true;
                    i += 1;
                } else if i + 1 < limit && bytes[i + 1] == b'*' {
                    in_block_comment = true;
                    i += 1;
                } else {
                    last = i + 1;
                }
            }
            _ => {
                if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
                    last = i + 1;
                }
            }
        }
        i += 1;
    }
    last
}

fn jsonc_last_comma(raw: &str) -> Option<usize> {
    let bytes = raw.as_bytes();
    let mut last = None;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_line_comment {
            if c == b'\n' || c == b'\r' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_string = true,
            b'/' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    in_line_comment = true;
                    i += 1;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    in_block_comment = true;
                    i += 1;
                }
            }
            b',' => last = Some(i),
            _ => {}
        }
        i += 1;
    }
    last
}

fn copilot_vscode_current_managed(raw: &str) -> bool {
    let Some((start, end)) = copilot_vscode_span(raw) else {
        return false;
    };
    let block = copilot_normalize_fence_block(&raw[start..end]);
    block == format!("{}\n", copilot_vscode_block(false))
        || block == format!("{}\n", copilot_vscode_block(true))
}

fn copilot_vscode_legacy_managed(raw: &str) -> bool {
    let Some((start, end)) = copilot_vscode_span(raw) else {
        return false;
    };
    let block = copilot_normalize_fence_block(&raw[start..end]);
    block == format!("{}\n", copilot_vscode_legacy_block(false))
        || block == format!("{}\n", copilot_vscode_legacy_block(true))
}

pub fn configure_copilot_cli_proxy() -> (bool, String) {
    let _guard = proxy_op_mu().lock().ok();
    configure_copilot_cli_proxy_safe().unwrap_or_else(|_| (false, copilot_shim_path()))
}

fn configure_copilot_cli_proxy_safe() -> Result<(bool, String), String> {
    let shim = copilot_shim_path();
    let changed = install_copilot_shim()?;
    if let Err(err) = save_copilot_provider_config() {
        if changed {
            let _ = remove_copilot_shim();
        }
        return Err(err);
    }
    Ok((changed, shim))
}

pub fn configure_copilot_vscode_proxy() -> (bool, bool) {
    let _guard = proxy_op_mu().lock().ok();
    configure_copilot_vscode()
}

pub fn configure_copilot_proxy() -> (bool, String) {
    let _guard = proxy_op_mu().lock().ok();
    configure_copilot_cli_proxy_safe().unwrap_or_else(|_| (false, copilot_shim_path()))
}

pub fn configure_copilot_all_proxy() -> bool {
    let _guard = proxy_op_mu().lock().ok();
    let cli_required = copilot_cli_proxy_applicable();
    let vscode_required = copilot_vscode_proxy_applicable();
    let _ = configure_copilot_proxy_locked();
    (!cli_required || copilot_cli_proxy_wired())
        && (!vscode_required || copilot_vscode_proxy_wired())
}

fn configure_copilot_proxy_locked() -> (bool, String) {
    let snapshot = match snapshot_copilot_proxy_files() {
        Ok(s) => s,
        Err(err) => {
            L.err(&format!("copilot proxy snapshot: {err}"));
            return (false, copilot_shim_path());
        }
    };
    let mut changed = false;
    let mut file = copilot_shim_path();
    if copilot_cli_proxy_applicable() {
        match configure_copilot_cli_proxy_safe() {
            Ok((cli_changed, cli_file)) => {
                file = cli_file;
                changed = changed || cli_changed;
                if !cli_changed && !copilot_cli_proxy_wired() {
                    if let Err(rollback_err) = restore_copilot_proxy_files(&snapshot) {
                        L.err(&format!("copilot proxy rollback failed: {rollback_err}"));
                    }
                    return (false, file);
                }
            }
            Err(_) => {
                if let Err(rollback_err) = restore_copilot_proxy_files(&snapshot) {
                    L.err(&format!("copilot proxy rollback failed: {rollback_err}"));
                }
                return (false, file);
            }
        }
    }
    if copilot_vscode_proxy_applicable() {
        let (vs_changed, vs_ok) = configure_copilot_vscode();
        if !vs_ok {
            if let Err(err) = restore_copilot_proxy_files(&snapshot) {
                L.err(&format!("copilot proxy rollback failed: {err}"));
            }
            return (false, file);
        }
        changed = changed || vs_changed;
    }
    (changed || copilot_proxy_wired(), file)
}

pub fn copilot_cli_proxy_applicable() -> bool {
    copilot_cli_proxy_wired() || !paths::which("copilot").is_empty()
}

pub fn copilot_vscode_proxy_applicable() -> bool {
    copilot_vscode_proxy_wired() || is_vscode_extension_installed("github.copilot-chat")
}

#[derive(Debug, Clone)]
struct CopilotProxyFileSnapshot {
    path: String,
    raw: String,
    exists: bool,
    mode: u32,
    symlink: String,
}

#[derive(Debug, Clone, Default)]
pub struct CopilotProxySnapshot {
    files: Vec<CopilotProxyFileSnapshot>,
}

impl CopilotProxySnapshot {
    pub fn restore(&self) -> Result<(), String> {
        let _guard = proxy_op_mu().lock().map_err(|e| e.to_string())?;
        restore_copilot_proxy_files(&self.files)
    }
}

pub fn snapshot_copilot_proxy() -> Result<CopilotProxySnapshot, String> {
    let _guard = proxy_op_mu().lock().map_err(|e| e.to_string())?;
    snapshot_copilot_proxy_locked()
}

fn snapshot_copilot_proxy_locked() -> Result<CopilotProxySnapshot, String> {
    let files = snapshot_copilot_proxy_files()?;
    Ok(CopilotProxySnapshot { files })
}

fn snapshot_copilot_proxy_files() -> Result<Vec<CopilotProxyFileSnapshot>, String> {
    let paths_list = vec![
        copilot_shim_path(),
        copilot_shim_digest_path(),
        copilot_stashed_path(),
        copilot_stash_marker_path(),
        copilot_config_path(),
        copilot_vscode_settings_file(),
    ];
    let mut result = Vec::with_capacity(paths_list.len());
    for path in paths_list {
        let mut s = CopilotProxyFileSnapshot {
            path: path.clone(),
            raw: String::new(),
            exists: false,
            mode: 0o644,
            symlink: String::new(),
        };
        match fs::symlink_metadata(&path) {
            Ok(mut meta) => {
                if meta.file_type().is_symlink() {
                    if path != copilot_vscode_settings_file()
                        && path != copilot_ide_hooks_file("tokless-codegraph-index.json")
                    {
                        return Err(format!("refusing Copilot proxy snapshot path {path}"));
                    }
                    meta = fs::metadata(&path).map_err(|e| e.to_string())?;
                    s.symlink = fs::read_link(&path)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .into_owned();
                }
                if !meta.file_type().is_file() {
                    return Err(format!("refusing Copilot proxy snapshot path {path}"));
                }
                let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                s.raw = contents;
                s.exists = true;
                s.mode = file_mode_or(&path, 0o644).unwrap_or(0o644);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        result.push(s);
    }
    Ok(result)
}

fn restore_copilot_proxy_files(snapshots: &[CopilotProxyFileSnapshot]) -> Result<(), String> {
    let mut errs = Vec::new();
    for snapshot in snapshots {
        if snapshot.exists {
            if let Err(err) =
                reject_copilot_proxy_symlink_path(&snapshot.path, !snapshot.symlink.is_empty())
            {
                errs.push(err);
                continue;
            }
            if !snapshot.symlink.is_empty() {
                let link = fs::read_link(&snapshot.path)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned());
                if link.as_deref() != Some(snapshot.symlink.as_str()) {
                    errs.push(format!(
                        "Copilot proxy symlink changed during rollback {}",
                        snapshot.path
                    ));
                    continue;
                }
            } else if fs::symlink_metadata(&snapshot.path)
                .ok()
                .is_some_and(|m| m.file_type().is_symlink())
            {
                errs.push(format!(
                    "Copilot proxy path became symlink during rollback {}",
                    snapshot.path
                ));
                continue;
            }
            let target = if !snapshot.symlink.is_empty() {
                let p = Path::new(&snapshot.symlink);
                if p.is_absolute() {
                    snapshot.symlink.clone()
                } else {
                    Path::new(&snapshot.path)
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(p)
                        .to_string_lossy()
                        .into_owned()
                }
            } else {
                snapshot.path.clone()
            };
            if let Err(err) = write_copilot_file_mode(&target, &snapshot.raw, snapshot.mode) {
                errs.push(err);
            }
        } else if let Ok(meta) = fs::symlink_metadata(&snapshot.path) {
            if !meta.file_type().is_file() || !copilot_proxy_rollback_owned(&snapshot.path) {
                errs.push(format!(
                    "Copilot proxy path changed during rollback {}",
                    snapshot.path
                ));
                continue;
            }
            if let Err(err) = fs::remove_file(&snapshot.path) {
                errs.push(err.to_string());
            }
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.join("; "))
    }
}

fn copilot_proxy_rollback_owned(path: &str) -> bool {
    let Some(raw) = paths::read_file_safe(path) else {
        return false;
    };
    if path == copilot_shim_path() {
        return copilot_shim_owned(&raw);
    }
    if path == copilot_shim_digest_path() {
        return raw.trim() == copilot_stash_marker(&copilot_shim_text()).trim();
    }
    if path == copilot_stashed_path() {
        return paths::read_file_safe(&copilot_stash_marker_path())
            .is_some_and(|m| m.trim() == copilot_stash_marker(&raw).trim());
    }
    if path == copilot_stash_marker_path() {
        return paths::read_file_safe(&copilot_stashed_path())
            .is_some_and(|stashed| raw.trim() == copilot_stash_marker(&stashed).trim());
    }
    if path == copilot_config_path() {
        return serde_json::from_str::<CopilotProviderConfig>(&raw)
            .ok()
            .is_some_and(|cfg| cfg.managed_by == "tokless");
    }
    if path == copilot_vscode_settings_file() {
        return copilot_vscode_current_managed(&raw) || copilot_vscode_legacy_managed(&raw);
    }
    false
}

fn reject_copilot_proxy_symlink_path(path: &str, allow_leaf: bool) -> Result<(), String> {
    let clean = Path::new(path);
    let mut current = clean;
    loop {
        if let Ok(meta) = fs::symlink_metadata(current)
            && meta.file_type().is_symlink()
            && !(allow_leaf && current == clean)
        {
            return Err(format!(
                "refusing Copilot proxy rollback symlink {}",
                current.display()
            ));
        }
        let Some(parent) = current.parent() else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent;
    }
    Ok(())
}

pub fn remove_copilot_cli_proxy() -> bool {
    let _guard = proxy_op_mu().lock().ok();
    remove_copilot_shim()
}

pub fn remove_copilot_vscode_proxy() -> Result<bool, String> {
    let _guard = proxy_op_mu().lock().map_err(|e| e.to_string())?;
    remove_copilot_vscode()
}

pub fn remove_copilot_proxy() -> bool {
    let _guard = proxy_op_mu().lock().ok();
    remove_copilot_proxy_locked()
}

fn remove_copilot_proxy_locked() -> bool {
    let Ok(snapshot) = snapshot_copilot_proxy_locked() else {
        return false;
    };
    let cli_wired_before = copilot_cli_proxy_wired();
    let vscode_wired_before = copilot_vscode_proxy_wired();
    let removed_cli = remove_copilot_shim();
    let vscode_res = remove_copilot_vscode();
    let removed_vscode = vscode_res.as_ref().copied().unwrap_or(false);
    if vscode_res.is_err()
        || (cli_wired_before && (!removed_cli || copilot_cli_proxy_wired()))
        || (vscode_wired_before && (!removed_vscode || copilot_vscode_proxy_wired()))
    {
        if let Err(rollback_err) = restore_copilot_proxy_files(&snapshot.files) {
            L.err(&format!("copilot proxy rollback failed: {rollback_err}"));
        }
        return false;
    }
    if let Some(raw) = paths::read_file_safe(&copilot_config_path())
        && let Ok(cfg) = serde_json::from_str::<CopilotProviderConfig>(&raw)
        && cfg.managed_by == "tokless"
        && fs::remove_file(copilot_config_path()).is_err()
    {
        if let Err(rollback_err) = restore_copilot_proxy_files(&snapshot.files) {
            L.err(&format!("copilot proxy rollback failed: {rollback_err}"));
        }
        return false;
    }
    removed_cli || removed_vscode || cli_wired_before || vscode_wired_before
}

pub fn copilot_proxy_wired() -> bool {
    copilot_cli_proxy_wired() || copilot_vscode_proxy_wired()
}

pub fn copilot_cli_proxy_wired() -> bool {
    copilot_shim_installed()
}

pub fn copilot_vscode_proxy_wired() -> bool {
    paths::read_file_safe(&copilot_vscode_settings_file()).is_some_and(|settings| {
        copilot_vscode_current_managed(&settings) || copilot_vscode_legacy_managed(&settings)
    })
}

pub(crate) fn detect_copilot_proxy(cap: &ProxyCapability) -> ProxyDetection {
    let (cli_state, cli_detail) = detect_copilot_cli_proxy();
    let (vscode_state, vscode_detail) = detect_copilot_vscode_proxy();
    for (name, state, detail) in [
        ("CLI", cli_state, &cli_detail),
        ("VS Code", vscode_state, &vscode_detail),
    ] {
        if matches!(
            state,
            ProxyState::ProxyStateConflict
                | ProxyState::ProxyStateForeignBYOK
                | ProxyState::ProxyStateUnreadable
        ) {
            return proxy_detection(&cap.id, &format!("{name}: {detail}"), state);
        }
    }
    if cli_state == ProxyState::ProxyStateManaged || vscode_state == ProxyState::ProxyStateManaged {
        if cli_state == ProxyState::ProxyStateNativeUnconfigured
            || vscode_state == ProxyState::ProxyStateNativeUnconfigured
        {
            return proxy_detection(
                &cap.id,
                "one Copilot surface managed; another unconfigured",
                ProxyState::ProxyStateNativeUnconfigured,
            );
        }
        if cli_state == ProxyState::ProxyStateManaged && vscode_state == ProxyState::ProxyStateAbsent
        {
            return proxy_detection(&cap.id, &cli_detail, ProxyState::ProxyStateManaged);
        }
        if vscode_state == ProxyState::ProxyStateManaged && cli_state == ProxyState::ProxyStateAbsent
        {
            return proxy_detection(&cap.id, &vscode_detail, ProxyState::ProxyStateManaged);
        }
        return proxy_detection(
            &cap.id,
            "managed Copilot surfaces",
            ProxyState::ProxyStateManaged,
        );
    }
    if cli_state == ProxyState::ProxyStateAbsent && vscode_state == ProxyState::ProxyStateAbsent {
        return proxy_detection(
            &cap.id,
            "Copilot surfaces absent",
            ProxyState::ProxyStateAbsent,
        );
    }
    proxy_detection(
        &cap.id,
        "Copilot surfaces not configured",
        ProxyState::ProxyStateNativeUnconfigured,
    )
}

fn detect_copilot_cli_proxy() -> (ProxyState, String) {
    if copilot_cli_proxy_wired() {
        return (
            ProxyState::ProxyStateManaged,
            format!(
                "managed executable launcher; dedicated Headroom wrapper port {}",
                copilot_cli_proxy_port()
            ),
        );
    }
    if let Some(raw) = paths::read_file_safe(&copilot_config_path()) {
        let valid = serde_json::from_str::<CopilotProviderConfig>(&raw)
            .ok()
            .is_some_and(|cfg| cfg.managed_by == "tokless");
        if !valid {
            return (
                ProxyState::ProxyStateConflict,
                "provider metadata is foreign or unreadable".to_string(),
            );
        }
    }
    if !paths::which("copilot").is_empty() {
        return (
            ProxyState::ProxyStateNativeUnconfigured,
            "managed Copilot CLI launcher not configured".to_string(),
        );
    }
    (
        ProxyState::ProxyStateAbsent,
        "Copilot CLI absent".to_string(),
    )
}

fn detect_copilot_vscode_proxy() -> (ProxyState, String) {
    if copilot_vscode_proxy_wired() {
        return (
            ProxyState::ProxyStateManaged,
            format!(
                "managed VS Code settings; private Headroom wrapper port {}",
                copilot_proxy_port()
            ),
        );
    }
    if !is_vscode_extension_installed("github.copilot-chat") {
        return (
            ProxyState::ProxyStateAbsent,
            "Copilot Chat extension absent".to_string(),
        );
    }
    let text = match fs::read_to_string(copilot_vscode_settings_file()) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return (
                ProxyState::ProxyStateNativeUnconfigured,
                "VS Code settings not configured".to_string(),
            );
        }
        Err(_) => {
            return (
                ProxyState::ProxyStateUnreadable,
                "VS Code settings unreadable".to_string(),
            );
        }
    };
    if (text.contains(COPILOT_VSCODE_MARKER_HEAD) || text.contains(COPILOT_VSCODE_MARKER_FOOT))
        && !copilot_vscode_current_managed(&text)
        && !copilot_vscode_legacy_managed(&text)
    {
        return (
            ProxyState::ProxyStateConflict,
            "Tokless settings block differs".to_string(),
        );
    }
    for key in [
        COPILOT_VSCODE_PROXY_KEY,
        COPILOT_VSCODE_AUTH_KEY,
        COPILOT_VSCODE_CAPI_KEY,
    ] {
        if text.contains(&format!("\"{key}\"")) {
            return (
                ProxyState::ProxyStateConflict,
                "reserved setting differs".to_string(),
            );
        }
    }
    (
        ProxyState::ProxyStateNativeUnconfigured,
        "VS Code settings not configured".to_string(),
    )
}

fn configure_copilot_vscode() -> (bool, bool) {
    match configure_copilot_vscode_settings() {
        Ok(changed) => (changed, true),
        Err(err) => {
            L.sub(&format!("copilot VS Code settings: {err}"));
            (false, false)
        }
    }
}

fn configure_copilot_vscode_settings() -> Result<bool, String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    configure_copilot_vscode_settings_locked()
}

fn configure_copilot_vscode_settings_locked() -> Result<bool, String> {
    if !is_vscode_extension_installed("github.copilot-chat") {
        return Ok(false);
    }
    let path = copilot_vscode_settings_file();
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "{}\n".to_string(),
        Err(e) => return Err(e.to_string()),
    };
    if try_parse_jsonc(&raw).is_none() {
        return Err("invalid JSONC".to_string());
    }
    if raw.contains(COPILOT_VSCODE_MARKER_HEAD) || raw.contains(COPILOT_VSCODE_MARKER_FOOT) {
        if !copilot_vscode_current_managed(&raw) && !copilot_vscode_legacy_managed(&raw) {
            return Err("conflicting block".to_string());
        }
        let Some((start, end)) = copilot_vscode_span(&raw) else {
            return Err("invalid span".to_string());
        };
        let comma_added = raw[start..end].contains("(comma-added)");
        let next = format!(
            "{}{}{}",
            &raw[..start],
            copilot_vscode_block(comma_added),
            &raw[end..]
        );
        if next == raw {
            return Ok(false);
        }
        write_copilot_vscode_file_locked(&path, &next)?;
        return Ok(true);
    }
    if raw.contains(COPILOT_VSCODE_PROXY_KEY)
        || raw.contains(COPILOT_VSCODE_AUTH_KEY)
        || raw.contains(COPILOT_VSCODE_CAPI_KEY)
    {
        return Err("reserved key exists".to_string());
    }
    let Some(close) = jsonc_root_object_end(&raw) else {
        return Err("no root object end".to_string());
    };
    let insert = jsonc_last_token_end(&raw, close);
    let comma_added =
        insert > 0 && raw.as_bytes()[insert - 1] != b'{' && raw.as_bytes()[insert - 1] != b',';
    let separator = if comma_added { "," } else { "" };
    let line_sep = if raw.contains("\r\n") { "\r\n" } else { "\n" };
    let before = format!("{}{separator}{}", &raw[..insert], &raw[insert..close]);
    let newline = if !before.ends_with('\n') && !before.ends_with('\r') {
        line_sep
    } else {
        ""
    };
    let block_str = copilot_vscode_block(comma_added).replace('\n', line_sep);
    let next = format!("{before}{newline}{block_str}{line_sep}{}", &raw[close..]);
    write_copilot_vscode_file_locked(&path, &next)?;
    Ok(true)
}

fn remove_copilot_vscode() -> Result<bool, String> {
    let _guard = project_write_mu().lock().map_err(|e| e.to_string())?;
    remove_copilot_vscode_locked()
}

fn remove_copilot_vscode_locked() -> Result<bool, String> {
    let path = copilot_vscode_settings_file();
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    if !raw.contains(COPILOT_VSCODE_MARKER_HEAD) && !raw.contains(COPILOT_VSCODE_MARKER_FOOT) {
        return Ok(false);
    }
    if !copilot_vscode_current_managed(&raw) && !copilot_vscode_legacy_managed(&raw) {
        return Err("conflicting block".to_string());
    }
    let Some((start, end)) = copilot_vscode_span(&raw) else {
        return Err("invalid span".to_string());
    };
    let mut prefix = raw[..start].to_string();
    let suffix = &raw[end..];
    if raw[start..end].contains("(comma-added)")
        && let Some(comma) = jsonc_last_comma(&prefix)
    {
        prefix = format!("{}{}", &prefix[..comma], &prefix[comma + 1..]);
    }
    let next = format!("{prefix}{suffix}");
    if try_parse_jsonc(&next).is_none() {
        return Err("invalid resulting JSONC".to_string());
    }
    write_copilot_vscode_file_locked(&path, &next)?;
    Ok(true)
}
