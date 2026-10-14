// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/claude.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::agents::proxy_protocol::proxy_endpoint_for;
use crate::util::byok::{
    byok_route_header, byok_route_protocol, delete_byok_route as util_delete_byok_route,
    read_byok_route, upsert_byok_route, BYOKRoute,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::jsonc::{has_jsonc_comments, stringify_json, try_parse_jsonc, OrderedMap, Value};
use crate::util::logger::L;
use crate::util::mcpspawn::{pick_mcp_spawn, wrap_auto_index, McpSpawn};
use crate::util::paths::{
    claude_code_paths, ensure_dir, exists, home, read_file_safe, write_file, write_file_atomic,
};

pub const CLAUDE_PROXY_ENV_KEY: &str = "ANTHROPIC_BASE_URL";
pub const CLAUDE_PROXY_HEADER_ENV_KEY: &str = "ANTHROPIC_CUSTOM_HEADERS";
pub const CLAUDE_API_KEY_ENV_KEY: &str = "ANTHROPIC_API_KEY";
pub const CLAUDE_AUTH_TOKEN_ENV_KEY: &str = "ANTHROPIC_AUTH_TOKEN";

const CLAUDE_PROXY_HEADER_NAME: &str = "x-headroom-base-url";
const CLAUDE_PROXY_HEADER_LINE: &str = "x-headroom-base-url: ";
const CLAUDE_BYOK_STASH_AGENT: &str = "claude";
const CLAUDE_BYOK_STASH_PROVIDER: &str = "claude";

const PROJECTMEM_TOOL_NAMES: &[&str] = &[
    "log_issue",
    "record_attempt",
    "record_fix",
    "add_decision",
    "add_note",
];

static CLAUDE_STASH_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClaudeByokStashEntry {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub upstream: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_key: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub had_base_key: bool,
    #[serde(default, rename = "had_auth_token", skip_serializing_if = "is_false")]
    pub had_auth: bool,
    #[serde(default, rename = "moved_base_key", skip_serializing_if = "is_false")]
    pub moved_base: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub had_header: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_line: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header_line: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub managed_native: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub original: Vec<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub managed: Vec<u8>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProxyRouteStashFile {
    pub providers: HashMap<String, ClaudeByokStashEntry>,
}

fn mcp_spawn_for(tool_id: &str) -> McpSpawn {
    pick_mcp_spawn(tool_id, &[])
}

fn to_value_vec(ss: &[String]) -> Vec<Value> {
    ss.iter().map(|s| Value::String(s.clone())).collect()
}

fn parse_strict_json_object(raw: &str) -> Option<OrderedMap> {
    if raw.trim().is_empty() {
        return Some(OrderedMap::new());
    }
    match try_parse_jsonc(raw) {
        Some(Value::Object(m)) => Some(m),
        _ => None,
    }
}

fn claude_mcp_equal(existing: &Value, desired: &OrderedMap) -> bool {
    let em = match existing.as_object() {
        Some(m) => m,
        None => return false,
    };
    if em.get("command") != desired.get("command") {
        return false;
    }
    let empty_arr = Value::Array(Vec::new());
    let args_a = em.get("args").unwrap_or(&empty_arr);
    let args_b = desired.get("args").unwrap_or(&empty_arr);
    if args_a != args_b {
        return false;
    }
    let empty_obj = Value::Object(OrderedMap::new());
    let env_a = em.get("env").unwrap_or(&empty_obj);
    let env_b = desired.get("env").unwrap_or(&empty_obj);
    env_a == env_b
}

pub fn configure_claude_mcp(tool_id: &str) -> (bool, String) {
    let p = claude_code_paths();
    let _ = ensure_dir(&p.dir);
    let (raw, file_exists) = match read_file_safe(&p.global_json) {
        Some(r) => (r, true),
        None => (String::new(), false),
    };
    if !file_exists && exists(&p.global_json) {
        return (false, p.global_json);
    }
    if has_jsonc_comments(&raw) {
        return (false, p.global_json);
    }
    let mut cfg = match parse_strict_json_object(&raw) {
        Some(m) => m,
        None => return (false, p.global_json),
    };

    if let Some(v) = cfg.get("mcpServers") {
        if !v.is_object() {
            return (false, p.global_json);
        }
    } else {
        cfg.insert("mcpServers".into(), Value::Object(OrderedMap::new()));
    }

    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("claude", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };

    let mut desired = OrderedMap::new();
    desired.insert("type".into(), Value::String("stdio".into()));
    desired.insert("command".into(), Value::String(spawn.command));
    desired.insert("args".into(), Value::Array(to_value_vec(&spawn.args)));

    let servers = cfg
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
        .unwrap();

    if let Some(existing) = servers.get(tool_id) {
        if claude_mcp_equal(existing, &desired) {
            allow_claude_mcp_tool(tool_id);
            allow_claude_mcp_tool_project_local(tool_id);
            return (false, p.global_json);
        }
        return (false, p.global_json);
    }

    servers.insert(tool_id.into(), Value::Object(desired));
    if write_file(&p.global_json, &stringify_json(&Value::Object(cfg))).is_err() {
        return (false, p.global_json);
    }
    allow_claude_mcp_tool(tool_id);
    allow_claude_mcp_tool_project_local(tool_id);
    (true, p.global_json)
}

pub fn allow_claude_mcp_tool_project_local(tool_id: &str) {
    let entries = claude_mcp_permission_entries(tool_id);
    allow_claude_project_local_entries(tool_id, &entries);
}

pub fn claude_mcp_permission_entries(tool_id: &str) -> Vec<String> {
    if tool_id == "codegraph" {
        return vec![
            "mcp__codegraph__.*".into(),
            "mcp__codegraph__codegraph_explore".into(),
        ];
    }
    let names = claude_mcp_tool_names(tool_id);
    if !names.is_empty() {
        return names
            .into_iter()
            .map(|name| format!("mcp__{}__{}", tool_id, name))
            .collect();
    }
    vec![format!("mcp__{}__.*", tool_id)]
}

pub fn claude_mcp_tool_names(tool_id: &str) -> Vec<String> {
    match tool_id {
        "context-mode" => vec![
            "ctx_search".into(),
            "ctx_execute".into(),
            "ctx_execute_file".into(),
            "ctx_batch_execute".into(),
            "ctx_index".into(),
            "ctx_fetch_and_index".into(),
        ],
        "codegraph" => vec!["codegraph_explore".into()],
        "projectmem" => PROJECTMEM_TOOL_NAMES.iter().map(|s| (*s).into()).collect(),
        _ => Vec::new(),
    }
}

fn allow_claude_project_local_entries(tool_id: &str, entries: &[String]) {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let dir = cwd.join(".claude");
    let _ = ensure_dir(&dir);
    let settings_file = dir.join("settings.local.json").to_string_lossy().to_string();

    let (raw, file_exists) = match read_file_safe(&settings_file) {
        Some(r) => (r, true),
        None => (String::new(), false),
    };
    if !file_exists && exists(&settings_file) {
        return;
    }
    if has_jsonc_comments(&raw) {
        return;
    }
    let mut cfg = match parse_strict_json_object(&raw) {
        Some(m) => m,
        None => return,
    };

    if let Some(v) = cfg.get("permissions") {
        if !v.is_object() {
            return;
        }
    } else {
        cfg.insert("permissions".into(), Value::Object(OrderedMap::new()));
    }

    let perms = cfg
        .get_mut("permissions")
        .and_then(|v| v.as_object_mut())
        .unwrap();

    let mut allow = match perms.get("allow") {
        Some(Value::Array(a)) => a.clone(),
        Some(_) => return,
        None => Vec::new(),
    };

    let before_wildcard = allow.len();
    allow = remove_claude_context_mode_wildcard(allow);
    let wildcard_removed = allow.len() != before_wildcard;

    let before = allow.len();
    allow = remove_claude_mcp_tool_entries(allow, tool_id);
    let removed = allow.len() != before;

    let mut seen: HashSet<String> = allow
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let mut changed = removed || wildcard_removed;
    for entry in entries {
        if seen.contains(entry) {
            continue;
        }
        allow.push(Value::String(entry.clone()));
        seen.insert(entry.clone());
        changed = true;
    }
    if !changed {
        return;
    }
    perms.insert("allow".into(), Value::Array(allow));
    let _ = write_file(&settings_file, &stringify_json(&Value::Object(cfg)));
}

pub fn allow_claude_mcp_tool(tool_id: &str) {
    let p = claude_code_paths();
    let (raw, file_exists) = match read_file_safe(&p.settings) {
        Some(r) => (r, true),
        None => (String::new(), false),
    };
    if !file_exists && exists(&p.settings) {
        return;
    }
    if has_jsonc_comments(&raw) {
        return;
    }
    let mut cfg = match parse_strict_json_object(&raw) {
        Some(m) => m,
        None => return,
    };

    if let Some(v) = cfg.get("permissions") {
        if !v.is_object() {
            return;
        }
    } else {
        cfg.insert("permissions".into(), Value::Object(OrderedMap::new()));
    }

    let perms = cfg
        .get_mut("permissions")
        .and_then(|v| v.as_object_mut())
        .unwrap();

    let mut allow = match perms.get("allow") {
        Some(Value::Array(a)) => a.clone(),
        Some(_) => return,
        None => Vec::new(),
    };

    let before_wildcard = allow.len();
    allow = remove_claude_context_mode_wildcard(allow);
    let wildcard_changed = allow.len() != before_wildcard;

    let before = allow.len();
    allow = remove_claude_mcp_tool_entries(allow, tool_id);
    let removed = allow.len() != before;

    let seen: HashSet<String> = allow
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let mut changed = removed || wildcard_changed;
    for entry in claude_mcp_permission_entries(tool_id) {
        if !seen.contains(&entry) {
            allow.push(Value::String(entry));
            changed = true;
        }
    }
    if !changed {
        return;
    }
    perms.insert("allow".into(), Value::Array(allow));
    let _ = write_file(&p.settings, &stringify_json(&Value::Object(cfg)));
}

pub fn remove_claude_context_mode_wildcard(entries: Vec<Value>) -> Vec<Value> {
    entries
        .into_iter()
        .filter(|entry| entry.as_str() != Some("mcp__context-mode__.*"))
        .collect()
}

fn remove_claude_mcp_tool_entries(entries: Vec<Value>, tool_id: &str) -> Vec<Value> {
    let prefix = format!("mcp__{}__", tool_id);
    entries
        .into_iter()
        .filter(|entry| match entry.as_str() {
            Some(s) => !s.starts_with(&prefix),
            None => true,
        })
        .collect()
}

pub fn disallow_claude_mcp_tool(tool_id: &str) {
    let p = claude_code_paths();
    let raw = match read_file_safe(&p.settings) {
        Some(r) => r,
        None => return,
    };
    if has_jsonc_comments(&raw) {
        return;
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return,
    };
    let perms = match cfg.get_mut("permissions").and_then(|v| v.as_object_mut()) {
        Some(m) => m,
        None => return,
    };
    let allow = match perms.get("allow").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return,
    };
    let mut wants: HashSet<String> = claude_mcp_permission_entries(tool_id).into_iter().collect();
    if tool_id == "context-mode" {
        wants.insert("mcp__context-mode__.*".into());
    }
    let mut kept = Vec::with_capacity(allow.len());
    let mut changed = false;
    for x in allow {
        if let Some(s) = x.as_str() {
            if wants.contains(s) {
                changed = true;
                continue;
            }
        }
        kept.push(x);
    }
    if !changed {
        return;
    }
    perms.insert("allow".into(), Value::Array(kept));
    let _ = write_file(&p.settings, &stringify_json(&Value::Object(cfg)));
}

pub fn allow_claude_bash_pattern_project_local(pattern: &str) {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let dir = cwd.join(".claude");
    let _ = ensure_dir(&dir);
    let settings_file = dir.join("settings.local.json").to_string_lossy().to_string();

    let (raw, file_exists) = match read_file_safe(&settings_file) {
        Some(r) => (r, true),
        None => (String::new(), false),
    };
    if !file_exists && exists(&settings_file) {
        return;
    }
    if has_jsonc_comments(&raw) {
        return;
    }
    let mut cfg = match parse_strict_json_object(&raw) {
        Some(m) => m,
        None => return,
    };

    if let Some(v) = cfg.get("permissions") {
        if !v.is_object() {
            return;
        }
    } else {
        cfg.insert("permissions".into(), Value::Object(OrderedMap::new()));
    }

    let perms = cfg
        .get_mut("permissions")
        .and_then(|v| v.as_object_mut())
        .unwrap();

    let mut allow = match perms.get("allow") {
        Some(Value::Array(a)) => a.clone(),
        Some(_) => return,
        None => Vec::new(),
    };

    if allow.iter().any(|x| x.as_str() == Some(pattern)) {
        return;
    }
    allow.push(Value::String(pattern.to_string()));
    perms.insert("allow".into(), Value::Array(allow));
    let _ = write_file(&settings_file, &stringify_json(&Value::Object(cfg)));
}

pub fn allow_claude_bash_pattern(pattern: &str) {
    let p = claude_code_paths();
    let (raw, file_exists) = match read_file_safe(&p.settings) {
        Some(r) => (r, true),
        None => (String::new(), false),
    };
    if !file_exists && exists(&p.settings) {
        return;
    }
    if has_jsonc_comments(&raw) {
        return;
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => OrderedMap::new(),
    };
    if !cfg.get("permissions").map(|v| v.is_object()).unwrap_or(false) {
        cfg.insert("permissions".into(), Value::Object(OrderedMap::new()));
    }
    let perms = cfg
        .get_mut("permissions")
        .and_then(|v| v.as_object_mut())
        .unwrap();
    let mut allow = perms
        .get("allow")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if allow.iter().any(|x| x.as_str() == Some(pattern)) {
        return;
    }
    allow.push(Value::String(pattern.to_string()));
    perms.insert("allow".into(), Value::Array(allow));
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
    let perms = match cfg.get_mut("permissions").and_then(|v| v.as_object_mut()) {
        Some(m) => m,
        None => return,
    };
    let arr = match perms.get("allow").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return,
    };
    let mut out = Vec::with_capacity(arr.len());
    let mut dropped = false;
    for e in arr {
        if e.as_str() == Some(pattern) {
            dropped = true;
            continue;
        }
        out.push(e);
    }
    if !dropped {
        return;
    }
    if out.is_empty() {
        perms.remove("allow");
    } else {
        perms.insert("allow".into(), Value::Array(out));
    }
    let _ = write_file(&p.settings, &stringify_json(&Value::Object(cfg)));
}

