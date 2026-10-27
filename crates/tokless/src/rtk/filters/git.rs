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
}

