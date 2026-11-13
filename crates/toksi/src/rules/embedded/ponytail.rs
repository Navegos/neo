// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/embedded/ponytail.rs
// created: 2026-10-29
// lastModified: 2026-11-13

//! Lazy senior developer build discipline ladder (Ponytail v5).
//! Includes intensity modes (lite/full/ultra/off/review), companion skills, commands, and statusline telemetry.

use super::{extract_section_body, EmbeddedCommand, EmbeddedSkill};

pub const ID: &str = "ponytail";
pub const TITLE: &str = "Build Discipline (ponytail)";
pub const DESCRIPTION: &str =
    "Minimal-code rules: reuse first, avoid speculative work, honor boundaries (the Ladder).";
pub const SECTION_HEADER: &str = "## Build Discipline (ponytail)";
pub const CURSOR_FILENAME: &str = "build-discipline.mdc";
pub const SKILL_DIR_NAME: &str = "ponytail";
pub const STATUSLINE_FLAG_FILE: &str = ".ponytail-active";

pub const RAW_MARKDOWN: &str = include_str!("../assets/ponytail.md");
pub const CURSOR_RULE: &str = include_str!("../assets/ponytail.mdc");
pub const SKILL_DEFINITION: &str = include_str!("../assets/ponytail.skill.md");

pub const AUDIT_SKILL: &str = include_str!("../assets/ponytail-audit.skill.md");
pub const DEBT_SKILL: &str = include_str!("../assets/ponytail-debt.skill.md");
pub const GAIN_SKILL: &str = include_str!("../assets/ponytail-gain.skill.md");
pub const HELP_SKILL: &str = include_str!("../assets/ponytail-help.skill.md");
pub const REVIEW_SKILL: &str = include_str!("../assets/ponytail-review.skill.md");

pub const CMD_PONYTAIL_TOML: &str = include_str!("../assets/commands/ponytail.toml");
pub const CMD_AUDIT_TOML: &str = include_str!("../assets/commands/ponytail-audit.toml");
pub const CMD_DEBT_TOML: &str = include_str!("../assets/commands/ponytail-debt.toml");
pub const CMD_GAIN_TOML: &str = include_str!("../assets/commands/ponytail-gain.toml");
pub const CMD_HELP_TOML: &str = include_str!("../assets/commands/ponytail-help.toml");
pub const CMD_REVIEW_TOML: &str = include_str!("../assets/commands/ponytail-review.toml");

const SKILLS: &[EmbeddedSkill] = &[
    EmbeddedSkill {
        dir_name: "ponytail",
        skill_md: SKILL_DEFINITION,
    },
    EmbeddedSkill {
        dir_name: "ponytail-audit",
        skill_md: AUDIT_SKILL,
    },
    EmbeddedSkill {
        dir_name: "ponytail-debt",
        skill_md: DEBT_SKILL,
    },
    EmbeddedSkill {
        dir_name: "ponytail-gain",
        skill_md: GAIN_SKILL,
    },
    EmbeddedSkill {
        dir_name: "ponytail-help",
        skill_md: HELP_SKILL,
    },
    EmbeddedSkill {
        dir_name: "ponytail-review",
        skill_md: REVIEW_SKILL,
    },
];

const COMMANDS: &[EmbeddedCommand] = &[
    EmbeddedCommand {
        filename: "ponytail.toml",
        toml: CMD_PONYTAIL_TOML,
    },
    EmbeddedCommand {
        filename: "ponytail-audit.toml",
        toml: CMD_AUDIT_TOML,
    },
    EmbeddedCommand {
        filename: "ponytail-debt.toml",
        toml: CMD_DEBT_TOML,
    },
    EmbeddedCommand {
        filename: "ponytail-gain.toml",
        toml: CMD_GAIN_TOML,
    },
    EmbeddedCommand {
        filename: "ponytail-help.toml",
        toml: CMD_HELP_TOML,
    },
    EmbeddedCommand {
        filename: "ponytail-review.toml",
        toml: CMD_REVIEW_TOML,
    },
];

/// Ponytail v5 execution levels plus session-only review mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PonytailMode {
    Off,
    Lite,
    Full,
    Ultra,
    Review,
}

