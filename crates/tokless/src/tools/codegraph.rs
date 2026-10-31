// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/codegraph.rs
// created: 2026-10-05
// lastModified: 2026-10-28

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::agents::{install_grok_codegraph_session_hook, pi_agent_dir_resolved, remove_grok_codegraph_session_hook};
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::util::instructions::{cursor_project_rule_content, install_cursor_project_rules};
use crate::util::mcpspawn::{pick_codegraph_spawn, wrap_auto_index};
use crate::util::paths::{
    claude_code_paths, cline_paths_resolved, cursor_paths_resolved, ensure_dir, exists,
    kilo_paths_resolved, opencode_paths_resolved, read_file_safe,
    tokless_persisted_abs, write_file_atomic,
};
use super::autoindex::unwire_auto_index;
use super::copilot_transaction::with_copilot_transaction;
use super::mcp_dispatch::{mcp_configure, mcp_has, mcp_remove};
use super::helpers::{
    allow_claude_mcp_server, configure_cursor_codegraph_permissions, cursor_mcp_path,
    disallow_claude_mcp_server, droid_mcp_path, grok_mcp_path, has_antigravity_mcp,
    has_array_mcp_entry, has_codex_codegraph_index_hook, has_codex_mcp_entry,
    has_copilot_codegraph_index_hook, has_copilot_ide_codegraph_index_hook, has_copilot_mcp,
    has_cursor_codegraph_index_hook, has_cursor_codegraph_permissions, has_mcp_entry_at_file,
    has_owner, install_codex_codegraph_index_hook, install_codex_perm_hook,
    install_copilot_codegraph_index_hook_safe, install_copilot_ide_codegraph_index_hook_safe,
    install_cursor_codegraph_index_hook, kilo_has_owner, kilo_remove_owner,
    kilo_write_owner, omp_mcp_path, pi_ensure_mcp_adapter, pi_mcp_adapter_ready, pi_mcp_path,
    pi_remove_mcp_adapter, remove_antigravity_mcp, remove_array_mcp_entry,
    remove_codex_codegraph_index_hook, remove_codex_mcp_entry, remove_codex_perm_hook,
    remove_copilot_codegraph_index_hook_safe, remove_copilot_ide_codegraph_index_hook_safe,
    remove_copilot_mcp_safe, remove_cursor_codegraph_index_hook,
    remove_cursor_codegraph_permissions, remove_json_mcp_entry, remove_owner_safe,
    sync_copilot_ide_instructions_safe, upsert_antigravity_mcp, upsert_array_mcp_entry,
    upsert_codex_mcp_entry, upsert_copilot_mcp_safe, upsert_json_mcp_entry, write_owner,
};

pub fn codegraph_ensure_installed(_opts: &RunOpts) -> Result<bool, DynError> {
    Ok(true)
}

pub fn codegraph_real_install(_opts: &RunOpts) -> Result<bool, DynError> {
    Ok(true)
}

