// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: tokless
// file: crates/tokless/src/agents/antigravity.rs
// created: 2026-10-11
// lastModified: 2026-10-31

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::AgentManifest;
use crate::util::jsonc::{OrderedMap, Value, has_jsonc_comments, stringify_json};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;

use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    detect_agent_with, json_args_eq, json_equal, json_map_child, json_map_child_mut,
    strings_to_values, tokless_command, tokless_command_ok,
    try_parse_jsonc_map as try_parse_jsonc,
};

pub const ANTIGRAVITY_CONTEXT_MODE_HOOK_COMMAND: &str =
    "context-mode hook antigravity-cli pretooluse";
pub const ANTIGRAVITY_PROJECTMEM_GROUP: &str = "projectmem-context";
pub const ANTIGRAVITY_IDE_ENDPOINT_KEY: &str = "jetski.cloudCodeUrl";
pub const ANTIGRAVITY_PROXY_ENV_KEY: &str = "GOOGLE_GEMINI_BASE_URL";
pub const ANTIGRAVITY_CLOUD_CODE_KEY: &str = "CLOUD_CODE_URL";
pub const ANTIGRAVITY_PROXY_FENCE_HEAD: &str = "# tokless:headroom begin";
pub const ANTIGRAVITY_PROXY_FENCE_FOOT: &str = "# tokless:headroom end";
pub const ANTIGRAVITY_SHELL_FENCE_HEAD: &str = "# >>> tokless antigravity headroom >>>";
pub const ANTIGRAVITY_SHELL_FENCE_FOOT: &str = "# <<< tokless antigravity headroom <<<";
pub const ANTIGRAVITY_WINDOWS_MARKER: &str = "TOKLESS_ANTIGRAVITY_HEADROOM_MANAGED";
pub const ANTIGRAVITY_SHIM_MARKER: &str = "# tokless:antigravity-launcher";

const CODEGRAPH_TOOL_DEF_FILES: &[&str] = &[
    "codegraph_explore.json",
    "codegraph_node.json",
    "codegraph_search.json",
    "codegraph_callers.json",
    "instructions.md",
];

pub fn antigravity_manifest() -> AgentManifest {
    AgentManifest {
        id: "antigravity".to_string(),
        label: "Antigravity".to_string(),
        homepage: "https://antigravity.google".to_string(),
        cli_bin: "agy".to_string(),
        config_dir: Some(Arc::new(|| paths::antigravity_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with(
                "agy",
                &paths::antigravity_paths_resolved().dir,
                &agy_known_bin_dirs(),
                &antigravity_desktop_paths(),
            )
        })),
    }
}

pub fn antigravity_mcp_config_files() -> Vec<String> {
    let p = paths::antigravity_paths_resolved();
    let cli_cfg = Path::new(&paths::home())
        .join(".gemini")
        .join("antigravity-cli")
        .join("mcp_config.json")
        .to_string_lossy()
        .into_owned();
    vec![p.mcp_config_cli, cli_cfg]
}

pub fn antigravity_legacy_mcp_files() -> Vec<String> {
    let p = paths::antigravity_paths_resolved();
    let gemini = Path::new(&paths::home()).join(".gemini");
    let mut files = vec![p.mcp_config, p.settings];
    for variant in ["antigravity-ide"] {
        let d = gemini.join(variant);
        if paths::exists(&d) {
            files.push(d.join("mcp_config.json").to_string_lossy().into_owned());
        }
    }
    files
}

pub fn antigravity_settings_files() -> Vec<String> {
    let gemini = Path::new(&paths::home()).join(".gemini");
    vec![
        gemini
            .join("antigravity-cli")
            .join("settings.json")
            .to_string_lossy()
            .into_owned(),
        gemini
            .join("antigravity-ide")
            .join("settings.json")
            .to_string_lossy()
            .into_owned(),
    ]
}

pub fn antigravity_global_settings_file() -> String {
    Path::new(&paths::home())
        .join(".gemini")
        .join("settings.json")
        .to_string_lossy()
        .into_owned()
}

pub fn antigravity_dead_gui_settings_files() -> Vec<String> {
    let gemini = Path::new(&paths::home()).join(".gemini");
    vec![
        gemini
            .join("antigravity-desktop")
            .join("settings.json")
            .to_string_lossy()
            .into_owned(),
    ]
}

pub fn clean_antigravity_dead_gui_settings() {
    for f in antigravity_dead_gui_settings_files() {
        let Some(raw) = paths::read_file_safe(&f) else {
            continue;
        };
        let Some(mut cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let mut empty_perms = false;
        let mut changed = false;
        if let Some(pm) = json_map_child_mut(&mut cfg, "permissions")
            && let Some(items) = pm.get_mut("allow").and_then(|v| v.as_array_mut())
        {
            let before = items.len();
            items.retain(|item| {
                !matches!(item.as_str(), Some("command(rtk)" | "command(rtk )"))
            });
            changed = items.len() != before;
            empty_perms = items.is_empty();
        }
        if !changed {
            continue;
        }
        if empty_perms {
            cfg.shift_remove("permissions");
        }
        let _ = paths::write_file(&f, &stringify_json(&Value::Object(cfg)));
    }
}

pub fn antigravity_hooks_file() -> String {
    Path::new(&paths::home())
        .join(".gemini")
        .join("config")
        .join("hooks.json")
        .to_string_lossy()
        .into_owned()
}

pub fn has_antigravity_context_mode_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&antigravity_hooks_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(group) = json_map_child(&cfg, "context-mode") else {
        return false;
    };
    let Some(arr) = group.get("PreToolUse").and_then(|v| v.as_array()) else {
        return false;
    };
    let Some(entry) = arr.first().and_then(|v| v.as_object()) else {
        return false;
    };
    if entry.get("matcher").and_then(|v| v.as_str())
        != Some("run_command|view_file|grep_search|web_fetch|read_url_content")
    {
        return false;
    }
    let Some(hooks_arr) = entry.get("hooks").and_then(|v| v.as_array()) else {
        return false;
    };
    let Some(hook) = hooks_arr.first().and_then(|v| v.as_object()) else {
        return false;
    };
    hook.get("command").and_then(|v| v.as_str()) == Some(ANTIGRAVITY_CONTEXT_MODE_HOOK_COMMAND)
}

pub fn install_antigravity_context_mode_hook() {
    let hooks_file = antigravity_hooks_file();
    let raw_opt = paths::read_file_safe(&hooks_file);
    let raw = raw_opt.as_deref().unwrap_or("");
    let mut cfg = if let Some(ref r) = raw_opt {
        if has_jsonc_comments(r) {
            return;
        }
        match try_parse_jsonc(r) {
            Some(c) => c,
            None => {
                if !r.trim().is_empty() {
                    return;
                }
                OrderedMap::new()
            }
        }
    } else {
        OrderedMap::new()
    };

    let mut hook_cfg = OrderedMap::new();
    hook_cfg.insert("type".to_string(), Value::String("command".to_string()));
    hook_cfg.insert(
        "command".to_string(),
        Value::String(ANTIGRAVITY_CONTEXT_MODE_HOOK_COMMAND.to_string()),
    );
    hook_cfg.insert("timeout".to_string(), Value::from(10));

    let mut entry = OrderedMap::new();
    entry.insert(
        "matcher".to_string(),
        Value::String("run_command|view_file|grep_search|web_fetch|read_url_content".to_string()),
    );
    entry.insert(
        "hooks".to_string(),
        Value::Array(vec![Value::Object(hook_cfg)]),
    );

    let mut group = OrderedMap::new();
    group.insert(
        "PreToolUse".to_string(),
        Value::Array(vec![Value::Object(entry)]),
    );

    cfg.insert("context-mode".to_string(), Value::Object(group));

    let next = stringify_json(&Value::Object(cfg));
    if next != raw {
        let _ = paths::write_file(&hooks_file, &next);
    }
}

