// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/filters/git.rs
// created: 2026-10-27
// lastModified: 2026-10-27

use std::collections::{BTreeMap, BTreeSet};
use crate::rtk::ansi::strip_ansi;

/// Filter git command output.
pub fn filter_git(cmd: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let raw = if !stderr.is_empty() && stdout.is_empty() {
        stderr
    } else {
        stdout
    };
    let clean = strip_ansi(raw);

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let subcommand = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match subcommand {
        "status" => filter_git_status(&clean),
        "diff" => filter_git_diff(&clean),
        "log" => filter_git_log(&clean),
        "branch" => filter_git_branch(&clean),
        "add" | "commit" | "push" | "pull" | "fetch" | "checkout" | "stash" => {
            filter_git_ops(&clean, subcommand, exit_code)
        }
        _ => clean,
    }
}

/// Filter `git status` output into a compact matrix.
pub fn filter_git_status(raw: &str) -> String {
    let mut branch = String::new();
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicts = Vec::new();

    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with("##") {
            // Branch header
            let b = trimmed.trim_start_matches("##").trim();
            branch = b.to_string();
            continue;
        }

        if trimmed.len() >= 3 {
            let xy = &trimmed[0..2];
            let file = trimmed[3..].trim();

            if xy == "??" {
                untracked.push(file.to_string());
                continue;
            }
            if xy == "UU" || xy == "AA" || xy == "DD" || xy.contains('U') {
                conflicts.push(format!("conflict: {}", file));
                continue;
            }
            let x = xy.chars().next().unwrap_or(' ');
            let y = xy.chars().nth(1).unwrap_or(' ');

            if x != ' ' && x != '?' {
                staged.push(format!("{} {}", x, file));
            }
            if y != ' ' && y != '?' {
                unstaged.push(format!("{} {}", y, file));
            }
        } else if trimmed.starts_with("On branch ") {
            branch = trimmed.trim_start_matches("On branch ").to_string();
        }
    }

    // Collapse untracked files by directory if many
    let mut compact_untracked = Vec::new();
    let mut dir_counts: BTreeMap<String, usize> = BTreeMap::new();
    for f in &untracked {
        if let Some((dir, _)) = f.rsplit_once('/') {
            *dir_counts.entry(dir.to_string()).or_default() += 1;
        } else if let Some((dir, _)) = f.rsplit_once('\\') {
            *dir_counts.entry(dir.to_string()).or_default() += 1;
        }
    }

    let mut handled_dirs = BTreeSet::new();
    for (dir, count) in &dir_counts {
        if *count >= 4 {
            compact_untracked.push(format!("{}/ ({} untracked files)", dir, count));
            handled_dirs.insert(dir.clone());
        }
    }
    for f in untracked {
        let is_in_handled = handled_dirs.iter().any(|d| f.starts_with(d));
        if !is_in_handled {
            compact_untracked.push(f);
        }
    }

    if staged.is_empty() && unstaged.is_empty() && compact_untracked.is_empty() && conflicts.is_empty() {
        if !branch.is_empty() {
            return format!("branch: {} (clean)", branch);
        }
        return "clean".to_string();
    }

    let mut out = Vec::new();
    if !branch.is_empty() {
        out.push(format!("branch: {}", branch));
    }
    if !conflicts.is_empty() {
        out.push(format!("CONFLICTS ({}):\n  {}", conflicts.len(), conflicts.join("\n  ")));
    }
    if !staged.is_empty() {
        out.push(format!("staged ({}):\n  {}", staged.len(), staged.join("\n  ")));
    }
    if !unstaged.is_empty() {
        out.push(format!("unstaged ({}):\n  {}", unstaged.len(), unstaged.join("\n  ")));
    }
    if !compact_untracked.is_empty() {
        out.push(format!("untracked ({}):\n  {}", compact_untracked.len(), compact_untracked.join("\n  ")));
    }

    out.join("\n")
}