pub fn codegraph_configure_mcp(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let raw_spawn = match pick_codegraph_spawn(&["serve", "--mcp"]) {
        Some(s) => s,
        None => return Err("codegraph binary not found; install @colbymchenry/codegraph".into()),
    };
    let spawn = wrap_auto_index(agent, raw_spawn.clone());

    match agent {
        "claude" => {
            let p = claude_code_paths();
            if !upsert_json_mcp_entry(&p.global_json, "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for claude".into());
            }
            allow_claude_mcp_server("codegraph");
            Ok(true)
        }
        "opencode" => {
            let p = opencode_paths_resolved();
            if !upsert_array_mcp_entry(&p.config, "codegraph", &spawn) {
                return Err("failed to configure codegraph MCP for opencode".into());
            }
            Ok(true)
        }
        "codex" => {
            if !upsert_codex_mcp_entry("codegraph", &spawn) {
                return Err("failed to configure codegraph MCP for codex".into());
            }
            let _ = install_codex_perm_hook();
            let _ = install_codex_codegraph_index_hook();
            Ok(true)
        }
        "antigravity" => {
            if !upsert_antigravity_mcp("codegraph", &spawn) {
                return Err("failed to configure codegraph MCP for antigravity".into());
            }
            Ok(true)
        }
        "copilot" => {
            let mut ide_spawn = wrap_auto_index("copilot", raw_spawn);
            if ide_spawn.args.len() >= 3 && ide_spawn.args[0] == "run-mcp" && ide_spawn.args[1] == "--agent" {
                let mut new_args = vec![
                    "run-mcp".into(),
                    "--agent".into(),
                    "copilot".into(),
                    "--workspace".into(),
                    "${workspaceFolder}".into(),
                ];
                new_args.extend_from_slice(&ide_spawn.args[3..]);
                ide_spawn.args = new_args;
            }
            upsert_copilot_mcp_safe("codegraph", &spawn, &ide_spawn)?;
            Ok(true)
        }
        "droid" => {
            if !upsert_json_mcp_entry(&droid_mcp_path(), "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for droid".into());
            }
            Ok(true)
        }
        "pi" => {
            pi_ensure_mcp_adapter(false);
            if !upsert_json_mcp_entry(&pi_mcp_path(), "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for pi".into());
            }
            write_pi_codegraph_index_extension();
            Ok(true)
        }
        "omp" => {
            if !upsert_json_mcp_entry(&omp_mcp_path(), "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for omp".into());
            }
            Ok(true)
        }
        "grok" => {
            if !upsert_json_mcp_entry(&grok_mcp_path(), "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for grok".into());
            }
            let _ = install_grok_codegraph_session_hook();
            Ok(true)
        }
        "cursor" => {
            if !upsert_json_mcp_entry(&cursor_mcp_path(), "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for cursor".into());
            }
            let _ = configure_cursor_codegraph_permissions();
            let _ = install_cursor_codegraph_index_hook();
            Ok(true)
        }
        "kilo" => {
            let p = kilo_paths_resolved();
            if !upsert_array_mcp_entry(&p.config, "codegraph", &spawn) {
                return Err("failed to configure codegraph MCP for kilo".into());
            }
            Ok(true)
        }
        "cline" => {
            let p = cline_paths_resolved();
            if !upsert_json_mcp_entry(&p.mcp_config, "mcpServers", "codegraph", &spawn, &[]) {
                return Err("failed to configure codegraph MCP for cline".into());
            }
            Ok(true)
        }
        "opencode2" | "amp" | "kimi" | "zcode" | "dsh" => {
            if !mcp_configure(agent, "codegraph") {
                return Err(format!("failed to configure codegraph MCP for {}", agent).into());
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn codegraph_verify(agent: &str) -> bool {
    match agent {
        "claude" => {
            has_mcp_entry_at_file(&claude_code_paths().global_json, "mcpServers", "codegraph", None)
                && has_owner("claude", "codegraph")
        }
        "opencode" => {
            has_array_mcp_entry(&opencode_paths_resolved().config, "codegraph", None)
                && has_owner("opencode", "codegraph")
        }
        "codex" => {
            has_codex_mcp_entry("codegraph", None)
                && has_codex_codegraph_index_hook()
                && has_owner("codex", "codegraph")
        }
        "antigravity" => {
            has_antigravity_mcp("codegraph", None) && has_owner("antigravity", "codegraph")
        }
        "copilot" => {
            has_copilot_mcp("codegraph", None, None)
                && has_copilot_codegraph_index_hook()
                && has_copilot_ide_codegraph_index_hook()
                && has_owner("copilot", "codegraph")
        }
        "droid" => {
            has_mcp_entry_at_file(&droid_mcp_path(), "mcpServers", "codegraph", None)
                && has_owner("droid", "codegraph")
        }
        "pi" => {
            pi_mcp_adapter_ready()
                && has_mcp_entry_at_file(&pi_mcp_path(), "mcpServers", "codegraph", None)
                && has_owner("pi", "codegraph")
        }
        "omp" => {
            has_mcp_entry_at_file(&omp_mcp_path(), "mcpServers", "codegraph", None)
                && has_owner("omp", "codegraph")
        }
        "grok" => {
            has_mcp_entry_at_file(&grok_mcp_path(), "mcpServers", "codegraph", None)
                && has_owner("grok", "codegraph")
        }
        "cursor" => {
            let rules_p = Path::new(&cursor_paths_resolved().rules_dir)
                .join("codegraph.mdc")
                .to_string_lossy()
                .to_string();
            let rules_ok = if let Some(raw) = read_file_safe(&rules_p) {
                let expected = cursor_project_rule_content("codegraph");
                !expected.is_empty() && raw.replace("\r\n", "\n") == expected.replace("\r\n", "\n")
            } else {
                true
            };
            has_mcp_entry_at_file(&cursor_mcp_path(), "mcpServers", "codegraph", None)
                && has_cursor_codegraph_permissions()
                && has_cursor_codegraph_index_hook()
                && rules_ok
        }
        "kilo" => {
            has_array_mcp_entry(&kilo_paths_resolved().config, "codegraph", None)
                && kilo_has_owner("codegraph")
        }
        "cline" => {
            has_mcp_entry_at_file(&cline_paths_resolved().mcp_config, "mcpServers", "codegraph", None)
                && has_owner("cline", "codegraph")
        }
        "opencode2" | "amp" | "kimi" | "zcode" | "dsh" => {
            mcp_has(agent, "codegraph", false) && has_owner(agent, "codegraph")
        }
        _ => false,
    }
}

pub fn has_codegraph_index(dir: &str) -> bool {
    let cg_dir = Path::new(dir).join(".codegraph");
    if !cg_dir.is_dir() {
        return false;
    }
    if let Ok(entries) = fs::read_dir(&cg_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.ends_with(".db") || name.ends_with(".sqlite") || name.ends_with(".json") {
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > 0 {
                        return true;
                    }
                }
            }
        }
    }
    exists(cg_dir.join("codegraph.db")) || exists(cg_dir.join("index.db")) || exists(&cg_dir)
}

