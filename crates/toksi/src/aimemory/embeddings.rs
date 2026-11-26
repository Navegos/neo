// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/embeddings.rs
// created: 2026-11-21

//! `text-embeddings-inference` (TEI) `/embed` & `/rerank` HTTP client with deterministic
//! zero-network local hash-projection fallback for offline indexing and testing.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub endpoint: String,
    pub dimensions: usize,
    pub normalize: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:8081".to_string(),
            dimensions: 64,
            normalize: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankScore {
    pub index: usize,
    pub score: f32,
}

pub struct TeiClient {
    config: EmbeddingConfig,
    http: reqwest::Client,
}

impl TeiClient {
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Computes a deterministic local n-gram hashed embedding vector when no TEI daemon is running.
    pub fn embed_local(&self, text: &str) -> Vec<f32> {
        let dims = self.config.dimensions.max(8);
        let mut vec = vec![0.0_f32; dims];
        for token in text.split_whitespace() {
            let mut hasher = Sha256::new();
            hasher.update(token.to_ascii_lowercase().as_bytes());
            let digest = hasher.finalize();
            let idx = (u16::from_le_bytes([digest[0], digest[1]]) as usize) % dims;
            let sign = if digest[2] & 1 == 0 { 1.0 } else { -1.0 };
            vec[idx] += sign;
        }
        if self.config.normalize {
            let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
            if norm > f32::EPSILON {
                for v in &mut vec {
                    *v /= norm;
                }
            }
        }
        vec
    }

    /// Calls local TEI `/embed` if reachable, otherwise falls back cleanly to `embed_local`.
    pub async fn embed(&self, inputs: &[String]) -> Vec<Vec<f32>> {
        let url = format!("{}/embed", self.config.endpoint.trim_end_matches('/'));
        if let Ok(resp) = self
            .http
            .post(&url)
            .json(&serde_json::json!({ "inputs": inputs }))
            .send()
            .await
            && resp.status().is_success()
                && let Ok(parsed) = resp.json::<Vec<Vec<f32>>>().await {
                    return parsed;
                }
        inputs.iter().map(|t| self.embed_local(t)).collect()
    }

    /// Calls local TEI `/rerank` if reachable, otherwise scores deterministically via token overlap.
    pub async fn rerank(&self, query: &str, texts: &[String]) -> Vec<RerankScore> {
        let url = format!("{}/rerank", self.config.endpoint.trim_end_matches('/'));
        if let Ok(resp) = self
            .http
            .post(&url)
            .json(&serde_json::json!({ "query": query, "texts": texts }))
            .send()
            .await
            && resp.status().is_success()
                && let Ok(parsed) = resp.json::<Vec<RerankScore>>().await {
                    return parsed;
                }
        let q_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        let mut out: Vec<RerankScore> = texts
            .iter()
            .enumerate()
            .map(|(index, doc)| {
                let d_low = doc.to_ascii_lowercase();
                let hits = q_terms
                    .iter()
                    .filter(|t| !t.is_empty() && d_low.contains(t.as_str()))
                    .count();
                let score = if q_terms.is_empty() {
                    0.0
                } else {
                    hits as f32 / q_terms.len() as f32
                };
                RerankScore { index, score }
            })
            .collect();
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }
}
