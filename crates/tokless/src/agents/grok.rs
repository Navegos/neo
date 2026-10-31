// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/grok.rs
// created: 2026-10-11
// lastModified: 2026-10-31

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use crate::core::AgentManifest;
use crate::util::grok::grok_oauth_proxy_port;
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;
use crate::util::toml::{
    TomlBlock, block_text, has_block, remove_block, toml_block_field, toml_quoted, upsert_block,
};

use super::proxy_detection::{
    ProxyCapability, ProxyConfigState, ProxyDetection, ProxyState, classify_grok_toml_proxy,
    proxy_detection,
};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    ERR_NO_STABLE_TOKLESS, ProxyRouteStashEntry, delete_byok_route, detect_agent_with,
    is_valid_byok_upstream, load_proxy_route_stash, load_proxy_route_stash_locked,
    normalized_headroom_upstream, proxy_endpoint_for_api, proxy_route_stash_path,
    proxy_route_stash_valid, proxy_wire_key, register_byok_route, restore_proxy_route_stash_logged,
    rollback_byok_route_logged, save_proxy_route_stash,
    tokless_command_ok, tokless_managed_command, with_proxy_route_stash_lock,
};

pub const GROK_BUILD_MARKER_START: &str = "/* tokless:headroom-grok-start */";
pub const GROK_BUILD_MARKER_END: &str = "/* tokless:headroom-grok-end */";
pub const GROK_SHIM_MARKER: &str = "# tokless:grok-launcher";
pub const GROK_OAUTH_UPSTREAM: &str = "https://cli-chat-proxy.grok.com";

pub fn grok_dir() -> String {
    paths::grok_paths_resolved().dir
}

pub fn grok_config_file() -> String {
    paths::grok_paths_resolved().config
}

pub fn grok_instructions_file() -> String {
    Path::new(&grok_dir())
        .join("rules")
        .join("AGENTS.md")
        .to_string_lossy()
        .into_owned()
}

pub fn grok_bin_file() -> String {
    paths::grok_paths_resolved().bin
}

pub fn grok_real_bin_file() -> String {
    let name = if paths::is_win() {
        "grok.real.exe"
    } else {
        "grok.real"
    };
    Path::new(&paths::grok_paths_resolved().install_dir)
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn grok_bin_dirs() -> Vec<String> {
    vec![paths::grok_paths_resolved().install_dir]
}

pub fn grok_manifest() -> AgentManifest {
    AgentManifest {
        id: "grok".to_string(),
        label: "Grok CLI".to_string(),
        homepage: "https://grok.com".to_string(),
        cli_bin: "grok".to_string(),
        config_dir: Some(Arc::new(grok_dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with("grok", &grok_dir(), &grok_bin_dirs(), &[])
        })),
    }
}

pub fn configure_grok_mcp(tool_id: &str) -> Result<(bool, String), String> {
    let p = grok_config_file();
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return Err(format!(
            "Grok config {p} is unreadable; refusing to overwrite"
        ));
    }
    let raw = raw_opt.unwrap_or_default();
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("grok", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let header = format!("mcp_servers.{tool_id}");
    let mut block = TomlBlock::new(&header);
    block.set("command", &toml_quoted(&spawn.command));
    if !spawn.args.is_empty() {
        let quoted: Vec<String> = spawn.args.iter().map(|s| toml_quoted(s)).collect();
        block.set("args", &format!("[{}]", quoted.join(", ")));
    }
    if tool_id == "context-mode" {
        block.set("env", "{ CONTEXT_MODE_PLATFORM = \"grok-cli\" }");
    }
    let next = upsert_block(&raw, &block, false);
    if next == raw {
        return Ok((false, p));
    }
    paths::write_file_atomic(&p, &next).map_err(|e| e.to_string())?;
    Ok((true, p))
}

pub fn remove_grok_mcp(tool_id: &str) -> Result<bool, String> {
    let p = grok_config_file();
    let Some(raw) = paths::read_file_safe(&p) else {
        if paths::exists(&p) {
            return Err(format!("Grok config {p} is unreadable"));
        }
        return Ok(false);
    };
    let header = format!("mcp_servers.{tool_id}");
    let Some(body) = block_text(&raw, &header) else {
        return Ok(false);
    };
    if !grok_mcp_block_managed(tool_id, &body) {
        return Ok(false);
    }
    let next = remove_block(&raw, &header);
    if next == raw {
        return Ok(false);
    }
    paths::write_file_atomic(&p, &next).map_err(|e| e.to_string())?;
    Ok(true)
}