pub fn codegraph_index_healthy(dir: &str) -> bool {
    let symbols_db = Path::new(dir).join(".codegraph").join("symbols.db");
    if symbols_db.exists() {
        if let Ok(meta) = fs::metadata(&symbols_db) {
            return meta.len() >= 4096;
        }
    }
    let db = Path::new(dir).join(".codegraph").join("codegraph.db");
    if !db.exists() {
        return has_codegraph_index(dir);
    }
    if let Ok(meta) = fs::metadata(&db) {
        meta.len() >= 4096
    } else {
        false
    }
}

pub fn run_codegraph_index(dir: &str) -> Result<bool, DynError> {
    let mut cg = crate::codegraph::CodeGraph::open(Path::new(dir))?;
    let _ = cg.ensure_indexed()?;
    Ok(true)
}

pub fn rebuild_codegraph_index(dir: &str) -> Result<bool, DynError> {
    let mut cg = crate::codegraph::CodeGraph::open(Path::new(dir))?;
    let _ = cg.index_force()?;
    Ok(true)
}

pub fn ensure_codegraph_index(dir: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if has_codegraph_index(dir) && codegraph_index_healthy(dir) {
        return Ok(true);
    }
    if has_codegraph_index(dir) && !codegraph_index_healthy(dir) {
        return rebuild_codegraph_index(dir);
    }
    run_codegraph_index(dir)
}

pub fn codegraph_index_project(dir: &str, opts: &RunOpts) -> Result<bool, DynError> {
    ensure_codegraph_index(dir, opts)
}

const PI_CODEGRAPH_INDEX_EXT: &str = r#"const TOKLESS_BIN = %TOKLESS%

export default function (pi: any) {
  pi.on("session_start", async () => {
    try {
      await pi.exec(TOKLESS_BIN, ["index", "--codegraph-hook"])
    } catch {}
  })
}
"#;

pub fn write_pi_codegraph_index_extension() {
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return;
    }
    let p = Path::new(&pi_agent_dir_resolved())
        .join("extensions")
        .join("codegraph-index.ts");
    if let Some(parent) = p.parent() {
        let _ = ensure_dir(parent);
    }
    let content = PI_CODEGRAPH_INDEX_EXT.replace("%TOKLESS%", &format!("{:?}", exe));
    let _ = write_file_atomic(p, &content);
}

pub fn write_codegraph_block(agent: &str) -> bool {
    match agent {
        "kilo" => kilo_write_owner("codegraph"),
        "cursor" => {
            let _ = install_cursor_project_rules(".", false);
            true
        }
        _ => write_owner(agent, "codegraph"),
    }
}

pub fn codegraph_wire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    if agent == "copilot" {
        return codegraph_wire_copilot(opts);
    }
    unwire_auto_index(agent);
    codegraph_configure_mcp(agent, opts)?;
    if !write_codegraph_block(agent) {
        return Err(format!("failed to write codegraph instructions for {}", agent).into());
    }
    Ok(codegraph_verify(agent))
}

pub fn codegraph_wire_copilot(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    with_copilot_transaction(&[], || {
        codegraph_configure_mcp("copilot", opts)?;
        install_copilot_codegraph_index_hook_safe()?;
        install_copilot_ide_codegraph_index_hook_safe()?;
        if !write_owner("copilot", "codegraph") {
            return Err("failed to write copilot codegraph instructions".into());
        }
        sync_copilot_ide_instructions_safe()?;
        Ok(codegraph_verify("copilot"))
    })
}