/// Telemetry descriptor matching `ponytail-statusline.sh` / `.ps1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PonytailTelemetry {
    pub mode: PonytailMode,
    pub flag_file: &'static str,
    pub badge: Option<String>,
    pub ansi_color: u8,
}

impl PonytailMode {
    /// Parses a ponytail mode level (`lite`, `full`, `ultra`, `review`, `off`).
    pub fn parse(raw: &str) -> Option<Self> {
        let normalized = raw.trim().trim_start_matches('/').to_ascii_lowercase();
        match normalized.as_str() {
            "off" | "normal" => Some(Self::Off),
            "lite" => Some(Self::Lite),
            "" | "full" | "ponytail" => Some(Self::Full),
            "ultra" => Some(Self::Ultra),
            "review" | "ponytail-review" => Some(Self::Review),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Lite => "lite",
            Self::Full => "full",
            Self::Ultra => "ultra",
            Self::Review => "review",
        }
    }

    /// Returns 256-color ANSI index (`173` amber for `ultra`, `108` green otherwise).
    pub const fn ansi_color(&self) -> u8 {
        match self {
            Self::Ultra => 173,
            _ => 108,
        }
    }

    /// Returns the statusline badge text (or `None` when `Off`).
    pub fn statusline_badge(&self) -> Option<String> {
        match self {
            Self::Off => None,
            Self::Full => Some("[PONYTAIL]".to_string()),
            other => Some(format!("[PONYTAIL:{}]", other.as_str().to_ascii_uppercase())),
        }
    }
}

/// Builds statusline telemetry payload for the given mode.
pub fn telemetry(mode: PonytailMode) -> PonytailTelemetry {
    PonytailTelemetry {
        mode,
        flag_file: STATUSLINE_FLAG_FILE,
        badge: mode.statusline_badge(),
        ansi_color: mode.ansi_color(),
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

/// Returns all primary and companion Ponytail v5 skills.
pub const fn companion_skills() -> &'static [EmbeddedSkill] {
    SKILLS
}

/// Returns all Gemini `.toml` command templates for Ponytail v5.
pub const fn gemini_commands() -> &'static [EmbeddedCommand] {
    COMMANDS
}

/// Looks up a Ponytail skill by name (`ponytail`, `ponytail-audit`, `ponytail-debt`, etc.).
pub fn lookup_skill(name: &str) -> Option<&'static str> {
    let key = name.trim().trim_start_matches('/').to_ascii_lowercase();
    SKILLS
        .iter()
        .find(|s| s.dir_name == key)
        .map(|s| s.skill_md)
}

/// Returns the formatted section body suitable for inclusion under `## Build Discipline (ponytail)`.
pub fn section_body() -> String {
    extract_section_body(RAW_MARKDOWN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ponytail_embedded_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(!skill_definition().is_empty());
        assert!(markdown().contains("The Ladder"));
        assert!(markdown().contains("lazy senior developer"));
        assert!(markdown().contains("Does this need to exist at all?"));
        assert!(markdown().contains("shortcut:"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
        assert!(skill_definition().starts_with("---\n"));
        assert!(skill_definition().contains("name: ponytail"));
    }

    #[test]
    fn test_ponytail_section_body() {
        let body = section_body();
        assert!(!body.is_empty());
        assert!(body.contains("lazy senior developer"));
    }

    #[test]
    fn test_ponytail_modes_telemetry_and_companions() {
        assert_eq!(PonytailMode::parse("ultra"), Some(PonytailMode::Ultra));
        assert_eq!(telemetry(PonytailMode::Full).badge.as_deref(), Some("[PONYTAIL]"));
        assert_eq!(telemetry(PonytailMode::Lite).badge.as_deref(), Some("[PONYTAIL:LITE]"));
        let ultra = telemetry(PonytailMode::Ultra);
        assert_eq!(ultra.badge.as_deref(), Some("[PONYTAIL:ULTRA]"));
        assert_eq!(ultra.ansi_color, 173);
        assert_eq!(telemetry(PonytailMode::Off).badge, None);
        assert!(lookup_skill("ponytail-debt").unwrap_or("").contains("no-trigger"));
        assert!(lookup_skill("ponytail-gain").unwrap_or("").contains("Ponytail 5"));
        assert_eq!(companion_skills().len(), 6);
        assert_eq!(gemini_commands().len(), 6);
    }
}
