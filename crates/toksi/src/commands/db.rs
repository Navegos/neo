// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/db.rs
// created: 2026-11-04
// lastModified: 2026-11-04

use std::path::Path;
use crate::db::{open_default_db, sync_from_legacy_byok_json, sync_to_legacy_byok_json};
use crate::util::colors::C;

pub fn run_db_cli(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");
    match sub {
        "status" => run_db_status(),
        "export" => {
            let mut out_path = None;
            let mut i = 1;
            while i < args.len() {
                if (args[i] == "--out" || args[i] == "-o") && i + 1 < args.len() {
                    out_path = Some(args[i + 1].clone());
                    i += 1;
                }
                i += 1;
            }
            run_db_export(out_path.as_deref())
        }
        "import" => {
            if args.len() < 2 {
                eprintln!("Usage: toksi db import <file.json> [--overwrite]");
                return 1;
            }
            let file_path = &args[1];
            let overwrite = args.iter().any(|a| a == "--overwrite");
            run_db_import(file_path, overwrite)
        }
        "backup" => {
            let mut out_path = None;
            let mut i = 1;
            while i < args.len() {
                if (args[i] == "--out" || args[i] == "-o") && i + 1 < args.len() {
                    out_path = Some(args[i + 1].clone());
                    i += 1;
                }
                i += 1;
            }
            run_db_backup(out_path.as_deref())
        }
        "restore" => {
            if args.len() < 2 {
                eprintln!("Usage: toksi db restore <file.db>");
                return 1;
            }
            run_db_restore(&args[1])
        }
        "sync" => run_db_sync(),
        _ => {
            eprintln!("Unknown db command: {}", sub);
            eprintln!("Usage: toksi db [status|export|import|backup|restore|sync]");
            1
        }
    }
}

fn run_db_status() -> i32 {
    let db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi configuration database: {}", e);
            return 1;
        }
    };
    let stats = match db.stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read database statistics: {}", e);
            return 1;
        }
    };

    println!("{}", C.bold("Toksi SQLite Configuration Database"));
    println!("{}", C.gray("----------------------------------------------------------------------"));
    println!("Database Path:    {}", stats.db_path);
    println!("File Size:        {} bytes ({:.2} KB)", stats.file_size_bytes, stats.file_size_bytes as f64 / 1024.0);
    println!("{}", C.gray("----------------------------------------------------------------------"));
    println!("Providers:        {}", stats.providers_count);
    println!("Provider Keys:    {}", stats.keys_count);
    println!("Provider Models:  {}", stats.models_count);
    println!("BYOK Routes:      {}", stats.routes_count);
    println!("Agent Configs:    {}", stats.agents_count);
    println!("Gateway Configs:  {}", stats.gateway_configs_count);
    println!("{}", C.gray("----------------------------------------------------------------------"));
    0
}

fn run_db_export(out_path: Option<&str>) -> i32 {
    let db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };
    let json_str = match db.export_json() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to export database configurations: {}", e);
            return 1;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = std::fs::write(path, json_str) {
            eprintln!("Failed to write export to '{}': {}", path, e);
            return 1;
        }
        println!("Successfully exported configurations to: {}", path);
    } else {
        println!("{}", json_str);
    }
    0
}

fn run_db_import(file_path: &str, overwrite: bool) -> i32 {
    let raw = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", file_path, e);
            return 1;
        }
    };

    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };

    match db.import_json(&raw, overwrite) {
        Ok(count) => {
            println!("Successfully imported {} records from: {}", count, file_path);
            let _ = sync_to_legacy_byok_json(&db);
            0
        }
        Err(e) => {
            eprintln!("Failed to import database configurations: {}", e);
            1
        }
    }
}

fn run_db_backup(out_path: Option<&str>) -> i32 {
    let db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };

    let dest = if let Some(p) = out_path {
        std::path::PathBuf::from(p)
    } else {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        db.path().with_file_name(format!("toksi_backup_{}.db", ts))
    };

    if let Err(e) = db.backup_to_file(&dest) {
        eprintln!("Failed to create database backup: {}", e);
        return 1;
    }

    println!("Database online backup completed: {}", dest.display());
    0
}

fn run_db_restore(src_path: &str) -> i32 {
    let src = Path::new(src_path);
    if !src.exists() {
        eprintln!("Backup file does not exist: {}", src_path);
        return 1;
    }

    let default_path = crate::db::default_db_path();
    if let Some(parent) = default_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Copy backup over destination
    if let Err(e) = std::fs::copy(src, &default_path) {
        eprintln!("Failed to restore database from '{}': {}", src_path, e);
        return 1;
    }

    println!("Database successfully restored from: {}", src_path);
    if let Ok(db) = open_default_db() {
        let _ = sync_to_legacy_byok_json(&db);
    }
    0
}

fn run_db_sync() -> i32 {
    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };

    match sync_from_legacy_byok_json(&mut db) {
        Ok(count) => {
            println!("Synced {} route(s) from legacy JSON into SQLite database.", count);
            let _ = sync_to_legacy_byok_json(&db);
            0
        }
        Err(e) => {
            eprintln!("Failed to sync routes: {}", e);
            1
        }
    }
}

