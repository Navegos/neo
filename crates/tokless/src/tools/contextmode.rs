// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/contextmode.rs
// created: 2026-10-05
// lastModified: 2026-10-28

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::agents::{
    allow_claude_mcp_tool, allow_claude_mcp_tool_project_local, antigravity_mcp_bounded,
    antigravity_mcp_has, cleanup_dead_ide_hooks, cleanup_legacy_antigravity_context_mode,
    cline_mcp_matches, copilot_mcp_bounded, copilot_mcp_has, cursor_mcp_has, droid_mcp_bounded,
    droid_mcp_has, grok_context_mode_mcp_has, has_antigravity_context_mode_hook,
    has_copilot_context_mode_hook, has_copilot_ide_context_mode_hook, has_cursor_mcp_permissions,
    has_droid_ctx_mode_pre_tool_use, install_copilot_context_mode_hook_safe,
    install_copilot_ide_context_mode_hook_safe, kilo_mcp_matches, omp_mcp_has,
    open_code_mcp_matches, pi_mcp_bounded, pi_mcp_has, pi_purge_context_mode_packages,
    remove_antigravity_context_mode_hook, remove_antigravity_entry,
    remove_copilot_context_mode_hook_safe, remove_copilot_ide_context_mode_hook_safe,
    remove_droid_ctx_mode_pre_tool_use, sync_copilot_ide_instructions_safe,
};
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::util::exec::{resolve_bun_binary, run, RunOptions};
use crate::util::instructions::section_header_for_owner;
use crate::util::jsonc::{stringify_json, try_parse_jsonc, OrderedMap, Value};
use crate::util::logger::L;
use crate::util::mcpspawn::pick_mcp_spawn;
use crate::util::paths::{
    claude_code_paths, codex_paths_resolved, ensure_dir, exists, home,
    opencode_paths_resolved, opencode_paths_v2, read_file_safe,
    which, write_file, write_file_atomic,
};
use crate::util::toml::remove_block;
use super::copilot_transaction::with_copilot_transaction;
use super::helpers::{
    get_arr, get_or_create_map, has_owner, is_test, kilo_has_owner, kilo_remove_owner,
    kilo_write_owner, load_ordered, remove_owner, remove_owner_safe, write_owner,
};
use super::mcp_dispatch::{mcp_adapter, mcp_configure, mcp_has, mcp_remove};

pub const CONTEXT_MODE_MIN_NODE: i32 = 22;
pub const CTX_GEMINI_MARKER: &str = "context-mode — MANDATORY routing rules";

pub fn plugin_is_context_mode(entry: &str) -> bool {
    entry == "context-mode" || entry.starts_with("context-mode@")
}

pub fn ctx_ensure_installed(opts: &RunOpts) -> Result<bool, DynError> {
    if let Some(ref report) = opts.report {
        report("ready", 1.0);
    }
    Ok(true)
}

pub fn set_context_mode_plugin_bare(config_path: &str) -> bool {
    let mut cfg = load_ordered(config_path);
    remove_context_mode_plugin_in_map(&mut cfg);
    let _ = write_file_atomic(config_path, &stringify_json(&Value::Object(cfg)));
    true
}

pub fn remove_context_mode_plugin(config_path: &str) -> bool {
    set_context_mode_plugin_bare(config_path)
}

pub fn remove_context_mode_plugin_in_map(cfg: &mut OrderedMap) {
    let plugins = get_arr(cfg, "plugin");
    let kept: Vec<Value> = plugins
        .into_iter()
        .filter(|p| {
            p.as_str()
                .map(|s| !plugin_is_context_mode(s))
                .unwrap_or(true)
        })
        .collect();
    if kept.is_empty() && !cfg.contains_key("plugin") {
        return;
    }
    cfg.insert("plugin".to_string(), Value::Array(kept));
}

