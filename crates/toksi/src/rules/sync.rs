// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/sync.rs
// created: 2026-11-05
// lastModified: 2026-11-09

//! Universal rules, skills & instructions synchronizer for workspace and agent configuration homes.
//!
//! Synchronizes in-tree coding guidelines (Principles, Caveman, Ponytail) across
//! workspace directories and user home config paths for all 17 supported agents and open conventions:
//! - Antigravity / Gemini (`GEMINI.md`, `.gemini/rules.md`, `.gemini/instructions.md`, `.gemini/skills/`, `~/.gemini/`)
//! - GitHub Copilot (`.copilot/`, `.github/copilot-instructions.md`, `.copilot/skills/`, `.github/skills/`, `~/.copilot/`)
//! - Claude Code (`CLAUDE.md`, `.claude/rules/rules.md`, `.claude/instructions.md`, `.claude/skills/`, `~/.claude/`)
//! - Codex (`CODEX.md`, `AGENTS.md`, `.codex/rules/rules.md`, `.codex/instructions.md`, `~/.codex/`)
//! - Cursor (`.cursorrules`, `.cursor/rules/*.mdc`, `.cursor/skills/`, `~/.cursor/`)
//! - Grok (`.grok/rules/AGENTS.md`, `.grok/rules.md`, `.grok/instructions.md`, `.grok/skills/`, `~/.grok/`)
//! - OpenAgents (`AGENTS.md`, `.agents/agents.md`, `.agents/rules.md`, `.agents/instructions.md`, `.agents/skills/`, `~/.agents/`)
//! - Cline (`.clinerules`, `.cline/rules.md`, `.cline/instructions.md`, `~/.cline/`)
//! - Additional 9 agents: Droid (.factory), OpenCode, OpenCode2, Kilo, OMP, Pi, Amp, Kimi, ZCode, DSH.
//!
//! Synchronizes any matching `rules*.md`, `skills*.md`, `instructions*.md`, and `agents*.md` across
//! workspace roots and user configuration homes.
//! Also wires tool MCP settings into `skills/<tool>`.

