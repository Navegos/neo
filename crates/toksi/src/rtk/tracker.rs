// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/tracker.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::util::colors::C;
use crate::util::paths::normalize_path;

#[derive(Debug, thiserror::Error)]
pub enum RtkError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Message(String),
}

/// Locate repo root (<repo>) by walking up from start dir to find `.git`.
/// Project-level RTK artifacts live strictly inside `<repo>/.toksi/rtk/`.
/// Never in user home project dirs or `<repo>/.git/`.
pub fn resolve_rtk_dir(start: Option<&Path>) -> Result<(PathBuf, String), RtkError> {
    let cwd = match start {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(RtkError::Io)?,
    };

    let mut current = cwd.clone();
    let mut repo_root: Option<PathBuf> = None;

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            repo_root = Some(current.clone());
            break;
        }
        if !current.pop() {
            break;
        }
    }

    let root = match repo_root {
        Some(r) => r,
        None => {
            // Check for project markers as secondary fallback
            let mut fallback = cwd.clone();
            let mut project_root: Option<PathBuf> = None;
            loop {
                if fallback.join("Cargo.toml").exists()
                    || fallback.join("package.json").exists()
                    || fallback.join(".toksi").exists()
                {
                    project_root = Some(fallback.clone());
                    break;
                }
                if !fallback.pop() {
                    break;
                }
            }
            project_root.unwrap_or(cwd.clone())
        }
    };

    let subproject_id = if cwd.starts_with(&root) && cwd != root {
        match cwd.strip_prefix(&root) {
            Ok(rel) => normalize_path(&rel.to_string_lossy()),
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    let rtk_dir = root.join(".toksi").join("rtk");
    if !rtk_dir.exists() {
        fs::create_dir_all(&rtk_dir).map_err(RtkError::Io)?;
    }

    Ok((rtk_dir, subproject_id))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub id: i64,
    pub timestamp: String,
    pub original_cmd: String,
    pub rtk_cmd: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub saved_tokens: usize,
    pub savings_pct: f64,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub subproject_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GainSummary {
    pub total_commands: usize,
    pub total_input: usize,
    pub total_output: usize,
    pub total_saved: usize,
    pub avg_savings_pct: f64,
    pub total_time_ms: u64,
    pub avg_time_ms: u64,
    pub by_command: Vec<CommandStat>,
    pub by_day: Vec<PeriodStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStat {
    pub command: String,
    pub count: usize,
    pub saved_tokens: usize,
    pub savings_pct: f64,
    pub avg_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodStat {
    pub period: String,
    pub count: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub saved_tokens: usize,
    pub savings_pct: f64,
    pub avg_time_ms: u64,
}

pub struct Tracker {
    conn: Connection,
    rtk_dir: PathBuf,
    subproject_id: String,
}

impl Tracker {
    pub fn new() -> Result<Self, RtkError> {
        Self::for_dir(None)
    }

    pub fn for_dir(dir: Option<&Path>) -> Result<Self, RtkError> {
        let (rtk_dir, subproject_id) = resolve_rtk_dir(dir)?;
        let db_path = rtk_dir.join("tracking.db");

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS command_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                original_cmd TEXT NOT NULL,
                rtk_cmd TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                saved_tokens INTEGER NOT NULL,
                savings_pct REAL NOT NULL,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                exit_code INTEGER NOT NULL DEFAULT 0,
                subproject_id TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON command_history(timestamp);
            CREATE INDEX IF NOT EXISTS idx_history_subproject ON command_history(subproject_id);",
            [],
        )?;

        Ok(Self {
            conn,
            rtk_dir,
            subproject_id,
        })
    }

    pub fn rtk_dir(&self) -> &Path {
        &self.rtk_dir
    }

    pub fn subproject_id(&self) -> &str {
        &self.subproject_id
    }

    pub fn record(
        &self,
        original_cmd: &str,
        rtk_cmd: &str,
        input_tokens: usize,
        output_tokens: usize,
        duration_ms: u64,
        exit_code: i32,
    ) -> Result<(), RtkError> {
        let saved_tokens = input_tokens.saturating_sub(output_tokens);
        let savings_pct = if input_tokens > 0 {
            (saved_tokens as f64 / input_tokens as f64) * 100.0
        } else {
            0.0
        };
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO command_history (
                timestamp, original_cmd, rtk_cmd, input_tokens, output_tokens,
                saved_tokens, savings_pct, duration_ms, exit_code, subproject_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                now,
                original_cmd,
                rtk_cmd,
                input_tokens as i64,
                output_tokens as i64,
                saved_tokens as i64,
                savings_pct,
                duration_ms as i64,
                exit_code,
                self.subproject_id,
            ],
        )?;

        let _ = self.cleanup_old(90);
        Ok(())
    }

    pub fn cleanup_old(&self, days: i64) -> Result<usize, RtkError> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();
        let deleted = self.conn.execute(
            "DELETE FROM command_history WHERE timestamp < ?1",
            params![cutoff_str],
        )?;
        Ok(deleted)
    }

    pub fn reset(&self) -> Result<(), RtkError> {
        self.conn.execute("DELETE FROM command_history", [])?;
        Ok(())
    }

    pub fn get_summary(&self, days: Option<u32>) -> Result<GainSummary, RtkError> {
        let where_clause = match days {
            Some(d) => {
                let cutoff = (Utc::now() - chrono::Duration::days(d as i64)).to_rfc3339();
                format!("WHERE timestamp >= '{}'", cutoff)
            }
            None => String::new(),
        };

        let sql = format!(
            "SELECT 
                COUNT(*),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(saved_tokens), 0),
                COALESCE(SUM(duration_ms), 0)
             FROM command_history {}",
            where_clause
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let row = stmt.query_row([], |r| {
            let count: i64 = r.get(0)?;
            let in_tok: i64 = r.get(1)?;
            let out_tok: i64 = r.get(2)?;
            let saved_tok: i64 = r.get(3)?;
            let total_ms: i64 = r.get(4)?;
            Ok((count, in_tok, out_tok, saved_tok, total_ms))
        })?;

        let total_commands = row.0.max(0) as usize;
        let total_input = row.1.max(0) as usize;
        let total_output = row.2.max(0) as usize;
        let total_saved = row.3.max(0) as usize;
        let total_time_ms = row.4.max(0) as u64;

        let avg_savings_pct = if total_input > 0 {
            (total_saved as f64 / total_input as f64) * 100.0
        } else {
            0.0
        };

        let avg_time_ms = if total_commands > 0 {
            total_time_ms / total_commands as u64
        } else {
            0
        };

        // Top commands by tokens saved
        let cmd_sql = format!(
            "SELECT 
                rtk_cmd,
                COUNT(*),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(saved_tokens), 0),
                COALESCE(SUM(duration_ms), 0)
             FROM command_history {}
             GROUP BY rtk_cmd
             ORDER BY SUM(saved_tokens) DESC
             LIMIT 10",
            where_clause
        );

        let mut cmd_stmt = self.conn.prepare(&cmd_sql)?;
        let cmd_iter = cmd_stmt.query_map([], |r| {
            let cmd: String = r.get(0)?;
            let count: i64 = r.get(1)?;
            let in_tok: i64 = r.get(2)?;
            let saved: i64 = r.get(3)?;
            let time_ms: i64 = r.get(4)?;
            Ok((cmd, count, in_tok, saved, time_ms))
        })?;

        let mut by_command = Vec::new();
        for res in cmd_iter {
            let (cmd, count, in_tok, saved, time_ms) = res?;
            let c_count = count.max(0) as usize;
            let c_in = in_tok.max(0) as usize;
            let c_saved = saved.max(0) as usize;
            let c_time = time_ms.max(0) as u64;
            let pct = if c_in > 0 {
                (c_saved as f64 / c_in as f64) * 100.0
            } else {
                0.0
            };
            let avg_t = if c_count > 0 { c_time / c_count as u64 } else { 0 };

            by_command.push(CommandStat {
                command: cmd,
                count: c_count,
                saved_tokens: c_saved,
                savings_pct: pct,
                avg_time_ms: avg_t,
            });
        }

        // Daily breakdown (last 30 days)
        let day_sql = format!(
            "SELECT 
                SUBSTR(timestamp, 1, 10) as day,
                COUNT(*),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(saved_tokens), 0),
                COALESCE(SUM(duration_ms), 0)
             FROM command_history {}
             GROUP BY day
             ORDER BY day DESC
             LIMIT 30",
            where_clause
        );

        let mut day_stmt = self.conn.prepare(&day_sql)?;
        let day_iter = day_stmt.query_map([], |r| {
            let day: String = r.get(0)?;
            let count: i64 = r.get(1)?;
            let in_tok: i64 = r.get(2)?;
            let out_tok: i64 = r.get(3)?;
            let saved: i64 = r.get(4)?;
            let time_ms: i64 = r.get(5)?;
            Ok((day, count, in_tok, out_tok, saved, time_ms))
        })?;

        let mut by_day = Vec::new();
        for res in day_iter {
            let (day, count, in_tok, out_tok, saved, time_ms) = res?;
            let d_count = count.max(0) as usize;
            let d_in = in_tok.max(0) as usize;
            let d_out = out_tok.max(0) as usize;
            let d_saved = saved.max(0) as usize;
            let d_time = time_ms.max(0) as u64;
            let pct = if d_in > 0 {
                (d_saved as f64 / d_in as f64) * 100.0
            } else {
                0.0
            };
            let avg_t = if d_count > 0 { d_time / d_count as u64 } else { 0 };

            by_day.push(PeriodStat {
                period: day,
                count: d_count,
                input_tokens: d_in,
                output_tokens: d_out,
                saved_tokens: d_saved,
                savings_pct: pct,
                avg_time_ms: avg_t,
            });
        }

        Ok(GainSummary {
            total_commands,
            total_input,
            total_output,
            total_saved,
            avg_savings_pct,
            total_time_ms,
            avg_time_ms,
            by_command,
            by_day,
        })
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<CommandRecord>, RtkError> {
        let sql = "SELECT id, timestamp, original_cmd, rtk_cmd, input_tokens, output_tokens,
                          saved_tokens, savings_pct, duration_ms, exit_code, subproject_id
                   FROM command_history
                   ORDER BY id DESC
                   LIMIT ?1";
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![limit as i64], |r| {
            Ok(CommandRecord {
                id: r.get(0)?,
                timestamp: r.get(1)?,
                original_cmd: r.get(2)?,
                rtk_cmd: r.get(3)?,
                input_tokens: r.get::<_, i64>(4)?.max(0) as usize,
                output_tokens: r.get::<_, i64>(5)?.max(0) as usize,
                saved_tokens: r.get::<_, i64>(6)?.max(0) as usize,
                savings_pct: r.get(7)?,
                duration_ms: r.get::<_, i64>(8)?.max(0) as u64,
                exit_code: r.get(9)?,
                subproject_id: r.get(10)?,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

/// Estimates tokens for a given text (~4 characters per token).
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let chars = text.chars().count();
    chars.div_ceil(4)
}

/// CLI handler for `toksi rtk gain`.
pub fn run_gain(args: &[String]) -> i32 {
    let mut json_format = false;
    let mut reset_requested = false;
    let mut history_requested = false;
    let mut days_filter: Option<u32> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" if i + 1 < args.len() && args[i + 1] == "json" => {
                json_format = true;
                i += 1;
            }
            "--json" => json_format = true,
            "--reset" => reset_requested = true,
            "--history" => history_requested = true,
            "--days" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<u32>() {
                    days_filter = Some(d);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let tracker = match Tracker::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("toksi rtk gain: failed to initialize tracker: {}", e);
            return 1;
        }
    };

    if reset_requested {
        if let Err(e) = tracker.reset() {
            eprintln!("toksi rtk gain: reset failed: {}", e);
            return 1;
        }
        if json_format {
            println!("{{\"reset\": true}}");
        } else {
            println!("{} RTK command history reset successfully.", C.green("✓"));
        }
        return 0;
    }

    if history_requested {
        let history = match tracker.get_history(50) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("toksi rtk gain: failed to query history: {}", e);
                return 1;
            }
        };

        if json_format {
            match serde_json::to_string_pretty(&history) {
                Ok(s) => println!("{}", s),
                Err(e) => {
                    eprintln!("toksi rtk gain: json serialization failed: {}", e);
                    return 1;
                }
            }
        } else {
            println!("{}", C.bold("Recent RTK Command History (last 50):"));
            println!("{:<6} {:<20} {:<30} {:>10} {:>10} {:>8}", "ID", "Time", "Command", "In Tok", "Saved", "Saved %");
            println!("{:-<90}", "");
            for r in &history {
                let time_display = if r.timestamp.len() >= 19 {
                    &r.timestamp[..19]
                } else {
                    &r.timestamp
                };
                let cmd_trunc = if r.original_cmd.len() > 28 {
                    format!("{}...", &r.original_cmd[..25])
                } else {
                    r.original_cmd.clone()
                };
                println!(
                    "{:<6} {:<20} {:<30} {:>10} {:>10} {:>7.1}%",
                    r.id, time_display, cmd_trunc, r.input_tokens, r.saved_tokens, r.savings_pct
                );
            }
        }
        return 0;
    }

    let summary = match tracker.get_summary(days_filter) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("toksi rtk gain: failed to compute summary: {}", e);
            return 1;
        }
    };

    if json_format {
        match serde_json::to_string_pretty(&summary) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("toksi rtk gain: json serialization failed: {}", e);
                return 1;
            }
        }
    } else {
        println!("{}", C.bold("RTK Token Savings Summary (toksi rtk gain):"));
        println!("  Database: {}", normalize_path(&tracker.rtk_dir().join("tracking.db").to_string_lossy()));
        if !tracker.subproject_id().is_empty() {
            println!("  Subproject: {}", tracker.subproject_id());
        }
        println!();
        println!("  Total commands filtered: {}", C.cyan(&summary.total_commands.to_string()));
        println!("  Total raw input tokens:  {}", summary.total_input);
        println!("  Total emitted tokens:    {}", summary.total_output);
        println!(
            "  Total tokens saved:      {} ({:.1}% reduction)",
            C.green(&summary.total_saved.to_string()),
            summary.avg_savings_pct
        );
        println!("  Average execution time:  {} ms", summary.avg_time_ms);

        if !summary.by_command.is_empty() {
            println!("\n{}", C.bold("Top Commands by Token Savings:"));
            println!("{:<25} {:>8} {:>12} {:>10} {:>10}", "Command", "Runs", "Saved Tok", "Saved %", "Avg ms");
            println!("{:-<70}", "");
            for c in &summary.by_command {
                println!(
                    "{:<25} {:>8} {:>12} {:>9.1}% {:>9}ms",
                    c.command, c.count, c.saved_tokens, c.savings_pct, c.avg_time_ms
                );
            }
        }

        if !summary.by_day.is_empty() {
            println!("\n{}", C.bold("Activity by Day:"));
            println!("{:<12} {:>8} {:>12} {:>12} {:>10}", "Date", "Runs", "In Tok", "Saved Tok", "Saved %");
            println!("{:-<60}", "");
            for d in summary.by_day.iter().take(7) {
                println!(
                    "{:<12} {:>8} {:>12} {:>12} {:>9.1}%",
                    d.period, d.count, d.input_tokens, d.saved_tokens, d.savings_pct
                );
            }
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tracker_record_and_summary() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempdir()?;
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(&git_dir)?;

        let tracker = Tracker::for_dir(Some(tmp.path()))?;

        tracker.record("cargo test", "toksi rtk cargo test", 1000, 200, 50, 0)?;
        tracker.record("git status", "toksi rtk git status", 500, 100, 10, 0)?;

        let summary = tracker.get_summary(None)?;

        assert_eq!(summary.total_commands, 2);
        assert_eq!(summary.total_input, 1500);
        assert_eq!(summary.total_output, 300);
        assert_eq!(summary.total_saved, 1200);
        assert!((summary.avg_savings_pct - 80.0).abs() < 0.1);

        let history = tracker.get_history(10)?;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].original_cmd, "git status");
        Ok(())
    }
}
