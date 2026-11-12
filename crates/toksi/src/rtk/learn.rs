// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/learn.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::tracker::{resolve_rtk_dir, RtkError};
use crate::util::colors::C;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRule {
    pub wrong_pattern: String,
    pub right_pattern: String,
    pub occurrences: usize,
    pub confidence: f64,
    pub advice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnReport {
    pub total_analyzed: usize,
    pub corrections_detected: usize,
    pub rules: Vec<CorrectionRule>,
}

#[derive(Debug, Clone)]
struct ExecRecord {
    pub command: String,
    pub exit_code: i32,
}

pub fn detect_corrections(repo_dir: Option<&Path>, _since_days: u32) -> Result<LearnReport, RtkError> {
    let (rtk_dir, _) = resolve_rtk_dir(repo_dir)?;
    let db_path = rtk_dir.join("tracking.db");

    let mut records: Vec<ExecRecord> = Vec::new();

    if let Ok(conn) = rusqlite::Connection::open(&db_path)
        && let Ok(mut stmt) = conn.prepare("SELECT original_cmd, exit_code FROM command_history ORDER BY id ASC LIMIT 1000")
        && let Ok(rows) = stmt.query_map([], |r| {
            Ok(ExecRecord {
                command: r.get(0)?,
                exit_code: r.get(1)?,
            })
        })
    {
        for r in rows.flatten() {
            records.push(r);
        }
    }

    let total_analyzed = records.len();
    let mut pairs_count: HashMap<(String, String), usize> = HashMap::new();

    for i in 0..records.len().saturating_sub(1) {
        let first = &records[i];
        let second = &records[i + 1];

        if first.exit_code != 0 && second.exit_code == 0 {
            let tool1 = first.command.split_whitespace().next().unwrap_or("");
            let tool2 = second.command.split_whitespace().next().unwrap_or("");

            if tool1 == tool2 && !tool1.is_empty() && first.command != second.command {
                let pair = (first.command.clone(), second.command.clone());
                *pairs_count.entry(pair).or_insert(0) += 1;
            }
        }
    }

    let mut rules = Vec::new();
    for ((wrong, right), count) in pairs_count {
        let advice = generate_advice(&wrong, &right);
        let confidence = (count as f64 / (count + 1) as f64).min(0.99);

        rules.push(CorrectionRule {
            wrong_pattern: wrong,
            right_pattern: right,
            occurrences: count,
            confidence,
            advice,
        });
    }

    rules.sort_by_key(|a| std::cmp::Reverse(a.occurrences));
    let corrections_detected = rules.len();

    Ok(LearnReport {
        total_analyzed,
        corrections_detected,
        rules,
    })
}

fn generate_advice(wrong: &str, right: &str) -> String {
    if wrong.starts_with("git") && right.starts_with("git") {
        if wrong.contains("checkout") && right.contains("switch") {
            "Use `git switch` instead of deprecated `git checkout` for branching.".to_string()
        } else {
            format!("Prefer `{}` over `{}`.", right, wrong)
        }
    } else if wrong.starts_with("cargo") && right.starts_with("cargo") {
        if wrong.contains("test") && right.contains("test --") {
            "Pass test filters after `--` separator in `cargo test`.".to_string()
        } else {
            format!("Use `{}` to resolve build/test parameters.", right)
        }
    } else if wrong.starts_with("npm") && right.starts_with("pnpm") {
        "Workspace uses pnpm; avoid invoking npm directly.".to_string()
    } else {
        format!("Corrected command: `{}`.", right)
    }
}

pub fn run_learn(args: &[String]) -> i32 {
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

    match detect_corrections(None, since_days) {
        Ok(report) => {
            if json_format {
                match serde_json::to_string_pretty(&report) {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        eprintln!("toksi rtk learn: json error: {}", e);
                        return 1;
                    }
                }
            } else {
                println!("{}", C.bold("RTK CLI Correction Detector (toksi rtk learn):"));
                println!("  Historical executions checked: {}", report.total_analyzed);
                println!(
                    "  Corrections detected:          {}",
                    C.cyan(&report.corrections_detected.to_string())
                );
                println!();

                if !report.rules.is_empty() {
                    println!("{}", C.bold("Detected Command Patterns & Recommendations:"));
                    for (idx, r) in report.rules.iter().take(10).enumerate() {
                        println!(
                            "  {}. {} -> {}",
                            idx + 1,
                            C.red(&r.wrong_pattern),
                            C.green(&r.right_pattern)
                        );
                        println!("     Times observed: {} | Confidence: {:.0}%", r.occurrences, r.confidence * 100.0);
                        println!("     Advice: {}", r.advice);
                        println!();
                    }
                } else {
                    println!("{} No repeated command mistakes or corrections detected.", C.green("✓"));
                }
            }
            0
        }
        Err(e) => {
            eprintln!("toksi rtk learn: failed: {}", e);
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_corrections_empty() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempdir()?;
        let git_dir = tmp.path().join(".git");
        std::fs::create_dir_all(&git_dir)?;

        let report = detect_corrections(Some(tmp.path()), 30)?;
        assert_eq!(report.corrections_detected, 0);
        Ok(())
    }
}