pub fn clean_all_context_mode_cache() {
    let cache_root = Path::new(&home()).join(".cache").join("opencode").join("packages");
    if let Ok(entries) = fs::read_dir(&cache_root) {
        let mut n = 0;
        for e in entries.flatten() {
            let d = e.file_name().to_string_lossy().to_string();
            if d == "context-mode" || d.starts_with("context-mode@") {
                let _ = fs::remove_dir_all(cache_root.join(&d));
                n += 1;
            }
        }
        if n > 0 {
            L.sub(&format!("cleaned {} old context-mode cache dir(s)", n));
        }
    }
    let legacy = Path::new(&home()).join(".cache").join("opencode").join("node_modules").join("context-mode");
    if legacy.exists() {
        let _ = fs::remove_dir_all(legacy);
    }
}

pub fn run_postinstall_in_opencode_cache() {
    if which("bun").is_empty() {
        return;
    }
    let cache_root = Path::new(&home()).join(".cache").join("opencode").join("packages");
    let Ok(entries) = fs::read_dir(&cache_root) else {
        return;
    };
    let bun_bin = resolve_bun_binary();
    if bun_bin.is_empty() {
        return;
    }
    for e in entries.flatten() {
        let d = e.file_name().to_string_lossy().to_string();
        if d != "context-mode" && !d.starts_with("context-mode@") {
            continue;
        }
        let pkg_host = cache_root.join(&d);
        if !exists(pkg_host.join("node_modules").join("context-mode")) {
            continue;
        }
        let r = run(
            &bun_bin,
            &["pm", "trust", "context-mode"],
            RunOptions {
                cwd: Some(pkg_host.to_string_lossy().to_string()),
                capture: true,
                ..Default::default()
            },
        );
        if r.code == 0 {
            L.sub(&format!("healed OpenCode plugin cache ({})", d));
        }
    }
}

fn is_ours_for_event(entry: &Value, event: &str) -> bool {
    let Some(em) = entry.as_object() else {
        return false;
    };
    let Some(Value::Array(arr)) = em.get("hooks") else {
        return false;
    };
    let ev_lower = event.to_lowercase();
    let prefix = format!("context-mode hook codex {}", ev_lower);
    let legacy_prefix = format!("tokless context-mode-hook codex {}", ev_lower);
    let alt_prefix = "tokless codex-sessionstart";
    for h in arr {
        let Some(hm) = h.as_object() else {
            continue;
        };
        if let Some(s) = hm.get("command").and_then(|v| v.as_str()) {
            if s.starts_with(&prefix) || s.starts_with(&legacy_prefix) || s.starts_with(alt_prefix) || s.contains("context-mode") {
                return true;
            }
        }
    }
    false
}

pub fn remove_codex_context_mode_hooks(mut out: OrderedMap) -> OrderedMap {
    let events = [
        "PreToolUse",
        "PostToolUse",
        "UserPromptSubmit",
        "SessionStart",
        "PreCompact",
        "Stop",
        "PermissionRequest",
    ];
    let hooks = get_or_create_map(&mut out, "hooks");
    for event in events {
        let arr = get_arr(hooks, event);
        let filtered: Vec<Value> = arr
            .into_iter()
            .filter(|entry| !is_ours_for_event(entry, event))
            .collect();
        if filtered.is_empty() {
            hooks.remove(event);
        } else {
            hooks.insert(event.to_string(), Value::Array(filtered));
        }
    }
    if hooks.is_empty() {
        out.remove("hooks");
    }
    out
}

fn cleanup_codex_context_mode_hooks_in_dir(dir: &str) {
    let hooks_path = Path::new(dir).join("hooks.json");
    let hooks_str = hooks_path.to_string_lossy().to_string();
    if !exists(&hooks_str) {
        return;
    }
    let raw = read_file_safe(&hooks_str).unwrap_or_default();
    let next = remove_codex_context_mode_hooks(load_ordered(&hooks_str));
    if next.is_empty() {
        let _ = fs::remove_file(&hooks_path);
        return;
    }
    let s = stringify_json(&Value::Object(next));
    if s != raw {
        let _ = write_file(&hooks_str, &s);
    }
}

pub fn cleanup_codex_context_mode_hooks() {
    let cx = codex_paths_resolved();
    cleanup_codex_context_mode_hooks_in_dir(&cx.dir);
    if let Ok(cwd) = std::env::current_dir() {
        let project_codex = cwd.join(".codex").to_string_lossy().to_string();
        if project_codex != cx.dir {
            cleanup_codex_context_mode_hooks_in_dir(&project_codex);
        }
    }
}

