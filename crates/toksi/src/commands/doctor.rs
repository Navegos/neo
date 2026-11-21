// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/doctor.rs
// created: 2026-10-05
// lastModified: 2026-11-18

#![deny(unsafe_code)]

use std::env;
use std::path::PathBuf;
use crate::agents::manager::AgentManager;
use crate::commands::doctor_probe::{format_runtime_issues, probe_agent_runtime, RuntimeIssue};
use crate::commands::doctor_stats::doctor_tool_stats;
use crate::commands::helpers::*;
use crate::core::{list_agents, list_tools, ToolManifest};
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::paths::normalize_path;
use crate::util::progress::*;
use crate::util::selection::is_tool_selected;
use crate::util::versions::*;

#[derive(Debug, Clone, Default)]
pub struct DoctorOptions {
    pub offline: bool,
    pub json_output: bool,
    pub fix: bool,
    pub workspace: Option<PathBuf>,
    pub include_workspace: bool,
}

#[derive(Clone, serde::Serialize)]
struct AgentReport {
    id: String,
    label: String,
    installed: bool,
    wired: bool,
    wire_mode: String,
    layout_category: String,
    config_dir: String,
    cli_in_path: bool,
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

const HOME_WORKSPACE_LEAK_DIRS: &[&str] = &[
    "projectmem",
    "icm",
    "ai-memory",
    "aimemory",
    "context",
    "context-mode",
    "codegraph",
];

pub fn audit_home_workspace_leakage() -> Vec<String> {
    let h = crate::util::paths::home();
    if h.is_empty() {
        return Vec::new();
    }
    let home_toksi = std::path::Path::new(&h).join(".toksi");
    let mut warnings = Vec::new();
    for tool in HOME_WORKSPACE_LEAK_DIRS {
        let leaked = home_toksi.join(tool);
        if leaked.is_dir() {
            warnings.push(format!(
                "[WARNING] Home-dir workspace store leakage detected at {}; workspace tools belong strictly in <repo>/.toksi/<tool>.",
                normalize_path(&leaked.to_string_lossy())
            ));
        }
    }
    warnings
}

pub fn clean_home_workspace_leakage() -> std::io::Result<usize> {
    let h = crate::util::paths::home();
    if h.is_empty() {
        return Ok(0);
    }
    let home_toksi = std::path::Path::new(&h).join(".toksi");
    let mut removed = 0usize;
    for tool in HOME_WORKSPACE_LEAK_DIRS {
        let leaked = home_toksi.join(tool);
        if leaked.is_dir() && std::fs::remove_dir_all(&leaked).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

fn subproject_id_for(root_canon: &std::path::Path, dir: &std::path::Path) -> String {
    let dir_canon = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if dir_canon == *root_canon {
        return ".".to_string();
    }
    dir_canon
        .strip_prefix(root_canon)
        .map(|rel| normalize_path(&rel.to_string_lossy()))
        .unwrap_or_else(|_| ".".to_string())
}

pub fn consolidate_nested_toksi_dirs(root: &std::path::Path) -> std::io::Result<usize> {
    let mut consolidated_count = 0;
    let root_canon = match std::fs::canonicalize(root) {
        Ok(c) => c,
        Err(_) => root.to_path_buf(),
    };
    let root_toksi = root_canon.join(".toksi");

    let tool_migrations = [
        (".codegraph", "codegraph"),
        (".context-mode", "context"),
        (".context", "context"),
        (".icm", "icm"),
        (".ai-memory", "ai-memory"),
        (".aimemory", "ai-memory"),
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
            if matches!(
                name_str.as_ref(),
                "target" | ".git" | "node_modules" | "dist" | "build" | "inspect-vault" | ".venv" | "venv" | "__pycache__" | "vendor"
            ) {
                continue;
            }
            if path.is_dir() {
                if name_str == ".toksi" {
                    let path_canon = match std::fs::canonicalize(&path) {
                        Ok(c) => c,
                        Err(_) => path.clone(),
                    };
                    if path_canon != root_toksi {
                        let _ = std::fs::create_dir_all(&root_toksi);
                        let sub_id = subproject_id_for(&root_canon, &dir);

                        // Merge SQLite DBs row-by-row tagged with subproject_id before deleting!
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
                                let _ = crate::db::merge_memory_db_with_subproject(&dst_rel, &mem_sub, &sub_id);
                            }
                        }
                        // c) symbols.db (in sub/.toksi or sub/.toksi/codegraph)
                        for sym_sub in [path.join("symbols.db"), path.join("codegraph").join("symbols.db")] {
                            if sym_sub.exists() {
                                let root_sym = root_toksi.join("codegraph").join("symbols.db");
                                let _ = crate::db::merge_symbols_db_with_subproject(&root_sym, &sym_sub, &sub_id);
                            }
                        }

                        // Normalize legacy .toksi/aimemory -> .toksi/ai-memory before directory merge
                        let nested_aimem = path.join("aimemory");
                        if nested_aimem.is_dir() {
                            merge_directory_recursive(&nested_aimem, &root_toksi.join("ai-memory"));
                            let _ = std::fs::remove_dir_all(&nested_aimem);
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
                        let _ = std::fs::create_dir_all(&root_toksi);
                        let sub_id = subproject_id_for(&root_canon, &dir);
                        let target_tool_dir = root_toksi.join(tool_name);
                        let _ = std::fs::create_dir_all(&target_tool_dir);
                        if *tool_name == "codegraph" {
                            let sub_sym = path.join("symbols.db");
                            if sub_sym.exists() {
                                let _ = crate::db::merge_symbols_db_with_subproject(
                                    &target_tool_dir.join("symbols.db"),
                                    &sub_sym,
                                    &sub_id,
                                );
                            }
                        } else if *tool_name == "icm" || *tool_name == "projectmem" {
                            let sub_mem = path.join("memory.db");
                            if sub_mem.exists() {
                                let _ = crate::db::merge_memory_db_with_subproject(
                                    &target_tool_dir.join("memory.db"),
                                    &sub_mem,
                                    &sub_id,
                                );
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
    for (legacy_name, tool_name) in tool_migrations {
        let legacy_path = root_canon.join(legacy_name);
        if legacy_path.exists() && legacy_path.is_dir() {
            let _ = std::fs::create_dir_all(&root_toksi);
            let target_tool_dir = root_toksi.join(tool_name);
            let _ = std::fs::create_dir_all(&target_tool_dir);
            if legacy_name == ".codegraph" {
                let sub_sym = legacy_path.join("symbols.db");
                if sub_sym.exists() {
                    let root_sym = target_tool_dir.join("symbols.db");
                    let _ = crate::db::merge_symbols_db(&root_sym, &sub_sym);
                }
            } else if legacy_name == ".icm" || legacy_name == ".projectmem" {
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

    // Migrate legacy root .toksi/aimemory -> .toksi/ai-memory if present
    let legacy_aimem = root_toksi.join("aimemory");
    if legacy_aimem.is_dir() {
        let target_aimem = root_toksi.join("ai-memory");
        merge_directory_recursive(&legacy_aimem, &target_aimem);
        let _ = std::fs::remove_dir_all(&legacy_aimem);
        consolidated_count += 1;
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
                        if let Some(parent) = path.parent() {
                            let _ = crate::agents::manager::migrate_legacy_mcp_dirs_to_skills(parent);
                        }
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

pub fn audit_toksi_config_toml() -> Vec<String> {
    let mut warnings = Vec::new();
    let cfg_path = crate::util::ports::toksi_config_toml_path();
    let cfg = if cfg_path.exists() {
        match std::fs::read_to_string(&cfg_path) {
            Ok(raw) => match toml::from_str::<crate::util::ports::ToksiTomlConfig>(&raw) {
                Ok(parsed) => parsed,
                Err(e) => {
                    warnings.push(format!(
                        "Malformed {}: {}",
                        normalize_path(&cfg_path.to_string_lossy()),
                        e
                    ));
                    return warnings;
                }
            },
            Err(e) => {
                warnings.push(format!(
                    "Unreadable {}: {}",
                    normalize_path(&cfg_path.to_string_lossy()),
                    e
                ));
                return warnings;
            }
        }
    } else {
        crate::util::ports::ToksiTomlConfig::default()
    };

    let p = &cfg.ports;
    let port_entries = p.all_ports();
    let mut seen_ports: std::collections::HashMap<u16, &str> = std::collections::HashMap::new();
    for (name, port) in port_entries {
        if port < 1024 {
            warnings.push(format!(
                "config.toml [ports].{}={} is below privileged port 1024",
                name, port
            ));
        }
        if let Some(prev) = seen_ports.insert(port, name) {
            warnings.push(format!(
                "config.toml [ports] port collision on {}: {} conflicts with {}",
                port, name, prev
            ));
        }
    }

    for (provider, url) in &cfg.base_urls {
        let trimmed = url.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("http://")
            && !trimmed.starts_with("https://")
        {
            warnings.push(format!(
                "config.toml [base_urls].{}='{}' must begin with http:// or https://",
                provider, trimmed
            ));
        }
    }

    warnings
}

/// Detect any loose legacy dot-folders (`.projectmem`, `.codegraph`, `.context-mode`,
/// `.icm`, `.ai-memory`, `.rtk`) or nested `.toksi` directories outside `<repo>/.toksi/`.
pub fn detect_loose_dot_folders(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let root_canon = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let root_toksi = root_canon.join(".toksi");
    const LOOSE_NAMES: &[&str] = &[
        ".projectmem",
        ".codegraph",
        ".context-mode",
        ".context",
        ".icm",
        ".ai-memory",
        ".aimemory",
        ".rtk",
    ];
    let mut stack = vec![root_canon];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if matches!(
                name.as_str(),
                ".git" | "target" | "node_modules" | "dist" | "build" | "inspect-vault" | ".venv" | "venv" | "__pycache__" | "vendor"
            ) {
                continue;
            }
            if name == ".toksi" {
                let path_canon = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                if path_canon != root_toksi {
                    found.push(path);
                }
                continue;
            }
            if LOOSE_NAMES.contains(&name.as_str()) {
                found.push(path);
                continue;
            }
            stack.push(path);
        }
    }
    found.sort();
    found
}

pub fn audit_externalized_skills_and_rules(ws: Option<&std::path::Path>) -> Vec<String> {
    let mut rows = Vec::new();
    let cfg = crate::rules::skills_repo::SkillsRepoConfig::load();

    // 1. Verify externalized <enforce> rules (Karpathy, Caveman v4, Ponytail)
    for id in ["principles", "caveman", "ponytail"] {
        let content = crate::rules::skills_repo::read_rule_markdown(id);
        if content.trim().is_empty() {
            rows.push(format!(
                "{} externalized rule '{}.md' missing from {}",
                C.yellow(Sym::WARN),
                id,
                normalize_path(&cfg.stage_rules_dir.to_string_lossy())
            ));
        } else if crate::rules::has_enforce_guard(&content) {
            rows.push(format!(
                "{} externalized rule '{}.md' (<enforce> verified)",
                C.green(Sym::CHECK),
                id
            ));
        } else {
            rows.push(format!(
                "{} externalized rule '{}.md' present (no <enforce> guard)",
                C.cyan(Sym::BULLET),
                id
            ));
        }
    }

    // 2. Run skilldoctor + agentskills YAML spec validation on externalized stage & user skills
    for (label, dir) in [
        ("staged agents/skills", cfg.stage_agents_dir.as_path()),
        ("user skills (~/.toksi/user-skills)", cfg.user_skills_dir.as_path()),
    ] {
        if !dir.is_dir() {
            continue;
        }
        if let Ok(report) = crate::rules::skill_validator::analyze_directory(dir) {
            if report.files.is_empty() {
                continue;
            }
            if report.error_count == 0 {
                rows.push(format!(
                    "{} skilldoctor ({}): {} files, score {}/100 ({:?})",
                    C.green(Sym::CHECK),
                    label,
                    report.files.len(),
                    report.score,
                    report.grade
                ));
            } else {
                rows.push(format!(
                    "{} skilldoctor ({}): {} files, score {}/100 ({:?}), {} errors, {} warnings — run {}",
                    C.yellow(Sym::WARN),
                    label,
                    report.files.len(),
                    report.score,
                    report.grade,
                    report.error_count,
                    report.warning_count,
                    paint_cmd("toksi doctor --fix")
                ));
            }
        }
    }

    // 3. Canonical <repo>/.toksi/<tool>/ vs loose dot-folder check
    if let Some(root) = ws {
        for loose in detect_loose_dot_folders(root) {
            rows.push(format!(
                "{} loose legacy/nested dot-folder {} detected — run {} or {} to migrate into .toksi/",
                C.yellow(Sym::WARN),
                normalize_path(&loose.to_string_lossy()),
                paint_cmd("toksi doctor --fix"),
                paint_cmd("toksi projectmem migrate")
            ));
        }
    }

    rows
}

pub fn heal_externalized_skills_in_dir(dir: &std::path::Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    let mut healed = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&cur) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if name != ".git" && name != "target" && name != "node_modules" {
                    stack.push(path);
                }
                continue;
            }
            let lower = name.to_ascii_lowercase();
            if lower == "skill.md" || lower.ends_with(".skill.md") {
                let Some(raw) = crate::util::paths::read_file_safe(&path) else {
                    continue;
                };
                let parent_name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str());
                let path_str = normalize_path(&path.to_string_lossy());
                let res = crate::rules::skill_validator::heal_skill_content(
                    &path_str,
                    &raw,
                    parent_name,
                    true,
                );
                if res.changed
                    && crate::util::paths::write_file_atomic(&path, &res.output).is_ok()
                {
                    healed += 1;
                }
            }
        }
    }
    healed
}

pub fn audit_wave18_topology(ws: Option<&std::path::Path>) -> Vec<String> {
    let mut rows = Vec::new();
    let cfg = crate::rules::skills_repo::SkillsRepoConfig::load();
    rows.push(format!(
        "{} Externalized <skills> remote: {}",
        C.green(Sym::CHECK),
        cfg.skills_remote
    ));

    let h = crate::util::paths::home();
    let mut plugin_dirs = Vec::new();
    if !h.is_empty() {
        plugin_dirs.push(std::path::PathBuf::from(&h).join(".toksi").join("plugins"));
    }
    if let Some(root) = ws {
        plugin_dirs.push(root.join(".toksi").join("plugins"));
        plugin_dirs.push(root.join(".agents").join("plugins"));
    }
    for pdir in plugin_dirs {
        if !pdir.is_dir() {
            continue;
        }
        let count = std::fs::read_dir(&pdir)
            .ok()
            .map(|rd| rd.flatten().filter(|e| e.path().is_dir()).count())
            .unwrap_or(0);
        rows.push(format!(
            "{} Plugin directory {}: {} plugin(s)",
            C.green(Sym::CHECK),
            normalize_path(&pdir.to_string_lossy()),
            count
        ));
    }
    rows
}

pub fn run_doctor(offline: bool, json_output: bool, fix: bool) -> i32 {
    run_doctor_with_options(DoctorOptions {
        offline,
        json_output,
        fix,
        workspace: None,
        include_workspace: false,
    })
}

pub fn run_doctor_with_options(opts: DoctorOptions) -> i32 {
    let offline = opts.offline;
    let json_output = opts.json_output;
    let fix = opts.fix;
    let ws_opt: Option<PathBuf> = if opts.include_workspace {
        Some(
            opts.workspace
                .clone()
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        )
    } else {
        None
    };

    let all_agents = list_agents();
    let tools = list_tools();

    if fix {
        if let Ok(cleaned) = clean_home_workspace_leakage()
            && cleaned > 0
        {
            L.raw(&format!(
                "  {} Cleaned {} home-directory workspace store leakage folder(s) under ~/.toksi/",
                C.green(Sym::CHECK),
                cleaned
            ));
        }
        if let Some(ref ws) = ws_opt {
            if let Ok(merged) = consolidate_nested_toksi_dirs(ws) {
                if merged > 0 {
                    L.raw(&format!(
                        "  {} Consolidated {} nested .toksi directory/directories into root workspace .toksi",
                        C.green(Sym::CHECK),
                        merged
                    ));
                }
            }
            if let Ok(merged_agents) = consolidate_nested_agent_dirs(ws) {
                if merged_agents > 0 {
                    L.raw(&format!(
                        "  {} Consolidated {} nested agent config directory/directories into workspace root",
                        C.green(Sym::CHECK),
                        merged_agents
                    ));
                }
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

        let config_healed = crate::agents::manager::heal_all_agent_legacy_configs(ws_opt.as_deref());
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
            if crate::env_injector::windows::get_env_reg_value("CLOUD_CODE_URL").is_some() {
                let _ = crate::env_injector::windows::delete_env_reg_value("CLOUD_CODE_URL");
                crate::env_injector::windows::broadcast_setting_change();
                L.raw(&format!(
                    "  {} Cleared stale CLOUD_CODE_URL from Windows registry",
                    C.green(Sym::CHECK)
                ));
                fixed_count += 1;
            }
        }

        if let Some(ref ws) = ws_opt {
            if let Ok(report) = crate::rules::sync_workspace_rules(ws, crate::rules::RuleKind::all()) {
                if !report.updated_files.is_empty() {
                    L.raw(&format!(
                        "  {} Synced in-tree rules across {} workspace file(s)",
                        C.green(Sym::CHECK),
                        report.updated_files.len()
                    ));
                    fixed_count += report.updated_files.len();
                }
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
            let detected = agent.detect().installed;
            let user_dir = AgentManager::user_config_dir(&agent.id);
            if !detected && !user_dir.exists() {
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
            if agent.id == "openagents" {
                if user_dir.exists() {
                    if let Ok(scaffolded) =
                        crate::agents::open_agents::scaffold_open_agents_structure(&user_dir)
                    {
                        fixed_count += scaffolded.len();
                    }
                }
                let _ = crate::agents::verify_direct_wire_bus(ws_opt.as_deref());
            }
        }

        // Heal externalized user skills (and workspace .agents/skills only on explicit --workspace opt-in)
        let skills_cfg = crate::rules::skills_repo::SkillsRepoConfig::load();
        let user_skills_healed = heal_externalized_skills_in_dir(&skills_cfg.user_skills_dir);
        if user_skills_healed > 0 {
            L.raw(&format!(
                "  {} Healed {} skill file(s) in ~/.toksi/user-skills (skilldoctor + agentskills)",
                C.green(Sym::CHECK),
                user_skills_healed
            ));
            fixed_count += user_skills_healed;
        }
        if let Some(ref ws) = ws_opt {
            let ws_skills = ws.join(".agents").join("skills");
            let ws_healed = heal_externalized_skills_in_dir(&ws_skills);
            if ws_healed > 0 {
                L.raw(&format!(
                    "  {} Healed {} workspace skill file(s) in .agents/skills",
                    C.green(Sym::CHECK),
                    ws_healed
                ));
                fixed_count += ws_healed;
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
                let wire_mode = match AgentManager::wire_mode(&agent.id) {
                    crate::agents::manager::WireMode::DirectWired => "direct-wired".to_string(),
                    crate::agents::manager::WireMode::ConfigWired => "config-wired".to_string(),
                };
                let layout_category = format!("{:?}", AgentManager::layout_category(&agent.id));
                let config_dir = normalize_path(
                    &AgentManager::user_config_dir(&agent.id).to_string_lossy(),
                );
                let cli_in_path = if agent.cli_bin.is_empty() {
                    true
                } else {
                    crate::util::env_path::check_executable_in_path(&agent.cli_bin).is_ok()
                };
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
                    wire_mode,
                    layout_category,
                    config_dir,
                    cli_in_path,
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
    let cwd = ws_opt
        .as_ref()
        .map(|p| normalize_path(&p.to_string_lossy()))
        .unwrap_or_else(|| {
            normalize_path(
                &env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| ".".into()),
            )
        });
    let tool_stats = doctor_tool_stats(&cwd, true);
    let config_warnings = audit_toksi_config_toml();
    let skills_audit = audit_externalized_skills_and_rules(ws_opt.as_deref());
    let topology_audit = audit_wave18_topology(ws_opt.as_deref());

    if json_output {
        #[derive(serde::Serialize)]
        struct DoctorJson<'a> {
            agents: &'a [AgentReport],
            stats: &'a std::collections::HashMap<String, Vec<String>>,
            config_warnings: &'a [String],
            skills_audit: &'a [String],
            topology_audit: &'a [String],
        }
        let doc = DoctorJson {
            agents: &reports,
            stats: &tool_stats,
            config_warnings: &config_warnings,
            skills_audit: &skills_audit,
            topology_audit: &topology_audit,
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

    if !config_warnings.is_empty() {
        tree_corner("Config (~/.toksi/config.toml)");
        for w in &config_warnings {
            tree_leaf(&C.yellow(w));
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

    if !skills_audit.is_empty() || !topology_audit.is_empty() {
        tree_corner("Externalized Skills (<enforce>) & Topology");
        for s in skills_audit.iter().chain(topology_audit.iter()) {
            tree_leaf(s);
        }
        tree_close();
    }

    let cli_warnings = audit_agent_cli_binaries();
    let home_leak_warnings = audit_home_workspace_leakage();
    if !cli_warnings.is_empty() || !home_leak_warnings.is_empty() {
        tree_corner("PATH & Topology Validation");
        for w in cli_warnings.into_iter().chain(home_leak_warnings.into_iter()) {
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
        let tool_ids = ["headroom", "projectmem", "rtk", "codegraph", "context", "context-mode"];
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
        if (r.id == "copilot" || r.id == "copilot-cli" || r.id == "copilot-vscode")
            && crate::agents::copilot::copilot_proxy_wired()
        {
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

    #[test]
    fn test_heal_externalized_skills_in_dir_adds_frontmatter_and_enforce() -> Result<(), Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let skill_dir = temp.path().join("my-skill");
        std::fs::create_dir_all(&skill_dir)?;
        let skill_file = skill_dir.join("SKILL.md");
        std::fs::write(&skill_file, "Follow strict engineering discipline.\n")?;

        let healed = heal_externalized_skills_in_dir(temp.path());
        assert_eq!(healed, 1);

        let updated = std::fs::read_to_string(&skill_file)?;
        assert!(updated.contains("name: my-skill"));
        assert!(updated.contains("<enforce>"));
        Ok(())
    }

    #[test]
    fn test_audit_externalized_skills_and_wave18_topology() {
        let rows = audit_externalized_skills_and_rules(None);
        assert!(!rows.is_empty());
        let topo = audit_wave18_topology(None);
        assert!(!topo.is_empty());
    }
}
