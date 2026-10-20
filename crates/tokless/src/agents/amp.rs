// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/amp.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::collections::HashSet;
use std::sync::Arc;
use serde_json::Value;

use crate::core::AgentManifest;
use crate::util::jsonc::{stringify_json, OrderedMap};
use crate::util::paths::{amp_paths_resolved, expected_bin_dirs, read_file_safe, write_file};
use crate::util::rtk_commands::rtk_commands;
use super::proxy_provider::{detect_agent_with, try_parse_jsonc_map as try_parse_jsonc};

pub fn amp_manifest() -> AgentManifest {
    AgentManifest {
        id: "amp".to_string(),
        label: "Amp".to_string(),
        homepage: "https://ampcode.com".to_string(),
        cli_bin: "amp".to_string(),
        config_dir: Some(Arc::new(|| amp_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with("amp", &amp_paths_resolved().dir, &expected_bin_dirs(), &[])
        })),
    }
}

pub fn amp_rtk_rules(commands: &[String]) -> Vec<String> {
    commands.iter().map(|prefix| format!("rtk {} *", prefix)).collect()
}

pub fn update_amp_rtk_permissions(remove: bool, commands: &[String], command_list_available: bool) -> bool {
    let f = amp_paths_resolved().config;
    let raw = read_file_safe(&f).unwrap_or_default();
    let mut cfg = try_parse_jsonc(&raw).unwrap_or_default();
    if !raw.is_empty() && cfg.is_empty() && serde_json::from_str::<Value>(&raw).is_err() {
        return false;
    }

    const PERMISSIONS_KEY: &str = "amp.permissions";
    let exists = cfg.contains_key(PERMISSIONS_KEY);
    let permissions = match cfg.get(PERMISSIONS_KEY) {
        Some(Value::Array(arr)) => arr.clone(),
        Some(_) => return false,
        None => Vec::new(),
    };

    let want_rules = if !remove && command_list_available {
        amp_rtk_rules(commands)
    } else {
        Vec::new()
    };

    let mut kept = Vec::with_capacity(permissions.len());
    for item in &permissions {
        if let Value::Object(m) = item {
            let tool = m.get("tool").and_then(|v| v.as_str());
            let action = m.get("action").and_then(|v| v.as_str());
            let cmd = m.get("matches").and_then(|mm| mm.as_object()).and_then(|obj| obj.get("cmd")).and_then(|c| c.as_str());
            if tool == Some("Bash") && action == Some("allow") && cmd.map(|s| s.starts_with("rtk ")).unwrap_or(false) {
                continue;
            }
        }
        kept.push(item.clone());
    }

    let mut front = Vec::with_capacity(want_rules.len());
    for cmd in want_rules {
        let mut matches = OrderedMap::new();
        matches.insert("cmd".to_string(), Value::String(cmd));
        let mut rule = OrderedMap::new();
        rule.insert("tool".to_string(), Value::String("Bash".to_string()));
        rule.insert("matches".to_string(), Value::Object(matches));
        rule.insert("action".to_string(), Value::String("allow".to_string()));
        front.push(Value::Object(rule));
    }

    let mut next = front;
    next.extend(kept);

    let changed = !amp_permissions_equal(&permissions, &next);
    if changed || !exists {
        cfg.insert(PERMISSIONS_KEY.to_string(), Value::Array(next));
        if write_file(&f, &stringify_json(&Value::Object(cfg))).is_err() {
            return false;
        }
    }
    changed
}

fn amp_permissions_equal(a: &[Value], b: &[Value]) -> bool {
    let wrap = |list: &[Value]| -> String {
        let mut m = OrderedMap::new();
        m.insert("permissions".to_string(), Value::Array(list.to_vec()));
        stringify_json(&Value::Object(m))
    };
    wrap(a) == wrap(b)
}

pub fn amp_rtk_commands() -> (Vec<String>, bool) {
    match rtk_commands() {
        Ok(cmds) => (cmds, true),
        Err(_) => (Vec::new(), false),
    }
}

pub fn configure_amp_rtk_permissions(commands: &[String], available: bool) -> (bool, String) {
    (update_amp_rtk_permissions(false, commands, available), amp_paths_resolved().config)
}

pub fn configure_amp_rtk_permissions_auto() -> (bool, String) {
    let (commands, ok) = amp_rtk_commands();
    configure_amp_rtk_permissions(&commands, ok)
}

pub fn remove_amp_rtk_permissions() -> bool {
    let (commands, ok) = amp_rtk_commands();
    update_amp_rtk_permissions(true, &commands, ok)
}

pub fn remove_amp_rtk_permissions_with(commands: &[String]) -> bool {
    update_amp_rtk_permissions(true, commands, true)
}

pub fn amp_rtk_permissions_wired() -> bool {
    let (commands, available) = amp_rtk_commands();
    amp_rtk_permissions_wired_with(&commands, available)
}

pub fn amp_rtk_permissions_wired_with(commands: &[String], available: bool) -> bool {
    if !available || commands.is_empty() {
        return false;
    }
    let raw = match read_file_safe(&amp_paths_resolved().config) {
        Some(s) => s,
        None => return false,
    };
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return false,
    };
    let rules = match cfg.get("amp.permissions") {
        Some(Value::Array(a)) => a,
        _ => return false,
    };

    let want = amp_rtk_rules(commands);
    let mut found = HashSet::new();

    for item in rules {
        if let Value::Object(m) = item {
            let tool = m.get("tool").and_then(|v| v.as_str());
            let action = m.get("action").and_then(|v| v.as_str());
            let cmd = m.get("matches").and_then(|mm| mm.as_object()).and_then(|obj| obj.get("cmd")).and_then(|c| c.as_str());
            if tool == Some("Bash") && action == Some("allow") && cmd.map(|s| s.starts_with("rtk ")).unwrap_or(false) {
                let cmd_str = cmd.unwrap();
                if cmd_str == "rtk *" || !want.iter().any(|c| c == cmd_str) {
                    return false;
                }
                found.insert(cmd_str.to_string());
            }
        }
    }

    for cmd in &want {
        if !found.contains(cmd) {
            return false;
        }
    }
    found.len() == commands.len()
}