pub fn cleanup_workspace_codex_context_mode_mcp(active_codex_dir: &str) {
    let Ok(cwd) = std::env::current_dir() else {
        return;
    };
    let project_codex = cwd.join(".codex").to_string_lossy().to_string();
    if project_codex == active_codex_dir {
        return;
    }
    let config_path = Path::new(&project_codex).join("config.toml").to_string_lossy().to_string();
    let Some(raw) = read_file_safe(&config_path) else {
        return;
    };
    let next = remove_block(&raw, "mcp_servers.context-mode");
    let next = remove_block(&next, "mcp_servers.context_mode");
    if next != raw {
        let _ = write_file(&config_path, &next);
    }
}

fn codex_hook_dirs(active_codex_dir: &str) -> Vec<String> {
    let mut dirs = vec![active_codex_dir.to_string()];
    if let Ok(cwd) = std::env::current_dir() {
        let project = cwd.join(".codex").to_string_lossy().to_string();
        if project != active_codex_dir {
            dirs.push(project);
        }
    }
    dirs
}

fn has_codex_hook_entry(hooks: &OrderedMap, event: &str) -> bool {
    let Some(Value::Array(arr)) = hooks.get(event) else {
        return false;
    };
    arr.iter().any(|entry| is_ours_for_event(entry, event))
}

pub fn ctx_configure_mcp(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if mcp_configure(agent, "context-mode") {
        Ok(true)
    } else {
        Err(format!("failed to configure context-mode MCP for {}", agent).into())
    }
}

pub fn ctx_verify_claude() -> bool {
    mcp_has("claude", "context-mode", false)
}

pub fn ctx_verify_opencode() -> bool {
    let op = opencode_paths_resolved();
    let Some(raw) = read_file_safe(&op.config) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    if let Some(Value::Array(arr)) = cfg.get("plugin") {
        for p in arr {
            if let Some(s) = p.as_str() {
                if plugin_is_context_mode(s) {
                    return false;
                }
            }
        }
    }
    let spawn = pick_mcp_spawn("context-mode", &[]);
    let mut command = vec![spawn.command];
    command.extend(spawn.args);
    open_code_mcp_matches(&cfg, "context-mode", &command)
}

pub fn ctx_verify_opencode_v2() -> bool {
    let op = opencode_paths_v2();
    let Some(raw) = read_file_safe(&op.config) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    if let Some(Value::Array(arr)) = cfg.get("plugin") {
        for p in arr {
            if let Some(s) = p.as_str() {
                if plugin_is_context_mode(s) {
                    return false;
                }
            }
        }
    }
    let spawn = pick_mcp_spawn("context-mode", &[]);
    let mut command = vec![spawn.command];
    command.extend(spawn.args);
    open_code_mcp_matches(&cfg, "context-mode", &command)
}

pub fn ctx_verify_codex() -> bool {
    let cx = codex_paths_resolved();
    let Some(raw) = read_file_safe(&cx.config) else {
        return false;
    };
    if raw.contains("[mcp_servers.context-mode]") {
        return false;
    }
    if !mcp_has("codex", "context-mode", false) {
        return false;
    }
    let Some(agents_raw) = read_file_safe(&cx.instructions) else {
        return false;
    };
    let hdr = section_header_for_owner("context-mode");
    if hdr.is_empty() || !agents_raw.contains(hdr) {
        return false;
    }
    for dir in codex_hook_dirs(&cx.dir) {
        let p = Path::new(&dir).join("hooks.json").to_string_lossy().to_string();
        if !exists(&p) {
            continue;
        }
        let data = load_ordered(&p);
        if let Some(Value::Object(hm)) = data.get("hooks") {
            for ev in [
                "PreToolUse",
                "PreCompact",
                "Stop",
                "SessionStart",
                "PostToolUse",
                "UserPromptSubmit",
                "PermissionRequest",
            ] {
                if has_codex_hook_entry(hm, ev) {
                    return false;
                }
            }
        }
    }
    true
}

pub fn ctx_verify_antigravity() -> bool {
    antigravity_mcp_bounded("context-mode") && !has_antigravity_context_mode_hook()
}

