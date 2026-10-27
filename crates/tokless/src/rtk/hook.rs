// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/hook.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::env;
use std::io::{self, Read};
use crate::rtk::engine::{can_filter, tokenize_command};

/// Check if a command segment already invokes rtk or tokless rtk.
pub fn is_already_rtk(cmd_line: &str) -> bool {
    let trimmed = cmd_line.trim();
    trimmed.starts_with("rtk ")
        || trimmed == "rtk"
        || trimmed.starts_with("tokless rtk ")
        || trimmed == "tokless rtk"
}

/// Check if a command contains shell pipes or complex redirects that should bypass auto-rewrite.
pub fn has_shell_pipe(cmd_line: &str) -> bool {
    let mut in_quote = false;
    let mut quote_char = ' ';

    for c in cmd_line.chars() {
        match c {
            '\'' | '"' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            '|' if !in_quote => return true,
            '>' if !in_quote => return true,
            '<' if !in_quote => return true,
            _ => {}
        }
    }

    false
}

/// Check if a command is unsafe for automatic rewrite (e.g. destructive commands).
pub fn is_unsafe_for_rewrite(cmd_line: &str) -> bool {
    let lower = cmd_line.to_lowercase();
    lower.contains("rm ")
        || lower.contains("rmdir")
        || lower.contains("del ")
        || lower.contains("format ")
        || lower.contains("drop database")
        || lower.contains("git reset --hard")
        || lower.contains("git clean -f")
}

/// Rewrite a raw command line to invoke tokless rtk if filterable.
pub fn rtk_rewrite(cmd_line: &str) -> (String, bool) {
    let trimmed = cmd_line.trim();
    if trimmed.is_empty() || is_already_rtk(trimmed) || has_shell_pipe(trimmed) || is_unsafe_for_rewrite(trimmed) {
        return (String::new(), false);
    }

    let tokens = tokenize_command(trimmed);
    if tokens.is_empty() {
        return (String::new(), false);
    }

    if can_filter(trimmed) {
        let rewritten = format!("tokless rtk {}", trimmed);
        return (rewritten, true);
    }

    (String::new(), false)
}

/// CLI runner for `tokless rtk-rewrite` command.
pub fn run_rtk_rewrite() -> i32 {
    let args: Vec<String> = env::args().skip(2).collect();
    let mut actual_args = &args[..];
    for (i, a) in args.iter().enumerate() {
        if a == "--" {
            actual_args = &args[i + 1..];
            break;
        }
    }

    let mut cmd_line = actual_args.join(" ").trim().to_string();
    if cmd_line.is_empty() {
        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_ok() {
            cmd_line = buf.trim().to_string();
        }
    }

    let (out, changed) = rtk_rewrite(&cmd_line);
    if !changed || out.is_empty() {
        return 1;
    }

    println!("{}", out);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtk_rewrite_cargo_test() {
        let (out, changed) = rtk_rewrite("cargo test");
        assert!(changed);
        assert_eq!(out, "tokless rtk cargo test");
    }

    #[test]
    fn test_rtk_rewrite_git_status() {
        let (out, changed) = rtk_rewrite("git status -s");
        assert!(changed);
        assert_eq!(out, "tokless rtk git status -s");
    }

    #[test]
    fn test_rtk_rewrite_skips_pipes() {
        let (_, changed) = rtk_rewrite("cargo test | head -n 10");
        assert!(!changed);
    }

    #[test]
    fn test_rtk_rewrite_skips_already_rtk() {
        let (_, changed) = rtk_rewrite("tokless rtk cargo test");
        assert!(!changed);
        let (_, changed2) = rtk_rewrite("rtk git diff");
        assert!(!changed2);
    }
}

