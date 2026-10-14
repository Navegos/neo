// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/cursor.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use crate::core::Detection;
use crate::util::exec::{RunOptions, find_binary, run};
use crate::util::jsonc::{OrderedMap, has_jsonc_comments, stringify_json, try_parse_jsonc};
use crate::util::mcpspawn::{McpSpawn, mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths::{
    IS_WIN, cursor_global_mcp_path, cursor_paths_resolved, exists, home, read_file_safe,
    split_command, tokless_abs, write_file,
};
use crate::util::preflight::windows_cursor_home_from_wsl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorConfigTarget {
    pub dir: String,
    pub bridge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorMcpTarget {
    pub path: String,
    pub bridge: bool,
}

#[derive(Debug, Clone)]
pub struct CursorProjectRulesHookSnapshot {
    files: Vec<CursorFileSnapshot>,
}

#[derive(Debug, Clone)]
struct CursorFileSnapshot {
    path: String,
    raw: String,
    exists: bool,
}

fn cursor_mcp_entry_for(tool_id: &str, windows_bridge: bool) -> Value {
    let mut spawn = mcp_spawn_for(tool_id);
    if tool_id == "codegraph" {
        spawn = wrap_auto_index("cursor", pick_mcp_spawn("codegraph", &["serve", "--mcp"]));
        let mut args = vec![
            "run-mcp".to_string(),
            "--agent".to_string(),
            "cursor".to_string(),
            "--workspace".to_string(),
            "${workspaceFolder}".to_string(),
        ];
        if spawn.args.len() >= 3 {
            args.extend_from_slice(&spawn.args[3..]);
        }
        spawn.args = args;
    }
    if windows_bridge {
        let mut args = vec!["--".to_string()];
        if tool_id == "codegraph" {
            let mut filtered = Vec::with_capacity(spawn.args.len());
            let mut i = 0;
            while i < spawn.args.len() {
                if spawn.args[i] == "--workspace"
                    && i + 1 < spawn.args.len()
                    && spawn.args[i + 1] == "${workspaceFolder}"
                {
                    i += 2;
                    continue;
                }
                filtered.push(spawn.args[i].clone());
                i += 1;
            }
            spawn.args = filtered;
        }
        if let Ok(distro) = std::env::var("WSL_DISTRO_NAME") {
            if !distro.is_empty() {
                args = vec!["-d".to_string(), distro, "--".to_string()];
            }
        }
        args.push(spawn.command);
        args.extend(spawn.args);
        spawn = McpSpawn {
            command: "wsl.exe".to_string(),
            args,
        };
    }
    json!({
        "type": "stdio",
        "command": spawn.command,
        "args": spawn.args,
    })
}

fn cursor_mcp_path() -> String {
    cursor_global_mcp_path()
}

pub fn cursor_config_targets() -> Vec<CursorConfigTarget> {
    let mut targets = vec![CursorConfigTarget {
        dir: cursor_paths_resolved().dir,
        bridge: false,
    }];
    if std::env::var("TOKLESS_CURSOR_WINDOWS_BRIDGE").unwrap_or_default() == "1" {
        let win_home = windows_cursor_home_from_wsl();
        if !win_home.is_empty() {
            targets.push(CursorConfigTarget {
                dir: Path::new(&win_home)
                    .join(".cursor")
                    .to_string_lossy()
                    .to_string(),
                bridge: true,
            });
        }
    }
    targets
}

pub fn cursor_mcp_targets() -> Vec<CursorMcpTarget> {
    cursor_config_targets()
        .into_iter()
        .map(|target| CursorMcpTarget {
            path: Path::new(&target.dir)
                .join("mcp.json")
                .to_string_lossy()
                .to_string(),
            bridge: target.bridge,
        })
        .collect()
}

pub fn cursor_mcp_config_path() -> String {
    cursor_mcp_path()
}

fn cursor_entry_has_exact_fields(a: &OrderedMap, want: &OrderedMap) -> bool {
    if a.len() != want.len() {
        return false;
    }
    want.keys().all(|k| a.contains_key(k))
}

fn any_string_slice_equal(val: Option<&Value>, want: &[String]) -> bool {
    let Some(Value::Array(arr)) = val else {
        return false;
    };
    if arr.len() != want.len() {
        return false;
    }
    arr.iter()
        .zip(want.iter())
        .all(|(v, w)| v.as_str() == Some(w.as_str()))
}

fn cursor_entry_matches(v: &Value, want: &Value) -> bool {
    let (Some(a), Some(want_map)) = (v.as_object(), want.as_object()) else {
        return false;
    };
    if !cursor_entry_has_exact_fields(a, want_map) {
        return false;
    }
    let at = a.get("type");
    let wt = want_map.get("type");
    let ac = a.get("command");
    let wc = want_map.get("command");
    let Some(Value::Array(wa_arr)) = want_map.get("args") else {
        return false;
    };
    let mut want_args = Vec::with_capacity(wa_arr.len());
    for item in wa_arr {
        let Some(s) = item.as_str() else {
            return false;
        };
        want_args.push(s.to_string());
    }
    at.is_some()
        && at == wt
        && ac.is_some()
        && ac == wc
        && any_string_slice_equal(a.get("args"), &want_args)
}

fn cursor_stale_tokless_entry_matches(v: &Value, want: &Value) -> bool {
    let (Some(a), Some(want_map)) = (v.as_object(), want.as_object()) else {
        return false;
    };
    if !cursor_entry_has_exact_fields(a, want_map) {
        return false;
    }
    let at = a.get("type").and_then(|x| x.as_str());
    let wt = want_map.get("type").and_then(|x| x.as_str());
    if at != Some("stdio") || wt != Some("stdio") {
        return false;
    }
    let (Some(old_command), Some(want_command)) = (
        a.get("command").and_then(|x| x.as_str()),
        want_map.get("command").and_then(|x| x.as_str()),
    ) else {
        return false;
    };
    let (Some(old_args), Some(want_args)) = (
        a.get("args").and_then(|x| x.as_array()),
        want_map.get("args").and_then(|x| x.as_array()),
    ) else {
        return false;
    };
    if cursor_tokless_codegraph_args(old_command, old_args)
        && cursor_tokless_codegraph_args(want_command, want_args)
    {
        return true;
    }
    if want_command == "wsl.exe" {
        if old_command != "wsl.exe" || old_args.len() != want_args.len() {
            return false;
        }
        let Some(stale_index) = tokless_arg_index(want_args) else {
            return false;
        };
        for i in 0..want_args.len() {
            if old_args[i] == want_args[i] {
                continue;
            }
            if i == stale_index
                && cursor_is_tokless_command_val(&old_args[i])
                && cursor_is_tokless_command_val(&want_args[i])
            {
                continue;
            }
            return false;
        }
        return true;
    }
    cursor_is_tokless_command_str(old_command)
        && cursor_is_tokless_command_str(want_command)
        && any_string_slice_equal(a.get("args"), &string_args(want_args))
}

fn cursor_tokless_codegraph_args<'a>(mut command: &'a str, mut args: &'a [Value]) -> bool {
    if command == "wsl.exe" {
        let Some(pos) = args.iter().position(|a| a.as_str() == Some("--")) else {
            return false;
        };
        let start = pos + 1;
        if start >= args.len() {
            return false;
        }
        command = args[start].as_str().unwrap_or("");
        args = &args[start + 1..];
    }
    if !cursor_is_tokless_command_str(command)
        || args.len() < 6
        || args[0].as_str() != Some("run-mcp")
        || args[1].as_str() != Some("--agent")
        || args[2].as_str() != Some("cursor")
    {
        return false;
    }
    let mut target = 3;
    if args[target].as_str() == Some("--workspace") {
        if args.len() < target + 3 {
            return false;
        }
        target += 2;
    }
    if target + 2 < args.len()
        && cursor_legacy_codegraph_command(args[target].as_str().unwrap_or(""))
        && args[target + 1].as_str() == Some("serve")
        && args[target + 2].as_str() == Some("--mcp")
    {
        return true;
    }
    cursor_npx_codegraph_args(&args[target..])
}

fn cursor_npx_codegraph_args(args: &[Value]) -> bool {
    if args.len() != 5 || !cursor_npx_command(args[0].as_str().unwrap_or("")) {
        return false;
    }
    args[1].as_str() == Some("--no-install")
        && args[2].as_str() == Some("@colbymchenry/codegraph")
        && args[3].as_str() == Some("serve")
        && args[4].as_str() == Some("--mcp")
}

fn tokless_arg_index(args: &[Value]) -> Option<usize> {
    for (i, arg) in args.iter().enumerate() {
        if arg.as_str() == Some("--") && i + 1 < args.len() {
            return Some(i + 1);
        }
    }
    None
}

fn cursor_is_tokless_command_val(command: &Value) -> bool {
    command
        .as_str()
        .is_some_and(cursor_is_tokless_command_str)
}

fn cursor_is_tokless_command_str(s: &str) -> bool {
    let normalized = s.replace('\\', "/");
    let base = Path::new(&normalized)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        base.as_str(),
        "tokless"
            | "tokless.exe"
            | "tokless-linux"
            | "tokless-linux-x64"
            | "tokless-linux-arm64"
            | "tokless-darwin"
            | "tokless-darwin-x64"
            | "tokless-darwin-arm64"
            | "tokless-windows"
            | "tokless-windows-x64.exe"
    ) || base.starts_with("tokless")
}

