// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/filters/test.rs
// created: 2026-10-27
// lastModified: 2026-11-05

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

    if bin == "cargo" && sub == "nextest" {
        filter_cargo_nextest(&clean, exit_code)
    } else if bin == "cargo" && sub == "test" {
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
    } else if bin == "sbt" && (sub == "test" || sub.contains("test") || sub.ends_with("/test")) {
        filter_sbt_test(&clean, exit_code)
    } else if bin == "rake" && (sub == "test" || cmd.contains("test")) {
        filter_rake_minitest(&clean, exit_code)
    } else if bin == "artisan" && sub == "test" {
        filter_artisan_test(&clean, exit_code)
    } else if bin == "paratest" || bin == "phpt" {
        filter_phpunit(&clean, exit_code)
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

fn nextest_status(trimmed: &str) -> &str {
    match trimmed.strip_prefix("TRY ") {
        Some(rest) => rest
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .trim_start(),
        None => trimmed,
    }
}

fn nextest_test_name(block: &str) -> &str {
    let line = block.lines().next().unwrap_or_default();
    let name = line.split_once(']').map_or(line, |(_, rest)| rest).trim();
    name.strip_prefix('(')
        .and_then(|rest| rest.split_once(") "))
        .map_or(name, |(_, name)| name)
}

/// Filter `cargo nextest` output:
/// Suppresses compilation noise and passing tests, extracts failure blocks with retry awareness,
/// and outputs compact summary.
pub fn filter_cargo_nextest(raw: &str, exit_code: i32) -> String {
    let mut failures: Vec<String> = Vec::new();
    let mut in_failure_block = false;
    let mut current_failure_header = String::new();
    let mut current_failure_body: Vec<String> = Vec::new();
    let mut summary_line = String::new();
    let mut binaries: u32 = 0;
    let mut passed_count = 0;
    let mut failed_count = 0;

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Compiling")
            || trimmed.starts_with("Checking")
            || trimmed.starts_with("Downloading")
            || trimmed.starts_with("Downloaded")
            || trimmed.starts_with("Finished")
            || trimmed.starts_with("Locking")
            || trimmed.starts_with("Updating")
            || trimmed.starts_with("────")
            || trimmed.starts_with("Nextest run ID")
        {
            continue;
        }

        if trimmed.starts_with("Starting") {
            if let Some(pos) = trimmed.find("across ") {
                let rest = &trimmed[pos + 7..];
                if let Some(num_str) = rest.split_whitespace().next() {
                    binaries = num_str.parse().unwrap_or(0);
                }
            }
            continue;
        }

        let status = nextest_status(trimmed);
        let is_failure = status.starts_with("FAIL")
            || status.starts_with("TIMEOUT")
            || status.starts_with("TMT");

        if is_failure || status.starts_with("PASS") {
            if in_failure_block {
                let mut block = current_failure_header.clone();
                if !current_failure_body.is_empty() {
                    block.push('\n');
                    block.push_str(&current_failure_body.join("\n"));
                }
                failures.push(block);
                current_failure_header.clear();
                current_failure_body.clear();
                in_failure_block = false;
            }

            let name = nextest_test_name(trimmed);
            failures.retain(|b| nextest_test_name(b) != name);

            if is_failure {
                failed_count += 1;
                current_failure_header = trimmed.to_string();
                in_failure_block = true;
            } else {
                passed_count += 1;
            }
            continue;
        }

        if in_failure_block {
            if trimmed.starts_with("Summary") {
                let mut block = current_failure_header.clone();
                if !current_failure_body.is_empty() {
                    block.push('\n');
                    block.push_str(&current_failure_body.join("\n"));
                }
                failures.push(block);
                current_failure_header.clear();
                current_failure_body.clear();
                in_failure_block = false;
                summary_line = trimmed.to_string();
            } else {
                current_failure_body.push(line.to_string());
            }
            continue;
        }

        if trimmed.starts_with("Summary") {
            summary_line = trimmed.to_string();
        }
    }

    if in_failure_block {
        let mut block = current_failure_header;
        if !current_failure_body.is_empty() {
            block.push('\n');
            block.push_str(&current_failure_body.join("\n"));
        }
        failures.push(block);
    }

    if failures.is_empty() {
        if exit_code == 0 {
            if !summary_line.is_empty() {
                // Example: Summary [ 2.503s] 2 tests run: 2 passed (1 slow), 0 skipped
                if let Some(dur_start) = summary_line.find('[') {
                    if let Some(dur_end) = summary_line.find(']') {
                        let dur = summary_line[dur_start + 1..dur_end].trim();
                        let bin_str = if binaries > 0 {
                            format!(" ({} binary, {})", binaries, dur)
                        } else {
                            format!(" ({})", dur)
                        };
                        return format!("cargo nextest: {} passed{}", passed_count.max(1), bin_str);
                    }
                }
                return format!("cargo nextest: {}", summary_line);
            }
            return format!("cargo nextest: {} passed", passed_count.max(1));
        } else {
            return format!("cargo nextest: failed (exit code {})", exit_code);
        }
    }

    let mut out = Vec::new();
    out.push(failures.join("\n\n"));
    if !summary_line.is_empty() {
        out.push(summary_line);
    } else {
        out.push(format!("cargo nextest: {} failed, {} passed", failed_count.max(failures.len()), passed_count));
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

/// Filter Scala SBT test output (ScalaTest & munit).
pub fn filter_sbt_test(raw: &str, exit_code: i32) -> String {
    let mut succeeded = 0;
    let mut failed = 0;
    let mut ignored = 0;
    let mut canceled = 0;
    let mut pending = 0;
    let mut suites = 0;
    let mut failures = Vec::new();
    let mut has_summary = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        let mut content = trimmed;
        if let Some(rest) = content.strip_prefix("[info]") {
            content = rest.trim();
        } else if let Some(rest) = content.strip_prefix("[error]") {
            content = rest.trim();
        }

        if content.starts_with("Tests: succeeded ") {
            // Tests: succeeded N, failed N, canceled N, ignored N, pending N
            let parts: Vec<&str> = content.split(',').collect();
            for part in parts {
                let p = part.trim();
                if let Some(rest) = p.strip_prefix("Tests: succeeded ") {
                    succeeded = rest.parse().unwrap_or(0);
                    has_summary = true;
                } else if let Some(rest) = p.strip_prefix("succeeded ") {
                    succeeded = rest.parse().unwrap_or(0);
                    has_summary = true;
                } else if let Some(rest) = p.strip_prefix("failed ") {
                    failed = rest.parse().unwrap_or(0);
                } else if let Some(rest) = p.strip_prefix("canceled ") {
                    canceled = rest.parse().unwrap_or(0);
                } else if let Some(rest) = p.strip_prefix("ignored ") {
                    ignored = rest.parse().unwrap_or(0);
                } else if let Some(rest) = p.strip_prefix("pending ") {
                    pending = rest.parse().unwrap_or(0);
                }
            }
            continue;
        }

        if trimmed.contains("*** FAILED ***") {
            let name = trimmed
                .replace("*** FAILED ***", "")
                .replace("[info]", "")
                .trim()
                .trim_start_matches('-')
                .trim()
                .to_string();
            failures.push(name);
            continue;
        }

        if content.starts_with("Suites: completed ") {
            let p = content.trim_start_matches("Suites: completed ");
            if let Some(num_str) = p.split(',').next() {
                suites = num_str.trim().parse().unwrap_or(0);
            }
        }
    }

    if exit_code == 0 && failed == 0 && canceled == 0 {
        if has_summary {
            let mut summary = format!("sbt test: {} passed", succeeded);
            if ignored > 0 {
                summary.push_str(&format!(", {} ignored", ignored));
            }
            if pending > 0 {
                summary.push_str(&format!(", {} pending", pending));
            }
            if suites > 0 {
                summary.push_str(&format!(" ({} suites)", suites));
            }
            summary
        } else {
            "sbt test: all tests passed".to_string()
        }
    } else {
        let mut out = Vec::new();
        if has_summary {
            out.push(format!("sbt test: {} passed, {} failed", succeeded, failed));
        } else {
            out.push(format!("sbt test: FAILED (exit code {})", exit_code));
        }
        if !failures.is_empty() {
            out.push("Failures:".to_string());
            for f in &failures {
                out.push(format!("  [FAIL] {}", f));
            }
        }
        out.join("\n")
    }
}

/// Filter Ruby Rake / Minitest output.
pub fn filter_rake_minitest(raw: &str, exit_code: i32) -> String {
    let mut runs = 0;
    let mut assertions = 0;
    let mut failures = 0;
    let mut errors = 0;
    let mut skips = 0;
    let mut failure_blocks = Vec::new();
    let mut in_failure = false;
    let mut current_block = Vec::new();
    let mut has_summary = false;

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.contains("runs,") && trimmed.contains("assertions,") {
            // N runs, N assertions, N failures, N errors, N skips
            has_summary = true;
            for part in trimmed.split(',') {
                let p = part.trim();
                let nums: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(val) = nums.parse::<usize>() {
                    if p.contains("run") {
                        runs = val;
                    } else if p.contains("assertion") {
                        assertions = val;
                    } else if p.contains("failure") {
                        failures = val;
                    } else if p.contains("error") {
                        errors = val;
                    } else if p.contains("skip") {
                        skips = val;
                    }
                }
            }
            continue;
        }

        if trimmed.starts_with("Failure:") || trimmed.starts_with("Error:") {
            if in_failure && !current_block.is_empty() {
                failure_blocks.push(current_block.join("\n"));
                current_block.clear();
            }
            in_failure = true;
            current_block.push(trimmed.to_string());
            continue;
        }

        if in_failure {
            if trimmed.is_empty() || trimmed.starts_with("Finished in") {
                in_failure = false;
                if !current_block.is_empty() {
                    failure_blocks.push(current_block.join("\n"));
                    current_block.clear();
                }
            } else if current_block.len() < 5 {
                current_block.push(line.to_string());
            }
        }
    }

    if in_failure && !current_block.is_empty() {
        failure_blocks.push(current_block.join("\n"));
    }

    if exit_code == 0 && failures == 0 && errors == 0 {
        if has_summary {
            format!("minitest: {} runs, {} assertions (all passed)", runs, assertions)
        } else {
            "minitest: all tests passed".to_string()
        }
    } else {
        let mut out = Vec::new();
        out.push(format!("minitest: {} runs, {} assertions, {} failures, {} errors", runs, assertions, failures, errors));
        if skips > 0 {
            out.push(format!("({} skipped)", skips));
        }
        if !failure_blocks.is_empty() {
            out.push(failure_blocks.join("\n\n"));
        }
        out.join("\n")
    }
}

