// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/byok.rs
// created: 2026-11-20
// lastModified: 2026-11-20

#![deny(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{open_default_db, ByokProfileRecord};
use crate::util::colors::C;

fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn run_byok_cli(args: &[String]) -> i32 {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
    match sub {
        "list" | "ls" => run_byok_list(&args[1..]),
        "add" => run_byok_add(&args[1..]),
        "update" => run_byok_update(&args[1..]),
        "delete" | "rm" => run_byok_delete(&args[1..]),
        "set-url" => run_byok_set_url(&args[1..]),
        "set-model" => run_byok_set_model(&args[1..]),
        "help" | "--help" | "-h" => {
            print_byok_usage();
            0
        }
        other => {
            eprintln!("Unknown byok subcommand: {}", other);
            print_byok_usage();
            1
        }
    }
}

fn print_byok_usage() {
    eprintln!(
        "Usage: toksi byok <subcommand>\n\
         Subcommands:\n  \
           list [--provider <id>] [--json]\n  \
           add <provider_id> --label <label> --model <model_id> --url <base_url> [--effort <none|low|medium|high|xhigh>] [--auth-mode <api_key|oauth|adc|keyring>] [--secret-ref <ref>] [--gcp-project <proj>] [--gcp-location <loc>] [--priority <n>] [--weight <n>] [--no-headroom] [--disabled]\n  \
           update <profile_id> [--label <label>] [--model <model_id>] [--effort <effort>] [--url <base_url>] [--auth-mode <mode>] [--secret-ref <ref>] [--gcp-project <proj>] [--gcp-location <loc>] [--priority <n>] [--weight <n>] [--headroom|--no-headroom] [--enable|--disable]\n  \
           delete <profile_id>\n  \
           set-url <profile_id> <base_url>\n  \
           set-model <profile_id> <model_id> [--effort <none|low|medium|high|xhigh>]"
    );
}

fn run_byok_list(args: &[String]) -> i32 {
    let mut provider_filter: Option<&str> = None;
    let mut as_json = false;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--provider" && i + 1 < args.len() {
            provider_filter = Some(&args[i + 1]);
            i += 1;
        } else if args[i] == "--json" {
            as_json = true;
        }
        i += 1;
    }

    let db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };
    let mut profiles = match db.list_byok_profiles() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to list BYOK profiles: {}", e);
            return 1;
        }
    };
    if let Some(pid) = provider_filter {
        profiles.retain(|p| p.provider_id.eq_ignore_ascii_case(pid));
    }

    if as_json {
        match serde_json::to_string_pretty(&profiles) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("JSON serialization error: {}", e);
                return 1;
            }
        }
        return 0;
    }

    println!("{}", C.bold("Toksi Multi-Instance <byok, model, effort> Profiles"));
    println!("{}", C.gray("------------------------------------------------------------------------------------------"));
    println!(
        "{:<24} {:<12} {:<12} {:<22} {:<8} {:<8} {:<8}",
        "Profile ID", "Provider", "Label", "Model", "Effort", "Auth", "Status"
    );
    println!("{}", C.gray("------------------------------------------------------------------------------------------"));
    if profiles.is_empty() {
        println!("{}", C.gray("No BYOK profiles configured."));
    } else {
        for p in profiles {
            let status = if p.is_active {
                C.green("Active")
            } else {
                C.gray("Disabled")
            };
            println!(
                "{:<24} {:<12} {:<12} {:<22} {:<8} {:<8} {:<8}",
                p.id, p.provider_id, p.instance_label, p.model_id, p.thinking_effort, p.auth_mode, status
            );
        }
    }
    0
}

fn run_byok_add(args: &[String]) -> i32 {
    if args.is_empty() {
        print_byok_usage();
        return 1;
    }
    let provider_id = args[0].trim().to_string();
    let mut instance_label = "default".to_string();
    let mut model_id = String::new();
    let mut thinking_effort = "medium".to_string();
    let mut base_url = String::new();
    let mut secret_ref = String::new();
    let mut auth_mode = "api_key".to_string();
    let mut gcp_project = String::new();
    let mut gcp_location = "us-central1".to_string();
    let mut priority = 100i32;
    let mut weight = 100i32;
    let mut headroom_enabled = true;
    let mut is_active = true;
    let mut explicit_id: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--id" if i + 1 < args.len() => {
                explicit_id = Some(args[i + 1].clone());
                i += 1;
            }
            "--label" if i + 1 < args.len() => {
                instance_label = args[i + 1].clone();
                i += 1;
            }
            "--model" if i + 1 < args.len() => {
                model_id = args[i + 1].clone();
                i += 1;
            }
            "--effort" if i + 1 < args.len() => {
                thinking_effort = args[i + 1].to_ascii_lowercase();
                i += 1;
            }
            "--url" if i + 1 < args.len() => {
                base_url = args[i + 1].clone();
                i += 1;
            }
            "--secret-ref" if i + 1 < args.len() => {
                secret_ref = args[i + 1].clone();
                i += 1;
            }
            "--auth-mode" if i + 1 < args.len() => {
                auth_mode = args[i + 1].to_ascii_lowercase();
                i += 1;
            }
            "--gcp-project" if i + 1 < args.len() => {
                gcp_project = args[i + 1].clone();
                i += 1;
            }
            "--gcp-location" if i + 1 < args.len() => {
                gcp_location = args[i + 1].clone();
                i += 1;
            }
            "--priority" if i + 1 < args.len() => {
                priority = args[i + 1].parse().unwrap_or(100);
                i += 1;
            }
            "--weight" if i + 1 < args.len() => {
                weight = args[i + 1].parse().unwrap_or(100);
                i += 1;
            }
            "--no-headroom" => headroom_enabled = false,
            "--disabled" => is_active = false,
            other => {
                eprintln!("Unknown option for 'byok add': {}", other);
                return 1;
            }
        }
        i += 1;
    }

    if provider_id.is_empty() || model_id.is_empty() || base_url.is_empty() {
        eprintln!("Missing required arguments: <provider_id>, --model <model_id>, and --url <base_url> are required.");
        return 1;
    }

    let now = now_epoch_secs();
    let id = explicit_id.unwrap_or_else(|| {
        format!("{}:{}:{}:{}", provider_id, instance_label, model_id, thinking_effort)
    });

    let record = ByokProfileRecord {
        id: id.clone(),
        provider_id,
        instance_label,
        model_id,
        thinking_effort,
        base_url,
        secret_ref,
        auth_mode,
        gcp_project,
        gcp_location,
        priority,
        weight,
        headroom_enabled,
        is_active,
        created_at: now,
        updated_at: now,
    };

    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };

    if let Err(e) = db.upsert_byok_profile(&record) {
        eprintln!("Failed to save BYOK profile '{}': {}", id, e);
        return 1;
    }
    println!("Saved BYOK profile '{}'.", id);
    0
}

