// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/doctor.rs
// created: 2026-10-05
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::env;
use crate::commands::doctor_probe::{format_runtime_issues, probe_agent_runtime, RuntimeIssue};
use crate::commands::doctor_stats::doctor_tool_stats;
use crate::commands::helpers::*;
use crate::core::{list_agents, list_tools, ToolManifest};
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::progress::*;
use crate::util::selection::is_tool_selected;
use crate::util::versions::*;

#[derive(Clone, serde::Serialize)]
struct AgentReport {
    id: String,
    label: String,
    installed: bool,
    wired: bool,
    missing: Vec<String>,
    tool_rows: Vec<String>,
    #[serde(skip_serializing)]
    runtime: Vec<RuntimeIssue>,
}

fn merge_events_jsonl(src: &std::path::Path, dst: &std::path::Path) {
    let Ok(src_raw) = std::fs::read_to_string(src) else {
        return;
    };
    let dst_raw = std::fs::read_to_string(dst).unwrap_or_default();
    let existing: std::collections::HashSet<&str> =
        dst_raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let mut appended = dst_raw.clone();
    if !appended.is_empty() && !appended.ends_with('\n') {
        appended.push('\n');
    }
    let mut changed = false;
    for line in src_raw.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !existing.contains(trimmed) {
            appended.push_str(trimmed);
            appended.push('\n');
            changed = true;
        }
    }
    if changed {
        let _ = std::fs::write(dst, appended);
    }
}

fn merge_directory_recursive(src: &std::path::Path, dst: &std::path::Path) {
    if !src.is_dir() {
        return;
    }
    let _ = std::fs::create_dir_all(dst);
    let Ok(entries) = std::fs::read_dir(src) else {
        return;
    };
    for entry in entries.flatten() {
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if src_path.is_dir() {
            merge_directory_recursive(&src_path, &dst_path);
        } else if src_path.is_file() {
            let fname_str = file_name.to_string_lossy();
            if fname_str == "events.jsonl" && dst_path.exists() {
                merge_events_jsonl(&src_path, &dst_path);
                let _ = std::fs::remove_file(&src_path);
            } else if !dst_path.exists() {
                if std::fs::rename(&src_path, &dst_path).is_err()
                    && std::fs::copy(&src_path, &dst_path).is_ok()
                {
                    let _ = std::fs::remove_file(&src_path);
                }
            }
        }
    }
}

