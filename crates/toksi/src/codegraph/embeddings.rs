// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/embeddings.rs
// created: 2026-11-21

//! CodeGraph local embedding & ingestion pipeline contracts (`text-embeddings-inference`,
//! `optimum-onnx`, `Curator`, `DALI`, `cuml`, `cudf`, `cuCollections`, `cuCascade`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::aimemory::embeddings::{EmbeddingConfig, TeiClient};
use crate::aimemory::lance_store::cosine_similarity;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeIngestionPipelineSpec {
    pub name: String,
    pub backend_vault: String,
    pub batch_size: usize,
    pub dimensions: usize,
}

pub fn default_code_ingestion_pipelines() -> Vec<CodeIngestionPipelineSpec> {
    vec![
        CodeIngestionPipelineSpec {
            name: "tei-local-code".to_string(),
            backend_vault: "text-embeddings-inference".to_string(),
            batch_size: 32,
            dimensions: 384,
        },
        CodeIngestionPipelineSpec {
            name: "optimum-onnx-cpu".to_string(),
            backend_vault: "optimum-onnx".to_string(),
            batch_size: 16,
            dimensions: 384,
        },
        CodeIngestionPipelineSpec {
            name: "nemo-curator-dedup".to_string(),
            backend_vault: "Curator".to_string(),
            batch_size: 128,
            dimensions: 768,
        },
    ]
}

/// Computes a deterministic local code-symbol embedding vector.
pub fn embed_code_symbol(symbol_sig: &str, dims: usize) -> Vec<f32> {
    let client = TeiClient::new(EmbeddingConfig {
        dimensions: dims.max(8),
        ..Default::default()
    });
    client.embed_local(symbol_sig)
}

/// Scores cosine similarity between two code symbol embeddings.
pub fn score_code_similarity(a: &[f32], b: &[f32]) -> f32 {
    cosine_similarity(a, b)
}
