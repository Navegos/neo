// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/engine.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::process::{Command, Stdio};
use crate::rtk::ansi::strip_ansi;
use crate::rtk::filters::{filter_command_output, is_filterable_command, toml::list_filter_names};

/// Parse a shell command line string into arguments respecting quotes.
pub fn tokenize_command(cmd_line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = cmd_line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\\' if !in_single_quote => {
                if let Some(next_c) = chars.next() {
                    current.push(next_c);
                }
            }
            c if c.is_whitespace() && !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Check if a command is filterable by the embedded RTK engine.
pub fn can_filter(cmd_line: &str) -> bool {
    is_filterable_command(cmd_line)
}

/// Filter raw command output, guaranteeing output is never worse than raw.
pub fn filter_output(cmd_line: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let clean_stdout = strip_ansi(stdout);
    let clean_stderr = strip_ansi(stderr);

    let raw_total_len = clean_stdout.len() + clean_stderr.len();
    let filtered = filter_command_output(cmd_line, &clean_stdout, &clean_stderr, exit_code);

    // Guard: if raw had content but filtered output is completely empty, keep raw
    if filtered.trim().is_empty() && raw_total_len > 0 {
        if !clean_stdout.is_empty() {
            return clean_stdout;
        } else {
            return clean_stderr;
        }
    }

    filtered
}

/// Execute a command and print filtered output to stdout, returning the exit code.
pub fn run_command_filtered(cmd_line: &str) -> std::io::Result<i32> {
    let tokens = tokenize_command(cmd_line);
    if tokens.is_empty() {
        return Ok(0);
    }

    let program = &tokens[0];
    let args = &tokens[1..];

    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let filtered = filter_output(cmd_line, &stdout, &stderr, exit_code);
    println!("{}", filtered);

    Ok(exit_code)
}

/// List all supported command filters (native commands + 60+ TOML filters).
pub fn list_supported_commands() -> Vec<&'static str> {
    let mut cmds = vec![
        "git", "git status", "git diff", "git log", "git branch", "git add", "git commit", "git push", "git pull",
        "cargo", "cargo build", "cargo check", "cargo clippy", "cargo test", "cargo nextest",
        "tsc", "pytest", "vitest", "jest", "go", "go build", "go test", "go vet",
        "dotnet", "dotnet build", "dotnet test",
        "mvn", "gradle", "gradlew", "mypy", "ruff", "make",
        "docker", "docker ps", "docker compose", "podman", "kubectl", "curl", "wget", "ls", "tree", "cat",
    ];

    cmds.extend(list_filter_names());
    cmds.sort();
    cmds.dedup();
    cmds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_command() {
        let cmd = "git commit -m \"initial commit\" --author 'Dev <dev@test.com>' file.rs";
        let tokens = tokenize_command(cmd);
        assert_eq!(
            tokens,
            vec!["git", "commit", "-m", "initial commit", "--author", "Dev <dev@test.com>", "file.rs"]
        );
    }

    #[test]
    fn test_supported_commands_count() {
        let list = list_supported_commands();
        assert!(list.len() >= 90);
    }
}