fn string_args(args: &[Value]) -> Vec<String> {
    args.iter()
        .map(|a| a.as_str().unwrap_or("").to_string())
        .collect()
}

fn cursor_legacy_entry_matches(v: &Value, want: &Value) -> bool {
    let (Some(a), Some(want_map)) = (v.as_object(), want.as_object()) else {
        return false;
    };
    if a.len() != 3 {
        return false;
    }
    let t = a.get("type").and_then(|x| x.as_str());
    let command_name = a.get("command").and_then(|x| x.as_str());
    let legacy_args = a.get("args").and_then(|x| x.as_array());
    let (Some("stdio"), Some(cmd), Some(l_args)) = (t, command_name, legacy_args) else {
        return false;
    };
    if !cursor_legacy_codegraph_command(cmd) || (l_args.len() != 2 && l_args.len() != 4) {
        return false;
    }
    if l_args[0].as_str() != Some("serve") || l_args[1].as_str() != Some("--mcp") {
        return false;
    }
    if l_args.len() == 4
        && (l_args[2].as_str() != Some("--path")
            || l_args[3].as_str() != Some("${workspaceFolder}"))
    {
        return false;
    }
    let Some(want_command) = want_map.get("command").and_then(|x| x.as_str()) else {
        return false;
    };
    let Some(mut want_args) = want_map.get("args").and_then(|x| x.as_array()).map(|a| a.as_slice())
    else {
        return false;
    };
    if want_command == "wsl.exe" {
        if let Some(pos) = want_args.iter().position(|a| a.as_str() == Some("--")) {
            if pos + 1 < want_args.len() {
                want_args = &want_args[pos + 1..];
            }
        }
    }
    for i in 0..want_args.len().saturating_sub(2) {
        if want_args[i].as_str() == Some("run-mcp")
            && want_args[i + 1].as_str() == Some("--agent")
            && want_args[i + 2].as_str() == Some("cursor")
        {
            return true;
        }
    }
    false
}

fn cursor_legacy_codegraph_command(command: &str) -> bool {
    let normalized = command.replace('\\', "/");
    let base = Path::new(&normalized)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        base.as_str(),
        "codegraph" | "codegraph.exe" | "codegraph.cmd" | "codegraph.bat"
    )
}

fn cursor_npx_command(command: &str) -> bool {
    let normalized = command.replace('\\', "/");
    let base = Path::new(&normalized)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(base.as_str(), "npx" | "npx.exe" | "npx.cmd" | "npx.bat")
}