use std::fs;
use std::path::{Path, PathBuf};
use crate::rules::{
    patch_agents_md, patch_agents_rules_md, patch_claude_md, patch_cline_rules,
    patch_codex_md, patch_copilot_md, patch_cursor_rules, patch_cursorrules,
    patch_gemini_md, patch_markdown_file, patch_skills_dir, RuleKind,
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

/// All 17 supported agent configuration directories.
pub const SUPPORTED_AGENT_DIRS: &[&str] = &[
    ".agents",
    ".gemini",
    ".copilot",
    ".github",
    ".claude",
    ".codex",
    ".cursor",
    ".grok",
    ".opencode",
    ".opencode2",
    ".cline",
    ".droid",
    ".factory",
    ".kilo",
    ".omp",
    ".pi",
    ".amp",
    ".kimi",
    ".zcode",
    ".dsh",
];

/// Helper determining if a file name matches convention targets:
/// `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md`.
pub fn matches_sync_target_filename(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if !lower.ends_with(".md") && lower != ".cursorrules" && lower != ".clinerules" {
        return false;
    }

    lower.starts_with("rules")
        || lower.starts_with("skills")
        || lower.starts_with("instructions")
        || lower.contains("instructions.md")
        || lower.starts_with("agents")
        || lower.contains("agents.md")
        || lower == "gemini.md"
        || lower == "claude.md"
        || lower == "codex.md"
        || lower == "system-prompt.md"
        || lower == ".cursorrules"
        || lower == ".clinerules"
}

/// Applies in-tree rules patch to a single markdown target file and logs into the report.
pub fn sync_markdown_target(report: &mut SyncReport, file_path: &Path, rules: &[RuleKind]) {
    match patch_markdown_file(file_path, rules) {
        Ok(true) => report.record_updated(file_path),
        Ok(false) => report.record_up_to_date(file_path),
        Err(e) => report.record_error(format!("{}: {}", file_path.display(), e)),
    }
}

/// Wires tool settings/instructions from agent `mcp` directories to `skills/<tool>`.
pub fn wire_mcp_skills(base_dir: impl AsRef<Path>, tools: &[&str]) -> Result<Vec<PathBuf>, DynError> {
    let base = base_dir.as_ref();
    let mut wired = Vec::new();

    for agent in SUPPORTED_AGENT_DIRS {
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
            } else if agent_path.exists() {
                // Synthesize standalone skill manifest for tool if agent directory exists
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

    Ok(wired)
}

/// Synchronizes rules across all recognized workspace configuration files.
/// Targets all 17 supported agents and open conventions.
pub fn sync_workspace_rules(workspace: impl AsRef<Path>, rules: &[RuleKind]) -> Result<SyncReport, DynError> {
    let ws = workspace.as_ref();
    let norm_ws = normalize_path(&ws.to_string_lossy());
    let mut report = SyncReport {
        workspace: PathBuf::from(norm_ws),
        ..Default::default()
    };

    // 1. Scan workspace root for any matching convention files:
    // `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md`, `CLAUDE.md`, `GEMINI.md`, etc.
    if let Ok(entries) = fs::read_dir(ws) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if matches_sync_target_filename(&name) {
                    sync_markdown_target(&mut report, &path, rules);
                }
            }
        }
    }

    // 2. Primary root files & standard agent directories guarantee
    // Claude
    let claude_file = ws.join("CLAUDE.md");
    match patch_claude_md(ws, rules) {
        Ok(true) => report.record_updated(&claude_file),
        Ok(false) => report.record_up_to_date(&claude_file),
        Err(e) => report.record_error(format!("CLAUDE.md: {}", e)),
    }
    let claude_rules_dir = ws.join(".claude").join("rules");
    let _ = fs::create_dir_all(&claude_rules_dir);
    let claude_rules_file = claude_rules_dir.join("rules.md");
    match patch_claude_md(&claude_rules_file, rules) {
        Ok(true) => report.record_updated(&claude_rules_file),
        Ok(false) => report.record_up_to_date(&claude_rules_file),
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

    // OpenAgents
    let agents_file = ws.join("AGENTS.md");
    match patch_agents_md(ws, rules) {
        Ok(true) => report.record_updated(&agents_file),
        Ok(false) => report.record_up_to_date(&agents_file),
        Err(e) => report.record_error(format!("AGENTS.md: {}", e)),
    }
    let agents_rules_file = ws.join(".agents").join("rules.md");
    match patch_agents_rules_md(ws, rules) {
        Ok(true) => report.record_updated(&agents_rules_file),
        Ok(false) => report.record_up_to_date(&agents_rules_file),
        Err(e) => report.record_error(format!(".agents/rules.md: {}", e)),
    }
    let agents_md_file = ws.join(".agents").join("agents.md");
    match patch_agents_md(&agents_md_file, rules) {
        Ok(true) => report.record_updated(&agents_md_file),
        Ok(false) => report.record_up_to_date(&agents_md_file),
        Err(e) => report.record_error(format!(".agents/agents.md: {}", e)),
    }
    let sys_prompt_file = ws.join(".agents").join("system-prompt.md");
    match patch_agents_md(&sys_prompt_file, rules) {
        Ok(true) => report.record_updated(&sys_prompt_file),
        Ok(false) => report.record_up_to_date(&sys_prompt_file),
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

    // Gemini / Antigravity
    let gemini_file = ws.join("GEMINI.md");
    match patch_gemini_md(ws, rules) {
        Ok(true) => report.record_updated(&gemini_file),
        Ok(false) => report.record_up_to_date(&gemini_file),
        Err(e) => report.record_error(format!("GEMINI.md: {}", e)),
    }
    let gemini_rules_dir = ws.join(".gemini").join("rules");
    let _ = fs::create_dir_all(&gemini_rules_dir);
    let gemini_rules_file = gemini_rules_dir.join("rules.md");
    match patch_gemini_md(&gemini_rules_file, rules) {
        Ok(true) => report.record_updated(&gemini_rules_file),
        Ok(false) => report.record_up_to_date(&gemini_rules_file),
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

    // Codex
    let codex_file = ws.join("CODEX.md");
    match patch_codex_md(ws, rules) {
        Ok(true) => report.record_updated(&codex_file),
        Ok(false) => report.record_up_to_date(&codex_file),
        Err(e) => report.record_error(format!("CODEX.md: {}", e)),
    }
    let codex_rules_dir = ws.join(".codex").join("rules");
    let _ = fs::create_dir_all(&codex_rules_dir);
    let codex_rules_file = codex_rules_dir.join("rules.md");
    match patch_codex_md(&codex_rules_file, rules) {
        Ok(true) => report.record_updated(&codex_rules_file),
        Ok(false) => report.record_up_to_date(&codex_rules_file),
        Err(e) => report.record_error(format!(".codex/rules/rules.md: {}", e)),
    }

    // Cline
    let cline_file = ws.join(".clinerules");
    match patch_cline_rules(ws, rules) {
        Ok(true) => report.record_updated(&cline_file),
        Ok(false) => report.record_up_to_date(&cline_file),
        Err(e) => report.record_error(format!(".clinerules: {}", e)),
    }

    // Cursor
    let cursorrules_file = ws.join(".cursorrules");
    match patch_cursorrules(ws, rules) {
        Ok(true) => report.record_updated(&cursorrules_file),
        Ok(false) => report.record_up_to_date(&cursorrules_file),
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

    // Copilot
    let copilot_file = ws.join(".github").join("copilot-instructions.md");
    match patch_copilot_md(ws, rules) {
        Ok(true) => report.record_updated(&copilot_file),
        Ok(false) => report.record_up_to_date(&copilot_file),
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

    // Grok
    let grok_rules_dir = ws.join(".grok").join("rules");
    let _ = fs::create_dir_all(&grok_rules_dir);
    let grok_rules_file = grok_rules_dir.join("AGENTS.md");
    match patch_agents_md(&grok_rules_file, rules) {
        Ok(true) => report.record_updated(&grok_rules_file),
        Ok(false) => report.record_up_to_date(&grok_rules_file),
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

    // 3. For all 17 agent directories in workspace, sync rules, skills, and instructions
    for agent in SUPPORTED_AGENT_DIRS {
        let agent_dir = ws.join(agent);
        if agent_dir.exists() {
            // Scan agent directory for matching files
            if let Ok(entries) = fs::read_dir(&agent_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if matches_sync_target_filename(&name) {
                            sync_markdown_target(&mut report, &path, rules);
                        }
                    }
                }
            }

            // Sync skills directory
            match patch_skills_dir(&agent_dir, rules) {
                Ok(paths) => {
                    for p in paths {
                        report.record_updated(p);
                    }
                }
                Err(e) => report.record_error(format!("{}/skills: {}", agent, e)),
            }

            // Guarantee agent rules.md and instructions.md exist
            let agent_rules = agent_dir.join("rules.md");
            sync_markdown_target(&mut report, &agent_rules, rules);

            let agent_instructions = agent_dir.join("instructions.md");
            sync_markdown_target(&mut report, &agent_instructions, rules);
        }
    }

    // 4. Wire MCP tool definitions to skills/<tool>
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

/// Synchronizes rules into global user home configuration files across all 17 agents.
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

    // 1. Primary agent home files
    // Claude global (~/.claude/CLAUDE.md)
    let claude_global = home_path.join(".claude").join("CLAUDE.md");
    match patch_claude_md(&claude_global, rules) {
        Ok(true) => report.record_updated(&claude_global),
        Ok(false) => report.record_up_to_date(&claude_global),
        Err(e) => report.record_error(format!("~/.claude/CLAUDE.md: {}", e)),
    }

    // Gemini / Antigravity global (~/.gemini/rules.md, ~/.gemini/GEMINI.md)
    let gemini_global = home_path.join(".gemini").join("rules.md");
    match patch_gemini_md(&gemini_global, rules) {
        Ok(true) => report.record_updated(&gemini_global),
        Ok(false) => report.record_up_to_date(&gemini_global),
        Err(e) => report.record_error(format!("~/.gemini/rules.md: {}", e)),
    }

    // Codex global (~/.codex/instructions.md, ~/.codex/CODEX.md)
    let codex_global = home_path.join(".codex").join("instructions.md");
    match patch_codex_md(&codex_global, rules) {
        Ok(true) => report.record_updated(&codex_global),
        Ok(false) => report.record_up_to_date(&codex_global),
        Err(e) => report.record_error(format!("~/.codex/instructions.md: {}", e)),
    }

    // Grok global (~/.grok/rules/AGENTS.md)
    let grok_global = home_path.join(".grok").join("rules").join("AGENTS.md");
    let _ = fs::create_dir_all(home_path.join(".grok").join("rules"));
    match patch_agents_md(&grok_global, rules) {
        Ok(true) => report.record_updated(&grok_global),
        Ok(false) => report.record_up_to_date(&grok_global),
        Err(e) => report.record_error(format!("~/.grok/rules/AGENTS.md: {}", e)),
    }

    // OpenAgents global (~/.agents/agents.md)
    let openagents_global = home_path.join(".agents").join("agents.md");
    let _ = fs::create_dir_all(home_path.join(".agents"));
    match patch_agents_md(&openagents_global, rules) {
        Ok(true) => report.record_updated(&openagents_global),
        Ok(false) => report.record_up_to_date(&openagents_global),
        Err(e) => report.record_error(format!("~/.agents/agents.md: {}", e)),
    }

    // Cline global (~/.cline/rules.md)
    let cline_global = home_path.join(".cline").join("rules.md");
    match patch_cline_rules(&cline_global, rules) {
        Ok(true) => report.record_updated(&cline_global),
        Ok(false) => report.record_up_to_date(&cline_global),
        Err(e) => report.record_error(format!("~/.cline/rules.md: {}", e)),
    }

    // 2. Iterate across all 17 agent directories in user home
    for agent in SUPPORTED_AGENT_DIRS {
        let agent_home = home_path.join(agent);
        if agent_home.exists() {
            // Scan home agent dir for matching convention files
            if let Ok(entries) = fs::read_dir(&agent_home) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if matches_sync_target_filename(&name) {
                            sync_markdown_target(&mut report, &path, rules);
                        }
                    }
                }
            }

            // Sync skills
            match patch_skills_dir(&agent_home, rules) {
                Ok(paths) => {
                    for p in paths {
                        report.record_updated(p);
                    }
                }
                Err(e) => report.record_error(format!("~/{}/skills: {}", agent, e)),
            }

            // Guarantee rules.md and instructions.md
            let home_rules = agent_home.join("rules.md");
            sync_markdown_target(&mut report, &home_rules, rules);

            let home_instructions = agent_home.join("instructions.md");
            sync_markdown_target(&mut report, &home_instructions, rules);
        }
    }

    // 3. Wire MCP tool settings in user home
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

    #[test]
    fn test_matches_sync_target_filename() {
        assert!(matches_sync_target_filename("rules.md"));
        assert!(matches_sync_target_filename("rules_dev.md"));
        assert!(matches_sync_target_filename("skills.md"));
        assert!(matches_sync_target_filename("instructions.md"));
        assert!(matches_sync_target_filename("copilot-instructions.md"));
        assert!(matches_sync_target_filename("agents.md"));
        assert!(matches_sync_target_filename("AGENTS.md"));
        assert!(matches_sync_target_filename("CLAUDE.md"));
        assert!(matches_sync_target_filename("GEMINI.md"));
        assert!(matches_sync_target_filename(".cursorrules"));
        assert!(matches_sync_target_filename(".clinerules"));

        assert!(!matches_sync_target_filename("Cargo.toml"));
        assert!(!matches_sync_target_filename("main.rs"));
        assert!(!matches_sync_target_filename("package.json"));
    }
}