/// Filter `git diff` output, compressing lockfiles and redundant hunks.
pub fn filter_git_diff(raw: &str) -> String {
    let mut out = Vec::new();
    let mut in_lockfile = false;
    let mut lockfile_name = String::new();
    let mut lockfile_add = 0;
    let mut lockfile_del = 0;

    for line in raw.lines() {
        if line.starts_with("diff --git ") {
            if in_lockfile {
                out.push(format!("[lockfile updated: {} (+{} -{})]", lockfile_name, lockfile_add, lockfile_del));
                in_lockfile = false;
                lockfile_add = 0;
                lockfile_del = 0;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let target = parts.get(2).and_then(|p| p.strip_prefix("a/")).unwrap_or("");
            if is_lockfile(target) {
                in_lockfile = true;
                lockfile_name = target.to_string();
                continue;
            }
            out.push(line.to_string());
            continue;
        }

        if in_lockfile {
            if line.starts_with('+') && !line.starts_with("+++") {
                lockfile_add += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lockfile_del += 1;
            }
            continue;
        }

        // Drop index, binary, and filemode noise
        if line.starts_with("index ") || line.starts_with("new file mode ") || line.starts_with("deleted file mode ") {
            continue;
        }

        out.push(line.to_string());
    }

    if in_lockfile {
        out.push(format!("[lockfile updated: {} (+{} -{})]", lockfile_name, lockfile_add, lockfile_del));
    }

    out.join("\n")
}

fn is_lockfile(path: &str) -> bool {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    matches!(
        name,
        "Cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "composer.lock"
            | "Pipfile.lock"
            | "poetry.lock"
            | "flake.lock"
    )
}

/// Filter `git log` output into single-line entries.
pub fn filter_git_log(raw: &str) -> String {
    let mut out = Vec::new();
    let mut current_hash = String::new();
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_msg = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("commit ") && trimmed.len() >= 14 {
            if !current_hash.is_empty() {
                out.push(format!("{} - {} ({}) <{}>", current_hash, current_msg, current_date, current_author));
                current_author.clear();
                current_date.clear();
                current_msg.clear();
            }
            let hash = trimmed.trim_start_matches("commit ").trim();
            current_hash = hash.chars().take(8).collect();
            continue;
        }

        if trimmed.starts_with("Author: ") {
            current_author = trimmed.trim_start_matches("Author: ").trim().to_string();
        } else if trimmed.starts_with("Date: ") {
            current_date = trimmed.trim_start_matches("Date: ").trim().to_string();
        } else if !trimmed.is_empty() && current_msg.is_empty() && !current_hash.is_empty() {
            current_msg = trimmed.to_string();
        }
    }

    if !current_hash.is_empty() {
        out.push(format!("{} - {} ({}) <{}>", current_hash, current_msg, current_date, current_author));
    }

    if out.is_empty() {
        raw.lines().take(50).collect::<Vec<&str>>().join("\n")
    } else {
        out.join("\n")
    }
}

/// Filter `git branch` output.
pub fn filter_git_branch(raw: &str) -> String {
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    lines.join("\n")
}

/// Filter operational git commands (add, commit, push, pull, checkout).
pub fn filter_git_ops(raw: &str, op: &str, exit_code: i32) -> String {
    let filtered_lines: Vec<&str> = raw
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("Writing objects:")
                && !t.starts_with("Counting objects:")
                && !t.starts_with("Compressing objects:")
                && !t.starts_with("Resolving deltas:")
                && !t.starts_with("Delta compression")
        })
        .collect();

    let text = filtered_lines.join("\n");
    if text.trim().is_empty() {
        if exit_code == 0 {
            format!("git {}: ok", op)
        } else {
            format!("git {}: failed (exit code {})", op, exit_code)
        }
    } else {
        text
    }
}

/// Filter markdown body removing badges, HTML comments, and noisy formatting.
pub fn filter_markdown_body(body: &str) -> String {
    if body.is_empty() {
        return String::new();
    }

    let mut out = Vec::new();
    let mut in_code_block = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            out.push(line);
            continue;
        }

        if in_code_block {
            out.push(line);
            continue;
        }

        // HTML comments
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            continue;
        }
        // Badge lines: [![...](...)](...)
        if trimmed.starts_with("[![") && trimmed.ends_with(')') {
            continue;
        }
        // Image-only lines: ![...](...)
        if trimmed.starts_with("![") && trimmed.ends_with(')') {
            continue;
        }
        // Horizontal rules: ---, ***, ___
        if trimmed == "---" || trimmed == "***" || trimmed == "___" || trimmed == "------" {
            continue;
        }

        out.push(line);
    }

    let joined = out.join("\n");
    // Collapse 3+ newlines into 2
    let mut collapsed = String::with_capacity(joined.len());
    let mut newline_count = 0;
    for c in joined.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                collapsed.push(c);
            }
        } else {
            newline_count = 0;
            collapsed.push(c);
        }
    }

    collapsed.trim().to_string()
}

/// Filter GitHub CLI (`gh`) PR checks output.
pub fn filter_gh_pr_checks(stdout: &str) -> String {
    let mut passed = 0;
    let mut failed = 0;
    let mut pending = 0;
    let mut failed_checks = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        if lower.contains("pass") || lower.contains("✓") || lower.contains("[ok]") {
            passed += 1;
        } else if lower.contains("fail") || lower.contains("✕") || lower.contains("error") {
            failed += 1;
            failed_checks.push(trimmed.to_string());
        } else if lower.contains("pending") || lower.contains("in_progress") || lower.contains("queued") {
            pending += 1;
        }
    }

    let mut out = Vec::new();
    out.push(format!("CI Checks: {} passed, {} failed, {} pending", passed, failed, pending));
    if !failed_checks.is_empty() {
        out.push("Failed checks:".to_string());
        for check in failed_checks {
            out.push(format!("  {}", check));
        }
    }

    out.join("\n")
}

