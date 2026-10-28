// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/compression/log.rs

use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogFormat {
    Cargo,
    Pytest,
    Npm,
    Jest,
    Make,
    Generic,
}

impl LogFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogFormat::Cargo => "cargo",
            LogFormat::Pytest => "pytest",
            LogFormat::Npm => "npm",
            LogFormat::Jest => "jest",
            LogFormat::Make => "make",
            LogFormat::Generic => "generic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Fail,
    Warn,
    Info,
    Debug,
    Trace,
    Unknown,
}

impl LogLevel {
    pub fn is_critical(&self) -> bool {
        matches!(self, LogLevel::Error | LogLevel::Fail)
    }
}

#[derive(Debug, Clone)]
pub struct LogCompressorConfig {
    pub max_repeated_lines: usize,
    pub max_warnings: usize,
    pub dedupe_warnings: bool,
    pub collapse_runtime_frames: bool,
    pub max_total_lines: usize,
}

impl Default for LogCompressorConfig {
    fn default() -> Self {
        Self {
            max_repeated_lines: 1,
            max_warnings: 5,
            dedupe_warnings: true,
            collapse_runtime_frames: true,
            max_total_lines: 500,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogCompressionResult {
    pub compressed: String,
    pub original_line_count: usize,
    pub compressed_line_count: usize,
    pub format_detected: LogFormat,
    pub stats: BTreeMap<String, usize>,
}

impl LogCompressionResult {
    pub fn tokens_saved_estimate(&self) -> usize {
        let orig_chars = self.original_line_count.saturating_mul(40);
        let comp_chars = self.compressed.len();
        if orig_chars > comp_chars {
            (orig_chars - comp_chars) / 4
        } else {
            0
        }
    }
}

pub fn detect_format(lines: &[&str]) -> LogFormat {
    let mut cargo_hits = 0;
    let mut pytest_hits = 0;
    let mut npm_hits = 0;
    let mut jest_hits = 0;
    let mut make_hits = 0;

    for line in lines.iter().take(80) {
        let t = line.trim();
        if t.starts_with("Compiling ") || t.starts_with("Finished ") || t.starts_with("error[E") || t.contains("cargo test") {
            cargo_hits += 1;
        }
        if t.starts_with("=== FAILURES ===") || t.contains("pytest") || t.contains("test session starts") {
            pytest_hits += 1;
        }
        if t.starts_with("npm ERR!") || t.starts_with("npm warn") {
            npm_hits += 1;
        }
        if t.contains("PASS ") || t.contains("FAIL ") || t.contains("Jest: ") {
            jest_hits += 1;
        }
        if t.starts_with("make[") || t.starts_with("make: ") {
            make_hits += 1;
        }
    }

    if cargo_hits >= 2 {
        LogFormat::Cargo
    } else if pytest_hits >= 2 {
        LogFormat::Pytest
    } else if jest_hits >= 2 {
        LogFormat::Jest
    } else if npm_hits >= 2 {
        LogFormat::Npm
    } else if make_hits >= 2 {
        LogFormat::Make
    } else {
        LogFormat::Generic
    }
}

pub fn classify_line(line: &str) -> (LogLevel, bool, bool) {
    let t = line.trim();
    let lower = t.to_lowercase();

    let is_summary = lower.starts_with("test result:")
        || lower.starts_with("failures:")
        || lower.starts_with("tests passed")
        || lower.contains("passed; 0 failed")
        || lower.starts_with("build succeeded")
        || lower.starts_with("build failed")
        || lower.starts_with("finished `");

    let is_stack_trace = t.starts_with("at ")
        || t.starts_with("File \"")
        || (t.starts_with("0x") && t.contains(" - "))
        || (t.chars().next().map_or(false, |c| c.is_ascii_digit()) && t.contains(": 0x"))
        || t.starts_with("stack backtrace:")
        || t.starts_with("Traceback (most recent call last):");

    let is_error = t.starts_with("error[E")
        || t.starts_with("error:")
        || lower.contains("fatal:")
        || lower.contains("fatal error")
        || lower.starts_with("panic:")
        || lower.contains("panicked at")
        || lower.starts_with("npm err!")
        || lower.starts_with("e   ")
        || lower.starts_with("assertionerror")
        || lower.contains(" uncaught exception ")
        || lower.starts_with("failed:");

    if is_error {
        return (LogLevel::Error, is_stack_trace, is_summary);
    }

    let is_fail = lower.starts_with("fail:") || lower.starts_with("failed:") || lower.starts_with("fail ");
    if is_fail {
        return (LogLevel::Fail, is_stack_trace, is_summary);
    }

    let is_warn = lower.starts_with("warning:") || lower.starts_with("warn:") || lower.starts_with("npm warn");
    if is_warn {
        return (LogLevel::Warn, is_stack_trace, is_summary);
    }

    let is_info = lower.starts_with("info:") || lower.starts_with("compiling ") || lower.starts_with("running ");
    if is_info {
        return (LogLevel::Info, is_stack_trace, is_summary);
    }

    let is_debug = lower.starts_with("debug:");
    if is_debug {
        return (LogLevel::Debug, is_stack_trace, is_summary);
    }

    let is_trace = lower.starts_with("trace:");
    if is_trace {
        return (LogLevel::Trace, is_stack_trace, is_summary);
    }

    (LogLevel::Unknown, is_stack_trace, is_summary)
}

fn normalize_for_dedupe(line: &str) -> String {
    let t = line.trim();
    if let Some((prefix, _)) = t.split_once(':') {
        if prefix.contains("error") || prefix.contains("warning") || prefix.contains("info") {
            return prefix.trim().to_string();
        }
    }
    t.to_string()
}

pub fn compress_logs(input: &str, config: &LogCompressorConfig) -> LogCompressionResult {
    let raw_lines: Vec<&str> = input.lines().collect();
    let original_line_count = raw_lines.len();

    if raw_lines.is_empty() {
        return LogCompressionResult {
            compressed: String::new(),
            original_line_count: 0,
            compressed_line_count: 0,
            format_detected: LogFormat::Generic,
            stats: BTreeMap::new(),
        };
    }

    let format_detected = detect_format(&raw_lines);
    let mut stats = BTreeMap::new();
    let mut out_lines = Vec::new();

    let mut i = 0;
    let mut warnings_seen = HashSet::new();
    let mut in_runtime_frames = false;
    let mut runtime_frames_count = 0usize;

    while i < raw_lines.len() {
        let cur = raw_lines[i];
        let (level, is_stack_trace, is_summary) = classify_line(cur);

        // Invariant: Fatal/Error/Panic lines survive byte-for-byte!
        if level.is_critical() {
            if in_runtime_frames {
                out_lines.push(format!("      [... {} runtime frames collapsed]", runtime_frames_count));
                *stats.entry("runtime_frames_collapsed".to_string()).or_insert(0) += runtime_frames_count;
                in_runtime_frames = false;
                runtime_frames_count = 0;
            }
            out_lines.push(cur.to_string());
            i += 1;
            continue;
        }

        // Summary lines always survive
        if is_summary {
            if in_runtime_frames {
                out_lines.push(format!("      [... {} runtime frames collapsed]", runtime_frames_count));
                *stats.entry("runtime_frames_collapsed".to_string()).or_insert(0) += runtime_frames_count;
                in_runtime_frames = false;
                runtime_frames_count = 0;
            }
            out_lines.push(cur.to_string());
            i += 1;
            continue;
        }

        // Collapse runtime frames in stack traces if requested
        if config.collapse_runtime_frames && is_stack_trace {
            let is_runtime = cur.contains("site-packages")
                || cur.contains("node_modules")
                || cur.contains("/rustc/")
                || cur.contains("core::ops::function")
                || cur.contains("<internal:")
                || cur.contains("std::sys::");

            if is_runtime {
                in_runtime_frames = true;
                runtime_frames_count += 1;
                i += 1;
                continue;
            } else if in_runtime_frames {
                out_lines.push(format!("      [... {} runtime frames collapsed]", runtime_frames_count));
                *stats.entry("runtime_frames_collapsed".to_string()).or_insert(0) += runtime_frames_count;
                in_runtime_frames = false;
                runtime_frames_count = 0;
            }
        } else if in_runtime_frames {
            out_lines.push(format!("      [... {} runtime frames collapsed]", runtime_frames_count));
            *stats.entry("runtime_frames_collapsed".to_string()).or_insert(0) += runtime_frames_count;
            in_runtime_frames = false;
            runtime_frames_count = 0;
        }

        // Warning deduplication
        if level == LogLevel::Warn && config.dedupe_warnings {
            let key = normalize_for_dedupe(cur);
            if warnings_seen.contains(&key) && warnings_seen.len() >= config.max_warnings {
                *stats.entry("warnings_deduped".to_string()).or_insert(0) += 1;
                i += 1;
                continue;
            }
            warnings_seen.insert(key);
        }

        // Check consecutive repeated lines
        let mut repeat_count = 0usize;
        let mut j = i + 1;
        while j < raw_lines.len() && raw_lines[j] == cur {
            repeat_count += 1;
            j += 1;
        }

        if repeat_count >= config.max_repeated_lines {
            out_lines.push(cur.to_string());
            out_lines.push(format!("      [... repeated {} times: \"{}\"]", repeat_count, cur.trim()));
            *stats.entry("repeated_lines_collapsed".to_string()).or_insert(0) += repeat_count;
            i = j;
            continue;
        }

        // Check multi-line 2-4 line periodic cycles
        let mut cycle_detected = false;
        for cycle_len in 2..=4 {
            if i + cycle_len * 2 <= raw_lines.len() {
                let candidate = &raw_lines[i..i + cycle_len];
                let mut cycles = 1;
                let mut k = i + cycle_len;
                while k + cycle_len <= raw_lines.len() && &raw_lines[k..k + cycle_len] == candidate {
                    cycles += 1;
                    k += cycle_len;
                }
                if cycles >= 3 {
                    for line in candidate {
                        out_lines.push(line.to_string());
                    }
                    out_lines.push(format!("      [... cycle of {} lines repeated {} times]", cycle_len, cycles - 1));
                    *stats.entry("cycle_repetitions_collapsed".to_string()).or_insert(0) += (cycles - 1) * cycle_len;
                    i = k;
                    cycle_detected = true;
                    break;
                }
            }
        }

        if cycle_detected {
            continue;
        }

        out_lines.push(cur.to_string());
        i += 1;
    }

    if in_runtime_frames {
        out_lines.push(format!("      [... {} runtime frames collapsed]", runtime_frames_count));
        *stats.entry("runtime_frames_collapsed".to_string()).or_insert(0) += runtime_frames_count;
    }

    // Apply global cap if requested, keeping head and tail
    if out_lines.len() > config.max_total_lines && config.max_total_lines > 20 {
        let keep_head = config.max_total_lines / 2;
        let keep_tail = config.max_total_lines - keep_head - 1;
        let omitted = out_lines.len() - keep_head - keep_tail;

        let mut clamped = Vec::with_capacity(config.max_total_lines);
        clamped.extend_from_slice(&out_lines[..keep_head]);
        clamped.push(format!("      [... {} lines omitted by headroom log cap ...]", omitted));
        clamped.extend_from_slice(&out_lines[out_lines.len() - keep_tail..]);
        out_lines = clamped;
        *stats.entry("lines_clamped_by_cap".to_string()).or_insert(0) += omitted;
    }

    let compressed = out_lines.join("\n");
    let compressed_line_count = out_lines.len();

    LogCompressionResult {
        compressed,
        original_line_count,
        compressed_line_count,
        format_detected,
        stats,
    }
}

pub fn deduplicate_logs(input: &str) -> String {
    let config = LogCompressorConfig::default();
    compress_logs(input, &config).compressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fatal_and_panic_lines_survive_verbatim() {
        let input = "Compiling app v0.1.0\nCompiling app v0.1.0\nCompiling app v0.1.0\nerror[E0425]: cannot find value `foo` in this scope\npanicked at 'explicit panic', src/main.rs:42:5\ntest result: FAILED. 1 failed; 0 passed";
        let res = compress_logs(input, &LogCompressorConfig::default());
        assert!(res.compressed.contains("error[E0425]: cannot find value `foo` in this scope"));
        assert!(res.compressed.contains("panicked at 'explicit panic', src/main.rs:42:5"));
        assert!(res.compressed.contains("test result: FAILED"));
        assert!(res.compressed.contains("[... repeated 2 times"));
    }

    #[test]
    fn test_collapse_runtime_stack_frames() {
        let input = "Traceback (most recent call last):\n  File \"/usr/lib/python3.11/site-packages/urllib3/connection.py\", line 10, in connect\n  File \"/usr/lib/python3.11/site-packages/urllib3/pool.py\", line 20, in get\n  File \"/app/main.py\", line 88, in process\nAssertionError: expected status 200";
        let res = compress_logs(input, &LogCompressorConfig::default());
        assert!(res.compressed.contains("runtime frames collapsed"));
        assert!(res.compressed.contains("/app/main.py"));
        assert!(res.compressed.contains("AssertionError: expected status 200"));
    }

    #[test]
    fn test_multi_line_cycle_collapse() {
        let cycle = "Connecting to broker...\nConnection refused (retrying in 1s)...\n";
        let input = cycle.repeat(5);
        let res = compress_logs(&input, &LogCompressorConfig::default());
        assert!(res.compressed.contains("Connecting to broker..."));
        assert!(res.compressed.contains("cycle of 2 lines repeated"));
        assert!(res.compressed_line_count < res.original_line_count);
    }
}
