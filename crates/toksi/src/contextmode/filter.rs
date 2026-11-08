// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/filter.rs
// created: 2026-10-27
// lastModified: 2026-11-05

use serde_json::Value;

pub const DEFAULT_MAX_LINES: usize = 200;
pub const DEFAULT_MAX_BYTES: usize = 64 * 1024; // 64KB max output per tool call
pub const OUTPUT_CAP: usize = 16384; // 16KB cap for single tool output
pub const EXEC_PREVIEW_BYTES: usize = 4096; // 4KB preview budget when output exceeds cap
pub const REPLACE_TRIGGER_BYTES: usize = 4096;
pub const REPLACE_TRIGGER_BYTES_DEFAULT: usize = 4096;
pub const REPLACE_RETURN_BYTES: usize = 96; // D-M7-SV-3
pub const SHAPE_RETURN_BYTES: usize = 256; // D-M2-JSON-1
pub const REPLACE_CHUNK_BYTES: usize = 48; // D-M7-SV-3
pub const MAX_CHUNK_BYTES: usize = 4096;
pub const RANGE_READ_TOOLS: &[&str] = &["Read", "view"];
pub const READ_RANGE_KEYS: &[&str] = &["offset", "limit", "view_range"];
pub const RECALL_QUERY_MAX_TERMS: usize = 64;

/// Read CTX_REPLACE_TRIGGER_BYTES from environment (defaults to 4096).
pub fn get_replace_trigger_bytes() -> usize {
    std::env::var("CTX_REPLACE_TRIGGER_BYTES")
        .ok()
        .and_then(|val| val.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(REPLACE_TRIGGER_BYTES_DEFAULT)
}

/// Context-mode handle line referencing full indexed output via ctx_search.
pub fn replace_handle_line(intent: &str, handle: &str, bytes: usize) -> String {
    replace_handle_line_full(intent, handle, bytes, "", "")
}

/// Full context-mode handle line supporting custom host prefix and asked query terms.
pub fn replace_handle_line_full(intent: &str, handle: &str, bytes: usize, prefix: &str, asked: &str) -> String {
    let mut queries = Vec::new();
    if !intent.is_empty() {
        queries.push(serde_json::to_string(intent).unwrap_or_else(|_| format!("\"{}\"", intent)));
    } else {
        queries.push("\"\"".to_string());
    }
    if !asked.is_empty() {
        queries.push(serde_json::to_string(asked).unwrap_or_else(|_| format!("\"{}\"", asked)));
    }
    format!(
        "{}ctx_search({{queries:[{}], source:\"{}\"}}) searches the whole output ({} B).",
        prefix,
        queries.join(","),
        handle,
        bytes
    )
}

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