/// Filter tabular listings from `gh` or `glab` (PRs, issues, pipelines).
pub fn filter_gh_tabular_list(stdout: &str) -> String {
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return "No items found".to_string();
    }

    if lines.len() <= 25 {
        return lines.join("\n");
    }

    let mut out: Vec<String> = lines.iter().take(25).map(|s| (*s).to_string()).collect();
    out.push(format!("... [{} more items omitted by rtk]", lines.len() - 25));
    out.join("\n")
}

/// Filter `gh run list` and `gh run view`.
pub fn filter_gh_run_list(stdout: &str) -> String {
    filter_gh_tabular_list(stdout)
}

/// Filter `gh run view` capturing failed step annotations and error output.
pub fn filter_gh_run_view(stdout: &str) -> String {
    let mut out = Vec::new();
    let mut in_failure = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("✓ ") {
            continue;
        }
        if trimmed.starts_with("X ") || trimmed.starts_with("✕ ") || trimmed.contains("FAILED") {
            in_failure = true;
            out.push(trimmed.to_string());
            continue;
        }
        if in_failure {
            out.push(trimmed.to_string());
            if trimmed.is_empty() {
                in_failure = false;
            }
        }
    }

    if out.is_empty() {
        filter_markdown_body(stdout)
    } else {
        out.join("\n")
    }
}

/// Filter GitHub CLI (`gh`) command output.
pub fn filter_gh(cmd: &str, stdout: &str, stderr: &str, _exit_code: i32) -> String {
    let raw = if !stderr.is_empty() && stdout.is_empty() {
        stderr
    } else {
        stdout
    };
    let clean = strip_ansi(raw);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let subcommand = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");
    let action = parts.iter().skip(2).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match (subcommand, action) {
        ("pr", "checks") => filter_gh_pr_checks(&clean),
        ("pr", "list") | ("issue", "list") => filter_gh_tabular_list(&clean),
        ("pr", "view") | ("issue", "view") => filter_markdown_body(&clean),
        ("run", "list") => filter_gh_run_list(&clean),
        ("run", "view") => filter_gh_run_view(&clean),
        _ => {
            if clean.starts_with('{') || clean.starts_with('[') {
                clean
            } else {
                filter_markdown_body(&clean)
            }
        }
    }
}

/// Filter GitLab CLI (`glab`) command output.
pub fn filter_glab(cmd: &str, stdout: &str, stderr: &str, _exit_code: i32) -> String {
    let raw = if !stderr.is_empty() && stdout.is_empty() {
        stderr
    } else {
        stdout
    };
    let clean = strip_ansi(raw);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let subcommand = parts.iter().skip(1).find(|&&p| !p.starts_with('-')).copied().unwrap_or("");

    match subcommand {
        "mr" | "issue" | "pipeline" => filter_gh_tabular_list(&clean),
        _ => filter_markdown_body(&clean),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_git_status() {
        let raw = "## main...origin/main [ahead 1]\nM  src/main.rs\n M src/lib.rs\n?? new_file.txt\n";
        let out = filter_git_status(raw);
        assert!(out.contains("branch: main...origin/main [ahead 1]"));
        assert!(out.contains("staged (1):"));
        assert!(out.contains("unstaged (1):"));
        assert!(out.contains("untracked (1):"));
    }

    #[test]
    fn test_filter_git_diff_lockfile() {
        let raw = "diff --git a/Cargo.lock b/Cargo.lock\nindex 1234..5678 100644\n--- a/Cargo.lock\n+++ b/Cargo.lock\n@@ -1,3 +1,5 @@\n+line1\n+line2\n-line3\n";
        let out = filter_git_diff(raw);
        assert!(out.contains("[lockfile updated: Cargo.lock (+2 -1)]"));
    }

    #[test]
    fn test_filter_markdown_body() {
        let raw = "<!-- comment -->\n[![build](https://badge.svg)](link)\n# Title\n---\nBody text\n![img](image.png)\n";
        let out = filter_markdown_body(raw);
        assert_eq!(out, "# Title\nBody text");
    }

    #[test]
    fn test_filter_gh_pr_checks() {
        let raw = "build\tpass\thttps://ci/1\ntest\tfail\thttps://ci/2\nlint\tpending\thttps://ci/3\n";
        let out = filter_gh_pr_checks(raw);
        assert!(out.contains("CI Checks: 1 passed, 1 failed, 1 pending"));
        assert!(out.contains("test\tfail\thttps://ci/2"));
    }
}