pub fn consolidate_nested_toksi_dirs(root: &std::path::Path) -> std::io::Result<usize> {
    let mut consolidated_count = 0;
    let root_canon = match std::fs::canonicalize(root) {
        Ok(c) => c,
        Err(_) => root.to_path_buf(),
    };
    let root_toksi = root_canon.join(".toksi");
    if !root_toksi.exists() {
        let _ = std::fs::create_dir_all(&root_toksi);
    }

    let tool_migrations = [
        (".codegraph", "codegraph"),
        (".context-mode", "context"),
        (".context", "context"),
        (".icm", "icm"),
        (".ai-memory", "ai-memory"),
        (".rtk", "rtk"),
        (".projectmem", "projectmem"),
    ];

    // 1. Discover subfolder .toksi and loose tool folders and merge rows
    let mut stack = vec![root_canon.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target"
                || name_str == ".git"
                || name_str == "node_modules"
                || name_str == "dist"
                || name_str == "build"
            {
                continue;
            }
            if path.is_dir() {
                if name_str == ".toksi" {
                    let path_canon = match std::fs::canonicalize(&path) {
                        Ok(c) => c,
                        Err(_) => path.clone(),
                    };
                    if path_canon != root_toksi {
                        // Merge SQLite DBs row-by-row before deleting!
                        // a) toksi.db
                        let sub_toksi_db = path.join("toksi.db");
                        if sub_toksi_db.exists() {
                            let root_toksi_db = root_toksi.join("toksi.db");
                            let _ = crate::db::merge_toksi_db(&root_toksi_db, &sub_toksi_db);
                        }
                        // b) memory.db (in sub/.toksi, sub/.toksi/icm, or sub/.toksi/projectmem)
                        for (mem_sub, dst_rel) in [
                            (path.join("memory.db"), root_toksi.join("icm").join("memory.db")),
                            (path.join("icm").join("memory.db"), root_toksi.join("icm").join("memory.db")),
                            (path.join("projectmem").join("memory.db"), root_toksi.join("projectmem").join("memory.db")),
                        ] {
                            if mem_sub.exists() {
                                let _ = crate::db::merge_memory_db(&dst_rel, &mem_sub);
                            }
                        }
                        // c) symbols.db (in sub/.toksi or sub/.toksi/codegraph)
                        for sym_sub in [path.join("symbols.db"), path.join("codegraph").join("symbols.db")] {
                            if sym_sub.exists() {
                                let root_sym = root_toksi.join("codegraph").join("symbols.db");
                                let _ = crate::db::merge_symbols_db(&root_sym, &sym_sub);
                            }
                        }

                        // Recursively merge subdirectories and files into root .toksi
                        merge_directory_recursive(&path, &root_toksi);
                        let _ = std::fs::remove_dir_all(&path);
                        consolidated_count += 1;
                    }
                } else if let Some((_, tool_name)) =
                    tool_migrations.iter().find(|(legacy, _)| *legacy == name_str.as_ref())
                {
                    let dir_canon = match std::fs::canonicalize(&dir) {
                        Ok(c) => c,
                        Err(_) => dir.clone(),
                    };
                    if dir_canon != root_canon {
                        let target_tool_dir = root_toksi.join(tool_name);
                        let _ = std::fs::create_dir_all(&target_tool_dir);
                        if *tool_name == "codegraph" {
                            let sub_sym = path.join("symbols.db");
                            if sub_sym.exists() {
                                let _ = crate::db::merge_symbols_db(&target_tool_dir.join("symbols.db"), &sub_sym);
                            }
                        } else if *tool_name == "icm" || *tool_name == "projectmem" {
                            let sub_mem = path.join("memory.db");
                            if sub_mem.exists() {
                                let _ = crate::db::merge_memory_db(&target_tool_dir.join("memory.db"), &sub_mem);
                            }
                        }
                        merge_directory_recursive(&path, &target_tool_dir);
                        let _ = std::fs::remove_dir_all(&path);
                        consolidated_count += 1;
                    }
                } else {
                    stack.push(path);
                }
            }
        }
    }

    // 2. Consolidate loose legacy tool folders at workspace root into .toksi/<tool>
    for (legacy_name, tool_name) in &tool_migrations[..6] {
        let legacy_path = root_canon.join(legacy_name);
        if legacy_path.exists() && legacy_path.is_dir() {
            let target_tool_dir = root_toksi.join(tool_name);
            let _ = std::fs::create_dir_all(&target_tool_dir);
            if *legacy_name == ".codegraph" {
                let sub_sym = legacy_path.join("symbols.db");
                if sub_sym.exists() {
                    let root_sym = target_tool_dir.join("symbols.db");
                    let _ = crate::db::merge_symbols_db(&root_sym, &sub_sym);
                }
            } else if *legacy_name == ".icm" {
                let sub_mem = legacy_path.join("memory.db");
                if sub_mem.exists() {
                    let root_mem = target_tool_dir.join("memory.db");
                    let _ = crate::db::merge_memory_db(&root_mem, &sub_mem);
                }
            }
            merge_directory_recursive(&legacy_path, &target_tool_dir);
            let _ = std::fs::remove_dir_all(&legacy_path);
            consolidated_count += 1;
        }
    }

    // Mirror/migrate loose .projectmem into .toksi/projectmem
    let legacy_pjm = root_canon.join(".projectmem");
    if legacy_pjm.exists() && legacy_pjm.is_dir() {
        let target_pjm = root_toksi.join("projectmem");
        let _ = std::fs::create_dir_all(&target_pjm);
        if let Ok(entries) = std::fs::read_dir(&legacy_pjm) {
            for entry in entries.flatten() {
                let dest = target_pjm.join(entry.file_name());
                if !dest.exists() {
                    let _ = std::fs::copy(entry.path(), dest);
                }
            }
        }
    }

    Ok(consolidated_count)
}

