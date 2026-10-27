// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/rtk_commands.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::collections::BTreeSet;
use std::process::Command;
use regex::Regex;

use crate::util::exec::resolve_rtk_bin;
use crate::util::paths::which;

pub fn rtk_commands_from_help(help: &str) -> Vec<String> {
    let re = Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();
    let mut commands = BTreeSet::new();
    let mut in_commands = false;

    for line in help.lines() {
        let trimmed = line.trim();
        if trimmed == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            break;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.is_empty() || fields[0].starts_with('-') {
            continue;
        }
        let name = fields[0];
        if !re.is_match(name) || name == "help" || name == "completions" || name.starts_with("__") {
            continue;
        }
        commands.insert(name.to_string());
    }

    commands.into_iter().collect()
}

pub fn rtk_commands() -> Result<Vec<String>, String> {
    let embedded: Vec<String> = crate::rtk::list_supported_commands().into_iter().map(String::from).collect();
    if !embedded.is_empty() {
        return Ok(embedded);
    }

    let mut bin = resolve_rtk_bin();
    if bin.is_empty() {
        bin = which("rtk");
    }
    if bin.is_empty() {
        return Err("rtk binary not found".to_string());
    }

    for args in [["--help"], ["help"]] {
        if let Ok(output) = Command::new(&bin).args(args).output() {
            let combined = String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr);
            let commands = rtk_commands_from_help(&combined);
            if !commands.is_empty() {
                return Ok(commands);
            }
        }
    }
    Err("rtk help did not list commands".to_string())
}

