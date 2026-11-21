// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/milvus_adapter.rs
// created: 2026-11-21

//! Pure-Rust Milvus-compatible partition-keyed hybrid dense+sparse vector adapter.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::lance_store::{LanceRecord, LanceScalarFilter, LanceSearchHit, LanceStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilvusEntity {
    pub id: String,
    pub partition_key: String, // subproject_id
    pub repo_hash: String,
    pub memory_type: String,
    pub source_path: String,
    pub text: String,
    pub dense_vector: Vec<f32>,
    #[serde(default)]
    pub sparse_terms: HashMap<String, f32>,
}

pub struct MilvusAdapter {
    store: LanceStore,
    sparse_index: HashMap<String, HashMap<String, f32>>,
}

impl MilvusAdapter {
    pub fn open(ai_memory_dir: &Path) -> Result<Self, String> {
        let store = LanceStore::open(&ai_memory_dir.join("milvus"))?;
        Ok(Self {
            store,
            sparse_index: HashMap::new(),
        })
    }

    pub fn insert(&mut self, entity: MilvusEntity) -> Result<(), String> {
        if !entity.sparse_terms.is_empty() {
            self.sparse_index
                .insert(entity.id.clone(), entity.sparse_terms);
        }
        self.store.upsert(LanceRecord {
            id: entity.id,
            subproject_id: entity.partition_key,
            repo_hash: entity.repo_hash,
            memory_type: entity.memory_type,
            source_path: entity.source_path,
            text: entity.text,
            vector: entity.dense_vector,
            quantized_i8: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn hybrid_search(
        &self,
        dense_query: &[f32],
        sparse_query: &HashMap<String, f32>,
        partition_key: Option<&str>,
        top_k: usize,
    ) -> Vec<LanceSearchHit> {
        let mut hits = self.store.search(
            dense_query,
            &LanceScalarFilter {
                subproject_id: partition_key.map(str::to_string),
                ..Default::default()
            },
            top_k.saturating_mul(2).max(8),
        );
        if !sparse_query.is_empty() {
            for hit in &mut hits {
                if let Some(terms) = self.sparse_index.get(&hit.id) {
                    let sparse_dot: f32 = sparse_query
                        .iter()
                        .filter_map(|(k, w)| terms.get(k).map(|tw| tw * w))
                        .sum();
                    hit.score = 0.7 * hit.score + 0.3 * sparse_dot;
                }
            }
            hits.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        hits.truncate(top_k);
        hits
    }

    pub fn purge_ignored(&mut self, patterns: &[String]) -> Result<usize, String> {
        self.store.purge_ignored(patterns)
    }
}
