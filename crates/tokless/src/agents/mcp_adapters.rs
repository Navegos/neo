// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/mcp_adapters.rs
// created: 2026-10-20
// lastModified: 2026-10-31

use std::path::Path;
use std::sync::Arc;
use serde_json::Value;

use crate::core::{with_any_shape, with_command, McpAdapter, McpOption, McpReply, McpRequest};
use crate::util::jsonc::{stringify_json, OrderedMap};
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::tools::copilot_ide_mcp_path;
use crate::util::paths::{
    amp_paths_resolved, antigravity_paths_resolved, claude_code_paths, cline_paths_resolved,
    codex_paths_resolved, copilot_paths_resolved, cursor_global_mcp_path, ensure_dir,
    kilo_paths_resolved, kimi_paths_resolved, omp_paths_resolved, opencode_paths_resolved,
    opencode_paths_v2, pi_paths_resolved, read_file_safe, write_file, zcode_paths_resolved,
};

use super::antigravity::*;
use super::claude::*;
use super::cline::*;
use super::codex::*;
use super::copilot::*;
use super::cursor::*;
use super::droid::*;
use super::dsh::*;
use super::grok::*;
use super::kilo::*;
use super::omp::*;
use super::opencode::*;
use super::pi::*;
use super::proxy_provider::{json_args_eq, try_parse_jsonc_map as try_parse_jsonc};

pub fn mcp_command(agent_id: &str, tool_id: &str) -> Vec<String> {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index(agent_id, pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut cmd = vec![spawn.command];
    cmd.extend(spawn.args);
    cmd
}

pub fn configure_amp_mcp(tool_id: &str) -> (bool, String) {
    let p = amp_paths_resolved();
    if let Some(parent) = Path::new(&p.config).parent() {
        let _ = ensure_dir(parent);
    }
    let raw = read_file_safe(&p.config).unwrap_or_default();
    let mut cfg = try_parse_jsonc(&raw).unwrap_or_default();
    if !raw.trim().is_empty() && cfg.is_empty() && serde_json::from_str::<Value>(&raw).is_err() {
        return (false, p.config);
    }

    if !cfg.contains_key("amp.mcpServers") {
        cfg.insert("amp.mcpServers".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(servers) = cfg.get_mut("amp.mcpServers").and_then(|v| v.as_object_mut()) else {
        return (false, p.config);
    };

    let cmd = mcp_command("amp", tool_id);
    let mut entry = OrderedMap::new();
    entry.insert("type".to_string(), Value::String("stdio".to_string()));
    entry.insert("command".to_string(), Value::String(cmd[0].clone()));
    let args_val: Vec<Value> = cmd[1..].iter().map(|s| Value::String(s.clone())).collect();
    entry.insert("args".to_string(), Value::Array(args_val));
    entry.insert("disabled".to_string(), Value::Bool(false));

    let entry_val = Value::Object(entry);
    if let Some(old) = servers.get(tool_id) {
        if stringify_json(old) == stringify_json(&entry_val) {
            return (false, p.config);
        }
    }
    servers.insert(tool_id.to_string(), entry_val);
    let next = stringify_json(&Value::Object(cfg));
    if next == raw {
        return (false, p.config);
    }
    (write_file(&p.config, &next).is_ok(), p.config)
}

pub fn amp_mcp_has(tool_id: &str, command: &[String]) -> bool {
    let raw = match read_file_safe(&amp_paths_resolved().config) {
        Some(r) => r,
        None => return false,
    };
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return false,
    };
    let Some(servers) = cfg.get("amp.mcpServers").and_then(|v| v.as_object()) else {
        return false;
    };
    let Some(entry) = servers.get(tool_id).and_then(|v| v.as_object()) else {
        return false;
    };

    let typ = entry.get("type").and_then(|v| v.as_str());
    let got = entry.get("command").and_then(|v| v.as_str());
    let disabled = entry.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);

    if command.is_empty() {
        return typ == Some("stdio") && !disabled;
    }
    typ == Some("stdio") && got == Some(&command[0]) && !disabled && json_args_eq(entry.get("args"), &command[1..])
}

pub fn remove_amp_mcp(tool_id: &str) -> bool {
    let f = amp_paths_resolved().config;
    let raw = match read_file_safe(&f) {
        Some(r) => r,
        None => return false,
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return false,
    };
    let Some(servers) = cfg.get_mut("amp.mcpServers").and_then(|v| v.as_object_mut()) else {
        return false;
    };
    if servers.remove(tool_id).is_none() {
        return false;
    }
    write_file(&f, &stringify_json(&Value::Object(cfg))).is_ok()
}

fn nested_child<'a>(cfg: &'a mut OrderedMap, path: &str, create: bool) -> Option<&'a mut OrderedMap> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = cfg;
    for part in parts {
        if !cur.contains_key(part) {
            if !create {
                return None;
            }
            cur.insert(part.to_string(), Value::Object(OrderedMap::new()));
        }
        let next = cur.get_mut(part)?.as_object_mut()?;
        cur = next;
    }
    Some(cur)
}

