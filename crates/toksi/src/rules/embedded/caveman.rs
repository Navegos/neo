// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/embedded/caveman.rs
// created: 2026-10-29
// lastModified: 2026-11-13

//! ASD-STE100 primitive English compression & Wave 17 Caveman v4 mode suite.
//! Includes caveman, ultracave, megacave, companion skills, shrink/browse/.blocks specs, and commands.

use super::{extract_section_body, EmbeddedCommand, EmbeddedSkill};

pub const ID: &str = "caveman";
pub const TITLE: &str = "Response Style (caveman)";
pub const DESCRIPTION: &str =
    "Compressed agent responses through primitive English (ASD-STE100) and Wave 17 Caveman v4 modes.";
pub const SECTION_HEADER: &str = "## Response Style (caveman)";
pub const CURSOR_FILENAME: &str = "response-style.mdc";
pub const SKILL_DIR_NAME: &str = "caveman";

pub const RAW_MARKDOWN: &str = include_str!("../assets/caveman.md");
pub const CURSOR_RULE: &str = include_str!("../assets/caveman.mdc");
pub const SKILL_DEFINITION: &str = include_str!("../assets/caveman.skill.md");

pub const ULTRACAVE_SKILL: &str = include_str!("../assets/ultracave.skill.md");
pub const MEGACAVE_SKILL: &str = include_str!("../assets/megacave.skill.md");
pub const CAVEMAN_COMMIT_SKILL: &str = include_str!("../assets/caveman-commit.skill.md");
pub const CAVEMAN_REVIEW_SKILL: &str = include_str!("../assets/caveman-review.skill.md");
pub const CAVEMAN_COMPRESS_SKILL: &str = include_str!("../assets/caveman-compress.skill.md");
pub const CAVEMAN_HELP_SKILL: &str = include_str!("../assets/caveman-help.skill.md");
pub const CAVEMAN_STATS_SKILL: &str = include_str!("../assets/caveman-stats.skill.md");
pub const CAVECREW_SKILL: &str = include_str!("../assets/cavecrew.skill.md");

pub const SHRINK_SPEC: &str = include_str!("../assets/caveman-shrink.md");
pub const BROWSE_SPEC: &str = include_str!("../assets/caveman-browse.md");
pub const BLOCKS_SPEC: &str = include_str!("../assets/caveman-blocks.md");

pub const CMD_CAVEMAN_TOML: &str = include_str!("../assets/commands/caveman.toml");
pub const CMD_ULTRACAVE_TOML: &str = include_str!("../assets/commands/ultracave.toml");
pub const CMD_MEGACAVE_TOML: &str = include_str!("../assets/commands/megacave.toml");
pub const CMD_COMMIT_TOML: &str = include_str!("../assets/commands/caveman-commit.toml");
pub const CMD_REVIEW_TOML: &str = include_str!("../assets/commands/caveman-review.toml");
pub const CMD_STATS_TOML: &str = include_str!("../assets/commands/caveman-stats.toml");

const SKILLS: &[EmbeddedSkill] = &[
    EmbeddedSkill {
        dir_name: "caveman",
        skill_md: SKILL_DEFINITION,
    },
    EmbeddedSkill {
        dir_name: "ultracave",
        skill_md: ULTRACAVE_SKILL,
    },
    EmbeddedSkill {
        dir_name: "megacave",
        skill_md: MEGACAVE_SKILL,
    },
    EmbeddedSkill {
        dir_name: "caveman-commit",
        skill_md: CAVEMAN_COMMIT_SKILL,
    },
    EmbeddedSkill {
        dir_name: "caveman-review",
        skill_md: CAVEMAN_REVIEW_SKILL,
    },
    EmbeddedSkill {
        dir_name: "caveman-compress",
        skill_md: CAVEMAN_COMPRESS_SKILL,
    },
    EmbeddedSkill {
        dir_name: "caveman-help",
        skill_md: CAVEMAN_HELP_SKILL,
    },
    EmbeddedSkill {
        dir_name: "caveman-stats",
        skill_md: CAVEMAN_STATS_SKILL,
    },
    EmbeddedSkill {
        dir_name: "cavecrew",
        skill_md: CAVECREW_SKILL,
    },
];

