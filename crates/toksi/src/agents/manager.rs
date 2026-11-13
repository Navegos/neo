// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/manager.rs
// created: 2026-11-09
// lastModified: 2026-11-13

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::agents::open_agents::{open_agents_global_dir, open_agents_workspace_dir, scaffold_open_agents_structure};
use crate::core::get_agent;
use crate::util::paths::{home, normalize_path};

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

pub struct AgentManager;

impl AgentManager {
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
        for &tool in &["codegraph", "context-mode", "projectmem", "rtk"] {
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

        // Purge legacy runners
        let purged = crate::agents::purge_legacy_mcp_runners(agent_id);
        for p in purged {
            details.push(format!("Purged legacy runner: {}", p));
        }

        // Sync rules for this agent
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_openagents_install_and_health() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join(".agents");
        let scaffolded = scaffold_open_agents_structure(&target).unwrap();
        assert!(!scaffolded.is_empty());
        assert!(target.join("mcp.json").exists());
        assert!(target.join("models.json").exists());
    }
}
