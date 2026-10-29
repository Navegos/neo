// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/filters/test.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use crate::rtk::ansi::strip_ansi;

/// Filter test runner outputs.
pub fn filter_test(cmd: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{}\n{}", stdout, stderr)
    };
    let clean = strip_ansi(&combined);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");
    let sub = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    if bin == "cargo" && (sub == "test" || sub == "nextest") {
        filter_cargo_test(&clean, exit_code)
    } else if bin == "pytest" || (bin == "python" && sub == "-m" && cmd.contains("pytest")) {
        filter_pytest(&clean, exit_code)
    } else if bin == "vitest" || (bin == "npm" && sub == "test") || bin == "jest" {
        filter_vitest_npm(&clean, exit_code)
    } else if bin == "go" && sub == "test" {
        filter_go_test(&clean, exit_code)
    } else if bin == "dotnet" && sub == "test" {
        filter_dotnet_test(&clean, exit_code)
    } else if bin == "rspec" {
        filter_rspec(&clean, exit_code)
    } else if bin == "phpunit" || bin == "pest" {
        filter_phpunit(&clean, exit_code)
    } else if bin == "playwright" || (bin == "npx" && sub == "playwright") {
        filter_playwright(&clean, exit_code)
    } else if bin == "ctest" {
        filter_ctest(&clean, exit_code)
    } else {
        clean
    }
}

/// Filter `cargo test` output:
/// Suppresses `test ... ok`, extracts failure blocks, assertions, stack traces, and summary.
pub fn filter_cargo_test(raw: &str, exit_code: i32) -> String {
    let mut passed_count = 0;
    let mut failed_tests = Vec::new();
    let mut failure_blocks = Vec::new();
    let mut current_block = Vec::new();
    let mut in_failure = false;
    let mut test_summaries = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("test ") && trimmed.ends_with("... ok") {
            passed_count += 1;
            continue;
        }

        if trimmed.starts_with("test ") && trimmed.ends_with("... FAILED") {
            let name = trimmed.trim_start_matches("test ").trim_end_matches("... FAILED").trim();
            failed_tests.push(name.to_string());
            continue;
        }

        if trimmed.starts_with("---- ") && trimmed.ends_with(" stdout ----") {
            if in_failure && !current_block.is_empty() {
                failure_blocks.push(current_block.join("\n"));
                current_block.clear();
            }
            in_failure = true;
            current_block.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("failures:") && in_failure {
            if !current_block.is_empty() {
                failure_blocks.push(current_block.join("\n"));
                current_block.clear();
            }
            in_failure = false;
            continue;
        }

        if trimmed.starts_with("test result:") {
            test_summaries.push(trimmed.to_string());
            if in_failure && !current_block.is_empty() {
                failure_blocks.push(current_block.join("\n"));
                current_block.clear();
            }
            in_failure = false;
            continue;
        }

        if in_failure {
            current_block.push(line.to_string());
        }
    }

    if in_failure && !current_block.is_empty() {
        failure_blocks.push(current_block.join("\n"));
    }

    if exit_code == 0 && failed_tests.is_empty() && failure_blocks.is_empty() {
        let suite_count = test_summaries.len().max(1);
        return format!("cargo test: {} passed ({} suites)", passed_count, suite_count);
    }

    let mut out = Vec::new();
    if !failed_tests.is_empty() {
        out.push(format!("FAILED TESTS ({}):\n  {}", failed_tests.len(), failed_tests.join("\n  ")));
    }
    if !failure_blocks.is_empty() {
        out.push(failure_blocks.join("\n\n"));
    }
    if !test_summaries.is_empty() {
        out.push(test_summaries.join("\n"));
    } else {
        out.push(format!("test result: FAILED (exit code {})", exit_code));
    }

    out.join("\n")
}

