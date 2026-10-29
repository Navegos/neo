// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rules/embedded/ponytail.rs

//! Lazy senior developer build discipline ladder (Ponytail mode).
//! The best code is the code never written. Reuse first, minimal diffs, runnable verification.

pub const ID: &str = "ponytail";
pub const TITLE: &str = "Build Discipline (ponytail)";
pub const DESCRIPTION: &str = "Minimal-code rules: reuse first, avoid speculative work (the Ladder).";
pub const SECTION_HEADER: &str = "## Build Discipline (ponytail)";
pub const CURSOR_FILENAME: &str = "build-discipline.mdc";

pub const RAW_MARKDOWN: &str = include_str!("../assets/ponytail.md");
pub const CURSOR_RULE: &str = include_str!("../assets/ponytail.mdc");

/// Returns the raw markdown guidelines.
pub fn markdown() -> &'static str {
    RAW_MARKDOWN
}

/// Returns the Cursor project rule (.mdc format with YAML frontmatter).
pub fn cursor_rule() -> &'static str {
    CURSOR_RULE
}

/// Returns the formatted section body suitable for inclusion under `## Build Discipline (ponytail)`.
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
    fn test_ponytail_embedded_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(markdown().contains("The Ladder"));
        assert!(markdown().contains("lazy senior developer"));
        assert!(markdown().contains("Does this need to exist at all?"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
    }

    #[test]
    fn test_ponytail_section_body() {
        let body = section_body();
        assert!(!body.is_empty());
        assert!(body.contains("lazy senior developer"));
    }
}

