use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::util::instructions::{cursor_project_rule_content, install_cursor_project_rules};
use crate::util::jsonc::{stringify_json, Value};
use crate::util::mcpspawn::{pick_mcp_spawn, wrap_context_mode};
use crate::util::npminstall::{node_major, node_too_old_hint, npm_install_global};
use crate::util::paths::{
    claude_code_paths, cline_paths_resolved, codex_paths_resolved, cursor_paths_resolved,
    home, is_win, kilo_paths_resolved, opencode_paths_resolved,
    prepend_process_path, read_file_safe, temp_dir, which, write_file_atomic,
};
use super::copilot_transaction::with_copilot_transaction;
use super::helpers::{
    allow_claude_mcp_server, cursor_mcp_path, disallow_claude_mcp_server, droid_mcp_path,
    grok_mcp_path, has_antigravity_mcp, has_array_mcp_entry, has_codex_mcp_entry, has_copilot_mcp,
    has_mcp_entry_at_file, has_owner, install_codex_perm_hook, is_test, kilo_has_owner,
    kilo_remove_owner, kilo_write_owner, load_ordered, omp_mcp_path, pi_ensure_mcp_adapter,
    pi_mcp_adapter_ready, pi_mcp_path, pi_remove_mcp_adapter, remove_antigravity_mcp,
    remove_array_mcp_entry, remove_codex_mcp_entry, remove_codex_perm_hook,
    remove_copilot_mcp_safe, remove_hook_group_in_hooks_obj, remove_json_mcp_entry,
    remove_owner_safe, sync_copilot_ide_instructions_safe, upsert_antigravity_mcp,
    upsert_array_mcp_entry, upsert_codex_mcp_entry, upsert_copilot_mcp_safe, upsert_json_mcp_entry,
    write_owner,
};

pub fn ctx_ensure_installed(opts: &RunOpts) -> Result<bool, DynError> {
    if is_test() {
        let shim_dir = Path::new(&temp_dir()).join("tokless-test-context-mode");
        let _ = fs::create_dir_all(&shim_dir);
        let shim_path = shim_dir.join(if is_win() { "context-mode.cmd" } else { "context-mode" });
        let _ = fs::write(&shim_path, if is_win() { "@echo 1.0.0" } else { "#!/bin/sh\necho 1.0.0\n" });
        prepend_process_path(&shim_dir.to_string_lossy());
        return Ok(true);
    }

    if let Some(ref report) = opts.report {
        report("checking", 0.1);
    }
    let major = node_major();
    if major < 18 {
        return Err(node_too_old_hint(18).into());
    }
    if !which("context-mode").is_empty() && !opts.upgrade {
        if let Some(ref report) = opts.report {
            report("already installed", 1.0);
        }
        return Ok(true);
    }
    if opts.dry_run {
        return Ok(true);
    }
    if let Some(ref report) = opts.report {
        report(if opts.upgrade { "upgrading" } else { "installing" }, 0.3);
    }
    if !npm_install_global("context-mode") {
        return Err("failed to install context-mode".into());
    }
    if let Some(ref report) = opts.report {
        report("ready", 1.0);
    }
    Ok(true)
}

pub fn set_context_mode_plugin_bare(config_path: &str) -> bool {
    let mut cfg = load_ordered(config_path);
    if let Some(Value::Array(arr)) = cfg.get_mut("plugin") {
        let before = arr.len();
        arr.retain(|v| {
            v.as_str()
                .map(|s| !s.starts_with("context-mode"))
                .unwrap_or(true)
        });
        if arr.len() != before {
            if arr.is_empty() {
                cfg.remove("plugin");
            }
            let _ = write_file_atomic(config_path, &stringify_json(&Value::Object(cfg)));
        }
    }
    true
}

pub fn remove_context_mode_plugin(config_path: &str) -> bool {
    set_context_mode_plugin_bare(config_path)
}

pub fn clean_all_context_mode_cache() {
    let cache_dir = Path::new(&home()).join(".cache").join("opencode").join("node_modules").join("context-mode");
    if cache_dir.exists() {
        let _ = fs::remove_dir_all(cache_dir);
    }
}

pub fn cleanup_codex_context_mode_hooks() {
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    remove_hook_group_in_hooks_obj(&p, "PreToolUse", "context-mode");
    remove_hook_group_in_hooks_obj(&p, "SessionStart", "context-mode");
}

