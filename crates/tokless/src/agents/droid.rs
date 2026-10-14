// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/droid.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agents::proxy_protocol::proxy_endpoint_for;
use crate::util::byok::{
    BYOKRoute, byok_gateway_endpoint, byok_route_header, byok_route_protocol,
    delete_byok_route as util_delete_byok_route, read_byok_route, upsert_byok_route,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::jsonc::{OrderedMap, has_jsonc_comments, stringify_json, try_parse_jsonc};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths::{
    IS_WIN, ensure_dir, exists, home, persisted_tokless_command, read_file_safe, split_command,
    tokless_persisted_abs, write_file, write_file_atomic,
};

pub const CONTEXT_MODE_DROID_TOOL_NAMES: &[&str] = &[
    "ctx_search",
    "ctx_execute",
    "ctx_execute_file",
    "ctx_batch_execute",
    "ctx_index",
    "ctx_fetch_and_index",
];

pub const CODEGRAPH_DROID_TOOL_NAMES: &[&str] = &["codegraph_explore"];

pub const PROJECTMEM_MCP_TOOL_NAMES: &[&str] = &[
    "log_issue",
    "record_attempt",
    "record_fix",
    "add_decision",
    "add_note",
];

pub const DROID_PROXY_DISPLAY_NAME: &str = "Headroom Proxy";
pub const DROID_PROXY_ID_PREFIX: &str = "custom:Headroom-Proxy-";
pub const DROID_PROXY_PROVIDER_KIND: &str = "generic-chat-completion-api";
const HEADROOM_BASE_URL_HEADER: &str = "x-headroom-base-url";

// --- paths ---

pub fn droid_dir() -> String {
    Path::new(&home())
        .join(".factory")
        .to_string_lossy()
        .to_string()
}

pub fn droid_hooks_file() -> String {
    Path::new(&droid_dir())
        .join("hooks.json")
        .to_string_lossy()
        .to_string()
}

pub fn droid_mcp_file() -> String {
    Path::new(&droid_dir())
        .join("mcp.json")
        .to_string_lossy()
        .to_string()
}

pub fn droid_instructions_file() -> String {
    Path::new(&droid_dir())
        .join("AGENTS.md")
        .to_string_lossy()
        .to_string()
}

pub fn droid_settings_file() -> String {
    Path::new(&droid_dir())
        .join("settings.json")
        .to_string_lossy()
        .to_string()
}

// --- detection ---

pub fn droid_known_bin_dirs() -> Vec<String> {
    let h = home();
    let mut dirs = vec![
        Path::new(&h)
            .join(".factory")
            .join("bin")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
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
                        .join("factory")
                        .join("bin")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    }
    dirs
}

pub fn droid_desktop_paths() -> Vec<String> {
    if IS_WIN {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                return vec![
                    Path::new(&local)
                        .join("Programs")
                        .join("Factory")
                        .join("Factory.exe")
                        .to_string_lossy()
                        .to_string(),
                ];
            }
        }
        return Vec::new();
    }
    if std::env::consts::OS == "macos" {
        return vec!["/Applications/Factory.app".to_string()];
    }
    vec!["/usr/bin/factory".to_string()]
}

// --- internal helpers ---

fn tokless_command(args: &[&str]) -> String {
    persisted_tokless_command(&tokless_persisted_abs(), args)
}

fn tokless_managed_command(cmd: &str, args: &[&str]) -> bool {
    let parts = split_command(cmd.trim());
    if parts.len() != args.len() + 1 {
        return false;
    }
    let normalized = parts[0].replace('\\', "/");
    let exe = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if exe != "tokless" && exe != "tokless.exe" {
        return false;
    }
    for (i, a) in args.iter().enumerate() {
        if parts[i + 1] != *a {
            return false;
        }
    }
    true
}

fn droid_enabled_tools(tool_id: &str) -> Option<&'static [&'static str]> {
    match tool_id {
        "codegraph" => Some(CODEGRAPH_DROID_TOOL_NAMES),
        "context-mode" => Some(CONTEXT_MODE_DROID_TOOL_NAMES),
        "projectmem" => Some(PROJECTMEM_MCP_TOOL_NAMES),
        _ => None,
    }
}

fn args_eq(have: Option<&Value>, want: &[String]) -> bool {
    let Some(Value::Array(arr)) = have else {
        return want.is_empty();
    };
    if arr.len() != want.len() {
        return false;
    }
    arr.iter()
        .zip(want.iter())
        .all(|(x, w)| x.as_str() == Some(w.as_str()))
}

fn enabled_tools_eq(have: Option<&Value>, want: Option<&[&str]>) -> bool {
    let Some(want) = want else {
        return have.is_none();
    };
    let Some(Value::Array(arr)) = have else {
        return false;
    };
    if arr.len() != want.len() {
        return false;
    }
    let mut got = HashSet::with_capacity(arr.len());
    for v in arr {
        let Some(s) = v.as_str() else {
            return false;
        };
        got.insert(s);
    }
    want.iter().all(|w| got.contains(w))
}

// --- MCP management ---

