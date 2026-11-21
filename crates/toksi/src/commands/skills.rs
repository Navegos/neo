// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/skills.rs
// created: 2026-11-21

//! CLI dispatcher for `toksi skills <sync|list|heal|grade|scan|config-dir>`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::config::{load_toksi_toml_config, save_toksi_toml_config};
use crate::rules::skill_validator::{analyze_directory, heal_skill_content};
use crate::rules::skills_repo::{
    discover_vetted_catalog, sync_external_skills_repo, SkillsRepoConfig,
};
use crate::util::paths::normalize_path;

pub fn run_skills_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    let arg1 = args.get(1).map(String::as_str).unwrap_or("");
    let repo_cfg = SkillsRepoConfig::load();

    match sub {
        "sync" => match sync_external_skills_repo(&repo_cfg) {
            Ok(rep) => {
                println!(
                    "Skills sync complete: synced={} healed={} blocked={}",
                    rep.synced_files.len(),
                    rep.healed_files.len(),
                    rep.blocked_files.len()
                );
                0
            }
            Err(e) => {
                eprintln!("Skills sync failed: {e}");
                1
            }
        },
        "list" => match discover_vetted_catalog(&repo_cfg) {
            Ok(cat) => {
                for entry in &cat.entries {
                    println!(
                        "  * {} [{} / {}] ({})",
                        entry.name, entry.grade, entry.score, entry.source_path
                    );
                }
                println!("Total vetted skills: {}", cat.entries.len());
                0
            }
            Err(e) => {
                eprintln!("Skills list failed: {e}");
                1
            }
        },
        "heal" if !arg1.is_empty() => {
            let p = Path::new(arg1);
            let Ok(raw) = std::fs::read_to_string(p) else {
                eprintln!("Failed to read '{}'", arg1);
                return 1;
            };
            let healed = heal_skill_content(arg1, &raw, None, true);
            if healed.changed {
                if let Err(e) = std::fs::write(p, &healed.output) {
                    eprintln!("Failed to write healed file: {e}");
                    return 1;
                }
            }
            println!("Healed '{}' (changed={})", arg1, healed.changed);
            0
        }
        "grade" | "scan" => {
            let dir = if arg1.is_empty() {
                repo_cfg.skills_dir.clone()
            } else {
                Path::new(arg1).to_path_buf()
            };
            match analyze_directory(&dir) {
                Ok(batch) => {
                    println!(
                        "Graded {} skills: score={} errors={} warnings={} info={}",
                        batch.files.len(),
                        batch.score,
                        batch.error_count,
                        batch.warning_count,
                        batch.info_count
                    );
                    0
                }
                Err(e) => {
                    eprintln!("Skills {sub} failed: {e}");
                    1
                }
            }
        }
        "config-dir" if !arg1.is_empty() => {
            let mut cfg = load_toksi_toml_config();
            cfg.repositories.skills_dir = normalize_path(arg1);
            if let Err(e) = save_toksi_toml_config(&cfg) {
                eprintln!("Failed to save config: {e}");
                return 1;
            }
            println!("Skills directory set to '{}'", cfg.repositories.skills_dir);
            0
        }
        _ => {
            eprintln!("Usage: toksi skills <sync|list|heal <file>|grade [dir]|scan [dir]|config-dir <path>>");
            1
        }
    }
}
