// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/caveman.rs
// created: 2026-10-29
// lastModified: 2026-11-16

//! ASD-STE100 primitive English compression & Wave 17 Caveman v4 mode suite
//! (loaded from externalized `<neo>/rules/`, `<neo>/agents/`, and `<skills>`).

#![deny(unsafe_code)]

use std::sync::OnceLock;

use crate::rules::skills_repo::{
    read_command_toml, read_cursor_rule, read_rule_markdown, read_skill_markdown,
    read_spec_markdown,
};
use crate::rules::{extract_section_body, CommandAsset, SkillAsset};

pub const ID: &str = "caveman";
pub const TITLE: &str = "Response Style (caveman)";
pub const DESCRIPTION: &str =
    "Compressed agent responses through primitive English (ASD-STE100) and Wave 17 Caveman v4 modes.";
pub const SECTION_HEADER: &str = "## Response Style (caveman)";
pub const CURSOR_FILENAME: &str = "response-style.mdc";
pub const SKILL_DIR_NAME: &str = "caveman";

static RAW_MARKDOWN_CACHE: OnceLock<String> = OnceLock::new();
static CURSOR_RULE_CACHE: OnceLock<String> = OnceLock::new();
static SKILL_DEF_CACHE: OnceLock<String> = OnceLock::new();
static ULTRACAVE_CACHE: OnceLock<String> = OnceLock::new();
static MEGACAVE_CACHE: OnceLock<String> = OnceLock::new();
static COMMIT_CACHE: OnceLock<String> = OnceLock::new();
static REVIEW_CACHE: OnceLock<String> = OnceLock::new();
static COMPRESS_CACHE: OnceLock<String> = OnceLock::new();
static HELP_CACHE: OnceLock<String> = OnceLock::new();
static STATS_CACHE: OnceLock<String> = OnceLock::new();
static CAVECREW_CACHE: OnceLock<String> = OnceLock::new();
static SHRINK_CACHE: OnceLock<String> = OnceLock::new();
static BROWSE_CACHE: OnceLock<String> = OnceLock::new();
static BLOCKS_CACHE: OnceLock<String> = OnceLock::new();

static CMD_CAVEMAN_CACHE: OnceLock<String> = OnceLock::new();
static CMD_ULTRACAVE_CACHE: OnceLock<String> = OnceLock::new();
static CMD_MEGACAVE_CACHE: OnceLock<String> = OnceLock::new();
static CMD_COMMIT_CACHE: OnceLock<String> = OnceLock::new();
static CMD_REVIEW_CACHE: OnceLock<String> = OnceLock::new();
static CMD_STATS_CACHE: OnceLock<String> = OnceLock::new();

static SKILLS_CACHE: OnceLock<Vec<SkillAsset>> = OnceLock::new();
static COMMANDS_CACHE: OnceLock<Vec<CommandAsset>> = OnceLock::new();

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

    /// Returns the externalized skill markdown for this mode when applicable.
    pub fn skill_definition(&self) -> Option<&'static str> {
        match self {
            Self::Caveman => Some(skill_definition()),
            Self::Ultracave => Some(ULTRACAVE_CACHE.get_or_init(|| read_skill_markdown("ultracave")).as_str()),
            Self::Megacave => Some(MEGACAVE_CACHE.get_or_init(|| read_skill_markdown("megacave")).as_str()),
            Self::Commit => Some(COMMIT_CACHE.get_or_init(|| read_skill_markdown("caveman-commit")).as_str()),
            Self::Review => Some(REVIEW_CACHE.get_or_init(|| read_skill_markdown("caveman-review")).as_str()),
            Self::Compress => Some(COMPRESS_CACHE.get_or_init(|| read_skill_markdown("caveman-compress")).as_str()),
            Self::Help => Some(HELP_CACHE.get_or_init(|| read_skill_markdown("caveman-help")).as_str()),
            Self::Stats => Some(STATS_CACHE.get_or_init(|| read_skill_markdown("caveman-stats")).as_str()),
            Self::Cavecrew => Some(CAVECREW_CACHE.get_or_init(|| read_skill_markdown("cavecrew")).as_str()),
            Self::Off => None,
        }
    }
}

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

