// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/principles.rs
// created: 2026-10-29
// lastModified: 2026-11-16

//! Karpathy behavioral coding guidelines (loaded from externalized `<neo>/rules/`
//! and `<skills>` `Navegos/skills` with zero binary bloat).

#![deny(unsafe_code)]

use std::sync::OnceLock;

use crate::rules::skills_repo::{read_cursor_rule, read_rule_markdown, read_skill_markdown};
use crate::rules::{extract_section_body, SkillAsset};

pub const ID: &str = "principles";
pub const ALIASES: &[&str] = &["karpathy", "karpathy-guidelines"];
pub const TITLE: &str = "Principles";
pub const DESCRIPTION: &str = "Coding principles: think first, simplify, edit surgically, verify.";
pub const SECTION_HEADER: &str = "## Principles";
pub const CURSOR_FILENAME: &str = "principles.mdc";
pub const SKILL_DIR_NAME: &str = "principles";

static RAW_MARKDOWN_CACHE: OnceLock<String> = OnceLock::new();
static CURSOR_RULE_CACHE: OnceLock<String> = OnceLock::new();
static SKILL_DEF_CACHE: OnceLock<String> = OnceLock::new();
static SKILLS_CACHE: OnceLock<Vec<SkillAsset>> = OnceLock::new();

/// Returns the externalized raw markdown guidelines.
pub fn markdown() -> &'static str {
    RAW_MARKDOWN_CACHE
        .get_or_init(|| read_rule_markdown(ID))
        .as_str()
}

/// Returns the externalized Cursor project rule (.mdc format with YAML frontmatter).
pub fn cursor_rule() -> &'static str {
    CURSOR_RULE_CACHE
        .get_or_init(|| read_cursor_rule(ID))
        .as_str()
}

/// Returns the externalized standalone skill definition (`SKILL.md` format with YAML frontmatter).
pub fn skill_definition() -> &'static str {
    SKILL_DEF_CACHE
        .get_or_init(|| read_skill_markdown(SKILL_DIR_NAME))
        .as_str()
}

/// Returns the primary skill descriptor for directory installation.
pub fn primary_skill() -> SkillAsset {
    SkillAsset {
        dir_name: SKILL_DIR_NAME,
        skill_md: skill_definition(),
    }
}

/// Returns all skills bundled with Principles (`&'static` slice).
pub fn companion_skills() -> &'static [SkillAsset] {
    SKILLS_CACHE
        .get_or_init(|| vec![primary_skill()])
        .as_slice()
}

/// Returns the formatted section body suitable for inclusion under `## Principles`.
pub fn section_body() -> String {
    extract_section_body(markdown())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_principles_external_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(!skill_definition().is_empty());
        assert!(markdown().contains("Epistemic Rigor"));
        assert!(markdown().contains("First-Principles Simplicity"));
        assert!(markdown().contains("Surgical Mutation"));
        assert!(markdown().contains("Deterministic Tool Orchestration"));
        assert!(markdown().contains("Closed-Loop Verification"));
        assert!(markdown().contains("<enforce>"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
        assert!(skill_definition().starts_with("---\n"));
        assert!(skill_definition().contains("name: principles"));
    }
}