pub fn configure_cursor_mcp(tool_id: &str) -> (bool, String) {
    let targets = cursor_mcp_targets();
    struct Prepared {
        path: String,
        before: String,
        after: String,
        changed: bool,
        exists: bool,
    }
    let mut prepared_files = Vec::with_capacity(targets.len());
    for target in targets {
        let p = target.path;
        let raw_opt = read_file_safe(&p);
        let file_exists = raw_opt.is_some();
        let raw = raw_opt.unwrap_or_default();
        if file_exists && has_jsonc_comments(&raw) {
            return (false, p);
        }
        let mut cfg = match try_parse_jsonc(&raw) {
            Some(v) if v.is_object() => v,
            _ => {
                if file_exists && !raw.trim().is_empty() {
                    return (false, p);
                }
                Value::Object(OrderedMap::new())
            }
        };
        let root = cfg.as_object_mut().unwrap();
        if !root.contains_key("mcpServers") {
            root.insert("mcpServers".to_string(), Value::Object(OrderedMap::new()));
        }
        let Some(servers) = root.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
            return (false, p);
        };
        let desired = cursor_mcp_entry_for(tool_id, target.bridge);
        if let Some(existing) = servers.get(tool_id) {
            if !cursor_entry_matches(existing, &desired)
                && !cursor_stale_tokless_entry_matches(existing, &desired)
                && !cursor_legacy_entry_matches(existing, &desired)
            {
                return (false, p);
            }
            if cursor_entry_matches(existing, &desired) {
                prepared_files.push(Prepared {
                    path: p,
                    before: raw,
                    after: String::new(),
                    changed: false,
                    exists: file_exists,
                });
                continue;
            }
        }
        servers.insert(tool_id.to_string(), desired);
        prepared_files.push(Prepared {
            path: p,
            before: raw,
            after: stringify_json(&cfg),
            changed: true,
            exists: file_exists,
        });
    }
    let mut changed = false;
    for i in 0..prepared_files.len() {
        if !prepared_files[i].changed {
            continue;
        }
        if write_file(&prepared_files[i].path, &prepared_files[i].after).is_err() {
            for prior in &prepared_files[..i] {
                if prior.changed {
                    if prior.exists {
                        let _ = write_file(&prior.path, &prior.before);
                    } else {
                        let _ = fs::remove_file(&prior.path);
                    }
                }
            }
            return (false, prepared_files[i].path.clone());
        }
        changed = true;
    }
    (changed, cursor_mcp_path())
}

pub fn remove_cursor_mcp(tool_id: &str) -> bool {
    struct Prepared {
        path: String,
        before: String,
        after: String,
        exists: bool,
    }
    let mut files = Vec::new();
    for target in cursor_mcp_targets() {
        let Some(raw) = read_file_safe(&target.path) else {
            continue;
        };
        if has_jsonc_comments(&raw) {
            return false;
        }
        let Some(mut cfg) = try_parse_jsonc(&raw) else {
            return false;
        };
        let Some(root) = cfg.as_object_mut() else {
            return false;
        };
        let Some(servers) = root.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
            continue;
        };
        let Some(v) = servers.get(tool_id) else {
            continue;
        };
        if !cursor_entry_matches(v, &cursor_mcp_entry_for(tool_id, target.bridge)) {
            return false;
        }
        servers.remove(tool_id);
        if servers.is_empty() {
            root.remove("mcpServers");
        }
        files.push(Prepared {
            path: target.path,
            before: raw,
            after: stringify_json(&cfg),
            exists: true,
        });
    }
    if files.is_empty() {
        return false;
    }
    for i in 0..files.len() {
        if write_file(&files[i].path, &files[i].after).is_err() {
            for prior in &files[..i] {
                if prior.exists {
                    let _ = write_file(&prior.path, &prior.before);
                } else {
                    let _ = fs::remove_file(&prior.path);
                }
            }
            return false;
        }
    }
    true
}

pub fn cursor_mcp_has(tool_id: &str) -> bool {
    for target in cursor_mcp_targets() {
        let Some(raw) = read_file_safe(&target.path) else {
            return false;
        };
        let Some(cfg) = try_parse_jsonc(&raw) else {
            return false;
        };
        let Some(servers) = cfg
            .as_object()
            .and_then(|m| m.get("mcpServers"))
            .and_then(|v| v.as_object())
        else {
            return false;
        };
        let Some(v) = servers.get(tool_id) else {
            return false;
        };
        if !cursor_entry_matches(v, &cursor_mcp_entry_for(tool_id, target.bridge)) {
            return false;
        }
    }
    true
}

pub fn cursor_config_dir() -> String {
    cursor_paths_resolved().dir
}

fn cursor_ide_config_dirs() -> Vec<String> {
    cursor_config_targets().into_iter().map(|t| t.dir).collect()
}

fn cursor_target_bridge(path: &str) -> bool {
    let p = Path::new(path);
    for target in cursor_config_targets() {
        let dir = Path::new(&target.dir);
        if p == dir || p.starts_with(dir) {
            return target.bridge;
        }
    }
    false
}

fn cursor_hooks_files() -> Vec<String> {
    cursor_ide_config_dirs()
        .into_iter()
        .map(|dir| Path::new(&dir).join("hooks.json").to_string_lossy().to_string())
        .collect()
}

fn cursor_permissions_files() -> Vec<String> {
    cursor_ide_config_dirs()
        .into_iter()
        .map(|dir| {
            Path::new(&dir)
                .join("permissions.json")
                .to_string_lossy()
                .to_string()
        })
        .collect()
}

fn cursor_native_cli_config_file() -> String {
    if IS_WIN || std::env::consts::OS == "macos" {
        return Path::new(&home())
            .join(".cursor")
            .join("cli-config.json")
            .to_string_lossy()
            .to_string();
    }
    if let Ok(dir) = std::env::var("CURSOR_CONFIG_DIR") {
        if !dir.is_empty() {
            return Path::new(&dir)
                .join("cli-config.json")
                .to_string_lossy()
                .to_string();
        }
    }
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return Path::new(&dir)
                .join("cursor")
                .join("cli-config.json")
                .to_string_lossy()
                .to_string();
        }
    }
    Path::new(&home())
        .join(".cursor")
        .join("cli-config.json")
        .to_string_lossy()
        .to_string()
}

fn cursor_cli_config_files() -> Vec<String> {
    vec![cursor_native_cli_config_file()]
}

fn cursor_windows_command_arg(arg: &str) -> String {
    format!("\"{}\"", arg.replace('"', "\\\""))
}

fn cursor_posix_command_arg(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', "'\\''"))
}

fn cursor_rtk_hook_command_for(bridge: bool) -> String {
    if bridge {
        let mut args = vec!["--".to_string()];
        if let Ok(distro) = std::env::var("WSL_DISTRO_NAME") {
            if !distro.is_empty() {
                args = vec![
                    "-d".to_string(),
                    cursor_windows_command_arg(&distro),
                    "--".to_string(),
                ];
            }
        }
        args.push(cursor_windows_command_arg(&tokless_abs()));
        args.push("rtk".to_string());
        args.push("hook".to_string());
        args.push("cursor".to_string());
        let mut full = vec!["wsl.exe".to_string()];
        full.extend(args);
        return full.join(" ");
    }
    "rtk hook cursor".to_string()
}