pub fn ctx_verify_copilot() -> bool {
    copilot_mcp_bounded("context-mode", false)
        && copilot_mcp_bounded("context-mode", true)
        && has_copilot_context_mode_hook()
        && has_copilot_ide_context_mode_hook()
}

pub fn ctx_verify_droid() -> bool {
    droid_mcp_bounded("context-mode") && !has_droid_ctx_mode_pre_tool_use()
}

pub fn ctx_verify_pi() -> bool {
    pi_mcp_bounded("context-mode")
}

pub fn ctx_verify_grok() -> bool {
    grok_context_mode_mcp_has() && has_owner("grok", "context-mode")
}

pub fn ctx_verify(agent: &str) -> bool {
    match agent {
        "claude" => ctx_verify_claude(),
        "opencode" => ctx_verify_opencode(),
        "opencode2" => ctx_verify_opencode_v2(),
        "codex" => ctx_verify_codex(),
        "cursor" => cursor_mcp_has("context-mode") && has_cursor_mcp_permissions("context-mode"),
        "antigravity" => ctx_verify_antigravity(),
        "copilot" => ctx_verify_copilot(),
        "droid" => ctx_verify_droid(),
        "grok" => ctx_verify_grok(),
        "pi" => ctx_verify_pi(),
        "omp" => omp_mcp_has("context-mode") && has_owner("omp", "context-mode"),
        "kilo" => {
            let spawn = pick_mcp_spawn("context-mode", &[]);
            let mut expected = vec![spawn.command];
            expected.extend(spawn.args);
            kilo_mcp_matches("context-mode", &expected) && kilo_has_owner("context-mode")
        }
        "cline" => {
            let spawn = pick_mcp_spawn("context-mode", &[]);
            let mut expected = vec![spawn.command];
            expected.extend(spawn.args);
            cline_mcp_matches("context-mode", &expected) && has_owner("cline", "context-mode")
        }
        "amp" | "kimi" | "cmd" | "dsh" | "zcode" => {
            mcp_has(agent, "context-mode", false) && has_owner(agent, "context-mode")
        }
        _ => false,
    }
}

pub fn ctx_wired_any_claude() -> bool {
    let Some(raw) = read_file_safe(&claude_code_paths().global_json) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    cfg.get("mcpServers")
        .and_then(|v| v.as_object())
        .is_some_and(|sm| sm.contains_key("context-mode"))
}

pub fn ctx_wired_any_opencode() -> bool {
    let Some(raw) = read_file_safe(&opencode_paths_resolved().config) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    if let Some(Value::Array(arr)) = cfg.get("plugin") {
        for p in arr {
            if let Some(s) = p.as_str() {
                if plugin_is_context_mode(s) {
                    return true;
                }
            }
        }
    }
    if let Some(Value::Object(mm)) = cfg.get("mcp") {
        if mm.contains_key("context-mode") {
            return true;
        }
        if let Some(Value::Object(sm)) = mm.get("servers") {
            if sm.contains_key("context-mode") {
                return true;
            }
        }
    }
    false
}

pub fn ctx_wired_any_opencode_v2() -> bool {
    let Some(raw) = read_file_safe(&opencode_paths_v2().config) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    if let Some(Value::Array(arr)) = cfg.get("plugin") {
        for p in arr {
            if let Some(s) = p.as_str() {
                if plugin_is_context_mode(s) {
                    return true;
                }
            }
        }
    }
    if let Some(Value::Object(mm)) = cfg.get("mcp") {
        if mm.contains_key("context-mode") {
            return true;
        }
        if let Some(Value::Object(sm)) = mm.get("servers") {
            return sm.contains_key("context-mode");
        }
    }
    false
}

pub fn ctx_wired_any_codex() -> bool {
    let Some(raw) = read_file_safe(&codex_paths_resolved().config) else {
        return false;
    };
    raw.contains("[mcp_servers.context_mode]") || raw.contains("[mcp_servers.context-mode]")
}

