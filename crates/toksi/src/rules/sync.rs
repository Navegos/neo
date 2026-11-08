// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/sync.rs
// created: 2026-11-05
// lastModified: 2026-11-08

//! Universal rules & skills synchronizer for workspace and agent configuration homes.
//!
//! Synchronizes in-tree coding guidelines (Principles, Caveman, Ponytail) across
//! workspace directories and user home config paths for all 17 supported agents and open conventions:
//! - OpenAgents (`.agents/agents.md`, `.agents/system-prompt.md`, `.agents/skills/`)
//! - Antigravity / Gemini (`GEMINI.md`, `.gemini/rules/`, `.gemini/skills/`, `~/.gemini/rules.md`)
//! - GitHub Copilot (`.copilot/`, `.github/copilot-instructions.md`, `.copilot/skills/`, `.github/skills/`)
//! - Claude Code (`CLAUDE.md`, `.claude/rules/`, `.claude/skills/`, `~/.claude/CLAUDE.md`)
//! - Codex (`CODEX.md`, `AGENTS.md`, `.codex/rules/`, `~/.codex/instructions.md`)
//! - Cursor (`.cursorrules`, `.cursor/rules/*.mdc`, `.cursor/skills/`)
//! - Grok (`.grok/skills/`, `.grok/rules/`, `~/.grok/rules/AGENTS.md`)
//! - Additional agents: Cline, OpenCode, Droid, Kilo, OMP, Pi, Amp, Kimi, ZCode, DSH.
//!
//! Also wires tool MCP settings into `skills/<tool>`:
//! - `.gemini/mcp` -> `skills/<tool>`
//! - `.copilot/mcp` -> `skills/<tool>`
//! - `.claude/mcp` -> `skills/<tool>`
//! - `.agents/mcp` -> `skills/<tool>`

use std::fs;
use std::path::{Path, PathBuf};
use crate::rules::{
    patch_agents_md, patch_agents_rules_md, patch_claude_md, patch_cline_rules,
    patch_codex_md, patch_copilot_md, patch_cursor_rules, patch_cursorrules,
    patch_gemini_md, patch_skills_dir, RuleKind,
};
use crate::util::paths::{home, normalize_path};

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

/// Synchronization outcome report.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub workspace: PathBuf,
    pub updated_files: Vec<PathBuf>,
    pub up_to_date_files: Vec<PathBuf>,
    pub errors: Vec<String>,
}

impl SyncReport {
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn total_touched(&self) -> usize {
        self.updated_files.len() + self.up_to_date_files.len()
    }

    /// Record a target path into the report using normalized forward-slash representation.
    pub fn record_updated(&mut self, path: impl AsRef<Path>) {
        let norm = normalize_path(&path.as_ref().to_string_lossy());
        self.updated_files.push(PathBuf::from(norm));
    }

    /// Record an up-to-date target path into the report.
    pub fn record_up_to_date(&mut self, path: impl AsRef<Path>) {
        let norm = normalize_path(&path.as_ref().to_string_lossy());
        self.up_to_date_files.push(PathBuf::from(norm));
    }

    /// Record an error message into the report.
    pub fn record_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
}

/// Known tool identifiers that get wired into skills.
pub const MANAGED_MCP_TOOLS: &[&str] = &["codegraph", "context", "projectmem", "rtk", "icm", "aimemory"];

