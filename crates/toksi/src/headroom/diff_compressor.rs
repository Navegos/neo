// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/diff_compressor.rs
// created: 2026-11-13
// lastModified: 2026-11-13

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
        input.contains("diff --git ") || (input.contains("\n@@ ") && input.contains("\n+++ "))
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
            } else if line.starts_with("@@ ") {
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

fn reduce_hunk_context(hunk: &[String], max_ctx: usize) -> Vec<String> {
    if hunk.is_empty() {
        return Vec::new();
    }
    let change_positions: Vec<usize> = hunk
        .iter()
        .enumerate()
        .skip(1)
        .filter_map(|(idx, line)| {
            if line.starts_with('+') || line.starts_with('-') {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    if change_positions.is_empty() {
        return hunk.to_vec();
    }

    let mut out = Vec::with_capacity(hunk.len());
    out.push(hunk[0].clone()); // @@ header

    let mut omitted = 0usize;
    for (idx, line) in hunk.iter().enumerate().skip(1) {
        if line.starts_with('+') || line.starts_with('-') {
            if omitted > 0 {
                out.push(format!("  [... {} unchanged context lines]", omitted));
                omitted = 0;
            }
            out.push(line.clone());
            continue;
        }
        let near_change = change_positions
            .iter()
            .any(|pos| pos.abs_diff(idx) <= max_ctx);
        if near_change {
            if omitted > 0 {
                out.push(format!("  [... {} unchanged context lines]", omitted));
                omitted = 0;
            }
            out.push(line.clone());
        } else {
            omitted += 1;
        }
    }
    if omitted > 0 {
        out.push(format!("  [... {} unchanged context lines]", omitted));
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
            "@@ -1,40 +1,40 @@".to_string(),
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
        let compressed = DiffCompressor::default()
            .compress(&raw, Some(&store))
            .expect("compressed");
        assert!(compressed.contains("-let old = 1;"));
        assert!(compressed.contains("+let new = 2;"));
        assert!(compressed.contains("<<ccr:"));
        assert!(compressed.len() < raw.len());
    }
}
