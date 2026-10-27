// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/opencode.rs
// created: 2026-10-11
// lastModified: 2026-10-27

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::core::AgentManifest;
use crate::util::byok::{
    BYOKRoute, byok_gateway_endpoint, byok_gateway_port, byok_route_header,
    is_loopback_host, read_byok_route, read_byok_routes, upsert_byok_route,
};
use crate::util::grok::grok_oauth_proxy_port;
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::headroom_proxy_port;
use crate::util::jsonc::{OrderedMap, Value, has_jsonc_comments, stringify_json};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;

use super::proxy_detection::{ProxyConfigState, classify_open_code_jsonc_proxy};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    ByokRouteRollback, HEADROOM_BASE_URL_HEADER, ProviderSpec, ProxyRouteStashEntry,
    delete_byok_route, detect_agent_with, is_absolute_http, is_valid_byok_upstream, json_args_eq,
    json_equal, json_get_or_create_map, json_map_child, json_map_child_mut, load_proxy_route_stash,
    load_proxy_route_stash_locked, normalized_headroom_upstream, open_code_protocol,
    provider_base_and_key, provider_spec_active, proxy_endpoint_for_api, proxy_route_stash_path,
    proxy_route_stash_valid, proxy_wire_key, register_byok_route, resolve_api_key,
    restore_deleted_byok_routes, restore_proxy_route_stash_logged, rollback_byok_route_logged,
    same_proxy_base, save_proxy_route_stash, strings_to_values,
    try_parse_jsonc_map as try_parse_jsonc, with_proxy_route_stash_lock,
};

pub const TOKLESS_OPENCODE_BYOK_PLUGIN: &str = r#"// tokless-opencode-byok-plugin: managed by tokless; do not edit by hand.
const HEADROOM_BYOK_HEADER = "x-headroom-byok-route";
const HEADROOM_BASE_URL_HEADER = "x-headroom-base-url";

function headerEntries(headers) {
  if (!headers) return [];
  if (typeof headers.entries === "function") return Array.from(headers.entries());
  if (Array.isArray(headers)) return headers;
  if (typeof headers === "object") return Object.entries(headers);
  return [];
}

function getHeader(headers, name) {
  const lower = String(name || "").toLowerCase();
  for (const [k, v] of headerEntries(headers)) {
    if (String(k).toLowerCase() === lower && v != null) return String(v);
  }
  return "";
}

function isHeadroomGatewayUrl(rawUrl) {
  if (!rawUrl || typeof rawUrl !== "string") return false;
  try {
    const u = new URL(rawUrl);
    if (u.protocol !== "http:" && u.protocol !== "https:") return false;
    const host = (u.hostname || "").toLowerCase();
    return host === "127.0.0.1" || host === "localhost" || host === "::1" || host === "[::1]";
  } catch {
    return false;
  }
}

function wrapFetchWithByokHeader(baseFetch, routeHeaderValue) {
  const f = typeof baseFetch === "function" ? baseFetch : globalThis.fetch;
  if (typeof f !== "function") return f;
  return async function toklessByokFetch(input, init) {
    const reqUrl = typeof input === "string"
      ? input
      : input && typeof input.url === "string"
        ? input.url
        : String(input || "");
    if (!isHeadroomGatewayUrl(reqUrl)) {
      return f(input, init);
    }
    const nextInit = Object.assign({}, init || {});
    const merged = new Headers(
      (input && typeof input === "object" && input.headers) || undefined,
    );
    for (const [k, v] of headerEntries(init && init.headers)) {
      if (v != null) merged.set(k, String(v));
    }
    if (routeHeaderValue && !merged.has(HEADROOM_BYOK_HEADER)) {
      merged.set(HEADROOM_BYOK_HEADER, routeHeaderValue);
    }
    nextInit.headers = merged;
    return f(input, nextInit);
  };
}

export const ToklessHeadroomTransportPlugin = async () => {
  return {
    "chat.params": async (input, output) => {
      try {
        const providerOpts =
          (input && input.provider && input.provider.options) ||
          (input && input.model && input.model.provider && input.model.provider.options) ||
          {};
        const hdrs = providerOpts.headers || {};
        const routeHdr = getHeader(hdrs, HEADROOM_BYOK_HEADER);
        const baseHdr = getHeader(hdrs, HEADROOM_BASE_URL_HEADER);
        if (!routeHdr && !baseHdr) return;
        if (output && typeof output === "object") {
          output.options = Object.assign({}, output.options || {});
          output.options.headers = Object.assign({}, output.options.headers || {}, hdrs);
          if (routeHdr) {
            output.options.fetch = wrapFetchWithByokHeader(output.options.fetch, routeHdr);
          }
        }
      } catch {
        // never break chat invocation on plugin inspection errors
      }
    },
  };
};

export default ToklessHeadroomTransportPlugin;
"#;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenCodeBYOK {
    pub id: String,
    pub file: String,
    pub base_url: String,
    pub api_key: String,
    pub npm: String,
    pub protocol: String,
    pub route_header: String,
    pub native: bool,
    pub name: String,
    pub raw_api_key: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenCodeSkip {
    pub id: String,
    pub reason: String,
}

pub fn opencode_desktop_paths() -> Vec<String> {
    if cfg!(windows) {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                return vec![Path::new(&local)
                    .join("Programs")
                    .join("OpenCode")
                    .join("OpenCode.exe")
                    .to_string_lossy()
                    .into_owned()];
            }
        }
        Vec::new()
    } else if cfg!(target_os = "macos") {
        vec!["/Applications/OpenCode.app".to_string()]
    } else {
        vec!["/usr/bin/ai.opencode.desktop".to_string()]
    }
}

pub fn opencode_manifest() -> AgentManifest {
    AgentManifest {
        id: "opencode".to_string(),
        label: "OpenCode".to_string(),
        homepage: "https://github.com/anomalyco/opencode".to_string(),
        cli_bin: "opencode".to_string(),
        config_dir: Some(Arc::new(|| paths::opencode_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let bins = opencode_known_bin_dirs();
            let majors = open_code_installed_majors();
            if majors.contains(&1) {
                return crate::core::Detection {
                    installed: true,
                    source: "cli".to_string(),
                };
            }
            if !majors.is_empty() {
                return crate::core::Detection {
                    installed: false,
                    source: String::new(),
                };
            }
            detect_agent_with(
                "opencode",
                &paths::opencode_paths_resolved().dir,
                &bins,
                &opencode_desktop_paths(),
            )
        })),
    }
}

pub fn opencode_known_bin_dirs() -> Vec<String> {
    let mut dirs = vec![
        Path::new(&paths::home()).join(".opencode").join("bin").to_string_lossy().to_string(),
        Path::new(&paths::home()).join(".opencode2").join("bin").to_string_lossy().to_string(),
    ];
    for name in ["opencode", "opencode2"] {
        let p = paths::find_binary::<&str>(name, &[]);
        if !p.is_empty() {
            if let Some(parent) = Path::new(&p).parent() {
                dirs.push(parent.to_string_lossy().to_string());
            }
        }
    }
    dirs.push(Path::new(&paths::home()).join(".local").join("bin").to_string_lossy().to_string());
    if paths::IS_WIN {
        dirs.push(Path::new(&paths::home()).join("scoop").join("shims").to_string_lossy().to_string());
        if let Ok(pd) = std::env::var("ProgramData") {
            if !pd.is_empty() {
                dirs.push(Path::new(&pd).join("chocolatey").join("bin").to_string_lossy().to_string());
            }
        }
    }
    dirs
}

pub fn open_code_major_version(bin: &str) -> i32 {
    let output = match std::process::Command::new(bin).arg("--version").output() {
        Ok(o) => o,
        Err(_) => return 0,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let re = regex::Regex::new(r"(?:^|\s)v?(\d+)\.").unwrap();
    if let Some(caps) = re.captures(text.trim()) {
        if let Some(m) = caps.get(1) {
            if let Ok(major) = m.as_str().parse::<i32>() {
                return major;
            }
        }
    }
    0
}

pub fn open_code_binary_paths() -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut add = |p: String| {
        if !p.is_empty() && !seen.contains(&p) {
            seen.insert(p.clone());
            out.push(p);
        }
    };
    let names = if paths::IS_WIN {
        &["opencode2.exe", "opencode.exe", "opencode2", "opencode"][..]
    } else {
        &["opencode2", "opencode"][..]
    };
    for dir in opencode_known_bin_dirs() {
        for &name in names {
            add(Path::new(&dir).join(name).to_string_lossy().to_string());
        }
    }
    for &name in &["opencode2", "opencode"] {
        add(paths::find_binary::<&str>(name, &[]));
    }
    out
}

pub fn open_code_installed_major() -> i32 {
    for bin in open_code_binary_paths() {
        let major = open_code_major_version(&bin);
        if major != 0 {
            return major;
        }
    }
    0
}

pub fn open_code_installed_majors() -> Vec<i32> {
    let mut majors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for bin in open_code_binary_paths() {
        let major = open_code_major_version(&bin);
        if major != 0 && !seen.contains(&major) {
            seen.insert(major);
            majors.push(major);
        }
    }
    majors
}

pub fn opencode2_manifest() -> AgentManifest {
    AgentManifest {
        id: "opencode2".to_string(),
        label: "OpenCode v2".to_string(),
        homepage: "https://github.com/anomalyco/opencode".to_string(),
        cli_bin: "opencode2".to_string(),
        config_dir: Some(Arc::new(|| paths::opencode_paths_v2().dir)),
        detect: Some(Arc::new(|| {
            for major in open_code_installed_majors() {
                if major == 2 {
                    return crate::core::Detection {
                        installed: true,
                        source: "cli".to_string(),
                    };
                }
            }
            crate::core::Detection {
                installed: false,
                source: String::new(),
            }
        })),
    }
}

pub fn open_code_mcp_command(agent_id: &str, tool_id: &str) -> Vec<String> {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index(
            agent_id,
            pick_mcp_spawn("codegraph", &["serve", "--mcp"]),
        )
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut cmd = Vec::with_capacity(1 + spawn.args.len());
    cmd.push(spawn.command);
    cmd.extend(spawn.args);
    cmd
}

fn open_code_agent_id(v2: bool) -> &'static str {
    if v2 { "opencode2" } else { "opencode" }
}

