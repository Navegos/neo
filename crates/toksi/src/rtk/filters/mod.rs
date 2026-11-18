// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/filters/mod.rs
// created: 2026-10-27
// lastModified: 2026-11-18

pub mod build;
pub mod git;
pub mod system;
pub mod test;
pub mod toml;

use self::build::filter_build;
use self::git::{filter_gh, filter_git, filter_glab, filter_gt};
use self::system::filter_system;
use self::test::filter_test;
use self::toml::{apply_filter, find_matching_filter};

/// Normalize an executable path to its canonical base name.
/// Strips quotes, directory components (both `/` and `\`), and file extensions (`.exe`, `.cmd`, `.bat`).
pub fn normalize_bin_name(bin: &str) -> &str {
    let mut s = bin.trim();
    if ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
        && s.len() >= 2
    {
        s = &s[1..s.len() - 1];
    }
    if let Some(idx) = s.rfind(['/', '\\']) {
        s = &s[idx + 1..];
    }
    if let Some(stripped) = s.strip_suffix(".exe") {
        s = stripped;
    } else if let Some(stripped) = s.strip_suffix(".cmd") {
        s = stripped;
    } else if let Some(stripped) = s.strip_suffix(".bat") {
        s = stripped;
    }
    s
}

/// Extract the canonical subcommand for a normalized binary, skipping toolchain overrides
/// (`+nightly`) and prefix flags that take values (`-C dir`, `--manifest-path path`, etc.).
pub fn extract_subcommand<'a>(bin: &str, parts: &[&'a str]) -> &'a str {
    let mut i = 1;
    // Rustup +toolchain override is only valid as the first argument to cargo/rustc
    if matches!(bin, "cargo" | "rustc" | "rustup")
        && parts.get(i).is_some_and(|p| p.starts_with('+'))
    {
        i += 1;
    }

    while i < parts.len() {
        let arg = parts[i];
        if arg == "--" {
            return "";
        }
        if arg.starts_with('-') {
            // Skip `--flag=value` (single token)
            if arg.contains('=') {
                i += 1;
                continue;
            }
            let takes_value = match bin {
                "cargo" => matches!(
                    arg,
                    "-C" | "-Z"
                        | "-p"
                        | "--package"
                        | "--manifest-path"
                        | "--target-dir"
                        | "--config"
                        | "--color"
                        | "--message-format"
                        | "--target"
                        | "-j"
                        | "--jobs"
                        | "-F"
                        | "--features"
                        | "--explain"
                ),
                "git" => matches!(
                    arg,
                    "-C" | "-c"
                        | "--git-dir"
                        | "--work-tree"
                        | "--namespace"
                        | "--exec-path"
                        | "--super-prefix"
                        | "--config-env"
                ),
                _ => matches!(arg, "-C" | "--config" | "--color"),
            };
            i += if takes_value { 2 } else { 1 };
            continue;
        }
        return arg;
    }
    ""
}

/// Check if a command can be filtered by either native filters or declarative TOML filters.
pub fn is_filterable_command(cmd_line: &str) -> bool {
    let trimmed = cmd_line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.iter().any(|&p| p == "--version" || p == "-V" || p == "--help" || p == "-h" || p == "help") {
        return false;
    }
    let raw_bin = parts.first().copied().unwrap_or("");
    let bin = normalize_bin_name(raw_bin);
    let sub = extract_subcommand(bin, &parts);

    match bin {
        "git" | "gh" | "glab" | "gt" => true,
        "cargo" => true,
        "pytest" => true,
        "vitest" | "jest" => true,
        "tsc" | "mypy" | "ruff" | "make" => true,
        "npm" | "pnpm" | "yarn" | "bun" | "deno" | "npx" | "pnpx" | "bunx" => true,
        "pip" | "uv" => true,
        "eslint" | "prettier" | "rubocop" | "phpstan" | "pint" | "golangci-lint" | "sqlfluff" => true,
        "next" | "prisma" | "ng" => true,
        "playwright" | "ctest" => true,
        "go" if sub == "test" || sub == "build" || sub == "vet" => true,
        "dotnet" if sub == "test" || sub == "build" => true,
        "mvn" | "mvn.cmd" | "gradle" | "gradlew" | "sbt" => true,
        "rspec" | "phpunit" | "pest" | "rake" | "artisan" | "paratest" => true,
        "docker" | "podman" | "kubectl" | "curl" | "wget" | "ls" | "dir" | "tree" | "find" | "wc" | "env" | "psql"
        | "aws" | "ast-grep" | "sg" | "black" | "json" | "log" => true,
        _ => find_matching_filter(trimmed).is_some(),
    }
}

