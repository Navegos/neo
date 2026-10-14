// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/separators.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use crate::util::instructions::{section_header_for_owner, TOKLESS_OWNERS};
use crate::util::paths::{read_file_safe, write_file_atomic};
use super::tokless_block::{instruction_path, kilo_instructions_path};

/// Re-reads the agent's instruction file and guarantees a blank line (`\n\n`)
/// immediately precedes every `#` or `##` heading after the first line.
pub fn ensure_instruction_separators(agent: &str) {
    let path = if agent == "kilo" {
        kilo_instructions_path()
    } else {
        instruction_path(agent)
    };
    if path.is_empty() {
        return;
    }
    let raw = match read_file_safe(&path) {
        Some(r) if !r.is_empty() => r,
        _ => return,
    };
    let fixed = normalize_separators(&raw);
    if fixed != raw {
        let _ = write_file_atomic(&path, &fixed);
    }
}

/// Ensures every top-level Tokless section heading (`# RTK`, `# Agent Instructions`,
/// `## Principles`, `## Response Style (caveman)`, `## Build Discipline (ponytail)`,
/// `## Code Index (codegraph)`, `## Context Tools (context-mode)`, `## Project Memory (projectmem)`)
/// is preceded by a blank line (`\n\n`) unless it is at the very start of the file.
pub fn normalize_separators(raw: &str) -> String {
    let mut s = raw.replace("\r\n", "\n");
    let mut headers: Vec<&str> = vec!["# Agent Instructions"];
    for owner in TOKLESS_OWNERS {
        let h = section_header_for_owner(owner);
        if !h.is_empty() {
            headers.push(h);
        }
    }
    for h in headers {
        if h.is_empty() {
            continue;
        }
        let mut search_from = 0;
        while search_from < s.len() {
            let rel = match s[search_from..].find(h) {
                Some(idx) => idx,
                None => break,
            };
            let pos = search_from + rel;
            let at_line_start = pos == 0 || s.as_bytes()[pos - 1] == b'\n';
            let after_end = pos + h.len();
            let at_line_end = after_end == s.len()
                || s.as_bytes()[after_end] == b'\n'
                || s.as_bytes()[after_end] == b'\r';
            if !at_line_start || !at_line_end {
                search_from = pos + h.len();
                continue;
            }
            if pos == 0 {
                search_from = pos + h.len();
                continue;
            }
            let mut nl_start = pos;
            while nl_start > 0
                && (s.as_bytes()[nl_start - 1] == b'\n'
                    || s.as_bytes()[nl_start - 1] == b' '
                    || s.as_bytes()[nl_start - 1] == b'\t')
            {
                nl_start -= 1;
            }
            if nl_start == 0 {
                s = format!("{}{}", &s[..nl_start], &s[pos..]);
                search_from = h.len();
                continue;
            }
            let newlines = count_newlines(&s[nl_start..pos]);
            if newlines != 2 {
                s = format!("{}\n\n{}", &s[..nl_start], &s[pos..]);
                search_from = nl_start + 2 + h.len();
                continue;
            }
            search_from = pos + h.len();
        }
    }
    s
}

fn count_newlines(sub: &str) -> usize {
    sub.bytes().filter(|&b| b == b'\n').count()
}