/// Filter `pytest` output:
/// Suppresses passing dots and test lists, captures `FAILURES` sections and summary.
pub fn filter_pytest(raw: &str, exit_code: i32) -> String {
    let mut failures = Vec::new();
    let mut in_failures = false;
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("=== FAILURES ===") {
            in_failures = true;
            continue;
        }
        if trimmed.starts_with("=== short test summary info ===") || (trimmed.starts_with("===") && trimmed.contains("passed")) {
            in_failures = false;
            if trimmed.contains("passed") || trimmed.contains("failed") {
                summary = trimmed.trim_matches('=').trim().to_string();
            }
            continue;
        }

        if in_failures {
            failures.push(line);
        }
    }

    if exit_code == 0 && failures.is_empty() {
        if !summary.is_empty() {
            format!("pytest: {}", summary)
        } else {
            "pytest: all tests passed".to_string()
        }
    } else {
        let mut out = Vec::new();
        if !failures.is_empty() {
            out.push(failures.join("\n"));
        }
        if !summary.is_empty() {
            out.push(format!("summary: {}", summary));
        } else {
            out.push(format!("pytest: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

/// Filter `vitest`, `jest`, `npm test` outputs.
pub fn filter_vitest_npm(raw: &str, exit_code: i32) -> String {
    let mut failures = Vec::new();
    let mut summary_lines = Vec::new();
    let mut in_fail_block = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("FAIL ") || trimmed.starts_with("✕ ") {
            in_fail_block = true;
            failures.push(line);
            continue;
        }
        if trimmed.starts_with("Test Files ") || trimmed.starts_with("Tests ") || trimmed.starts_with("Time ") {
            in_fail_block = false;
            summary_lines.push(trimmed);
            continue;
        }
        if in_fail_block && (trimmed.contains("AssertionError") || trimmed.contains("Expected:") || trimmed.contains("Received:") || trimmed.contains("at ")) {
            failures.push(line);
        }
    }

    if exit_code == 0 && failures.is_empty() {
        if !summary_lines.is_empty() {
            summary_lines.join(", ")
        } else {
            "test: all tests passed".to_string()
        }
    } else {
        let mut out = Vec::new();
        if !failures.is_empty() {
            out.push(failures.join("\n"));
        }
        if !summary_lines.is_empty() {
            out.push(summary_lines.join("\n"));
        } else {
            out.push(format!("test: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

/// Filter `go test` output:
/// Suppresses `=== RUN`, `--- PASS:`, preserves `--- FAIL:`, panic traces, and packages summary.
pub fn filter_go_test(raw: &str, exit_code: i32) -> String {
    let mut failures = Vec::new();
    let mut summary = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--- FAIL:") || trimmed.starts_with("panic:") {
            failures.push(trimmed.to_string());
        } else if trimmed.starts_with("FAIL\t") || trimmed.starts_with("ok\t") {
            summary.push(trimmed.to_string());
        }
    }

    if exit_code == 0 && failures.is_empty() {
        format!("go test: {} packages passed", summary.len())
    } else {
        let mut out = Vec::new();
        if !failures.is_empty() {
            out.push(failures.join("\n"));
        }
        if !summary.is_empty() {
            out.push(summary.join("\n"));
        } else {
            out.push(format!("go test: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

/// Filter `dotnet test` output.
pub fn filter_dotnet_test(raw: &str, exit_code: i32) -> String {
    let mut failed = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Failed ") {
            failed.push(trimmed.to_string());
        } else if trimmed.starts_with("Passed!") || trimmed.starts_with("Failed!") || trimmed.contains("Total tests:") {
            summary = trimmed.to_string();
        }
    }

    if exit_code == 0 && failed.is_empty() {
        "dotnet test: all tests passed".to_string()
    } else {
        let mut out = Vec::new();
        if !failed.is_empty() {
            out.push(failed.join("\n"));
        }
        if !summary.is_empty() {
            out.push(summary);
        } else {
            out.push(format!("dotnet test: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

/// Filter `rspec` (Ruby) output.
pub fn filter_rspec(raw: &str, exit_code: i32) -> String {
    let mut failures = Vec::new();
    let mut summary = String::new();
    let mut in_failures = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "Failures:" {
            in_failures = true;
            continue;
        }
        if trimmed.contains("examples,") && trimmed.contains("failures") {
            in_failures = false;
            summary = trimmed.to_string();
            continue;
        }
        if in_failures {
            failures.push(line);
        }
    }

    if exit_code == 0 && failures.is_empty() {
        format!("rspec: {}", summary)
    } else {
        let mut out = Vec::new();
        if !failures.is_empty() {
            out.push(failures.join("\n"));
        }
        if !summary.is_empty() {
            out.push(summary);
        } else {
            out.push(format!("rspec: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

/// Filter `phpunit` and `pest` (PHP) output.
pub fn filter_phpunit(raw: &str, exit_code: i32) -> String {
    let mut failures = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("There was 1 failure:") || trimmed.starts_with("There were") || trimmed.starts_with("FAILURES!") {
            failures.push(trimmed.to_string());
        } else if trimmed.starts_with("OK (") {
            summary = trimmed.to_string();
        }
    }

    if exit_code == 0 && failures.is_empty() {
        format!("phpunit: {}", summary)
    } else {
        let mut out = Vec::new();
        if !failures.is_empty() {
            out.push(failures.join("\n"));
        }
        out.push(format!("phpunit: FAILED (exit code {})", exit_code));
        out.join("\n")
    }
}

/// Filter `playwright` test runner outputs.
pub fn filter_playwright(raw: &str, exit_code: i32) -> String {
    let mut passed = 0;
    let mut failures = Vec::new();
    let mut in_fail = false;
    let mut summary = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("✓ ") || trimmed.starts_with("[pass]") {
            passed += 1;
            continue;
        }
        if trimmed.starts_with("✘ ") || trimmed.starts_with("✕ ") || trimmed.starts_with("Error: ") {
            in_fail = true;
            failures.push(line.to_string());
            continue;
        }
        if trimmed.starts_with("Passed:") || trimmed.contains("passed (") || trimmed.contains("failed (") {
            summary = Some(trimmed.to_string());
            in_fail = false;
            continue;
        }
        if in_fail {
            failures.push(line.to_string());
        }
    }

    if exit_code == 0 && failures.is_empty() {
        if let Some(s) = summary {
            format!("playwright: {}", s)
        } else {
            format!("playwright: {} tests passed", passed)
        }
    } else {
        let mut out = Vec::new();
        if !failures.is_empty() {
            out.push(failures.join("\n"));
        }
        if let Some(s) = summary {
            out.push(s);
        } else {
            out.push(format!("playwright: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

/// Filter `ctest` CMake test runner outputs.
pub fn filter_ctest(raw: &str, exit_code: i32) -> String {
    let mut passed = 0;
    let mut failed = Vec::new();
    let mut summary = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.contains("...   Passed") {
            passed += 1;
        } else if trimmed.contains("***Failed") || trimmed.contains("***Exception") {
            failed.push(trimmed.to_string());
        } else if trimmed.contains("% tests passed,") {
            summary = Some(trimmed.to_string());
        }
    }

    if exit_code == 0 && failed.is_empty() {
        if let Some(s) = summary {
            format!("ctest: {}", s)
        } else {
            format!("ctest: {} passed", passed)
        }
    } else {
        let mut out = Vec::new();
        if !failed.is_empty() {
            out.push("Failed tests:".to_string());
            for f in failed {
                out.push(format!("  {}", f));
            }
        }
        if let Some(s) = summary {
            out.push(s);
        } else {
            out.push(format!("ctest: FAILED (exit code {})", exit_code));
        }
        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_playwright() {
        let raw = "✓ 1 test.spec.ts:12:1 › should load page\n✓ 2 test.spec.ts:25:1 › should submit form\n  2 passed (3.4s)\n";
        let out = filter_playwright(raw, 0);
        assert_eq!(out, "playwright: 2 passed (3.4s)");
    }

    #[test]
    fn test_filter_ctest() {
        let raw = "1/2 Test #1: TestA ....................   Passed    0.02 sec\n2/2 Test #2: TestB ....................***Failed    0.05 sec\n\n50% tests passed, 1 tests failed out of 2\n";
        let out = filter_ctest(raw, 1);
        assert!(out.contains("Failed tests:"));
        assert!(out.contains("TestB"));
        assert!(out.contains("50% tests passed"));
    }

    #[test]
    fn test_filter_cargo_test_success() {
        let raw = "running 2 tests\ntest tests::test_one ... ok\ntest tests::test_two ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored\n";
        let out = filter_cargo_test(raw, 0);
        assert_eq!(out, "cargo test: 2 passed (1 suites)");
    }

    #[test]
    fn test_filter_cargo_test_failure() {
        let raw = "running 2 tests\ntest tests::test_one ... ok\ntest tests::test_fail ... FAILED\n\nfailures:\n\n---- tests::test_fail stdout ----\nassertion `left == right` failed\n  left: 1\n right: 2\n\nfailures:\n    tests::test_fail\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\n";
        let out = filter_cargo_test(raw, 101);
        assert!(out.contains("FAILED TESTS (1):"));
        assert!(out.contains("tests::test_fail"));
        assert!(out.contains("assertion `left == right` failed"));
        assert!(!out.contains("test tests::test_one ... ok"));
    }

    #[test]
    fn test_filter_pytest_success() {
        let raw = "============================= test session starts =============================\ncollected 5 items\n\ntest_sample.py .....                                                     [100%]\n\n============================== 5 passed in 0.12s ==============================\n";
        let out = filter_pytest(raw, 0);
        assert!(out.contains("pytest: 5 passed in 0.12s"));
    }
}

