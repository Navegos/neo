// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/open_agents.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::core::AgentManifest;
use crate::util::paths::{home, normalize_path};

/// Directory names defined by the Open Agents specification.
pub const OPEN_AGENTS_DIR: &str = ".agents";
pub const OPEN_AGENTS_SUBDIRS: &[&str] = &[
    "agents",
    "skills",
    "tasks",
    "knowledge",
    "memories",
    "layouts",
];

/// Configuration files in the Open Agents specification.
pub const OPEN_AGENTS_MCP_FILE: &str = "mcp.json";
pub const OPEN_AGENTS_MODELS_FILE: &str = "models.json";
pub const OPEN_AGENTS_SYSTEM_PROMPT_FILE: &str = "system-prompt.md";
pub const OPEN_AGENTS_AGENTS_FILE: &str = "agents.md";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAgentsMcpServer {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAgentsMcpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: std::collections::BTreeMap<String, OpenAgentsMcpServer>,
}

/// Returns the user global Open Agents directory: `~/.agents/`.
pub fn open_agents_global_dir() -> PathBuf {
    PathBuf::from(home()).join(OPEN_AGENTS_DIR)
}

/// Returns the user global Open Agents directory as normalized String.
pub fn open_agents_global_dir_string() -> String {
    normalize_path(&open_agents_global_dir().to_string_lossy())
}

/// Returns the workspace Open Agents directory: `<root>/.agents/`.
pub fn open_agents_workspace_dir(root: &Path) -> PathBuf {
    root.join(OPEN_AGENTS_DIR)
}

/// Checks whether an Open Agents directory exists at the given path.
pub fn is_open_agents_present(dir: &Path) -> bool {
    dir.is_dir()
}

/// Detects Open Agents directory at user level (`~/.agents`) or workspace root (`./.agents`).
pub fn detect_open_agents() -> crate::core::Detection {
    let global_dir = open_agents_global_dir();
    let ws_dir = open_agents_workspace_dir(Path::new("."));

    if global_dir.is_dir() {
        return crate::core::Detection {
            installed: true,
            source: normalize_path(&global_dir.to_string_lossy()),
        };
    }

    if ws_dir.is_dir() {
        return crate::core::Detection {
            installed: true,
            source: normalize_path(&ws_dir.to_string_lossy()),
        };
    }

    crate::core::Detection {
        installed: false,
        source: String::new(),
    }
}

/// Generates the standard Open Agents manifest.
pub fn openagents_manifest() -> AgentManifest {
    AgentManifest {
        id: "openagents".to_string(),
        label: "Open Agents".to_string(),
        homepage: "https://github.com/open-agents/open-agents".to_string(),
        cli_bin: "openagents".to_string(),
        config_dir: Some(Arc::new(open_agents_global_dir_string)),
        detect: Some(Arc::new(detect_open_agents)),
    }
}

