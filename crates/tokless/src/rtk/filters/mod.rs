// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/filters/mod.rs
// created: 2026-10-27
// lastModified: 2026-10-27

pub mod build;
pub mod git;
pub mod system;
pub mod test;
pub mod toml;

use self::build::filter_build;
use self::git::{filter_gh, filter_git, filter_glab};
use self::system::filter_system;
use self::test::filter_test;
use self::toml::{apply_filter, find_matching_filter};

/// Check if a command can be filtered by either native filters or declarative TOML filters.
pub fn is_filterable_command(cmd_line: &str) -> bool {
    let trimmed = cmd_line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");
    let sub = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match bin {
        "git" | "gh" | "glab" => true,
        "cargo" => true,
        "pytest" => true,
        "vitest" | "jest" => true,
        "tsc" | "mypy" | "ruff" | "make" => true,
        "npm" | "pnpm" | "yarn" | "bun" | "deno" | "npx" | "pnpx" | "bunx" => true,
        "pip" | "uv" => true,
        "eslint" | "prettier" => true,
        "playwright" | "ctest" => true,
        "go" if sub == "test" || sub == "build" || sub == "vet" => true,
        "dotnet" if sub == "test" || sub == "build" => true,
        "mvn" | "mvn.cmd" | "gradle" | "gradlew" => true,
        "rspec" | "phpunit" | "pest" => true,
        "docker" | "podman" | "kubectl" | "curl" | "wget" | "ls" | "tree" | "find" | "wc" | "env" | "psql"
        | "aws" => true,
        _ => find_matching_filter(trimmed).is_some(),
    }
}

/// Filter command output based on the command line string.
pub fn filter_command_output(cmd_line: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let trimmed = cmd_line.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");
    let sub = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match bin {
        "git" => filter_git(trimmed, stdout, stderr, exit_code),
        "gh" => filter_gh(trimmed, stdout, stderr, exit_code),
        "glab" => filter_glab(trimmed, stdout, stderr, exit_code),
        "cargo" if sub == "test" || sub == "nextest" => filter_test(trimmed, stdout, stderr, exit_code),
        "cargo" => filter_build(trimmed, stdout, stderr, exit_code),
        "pytest" => filter_test(trimmed, stdout, stderr, exit_code),
        "vitest" | "jest" => filter_test(trimmed, stdout, stderr, exit_code),
        "playwright" | "ctest" => filter_test(trimmed, stdout, stderr, exit_code),
        "tsc" | "mypy" | "ruff" | "make" | "eslint" | "prettier" => {
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
        "rspec" | "phpunit" | "pest" => filter_test(trimmed, stdout, stderr, exit_code),
        "docker" | "podman" | "kubectl" | "curl" | "wget" | "ls" | "tree" | "find" | "wc" | "env" | "psql"
        | "aws" => filter_system(trimmed, stdout, stderr, exit_code),
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