pub fn remove_antigravity_context_mode_hook() {
    let hooks_file = antigravity_hooks_file();
    let Some(raw) = paths::read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    if cfg.shift_remove("context-mode").is_some() {
        if cfg.is_empty() {
            let _ = fs::remove_file(&hooks_file);
            return;
        }
        let _ = paths::write_file(&hooks_file, &stringify_json(&Value::Object(cfg)));
    }
}

pub fn cleanup_legacy_antigravity_context_mode() {
    let plugins_dir = Path::new(&paths::home())
        .join(".gemini")
        .join("config")
        .join("plugins")
        .join("context-mode");
    let _ = fs::remove_dir_all(plugins_dir);

    let hooks_file = antigravity_hooks_file();
    let Some(raw) = paths::read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    let mut changed = false;
    if cfg.shift_remove("ctx").is_some() {
        changed = true;
    }
    if cfg.shift_remove("tokless-context-mode").is_some() {
        changed = true;
    }
    let group_names: Vec<String> = cfg.keys().cloned().collect();
    for group_name in group_names {
        let mut empty_group = false;
        if let Some(group) = json_map_child_mut(&mut cfg, &group_name) {
            let events: Vec<String> = group.keys().cloned().collect();
            for event in events {
                if let Some(entries) = group.get(&event).and_then(|v| v.as_array()).cloned() {
                    let kept = filter_context_mode_hook_entries(&entries);
                    if kept.len() != entries.len() {
                        changed = true;
                        if kept.is_empty() {
                            group.shift_remove(&event);
                        } else {
                            group.insert(event, Value::Array(kept));
                        }
                    }
                }
            }
            empty_group = group.is_empty();
        }
        if empty_group {
            cfg.shift_remove(&group_name);
            changed = true;
        }
    }
    if !changed {
        return;
    }
    if cfg.is_empty() {
        let _ = fs::remove_file(&hooks_file);
        return;
    }
    let next = stringify_json(&Value::Object(cfg));
    if next != raw {
        let _ = paths::write_file(&hooks_file, &next);
    }
}

fn filter_context_mode_hook_entries(arr: &[Value]) -> Vec<Value> {
    let mut kept = Vec::new();
    for e in arr {
        let Some(em) = e.as_object() else {
            kept.push(e.clone());
            continue;
        };
        let Some(ha) = em.get("hooks").and_then(|v| v.as_array()) else {
            kept.push(e.clone());
            continue;
        };
        let drop_entry = ha.iter().any(|h| {
            h.as_object()
                .and_then(|hm| hm.get("command"))
                .and_then(|c| c.as_str())
                .is_some_and(|s| s.contains("context-mode"))
        });
        if !drop_entry {
            kept.push(e.clone());
        }
    }
    kept
}

fn antigravity_rewrite_script() -> String {
    Path::new(&paths::home())
        .join(".gemini")
        .join("config")
        .join("tokless")
        .join("rtk-rewrite.sh")
        .to_string_lossy()
        .into_owned()
}

fn antigravity_legacy_rewrite_script() -> String {
    Path::new(&paths::home())
        .join(".gemini")
        .join("config")
        .join("tokless-rtk-rewrite.sh")
        .to_string_lossy()
        .into_owned()
}

pub fn install_antigravity_rtk_hook() -> bool {
    let (command, ok) = tokless_command_ok(&["rtk-hook", "agy"]);
    if !ok {
        return false;
    }
    let hooks_file = antigravity_hooks_file();
    let raw_opt = paths::read_file_safe(&hooks_file);
    let raw = raw_opt.as_deref().unwrap_or("");
    let mut cfg = if let Some(ref r) = raw_opt {
        if has_jsonc_comments(r) {
            return false;
        }
        match try_parse_jsonc(r) {
            Some(c) => c,
            None => {
                if !r.trim().is_empty() {
                    return false;
                }
                OrderedMap::new()
            }
        }
    } else {
        OrderedMap::new()
    };

    if let Some(hooks_val) = cfg.get("hooks") {
        let Some(hooks_map) = hooks_val.as_object() else {
            return false;
        };
        if let Some(pre_val) = hooks_map.get("PreToolUse") {
            let Some(pre_arr) = pre_val.as_array() else {
                return false;
            };
            for item in pre_arr {
                let Some(entry) = item.as_object() else {
                    return false;
                };
                if let Some(nested) = entry.get("hooks")
                    && !nested.is_array()
                {
                    return false;
                }
            }
        }
    }

    let mut hook_cfg = OrderedMap::new();
    hook_cfg.insert("type".to_string(), Value::String("command".to_string()));
    hook_cfg.insert("command".to_string(), Value::String(command));

    let mut pre_tool_use_entry = OrderedMap::new();
    pre_tool_use_entry.insert("matcher".to_string(), Value::String("run_command".to_string()));
    pre_tool_use_entry.insert(
        "hooks".to_string(),
        Value::Array(vec![Value::Object(hook_cfg)]),
    );

    let mut rtk_group = OrderedMap::new();
    rtk_group.insert(
        "PreToolUse".to_string(),
        Value::Array(vec![Value::Object(pre_tool_use_entry)]),
    );

    cfg.insert("rtk".to_string(), Value::Object(rtk_group));

    let next = stringify_json(&Value::Object(cfg));
    if next != raw && paths::write_file(&hooks_file, &next).is_err() {
        return false;
    }
    let _ = fs::remove_file(antigravity_rewrite_script());
    let _ = fs::remove_file(antigravity_legacy_rewrite_script());
    clean_antigravity_dead_gui_settings();
    remove_antigravity_codegraph_tool_defs();
    clean_antigravity_dead_hook_groups();
    allow_antigravity_entry("command(rtk)");
    allow_antigravity_entry("command(rtk )");
    has_antigravity_rtk_hook()
}

pub fn remove_antigravity_rtk_hook() {
    let _ = fs::remove_file(antigravity_rewrite_script());
    let _ = fs::remove_file(antigravity_legacy_rewrite_script());
    let hooks_file = antigravity_hooks_file();
    let Some(raw) = paths::read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    if let Some(group) = cfg.get("rtk")
        && antigravity_rtk_group_managed(group)
    {
        cfg.shift_remove("rtk");
        let _ = paths::write_file(&hooks_file, &stringify_json(&Value::Object(cfg)));
    }
}

fn antigravity_rtk_group_managed(value: &Value) -> bool {
    let Some(group) = value.as_object() else {
        return false;
    };
    let Some(entries) = group.get("PreToolUse").and_then(|v| v.as_array()) else {
        return false;
    };
    let want = tokless_command(&["rtk-hook", "agy"]);
    for item in entries {
        let Some(entry) = item.as_object() else {
            continue;
        };
        let Some(arr) = entry.get("hooks").and_then(|v| v.as_array()) else {
            continue;
        };
        for hook in arr {
            if let Some(hm) = hook.as_object()
                && hm.get("command").and_then(|v| v.as_str()) == Some(want.as_str())
            {
                return true;
            }
        }
    }
    false
}

pub fn has_antigravity_rtk_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&antigravity_hooks_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(rtk_group) = json_map_child(&cfg, "rtk") else {
        return false;
    };
    let Some(pre_arr) = rtk_group.get("PreToolUse").and_then(|v| v.as_array()) else {
        return false;
    };
    let want = tokless_command(&["rtk-hook", "agy"]);
    for value in pre_arr {
        let Some(entry) = value.as_object() else {
            continue;
        };
        if entry.get("matcher").and_then(|v| v.as_str()) != Some("run_command") {
            continue;
        }
        let Some(hooks_arr) = entry.get("hooks").and_then(|v| v.as_array()) else {
            continue;
        };
        for hook_val in hooks_arr {
            let Some(hook) = hook_val.as_object() else {
                continue;
            };
            if hook.get("type").and_then(|v| v.as_str()) != Some("command") {
                continue;
            }
            if hook.get("command").and_then(|v| v.as_str()) == Some(want.as_str()) {
                return true;
            }
        }
    }
    false
}

