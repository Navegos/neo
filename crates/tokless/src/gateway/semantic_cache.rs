// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/semantic_cache.rs

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, RwLock};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    pub hash: String,
    pub prompt: String,
    pub response: String,
    pub model: String,
    pub embedding: Option<Vec<f32>>,
    pub hit_count: u64,
}

pub struct DualTierSemanticCache {
    exact_cache: RwLock<HashMap<String, CachedEntry>>,
    semantic_entries: RwLock<Vec<CachedEntry>>,
    db_conn: Mutex<Option<Connection>>,
    similarity_threshold: f32,
}

impl Default for DualTierSemanticCache {
    fn default() -> Self {
        Self::new(0.92)
    }
}

impl DualTierSemanticCache {
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            exact_cache: RwLock::new(HashMap::new()),
            semantic_entries: RwLock::new(Vec::new()),
            db_conn: Mutex::new(None),
            similarity_threshold,
        }
    }

    pub fn init_db(&self, db_path: &Path) -> Result<(), String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS semantic_cache (
                hash TEXT PRIMARY KEY,
                prompt TEXT NOT NULL,
                response TEXT NOT NULL,
                model TEXT NOT NULL,
                hit_count INTEGER DEFAULT 0
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        let mut lock = self.db_conn.lock().unwrap();
        *lock = Some(conn);
        Ok(())
    }

    pub fn compute_prompt_hash(model: &str, prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(b":");
        hasher.update(prompt.as_bytes());
        let res = hasher.finalize();
        res.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Fast Tier-1 direct exact SHA-256 match (<1ms)
    pub fn lookup_exact(&self, model: &str, prompt: &str) -> Option<String> {
        let hash = Self::compute_prompt_hash(model, prompt);
        let mut cache = self.exact_cache.write().unwrap();
        if let Some(entry) = cache.get_mut(&hash) {
            entry.hit_count += 1;
            return Some(entry.response.clone());
        }
        None
    }

    /// Tier-2 Semantic Cosine Similarity match
    pub fn lookup_semantic(&self, query_embedding: &[f32], model: &str) -> Option<(String, f32)> {
        let entries = self.semantic_entries.read().unwrap();
        let mut best_match: Option<(String, f32)> = None;

        for e in entries.iter().filter(|e| e.model == model) {
            if let Some(ref emb) = e.embedding {
                let sim = cosine_similarity(query_embedding, emb);
                if sim >= self.similarity_threshold {
                    if best_match.as_ref().map(|(_, best)| sim > *best).unwrap_or(true) {
                        best_match = Some((e.response.clone(), sim));
                    }
                }
            }
        }
        best_match
    }

    pub fn store(&self, model: &str, prompt: &str, response: &str, embedding: Option<Vec<f32>>) {
        let hash = Self::compute_prompt_hash(model, prompt);
        let entry = CachedEntry {
            hash: hash.clone(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            model: model.to_string(),
            embedding: embedding.clone(),
            hit_count: 0,
        };

        {
            let mut cache = self.exact_cache.write().unwrap();
            cache.insert(hash.clone(), entry.clone());
        }

        if embedding.is_some() {
            let mut sem = self.semantic_entries.write().unwrap();
            sem.push(entry);
        }

        // Async write to SQLite if available
        if let Ok(guard) = self.db_conn.lock() {
            if let Some(ref conn) = *guard {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO semantic_cache (hash, prompt, response, model, hit_count) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![hash, prompt, response, model, 0],
                );
            }
        }
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}