/// Filter PHP Laravel Artisan test runner output.
pub fn filter_artisan_test(raw: &str, exit_code: i32) -> String {
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();
    let mut summary = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("PASS ") {
            passed += 1;
        } else if trimmed.starts_with("FAIL ") {
            failed += 1;
            failures.push(trimmed.to_string());
        } else if trimmed.starts_with("Tests:") || trimmed.contains("passed") && trimmed.contains("failed") {
            summary = trimmed.to_string();
        }
    }

    if exit_code == 0 && failed == 0 {
        if !summary.is_empty() {
            format!("artisan test: {}", summary)
        } else {
            format!("artisan test: {} passed", passed)
        }
    } else {
        let mut out = Vec::new();
        if !summary.is_empty() {
            out.push(format!("artisan test: {}", summary));
        } else {
            out.push(format!("artisan test: FAILED (exit code {})", exit_code));
        }
        if !failures.is_empty() {
            out.push("Failures:".to_string());
            for f in failures.iter().take(10) {
                out.push(format!("  {}", f));
            }
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

    #[test]
    fn test_filter_sbt_test() {
        let raw = "[info] - should compute sum *** FAILED ***\n[info]   42 was not equal to 43\n[info] Suites: completed 2, aborted 0\n[info] Tests: succeeded 5, failed 1, canceled 0, ignored 0, pending 0\n";
        let out = filter_sbt_test(raw, 1);
        assert!(out.contains("sbt test: 5 passed, 1 failed"));
        assert!(out.contains("should compute sum"));

        let pass_raw = "[info] Suites: completed 2, aborted 0\n[info] Tests: succeeded 6, failed 0, canceled 0, ignored 0, pending 0\n";
        let pass_out = filter_sbt_test(pass_raw, 0);
        assert_eq!(pass_out, "sbt test: 6 passed (2 suites)");
    }

    #[test]
    fn test_filter_rake_minitest() {
        let raw = "Run options: --seed 1234\n\n# Running:\n\n..\n\nFinished in 0.002s\n2 runs, 2 assertions, 0 failures, 0 errors, 0 skips\n";
        let out = filter_rake_minitest(raw, 0);
        assert_eq!(out, "minitest: 2 runs, 2 assertions (all passed)");
    }

    #[test]
    fn test_filter_artisan_test() {
        let raw = "   PASS  Tests\\Unit\\ExampleTest\n  ✓ basic test\n\n  Tests:  1 passed (1 assertions)\n  Duration: 0.05s\n";
        let out = filter_artisan_test(raw, 0);
        assert!(out.contains("artisan test: Tests:  1 passed (1 assertions)"));
    }
}