pub fn remove_antigravity_codegraph_tool_defs() {
    let gemini = Path::new(&paths::home()).join(".gemini");
    for variant in ["antigravity-cli", "antigravity-ide"] {
        let tool_dir = gemini.join(variant).join("mcp").join("codegraph");
        for file in CODEGRAPH_TOOL_DEF_FILES {
            let _ = fs::remove_file(tool_dir.join(file));
        }
        let _ = fs::remove_dir(&tool_dir);
    }
}

pub fn has_antigravity_codegraph_index_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&antigravity_hooks_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(group) = json_map_child(&cfg, "tokless-codegraph-index") else {
        return false;
    };
    for ev in ["PostToolUse", "PreInvocation"] {
        if has_codegraph_index_entry(group, ev) {
            return true;
        }
    }
    false
}

fn has_codegraph_index_entry(group: &OrderedMap, event: &str) -> bool {
    let Some(arr) = group.get(event).and_then(|v| v.as_array()) else {
        return false;
    };
    let Some(entry) = arr.first().and_then(|v| v.as_object()) else {
        return false;
    };
    if let Some(cmd_str) = entry.get("command").and_then(|v| v.as_str()) {
        return cmd_str.contains("agy-hook codegraph-index");
    }
    let Some(hooks_arr) = entry.get("hooks").and_then(|v| v.as_array()) else {
        return false;
    };
    let Some(hook) = hooks_arr.first().and_then(|v| v.as_object()) else {
        return false;
    };
    hook.get("command")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s.contains("agy-hook codegraph-index"))
}

pub fn remove_antigravity_codegraph_index_hook() {
    let hooks_file = antigravity_hooks_file();
    let Some(raw) = paths::read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    if cfg.shift_remove("tokless-codegraph-index").is_some() {
        let _ = paths::write_file(&hooks_file, &stringify_json(&Value::Object(cfg)));
    }
}

pub fn install_antigravity_codegraph_index_hook() {
    let (command, ok) = tokless_command_ok(&["agy-hook", "codegraph-index"]);
    if !ok {
        return;
    }
    let hooks_file = antigravity_hooks_file();
    let raw_opt = paths::read_file_safe(&hooks_file);
    let raw = raw_opt.as_deref().unwrap_or("");
    let mut cfg = if let Some(ref r) = raw_opt {
        if has_jsonc_comments(r) {
            return;
        }
        match try_parse_jsonc(r) {
            Some(c) => c,
            None => {
                if !r.trim().is_empty() {
                    return;
                }
                OrderedMap::new()
            }
        }
    } else {
        OrderedMap::new()
    };

    let mut hook_cfg = OrderedMap::new();
    hook_cfg.insert("type".to_string(), Value::String("command".to_string()));
    hook_cfg.insert("command".to_string(), Value::String(command.clone()));
    hook_cfg.insert("timeout".to_string(), Value::from(120));

    let mut post_tool_entry = OrderedMap::new();
    post_tool_entry.insert("matcher".to_string(), Value::String(String::new()));
    post_tool_entry.insert(
        "hooks".to_string(),
        Value::Array(vec![Value::Object(hook_cfg)]),
    );

    let mut pre_inv_hook = OrderedMap::new();
    pre_inv_hook.insert("type".to_string(), Value::String("command".to_string()));
    pre_inv_hook.insert("command".to_string(), Value::String(command));
    pre_inv_hook.insert("timeout".to_string(), Value::from(120));

    let mut group = OrderedMap::new();
    group.insert(
        "PostToolUse".to_string(),
        Value::Array(vec![Value::Object(post_tool_entry)]),
    );
    group.insert(
        "PreInvocation".to_string(),
        Value::Array(vec![Value::Object(pre_inv_hook)]),
    );

    cfg.insert("tokless-codegraph-index".to_string(), Value::Object(group));

    let next = stringify_json(&Value::Object(cfg));
    if next != raw {
        let _ = paths::write_file(&hooks_file, &next);
    }
}

pub fn set_antigravity_compact_tool_output(enabled: bool) {
    for f in antigravity_settings_files() {
        if let Some(parent) = Path::new(&f).parent() {
            let _ = paths::ensure_dir(parent);
        }
        let raw_opt = paths::read_file_safe(&f);
        let exists = raw_opt.is_some();
        let raw = raw_opt.as_deref().unwrap_or("");
        if has_jsonc_comments(raw) {
            continue;
        }
        let mut cfg = match try_parse_jsonc(raw) {
            Some(c) => c,
            None => {
                if exists && !raw.trim().is_empty() {
                    continue;
                }
                OrderedMap::new()
            }
        };
        if cfg.get("ui").is_some_and(|v| !v.is_object()) {
            continue;
        }
        if !cfg.contains_key("ui") {
            cfg.insert("ui".to_string(), Value::Object(OrderedMap::new()));
        }
        if let Some(ui) = json_map_child_mut(&mut cfg, "ui") {
            ui.insert("compactToolOutput".to_string(), Value::Bool(enabled));
        }
        let next = stringify_json(&Value::Object(cfg));
        if next != raw {
            let _ = paths::write_file(&f, &next);
        }
    }
}

pub fn allow_antigravity_entry(entry: &str) {
    let mut files = antigravity_settings_files();
    if entry.starts_with("command(") {
        files.push(antigravity_global_settings_file());
    }
    for f in files {
        if let Some(parent) = Path::new(&f).parent() {
            let _ = paths::ensure_dir(parent);
        }
        let raw_opt = paths::read_file_safe(&f);
        let exists = raw_opt.is_some();
        let raw = raw_opt.as_deref().unwrap_or("");
        if has_jsonc_comments(raw) {
            continue;
        }
        let mut cfg = match try_parse_jsonc(raw) {
            Some(c) => c,
            None => {
                if exists && !raw.trim().is_empty() {
                    continue;
                }
                OrderedMap::new()
            }
        };
        if cfg.get("permissions").is_some_and(|v| !v.is_object()) {
            continue;
        }
        if !cfg.contains_key("permissions") {
            cfg.insert("permissions".to_string(), Value::Object(OrderedMap::new()));
        }
        let Some(perms) = json_map_child_mut(&mut cfg, "permissions") else {
            continue;
        };
        let mut allow = if let Some(v) = perms.get("allow") {
            let Some(arr) = v.as_array() else {
                continue;
            };
            arr.clone()
        } else {
            Vec::new()
        };
        let has = allow.iter().any(|e| e.as_str() == Some(entry));
        if !has {
            allow.push(Value::String(entry.to_string()));
        }
        perms.insert("allow".to_string(), Value::Array(allow));
        let next = stringify_json(&Value::Object(cfg));
        if next != raw {
            let _ = paths::write_file(&f, &next);
        }
    }
}