pub fn ctx_wired_any(agent: &str) -> bool {
    match agent {
        "claude" => ctx_wired_any_claude(),
        "opencode" => ctx_wired_any_opencode(),
        "opencode2" => ctx_wired_any_opencode_v2(),
        "codex" => ctx_wired_any_codex(),
        "droid" => droid_mcp_has("context-mode"),
        "pi" => pi_mcp_has("context-mode"),
        "antigravity" => antigravity_mcp_has("context-mode"),
        "copilot" => copilot_mcp_has("context-mode"),
        _ => mcp_has(agent, "context-mode", true),
    }
}

pub fn ctx_wire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    match agent {
        "claude" => {
            if is_test() {
                let spawn = pick_mcp_spawn("context-mode", &[]);
                let cp = claude_code_paths();
                let _ = ensure_dir(&cp.dir);
                let mut cfg = load_ordered(&cp.global_json);
                let servers = get_or_create_map(&mut cfg, "mcpServers");
                let mut entry = OrderedMap::new();
                entry.insert("type".to_string(), Value::String("stdio".to_string()));
                entry.insert("command".to_string(), Value::String(spawn.command));
                entry.insert(
                    "args".to_string(),
                    Value::Array(spawn.args.into_iter().map(Value::String).collect()),
                );
                servers.insert("context-mode".to_string(), Value::Object(entry));
                let _ = write_file(&cp.global_json, &stringify_json(&Value::Object(cfg)));
                allow_claude_mcp_tool("context-mode");
                write_owner("claude", "context-mode");
                return Ok(ctx_verify_claude());
            }
            mcp_configure("claude", "context-mode");
            allow_claude_mcp_tool_project_local("context-mode");
            write_owner("claude", "context-mode");
            L.sub("tip: to enable slash commands, type inside Claude Code: /plugin marketplace add mksglu/context-mode && /plugin install context-mode@context-mode");
            Ok(ctx_verify_claude())
        }
        "opencode" => {
            let op = opencode_paths_resolved();
            let _ = ensure_dir(&op.dir);
            let mut cfg = load_ordered(&op.config);
            remove_context_mode_plugin_in_map(&mut cfg);
            let _ = write_file(&op.config, &stringify_json(&Value::Object(cfg)));
            mcp_configure("opencode", "context-mode");
            write_owner("opencode", "context-mode");
            if !is_test() {
                clean_all_context_mode_cache();
                run_postinstall_in_opencode_cache();
            }
            Ok(ctx_verify_opencode())
        }
        "opencode2" => {
            let op = opencode_paths_v2();
            let _ = ensure_dir(&op.dir);
            let mut cfg = load_ordered(&op.config);
            remove_context_mode_plugin_in_map(&mut cfg);
            let _ = write_file(&op.config, &stringify_json(&Value::Object(cfg)));
            mcp_configure("opencode2", "context-mode");
            write_owner("opencode2", "context-mode");
            if !is_test() {
                clean_all_context_mode_cache();
                run_postinstall_in_opencode_cache();
            }
            Ok(ctx_verify_opencode_v2())
        }
        "codex" => {
            let cx = codex_paths_resolved();
            let _ = ensure_dir(&cx.dir);
            if let Some(raw) = read_file_safe(&cx.config) {
                let next = remove_block(&raw, "mcp_servers.context-mode");
                if next != raw {
                    let _ = write_file(&cx.config, &next);
                }
            }
            if !mcp_configure("codex", "context-mode") {
                return Ok(false);
            }
            cleanup_codex_context_mode_hooks();
            cleanup_workspace_codex_context_mode_mcp(&cx.dir);
            if !cx.instructions.is_empty() {
                write_owner("codex", "context-mode");
            }
            Ok(mcp_has("codex", "context-mode", false))
        }
        "cursor" => {
            if !mcp_configure("cursor", "context-mode") {
                return Ok(false);
            }
            Ok(mcp_has("cursor", "context-mode", false))
        }
        "antigravity" => {
            mcp_configure("antigravity", "context-mode");
            cleanup_legacy_antigravity_context_mode();
            cleanup_dead_ide_hooks();
            remove_antigravity_entry("command(echo)");
            remove_antigravity_context_mode_hook();
            write_owner("antigravity", "context-mode");
            Ok(ctx_verify_antigravity())
        }
        "copilot" => {
            with_copilot_transaction(&[], || {
                if let Some(adapter) = mcp_adapter("copilot") {
                    let reply = adapter.configure_mcp("context-mode", &[]);
                    if let Some(err) = reply.err {
                        return Err(err.into());
                    }
                }
                install_copilot_context_mode_hook_safe().map_err(|e| -> DynError { e.into() })?;
                install_copilot_ide_context_mode_hook_safe().map_err(|e| -> DynError { e.into() })?;
                if !write_owner("copilot", "context-mode") && !has_owner("copilot", "context-mode") {
                    return Err("failed to write Copilot context-mode owner".into());
                }
                sync_copilot_ide_instructions_safe().map_err(|e| -> DynError { e.into() })?;
                Ok(ctx_verify_copilot())
            })
        }
        "droid" => {
            mcp_configure("droid", "context-mode");
            remove_droid_ctx_mode_pre_tool_use();
            write_owner("droid", "context-mode");
            Ok(ctx_verify_droid())
        }
        "grok" => {
            if !mcp_configure("grok", "context-mode") {
                return Ok(false);
            }
            write_owner("grok", "context-mode");
            if !has_owner("grok", "context-mode") {
                mcp_remove("grok", "context-mode");
                return Ok(false);
            }
            Ok(ctx_verify_grok())
        }
        "pi" => {
            pi_purge_context_mode_packages();
            if !mcp_configure("pi", "context-mode") {
                return Ok(false);
            }
            write_owner("pi", "context-mode");
            Ok(ctx_verify_pi())
        }
        "omp" => {
            if !mcp_configure("omp", "context-mode") {
                return Ok(false);
            }
            write_owner("omp", "context-mode");
            Ok(has_owner("omp", "context-mode"))
        }
        "kilo" => {
            if !mcp_configure("kilo", "context-mode") {
                return Ok(false);
            }
            kilo_write_owner("context-mode");
            Ok(mcp_has("kilo", "context-mode", false) && kilo_has_owner("context-mode"))
        }
        "cline" => {
            if !mcp_configure("cline", "context-mode") {
                return Ok(false);
            }
            write_owner("cline", "context-mode");
            Ok(mcp_has("cline", "context-mode", false) && has_owner("cline", "context-mode"))
        }
        _ => {
            if !mcp_configure(agent, "context-mode") {
                return Ok(false);
            }
            write_owner(agent, "context-mode");
            Ok(mcp_has(agent, "context-mode", false) && has_owner(agent, "context-mode"))
        }
    }
}

