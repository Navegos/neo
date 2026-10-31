// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rules/mod.rs

//! In-tree rules and skills engine for AI coding agents.
//!
//! Provides embedded, zero-dependency guidelines compiled directly into the
//! tokless binary with `include_str!`, eliminating external repository clone links.
//!
//! Exposes functions to inject, patch, and verify instructions across agent
//! configs (`CLAUDE.md`, `.cursor/rules/`, `AGENTS.md`, and custom agent paths).

pub mod cli;
pub mod embedded;

use std::fs;
use std::path::{Path, PathBuf};

pub use cli::run_rules_cli;
pub use embedded::caveman as caveman_rule;
pub use embedded::ponytail as ponytail_rule;
pub use embedded::principles as principles_rule;

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

/// In-tree rule types supported by Tokless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleKind {
    Principles,
    Caveman,
    Ponytail,
}

impl RuleKind {
    /// Returns all available in-tree rule kinds.
    pub const fn all() -> &'static [RuleKind] {
        &[RuleKind::Principles, RuleKind::Caveman, RuleKind::Ponytail]
    }

    /// Returns the canonical rule identifier string.
    pub const fn as_str(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::ID,
            RuleKind::Caveman => embedded::caveman::ID,
            RuleKind::Ponytail => embedded::ponytail::ID,
        }
    }

    /// Parses a rule kind from its identifier string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().trim() {
            "principles" | "karpathy" | "karpathy-guidelines" => Some(RuleKind::Principles),
            "caveman" | "response-style" => Some(RuleKind::Caveman),
            "ponytail" | "build-discipline" => Some(RuleKind::Ponytail),
            _ => None,
        }
    }

    /// Returns the title of the rule.
    pub const fn title(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::TITLE,
            RuleKind::Caveman => embedded::caveman::TITLE,
            RuleKind::Ponytail => embedded::ponytail::TITLE,
        }
    }

    /// Returns the short description of the rule.
    pub const fn description(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::DESCRIPTION,
            RuleKind::Caveman => embedded::caveman::DESCRIPTION,
            RuleKind::Ponytail => embedded::ponytail::DESCRIPTION,
        }
    }

    /// Returns the section header markdown tag (e.g. `## Principles`).
    pub const fn section_header(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::SECTION_HEADER,
            RuleKind::Caveman => embedded::caveman::SECTION_HEADER,
            RuleKind::Ponytail => embedded::ponytail::SECTION_HEADER,
        }
    }

    /// Returns the Cursor project rule file name (e.g. `principles.mdc`).
    pub const fn cursor_filename(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::CURSOR_FILENAME,
            RuleKind::Caveman => embedded::caveman::CURSOR_FILENAME,
            RuleKind::Ponytail => embedded::ponytail::CURSOR_FILENAME,
        }
    }

    /// Returns the embedded raw markdown string for the rule.
    pub fn markdown(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::markdown(),
            RuleKind::Caveman => embedded::caveman::markdown(),
            RuleKind::Ponytail => embedded::ponytail::markdown(),
        }
    }

    /// Returns the embedded Cursor `.mdc` file content with YAML frontmatter.
    pub fn cursor_rule(&self) -> &'static str {
        match self {
            RuleKind::Principles => embedded::principles::cursor_rule(),
            RuleKind::Caveman => embedded::caveman::cursor_rule(),
            RuleKind::Ponytail => embedded::ponytail::cursor_rule(),
        }
    }

    /// Returns the section body content.
    pub fn section_body(&self) -> String {
        match self {
            RuleKind::Principles => embedded::principles::section_body(),
            RuleKind::Caveman => embedded::caveman::section_body(),
            RuleKind::Ponytail => embedded::ponytail::section_body(),
        }
    }
}

impl std::fmt::Display for RuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Markdown Rendering and Patcher Helpers
// ---------------------------------------------------------------------------

/// Normalizes line endings to LF.
fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Checks whether a given markdown content already contains the rule section.
pub fn content_has_rule(content: &str, rule: RuleKind) -> bool {
    let norm = normalize_newlines(content);
    let marker = rule.section_header();
    norm.contains(marker)
}

