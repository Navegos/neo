// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/sync.rs
// created: 2026-11-05
// lastModified: 2026-11-05

//! Universal rules & skills synchronizer for workspace and agent configuration homes.
//!
//! Synchronizes in-tree coding guidelines (Principles, Caveman, Ponytail) across
//! workspace directories and user home config paths for all 17 supported agents:
//! - Antigravity / Gemini (`GEMINI.md`, `~/.gemini/rules.md`, `.agents/rules.md`)
//! - Claude Code (`CLAUDE.md`, `~/.claude/CLAUDE.md`)
//! - Cursor (`.cursorrules`, `.cursor/rules/*.mdc`)
//! - GitHub Copilot (`.github/copilot-instructions.md`)
//! - Cline (`.clinerules`, `~/.cline/rules.md`)
//! - OpenCode (`AGENTS.md`)
//! - Codex (`CODEX.md`, `~/.codex/instructions.md`)
//! - Droid, Grok, Kilo, OMP, Pi, Amp, Kimi, ZCode, DSH (`AGENTS.md`, `.agents/rules.md`)

use std::path::{Path, PathBuf};
use crate::rules::{
    patch_agents_md, patch_agents_rules_md, patch_claude_md, patch_cline_rules,
    patch_codex_md, patch_copilot_md, patch_cursor_rules, patch_cursorrules,
    patch_gemini_md, RuleKind,
};
use crate::util::paths::home;

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
}

/// Synchronizes rules across all recognized workspace configuration files.
pub fn sync_workspace_rules(workspace: impl AsRef<Path>, rules: &[RuleKind]) -> Result<SyncReport, DynError> {
    let ws = workspace.as_ref();
    let mut report = SyncReport {
        workspace: ws.to_path_buf(),
        ..Default::default()
    };

    let claude_file = ws.join("CLAUDE.md");
    match patch_claude_md(ws, rules) {
        Ok(true) => report.updated_files.push(claude_file),
        Ok(false) => report.up_to_date_files.push(claude_file),
        Err(e) => report.errors.push(format!("CLAUDE.md: {}", e)),
    }

    let agents_file = ws.join("AGENTS.md");
    match patch_agents_md(ws, rules) {
        Ok(true) => report.updated_files.push(agents_file),
        Ok(false) => report.up_to_date_files.push(agents_file),
        Err(e) => report.errors.push(format!("AGENTS.md: {}", e)),
    }

    let gemini_file = ws.join("GEMINI.md");
    match patch_gemini_md(ws, rules) {
        Ok(true) => report.updated_files.push(gemini_file),
        Ok(false) => report.up_to_date_files.push(gemini_file),
        Err(e) => report.errors.push(format!("GEMINI.md: {}", e)),
    }

    let copilot_file = ws.join(".github").join("copilot-instructions.md");
    match patch_copilot_md(ws, rules) {
        Ok(true) => report.updated_files.push(copilot_file),
        Ok(false) => report.up_to_date_files.push(copilot_file),
        Err(e) => report.errors.push(format!(".github/copilot-instructions.md: {}", e)),
    }

    let cline_file = ws.join(".clinerules");
    match patch_cline_rules(ws, rules) {
        Ok(true) => report.updated_files.push(cline_file),
        Ok(false) => report.up_to_date_files.push(cline_file),
        Err(e) => report.errors.push(format!(".clinerules: {}", e)),
    }

    let codex_file = ws.join("CODEX.md");
    match patch_codex_md(ws, rules) {
        Ok(true) => report.updated_files.push(codex_file),
        Ok(false) => report.up_to_date_files.push(codex_file),
        Err(e) => report.errors.push(format!("CODEX.md: {}", e)),
    }

    let agents_rules_file = ws.join(".agents").join("rules.md");
    match patch_agents_rules_md(ws, rules) {
        Ok(true) => report.updated_files.push(agents_rules_file),
        Ok(false) => report.up_to_date_files.push(agents_rules_file),
        Err(e) => report.errors.push(format!(".agents/rules.md: {}", e)),
    }

    let cursorrules_file = ws.join(".cursorrules");
    match patch_cursorrules(ws, rules) {
        Ok(true) => report.updated_files.push(cursorrules_file),
        Ok(false) => report.up_to_date_files.push(cursorrules_file),
        Err(e) => report.errors.push(format!(".cursorrules: {}", e)),
    }

    match patch_cursor_rules(ws, rules) {
        Ok(paths) => {
            for p in paths {
                report.updated_files.push(p);
            }
        }
        Err(e) => report.errors.push(format!(".cursor/rules: {}", e)),
    }

    Ok(report)
}

/// Synchronizes rules into global user home configuration files.
pub fn sync_global_rules(rules: &[RuleKind]) -> Result<SyncReport, DynError> {
    let home_dir = home();
    let home_path = PathBuf::from(&home_dir);
    let mut report = SyncReport {
        workspace: home_path.clone(),
        ..Default::default()
    };

    if home_dir.is_empty() {
        report.errors.push("Cannot resolve user home directory".to_string());
        return Ok(report);
    }

    // Claude global (~/.claude/CLAUDE.md)
    let claude_global = home_path.join(".claude").join("CLAUDE.md");
    match patch_claude_md(&claude_global, rules) {
        Ok(true) => report.updated_files.push(claude_global),
        Ok(false) => report.up_to_date_files.push(claude_global),
        Err(e) => report.errors.push(format!("~/.claude/CLAUDE.md: {}", e)),
    }

    // Gemini / Antigravity global (~/.gemini/rules.md)
    let gemini_global = home_path.join(".gemini").join("rules.md");
    match patch_gemini_md(&gemini_global, rules) {
        Ok(true) => report.updated_files.push(gemini_global),
        Ok(false) => report.up_to_date_files.push(gemini_global),
        Err(e) => report.errors.push(format!("~/.gemini/rules.md: {}", e)),
    }

    // Codex global (~/.codex/instructions.md)
    let codex_global = home_path.join(".codex").join("instructions.md");
    match patch_codex_md(&codex_global, rules) {
        Ok(true) => report.updated_files.push(codex_global),
        Ok(false) => report.up_to_date_files.push(codex_global),
        Err(e) => report.errors.push(format!("~/.codex/instructions.md: {}", e)),
    }

    // Cline global (~/.cline/rules.md)
    let cline_global = home_path.join(".cline").join("rules.md");
    match patch_cline_rules(&cline_global, rules) {
        Ok(true) => report.updated_files.push(cline_global),
        Ok(false) => report.up_to_date_files.push(cline_global),
        Err(e) => report.errors.push(format!("~/.cline/rules.md: {}", e)),
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
        let tmp = tempdir().unwrap();
        let report = sync_workspace_rules(tmp.path(), RuleKind::all()).unwrap();

        assert!(report.is_success());
        assert!(!report.updated_files.is_empty());
        assert!(tmp.path().join("CLAUDE.md").exists());
        assert!(tmp.path().join("AGENTS.md").exists());
        assert!(tmp.path().join("GEMINI.md").exists());
        assert!(tmp.path().join(".github").join("copilot-instructions.md").exists());
        assert!(tmp.path().join(".clinerules").exists());
        assert!(tmp.path().join("CODEX.md").exists());
        assert!(tmp.path().join(".cursor").join("rules").join("principles.mdc").exists());

        // Second sync should report them as up to date
        let report2 = sync_workspace_rules(tmp.path(), RuleKind::all()).unwrap();
        assert!(report2.is_success());
        assert!(report2.up_to_date_files.len() >= 6);
    }
}

