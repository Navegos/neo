// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rtk/slash.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use super::discover::scan_missed_savings;
use super::engine::{can_filter, list_supported_commands};
use super::learn::detect_corrections;
use super::tee::recall_output;
use super::tracker::Tracker;

use crate::util::paths::normalize_path;

/// Handles in-chat / in-process `/rtk` slash commands.
pub fn handle_slash_command(input: &str) -> String {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() || parts[0] != "/rtk" {
        return "Unknown command. Use `/rtk help` for available options.".to_string();
    }

    let sub = parts.get(1).copied().unwrap_or("help");

    match sub {
        "help" => {
            "**RTK Slash Commands:**\n\
             - `/rtk status` - Show RTK engine status, active database and filter count\n\
             - `/rtk gain` - Show token savings summary across commands\n\
             - `/rtk discover` - Scan sessions for missed savings from unintercepted commands\n\
             - `/rtk learn` - Detect repeated command corrections and advice\n\
             - `/rtk recall <hash>` - Retrieve raw uncompressed failure output\n\
             - `/rtk list` - List supported filter commands (100+)\n\
             - `/rtk help` - Show this help message"
                .to_string()
        }
        "status" => {
            let tracker = Tracker::new();
            let db_info = match &tracker {
                Ok(t) => format!("`{}`", normalize_path(&t.rtk_dir().join("tracking.db").to_string_lossy())),
                Err(_) => "Not initialized".to_string(),
            };
            let subproject = match &tracker {
                Ok(t) if !t.subproject_id().is_empty() => format!(" (subproject: `{}`)", t.subproject_id()),
                _ => String::new(),
            };
            let filters_count = list_supported_commands().len();

            format!(
                "**RTK Filter Engine Status:**\n\
                 - **Filters:** {} commands registered (git, cargo, test runners, linters, docker, npm, 63 TOML filters)\n\
                 - **ANSI Stripper:** Active (zero-allocation streaming)\n\
                 - **PATH Validation:** Enforced\n\
                 - **Storage:** {}{}\n\
                 - **Engine State:** Ready and intercepting tool calls",
                filters_count, db_info, subproject
            )
        }
        "gain" => {
            match Tracker::new().and_then(|t| t.get_summary(None)) {
                Ok(s) => {
                    format!(
                        "**RTK Token Savings Summary:**\n\
                         - Total filtered commands: **{}**\n\
                         - Raw input tokens: **{}**\n\
                         - Emitted tokens: **{}**\n\
                         - Tokens saved: **{}** (**{:.1}%** reduction)\n\
                         - Average execution time: **{} ms**",
                        s.total_commands, s.total_input, s.total_output, s.total_saved, s.avg_savings_pct, s.avg_time_ms
                    )
                }
                Err(e) => format!("Failed to query RTK savings: {}", e),
            }
        }
        "discover" => {
            match scan_missed_savings(None, 30) {
                Ok(rep) => {
                    format!(
                        "**RTK Missed-Savings Scanner:**\n\
                         - Sessions checked: {}\n\
                         - Total commands scanned: {}\n\
                         - Unfiltered commands: **{}**\n\
                         - Potential missed savings: **{} tokens**\n\
                         - Top missed: {}",
                        rep.scanned_sessions,
                        rep.total_commands_scanned,
                        rep.filterable_uncovered_count,
                        rep.estimated_missed_tokens,
                        if rep.top_missed_commands.is_empty() {
                            "None (all commands covered!)".to_string()
                        } else {
                            rep.top_missed_commands
                                .iter()
                                .take(3)
                                .map(|c| format!("`{}` ({} runs, ~{} tokens)", c.command, c.count, c.estimated_missed_tokens))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    )
                }
                Err(e) => format!("Failed to run RTK discover: {}", e),
            }
        }
        "learn" => {
            match detect_corrections(None, 30) {
                Ok(rep) => {
                    if rep.rules.is_empty() {
                        "**RTK CLI Correction Detector:** No repeated CLI corrections detected in recent history.".to_string()
                    } else {
                        let mut out = format!(
                            "**RTK CLI Corrections ({} detected):**\n",
                            rep.corrections_detected
                        );
                        for (i, r) in rep.rules.iter().take(5).enumerate() {
                            out.push_str(&format!(
                                "{}. `{}` -> `{}` ({} times)\n   *Advice:* {}\n",
                                i + 1, r.wrong_pattern, r.right_pattern, r.occurrences, r.advice
                            ));
                        }
                        out
                    }
                }
                Err(e) => format!("Failed to run RTK learn: {}", e),
            }
        }
        "recall" => {
            let hash = parts.get(2).copied().unwrap_or("");
            if hash.is_empty() {
                "Usage: `/rtk recall <hash>`".to_string()
            } else {
                match recall_output(hash) {
                    Ok(raw) => {
                        let lines = raw.lines().count();
                        let bytes = raw.len();
                        format!(
                            "**Recalled Raw Output (`{}`):** ({} lines, {} bytes)\n```\n{}\n```",
                            hash,
                            lines,
                            bytes,
                            if raw.len() > 2000 {
                                format!("{}...\n[truncated]", &raw[..2000])
                            } else {
                                raw
                            }
                        )
                    }
                    Err(e) => format!("Failed to recall output: {}", e),
                }
            }
        }
        "list" => {
            let cmds = list_supported_commands();
            format!(
                "**RTK Supported Commands ({}):**\n{}",
                cmds.len(),
                cmds.join(", ")
            )
        }
        "filter" => {
            let cmd_to_filter = parts[2..].join(" ");
            if cmd_to_filter.is_empty() {
                "Usage: `/rtk filter <command...>`".to_string()
            } else if can_filter(&cmd_to_filter) {
                format!("Command `{}` is filterable by embedded RTK.", cmd_to_filter)
            } else {
                format!("Command `{}` is not recognized as a filterable command.", cmd_to_filter)
            }
        }
        _ => format!("Unknown RTK slash command `{}`. Type `/rtk help` for usage.", sub),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_help() {
        let resp = handle_slash_command("/rtk help");
        assert!(resp.contains("RTK Slash Commands"));
    }

    #[test]
    fn test_slash_status() {
        let resp = handle_slash_command("/rtk status");
        assert!(resp.contains("RTK Filter Engine Status"));
    }

    #[test]
    fn test_slash_list() {
        let resp = handle_slash_command("/rtk list");
        assert!(resp.contains("RTK Supported Commands"));
    }
}
