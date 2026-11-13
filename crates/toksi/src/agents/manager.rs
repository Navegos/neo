// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/manager.rs
// created: 2026-11-09
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::agents::mcp_adapters::normalize_mcp_tool_id;
use crate::agents::open_agents::{open_agents_global_dir, open_agents_workspace_dir, scaffold_open_agents_structure};
use crate::core::get_agent;
use crate::util::env_path::check_executable_in_path;
use crate::util::jsonc::{Value, stringify_json, try_parse_jsonc};
use crate::util::paths::{home, normalize_path, read_file_safe, write_file_atomic};

#[derive(Error, Debug)]
pub enum AgentManagerError {
    #[error("Agent '{0}' not found in registry")]
    AgentNotFound(String),
    #[error("Filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthReport {
    pub id: String,
    pub label: String,
    pub installed: bool,
    pub config_dir: String,
    pub mcp_configured: bool,
    pub hooks_configured: bool,
    pub rules_synced: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMismatchReport {
    pub id: String,
    pub label: String,
    pub user_dir: String,
    pub workspace_dir: Option<String>,
    pub missing_hooks: Vec<String>,
    pub missing_mcp_servers: Vec<String>,
    pub outdated_rules: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLifecycleResult {
    pub action: String,
    pub agent: String,
    pub success: bool,
    pub target_path: String,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireMode {
    DirectWired,
    ConfigWired,
}

pub struct AgentManager;

impl AgentManager {
    /// Returns the preferred wiring mode for an agent harness (`DirectWired` for
    /// in-process `DirectWireBus` dispatch alongside `ConfigWired` config files).
    pub fn wire_mode(agent_id: &str) -> WireMode {
        if agent_id.eq_ignore_ascii_case("toksi") || agent_id.eq_ignore_ascii_case("openagents") {
            WireMode::DirectWired
        } else {
            WireMode::ConfigWired
        }
    }

    /// Supported vendor and standard agent identifiers.
    pub fn all_managed_agents() -> Vec<String> {
        vec![
            "antigravity".to_string(),
            "copilot".to_string(),
            "claude".to_string(),
            "codex".to_string(),
            "cursor".to_string(),
            "grok".to_string(),
            "droid".to_string(),
            "cline".to_string(),
            "opencode".to_string(),
            "opencode2".to_string(),
            "kilo".to_string(),
            "omp".to_string(),
            "pi".to_string(),
            "amp".to_string(),
            "kimi".to_string(),
            "zcode".to_string(),
            "dsh".to_string(),
            "openagents".to_string(),
        ]
    }

    /// User-level configuration directory for a given agent.
    pub fn user_config_dir(agent_id: &str) -> PathBuf {
        let home = PathBuf::from(home());
        match agent_id {
            "openagents" => open_agents_global_dir(),
            "antigravity" | "agy" => home.join(".gemini"),
            "copilot" => home.join(".copilot"),
            "claude" => home.join(".claude"),
            "codex" => home.join(".codex"),
            "cursor" => home.join(".cursor"),
            "grok" => home.join(".grok"),
            "droid" => home.join(".factory"),
            "cline" => home.join(".cline"),
            _ => {
                if let Some(agent) = get_agent(agent_id) {
                    if let Some(ref dir_fn) = agent.config_dir {
                        return PathBuf::from(dir_fn());
                    }
                }
                home.join(format!(".{}", agent_id))
            }
        }
    }

    /// Workspace-level configuration directory for a given agent.
    pub fn workspace_config_dir(agent_id: &str, root: &Path) -> PathBuf {
        match agent_id {
            "openagents" => open_agents_workspace_dir(root),
            "antigravity" | "agy" => root.join(".gemini"),
            "copilot" => root.join(".github"),
            "claude" => root.join(".claude"),
            "codex" => root.join(".codex"),
            "cursor" => root.join(".cursor"),
            "grok" => root.join(".grok"),
            "droid" => root.join(".factory"),
            "cline" => root.join(".cline"),
            _ => root.join(format!(".{}", agent_id)),
        }
    }

    /// Installs agent configuration.
    /// User-level is the default auto-install location.
    /// Workspace-level is only populated if `workspace: true`.
    pub fn install(agent_id: &str, workspace: bool) -> Result<AgentLifecycleResult, AgentManagerError> {
        let target_dir = if workspace {
            Self::workspace_config_dir(agent_id, Path::new("."))
        } else {
            Self::user_config_dir(agent_id)
        };

        let mut details = Vec::new();

        if agent_id == "openagents" {
            let scaffolded = scaffold_open_agents_structure(&target_dir)?;
            details.extend(scaffolded);
        } else {
            if !target_dir.exists() {
                fs::create_dir_all(&target_dir)?;
                details.push(format!("Created directory: {}", normalize_path(&target_dir.to_string_lossy())));
            }

            // Sync standard in-tree rules & skills strictly for this selected agent
            let sync_opts = crate::rules::SyncOptions::for_agents([agent_id]);
            if workspace {
                let _ = crate::rules::sync_workspace_rules_with_options(
                    ".",
                    crate::rules::RuleKind::all(),
                    &sync_opts,
                );
                details.push("Synced workspace rules (AGENTS.md, instructions, skills)".to_string());
            } else {
                let _ = crate::rules::sync_global_rules_with_options(
                    crate::rules::RuleKind::all(),
                    &sync_opts,
                );
                details.push(format!("Synced user-level rules into {}", normalize_path(&target_dir.to_string_lossy())));
            }
        }

        Ok(AgentLifecycleResult {
            action: "install".to_string(),
            agent: agent_id.to_string(),
            success: true,
            target_path: normalize_path(&target_dir.to_string_lossy()),
            details,
        })
    }

    /// Performs a health audit for an agent.
    pub fn health(agent_id: &str) -> Result<AgentHealthReport, AgentManagerError> {
        let user_dir = Self::user_config_dir(agent_id);
        let label = get_agent(agent_id).map(|a| a.label.clone()).unwrap_or_else(|| agent_id.to_string());
        let mut issues = Vec::new();
        let installed = user_dir.exists();

        if !installed {
            issues.push(format!("User configuration directory missing: {}", normalize_path(&user_dir.to_string_lossy())));
        }

        // Probe MCP configuration
        let mcp_configured = if agent_id == "openagents" {
            let mcp_path = user_dir.join("mcp.json");
            let exists = mcp_path.exists();
            if !exists && installed {
                issues.push("mcp.json missing from .agents directory".to_string());
            }
            exists
        } else if agent_id == "claude" {
            user_dir.join(".mcp.json").exists()
        } else if agent_id == "antigravity" || agent_id == "agy" {
            user_dir.join("mcp_config.json").exists()
        } else if agent_id == "cursor" {
            user_dir.join("mcp.json").exists()
        } else if agent_id == "codex" || agent_id == "grok" {
            user_dir.join("config.toml").exists()
        } else if agent_id == "copilot" {
            user_dir.join("mcp-config.json").exists()
        } else {
            installed
        };

        // Probe hooks configuration
        let hooks_path = user_dir.join("hooks.json");
        let hooks_dir = user_dir.join("hooks");
        let hooks_configured = hooks_path.exists() || hooks_dir.exists();

        let rules_synced = user_dir.join("rules").exists()
            || user_dir.join("skills").exists()
            || user_dir.join("agents.md").exists();

        Ok(AgentHealthReport {
            id: agent_id.to_string(),
            label,
            installed,
            config_dir: normalize_path(&user_dir.to_string_lossy()),
            mcp_configured,
            hooks_configured,
            rules_synced,
            issues,
        })
    }

    /// Audits configuration mismatches between user directory and expected wiring.
    pub fn mismatch(agent_id: &str) -> Result<AgentMismatchReport, AgentManagerError> {
        let user_dir = Self::user_config_dir(agent_id);
        let ws_dir = Self::workspace_config_dir(agent_id, Path::new("."));
        let label = get_agent(agent_id).map(|a| a.label.clone()).unwrap_or_else(|| agent_id.to_string());

        let mut missing_hooks = Vec::new();
        let missing_mcp_servers = Vec::new();

        // Check essential tools wiring
        for &tool in &["codegraph", "context", "projectmem", "rtk", "icm", "aimemory"] {
            let hook_sub = user_dir.join("hooks").join(format!("{}.json", tool));
            if !hook_sub.exists() && !user_dir.join("hooks.json").exists() {
                missing_hooks.push(tool.to_string());
            }
        }

        let outdated_rules = !user_dir.join("skills").exists() && !user_dir.join("rules").exists();

        Ok(AgentMismatchReport {
            id: agent_id.to_string(),
            label,
            user_dir: normalize_path(&user_dir.to_string_lossy()),
            workspace_dir: if ws_dir.exists() {
                Some(normalize_path(&ws_dir.to_string_lossy()))
            } else {
                None
            },
            missing_hooks,
            missing_mcp_servers,
            outdated_rules,
        })
    }

    /// Upgrades agent configuration and refreshes in-tree rules.
    pub fn upgrade(agent_id: &str) -> Result<AgentLifecycleResult, AgentManagerError> {
        let user_dir = Self::user_config_dir(agent_id);
        let mut details = Vec::new();

        if user_dir.exists() {
            // Re-sync rules and skills for this agent
            let sync_opts = crate::rules::SyncOptions::for_agents([agent_id]);
            let _ = crate::rules::sync_global_rules_with_options(
                crate::rules::RuleKind::all(),
                &sync_opts,
            );
            details.push("Refreshed user-level rules and skills".to_string());

            if agent_id == "openagents" {
                let scaffolded = scaffold_open_agents_structure(&user_dir)?;
                details.extend(scaffolded);
            }
        }

        Ok(AgentLifecycleResult {
            action: "upgrade".to_string(),
            agent: agent_id.to_string(),
            success: true,
            target_path: normalize_path(&user_dir.to_string_lossy()),
            details,
        })
    }

    /// Fixes configuration regressions, purges legacy runners, and restores clean state.
    pub fn fix(agent_id: &str) -> Result<AgentLifecycleResult, AgentManagerError> {
        let user_dir = Self::user_config_dir(agent_id);
        let mut details = Vec::new();

        // 1. Heal hijacked binaries, shims, symlinks/junctions, and orphaned state files
        let heal_report = crate::env_injector::heal_legacy_hijacks();
        for (real, restored) in &heal_report.restored_binaries {
            details.push(format!(
                "Restored binary: {} -> {}",
                normalize_path(&real.to_string_lossy()),
                normalize_path(&restored.to_string_lossy())
            ));
        }
        for shim in &heal_report.removed_shims {
            details.push(format!(
                "Removed shim: {}",
                normalize_path(&shim.to_string_lossy())
            ));
        }
        for state in &heal_report.removed_state_files {
            details.push(format!(
                "Removed state file: {}",
                normalize_path(&state.to_string_lossy())
            ));
        }

        // 2. Purge legacy runners
        let purged = crate::agents::purge_legacy_mcp_runners(agent_id);
        for p in purged {
            details.push(format!("Purged legacy runner: {}", p));
        }

        // 3. Migrate legacy mcp folders to skills/<tool> and standardize hook/MCP configs
        let ws = std::env::current_dir().ok();
        let config_healed = heal_agent_legacy_configs(agent_id, ws.as_deref());
        details.extend(config_healed);

        // 4. Verify CLI executable in PATH when applicable
        if let Some(manifest) = get_agent(agent_id)
            && !manifest.cli_bin.is_empty()
            && check_executable_in_path(&manifest.cli_bin).is_ok()
        {
            details.push(format!("Verified CLI binary in PATH: {}", manifest.cli_bin));
        }

        // 5. Sync rules for this agent
        let sync_opts = crate::rules::SyncOptions::for_agents([agent_id]);
        let _ = crate::rules::sync_global_rules_with_options(
            crate::rules::RuleKind::all(),
            &sync_opts,
        );
        details.push("Repaired in-tree rules and skills".to_string());

        if agent_id == "openagents" {
            let scaffolded = scaffold_open_agents_structure(&user_dir)?;
            details.extend(scaffolded);
        }

        Ok(AgentLifecycleResult {
            action: "fix".to_string(),
            agent: agent_id.to_string(),
            success: true,
            target_path: normalize_path(&user_dir.to_string_lossy()),
            details,
        })
    }

    /// Restores clean backup of agent configuration if backup exists.
    pub fn restore(agent_id: &str) -> Result<AgentLifecycleResult, AgentManagerError> {
        let user_dir = Self::user_config_dir(agent_id);
        let backup_dir = user_dir.with_extension("toksi_bak");
        let mut details = Vec::new();

        if backup_dir.exists() {
            if user_dir.exists() {
                let _ = fs::remove_dir_all(&user_dir);
            }
            fs::rename(&backup_dir, &user_dir)?;
            details.push(format!("Restored configuration from backup: {}", normalize_path(&backup_dir.to_string_lossy())));
        } else {
            details.push("No backup found; re-installing clean default configuration".to_string());
            let install_res = Self::install(agent_id, false)?;
            details.extend(install_res.details);
        }

        Ok(AgentLifecycleResult {
            action: "restore".to_string(),
            agent: agent_id.to_string(),
            success: true,
            target_path: normalize_path(&user_dir.to_string_lossy()),
            details,
        })
    }

    /// Removes Toksi configuration from the agent directory without destroying user history.
    pub fn remove(agent_id: &str, workspace: bool) -> Result<AgentLifecycleResult, AgentManagerError> {
        let target_dir = if workspace {
            Self::workspace_config_dir(agent_id, Path::new("."))
        } else {
            Self::user_config_dir(agent_id)
        };

        let mut details = Vec::new();

        if target_dir.exists() {
            // Remove managed hooks and mcp config files
            let candidates = ["mcp.json", "mcp_config.json", "mcp-config.json", ".mcp.json", "hooks.json"];
            for file in &candidates {
                let path = target_dir.join(file);
                if path.exists() {
                    let _ = fs::remove_file(&path);
                    details.push(format!("Removed: {}", normalize_path(&path.to_string_lossy())));
                }
            }
            let hooks_sub = target_dir.join("hooks");
            if hooks_sub.exists() {
                let _ = fs::remove_dir_all(&hooks_sub);
                details.push(format!("Removed directory: {}", normalize_path(&hooks_sub.to_string_lossy())));
            }
        }

        Ok(AgentLifecycleResult {
            action: "remove".to_string(),
            agent: agent_id.to_string(),
            success: true,
            target_path: normalize_path(&target_dir.to_string_lossy()),
            details,
        })
    }

    /// Prints the primary configuration file path for editing.
    pub fn edit(agent_id: &str) -> Result<String, AgentManagerError> {
        let user_dir = Self::user_config_dir(agent_id);
        let target_file = match agent_id {
            "openagents" => user_dir.join("mcp.json"),
            "claude" => user_dir.join("settings.json"),
            "antigravity" | "agy" => user_dir.join("settings.json"),
            "cursor" => user_dir.join("mcp.json"),
            "codex" | "grok" => user_dir.join("config.toml"),
            "copilot" => user_dir.join("settings.json"),
            _ => user_dir.join("config.json"),
        };

        Ok(normalize_path(&target_file.to_string_lossy()))
    }
}

const DEPRECATED_CODEGRAPH_TOOL_DEFS: &[&str] = &[
    "codegraph_explore.json",
    "codegraph_node.json",
    "codegraph_search.json",
    "codegraph_callers.json",
];

/// Migrates legacy `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp`, `.agents/mcp` (and `.gemini/antigravity*/mcp`)
/// directories under `base_dir` into `skills/<tool>`.
pub fn migrate_legacy_mcp_dirs_to_skills(base_dir: &Path) -> Vec<String> {
    let mut actions = Vec::new();

    let mut pairs: Vec<(PathBuf, PathBuf)> = vec![
        (
            base_dir.join(".gemini").join("mcp"),
            base_dir.join(".gemini").join("skills"),
        ),
        (
            base_dir.join(".gemini").join("antigravity").join("mcp"),
            base_dir.join(".gemini").join("skills"),
        ),
        (
            base_dir.join(".gemini").join("antigravity-cli").join("mcp"),
            base_dir.join(".gemini").join("skills"),
        ),
        (
            base_dir.join(".gemini").join("antigravity-ide").join("mcp"),
            base_dir.join(".gemini").join("skills"),
        ),
        (
            base_dir.join(".copilot").join("mcp"),
            base_dir.join(".copilot").join("skills"),
        ),
        (
            base_dir.join(".claude").join("mcp"),
            base_dir.join(".claude").join("skills"),
        ),
        (
            base_dir.join(".agents").join("mcp"),
            base_dir.join(".agents").join("skills"),
        ),
    ];

    if let Some(dir_name) = base_dir.file_name().and_then(|n| n.to_str())
        && matches!(dir_name, ".gemini" | ".copilot" | ".claude" | ".agents")
    {
        pairs.push((base_dir.join("mcp"), base_dir.join("skills")));
    }

    for (mcp_dir, skills_dir) in pairs {
        if !mcp_dir.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&mcp_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let src_path = entry.path();
            let Some(raw_name) = src_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if src_path.is_dir() {
                let norm_tool = normalize_mcp_tool_id(raw_name);
                let dest_tool_dir = skills_dir.join(norm_tool);
                let _ = fs::create_dir_all(&dest_tool_dir);
                if let Ok(sub_entries) = fs::read_dir(&src_path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        let Some(sub_name) = sub_path.file_name().and_then(|n| n.to_str()) else {
                            continue;
                        };
                        if DEPRECATED_CODEGRAPH_TOOL_DEFS.contains(&sub_name) {
                            let _ = fs::remove_file(&sub_path);
                            continue;
                        }
                        let dest_file = dest_tool_dir.join(sub_name);
                        if !dest_file.exists() {
                            if fs::rename(&sub_path, &dest_file).is_err()
                                && fs::copy(&sub_path, &dest_file).is_ok()
                            {
                                let _ = fs::remove_file(&sub_path);
                            }
                        } else {
                            let _ = fs::remove_file(&sub_path);
                        }
                    }
                }
                let _ = fs::remove_dir_all(&src_path);
                let msg = format!(
                    "Migrated legacy MCP folder {} -> {}",
                    normalize_path(&src_path.to_string_lossy()),
                    normalize_path(&dest_tool_dir.to_string_lossy())
                );
                crate::util::logger::L.debug(&msg);
                actions.push(msg);
            } else if src_path.is_file() {
                let stem = src_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(raw_name);
                let norm_tool = normalize_mcp_tool_id(stem);
                let dest_tool_dir = skills_dir.join(norm_tool);
                let _ = fs::create_dir_all(&dest_tool_dir);
                let dest_file = dest_tool_dir.join(raw_name);
                if !dest_file.exists() {
                    if fs::rename(&src_path, &dest_file).is_err()
                        && fs::copy(&src_path, &dest_file).is_ok()
                    {
                        let _ = fs::remove_file(&src_path);
                    }
                } else {
                    let _ = fs::remove_file(&src_path);
                }
                actions.push(format!(
                    "Migrated legacy MCP file {} -> {}",
                    normalize_path(&src_path.to_string_lossy()),
                    normalize_path(&dest_file.to_string_lossy())
                ));
            }
        }
        let _ = fs::remove_dir(&mcp_dir);
    }

    actions
}

fn rewrite_legacy_hook_command_string(cmd: &str) -> Option<String> {
    let mut updated = cmd.to_string();
    let mut changed = false;

    if updated.contains("agy-hook codegraph-index") {
        updated = updated.replace("agy-hook codegraph-index", "codegraph-hook agy");
        changed = true;
    }
    if updated.contains("copilot-hook codegraph-index") {
        updated = updated.replace("copilot-hook codegraph-index", "codegraph-hook copilot");
        changed = true;
    }
    if updated.contains("cursor-hook codegraph-index") {
        updated = updated.replace("cursor-hook codegraph-index", "codegraph-hook cursor");
        changed = true;
    }
    if updated.trim() == "rtk hook cursor" {
        updated = "toksi rtk-hook cursor".to_string();
        changed = true;
    } else if updated.contains("rtk hook cursor") {
        updated = updated.replace("rtk hook cursor", "rtk-hook cursor");
        changed = true;
    }
    if updated.starts_with("tokless ") {
        updated = format!("toksi {}", &updated["tokless ".len()..]);
        changed = true;
    }

    if changed { Some(updated) } else { None }
}

fn rewrite_commands_in_json_value(val: &mut Value) -> bool {
    let mut changed = false;
    match val {
        Value::Object(map) => {
            if let Some(Value::String(cmd)) = map.get("command")
                && let Some(rewritten) = rewrite_legacy_hook_command_string(cmd)
            {
                map.insert("command".to_string(), Value::String(rewritten));
                changed = true;
            }
            for (_, v) in map.iter_mut() {
                if rewrite_commands_in_json_value(v) {
                    changed = true;
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                if rewrite_commands_in_json_value(item) {
                    changed = true;
                }
            }
        }
        _ => {}
    }
    changed
}

/// Standardizes hook invocations to `<tool>-hook <agent>`, renames legacy hook/plugin files,
/// and normalizes legacy MCP server keys under `base_dir`.
pub fn heal_hook_and_mcp_configs_in_dir(base_dir: &Path) -> Vec<String> {
    let mut actions = Vec::new();

    // 1. Antigravity / Gemini hooks & settings files
    let gemini_hooks = [
        base_dir.join(".gemini").join("config").join("hooks.json"),
        base_dir.join(".gemini").join("antigravity").join("hooks.json"),
        base_dir.join(".gemini").join("antigravity-cli").join("hooks.json"),
        base_dir.join(".gemini").join("antigravity-ide").join("hooks.json"),
        base_dir.join(".gemini").join("hooks.json"),
    ];
    for hf in gemini_hooks {
        let Some(raw) = read_file_safe(&hf) else {
            continue;
        };
        let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) else {
            continue;
        };
        let mut changed = false;
        for legacy_cg in ["toksi-codegraph-index", "tokless-codegraph-index", "toksi-codegraph"] {
            if let Some(val) = cfg.shift_remove(legacy_cg) {
                if !cfg.contains_key("codegraph") {
                    cfg.insert("codegraph".to_string(), val);
                }
                changed = true;
            }
        }
        for legacy_pm in ["toksi-projectmem", "tokless-projectmem"] {
            if let Some(val) = cfg.shift_remove(legacy_pm) {
                if !cfg.contains_key("projectmem-context") && !cfg.contains_key("projectmem") {
                    cfg.insert("projectmem-context".to_string(), val);
                }
                changed = true;
            }
        }
        for legacy_ctx in ["toksi-context-mode", "tokless-context-mode", "ctx"] {
            if let Some(val) = cfg.shift_remove(legacy_ctx) {
                if !cfg.contains_key("context-mode") {
                    cfg.insert("context-mode".to_string(), val);
                }
                changed = true;
            }
        }
        let mut root_val = Value::Object(cfg);
        if rewrite_commands_in_json_value(&mut root_val) {
            changed = true;
        }
        if changed && write_file_atomic(&hf, &stringify_json(&root_val)).is_ok() {
            actions.push(format!(
                "Standardized hooks in {}",
                normalize_path(&hf.to_string_lossy())
            ));
        }
    }

    // 2. Copilot & .github hook files
    for hooks_dir in [
        base_dir.join(".copilot").join("hooks"),
        base_dir.join(".github").join("hooks"),
    ] {
        if !hooks_dir.is_dir() {
            continue;
        }
        for legacy_cg_file in ["toksi-codegraph-index.json", "tokless-codegraph-index.json"] {
            let legacy_path = hooks_dir.join(legacy_cg_file);
            if legacy_path.is_file() {
                let target_path = hooks_dir.join("codegraph.json");
                if !target_path.exists()
                    && let Some(raw) = read_file_safe(&legacy_path)
                    && let Some(mut val) = try_parse_jsonc(&raw)
                {
                    let _ = rewrite_commands_in_json_value(&mut val);
                    let _ = write_file_atomic(&target_path, &stringify_json(&val));
                }
                let _ = fs::remove_file(&legacy_path);
                actions.push(format!(
                    "Removed legacy hook file {}",
                    normalize_path(&legacy_path.to_string_lossy())
                ));
            }
        }
        for legacy_pm_file in ["toksi-projectmem.json", "tokless-projectmem.json"] {
            let legacy_path = hooks_dir.join(legacy_pm_file);
            if legacy_path.is_file() {
                let target_path = hooks_dir.join("projectmem-context.json");
                if !target_path.exists()
                    && let Some(raw) = read_file_safe(&legacy_path)
                    && let Some(mut val) = try_parse_jsonc(&raw)
                {
                    let _ = rewrite_commands_in_json_value(&mut val);
                    let _ = write_file_atomic(&target_path, &stringify_json(&val));
                }
                let _ = fs::remove_file(&legacy_path);
                actions.push(format!(
                    "Removed legacy hook file {}",
                    normalize_path(&legacy_path.to_string_lossy())
                ));
            }
        }
    }

    // 3. Cursor, Claude, Codex, Droid, Grok hooks
    for hf in [
        base_dir.join(".cursor").join("hooks.json"),
        base_dir.join(".claude").join("settings.json"),
        base_dir.join(".codex").join("hooks.json"),
        base_dir.join(".factory").join("hooks.json"),
    ] {
        let Some(raw) = read_file_safe(&hf) else {
            continue;
        };
        let Some(mut val) = try_parse_jsonc(&raw) else {
            continue;
        };
        if rewrite_commands_in_json_value(&mut val)
            && write_file_atomic(&hf, &stringify_json(&val)).is_ok()
        {
            actions.push(format!(
                "Standardized hook commands in {}",
                normalize_path(&hf.to_string_lossy())
            ));
        }
    }

    // Grok legacy hook file
    let grok_hooks = base_dir.join(".grok").join("hooks");
    if grok_hooks.is_dir() {
        for legacy_pm in ["toksi-projectmem.json", "tokless-projectmem.json"] {
            let lp = grok_hooks.join(legacy_pm);
            if lp.is_file() {
                let dest = grok_hooks.join("projectmem.json");
                if !dest.exists() {
                    let _ = fs::rename(&lp, &dest);
                } else {
                    let _ = fs::remove_file(&lp);
                }
                actions.push(format!(
                    "Migrated legacy Grok hook {}",
                    normalize_path(&lp.to_string_lossy())
                ));
            }
        }
    }

    // 4. OpenCode & OMP legacy plugin/extension files (`toksi-codegraph-init.js`, `toksi-projectmem.ts`)
    for plugin_dir in [
        base_dir.join(".config").join("opencode").join("plugins"),
        base_dir.join(".local").join("share").join("opencode").join("plugins"),
        base_dir.join(".opencode").join("plugins"),
        base_dir.join(".omp").join("agent").join("extensions"),
        base_dir.join(".pi").join("agent").join("extensions"),
    ] {
        if !plugin_dir.is_dir() {
            continue;
        }
        for dead in [
            "toksi-codegraph-init.js",
            "tokless-codegraph-init.js",
            "toksi-index.ts",
            "tokless-index.ts",
        ] {
            let dp = plugin_dir.join(dead);
            if dp.is_file() && fs::remove_file(&dp).is_ok() {
                actions.push(format!(
                    "Removed legacy auto-index plugin {}",
                    normalize_path(&dp.to_string_lossy())
                ));
            }
        }
        for legacy_pm in ["toksi-projectmem.ts", "tokless-projectmem.ts"] {
            let lp = plugin_dir.join(legacy_pm);
            if lp.is_file() {
                let dest = plugin_dir.join("projectmem.ts");
                if !dest.exists() {
                    let _ = fs::rename(&lp, &dest);
                } else {
                    let _ = fs::remove_file(&lp);
                }
                actions.push(format!(
                    "Migrated legacy extension {}",
                    normalize_path(&lp.to_string_lossy())
                ));
            }
        }
    }

    // 5. Normalize legacy MCP server keys (`toksi-projectmem`, `toksi-codegraph`, `toksi-codegraph-index`)
    let mcp_files = [
        base_dir.join(".gemini").join("config").join("mcp_config.json"),
        base_dir.join(".gemini").join("antigravity").join("mcp_config.json"),
        base_dir.join(".gemini").join("antigravity-cli").join("mcp_config.json"),
        base_dir.join(".gemini").join("antigravity-ide").join("mcp_config.json"),
        base_dir.join(".copilot").join("mcp-config.json"),
        base_dir.join(".vscode").join("mcp.json"),
        base_dir.join(".cursor").join("mcp.json"),
        base_dir.join(".claude.json"),
        base_dir.join(".claude").join("settings.json"),
        base_dir.join(".mcp.json"),
        base_dir.join(".agents").join("mcp.json"),
    ];
    for mf in mcp_files {
        let Some(raw) = read_file_safe(&mf) else {
            continue;
        };
        let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) else {
            continue;
        };
        let mut changed = false;
        for section in ["mcpServers", "servers"] {
            if let Some(Value::Object(servers)) = cfg.get_mut(section) {
                let keys: Vec<String> = servers.keys().cloned().collect();
                for k in keys {
                    if matches!(
                        k.as_str(),
                        "toksi-projectmem"
                            | "tokless-projectmem"
                            | "toksi-codegraph"
                            | "tokless-codegraph"
                            | "toksi-codegraph-index"
                            | "tokless-codegraph-index"
                            | "toksi-context-mode"
                            | "tokless-context-mode"
                    ) {
                        let norm = normalize_mcp_tool_id(&k);
                        if let Some(val) = servers.shift_remove(&k) {
                            if !servers.contains_key(norm) {
                                servers.insert(norm.to_string(), val);
                            }
                            changed = true;
                        }
                    }
                }
            }
        }
        if changed && write_file_atomic(&mf, &stringify_json(&Value::Object(cfg))).is_ok() {
            actions.push(format!(
                "Normalized legacy MCP keys in {}",
                normalize_path(&mf.to_string_lossy())
            ));
        }
    }