const COMMANDS: &[EmbeddedCommand] = &[
    EmbeddedCommand {
        filename: "caveman.toml",
        toml: CMD_CAVEMAN_TOML,
    },
    EmbeddedCommand {
        filename: "ultracave.toml",
        toml: CMD_ULTRACAVE_TOML,
    },
    EmbeddedCommand {
        filename: "megacave.toml",
        toml: CMD_MEGACAVE_TOML,
    },
    EmbeddedCommand {
        filename: "caveman-commit.toml",
        toml: CMD_COMMIT_TOML,
    },
    EmbeddedCommand {
        filename: "caveman-review.toml",
        toml: CMD_REVIEW_TOML,
    },
    EmbeddedCommand {
        filename: "caveman-stats.toml",
        toml: CMD_STATS_TOML,
    },
];

/// Wave 17 Caveman v4 modes and companion skill identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CavemanMode {
    Caveman,
    Ultracave,
    Megacave,
    Commit,
    Review,
    Compress,
    Help,
    Stats,
    Cavecrew,
    Off,
}

impl CavemanMode {
    /// Parses a Caveman v4 mode or legacy alias (`lite`, `full`, `ultra`, `wenyan*`, `/ultracave`).
    pub fn parse(raw: &str) -> Option<Self> {
        let normalized = raw.trim().trim_start_matches('/').to_ascii_lowercase();
        match normalized.as_str() {
            "caveman" | "lite" | "full" => Some(Self::Caveman),
            "ultracave" | "ultra" | "caveman ultra" | "caveman-ultra" => Some(Self::Ultracave),
            "megacave" | "wenyan" | "wenyan-lite" | "wenyan-full" | "wenyan-ultra"
            | "caveman wenyan" | "caveman-wenyan" => Some(Self::Megacave),
            "commit" | "caveman-commit" => Some(Self::Commit),
            "review" | "caveman-review" => Some(Self::Review),
            "compress" | "caveman-compress" => Some(Self::Compress),
            "help" | "caveman-help" => Some(Self::Help),
            "stats" | "caveman-stats" => Some(Self::Stats),
            "cavecrew" => Some(Self::Cavecrew),
            "off" | "manual" | "normal" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Caveman => "caveman",
            Self::Ultracave => "ultracave",
            Self::Megacave => "megacave",
            Self::Commit => "caveman-commit",
            Self::Review => "caveman-review",
            Self::Compress => "caveman-compress",
            Self::Help => "caveman-help",
            Self::Stats => "caveman-stats",
            Self::Cavecrew => "cavecrew",
            Self::Off => "off",
        }
    }

    /// Returns the statusline badge text matching `caveman-statusline.sh` (or `None` when `Off`).
    pub fn statusline_badge(&self) -> Option<&'static str> {
        match self {
            Self::Caveman => Some("[CAVEMAN]"),
            Self::Ultracave => Some("[ULTRACAVE]"),
            Self::Megacave => Some("[MEGACAVE]"),
            Self::Commit => Some("[CAVEMAN:COMMIT]"),
            Self::Review => Some("[CAVEMAN:REVIEW]"),
            Self::Compress => Some("[CAVEMAN:COMPRESS]"),
            Self::Help | Self::Stats | Self::Cavecrew | Self::Off => None,
        }
    }

    /// Returns the embedded skill markdown for this mode when applicable.
    pub fn skill_definition(&self) -> Option<&'static str> {
        match self {
            Self::Caveman => Some(SKILL_DEFINITION),
            Self::Ultracave => Some(ULTRACAVE_SKILL),
            Self::Megacave => Some(MEGACAVE_SKILL),
            Self::Commit => Some(CAVEMAN_COMMIT_SKILL),
            Self::Review => Some(CAVEMAN_REVIEW_SKILL),
            Self::Compress => Some(CAVEMAN_COMPRESS_SKILL),
            Self::Help => Some(CAVEMAN_HELP_SKILL),
            Self::Stats => Some(CAVEMAN_STATS_SKILL),
            Self::Cavecrew => Some(CAVECREW_SKILL),
            Self::Off => None,
        }
    }
}

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

