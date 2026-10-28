// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/filter.rs
// created: 2026-10-27

use serde_json::Value;

pub const DEFAULT_MAX_LINES: usize = 200;
pub const DEFAULT_MAX_BYTES: usize = 64 * 1024; // 64KB max output per tool call

/// Strip ANSI escape codes and apply RTK command filter if applicable.
pub fn filter_and_pipe_rtk(cmd: Option<&str>, output: &str) -> String {
    if let Some(c) = cmd {
        crate::rtk::filter_sandbox_output(c, output)
    } else {
        crate::rtk::strip_ansi(output)
    }
}

/// Longest character prefix of `s` whose UTF-8 encoding is at most `max_bytes` bytes.
/// Guarantees ending on a valid UTF-8 character boundary.
pub fn byte_safe_prefix(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Slices text by line budget. If number of lines exceeds `max_lines`,
/// preserves the head and tail portions, inserting an omitted indicator.
pub fn slice_lines(text: &str, max_lines: usize) -> String {
    if max_lines == 0 {
        return String::new();
    }
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_lines {
        return text.to_string();
    }

    let half = max_lines / 2;
    let head = &lines[..half];
    let tail_start = lines.len() - (max_lines - half);
    let tail = &lines[tail_start..];
    let omitted = lines.len() - max_lines;

    let mut out = String::with_capacity(text.len());
    for line in head {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!("\n... [{} lines omitted by context-mode] ...\n\n", omitted));
    for (i, line) in tail.iter().enumerate() {
        out.push_str(line);
        if i + 1 < tail.len() {
            out.push('\n');
        }
    }
    out
}

/// Filters raw execution output:
/// 1. Passes through RTK filter engine (or ANSI stripper).
/// 2. Applies line slicing.
/// 3. Clamps to max_bytes budget with byte-safe truncation marker.
pub fn process_sandbox_output(cmd: Option<&str>, raw_output: &str, max_lines: usize, max_bytes: usize) -> String {
    let rtk_filtered = filter_and_pipe_rtk(cmd, raw_output);
    let line_sliced = slice_lines(&rtk_filtered, max_lines);

    if line_sliced.len() <= max_bytes {
        line_sliced
    } else {
        let prefix = byte_safe_prefix(&line_sliced, max_bytes.saturating_sub(64));
        format!("{}\n\n... [truncated at {} bytes by context-mode]", prefix, max_bytes)
    }
}

/// Truncate JSON stringification to `max_bytes` bytes.
pub fn truncate_json(val: &Value, max_bytes: usize) -> String {
    let s = serde_json::to_string_pretty(val).unwrap_or_else(|_| "null".to_string());
    if s.len() <= max_bytes {
        s
    } else {
        let prefix = byte_safe_prefix(&s, max_bytes.saturating_sub(32));
        format!("{}... [truncated]", prefix)
    }
}

