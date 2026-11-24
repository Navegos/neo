// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/pmat.rs
// created: 2026-11-24

//! Pure-Rust `PMAT` code-quality, cyclomatic complexity, and nesting scorer.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PmatMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub function_count: usize,
    pub cyclomatic_complexity: usize,
    pub max_nesting_depth: usize,
    pub score: f64,
    pub grade: char,
}

/// Computes deterministic complexity & maintainability score (`0..=100`, `A..=F`) on source text.
pub fn score_source_pmat(source: &str) -> PmatMetrics {
    let mut total_lines = 0usize;
    let mut code_lines = 0usize;
    let mut comment_lines = 0usize;
    let mut function_count = 0usize;
    let mut cyclomatic_complexity = 1usize;
    let mut current_depth = 0usize;
    let mut max_nesting_depth = 0usize;

    for raw in source.lines() {
        total_lines += 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("//") || line.starts_with('#') {
            comment_lines += 1;
            continue;
        }
        code_lines += 1;
        if line.contains("fn ") || line.contains("def ") || line.contains("function ") {
            function_count += 1;
        }
        for kw in ["if ", "else if ", "match ", "while ", "for ", "elif ", "&&", "||"] {
            if line.contains(kw) {
                cyclomatic_complexity += 1;
            }
        }
        for ch in line.chars() {
            if ch == '{' {
                current_depth += 1;
                max_nesting_depth = max_nesting_depth.max(current_depth);
            } else if ch == '}' {
                current_depth = current_depth.saturating_sub(1);
            }
        }
    }

    let branch_penalty = (cyclomatic_complexity.saturating_sub(1) as f64) * 3.0;
    let nesting_penalty = (max_nesting_depth.saturating_sub(3) as f64) * 6.0;
    let score = (100.0 - branch_penalty - nesting_penalty).clamp(0.0, 100.0);
    let grade = if score >= 90.0 {
        'A'
    } else if score >= 80.0 {
        'B'
    } else if score >= 70.0 {
        'C'
    } else if score >= 60.0 {
        'D'
    } else {
        'F'
    };

    PmatMetrics {
        total_lines,
        code_lines,
        comment_lines,
        function_count,
        cyclomatic_complexity,
        max_nesting_depth,
        score,
        grade,
    }
}
