// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rtk/mod.rs
// created: 2026-10-27
// lastModified: 2026-10-27

pub mod ansi;
pub mod engine;
pub mod filters;
pub mod hook;

pub use ansi::{has_ansi, strip_ansi, strip_ansi_cow, visible_len};
pub use engine::{can_filter, filter_output, list_supported_commands, run_command_filtered, tokenize_command};
pub use filters::{filter_command_output, is_filterable_command};
pub use hook::{is_already_rtk, rtk_rewrite, run_rtk_rewrite};

/// Cross-tool adapter for Context-Mode:
/// Filters sandbox subprocess outputs through RTK command profiles.
pub fn filter_sandbox_output(cmd: &str, output: &str) -> String {
    if can_filter(cmd) {
        filter_output(cmd, output, "", 0)
    } else {
        strip_ansi(output)
    }
}

/// Cross-tool adapter for Headroom Proxy:
/// Pre-compresses tool outputs on the streaming reverse proxy pipeline before LLM token alignment.
pub fn compress_tool_output(cmd_or_tool: &str, output: &str) -> String {
    if can_filter(cmd_or_tool) {
        filter_output(cmd_or_tool, output, "", 0)
    } else {
        strip_ansi(output)
    }
}

/// Cross-tool adapter for ProjectMem:
/// Extracts root cause error lines from failed test/build outputs to populate `record_attempt(outcome="failed")`.
pub fn extract_root_cause(cmd: &str, output: &str) -> Option<String> {
    let clean = strip_ansi(output);
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let bin = parts.first().copied().unwrap_or("");

    for line in clean.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error[E") || trimmed.starts_with("error:") {
            return Some(trimmed.to_string());
        }
        if bin == "cargo" && trimmed.contains("assertion `left == right` failed") {
            return Some(trimmed.to_string());
        }
        if (bin == "pytest" || bin == "python") && trimmed.starts_with("E   ") {
            return Some(trimmed.trim_start_matches("E   ").to_string());
        }
        if (bin == "vitest" || bin == "npm") && trimmed.contains("AssertionError:") {
            return Some(trimmed.to_string());
        }
        if (bin == "tsc") && trimmed.contains(": error TS") {
            return Some(trimmed.to_string());
        }
    }

    None
}