pub fn audit_nested_agent_dirs(root: &std::path::Path) -> Vec<String> {
    let mut warnings = Vec::new();
    let root_canon = match std::fs::canonicalize(root) {
        Ok(c) => c,
        Err(_) => root.to_path_buf(),
    };
    let agent_dirs = [".claude", ".cursor", ".vscode", ".grok", ".cline", ".agents", ".gemini", ".github"];

    let mut stack = vec![root_canon.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str == ".git" || name_str == "node_modules" || name_str == "dist" || name_str == "build" {
                continue;
            }
            if path.is_dir() {
                if agent_dirs.contains(&name_str.as_ref()) {
                    let path_canon = match std::fs::canonicalize(&path) {
                        Ok(c) => c,
                        Err(_) => path.clone(),
                    };
                    if path_canon.parent() != Some(&root_canon) {
                        warnings.push(format!(
                            "[WARNING] Nested agent config directory found: {}. Agent configs must strictly reside ONLY in workspace root.",
                            crate::util::paths::normalize_path(&path.to_string_lossy())
                        ));
                    }
                } else {
                    stack.push(path);
                }
            }
        }
    }
    warnings
}

pub fn consolidate_nested_agent_dirs(root: &std::path::Path) -> std::io::Result<usize> {
    let mut count = 0;
    let root_canon = match std::fs::canonicalize(root) {
        Ok(c) => c,
        Err(_) => root.to_path_buf(),
    };
    let agent_dirs = [".claude", ".cursor", ".vscode", ".grok", ".cline", ".agents", ".gemini", ".github"];

    let mut stack = vec![root_canon.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str == ".git" || name_str == "node_modules" || name_str == "dist" || name_str == "build" {
                continue;
            }
            if path.is_dir() {
                if agent_dirs.contains(&name_str.as_ref()) {
                    let path_canon = match std::fs::canonicalize(&path) {
                        Ok(c) => c,
                        Err(_) => path.clone(),
                    };
                    if path_canon.parent() != Some(&root_canon) {
                        let _ = crate::agents::manager::migrate_legacy_mcp_dirs_to_skills(&path);
                        let root_dest = root_canon.join(&name);
                        merge_directory_recursive(&path, &root_dest);
                        let _ = std::fs::remove_dir_all(&path);
                        count += 1;
                    }
                } else {
                    stack.push(path);
                }
            }
        }
    }
    let migrated_root = crate::agents::manager::migrate_legacy_mcp_dirs_to_skills(&root_canon);
    count += migrated_root.len();
    Ok(count)
}

pub fn audit_agent_cli_binaries() -> Vec<String> {
    let mut warnings = Vec::new();
    let all_agents = list_agents();
    for agent in &all_agents {
        if !agent.detect().installed {
            continue;
        }
        let cli = &agent.cli_bin;
        if cli.is_empty() {
            continue;
        }
        if let Err(e) = crate::util::env_path::check_executable_in_path(cli) {
            warnings.push(e);
        }
    }
    warnings
}

