// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/retriever.rs
// created: 2026-11-21

//! `NeMo-Retriever`-style multi-stage dense + sparse + cross-encoder rerank pipeline.

#![deny(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::embeddings::TeiClient;
use super::lance_store::cosine_similarity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieverCandidate {
    pub id: String,
    pub text: String,
    pub source_path: String,
    pub dense_score: f32,
    pub sparse_score: f32,
    pub rrf_score: f32,
}

pub struct RetrieverPipeline {
    tei: TeiClient,
    rrf_k: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridWorkflowStage {
    LlamaIndex,
    LlamaAgents,
    Haystack,
    NemoAgentToolkit,
}

impl HybridWorkflowStage {
    pub const fn boost_factor(self) -> f32 {
        match self {
            Self::LlamaIndex => 1.05,
            Self::LlamaAgents => 1.10,
            Self::Haystack => 1.02,
            Self::NemoAgentToolkit => 1.15,
        }
    }
}

impl RetrieverPipeline {
    pub fn new(tei: TeiClient) -> Self {
        Self { tei, rrf_k: 60.0 }
    }

    /// Fuses dense cosine similarity and sparse lexical overlap using Reciprocal Rank Fusion (RRF).
    pub fn fuse_and_rerank(
        &self,
        query: &str,
        docs: &[(String, String, String)], // (id, source_path, text)
        top_k: usize,
    ) -> Vec<RetrieverCandidate> {
        let q_vec = self.tei.embed_local(query);
        let q_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_ascii_lowercase())
            .collect();

        let mut scored: Vec<RetrieverCandidate> = docs
            .iter()
            .map(|(id, path, text)| {
                let d_vec = self.tei.embed_local(text);
                let dense = cosine_similarity(&q_vec, &d_vec);
                let low = text.to_ascii_lowercase();
                let matches = q_terms
                    .iter()
                    .filter(|t| !t.is_empty() && low.contains(t.as_str()))
                    .count();
                let sparse = if q_terms.is_empty() {
                    0.0
                } else {
                    matches as f32 / q_terms.len() as f32
                };
                RetrieverCandidate {
                    id: id.clone(),
                    text: text.clone(),
                    source_path: path.clone(),
                    dense_score: dense,
                    sparse_score: sparse,
                    rrf_score: 0.0,
                }
            })
            .collect();

        let mut dense_order: Vec<usize> = (0..scored.len()).collect();
        dense_order.sort_by(|&a, &b| {
            scored[b]
                .dense_score
                .partial_cmp(&scored[a].dense_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut sparse_order: Vec<usize> = (0..scored.len()).collect();
        sparse_order.sort_by(|&a, &b| {
            scored[b]
                .sparse_score
                .partial_cmp(&scored[a].sparse_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut rrf_map: HashMap<usize, f32> = HashMap::new();
        for (rank, idx) in dense_order.into_iter().enumerate() {
            *rrf_map.entry(idx).or_insert(0.0) += 1.0 / (self.rrf_k + rank as f32 + 1.0);
        }
        for (rank, idx) in sparse_order.into_iter().enumerate() {
            *rrf_map.entry(idx).or_insert(0.0) += 1.0 / (self.rrf_k + rank as f32 + 1.0);
        }
        for (idx, score) in rrf_map {
            scored[idx].rrf_score = score;
        }
        scored.sort_by(|a, b| {
            b.rrf_score
                .partial_cmp(&a.rrf_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_k);
        scored
    }

    pub fn retrieve_with_stages(
        &self,
        query: &str,
        docs: &[(String, String, String)],
        stage: HybridWorkflowStage,
        top_k: usize,
    ) -> Vec<RetrieverCandidate> {
        let mut results = self.fuse_and_rerank(query, docs, top_k);
        let boost = stage.boost_factor();
        for item in &mut results {
            item.rrf_score *= boost;
        }
        results
    }

    pub fn chunk_with_liteparse(text: &str, max_chars: usize) -> Vec<String> {
        crate::training::dataset::liteparse_extract_chunks(text, max_chars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aimemory::embeddings::EmbeddingConfig;

    #[test]
    fn test_retriever_stages_and_liteparse_chunking() {
        let tei = TeiClient::new(EmbeddingConfig::default());
        let pipe = RetrieverPipeline::new(tei);
        let docs = vec![
            ("d1".into(), "src/a.rs".into(), "semantic router query cache".into()),
            ("d2".into(), "src/b.rs".into(), "unrelated text block".into()),
        ];
        let hits = pipe.retrieve_with_stages("semantic router", &docs, HybridWorkflowStage::NemoAgentToolkit, 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "d1");
        assert!(!RetrieverPipeline::chunk_with_liteparse("a\n\nb", 64).is_empty());
    }
}
