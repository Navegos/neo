// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/mod.rs
// created: 2026-10-27
// lastModified: 2026-10-27

pub mod ansi;
pub mod engine;
pub mod filters;
pub mod hook;

pub use ansi::{has_ansi, strip_ansi, strip_ansi_cow, visible_len};
pub use engine::{can_filter, filter_output, list_supported_commands, run_command_filtered, tokenize_command};
pub use filters::{filter_command_output, is_filterable_command};
pub use hook::{is_already_rtk, rtk_rewrite, run_rtk_rewrite};

/// Cross-tool adapter for Context-Mode:
/// Filters sandbox subprocess outputs through RTK command profiles.
pub fn filter_sandbox_output(cmd: &str, output: &str) -> String {
    if can_filter(cmd) {
        filter_output(cmd, output, "", 0)
    } else {
        strip_ansi(output)
    }
}

/// Cross-tool adapter for Headroom Proxy:
/// Pre-compresses tool outputs on the streaming reverse proxy pipeline before LLM token alignment.
pub fn compress_tool_output(cmd_or_tool: &str, output: &str) -> String {
    if can_filter(cmd_or_tool) {
        filter_output(cmd_or_tool, output, "", 0)
    } else {
        strip_ansi(output)
    }
}

/// Cross-tool adapter for ProjectMem:
/// Extracts root cause error lines from failed test/build outputs to populate `record_attempt(outcome="failed")`.
pub fn extract_root_cause(cmd: &str, output: &str) -> Option<String> {
    let clean = strip_ansi(output);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error[E")
            || (trimmed.starts_with("error:") && !trimmed.contains("could not compile"))
        {
            return Some(trimmed.to_string());
        }
        if bin == "cargo" || cmd.contains("cargo") {
            if trimmed.contains("assertion `left == right` failed")
                || trimmed.contains("panicked at")
                || (trimmed.starts_with("thread '") && trimmed.contains("panicked"))
            {
                return Some(trimmed.to_string());
            }
        }
        if (bin == "pytest" || bin == "python" || cmd.contains("pytest")) && trimmed.starts_with("E   ") {
            return Some(trimmed.trim_start_matches("E   ").trim().to_string());
        }
        if (bin == "vitest" || bin == "jest" || bin == "npm" || bin == "pnpm")
            && (trimmed.contains("AssertionError:") || trimmed.starts_with("Error: "))
        {
            return Some(trimmed.to_string());
        }
        if (bin == "tsc" || cmd.contains("tsc")) && trimmed.contains(": error TS") {
            return Some(trimmed.to_string());
        }
        if (bin == "go" || cmd.contains("go")) && (trimmed.starts_with("--- FAIL:") || trimmed.starts_with("panic:")) {
            return Some(trimmed.to_string());
        }
        if (bin == "dotnet" || cmd.contains("dotnet"))
            && (trimmed.contains(": error CS") || trimmed.starts_with("Failed "))
        {
            return Some(trimmed.to_string());
        }
        if (bin == "mvn" || bin == "gradle" || bin == "gradlew")
            && (trimmed.starts_with("[ERROR]") || trimmed.starts_with("FAILURE:"))
        {
            return Some(trimmed.to_string());
        }
        if (bin == "mypy" || bin == "ruff") && (trimmed.contains(": error:") || trimmed.contains(": E")) {
            return Some(trimmed.to_string());
        }
        if (bin == "playwright" || cmd.contains("playwright"))
            && (trimmed.starts_with("Error: expect") || trimmed.starts_with("locator."))
        {
            return Some(trimmed.to_string());
        }
    }

    // Fallback: look for general exception or error patterns regardless of bin
    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("AssertionError:")
            || trimmed.starts_with("TypeError:")
            || trimmed.starts_with("ReferenceError:")
            || trimmed.starts_with("SyntaxError:")
            || trimmed.starts_with("RuntimeError:")
            || trimmed.starts_with("panic:")
            || trimmed.starts_with("FATAL:")
        {
            return Some(trimmed.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_sandbox_output() {
        let raw = "## main...origin/main\nM  foo.rs\n";
        let out = filter_sandbox_output("git status", raw);
        assert!(out.contains("staged (1):"));
    }

    #[test]
    fn test_compress_tool_output() {
        let raw = "\x1b[31merror[E0308]: mismatched types\x1b[0m\n";
        let out = compress_tool_output("cargo check", raw);
        assert!(out.contains("error[E0308]: mismatched types"));
        assert!(!crate::rtk::has_ansi(&out));
    }

    #[test]
    fn test_extract_root_cause() {
        let cargo_err = "   Compiling app v0.1.0\nerror[E0433]: failed to resolve\nerror: could not compile `app`";
        assert_eq!(
            extract_root_cause("cargo build", cargo_err),
            Some("error[E0433]: failed to resolve".to_string())
        );

        let pytest_err = "def test_answer():\n>       assert func(3) == 5\nE       AssertionError: assert 4 == 5";
        assert_eq!(
            extract_root_cause("pytest tests/", pytest_err),
            Some("AssertionError: assert 4 == 5".to_string())
        );

        let go_err = "=== RUN   TestFailure\n--- FAIL: TestFailure (0.00s)\nFAIL\n";
        assert_eq!(
            extract_root_cause("go test ./...", go_err),
            Some("--- FAIL: TestFailure (0.00s)".to_string())
        );
    }
}

