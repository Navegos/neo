// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/mod.rs
// created: 2026-10-27
// lastModified: 2026-11-08

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
    let raw_bin = parts.first().copied().unwrap_or("");
    let bin = filters::normalize_bin_name(raw_bin);

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error[E")
            || (trimmed.starts_with("error:") && !trimmed.contains("could not compile"))
        {
            return Some(trimmed.to_string());
        }
        if (bin == "cargo" || cmd.contains("cargo"))
            && (trimmed.contains("assertion `left == right` failed")
                || trimmed.contains("panicked at")
                || (trimmed.starts_with("thread '") && trimmed.contains("panicked")))
        {
            return Some(trimmed.to_string());
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
        if (bin == "sbt" || cmd.contains("sbt"))
            && (trimmed.contains("*** FAILED ***") || (trimmed.starts_with("[error]") && !trimmed.contains("Total time:")))
        {
            return Some(trimmed.to_string());
        }
        if (bin == "rubocop" || bin == "rake" || cmd.contains("rubocop") || cmd.contains("rake"))
            && (trimmed.starts_with("Failure:") || trimmed.starts_with("Error:") || (trimmed.contains(".rb:") && trimmed.contains(": ")))
        {
            return Some(trimmed.to_string());
        }
        if (bin == "phpunit" || bin == "artisan" || bin == "phpstan" || bin == "pest")
            && (trimmed.starts_with("FAIL ") || trimmed.starts_with("FAILURES!") || trimmed.starts_with("Fatal error:") || (trimmed.starts_with("[ERROR]") && trimmed.contains("errors")))
        {
            return Some(trimmed.to_string());
        }
        if bin == "golangci-lint" && trimmed.contains(".go:") {
            return Some(trimmed.to_string());
        }
        if (bin == "git" || cmd.contains("git")) && (trimmed.starts_with("fatal:") || trimmed.starts_with("error:")) {
            return Some(trimmed.to_string());
        }
        if (bin == "docker" || bin == "podman")
            && (trimmed.starts_with("Error response from daemon:") || trimmed.contains("Cannot connect to the Docker daemon"))
        {
            return Some(trimmed.to_string());
        }
        if bin == "kubectl" && trimmed.starts_with("Error from server") {
            return Some(trimmed.to_string());
        }
        if bin == "ctest" && (trimmed.contains("***Failed") || trimmed.starts_with("The following tests FAILED:")) {
            return Some(trimmed.to_string());
        }
        if (bin == "gcc" || bin == "g++" || bin == "clang" || bin == "clang++") && trimmed.contains(": error:") {
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

        let sbt_err = "[info] - math.Add should return 4 *** FAILED ***\n[info]   42 was not equal to 4";
        assert_eq!(
            extract_root_cause("sbt test", sbt_err),
            Some("[info] - math.Add should return 4 *** FAILED ***".to_string())
        );

        let php_err = "Fatal error: Uncaught Error: Call to undefined function foo()\n";
        assert_eq!(
            extract_root_cause("artisan test", php_err),
            Some("Fatal error: Uncaught Error: Call to undefined function foo()".to_string())
        );

        // Path normalization checks
        let path_cargo_err = "error[E0433]: failed to resolve";
        assert_eq!(
            extract_root_cause("C:\\tools\\cargo.exe test", path_cargo_err),
            Some("error[E0433]: failed to resolve".to_string())
        );
        assert_eq!(
            extract_root_cause("/usr/bin/cargo test", path_cargo_err),
            Some("error[E0433]: failed to resolve".to_string())
        );
    }

    #[test]
    fn test_normalize_bin_name() {
        assert_eq!(filters::normalize_bin_name("/usr/bin/git"), "git");
        assert_eq!(filters::normalize_bin_name("C:\\tools\\git.exe"), "git");
        assert_eq!(filters::normalize_bin_name("\"./node_modules/.bin/eslint\""), "eslint");
        assert_eq!(filters::normalize_bin_name("cargo.cmd"), "cargo");
    }

    #[test]
    fn test_path_invoked_command_filtering() {
        let raw = "## main...origin/main\nM  foo.rs\n";
        let out = filter_sandbox_output("/usr/bin/git status", raw);
        assert!(out.contains("staged (1):"));
    }
}