fn cursor_codegraph_hook_command_for(bridge: bool) -> String {
    let args = ["cursor-hook", "codegraph-index"];
    if bridge {
        let mut wsl = vec!["--".to_string()];
        if let Ok(distro) = std::env::var("WSL_DISTRO_NAME") {
            if !distro.is_empty() {
                wsl = vec![
                    "-d".to_string(),
                    cursor_windows_command_arg(&distro),
                    "--".to_string(),
                ];
            }
        }
        let mut full = vec!["wsl.exe".to_string()];
        full.extend(wsl);
        full.push(cursor_windows_command_arg(&tokless_abs()));
        full.extend(args.iter().map(|s| s.to_string()));
        return full.join(" ");
    }
    let path = tokless_abs();
    let quoted = if IS_WIN {
        cursor_windows_command_arg(&path)
    } else {
        cursor_posix_command_arg(&path)
    };
    format!("{} {}", quoted, args.join(" "))
}

fn cursor_project_rules_hook_command_for(bridge: bool) -> String {
    let args = ["cursor-hook", "project-rules"];
    if bridge {
        let mut wsl = vec!["--".to_string()];
        if let Ok(distro) = std::env::var("WSL_DISTRO_NAME") {
            if !distro.is_empty() {
                wsl = vec![
                    "-d".to_string(),
                    cursor_windows_command_arg(&distro),
                    "--".to_string(),
                ];
            }
        }
        let mut full = vec!["wsl.exe".to_string()];
        full.extend(wsl);
        full.push(cursor_windows_command_arg(&tokless_abs()));
        full.extend(args.iter().map(|s| s.to_string()));
        return full.join(" ");
    }
    let path = tokless_abs();
    let quoted = if IS_WIN {
        cursor_windows_command_arg(&path)
    } else {
        cursor_posix_command_arg(&path)
    };
    format!("{} {}", quoted, args.join(" "))
}

fn cursor_owned_codegraph_hook_entry_for(v: &Value, want: &str) -> bool {
    let Some(e) = v.as_object() else {
        return false;
    };
    if e.len() != 1 {
        return false;
    }
    e.get("command").and_then(|c| c.as_str()) == Some(want)
}

pub fn has_cursor_codegraph_index_hook() -> bool {
    for path in cursor_hooks_files() {
        let want = cursor_codegraph_hook_command_for(cursor_target_bridge(&path));
        if !has_cursor_codegraph_index_hook_at(&path, &want) {
            return false;
        }
    }
    true
}

fn has_cursor_codegraph_index_hook_at(path: &str, want: &str) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(hooks) = cfg
        .as_object()
        .and_then(|m| m.get("hooks"))
        .and_then(|v| v.as_object())
    else {
        return false;
    };
    for event in ["sessionStart", "workspaceOpen"] {
        let Some(entries) = hooks.get(event).and_then(|v| v.as_array()) else {
            return false;
        };
        let count = entries
            .iter()
            .filter(|entry| cursor_owned_codegraph_hook_entry_for(entry, want))
            .count();
        if count != 1 {
            return false;
        }
    }
    true
}

fn cursor_version_one(v: &Value) -> bool {
    v.as_i64() == Some(1) || v.as_u64() == Some(1) || v.as_f64() == Some(1.0)
}

fn install_cursor_codegraph_index_hook_at(path: &str) -> bool {
    let command = cursor_codegraph_hook_command_for(cursor_target_bridge(path));
    let raw_opt = read_file_safe(path);
    let file_exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    if file_exists && !raw.trim().is_empty() && has_jsonc_comments(&raw) {
        return false;
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if file_exists && !raw.trim().is_empty() {
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root = cfg.as_object_mut().unwrap();
    if let Some(version) = root.get("version") {
        if !cursor_version_one(version) {
            return false;
        }
    }
    if !root.contains_key("hooks") {
        root.insert("hooks".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return false;
    };
    for event in ["sessionStart", "workspaceOpen"] {
        let entries = match hooks.get(event) {
            Some(Value::Array(arr)) => arr.clone(),
            Some(_) => return false,
            None => Vec::new(),
        };
        let mut kept: Vec<Value> = entries
            .into_iter()
            .filter(|entry| !cursor_owned_codegraph_hook_entry_for(entry, &command))
            .collect();
        kept.push(json!({ "command": command }));
        hooks.insert(event.to_string(), Value::Array(kept));
    }
    root.insert("version".to_string(), json!(1));
    let next = stringify_json(&cfg);
    next == raw || write_file(path, &next).is_ok()
}

pub fn install_cursor_codegraph_index_hook() -> bool {
    let paths = cursor_hooks_files();
    let snapshots = cursor_file_snapshots(&paths);
    for (i, path) in paths.iter().enumerate() {
        if !install_cursor_codegraph_index_hook_at(path) {
            let _ = restore_cursor_files(&snapshots[..i]);
            return false;
        }
    }
    true
}

fn remove_cursor_codegraph_index_hook_at(path: &str) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return true;
    };
    if !raw.trim().is_empty() && has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(root) = cfg.as_object_mut() else {
        return false;
    };
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return true;
    };
    let want = cursor_codegraph_hook_command_for(cursor_target_bridge(path));
    let mut changed = false;
    for event in ["sessionStart", "workspaceOpen"] {
        let Some(v) = hooks.get(event) else {
            continue;
        };
        let Some(entries) = v.as_array().cloned() else {
            return false;
        };
        let mut kept = Vec::with_capacity(entries.len());
        for entry in entries {
            if !cursor_owned_codegraph_hook_entry_for(&entry, &want) {
                kept.push(entry);
            } else {
                changed = true;
            }
        }
        if kept.is_empty() {
            hooks.remove(event);
        } else {
            hooks.insert(event.to_string(), Value::Array(kept));
        }
    }
    if hooks.is_empty() {
        root.remove("hooks");
    }
    if !changed {
        return true;
    }
    write_file(path, &stringify_json(&cfg)).is_ok()
}

