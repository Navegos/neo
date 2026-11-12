// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/discover.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::engine::can_filter;
use super::hook::is_already_rtk;
use super::tracker::{resolve_rtk_dir, RtkError};
use crate::util::colors::C;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverReport {
    pub scanned_sessions: usize,
    pub total_commands_scanned: usize,
    pub filterable_uncovered_count: usize,
    pub estimated_missed_tokens: usize,
    pub top_missed_commands: Vec<MissedCommandStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissedCommandStat {
    pub command: String,
    pub count: usize,
    pub estimated_missed_tokens: usize,
}

/// Scans sessions and historical command sources for filterable commands run raw without RTK.
pub fn scan_missed_savings(repo_dir: Option<&Path>, _since_days: u32) -> Result<DiscoverReport, RtkError> {
    let (rtk_dir, _) = resolve_rtk_dir(repo_dir)?;
    let repo_root = rtk_dir.parent().and_then(|p| p.parent()).unwrap_or(&rtk_dir);

    let mut scanned_sessions = 0;
    let mut all_commands = Vec::new();

    // 1. Scan repo-local agent transcripts (.claude, .agents, .gemini)
    let candidate_dirs = [
        repo_root.join(".claude"),
        repo_root.join(".agents"),
        repo_root.join(".gemini"),
    ];

    for cdir in &candidate_dirs {
        if let Ok(entries) = fs::read_dir(cdir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.ends_with(".json") || name.ends_with(".jsonl") {
                        scanned_sessions += 1;
                        extract_commands_from_file(&path, &mut all_commands);
                    }
                } else if path.is_dir()
                    && let Ok(sub_entries) = fs::read_dir(&path)
                {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if sub_path.is_file() {
                            let sub_name = sub_path.file_name().unwrap_or_default().to_string_lossy();
                            if sub_name.ends_with(".json") || sub_name.ends_with(".jsonl") {
                                scanned_sessions += 1;
                                extract_commands_from_file(&sub_path, &mut all_commands);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan home directory user transcripts if available (~/.claude/projects)
    if let Some(home) = dirs::home_dir() {
        let claude_home = home.join(".claude").join("projects");
        if let Ok(entries) = fs::read_dir(&claude_home) {
            for entry in entries.flatten().take(10) {
                if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                    for sub in sub_entries.flatten() {
                        let sp = sub.path();
                        if sp.is_file() && sp.to_string_lossy().ends_with(".jsonl") {
                            scanned_sessions += 1;
                            extract_commands_from_file(&sp, &mut all_commands);
                        }
                    }
                }
            }
        }
    }

    // 3. Scan tracking database history for comparison
    let db_path = rtk_dir.join("tracking.db");
    if let Ok(conn) = rusqlite::Connection::open(&db_path)
        && let Ok(mut stmt) = conn.prepare("SELECT original_cmd FROM command_history ORDER BY id DESC LIMIT 500")
        && let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0))
    {
        for r in rows.flatten() {
            all_commands.push(r);
        }
    }

    let total_commands_scanned = all_commands.len();
    let mut missed_counts: HashMap<String, usize> = HashMap::new();

    for cmd in &all_commands {
        let trimmed = cmd.trim();
        if trimmed.is_empty() || is_already_rtk(trimmed) {
            continue;
        }

        if can_filter(trimmed) {
            let base_key = normalize_command_key(trimmed);
            *missed_counts.entry(base_key).or_insert(0) += 1;
        }
    }

    let mut filterable_uncovered_count = 0;
    let mut estimated_missed_tokens = 0;
    let mut stats_vec = Vec::new();

    for (cmd, count) in missed_counts {
        filterable_uncovered_count += count;
        let avg_savings_per_cmd = average_savings_for_command(&cmd);
        let missed_tok = count * avg_savings_per_cmd;
        estimated_missed_tokens += missed_tok;

        stats_vec.push(MissedCommandStat {
            command: cmd,
            count,
            estimated_missed_tokens: missed_tok,
        });
    }

    stats_vec.sort_by_key(|a| std::cmp::Reverse(a.estimated_missed_tokens));

    Ok(DiscoverReport {
        scanned_sessions,
        total_commands_scanned,
        filterable_uncovered_count,
        estimated_missed_tokens,
        top_missed_commands: stats_vec,
    })
}

fn extract_commands_from_file(path: &Path, out: &mut Vec<String>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    if path.to_string_lossy().ends_with(".jsonl") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                extract_commands_from_json(&val, out);
            }
        }
    } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
        extract_commands_from_json(&val, out);
    }
}