pub fn remove_antigravity_entry(entry: &str) {
    let want = entry;
    let mut files = antigravity_settings_files();
    if entry.starts_with("command(") {
        files.push(antigravity_global_settings_file());
    }
    let is_managed = antigravity_permission_managed(entry);
    for f in files {
        let Some(raw) = paths::read_file_safe(&f) else {
            continue;
        };
        let Some(mut cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let mut dropped = false;
        if let Some(pm) = json_map_child_mut(&mut cfg, "permissions")
            && let Some(arr) = pm.get("allow").and_then(|v| v.as_array()).cloned()
        {
            let mut out = Vec::with_capacity(arr.len());
            for e in arr {
                if e.as_str() == Some(want) && is_managed {
                    dropped = true;
                    continue;
                }
                out.push(e);
            }
            if dropped {
                pm.insert("allow".to_string(), Value::Array(out));
            }
        }
        if dropped {
            let _ = paths::write_file(&f, &stringify_json(&Value::Object(cfg)));
        }
    }
}

fn antigravity_permission_managed(entry: &str) -> bool {
    if entry == "command(rtk)" || entry == "command(rtk )" {
        return has_antigravity_rtk_hook();
    }
    if let Some(rest) = entry.strip_prefix("mcp(") {
        let tool_id = rest.strip_suffix("/*)").unwrap_or(rest);
        let mut all_files = antigravity_mcp_config_files();
        all_files.extend(antigravity_legacy_mcp_files());
        for file in all_files {
            let Some(raw) = paths::read_file_safe(&file) else {
                continue;
            };
            let Some(cfg) = try_parse_jsonc(&raw) else {
                continue;
            };
            if let Some(sm) = json_map_child(&cfg, "mcpServers")
                && let Some(val) = sm.get(tool_id)
                && antigravity_mcp_managed(tool_id, val)
            {
                return true;
            }
        }
    }
    false
}

pub fn configure_antigravity_mcp(tool_id: &str) -> (bool, String) {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("antigravity", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut all_configured = true;
    let mut changed = false;
    let mut file = String::new();

    for f in antigravity_mcp_config_files() {
        let raw_opt = paths::read_file_safe(&f);
        let exists = raw_opt.is_some();
        let raw = raw_opt.as_deref().unwrap_or("");
        let mut cfg = match try_parse_jsonc(raw) {
            Some(c) => c,
            None => {
                if exists && !raw.trim().is_empty() {
                    L.err(&format!(
                        "couldn't parse Antigravity MCP config {f}; leaving it unchanged"
                    ));
                    all_configured = false;
                    continue;
                }
                OrderedMap::new()
            }
        };
        if cfg.get("mcpServers").is_some_and(|v| !v.is_object()) {
            all_configured = false;
            continue;
        }
        if !cfg.contains_key("mcpServers") {
            cfg.insert("mcpServers".to_string(), Value::Object(OrderedMap::new()));
        }
        let mut entry = OrderedMap::new();
        entry.insert("command".to_string(), Value::String(spawn.command.clone()));
        if !spawn.args.is_empty() {
            entry.insert(
                "args".to_string(),
                Value::Array(strings_to_values(&spawn.args)),
            );
        }
        entry.insert("trust".to_string(), Value::Bool(true));
        let entry_val = Value::Object(entry);

        let servers = json_map_child_mut(&mut cfg, "mcpServers").unwrap();
        if let Some(existing) = servers.get(tool_id) {
            if !json_equal(existing, &entry_val) {
                all_configured = false;
            }
            continue;
        }
        servers.insert(tool_id.to_string(), entry_val);
        let next = stringify_json(&Value::Object(cfg));
        if next != raw {
            if let Err(err) = paths::write_file(&f, &next) {
                L.err(&format!("couldn't write Antigravity MCP config {f}: {err}"));
                all_configured = false;
                continue;
            }
            changed = true;
            file = f;
        }
    }
    if all_configured {
        if tool_id == "codegraph" {
            remove_antigravity_codegraph_tool_defs();
        }
        remove_antigravity_mcp_from_files(tool_id, &antigravity_legacy_mcp_files());
        allow_antigravity_entry(&format!("mcp({tool_id}/*)"));
    }
    (changed, file)
}

pub fn antigravity_mcp_bounded(tool_id: &str) -> bool {
    let files = antigravity_mcp_config_files();
    if files.is_empty() {
        return false;
    }
    for f in files {
        let Some(raw) = paths::read_file_safe(&f) else {
            return false;
        };
        let Some(cfg) = try_parse_jsonc(&raw) else {
            return false;
        };
        let Some(sm) = json_map_child(&cfg, "mcpServers") else {
            return false;
        };
        let Some(existing) = sm.get(tool_id) else {
            return false;
        };
        if !antigravity_mcp_managed(tool_id, existing) {
            return false;
        }
    }
    true
}

pub fn antigravity_projectmem_mcp_bounded() -> bool {
    antigravity_mcp_bounded("projectmem")
}

pub fn antigravity_mcp_has(tool_id: &str) -> bool {
    for f in antigravity_mcp_config_files() {
        let Some(raw) = paths::read_file_safe(&f) else {
            continue;
        };
        let Some(cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        if let Some(sm) = json_map_child(&cfg, "mcpServers")
            && sm.contains_key(tool_id)
        {
            return true;
        }
    }
    false
}

pub fn agy_known_bin_dirs() -> Vec<String> {
    if paths::is_win() {
        if let Ok(local) = std::env::var("LOCALAPPDATA")
            && !local.is_empty()
        {
            return vec![
                Path::new(&local)
                    .join("agy")
                    .join("bin")
                    .to_string_lossy()
                    .into_owned(),
            ];
        }
        return Vec::new();
    }
    vec![
        Path::new(&paths::home())
            .join(".local")
            .join("bin")
            .to_string_lossy()
            .into_owned(),
    ]
}

fn antigravity_desktop_paths() -> Vec<String> {
    if paths::is_win() {
        if let Ok(local) = std::env::var("LOCALAPPDATA")
            && !local.is_empty()
        {
            return vec![
                Path::new(&local)
                    .join("Programs")
                    .join("Antigravity")
                    .join("Antigravity.exe")
                    .to_string_lossy()
                    .into_owned(),
                Path::new(&local)
                    .join("Programs")
                    .join("Antigravity IDE")
                    .join("Antigravity IDE.exe")
                    .to_string_lossy()
                    .into_owned(),
            ];
        }
        return Vec::new();
    }
    if cfg!(target_os = "macos") {
        return vec![
            "/Applications/Antigravity.app".to_string(),
            "/Applications/Antigravity IDE.app".to_string(),
        ];
    }
    vec![
        "/opt/antigravity".to_string(),
        "/opt/antigravity-ide".to_string(),
        "/usr/local/bin/antigravity".to_string(),
        "/usr/local/bin/antigravity-ide".to_string(),
    ]
}

fn clean_antigravity_dead_hook_groups() {
    let hooks_file = antigravity_hooks_file();
    let Some(raw) = paths::read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    let mut changed = false;
    let keys: Vec<String> = cfg.keys().cloned().collect();
    for name in keys {
        let should_delete = cfg
            .get(&name)
            .and_then(|v| v.as_object())
            .and_then(|gm| gm.get("hook_event_name"))
            .and_then(|v| v.as_str())
            == Some("PreToolUse");
        if should_delete {
            cfg.shift_remove(&name);
            changed = true;
        }
    }
    if changed {
        let _ = paths::write_file(&hooks_file, &stringify_json(&Value::Object(cfg)));
    }
}

pub fn cleanup_dead_ide_hooks() {
    let gemini = Path::new(&paths::home()).join(".gemini");
    let candidates = [
        gemini.join("settings.json"),
        gemini.join("antigravity").join("settings.json"),
        gemini.join("antigravity-ide").join("settings.json"),
        gemini.join("antigravity-cli").join("settings.json"),
    ];
    for p in candidates {
        let Some(raw) = paths::read_file_safe(&p) else {
            continue;
        };
        let Some(mut cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let mut changed = false;
        let mut hooks_empty = false;
        if let Some(hooks) = json_map_child_mut(&mut cfg, "hooks") {
            let events: Vec<String> = hooks.keys().cloned().collect();
            for event in events {
                if let Some(entries) = hooks.get(&event).and_then(|v| v.as_array()).cloned() {
                    let kept = filter_codegraph_index_entries(&entries);
                    if kept.len() != entries.len() {
                        changed = true;
                        if kept.is_empty() {
                            hooks.shift_remove(&event);
                        } else {
                            hooks.insert(event, Value::Array(kept));
                        }
                    }
                }
            }
            hooks_empty = hooks.is_empty();
        }
        if !changed {
            continue;
        }
        if hooks_empty {
            cfg.shift_remove("hooks");
        }
        let next = stringify_json(&Value::Object(cfg));
        if next != raw {
            let _ = paths::write_file(&p, &next);
        }
    }
}

fn filter_codegraph_index_entries(arr: &[Value]) -> Vec<Value> {
    let mut kept = Vec::new();
    for e in arr {
        let Some(em) = e.as_object() else {
            kept.push(e.clone());
            continue;
        };
        let Some(ha) = em.get("hooks").and_then(|v| v.as_array()) else {
            kept.push(e.clone());
            continue;
        };
        let drop_entry = ha.iter().any(|h| {
            h.as_object()
                .and_then(|hm| hm.get("command"))
                .and_then(|c| c.as_str())
                .is_some_and(|s| {
                    s.contains("agy-hook codegraph-index")
                        || s.contains("context-mode hook gemini-cli")
                })
        });
        if !drop_entry {
            kept.push(e.clone());
        }
    }
    kept
}

pub fn remove_antigravity_mcp(tool_id: &str) {
    let mut files = antigravity_mcp_config_files();
    files.extend(antigravity_legacy_mcp_files());
    remove_antigravity_entry(&format!("mcp({tool_id}/*)"));
    remove_antigravity_mcp_from_files(tool_id, &files);
}

fn remove_antigravity_mcp_from_files(tool_id: &str, files: &[String]) {
    let entry = format!("mcp({tool_id}/*)");
    for f in files {
        let Some(raw) = paths::read_file_safe(f) else {
            continue;
        };
        if has_jsonc_comments(&raw) {
            continue;
        }
        let Some(mut cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let mut wrote_mcp = false;
        if let Some(sm) = json_map_child_mut(&mut cfg, "mcpServers")
            && let Some(existing) = sm.get(tool_id)
            && antigravity_mcp_managed(tool_id, existing)
        {
            sm.shift_remove(tool_id);
            wrote_mcp = true;
        }
        if wrote_mcp {
            let _ = paths::write_file(f, &stringify_json(&Value::Object(cfg.clone())));
        }
        let mut perms_empty = false;
        let mut changed_perms = false;
        if let Some(pm) = json_map_child_mut(&mut cfg, "permissions")
            && let Some(arr) = pm.get("allow").and_then(|v| v.as_array()).cloned()
        {
            let out: Vec<Value> = arr
                .into_iter()
                .filter(|e| e.as_str() != Some(entry.as_str()))
                .collect();
            if out.is_empty() {
                pm.shift_remove("allow");
            } else {
                pm.insert("allow".to_string(), Value::Array(out));
            }
            perms_empty = pm.is_empty();
            changed_perms = true;
        }
        if changed_perms {
            if perms_empty {
                cfg.shift_remove("permissions");
            }
            let _ = paths::write_file(f, &stringify_json(&Value::Object(cfg)));
        }
    }
}

fn antigravity_mcp_managed(tool_id: &str, existing: &Value) -> bool {
    let Some(em) = existing.as_object() else {
        return false;
    };
    let command = em.get("command").and_then(|v| v.as_str());
    let args = em.get("args");
    let trust = em.get("trust").and_then(|v| v.as_bool());
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("antigravity", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    command == Some(spawn.command.as_str())
        && json_args_eq(args, &spawn.args)
        && trust == Some(true)
}

pub fn install_antigravity_projectmem_hook() -> bool {
    let hooks_file = antigravity_hooks_file();
    let raw_opt = paths::read_file_safe(&hooks_file);
    let exists = raw_opt.is_some();
    let raw = raw_opt.as_deref().unwrap_or("");
    if exists && has_jsonc_comments(raw) {
        return false;
    }
    let mut cfg = match try_parse_jsonc(raw) {
        Some(c) => c,
        None => {
            if exists && !raw.trim().is_empty() {
                return false;
            }
            OrderedMap::new()
        }
    };
    let (command, _) = tokless_command_ok(&["projectmem-hook", "agy"]);
    if command.is_empty() {
        return false;
    }
    let mut hook = OrderedMap::new();
    hook.insert("type".to_string(), Value::String("command".to_string()));
    hook.insert("command".to_string(), Value::String(command));
    hook.insert("timeout".to_string(), Value::from(15));
    let mut group = OrderedMap::new();
    group.insert(
        "SessionStart".to_string(),
        Value::Array(vec![Value::Object(hook)]),
    );
    cfg.shift_remove("tokless-projectmem");
    cfg.insert(
        ANTIGRAVITY_PROJECTMEM_GROUP.to_string(),
        Value::Object(group),
    );
    if let Some(parent) = Path::new(&hooks_file).parent()
        && paths::ensure_dir(parent).is_err()
    {
        return false;
    }
    let next = stringify_json(&Value::Object(cfg));
    if next != raw && paths::write_file(&hooks_file, &next).is_err() {
        return false;
    }
    has_antigravity_projectmem_hook()
}

pub fn has_antigravity_projectmem_hook() -> bool {
    let Some(raw) = paths::read_file_safe(&antigravity_hooks_file()) else {
        return false;
    };
    raw.contains(&format!("\"{ANTIGRAVITY_PROJECTMEM_GROUP}\""))
        && raw.contains("projectmem-hook agy")
}

pub fn remove_antigravity_projectmem_hook() {
    let hooks_file = antigravity_hooks_file();
    let Some(raw) = paths::read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    if cfg.shift_remove(ANTIGRAVITY_PROJECTMEM_GROUP).is_some() {
        let _ = paths::write_file(&hooks_file, &stringify_json(&Value::Object(cfg.clone())));
    }
    if cfg.shift_remove("tokless-projectmem").is_some() {
        let _ = paths::write_file(&hooks_file, &stringify_json(&Value::Object(cfg)));
    }
}

// --- antigravity_ide_proxy.go ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AntigravityIDEProxyState {
    pub path: String,
    #[serde(rename = "hadValue")]
    pub had_value: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

pub fn antigravity_ide_settings_candidates() -> Vec<String> {
    let home = paths::home();
    let base = if paths::is_win() {
        std::env::var("APPDATA")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                Path::new(&home)
                    .join("AppData")
                    .join("Roaming")
                    .to_string_lossy()
                    .into_owned()
            })
    } else if cfg!(target_os = "macos") {
        Path::new(&home)
            .join("Library")
            .join("Application Support")
            .to_string_lossy()
            .into_owned()
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| Path::new(&home).join(".config").to_string_lossy().into_owned())
    };
    vec![
        Path::new(&base)
            .join("Antigravity IDE")
            .join("User")
            .join("settings.json")
            .to_string_lossy()
            .into_owned(),
        Path::new(&base)
            .join("Antigravity")
            .join("User")
            .join("settings.json")
            .to_string_lossy()
            .into_owned(),
    ]
}

pub fn antigravity_ide_settings_file() -> Option<String> {
    let candidates = antigravity_ide_settings_candidates();
    for path in &candidates {
        if paths::exists(path) {
            return Some(path.clone());
        }
    }
    for path in &candidates {
        if let Some(dir) = Path::new(path).parent()
            && paths::exists(dir)
        {
            return Some(path.clone());
        }
    }
    None
}

fn antigravity_ide_state_file() -> String {
    Path::new(&paths::tokless_data_dir())
        .join("antigravity-ide-proxy.json")
        .to_string_lossy()
        .into_owned()
}

fn antigravity_ide_config() -> (String, Option<OrderedMap>, String, bool) {
    let Some(path) = antigravity_ide_settings_file() else {
        return (String::new(), None, String::new(), false);
    };
    let Some(raw) = paths::read_file_safe(&path) else {
        if paths::exists(&path) {
            return (path, None, String::new(), false);
        }
        return (path, Some(OrderedMap::new()), String::new(), true);
    };
    if has_jsonc_comments(&raw) {
        return (path, None, raw, false);
    }
    let cfg = try_parse_jsonc(&raw);
    let ok = cfg.is_some();
    (path, cfg, raw, ok)
}

pub(crate) fn antigravity_ide_compatible() -> bool {
    let (_, cfg, _, ok) = antigravity_ide_config();
    let Some(cfg) = cfg.filter(|_| ok) else {
        return !antigravity_ide_applicable();
    };
    let Some(value) = cfg.get(ANTIGRAVITY_IDE_ENDPOINT_KEY) else {
        return true;
    };
    value.as_str().is_some_and(|s| s.is_empty() || s == antigravity_url())
}

pub(crate) fn antigravity_ide_applicable() -> bool {
    antigravity_ide_settings_file().is_some()
}

fn configure_antigravity_ide() -> Result<bool, String> {
    let (path, cfg, _, ok) = antigravity_ide_config();
    let Some(mut cfg) = cfg.filter(|_| ok) else {
        if !antigravity_ide_applicable() {
            return Ok(false);
        }
        return Err("invalid IDE config".to_string());
    };
    let old = cfg.get(ANTIGRAVITY_IDE_ENDPOINT_KEY).cloned();
    let had = old.is_some();
    if let Some(ref v) = old
        && v.as_str() != Some("")
        && v.as_str() != Some(antigravity_url().as_str())
    {
        return Err("IDE endpoint already exists".to_string());
    }
    if antigravity_ide_proxy_wired() {
        return Ok(false);
    }
    let old_serde = old.and_then(|v| serde_json::to_value(v).ok());
    let state = serde_json::to_string(&AntigravityIDEProxyState {
        path: path.clone(),
        had_value: had,
        value: old_serde,
    })
    .map_err(|e| e.to_string())?;
    paths::write_file(&antigravity_ide_state_file(), &state).map_err(|e| e.to_string())?;
    cfg.insert(
        ANTIGRAVITY_IDE_ENDPOINT_KEY.to_string(),
        Value::String(antigravity_url()),
    );
    if let Err(err) = paths::write_file(&path, &stringify_json(&Value::Object(cfg))) {
        let _ = fs::remove_file(antigravity_ide_state_file());
        return Err(err.to_string());
    }
    Ok(true)
}

fn read_antigravity_ide_state() -> Option<AntigravityIDEProxyState> {
    let raw = paths::read_file_safe(&antigravity_ide_state_file())?;
    let state: AntigravityIDEProxyState = serde_json::from_str(&raw).ok()?;
    for candidate in antigravity_ide_settings_candidates() {
        if Path::new(&state.path) == Path::new(&candidate) {
            return Some(state);
        }
    }
    None
}

fn remove_antigravity_ide() -> bool {
    let Some(state) = read_antigravity_ide_state() else {
        return false;
    };
    let Some(raw) = paths::read_file_safe(&state.path) else {
        return paths::remove_file_retry(antigravity_ide_state_file()).is_ok();
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let current = cfg.get(ANTIGRAVITY_IDE_ENDPOINT_KEY).and_then(|v| v.as_str());
    if current != Some(antigravity_url().as_str()) {
        return false;
    }
    if state.had_value {
        if let Some(val) = state.value.and_then(|v| serde_json::from_value::<Value>(v).ok()) {
            cfg.insert(ANTIGRAVITY_IDE_ENDPOINT_KEY.to_string(), val);
        } else {
            cfg.shift_remove(ANTIGRAVITY_IDE_ENDPOINT_KEY);
        }
    } else {
        cfg.shift_remove(ANTIGRAVITY_IDE_ENDPOINT_KEY);
    }
    if cfg.is_empty() {
        if let Err(err) = paths::remove_file_retry(&state.path)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            return false;
        }
    } else if paths::write_file(&state.path, &stringify_json(&Value::Object(cfg))).is_err() {
        return false;
    }
    paths::remove_file_retry(antigravity_ide_state_file()).is_ok()
}

pub fn antigravity_ide_proxy_wired() -> bool {
    let Some(state) = read_antigravity_ide_state() else {
        return false;
    };
    let Some(raw) = paths::read_file_safe(&state.path) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    cfg.get(ANTIGRAVITY_IDE_ENDPOINT_KEY)
        .and_then(|v| v.as_str())
        == Some(antigravity_url().as_str())
}

// --- antigravity_proxy.go ---

pub(crate) fn antigravity_env_file() -> String {
    Path::new(&paths::home())
        .join(".gemini")
        .join(".env")
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn antigravity_url() -> String {
    proxy_endpoint_for("antigravity")
}

pub(crate) fn antigravity_env_value(raw: &str, key: &str) -> String {
    let mut in_fence = false;
    let mut fenced = String::new();
    let mut unfenced = String::new();
    for line in raw.split('\n') {
        let trimmed = line.trim();
        if trimmed == ANTIGRAVITY_PROXY_FENCE_HEAD {
            in_fence = true;
            continue;
        }
        if trimmed == ANTIGRAVITY_PROXY_FENCE_FOOT {
            in_fence = false;
            continue;
        }
        let Some((name, value)) = trimmed.split_once('=') else {
            continue;
        };
        if name.trim() != key {
            continue;
        }
        let v = value.trim().to_string();
        if in_fence {
            fenced = v;
        } else if unfenced.is_empty() {
            unfenced = v;
        }
    }
    if !fenced.is_empty() {
        fenced
    } else {
        unfenced
    }
}

fn antigravity_dot_env_block() -> String {
    let u = antigravity_url();
    format!(
        "{ANTIGRAVITY_PROXY_FENCE_HEAD}\n{ANTIGRAVITY_PROXY_ENV_KEY}={u}\n{ANTIGRAVITY_CLOUD_CODE_KEY}={u}\n{ANTIGRAVITY_PROXY_FENCE_FOOT}\n"
    )
}

fn antigravity_can_replace(raw: &str, key: &str, want: &str) -> bool {
    let v = antigravity_env_value(raw, key);
    v.is_empty() || v == want
}

fn antigravity_legacy_shell_cleanup_paths() -> Vec<String> {
    let h = Path::new(&paths::home()).to_path_buf();
    vec![
        h.join(".zshenv").to_string_lossy().into_owned(),
        h.join(".zprofile").to_string_lossy().into_owned(),
        h.join(".zshrc").to_string_lossy().into_owned(),
        h.join(".bash_profile").to_string_lossy().into_owned(),
        h.join(".bashrc").to_string_lossy().into_owned(),
        h.join(".profile").to_string_lossy().into_owned(),
        h.join(".config")
            .join("environment.d")
            .join("tokless-antigravity.conf")
            .to_string_lossy()
            .into_owned(),
    ]
}

fn antigravity_remove_shell_exports() -> bool {
    if paths::is_win() {
        return false;
    }
    let mut removed = false;
    for file in antigravity_legacy_shell_cleanup_paths() {
        let Some(raw) = paths::read_file_safe(&file) else {
            continue;
        };
        if Path::new(&file).file_name().and_then(|s| s.to_str())
            == Some("tokless-antigravity.conf")
        {
            let u = antigravity_url();
            let want = format!(
                "{ANTIGRAVITY_PROXY_ENV_KEY}={u}\n{ANTIGRAVITY_CLOUD_CODE_KEY}={u}\n"
            );
            if raw == want && fs::remove_file(&file).is_ok() {
                removed = true;
            }
            continue;
        }
        if !raw.contains(ANTIGRAVITY_SHELL_FENCE_HEAD) {
            continue;
        }
        let next = antigravity_remove_shell_block(&raw);
        if next == raw {
            continue;
        }
        let next = next.strip_suffix('\n').unwrap_or(&next);
        if next.trim().is_empty() {
            if paths::write_file(&file, "").is_ok() {
                removed = true;
            }
        } else if paths::write_file(&file, &format!("{next}\n")).is_ok() {
            removed = true;
        }
    }
    removed
}

fn antigravity_remove_shell_block(raw: &str) -> String {
    let u = antigravity_url();
    let exp1 = format!("export {ANTIGRAVITY_PROXY_ENV_KEY}={u}");
    let exp2 = format!("export {ANTIGRAVITY_CLOUD_CODE_KEY}={u}");
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in raw.split('\n') {
        let trimmed = line.trim();
        if trimmed == ANTIGRAVITY_SHELL_FENCE_HEAD {
            in_fence = true;
            continue;
        }
        if trimmed == ANTIGRAVITY_SHELL_FENCE_FOOT {
            in_fence = false;
            continue;
        }
        if in_fence && (trimmed == exp1 || trimmed == exp2) {
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

fn antigravity_strip_fence(raw: &str) -> String {
    antigravity_strip_marked_block(raw, ANTIGRAVITY_PROXY_FENCE_HEAD, ANTIGRAVITY_PROXY_FENCE_FOOT)
}

fn antigravity_strip_marked_block(raw: &str, head: &str, foot: &str) -> String {
    let lines: Vec<&str> = raw.split('\n').collect();
    let mut in_fence = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed == head {
            if in_fence {
                return raw.to_string();
            }
            in_fence = true;
        } else if trimmed == foot {
            if !in_fence {
                return raw.to_string();
            }
            in_fence = false;
        }
    }
    if in_fence {
        return raw.to_string();
    }
    let mut out = Vec::new();
    in_fence = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == head {
            in_fence = true;
            continue;
        }
        if trimmed == foot {
            in_fence = false;
            continue;
        }
        if in_fence {
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

fn antigravity_write_dot_env() -> Result<bool, String> {
    let file = antigravity_env_file();
    let url = antigravity_url();
    let raw = paths::read_file_safe(&file).unwrap_or_default();
    if !antigravity_can_replace(&raw, ANTIGRAVITY_PROXY_ENV_KEY, &url)
        || !antigravity_can_replace(&raw, ANTIGRAVITY_CLOUD_CODE_KEY, &url)
    {
        return Ok(false);
    }
    if antigravity_env_value(&raw, ANTIGRAVITY_PROXY_ENV_KEY) == url
        && antigravity_env_value(&raw, ANTIGRAVITY_CLOUD_CODE_KEY) == url
    {
        return Ok(false);
    }
    let next = antigravity_strip_fence(&raw);
    let mut sb = next.strip_suffix('\n').unwrap_or(&next).to_string();
    if !sb.is_empty() {
        sb.push('\n');
    }
    sb.push_str(&antigravity_dot_env_block());
    paths::write_file(&file, &sb).map_err(|e| e.to_string())?;
    Ok(true)
}

fn antigravity_apply_process_env() {
    let u = antigravity_url();
    for key in [ANTIGRAVITY_PROXY_ENV_KEY, ANTIGRAVITY_CLOUD_CODE_KEY] {
        let cur = std::env::var(key).unwrap_or_default();
        if cur.is_empty() || cur == u {
            // SAFETY: Tokless CLI updates process environment synchronously on main thread.
            unsafe {
                std::env::set_var(key, &u);
            }
        }
    }
}

fn antigravity_process_env_compatible() -> bool {
    let u = antigravity_url();
    for key in [ANTIGRAVITY_PROXY_ENV_KEY, ANTIGRAVITY_CLOUD_CODE_KEY] {
        if let Ok(v) = std::env::var(key)
            && !v.is_empty()
            && v != u
        {
            return false;
        }
    }
    true
}

fn antigravity_clear_process_env() {
    let u = antigravity_url();
    for key in [ANTIGRAVITY_CLOUD_CODE_KEY, ANTIGRAVITY_PROXY_ENV_KEY] {
        if std::env::var(key).ok().as_deref() == Some(u.as_str()) {
            // SAFETY: Tokless CLI updates process environment synchronously on main thread.
            unsafe {
                std::env::remove_var(key);
            }
        }
    }
}

fn antigravity_windows_user_env_compatible() -> bool {
    if !paths::is_win() {
        return true;
    }
    let u = antigravity_url();
    #[cfg(windows)]
    {
        for name in [ANTIGRAVITY_PROXY_ENV_KEY, ANTIGRAVITY_CLOUD_CODE_KEY] {
            if let Some(cur) = crate::env_injector::windows::get_env_reg_value(name) {
                if !cur.is_empty() && cur != u {
                    return false;
                }
            }
        }
    }
    true
}

fn antigravity_write_windows_user_env() -> (bool, bool) {
    if !paths::is_win() {
        return (false, true);
    }
    let u = antigravity_url();
    #[cfg(windows)]
    {
        let mut changed = false;
        let mut mask: u32 = 0;
        let old_marker = crate::env_injector::windows::get_env_reg_value(ANTIGRAVITY_WINDOWS_MARKER);
        if let Some(ref m) = old_marker {
            if let Some(caps) = regex::Regex::new(r"(?:^|,)proxy=(\d+)").ok().and_then(|re| re.captures(m)) {
                if let Some(m_val) = caps.get(1) {
                    mask = m_val.as_str().parse().unwrap_or(0);
                }
            }
        }

        let keys = [(ANTIGRAVITY_PROXY_ENV_KEY, 1u32), (ANTIGRAVITY_CLOUD_CODE_KEY, 2u32)];
        for (key, bit) in keys {
            let cur = crate::env_injector::windows::get_env_reg_value(key);
            if cur.as_deref() != Some(&u) {
                if crate::env_injector::windows::set_env_reg_value(key, &u).is_err() {
                    return (false, false);
                }
                changed = true;
                mask |= bit;
            }
        }

        let marker_val = format!("proxy={}", mask);
        let _ = crate::env_injector::windows::set_env_reg_value(ANTIGRAVITY_WINDOWS_MARKER, &marker_val);
        if changed {
            crate::env_injector::windows::broadcast_setting_change();
        }
        (changed, true)
    }
    #[cfg(not(windows))]
    {
        (false, true)
    }
}

fn antigravity_clear_windows_user_env() -> bool {
    if !paths::is_win() {
        return false;
    }
    #[cfg(windows)]
    {
        let u = antigravity_url();
        let marker = match crate::env_injector::windows::get_env_reg_value(ANTIGRAVITY_WINDOWS_MARKER) {
            Some(m) => m,
            None => return false,
        };

        let mut mask: u32 = 0;
        if let Some(caps) = regex::Regex::new(r"^proxy=(\d+)$").ok().and_then(|re| re.captures(&marker)) {
            if let Some(m_val) = caps.get(1) {
                mask = m_val.as_str().parse().unwrap_or(0);
            }
        }

        let mut changed = false;
        let keys = [(ANTIGRAVITY_PROXY_ENV_KEY, 1u32), (ANTIGRAVITY_CLOUD_CODE_KEY, 2u32)];
        for (key, bit) in keys {
            if (mask & bit) != 0 {
                if let Some(cur) = crate::env_injector::windows::get_env_reg_value(key) {
                    if cur == u {
                        let _ = crate::env_injector::windows::delete_env_reg_value(key);
                        changed = true;
                    }
                }
            }
        }

        let _ = crate::env_injector::windows::delete_env_reg_value(ANTIGRAVITY_WINDOWS_MARKER);
        if changed {
            crate::env_injector::windows::broadcast_setting_change();
        }
        changed
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn configure_antigravity_proxy() -> (bool, String) {
    let file = antigravity_env_file();
    let raw_opt = paths::read_file_safe(&file);
    let existed = raw_opt.is_some();
    let raw = raw_opt.as_deref().unwrap_or("");
    if !existed && paths::exists(&file) {
        return (false, file);
    }
    let url = antigravity_url();
    if !antigravity_process_env_compatible()
        || !antigravity_windows_user_env_compatible()
        || !antigravity_shim_compatible()
        || !antigravity_ide_compatible()
        || !antigravity_can_replace(raw, ANTIGRAVITY_PROXY_ENV_KEY, &url)
        || !antigravity_can_replace(raw, ANTIGRAVITY_CLOUD_CODE_KEY, &url)
    {
        return (false, file);
    }
    let Ok(dot_changed) = antigravity_write_dot_env() else {
        return (false, file);
    };
    let (win_changed, win_ok) = antigravity_write_windows_user_env();
    if !win_ok {
        if existed {
            if let Err(rollback_err) = paths::write_file(&file, raw) {
                L.err(&format!("antigravity proxy rollback failed: {rollback_err}"));
            }
        } else if let Err(rollback_err) = fs::remove_file(&file)
            && rollback_err.kind() != std::io::ErrorKind::NotFound
        {
            L.err(&format!("antigravity proxy rollback failed: {rollback_err}"));
        }
        return (false, file);
    }
    let ide_was_wired = antigravity_ide_proxy_wired();
    let Ok(ide_changed) = configure_antigravity_ide() else {
        if win_changed {
            let _ = antigravity_clear_windows_user_env();
        }
        if existed {
            let _ = paths::write_file(&file, raw);
        } else {
            let _ = fs::remove_file(&file);
        }
        return (false, file);
    };
    let Ok(shim_changed) = install_antigravity_shim() else {
        if ide_changed && !ide_was_wired {
            let _ = remove_antigravity_ide();
        }
        if win_changed {
            let _ = antigravity_clear_windows_user_env();
        }
        if existed {
            let _ = paths::write_file(&file, raw);
        } else {
            let _ = fs::remove_file(&file);
        }
        return (false, file);
    };
    antigravity_apply_process_env();
    (
        dot_changed || win_changed || ide_changed || shim_changed,
        file,
    )
}

pub fn remove_antigravity_proxy() -> bool {
    let file = antigravity_env_file();
    let raw_opt = paths::read_file_safe(&file);
    let url = antigravity_url();
    let mut removed = false;
    if remove_antigravity_shim() {
        removed = true;
    }
    if remove_antigravity_ide() {
        removed = true;
    }
    if let Some(ref raw) = raw_opt
        && antigravity_env_value(raw, ANTIGRAVITY_PROXY_ENV_KEY) == url
        && antigravity_env_value(raw, ANTIGRAVITY_CLOUD_CODE_KEY) == url
    {
        let next = antigravity_strip_fence(raw);
        if next != *raw {
            let next = next.strip_suffix('\n').unwrap_or(&next);
            if next.trim().is_empty() {
                removed = remove_file_if_exists(&file);
            } else {
                removed = paths::write_file(&file, &format!("{next}\n")).is_ok();
            }
        }
    }
    if antigravity_remove_shell_exports() {
        removed = true;
    }
    if antigravity_clear_windows_user_env() {
        removed = true;
    }
    antigravity_clear_process_env();
    removed
}

pub fn antigravity_proxy_wired() -> bool {
    let Some(raw) = paths::read_file_safe(&antigravity_env_file()) else {
        return false;
    };
    let url = antigravity_url();
    if antigravity_env_value(&raw, ANTIGRAVITY_PROXY_ENV_KEY) != url
        || antigravity_env_value(&raw, ANTIGRAVITY_CLOUD_CODE_KEY) != url
    {
        return false;
    }
    let mut has_surface = false;
    if antigravity_shim_applicable() {
        has_surface = true;
        if !antigravity_shim_wired() {
            return false;
        }
    }
    if antigravity_ide_applicable() {
        has_surface = true;
        if !antigravity_ide_proxy_wired() {
            return false;
        }
    }
    if paths::is_win() {
        has_surface = true;
    }
    if !has_surface {
        has_surface = true;
    }
    has_surface
}

pub fn antigravity_proxy_session_ready() -> bool {
    let u = antigravity_url();
    std::env::var(ANTIGRAVITY_CLOUD_CODE_KEY).ok().as_deref() == Some(u.as_str())
        && std::env::var(ANTIGRAVITY_PROXY_ENV_KEY).ok().as_deref() == Some(u.as_str())
}

fn remove_file_if_exists(path: &str) -> bool {
    if paths::exists(path) && paths::remove_file_retry(path).is_err() {
        return false;
    }
    true
}

// --- antigravity_shim.go ---

fn antigravity_bin_file() -> String {
    let name = if paths::is_win() { "agy.cmd" } else { "agy" };
    let dirs = agy_known_bin_dirs();
    for dir in &dirs {
        let bin = Path::new(dir).join(name).to_string_lossy().into_owned();
        if paths::exists(&bin)
            || paths::exists(&antigravity_real_bin_in(dir))
            || paths::exists(&antigravity_vendor_bin_in(dir))
        {
            return bin;
        }
    }
    let Some(first) = dirs.first() else {
        return String::new();
    };
    Path::new(first).join(name).to_string_lossy().into_owned()
}

fn antigravity_vendor_bin_in(dir: &str) -> String {
    let name = if paths::is_win() { "agy.exe" } else { "agy" };
    Path::new(dir).join(name).to_string_lossy().into_owned()
}

fn antigravity_real_bin_in(dir: &str) -> String {
    let name = if paths::is_win() {
        "agy.real.exe"
    } else {
        "agy.real"
    };
    Path::new(dir).join(name).to_string_lossy().into_owned()
}

fn antigravity_vendor_bin_file() -> String {
    let bin = antigravity_bin_file();
    let dir = Path::new(&bin)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    antigravity_vendor_bin_in(&dir)
}

fn antigravity_real_bin_file() -> String {
    let bin = antigravity_bin_file();
    let dir = Path::new(&bin)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    antigravity_real_bin_in(&dir)
}

fn antigravity_shim_state_file() -> String {
    Path::new(&paths::tokless_data_dir())
        .join("antigravity-cli-proxy.json")
        .to_string_lossy()
        .into_owned()
}

fn looks_like_antigravity_shim(raw: &str) -> bool {
    if paths::is_win() {
        return raw.starts_with("@echo off\r\n")
            && raw.contains(ANTIGRAVITY_SHIM_MARKER)
            && raw.contains("__proxy-ensure")
            && raw.contains(ANTIGRAVITY_PROXY_ENV_KEY)
            && raw.contains(ANTIGRAVITY_CLOUD_CODE_KEY)
            && raw.contains("\"%REAL%\" %*");
    }
    raw.starts_with("#!/bin/sh\n")
        && raw.contains(ANTIGRAVITY_SHIM_MARKER)
        && raw.contains("TOKLESS=")
        && raw.contains("__proxy-ensure")
        && raw.contains("GOOGLE_GEMINI_BASE_URL=")
        && raw.contains("CLOUD_CODE_URL=")
        && raw.trim().ends_with("exec \"$REAL\" \"$@\"")
}

fn read_antigravity_launcher(path: &str) -> Result<(String, bool), std::io::Error> {
    let mut f = File::open(path)?;
    let mut header = [0u8; 2];
    let n = f.read(&mut header)?;
    if n < 2 || &header != b"#!" {
        return Ok((String::new(), false));
    }
    f.seek(SeekFrom::Start(0))?;
    let mut buf = Vec::new();
    f.take(64 << 10).read_to_end(&mut buf)?;
    Ok((String::from_utf8_lossy(&buf).into_owned(), true))
}

pub fn antigravity_shim_applicable() -> bool {
    false
}

pub fn antigravity_shim_compatible() -> bool {
    true
}

fn install_antigravity_shim() -> Result<bool, String> {
    // Purged in Phase 9: Tokless never hijacks agent binaries or creates shim wrappers.
    let _ = remove_antigravity_shim();
    Ok(false)
}

fn remove_antigravity_shim() -> bool {
    let bin = antigravity_bin_file();
    let vendor = antigravity_vendor_bin_file();
    let real = antigravity_real_bin_file();
    let mut changed = false;

    // Delete shim wrapper if it exists and matches Tokless pattern
    if paths::is_win() {
        if let Some(raw) = paths::read_file_safe(&bin) {
            if looks_like_antigravity_shim(&raw) {
                let _ = paths::remove_file_retry(&bin);
                changed = true;
            }
        }
        // Restore real binary to vendor (agy.exe)
        if paths::exists(&real) {
            if paths::exists(&vendor) {
                let _ = paths::remove_file_retry(&vendor);
            }
            if fs::rename(&real, &vendor).is_ok() {
                changed = true;
            }
        }
    } else {
        if let Ok((raw, _)) = read_antigravity_launcher(&bin) {
            if looks_like_antigravity_shim(&raw) {
                let _ = paths::remove_file_retry(&bin);
                changed = true;
            }
        }
        if paths::exists(&real) {
            if paths::exists(&bin) {
                let _ = paths::remove_file_retry(&bin);
            }
            if fs::rename(&real, &bin).is_ok() {
                changed = true;
            }
        }
    }

    if paths::exists(&antigravity_shim_state_file()) {
        let _ = paths::remove_file_retry(antigravity_shim_state_file());
        changed = true;
    }

    changed
}

pub fn antigravity_shim_wired() -> bool {
    false
}
