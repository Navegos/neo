// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase1_rtk_verification.rs

use tokless::rtk::{
    compress_tool_output, extract_root_cause, filter_command_output, filter_sandbox_output,
    has_ansi, is_already_rtk, is_filterable_command, list_supported_commands,
    rtk_rewrite, strip_ansi, strip_ansi_cow, visible_len,
};
use tokless::commands::run_rtk_cli;

#[test]
fn test_rtk_ansi_stripper_zero_allocation_and_accuracy() {
    let plain = "Clean text without ansi escape sequences";
    let cow = strip_ansi_cow(plain);
    assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    assert_eq!(cow.as_ref(), plain);
    assert_eq!(visible_len(plain), plain.len());
    assert!(!has_ansi(plain));

    let colored = "\x1b[31;1mError:\x1b[0m \x1b[32mSuccess\x1b[0m";
    assert!(has_ansi(colored));
    assert_eq!(strip_ansi(colored), "Error: Success");
    assert_eq!(visible_len(colored), 14);

    let osc_title = "\x1b]0;Term Title\x07Running build...\x1b[0m";
    assert_eq!(strip_ansi(osc_title), "Running build...");
}

#[test]
fn test_rtk_supported_commands_count_over_100() {
    let commands = list_supported_commands();
    assert!(
        commands.len() >= 100,
        "Expected at least 100 supported commands, found {}",
        commands.len()
    );

    // Verify critical categories
    assert!(commands.contains(&"git"));
    assert!(commands.contains(&"cargo"));
    assert!(commands.contains(&"pytest"));
    assert!(commands.contains(&"vitest"));
    assert!(commands.contains(&"tsc"));
    assert!(commands.contains(&"docker"));
    assert!(commands.contains(&"kubectl"));
    assert!(commands.contains(&"aws"));
}

#[test]
fn test_rtk_git_filters() {
    assert!(is_filterable_command("git status"));
    assert!(is_filterable_command("git diff"));
    assert!(is_filterable_command("git show"));
    assert!(is_filterable_command("git log"));

    let status_raw = "## main...origin/main [ahead 2]\nM  crates/tokless/src/rtk/mod.rs\n?? new_test.rs\n";
    let status_filtered = filter_command_output("git status", status_raw, "", 0);
    assert!(status_filtered.contains("branch: main...origin/main [ahead 2]"));
    assert!(status_filtered.contains("staged (1):"));
    assert!(status_filtered.contains("untracked (1):"));

    let diff_raw = "diff --git a/Cargo.lock b/Cargo.lock\nindex 1234..5678 100644\n--- a/Cargo.lock\n+++ b/Cargo.lock\n@@ -1,2 +1,3 @@\n+foo\n";
    let diff_filtered = filter_command_output("git diff", diff_raw, "", 0);
    assert!(diff_filtered.contains("[lockfile updated: Cargo.lock (+1 -0)]"));
}

#[test]
fn test_rtk_cargo_and_compiler_filters() {
    assert!(is_filterable_command("cargo build"));
    assert!(is_filterable_command("cargo test"));
    assert!(is_filterable_command("tsc --noEmit"));

    let test_raw = "running 3 tests\ntest tests::test_one ... ok\ntest tests::test_fail ... FAILED\ntest tests::test_two ... ok\n\nfailures:\n---- tests::test_fail stdout ----\nthread 'tests::test_fail' panicked at 'assertion failed: 1 == 2'\n\nfailures:\n    tests::test_fail\n\ntest result: FAILED. 2 passed; 1 failed;\n";
    let test_filtered = filter_command_output("cargo test", test_raw, "", 101);
    assert!(test_filtered.contains("FAILED"));
    assert!(test_filtered.contains("assertion failed: 1 == 2"));
    assert!(test_filtered.contains("2 passed"));

    let tsc_raw = "src/index.ts(12,5): error TS2322: Type 'string' is not assignable to type 'number'.\n";
    let tsc_filtered = filter_command_output("tsc", tsc_raw, "", 1);
    assert!(tsc_filtered.contains("TS2322"));
    assert!(tsc_filtered.contains("1 errors in 1 files"));
}

#[test]
fn test_rtk_cross_tool_adapters() {
    // 1. ContextMode sandbox filtering
    let cmd = "git status";
    let raw = "## master\nM  file.txt\n";
    let filtered_sandbox = filter_sandbox_output(cmd, raw);
    assert!(filtered_sandbox.contains("staged (1):"));

    // 2. Headroom proxy compression
    let ansi_tool = "\x1b[31merror[E0308]: mismatched types\x1b[0m\n";
    let compressed_tool = compress_tool_output("cargo check", ansi_tool);
    assert!(compressed_tool.contains("error[E0308]: mismatched types"));
    assert!(!has_ansi(&compressed_tool));

    // 3. ProjectMem root cause extraction
    let cargo_err = "   Compiling app v0.1.0\nerror[E0433]: failed to resolve\nerror: could not compile `app`";
    let cause = extract_root_cause("cargo build", cargo_err);
    assert_eq!(cause.as_deref(), Some("error[E0433]: failed to resolve"));

    let pytest_err = "def test_answer():\n>       assert func(3) == 5\nE       AssertionError: assert 4 == 5";
    let py_cause = extract_root_cause("pytest tests/", pytest_err);
    assert_eq!(py_cause.as_deref(), Some("AssertionError: assert 4 == 5"));
}

#[test]
fn test_rtk_hook_rewrites() {
    assert!(!is_already_rtk("git status"));
    assert!(is_already_rtk("tokless rtk git status"));
    assert!(is_already_rtk("rtk git status"));

    let (rewritten, changed) = rtk_rewrite("git status");
    assert!(changed);
    assert_eq!(rewritten, "tokless rtk git status");

    let (unchanged, did_change) = rtk_rewrite("tokless rtk git status");
    assert!(!did_change);
    assert!(unchanged.is_empty());
}

#[test]
fn test_rtk_cli_list_and_help() {
    assert_eq!(run_rtk_cli(&["--help".to_string()]), 0);
    assert_eq!(run_rtk_cli(&["list".to_string()]), 0);
}
