// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/mcp_adapters.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::path::Path;
use std::sync::Arc;
use serde_json::Value;

use crate::core::{with_any_shape, with_command, McpAdapter, McpOption, McpReply, McpRequest};
use crate::util::jsonc::{stringify_json, OrderedMap};
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths::{
    amp_paths_resolved, claude_code_paths, cline_paths_resolved, codex_paths_resolved,
    cursor_global_mcp_path, ensure_dir, kilo_paths_resolved, kimi_paths_resolved,
    omp_paths_resolved, opencode_paths_resolved, opencode_paths_v2, pi_paths_resolved,
    read_file_safe, write_file, zcode_paths_resolved,
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
    let filename = Path::new(first).file_stem().and_then(|s| s.to_str()).unwrap_or("");
    filename.eq_ignore_ascii_case("tokless")
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
            if !tokless_mcp_entry(current) {
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
        if !tokless_mcp_entry(current) {
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
            has: Arc::new(|req| opencode_mcp_has(&req.tool_id)),
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
            has: Arc::new(|req| opencode2_mcp_has(&req.tool_id)),
        })),
        "codex" => Some(Arc::new(FnAgentMcpAdapter {
            id: "codex".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_codex_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let p = codex_paths_resolved().config;
                let raw = read_file_safe(&p).unwrap_or_default();
                let next = crate::util::toml::remove_block(&raw, &format!("mcp_servers.{}", req.tool_id));
                let changed = write_file(&p, &next).is_ok();
                McpReply { changed, file: p, err: None }
            }),
            has: Arc::new(|req| {
                let p = codex_paths_resolved().config;
                read_file_safe(&p).map(|r| r.contains(&format!("mcp_servers.{}", req.tool_id))).unwrap_or(false)
            }),
        })),
        "cursor" => Some(Arc::new(FnAgentMcpAdapter {
            id: "cursor".to_string(),
            configure: Arc::new(|req| {
                let (changed, file) = configure_cursor_mcp(&req.tool_id);
                McpReply { changed, file, err: None }
            }),
            remove: Arc::new(|req| {
                let changed = remove_cursor_mcp(&req.tool_id);
                McpReply { changed, file: cursor_global_mcp_path(), err: None }
            }),
            has: Arc::new(|req| cursor_mcp_has(&req.tool_id)),
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
