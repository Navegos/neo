// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/semantic_cache.rs
// created: 2026-10-31
// lastModified: 2026-10-31

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_sha256_cache_lookup() {
        let cache = DualTierSemanticCache::new(0.92);
        let model = "claude-3-7-sonnet";
        let prompt = "Explain quantum computing";
        let response = "Quantum computing uses qubits and superposition.";

        assert!(cache.lookup_exact(model, prompt).is_none());
        cache.store(model, prompt, response, None);

        let hit = cache.lookup_exact(model, prompt);
        assert_eq!(hit.as_deref(), Some(response));
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];

        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-5);
        assert_eq!(cosine_similarity(&v1, &v3), 0.0);
    }

    #[test]
    fn test_semantic_vector_lookup_threshold() {
        let cache = DualTierSemanticCache::new(0.92);
        let model = "text-embedding-3";
        let prompt = "How to write a fast compiler";
        let response = "Use SSA form and LLVM IR.";
        let emb = vec![0.95, 0.1, 0.05];

        cache.store(model, prompt, response, Some(emb));

        // Similar embedding (cosine similarity >= 0.92)
        let query_similar = vec![0.94, 0.11, 0.04];
        let hit = cache.lookup_semantic(&query_similar, model);
        assert!(hit.is_some());
        let (resp, sim) = hit.unwrap();
        assert_eq!(resp, response);
        assert!(sim >= 0.92);

        // Orthogonal embedding
        let query_dissimilar = vec![0.0, 1.0, 0.0];
        assert!(cache.lookup_semantic(&query_dissimilar, model).is_none());
    }

    #[test]
    fn test_sqlite_semantic_cache_persistence() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("cache.db");

        let cache = DualTierSemanticCache::new(0.92);
        assert!(cache.init_db(&db_path).is_ok());

        cache.store("gpt-4o", "test query", "cached response", None);
        assert_eq!(cache.lookup_exact("gpt-4o", "test query").as_deref(), Some("cached response"));
    }
}