pub fn run_doctor(offline: bool, json_output: bool, fix: bool) -> i32 {
    let all_agents = list_agents();
    let tools = list_tools();

    if fix {
        if let Ok(merged) = consolidate_nested_toksi_dirs(std::path::Path::new(".")) {
            if merged > 0 {
                L.raw(&format!(
                    "  {} Consolidated {} nested .toksi directory/directories into root workspace .toksi",
                    C.green(Sym::CHECK),
                    merged
                ));
            }
        }
        if let Ok(merged_agents) = consolidate_nested_agent_dirs(std::path::Path::new(".")) {
            if merged_agents > 0 {
                L.raw(&format!(
                    "  {} Consolidated {} nested agent config directory/directories into workspace root",
                    C.green(Sym::CHECK),
                    merged_agents
                ));
            }
        }
        let healing = crate::env_injector::heal_legacy_hijacks();
        if !healing.is_clean() {
            L.raw(&format!(
                "  {} Healed legacy binary hijacks: restored {} binaries, removed {} shims, cleaned {} state files",
                C.green(Sym::CHECK),
                healing.restored_binaries.len(),
                healing.removed_shims.len(),
                healing.removed_state_files.len()
            ));
        } else {
            L.raw(&format!(
                "  {} Legacy binary hijacking: clean (no .real binaries or shims found)",
                C.green(Sym::CHECK)
            ));
        }

        let config_healed = crate::agents::manager::heal_all_agent_legacy_configs(Some(std::path::Path::new(".")));
        if !config_healed.is_empty() {
            L.raw(&format!(
                "  {} Healed {} legacy agent MCP/hook configuration item(s)",
                C.green(Sym::CHECK),
                config_healed.len()
            ));
        }

        let mut fixed_count = config_healed.len();
        #[cfg(windows)]
        {
            if let Some(_) = crate::env_injector::windows::get_env_reg_value("CLOUD_CODE_URL") {
                let _ = crate::env_injector::windows::delete_env_reg_value("CLOUD_CODE_URL");
                crate::env_injector::windows::broadcast_setting_change();
                L.raw(&format!("  {} Cleared stale CLOUD_CODE_URL from Windows registry", C.green(Sym::CHECK)));
                fixed_count += 1;
            }
        }

        if let Ok(report) = crate::rules::sync_workspace_rules(".", crate::rules::RuleKind::all()) {
            if !report.updated_files.is_empty() {
                L.raw(&format!(
                    "  {} Synced in-tree rules across {} workspace file(s)",
                    C.green(Sym::CHECK),
                    report.updated_files.len()
                ));
                fixed_count += report.updated_files.len();
            }
        }
        if let Ok(report) = crate::rules::sync_global_rules(crate::rules::RuleKind::all()) {
            if !report.updated_files.is_empty() {
                L.raw(&format!(
                    "  {} Synced in-tree rules across {} global config file(s)",
                    C.green(Sym::CHECK),
                    report.updated_files.len()
                ));
                fixed_count += report.updated_files.len();
            }
        }

        for agent in &all_agents {
            if !agent.detect().installed {
                continue;
            }
            let purged = crate::agents::purge_legacy_mcp_runners(&agent.id);
            if !purged.is_empty() {
                L.raw(&format!(
                    "  {} Purged broken legacy MCP runners for {}: {}",
                    C.green(Sym::CHECK),
                    agent.label,
                    purged.join(", ")
                ));
                fixed_count += purged.len();
            }
            for tool in &tools {
                if !is_tool_selected(&tool.id) {
                    continue;
                }
                if let Some(ref verify) = tool.verify_for.get(&agent.id) {
                    if verify() == Some(false) {
                        if let Some(ref wire) = tool.wire_for.get(&agent.id) {
                            let ro = crate::core::RunOpts {
                                dry_run: false,
                                agent: agent.id.clone(),
                                ..Default::default()
                            };
                            if let Ok(true) = wire(&ro) {
                                fixed_count += 1;
                            }
                        }
                    }
                }
            }
        }
        if fixed_count > 0 {
            L.raw(&format!(
                "  {} Repaired agent settings (in-tree rules and in-process native MCP)",
                C.green(Sym::CHECK)
            ));
        }
        L.raw("");
    }
    let mut reports_opt: Vec<Option<AgentReport>> = vec![None; all_agents.len()];

    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(all_agents.len());
        for agent in &all_agents {
            let tools_ref = &tools;
            handles.push(s.spawn(move || {
                if !agent.detect().installed {
                    return None;
                }
                let missing = doctor_missing_tools(tools_ref, &agent.id);
                let runtime = probe_agent_runtime(&agent.id);
                let wired = missing.is_empty() && runtime.is_empty();
                let mut tool_rows = Vec::new();
                for t in tools_ref {
                    if !is_tool_selected(&t.id) {
                        continue;
                    }
                    if let Some(ref verify) = t.verify_for.get(&agent.id) {
                        let ok = verify().unwrap_or(false);
                        let mark = if ok { C.green(Sym::CHECK) } else { C.red("x") };
                        tool_rows.push(format!("  {} {}", mark, t.label));
                    }
                }
                Some(AgentReport {
                    id: agent.id.clone(),
                    label: agent.label.clone(),
                    installed: true,
                    wired,
                    missing,
                    tool_rows,
                    runtime,
                })
            }));
        }
        for (i, handle) in handles.into_iter().enumerate() {
            if let Ok(res) = handle.join() {
                reports_opt[i] = res;
            }
        }
    });

    let reports: Vec<AgentReport> = reports_opt.into_iter().flatten().collect();
    let cwd = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into());
    let tool_stats = doctor_tool_stats(&cwd, true);

    if json_output {
        #[derive(serde::Serialize)]
        struct DoctorJson<'a> {
            agents: &'a [AgentReport],
            stats: &'a std::collections::HashMap<String, Vec<String>>,
        }
        let doc = DoctorJson {
            agents: &reports,
            stats: &tool_stats,
        };
        if let Ok(out) = serde_json::to_string_pretty(&doc) {
            println!("{}", out);
            return 0;
        }
    }

    cmd_header("doctor", "quick health check");

    tree_top("Agents");
    if reports.is_empty() {
        tree_leaf(&C.dim("none detected on this machine"));
    } else {
        for r in &reports {
            tree_leaf(&doctor_summary_line(r));
            if !r.wired {
                for tr in &r.tool_rows {
                    tree_leaf(&C.dim(tr));
                }
            }
        }
    }
    tree_close();

    let port_rows = doctor_proxy_ports(&reports);
    if !port_rows.is_empty() {
        tree_corner("Ports");
        for p in port_rows {
            tree_leaf(&p);
        }
        tree_close();
    }

    let env_rows = doctor_env_report();
    if !env_rows.is_empty() {
        tree_corner("Environment");
        for e in env_rows {
            tree_leaf(&e);
        }
        tree_close();
    }

    let rules_rows = doctor_rules_report(&cwd);
    if !rules_rows.is_empty() {
        tree_corner("Rules");
        for r in rules_rows {
            tree_leaf(&r);
        }
        tree_close();
    }

    let cli_warnings = audit_agent_cli_binaries();
    if !cli_warnings.is_empty() {
        tree_corner("PATH Validation");
        for w in cli_warnings {
            for line in w.lines() {
                tree_leaf(&C.yellow(line));
            }
        }
        tree_close();
    }

    let broken = reports.iter().filter(|r| !r.wired).count();
    let installed_agents = reports.len();

    let mut outdated = -1;
    if !offline && env::var("TOKSI_TEST").as_deref() != Ok("1") {
        let v = gather_versions();
        outdated = count_outdated(&v) as i32;

        tree_corner("Tools");
        for tool in &tools {
            if tool.instruction_only {
                continue;
            }
            if let Some(info) = v.get(&tool.id) {
                tree_leaf(&tool_version_display_line(&tool.id, Some(info), false));
            }
        }
        tree_close();
    }

    tree_corner("Stats");
    if tool_stats.is_empty() {
        tree_leaf(&C.dim("no installed tools with local stats"));
    } else {
        let tool_ids = ["headroom", "projectmem", "rtk", "codegraph", "context-mode"];
        for id in tool_ids {
            if let Some(rows) = tool_stats.get(id) {
                tree_leaf(&format!("{}:", paint_name(id)));
                for row in rows {
                    tree_leaf(&format!("  {}", row));
                }
            }
        }
    }
    tree_close();

    let status_lines = doctor_status_lines(outdated, broken, installed_agents);
    tree_status(&status_lines);
    print_repo_footer(true);
    L.raw("");
    0
}