/// Scaffolds the Open Agents directory structure in the target directory (`user` or `workspace`).
///
/// Layout:
/// ```text
/// target_dir/
///   mcp.json
///   models.json
///   system-prompt.md
///   agents.md
///   agents/
///   skills/
///   tasks/
///   knowledge/
///   memories/
///   layouts/
/// ```
pub fn scaffold_open_agents_structure(base_dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut created = Vec::new();
    let agents_dir = if base_dir.ends_with(OPEN_AGENTS_DIR) {
        base_dir.to_path_buf()
    } else {
        base_dir.join(OPEN_AGENTS_DIR)
    };

    if !agents_dir.exists() {
        fs::create_dir_all(&agents_dir)?;
        created.push(normalize_path(&agents_dir.to_string_lossy()));
    }

    for sub in OPEN_AGENTS_SUBDIRS {
        let sub_path = agents_dir.join(sub);
        if !sub_path.exists() {
            fs::create_dir_all(&sub_path)?;
            created.push(normalize_path(&sub_path.to_string_lossy()));
        }
    }

    // Default mcp.json wiring
    let mcp_path = agents_dir.join(OPEN_AGENTS_MCP_FILE);
    if !mcp_path.exists() {
        let default_mcp = default_open_agents_mcp();
        let json_str = serde_json::to_string_pretty(&default_mcp).unwrap_or_else(|_| "{}".to_string());
        fs::write(&mcp_path, json_str)?;
        created.push(normalize_path(&mcp_path.to_string_lossy()));
    }

    // Default models.json
    let models_path = agents_dir.join(OPEN_AGENTS_MODELS_FILE);
    if !models_path.exists() {
        let default_models = r#"{
  "default": "gemini-2.5-pro",
  "providers": {
    "gateway": "http://127.0.0.1:9099/v1"
  }
}
"#;
        fs::write(&models_path, default_models)?;
        created.push(normalize_path(&models_path.to_string_lossy()));
    }

    // Default system-prompt.md
    let prompt_path = agents_dir.join(OPEN_AGENTS_SYSTEM_PROMPT_FILE);
    if !prompt_path.exists() {
        let default_prompt = r#"# Open Agents System Prompt
Follow principles: think before coding, simplicity first, surgical changes, goal-driven execution.
Use toksi in-tree rules (Karpathy, Caveman, Ponytail) for compact, verified engineering.
"#;
        fs::write(&prompt_path, default_prompt)?;
        created.push(normalize_path(&prompt_path.to_string_lossy()));
    }

    // Default agents.md
    let agents_md_path = agents_dir.join(OPEN_AGENTS_AGENTS_FILE);
    if !agents_md_path.exists() {
        let default_agents_md = r#"# Agent Instructions
Apply on every coding task:
- Principles — think, simplify, edit surgically, verify.
- Response Style (caveman) — terse prose, full technical accuracy.
- Build Discipline (ponytail) — reuse first, write only what must exist.
"#;
        fs::write(&agents_md_path, default_agents_md)?;
        created.push(normalize_path(&agents_md_path.to_string_lossy()));
    }

    Ok(created)
}

/// Generates the standard default MCP configuration for Open Agents.
pub fn default_open_agents_mcp() -> OpenAgentsMcpConfig {
    let mut mcp_servers = std::collections::BTreeMap::new();

    mcp_servers.insert(
        "codegraph".to_string(),
        OpenAgentsMcpServer {
            command: "toksi".to_string(),
            args: vec![
                "run-mcp".to_string(),
                "--agent".to_string(),
                "openagents".to_string(),
                "toksi".to_string(),
                "graph".to_string(),
                "mcp".to_string(),
            ],
        },
    );

    mcp_servers.insert(
        "context-mode".to_string(),
        OpenAgentsMcpServer {
            command: "toksi".to_string(),
            args: vec![
                "run-mcp".to_string(),
                "--tool".to_string(),
                "context-mode".to_string(),
                "toksi".to_string(),
                "ctx".to_string(),
                "mcp".to_string(),
            ],
        },
    );

    mcp_servers.insert(
        "projectmem".to_string(),
        OpenAgentsMcpServer {
            command: "toksi".to_string(),
            args: vec![
                "run-mcp".to_string(),
                "--tool".to_string(),
                "projectmem".to_string(),
                "toksi".to_string(),
                "mem".to_string(),
                "mcp".to_string(),
            ],
        },
    );

    OpenAgentsMcpConfig { mcp_servers }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_agents_structure_scaffold() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("test_scaffold");
        let created = scaffold_open_agents_structure(&target).unwrap();
        assert!(!created.is_empty());

        let agents_dir = target.join(OPEN_AGENTS_DIR);
        assert!(agents_dir.exists());
        assert!(agents_dir.join("mcp.json").exists());
        assert!(agents_dir.join("models.json").exists());
        assert!(agents_dir.join("system-prompt.md").exists());
        assert!(agents_dir.join("agents.md").exists());

        for sub in OPEN_AGENTS_SUBDIRS {
            assert!(agents_dir.join(sub).exists());
        }
    }

    #[test]
    fn test_default_open_agents_mcp() {
        let mcp = default_open_agents_mcp();
        assert!(mcp.mcp_servers.contains_key("codegraph"));
        assert!(mcp.mcp_servers.contains_key("context-mode"));
        assert!(mcp.mcp_servers.contains_key("projectmem"));
    }
}