fn grok_mcp_block_managed(tool_id: &str, body: &str) -> bool {
    let mut cmd_str = String::new();
    for line in body.split('\n') {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once('=')
            && k.trim() == "command"
        {
            cmd_str = unquote_toml_str(v.trim());
            break;
        }
    }
    if cmd_str.is_empty() {
        return false;
    }
    let base_cmd = Path::new(&cmd_str.replace('\\', "/"))
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match tool_id {
        "context-mode" => {
            base_cmd.contains("context-mode")
                || (base_cmd.contains("npx") && body.contains("\"context-mode\""))
        }
        "codegraph" => {
            (base_cmd.contains("tokless")
                && body.contains("\"--agent\"")
                && body.contains("\"grok\"")
                && body.contains("codegraph"))
                || (base_cmd.contains("codegraph")
                    && body.contains("\"serve\"")
                    && body.contains("\"--mcp\""))
                || (base_cmd.contains("npx") && body.contains("@colbymchenry/codegraph"))
        }
        "projectmem" => {
            base_cmd.contains("pjm-mcp")
                || (base_cmd.contains("tokless") && body.contains("projectmem"))
        }
        _ => false,
    }
}

pub fn grok_mcp_has(tool_id: &str) -> bool {
    paths::read_file_safe(&grok_config_file())
        .is_some_and(|raw| has_block(&raw, &format!("mcp_servers.{tool_id}")))
}

pub fn grok_context_mode_mcp_has() -> bool {
    let Some(raw) = paths::read_file_safe(&grok_config_file()) else {
        return false;
    };
    let Some(body) = block_text(&raw, "mcp_servers.context-mode") else {
        return false;
    };
    grok_mcp_block_managed("context-mode", &body)
        && body.contains("CONTEXT_MODE_PLATFORM = \"grok-cli\"")
}

pub fn grok_projectmem_mcp_has() -> bool {
    let Some(raw) = paths::read_file_safe(&grok_config_file()) else {
        return false;
    };
    let Some(body) = block_text(&raw, "mcp_servers.projectmem") else {
        return false;
    };
    grok_mcp_block_managed("projectmem", &body)
}

pub fn install_grok_rtk_hook() -> Result<(), String> {
    let (command, ok) = tokless_command_ok(&["rtk-hook", "grok"]);
    if !ok {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }
    grok_hooks_set("pre_tool_use", &command, is_grok_rtk_command)
}

pub fn remove_grok_rtk_hook() {
    let _ = grok_hooks_remove("pre_tool_use", is_grok_rtk_command);
}

pub fn has_grok_rtk_hook() -> bool {
    grok_hooks_has("pre_tool_use", is_grok_rtk_command)
}

pub fn install_grok_codegraph_session_hook() -> Result<(), String> {
    let (command, ok) = tokless_command_ok(&["grok-hook", "codegraph-index"]);
    if !ok {
        return Err(ERR_NO_STABLE_TOKLESS.to_string());
    }
    grok_hooks_set("on_session_start", &command, is_grok_codegraph_session_command)
}

pub fn has_grok_codegraph_session_hook() -> bool {
    grok_hooks_has("on_session_start", is_grok_codegraph_session_command)
}

pub fn remove_grok_codegraph_session_hook() {
    let _ = grok_hooks_remove("on_session_start", is_grok_codegraph_session_command);
}

fn is_grok_rtk_command(cmd: &str) -> bool {
    tokless_managed_command(cmd, &["rtk-hook", "grok"])
}

fn is_grok_codegraph_session_command(cmd: &str) -> bool {
    tokless_managed_command(cmd, &["grok-hook", "codegraph-index"])
}