fn nested_child_ref<'a>(cfg: &'a OrderedMap, path: &str) -> Option<&'a OrderedMap> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = cfg;
    for part in parts {
        cur = cur.get(part)?.as_object()?;
    }
    Some(cur)
}

pub fn is_legacy_mcp_runner_str(cmd: &str) -> bool {
    let norm = cmd.replace('\\', "/");
    let stem = Path::new(&norm)
        .file_stem()
        .and_then(|st| st.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        stem.as_str(),
        "npx" | "uvx" | "pip" | "pip3" | "pipx" | "npm" | "bunx" | "pnpm" | "python" | "python3" | "node"
    )
}

pub fn is_tokless_mcp_command_str(cmd: &str) -> bool {
    let norm = cmd.replace('\\', "/");
    let stem = Path::new(&norm)
        .file_stem()
        .and_then(|st| st.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        stem.as_str(),
        "tokless"
            | "tokless-linux"
            | "tokless-linux-x64"
            | "tokless-linux-arm64"
            | "tokless-darwin"
            | "tokless-darwin-x64"
            | "tokless-darwin-arm64"
    )
}

pub fn is_legacy_mcp_entry(v: &Value) -> bool {
    let Some(m) = v.as_object() else {
        return false;
    };
    if let Some(cmd) = m.get("command") {
        match cmd {
            Value::String(s) => {
                if is_legacy_mcp_runner_str(s) {
                    return true;
                }
            }
            Value::Array(arr) => {
                if let Some(first) = arr.first().and_then(|v| v.as_str()) {
                    if is_legacy_mcp_runner_str(first) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn tokless_mcp_entry(v: &Value) -> bool {
    let Some(m) = v.as_object() else {
        return false;
    };
    let Some(command) = m.get("command") else {
        return false;
    };
    let first = match command {
        Value::String(s) => s.as_str(),
        Value::Array(arr) => match arr.first().and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return false,
        },
        _ => return false,
    };
    is_tokless_mcp_command_str(first)
}

pub struct GenericJsonMcpAdapter {
    id: String,
    path_fn: Arc<dyn Fn() -> String + Send + Sync>,
    nested: String,
}

impl McpAdapter for GenericJsonMcpAdapter {
    fn configure_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply {
        let req = build_mcp_request(&self.id, tool_id, opts);
        let path = (self.path_fn)();
        if let Some(parent) = Path::new(&path).parent() {
            let _ = ensure_dir(parent);
        }
        let raw = read_file_safe(&path).unwrap_or_default();
        let mut cfg = try_parse_jsonc(&raw).unwrap_or_default();
        if !raw.trim().is_empty() && cfg.is_empty() && serde_json::from_str::<Value>(&raw).is_err() {
            return McpReply {
                file: path.clone(),
                err: Some(format!("mcp {}: unparseable config {}", req.tool_id, path)),
                changed: false,
            };
        }
        let Some(servers) = nested_child(&mut cfg, &self.nested, true) else {
            return McpReply {
                file: path.clone(),
                err: Some(format!("mcp {}: config path {} is not an object", req.tool_id, self.nested)),
                changed: false,
            };
        };
        if let Some(current) = servers.get(&req.tool_id) {
            if !tokless_mcp_entry(current) && !is_legacy_mcp_entry(current) {
                return McpReply {
                    file: path,
                    err: Some(format!("mcp {}: foreign entry present; refusing to overwrite", req.tool_id)),
                    changed: false,
                };
            }
        }
        let mut entry = OrderedMap::new();
        let cmd0 = req.command.first().cloned().unwrap_or_default();
        let args_val: Vec<Value> = req.command.iter().skip(1).map(|s| Value::String(s.clone())).collect();
        entry.insert("command".to_string(), Value::String(cmd0));
        entry.insert("args".to_string(), Value::Array(args_val));
        entry.insert("type".to_string(), Value::String("stdio".to_string()));
        entry.insert("enable".to_string(), Value::Bool(true));
        servers.insert(req.tool_id, Value::Object(entry));

        let changed = write_file(&path, &stringify_json(&Value::Object(cfg))).is_ok();
        McpReply { changed, file: path, err: None }
    }

    fn remove_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply {
        let req = build_mcp_request(&self.id, tool_id, opts);
        let path = (self.path_fn)();
        let raw = match read_file_safe(&path) {
            Some(r) => r,
            None => return McpReply { changed: false, file: path, err: None },
        };
        let mut cfg = match try_parse_jsonc(&raw) {
            Some(c) => c,
            None => {
                if !raw.trim().is_empty() {
                    return McpReply {
                        file: path.clone(),
                        err: Some(format!("mcp {}: unparseable config {}", req.tool_id, path)),
                        changed: false,
                    };
                }
                return McpReply { changed: false, file: path, err: None };
            }
        };
        let Some(servers) = nested_child(&mut cfg, &self.nested, false) else {
            return McpReply { changed: false, file: path, err: None };
        };
        let Some(current) = servers.get(&req.tool_id) else {
            return McpReply { changed: false, file: path, err: None };
        };
        if !tokless_mcp_entry(current) && !is_legacy_mcp_entry(current) {
            return McpReply {
                file: path,
                err: Some(format!("mcp {}: foreign entry present; refusing to remove", req.tool_id)),
                changed: false,
            };
        }
        servers.remove(&req.tool_id);
        let changed = write_file(&path, &stringify_json(&Value::Object(cfg))).is_ok();
        McpReply { changed, file: path, err: None }
    }

    fn has_mcp(&self, tool_id: &str, opts: &[McpOption]) -> bool {
        let req = build_mcp_request(&self.id, tool_id, opts);
        let path = (self.path_fn)();
        let raw = match read_file_safe(&path) {
            Some(r) => r,
            None => return false,
        };
        let cfg = match try_parse_jsonc(&raw) {
            Some(c) => c,
            None => return false,
        };
        let Some(servers) = nested_child_ref(&cfg, &self.nested) else {
            return false;
        };
        let Some(v) = servers.get(&req.tool_id) else {
            return false;
        };
        if req.any_shape {
            return true;
        }
        let Some(m) = v.as_object() else {
            return false;
        };
        let cmd = m.get("command").and_then(|c| c.as_str());
        let typ = m.get("type").and_then(|t| t.as_str());
        let expected_cmd = req.command.first().map(|s| s.as_str());
        cmd == expected_cmd && json_args_eq(m.get("args"), &req.command[1..]) && (typ == Some("stdio") || typ.is_none())
    }
}

pub struct DshMcpAdapter;

impl McpAdapter for DshMcpAdapter {
    fn configure_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply {
        let req = build_mcp_request("dsh", tool_id, opts);
        let path = dsh_patch_file();
        let cmd = req.command;
        let cmd0 = cmd.first().cloned().unwrap_or_default();
        let args_json = serde_json::to_string(&cmd[1..]).unwrap_or_else(|_| "[]".to_string());
        let cmd_quoted = serde_json::to_string(&cmd0).unwrap_or_else(|_| format!("\"{}\"", cmd0));
        let entry = format!(
            "\n- insert:\n    - id: mcp-{}\n      name: '@deepseek-ai/dsh-mcp-client'\n      config:\n        serverName: {}\n        transport: stdio\n        command: {}\n        args: {}\n",
            req.tool_id, req.tool_id, cmd_quoted, args_json
        );
        let raw = read_file_safe(&path).unwrap_or_default();
        if raw.contains(&format!("serverName: {}", req.tool_id)) {
            return McpReply { changed: false, file: path, err: None };
        }
        if raw.contains(&format!("id: mcp-{}", req.tool_id)) {
            return McpReply {
                file: path,
                err: Some(format!("mcp {}: foreign entry present; refusing to overwrite", req.tool_id)),
                changed: false,
            };
        }
        if let Some(parent) = Path::new(&path).parent() {
            let _ = ensure_dir(parent);
        }
        let updated = format!("{}{}", raw, entry);
        let changed = write_file(&path, &updated).is_ok();
        McpReply { changed, file: path, err: None }
    }

    fn remove_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply {
        let req = build_mcp_request("dsh", tool_id, opts);
        let path = dsh_patch_file();
        let raw = match read_file_safe(&path) {
            Some(r) => r,
            None => return McpReply { changed: false, file: path, err: None },
        };
        let id_marker = format!("    - id: mcp-{}\n", req.tool_id);
        let Some(idx) = raw.find(&id_marker) else {
            return McpReply { changed: false, file: path, err: None };
        };
        let start = match raw[..idx].rfind("\n- insert:") {
            Some(s) => s + 1,
            None => {
                if raw.starts_with("- insert:") {
                    0
                } else {
                    return McpReply {
                        file: path,
                        err: Some(format!("mcp {}: refusing to remove foreign block", req.tool_id)),
                        changed: false,
                    };
                }
            }
        };
        let line_end = idx + id_marker.len();
        let end = match raw[line_end..].find("\n- ") {
            Some(e) => line_end + e,
            None => raw.len(),
        };
        let block = &raw[start..end];
        if !block.contains(&format!("id: mcp-{}", req.tool_id))
            || !block.contains(&format!("serverName: {}", req.tool_id))
            || !block.contains("dsh-mcp-client")
            || !block.contains("tokless")
        {
            return McpReply {
                file: path,
                err: Some(format!("mcp {}: refusing to remove foreign block", req.tool_id)),
                changed: false,
            };
        }
        let mut result = format!("{}{}", &raw[..start], &raw[end..]);
        if end == raw.len() {
            result = format!("{}\n", result.trim_end_matches(['\r', '\n']));
        }
        let changed = write_file(&path, &result).is_ok();
        McpReply { changed, file: path, err: None }
    }

    fn has_mcp(&self, tool_id: &str, opts: &[McpOption]) -> bool {
        let req = build_mcp_request("dsh", tool_id, opts);
        let raw = match read_file_safe(&dsh_patch_file()) {
            Some(r) => r,
            None => return false,
        };
        raw.contains(&format!("serverName: {}", req.tool_id))
    }
}

pub struct FnAgentMcpAdapter {
    id: String,
    configure: Arc<dyn Fn(&McpRequest) -> McpReply + Send + Sync>,
    remove: Arc<dyn Fn(&McpRequest) -> McpReply + Send + Sync>,
    has: Arc<dyn Fn(&McpRequest) -> bool + Send + Sync>,
}

impl McpAdapter for FnAgentMcpAdapter {
    fn configure_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply {
        let req = build_mcp_request(&self.id, tool_id, opts);
        (self.configure)(&req)
    }

    fn remove_mcp(&self, tool_id: &str, opts: &[McpOption]) -> McpReply {
        let req = build_mcp_request(&self.id, tool_id, opts);
        (self.remove)(&req)
    }

    fn has_mcp(&self, tool_id: &str, opts: &[McpOption]) -> bool {
        let req = build_mcp_request(&self.id, tool_id, opts);
        (self.has)(&req)
    }
}

fn build_mcp_request(agent_id: &str, tool_id: &str, opts: &[McpOption]) -> McpRequest {
    let mut req = McpRequest {
        tool_id: tool_id.to_string(),
        command: Vec::new(),
        ide: false,
        any_shape: false,
    };
    for opt in opts {
        opt(&mut req);
    }
    if req.command.is_empty() {
        req.command = mcp_command(agent_id, tool_id);
    }
    req
}

fn codex_mcp_section(tool_id: &str) -> String {
    if tool_id == "context-mode" {
        "mcp_servers.context_mode".to_string()
    } else {
        format!("mcp_servers.{tool_id}")
    }
}

fn codex_mcp_block(raw: &str, tool_id: &str) -> Option<String> {
    if let Some(text) = crate::util::toml::block_text(raw, &codex_mcp_section(tool_id)) {
        return Some(text);
    }
    if tool_id == "context-mode" {
        return crate::util::toml::block_text(raw, "mcp_servers.context-mode");
    }
    None
}

fn codex_mcp_present(tool_id: &str) -> bool {
    let Some(raw) = read_file_safe(&codex_paths_resolved().config) else {
        return false;
    };
    codex_mcp_block(&raw, tool_id).is_some()
}

fn strip_codex_toml_comment(value: &str) -> &str {
    let bytes = value.as_bytes();
    let mut in_basic = false;
    let mut in_literal = false;
    let mut escaped = false;
    for (i, &c) in bytes.iter().enumerate() {
        if in_basic {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_basic = false;
            }
            continue;
        }
        if in_literal {
            if c == b'\'' {
                in_literal = false;
            }
            continue;
        }
        match c {
            b'"' => in_basic = true,
            b'\'' => in_literal = true,
            b'#' => return value[..i].trim_end_matches([' ', '\t']),
            _ => {}
        }
    }
    value
}

fn parse_codex_mcp_block(text: &str) -> (String, Vec<String>) {
    let mut command = String::new();
    let mut args = Vec::new();
    for line in text.split('\n') {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some((k, v)) = t.split_once('=') else {
            continue;
        };
        let key = k.trim();
        let val = strip_codex_toml_comment(v.trim());
        match key {
            "command" => {
                if let Ok(s) = serde_json::from_str::<String>(val) {
                    command = s;
                } else if let Some(s) = val.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
                    command = s.to_string();
                }
            }
            "args" => {
                if val.starts_with('[') && val.ends_with(']') {
                    if let Ok(parsed) = serde_json::from_str::<Vec<String>>(val) {
                        args = parsed;
                    }
                }
            }
            _ => {}
        }
    }
    (command, args)
}

fn codex_mcp_has(tool_id: &str, command: &[String]) -> bool {
    if command.is_empty() {
        return false;
    }
    let Some(raw) = read_file_safe(&codex_paths_resolved().config) else {
        return false;
    };
    let Some(text) = codex_mcp_block(&raw, tool_id) else {
        return false;
    };
    let (cmd, args) = parse_codex_mcp_block(&text);
    cmd == command[0] && args == command[1..]
}

fn remove_codex_mcp_blocks(raw: &str, tool_id: &str) -> String {
    let mut next = crate::util::toml::remove_block(raw, &codex_mcp_section(tool_id));
    if tool_id == "context-mode" {
        next = crate::util::toml::remove_block(&next, "mcp_servers.context-mode");
    }
    next
}

fn grok_mcp_present(tool_id: &str) -> bool {
    let Some(raw) = read_file_safe(&grok_config_file()) else {
        return false;
    };
    let Some(body) = crate::util::toml::block_text(&raw, &format!("mcp_servers.{tool_id}")) else {
        return false;
    };
    let mut has_cmd = false;
    for line in body.split('\n') {
        let t = line.trim();
        if let Some((k, v)) = t.split_once('=') {
            let key = k.trim();
            let val = v.trim();
            if key == "command" && !val.is_empty() {
                has_cmd = true;
            }
            if (key == "enabled" && val == "false") || (key == "disabled" && val == "true") {
                return false;
            }
        }
    }
    has_cmd
}

pub fn mcp_adapter_for(id: &str) -> Option<Arc<dyn McpAdapter>> {
    match id {
        "amp" => Some(Arc::new(FnAgentMcpAdapter {
            id: "amp".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_amp_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_amp_mcp(&req.tool_id);
                McpReply { changed, file: amp_paths_resolved().config, err: None }
            }),
            has: Arc::new(|req| amp_mcp_has(&req.tool_id, &req.command)),
        })),
        "kimi" => Some(Arc::new(GenericJsonMcpAdapter {
            id: "kimi".to_string(),
            path_fn: Arc::new(|| kimi_paths_resolved().mcp_config),
            nested: "mcpServers".to_string(),
        })),
        "zcode" => Some(Arc::new(GenericJsonMcpAdapter {
            id: "zcode".to_string(),
            path_fn: Arc::new(|| zcode_paths_resolved().mcp_config),
            nested: "mcp.servers".to_string(),
        })),
        "dsh" => Some(Arc::new(DshMcpAdapter)),
        "claude" => Some(Arc::new(FnAgentMcpAdapter {
            id: "claude".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_claude_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_claude_mcp(&req.tool_id);
                McpReply { changed, file: claude_code_paths().global_json, err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    claude_mcp_present(&req.tool_id)
                } else {
                    claude_mcp_has(&req.tool_id, &req.command)
                }
            }),
        })),
        "opencode" => Some(Arc::new(FnAgentMcpAdapter {
            id: "opencode".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_opencode_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_opencode_mcp(&req.tool_id);
                McpReply { changed, file: opencode_paths_resolved().config, err: None }
            }),
            has: Arc::new(|req| {
                let Some(raw) = read_file_safe(&opencode_paths_resolved().config) else {
                    return false;
                };
                let Some(cfg) = try_parse_jsonc(&raw) else {
                    return false;
                };
                if req.any_shape {
                    open_code_mcp_has(&cfg, &req.tool_id)
                } else {
                    open_code_mcp_matches(&cfg, &req.tool_id, &req.command)
                }
            }),
        })),
        "opencode2" => Some(Arc::new(FnAgentMcpAdapter {
            id: "opencode2".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_opencode_mcp_v2(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_opencode_mcp_v2(&req.tool_id);
                McpReply { changed, file: opencode_paths_v2().config, err: None }
            }),
            has: Arc::new(|req| {
                let Some(raw) = read_file_safe(&opencode_paths_v2().config) else {
                    return false;
                };
                let Some(cfg) = try_parse_jsonc(&raw) else {
                    return false;
                };
                if req.any_shape {
                    open_code_mcp_has(&cfg, &req.tool_id)
                } else {
                    open_code_mcp_matches(&cfg, &req.tool_id, &req.command)
                }
            }),
        })),
        "codex" => Some(Arc::new(FnAgentMcpAdapter {
            id: "codex".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_codex_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let p = codex_paths_resolved().config;
                let Some(raw) = read_file_safe(&p) else {
                    return McpReply { changed: false, file: p, err: None };
                };
                let next = remove_codex_mcp_blocks(&raw, &req.tool_id);
                if next == raw {
                    return McpReply { changed: false, file: p, err: None };
                }
                match write_file(&p, &next) {
                    Ok(()) => McpReply { changed: true, file: p, err: None },
                    Err(e) => McpReply { changed: false, file: p, err: Some(e.to_string()) },
                }
            }),
            has: Arc::new(|req| {
                if !req.any_shape
                    && req.tool_id == "context-mode"
                    && read_file_safe(&codex_paths_resolved().config)
                        .is_some_and(|r| r.contains("[mcp_servers.context-mode]"))
                {
                    return false;
                }
                if req.any_shape {
                    codex_mcp_present(&req.tool_id)
                } else {
                    codex_mcp_has(&req.tool_id, &req.command)
                }
            }),
        })),
        "cursor" => Some(Arc::new(FnAgentMcpAdapter {
            id: "cursor".to_string(),
            configure: Arc::new(|req| {
                let (mut changed, file) = configure_cursor_mcp(&req.tool_id);
                if configure_cursor_mcp_permissions(&req.tool_id) {
                    changed = true;
                }
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let mut changed = remove_cursor_mcp(&req.tool_id);
                if remove_cursor_mcp_permissions(&req.tool_id) {
                    changed = true;
                }
                McpReply { changed, file: cursor_global_mcp_path(), err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    cursor_mcp_has(&req.tool_id)
                } else {
                    cursor_mcp_has(&req.tool_id) && has_cursor_mcp_permissions(&req.tool_id)
                }
            }),
        })),
        "antigravity" => Some(Arc::new(FnAgentMcpAdapter {
            id: "antigravity".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_antigravity_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let _ = remove_antigravity_mcp(&req.tool_id);
                McpReply { changed: true, file: String::new(), err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    antigravity_mcp_has(&req.tool_id)
                } else {
                    antigravity_mcp_bounded(&req.tool_id)
                }
            }),
        })),
        "copilot" => Some(Arc::new(FnAgentMcpAdapter {
            id: "copilot".to_string(),
            configure: Arc::new(|req| {
                if req.ide {
                    return match configure_copilot_ide_mcp_safe(&req.tool_id) {
                        Ok((changed, file)) => McpReply { changed, file, err: None },
                        Err(e) => McpReply { changed: false, file: String::new(), err: Some(e) },
                    };
                }
                let cli_res = configure_copilot_mcp_safe(&req.tool_id);
                let ide_res = configure_copilot_ide_mcp_safe(&req.tool_id);
                let cli_ok = cli_res.as_ref().map(|(c, _)| *c).unwrap_or(false);
                let ide_ok = ide_res.as_ref().map(|(c, _)| *c).unwrap_or(false);
                let file = cli_res.as_ref().map(|(_, f)| f.clone()).unwrap_or_else(|_| {
                    ide_res.as_ref().map(|(_, f)| f.clone()).unwrap_or_default()
                });
                let err = cli_res.err().or(ide_res.err());
                McpReply { changed: cli_ok || ide_ok, file, err }
            }),
            remove: Arc::new(|req| {
                if req.ide {
                    return match remove_copilot_ide_mcp_safe(&req.tool_id) {
                        Ok(changed) => McpReply { changed, file: String::new(), err: None },
                        Err(e) => McpReply { changed: false, file: String::new(), err: Some(e) },
                    };
                }
                let cli_res = remove_copilot_mcp_safe(&req.tool_id);
                let ide_res = remove_copilot_ide_mcp_safe(&req.tool_id);
                let cli_ok = cli_res.as_ref().copied().unwrap_or(false);
                let ide_ok = ide_res.as_ref().copied().unwrap_or(false);
                let err = cli_res.err().or(ide_res.err());
                McpReply { changed: cli_ok || ide_ok, file: String::new(), err }
            }),
            has: Arc::new(|req| {
                if req.ide {
                    copilot_ide_mcp_has(&req.tool_id)
                } else if req.any_shape {
                    copilot_mcp_has(&req.tool_id) && copilot_ide_mcp_has(&req.tool_id)
                } else {
                    copilot_mcp_bounded(&req.tool_id, false) && copilot_mcp_bounded(&req.tool_id, true)
                }
            }),
        })),
        "droid" => Some(Arc::new(FnAgentMcpAdapter {
            id: "droid".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_droid_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_droid_mcp(&req.tool_id);
                McpReply { changed, file: droid_mcp_file(), err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    droid_mcp_has(&req.tool_id)
                } else {
                    droid_mcp_bounded(&req.tool_id)
                }
            }),
        })),
        "grok" => Some(Arc::new(FnAgentMcpAdapter {
            id: "grok".to_string(),
            configure: Arc::new(|req| {
                match configure_grok_mcp(&req.tool_id) {
                    Ok((changed, file)) => McpReply { changed, file, err: None },
                    Err(e) => McpReply { changed: false, file: String::new(), err: Some(e) },
                }
            }),
            remove: Arc::new(|req| {
                match remove_grok_mcp(&req.tool_id) {
                    Ok(changed) => McpReply { changed, file: grok_config_file(), err: None },
                    Err(e) => McpReply { changed: false, file: grok_config_file(), err: Some(e) },
                }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    return grok_mcp_present(&req.tool_id);
                }
                match req.tool_id.as_str() {
                    "codegraph" => grok_mcp_has("codegraph"),
                    "context-mode" => grok_context_mode_mcp_has(),
                    "projectmem" => grok_projectmem_mcp_has(),
                    _ => false,
                }
            }),
        })),
        "pi" => Some(Arc::new(FnAgentMcpAdapter {
            id: "pi".to_string(),
            configure: Arc::new(|req| {
                let _ = pi_remove_mcp_adapter();
                let (changed, file) = configure_pi_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_pi_mcp(&req.tool_id);
                McpReply { changed, file: pi_paths_resolved().mcp_config, err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    pi_mcp_has(&req.tool_id)
                } else {
                    pi_mcp_bounded(&req.tool_id)
                }
            }),
        })),
        "omp" => Some(Arc::new(FnAgentMcpAdapter {
            id: "omp".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_omp_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_omp_mcp(&req.tool_id);
                McpReply { changed, file: omp_paths_resolved().config, err: None }
            }),
            has: Arc::new(|req| omp_mcp_has(&req.tool_id)),
        })),
        "kilo" => Some(Arc::new(FnAgentMcpAdapter {
            id: "kilo".to_string(),
            configure: Arc::new(|req| {
                match configure_kilo_mcp_safe(&req.tool_id, &req.command) {
                    Ok((changed, file)) => McpReply { changed, file, err: None },
                    Err(err) => McpReply { changed: false, file: String::new(), err: Some(err) },
                }
            }),
            remove: Arc::new(|req| {
                let changed = remove_kilo_mcp(&req.tool_id);
                McpReply { changed, file: kilo_paths_resolved().config, err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    kilo_mcp_configured(&req.tool_id)
                } else {
                    kilo_mcp_matches(&req.tool_id, &req.command)
                }
            }),
        })),
        "cline" => Some(Arc::new(FnAgentMcpAdapter {
            id: "cline".to_string(),
            configure: Arc::new(|req| {
                match configure_cline_mcp_safe(&req.tool_id, &req.command) {
                    Ok((changed, file)) => McpReply { changed, file, err: None },
                    Err(err) => McpReply { changed: false, file: String::new(), err: Some(err) },
                }
            }),
            remove: Arc::new(|req| {
                let changed = remove_cline_mcp(&req.tool_id);
                McpReply { changed, file: cline_paths_resolved().mcp_config, err: None }
            }),
            has: Arc::new(|req| {
                if req.any_shape {
                    cline_mcp_configured(&req.tool_id)
                } else {
                    cline_mcp_matches(&req.tool_id, &req.command)
                }
            }),
        })),
        _ => None,
    }
}

