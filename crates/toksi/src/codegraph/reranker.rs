// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/reranker.rs
// created: 2026-11-21

//! CodeGraph cross-encoder / lexical-vector reranker (`NeMo-Retriever`,
//! `text-embeddings-inference` `/rerank`, `optimum-onnx`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeRerankCandidate {
    pub symbol_id: String,
    pub snippet: String,
    pub base_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeRerankResult {
    pub symbol_id: String,
    pub final_score: f32,
}

/// Reranks code symbol candidates deterministically using query term overlap + base score.
pub fn rerank_code_candidates(
    query: &str,
    candidates: &[CodeRerankCandidate],
) -> Vec<CodeRerankResult> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|t| t.to_ascii_lowercase())
        .filter(|t| !t.is_empty())
        .collect();

    let mut out: Vec<CodeRerankResult> = candidates
        .iter()
        .map(|c| {
            let lower = c.snippet.to_ascii_lowercase();
            let hits = terms.iter().filter(|t| lower.contains(t.as_str())).count();
            let lexical_boost = if terms.is_empty() {
                0.0
            } else {
                (hits as f32 / terms.len() as f32) * 0.5
            };
            CodeRerankResult {
                symbol_id: c.symbol_id.clone(),
                final_score: c.base_score + lexical_boost,
            }
        })
        .collect();

    out.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}