pub fn remove_claude_mcp(tool_id: &str) -> bool {
    let p = claude_code_paths();
    let mut removed = false;
    if let Some(raw) = read_file_safe(&p.global_json) {
        if has_jsonc_comments(&raw) {
            return false;
        }
        if let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) {
            let can_remove = if let Some(Value::Object(sm)) = cfg.get_mut("mcpServers") {
                if let Some(existing) = sm.get(tool_id) {
                    let spawn = if tool_id == "codegraph" {
                        wrap_auto_index("claude", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
                    } else {
                        mcp_spawn_for(tool_id)
                    };
                    let mut desired = OrderedMap::new();
                    desired.insert("type".into(), Value::String("stdio".into()));
                    desired.insert("command".into(), Value::String(spawn.command));
                    desired.insert("args".into(), Value::Array(to_value_vec(&spawn.args)));
                    if !claude_mcp_equal(existing, &desired) {
                        return false;
                    }
                    sm.remove(tool_id);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if can_remove {
                let _ = write_file(&p.global_json, &stringify_json(&Value::Object(cfg)));
                removed = true;
            }
        }
    }
    disallow_claude_mcp_tool(tool_id);
    removed
}

// --- Claude headroom HTTP proxy & BYOK ---

fn proxy_route_stash_path(agent: &str) -> String {
    Path::new(&headroom_paths_resolved().root)
        .join(format!("{}.byok.stash.json", agent))
        .to_string_lossy()
        .to_string()
}

fn with_proxy_route_stash_lock<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R,
{
    let _guard = CLAUDE_STASH_LOCK
        .lock()
        .map_err(|e| format!("lock poisoned: {}", e))?;
    ensure_dir(&headroom_paths_resolved().root).map_err(|e| e.to_string())?;
    Ok(f())
}

fn proxy_route_stash_valid(agent: &str) -> bool {
    let raw = match read_file_safe(proxy_route_stash_path(agent)) {
        Some(r) => r,
        None => return true,
    };
    serde_json::from_str::<ProxyRouteStashFile>(&raw).is_ok()
}

fn load_proxy_route_stash_locked(agent: &str) -> HashMap<String, ClaudeByokStashEntry> {
    let raw = match read_file_safe(proxy_route_stash_path(agent)) {
        Some(r) => r,
        None => return HashMap::new(),
    };
    serde_json::from_str::<ProxyRouteStashFile>(&raw)
        .map(|f| f.providers)
        .unwrap_or_default()
}

fn save_proxy_route_stash(
    agent: &str,
    providers: &HashMap<String, ClaudeByokStashEntry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = proxy_route_stash_path(agent);
    if providers.is_empty() {
        if let Some(raw) = read_file_safe(&path) {
            if serde_json::from_str::<ProxyRouteStashFile>(&raw).is_err() {
                return Err(format!("refusing to replace malformed proxy stash {}", path).into());
            }
        }
        if Path::new(&path).exists() {
            std::fs::remove_file(&path)?;
        }
        return Ok(());
    }
    let content = serde_json::to_string(&ProxyRouteStashFile {
        providers: providers.clone(),
    })?;
    write_file_atomic(&path, &content)?;
    Ok(())
}

fn strip_v1_suffix(u: &str) -> String {
    let s = u.trim_end_matches('/');
    s.strip_suffix("/v1").unwrap_or(s).to_string()
}

fn same_proxy_base(a: &str, b: &str) -> bool {
    let a_clean = a.to_lowercase().trim_end_matches('/').to_string();
    let b_clean = b.to_lowercase().trim_end_matches('/').to_string();
    !a_clean.is_empty() && a_clean == b_clean
}

fn register_byok_route(
    agent: &str,
    id: &str,
    api: &str,
    upstream: &str,
) -> Option<(String, String, Option<BYOKRoute>)> {
    let protocol = byok_route_protocol(api);
    if protocol.is_empty() || upstream.trim().is_empty() {
        return None;
    }
    let namespaced_id = format!("{}:{}", agent, id);
    let (route, previous) = upsert_byok_route(&namespaced_id, protocol, upstream).ok()?;
    Some((
        crate::util::byok::byok_gateway_endpoint(),
        byok_route_header(&route),
        previous,
    ))
}

fn rollback_byok_route_logged(agent: &str, id: &str, previous: Option<BYOKRoute>) -> bool {
    let namespaced_id = format!("{}:{}", agent, id);
    let res = match previous {
        None => util_delete_byok_route(&namespaced_id),
        Some(prev) => upsert_byok_route(&namespaced_id, &prev.protocol, &prev.upstream).map(|_| ()),
    };
    if let Err(err) = res {
        L.err(&format!("{} BYOK route rollback failed: {}", agent, err));
        return false;
    }
    true
}

fn delete_byok_route(agent: &str, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let namespaced_id = format!("{}:{}", agent, id);
    util_delete_byok_route(&namespaced_id)
}

fn restore_deleted_byok_routes(routes: &[BYOKRoute]) {
    for route in routes {
        if let Err(err) = upsert_byok_route(&route.id, &route.protocol, &route.upstream) {
            L.err(&format!("BYOK route restore failed: {}", err));
        }
    }
}

pub fn configure_claude_proxy() -> (bool, String) {
    let default_file = claude_code_paths().settings;
    match with_proxy_route_stash_lock(configure_claude_proxy_locked) {
        Ok(res) => res,
        Err(err) => {
            L.err(&format!("claude proxy lock failed: {}", err));
            (false, default_file)
        }
    }
}

fn configure_claude_proxy_locked() -> (bool, String) {
    let p = claude_code_paths();
    if !proxy_route_stash_valid(CLAUDE_BYOK_STASH_AGENT) {
        return (false, p.settings);
    }
    let url = proxy_endpoint_for("claude");
    let _ = ensure_dir(&p.dir);
    let (raw, ok) = match read_file_safe(&p.settings) {
        Some(r) => (r, true),
        None => (String::new(), false),
    };
    if has_jsonc_comments(&raw) {
        return (false, p.settings);
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => {
            if ok {
                return (false, p.settings);
            }
            OrderedMap::new()
        }
    };
    let mut env = match cfg.get("env") {
        Some(Value::Object(em)) => em.clone(),
        Some(_) => return (false, p.settings),
        None => OrderedMap::new(),
    };

    let mut dirty = false;
    if let Some(v) = env.get(CLAUDE_PROXY_ENV_KEY) {
        let s = match v.as_str() {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => return (false, p.settings),
        };
        if s != url {
            if !claude_takeover_byok(&mut cfg, &mut env, &s) {
                return (false, p.settings);
            }
            return (true, p.settings);
        }
    } else {
        env.insert(CLAUDE_PROXY_ENV_KEY.into(), Value::String(url.clone()));
        dirty = true;
    }

    if !dirty {
        return (false, p.settings);
    }
    cfg.insert("env".into(), Value::Object(env));
    let managed = stringify_json(&Value::Object(cfg));
    let mut stash = load_proxy_route_stash_locked(CLAUDE_BYOK_STASH_AGENT);
    let previous_stash = stash.clone();
    let mut entry = ClaudeByokStashEntry {
        file: p.settings.clone(),
        provider: CLAUDE_BYOK_STASH_PROVIDER.into(),
        base_url: url,
        managed_native: true,
        managed: managed.as_bytes().to_vec(),
        ..Default::default()
    };
    if ok {
        entry.original = raw.into_bytes();
    }
    if save_claude_byok_stash_locked(&entry, &mut stash).is_err() {
        return (false, p.settings);
    }
    if write_file_atomic(&p.settings, &managed).is_err() {
        let _ = save_proxy_route_stash(CLAUDE_BYOK_STASH_AGENT, &previous_stash);
        return (false, p.settings);
    }
    (true, p.settings)
}

pub fn remove_claude_proxy() -> bool {
    match with_proxy_route_stash_lock(remove_claude_proxy_locked) {
        Ok(removed) => removed,
        Err(err) => {
            L.err(&format!("claude proxy lock failed: {}", err));
            false
        }
    }
}

fn remove_claude_proxy_locked() -> bool {
    if !proxy_route_stash_valid(CLAUDE_BYOK_STASH_AGENT) {
        return false;
    }
    let url = proxy_endpoint_for("claude");
    let p = claude_code_paths();
    let raw = match read_file_safe(&p.settings) {
        Some(r) => r,
        None => return false,
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    let env = match cfg.get("env").and_then(|v| v.as_object()) {
        Some(m) => m,
        None => return false,
    };
    let s = match env.get(CLAUDE_PROXY_ENV_KEY).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    let entry = match load_claude_byok_stash_locked() {
        Some(e) => e,
        None => return false,
    };
    if s != url && !claude_owned_leftover(s, &entry, &raw) {
        return false;
    }
    claude_restore_byok(&cfg, env)
}

pub fn claude_proxy_wired() -> bool {
    let url = proxy_endpoint_for("claude");
    let raw = match read_file_safe(claude_code_paths().settings) {
        Some(r) => r,
        None => return false,
    };
    let cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    let env = match cfg.get("env").and_then(|v| v.as_object()) {
        Some(m) => m,
        None => return false,
    };
    env.get(CLAUDE_PROXY_ENV_KEY).and_then(|v| v.as_str()) == Some(url.as_str())
}

pub fn claude_byok_upstream(env: &OrderedMap) -> String {
    let s = match env.get(CLAUDE_PROXY_ENV_KEY).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return String::new(),
    };
    if s.is_empty() || s == proxy_endpoint_for("claude") {
        return String::new();
    }
    s.to_string()
}

pub fn claude_custom_headers_lines(env: &OrderedMap) -> Option<Vec<String>> {
    let s = env.get(CLAUDE_PROXY_HEADER_ENV_KEY)?.as_str()?;
    let lines = s
        .split('\n')
        .filter(|ln| !ln.trim().is_empty())
        .map(|ln| ln.to_string())
        .collect();
    Some(lines)
}

pub fn claude_custom_headers_set(env: &mut OrderedMap, lines: &[String]) {
    if lines.is_empty() {
        env.remove(CLAUDE_PROXY_HEADER_ENV_KEY);
    } else {
        env.insert(
            CLAUDE_PROXY_HEADER_ENV_KEY.into(),
            Value::String(lines.join("\n")),
        );
    }
}

pub fn claude_custom_headers_drop(lines: &[String]) -> (Vec<String>, bool) {
    let mut kept = Vec::with_capacity(lines.len());
    let mut dropped = false;
    for ln in lines {
        if let Some((name, _)) = ln.split_once(':') {
            if name
                .trim_end_matches([' ', '\t', '\r'])
                .trim()
                .eq_ignore_ascii_case(CLAUDE_PROXY_HEADER_NAME)
            {
                dropped = true;
                continue;
            }
        } else if ln
            .trim_end_matches([' ', '\t', '\r'])
            .trim()
            .eq_ignore_ascii_case(CLAUDE_PROXY_HEADER_NAME)
        {
            dropped = true;
            continue;
        }
        kept.push(ln.clone());
    }
    (kept, dropped)
}

pub fn load_claude_byok_stash() -> Option<ClaudeByokStashEntry> {
    let all = load_proxy_route_stash_locked(CLAUDE_BYOK_STASH_AGENT);
    claude_byok_stash_from(&all)
}

fn load_claude_byok_stash_locked() -> Option<ClaudeByokStashEntry> {
    let all = load_proxy_route_stash_locked(CLAUDE_BYOK_STASH_AGENT);
    claude_byok_stash_from(&all)
}

fn claude_byok_stash_from(
    all: &HashMap<String, ClaudeByokStashEntry>,
) -> Option<ClaudeByokStashEntry> {
    let e = all.get(CLAUDE_BYOK_STASH_PROVIDER)?;
    if e.base_url.is_empty() || e.file.is_empty() {
        return None;
    }
    Some(e.clone())
}

pub fn save_claude_byok_stash(
    entry: &ClaudeByokStashEntry,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    with_proxy_route_stash_lock(|| {
        let mut all = load_proxy_route_stash_locked(CLAUDE_BYOK_STASH_AGENT);
        save_claude_byok_stash_locked(entry, &mut all)
    })
    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?
}

fn save_claude_byok_stash_locked(
    entry: &ClaudeByokStashEntry,
    all: &mut HashMap<String, ClaudeByokStashEntry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if entry.base_url.is_empty() {
        all.remove(CLAUDE_BYOK_STASH_PROVIDER);
    } else {
        all.insert(CLAUDE_BYOK_STASH_PROVIDER.into(), entry.clone());
    }
    save_proxy_route_stash(CLAUDE_BYOK_STASH_AGENT, all)
}

pub fn claude_takeover_byok(
    cfg: &mut OrderedMap,
    env: &mut OrderedMap,
    upstream: &str,
) -> bool {
    let settings_path = claude_code_paths().settings;
    let original = match read_file_safe(&settings_path) {
        Some(r) => r,
        None => return false,
    };
    let (lines, had_headers) = match claude_custom_headers_lines(env) {
        Some(l) => (l, true),
        None => (Vec::new(), false),
    };
    let prev_headers = if had_headers {
        lines.join("\n")
    } else {
        String::new()
    };
    let mut entry = ClaudeByokStashEntry {
        file: settings_path.clone(),
        provider: CLAUDE_BYOK_STASH_PROVIDER.into(),
        base_url: upstream.to_string(),
        had_header: had_headers,
        header: prev_headers,
        original: original.into_bytes(),
        ..Default::default()
    };
    if let Some(s) = env.get(CLAUDE_API_KEY_ENV_KEY).and_then(|v| v.as_str()) {
        entry.had_base_key = true;
        entry.base_key = s.to_string();
    }
    if let Some(s) = env.get(CLAUDE_AUTH_TOKEN_ENV_KEY).and_then(|v| v.as_str()) {
        entry.had_auth = true;
        entry.base_line = s.to_string();
    }
    let mut stash = load_proxy_route_stash_locked(CLAUDE_BYOK_STASH_AGENT);
    let previous_stash = stash.clone();
    if !entry.base_key.is_empty() {
        entry.moved_base = true;
        env.remove(CLAUDE_API_KEY_ENV_KEY);
        env.insert(
            CLAUDE_AUTH_TOKEN_ENV_KEY.into(),
            Value::String(entry.base_key.clone()),
        );
    }
    let normalized_upstream = strip_v1_suffix(upstream);
    let mut previous_route: Option<BYOKRoute> = None;
    let mut route_registered = false;
    if normalized_upstream.starts_with("https://") {
        match register_byok_route(
            "claude",
            CLAUDE_BYOK_STASH_PROVIDER,
            "anthropic-messages",
            &normalized_upstream,
        ) {
            Some((_, _, prev)) => {
                previous_route = prev;
                route_registered = true;
            }
            None => return false,
        }
    } else if let Some((_, _, prev)) = register_byok_route(
        "claude",
        CLAUDE_BYOK_STASH_PROVIDER,
        "anthropic-messages",
        &normalized_upstream,
    ) {
        previous_route = prev;
        route_registered = true;
    }

    env.insert(
        CLAUDE_PROXY_ENV_KEY.into(),
        Value::String(proxy_endpoint_for("claude")),
    );
    let (mut kept, _) = claude_custom_headers_drop(&lines);
    kept.push(format!("{}{}", CLAUDE_PROXY_HEADER_LINE, normalized_upstream));
    claude_custom_headers_set(env, &kept);
    cfg.insert("env".into(), Value::Object(env.clone()));
    let managed = stringify_json(&Value::Object(cfg.clone()));
    entry.managed = managed.as_bytes().to_vec();
    if save_claude_byok_stash_locked(&entry, &mut stash).is_err() {
        if route_registered {
            rollback_byok_route_logged("claude", CLAUDE_BYOK_STASH_PROVIDER, previous_route);
        }
        return false;
    }
    if write_file_atomic(&settings_path, &managed).is_err() {
        let _ = save_proxy_route_stash(CLAUDE_BYOK_STASH_AGENT, &previous_stash);
        if route_registered {
            rollback_byok_route_logged("claude", CLAUDE_BYOK_STASH_PROVIDER, previous_route);
        }
        return false;
    }
    true
}

pub fn claude_restore_byok(_cfg: &OrderedMap, _env: &OrderedMap) -> bool {
    let mut stash = load_proxy_route_stash_locked(CLAUDE_BYOK_STASH_AGENT);
    let entry = match claude_byok_stash_from(&stash) {
        Some(e) => e,
        None => return false,
    };
    let settings_path = claude_code_paths().settings;
    let current = match read_file_safe(&settings_path) {
        Some(c) => c,
        None => return false,
    };
    if !entry.original.is_empty()
        && !entry.managed.is_empty()
        && current.as_bytes() == entry.managed.as_slice()
    {
        let prev_route = read_byok_route(&format!("claude:{}", CLAUDE_BYOK_STASH_PROVIDER));
        if delete_byok_route("claude", CLAUDE_BYOK_STASH_PROVIDER).is_err() {
            return false;
        }
        let orig_str = String::from_utf8_lossy(&entry.original);
        if write_file_atomic(&settings_path, &orig_str).is_err() {
            if let Some(prev) = prev_route {
                restore_deleted_byok_routes(&[prev]);
            }
            return false;
        }
        if save_claude_byok_stash_locked(&ClaudeByokStashEntry::default(), &mut stash).is_err() {
            let _ = write_file_atomic(&settings_path, &current);
            if let Some(prev) = prev_route {
                restore_deleted_byok_routes(&[prev]);
            }
            return false;
        }
        return true;
    }
    if entry.managed_native {
        let mut cfg = match try_parse_jsonc(&current) {
            Some(Value::Object(m)) => m,
            _ => return false,
        };
        let env_empty = {
            let env = match cfg.get_mut("env").and_then(|v| v.as_object_mut()) {
                Some(m) => m,
                None => return false,
            };
            match env.get(CLAUDE_PROXY_ENV_KEY) {
                Some(v) if claude_owned_env_url(v, &entry) => {}
                _ => return false,
            }
            env.remove(CLAUDE_PROXY_ENV_KEY);
            env.is_empty()
        };
        if env_empty {
            cfg.remove("env");
        }
        let prev_route = read_byok_route(&format!("claude:{}", CLAUDE_BYOK_STASH_PROVIDER));
        if delete_byok_route("claude", CLAUDE_BYOK_STASH_PROVIDER).is_err() {
            return false;
        }
        if write_file_atomic(&settings_path, &stringify_json(&Value::Object(cfg))).is_err() {
            if let Some(prev) = prev_route {
                restore_deleted_byok_routes(&[prev]);
            }
            return false;
        }
        if save_claude_byok_stash_locked(&ClaudeByokStashEntry::default(), &mut stash).is_err() {
            let _ = write_file_atomic(&settings_path, &current);
            if let Some(prev) = prev_route {
                restore_deleted_byok_routes(&[prev]);
            }
            return false;
        }
        return true;
    }

    let mut cfg = match try_parse_jsonc(&current) {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    {
        let env = match cfg.get_mut("env").and_then(|v| v.as_object_mut()) {
            Some(m) => m,
            None => return false,
        };
        match env.get(CLAUDE_PROXY_ENV_KEY) {
            Some(v) if claude_owned_env_url(v, &entry) => {}
            _ => return false,
        }
        let lines = claude_custom_headers_lines(env).unwrap_or_default();
        let managed_header = format!(
            "{}{}",
            CLAUDE_PROXY_HEADER_LINE,
            strip_v1_suffix(&entry.base_url)
        );
        let kept: Vec<String> = lines
            .into_iter()
            .filter(|line| line != &managed_header)
            .collect();
        claude_custom_headers_set(env, &kept);
        env.insert(
            CLAUDE_PROXY_ENV_KEY.into(),
            Value::String(entry.base_url.clone()),
        );
        if entry.moved_base {
            let auth_matches = env
                .get(CLAUDE_AUTH_TOKEN_ENV_KEY)
                .and_then(|v| v.as_str())
                .map(|s| s == entry.base_key)
                .unwrap_or(false);
            let key_is_str = env
                .get(CLAUDE_API_KEY_ENV_KEY)
                .map(|v| v.is_string())
                .unwrap_or(false);
            if auth_matches && !key_is_str {
                if entry.had_auth {
                    env.insert(
                        CLAUDE_AUTH_TOKEN_ENV_KEY.into(),
                        Value::String(entry.base_line.clone()),
                    );
                } else {
                    env.remove(CLAUDE_AUTH_TOKEN_ENV_KEY);
                }
                if entry.had_base_key {
                    env.insert(
                        CLAUDE_API_KEY_ENV_KEY.into(),
                        Value::String(entry.base_key.clone()),
                    );
                }
            }
        }
    }
    let prev_route = read_byok_route(&format!("claude:{}", CLAUDE_BYOK_STASH_PROVIDER));
    if delete_byok_route("claude", CLAUDE_BYOK_STASH_PROVIDER).is_err() {
        return false;
    }
    if write_file_atomic(&settings_path, &stringify_json(&Value::Object(cfg))).is_err() {
        if let Some(prev) = prev_route {
            restore_deleted_byok_routes(&[prev]);
        }
        return false;
    }
    if save_claude_byok_stash_locked(&ClaudeByokStashEntry::default(), &mut stash).is_err() {
        let _ = write_file_atomic(&settings_path, &current);
        if let Some(prev) = prev_route {
            restore_deleted_byok_routes(&[prev]);
        }
        return false;
    }
    true
}

pub fn claude_owned_leftover(s: &str, entry: &ClaudeByokStashEntry, current: &str) -> bool {
    if entry.managed_native && same_proxy_base(s, &entry.base_url) {
        return true;
    }
    !entry.original.is_empty()
        && !entry.managed.is_empty()
        && current.as_bytes() == entry.managed.as_slice()
}

pub fn claude_owned_env_url(v: &Value, entry: &ClaudeByokStashEntry) -> bool {
    let s = match v.as_str() {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    if same_proxy_base(s, &proxy_endpoint_for("claude")) {
        return true;
    }
    entry.managed_native && same_proxy_base(s, &entry.base_url)
}

pub fn claude_known_bin_dirs() -> Vec<String> {
    vec![Path::new(&home())
        .join(".local")
        .join("bin")
        .to_string_lossy()
        .to_string()]
}

pub fn claude_desktop_paths() -> Vec<String> {
    if cfg!(windows) {
        let mut paths = Vec::new();
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                paths.push(
                    Path::new(&local)
                        .join("AnthropicClaude")
                        .join("claude.exe")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
        if let Ok(roam) = std::env::var("APPDATA") {
            if !roam.is_empty() {
                paths.push(
                    Path::new(&roam)
                        .join("Claude")
                        .join("claude.exe")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
        paths
    } else if cfg!(target_os = "macos") {
        vec!["/Applications/Claude.app".into()]
    } else {
        Vec::new()
    }
}