fn doctor_proxy_ports(reports: &[AgentReport]) -> Vec<String> {
    let mut ports = Vec::new();
    let main_port = crate::util::headroom_proxy::headroom_proxy_port();
    let main_running = if env::var("TOKSI_TEST").as_deref() == Ok("1") { false } else { crate::headroom::proxy_running() };
    let main_status = if main_running {
        C.green("running")
    } else {
        C.gray("stopped")
    };
    ports.push(format!(
        "{} Headroom proxy (127.0.0.1:{}) — {}",
        if main_running { C.green(Sym::CHECK) } else { C.gray(Sym::BULLET) },
        main_port,
        main_status
    ));

    let byok_routes = crate::util::byok::read_byok_routes().unwrap_or_default();
    if !byok_routes.is_empty() {
        let gw_port = crate::util::byok::byok_gateway_port();
        ports.push(format!(
            "{} BYOK gateway (127.0.0.1:{}) — {}",
            C.green(Sym::CHECK),
            gw_port,
            C.green(&format!("{} routes active", byok_routes.len()))
        ));
    }

    // Keyring status
    let backend = crate::keyring::current_backend_name();
    let active_keys = if env::var("TOKSI_TEST").as_deref() == Ok("1") {
        0
    } else {
        let statuses = crate::keyring::audit_keyring_statuses();
        statuses.iter().filter(|s| s.has_credential && !s.is_expired).count()
    };
    ports.push(format!(
        "{} OS Keyring ({}) — {}",
        C.green(Sym::CHECK),
        backend,
        if active_keys > 0 { C.green(&format!("{} credentials active", active_keys)) } else { C.gray("ready") }
    ));

    // SQLite config database status
    if let Ok(db) = crate::db::open_default_db() {
        if let Ok(stats) = db.stats() {
            ports.push(format!(
                "{} Config DB (SQLite WAL) — {}",
                C.green(Sym::CHECK),
                C.green(&format!("{} routes, {} providers", stats.routes_count, stats.providers_count))
            ));
        }
    }

    for r in reports {
        if r.id == "copilot" && crate::agents::copilot::copilot_proxy_wired() {
            ports.push(format!(
                "{} Copilot proxy (127.0.0.1:{}) — {}",
                C.green(Sym::CHECK),
                crate::agents::copilot::copilot_proxy_port(),
                C.green("wired")
            ));
        }
        if r.id == "grok" && crate::agents::grok::grok_oauth_proxy_wired() {
            ports.push(format!(
                "{} Grok OAuth proxy (127.0.0.1:{}) — {}",
                C.green(Sym::CHECK),
                crate::util::grok::grok_oauth_proxy_port(),
                C.green("wired")
            ));
        }
    }
    ports
}

