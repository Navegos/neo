// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/search_compressor.rs
// created: 2026-11-13
// lastModified: 2026-11-16

//! Search-results compressor ported from `headroom-core::transforms::search_compressor`.
//! Handles Windows drive prefixes (`C:\...`) and hyphenated filenames (`pre-commit.yaml`).

use std::collections::{BTreeSet, HashMap};

use super::ccr::{compute_key, marker_for, CcrStore};

#[derive(Debug, Clone)]
pub struct SearchCompressorConfig {
    pub max_files: usize,
    pub max_matches_per_file: usize,
    pub min_lines_for_ccr: usize,
}

impl Default for SearchCompressorConfig {
    fn default() -> Self {
        Self {
            max_files: 20,
            max_matches_per_file: 5,
            min_lines_for_ccr: 16,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchCompressor {
    config: SearchCompressorConfig,
}

impl SearchCompressor {
    pub fn new(config: SearchCompressorConfig) -> Self {
        Self { config }
    }

    pub fn looks_like_search_output(input: &str) -> bool {
        let mut hits = 0usize;
        for line in input.lines().take(25) {
            if parse_search_line(line).is_some() {
                hits += 1;
                if hits >= 3 {
                    return true;
                }
            }
        }
        false
    }

    pub fn compress(&self, input: &str, query: &str, store: Option<&dyn CcrStore>) -> Option<String> {
        let lines: Vec<&str> = input.lines().collect();
        if lines.len() < self.config.min_lines_for_ccr {
            return None;
        }

        // Preserve each file's first-seen order and every match's input index + verbatim raw line.
        let mut by_file: HashMap<String, (usize, Vec<(usize, usize, String, String)>)> =
            HashMap::new();
        for (input_idx, line) in lines.iter().enumerate() {
            if let Some((file, line_no, content, raw_line)) = parse_search_line_verbatim(line) {
                let next_order = by_file.len();
                let entry = by_file
                    .entry(file)
                    .or_insert_with(|| (next_order, Vec::new()));
                entry.1.push((input_idx, line_no, content, raw_line));
            }
        }
        if by_file.is_empty() {
            return None;
        }

        let query_terms: Vec<String> = query
            .to_ascii_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect();

        let mut ranked_files: Vec<(f64, usize, String, Vec<(usize, usize, String, String)>)> =
            by_file
                .into_iter()
                .map(|(file, (first_order, matches))| {
                    let total_score: f64 = matches
                        .iter()
                        .map(|(_, _, text, _)| score_match(text, &query_terms))
                        .sum();
                    (total_score, first_order, file, matches)
                })
                .collect();
        ranked_files.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.cmp(&b.1))
        });

        let dropped_files = ranked_files.len().saturating_sub(self.config.max_files);
        ranked_files.truncate(self.config.max_files);
        ranked_files.sort_by_key(|f| f.1);

        let mut out = Vec::new();
        for (_, _, file, matches) in ranked_files {
            if matches.len() <= self.config.max_matches_per_file {
                for (_, _, _, raw_line) in matches {
                    out.push(raw_line);
                }
                continue;
            }

            let mut scored: Vec<(usize, f64, usize, String, String)> = matches
                .into_iter()
                .enumerate()
                .map(|(pos, (_input_idx, ln, txt, raw_line))| {
                    (pos, score_match(&txt, &query_terms), ln, txt, raw_line)
                })
                .collect();
            let last_pos = scored.len().saturating_sub(1);
            let mut chosen = BTreeSet::new();
            let mut seen_keys: BTreeSet<(usize, String)> = BTreeSet::new();

            let mut try_pick = |pos: usize| {
                let key = (scored[pos].2, scored[pos].3.clone());
                if seen_keys.insert(key) {
                    chosen.insert(pos);
                }
            };
            try_pick(0);
            try_pick(last_pos);

            let mut by_score: Vec<(usize, f64, usize)> = scored
                .iter()
                .map(|(pos, score, ln, _, _)| (*pos, *score, *ln))
                .collect();
            by_score.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.2.cmp(&b.2))
                    .then_with(|| a.0.cmp(&b.0))
            });
            for (pos, _, _) in by_score {
                if chosen.len() >= self.config.max_matches_per_file {
                    break;
                }
                let key = (scored[pos].2, scored[pos].3.clone());
                if seen_keys.insert(key) {
                    chosen.insert(pos);
                }
            }

            // Retain in original input order and emit exact verbatim line bytes.
            scored.retain(|(pos, _, _, _, _)| chosen.contains(pos));
            for (_, _, _, _, raw_line) in &scored {
                out.push(raw_line.clone());
            }
            let extra = (last_pos + 1).saturating_sub(scored.len());
            if extra > 0 {
                out.push(format!("[... and {} more matches in {}]", extra, file));
            }
        }

        if dropped_files > 0 {
            out.push(format!("[... and {} more files]", dropped_files));
        }

        if let Some(ccr) = store {
            if out.len() * 10 < lines.len() * 8 {
                let key = compute_key(input.as_bytes());
                ccr.put(&key, input);
                out.push(format!("[Full search results: headroom_retrieve({})]", marker_for(&key)));
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

fn score_match(text: &str, query_terms: &[String]) -> f64 {
    let low = text.to_ascii_lowercase();
    let mut score = 0.2;
    if low.contains("error") || low.contains("fatal") || low.contains("panic") || low.contains("fn ") || low.contains("struct ") {
        score += 0.4;
    }
    for term in query_terms {
        if low.contains(term) {
            score += 0.3;
        }
    }
    score
}

/// Parses `file:line:content` or `file-line-content` while preserving `C:\...` Windows drive letters,
/// hyphenated filenames, leading indentation, and rejecting zero-led timestamps like `12:00:01`.
pub fn parse_search_line(line: &str) -> Option<(String, usize, String)> {
    let (file, line_no, content, _) = parse_search_line_verbatim(line)?;
    Some((file, line_no, content))
}

/// Parses a search result or grep-context row and also returns the exact original line verbatim.
pub fn parse_search_line_verbatim(line: &str) -> Option<(String, usize, String, String)> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let start = if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        3
    } else {
        0
    };

    for idx in start..bytes.len() {
        let sep = bytes[idx];
        if sep != b':' && sep != b'-' {
            continue;
        }
        // Real line numbers are positive and never start with '0' (rejects `12:00:01`, `2026-05-03`).
        if idx + 1 >= bytes.len() || !bytes[idx + 1].is_ascii_digit() || bytes[idx + 1] == b'0' {
            continue;
        }
        let mut end = idx + 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end < bytes.len() && bytes[end] == sep {
            if let Ok(line_no) = trimmed[idx + 1..end].parse::<usize>() {
                let file = trimmed[..idx].to_string();
                let content = trimmed[end + 1..].to_string();
                if !file.is_empty() && line_no > 0 {
                    return Some((file, line_no, content, line.to_string()));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headroom::ccr::InMemoryCcrStore;

    #[test]
    fn test_windows_drive_and_hyphen_filename_parsing() {
        let win = parse_search_line(r"C:\repo\src\main.rs:42:fn main() {}");
        assert_eq!(win, Some((r"C:\repo\src\main.rs".into(), 42, "fn main() {}".into())));

        let hyp = parse_search_line("pre-commit-config.yaml-17-  - repo: local");
        assert_eq!(hyp, Some(("pre-commit-config.yaml".into(), 17, "  - repo: local".into())));

        // Zero-led timestamps and dates must not misparse as line numbers
        assert_eq!(parse_search_line("12:00:01 info starting server"), None);
        assert_eq!(parse_search_line("2026-05-03 12:05:00 ready"), None);
    }

    #[test]
    fn test_search_compressor_ccr_and_verbatim_context_lines() {
        let mut lines = Vec::new();
        lines.push("  src/app.rs-1-let init = 0;".to_string());
        for i in 2..=35 {
            if i == 18 {
                lines.push(format!("  src/app.rs:{}:error: fatal failure", i));
            } else {
                lines.push(format!("  src/app.rs-{}-let x_{} = {};", i, i, i));
            }
        }
        let raw = lines.join("\n");
        let store = InMemoryCcrStore::default();
        let compressed = SearchCompressor::default().compress(&raw, "error", Some(&store));
        assert!(compressed.is_some());
        if let Some(c) = compressed {
            assert!(c.contains("<<ccr:"));
            assert!(c.len() < raw.len());
            // Kept context lines retain their original indent and `-N-` marker
            assert!(c.contains("  src/app.rs-1-let init = 0;"));
            assert!(c.contains("  src/app.rs:18:error: fatal failure"));
        }
    }
}