fn extract_commands_from_json(val: &serde_json::Value, out: &mut Vec<String>) {
    match val {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(cmd)) = map.get("command") {
                out.push(cmd.clone());
            }
            if let Some(serde_json::Value::String(cmd)) = map.get("CommandLine") {
                out.push(cmd.clone());
            }
            for v in map.values() {
                extract_commands_from_json(v, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_commands_from_json(v, out);
            }
        }
        _ => {}
    }
}

fn normalize_command_key(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return cmd.to_string();
    }
    let bin = parts[0];
    if parts.len() > 1 && !parts[1].starts_with('-') {
        format!("{} {}", bin, parts[1])
    } else {
        bin.to_string()
    }
}

fn average_savings_for_command(cmd: &str) -> usize {
    if cmd.starts_with("cargo test") || cmd.starts_with("pytest") || cmd.starts_with("vitest") {
        650
    } else if cmd.starts_with("cargo build") || cmd.starts_with("tsc") || cmd.starts_with("go build") {
        350
    } else if cmd.starts_with("git status") || cmd.starts_with("git diff") || cmd.starts_with("git log") {
        400
    } else if cmd.starts_with("docker") || cmd.starts_with("kubectl") {
        300
    } else if cmd.starts_with("npm") || cmd.starts_with("pnpm") || cmd.starts_with("yarn") {
        450
    } else {
        250
    }
}

pub fn run_discover(args: &[String]) -> i32 {
    let mut json_format = false;
    let mut since_days = 30;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" if i + 1 < args.len() && args[i + 1] == "json" => {
                json_format = true;
                i += 1;
            }
            "--json" => json_format = true,
            "--since" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<u32>() {
                    since_days = d;
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    match scan_missed_savings(None, since_days) {
        Ok(report) => {
            if json_format {
                match serde_json::to_string_pretty(&report) {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        eprintln!("toksi rtk discover: json error: {}", e);
                        return 1;
                    }
                }
            } else {
                println!("{}", C.bold("RTK Missed-Savings Scanner (toksi rtk discover):"));
                println!("  Sessions scanned:             {}", report.scanned_sessions);
                println!("  Total commands checked:       {}", report.total_commands_scanned);
                println!(
                    "  Filterable raw commands:      {}",
                    C.cyan(&report.filterable_uncovered_count.to_string())
                );
                println!(
                    "  Estimated missed savings:     {} tokens",
                    C.green(&report.estimated_missed_tokens.to_string())
                );
                println!();

                if !report.top_missed_commands.is_empty() {
                    println!("{}", C.bold("Top Commands That Could Benefit From RTK:"));
                    println!("{:<25} {:>10} {:>18}", "Command", "Runs", "Est. Saved Tokens");
                    println!("{:-<58}", "");
                    for item in report.top_missed_commands.iter().take(15) {
                        println!(
                            "{:<25} {:>10} {:>18}",
                            item.command, item.count, item.estimated_missed_tokens
                        );
                    }
                    println!();
                    println!(
                        "{} Run '{}' to auto-rewrite commands in tool calls.",
                        C.cyan("Tip:"),
                        C.bold("toksi rtk-hook <agent>")
                    );
                } else {
                    println!("{} No unintercepted commands detected. RTK is actively filtering!", C.green("✓"));
                }
            }
            0
        }
        Err(e) => {
            eprintln!("toksi rtk discover: failed to scan sessions: {}", e);
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_scan_missed_savings_empty() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempdir()?;
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(&git_dir)?;

        let report = scan_missed_savings(Some(tmp.path()), 30)?;
        assert_eq!(report.filterable_uncovered_count, 0);
        assert_eq!(report.estimated_missed_tokens, 0);
        Ok(())
    }
}