/// Renders a list of rules into a clean markdown document.
///
/// If `user_preamble` is provided and non-empty, it is preserved at the top.
pub fn render_rules_markdown(rules: &[RuleKind], user_preamble: Option<&str>) -> String {
    let mut out = String::new();

    if let Some(preamble) = user_preamble {
        let trimmed = preamble.trim();
        if !trimmed.is_empty() {
            out.push_str(trimmed);
            out.push_str("\n\n");
        }
    }

    if !rules.is_empty() {
        out.push_str("# Agent Instructions\n\n");
        out.push_str("Apply on every coding task:\n\n");
        out.push_str("**Skills (apply):**\n");
        for rule in rules {
            match rule {
                RuleKind::Principles => out.push_str("- Principles — think, simplify, edit surgically, verify.\n"),
                RuleKind::Caveman => out.push_str("- Response Style (caveman) — terse prose, full technical accuracy.\n"),
                RuleKind::Ponytail => out.push_str("- Build Discipline (ponytail) — reuse first, write only what must exist.\n"),
            }
        }
        out.push('\n');

        for (i, rule) in rules.iter().enumerate() {
            if i > 0 {
                out.push_str("\n\n");
            }
            out.push_str(rule.section_header());
            out.push_str("\n\n");
            out.push_str(&rule.section_body());
        }
        out.push('\n');
    }

    out
}

/// Injects a rule into an existing markdown string, preserving user content.
pub fn inject_rule_into_content(content: &str, rule: RuleKind) -> String {
    if content_has_rule(content, rule) {
        return content.to_string();
    }

    let norm = normalize_newlines(content);
    let (preamble, mut existing_rules) = parse_content_sections(&norm);

    if !existing_rules.iter().any(|&r| r == rule) {
        existing_rules.push(rule);
    }

    render_rules_markdown(&existing_rules, if preamble.is_empty() { None } else { Some(&preamble) })
}

/// Removes a rule from an existing markdown string.
pub fn remove_rule_from_content(content: &str, rule: RuleKind) -> String {
    if !content_has_rule(content, rule) {
        return content.to_string();
    }

    let norm = normalize_newlines(content);
    let (preamble, mut existing_rules) = parse_content_sections(&norm);

    existing_rules.retain(|&r| r != rule);

    if existing_rules.is_empty() {
        if preamble.is_empty() {
            String::new()
        } else {
            preamble
        }
    } else {
        render_rules_markdown(&existing_rules, if preamble.is_empty() { None } else { Some(&preamble) })
    }
}

/// Parses existing sections to separate user preamble from recognized rule sections.
fn parse_content_sections(content: &str) -> (String, Vec<RuleKind>) {
    let mut detected_rules = Vec::new();
    let mut preamble_lines = Vec::new();

    let mut in_rule_section = false;
    let mut in_index_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "# Agent Instructions" {
            in_index_section = true;
            continue;
        }

        let is_any_rule_header = RuleKind::all().iter().any(|r| trimmed == r.section_header());
        if is_any_rule_header {
            in_index_section = false;
            in_rule_section = true;
            for r in RuleKind::all() {
                if trimmed == r.section_header() && !detected_rules.contains(r) {
                    detected_rules.push(*r);
                }
            }
            continue;
        }

        if in_rule_section {
            // Check if leaving rule section for another top-level user heading
            if trimmed.starts_with("# ") && !trimmed.starts_with("### ") {
                in_rule_section = false;
                preamble_lines.push(line);
            }
            continue;
        }

        if in_index_section {
            if trimmed.starts_with("## ") {
                in_index_section = false;
                // If it's a rule header, handled above on next iteration
            }
            continue;
        }

        preamble_lines.push(line);
    }

    let preamble = preamble_lines.join("\n").trim().to_string();
    (preamble, detected_rules)
}

// ---------------------------------------------------------------------------
// File and Agent Config Patching APIs
// ---------------------------------------------------------------------------