    actions
}

/// Runs legacy folder and hook/MCP config healing for a specific agent across user home and workspace root.
pub fn heal_agent_legacy_configs(_agent_id: &str, workspace_root: Option<&Path>) -> Vec<String> {
    heal_all_agent_legacy_configs(workspace_root)
}

/// Runs legacy folder (`mcp` -> `skills/<tool>`) and hook/MCP config healing across user home and workspace root.
pub fn heal_all_agent_legacy_configs(workspace_root: Option<&Path>) -> Vec<String> {
    let mut actions = Vec::new();
    let h = home();
    if !h.is_empty() {
        let hp = Path::new(&h);
        actions.extend(migrate_legacy_mcp_dirs_to_skills(hp));
        actions.extend(heal_hook_and_mcp_configs_in_dir(hp));
    }
    if let Some(ws) = workspace_root {
        actions.extend(migrate_legacy_mcp_dirs_to_skills(ws));
        actions.extend(heal_hook_and_mcp_configs_in_dir(ws));
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_all_managed_agents_list() {
        let agents = AgentManager::all_managed_agents();
        assert!(agents.contains(&"antigravity".to_string()));
        assert!(agents.contains(&"copilot".to_string()));
        assert!(agents.contains(&"claude".to_string()));
        assert!(agents.contains(&"codex".to_string()));
        assert!(agents.contains(&"cursor".to_string()));
        assert!(agents.contains(&"grok".to_string()));
        assert!(agents.contains(&"openagents".to_string()));
    }

    #[test]
    fn test_openagents_install_and_health() -> TestResult {
        let temp = tempfile::tempdir()?;
        let target = temp.path().join(".agents");
        let scaffolded = scaffold_open_agents_structure(&target)?;
        assert!(!scaffolded.is_empty());
        assert!(target.join("mcp.json").exists());
        assert!(target.join("models.json").exists());
        Ok(())
    }

    #[test]
    fn test_migrate_legacy_mcp_and_standardize_hooks() -> TestResult {
        let temp = tempfile::tempdir()?;
        let root = temp.path();

        // 1. Legacy .gemini/mcp/toksi-codegraph and .copilot/mcp/toksi-projectmem
        let gemini_mcp_cg = root.join(".gemini").join("mcp").join("toksi-codegraph");
        fs::create_dir_all(&gemini_mcp_cg)?;
        fs::write(gemini_mcp_cg.join("SKILL.md"), "# Codegraph Skill")?;
        fs::write(gemini_mcp_cg.join("codegraph_explore.json"), "{}")?;

        let copilot_mcp_pm = root.join(".copilot").join("mcp").join("toksi-projectmem");
        fs::create_dir_all(&copilot_mcp_pm)?;
        fs::write(copilot_mcp_pm.join("SKILL.md"), "# ProjectMem Skill")?;

        // 2. Legacy hooks in .gemini/config/hooks.json and .cursor/hooks.json
        let gemini_hooks = root.join(".gemini").join("config").join("hooks.json");
        fs::create_dir_all(gemini_hooks.parent().ok_or("no parent")?)?;
        fs::write(
            &gemini_hooks,
            r#"{"toksi-codegraph-index":{"PostToolUse":[{"matcher":"","hooks":[{"type":"command","command":"toksi agy-hook codegraph-index"}]}]}}"#,
        )?;

        let cursor_hooks = root.join(".cursor").join("hooks.json");
        fs::create_dir_all(cursor_hooks.parent().ok_or("no parent")?)?;
        fs::write(
            &cursor_hooks,
            r#"{"version":1,"hooks":{"preToolUse":[{"command":"rtk hook cursor","matcher":"Shell"}]}}"#,
        )?;

        let opencode_plugins = root.join(".config").join("opencode").join("plugins");
        fs::create_dir_all(&opencode_plugins)?;
        fs::write(opencode_plugins.join("toksi-codegraph-init.js"), "// legacy")?;

        let actions = heal_all_agent_legacy_configs(Some(root));
        assert!(!actions.is_empty());

        // Verify .gemini/mcp and .copilot/mcp migrated to skills/<tool>
        assert!(!gemini_mcp_cg.exists());
        assert!(root.join(".gemini").join("skills").join("codegraph").join("SKILL.md").exists());
        assert!(!root.join(".gemini").join("skills").join("codegraph").join("codegraph_explore.json").exists());
        assert!(root.join(".copilot").join("skills").join("projectmem").join("SKILL.md").exists());

        // Verify hooks standardized
        let healed_gemini = fs::read_to_string(&gemini_hooks)?;
        assert!(healed_gemini.contains("\"codegraph\""));
        assert!(healed_gemini.contains("codegraph-hook agy"));
        assert!(!healed_gemini.contains("agy-hook codegraph-index"));

        let healed_cursor = fs::read_to_string(&cursor_hooks)?;
        assert!(healed_cursor.contains("rtk-hook cursor"));

        assert!(!opencode_plugins.join("toksi-codegraph-init.js").exists());
        Ok(())
    }

    #[test]
    fn test_in_tree_mcp_spawn_routing_and_wire_mode() {
        use crate::util::mcpspawn::pick_mcp_spawn;
        assert_eq!(pick_mcp_spawn("projectmem", &[]).args, vec!["mem", "mcp"]);
        assert_eq!(pick_mcp_spawn("context", &[]).args, vec!["ctx", "mcp"]);
        assert_eq!(pick_mcp_spawn("codegraph", &[]).args, vec!["graph", "mcp"]);
        assert_eq!(pick_mcp_spawn("icm", &[]).args, vec!["icm", "mcp"]);
        assert_eq!(pick_mcp_spawn("aimemory", &[]).args, vec!["aimemory", "mcp"]);
        assert_eq!(pick_mcp_spawn("ai-memory", &[]).args, vec!["aimemory", "mcp"]);
        assert_eq!(pick_mcp_spawn("rtk", &[]).args, vec!["rtk", "mcp"]);
        assert_eq!(AgentManager::wire_mode("openagents"), WireMode::DirectWired);
        assert_eq!(AgentManager::wire_mode("claude"), WireMode::ConfigWired);
    }
}