/// Wires tool settings/instructions from agent `mcp` directories to `skills/<tool>`.
/// Supports:
/// - `.gemini/mcp` -> `.gemini/skills/<tool>`
/// - `.copilot/mcp` -> `.copilot/skills/<tool>`
/// - `.claude/mcp` -> `.claude/skills/<tool>`
/// - `.agents/mcp` -> `.agents/skills/<tool>`
pub fn wire_mcp_skills(base_dir: impl AsRef<Path>, tools: &[&str]) -> Result<Vec<PathBuf>, DynError> {
    let base = base_dir.as_ref();
    let mut wired = Vec::new();

    let agent_dirs = [".gemini", ".copilot", ".claude", ".agents"];

    for agent in &agent_dirs {
        let agent_path = base.join(agent);
        let mcp_dir = agent_path.join("mcp");
        let skills_dir = agent_path.join("skills");

        for &tool in tools {
            let tool_mcp = mcp_dir.join(tool);
            let tool_skill = skills_dir.join(tool);

            // If the tool has MCP files or instructions, mirror/wire into skills/<tool>
            if tool_mcp.exists() {
                if let Err(e) = fs::create_dir_all(&tool_skill) {
                    return Err(format!("Failed to create {}: {}", tool_skill.display(), e).into());
                }

                // Copy instructions and definitions if present
                if let Ok(entries) = fs::read_dir(&tool_mcp) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = entry.file_name();
                            let target = tool_skill.join(&file_name);
                            if let Ok(content) = fs::read_to_string(&path) {
                                let _ = fs::write(&target, content);
                                wired.push(PathBuf::from(normalize_path(&target.to_string_lossy())));
                            }
                        }
                    }
                }
            } else {
                // Synthesize standalone skill manifest for tool if mcp directory exists or agent is present
                if agent_path.exists() {
                    let skill_file = tool_skill.join("SKILL.md");
                    if !skill_file.exists() {
                        let _ = fs::create_dir_all(&tool_skill);
                        let manifest = format!(
                            "---\nname: {}\ndescription: Toksi consolidated {} intelligence tool for AI coding agents.\nlicense: Apache-2.0\n---\n\n# {}\n\nToksi in-process tool wire for {}.\n",
                            tool, tool, tool, tool
                        );
                        let _ = fs::write(&skill_file, manifest);
                        wired.push(PathBuf::from(normalize_path(&skill_file.to_string_lossy())));
                    }
                }
            }
        }
    }

    Ok(wired)
}

