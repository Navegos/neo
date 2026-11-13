// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/hooks.rs
// created: 2026-10-05
// lastModified: 2026-11-13

pub use super::antigravity::*;
pub use super::claude::*;
pub use super::cline::*;
pub use super::codex::*;
pub use super::copilot::*;
pub use super::cursor::*;
pub use super::droid::*;
pub use super::grok::*;
pub use super::kilo::*;
pub use super::omp::*;
pub use super::opencode::*;
pub use super::pi::*;
pub use super::proxy_agent::*;
pub use super::proxy_detection::*;
pub use super::proxy_protocol::*;
pub use super::proxy_provider::*;
pub use super::vscode::*;

pub type CursorHookSnapshot = CursorProjectRulesHookSnapshot;

/// Standardized uniform hook command syntax: `<tool>-hook <agent>`
/// e.g. `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`
pub fn standard_hook_command(tool: &str, agent: &str) -> Vec<String> {
    let norm_tool = normalize_hook_tool_name(tool);
    let norm_agent = normalize_hook_agent_name(agent);
    vec![format!("{}-hook", norm_tool), norm_agent.to_string()]
}

/// Normalizes tool names for hooks:
/// e.g. `toksi-projectmem` -> `projectmem`, `toksi-context-mode` / `context-mode` -> `context`, `toksi-codegraph` -> `codegraph`
pub fn normalize_hook_tool_name(tool: &str) -> &'static str {
    match tool.trim() {
        "projectmem" | "toksi-projectmem" | "pjm" => "projectmem",
        "context" | "context-mode" | "toksi-context-mode" | "ctx" => "context",
        "codegraph" | "toksi-codegraph" | "graph" => "codegraph",
        "rtk" | "toksi-rtk" => "rtk",
        "icm" | "toksi-icm" => "icm",
        "aimemory" | "ai-memory" | "toksi-aimemory" => "aimemory",
        "headroom" | "toksi-headroom" => "headroom",
        _ => "tool",
    }
}

/// Normalizes agent identifiers for hook syntax
pub fn normalize_hook_agent_name(agent: &str) -> &'static str {
    match agent.trim().to_ascii_lowercase().as_str() {
        "antigravity" | "agy" | "antigravity-cli" | "antigravity-ide" => "agy",
        "copilot" | "copilot-cli" | "copilot-vscode" | "copilot-ide" => "copilot",
        "claude" | "claude-cli" | "claude-desktop" | "claude-code" => "claude",
        "cursor" | "cursor-cli" | "cursor-ide" => "cursor",
        "grok" | "grok-cli" | "grok-ide" => "grok",
        "codex" => "codex",
        "cline" => "cline",
        "droid" | "factory" => "droid",
        "opencode" | "opencode2" => "opencode",
        "pi" => "pi",
        "omp" => "omp",
        "kilo" => "kilo",
        "kimi" => "kimi",
        "zcode" => "zcode",
        _ => "agent",
    }
}

/// Checks whether a command string matches standard hook pattern
pub fn is_hook_command_match(cmd_str: &str, tool: &str, agent: &str) -> bool {
    let norm_tool = normalize_hook_tool_name(tool);
    let norm_agent = normalize_hook_agent_name(agent);
    let standard = format!("{}-hook {}", norm_tool, norm_agent);
    cmd_str.contains(&standard)
}

/// Differentiates agent CLI vs agent editor/IDE
pub fn is_agent_cli(agent: &str) -> bool {
    matches!(
        agent.trim().to_ascii_lowercase().as_str(),
        "antigravity-cli" | "agy" | "copilot-cli" | "claude-cli" | "claude-code" | "cursor-cli" | "grok-cli"
    )
}

pub fn is_agent_ide(agent: &str) -> bool {
    matches!(
        agent.trim().to_ascii_lowercase().as_str(),
        "antigravity" | "antigravity-ide" | "copilot" | "copilot-vscode" | "copilot-ide" | "claude-desktop" | "cursor" | "cursor-ide" | "grok-ide" | "vscode"
    )
}

/// Maps MCP settings to skills directory path:
/// e.g. .gemini/mcp -> .gemini/skills/<tool>, .copilot/mcp -> .copilot/skills/<tool>
pub fn agent_mcp_to_skills_path(agent_dir: &str, tool: &str) -> String {
    let norm_tool = normalize_hook_tool_name(tool);
    crate::util::paths::normalize_path(&format!("{}/skills/{}", agent_dir, norm_tool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_hook_command() {
        assert_eq!(standard_hook_command("codegraph", "agy"), vec!["codegraph-hook", "agy"]);
        assert_eq!(standard_hook_command("projectmem", "copilot"), vec!["projectmem-hook", "copilot"]);
        assert_eq!(standard_hook_command("context-mode", "cursor"), vec!["context-hook", "cursor"]);
        assert_eq!(standard_hook_command("toksi-rtk", "claude-cli"), vec!["rtk-hook", "claude"]);
    }

    #[test]
    fn test_is_hook_command_match() {
        assert!(is_hook_command_match("toksi codegraph-hook agy", "codegraph", "antigravity"));
        assert!(!is_hook_command_match("toksi agy-hook codegraph-index", "codegraph", "antigravity"));
        assert!(is_hook_command_match("toksi projectmem-hook copilot", "projectmem", "copilot-vscode"));
    }

    #[test]
    fn test_agent_taxonomy() {
        assert!(is_agent_cli("agy"));
        assert!(is_agent_cli("copilot-cli"));
        assert!(is_agent_cli("claude-cli"));
        assert!(is_agent_cli("cursor-cli"));
        assert!(is_agent_cli("grok-cli"));

        assert!(is_agent_ide("antigravity-ide"));
        assert!(is_agent_ide("copilot-vscode"));
        assert!(is_agent_ide("claude-desktop"));
        assert!(is_agent_ide("cursor-ide"));
        assert!(is_agent_ide("grok-ide"));
    }

    #[test]
    fn test_agent_mcp_to_skills_path() {
        let p = agent_mcp_to_skills_path(".gemini", "codegraph");
        assert_eq!(p, ".gemini/skills/codegraph");

        let p2 = agent_mcp_to_skills_path(".copilot", "context-mode");
        assert_eq!(p2, ".copilot/skills/context");
    }
}