fn open_code_mcp_entry_enabled(entry: &OrderedMap) -> bool {
    if entry.get("enabled").and_then(|v| v.as_bool()) == Some(false) {
        return false;
    }
    if entry.get("disabled").and_then(|v| v.as_bool()) == Some(true) {
        return false;
    }
    true
}

fn open_code_mcp_entry_command_matches(entry: &OrderedMap, command: &[String]) -> bool {
    json_args_eq(entry.get("command"), command)
}

fn open_code_mcp_root_entry<'a>(mcp: &'a OrderedMap, tool_id: &str) -> Option<&'a OrderedMap> {
    mcp.get(tool_id).and_then(|v| v.as_object())
}

fn open_code_mcp_server_entry<'a>(mcp: &'a OrderedMap, tool_id: &str) -> Option<&'a OrderedMap> {
    json_map_child(mcp, "servers").and_then(|s| s.get(tool_id)).and_then(|v| v.as_object())
}

pub fn open_code_mcp_has(cfg: &OrderedMap, tool_id: &str) -> bool {
    let Some(mcp) = json_map_child(cfg, "mcp") else {
        return false;
    };
    if let Some(em) = open_code_mcp_root_entry(mcp, tool_id)
        && open_code_mcp_entry_enabled(em)
    {
        return true;
    }
    if let Some(em) = open_code_mcp_server_entry(mcp, tool_id)
        && open_code_mcp_entry_enabled(em)
    {
        return true;
    }
    false
}

pub fn open_code_mcp_matches(cfg: &OrderedMap, tool_id: &str, command: &[String]) -> bool {
    let Some(mcp) = json_map_child(cfg, "mcp") else {
        return false;
    };
    if let Some(em) = open_code_mcp_root_entry(mcp, tool_id)
        && open_code_mcp_entry_enabled(em)
        && open_code_mcp_entry_command_matches(em, command)
    {
        return true;
    }
    if let Some(em) = open_code_mcp_server_entry(mcp, tool_id)
        && open_code_mcp_entry_enabled(em)
        && open_code_mcp_entry_command_matches(em, command)
    {
        return true;
    }
    false
}

fn configure_open_code_mcp_impl(tool_id: &str, file: &str, v2: bool) -> (bool, String) {
    let path = file.to_string();
    if let Some(parent) = Path::new(&path).parent()
        && paths::ensure_dir(parent).is_err()
    {
        return (false, path);
    }
    let raw_opt = paths::read_file_safe(&path);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&path) {
        return (false, path);
    }
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comments(&raw) {
        return (false, path);
    }
    let mut cfg = if raw.trim().is_empty() {
        OrderedMap::new()
    } else {
        let Some(c) = try_parse_jsonc(&raw) else {
            return (false, path);
        };
        c
    };
    if !cfg.contains_key("$schema") {
        cfg.insert(
            "$schema".to_string(),
            Value::String("https://opencode.ai/config.json".to_string()),
        );
    }
    if cfg.get("mcp").is_some_and(|v| !v.is_object()) {
        return (false, path);
    }
    if !cfg.contains_key("mcp") {
        cfg.insert("mcp".to_string(), Value::Object(OrderedMap::new()));
    }
    let mcp = json_map_child_mut(&mut cfg, "mcp").unwrap();
    let parent = if v2 {
        if mcp.get("servers").is_some_and(|v| !v.is_object()) {
            return (false, path);
        }
        if !mcp.contains_key("servers") {
            mcp.insert("servers".to_string(), Value::Object(OrderedMap::new()));
        }
        json_map_child_mut(mcp, "servers").unwrap()
    } else {
        mcp
    };
    let command = open_code_mcp_command(open_code_agent_id(v2), tool_id);
    if let Some(existing) = parent.get(tool_id) {
        if let Some(em) = existing.as_object() {
            if open_code_mcp_entry_enabled(em) && open_code_mcp_entry_command_matches(em, &command) {
                return (false, path);
            }
            if !opencode_mcp_managed_for(open_code_agent_id(v2), tool_id, existing) {
                return (false, path);
            }
        } else {
            return (false, path);
        }
    }
    let mut entry = OrderedMap::new();
    entry.insert("type".to_string(), Value::String("local".to_string()));
    entry.insert("command".to_string(), Value::Array(strings_to_values(&command)));
    if v2 {
        entry.insert("disabled".to_string(), Value::Bool(false));
    } else {
        entry.insert("enabled".to_string(), Value::Bool(true));
    }
    parent.insert(tool_id.to_string(), Value::Object(entry));
    let next = stringify_json(&Value::Object(cfg));
    if next == raw {
        return (false, path);
    }
    if paths::write_file_atomic(&path, &next).is_err() {
        return (false, path);
    }
    (true, path)
}

pub fn configure_opencode_mcp_at(p: &str, tool_id: &str) -> (bool, String) {
    configure_open_code_mcp_impl(tool_id, p, false)
}

pub fn configure_opencode_mcp_v1(tool_id: &str) -> (bool, String) {
    configure_open_code_mcp_impl(tool_id, &paths::opencode_paths_resolved().config, false)
}

pub fn configure_opencode_mcp(tool_id: &str) -> (bool, String) {
    configure_opencode_mcp_v1(tool_id)
}

pub fn configure_opencode_mcp_v2(tool_id: &str) -> (bool, String) {
    configure_open_code_mcp_impl(tool_id, &paths::opencode_paths_v2().config, true)
}

fn remove_open_code_mcp_impl(tool_id: &str, path: &str, v2: bool) -> bool {
    let Some(raw) = paths::read_file_safe(path) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = json_map_child_mut(&mut cfg, "mcp") else {
        return false;
    };
    let command = open_code_mcp_command(open_code_agent_id(v2), tool_id);
    if v2 {
        let Some(servers) = json_map_child_mut(mcp, "servers") else {
            return false;
        };
        let Some(existing) = servers.get(tool_id).and_then(|v| v.as_object()) else {
            return false;
        };
        if !open_code_mcp_entry_command_matches(existing, &command) {
            return false;
        }
        servers.shift_remove(tool_id);
        if servers.is_empty() {
            mcp.shift_remove("servers");
        }
    } else {
        let Some(existing) = mcp.get(tool_id).and_then(|v| v.as_object()) else {
            return false;
        };
        if !open_code_mcp_entry_command_matches(existing, &command) {
            return false;
        }
        mcp.shift_remove(tool_id);
    }
    if mcp.is_empty() {
        cfg.shift_remove("mcp");
    }
    let next = stringify_json(&Value::Object(cfg));
    paths::write_file_atomic(path, &next).is_ok()
}

pub fn remove_opencode_mcp_at(p: &str, tool_id: &str) -> bool {
    remove_open_code_mcp_impl(tool_id, p, false)
}

pub fn remove_opencode_mcp_v1(tool_id: &str) -> bool {
    remove_open_code_mcp_impl(tool_id, &paths::opencode_paths_resolved().config, false)
}

pub fn remove_opencode_mcp_v2(tool_id: &str) -> bool {
    remove_open_code_mcp_impl(tool_id, &paths::opencode_paths_v2().config, true)
}

pub fn remove_opencode_mcp(tool_id: &str) -> bool {
    let r1 = remove_opencode_mcp_v1(tool_id);
    let r2 = remove_opencode_mcp_v2(tool_id);
    r1 || r2
}

pub fn opencode_mcp_has_at(p: &str, tool_id: &str) -> bool {
    let Some(raw) = paths::read_file_safe(p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    open_code_mcp_has(&cfg, tool_id)
}

pub fn opencode_mcp_has(tool_id: &str) -> bool {
    opencode_mcp_has_at(&paths::opencode_paths_resolved().config, tool_id)
}

pub fn opencode2_mcp_has(tool_id: &str) -> bool {
    opencode_mcp_has_at(&paths::opencode_paths_v2().config, tool_id)
}

fn opencode_mcp_managed_for(agent_id: &str, tool_id: &str, existing: &Value) -> bool {
    let Some(em) = existing.as_object() else {
        return false;
    };
    let cmd = open_code_mcp_command(agent_id, tool_id);
    em.get("type").and_then(|v| v.as_str()) == Some("local")
        && json_args_eq(em.get("command"), &cmd)
}

// --- opencode_proxy.go ---

pub fn open_code_byok_plugin_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("opencode-byok-plugin.js")
        .to_string_lossy()
        .into_owned()
}

pub fn open_code_headroom_plugin_dir() -> String {
    let p = headroom_paths_resolved();
    let mut root = Path::new(&p.tools).join("headroom-ai");
    if cfg!(windows) {
        root = root.join("Lib").join("site-packages");
    } else {
        root = root.join("lib").join("python3.14").join("site-packages");
    }
    root.join("headroom")
        .join("providers")
        .join("opencode")
        .join("_dist")
        .to_string_lossy()
        .into_owned()
}

pub fn open_code_transport_plugin_path() -> String {
    let dir = open_code_headroom_plugin_dir();
    let index = Path::new(&dir).join("index.js");
    if paths::exists(&index) {
        return index.to_string_lossy().into_owned();
    }
    Path::new(&dir)
        .join("entry.opencode.js")
        .to_string_lossy()
        .into_owned()
}