/// Filter command output based on the command line string.
pub fn filter_command_output(cmd_line: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let trimmed = cmd_line.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.iter().any(|&p| p == "--version" || p == "-V" || p == "--help" || p == "-h" || p == "help") {
        return if !stdout.is_empty() { stdout.to_string() } else { stderr.to_string() };
    }
    let raw_bin = parts.first().copied().unwrap_or("");
    let bin = normalize_bin_name(raw_bin);
    let sub = extract_subcommand(bin, &parts);

    match bin {
        "git" => filter_git(trimmed, stdout, stderr, exit_code),
        "gh" => filter_gh(trimmed, stdout, stderr, exit_code),
        "glab" => filter_glab(trimmed, stdout, stderr, exit_code),
        "gt" => filter_gt(trimmed, stdout, stderr, exit_code),
        "cargo" if sub == "test" || sub == "nextest" => filter_test(trimmed, stdout, stderr, exit_code),
        "cargo" => filter_build(trimmed, stdout, stderr, exit_code),
        "pytest" => filter_test(trimmed, stdout, stderr, exit_code),
        "vitest" | "jest" => filter_test(trimmed, stdout, stderr, exit_code),
        "playwright" | "ctest" => filter_test(trimmed, stdout, stderr, exit_code),
        "sbt" if sub == "test" || sub.contains("test") => filter_test(trimmed, stdout, stderr, exit_code),
        "sbt" => filter_build(trimmed, stdout, stderr, exit_code),
        "rake" if sub == "test" || trimmed.contains("test") => filter_test(trimmed, stdout, stderr, exit_code),
        "artisan" if sub == "test" => filter_test(trimmed, stdout, stderr, exit_code),
        "tsc" | "mypy" | "ruff" | "make" | "eslint" | "prettier" | "rubocop" | "phpstan" | "pint"
        | "golangci-lint" | "sqlfluff" | "next" | "prisma" | "ng" => {
            filter_build(trimmed, stdout, stderr, exit_code)
        }
        "npm" | "pnpm" | "yarn" | "bun" | "deno" | "npx" | "pnpx" | "bunx" => {
            if sub == "test" {
                filter_test(trimmed, stdout, stderr, exit_code)
            } else {
                filter_build(trimmed, stdout, stderr, exit_code)
            }
        }
        "pip" | "uv" => filter_build(trimmed, stdout, stderr, exit_code),
        "go" if sub == "test" => filter_test(trimmed, stdout, stderr, exit_code),
        "go" => filter_build(trimmed, stdout, stderr, exit_code),
        "dotnet" if sub == "test" => filter_test(trimmed, stdout, stderr, exit_code),
        "dotnet" => filter_build(trimmed, stdout, stderr, exit_code),
        "mvn" | "mvn.cmd" | "gradle" | "gradlew" => filter_build(trimmed, stdout, stderr, exit_code),
        "rspec" | "phpunit" | "pest" | "paratest" => filter_test(trimmed, stdout, stderr, exit_code),
        "docker" | "podman" | "kubectl" | "curl" | "wget" | "ls" | "dir" | "tree" | "find" | "wc" | "env" | "psql"
        | "aws" | "ast-grep" | "sg" | "black" | "json" | "log" => filter_system(trimmed, stdout, stderr, exit_code),
        _ => {
            if let Some(toml_filter) = find_matching_filter(trimmed) {
                let combined = if toml_filter.filter_stderr && !stderr.is_empty() {
                    format!("{}\n{}", stdout, stderr)
                } else if stdout.is_empty() && !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    stdout.to_string()
                };
                apply_filter(toml_filter, &combined)
            } else if !stdout.is_empty() {
                stdout.to_string()
            } else {
                stderr.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_subcommand_cargo_toolchain_and_prefixes() {
        let p1: Vec<&str> = "cargo +nightly test -p toksi".split_whitespace().collect();
        assert_eq!(extract_subcommand("cargo", &p1), "test");

        let p2: Vec<&str> = "cargo --manifest-path foo/Cargo.toml --locked clippy"
            .split_whitespace()
            .collect();
        assert_eq!(extract_subcommand("cargo", &p2), "clippy");

        let p3: Vec<&str> = "cargo -Z unstable-options --color=always check"
            .split_whitespace()
            .collect();
        assert_eq!(extract_subcommand("cargo", &p3), "check");

        let p4: Vec<&str> = "git -C inspect-vault/rtk status -sb".split_whitespace().collect();
        assert_eq!(extract_subcommand("git", &p4), "status");
    }

    #[test]
    fn test_filter_cargo_toolchain_and_git_c() {
        let raw_test = "test a ... ok\ntest b ... ok\ntest result: ok. 2 passed; 0 failed;\n";
        let out = filter_command_output("cargo +nightly -p toksi test", raw_test, "", 0);
        assert_eq!(out, "cargo test: 2 passed (1 suites)");

        let raw_git = "## main...origin/main\nM  foo.rs\n";
        let out_git = filter_command_output("git -C sub/dir status", raw_git, "", 0);
        assert!(out_git.contains("staged (1):"));
    }
}