pub fn ctx_unwire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    match agent {
        "claude" => {
            mcp_remove("claude", "context-mode");
            remove_owner("claude", "context-mode");
            Ok(true)
        }
        "opencode" => {
            let op = opencode_paths_resolved();
            if let Some(raw) = read_file_safe(&op.config) {
                let mut cfg = match try_parse_jsonc(&raw) {
                    Some(Value::Object(m)) => m,
                    _ => OrderedMap::new(),
                };
                if let Some(Value::Array(arr)) = cfg.get("plugin").cloned() {
                    let kept: Vec<Value> = arr
                        .into_iter()
                        .filter(|p| p.as_str().map(|s| !plugin_is_context_mode(s)).unwrap_or(true))
                        .collect();
                    if kept.is_empty() {
                        cfg.remove("plugin");
                    } else {
                        cfg.insert("plugin".to_string(), Value::Array(kept));
                    }
                }
                let _ = write_file(&op.config, &stringify_json(&Value::Object(cfg)));
            }
            mcp_remove("opencode", "context-mode");
            remove_owner("opencode", "context-mode");
            Ok(true)
        }
        "opencode2" => {
            let op = opencode_paths_v2();
            if let Some(raw) = read_file_safe(&op.config) {
                let mut cfg = match try_parse_jsonc(&raw) {
                    Some(Value::Object(m)) => m,
                    _ => OrderedMap::new(),
                };
                remove_context_mode_plugin_in_map(&mut cfg);
                let _ = write_file(&op.config, &stringify_json(&Value::Object(cfg)));
            }
            mcp_remove("opencode2", "context-mode");
            remove_owner("opencode2", "context-mode");
            Ok(true)
        }
        "codex" => {
            mcp_remove("codex", "context-mode");
            cleanup_codex_context_mode_hooks();
            remove_owner("codex", "context-mode");
            Ok(true)
        }
        "cursor" => {
            if !mcp_remove("cursor", "context-mode") {
                return Ok(false);
            }
            remove_owner("cursor", "context-mode");
            Ok(true)
        }
        "antigravity" => {
            mcp_remove("antigravity", "context-mode");
            cleanup_legacy_antigravity_context_mode();
            cleanup_dead_ide_hooks();
            remove_antigravity_context_mode_hook();
            remove_owner("antigravity", "context-mode");
            if let Ok(cwd) = std::env::current_dir() {
                let dest = cwd.join("GEMINI.md");
                if let Some(raw) = read_file_safe(dest.to_string_lossy().as_ref()) {
                    if raw.contains(CTX_GEMINI_MARKER) {
                        let _ = fs::remove_file(dest);
                    }
                }
            }
            Ok(true)
        }
        "copilot" => {
            with_copilot_transaction(&[], || {
                if let Some(adapter) = mcp_adapter("copilot") {
                    let reply = adapter.remove_mcp("context-mode", &[]);
                    if let Some(err) = reply.err {
                        return Err(err.into());
                    }
                }
                remove_copilot_context_mode_hook_safe().map_err(|e| -> DynError { e.into() })?;
                remove_copilot_ide_context_mode_hook_safe().map_err(|e| -> DynError { e.into() })?;
                remove_owner_safe("copilot", "context-mode").map_err(|e| -> DynError { e.into() })?;
                sync_copilot_ide_instructions_safe().map_err(|e| -> DynError { e.into() })?;
                Ok(true)
            })
        }
        "droid" => {
            mcp_remove("droid", "context-mode");
            remove_droid_ctx_mode_pre_tool_use();
            remove_owner("droid", "context-mode");
            Ok(true)
        }
        "grok" => {
            if !mcp_remove("grok", "context-mode") {
                return Ok(false);
            }
            remove_owner("grok", "context-mode");
            Ok(true)
        }
        "pi" => {
            pi_purge_context_mode_packages();
            mcp_remove("pi", "context-mode");
            remove_owner("pi", "context-mode");
            Ok(true)
        }
        "omp" => {
            if !mcp_remove("omp", "context-mode") {
                return Ok(false);
            }
            remove_owner("omp", "context-mode");
            Ok(true)
        }
        "kilo" => {
            if !mcp_remove("kilo", "context-mode") {
                return Ok(false);
            }
            kilo_remove_owner("context-mode");
            Ok(true)
        }
        "cline" => {
            if !mcp_remove("cline", "context-mode") {
                return Ok(false);
            }
            remove_owner("cline", "context-mode");
            Ok(true)
        }
        _ => {
            mcp_remove(agent, "context-mode");
            remove_owner(agent, "context-mode");
            Ok(true)
        }
    }
}