pub fn open_code_byok_plugin_source() -> String {
    let transport_path = open_code_transport_plugin_path();
    let base = Path::new(&transport_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let import_path = format!("\"./{}\"", base);
    TOKLESS_OPENCODE_BYOK_PLUGIN.replacen("\"./entry.opencode.js\"", &import_path, 1)
}

pub fn is_open_code_transport_plugin_path_entry(v: &Value) -> bool {
    let Some(s) = v.as_str() else {
        return false;
    };
    let normalized = s.replace('\\', "/");
    normalized.ends_with("/opencode-byok-plugin.js")
        || normalized.contains("tokless-headroom-transport.js")
        || s == open_code_byok_plugin_path()
}

pub fn open_code_proxy_provider_block(base_url: &str) -> OrderedMap {
    open_code_proxy_provider_block_for(base_url, &provider_spec_active())
}

pub fn open_code_proxy_provider_block_for(base_url: &str, spec: &ProviderSpec) -> OrderedMap {
    let mut block = OrderedMap::new();
    block.insert("npm".to_string(), Value::String(spec.npm.clone()));
    block.insert("name".to_string(), Value::String(spec.name.clone()));

    let mut options = OrderedMap::new();
    options.insert("baseURL".to_string(), Value::String(base_url.to_string()));
    if !spec.key_env.is_empty() {
        options.insert(
            "apiKey".to_string(),
            Value::String(format!("{{env:{}}}", spec.key_env)),
        );
    } else {
        options.insert("apiKey".to_string(), Value::String(proxy_wire_key()));
    }
    block.insert("options".to_string(), Value::Object(options));

    let mut models = OrderedMap::new();
    for m in &spec.models {
        let mut model_entry = OrderedMap::new();
        model_entry.insert("name".to_string(), Value::String(m.display.clone()));
        if m.context > 0 || m.output > 0 {
            let mut limit = OrderedMap::new();
            if m.context > 0 {
                limit.insert("context".to_string(), Value::from(m.context));
            }
            if m.output > 0 {
                limit.insert("output".to_string(), Value::from(m.output));
            }
            model_entry.insert("limit".to_string(), Value::Object(limit));
        }
        if m.reasoning {
            model_entry.insert("reasoning".to_string(), Value::Bool(true));
        }
        if m.tool_call {
            model_entry.insert("tool_call".to_string(), Value::Bool(true));
        }
        models.insert(m.id.clone(), Value::Object(model_entry));
    }
    block.insert("models".to_string(), Value::Object(models));
    block
}

pub fn open_code_config_files() -> Vec<String> {
    let p = paths::opencode_paths_resolved();
    open_code_v2_config_files_for(&p.dir, &p.config)
}

pub fn open_code_v2_config_files() -> Vec<String> {
    let p = paths::opencode_paths_v2();
    open_code_v2_config_files_for(&p.dir, &p.config)
}

pub fn open_code_v2_config_files_for(dir: &str, config: &str) -> Vec<String> {
    let mut out = vec![
        Path::new(dir).join("config.json").to_string_lossy().into_owned(),
        Path::new(dir).join("opencode.json").to_string_lossy().into_owned(),
        Path::new(dir).join("opencode.jsonc").to_string_lossy().into_owned(),
    ];
    if !out.iter().any(|p| p == config) {
        out.push(config.to_string());
    }
    out
}

fn open_code_candidate_files() -> Vec<String> {
    let p = paths::opencode_paths_resolved();
    let mut out = vec![p.config.clone()];
    let jsonc = Path::new(&p.dir)
        .join("opencode.jsonc")
        .to_string_lossy()
        .into_owned();
    if jsonc != p.config && paths::exists(&jsonc) {
        out.push(jsonc);
    }
    out
}

pub fn discover_open_code_byok() -> Vec<OpenCodeBYOK> {
    let active_key = provider_spec_active().key;
    let stash = load_proxy_route_stash("opencode");
    let mut out = Vec::new();
    for file in open_code_candidate_files() {
        let Some(raw) = paths::read_file_safe(&file) else {
            continue;
        };
        let Some(cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let Some(pm) = json_map_child(&cfg, "provider") else {
            continue;
        };
        for (id, entry_val) in pm {
            if id == &active_key || id == "tokless-headroom" {
                continue;
            }
            let Some(m) = entry_val.as_object() else {
                continue;
            };
            let npm = m
                .get("npm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let protocol = open_code_protocol(&npm);
            let (mut base, key) = provider_base_and_key(m);
            if let Some(stashed) = stash.get(id)
                && !stashed.base_url.is_empty()
            {
                base = stashed.base_url.clone();
            }
            if base.is_empty() || same_proxy_base(&base, &proxy_endpoint_for("opencode")) {
                continue;
            }
            if !is_valid_byok_upstream(&base) {
                continue;
            }
            let route_header = read_byok_route(&format!("opencode:{id}"))
                .map(|r| byok_route_header(&r))
                .unwrap_or_default();
            out.push(OpenCodeBYOK {
                id: id.clone(),
                file: file.clone(),
                base_url: base,
                api_key: key.clone(),
                raw_api_key: key,
                npm,
                protocol,
                route_header,
                native: false,
                name: String::new(),
            });
        }
    }
    out
}

pub fn discover_opencode_byok() -> Vec<OpenCodeBYOK> {
    discover_open_code_byok()
}

pub fn sync_open_code_byok_routes() -> usize {
    let entries = discover_open_code_byok();
    let mut synced = 0usize;
    for entry in entries {
        let api = if entry.protocol == "anthropic-messages" {
            "anthropic-messages"
        } else {
            "openai-completions"
        };
        let upstream = normalized_headroom_upstream(&entry.base_url, api);
        if !is_valid_byok_upstream(&upstream) {
            continue;
        }
        let (_, _, _, ok) = register_byok_route("opencode", &entry.id, api, &upstream);
        if ok {
            synced += 1;
        }
    }
    synced
}

pub fn sync_opencode_byok_routes() -> usize {
    sync_open_code_byok_routes()
}

fn ensure_open_code_byok_plugin_file() -> Result<String, String> {
    let path = open_code_byok_plugin_path();
    let src = open_code_byok_plugin_source();
    if let Some(raw) = paths::read_file_safe(&path)
        && raw == src
    {
        return Ok(path);
    }
    paths::write_file_atomic(&path, &src).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn configure_open_code_proxy() -> (bool, String) {
    let mut out = (false, paths::opencode_paths_resolved().config);
    if let Err(err) = with_proxy_route_stash_lock(|| {
        out = configure_open_code_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("opencode proxy lock failed: {err}"));
    }
    out
}

pub fn configure_opencode_proxy() -> (bool, String) {
    configure_open_code_proxy()
}

fn configure_open_code_proxy_locked() -> (bool, String) {
    let p = paths::opencode_paths_resolved().config;
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return (false, p);
    }
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comments(&raw) {
        return (false, p);
    }
    if !proxy_route_stash_valid("opencode") {
        return (false, p);
    }
    let mut stash = load_proxy_route_stash_locked("opencode");
    let spec = provider_spec_active();
    let endpoint = proxy_endpoint_for("opencode");
    let st = classify_open_code_jsonc_proxy(
        &raw,
        &spec.key,
        &endpoint,
        &stash,
        is_open_code_transport_plugin_path_entry,
    );
    if matches!(st, ProxyConfigState::Unreadable | ProxyConfigState::Conflict) {
        return (false, p);
    }

    let mut cfg = if raw.trim().is_empty() {
        OrderedMap::new()
    } else {
        let Some(c) = try_parse_jsonc(&raw) else {
            return (false, p);
        };
        c
    };

    let stash_path = proxy_route_stash_path("opencode");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut rollbacks: Vec<ByokRouteRollback> = Vec::new();
    let mut changed = false;

    if !cfg.contains_key("$schema") {
        cfg.insert(
            "$schema".to_string(),
            Value::String("https://opencode.ai/config.json".to_string()),
        );
        changed = true;
    }

    let mut need_plugin = false;
    {
        let provider_map = json_get_or_create_map(&mut cfg, "provider");
        let provider_ids: Vec<String> = provider_map.keys().cloned().collect();
        for id in provider_ids {
            if id == spec.key || id == "tokless-headroom" {
                continue;
            }
            let Some(entry_val) = provider_map.get(&id).cloned() else {
                continue;
            };
            let Some(mut entry_map) = entry_val.as_object().cloned() else {
                continue;
            };
            let (base, _) = provider_base_and_key(&entry_map);
            if base.is_empty() || same_proxy_base(&base, &endpoint) {
                if stash.contains_key(&id) {
                    need_plugin = true;
                }
                continue;
            }
            let npm = entry_map
                .get("npm")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let api = match open_code_protocol(&npm).as_str() {
                "anthropic-messages" => "anthropic-messages",
                _ => "openai-completions",
            };
            let upstream = normalized_headroom_upstream(&base, api);
            if !is_valid_byok_upstream(&upstream) {
                for rb in &rollbacks {
                    rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
                }
                return (false, p);
            }
            let (_, hdr, prev_route, ok) = register_byok_route("opencode", &id, api, &upstream);
            if !ok {
                for rb in &rollbacks {
                    rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
                }
                return (false, p);
            }
            rollbacks.push(ByokRouteRollback {
                id: id.clone(),
                previous: prev_route,
            });

            let had_top_base = entry_map.contains_key("baseURL");
            entry_map.shift_remove("baseURL");
            let opts = json_get_or_create_map(&mut entry_map, "options");
            let base_key = if opts.contains_key("baseUrl") && !opts.contains_key("baseURL") {
                "baseUrl".to_string()
            } else {
                "baseURL".to_string()
            };
            let target_ep = proxy_endpoint_for_api(api);
            opts.insert(base_key.clone(), Value::String(target_ep));
            let headers = json_get_or_create_map(opts, "headers");
            if let Some((hk, hv)) = hdr.split_once(':') {
                headers.insert(hk.trim().to_string(), Value::String(hv.trim().to_string()));
            }
            let managed_val = Value::Object(entry_map);
            stash.insert(
                id.clone(),
                ProxyRouteStashEntry {
                    file: p.clone(),
                    provider: id.clone(),
                    base_url: base,
                    upstream,
                    base_key,
                    had_base_key: true,
                    moved_base: had_top_base,
                    header: hdr,
                    original: serde_json::to_vec(&entry_val).unwrap_or_default(),
                    managed: serde_json::to_vec(&managed_val).unwrap_or_default(),
                    ..Default::default()
                },
            );
            provider_map.insert(id, managed_val);
            need_plugin = true;
            changed = true;
        }

        let desired_block = Value::Object(open_code_proxy_provider_block_for(&endpoint, &spec));
        match provider_map.get(&spec.key) {
            Some(existing) if json_equal(existing, &desired_block) => {}
            _ => {
                provider_map.insert(spec.key.clone(), desired_block);
                changed = true;
            }
        }
    }

    if need_plugin {
        let Ok(plugin_path) = ensure_open_code_byok_plugin_file() else {
            for rb in &rollbacks {
                rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
            }
            return (false, p);
        };
        let mut plugins = cfg
            .get("plugin")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if !plugins.iter().any(is_open_code_transport_plugin_path_entry) {
            plugins.push(Value::String(plugin_path));
            cfg.insert("plugin".to_string(), Value::Array(plugins));
            changed = true;
        }
    }

    if save_proxy_route_stash("opencode", &stash).is_err() {
        for rb in &rollbacks {
            rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
        }
        return (false, p);
    }

    if !changed {
        return (false, p);
    }

    let next = stringify_json(&Value::Object(cfg));
    if paths::write_file_atomic(&p, &next).is_err() {
        restore_proxy_route_stash_logged(
            "opencode",
            prev_stash_raw.as_deref().unwrap_or(""),
            prev_stash_exists,
        );
        for rb in &rollbacks {
            rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
        }
        return (false, p);
    }
    (true, p)
}

pub fn remove_open_code_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_open_code_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("opencode proxy lock failed: {err}"));
    }
    removed
}