pub fn remove_cursor_codegraph_index_hook() -> bool {
    let paths = cursor_hooks_files();
    let snapshots = cursor_file_snapshots(&paths);
    for (i, path) in paths.iter().enumerate() {
        if !remove_cursor_codegraph_index_hook_at(path) {
            let _ = restore_cursor_files(&snapshots[..i]);
            return false;
        }
    }
    true
}

fn cursor_owned_project_rules_hook_entry_for(v: &Value, want: &str) -> bool {
    let Some(e) = v.as_object() else {
        return false;
    };
    if e.len() != 1 {
        return false;
    }
    e.get("command").and_then(|c| c.as_str()) == Some(want)
}

pub fn has_cursor_project_rules_hook() -> bool {
    for path in cursor_hooks_files() {
        let want = cursor_project_rules_hook_command_for(cursor_target_bridge(&path));
        if !has_cursor_project_rules_hook_at(&path, &want) {
            return false;
        }
    }
    true
}

fn has_cursor_project_rules_hook_at(path: &str, want: &str) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(hooks) = cfg
        .as_object()
        .and_then(|m| m.get("hooks"))
        .and_then(|v| v.as_object())
    else {
        return false;
    };
    for event in ["sessionStart", "workspaceOpen"] {
        let Some(entries) = hooks.get(event).and_then(|v| v.as_array()) else {
            return false;
        };
        let count = entries
            .iter()
            .filter(|entry| cursor_owned_project_rules_hook_entry_for(entry, want))
            .count();
        if count != 1 {
            return false;
        }
    }
    true
}

