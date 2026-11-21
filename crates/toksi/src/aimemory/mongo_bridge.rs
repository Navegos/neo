// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/mongo_bridge.rs
// created: 2026-11-21

//! Pure-Rust `langchain-mongodb` document + vector search bridge backed by `<repo>/.toksi/ai-memory/`.

#![deny(unsafe_code)]

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::qdrant_store::{QdrantCollection, QdrantDistance, QdrantFilter, QdrantPoint, QdrantScoredPoint};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDocRecord {
    pub doc_id: String,
    pub collection: String,
    pub subproject_id: String,
    pub repo_hash: String,
    pub memory_type: String,
    pub source_path: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Vec<f32>,
}

pub struct MongoVectorBridge {
    collection: QdrantCollection,
}

impl MongoVectorBridge {
    pub fn open(ai_memory_dir: &Path, collection_name: &str) -> Result<Self, String> {
        let collection = QdrantCollection::open(
            &ai_memory_dir.join("mongo_bridge"),
            collection_name,
            QdrantDistance::Cosine,
        )?;
        Ok(Self { collection })
    }

    pub fn add_document(&mut self, doc: MongoDocRecord) -> Result<(), String> {
        self.collection.upsert(QdrantPoint {
            id: doc.doc_id,
            subproject_id: doc.subproject_id,
            repo_hash: doc.repo_hash,
            memory_type: doc.memory_type,
            source_path: doc.source_path,
            payload: serde_json::json!({
                "collection": doc.collection,
                "content": doc.content,
                "metadata": doc.metadata,
            }),
            vector: doc.embedding,
            scalar_quantized: Vec::new(),
        })
    }

    pub fn similarity_search(
        &self,
        query_embedding: &[f32],
        filter: &QdrantFilter,
        k: usize,
    ) -> Vec<QdrantScoredPoint> {
        self.collection.search(query_embedding, filter, k)
    }

    pub fn purge_ignored(&mut self, patterns: &[String]) -> Result<usize, String> {
        self.collection.purge_ignored(patterns)
    }
}