pub fn codegraph_unwire(agent: &str, opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    unwire_auto_index(agent);
    match agent {
        "claude" => {
            remove_json_mcp_entry(&claude_code_paths().global_json, "mcpServers", "codegraph");
            disallow_claude_mcp_server("codegraph");
            remove_owner_safe("claude", "codegraph")?;
        }
        "opencode" => {
            remove_array_mcp_entry(&opencode_paths_resolved().config, "codegraph");
            remove_owner_safe("opencode", "codegraph")?;
        }
        "codex" => {
            remove_codex_mcp_entry("codegraph");
            remove_codex_codegraph_index_hook();
            if !has_codex_mcp_entry("context-mode", None) {
                remove_codex_perm_hook();
            }
            remove_owner_safe("codex", "codegraph")?;
        }
        "antigravity" => {
            remove_antigravity_mcp("codegraph");
            remove_owner_safe("antigravity", "codegraph")?;
        }
        "copilot" => {
            return with_copilot_transaction(&[], || {
                remove_copilot_mcp_safe("codegraph")?;
                remove_copilot_codegraph_index_hook_safe()?;
                remove_copilot_ide_codegraph_index_hook_safe()?;
                remove_owner_safe("copilot", "codegraph")?;
                sync_copilot_ide_instructions_safe()?;
                Ok(true)
            });
        }
        "droid" => {
            remove_json_mcp_entry(&droid_mcp_path(), "mcpServers", "codegraph");
            remove_owner_safe("droid", "codegraph")?;
        }
        "pi" => {
            remove_json_mcp_entry(&pi_mcp_path(), "mcpServers", "codegraph");
            let ext = Path::new(&pi_agent_dir_resolved())
                .join("extensions")
                .join("codegraph-index.ts");
            if ext.exists() {
                let _ = fs::remove_file(ext);
            }
            pi_remove_mcp_adapter();
            remove_owner_safe("pi", "codegraph")?;
        }
        "omp" => {
            remove_json_mcp_entry(&omp_mcp_path(), "mcpServers", "codegraph");
            remove_owner_safe("omp", "codegraph")?;
        }
        "grok" => {
            remove_json_mcp_entry(&grok_mcp_path(), "mcpServers", "codegraph");
            remove_grok_codegraph_session_hook();
            remove_owner_safe("grok", "codegraph")?;
        }
        "cursor" => {
            remove_json_mcp_entry(&cursor_mcp_path(), "mcpServers", "codegraph");
            remove_cursor_codegraph_permissions();
            remove_cursor_codegraph_index_hook();
            let p = Path::new(&cursor_paths_resolved().rules_dir).join("codegraph.mdc");
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }
        "kilo" => {
            remove_array_mcp_entry(&kilo_paths_resolved().config, "codegraph");
            kilo_remove_owner("codegraph");
        }
        "cline" => {
            remove_json_mcp_entry(&cline_paths_resolved().mcp_config, "mcpServers", "codegraph");
            remove_owner_safe("cline", "codegraph")?;
        }
        "opencode2" | "amp" | "kimi" | "zcode" | "dsh" => {
            mcp_remove(agent, "codegraph");
            remove_owner_safe(agent, "codegraph")?;
        }
        _ => {}
    }
    Ok(true)
}

pub fn build_codegraph() -> ToolManifest {
    let mut tool = ToolManifest::new("codegraph", "CodeGraph");
    tool.description = "Native indexed code graph for source, call paths, and impact.".into();
    tool.homepage = "https://github.com/Navegos/neo".into();
    tool.install_hint = "Native embedded in tokless".into();
    tool.channel = Channel::Binary;
    tool.needs_node = false;
    tool.min_node_major = 0;
    tool.install = Some(Arc::new(|opts| codegraph_ensure_installed(opts)));
    tool.index_ready = Some(Arc::new(|| {
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".into());
        has_codegraph_index(&cwd) && codegraph_index_healthy(&cwd)
    }));
    tool.index_project = Some(Arc::new(|dir, opts| codegraph_index_project(dir, opts)));

    let agents = [
        "claude", "opencode", "opencode2", "codex", "antigravity", "copilot", "droid", "grok",
        "pi", "omp", "kilo", "cline", "cursor", "amp", "kimi", "zcode", "dsh",
    ];

    for agent_id in agents {
        let ag = agent_id.to_string();
        let ag_w = ag.clone();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(move |opts| codegraph_wire(&ag_w, opts)),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| codegraph_unwire(&ag_u, opts)),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || {
                Some(codegraph_verify(&ag_v))
            }),
        );

        let ag_a = ag.clone();
        tool.wired_any_for.insert(
            ag.clone(),
            Arc::new(move || Some(mcp_has(&ag_a, "codegraph", true))),
        );
    }

    tool
}
