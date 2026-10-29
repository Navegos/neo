// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rules/embedded/principles.rs

//! Karpathy behavioral coding guidelines.
//! Think before coding, simplicity first, surgical changes, goal-driven execution.

pub const ID: &str = "principles";
pub const TITLE: &str = "Principles";
pub const DESCRIPTION: &str = "Coding principles: think first, simplify, edit surgically, verify.";
pub const SECTION_HEADER: &str = "## Principles";
pub const CURSOR_FILENAME: &str = "principles.mdc";

pub const RAW_MARKDOWN: &str = include_str!("../assets/principles.md");
pub const CURSOR_RULE: &str = include_str!("../assets/principles.mdc");

/// Returns the raw markdown guidelines.
pub fn markdown() -> &'static str {
    RAW_MARKDOWN
}

/// Returns the Cursor project rule (.mdc format with YAML frontmatter).
pub fn cursor_rule() -> &'static str {
    CURSOR_RULE
}

/// Returns the formatted section body suitable for inclusion under `## Principles`.
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
    fn test_principles_embedded_assets() {
        assert!(!markdown().is_empty());
        assert!(!cursor_rule().is_empty());
        assert!(markdown().contains("Think Before Coding"));
        assert!(markdown().contains("Simplicity First"));
        assert!(markdown().contains("Surgical Changes"));
        assert!(markdown().contains("Goal-Driven Execution"));
        assert!(cursor_rule().starts_with("---\n"));
        assert!(cursor_rule().contains("alwaysApply: true"));
    }

    #[test]
    fn test_principles_section_body() {
        let body = section_body();
        assert!(!body.is_empty());
        assert!(body.contains("Behavioral guidelines to reduce common LLM coding mistakes"));
    }
}

