// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rules/embedded/caveman.rs

//! ASD-STE100 primitive English compression guidelines (Caveman style).
//! Maximum density, zero fluff, all technical substance preserved.

pub const ID: &str = "caveman";
pub const TITLE: &str = "Response Style (caveman)";
pub const DESCRIPTION: &str = "Compressed agent prompts through primitive English (ASD-STE100).";
pub const SECTION_HEADER: &str = "## Response Style (caveman)";
pub const CURSOR_FILENAME: &str = "response-style.mdc";

pub const RAW_MARKDOWN: &str = include_str!("../assets/caveman.md");
pub const CURSOR_RULE: &str = include_str!("../assets/caveman.mdc");

/// Returns the raw markdown guidelines.
pub fn markdown() -> &'static str {
    RAW_MARKDOWN
}

/// Returns the Cursor project rule (.mdc format with YAML frontmatter).
pub fn cursor_rule() -> &'static str {
    CURSOR_RULE
}

/// Returns the formatted section body suitable for inclusion under `## Response Style (caveman)`.
pub fn section_body() -> String {
    let raw = RAW_MARKDOWN.replace("\r\n", "\n");
    if let Some(pos) = raw.find('\n') {
        raw[pos + 1..].trim().to_string()
    } else {
        raw.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caveman_embedded_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(markdown().contains("ASD-STE100"));
        assert!(markdown().contains("Respond terse like smart caveman"));
        assert!(markdown().contains("Auto-Clarity"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
    }

    #[test]
    fn test_caveman_section_body() {
        let body = section_body();
        assert!(!body.is_empty());
        assert!(body.contains("Respond terse like smart caveman"));
    }
}

