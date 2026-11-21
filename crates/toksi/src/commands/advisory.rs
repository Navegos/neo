// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/advisory.rs
// created: 2026-11-21

//! CLI dispatcher for `toksi advisory <search|validate|prune-fp|build-shards|publish-pr|config-dir>`.

#![deny(unsafe_code)]

use crate::config::{load_toksi_toml_config, save_toksi_toml_config};
use crate::security::advisory_client::{
    check_upstream_duplicates, search_advisory_local_o1, UPSTREAM_ADVISORY_DATABASES,
};
use crate::validator::git_pr_driver::{commit_advisory_branch, resolve_advisory_repo_dir};
use crate::validator::shard_builder::build_three_tree_shards;
use crate::validator::validate::validate_advisory_json;

pub fn run_advisory_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("search");
    let arg1 = args.get(1).map(String::as_str).unwrap_or("");
    let cfg = load_toksi_toml_config();
    let repo_dir = resolve_advisory_repo_dir(Some(&cfg.repositories.advisory_dir));

    match sub {
        "search" if !arg1.is_empty() => {
            let local = search_advisory_local_o1(&repo_dir, arg1);
            let dedup = check_upstream_duplicates(&repo_dir, arg1);
            println!(
                "Advisory search '{}': local_hits={} checked_dbs={} duplicate={}",
                arg1,
                local.len(),
                UPSTREAM_ADVISORY_DATABASES.len(),
                dedup.duplicate_found
            );
            0
        }
        "validate" if !arg1.is_empty() => {
            let Ok(raw) = std::fs::read_to_string(arg1) else {
                eprintln!("Failed to read '{}'", arg1);
                return 1;
            };
            let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) else {
                eprintln!("Invalid JSON in '{}'", arg1);
                return 1;
            };
            let res = validate_advisory_json(&val);
            println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
            if res.valid { 0 } else { 1 }
        }
        "build-shards" => match build_three_tree_shards(&repo_dir, &[]) {
            Ok(rep) => {
                println!(
                    "Built 3-tree shards: advisories={} pages={} lookup={}",
                    rep.advisories_written, rep.domain_pages_written, rep.lookup_shards_written
                );
                0
            }
            Err(e) => {
                eprintln!("build-shards failed: {e}");
                1
            }
        },
        "publish-pr" => {
            let branch = if arg1.is_empty() { "toksi/advisory-update" } else { arg1 };
            match commit_advisory_branch(&repo_dir, branch, "chore(advisory): update zero-plaintext shards") {
                Ok(_) => {
                    println!("Prepared branch '{}' in '{}'", branch, repo_dir.display());
                    0
                }
                Err(e) => {
                    eprintln!("publish-pr failed: {e}");
                    1
                }
            }
        }
        "config-dir" if !arg1.is_empty() => {
            let mut next = cfg;
            next.repositories.advisory_dir = crate::util::paths::normalize_path(arg1);
            if let Err(e) = save_toksi_toml_config(&next) {
                eprintln!("Failed to save config: {e}");
                return 1;
            }
            println!("Advisory directory set to '{}'", next.repositories.advisory_dir);
            0
        }
        _ => {
            eprintln!("Usage: toksi advisory <search <q>|validate <file>|build-shards|publish-pr [branch]|config-dir <path>>");
            1
        }
    }
}