/// Resolves a config file path. If `target` is a directory, joins `default_filename`.
fn resolve_target_file(target: impl AsRef<Path>, default_filename: &str) -> PathBuf {
    let p = target.as_ref();
    if p.is_dir() {
        p.join(default_filename)
    } else if !p.exists() && p.extension().is_none() {
        // Assume directory if path has no extension and doesn't exist
        p.join(default_filename)
    } else {
        p.to_path_buf()
    }
}

/// Atomically writes content to a file, creating parent directories if needed.
fn safe_write_file(path: &Path, content: &str) -> Result<(), DynError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, content)?;
    Ok(())
}

/// Injects or patches in-tree rules into `CLAUDE.md`.
///
/// Accepts either the project workspace directory or a direct path to `CLAUDE.md`.
/// Preserves any existing user notes or custom instructions.
pub fn patch_claude_md(target: impl AsRef<Path>, rules: &[RuleKind]) -> Result<bool, DynError> {
    let file_path = resolve_target_file(target, "CLAUDE.md");
    let existing = if file_path.exists() {
        fs::read_to_string(&file_path)?
    } else {
        String::new()
    };

    let (preamble, mut current_rules) = parse_content_sections(&existing);
    let mut modified = false;

    for &r in rules {
        if !current_rules.contains(&r) {
            current_rules.push(r);
            modified = true;
        }
    }

    if modified || !file_path.exists() {
        let rendered = render_rules_markdown(&current_rules, if preamble.is_empty() { None } else { Some(&preamble) });
        safe_write_file(&file_path, &rendered)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Injects or patches in-tree rules into `AGENTS.md`.
///
/// Accepts either the project workspace directory or a direct path to `AGENTS.md`.
/// Preserves any existing user notes or custom instructions.
pub fn patch_agents_md(target: impl AsRef<Path>, rules: &[RuleKind]) -> Result<bool, DynError> {
    let file_path = resolve_target_file(target, "AGENTS.md");
    let existing = if file_path.exists() {
        fs::read_to_string(&file_path)?
    } else {
        String::new()
    };

    let (preamble, mut current_rules) = parse_content_sections(&existing);
    let mut modified = false;

    for &r in rules {
        if !current_rules.contains(&r) {
            current_rules.push(r);
            modified = true;
        }
    }

    if modified || !file_path.exists() {
        let rendered = render_rules_markdown(&current_rules, if preamble.is_empty() { None } else { Some(&preamble) });
        safe_write_file(&file_path, &rendered)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Injects or patches in-tree rules into Cursor `.cursor/rules/` directory.
///
/// Accepts either the workspace directory, `.cursor` directory, or `.cursor/rules` directory.
/// Writes each rule as an `.mdc` file with YAML frontmatter.
/// Returns the list of written rule file paths.
pub fn patch_cursor_rules(target: impl AsRef<Path>, rules: &[RuleKind]) -> Result<Vec<PathBuf>, DynError> {
    let p = target.as_ref();
    let rules_dir = if p.ends_with("rules") {
        p.to_path_buf()
    } else if p.ends_with(".cursor") {
        p.join("rules")
    } else {
        p.join(".cursor").join("rules")
    };

    fs::create_dir_all(&rules_dir)?;

    let mut written = Vec::new();
    for &rule in rules {
        let file_path = rules_dir.join(rule.cursor_filename());
        let content = rule.cursor_rule();
        fs::write(&file_path, content)?;
        written.push(file_path);
    }

    Ok(written)
}

/// Checks whether a rule is present at a specific file path.
pub fn has_rule_at_path(path: impl AsRef<Path>, rule: RuleKind) -> bool {
    let p = path.as_ref();
    if !p.exists() {
        return false;
    }
    match fs::read_to_string(p) {
        Ok(content) => content_has_rule(&content, rule),
        Err(_) => false,
    }
}

/// Injects a single rule at a specific file path, preserving other content.
pub fn inject_rule_at_path(path: impl AsRef<Path>, rule: RuleKind) -> Result<bool, DynError> {
    let p = path.as_ref();
    let existing = if p.exists() {
        fs::read_to_string(p)?
    } else {
        String::new()
    };

    if content_has_rule(&existing, rule) {
        return Ok(false);
    }

    let updated = inject_rule_into_content(&existing, rule);
    safe_write_file(p, &updated)?;
    Ok(true)
}

/// Removes a single rule from a specific file path.
pub fn remove_rule_from_path(path: impl AsRef<Path>, rule: RuleKind) -> Result<bool, DynError> {
    let p = path.as_ref();
    if !p.exists() {
        return Ok(false);
    }
    let existing = fs::read_to_string(p)?;
    if !content_has_rule(&existing, rule) {
        return Ok(false);
    }

    let updated = remove_rule_from_content(&existing, rule);
    if updated.trim().is_empty() {
        let _ = fs::remove_file(p);
    } else {
        safe_write_file(p, &updated)?;
    }
    Ok(true)
}

/// Composite report of patched agent configurations.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PatchReport {
    pub claude_patched: bool,
    pub agents_patched: bool,
    pub cursor_rules_written: Vec<PathBuf>,
}

/// Injects or patches in-tree rules into all supported agent configurations in `dir`:
/// `CLAUDE.md`, `AGENTS.md`, and `.cursor/rules/*.mdc`.
pub fn patch_all_agent_configs(dir: impl AsRef<Path>, rules: &[RuleKind]) -> Result<PatchReport, DynError> {
    let p = dir.as_ref();
    let claude_patched = patch_claude_md(p, rules)?;
    let agents_patched = patch_agents_md(p, rules)?;
    let cursor_rules_written = patch_cursor_rules(p, rules)?;
    Ok(PatchReport {
        claude_patched,
        agents_patched,
        cursor_rules_written,
    })
}

/// Status report for an agent config file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRuleStatus {
    pub target: String,
    pub present_rules: Vec<RuleKind>,
}

/// Inspects configured rules across CLAUDE.md, AGENTS.md, and .cursor/rules in a workspace.
pub fn inspect_agent_rules(dir: impl AsRef<Path>) -> Vec<ConfigRuleStatus> {
    let p = dir.as_ref();
    let mut out = Vec::new();

    // Check CLAUDE.md
    let claude_path = p.join("CLAUDE.md");
    if claude_path.exists() {
        if let Ok(content) = fs::read_to_string(&claude_path) {
            let present: Vec<RuleKind> = RuleKind::all()
                .iter()
                .copied()
                .filter(|&r| content_has_rule(&content, r))
                .collect();
            out.push(ConfigRuleStatus {
                target: "CLAUDE.md".to_string(),
                present_rules: present,
            });
        }
    }

    // Check AGENTS.md
    let agents_path = p.join("AGENTS.md");
    if agents_path.exists() {
        if let Ok(content) = fs::read_to_string(&agents_path) {
            let present: Vec<RuleKind> = RuleKind::all()
                .iter()
                .copied()
                .filter(|&r| content_has_rule(&content, r))
                .collect();
            out.push(ConfigRuleStatus {
                target: "AGENTS.md".to_string(),
                present_rules: present,
            });
        }
    }

    // Check .cursor/rules/
    let cursor_rules_dir = p.join(".cursor").join("rules");
    if cursor_rules_dir.is_dir() {
        let mut present = Vec::new();
        for &r in RuleKind::all() {
            let rule_file = cursor_rules_dir.join(r.cursor_filename());
            if rule_file.exists() {
                present.push(r);
            }
        }
        out.push(ConfigRuleStatus {
            target: ".cursor/rules/".to_string(),
            present_rules: present,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rule_kind_all_and_lookup() {
        let all = RuleKind::all();
        assert_eq!(all.len(), 3);
        assert_eq!(RuleKind::from_str("principles"), Some(RuleKind::Principles));
        assert_eq!(RuleKind::from_str("karpathy"), Some(RuleKind::Principles));
        assert_eq!(RuleKind::from_str("caveman"), Some(RuleKind::Caveman));
        assert_eq!(RuleKind::from_str("response-style"), Some(RuleKind::Caveman));
        assert_eq!(RuleKind::from_str("ponytail"), Some(RuleKind::Ponytail));
        assert_eq!(RuleKind::from_str("build-discipline"), Some(RuleKind::Ponytail));
        assert_eq!(RuleKind::from_str("nonexistent"), None);
    }

    #[test]
    fn test_rule_embedded_assets_compiled() {
        for rule in RuleKind::all() {
            assert!(!rule.as_str().is_empty());
            assert!(!rule.title().is_empty());
            assert!(!rule.description().is_empty());
            assert!(!rule.markdown().is_empty());
            assert!(!rule.cursor_rule().is_empty());
            assert!(!rule.section_body().is_empty());
            assert!(rule.cursor_filename().ends_with(".mdc"));
        }
    }

    #[test]
    fn test_inject_and_remove_rule_content() {
        let initial = "# User Notes\n\nProject specifics.\n";
        let with_principles = inject_rule_into_content(initial, RuleKind::Principles);
        assert!(content_has_rule(&with_principles, RuleKind::Principles));
        assert!(with_principles.contains("# User Notes"));

        let with_caveman = inject_rule_into_content(&with_principles, RuleKind::Caveman);
        assert!(content_has_rule(&with_caveman, RuleKind::Principles));
        assert!(content_has_rule(&with_caveman, RuleKind::Caveman));

        let without_principles = remove_rule_from_content(&with_caveman, RuleKind::Principles);
        assert!(!content_has_rule(&without_principles, RuleKind::Principles));
        assert!(content_has_rule(&without_principles, RuleKind::Caveman));
        assert!(without_principles.contains("# User Notes"));
    }

    #[test]
    fn test_patch_claude_md() {
        let tmp = tempdir().unwrap();
        let claude_path = tmp.path().join("CLAUDE.md");

        let patched = patch_claude_md(&claude_path, &[RuleKind::Principles, RuleKind::Ponytail]).unwrap();
        assert!(patched);
        assert!(claude_path.exists());

        let content = fs::read_to_string(&claude_path).unwrap();
        assert!(content.contains("## Principles"));
        assert!(content.contains("## Build Discipline (ponytail)"));
        assert!(!content.contains("## Response Style (caveman)"));

        // Second run with same rules should return false (no-op)
        let no_op = patch_claude_md(&claude_path, &[RuleKind::Principles, RuleKind::Ponytail]).unwrap();
        assert!(!no_op);

        // Add Caveman
        let patched2 = patch_claude_md(&claude_path, &[RuleKind::Caveman]).unwrap();
        assert!(patched2);
        let content2 = fs::read_to_string(&claude_path).unwrap();
        assert!(content2.contains("## Response Style (caveman)"));
    }

    #[test]
    fn test_patch_agents_md() {
        let tmp = tempdir().unwrap();
        let agents_path = tmp.path().join("AGENTS.md");

        let patched = patch_agents_md(&agents_path, &[RuleKind::Caveman]).unwrap();
        assert!(patched);
        assert!(agents_path.exists());

        let content = fs::read_to_string(&agents_path).unwrap();
        assert!(content.contains("## Response Style (caveman)"));
    }

    #[test]
    fn test_patch_cursor_rules() {
        let tmp = tempdir().unwrap();
        let rules = RuleKind::all();
        let written = patch_cursor_rules(tmp.path(), rules).unwrap();

        assert_eq!(written.len(), 3);
        for p in written {
            assert!(p.exists());
            let c = fs::read_to_string(&p).unwrap();
            assert!(c.starts_with("---\n"));
            assert!(c.contains("alwaysApply: true"));
        }
    }

    #[test]
    fn test_inject_and_remove_at_path() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("CUSTOM_AGENT.md");

        assert!(!has_rule_at_path(&file_path, RuleKind::Ponytail));
        let injected = inject_rule_at_path(&file_path, RuleKind::Ponytail).unwrap();
        assert!(injected);
        assert!(has_rule_at_path(&file_path, RuleKind::Ponytail));

        let removed = remove_rule_from_path(&file_path, RuleKind::Ponytail).unwrap();
        assert!(removed);
        assert!(!has_rule_at_path(&file_path, RuleKind::Ponytail));
    }
}