/// Returns the `shrink` tool catalog compression specification.
pub fn shrink_spec() -> &'static str {
    SHRINK_CACHE
        .get_or_init(|| read_spec_markdown("caveman-shrink.md"))
        .as_str()
}

/// Returns the `browse` local accessibility browser MCP specification.
pub fn browse_spec() -> &'static str {
    BROWSE_CACHE
        .get_or_init(|| read_spec_markdown("caveman-browse.md"))
        .as_str()
}

/// Returns the `.blocks` structured block verification specification.
pub fn blocks_spec() -> &'static str {
    BLOCKS_CACHE
        .get_or_init(|| read_spec_markdown("caveman-blocks.md"))
        .as_str()
}

/// Returns all primary and companion Caveman v4 skills.
pub fn companion_skills() -> &'static [SkillAsset] {
    SKILLS_CACHE
        .get_or_init(|| {
            vec![
                SkillAsset {
                    dir_name: "caveman",
                    skill_md: skill_definition(),
                },
                SkillAsset {
                    dir_name: "ultracave",
                    skill_md: CavemanMode::Ultracave.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "megacave",
                    skill_md: CavemanMode::Megacave.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "caveman-commit",
                    skill_md: CavemanMode::Commit.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "caveman-review",
                    skill_md: CavemanMode::Review.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "caveman-compress",
                    skill_md: CavemanMode::Compress.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "caveman-help",
                    skill_md: CavemanMode::Help.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "caveman-stats",
                    skill_md: CavemanMode::Stats.skill_definition().unwrap_or(""),
                },
                SkillAsset {
                    dir_name: "cavecrew",
                    skill_md: CavemanMode::Cavecrew.skill_definition().unwrap_or(""),
                },
            ]
        })
        .as_slice()
}

/// Returns all Gemini `.toml` command templates for Caveman v4.
pub fn gemini_commands() -> &'static [CommandAsset] {
    COMMANDS_CACHE
        .get_or_init(|| {
            vec![
                CommandAsset {
                    filename: "caveman.toml",
                    toml: CMD_CAVEMAN_CACHE.get_or_init(|| read_command_toml("caveman.toml")).as_str(),
                },
                CommandAsset {
                    filename: "ultracave.toml",
                    toml: CMD_ULTRACAVE_CACHE.get_or_init(|| read_command_toml("ultracave.toml")).as_str(),
                },
                CommandAsset {
                    filename: "megacave.toml",
                    toml: CMD_MEGACAVE_CACHE.get_or_init(|| read_command_toml("megacave.toml")).as_str(),
                },
                CommandAsset {
                    filename: "caveman-commit.toml",
                    toml: CMD_COMMIT_CACHE.get_or_init(|| read_command_toml("caveman-commit.toml")).as_str(),
                },
                CommandAsset {
                    filename: "caveman-review.toml",
                    toml: CMD_REVIEW_CACHE.get_or_init(|| read_command_toml("caveman-review.toml")).as_str(),
                },
                CommandAsset {
                    filename: "caveman-stats.toml",
                    toml: CMD_STATS_CACHE.get_or_init(|| read_command_toml("caveman-stats.toml")).as_str(),
                },
            ]
        })
        .as_slice()
}

/// Looks up an externalized skill or subsystem spec by key (`ultracave`, `shrink`, `browse`, `.blocks`, etc.).
pub fn lookup_asset(name: &str) -> Option<&'static str> {
    let key = name.trim().trim_start_matches('/').to_ascii_lowercase();
    match key.as_str() {
        "shrink" | "caveman-shrink" => Some(shrink_spec()),
        "browse" | "caveman-browse" => Some(browse_spec()),
        "blocks" | ".blocks" | "caveman-blocks" => Some(blocks_spec()),
        _ => CavemanMode::parse(&key).and_then(|m| m.skill_definition()),
    }
}

/// Returns the formatted section body suitable for inclusion under `## Response Style (caveman)`.
pub fn section_body() -> String {
    extract_section_body(markdown())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caveman_external_assets_and_modes() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(!skill_definition().is_empty());
        assert!(markdown().contains("ASD-STE100"));
        assert!(markdown().contains("<enforce>"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
        assert!(skill_definition().starts_with("---\n"));
        assert!(skill_definition().contains("name: caveman"));

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