pub fn build_context_mode() -> ToolManifest {
    let mut tool = ToolManifest::new("context-mode", "Context-Mode");
    tool.description =
        "Routes long context off-thread to a sandbox, keeping the agent's window small.".into();
    tool.homepage = "https://github.com/mksglu/context-mode".into();
    tool.install_hint = "Native embedded in tokless (tokless ctx)".into();
    tool.channel = Channel::Binary;
    tool.needs_node = false;
    tool.min_node_major = 0;
    tool.install = Some(Arc::new(|opts| ctx_ensure_installed(opts)));

    let agents = [
        "claude",
        "opencode",
        "opencode2",
        "codex",
        "cursor",
        "antigravity",
        "copilot",
        "droid",
        "grok",
        "pi",
        "omp",
        "kilo",
        "cline",
        "amp",
        "kimi",
        "zcode",
        "dsh",
    ];

    for agent_id in agents {
        let ag = agent_id.to_string();
        let ag_w = ag.clone();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(move |opts| ctx_wire(&ag_w, opts)),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| ctx_unwire(&ag_u, opts)),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(ctx_verify(&ag_v))),
        );

        let ag_a = ag.clone();
        tool.wired_any_for.insert(
            ag,
            Arc::new(move || Some(ctx_wired_any(&ag_a))),
        );
    }

    tool
}