/// Synchronizes rules across all recognized workspace configuration files.
/// Targets 17 supported agents and open conventions:
/// - `.agents/` (OpenAgents: `agents.md`, `system-prompt.md`, `skills/`)
/// - `.gemini/` (Antigravity: `GEMINI.md`, `rules/`, `skills/`)
/// - `.copilot/` & `.github/` (Copilot: `copilot-instructions.md`, `skills/`)
/// - `.claude/` (Claude Code: `CLAUDE.md`, `rules/`, `skills/`)
/// - `.codex/` (Codex: `AGENTS.md`, `rules/`)
/// - `.cursor/` (Cursor: `.cursorrules`, `rules/*.mdc`, `skills/`)
/// - `.grok/` (Grok: `skills/`, `rules/`)
pub fn sync_workspace_rules(workspace: impl AsRef<Path>, rules: &[RuleKind]) -> Result<SyncReport, DynError> {
    let ws = workspace.as_ref();
    let norm_ws = normalize_path(&ws.to_string_lossy());
    let mut report = SyncReport {
        workspace: PathBuf::from(norm_ws),
        ..Default::default()
    };

    // 1. Claude Code: CLAUDE.md and .claude/
    let claude_file = ws.join("CLAUDE.md");
    match patch_claude_md(ws, rules) {
        Ok(true) => report.record_updated(claude_file),
        Ok(false) => report.record_up_to_date(claude_file),
        Err(e) => report.record_error(format!("CLAUDE.md: {}", e)),
    }

    // .claude/rules/ and .claude/skills/
    let claude_rules_dir = ws.join(".claude").join("rules");
    let _ = fs::create_dir_all(&claude_rules_dir);
    let claude_rules_file = claude_rules_dir.join("rules.md");
    match patch_claude_md(&claude_rules_file, rules) {
        Ok(true) => report.record_updated(claude_rules_file),
        Ok(false) => report.record_up_to_date(claude_rules_file),
        Err(e) => report.record_error(format!(".claude/rules/rules.md: {}", e)),
    }
    match patch_skills_dir(ws.join(".claude"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".claude/skills: {}", e)),
    }

    // 2. OpenAgents standard: AGENTS.md, .agents/agents.md, .agents/system-prompt.md, .agents/skills/
    let agents_file = ws.join("AGENTS.md");
    match patch_agents_md(ws, rules) {
        Ok(true) => report.record_updated(agents_file),
        Ok(false) => report.record_up_to_date(agents_file),
        Err(e) => report.record_error(format!("AGENTS.md: {}", e)),
    }

    let agents_rules_file = ws.join(".agents").join("rules.md");
    match patch_agents_rules_md(ws, rules) {
        Ok(true) => report.record_updated(agents_rules_file),
        Ok(false) => report.record_up_to_date(agents_rules_file),
        Err(e) => report.record_error(format!(".agents/rules.md: {}", e)),
    }

    let agents_md_file = ws.join(".agents").join("agents.md");
    match patch_agents_md(&agents_md_file, rules) {
        Ok(true) => report.record_updated(agents_md_file),
        Ok(false) => report.record_up_to_date(agents_md_file),
        Err(e) => report.record_error(format!(".agents/agents.md: {}", e)),
    }

    let sys_prompt_file = ws.join(".agents").join("system-prompt.md");
    match patch_agents_md(&sys_prompt_file, rules) {
        Ok(true) => report.record_updated(sys_prompt_file),
        Ok(false) => report.record_up_to_date(sys_prompt_file),
        Err(e) => report.record_error(format!(".agents/system-prompt.md: {}", e)),
    }

    match patch_skills_dir(ws.join(".agents"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".agents/skills: {}", e)),
    }

    // 3. Gemini / Antigravity: GEMINI.md, .gemini/rules/, .gemini/skills/
    let gemini_file = ws.join("GEMINI.md");
    match patch_gemini_md(ws, rules) {
        Ok(true) => report.record_updated(gemini_file),
        Ok(false) => report.record_up_to_date(gemini_file),
        Err(e) => report.record_error(format!("GEMINI.md: {}", e)),
    }

    let gemini_rules_dir = ws.join(".gemini").join("rules");
    let _ = fs::create_dir_all(&gemini_rules_dir);
    let gemini_rules_file = gemini_rules_dir.join("rules.md");
    match patch_gemini_md(&gemini_rules_file, rules) {
        Ok(true) => report.record_updated(gemini_rules_file),
        Ok(false) => report.record_up_to_date(gemini_rules_file),
        Err(e) => report.record_error(format!(".gemini/rules/rules.md: {}", e)),
    }

    match patch_skills_dir(ws.join(".gemini"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".gemini/skills: {}", e)),
    }

    // 4. GitHub Copilot: .github/copilot-instructions.md, .copilot/skills/, .github/skills/
    let copilot_file = ws.join(".github").join("copilot-instructions.md");
    match patch_copilot_md(ws, rules) {
        Ok(true) => report.record_updated(copilot_file),
        Ok(false) => report.record_up_to_date(copilot_file),
        Err(e) => report.record_error(format!(".github/copilot-instructions.md: {}", e)),
    }

    match patch_skills_dir(ws.join(".copilot"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".copilot/skills: {}", e)),
    }

    match patch_skills_dir(ws.join(".github"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".github/skills: {}", e)),
    }

    // 5. Cline: .clinerules
    let cline_file = ws.join(".clinerules");
    match patch_cline_rules(ws, rules) {
        Ok(true) => report.record_updated(cline_file),
        Ok(false) => report.record_up_to_date(cline_file),
        Err(e) => report.record_error(format!(".clinerules: {}", e)),
    }

    // 6. Codex: CODEX.md, .codex/rules/
    let codex_file = ws.join("CODEX.md");
    match patch_codex_md(ws, rules) {
        Ok(true) => report.record_updated(codex_file),
        Ok(false) => report.record_up_to_date(codex_file),
        Err(e) => report.record_error(format!("CODEX.md: {}", e)),
    }

    let codex_rules_dir = ws.join(".codex").join("rules");
    let _ = fs::create_dir_all(&codex_rules_dir);
    let codex_rules_file = codex_rules_dir.join("rules.md");
    match patch_codex_md(&codex_rules_file, rules) {
        Ok(true) => report.record_updated(codex_rules_file),
        Ok(false) => report.record_up_to_date(codex_rules_file),
        Err(e) => report.record_error(format!(".codex/rules/rules.md: {}", e)),
    }

    // 7. Cursor: .cursorrules, .cursor/rules/*.mdc, .cursor/skills/
    let cursorrules_file = ws.join(".cursorrules");
    match patch_cursorrules(ws, rules) {
        Ok(true) => report.record_updated(cursorrules_file),
        Ok(false) => report.record_up_to_date(cursorrules_file),
        Err(e) => report.record_error(format!(".cursorrules: {}", e)),
    }

    match patch_cursor_rules(ws, rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".cursor/rules: {}", e)),
    }

    match patch_skills_dir(ws.join(".cursor"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".cursor/skills: {}", e)),
    }

    // 8. Grok: .grok/rules/, .grok/skills/
    let grok_rules_dir = ws.join(".grok").join("rules");
    let _ = fs::create_dir_all(&grok_rules_dir);
    let grok_rules_file = grok_rules_dir.join("AGENTS.md");
    match patch_agents_md(&grok_rules_file, rules) {
        Ok(true) => report.record_updated(grok_rules_file),
        Ok(false) => report.record_up_to_date(grok_rules_file),
        Err(e) => report.record_error(format!(".grok/rules/AGENTS.md: {}", e)),
    }

    match patch_skills_dir(ws.join(".grok"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!(".grok/skills: {}", e)),
    }

    // Wire MCP settings to skills/<tool>
    match wire_mcp_skills(ws, MANAGED_MCP_TOOLS) {
        Ok(wired_paths) => {
            for wp in wired_paths {
                report.record_updated(wp);
            }
        }
        Err(e) => report.record_error(format!("wire_mcp_skills: {}", e)),
    }

    Ok(report)
}

