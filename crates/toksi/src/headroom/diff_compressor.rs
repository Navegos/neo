// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/diff_compressor.rs
// created: 2026-11-13
// lastModified: 2026-11-16

//! Unified-diff compressor ported from `headroom-core::transforms::diff_compressor`.
//! Trims unchanged context lines around `+`/`-` hunks while keeping every edit line intact.

use super::ccr::{compute_key, marker_for, CcrStore};

#[derive(Debug, Clone)]
pub struct DiffCompressorConfig {
    pub max_context_lines: usize,
    pub max_hunks_per_file: usize,
    pub max_files: usize,
    pub min_lines_for_ccr: usize,
}

impl Default for DiffCompressorConfig {
    fn default() -> Self {
        Self {
            max_context_lines: 2,
            max_hunks_per_file: 10,
            max_files: 20,
            min_lines_for_ccr: 30,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiffCompressor {
    config: DiffCompressorConfig,
}

impl DiffCompressor {
    pub fn new(config: DiffCompressorConfig) -> Self {
        Self { config }
    }

    pub fn looks_like_unified_diff(input: &str) -> bool {
        input.contains("diff --git ")
            || ((input.contains("\n@@ ") || input.contains("\n@@@ ")) && input.contains("\n+++ "))
    }

    pub fn compress(&self, input: &str, store: Option<&dyn CcrStore>) -> Option<String> {
        let lines: Vec<&str> = input.lines().collect();
        if lines.len() < self.config.min_lines_for_ccr || !Self::looks_like_unified_diff(input) {
            return None;
        }

        let mut files: Vec<(Vec<String>, Vec<Vec<String>>)> = Vec::new();
        let mut cur_headers: Vec<String> = Vec::new();
        let mut cur_hunks: Vec<Vec<String>> = Vec::new();
        let mut cur_hunk: Vec<String> = Vec::new();
        let mut in_hunk = false;

        for line in &lines {
            if line.starts_with("diff --git ") {
                if !cur_hunk.is_empty() {
                    cur_hunks.push(std::mem::take(&mut cur_hunk));
                }
                if !cur_headers.is_empty() || !cur_hunks.is_empty() {
                    files.push((std::mem::take(&mut cur_headers), std::mem::take(&mut cur_hunks)));
                }
                cur_headers.push((*line).to_string());
                in_hunk = false;
            } else if line.starts_with("@@ ") || line.starts_with("@@@ ") {
                if !cur_hunk.is_empty() {
                    cur_hunks.push(std::mem::take(&mut cur_hunk));
                }
                cur_hunk.push((*line).to_string());
                in_hunk = true;
            } else if in_hunk {
                cur_hunk.push((*line).to_string());
            } else {
                cur_headers.push((*line).to_string());
            }
        }
        if !cur_hunk.is_empty() {
            cur_hunks.push(cur_hunk);
        }
        if !cur_headers.is_empty() || !cur_hunks.is_empty() {
            files.push((cur_headers, cur_hunks));
        }

        if files.is_empty() {
            return None;
        }

        let total_files = files.len();
        files.truncate(self.config.max_files);

        let mut out = Vec::new();
        for (headers, hunks) in files {
            out.extend(headers);
            let total_hunks = hunks.len();
            for hunk in hunks.into_iter().take(self.config.max_hunks_per_file) {
                out.extend(reduce_hunk_context(&hunk, self.config.max_context_lines));
            }
            if total_hunks > self.config.max_hunks_per_file {
                out.push(format!(
                    "... [{} more hunks omitted]",
                    total_hunks - self.config.max_hunks_per_file
                ));
            }
        }
        if total_files > self.config.max_files {
            out.push(format!("... [{} more files omitted]", total_files - self.config.max_files));
        }

        if let Some(ccr) = store {
            if out.len() * 10 < lines.len() * 8 {
                let key = compute_key(input.as_bytes());
                ccr.put(&key, input);
                out.push(format!("[Full diff stored: headroom_retrieve({})]", marker_for(&key)));
            }
        }

        let compressed = out.join("\n");
        if compressed.len() < input.len() {
            Some(compressed)
        } else {
            None
        }
    }
}

fn hunk_parents(header: &str) -> usize {
    header
        .bytes()
        .take_while(|&b| b == b'@')
        .count()
        .saturating_sub(1)
        .max(1)
}

fn line_change_kind(line: &str, parents: usize) -> (bool, bool) {
    if parents > 1 {
        let cols = &line.as_bytes()[..parents.min(line.len())];
        (cols.contains(&b'+'), cols.contains(&b'-'))
    } else {
        (line.starts_with('+'), line.starts_with('-'))
    }
}

struct HunkRanges {
    old_next: Vec<usize>,
    old_len: Vec<usize>,
    new_next: usize,
    new_len: usize,
    heading: String,
}

fn parse_hunk_ranges(header: &str, parents: usize) -> Option<HunkRanges> {
    let marker = "@".repeat(parents + 1);
    let rest = header.strip_prefix(&marker)?;
    let end = rest.find(&format!(" {}", marker))?;
    let heading = rest[end + 1 + marker.len()..].to_string();
    let ranges: Vec<&str> = rest[..end].split_whitespace().collect();
    if ranges.len() != parents + 1 {
        return None;
    }
    let parse = |range: &str, sign: char| -> Option<(usize, usize)> {
        let body = range.strip_prefix(sign)?;
        let (start, count) = match body.split_once(',') {
            Some((s, c)) => (s.parse::<usize>().ok()?, c.parse::<usize>().ok()?),
            None => (body.parse::<usize>().ok()?, 1),
        };
        Some((
            if count == 0 && parents == 1 {
                start.checked_add(1)?
            } else {
                start
            },
            count,
        ))
    };
    let old = ranges[..parents]
        .iter()
        .map(|r| parse(r, '-'))
        .collect::<Option<Vec<_>>>()?;
    let (new_next, new_len) = parse(ranges[parents], '+')?;
    Some(HunkRanges {
        old_next: old.iter().map(|&(n, _)| n).collect(),
        old_len: old.iter().map(|&(_, c)| c).collect(),
        new_next,
        new_len,
        heading,
    })
}

fn advance_line(line: &str, parents: usize, old_next: &mut [usize], new_next: &mut usize) {
    if line.starts_with('\\') {
        return;
    }
    let cols = line.as_bytes();
    let col = |j: usize| cols.get(j).copied().unwrap_or(b' ');
    let deleted = (0..parents).any(|j| col(j) == b'-');
    for (j, next) in old_next.iter_mut().enumerate() {
        let in_parent = if deleted {
            col(j) == b'-'
        } else {
            col(j) != b'+'
        };
        if in_parent {
            *next = next.saturating_add(1);
        }
    }
    if !deleted {
        *new_next = new_next.saturating_add(1);
    }
}

fn hunk_body_len(body_lines: &[String], parents: usize, ranges: &HunkRanges) -> usize {
    let (mut old, mut new) = (ranges.old_next.clone(), ranges.new_next);
    let done = |old: &[usize], new: usize| {
        new.saturating_sub(ranges.new_next) >= ranges.new_len
            && old
                .iter()
                .zip(&ranges.old_next)
                .zip(&ranges.old_len)
                .all(|((o, start), len)| o.saturating_sub(*start) >= *len)
    };
    let mut n = 0usize;
    while n < body_lines.len() && !done(&old, new) {
        advance_line(&body_lines[n], parents, &mut old, &mut new);
        n += 1;
    }
    n
}

fn format_range(sign: char, next_line: usize, count: usize, combined: bool) -> String {
    if combined {
        return format!(" {}{},{}", sign, next_line, count);
    }
    let start = if count == 0 {
        next_line.saturating_sub(1)
    } else {
        next_line
    };
    if count == 1 {
        format!(" {}{}", sign, start)
    } else {
        format!(" {}{},{}", sign, start, count)
    }
}

pub fn reduce_hunk_context(hunk: &[String], max_ctx: usize) -> Vec<String> {
    if hunk.is_empty() {
        return Vec::new();
    }
    let header = &hunk[0];
    let body = &hunk[1..];
    if body.is_empty() {
        return hunk.to_vec();
    }

    let parents = hunk_parents(header);
    let ranges = parse_hunk_ranges(header, parents);
    let body_len = ranges
        .as_ref()
        .map_or(body.len(), |r| hunk_body_len(body, parents, r));

    let signature = (body_len..body.len()).find(|&i| body[i] == "-- ");
    let sig_end = signature.unwrap_or(body.len());
    let understated = body[body_len..sig_end].iter().any(|l| {
        let (added, deleted) = line_change_kind(l, parents);
        added || deleted
    });
    if understated {
        return hunk.to_vec();
    }

    let change_positions: Vec<usize> = body
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            let (added, deleted) = line_change_kind(line, parents);
            if idx < body_len && (added || deleted) {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    if change_positions.is_empty() {
        return hunk.to_vec();
    }

    let mut keep = std::collections::BTreeSet::new();
    for &pos in &change_positions {
        keep.insert(pos);
        for i in pos.saturating_sub(max_ctx)..pos {
            keep.insert(i);
        }
        let hi = (pos + max_ctx + 1).min(body_len);
        for i in (pos + 1)..hi {
            keep.insert(i);
        }
    }
    for (i, line) in body.iter().enumerate() {
        if i < body_len && line.starts_with('\\') {
            keep.insert(i);
        }
    }
    let last_nonempty = body.iter().rposition(|l| !l.is_empty());
    if let (Some(sig_idx), Some(last_idx)) = (signature, last_nonempty) {
        if sig_idx <= last_idx {
            for i in sig_idx..=last_idx {
                keep.insert(i);
            }
        }
    }

    // If nothing was dropped from the body, return the original hunk byte-for-byte.
    let kept_in_body = keep.range(..body_len).count();
    if kept_in_body == body_len {
        return hunk.to_vec();
    }

    let Some(ranges) = ranges else {
        let mut out = Vec::with_capacity(1 + keep.len());
        out.push(header.clone());
        for &i in &keep {
            out.push(body[i].clone());
        }
        return out;
    };

    // Split kept body indices into contiguous sub-hunks and renumber each header.
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for &i in &keep {
        match runs.last_mut() {
            Some(run) if i >= body_len || run.last().copied() == Some(i.saturating_sub(1)) => {
                run.push(i);
            }
            _ => runs.push(vec![i]),
        }
    }

    let marker = "@".repeat(parents + 1);
    let mut old_next = ranges.old_next;
    let mut new_next = ranges.new_next;
    let mut out = Vec::with_capacity(body.len() + runs.len());
    let mut pos = 0usize;

    for run in runs {
        while pos < run[0] && pos < body_len {
            advance_line(&body[pos], parents, &mut old_next, &mut new_next);
            pos += 1;
        }
        let old_start = old_next.clone();
        let new_start = new_next;
        for &i in run.iter().filter(|&&i| i < body_len) {
            advance_line(&body[i], parents, &mut old_next, &mut new_next);
        }
        if let Some(&last_i) = run.last() {
            pos = last_i + 1;
        }

        let mut sub_header = marker.clone();
        for (start, next) in old_start.iter().zip(&old_next) {
            sub_header.push_str(&format_range('-', *start, next.saturating_sub(*start), parents > 1));
        }
        sub_header.push_str(&format_range(
            '+',
            new_start,
            new_next.saturating_sub(new_start),
            parents > 1,
        ));
        sub_header.push(' ');
        sub_header.push_str(&marker);
        sub_header.push_str(&ranges.heading);

        out.push(sub_header);
        for &i in &run {
            out.push(body[i].clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headroom::ccr::InMemoryCcrStore;

    #[test]
    fn test_diff_compressor_preserves_changes_and_trims_context() {
        let mut lines = vec![
            "diff --git a/src/lib.rs b/src/lib.rs".to_string(),
            "index 111..222 100644".to_string(),
            "--- a/src/lib.rs".to_string(),
            "+++ b/src/lib.rs".to_string(),
            "@@ -1,42 +1,42 @@".to_string(),
        ];
        for i in 0..20 {
            lines.push(format!(" let unchanged_{} = {};", i, i));
        }
        lines.push("-let old = 1;".to_string());
        lines.push("+let new = 2;".to_string());
        for i in 20..40 {
            lines.push(format!(" let unchanged_{} = {};", i, i));
        }
        let raw = lines.join("\n");
        let store = InMemoryCcrStore::default();
        let compressed = DiffCompressor::default().compress(&raw, Some(&store));
        assert!(compressed.is_some());
        if let Some(c) = compressed {
            assert!(c.contains("-let old = 1;"));
            assert!(c.contains("+let new = 2;"));
            assert!(c.contains("<<ccr:"));
            assert!(c.len() < raw.len());
            // Renumbered header starts around line 19 with 5 lines (2 before + 1/-1 + 2 after)
            assert!(c.contains("@@ -19,5 +19,5 @@"));
        }
    }

    #[test]
    fn test_diff_hunk_splits_and_renumbers_across_gap() {
        let mut hunk = vec!["@@ -7,14 +7,14 @@ fn demo()".to_string()];
        hunk.push(" line 7".to_string());
        hunk.push(" line 8".to_string());
        hunk.push(" line 9".to_string());
        hunk.push("-line 10".to_string());
        hunk.push("+line 10 changed".to_string());
        hunk.push(" line 11".to_string());
        hunk.push(" line 12".to_string());
        hunk.push(" line 13".to_string());
        hunk.push(" line 14".to_string());
        hunk.push(" line 15".to_string());
        hunk.push(" line 16".to_string());
        hunk.push("-line 17".to_string());
        hunk.push("+line 17 changed".to_string());
        hunk.push(" line 18".to_string());
        hunk.push(" line 19".to_string());
        hunk.push(" line 20".to_string());

        let out = reduce_hunk_context(&hunk, 2);
        let headers: Vec<&str> = out.iter().filter(|l| l.starts_with("@@ ")).map(|s| s.as_str()).collect();
        assert_eq!(headers, vec!["@@ -8,5 +8,5 @@ fn demo()", "@@ -15,5 +15,5 @@ fn demo()"]);
    }
}