pub fn remove_opencode_proxy() -> bool {
    remove_open_code_proxy()
}

fn remove_open_code_proxy_locked() -> bool {
    let p = paths::opencode_paths_resolved().config;
    if !proxy_route_stash_valid("opencode") {
        return false;
    }
    let mut stash = load_proxy_route_stash_locked("opencode");
    let Some(raw) = paths::read_file_safe(&p) else {
        let _ = fs::remove_file(open_code_byok_plugin_path());
        if !stash.is_empty() {
            for id in stash.keys() {
                let _ = delete_byok_route("opencode", id);
            }
            stash.clear();
            let _ = save_proxy_route_stash("opencode", &stash);
            return true;
        }
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let spec = provider_spec_active();
    let endpoint = proxy_endpoint_for("opencode");
    let st = classify_open_code_jsonc_proxy(
        &raw,
        &spec.key,
        &endpoint,
        &stash,
        is_open_code_transport_plugin_path_entry,
    );
    if st != ProxyConfigState::Managed {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };

    let stash_path = proxy_route_stash_path("opencode");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();
    let existing_routes: Vec<BYOKRoute> = read_byok_routes()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.id.starts_with("opencode:"))
        .collect();

    let mut changed = false;
    if let Some(pm) = json_map_child_mut(&mut cfg, "provider") {
        if pm.shift_remove(&spec.key).is_some() {
            changed = true;
        }
        if pm.shift_remove("tokless-headroom").is_some() {
            changed = true;
        }
        let stashed_ids: Vec<String> = stash.keys().cloned().collect();
        for id in stashed_ids {
            let Some(entry) = stash.get(&id) else {
                continue;
            };
            if let Ok(orig_val) = serde_json::from_slice::<Value>(&entry.original) {
                pm.insert(id.clone(), orig_val);
                changed = true;
            }
            let _ = delete_byok_route("opencode", &id);
            stash.remove(&id);
        }
        if pm.is_empty() {
            cfg.shift_remove("provider");
        }
    }

    if let Some(plugins) = cfg.get("plugin").and_then(|v| v.as_array()).cloned() {
        let kept: Vec<Value> = plugins
            .into_iter()
            .filter(|v| !is_open_code_transport_plugin_path_entry(v))
            .collect();
        if kept.is_empty() {
            cfg.shift_remove("plugin");
        } else {
            cfg.insert("plugin".to_string(), Value::Array(kept));
        }
        changed = true;
    }

    if cfg.len() == 1 && cfg.contains_key("$schema") {
        cfg.shift_remove("$schema");
    }

    if save_proxy_route_stash("opencode", &stash).is_err() {
        restore_deleted_byok_routes(&existing_routes);
        return false;
    }

    if changed {
        if cfg.is_empty() {
            if fs::remove_file(&p).is_err() {
                restore_proxy_route_stash_logged(
                    "opencode",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        } else {
            let next = stringify_json(&Value::Object(cfg));
            if paths::write_file_atomic(&p, &next).is_err() {
                restore_proxy_route_stash_logged(
                    "opencode",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        }
    }

    let _ = fs::remove_file(open_code_byok_plugin_path());
    changed
}

pub fn open_code_proxy_wired() -> bool {
    let p = paths::opencode_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    if !proxy_route_stash_valid("opencode") {
        return false;
    }
    let stash = load_proxy_route_stash("opencode");
    let spec = provider_spec_active();
    classify_open_code_jsonc_proxy(
        &raw,
        &spec.key,
        &proxy_endpoint_for("opencode"),
        &stash,
        is_open_code_transport_plugin_path_entry,
    ) == ProxyConfigState::Managed
}

pub fn opencode_proxy_wired() -> bool {
    open_code_proxy_wired()
}

pub fn open_code_proxy_satisfied() -> bool {
    open_code_proxy_wired()
}

pub fn opencode_proxy_satisfied() -> bool {
    open_code_proxy_wired()
}

// --- OpenCode v2 proxy & BYOK mirror ---

pub const OPEN_CODE_V2_ORIGINAL_PATH_HEADER: &str = "x-headroom-original-path";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct OpenCodeV2ByokStashEntry {
    pub file: String,
    pub base_url: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub native: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct OpenCodeLaneStash {
    pub dir: String,
    pub config: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct OpenCodeV2ByokStashFile {
    pub providers: std::collections::HashMap<String, OpenCodeV2ByokStashEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane: Option<OpenCodeLaneStash>,
}

fn byok_stash_path_for(agent: &str) -> String {
    Path::new(&headroom_paths_resolved().root)
        .join(format!("{agent}.byok.stash.json"))
        .to_string_lossy()
        .into_owned()
}

fn load_v2_byok_stash_file_at(path: &str) -> OpenCodeV2ByokStashFile {
    let Some(raw) = paths::read_file_safe(path) else {
        return OpenCodeV2ByokStashFile::default();
    };
    serde_json::from_str::<OpenCodeV2ByokStashFile>(&raw).unwrap_or_default()
}

fn save_v2_byok_stash_file_at(path: &str, f: &OpenCodeV2ByokStashFile) -> Result<(), String> {
    let b = serde_json::to_string(f).map_err(|e| e.to_string())?;
    paths::write_file_mode(path, &b, 0o600).map_err(|e| e.to_string())
}

fn v2_byok_stash_valid_at(path: &str) -> bool {
    let Some(raw) = paths::read_file_safe(path) else {
        return true;
    };
    let Ok(f) = serde_json::from_str::<OpenCodeV2ByokStashFile>(&raw) else {
        return false;
    };
    f.providers
        .iter()
        .all(|(id, e)| !id.is_empty() && !e.file.is_empty() && !e.base_url.is_empty())
}

fn clear_v2_byok_stash_at(path: &str) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn open_code_retrieve_state_path_for(agent: &str) -> String {
    Path::new(&headroom_paths_resolved().root)
        .join(format!("{agent}.retrieve.stash.json"))
        .to_string_lossy()
        .into_owned()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RetrieveStateFile {
    present: bool,
    value: Value,
}

fn save_open_code_retrieve_state_at(path: &str, value: Option<Value>) -> Result<(), String> {
    let f = RetrieveStateFile {
        present: value.is_some(),
        value: value.unwrap_or(Value::Null),
    };
    let b = serde_json::to_string(&f).map_err(|e| e.to_string())?;
    paths::write_file_mode(path, &b, 0o600).map_err(|e| e.to_string())
}

fn load_open_code_retrieve_state_at(path: &str) -> Option<Option<Value>> {
    let raw = paths::read_file_safe(path)?;
    let f: RetrieveStateFile = serde_json::from_str(&raw).ok()?;
    if !f.present {
        Some(None)
    } else {
        Some(Some(f.value))
    }
}

fn clear_open_code_retrieve_state_at(path: &str) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn restore_file_at(path: &str, raw: Option<&str>) -> Result<(), String> {
    if let Some(content) = raw {
        paths::write_file_atomic(path, content).map_err(|e| e.to_string())
    } else {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn open_code_proxy_endpoint_for(prefix: &str) -> String {
    let agent = prefix.trim_end_matches(':');
    let ep = proxy_endpoint_for(agent);
    let trimmed = ep.trim();
    if trimmed.is_empty() {
        byok_gateway_endpoint()
    } else {
        trimmed.to_string()
    }
}

pub fn open_code_v2_provider_base_and_key(m: &OrderedMap) -> (String, String) {
    let mut base = String::new();
    let mut key = String::new();
    if let Some(settings) = json_map_child(m, "settings") {
        if let Some(s) = settings.get("baseURL").and_then(|v| v.as_str()) {
            base = s.to_string();
        }
        if base.is_empty()
            && let Some(s) = settings.get("baseUrl").and_then(|v| v.as_str())
        {
            base = s.to_string();
        }
        if let Some(s) = settings.get("apiKey").and_then(|v| v.as_str()) {
            key = s.to_string();
        }
    }
    (base.trim().to_string(), key.trim().to_string())
}

pub fn open_code_v2_proxy_endpoint(raw: &str, proxy_base: &str) -> bool {
    let raw = raw.trim();
    if raw.is_empty() {
        return false;
    }
    if same_proxy_base(raw, proxy_base) {
        return true;
    }
    let rest = raw
        .strip_prefix("http://")
        .or_else(|| raw.strip_prefix("https://"))
        .unwrap_or("");
    let host_port = rest.split('/').next().unwrap_or("");
    if host_port.is_empty() {
        return false;
    }
    let (host, port_str) = if let Some(rest_v6) = host_port.strip_prefix('[') {
        if let Some((h, after)) = rest_v6.split_once(']') {
            (h, after.strip_prefix(':').unwrap_or(""))
        } else {
            return false;
        }
    } else if let Some((h, p)) = host_port.rsplit_once(':') {
        (h, p)
    } else {
        return false;
    };
    if !is_loopback_host(host) || port_str.is_empty() {
        return false;
    }
    let Ok(port) = port_str.parse::<u16>() else {
        return false;
    };
    port == headroom_proxy_port()
        || port == byok_gateway_port()
        || port == grok_oauth_proxy_port()
}

fn open_code_v2_endpoint_slot(m: &OrderedMap, base: &str) -> bool {
    let Some(headers) = json_map_child(m, "headers") else {
        return false;
    };
    if !headers.contains_key("X-Tokless-Route") {
        return false;
    }
    let rest = base
        .trim()
        .strip_prefix("http://")
        .or_else(|| base.trim().strip_prefix("https://"))
        .unwrap_or("");
    let host_port = rest.split('/').next().unwrap_or("");
    let host = if let Some(rest_v6) = host_port.strip_prefix('[') {
        rest_v6.split(']').next().unwrap_or("")
    } else {
        host_port.split(':').next().unwrap_or("")
    };
    is_loopback_host(host)
}

fn open_code_v2_upstream_with_path(origin: &str, headers: &OrderedMap) -> String {
    let origin = origin.trim().trim_end_matches('/');
    let mut path = String::new();
    if let Some(raw) = headers
        .get(OPEN_CODE_V2_ORIGINAL_PATH_HEADER)
        .and_then(|v| v.as_str())
    {
        let trimmed = raw.trim();
        let stripped = trimmed.strip_suffix("/chat/completions").unwrap_or(trimmed);
        path = stripped.trim_end_matches('/').to_string();
    }
    if path.is_empty() {
        origin.to_string()
    } else {
        format!("{origin}{path}")
    }
}

fn open_code_v2_provider_upstream(
    m: &OrderedMap,
    base: &str,
    proxy_base: &str,
    stashed: &str,
) -> String {
    if !base.is_empty()
        && is_absolute_http(base)
        && !open_code_v2_proxy_endpoint(base, proxy_base)
        && !open_code_v2_endpoint_slot(m, base)
    {
        return base.to_string();
    }
    if let Some(headers) = json_map_child(m, "headers")
        && let Some(origin) = headers
            .get(HEADROOM_BASE_URL_HEADER)
            .and_then(|v| v.as_str())
    {
        let trimmed = origin.trim();
        if !trimmed.is_empty() {
            return open_code_v2_upstream_with_path(trimmed, headers);
        }
    }
    stashed.trim().to_string()
}

fn open_code_v2_provider_npm(m: &OrderedMap) -> String {
    let pkg = m
        .get("package")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    match pkg {
        "@opencode/ai/providers/openai-compatible" => "@ai-sdk/openai-compatible".to_string(),
        "@opencode/ai/providers/openai" => "@ai-sdk/openai".to_string(),
        "@opencode/ai/providers/anthropic" => "@ai-sdk/anthropic".to_string(),
        _ => String::new(),
    }
}

fn open_code_v2_package(npm: &str) -> &'static str {
    match npm.trim() {
        "@ai-sdk/openai-compatible" => "@opencode/ai/providers/openai-compatible",
        "@ai-sdk/openai" => "@opencode/ai/providers/openai",
        "@ai-sdk/anthropic" => "@opencode/ai/providers/anthropic",
        _ => "@opencode/ai/providers/openai-compatible",
    }
}

fn provider_base_and_raw_key(m: &OrderedMap) -> (String, String) {
    let mut base = String::new();
    let mut key = String::new();
    if let Some(om) = json_map_child(m, "options") {
        if let Some(s) = om.get("baseURL").and_then(|v| v.as_str()) {
            base = s.to_string();
        }
        if base.is_empty()
            && let Some(s) = om.get("baseUrl").and_then(|v| v.as_str())
        {
            base = s.to_string();
        }
        if let Some(s) = om.get("apiKey").and_then(|v| v.as_str()) {
            key = s.to_string();
        }
    }
    if base.is_empty()
        && let Some(s) = m.get("baseURL").and_then(|v| v.as_str())
    {
        base = s.to_string();
    }
    if key.is_empty()
        && let Some(s) = m.get("apiKey").and_then(|v| v.as_str())
    {
        key = s.to_string();
    }
    (base.trim().to_string(), key.trim().to_string())
}

fn discover_open_code_byok_in(
    files: &[String],
    prefix: &str,
    stash_path: &str,
) -> Vec<OpenCodeBYOK> {
    let proxy_base = proxy_endpoint_for(prefix.trim_end_matches(':'))
        .trim_end_matches('/')
        .to_string();
    #[derive(Default)]
    struct Acc {
        base: String,
        key: String,
        raw: String,
        file: String,
        npm: String,
    }
    let mut got: std::collections::HashMap<String, Acc> = std::collections::HashMap::new();
    let mut duplicates: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut order: Vec<String> = Vec::new();

    for path in files {
        let Some(raw) = paths::read_file_safe(path) else {
            continue;
        };
        if has_jsonc_comments(&raw) {
            continue;
        }
        let Some(cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let Some(providers) = json_map_child(&cfg, "provider") else {
            continue;
        };
        for (id, block) in providers {
            let Some(m) = block.as_object() else {
                continue;
            };
            let (base, raw_key) = provider_base_and_raw_key(m);
            let key = resolve_api_key(&raw_key);
            if base.is_empty() && key.is_empty() {
                continue;
            }
            if let Some(a) = got.get_mut(id) {
                if (!base.is_empty() && !a.base.is_empty() && base != a.base)
                    || (!key.is_empty() && !a.key.is_empty() && key != a.key)
                {
                    duplicates.insert(id.clone());
                }
            } else {
                got.insert(id.clone(), Acc::default());
                order.push(id.clone());
            }
            let a = got.get_mut(id).unwrap();
            if !key.is_empty() {
                a.key = key;
                a.raw = raw_key;
            }
            if let Some(npm) = m.get("npm").and_then(|v| v.as_str())
                && !npm.is_empty()
            {
                a.npm = npm.to_string();
            }
            if !base.is_empty()
                && is_valid_byok_upstream(&base)
                && !same_proxy_base(&base, &proxy_base)
            {
                a.base = base;
                a.file = path.clone();
            }
        }
    }

    let stashed = load_v2_byok_stash_file_at(stash_path).providers;
    let mut out = Vec::new();
    for id in order {
        if duplicates.contains(&id) {
            continue;
        }
        let Some(a) = got.get_mut(&id) else {
            continue;
        };
        if a.base.is_empty()
            && let Some(s) = stashed.get(&id)
            && !s.base_url.is_empty()
        {
            a.base = s.base_url.clone();
            if a.file.is_empty() {
                a.file = s.file.clone();
            }
        }
        if a.base.is_empty() || a.key.is_empty() || a.file.is_empty() {
            continue;
        }
        let route_header = read_byok_route(&format!("{prefix}{id}"))
            .map(|r| byok_route_header(&r))
            .unwrap_or_default();
        out.push(OpenCodeBYOK {
            id,
            file: a.file.clone(),
            base_url: a.base.clone(),
            api_key: a.key.clone(),
            raw_api_key: a.raw.clone(),
            npm: a.npm.clone(),
            protocol: open_code_protocol(&a.npm),
            route_header,
            native: false,
            name: String::new(),
        });
    }
    out
}

fn discover_open_code_v2_byok_in(
    files: &[String],
    prefix: &str,
    stash_path: &str,
) -> (Vec<OpenCodeBYOK>, Vec<OpenCodeSkip>) {
    let proxy_base = open_code_proxy_endpoint_for(prefix);
    let stashed = load_v2_byok_stash_file_at(stash_path).providers;
    let mut out = Vec::new();
    let mut skipped = Vec::new();

    for path in files {
        let Some(raw) = paths::read_file_safe(path) else {
            continue;
        };
        if has_jsonc_comments(&raw) {
            skipped.push(OpenCodeSkip {
                id: String::new(),
                reason: format!("comment-bearing config {path}"),
            });
            continue;
        }
        let Some(cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let Some(providers) = json_map_child(&cfg, "providers") else {
            continue;
        };
        for (id, block) in providers {
            let Some(m) = block.as_object() else {
                continue;
            };
            let npm = open_code_v2_provider_npm(m);
            if npm.is_empty() {
                skipped.push(OpenCodeSkip {
                    id: id.clone(),
                    reason: "unknown package".to_string(),
                });
                continue;
            }
            let (base, key) = open_code_v2_provider_base_and_key(m);
            let stashed_base = stashed
                .get(id)
                .map(|s| s.base_url.as_str())
                .unwrap_or("");
            let mut upstream = open_code_v2_provider_upstream(m, &base, &proxy_base, stashed_base);
            if !is_valid_byok_upstream(&upstream) {
                upstream.clear();
            }
            if upstream.is_empty() {
                skipped.push(OpenCodeSkip {
                    id: id.clone(),
                    reason: "no upstream".to_string(),
                });
                continue;
            }
            if key.is_empty() {
                skipped.push(OpenCodeSkip {
                    id: id.clone(),
                    reason: "no api key".to_string(),
                });
                continue;
            }
            let route_header = read_byok_route(&format!("{prefix}{id}"))
                .map(|r| byok_route_header(&r))
                .unwrap_or_default();
            let name = m
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            out.push(OpenCodeBYOK {
                id: id.clone(),
                file: path.clone(),
                base_url: upstream,
                api_key: key.clone(),
                raw_api_key: key,
                npm: npm.clone(),
                protocol: open_code_protocol(&npm),
                route_header,
                native: true,
                name,
            });
        }
    }
    (out, skipped)
}

fn discover_open_code_v2_lane_byok() -> (Vec<OpenCodeBYOK>, Vec<OpenCodeSkip>) {
    let files = open_code_v2_config_files();
    let stash_path = byok_stash_path_for("opencode2");
    let mut out = discover_open_code_byok_in(&files, "opencode2:", &stash_path);
    let mut seen: std::collections::HashSet<String> =
        out.iter().map(|b| b.id.clone()).collect();
    let (native, skipped) = discover_open_code_v2_byok_in(&files, "opencode2:", &stash_path);
    for b in native {
        if seen.insert(b.id.clone()) {
            out.push(b);
        }
    }
    (out, skipped)
}

pub fn open_code_v2_skipped_providers() -> Vec<String> {
    let (_, skipped) = discover_open_code_v2_lane_byok();
    skipped
        .into_iter()
        .map(|s| {
            if s.id.is_empty() {
                s.reason
            } else {
                format!("{}: {}", s.id, s.reason)
            }
        })
        .collect()
}

fn open_code_v2_shares_v1_config() -> bool {
    let v2 = paths::opencode_paths_v2().config;
    let v1 = paths::opencode_paths_resolved().config;
    Path::new(&v2) == Path::new(&v1)
}

fn open_code_retrieve_tool_disabled_at(file: &str) -> bool {
    let Some(raw) = paths::read_file_safe(file) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    json_map_child(&cfg, "tools")
        .and_then(|t| t.get("headroom_retrieve"))
        .and_then(|v| v.as_bool())
        == Some(false)
}

fn open_code_v2_providers_of(file: &str) -> Option<OrderedMap> {
    let raw = paths::read_file_safe(file)?;
    if has_jsonc_comments(&raw) {
        return None;
    }
    let cfg = try_parse_jsonc(&raw)?;
    json_map_child(&cfg, "providers").cloned()
}

fn open_code_v2_mirror_wired() -> bool {
    let Some(providers) = open_code_v2_providers_of(&paths::opencode_paths_v2().config) else {
        return false;
    };
    let endpoint = open_code_proxy_endpoint_for("opencode2:");
    for (_, val) in &providers {
        if let Some(entry) = val.as_object() {
            let (base, _) = open_code_v2_provider_base_and_key(entry);
            if base == endpoint {
                return true;
            }
        }
    }
    false
}

pub fn open_code_proxy_v2_wired() -> bool {
    if open_code_v2_shares_v1_config() {
        return open_code_proxy_wired();
    }
    let v2_cfg = paths::opencode_paths_v2().config;
    open_code_v2_mirror_wired() && open_code_retrieve_tool_disabled_at(&v2_cfg)
}

pub fn opencode_proxy_v2_wired() -> bool {
    open_code_proxy_v2_wired()
}

fn open_code_v2_upstream_origin_and_path(upstream: &str) -> (String, String) {
    let s = upstream.trim();
    let (scheme, rest) = if let Some(r) = s.strip_prefix("http://") {
        ("http://", r)
    } else if let Some(r) = s.strip_prefix("https://") {
        ("https://", r)
    } else {
        return (s.to_string(), String::new());
    };
    let (host, path_raw) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, ""),
    };
    let origin = format!("{scheme}{host}");
    let p = path_raw.trim_end_matches('/');
    if p.is_empty() {
        (origin, String::new())
    } else if let Some(before_v1) = p.strip_suffix("/v1") {
        let full_origin = format!("{origin}{before_v1}")
            .trim_end_matches('/')
            .to_string();
        (full_origin, "/v1/chat/completions".to_string())
    } else {
        (origin, format!("{p}/chat/completions"))
    }
}

fn open_code_v2_route_prefix_owned(route: &str, prefix: &str) -> bool {
    route.starts_with(prefix) || route.starts_with("opencode:")
}

fn open_code_v2_owned_route(providers: &OrderedMap, id: &str, prefix: &str) -> bool {
    let Some(entry) = providers.get(id).and_then(|v| v.as_object()) else {
        return false;
    };
    let Some(headers) = json_map_child(entry, "headers") else {
        return false;
    };
    let Some(route) = headers.get("X-Tokless-Route").and_then(|v| v.as_str()) else {
        return false;
    };
    open_code_v2_route_prefix_owned(route, prefix)
}

fn open_code_v2_models_mirrored(provider: &OrderedMap, endpoint: &str, upstream: &str) -> bool {
    let Some(models) = json_map_child(provider, "models") else {
        return true;
    };
    let (origin, path) = open_code_v2_upstream_origin_and_path(upstream);
    for (_, val) in models {
        let Some(model) = val.as_object() else {
            continue;
        };
        let Some(settings) = json_map_child(model, "settings") else {
            return false;
        };
        if settings.get("baseURL").and_then(|v| v.as_str()) != Some(endpoint) {
            return false;
        }
        let Some(headers) = json_map_child(model, "headers") else {
            return false;
        };
        if headers.get(HEADROOM_BASE_URL_HEADER).and_then(|v| v.as_str()) != Some(origin.as_str()) {
            return false;
        }
        let current = headers
            .get(OPEN_CODE_V2_ORIGINAL_PATH_HEADER)
            .and_then(|v| v.as_str());
        if path.is_empty() {
            if headers.contains_key(OPEN_CODE_V2_ORIGINAL_PATH_HEADER) {
                return false;
            }
        } else if current != Some(path.as_str()) {
            return false;
        }
    }
    true
}

fn open_code_v2_provider_entry_matches(
    providers: &OrderedMap,
    id: &str,
    endpoint: &str,
    route_header: &str,
    upstream: &str,
) -> bool {
    let Some(p) = providers.get(id).and_then(|v| v.as_object()) else {
        return false;
    };
    let Some(settings) = json_map_child(p, "settings") else {
        return false;
    };
    if settings.get("baseURL").and_then(|v| v.as_str()) != Some(endpoint) {
        return false;
    }
    let Some(headers) = json_map_child(p, "headers") else {
        return false;
    };
    let (origin, path) = open_code_v2_upstream_origin_and_path(upstream);
    if headers.get("X-Tokless-Route").and_then(|v| v.as_str()) != Some(route_header)
        || headers.get(HEADROOM_BASE_URL_HEADER).and_then(|v| v.as_str()) != Some(origin.as_str())
    {
        return false;
    }
    let current = headers
        .get(OPEN_CODE_V2_ORIGINAL_PATH_HEADER)
        .and_then(|v| v.as_str());
    if path.is_empty() {
        if headers.contains_key(OPEN_CODE_V2_ORIGINAL_PATH_HEADER) {
            return false;
        }
    } else if current != Some(path.as_str()) {
        return false;
    }
    open_code_v2_models_mirrored(p, endpoint, upstream)
}

fn open_code_v2_mirror_models(models: &mut OrderedMap, endpoint: &str, upstream: &str) {
    let (origin, path) = open_code_v2_upstream_origin_and_path(upstream);
    let keys: Vec<String> = models.keys().cloned().collect();
    for key in keys {
        let Some(model) = models.get_mut(&key).and_then(|v| v.as_object_mut()) else {
            continue;
        };
        let settings = json_get_or_create_map(model, "settings");
        settings.insert("baseURL".to_string(), Value::String(endpoint.to_string()));
        let headers = json_get_or_create_map(model, "headers");
        headers.insert(
            HEADROOM_BASE_URL_HEADER.to_string(),
            Value::String(origin.clone()),
        );
        if path.is_empty() {
            headers.shift_remove(OPEN_CODE_V2_ORIGINAL_PATH_HEADER);
        } else {
            headers.insert(
                OPEN_CODE_V2_ORIGINAL_PATH_HEADER.to_string(),
                Value::String(path.clone()),
            );
        }
    }
}

fn open_code_v2_models(file: &str, id: &str) -> OrderedMap {
    let mut out = OrderedMap::new();
    let Some(raw) = paths::read_file_safe(file) else {
        return out;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return out;
    };
    let mut p_opt = None;
    for key in ["provider", "providers"] {
        if let Some(ps) = json_map_child(&cfg, key)
            && let Some(p) = ps.get(id).and_then(|v| v.as_object())
        {
            p_opt = Some(p);
            break;
        }
    }
    let Some(p) = p_opt else {
        return out;
    };
    let Some(models) = json_map_child(p, "models") else {
        return out;
    };
    for (key, val) in models {
        let Some(m) = val.as_object() else {
            continue;
        };
        let mut entry = OrderedMap::new();
        let model_id = m
            .get("modelID")
            .cloned()
            .unwrap_or_else(|| Value::String(key.clone()));
        entry.insert("modelID".to_string(), model_id);
        for field in ["name", "limit"] {
            if let Some(v) = m.get(field) {
                entry.insert(field.to_string(), v.clone());
            }
        }
        out.insert(key.clone(), Value::Object(entry));
    }
    out
}

fn open_code_byok_raw_key(b: &OpenCodeBYOK) -> &str {
    if !b.raw_api_key.is_empty() {
        &b.raw_api_key
    } else {
        &b.api_key
    }
}

fn open_code_v2_provider_entry(b: &OpenCodeBYOK, endpoint: &str) -> OrderedMap {
    let mut entry = OrderedMap::new();
    let name = if b.name.is_empty() { &b.id } else { &b.name };
    entry.insert("name".to_string(), Value::String(name.clone()));
    entry.insert(
        "package".to_string(),
        Value::String(open_code_v2_package(&b.npm).to_string()),
    );
    let mut settings = OrderedMap::new();
    settings.insert("baseURL".to_string(), Value::String(endpoint.to_string()));
    let key = open_code_byok_raw_key(b);
    if !key.is_empty() {
        settings.insert("apiKey".to_string(), Value::String(key.to_string()));
    }
    entry.insert("settings".to_string(), Value::Object(settings));
    let (origin, path) = open_code_v2_upstream_origin_and_path(&b.base_url);
    let mut headers = OrderedMap::new();
    headers.insert(
        "X-Tokless-Route".to_string(),
        Value::String(b.route_header.clone()),
    );
    headers.insert(HEADROOM_BASE_URL_HEADER.to_string(), Value::String(origin));
    if !path.is_empty() {
        headers.insert(
            OPEN_CODE_V2_ORIGINAL_PATH_HEADER.to_string(),
            Value::String(path),
        );
    }
    entry.insert("headers".to_string(), Value::Object(headers));
    let mut models = open_code_v2_models(&b.file, &b.id);
    open_code_v2_mirror_models(&mut models, endpoint, &b.base_url);
    entry.insert("models".to_string(), Value::Object(models));
    entry
}

fn open_code_v2_rewrite_provider_entry(
    entry: &mut OrderedMap,
    b: &OpenCodeBYOK,
    endpoint: &str,
) {
    let key_to_set = {
        let settings = json_get_or_create_map(entry, "settings");
        settings.insert("baseURL".to_string(), Value::String(endpoint.to_string()));
        settings.shift_remove("baseUrl");
        if b.native
            && let Some(existing_key) = settings.get("apiKey").and_then(|v| v.as_str())
            && !existing_key.trim().is_empty()
        {
            existing_key.to_string()
        } else {
            open_code_byok_raw_key(b).to_string()
        }
    };
    if !key_to_set.is_empty() {
        let settings = json_get_or_create_map(entry, "settings");
        settings.insert("apiKey".to_string(), Value::String(key_to_set));
    }
    if !b.native {
        entry.insert(
            "package".to_string(),
            Value::String(open_code_v2_package(&b.npm).to_string()),
        );
    }
    let (origin, path) = open_code_v2_upstream_origin_and_path(&b.base_url);
    let headers = json_get_or_create_map(entry, "headers");
    headers.insert(
        "X-Tokless-Route".to_string(),
        Value::String(b.route_header.clone()),
    );
    headers.insert(HEADROOM_BASE_URL_HEADER.to_string(), Value::String(origin));
    if !path.is_empty() {
        headers.insert(
            OPEN_CODE_V2_ORIGINAL_PATH_HEADER.to_string(),
            Value::String(path),
        );
    } else {
        headers.shift_remove(OPEN_CODE_V2_ORIGINAL_PATH_HEADER);
    }
    if let Some(models) = json_map_child_mut(entry, "models") {
        open_code_v2_mirror_models(models, endpoint, &b.base_url);
    }
}

fn restore_open_code_v2_provider_entry(
    entry: &mut OrderedMap,
    base_url: &str,
    api_key: &str,
    endpoint: &str,
) -> bool {
    let settings = json_get_or_create_map(entry, "settings");
    if !base_url.is_empty() {
        settings.insert("baseURL".to_string(), Value::String(base_url.to_string()));
        settings.shift_remove("baseUrl");
    }
    if !api_key.is_empty() {
        settings.insert("apiKey".to_string(), Value::String(api_key.to_string()));
    }
    if let Some(models) = json_map_child_mut(entry, "models") {
        let keys: Vec<String> = models.keys().cloned().collect();
        for k in keys {
            let Some(model) = models.get_mut(&k).and_then(|v| v.as_object_mut()) else {
                continue;
            };
            if let Some(ms) = json_map_child_mut(model, "settings") {
                if ms.get("baseURL").and_then(|v| v.as_str()) == Some(endpoint) {
                    ms.shift_remove("baseURL");
                }
                if ms.is_empty() {
                    model.shift_remove("settings");
                }
            }
            if let Some(mh) = json_map_child_mut(model, "headers") {
                mh.shift_remove(HEADROOM_BASE_URL_HEADER);
                mh.shift_remove(OPEN_CODE_V2_ORIGINAL_PATH_HEADER);
                if mh.is_empty() {
                    model.shift_remove("headers");
                }
            }
        }
    }
    if let Some(headers) = json_map_child_mut(entry, "headers") {
        headers.shift_remove("X-Tokless-Route");
        headers.shift_remove(HEADROOM_BASE_URL_HEADER);
        headers.shift_remove(OPEN_CODE_V2_ORIGINAL_PATH_HEADER);
        if headers.is_empty() {
            entry.shift_remove("headers");
        }
    }
    true
}

fn open_code_v2_stash_for(
    byoks: &[OpenCodeBYOK],
    mirror_file: &str,
    prefix: &str,
) -> std::collections::HashMap<String, OpenCodeV2ByokStashEntry> {
    let mut out = std::collections::HashMap::new();
    let providers = open_code_v2_providers_of(mirror_file);
    let proxy_base = open_code_proxy_endpoint_for(prefix);
    let previous =
        load_v2_byok_stash_file_at(&byok_stash_path_for(prefix.trim_end_matches(':'))).providers;

    for b in byoks {
        if b.route_header.is_empty() {
            continue;
        }
        let m_opt = providers
            .as_ref()
            .and_then(|p| p.get(&b.id))
            .and_then(|v| v.as_object());
        let native = m_opt.is_some()
            && !providers
                .as_ref()
                .is_some_and(|p| open_code_v2_owned_route(p, &b.id, prefix));
        let mut base = b.base_url.clone();
        let mut key = String::new();
        if native && let Some(m) = m_opt {
            let (entry_base, entry_key) = open_code_v2_provider_base_and_key(m);
            let prev_base = previous
                .get(&b.id)
                .map(|p| p.base_url.as_str())
                .unwrap_or("");
            let up = open_code_v2_provider_upstream(m, &entry_base, &proxy_base, prev_base);
            if !up.is_empty() {
                base = up;
            }
            key = entry_key;
        }
        out.insert(
            b.id.clone(),
            OpenCodeV2ByokStashEntry {
                file: mirror_file.to_string(),
                base_url: base,
                native,
                api_key: key,
            },
        );
    }
    for (id, prev) in previous {
        let Some(cur) = out.get_mut(&id) else {
            out.insert(id, prev);
            continue;
        };
        if !prev.native {
            continue;
        }
        cur.native = true;
        if !prev.file.is_empty() {
            cur.file = prev.file;
        }
        if !prev.base_url.is_empty() && !open_code_v2_proxy_endpoint(&prev.base_url, &proxy_base) {
            cur.base_url = prev.base_url;
        }
        if !prev.api_key.is_empty() {
            cur.api_key = prev.api_key;
        }
    }
    out
}

fn sync_open_code_v2_byok_for(
    byoks: &[OpenCodeBYOK],
    file: &str,
    prefix: &str,
    prune: bool,
) -> (bool, bool) {
    let raw_opt = paths::read_file_safe(file);
    let exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_else(|| "{}".to_string());
    if has_jsonc_comments(&raw) {
        return (false, false);
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return (false, false);
    };
    let has = json_map_child(&cfg, "providers").is_some();
    if !has && byoks.is_empty() {
        return (false, true);
    }
    let endpoint = open_code_proxy_endpoint_for(prefix);
    if !prune {
        let has_wanted = byoks.iter().any(|b| !b.route_header.is_empty());
        if !has_wanted {
            return (false, true);
        }
    }
    if !has {
        if prune {
            return (false, true);
        }
        cfg.insert("providers".to_string(), Value::Object(OrderedMap::new()));
    }
    let stash_path = byok_stash_path_for(prefix.trim_end_matches(':'));
    let (native_vec, _) = discover_open_code_v2_byok_in(&[file.to_string()], prefix, &stash_path);
    let native_ids: std::collections::HashSet<String> =
        native_vec.into_iter().map(|b| b.id).collect();

    let mut changed = false;
    {
        let Some(providers) = json_map_child_mut(&mut cfg, "providers") else {
            return (false, false);
        };
        if !prune {
            for b in byoks {
                if b.route_header.is_empty() {
                    continue;
                }
                let is_native = native_ids.contains(&b.id);
                let present = providers.contains_key(&b.id);
                let is_entry = providers.get(&b.id).is_some_and(|v| v.is_object());
                if present && is_entry {
                    if open_code_v2_provider_entry_matches(
                        providers,
                        &b.id,
                        &endpoint,
                        &b.route_header,
                        &b.base_url,
                    ) {
                        continue;
                    }
                    if !open_code_v2_owned_route(providers, &b.id, prefix) && !is_native {
                        continue;
                    }
                    if let Some(entry) = providers.get_mut(&b.id).and_then(|v| v.as_object_mut()) {
                        open_code_v2_rewrite_provider_entry(entry, b, &endpoint);
                        changed = true;
                    }
                    continue;
                }
                if present {
                    continue;
                }
                providers.insert(
                    b.id.clone(),
                    Value::Object(open_code_v2_provider_entry(b, &endpoint)),
                );
                changed = true;
            }
        } else {
            let stashed = load_v2_byok_stash_file_at(&stash_path).providers;
            let ids: Vec<String> = providers.keys().cloned().collect();
            for id in ids {
                let owned = providers
                    .get(&id)
                    .and_then(|v| v.as_object())
                    .and_then(|m| json_map_child(m, "headers"))
                    .and_then(|h| h.get("X-Tokless-Route"))
                    .and_then(|r| r.as_str())
                    .is_some_and(|s| open_code_v2_route_prefix_owned(s, prefix));
                if !owned {
                    continue;
                }
                if let Some(st) = stashed.get(&id)
                    && st.native
                    && st.file == file
                {
                    if let Some(m) = providers.get_mut(&id).and_then(|v| v.as_object_mut())
                        && restore_open_code_v2_provider_entry(
                            m,
                            &st.base_url,
                            &st.api_key,
                            &endpoint,
                        )
                    {
                        changed = true;
                    }
                    continue;
                }
                providers.shift_remove(&id);
                changed = true;
            }
            if !changed {
                return (false, true);
            }
        }
        if providers.is_empty() && changed {
            cfg.shift_remove("providers");
        }
    }

    let new_raw = stringify_json(&Value::Object(cfg));
    if exists && new_raw == raw {
        return (changed, true);
    }
    if paths::write_file_atomic(file, &new_raw).is_err() {
        return (false, false);
    }
    (true, true)
}

fn configure_open_code_v2_retrieve_and_prune_plugin(
    file: &str,
    retrieve_stash: &str,
) -> (bool, bool) {
    let raw_opt = paths::read_file_safe(file);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(file) {
        return (false, false);
    }
    let retrieve_raw = paths::read_file_safe(retrieve_stash);
    let raw = raw_opt.unwrap_or_default();
    if has_jsonc_comments(&raw) {
        return (false, false);
    }
    let mut cfg = if raw.trim().is_empty() {
        OrderedMap::new()
    } else {
        let Some(c) = try_parse_jsonc(&raw) else {
            return (false, false);
        };
        c
    };
    if cfg.get("tools").is_some_and(|v| !v.is_object()) {
        let _ = restore_file_at(retrieve_stash, retrieve_raw.as_deref());
        return (false, false);
    }
    let mut changed = false;
    {
        let tools = json_get_or_create_map(&mut cfg, "tools");
        let cur_val = tools.get("headroom_retrieve").cloned();
        if cur_val.as_ref().and_then(|v| v.as_bool()) != Some(false) {
            if save_open_code_retrieve_state_at(retrieve_stash, cur_val).is_err() {
                return (false, false);
            }
            tools.insert("headroom_retrieve".to_string(), Value::Bool(false));
            changed = true;
        }
    }
    if let Some(plugin_val) = cfg.get("plugin").cloned() {
        let Some(entries) = plugin_val.as_array() else {
            let _ = restore_file_at(retrieve_stash, retrieve_raw.as_deref());
            return (false, false);
        };
        let mut next = Vec::with_capacity(entries.len());
        let mut removed = false;
        for entry in entries {
            let is_managed_tuple = entry
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.ends_with("/tokless-byok.js"))
                || is_open_code_transport_plugin_path_entry(entry);
            if is_managed_tuple {
                removed = true;
            } else {
                next.push(entry.clone());
            }
        }
        if removed {
            if next.is_empty() {
                cfg.shift_remove("plugin");
            } else {
                cfg.insert("plugin".to_string(), Value::Array(next));
            }
            changed = true;
        }
    }
    if !cfg.contains_key("$schema") {
        cfg.insert(
            "$schema".to_string(),
            Value::String("https://opencode.ai/config.json".to_string()),
        );
    }
    let next_raw = stringify_json(&Value::Object(cfg));
    if paths::write_file_atomic(file, &next_raw).is_err() {
        let _ = restore_file_at(retrieve_stash, retrieve_raw.as_deref());
        return (false, false);
    }
    (changed, true)
}

fn remove_open_code_v2_retrieve_and_plugin(file: &str, retrieve_stash: &str) -> bool {
    let Some(raw) = paths::read_file_safe(file) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let mut removed = false;
    if let Some(plugin_val) = cfg.get("plugin").cloned() {
        let Some(entries) = plugin_val.as_array() else {
            return false;
        };
        let mut next = Vec::with_capacity(entries.len());
        for entry in entries {
            let is_managed = entry
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.ends_with("/tokless-byok.js"))
                || is_open_code_transport_plugin_path_entry(entry);
            if is_managed {
                removed = true;
            } else {
                next.push(entry.clone());
            }
        }
        if removed {
            if next.is_empty() {
                cfg.shift_remove("plugin");
            } else {
                cfg.insert("plugin".to_string(), Value::Array(next));
            }
        }
    }
    if paths::read_file_safe(retrieve_stash).is_some()
        && load_open_code_retrieve_state_at(retrieve_stash).is_none()
    {
        return false;
    }
    let retrieve_state = load_open_code_retrieve_state_at(retrieve_stash);
    let suppressed = json_map_child(&cfg, "tools")
        .and_then(|t| t.get("headroom_retrieve"))
        .and_then(|v| v.as_bool())
        == Some(false);
    if !removed && retrieve_state.is_none() && !suppressed {
        return false;
    }
    if let Some(val_opt) = &retrieve_state {
        match val_opt {
            None => {
                if let Some(tools) = json_map_child_mut(&mut cfg, "tools") {
                    tools.shift_remove("headroom_retrieve");
                    if tools.is_empty() {
                        cfg.shift_remove("tools");
                    }
                }
            }
            Some(v) => {
                let tools = json_get_or_create_map(&mut cfg, "tools");
                tools.insert("headroom_retrieve".to_string(), v.clone());
            }
        }
    } else if suppressed
        && let Some(tools) = json_map_child_mut(&mut cfg, "tools")
    {
        tools.shift_remove("headroom_retrieve");
        if tools.is_empty() {
            cfg.shift_remove("tools");
        }
    }
    let next_raw = stringify_json(&Value::Object(cfg));
    if paths::write_file_atomic(file, &next_raw).is_err() {
        return false;
    }
    if retrieve_state.is_some() && clear_open_code_retrieve_state_at(retrieve_stash).is_err() {
        let _ = paths::write_file_atomic(file, &raw);
        return false;
    }
    true
}

fn configure_open_code_proxy_v2_core_locked(manage_transport: bool) -> (bool, String) {
    let file = paths::opencode_paths_v2().config;
    if let Some(raw) = paths::read_file_safe(&file)
        && has_jsonc_comments(&raw)
    {
        return (false, file);
    }
    let config_raw = paths::read_file_safe(&file);
    let retrieve_stash = open_code_retrieve_state_path_for("opencode2");
    let retrieve_raw = paths::read_file_safe(&retrieve_stash);

    let (byoks, skipped) = discover_open_code_v2_lane_byok();
    if !skipped.is_empty() {
        let msgs: Vec<String> = skipped
            .iter()
            .map(|s| {
                if s.id.is_empty() {
                    s.reason.clone()
                } else {
                    format!("{}: {}", s.id, s.reason)
                }
            })
            .collect();
        L.warn(&format!("opencode2 skipped providers: {}", msgs.join(", ")));
    }

    let mut rollbacks: Vec<ByokRouteRollback> = Vec::new();
    for provider in &byoks {
        if provider.protocol.is_empty() {
            continue;
        }
        let id = format!("opencode2:{}", provider.id);
        let Ok((_, prev_opt)) = upsert_byok_route(&id, &provider.protocol, &provider.base_url) else {
            for rb in &rollbacks {
                rollback_byok_route_logged("opencode2", &rb.id, &rb.previous);
            }
            return (false, file);
        };
        rollbacks.push(ByokRouteRollback {
            id: provider.id.clone(),
            previous: prev_opt.unwrap_or_default(),
        });
    }

    let (byoks, _) = discover_open_code_v2_lane_byok();
    let mut changed = false;
    if manage_transport {
        let (configured, ok) =
            configure_open_code_v2_retrieve_and_prune_plugin(&file, &retrieve_stash);
        if !ok || (!configured && !open_code_retrieve_tool_disabled_at(&file)) {
            for rb in &rollbacks {
                rollback_byok_route_logged("opencode2", &rb.id, &rb.previous);
            }
            return (false, file);
        }
        changed = configured;
    }

    let stash_path = byok_stash_path_for("opencode2");
    let stash_raw = paths::read_file_safe(&stash_path);
    let providers = open_code_v2_stash_for(&byoks, &file, "opencode2:");
    let mut lane_files = load_v2_byok_stash_file_at(&stash_path)
        .lane
        .map(|l| l.files)
        .unwrap_or_default();
    lane_files.push(file.clone());
    for entry in providers.values() {
        lane_files.push(entry.file.clone());
    }
    let mut unique_files = Vec::new();
    let mut seen_f = std::collections::HashSet::new();
    for f in lane_files {
        if !f.is_empty() && seen_f.insert(f.clone()) {
            unique_files.push(f);
        }
    }
    let stash_file = OpenCodeV2ByokStashFile {
        providers,
        lane: Some(OpenCodeLaneStash {
            dir: paths::opencode_paths_v2().dir,
            config: file.clone(),
            files: unique_files,
        }),
    };
    if save_v2_byok_stash_file_at(&stash_path, &stash_file).is_err() {
        for rb in &rollbacks {
            rollback_byok_route_logged("opencode2", &rb.id, &rb.previous);
        }
        return (false, file);
    }

    let (mirror_changed, ok) = sync_open_code_v2_byok_for(&byoks, &file, "opencode2:", false);
    if !ok {
        for rb in &rollbacks {
            rollback_byok_route_logged("opencode2", &rb.id, &rb.previous);
        }
        let _ = restore_file_at(&file, config_raw.as_deref());
        let _ = restore_file_at(&retrieve_stash, retrieve_raw.as_deref());
        let _ = restore_file_at(&stash_path, stash_raw.as_deref());
        return (false, file);
    }
    (changed || mirror_changed, file)
}

pub fn configure_open_code_proxy_v2() -> (bool, String) {
    let mut out = (false, paths::opencode_paths_v2().config);
    if open_code_v2_shares_v1_config() {
        if let Err(err) = with_proxy_route_stash_lock(|| {
            let (v1_changed, file) = configure_open_code_proxy_locked();
            let (v2_changed, _) = configure_open_code_proxy_v2_core_locked(false);
            out = (v1_changed || v2_changed, file);
            Ok(())
        }) {
            L.err(&format!("opencode2 proxy lock failed: {err}"));
        }
        return out;
    }
    if let Err(err) = with_proxy_route_stash_lock(|| {
        out = configure_open_code_proxy_v2_core_locked(true);
        Ok(())
    }) {
        L.err(&format!("opencode2 proxy lock failed: {err}"));
    }
    out
}

pub fn configure_opencode_proxy_v2() -> (bool, String) {
    configure_open_code_proxy_v2()
}

fn remove_open_code_proxy_v2_locked() -> bool {
    let stash_path = byok_stash_path_for("opencode2");
    if !v2_byok_stash_valid_at(&stash_path) {
        return false;
    }
    let stash = load_v2_byok_stash_file_at(&stash_path);
    let stashed = stash.providers;
    let config_path = stash
        .lane
        .as_ref()
        .map(|l| l.config.clone())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| paths::opencode_paths_v2().config);
    let retrieve_stash = open_code_retrieve_state_path_for("opencode2");
    if paths::read_file_safe(&retrieve_stash).is_some()
        && load_open_code_retrieve_state_at(&retrieve_stash).is_none()
    {
        return false;
    }

    let mut route_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for route in read_byok_routes().unwrap_or_default() {
        if let Some(stripped) = route.id.strip_prefix("opencode2:") {
            route_ids.insert(stripped.to_string());
        }
    }
    let (byoks, _) = discover_open_code_v2_lane_byok();
    for provider in &byoks {
        route_ids.insert(provider.id.clone());
    }
    for id in stashed.keys() {
        route_ids.insert(id.clone());
    }

    let config_raw = paths::read_file_safe(&config_path);
    let retrieve_raw = paths::read_file_safe(&retrieve_stash);
    let stash_raw = paths::read_file_safe(&stash_path);

    let mut unwire_files = open_code_v2_config_files();
    if let Some(lane) = &stash.lane {
        unwire_files.extend(open_code_v2_config_files_for(&lane.dir, &lane.config));
        unwire_files.extend(lane.files.clone());
    }
    for entry in stashed.values() {
        unwire_files.push(entry.file.clone());
    }
    let mut unique_files = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for f in unwire_files {
        if !f.is_empty() && seen.insert(f.clone()) {
            unique_files.push(f);
        }
    }

    let mut mirror_removed = false;
    for f in &unique_files {
        let (removed, ok) = sync_open_code_v2_byok_for(&byoks, f, "opencode2:", true);
        if !ok {
            return false;
        }
        mirror_removed = mirror_removed || removed;
    }

    let plugin_removed = remove_open_code_v2_retrieve_and_plugin(&config_path, &retrieve_stash);
    for id in &route_ids {
        if delete_byok_route("opencode2", id).is_err() {
            let _ = restore_file_at(&stash_path, stash_raw.as_deref());
            let _ = restore_file_at(&config_path, config_raw.as_deref());
            let _ = restore_file_at(&retrieve_stash, retrieve_raw.as_deref());
            return false;
        }
    }
    if clear_v2_byok_stash_at(&stash_path).is_err() {
        return false;
    }
    plugin_removed || mirror_removed
}

pub fn remove_open_code_proxy_v2() -> bool {
    let mut removed = false;
    if open_code_v2_shares_v1_config() {
        if let Err(err) = with_proxy_route_stash_lock(|| {
            let v1_removed = remove_open_code_proxy_locked();
            let v2_removed = if paths::exists(&byok_stash_path_for("opencode2")) {
                remove_open_code_proxy_v2_locked()
            } else {
                false
            };
            removed = v1_removed || v2_removed;
            Ok(())
        }) {
            L.err(&format!("opencode2 proxy lock failed: {err}"));
            return false;
        }
        return removed;
    }
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_open_code_proxy_v2_locked();
        Ok(())
    }) {
        L.err(&format!("opencode2 proxy lock failed: {err}"));
        return false;
    }
    removed
}

pub fn remove_opencode_proxy_v2() -> bool {
    remove_open_code_proxy_v2()
}

