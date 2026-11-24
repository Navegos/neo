// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/hooks.rs
// created: 2026-10-05
// lastModified: 2026-11-16

#![deny(unsafe_code)]

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
/// e.g. `codegraph-hook agy`, `projectmem-hook agy`, `ctxmode-hook agy`, `rtk-hook agy`
pub fn standard_hook_command(tool: &str, agent: &str) -> Vec<String> {
    let norm_tool = normalize_hook_tool_name(tool);
    let norm_agent = normalize_hook_agent_name(agent);
    vec![format!("{}-hook", norm_tool), norm_agent.to_string()]
}

/// Normalizes tool names for hooks:
/// e.g. `toksi-projectmem` -> `projectmem`, `toksi-context-mode` / `context-mode` / `context` -> `ctxmode`, `ai-memory` -> `aimemory`
pub fn normalize_hook_tool_name(tool: &str) -> &'static str {
    match tool.trim().to_ascii_lowercase().as_str() {
        "projectmem"
        | "toksi-projectmem"
        | "tokless-projectmem"
        | "tokless_projectmem"
        | "projectmem-context"
        | "pjm" => "projectmem",
        "ctxmode"
        | "context"
        | "context-mode"
        | "contextmode"
        | "toksi-ctxmode"
        | "toksi-context"
        | "toksi-context-mode"
        | "tokless-context"
        | "tokless_context"
        | "tokless-context-mode"
        | "ctx" => "ctxmode",
        "codegraph"
        | "toksi-codegraph"
        | "toksi-codegraph-index"
        | "tokless-codegraph"
        | "tokless_codegraph"
        | "tokless-codegraph-index"
        | "graph" => "codegraph",
        "rtk" | "toksi-rtk" | "tokless-rtk" | "tokless_rtk" => "rtk",
        "icm" | "toksi-icm" | "tokless-icm" | "tokless_icm" => "icm",
        "aimemory"
        | "ai-memory"
        | "ai_memory"
        | "toksi-aimemory"
        | "toksi-ai-memory"
        | "tokless-aimemory"
        | "tokless_aimemory" => "aimemory",
        "headroom" | "toksi-headroom" => "headroom",
        "security" | "toksi-security" => "security",
        _ => "tool",
    }
}

/// Normalizes agent identifiers for hook syntax while preserving CLI vs IDE distinction:
/// `antigravity-cli` vs `antigravity-ide`, `copilot-cli` vs `copilot-vscode`,
/// `claude-cli` vs `claude-desktop`, `cursor-cli` vs `cursor-ide`, `grok-cli` vs `grok-ide`,
/// plus `"openagents"`.
pub fn normalize_hook_agent_name(agent: &str) -> &'static str {
    match agent.trim().to_ascii_lowercase().as_str() {
        "antigravity" | "agy" => "agy",
        "antigravity-cli" => "antigravity-cli",
        "antigravity-ide" => "antigravity-ide",
        "copilot" => "copilot",
        "copilot-cli" => "copilot-cli",
        "copilot-vscode" | "copilot-ide" => "copilot-vscode",
        "claude" => "claude",
        "claude-cli" | "claude-code" => "claude-cli",
        "claude-desktop" => "claude-desktop",
        "cursor" => "cursor",
        "cursor-cli" => "cursor-cli",
        "cursor-ide" => "cursor-ide",
        "grok" => "grok",
        "grok-cli" => "grok-cli",
        "grok-ide" => "grok-ide",
        "codex" => "codex",
        "cline" => "cline",
        "droid" | "factory" => "droid",
        "opencode" => "opencode",
        "opencode2" => "opencode2",
        "openagents" => "openagents",
        "pi" => "pi",
        "omp" => "omp",
        "kilo" => "kilo",
        "kimi" => "kimi",
        "zcode" => "zcode",
        "amp" => "amp",
        "dsh" => "dsh",
        _ => "agent",
    }
}

fn family_base_agent(norm_agent: &str) -> &'static str {
    match norm_agent {
        "antigravity-cli" | "antigravity-ide" => "agy",
        "copilot-cli" | "copilot-vscode" => "copilot",
        "claude-cli" | "claude-desktop" => "claude",
        "cursor-cli" | "cursor-ide" => "cursor",
        "grok-cli" | "grok-ide" => "grok",
        "opencode2" => "opencode",
        _ => "",
    }
}

/// Checks whether a command string matches standard hook pattern `<tool>-hook <agent>`
pub fn is_hook_command_match(cmd_str: &str, tool: &str, agent: &str) -> bool {
    let norm_tool = normalize_hook_tool_name(tool);
    let norm_agent = normalize_hook_agent_name(agent);
    let standard = format!("{}-hook {}", norm_tool, norm_agent);
    if cmd_str.contains(&standard) {
        return true;
    }
    let base = family_base_agent(norm_agent);
    if !base.is_empty() {
        let family = format!("{}-hook {}", norm_tool, base);
        if cmd_str.contains(&family) {
            return true;
        }
    }
    false
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
        assert_eq!(standard_hook_command("codegraph", "antigravity-cli"), vec!["codegraph-hook", "antigravity-cli"]);
        assert_eq!(standard_hook_command("codegraph", "antigravity-ide"), vec!["codegraph-hook", "antigravity-ide"]);
        assert_eq!(standard_hook_command("projectmem", "copilot"), vec!["projectmem-hook", "copilot"]);
        assert_eq!(standard_hook_command("projectmem", "copilot-cli"), vec!["projectmem-hook", "copilot-cli"]);
        assert_eq!(standard_hook_command("projectmem", "copilot-vscode"), vec!["projectmem-hook", "copilot-vscode"]);
        assert_eq!(standard_hook_command("context-mode", "cursor"), vec!["ctxmode-hook", "cursor"]);
        assert_eq!(standard_hook_command("toksi-rtk", "claude-cli"), vec!["rtk-hook", "claude-cli"]);
        assert_eq!(standard_hook_command("rtk", "claude-desktop"), vec!["rtk-hook", "claude-desktop"]);
        assert_eq!(standard_hook_command("ctxmode", "openagents"), vec!["ctxmode-hook", "openagents"]);
        assert_eq!(standard_hook_command("context", "openagents"), vec!["ctxmode-hook", "openagents"]);
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
        assert_eq!(p2, ".copilot/skills/ctxmode");
        let p3 = agent_mcp_to_skills_path(".copilot", "ctxmode");
        assert_eq!(p3, ".copilot/skills/ctxmode");
    }
}