/// Synchronizes rules into global user home configuration files.
pub fn sync_global_rules(rules: &[RuleKind]) -> Result<SyncReport, DynError> {
    let home_dir = home();
    let home_path = PathBuf::from(&home_dir);
    let norm_home = normalize_path(&home_path.to_string_lossy());
    let mut report = SyncReport {
        workspace: PathBuf::from(norm_home),
        ..Default::default()
    };

    if home_dir.is_empty() {
        report.record_error("Cannot resolve user home directory");
        return Ok(report);
    }

    // Claude global (~/.claude/CLAUDE.md and ~/.claude/skills/)
    let claude_global = home_path.join(".claude").join("CLAUDE.md");
    match patch_claude_md(&claude_global, rules) {
        Ok(true) => report.record_updated(claude_global),
        Ok(false) => report.record_up_to_date(claude_global),
        Err(e) => report.record_error(format!("~/.claude/CLAUDE.md: {}", e)),
    }
    match patch_skills_dir(home_path.join(".claude"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!("~/.claude/skills: {}", e)),
    }

    // Gemini / Antigravity global (~/.gemini/rules.md, ~/.gemini/skills/)
    let gemini_global = home_path.join(".gemini").join("rules.md");
    match patch_gemini_md(&gemini_global, rules) {
        Ok(true) => report.record_updated(gemini_global),
        Ok(false) => report.record_up_to_date(gemini_global),
        Err(e) => report.record_error(format!("~/.gemini/rules.md: {}", e)),
    }
    match patch_skills_dir(home_path.join(".gemini"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!("~/.gemini/skills: {}", e)),
    }

    // Codex global (~/.codex/instructions.md, ~/.codex/rules/)
    let codex_global = home_path.join(".codex").join("instructions.md");
    match patch_codex_md(&codex_global, rules) {
        Ok(true) => report.record_updated(codex_global),
        Ok(false) => report.record_up_to_date(codex_global),
        Err(e) => report.record_error(format!("~/.codex/instructions.md: {}", e)),
    }

    // Grok global (~/.grok/rules/AGENTS.md, ~/.grok/skills/)
    let grok_global = home_path.join(".grok").join("rules").join("AGENTS.md");
    let _ = fs::create_dir_all(home_path.join(".grok").join("rules"));
    match patch_agents_md(&grok_global, rules) {
        Ok(true) => report.record_updated(grok_global),
        Ok(false) => report.record_up_to_date(grok_global),
        Err(e) => report.record_error(format!("~/.grok/rules/AGENTS.md: {}", e)),
    }
    match patch_skills_dir(home_path.join(".grok"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!("~/.grok/skills: {}", e)),
    }

    // OpenAgents global (~/.agents/agents.md, ~/.agents/skills/)
    let openagents_global = home_path.join(".agents").join("agents.md");
    let _ = fs::create_dir_all(home_path.join(".agents"));
    match patch_agents_md(&openagents_global, rules) {
        Ok(true) => report.record_updated(openagents_global),
        Ok(false) => report.record_up_to_date(openagents_global),
        Err(e) => report.record_error(format!("~/.agents/agents.md: {}", e)),
    }
    match patch_skills_dir(home_path.join(".agents"), rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!("~/.agents/skills: {}", e)),
    }

    // Cline global (~/.cline/rules.md)
    let cline_global = home_path.join(".cline").join("rules.md");
    match patch_cline_rules(&cline_global, rules) {
        Ok(true) => report.record_updated(cline_global),
        Ok(false) => report.record_up_to_date(cline_global),
        Err(e) => report.record_error(format!("~/.cline/rules.md: {}", e)),
    }

    // Wire MCP settings in user home
    match wire_mcp_skills(&home_path, MANAGED_MCP_TOOLS) {
        Ok(wired_paths) => {
            for wp in wired_paths {
                report.record_updated(wp);
            }
        }
        Err(e) => report.record_error(format!("wire_mcp_skills (global): {}", e)),
    }

    Ok(report)
}