fn unquote_toml_str(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        if let Ok(unquoted) = serde_json::from_str::<String>(trimmed) {
            return unquoted;
        }
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

fn parse_toml_string_array(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    let inner = trimmed[1..trimmed.len() - 1].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let bytes = inner.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let quote = bytes[i];
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        let start = i;
        i += 1;
        let mut escaped = false;
        while i < bytes.len() {
            let c = bytes[i];
            if quote == b'"' && escaped {
                escaped = false;
                i += 1;
                continue;
            }
            if quote == b'"' && c == b'\\' {
                escaped = true;
                i += 1;
                continue;
            }
            if c == quote {
                break;
            }
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != quote {
            return None;
        }
        let token = &inner[start..=i];
        out.push(unquote_toml_str(token));
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() {
            if bytes[i] != b',' {
                return None;
            }
            i += 1;
        }
    }
    Some(out)
}

fn format_toml_string_array(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| toml_quoted(s)).collect();
    format!("[{}]", quoted.join(", "))
}

fn grok_hooks_set(key: &str, command: &str, pred: fn(&str) -> bool) -> Result<(), String> {
    let p = grok_config_file();
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return Err(format!(
            "Grok config {p} is unreadable; refusing to overwrite"
        ));
    }
    let raw = raw_opt.unwrap_or_default();
    let cur_field = toml_block_field(&raw, "hooks", key);
    let mut list = if cur_field.is_empty() {
        Vec::new()
    } else {
        parse_toml_string_array(&cur_field)
            .ok_or_else(|| format!("Grok hooks.{key} is not a valid string array"))?
    };
    list.retain(|item| !pred(item));
    list.push(command.to_string());
    let mut block = TomlBlock::new("hooks");
    block.set(key, &format_toml_string_array(&list));
    let next = upsert_block(&raw, &block, true);
    if next != raw {
        paths::write_file_atomic(&p, &next).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn grok_hooks_remove(key: &str, pred: fn(&str) -> bool) -> Result<(), String> {
    let p = grok_config_file();
    let Some(raw) = paths::read_file_safe(&p) else {
        return Ok(());
    };
    let cur_field = toml_block_field(&raw, "hooks", key);
    if cur_field.is_empty() {
        return Ok(());
    }
    let Some(mut list) = parse_toml_string_array(&cur_field) else {
        return Ok(());
    };
    let before = list.len();
    list.retain(|item| !pred(item));
    if list.len() == before {
        return Ok(());
    }
    let mut block = TomlBlock::new("hooks");
    block.set(key, &format_toml_string_array(&list));
    let next = upsert_block(&raw, &block, true);
    if next != raw {
        paths::write_file_atomic(&p, &next).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn grok_hooks_has(key: &str, pred: fn(&str) -> bool) -> bool {
    let Some(raw) = paths::read_file_safe(&grok_config_file()) else {
        return false;
    };
    let cur_field = toml_block_field(&raw, "hooks", key);
    if cur_field.is_empty() {
        return false;
    }
    let Some(list) = parse_toml_string_array(&cur_field) else {
        return false;
    };
    list.iter().any(|item| pred(item))
}

// --- grok_build.go ---

pub(crate) fn strip_grok_build_blocks(src: &str) -> (String, bool) {
    if !src.contains(GROK_BUILD_MARKER_START) {
        return (src.to_string(), false);
    }
    let mut cur = src.to_string();
    let mut changed = false;
    while let Some(s) = cur.find(GROK_BUILD_MARKER_START) {
        let Some(e_rel) = cur[s..].find(GROK_BUILD_MARKER_END) else {
            break;
        };
        let mut end = s + e_rel + GROK_BUILD_MARKER_END.len();
        if cur[end..].starts_with("\r\n") {
            end += 2;
        } else if cur[end..].starts_with('\n') {
            end += 1;
        }
        cur = format!("{}{}", &cur[..s], &cur[end..]);
        changed = true;
    }
    (cur, changed)
}

pub fn remove_grok_build_proxy() -> bool {
    let repo = std::env::var("TOKLESS_GROK_REPO").unwrap_or_default();
    let trimmed = repo.trim();
    if trimmed.is_empty() {
        return false;
    }
    let rels = [
        "src/api/client.ts",
        "src/commands/ModelsCommand.tsx",
        "src/config/config.ts",
    ];
    let mut changed = false;
    for rel in rels {
        let p = Path::new(trimmed).join(rel);
        let p_str = p.to_string_lossy().into_owned();
        let Some(raw) = paths::read_file_safe(&p_str) else {
            continue;
        };
        let (stripped, ok) = strip_grok_build_blocks(&raw);
        if ok && paths::write_file_atomic(&p_str, &stripped).is_ok() {
            changed = true;
        }
    }
    changed
}

// --- grok_shim.go ---

fn grok_shim_state_file() -> String {
    Path::new(&paths::tokless_data_dir())
        .join("grok-cli-proxy.json")
        .to_string_lossy()
        .into_owned()
}

fn looks_like_grok_shim(raw: &str) -> bool {
    if paths::is_win() {
        return raw.starts_with("@echo off\r\n")
            && raw.contains(GROK_SHIM_MARKER)
            && raw.contains("__grok-proxy-ensure")
            && raw.contains("HTTPS_PROXY=http://127.0.0.1:")
            && raw.contains("NODE_EXTRA_CA_CERTS=")
            && raw.contains("\"%REAL%\" %*");
    }
    raw.starts_with("#!/bin/sh\n")
        && raw.contains(GROK_SHIM_MARKER)
        && raw.contains("TOKLESS=")
        && raw.contains("__grok-proxy-ensure")
        && raw.contains("HTTPS_PROXY=http://127.0.0.1:")
        && raw.contains("NODE_EXTRA_CA_CERTS=")
        && raw.trim().ends_with("exec \"$REAL\" \"$@\"")
}

fn read_grok_launcher(path: &str) -> Result<(String, bool), std::io::Error> {
    let mut f = File::open(path)?;
    let mut header = [0u8; 2];
    let n = f.read(&mut header)?;
    let is_script = (n >= 2 && &header == b"#!")
        || (paths::is_win() && paths::read_file_safe(path).is_some_and(|s| s.starts_with("@echo off")));
    if !is_script {
        return Ok((String::new(), false));
    }
    f.seek(SeekFrom::Start(0))?;
    let mut buf = Vec::new();
    f.take(64 << 10).read_to_end(&mut buf)?;
    Ok((String::from_utf8_lossy(&buf).into_owned(), true))
}

pub fn install_grok_shim() -> Result<bool, String> {
    // Purged in Phase 9: Tokless never hijacks agent binaries or creates shim wrappers.
    let _ = remove_grok_shim();
    Ok(false)
}

pub fn remove_grok_shim() -> bool {
    remove_grok_shim_checked().unwrap_or(false)
}

pub fn remove_grok_shim_checked() -> Result<bool, String> {
    let bin = grok_bin_file();
    let real = grok_real_bin_file();
    let mut changed = false;

    if paths::is_win() {
        if let Some(raw) = paths::read_file_safe(&bin) {
            if looks_like_grok_shim(&raw) {
                let _ = fs::remove_file(&bin);
                changed = true;
            }
        }
        if paths::exists(&real) {
            if paths::exists(&bin) {
                let _ = fs::remove_file(&bin);
            }
            if fs::rename(&real, &bin).is_ok() {
                changed = true;
            }
        }
    } else {
        if let Ok((raw, script)) = read_grok_launcher(&bin) {
            if script && looks_like_grok_shim(&raw) {
                let _ = fs::remove_file(&bin);
                changed = true;
            }
        }
        if paths::exists(&real) {
            if paths::exists(&bin) {
                let _ = fs::remove_file(&bin);
            }
            if fs::rename(&real, &bin).is_ok() {
                changed = true;
            }
        }
    }

    if paths::exists(&grok_shim_state_file()) {
        let _ = fs::remove_file(grok_shim_state_file());
        changed = true;
    }

    Ok(changed)
}

pub fn grok_shim_wired() -> bool {
    false
}

// --- grok_proxy.go ---

pub fn grok_proxy_applicable() -> bool {
    let d = detect_agent_with("grok", &grok_dir(), &grok_bin_dirs(), &[]);
    d.installed || grok_proxy_wired()
}

pub fn configure_grok_proxy() -> (bool, String) {
    match configure_grok_proxy_checked() {
        Ok(res) => res,
        Err(err) => {
            L.sub(&format!("grok proxy: {err}"));
            (false, grok_config_file())
        }
    }
}

pub fn configure_grok_proxy_checked() -> Result<(bool, String), String> {
    let mut out = (false, grok_config_file());
    with_proxy_route_stash_lock(|| {
        out = configure_grok_proxy_locked()?;
        Ok(())
    })?;
    Ok(out)
}

fn configure_grok_proxy_locked() -> Result<(bool, String), String> {
    let _ = remove_grok_build_proxy();
    let cfg_path = grok_config_file();
    let raw_opt = paths::read_file_safe(&cfg_path);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&cfg_path) {
        return Err(format!("Grok config {cfg_path} is unreadable"));
    }
    let raw = raw_opt.unwrap_or_default();
    if !proxy_route_stash_valid("grok") {
        return Err("Grok proxy stash is malformed".to_string());
    }
    let stash = load_proxy_route_stash_locked("grok");
    let stashed = stash.get("api");
    let state = classify_grok_toml_proxy(&raw, &proxy_endpoint_for("grok"), stashed);

    match state {
        ProxyConfigState::Unreadable | ProxyConfigState::Conflict => {
            return Err("Grok config has conflicting or unreadable [api] settings".to_string());
        }
        ProxyConfigState::ForeignBYOK => {
            return configure_grok_byok_locked(&cfg_path, &raw, exists, stash);
        }
        ProxyConfigState::Managed => {
            if stashed.is_some_and(|s| !s.upstream.is_empty()) {
                return Ok((false, cfg_path));
            }
        }
        ProxyConfigState::Absent
        | ProxyConfigState::NativeUnconfigured
        | ProxyConfigState::Unknown => {}
    }

    let changed = install_grok_shim()?;
    Ok((changed, grok_bin_file()))
}

fn configure_grok_byok_locked(
    cfg_path: &str,
    raw: &str,
    exists: bool,
    mut stash: HashMap<String, ProxyRouteStashEntry>,
) -> Result<(bool, String), String> {
    let base_raw = toml_block_field(raw, "api", "base_url");
    let base_url = unquote_toml_str(&base_raw);
    let key_raw = toml_block_field(raw, "api", "api_key");
    let had_key = !key_raw.is_empty();
    let upstream = normalized_headroom_upstream(&base_url, "openai-completions");
    if !is_valid_byok_upstream(&upstream) {
        return Err(format!("unsupported Grok BYOK upstream {base_url}"));
    }
    let (_, route_token_hdr, prev_route, ok) =
        register_byok_route("grok", "api", "openai-completions", &upstream);
    if !ok {
        return Err("failed to register Grok BYOK route".to_string());
    }
    let route_token = route_token_hdr
        .split_once(':')
        .map(|(_, v)| v.trim().to_string())
        .unwrap_or_else(|| proxy_wire_key());

    let stash_path = proxy_route_stash_path("grok");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut block = TomlBlock::new("api");
    let endpoint = proxy_endpoint_for_api("openai-completions");
    block.set("base_url", &toml_quoted(&endpoint));
    if !had_key {
        block.set("api_key", &toml_quoted(&route_token));
    }
    let next = upsert_block(raw, &block, true);

    stash.insert(
        "api".to_string(),
        ProxyRouteStashEntry {
            file: cfg_path.to_string(),
            provider: "api".to_string(),
            base_url: base_url.clone(),
            upstream,
            base_key: "base_url".to_string(),
            had_base_key: true,
            had_auth: had_key,
            original: raw.as_bytes().to_vec(),
            managed: next.as_bytes().to_vec(),
            ..Default::default()
        },
    );

    if let Err(err) = save_proxy_route_stash("grok", &stash) {
        rollback_byok_route_logged("grok", "api", &prev_route);
        return Err(err);
    }
    if let Err(err) = paths::write_file_atomic(cfg_path, &next) {
        restore_proxy_route_stash_logged(
            "grok",
            prev_stash_raw.as_deref().unwrap_or(""),
            prev_stash_exists,
        );
        rollback_byok_route_logged("grok", "api", &prev_route);
        if !exists {
            let _ = fs::remove_file(cfg_path);
        }
        return Err(err.to_string());
    }
    Ok((true, cfg_path.to_string()))
}

pub fn remove_grok_proxy() -> bool {
    remove_grok_proxy_checked().unwrap_or(false)
}

pub fn remove_grok_proxy_checked() -> Result<bool, String> {
    let mut removed = false;
    with_proxy_route_stash_lock(|| {
        removed = remove_grok_proxy_locked()?;
        Ok(())
    })?;
    Ok(removed)
}

fn remove_grok_proxy_locked() -> Result<bool, String> {
    let mut removed = remove_grok_build_proxy();
    if remove_grok_shim_checked()? {
        removed = true;
    }
    if !proxy_route_stash_valid("grok") {
        return Err("Grok proxy stash is malformed".to_string());
    }
    let mut stash = load_proxy_route_stash_locked("grok");
    let Some(entry) = stash.get("api").cloned() else {
        return Ok(removed);
    };
    let cfg_path = grok_config_file();
    let Some(raw) = paths::read_file_safe(&cfg_path) else {
        let _ = delete_byok_route("grok", "api");
        stash.remove("api");
        let _ = save_proxy_route_stash("grok", &stash);
        return Ok(true);
    };
    let state = classify_grok_toml_proxy(&raw, &proxy_endpoint_for("grok"), Some(&entry));
    if state != ProxyConfigState::Managed {
        return Err("Grok [api] block was modified outside Tokless; refusing to remove".to_string());
    }
    let restored = if !entry.original.is_empty() && raw.as_bytes() == entry.managed.as_slice() {
        String::from_utf8_lossy(&entry.original).into_owned()
    } else {
        let mut block = TomlBlock::new("api");
        block.set("base_url", &toml_quoted(&entry.base_url));
        upsert_block(&raw, &block, true)
    };
    paths::write_file_atomic(&cfg_path, &restored).map_err(|e| e.to_string())?;
    let _ = delete_byok_route("grok", "api");
    stash.remove("api");
    save_proxy_route_stash("grok", &stash)?;
    Ok(true)
}

pub fn grok_oauth_proxy_wired() -> bool {
    grok_shim_wired()
}

pub fn grok_proxy_wired() -> bool {
    if grok_shim_wired() {
        return true;
    }
    let Some(raw) = paths::read_file_safe(&grok_config_file()) else {
        return false;
    };
    let stash = load_proxy_route_stash("grok");
    classify_grok_toml_proxy(&raw, &proxy_endpoint_for("grok"), stash.get("api"))
        == ProxyConfigState::Managed
}

pub fn grok_proxy_uses_headroom() -> bool {
    grok_proxy_wired()
}

pub(crate) fn detect_grok_proxy(cap: &ProxyCapability) -> ProxyDetection {
    if grok_shim_wired() {
        return proxy_detection(
            &cap.id,
            &format!(
                "managed OAuth launcher shim -> 127.0.0.1:{}",
                grok_oauth_proxy_port()
            ),
            ProxyState::ProxyStateManaged,
        );
    }
    let cfg_path = grok_config_file();
    if let Some(raw) = paths::read_file_safe(&cfg_path) {
        let stash = load_proxy_route_stash("grok");
        let st = classify_grok_toml_proxy(&raw, &proxy_endpoint_for("grok"), stash.get("api"));
        match st {
            ProxyConfigState::Managed => {
                return proxy_detection(
                    &cap.id,
                    "managed [api] base_url in config.toml",
                    ProxyState::ProxyStateManaged,
                );
            }
            ProxyConfigState::ForeignBYOK => {
                return proxy_detection(
                    &cap.id,
                    "custom [api] base_url in config.toml",
                    ProxyState::ProxyStateForeignBYOK,
                );
            }
            ProxyConfigState::Conflict => {
                return proxy_detection(
                    &cap.id,
                    "conflicting [api] block in config.toml",
                    ProxyState::ProxyStateConflict,
                );
            }
            ProxyConfigState::Unreadable => {
                return proxy_detection(
                    &cap.id,
                    "unreadable config.toml",
                    ProxyState::ProxyStateUnreadable,
                );
            }
            ProxyConfigState::Absent
            | ProxyConfigState::NativeUnconfigured
            | ProxyConfigState::Unknown => {}
        }
    } else if paths::exists(&cfg_path) {
        return proxy_detection(
            &cap.id,
            "unreadable config.toml",
            ProxyState::ProxyStateUnreadable,
        );
    }
    let d = detect_agent_with("grok", &grok_dir(), &grok_bin_dirs(), &[]);
    if !d.installed {
        return proxy_detection(&cap.id, "Grok CLI absent", ProxyState::ProxyStateAbsent);
    }
    proxy_detection(
        &cap.id,
        "Grok CLI not wired to Headroom proxy",
        ProxyState::ProxyStateNativeUnconfigured,
    )
}