/// Returns the `shrink` tool catalog compression specification.
pub fn shrink_spec() -> &'static str {
    SHRINK_SPEC
}

/// Returns the `browse` local accessibility browser MCP specification.
pub fn browse_spec() -> &'static str {
    BROWSE_SPEC
}

/// Returns the `.blocks` structured block verification specification.
pub fn blocks_spec() -> &'static str {
    BLOCKS_SPEC
}

/// Returns all primary and companion Caveman v4 skills.
pub const fn companion_skills() -> &'static [EmbeddedSkill] {
    SKILLS
}

/// Returns all Gemini `.toml` command templates for Caveman v4.
pub const fn gemini_commands() -> &'static [EmbeddedCommand] {
    COMMANDS
}

/// Looks up an embedded skill or subsystem spec by key (`ultracave`, `shrink`, `browse`, `.blocks`, etc.).
pub fn lookup_asset(name: &str) -> Option<&'static str> {
    let key = name.trim().trim_start_matches('/').to_ascii_lowercase();
    match key.as_str() {
        "shrink" | "caveman-shrink" => Some(SHRINK_SPEC),
        "browse" | "caveman-browse" => Some(BROWSE_SPEC),
        "blocks" | ".blocks" | "caveman-blocks" => Some(BLOCKS_SPEC),
        _ => CavemanMode::parse(&key).and_then(|m| m.skill_definition()),
    }
}

/// Returns the formatted section body suitable for inclusion under `## Response Style (caveman)`.
pub fn section_body() -> String {
    extract_section_body(RAW_MARKDOWN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caveman_embedded_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(!skill_definition().is_empty());
        assert!(markdown().contains("ASD-STE100"));
        assert!(markdown().contains("Respond terse like smart caveman"));
        assert!(markdown().contains("Auto-Clarity"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
        assert!(skill_definition().starts_with("---\n"));
        assert!(skill_definition().contains("name: caveman"));
    }

    #[test]
    fn test_caveman_section_body() {
        let body = section_body();
        assert!(!body.is_empty());
        assert!(body.contains("Respond terse like smart caveman"));
    }

    #[test]
    fn test_caveman_v4_modes_and_subsystems() {
        assert_eq!(CavemanMode::parse("/ultracave"), Some(CavemanMode::Ultracave));
        assert_eq!(CavemanMode::parse("wenyan-ultra"), Some(CavemanMode::Megacave));
        assert_eq!(CavemanMode::parse("lite"), Some(CavemanMode::Caveman));
        assert_eq!(CavemanMode::Ultracave.statusline_badge(), Some("[ULTRACAVE]"));
        assert_eq!(CavemanMode::Megacave.statusline_badge(), Some("[MEGACAVE]"));
        assert_eq!(CavemanMode::Commit.statusline_badge(), Some("[CAVEMAN:COMMIT]"));
        assert_eq!(CavemanMode::Off.statusline_badge(), None);
        assert!(lookup_asset("shrink").unwrap_or("").contains("ccr_..."));
        assert!(lookup_asset("browse").unwrap_or("").contains("browser_snapshot"));
        assert!(lookup_asset(".blocks").unwrap_or("").contains("# /// block"));
        assert_eq!(companion_skills().len(), 9);
        assert_eq!(gemini_commands().len(), 6);
    }
}