pub fn mcp_configure(agent_id: &str, tool_id: &str) -> bool {
    let Some(adapter) = mcp_adapter_for(agent_id) else {
        return true;
    };
    let reply = adapter.configure_mcp(tool_id, &[with_command(mcp_command(agent_id, tool_id))]);
    reply.err.is_none() && (reply.changed || adapter.has_mcp(tool_id, &[]))
}

pub fn mcp_remove(agent_id: &str, tool_id: &str) -> bool {
    let Some(adapter) = mcp_adapter_for(agent_id) else {
        return true;
    };
    let reply = adapter.remove_mcp(tool_id, &[with_command(mcp_command(agent_id, tool_id))]);
    reply.err.is_none()
}

pub fn mcp_has(agent_id: &str, tool_id: &str, any_shape: bool) -> bool {
    let Some(adapter) = mcp_adapter_for(agent_id) else {
        return false;
    };
    let mut opts = vec![with_command(mcp_command(agent_id, tool_id))];
    if any_shape {
        opts.push(with_any_shape());
    }
    adapter.has_mcp(tool_id, &opts)
}

pub const MANAGED_MCP_TOOLS: &[&str] = &["codegraph", "context-mode", "projectmem", "icm"];

pub fn agent_mcp_json_files(agent_id: &str) -> Vec<(String, String)> {
    match agent_id {
        "claude" => vec![(claude_code_paths().global_json, "mcpServers".into())],
        "cursor" => vec![
            (cursor_global_mcp_path(), "mcpServers".into()),
            (".cursor/mcp.json".into(), "mcpServers".into()),
        ],
        "opencode" => vec![(opencode_paths_resolved().config, "mcp".into())],
        "opencode2" => vec![(opencode_paths_v2().config, "mcp".into())],
        "antigravity" => vec![
            (antigravity_paths_resolved().mcp_config_cli, "mcpServers".into()),
            (antigravity_paths_resolved().mcp_config_ide, "mcpServers".into()),
        ],
        "copilot" => vec![
            (copilot_paths_resolved().mcp_config, "mcpServers".into()),
            (copilot_ide_mcp_path(), "servers".into()),
        ],
        "droid" => vec![(droid_mcp_file(), "mcpServers".into())],
        "kilo" => vec![(kilo_paths_resolved().config, "mcp".into())],
        "omp" => vec![
            (omp_paths_resolved().config, "mcpServers".into()),
            (format!("{}/mcp.json", omp_agent_dir_resolved()), "mcpServers".into()),
        ],
        "pi" => vec![(pi_paths_resolved().mcp_config, "mcpServers".into())],
        "cline" => vec![(cline_paths_resolved().mcp_config, "mcpServers".into())],
        "amp" => vec![(amp_paths_resolved().config, "amp.mcpServers".into())],
        "kimi" => vec![(kimi_paths_resolved().mcp_config, "mcpServers".into())],
        "zcode" => vec![(zcode_paths_resolved().mcp_config, "mcp.servers".into())],
        _ => Vec::new(),
    }
}

