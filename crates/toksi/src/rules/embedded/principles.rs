// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/embedded/principles.rs
// created: 2026-10-29
// lastModified: 2026-11-13

//! Karpathy behavioral coding guidelines.
//! Think before coding, simplicity first, surgical changes, goal-driven execution.

use super::{extract_section_body, EmbeddedSkill};

pub const ID: &str = "principles";
pub const ALIASES: &[&str] = &["karpathy", "karpathy-guidelines"];
pub const TITLE: &str = "Principles";
pub const DESCRIPTION: &str = "Coding principles: think first, simplify, edit surgically, verify.";
pub const SECTION_HEADER: &str = "## Principles";
pub const CURSOR_FILENAME: &str = "principles.mdc";
pub const SKILL_DIR_NAME: &str = "principles";

pub const RAW_MARKDOWN: &str = include_str!("../assets/principles.md");
pub const CURSOR_RULE: &str = include_str!("../assets/principles.mdc");
pub const SKILL_DEFINITION: &str = include_str!("../assets/principles.skill.md");

/// Returns the raw markdown guidelines.
pub fn markdown() -> &'static str {
    RAW_MARKDOWN
}

/// Returns the Cursor project rule (.mdc format with YAML frontmatter).
pub fn cursor_rule() -> &'static str {
    CURSOR_RULE
}

/// Returns the standalone skill definition (SKILL.md format with YAML frontmatter).
pub fn skill_definition() -> &'static str {
    SKILL_DEFINITION
}

/// Returns the primary skill descriptor for directory installation.
pub const fn primary_skill() -> EmbeddedSkill {
    EmbeddedSkill {
        dir_name: SKILL_DIR_NAME,
        skill_md: SKILL_DEFINITION,
    }
}

const SKILLS: &[EmbeddedSkill] = &[primary_skill()];

/// Returns all skills bundled with Principles (`&'static` slice).
pub const fn companion_skills() -> &'static [EmbeddedSkill] {
    SKILLS
}

/// Returns the formatted section body suitable for inclusion under `## Principles`.
pub fn section_body() -> String {
    extract_section_body(RAW_MARKDOWN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_principles_embedded_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(!skill_definition().is_empty());
        assert!(markdown().contains("Think Before Coding"));
        assert!(markdown().contains("Simplicity First"));
        assert!(markdown().contains("Surgical Changes"));
        assert!(markdown().contains("Goal-Driven Execution"));
        assert!(markdown().contains("These guidelines are working if:"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
        assert!(skill_definition().starts_with("---\n"));
        assert!(skill_definition().contains("name: principles"));
    }

    #[test]
    fn test_principles_section_body() {
        let body = section_body();
        assert!(!body.is_empty());
        assert!(body.contains("Behavioral guidelines to reduce common LLM coding mistakes"));
        assert!(body.contains("These guidelines are working if:"));
    }
}