fn doctor_missing_tools(tools: &[std::sync::Arc<ToolManifest>], agent_id: &str) -> Vec<String> {
    let mut missing = Vec::new();
    for tool in tools {
        if !is_tool_selected(&tool.id) {
            continue;
        }
        if let Some(ref verify) = tool.verify_for.get(agent_id) {
            if let Some(false) = verify() {
                missing.push(tool.label.clone());
            }
        }
    }
    missing
}

fn doctor_env_report() -> Vec<String> {
    let mut rows = Vec::new();
    let env_status = crate::env_injector::audit_environment_status();

    if env_status.binaries_clean {
        rows.push(format!(
            "{} Binaries: clean (no .real hijacked binaries, shims, or orphaned state files)",
            C.green(Sym::CHECK)
        ));
    } else {
        rows.push(format!(
            "{} Binaries: hijacked binaries or shims detected (run {})",
            C.yellow(Sym::WARN),
            paint_cmd("toksi doctor --fix")
        ));
        for b in &env_status.hijacked_binaries {
            rows.push(format!("  {} hijacked: {}", C.red("x"), b));
        }
        for s in &env_status.shims_detected {
            rows.push(format!("  {} shim: {}", C.red("x"), s));
        }
    }

    if env_status.env_vars_healthy {
        let count = env_status.active_proxy_vars.len();
        if count > 0 {
            rows.push(format!(
                "{} Environment: healthy ({} proxy variables active)",
                C.green(Sym::CHECK),
                count
            ));
        } else {
            rows.push(format!(
                "{} Environment: healthy (default OS state, no proxy overrides active)",
                C.green(Sym::CHECK)
            ));
        }
    } else {
        rows.push(format!(
            "{} Environment: stale or orphaned proxy variables detected (run {})",
            C.yellow(Sym::WARN),
            paint_cmd("toksi doctor --fix")
        ));
        for d in &env_status.details {
            rows.push(format!("  {} {}", C.yellow(Sym::WARN), d));
        }
    }

    let process_proxies: Vec<String> = [
        "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY",
        "http_proxy", "https_proxy", "all_proxy",
        "GOOGLE_GEMINI_BASE_URL", "ANTHROPIC_BASE_URL", "CLOUD_CODE_URL"
    ]
    .iter()
    .filter_map(|&k| std::env::var(k).ok().map(|v| format!("{}={}", k, v)))
    .collect();

    if !process_proxies.is_empty() {
        rows.push(format!(
            "{} Process proxies: active ({})",
            C.cyan(Sym::BULLET),
            process_proxies.join(", ")
        ));
    }

    rows
}

