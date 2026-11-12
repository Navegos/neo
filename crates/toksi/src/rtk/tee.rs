// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/tee.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::fs;
use std::path::Path;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::tracker::{resolve_rtk_dir, RtkError};
use crate::util::colors::C;
use crate::util::paths::normalize_path;

pub const MIN_TEE_BYTES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeRecord {
    pub hash: String,
    pub timestamp: String,
    pub command: String,
    pub exit_code: i32,
    pub raw_size: usize,
    pub file_path: String,
}

pub fn save_tee_output(cmd: &str, raw_output: &str, exit_code: i32) -> Result<Option<String>, RtkError> {
    if raw_output.len() < MIN_TEE_BYTES {
        return Ok(None);
    }

    let (rtk_dir, _) = resolve_rtk_dir(None)?;
    let tee_dir = rtk_dir.join("tee");
    if !tee_dir.exists() {
        fs::create_dir_all(&tee_dir).map_err(RtkError::Io)?;
    }

    let mut hasher = Sha256::new();
    hasher.update(raw_output.as_bytes());
    let result = hasher.finalize();
    let full_hash: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    let short_hash = &full_hash[..10];

    let file_path = tee_dir.join(format!("{}.txt", short_hash));
    fs::write(&file_path, raw_output.as_bytes()).map_err(RtkError::Io)?;

    let db_path = rtk_dir.join("tracking.db");
    if let Ok(conn) = Connection::open(&db_path) {
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS tee_recalls (
                hash TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                command TEXT NOT NULL,
                exit_code INTEGER NOT NULL,
                raw_size INTEGER NOT NULL,
                file_path TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tee_timestamp ON tee_recalls(timestamp);",
            [],
        );

        let now = Utc::now().to_rfc3339();
        let norm_path = normalize_path(&file_path.to_string_lossy());
        let _ = conn.execute(
            "INSERT OR REPLACE INTO tee_recalls (hash, timestamp, command, exit_code, raw_size, file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                short_hash,
                now,
                cmd,
                exit_code,
                raw_output.len() as i64,
                norm_path
            ],
        );
    }

    Ok(Some(format!("[full output: toksi rtk recall {}]", short_hash)))
}

pub fn recall_output(hash: &str) -> Result<String, RtkError> {
    let trimmed = hash.trim();
    if trimmed.is_empty() {
        return Err(RtkError::NotFound("hash parameter cannot be empty".to_string()));
    }

    let (rtk_dir, _) = resolve_rtk_dir(None)?;
    let tee_dir = rtk_dir.join("tee");

    // 1. Check direct file by prefix
    if let Ok(entries) = fs::read_dir(&tee_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(trimmed) && name.ends_with(".txt") {
                return fs::read_to_string(entry.path()).map_err(RtkError::Io);
            }
        }
    }

    // 2. Fallback to SQLite tee_recalls table
    let db_path = rtk_dir.join("tracking.db");
    if let Ok(conn) = Connection::open(&db_path) {
        let mut stmt = conn.prepare(
            "SELECT file_path FROM tee_recalls WHERE hash LIKE ?1 LIMIT 1"
        )?;
        let pattern = format!("{}%", trimmed);
        if let Ok(path_str) = stmt.query_row(params![pattern], |r| r.get::<_, String>(0)) {
            let p = Path::new(&path_str);
            if p.exists() {
                return fs::read_to_string(p).map_err(RtkError::Io);
            }
        }
    }

    Err(RtkError::NotFound(format!("No saved raw output found matching hash '{}'", trimmed)))
}

pub fn list_recalls(limit: usize) -> Result<Vec<TeeRecord>, RtkError> {
    let (rtk_dir, _) = resolve_rtk_dir(None)?;
    let db_path = rtk_dir.join("tracking.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open(&db_path)?;
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS tee_recalls (
            hash TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            command TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            raw_size INTEGER NOT NULL,
            file_path TEXT NOT NULL
        )",
        [],
    );

    let mut stmt = conn.prepare(
        "SELECT hash, timestamp, command, exit_code, raw_size, file_path
         FROM tee_recalls
         ORDER BY timestamp DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit as i64], |r| {
        Ok(TeeRecord {
            hash: r.get(0)?,
            timestamp: r.get(1)?,
            command: r.get(2)?,
            exit_code: r.get(3)?,
            raw_size: r.get::<_, i64>(4)?.max(0) as usize,
            file_path: r.get(5)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn clear_recalls() -> Result<usize, RtkError> {
    let (rtk_dir, _) = resolve_rtk_dir(None)?;
    let tee_dir = rtk_dir.join("tee");
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&tee_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                let _ = fs::remove_file(entry.path());
                count += 1;
            }
        }
    }

    let db_path = rtk_dir.join("tracking.db");
    if let Ok(conn) = Connection::open(&db_path) {
        let _ = conn.execute("DELETE FROM tee_recalls", []);
    }

    Ok(count)
}

pub fn run_recall_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: toksi rtk recall <hash>");
        return 1;
    }

    let hash = &args[0];
    match recall_output(hash) {
        Ok(content) => {
            print!("{}", content);
            0
        }
        Err(e) => {
            eprintln!("toksi rtk recall: {}", e);
            1
        }
    }
}

pub fn run_tee_cli(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");

    match sub {
        "clear" => {
            match clear_recalls() {
                Ok(count) => {
                    println!("{} Cleared {} saved raw failure logs.", C.green("✓"), count);
                    0
                }
                Err(e) => {
                    eprintln!("toksi rtk tee clear: {}", e);
                    1
                }
            }
        }
        _ => {
            match list_recalls(20) {
                Ok(recalls) => {
                    if recalls.is_empty() {
                        println!("No saved raw failure logs found.");
                        return 0;
                    }
                    println!("{}", C.bold("Saved Raw Failure Logs (toksi rtk recall <hash>):"));
                    println!("{:<12} {:<20} {:<30} {:>8} {:>10}", "Hash", "Timestamp", "Command", "Exit", "Bytes");
                    println!("{:-<85}", "");
                    for r in &recalls {
                        let time_display = if r.timestamp.len() >= 19 {
                            &r.timestamp[..19]
                        } else {
                            &r.timestamp
                        };
                        let cmd_trunc = if r.command.len() > 28 {
                            format!("{}...", &r.command[..25])
                        } else {
                            r.command.clone()
                        };
                        println!(
                            "{:<12} {:<20} {:<30} {:>8} {:>10}",
                            C.cyan(&r.hash), time_display, cmd_trunc, r.exit_code, r.raw_size
                        );
                    }
                    0
                }
                Err(e) => {
                    eprintln!("toksi rtk tee: {}", e);
                    1
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_recall_tee() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempdir()?;
        let git_dir = tmp.path().join(".git");
        fs::create_dir_all(&git_dir)?;

        let raw_long = "error[E0308]: mismatched types\n".repeat(20);
        let hint = save_tee_output("cargo test", &raw_long, 101)?;
        assert!(hint.is_some());
        let hint_str = hint.unwrap_or_default();
        assert!(hint_str.contains("toksi rtk recall"));

        let parts: Vec<&str> = hint_str.split_whitespace().collect();
        let hash = parts.last().unwrap_or(&"").trim_end_matches(']');
        assert!(!hash.is_empty());

        let recovered = recall_output(hash)?;
        assert_eq!(recovered, raw_long);

        let list = list_recalls(5)?;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].hash, hash);

        let cleared = clear_recalls()?;
        assert_eq!(cleared, 1);
        Ok(())
    }
}