pub fn ctx_configure_mcp(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let raw_spawn = pick_mcp_spawn("context-mode", &[]);
    let spawn = wrap_context_mode(raw_spawn);

    match agent {
        "claude" => {
            let p = claude_code_paths();
            if !upsert_json_mcp_entry(&p.global_json, "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for claude".into());
            }
            allow_claude_mcp_server("context-mode");
            Ok(true)
        }
        "opencode" => {
            let p = opencode_paths_resolved();
            set_context_mode_plugin_bare(&p.config);
            if !upsert_array_mcp_entry(&p.config, "context-mode", &spawn) {
                return Err("failed to configure context-mode MCP for opencode".into());
            }
            Ok(true)
        }
        "codex" => {
            cleanup_codex_context_mode_hooks();
            if !upsert_codex_mcp_entry("context-mode", &spawn) {
                return Err("failed to configure context-mode MCP for codex".into());
            }
            let _ = install_codex_perm_hook();
            Ok(true)
        }
        "antigravity" => {
            if !upsert_antigravity_mcp("context-mode", &spawn) {
                return Err("failed to configure context-mode MCP for antigravity".into());
            }
            Ok(true)
        }
        "copilot" => {
            upsert_copilot_mcp_safe("context-mode", &spawn, &spawn)?;
            Ok(true)
        }
        "droid" => {
            if !upsert_json_mcp_entry(&droid_mcp_path(), "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for droid".into());
            }
            Ok(true)
        }
        "pi" => {
            pi_ensure_mcp_adapter(false);
            if !upsert_json_mcp_entry(&pi_mcp_path(), "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for pi".into());
            }
            Ok(true)
        }
        "omp" => {
            if !upsert_json_mcp_entry(&omp_mcp_path(), "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for omp".into());
            }
            Ok(true)
        }
        "grok" => {
            if !upsert_json_mcp_entry(&grok_mcp_path(), "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for grok".into());
            }
            Ok(true)
        }
        "cursor" => {
            if !upsert_json_mcp_entry(&cursor_mcp_path(), "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for cursor".into());
            }
            Ok(true)
        }
        "kilo" => {
            let p = kilo_paths_resolved();
            if !upsert_array_mcp_entry(&p.config, "context-mode", &spawn) {
                return Err("failed to configure context-mode MCP for kilo".into());
            }
            Ok(true)
        }
        "cline" => {
            let p = cline_paths_resolved();
            if !upsert_json_mcp_entry(&p.mcp_config, "mcpServers", "context-mode", &spawn, &[]) {
                return Err("failed to configure context-mode MCP for cline".into());
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn ctx_verify(agent: &str) -> bool {
    match agent {
        "claude" => {
            has_mcp_entry_at_file(&claude_code_paths().global_json, "mcpServers", "context-mode", None)
                && has_owner("claude", "context-mode")
        }
        "opencode" => {
            has_array_mcp_entry(&opencode_paths_resolved().config, "context-mode", None)
                && has_owner("opencode", "context-mode")
        }
        "codex" => {
            has_codex_mcp_entry("context-mode", None) && has_owner("codex", "context-mode")
        }
        "antigravity" => {
            has_antigravity_mcp("context-mode", None) && has_owner("antigravity", "context-mode")
        }
        "copilot" => {
            has_copilot_mcp("context-mode", None, None) && has_owner("copilot", "context-mode")
        }
        "droid" => {
            has_mcp_entry_at_file(&droid_mcp_path(), "mcpServers", "context-mode", None)
                && has_owner("droid", "context-mode")
        }
        "pi" => {
            pi_mcp_adapter_ready()
                && has_mcp_entry_at_file(&pi_mcp_path(), "mcpServers", "context-mode", None)
                && has_owner("pi", "context-mode")
        }
        "omp" => {
            has_mcp_entry_at_file(&omp_mcp_path(), "mcpServers", "context-mode", None)
                && has_owner("omp", "context-mode")
        }
        "grok" => {
            has_mcp_entry_at_file(&grok_mcp_path(), "mcpServers", "context-mode", None)
                && has_owner("grok", "context-mode")
        }
        "cursor" => {
            let rules_p = Path::new(&cursor_paths_resolved().rules_dir)
                .join("context-mode.mdc")
                .to_string_lossy()
                .to_string();
            let rules_ok = if let Some(raw) = read_file_safe(&rules_p) {
                let expected = cursor_project_rule_content("context-mode");
                !expected.is_empty() && raw.replace("\r\n", "\n") == expected.replace("\r\n", "\n")
            } else {
                true
            };
            has_mcp_entry_at_file(&cursor_mcp_path(), "mcpServers", "context-mode", None) && rules_ok
        }
        "kilo" => {
            has_array_mcp_entry(&kilo_paths_resolved().config, "context-mode", None)
                && kilo_has_owner("context-mode")
        }
        "cline" => {
            has_mcp_entry_at_file(&cline_paths_resolved().mcp_config, "mcpServers", "context-mode", None)
                && has_owner("cline", "context-mode")
        }
        _ => false,
    }
}

pub fn ctx_wire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if agent == "copilot" {
        return with_copilot_transaction(&[], || {
            ctx_configure_mcp("copilot", opts)?;
            if !write_owner("copilot", "context-mode") {
                return Err("failed to write copilot context-mode instructions".into());
            }
            sync_copilot_ide_instructions_safe()?;
            Ok(ctx_verify("copilot"))
        });
    }
    ctx_configure_mcp(agent, opts)?;
    match agent {
        "kilo" => {
            if !kilo_write_owner("context-mode") {
                return Err("failed to write kilo context-mode instructions".into());
            }
        }
        "cursor" => {
            let _ = install_cursor_project_rules(".", false);
        }
        _ => {
            if !write_owner(agent, "context-mode") {
                return Err(format!("failed to write context-mode instructions for {}", agent).into());
            }
        }
    }
    Ok(ctx_verify(agent))
}

pub fn ctx_unwire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    match agent {
        "claude" => {
            remove_json_mcp_entry(&claude_code_paths().global_json, "mcpServers", "context-mode");
            disallow_claude_mcp_server("context-mode");
            remove_owner_safe("claude", "context-mode")?;
        }
        "opencode" => {
            let p = opencode_paths_resolved();
            remove_context_mode_plugin(&p.config);
            remove_array_mcp_entry(&p.config, "context-mode");
            clean_all_context_mode_cache();
            remove_owner_safe("opencode", "context-mode")?;
        }
        "codex" => {
            cleanup_codex_context_mode_hooks();
            remove_codex_mcp_entry("context-mode");
            if !has_codex_mcp_entry("codegraph", None) {
                remove_codex_perm_hook();
            }
            remove_owner_safe("codex", "context-mode")?;
        }
        "antigravity" => {
            remove_antigravity_mcp("context-mode");
            remove_owner_safe("antigravity", "context-mode")?;
        }
        "copilot" => {
            return with_copilot_transaction(&[], || {
                remove_copilot_mcp_safe("context-mode")?;
                remove_owner_safe("copilot", "context-mode")?;
                sync_copilot_ide_instructions_safe()?;
                Ok(true)
            });
        }
        "droid" => {
            remove_json_mcp_entry(&droid_mcp_path(), "mcpServers", "context-mode");
            remove_owner_safe("droid", "context-mode")?;
        }
        "pi" => {
            remove_json_mcp_entry(&pi_mcp_path(), "mcpServers", "context-mode");
            pi_remove_mcp_adapter();
            remove_owner_safe("pi", "context-mode")?;
        }
        "omp" => {
            remove_json_mcp_entry(&omp_mcp_path(), "mcpServers", "context-mode");
            remove_owner_safe("omp", "context-mode")?;
        }
        "grok" => {
            remove_json_mcp_entry(&grok_mcp_path(), "mcpServers", "context-mode");
            remove_owner_safe("grok", "context-mode")?;
        }
        "cursor" => {
            remove_json_mcp_entry(&cursor_mcp_path(), "mcpServers", "context-mode");
            let p = Path::new(&cursor_paths_resolved().rules_dir).join("context-mode.mdc");
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }
        "kilo" => {
            remove_array_mcp_entry(&kilo_paths_resolved().config, "context-mode");
            kilo_remove_owner("context-mode");
        }
        "cline" => {
            remove_json_mcp_entry(&cline_paths_resolved().mcp_config, "mcpServers", "context-mode");
            remove_owner_safe("cline", "context-mode")?;
        }
        _ => {}
    }
    Ok(true)
}

pub fn build_context_mode() -> ToolManifest {
    let mut tool = ToolManifest::new("context-mode", "Context Mode");
    tool.description = "Sandboxed context tools with session memory and focused analysis.".into();
    tool.homepage = "https://github.com/Navegos/context-mode".into();
    tool.install_hint = "npm install -g @navegos/context-mode".into();
    tool.channel = Channel::Npm;
    tool.needs_node = true;
    tool.min_node_major = 18;
    tool.install = Some(Arc::new(|opts| ctx_ensure_installed(opts)));

    let agents = [
        "claude", "opencode", "codex", "antigravity", "copilot", "droid", "grok", "pi", "omp", "kilo", "cline", "cursor",
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
            Arc::new(move || {
                if !is_test() && which("context-mode").is_empty() && which("npx").is_empty() {
                    return Some(false);
                }
                Some(ctx_verify(&ag_v))
            }),
        );
    }

    tool
}