fn doctor_rules_report(workspace: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let ws = std::path::Path::new(workspace);
    let rule_files = [
        ("CLAUDE.md", "Claude Code"),
        ("AGENTS.md", "OpenCode / Agents"),
        ("GEMINI.md", "Gemini / Antigravity"),
        (".github/copilot-instructions.md", "GitHub Copilot"),
        (".clinerules", "Cline"),
        ("CODEX.md", "Codex"),
        (".cursorrules", "Cursor"),
    ];

    for (file, label) in rule_files {
        let p = ws.join(file);
        if p.exists() {
            let raw = crate::util::paths::read_file_safe(&p).unwrap_or_default();
            let has_principles = raw.contains("## Principles");
            let has_caveman = raw.contains("## Response Style (caveman)");
            let has_ponytail = raw.contains("## Build Discipline (ponytail)");
            if has_principles && has_caveman && has_ponytail {
                rows.push(format!("{} {} ({}) — in-tree rules synced", C.green(Sym::CHECK), file, label));
            } else {
                rows.push(format!("{} {} ({}) — partial/missing in-tree rules", C.yellow(Sym::WARN), file, label));
            }
        }
    }

    let cursor_rules_dir = ws.join(".cursor").join("rules");
    if cursor_rules_dir.is_dir() {
        let principles_ok = cursor_rules_dir.join("principles.mdc").exists();
        let caveman_ok = cursor_rules_dir.join("response-style.mdc").exists();
        let ponytail_ok = cursor_rules_dir.join("build-discipline.mdc").exists();
        if principles_ok && caveman_ok && ponytail_ok {
            rows.push(format!("{} .cursor/rules/*.mdc (Cursor) — in-tree rules synced", C.green(Sym::CHECK)));
        } else {
            rows.push(format!("{} .cursor/rules/*.mdc (Cursor) — missing in-tree rule files", C.yellow(Sym::WARN)));
        }
    }
    rows
}

fn doctor_summary_line(r: &AgentReport) -> String {
    let name = paint_name(&pad_end(&r.label, 16));
    if !r.installed {
        return format!(
            "{}{} {}",
            C.gray(&format!("{} ", Sym::BULLET)),
            C.dim(&pad_end(&r.label, 16)),
            C.gray("not installed")
        );
    }
    if r.wired {
        format!("{} {} {}", C.green(Sym::CHECK), name, C.green("all tools wired"))
    } else if !r.runtime.is_empty() && r.missing.is_empty() {
        format!(
            "{} {} {}",
            C.yellow(Sym::WARN),
            name,
            C.yellow(&format!("runtime: {}", format_runtime_issues(&r.runtime)))
        )
    } else if !r.runtime.is_empty() {
        format!(
            "{} {} {}",
            C.yellow(Sym::WARN),
            name,
            C.yellow(&format!(
                "missing: {}; runtime: {}",
                r.missing.join(", "),
                format_runtime_issues(&r.runtime)
            ))
        )
    } else {
        format!(
            "{} {} {}",
            C.yellow(Sym::WARN),
            name,
            C.yellow(&format!("missing: {}", r.missing.join(", ")))
        )
    }
}

fn doctor_status_lines(outdated: i32, broken: usize, installed_agents: usize) -> Vec<String> {
    let tools_ok = outdated == 0;
    let agents_ok = broken == 0;
    let online = outdated >= 0;

    if online && tools_ok && agents_ok {
        return vec![status_ok("Everything up to date.")];
    }

    let mut lines = Vec::new();
    if online {
        if outdated > 0 {
            lines.push(status_kv(
                &C.yellow(Sym::WARN),
                "tools",
                &format!(
                    "{} — run {}",
                    C.yellow(&format!("{} available", plural(outdated as usize))),
                    paint_cmd("toksi update")
                ),
            ));
        } else {
            lines.push(status_kv(
                &C.green(Sym::CHECK),
                "tools",
                &C.green("all up to date"),
            ));
        }
    }

    if broken > 0 {
        lines.push(status_kv(
            &C.yellow(Sym::WARN),
            "agents",
            &format!(
                "{} — run {}",
                C.yellow(&format!("{} incomplete", broken)),
                paint_cmd("toksi")
            ),
        ));
    } else if installed_agents > 0 {
        lines.push(status_kv(
            &C.green(Sym::CHECK),
            "agents",
            &C.green("all wired"),
        ));
    } else {
        lines.push(status_kv(
            &C.gray(Sym::BULLET),
            "agents",
            &C.dim("none installed"),
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_rules_report_on_empty_dir() {
        let rows = doctor_rules_report(".");
        // Empty directory or project directory returns valid formatted lines without panicking
        for r in rows {
            assert!(!r.is_empty());
        }
    }

    #[test]
    fn test_doctor_env_report_runs() {
        let rows = doctor_env_report();
        assert!(!rows.is_empty());
    }
}