fn run_byok_update(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: toksi byok update <profile_id> [options]");
        return 1;
    }
    let profile_id = &args[0];
    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };
    let mut record = match db.get_byok_profile(profile_id) {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("BYOK profile '{}' not found.", profile_id);
            return 1;
        }
        Err(e) => {
            eprintln!("Failed to query BYOK profile '{}': {}", profile_id, e);
            return 1;
        }
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--label" if i + 1 < args.len() => {
                record.instance_label = args[i + 1].clone();
                i += 1;
            }
            "--model" if i + 1 < args.len() => {
                record.model_id = args[i + 1].clone();
                i += 1;
            }
            "--effort" if i + 1 < args.len() => {
                record.thinking_effort = args[i + 1].to_ascii_lowercase();
                i += 1;
            }
            "--url" if i + 1 < args.len() => {
                record.base_url = args[i + 1].clone();
                i += 1;
            }
            "--secret-ref" if i + 1 < args.len() => {
                record.secret_ref = args[i + 1].clone();
                i += 1;
            }
            "--auth-mode" if i + 1 < args.len() => {
                record.auth_mode = args[i + 1].to_ascii_lowercase();
                i += 1;
            }
            "--gcp-project" if i + 1 < args.len() => {
                record.gcp_project = args[i + 1].clone();
                i += 1;
            }
            "--gcp-location" if i + 1 < args.len() => {
                record.gcp_location = args[i + 1].clone();
                i += 1;
            }
            "--priority" if i + 1 < args.len() => {
                record.priority = args[i + 1].parse().unwrap_or(record.priority);
                i += 1;
            }
            "--weight" if i + 1 < args.len() => {
                record.weight = args[i + 1].parse().unwrap_or(record.weight);
                i += 1;
            }
            "--headroom" => record.headroom_enabled = true,
            "--no-headroom" => record.headroom_enabled = false,
            "--enable" => record.is_active = true,
            "--disable" => record.is_active = false,
            other => {
                eprintln!("Unknown option for 'byok update': {}", other);
                return 1;
            }
        }
        i += 1;
    }
    record.updated_at = now_epoch_secs();

    if let Err(e) = db.upsert_byok_profile(&record) {
        eprintln!("Failed to update BYOK profile '{}': {}", profile_id, e);
        return 1;
    }
    println!("Updated BYOK profile '{}'.", profile_id);
    0
}

fn run_byok_delete(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Usage: toksi byok delete <profile_id>");
        return 1;
    }
    let profile_id = &args[0];
    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };
    match db.delete_byok_profile(profile_id) {
        Ok(true) => {
            println!("Deleted BYOK profile '{}'.", profile_id);
            0
        }
        Ok(false) => {
            eprintln!("BYOK profile '{}' not found.", profile_id);
            1
        }
        Err(e) => {
            eprintln!("Failed to delete BYOK profile '{}': {}", profile_id, e);
            1
        }
    }
}

fn run_byok_set_url(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: toksi byok set-url <profile_id> <base_url>");
        return 1;
    }
    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };
    match db.update_byok_profile_url(&args[0], &args[1]) {
        Ok(true) => {
            println!("Updated base_url for '{}' -> {}", args[0], args[1]);
            0
        }
        Ok(false) => {
            eprintln!("BYOK profile '{}' not found.", args[0]);
            1
        }
        Err(e) => {
            eprintln!("Failed to update base_url: {}", e);
            1
        }
    }
}

fn run_byok_set_model(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: toksi byok set-model <profile_id> <model_id> [--effort <effort>]");
        return 1;
    }
    let mut effort: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--effort" && i + 1 < args.len() {
            effort = Some(args[i + 1].to_ascii_lowercase());
            i += 1;
        }
        i += 1;
    }
    let mut db = match open_default_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to open Toksi database: {}", e);
            return 1;
        }
    };
    match db.update_byok_profile_model(&args[0], &args[1], effort.as_deref()) {
        Ok(true) => {
            println!("Updated model for '{}' -> {}", args[0], args[1]);
            0
        }
        Ok(false) => {
            eprintln!("BYOK profile '{}' not found.", args[0]);
            1
        }
        Err(e) => {
            eprintln!("Failed to update model: {}", e);
            1
        }
    }
}
