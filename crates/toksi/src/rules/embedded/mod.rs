// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/embedded/mod.rs
// created: 2026-10-29
// lastModified: 2026-11-13

//! Embedded in-tree rules & skills assets compiled directly into the toksi binary.

pub mod caveman;
pub mod ponytail;
pub mod principles;

pub use caveman as embedded_caveman;
pub use ponytail as embedded_ponytail;
pub use principles as embedded_principles;

/// Compile-time embedded skill package descriptor (`SKILL.md` in `skills/<dir_name>/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedSkill {
    pub dir_name: &'static str,
    pub skill_md: &'static str,
}

/// Compile-time embedded Gemini custom command descriptor (`.toml` in `commands/<filename>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedCommand {
    pub filename: &'static str,
    pub toml: &'static str,
}

/// Extracts section body content below the top `# ...` heading using normalized `\n` newlines.
pub(crate) fn extract_section_body(raw_markdown: &str) -> String {
    let raw = raw_markdown.replace("\r\n", "\n");
    if let Some(pos) = raw.find('\n') {
        raw[pos + 1..].trim().to_string()
    } else {
        raw.trim().to_string()
    }
}

/// Returns all primary and companion skills across Principles, Caveman v4, and Ponytail v5.
pub fn all_skills() -> Vec<EmbeddedSkill> {
    let mut skills = vec![principles::primary_skill()];
    skills.extend_from_slice(caveman::companion_skills());
    skills.extend_from_slice(ponytail::companion_skills());
    skills
}

/// Returns all Gemini custom `.toml` command files for Caveman v4 and Ponytail v5.
pub fn all_commands() -> Vec<EmbeddedCommand> {
    let mut commands = Vec::new();
    commands.extend_from_slice(caveman::gemini_commands());
    commands.extend_from_slice(ponytail::gemini_commands());
    commands
}