fn install_cursor_project_rules_hook_at(path: &str) -> bool {
    let raw_opt = read_file_safe(path);
    let file_exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    if file_exists && !raw.trim().is_empty() && has_jsonc_comments(&raw) {
        return false;
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if file_exists && !raw.trim().is_empty() {
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root = cfg.as_object_mut().unwrap();
    if !root.contains_key("hooks") {
        root.insert("hooks".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return false;
    };
    let command = cursor_project_rules_hook_command_for(cursor_target_bridge(path));
    for event in ["sessionStart", "workspaceOpen"] {
        let entries = match hooks.get(event) {
            Some(Value::Array(arr)) => arr.clone(),
            Some(_) => return false,
            None => Vec::new(),
        };
        let mut kept: Vec<Value> = entries
            .into_iter()
            .filter(|entry| !cursor_owned_project_rules_hook_entry_for(entry, &command))
            .collect();
        kept.push(json!({ "command": command }));
        hooks.insert(event.to_string(), Value::Array(kept));
    }
    root.insert("version".to_string(), json!(1));
    let next = stringify_json(&cfg);
    next == raw || write_file(path, &next).is_ok()
}

pub fn install_cursor_project_rules_hook() -> bool {
    let paths = cursor_hooks_files();
    let snapshots = cursor_file_snapshots(&paths);
    for (i, path) in paths.iter().enumerate() {
        if !install_cursor_project_rules_hook_at(path) {
            let _ = restore_cursor_files(&snapshots[..i]);
            return false;
        }
    }
    true
}

pub fn snapshot_cursor_project_rules_hook() -> std::io::Result<CursorProjectRulesHookSnapshot> {
    let mut files = Vec::new();
    for path in cursor_hooks_files() {
        if let Some(raw) = read_file_safe(&path) {
            files.push(CursorFileSnapshot {
                path,
                raw,
                exists: true,
            });
        } else {
            if let Err(err) = fs::metadata(&path) {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(err);
                }
            }
            files.push(CursorFileSnapshot {
                path,
                raw: String::new(),
                exists: false,
            });
        }
    }
    Ok(CursorProjectRulesHookSnapshot { files })
}

impl CursorProjectRulesHookSnapshot {
    pub fn restore(&self) -> std::io::Result<()> {
        for file in &self.files {
            if file.exists {
                write_file(&file.path, &file.raw)?;
                continue;
            }
            let want = cursor_project_rules_hook_command_for(cursor_target_bridge(&file.path));
            if read_file_safe(&file.path).is_none()
                || !has_cursor_project_rules_hook_at(&file.path, &want)
            {
                continue;
            }
            if !remove_cursor_project_rules_hook_at(&file.path) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("restore Cursor project-rules hook {}", file.path),
                ));
            }
            if let Some(current) = read_file_safe(&file.path) {
                if cursor_project_rules_hook_scaffold(&current) {
                    if let Err(err) = fs::remove_file(&file.path) {
                        if err.kind() != std::io::ErrorKind::NotFound {
                            return Err(err);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn cursor_project_rules_hook_scaffold(raw: &str) -> bool {
    let Some(cfg) = try_parse_jsonc(raw) else {
        return false;
    };
    let Some(obj) = cfg.as_object() else {
        return false;
    };
    if obj.len() != 1 {
        return false;
    }
    obj.get("version").is_some_and(cursor_version_one)
}

fn remove_cursor_project_rules_hook_at(path: &str) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return true;
    };
    if !raw.trim().is_empty() && has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(root) = cfg.as_object_mut() else {
        return false;
    };
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return true;
    };
    let command = cursor_project_rules_hook_command_for(cursor_target_bridge(path));
    let mut changed = false;
    for event in ["sessionStart", "workspaceOpen"] {
        let Some(v) = hooks.get(event) else {
            continue;
        };
        let Some(entries) = v.as_array().cloned() else {
            return false;
        };
        let mut kept = Vec::with_capacity(entries.len());
        for entry in entries {
            if cursor_owned_project_rules_hook_entry_for(&entry, &command) {
                changed = true;
            } else {
                kept.push(entry);
            }
        }
        if kept.is_empty() {
            hooks.remove(event);
        } else {
            hooks.insert(event.to_string(), Value::Array(kept));
        }
    }
    if hooks.is_empty() {
        root.remove("hooks");
    }
    if !changed {
        return true;
    }
    write_file(path, &stringify_json(&cfg)).is_ok()
}

pub fn remove_cursor_project_rules_hook() -> bool {
    let paths = cursor_hooks_files();
    let snapshots = cursor_file_snapshots(&paths);
    for (i, path) in paths.iter().enumerate() {
        if !remove_cursor_project_rules_hook_at(path) {
            let _ = restore_cursor_files(&snapshots[..i]);
            return false;
        }
    }
    true
}

fn cursor_rtk_hook_entry_for(command: &str) -> Value {
    json!({
        "command": command,
        "matcher": "Shell",
    })
}

fn cursor_rtk_hook_entry_matches_for(v: &Value, want: &str) -> bool {
    let Some(e) = v.as_object() else {
        return false;
    };
    if e.len() != 2 {
        return false;
    }
    e.get("command").and_then(|c| c.as_str()) == Some(want)
        && e.get("matcher").and_then(|m| m.as_str()) == Some("Shell")
}

fn cursor_tokless_launcher(token: &str) -> bool {
    let mut base = token.trim_matches(['"', '\'']);
    if let Some(i) = base.rfind(['/', '\\']) {
        base = &base[i + 1..];
    }
    let lower = base.to_lowercase();
    lower == "tokless" || lower == "tokless.exe"
}

fn cursor_owned_rtk_hook_command(command: &str) -> bool {
    if command == "rtk hook cursor" {
        return true;
    }
    let parts = split_command(command);
    if parts.len() == 3
        && cursor_tokless_launcher(&parts[0])
        && parts[1] == "rtk-hook"
        && parts[2] == "cursor"
    {
        return true;
    }
    if parts.len() == 4
        && cursor_tokless_launcher(&parts[0])
        && parts[1] == "rtk"
        && parts[2] == "hook"
        && parts[3] == "cursor"
    {
        return true;
    }
    let first_base = Path::new(&parts.first().map(|s| s.replace('\\', "/")).unwrap_or_default())
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    if parts.len() < 5 || first_base != "wsl.exe" {
        return false;
    }
    let Some(separator) = parts.iter().skip(1).position(|p| p == "--").map(|i| i + 1) else {
        return false;
    };
    separator >= 1
        && parts.len() == separator + 5
        && cursor_tokless_launcher(&parts[separator + 1])
        && parts[separator + 2] == "rtk"
        && parts[separator + 3] == "hook"
        && parts[separator + 4] == "cursor"
}

fn cursor_owned_rtk_hook_entry_matches(v: &Value) -> bool {
    let Some(e) = v.as_object() else {
        return false;
    };
    if e.len() != 2 {
        return false;
    }
    let Some(cmd) = e.get("command").and_then(|c| c.as_str()) else {
        return false;
    };
    cursor_owned_rtk_hook_command(cmd)
        && e.get("matcher").and_then(|m| m.as_str()) == Some("Shell")
}

fn install_cursor_rtk_hook_at(path: &str) -> bool {
    let command = cursor_rtk_hook_command_for(cursor_target_bridge(path));
    let raw_opt = read_file_safe(path);
    let file_exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    if file_exists && !raw.trim().is_empty() && has_jsonc_comments(&raw) {
        return false;
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if file_exists && !raw.trim().is_empty() {
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root = cfg.as_object_mut().unwrap();
    if let Some(version) = root.get("version") {
        if !cursor_version_one(version) {
            return false;
        }
    }
    if !root.contains_key("hooks") {
        root.insert("hooks".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return false;
    };
    let entries = match hooks.get("preToolUse") {
        Some(Value::Array(arr)) => arr.clone(),
        Some(_) => return false,
        None => Vec::new(),
    };
    let mut kept = Vec::with_capacity(entries.len() + 1);
    let current = cursor_rtk_hook_entry_for(&command);
    let mut current_count = 0;
    for entry in &entries {
        if cursor_rtk_hook_entry_matches_for(entry, &command) {
            current_count += 1;
            continue;
        }
        if cursor_owned_rtk_hook_entry_matches(entry) {
            continue;
        }
        kept.push(entry.clone());
    }
    if current_count == 1 && kept.len() + 1 == entries.len() {
        return true;
    }
    kept.push(current);
    hooks.insert("preToolUse".to_string(), Value::Array(kept));
    root.insert("version".to_string(), json!(1));
    let next = stringify_json(&cfg);
    if next != raw {
        return write_file(path, &next).is_ok();
    }
    true
}

pub fn install_cursor_rtk_hook() -> bool {
    let paths = cursor_hooks_files();
    let snapshots = cursor_file_snapshots(&paths);
    for (i, path) in paths.iter().enumerate() {
        if !install_cursor_rtk_hook_at(path) {
            let _ = restore_cursor_files(&snapshots[..i]);
            return false;
        }
    }
    true
}

fn remove_cursor_rtk_hook_at(path: &str) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return true;
    };
    if !raw.trim().is_empty() && has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(root) = cfg.as_object_mut() else {
        return false;
    };
    if let Some(version) = root.get("version") {
        if !cursor_version_one(version) {
            return false;
        }
    }
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return true;
    };
    let Some(entries) = hooks
        .get("preToolUse")
        .and_then(|v| v.as_array())
        .cloned()
    else {
        return false;
    };
    let want = cursor_rtk_hook_command_for(cursor_target_bridge(path));
    let kept: Vec<Value> = entries
        .iter()
        .filter(|entry| !cursor_rtk_hook_entry_matches_for(entry, &want))
        .cloned()
        .collect();
    if kept.len() == entries.len() {
        return true;
    }
    if kept.is_empty() {
        hooks.remove("preToolUse");
    } else {
        hooks.insert("preToolUse".to_string(), Value::Array(kept));
    }
    if hooks.is_empty() {
        root.remove("hooks");
    }
    if root.is_empty() {
        return fs::remove_file(path).is_ok();
    }
    write_file(path, &stringify_json(&cfg)).is_ok()
}

pub fn remove_cursor_rtk_hook() -> bool {
    let paths = cursor_hooks_files();
    let snapshots = cursor_file_snapshots(&paths);
    for (i, path) in paths.iter().enumerate() {
        if !remove_cursor_rtk_hook_at(path) {
            let _ = restore_cursor_files(&snapshots[..i]);
            return false;
        }
    }
    true
}

pub fn has_cursor_rtk_hook() -> bool {
    for path in cursor_hooks_files() {
        let want = cursor_rtk_hook_command_for(cursor_target_bridge(&path));
        if !has_cursor_rtk_hook_at(&path, &want) {
            return false;
        }
    }
    true
}

fn has_cursor_rtk_hook_at(path: &str, want: &str) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(root) = cfg.as_object() else {
        return false;
    };
    let Some(version) = root.get("version") else {
        return false;
    };
    if !cursor_version_one(version) {
        return false;
    }
    let Some(entries) = root
        .get("hooks")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get("preToolUse"))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    entries
        .iter()
        .any(|entry| cursor_rtk_hook_entry_matches_for(entry, want))
}

fn parse_cursor_permission_config(raw: &str, file_exists: bool, allow_jsonc: bool) -> Option<Value> {
    if !allow_jsonc {
        if !file_exists || raw.trim().is_empty() {
            return Some(Value::Object(OrderedMap::new()));
        }
        if !raw.trim().starts_with('{') {
            return None;
        }
        let v: Value = serde_json::from_str(raw).ok()?;
        if !v.is_object() {
            return None;
        }
        return Some(v);
    }
    if !file_exists || raw.trim().is_empty() {
        return Some(Value::Object(OrderedMap::new()));
    }
    let v = try_parse_jsonc(raw)?;
    if !v.is_object() {
        return None;
    }
    Some(v)
}

fn cursor_permission(path: &str, key: &str, want: &str, nested: bool, remove: bool) -> bool {
    let raw_opt = read_file_safe(path);
    let file_exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match parse_cursor_permission_config(&raw, file_exists, !nested) {
        Some(v) => v,
        None => {
            if file_exists && !raw.trim().is_empty() {
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root_obj = cfg.as_object_mut().unwrap();
    if nested && !root_obj.contains_key("permissions") {
        root_obj.insert("permissions".to_string(), Value::Object(OrderedMap::new()));
    }
    let target_map = if nested {
        let Some(m) = root_obj
            .get_mut("permissions")
            .and_then(|v| v.as_object_mut())
        else {
            return false;
        };
        m
    } else {
        root_obj
    };
    let mut items: Vec<String> = Vec::new();
    if let Some(v) = target_map.get(key) {
        let Some(arr) = v.as_array() else {
            return false;
        };
        for item in arr {
            let Some(s) = item.as_str() else {
                return false;
            };
            items.push(s.to_string());
        }
    }
    let mut kept: Vec<Value> = Vec::with_capacity(items.len() + 1);
    let mut changed = false;
    for item in &items {
        if remove && item == want {
            changed = true;
            continue;
        }
        kept.push(Value::String(item.clone()));
    }
    if !remove {
        if items.iter().any(|item| item == want) {
            return true;
        }
        kept.push(Value::String(want.to_string()));
        changed = true;
    }
    if !changed {
        return true;
    }
    if kept.is_empty() {
        target_map.remove(key);
    } else {
        target_map.insert(key.to_string(), Value::Array(kept));
    }
    let root_obj = cfg.as_object_mut().unwrap();
    if nested {
        if root_obj
            .get("permissions")
            .and_then(|v| v.as_object())
            .is_some_and(|m| m.is_empty())
        {
            root_obj.remove("permissions");
        }
    }
    if !nested && root_obj.is_empty() {
        return fs::remove_file(path).is_ok();
    }
    write_file(path, &stringify_json(&cfg)).is_ok()
}

struct CursorPermissionSpec {
    path: String,
    key: String,
    want: String,
    nested: bool,
}

fn cursor_file_snapshots(paths: &[String]) -> Vec<CursorFileSnapshot> {
    paths
        .iter()
        .map(|path| {
            let raw_opt = read_file_safe(path);
            CursorFileSnapshot {
                path: path.clone(),
                exists: raw_opt.is_some(),
                raw: raw_opt.unwrap_or_default(),
            }
        })
        .collect()
}

fn restore_cursor_files(snapshots: &[CursorFileSnapshot]) -> bool {
    let mut ok = true;
    for snapshot in snapshots {
        if snapshot.exists {
            if write_file(&snapshot.path, &snapshot.raw).is_err() {
                ok = false;
            }
        } else if let Err(err) = fs::remove_file(&snapshot.path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                ok = false;
            }
        }
    }
    ok
}

fn update_cursor_permissions(specs: &[CursorPermissionSpec], remove: bool) -> bool {
    let mut snapshots = Vec::with_capacity(specs.len());
    let mut seen = HashSet::new();
    for spec in specs {
        if !seen.insert(spec.path.clone()) {
            continue;
        }
        let raw_opt = read_file_safe(&spec.path);
        snapshots.push(CursorFileSnapshot {
            path: spec.path.clone(),
            exists: raw_opt.is_some(),
            raw: raw_opt.unwrap_or_default(),
        });
    }
    for spec in specs {
        if !cursor_permission(&spec.path, &spec.key, &spec.want, spec.nested, remove) {
            let _ = restore_cursor_files(&snapshots);
            return false;
        }
    }
    true
}

fn has_cursor_permissions(specs: &[CursorPermissionSpec]) -> bool {
    specs.iter().all(|spec| {
        cursor_permission_present(&spec.path, &spec.key, &spec.want, spec.nested)
    })
}

fn cursor_permission_present(path: &str, key: &str, want: &str, nested: bool) -> bool {
    let Some(raw) = read_file_safe(path) else {
        return false;
    };
    let Some(cfg) = parse_cursor_permission_config(&raw, true, !nested) else {
        return false;
    };
    let Some(root_obj) = cfg.as_object() else {
        return false;
    };
    let target_map = if nested {
        let Some(m) = root_obj.get("permissions").and_then(|v| v.as_object()) else {
            return false;
        };
        m
    } else {
        root_obj
    };
    let Some(items) = target_map.get(key).and_then(|v| v.as_array()) else {
        return false;
    };
    items.iter().any(|item| item.as_str() == Some(want))
}

fn cursor_rtk_permission_specs() -> Vec<CursorPermissionSpec> {
    let mut specs = Vec::new();
    for path in cursor_permissions_files() {
        specs.push(CursorPermissionSpec {
            path,
            key: "terminalAllowlist".to_string(),
            want: "rtk".to_string(),
            nested: false,
        });
    }
    for path in cursor_cli_config_files() {
        specs.push(CursorPermissionSpec {
            path,
            key: "allow".to_string(),
            want: "Shell(rtk)".to_string(),
            nested: true,
        });
    }
    specs
}

fn cursor_mcp_permission_specs(tool_id: &str) -> Vec<CursorPermissionSpec> {
    let mut specs = Vec::new();
    for path in cursor_permissions_files() {
        specs.push(CursorPermissionSpec {
            path,
            key: "mcpAllowlist".to_string(),
            want: format!("{}:*", tool_id),
            nested: false,
        });
    }
    for path in cursor_cli_config_files() {
        specs.push(CursorPermissionSpec {
            path,
            key: "allow".to_string(),
            want: format!("Mcp({}:*)", tool_id),
            nested: true,
        });
    }
    specs
}

pub fn configure_cursor_rtk_permissions() -> bool {
    update_cursor_permissions(&cursor_rtk_permission_specs(), false)
}

pub fn remove_cursor_rtk_permissions() -> bool {
    update_cursor_permissions(&cursor_rtk_permission_specs(), true)
}

pub fn has_cursor_rtk_permissions() -> bool {
    has_cursor_permissions(&cursor_rtk_permission_specs())
}

pub fn configure_cursor_mcp_permissions(tool_id: &str) -> bool {
    update_cursor_permissions(&cursor_mcp_permission_specs(tool_id), false)
}

pub fn remove_cursor_mcp_permissions(tool_id: &str) -> bool {
    update_cursor_permissions(&cursor_mcp_permission_specs(tool_id), true)
}

pub fn has_cursor_mcp_permissions(tool_id: &str) -> bool {
    has_cursor_permissions(&cursor_mcp_permission_specs(tool_id))
}

pub fn cursor_desktop_paths() -> Vec<String> {
    if IS_WIN {
        let mut paths = Vec::new();
        if let Ok(d) = std::env::var("LOCALAPPDATA") {
            if !d.is_empty() {
                paths.push(
                    Path::new(&d)
                        .join("Programs")
                        .join("cursor")
                        .join("Cursor.exe")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
        if let Ok(d) = std::env::var("ProgramFiles") {
            if !d.is_empty() {
                paths.push(
                    Path::new(&d)
                        .join("cursor")
                        .join("Cursor.exe")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
        return paths;
    }
    if std::env::consts::OS == "macos" {
        return vec![
            "/Applications/Cursor.app".to_string(),
            "/Applications/Cursor Nightly.app".to_string(),
        ];
    }
    if std::env::consts::OS == "linux" {
        return vec![
            Path::new(&home())
                .join(".local")
                .join("share")
                .join("applications")
                .join("cursor.desktop")
                .to_string_lossy()
                .to_string(),
            "/usr/local/share/applications/cursor.desktop".to_string(),
            "/usr/share/applications/cursor.desktop".to_string(),
        ];
    }
    Vec::new()
}

fn cursor_desktop_file_valid(path: &str) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let mut type_application = false;
    let mut name_cursor = false;
    let mut exec_cursor = false;
    for line in raw.split('\n') {
        let line = line.trim();
        if line == "Type=Application" {
            type_application = true;
        }
        if let Some(rest) = line.strip_prefix("Name=") {
            if rest.to_lowercase().contains("cursor") {
                name_cursor = true;
            }
        }
        if let Some(rest) = line.strip_prefix("Exec=") {
            for token in rest.split_whitespace() {
                let token = token.trim_matches(['"', '\'']);
                let base = Path::new(token)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if base.contains("cursor") {
                    exec_cursor = true;
                    break;
                }
            }
        }
    }
    type_application && name_cursor && exec_cursor
}

fn cursor_mac_app_valid(path: &str) -> bool {
    if !exists(path) {
        return false;
    }
    let plist = Path::new(path).join("Contents").join("Info.plist");
    let Ok(raw) = fs::read_to_string(plist) else {
        return true;
    };
    raw.contains("com.todesktop.230313mzl4w4u92") || raw.contains("co.anysphere.cursor.nightly")
}

pub fn cursor_desktop_installed() -> bool {
    if !windows_cursor_home_from_wsl().is_empty() {
        return true;
    }
    for p in cursor_desktop_paths() {
        if std::env::consts::OS == "linux" {
            if cursor_desktop_file_valid(&p) {
                return true;
            }
        } else if std::env::consts::OS == "macos" {
            if cursor_mac_app_valid(&p) {
                return true;
            }
        } else if exists(&p) {
            return true;
        }
    }
    false
}

pub fn cursor_known_bin_dirs() -> Vec<String> {
    let mut dirs = vec![
        Path::new(&home())
            .join(".local")
            .join("bin")
            .to_string_lossy()
            .to_string(),
    ];
    if IS_WIN {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                dirs.push(
                    Path::new(&local)
                        .join("Programs")
                        .join("cursor")
                        .to_string_lossy()
                        .to_string(),
                );
                dirs.push(
                    Path::new(&local)
                        .join("Programs")
                        .join("Cursor")
                        .join("resources")
                        .join("app")
                        .join("bin")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
        dirs.push(
            Path::new(&home())
                .join(".cursor")
                .join("bin")
                .to_string_lossy()
                .to_string(),
        );
    }
    dirs
}

pub fn cursor_cli_path() -> String {
    let known = cursor_known_bin_dirs();
    let known_refs: Vec<&str> = known.iter().map(|s| s.as_str()).collect();
    for name in ["cursor-agent", "agent"] {
        let bin = find_binary(name, &known_refs);
        if !bin.is_empty() {
            return bin;
        }
    }
    String::new()
}

pub fn cursor_cli_is_cursor() -> bool {
    let bin = cursor_cli_path();
    if bin.is_empty() {
        return false;
    }
    for arg in [&["--version"][..], &["--help"][..]] {
        let result = run(
            &bin,
            arg,
            RunOptions {
                capture: true,
                quiet: true,
                ..Default::default()
            },
        );
        let combined = format!("{}\n{}", result.stdout, result.stderr).to_lowercase();
        if combined.contains("cursor") {
            return true;
        }
    }
    false
}

pub fn detect_cursor() -> Detection {
    let cli = cursor_cli_is_cursor();
    let desktop = cursor_desktop_installed();
    let config =
        std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" && exists(cursor_config_dir());
    if cli && desktop {
        Detection {
            installed: true,
            source: "cli+desktop".to_string(),
        }
    } else if cli {
        Detection {
            installed: true,
            source: "cli".to_string(),
        }
    } else if desktop {
        Detection {
            installed: true,
            source: "desktop".to_string(),
        }
    } else if config {
        Detection {
            installed: true,
            source: "config".to_string(),
        }
    } else {
        Detection::default()
    }
}
