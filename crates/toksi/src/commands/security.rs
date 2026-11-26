// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/security.rs
// created: 2026-11-21

//! CLI dispatcher for `toksi security [scan|scan-skills|guardrails|audit-evidence|heal-skills|analyzers]`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::rules::skill_validator::{analyze_directory, heal_skill_content};
use crate::security::analyzer::{format_scan_summary, scan_path_security};
use crate::security::audit::list_audit_evidence;
use crate::security::cel_decision::ScanPreset;
use crate::security::external_analyzers::probe_external_analyzers;
use crate::security::garak_redteam::run_defensive_garak_eval;
use crate::security::sarif::ReportFormat;
use crate::security::skill_scanner::scan_skill_or_plugin_with_preset;

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].as_str());
        }
        if let Some(rest) = args[i].strip_prefix(&format!("{flag}=")) {
            return Some(rest);
        }
        i += 1;
    }
    None
}

pub fn run_security_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("scan");
    let format = flag_value(args, "--format")
        .map(ReportFormat::from_str_loose)
        .unwrap_or(ReportFormat::Table);
    let preset = flag_value(args, "--policy")
        .map(ScanPreset::from_str_loose)
        .unwrap_or(ScanPreset::Balanced);

    match sub {
        "scan" => {
            let target = args
                .get(1)
                .filter(|s| !s.starts_with('-'))
                .map(String::as_str)
                .unwrap_or(".");
            let summary = scan_path_security(Path::new(target), None);
            println!("{}", format_scan_summary(&summary, format));
            0
        }
        "scan-skills" => {
            let target = args
                .get(1)
                .filter(|s| !s.starts_with('-'))
                .map(String::as_str)
                .unwrap_or("agents/skills");
            let path = Path::new(target);
            if path.is_file() {
                let raw = std::fs::read_to_string(path).unwrap_or_default();
                let rep = scan_skill_or_plugin_with_preset(target, &raw, None, preset);
                println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
            } else {
                match analyze_directory(path) {
                    Ok(batch) => {
                        println!(
                            "Scanned {} skill files: score={} errors={} warnings={} info={}",
                            batch.files.len(),
                            batch.score,
                            batch.error_count,
                            batch.warning_count,
                            batch.info_count
                        );
                    }
                    Err(e) => {
                        eprintln!("scan-skills error: {e}");
                        return 1;
                    }
                }
            }
            0
        }
        "guardrails" => {
            let eval = run_defensive_garak_eval();
            println!("{}", serde_json::to_string_pretty(&eval).unwrap_or_default());
            0
        }
        "audit-evidence" => {
            let entries = list_audit_evidence(Some(Path::new(".")));
            println!("{}", serde_json::to_string_pretty(&entries).unwrap_or_default());
            0
        }
        "heal-skills" => {
            let target = args.get(1).map(String::as_str).unwrap_or("");
            if target.is_empty() {
                eprintln!("Usage: toksi security heal-skills <SKILL.md>");
                return 1;
            }
            let p = Path::new(target);
            let Ok(raw) = std::fs::read_to_string(p) else {
                eprintln!("Failed to read '{}'", target);
                return 1;
            };
            let healed = heal_skill_content(target, &raw, None, true);
            if healed.changed
                && let Err(e) = std::fs::write(p, &healed.output) {
                    eprintln!("Failed to write healed file: {e}");
                    return 1;
                }
            println!("Healed '{}' (changed={})", target, healed.changed);
            0
        }
        "analyzers" => {
            let status = probe_external_analyzers();
            println!("{}", serde_json::to_string_pretty(&status).unwrap_or_default());
            0
        }
        _ => {
            eprintln!("Usage: toksi security <scan|scan-skills|guardrails|audit-evidence|heal-skills|analyzers> [--format json|sarif|markdown|table|html] [--policy strict|balanced|low-noise|quiet|permissive]");
            1
        }
    }
}
