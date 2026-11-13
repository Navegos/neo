// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/search_compressor.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Search-results compressor ported from `headroom-core::transforms::search_compressor`.
//! Handles Windows drive prefixes (`C:\...`) and hyphenated filenames (`pre-commit.yaml`).

use std::collections::{BTreeMap, BTreeSet};

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

        let mut by_file: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();
        for line in &lines {
            if let Some((file, line_no, content)) = parse_search_line(line) {
                by_file.entry(file).or_default().push((line_no, content));
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

        let mut ranked_files: Vec<(f64, String, Vec<(usize, String)>)> = by_file
            .into_iter()
            .map(|(file, matches)| {
                let total_score: f64 = matches
                    .iter()
                    .map(|(_, text)| score_match(text, &query_terms))
                    .sum();
                (total_score, file, matches)
            })
            .collect();
        ranked_files.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let dropped_files = ranked_files.len().saturating_sub(self.config.max_files);
        ranked_files.truncate(self.config.max_files);

        let mut out = Vec::new();
        for (_, file, matches) in ranked_files {
            if matches.len() <= self.config.max_matches_per_file {
                for (ln, txt) in matches {
                    out.push(format!("{}:{}:{}", file, ln, txt));
                }
                continue;
            }

            let mut scored: Vec<(usize, f64, usize, String)> = matches
                .into_iter()
                .enumerate()
                .map(|(idx, (ln, txt))| (idx, score_match(&txt, &query_terms), ln, txt))
                .collect();
            let last_idx = scored.len().saturating_sub(1);
            let mut chosen = BTreeSet::new();
            chosen.insert(0);
            chosen.insert(last_idx);

            let mut by_score = scored.clone();
            by_score.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (idx, _, _, _) in by_score {
                if chosen.len() >= self.config.max_matches_per_file {
                    break;
                }
                chosen.insert(idx);
            }

            scored.retain(|(idx, _, _, _)| chosen.contains(idx));
            for (_, _, ln, txt) in &scored {
                out.push(format!("{}:{}:{}", file, ln, txt));
            }
            let extra = (last_idx + 1).saturating_sub(scored.len());
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

/// Parses `file:line:content` or `file-line-content` while preserving `C:\...` Windows drive letters
/// and filenames containing hyphens.
pub fn parse_search_line(line: &str) -> Option<(String, usize, String)> {
    let bytes = line.as_bytes();
    let start = if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        3
    } else {
        0
    };

    for idx in start..bytes.len() {
        let sep = bytes[idx];
        if sep != b':' && sep != b'-' {
            continue;
        }
        let mut end = idx + 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > idx + 1 && end < bytes.len() && bytes[end] == sep {
            if let Ok(line_no) = line[idx + 1..end].parse::<usize>() {
                let file = line[..idx].to_string();
                let content = line[end + 1..].to_string();
                if !file.is_empty() {
                    return Some((file, line_no, content));
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
    }

    #[test]
    fn test_search_compressor_ccr() {
        let mut lines = Vec::new();
        for i in 1..=35 {
            lines.push(format!(r"C:\repo\app\mod-1.rs:{}:let x_{} = {};", i, i, i));
        }
        let raw = lines.join("\n");
        let store = InMemoryCcrStore::default();
        let compressed = SearchCompressor::default()
            .compress(&raw, "main", Some(&store))
            .expect("compressed");
        assert!(compressed.contains("<<ccr:"));
        assert!(compressed.len() < raw.len());
    }
}