pub fn audit_agent_legacy_mcp(agent_id: &str) -> Vec<String> {
    let mut issues = Vec::new();
    match agent_id {
        "codex" => {
            let p = codex_paths_resolved().config;
            if let Some(raw) = read_file_safe(&p) {
                for &tool in MANAGED_MCP_TOOLS {
                    let header = format!("mcp_servers.{}", tool);
                    if let Some(block) = crate::util::toml::block_text(&raw, &header) {
                        for line in block.lines() {
                            let t = line.trim();
                            if let Some((k, v)) = t.split_once('=') {
                                if k.trim() == "command" {
                                    let cmd = v.trim().trim_matches('"').trim_matches('\'');
                                    if is_legacy_mcp_runner_str(cmd) {
                                        issues.push(format!("{}: legacy runner {}", tool, cmd));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        "grok" => {
            let p = grok_config_file();
            if let Some(raw) = read_file_safe(&p) {
                for &tool in MANAGED_MCP_TOOLS {
                    let header = format!("mcp_servers.{}", tool);
                    if let Some(block) = crate::util::toml::block_text(&raw, &header) {
                        for line in block.lines() {
                            let t = line.trim();
                            if let Some((k, v)) = t.split_once('=') {
                                if k.trim() == "command" {
                                    let cmd = v.trim().trim_matches('"').trim_matches('\'');
                                    if is_legacy_mcp_runner_str(cmd) {
                                        issues.push(format!("{}: legacy runner {}", tool, cmd));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        "dsh" => {
            let p = dsh_patch_file();
            if let Some(raw) = read_file_safe(&p) {
                for &tool in MANAGED_MCP_TOOLS {
                    let marker = format!("id: mcp-{}", tool);
                    if raw.contains(&marker) && (raw.contains("npx") || raw.contains("uvx") || raw.contains("pip")) {
                        issues.push(format!("{}: legacy runner in dsh patch", tool));
                    }
                }
            }
        }
        _ => {
            let json_files = agent_mcp_json_files(agent_id);
            for (path, key) in json_files {
                if let Some(raw) = read_file_safe(&path) {
                    if let Some(cfg) = try_parse_jsonc(&raw) {
                        if let Some(servers) = nested_child_ref(&cfg, &key) {
                            for &tool in MANAGED_MCP_TOOLS {
                                if let Some(v) = servers.get(tool) {
                                    if is_legacy_mcp_entry(v) {
                                        issues.push(format!("{}: legacy runner in {}", tool, path));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    issues
}

pub fn purge_legacy_mcp_runners(agent_id: &str) -> Vec<String> {
    let mut purged = Vec::new();
    match agent_id {
        "codex" => {
            let p = codex_paths_resolved().config;
            if let Some(raw) = read_file_safe(&p) {
                let mut next = raw.clone();
                for &tool in MANAGED_MCP_TOOLS {
                    let header = format!("mcp_servers.{}", tool);
                    if let Some(block) = crate::util::toml::block_text(&next, &header) {
                        if block.contains("npx") || block.contains("uvx") || block.contains("pip") || block.contains("python") {
                            next = crate::util::toml::remove_block(&next, &header);
                            purged.push(format!("{}: {}", agent_id, tool));
                        }
                    }
                }
                if next != raw {
                    let _ = write_file(&p, &next);
                }
            }
        }
        "grok" => {
            let p = grok_config_file();
            if let Some(raw) = read_file_safe(&p) {
                let mut next = raw.clone();
                for &tool in MANAGED_MCP_TOOLS {
                    let header = format!("mcp_servers.{}", tool);
                    if let Some(block) = crate::util::toml::block_text(&next, &header) {
                        if block.contains("npx") || block.contains("uvx") || block.contains("pip") || block.contains("python") {
                            next = crate::util::toml::remove_block(&next, &header);
                            purged.push(format!("{}: {}", agent_id, tool));
                        }
                    }
                }
                if next != raw {
                    let _ = write_file(&p, &next);
                }
            }
        }
        "dsh" => {
            let p = dsh_patch_file();
            if let Some(raw) = read_file_safe(&p) {
                for &tool in MANAGED_MCP_TOOLS {
                    let marker = format!("id: mcp-{}", tool);
                    if raw.contains(&marker) && (raw.contains("npx") || raw.contains("uvx") || raw.contains("pip")) {
                        let _ = remove_dsh_mcp_block(&p, tool);
                        purged.push(format!("{}: {}", agent_id, tool));
                    }
                }
            }
        }
        _ => {
            let json_files = agent_mcp_json_files(agent_id);
            for (path, key) in json_files {
                if let Some(raw) = read_file_safe(&path) {
                    if let Some(mut cfg) = try_parse_jsonc(&raw) {
                        let mut changed = false;
                        if let Some(servers) = nested_child(&mut cfg, &key, false) {
                            for &tool in MANAGED_MCP_TOOLS {
                                if let Some(v) = servers.get(tool) {
                                    if is_legacy_mcp_entry(v) {
                                        servers.remove(tool);
                                        changed = true;
                                        purged.push(format!("{}: {}", agent_id, tool));
                                    }
                                }
                            }
                        }
                        if changed {
                            let _ = write_file(&path, &stringify_json(&Value::Object(cfg)));
                        }
                    }
                }
            }
        }
    }
    purged
}

fn remove_dsh_mcp_block(path: &str, tool_id: &str) -> bool {
    let raw = match read_file_safe(path) {
        Some(r) => r,
        None => return false,
    };
    let id_marker = format!("    - id: mcp-{}\n", tool_id);
    let Some(idx) = raw.find(&id_marker) else {
        return false;
    };
    let start = match raw[..idx].rfind("\n- insert:") {
        Some(s) => s + 1,
        None => {
            if raw.starts_with("- insert:") {
                0
            } else {
                return false;
            }
        }
    };
    let line_end = idx + id_marker.len();
    let end = match raw[line_end..].find("\n- ") {
        Some(e) => line_end + e,
        None => raw.len(),
    };
    let mut result = format!("{}{}", &raw[..start], &raw[end..]);
    if end == raw.len() {
        result = format!("{}\n", result.trim_end_matches(['\r', '\n']));
    }
    write_file(path, &result).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_17_agents_have_mcp_adapters() {
        let expected = [
            "amp", "kimi", "zcode", "dsh", "claude", "opencode", "opencode2",
            "codex", "cursor", "antigravity", "copilot", "droid", "grok",
            "pi", "omp", "kilo", "cline",
        ];

        for agent in expected {
            let adapter = mcp_adapter_for(agent);
            assert!(adapter.is_some(), "missing mcp adapter for agent: {}", agent);
        }

        assert!(mcp_adapter_for("unknown_agent").is_none());
    }

    #[test]
    fn test_mcp_command_generation() {
        let cmd = mcp_command("claude", "contextmode");
        assert!(!cmd.is_empty());
        let cmd_cg = mcp_command("claude", "codegraph");
        assert!(!cmd_cg.is_empty());
    }
}