/// Synchronizes rules across workspace and optionally global user home directories.
pub fn sync_all_rules(
    workspace: impl AsRef<Path>,
    include_global: bool,
    rules: &[RuleKind],
) -> Result<SyncReport, DynError> {
    let mut ws_report = sync_workspace_rules(workspace, rules)?;
    if include_global {
        let global_report = sync_global_rules(rules)?;
        ws_report.updated_files.extend(global_report.updated_files);
        ws_report.up_to_date_files.extend(global_report.up_to_date_files);
        ws_report.errors.extend(global_report.errors);
    }
    Ok(ws_report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sync_workspace_rules() {
        let tmp = match tempdir() {
            Ok(d) => d,
            Err(e) => panic!("tempdir creation failed: {}", e),
        };
        let report = match sync_workspace_rules(tmp.path(), RuleKind::all()) {
            Ok(r) => r,
            Err(e) => panic!("sync_workspace_rules failed: {}", e),
        };

        assert!(report.is_success());
        assert!(!report.updated_files.is_empty());
        assert!(tmp.path().join("CLAUDE.md").exists());
        assert!(tmp.path().join("AGENTS.md").exists());
        assert!(tmp.path().join("GEMINI.md").exists());
        assert!(tmp.path().join(".github").join("copilot-instructions.md").exists());
        assert!(tmp.path().join(".clinerules").exists());
        assert!(tmp.path().join("CODEX.md").exists());
        assert!(tmp.path().join(".cursor").join("rules").join("principles.mdc").exists());
        assert!(tmp.path().join(".agents").join("skills").join("principles").join("SKILL.md").exists());
        assert!(tmp.path().join(".grok").join("rules").join("AGENTS.md").exists());

        // Second sync should report updated items or up to date
        let report2 = match sync_workspace_rules(tmp.path(), RuleKind::all()) {
            Ok(r) => r,
            Err(e) => panic!("sync_workspace_rules second call failed: {}", e),
        };
        assert!(report2.is_success());
        assert!(report2.up_to_date_files.len() >= 6);
    }

    #[test]
    fn test_wire_mcp_skills() {
        let tmp = match tempdir() {
            Ok(d) => d,
            Err(e) => panic!("tempdir creation failed: {}", e),
        };
        let agent_gemini = tmp.path().join(".gemini");
        let mcp_dir = agent_gemini.join("mcp").join("codegraph");
        let _ = fs::create_dir_all(&mcp_dir);
        let _ = fs::write(mcp_dir.join("instructions.md"), "# CodeGraph Instructions\n");

        let wired = match wire_mcp_skills(tmp.path(), &["codegraph"]) {
            Ok(w) => w,
            Err(e) => panic!("wire_mcp_skills failed: {}", e),
        };
        assert!(!wired.is_empty());
        assert!(agent_gemini.join("skills").join("codegraph").join("instructions.md").exists());
    }
}