pub fn configure_droid_mcp(tool_id: &str) -> (bool, String) {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("droid", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };

    let f = droid_mcp_file();
    if let Some(parent) = Path::new(&f).parent() {
        let _ = ensure_dir(parent);
    }
    let raw_opt = read_file_safe(&f);
    let file_exists = raw_opt.is_some();
    if !file_exists && exists(&f) {
        return (false, f);
    }
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if !raw.trim().is_empty() {
                return (false, f);
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root = cfg.as_object_mut().unwrap();
    if !root.contains_key("mcpServers") {
        root.insert("mcpServers".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(servers) = root.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
        return (false, f);
    };

    let mut entry = OrderedMap::new();
    entry.insert("command".to_string(), Value::String(spawn.command.clone()));
    if !spawn.args.is_empty() {
        entry.insert(
            "args".to_string(),
            Value::Array(
                spawn
                    .args
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    let want_tools = droid_enabled_tools(tool_id);
    if let Some(tools) = want_tools {
        entry.insert(
            "enabledTools".to_string(),
            Value::Array(tools.iter().map(|s| Value::String((*s).to_string())).collect()),
        );
    }

    if let Some(existing) = servers.get(tool_id) {
        let Some(em) = existing.as_object() else {
            return (false, f);
        };
        let ec = em.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let ea = em.get("args");
        let et = em.get("enabledTools");
        if ec == spawn.command && args_eq(ea, &spawn.args) && enabled_tools_eq(et, want_tools) {
            return (false, f);
        }
        if ec != spawn.command {
            return (false, f);
        }
    }

    servers.insert(tool_id.to_string(), Value::Object(entry));
    let next = stringify_json(&cfg);
    if next != raw {
        if write_file(&f, &next).is_err() {
            return (false, f);
        }
        return (true, f);
    }
    (false, String::new())
}

pub fn remove_droid_mcp(tool_id: &str) -> bool {
    let f = droid_mcp_file();
    let Some(raw) = read_file_safe(&f) else {
        return false;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(sm) = cfg
        .as_object_mut()
        .and_then(|m| m.get_mut("mcpServers"))
        .and_then(|v| v.as_object_mut())
    else {
        return false;
    };
    let Some(existing) = sm.get(tool_id) else {
        return false;
    };
    let Some(em) = existing.as_object() else {
        return false;
    };
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("droid", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let command = em.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let args = em.get("args");
    let tools = em.get("enabledTools");
    if command != spawn.command
        || !args_eq(args, &spawn.args)
        || !enabled_tools_eq(tools, droid_enabled_tools(tool_id))
    {
        return false;
    }
    sm.remove(tool_id);
    write_file(&f, &stringify_json(&cfg)).is_ok()
}

pub fn droid_mcp_bounded(tool_id: &str) -> bool {
    let Some(raw) = read_file_safe(droid_mcp_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(em) = cfg
        .as_object()
        .and_then(|m| m.get("mcpServers"))
        .and_then(|v| v.as_object())
        .and_then(|sm| sm.get(tool_id))
        .and_then(|v| v.as_object())
    else {
        return false;
    };
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("droid", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let ec = em.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let ea = em.get("args");
    let et = em.get("enabledTools");
    ec == spawn.command
        && args_eq(ea, &spawn.args)
        && enabled_tools_eq(et, droid_enabled_tools(tool_id))
}

pub fn droid_projectmem_mcp_bounded() -> bool {
    droid_mcp_bounded("projectmem")
}

pub fn droid_mcp_has(tool_id: &str) -> bool {
    let Some(raw) = read_file_safe(droid_mcp_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    cfg.as_object()
        .and_then(|m| m.get("mcpServers"))
        .and_then(|v| v.as_object())
        .is_some_and(|sm| sm.contains_key(tool_id))
}

// --- hooks management ---

fn droid_hooks_load() -> Option<Value> {
    let path = droid_hooks_file();
    let Some(raw) = read_file_safe(&path) else {
        if exists(&path) {
            return None;
        }
        return Some(Value::Object(OrderedMap::new()));
    };
    if raw.trim().is_empty() {
        return Some(Value::Object(OrderedMap::new()));
    }
    let v = try_parse_jsonc(&raw)?;
    if !v.is_object() {
        return None;
    }
    Some(v)
}

fn droid_hooks_save(cfg: &Value, raw: &str) -> bool {
    let next = stringify_json(cfg);
    if next != raw {
        return write_file(droid_hooks_file(), &next).is_ok();
    }
    true
}

fn droid_raw_hooks() -> String {
    read_file_safe(droid_hooks_file()).unwrap_or_default()
}

fn droid_add_hook_group(
    cfg: &mut Value,
    event: &str,
    matcher: &str,
    managed_args: &[&str],
    hook_cfg: Value,
) -> bool {
    let Some(root) = cfg.as_object_mut() else {
        return false;
    };
    let arr = match root.get(event) {
        Some(Value::Array(a)) => a.clone(),
        Some(_) => return false,
        None => Vec::new(),
    };

    let mut found = false;
    let mut out = Vec::with_capacity(arr.len() + 1);
    for existing in arr {
        let Some(mut em) = existing.as_object().cloned() else {
            return false;
        };
        let em_matcher = em.get("matcher").and_then(|v| v.as_str()).unwrap_or("");
        let has_matcher = em.get("matcher").is_some_and(|v| v.as_str().is_some());
        let Some(ha) = em.get("hooks").and_then(|v| v.as_array()).cloned() else {
            return false;
        };
        if has_matcher && em_matcher == matcher && !ha.is_empty() {
            let mut kept = Vec::with_capacity(ha.len());
            for h in &ha {
                let Some(hm) = h.as_object() else {
                    return false;
                };
                let cs = hm.get("command").and_then(|v| v.as_str()).unwrap_or("");
                if !tokless_managed_command(cs, managed_args) {
                    kept.push(h.clone());
                    continue;
                }
                if !found {
                    kept.push(hook_cfg.clone());
                    found = true;
                }
            }
            if !kept.is_empty() || ha.is_empty() {
                em.insert("hooks".to_string(), Value::Array(kept));
                out.push(Value::Object(em));
            }
        } else {
            out.push(Value::Object(em));
        }
    }
    if !found {
        out.push(json!({
            "matcher": matcher,
            "hooks": [hook_cfg],
        }));
    }
    root.insert(event.to_string(), Value::Array(out));
    true
}

fn droid_remove_hook_group(cfg: &mut Value, event: &str, matcher: &str, managed_args: &[&str]) {
    let Some(root) = cfg.as_object_mut() else {
        return;
    };
    let Some(Value::Array(arr)) = root.get(event).cloned() else {
        return;
    };
    let mut kept = Vec::new();
    for g in arr {
        let Some(mut gm) = g.as_object().cloned() else {
            return;
        };
        if let Some(mv) = gm.get("matcher") {
            if mv.as_str() != Some(matcher) {
                kept.push(Value::Object(gm));
                continue;
            }
        }
        let Some(ha) = gm.get("hooks").and_then(|v| v.as_array()).cloned() else {
            return;
        };
        let mut kept_hooks = Vec::with_capacity(ha.len());
        for h in &ha {
            let Some(hm) = h.as_object() else {
                return;
            };
            let cs = hm.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if !tokless_managed_command(cs, managed_args) {
                kept_hooks.push(h.clone());
            }
        }
        if !kept_hooks.is_empty() || ha.is_empty() {
            gm.insert("hooks".to_string(), Value::Array(kept_hooks));
            kept.push(Value::Object(gm));
        }
    }
    if kept.is_empty() {
        root.remove(event);
    } else {
        root.insert(event.to_string(), Value::Array(kept));
    }
}

fn droid_has_hook(event: &str, cmd_substring: &str) -> bool {
    let Some(cfg) = droid_hooks_load() else {
        return false;
    };
    let Some(arr) = cfg
        .as_object()
        .and_then(|m| m.get(event))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    for g in arr {
        let Some(gm) = g.as_object() else {
            continue;
        };
        let Some(ha) = gm.get("hooks").and_then(|v| v.as_array()) else {
            continue;
        };
        for h in ha {
            if let Some(cs) = h
                .as_object()
                .and_then(|hm| hm.get("command"))
                .and_then(|c| c.as_str())
            {
                if cs.contains(cmd_substring) {
                    return true;
                }
            }
        }
    }
    false
}

fn droid_has_exact_hook(event: &str, matcher: &str, command: &str) -> bool {
    let Some(cfg) = droid_hooks_load() else {
        return false;
    };
    let Some(arr) = cfg
        .as_object()
        .and_then(|m| m.get(event))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    for group in arr {
        let Some(gm) = group.as_object() else {
            continue;
        };
        if gm.get("matcher").and_then(|v| v.as_str()) != Some(matcher) {
            continue;
        }
        let Some(hook_arr) = gm.get("hooks").and_then(|v| v.as_array()) else {
            continue;
        };
        for hook in hook_arr {
            let Some(hm) = hook.as_object() else {
                continue;
            };
            if hm.get("type").and_then(|v| v.as_str()) != Some("command") {
                continue;
            }
            if hm.get("command").and_then(|v| v.as_str()) == Some(command) {
                return true;
            }
        }
    }
    false
}

// --- RTK hook ---

pub fn install_droid_rtk_hook() -> bool {
    let command = tokless_command(&["rtk-hook", "droid"]);
    if command.is_empty() {
        return false;
    }
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return false;
    };
    let hook_cfg = json!({
        "type": "command",
        "command": command,
    });
    if !droid_add_hook_group(
        &mut cfg,
        "PreToolUse",
        "Execute",
        &["rtk-hook", "droid"],
        hook_cfg,
    ) {
        return false;
    }
    droid_hooks_save(&cfg, &raw) && has_droid_rtk_hook()
}

pub fn remove_droid_rtk_hook() {
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return;
    };
    droid_remove_hook_group(&mut cfg, "PreToolUse", "Execute", &["rtk-hook", "droid"]);
    droid_hooks_save(&cfg, &raw);
}

pub fn has_droid_rtk_hook() -> bool {
    droid_has_exact_hook(
        "PreToolUse",
        "Execute",
        &tokless_command(&["rtk-hook", "droid"]),
    )
}

// --- CodeGraph index hook ---

pub fn install_droid_codegraph_index_hook() -> bool {
    let command = tokless_command(&["index", "--auto", "droid"]);
    if command.is_empty() {
        return false;
    }
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return false;
    };
    let hook_cfg = json!({
        "type": "command",
        "command": command,
        "timeout": 120,
    });
    if !droid_add_hook_group(
        &mut cfg,
        "SessionStart",
        "",
        &["index", "--auto", "droid"],
        hook_cfg,
    ) {
        return false;
    }
    droid_hooks_save(&cfg, &raw) && has_droid_codegraph_index_hook()
}

pub fn remove_droid_codegraph_index_hook() {
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return;
    };
    droid_remove_hook_group(
        &mut cfg,
        "SessionStart",
        "",
        &["index", "--auto", "droid"],
    );
    droid_hooks_save(&cfg, &raw);
}

pub fn has_droid_codegraph_index_hook() -> bool {
    droid_has_hook("SessionStart", "index --auto droid")
}

// --- projectmem SessionStart hook ---

pub fn install_droid_projectmem_hook() -> bool {
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return false;
    };
    let projectmem_cmd = tokless_command(&["projectmem-hook", "claude"]);
    if projectmem_cmd.is_empty() {
        return false;
    }
    let hook_cfg = json!({
        "type": "command",
        "command": projectmem_cmd,
        "timeout": 15,
    });
    if !droid_add_hook_group(
        &mut cfg,
        "SessionStart",
        "",
        &["projectmem-hook", "claude"],
        hook_cfg,
    ) {
        return false;
    }
    droid_hooks_save(&cfg, &raw) && has_droid_projectmem_hook()
}

pub fn remove_droid_projectmem_hook() {
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return;
    };
    droid_remove_hook_group(
        &mut cfg,
        "SessionStart",
        "",
        &["projectmem-hook", "claude"],
    );
    droid_hooks_save(&cfg, &raw);
}

pub fn has_droid_projectmem_hook() -> bool {
    droid_has_hook("SessionStart", "projectmem-hook claude")
}

// --- Context-Mode: MCP + AGENTS.md instruction only, no PreToolUse hook ---

pub fn remove_droid_ctx_mode_pre_tool_use() {
    let raw = droid_raw_hooks();
    let Some(mut cfg) = droid_hooks_load() else {
        return;
    };
    droid_remove_hook_group(&mut cfg, "PreToolUse", "", &["ctx-hook", "droid"]);
    droid_hooks_save(&cfg, &raw);
}

pub fn has_droid_ctx_mode_pre_tool_use() -> bool {
    droid_has_hook("PreToolUse", "ctx-hook droid")
}

// --- Droid headroom HTTP proxy ---

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct ProxyRouteStashEntry {
    #[serde(default)]
    file: String,
    #[serde(default)]
    provider: String,
    #[serde(default)]
    base_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    upstream: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    base_key: String,
    #[serde(default, skip_serializing_if = "is_false")]
    had_base_key: bool,
    #[serde(default, rename = "had_auth_token", skip_serializing_if = "is_false")]
    had_auth: bool,
    #[serde(default, rename = "moved_base_key", skip_serializing_if = "is_false")]
    moved_base: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    had_header: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    base_line: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    header_line: String,
    #[serde(default, skip_serializing_if = "is_false")]
    managed_native: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    original: Vec<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    managed: Vec<u8>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProxyRouteStashFile {
    providers: Option<HashMap<String, ProxyRouteStashEntry>>,
}

struct ByokRouteRollback {
    id: String,
    previous: Option<BYOKRoute>,
}

fn proxy_wire_model() -> String {
    if let Ok(v) = std::env::var("TOKLESS_PROXY_MODEL") {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    "headroom".to_string()
}

fn proxy_wire_key() -> String {
    if let Ok(v) = std::env::var("TOKLESS_PROXY_KEY") {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    "tokless".to_string()
}

fn is_absolute_http(u: &str) -> bool {
    let parsed = match url::Url::parse(u.trim()) {
        Ok(p) => p,
        Err(_) => return false,
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }
    match parsed.host_str() {
        Some(h) if !h.is_empty() => true,
        _ => false,
    }
}

fn same_proxy_base(a: &str, b: &str) -> bool {
    let a_clean = a.trim().trim_end_matches('/').to_lowercase();
    let b_clean = b.trim().trim_end_matches('/').to_lowercase();
    !a_clean.is_empty() && a_clean == b_clean
}

fn normalized_headroom_upstream(base_url: &str, api: &str) -> String {
    let base = base_url.trim().trim_end_matches('/');
    if (api == "openai-completions" || api == "openai-responses" || api == "anthropic-messages")
        && base.to_lowercase().ends_with("/v1")
    {
        return base[..base.len() - 3].to_string();
    }
    base.to_string()
}

fn proxy_route_stash_path(agent: &str) -> String {
    Path::new(&headroom_paths_resolved().root)
        .join(format!("{}.byok.stash.json", agent))
        .to_string_lossy()
        .to_string()
}

fn proxy_route_stash_lock_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("byok.stash.lock")
        .to_string_lossy()
        .to_string()
}

fn with_proxy_route_stash_lock<F>(f: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    let root = headroom_paths_resolved().root;
    ensure_dir(&root).map_err(|e| e.to_string())?;
    let lock_path = proxy_route_stash_lock_path();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(file) => {
                drop(file);
                let res = f();
                let _ = fs::remove_file(&lock_path);
                return res;
            }
            Err(_) => {
                if let Ok(meta) = fs::metadata(&lock_path) {
                    if let Ok(modified) = meta.modified() {
                        if modified.elapsed().unwrap_or_default() > Duration::from_secs(10) {
                            let _ = fs::remove_file(&lock_path);
                            continue;
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Err(format!("timed out acquiring {}", lock_path));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn load_proxy_route_stash_locked(agent: &str) -> HashMap<String, ProxyRouteStashEntry> {
    let Some(raw) = read_file_safe(proxy_route_stash_path(agent)) else {
        return HashMap::new();
    };
    let Ok(f) = serde_json::from_str::<ProxyRouteStashFile>(&raw) else {
        return HashMap::new();
    };
    f.providers.unwrap_or_default()
}

fn proxy_route_stash_valid(agent: &str) -> bool {
    let Some(raw) = read_file_safe(proxy_route_stash_path(agent)) else {
        return true;
    };
    serde_json::from_str::<ProxyRouteStashFile>(&raw)
        .map(|f| f.providers.is_some())
        .unwrap_or(false)
}

fn restore_proxy_route_stash(agent: &str, raw: &str, file_exists: bool) -> std::io::Result<()> {
    let path = proxy_route_stash_path(agent);
    if file_exists {
        write_file(&path, raw)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        return Ok(());
    }
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn restore_proxy_route_stash_logged(agent: &str, raw: &str, file_exists: bool) -> bool {
    if let Err(err) = restore_proxy_route_stash(agent, raw, file_exists) {
        L.err(&format!("{} proxy stash rollback failed: {}", agent, err));
        return false;
    }
    true
}

fn save_proxy_route_stash(
    agent: &str,
    providers: &HashMap<String, ProxyRouteStashEntry>,
) -> Result<(), String> {
    let path = proxy_route_stash_path(agent);
    if providers.is_empty() {
        if let Some(raw) = read_file_safe(&path) {
            let valid = serde_json::from_str::<ProxyRouteStashFile>(&raw)
                .map(|f| f.providers.is_some())
                .unwrap_or(false);
            if !valid {
                return Err(format!("refusing to replace malformed proxy stash {}", path));
            }
        }
        match fs::remove_file(&path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.to_string()),
        }
    }
    let b = serde_json::to_string(&ProxyRouteStashFile {
        providers: Some(providers.clone()),
    })
    .map_err(|e| e.to_string())?;
    write_file_atomic(&path, &b).map_err(|e| e.to_string())
}

fn register_byok_route(
    agent: &str,
    id: &str,
    api: &str,
    upstream: &str,
) -> (String, String, Option<BYOKRoute>, bool) {
    let protocol = byok_route_protocol(api);
    if protocol.is_empty() || upstream.trim().is_empty() {
        return (String::new(), String::new(), None, false);
    }
    let namespaced_id = format!("{}:{}", agent, id);
    let Ok((route, previous)) = upsert_byok_route(&namespaced_id, protocol, upstream) else {
        return (String::new(), String::new(), None, false);
    };
    (
        byok_gateway_endpoint(),
        byok_route_header(&route),
        previous,
        true,
    )
}

fn rollback_byok_route(agent: &str, id: &str, previous: Option<&BYOKRoute>) -> Result<(), String> {
    let namespaced_id = format!("{}:{}", agent, id);
    if let Some(prev) = previous {
        if !prev.id.is_empty() {
            return upsert_byok_route(&namespaced_id, &prev.protocol, &prev.upstream)
                .map(|_| ())
                .map_err(|e| e.to_string());
        }
    }
    util_delete_byok_route(&namespaced_id).map_err(|e| e.to_string())
}

fn rollback_byok_route_logged(agent: &str, id: &str, previous: Option<&BYOKRoute>) -> bool {
    if let Err(err) = rollback_byok_route(agent, id, previous) {
        L.err(&format!("{} BYOK route rollback failed: {}", agent, err));
        return false;
    }
    true
}

fn delete_byok_route(agent: &str, id: &str) -> Result<(), String> {
    let namespaced_id = format!("{}:{}", agent, id);
    util_delete_byok_route(&namespaced_id).map_err(|e| e.to_string())
}

fn restore_deleted_byok_routes(routes: &[BYOKRoute]) {
    for route in routes {
        if let Err(err) = upsert_byok_route(&route.id, &route.protocol, &route.upstream) {
            L.err(&format!("BYOK route restore failed: {}", err));
        }
    }
}

fn droid_model_route(model: &OrderedMap) -> Option<(String, String, OrderedMap)> {
    if model.get("provider").and_then(|v| v.as_str()) != Some(DROID_PROXY_PROVIDER_KIND) {
        return None;
    }
    let id = model.get("model")?.as_str()?;
    if id.is_empty() || id == proxy_wire_model() {
        return None;
    }
    let base = model.get("baseUrl")?.as_str()?;
    if !is_absolute_http(base) || base.trim().is_empty() {
        return None;
    }
    let key = model.get("apiKey").and_then(|v| v.as_str())?;
    if key.trim().is_empty() {
        return None;
    }
    let headers = if let Some(extra) = model.get("extraHeaders") {
        extra.as_object()?.clone()
    } else {
        OrderedMap::new()
    };
    Some((id.to_string(), base.to_string(), headers))
}

fn droid_gateway_base(base: &str) -> bool {
    let gateway = byok_gateway_endpoint();
    same_proxy_base(base, &gateway) || same_proxy_base(base, &format!("{}/v1", gateway))
}

fn droid_native_routes(
    cfg: &mut OrderedMap,
    stash: &mut HashMap<String, ProxyRouteStashEntry>,
) -> (bool, bool, Vec<ByokRouteRollback>) {
    let Some(Value::Array(models)) = cfg.get("customModels").cloned() else {
        return (false, false, Vec::new());
    };
    let mut seen_ids = HashSet::new();
    for val in &models {
        let Some(model) = val.as_object() else {
            continue;
        };
        let Some((id, _, _)) = droid_model_route(model) else {
            continue;
        };
        if !seen_ids.insert(id) {
            return (false, true, Vec::new());
        }
    }

    let mut routed = HashSet::new();
    let mut registered_routes: Vec<ByokRouteRollback> = Vec::new();
    let mut changed = false;
    let mut found = false;
    let mut new_models = models.clone();

    for val in &mut new_models {
        let Some(model) = val.as_object_mut() else {
            continue;
        };
        let Some((id, base, mut headers)) = droid_model_route(model) else {
            continue;
        };
        let upstream = normalized_headroom_upstream(&base, "openai-completions");
        let mut endpoint = proxy_endpoint_for("droid");
        if droid_gateway_base(&base) {
            if let Some(route) = read_byok_route(&format!("droid:{}", id)) {
                if headers.get("X-Tokless-Route").and_then(|v| v.as_str())
                    == Some(&byok_route_header(&route))
                {
                    routed.insert(id);
                    found = true;
                    continue;
                }
            }
        }
        if let Some(current) = headers.get(HEADROOM_BASE_URL_HEADER) {
            let Some(current_str) = current.as_str() else {
                continue;
            };
            if current_str != upstream && !same_proxy_base(&base, &endpoint) {
                continue;
            }
        }
        found = true;
        if same_proxy_base(&base, &endpoint) {
            if headers.get(HEADROOM_BASE_URL_HEADER).and_then(|v| v.as_str()) != Some(&upstream) {
                continue;
            }
            routed.insert(id);
            continue;
        }
        let (route_endpoint, route_header, previous_route, route_ok) =
            register_byok_route("droid", &id, "openai-completions", &upstream);
        if upstream.to_lowercase().starts_with("https://") && !route_ok {
            for r in &registered_routes {
                let _ = rollback_byok_route("droid", &r.id, r.previous.as_ref());
            }
            return (false, true, Vec::new());
        }
        if route_ok {
            endpoint = route_endpoint;
            registered_routes.push(ByokRouteRollback {
                id: id.clone(),
                previous: previous_route,
            });
        }
        let mut had_header = false;
        let mut header_val = String::new();
        if let Some(current) = headers.get(HEADROOM_BASE_URL_HEADER) {
            let Some(s) = current.as_str() else {
                continue;
            };
            had_header = true;
            header_val = s.to_string();
        }
        stash.insert(
            id.clone(),
            ProxyRouteStashEntry {
                file: droid_settings_file(),
                provider: id.clone(),
                base_url: base,
                upstream: upstream.clone(),
                had_header,
                header: header_val,
                ..Default::default()
            },
        );
        routed.insert(id);
        model.insert("baseUrl".to_string(), Value::String(endpoint));
        headers.insert(
            HEADROOM_BASE_URL_HEADER.to_string(),
            Value::String(upstream),
        );
        if route_ok {
            headers.insert("X-Tokless-Route".to_string(), Value::String(route_header));
        }
        model.insert("extraHeaders".to_string(), Value::Object(headers));
        changed = true;
    }

    if changed {
        cfg.insert("customModels".to_string(), Value::Array(new_models.clone()));
    }

    let stash_keys: Vec<String> = stash.keys().cloned().collect();
    for id in stash_keys {
        if !routed.contains(&id) {
            let still_present = new_models.iter().any(|val| {
                val.as_object()
                    .and_then(|m| m.get("model"))
                    .and_then(|v| v.as_str())
                    == Some(id.as_str())
            });
            if !still_present {
                stash.remove(&id);
            }
        }
    }

    (changed, found, registered_routes)
}

fn droid_entry_managed(em: &OrderedMap, desired: &OrderedMap) -> bool {
    let mut normalized = OrderedMap::new();
    for (k, v) in em {
        if matches!(k.as_str(), "id" | "index" | "noImageSupport") {
            continue;
        }
        normalized.insert(k.clone(), v.clone());
    }
    Value::Object(normalized) == Value::Object(desired.clone())
}

fn droid_proxy_entry(endpoint: &str) -> OrderedMap {
    let mut entry = OrderedMap::new();
    entry.insert("model".to_string(), Value::String(proxy_wire_model()));
    entry.insert(
        "displayName".to_string(),
        Value::String(DROID_PROXY_DISPLAY_NAME.to_string()),
    );
    entry.insert("baseUrl".to_string(), Value::String(endpoint.to_string()));
    entry.insert(
        "provider".to_string(),
        Value::String(DROID_PROXY_PROVIDER_KIND.to_string()),
    );
    let k = proxy_wire_key();
    if k != "tokless" {
        entry.insert("apiKey".to_string(), Value::String(k));
    }
    entry
}

pub fn configure_droid_proxy() -> (bool, String) {
    let mut changed = false;
    let mut file = droid_settings_file();
    if let Err(err) = with_proxy_route_stash_lock(|| {
        let (c, f) = configure_droid_proxy_locked();
        changed = c;
        file = f;
        Ok(())
    }) {
        L.err(&format!("droid proxy lock failed: {}", err));
    }
    (changed, file)
}

fn configure_droid_proxy_locked() -> (bool, String) {
    let f = droid_settings_file();
    if !proxy_route_stash_valid("droid") {
        return (false, f);
    }
    let _ = ensure_dir(droid_dir());
    let raw_opt = read_file_safe(&f);
    let file_exists = raw_opt.is_some();
    if !file_exists && exists(&f) {
        return (false, f);
    }
    let raw = raw_opt.unwrap_or_default();
    if has_jsonc_comments(&raw) {
        return (false, f);
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => {
            if file_exists {
                return (false, f);
            }
            OrderedMap::new()
        }
    };
    let mut stash = load_proxy_route_stash_locked("droid");
    let stash_opt = read_file_safe(proxy_route_stash_path("droid"));
    let stash_exists = stash_opt.is_some();
    let stash_raw = stash_opt.unwrap_or_default();
    let prev_stash_len = stash.len();
    let (native_changed, native_found, registered_routes) =
        droid_native_routes(&mut cfg, &mut stash);
    if native_found {
        if save_proxy_route_stash("droid", &stash).is_err() {
            for r in &registered_routes {
                rollback_byok_route_logged("droid", &r.id, r.previous.as_ref());
            }
            return (false, f);
        }
        if !native_changed {
            for r in &registered_routes {
                rollback_byok_route_logged("droid", &r.id, r.previous.as_ref());
            }
            return (false, f);
        }
        if write_file(&f, &stringify_json(&Value::Object(cfg))).is_err() {
            restore_proxy_route_stash_logged("droid", &stash_raw, stash_exists);
            for r in &registered_routes {
                rollback_byok_route_logged("droid", &r.id, r.previous.as_ref());
            }
            return (false, f);
        }
        return (true, f);
    } else if prev_stash_len > 0 {
        let _ = save_proxy_route_stash("droid", &stash);
        return (false, f);
    }

    let endpoint = proxy_endpoint_for("droid");
    let desired = droid_proxy_entry(&endpoint);
    let mut models = match cfg.get("customModels") {
        Some(Value::Array(arr)) => arr.clone(),
        Some(_) => return (false, f),
        None => Vec::new(),
    };
    let wire_model = proxy_wire_model();
    for existing in &models {
        let Some(em) = existing.as_object() else {
            continue;
        };
        let model = em.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let display = em.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
        if model == wire_model && display == DROID_PROXY_DISPLAY_NAME {
            if Value::Object(em.clone()) == Value::Object(desired.clone()) {
                return (false, f);
            }
            return (false, f);
        }
    }
    models.push(Value::Object(desired));
    cfg.insert("customModels".to_string(), Value::Array(models));
    if write_file(&f, &stringify_json(&Value::Object(cfg))).is_err() {
        return (false, f);
    }
    (true, f)
}

pub fn remove_droid_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_droid_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("droid proxy lock failed: {}", err));
    }
    removed
}

fn remove_droid_proxy_locked() -> bool {
    if !proxy_route_stash_valid("droid") {
        return false;
    }
    let f = droid_settings_file();
    let stash = load_proxy_route_stash_locked("droid");
    if !stash.is_empty() {
        let Some(raw) = read_file_safe(&f) else {
            return false;
        };
        if has_jsonc_comments(&raw) {
            return false;
        }
        let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) else {
            return false;
        };
        let Some(Value::Array(mut models)) = cfg.get("customModels").cloned() else {
            return false;
        };
        let mut remaining: HashMap<String, ProxyRouteStashEntry> = HashMap::new();
        let original = raw.clone();
        let mut changed = false;
        for val in &mut models {
            let Some(model) = val.as_object_mut() else {
                continue;
            };
            let Some((id, base, mut headers)) = droid_model_route(model) else {
                if let Some(model_id) = model.get("model").and_then(|v| v.as_str()) {
                    if let Some(entry) = stash.get(model_id) {
                        remaining.insert(model_id.to_string(), entry.clone());
                    }
                }
                continue;
            };
            let Some(entry) = stash.get(&id) else {
                continue;
            };
            let header_ok = headers.contains_key(HEADROOM_BASE_URL_HEADER);
            let route_header = headers.get("X-Tokless-Route").and_then(|v| v.as_str());
            let matches_proxy = same_proxy_base(&base, &proxy_endpoint_for("droid")) && header_ok;
            let matches_byok = if let Some(route) = read_byok_route(&format!("droid:{}", id)) {
                droid_gateway_base(&base) && route_header == Some(&byok_route_header(&route))
            } else {
                false
            };
            if !matches_proxy && !matches_byok {
                remaining.insert(id, entry.clone());
                continue;
            }
            if let Some(val) = headers.get(HEADROOM_BASE_URL_HEADER) {
                if val.as_str() != Some(&entry.upstream) {
                    remaining.insert(id, entry.clone());
                    continue;
                }
            }
            model.insert("baseUrl".to_string(), Value::String(entry.base_url.clone()));
            if entry.had_header {
                headers.insert(
                    HEADROOM_BASE_URL_HEADER.to_string(),
                    Value::String(entry.header.clone()),
                );
            } else {
                headers.remove(HEADROOM_BASE_URL_HEADER);
            }
            headers.remove("X-Tokless-Route");
            if headers.is_empty() {
                model.remove("extraHeaders");
            } else {
                model.insert("extraHeaders".to_string(), Value::Object(headers));
            }
            changed = true;
        }
        if !changed {
            return false;
        }
        cfg.insert("customModels".to_string(), Value::Array(models));
        if write_file(&f, &stringify_json(&Value::Object(cfg))).is_err() {
            return false;
        }
        let mut deleted_routes = Vec::new();
        for id in stash.keys() {
            if !remaining.contains_key(id) {
                if let Some(route) = read_byok_route(&format!("droid:{}", id)) {
                    deleted_routes.push(route);
                }
                if delete_byok_route("droid", id).is_err() {
                    restore_deleted_byok_routes(&deleted_routes);
                    let _ = write_file(&f, &original);
                    return false;
                }
            }
        }
        if let Err(err) = save_proxy_route_stash("droid", &remaining) {
            restore_deleted_byok_routes(&deleted_routes);
            if let Err(restore_err) = write_file(&f, &original) {
                L.err(&format!("droid proxy rollback failed: {}", restore_err));
            }
            L.err(&format!("droid proxy stash update failed: {}", err));
            return false;
        }
        return true;
    }

    let Some(raw) = read_file_safe(&f) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(Value::Array(models)) = cfg.get("customModels").cloned() else {
        return false;
    };
    let endpoint = proxy_endpoint_for("droid");
    let desired = droid_proxy_entry(&endpoint);
    let mut kept = Vec::with_capacity(models.len());
    let mut dropped = false;
    for entry in models {
        let Some(em) = entry.as_object() else {
            kept.push(entry);
            continue;
        };
        if droid_entry_managed(em, &desired) {
            dropped = true;
            continue;
        }
        kept.push(entry);
    }
    if !dropped {
        return false;
    }
    if kept.is_empty() {
        cfg.remove("customModels");
    } else {
        cfg.insert("customModels".to_string(), Value::Array(kept));
    }
    write_file(&f, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn droid_proxy_wired() -> bool {
    let Some(raw) = read_file_safe(droid_settings_file()) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    let stash = load_proxy_route_stash_locked("droid");
    if !stash.is_empty() {
        let Some(Value::Array(models)) = cfg.get("customModels") else {
            return false;
        };
        for (id, entry) in &stash {
            let mut found = false;
            for val in models {
                let Some(model) = val.as_object() else {
                    continue;
                };
                if model.get("model").and_then(|v| v.as_str()) != Some(id.as_str()) {
                    continue;
                }
                let Some((_, base, headers)) = droid_model_route(model) else {
                    break;
                };
                let upstream = headers
                    .get(HEADROOM_BASE_URL_HEADER)
                    .and_then(|v| v.as_str());
                let route_header = headers.get("X-Tokless-Route").and_then(|v| v.as_str());
                let matches_proxy = same_proxy_base(&base, &proxy_endpoint_for("droid"))
                    && upstream == Some(entry.upstream.as_str());
                let matches_byok = if let Some(route) = read_byok_route(&format!("droid:{}", id)) {
                    droid_gateway_base(&base) && route_header == Some(&byok_route_header(&route))
                } else {
                    false
                };
                if matches_proxy || matches_byok {
                    found = true;
                }
                break;
            }
            if !found {
                return false;
            }
        }
        return true;
    }
    let Some(Value::Array(models)) = cfg.get("customModels") else {
        return false;
    };
    let endpoint = proxy_endpoint_for("droid");
    let desired = droid_proxy_entry(&endpoint);
    for entry in models {
        let Some(em) = entry.as_object() else {
            continue;
        };
        if droid_entry_managed(em, &desired) {
            return true;
        }
    }
    false
}
