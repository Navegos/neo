// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/slash.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use std::path::Path;

use super::heatmap::{calculate_score, format_score_text, generate_heatmap};
use super::precheck::run_precheck;
use super::staleness::{find_stale_events, stale_label};
use super::store::{discover_mem_dir, read_events, summary_path};
use crate::util::paths::normalize_path;

pub fn handle_slash_command(input: &str, root: Option<&Path>) -> String {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() || parts[0] != "/projectmem" {
        return "Unknown command. Use `/projectmem help` for available options.".to_string();
    }

    let sub = parts.get(1).copied().unwrap_or("help");

    match sub {
        "help" => {
            "**ProjectMem Slash Commands:**\n\
             - `/projectmem status` - Show project memory location, events count, and open issues\n\
             - `/projectmem score` - Show token & ROI failure prevention score\n\
             - `/projectmem heatmap` - Show top failure hotspots and failure clustering\n\
             - `/projectmem staleness` - Scan memories against git history for stale claims\n\
             - `/projectmem precheck [file]` - Check file or working tree for known failure history\n\
             - `/projectmem summary` - Show current distilled project summary\n\
             - `/projectmem gotchas` - List known gotchas and constraint notes\n\
             - `/projectmem help` - Show this help message"
                .to_string()
        }
        "status" => {
            let mem_dir = match discover_mem_dir(root) {
                Some(d) => normalize_path(&d.to_string_lossy()),
                None => "Not initialized (run `toksi mem init`)".to_string(),
            };
            let events = read_events(root).unwrap_or_default();
            let total = events.len();
            let open_issues = events.iter().filter(|e| e.event_type == "issue").count();
            let decisions = events.iter().filter(|e| e.event_type == "decision").count();
            let notes = events.iter().filter(|e| e.event_type == "note").count();

            format!(
                "**ProjectMem Status:**\n\
                 - Location: `{}`\n\
                 - Total Events: {}\n\
                 - Issues Logged: {}\n\
                 - Decisions: {}\n\
                 - Gotchas / Notes: {}",
                mem_dir, total, open_issues, decisions, notes
            )
        }
        "score" | "stats" => {
            let events = match read_events(root) {
                Ok(evs) => evs,
                Err(e) => return format!("Error reading events: {}", e),
            };
            let res = calculate_score(&events, None);
            format_score_text(&res)
        }
        "heatmap" => {
            let events = match read_events(root) {
                Ok(evs) => evs,
                Err(e) => return format!("Error reading events: {}", e),
            };
            let heatmap = generate_heatmap(&events);
            if heatmap.top_hotspots.is_empty() {
                return "No failure hotspots detected. Project memory is clean.".to_string();
            }
            let mut lines = vec!["**Top Failure Hotspots:**".to_string()];
            for h in heatmap.top_hotspots.iter().take(10) {
                lines.push(format!(
                    "- `{}`: {} failed attempts, {} open issues, {} total events",
                    h.file, h.failed_attempts, h.open_issues, h.total_events
                ));
            }
            lines.join("\n")
        }
        "staleness" => {
            let events = match read_events(root) {
                Ok(evs) => evs,
                Err(e) => return format!("Error reading events: {}", e),
            };
            let stale = find_stale_events(&events, root, super::staleness::STALE_COMMIT_THRESHOLD, None);
            if stale.is_empty() {
                return "No stale memories found — all decisions and fixes match current code.".to_string();
            }
            let mut lines = vec![format!("**Found {} potentially stale memory entry(ies):**", stale.len())];
            for s in stale.iter().take(10) {
                lines.push(format!("- {}", stale_label(s)));
            }
            lines.join("\n")
        }
        "precheck" => {
            let file_arg = parts.get(2).map(|s| vec![s.to_string()]);
            match run_precheck(file_arg.as_deref(), false, None, true, root) {
                Ok(warns) => {
                    if warns.is_empty() {
                        "projectmem precheck: no warnings. Safe to modify.".to_string()
                    } else {
                        let mut lines = vec![format!("projectmem precheck: {} warning(s):", warns.len())];
                        for w in warns {
                            lines.push(format!("  [{}] {} — {}", w.severity.to_uppercase(), w.file, w.title));
                        }
                        lines.join("\n")
                    }
                }
                Err(e) => format!("Precheck error: {}", e),
            }
        }
        "summary" => {
            if let Ok(p) = summary_path(root) {
                if p.exists() {
                    return std::fs::read_to_string(p).unwrap_or_else(|_| "Failed to read summary.".into());
                }
            }
            "No summary found. Run `toksi mem init` first.".to_string()
        }
        "gotchas" => {
            let events = match read_events(root) {
                Ok(evs) => evs,
                Err(e) => return format!("Error reading events: {}", e),
            };
            let notes: Vec<&super::schema::Event> = events.iter().filter(|e| e.event_type == "note").collect();
            if notes.is_empty() {
                return "No notes or gotchas logged yet.".to_string();
            }
            let mut lines = vec!["**Known Gotchas & Notes:**".to_string()];
            for n in notes.iter().rev().take(15) {
                let loc = n.location.as_deref().map(|l| format!(" [{}]", l)).unwrap_or_default();
                lines.push(format!("- {}{}", n.summary, loc));
            }
            lines.join("\n")
        }
        _ => format!("Unknown ProjectMem slash command `{}`. Type `/projectmem help` for usage.", sub),
    }
}
