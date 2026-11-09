// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/semantic_cache.rs
// created: 2026-10-31
// lastModified: 2026-11-09

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, RwLock};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticCacheError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Internal lock poisoned")]
    LockPoisoned,
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// Supported Chroma-style distance metrics for vector similarity calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    L2,
    Ip,
}

/// Metadata and definition for a Chroma vector collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaCollection {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    pub hash: String,
    pub prompt: String,
    pub response: String,
    pub model: String,
    pub embedding: Option<Vec<f32>>,
    pub hit_count: u64,
}

/// Dual-tier semantic cache backing store inspired by Chroma vector database.
pub struct DualTierSemanticCache {
    exact_cache: RwLock<HashMap<String, CachedEntry>>,
    semantic_entries: RwLock<Vec<CachedEntry>>,
    db_conn: Mutex<Option<Connection>>,
    similarity_threshold: f32,
    metric: DistanceMetric,
}

impl Default for DualTierSemanticCache {
    fn default() -> Self {
        Self::new(0.92)
    }
}

impl DualTierSemanticCache {
    pub fn new(similarity_threshold: f32) -> Self {
        Self::with_metric(similarity_threshold, DistanceMetric::Cosine)
    }

    pub fn with_metric(similarity_threshold: f32, metric: DistanceMetric) -> Self {
        Self {
            exact_cache: RwLock::new(HashMap::new()),
            semantic_entries: RwLock::new(Vec::new()),
            db_conn: Mutex::new(None),
            similarity_threshold,
            metric,
        }
    }

    pub fn init_db(&self, db_path: &Path) -> Result<(), SemanticCacheError> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS semantic_cache (
                hash TEXT PRIMARY KEY,
                prompt TEXT NOT NULL,
                response TEXT NOT NULL,
                model TEXT NOT NULL,
                hit_count INTEGER DEFAULT 0
            )",
            [],
        )?;

        let mut lock = self.db_conn.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut cache = self.exact_cache.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = cache.get_mut(&hash) {
            entry.hit_count += 1;
            return Some(entry.response.clone());
        }
        None
    }

    /// Tier-2 Semantic Vector Similarity match based on configured distance metric.
    pub fn lookup_semantic(&self, query_embedding: &[f32], model: &str) -> Option<(String, f32)> {
        let entries = self.semantic_entries.read().unwrap_or_else(|e| e.into_inner());
        let mut best_match: Option<(String, f32)> = None;

        for e in entries.iter().filter(|e| e.model == model) {
            if let Some(ref emb) = e.embedding {
                let score = match self.metric {
                    DistanceMetric::Cosine => cosine_similarity(query_embedding, emb),
                    DistanceMetric::Ip => inner_product(query_embedding, emb),
                    DistanceMetric::L2 => {
                        let dist = l2_distance(query_embedding, emb);
                        1.0 / (1.0 + dist)
                    }
                };

                if score >= self.similarity_threshold {
                    if best_match.as_ref().map(|(_, best)| score > *best).unwrap_or(true) {
                        best_match = Some((e.response.clone(), score));
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
            let mut cache = self.exact_cache.write().unwrap_or_else(|e| e.into_inner());
            cache.insert(hash.clone(), entry.clone());
        }

        if embedding.is_some() {
            let mut sem = self.semantic_entries.write().unwrap_or_else(|e| e.into_inner());
            sem.push(entry);
        }

        // Write to SQLite WAL if available
        if let Ok(guard) = self.db_conn.lock() {
            if let Some(ref conn) = *guard {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO semantic_cache (hash, prompt, response, model, hit_count) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![hash, prompt, response, model, 0],
                );
            }
        }
    }

    /// Dynamic purge: purge cached entries whose prompts or responses match ignored paths or patterns.
    pub fn purge_ignored_entries(&self, root: &Path) -> usize {
        let ignore = crate::codegraph::walker::build_project_ignore(root);
        let mut purged_hashes = Vec::new();

        {
            let cache = self.exact_cache.read().unwrap_or_else(|e| e.into_inner());
            for (hash, entry) in cache.iter() {
                let words: Vec<&str> = entry.prompt.split_whitespace()
                    .chain(entry.response.split_whitespace())
                    .collect();
                for w in words {
                    let cleaned = w.trim_matches(|c| c == '\'' || c == '"' || c == '`' || c == '(' || c == ')' || c == ':');
                    if cleaned.contains('/') || cleaned.contains('\\') || cleaned.ends_with(".lock") {
                        let norm = crate::util::paths::normalize_path(cleaned);
                        let full = if Path::new(&norm).is_absolute() {
                            std::path::PathBuf::from(&norm)
                        } else {
                            root.join(&norm)
                        };
                        let is_dir = full.is_dir();
                        if ignore.matched_path_or_any_parents(&full, is_dir).is_ignore()
                            || crate::codegraph::walker::is_build_or_vendor_artifact(&norm)
                            || norm.split('/').any(crate::codegraph::walker::is_build_or_vendor_artifact)
                        {
                            purged_hashes.push(hash.clone());
                            break;
                        }
                    }
                }
            }
        }

        let purged_count = purged_hashes.len();
        if !purged_hashes.is_empty() {
            let mut cache = self.exact_cache.write().unwrap_or_else(|e| e.into_inner());
            let mut sem = self.semantic_entries.write().unwrap_or_else(|e| e.into_inner());
            for h in &purged_hashes {
                cache.remove(h);
            }
            sem.retain(|e| !purged_hashes.contains(&e.hash));

            if let Ok(guard) = self.db_conn.lock() {
                if let Some(ref conn) = *guard {
                    for h in &purged_hashes {
                        let _ = conn.execute("DELETE FROM semantic_cache WHERE hash = ?1", params![h]);
                    }
                }
            }
        }

        purged_count
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

pub fn inner_product(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }
    a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f32>().sqrt()
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
        if let Some((resp, sim)) = hit {
            assert_eq!(resp, response);
            assert!(sim >= 0.92);
        }

        // Orthogonal embedding
        let query_dissimilar = vec![0.0, 1.0, 0.0];
        assert!(cache.lookup_semantic(&query_dissimilar, model).is_none());
    }

    #[test]
    fn test_sqlite_semantic_cache_persistence() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("cache.db");

        let cache = DualTierSemanticCache::new(0.92);
        assert!(cache.init_db(&db_path).is_ok());

        cache.store("gpt-4o", "test query", "cached response", None);
        assert_eq!(cache.lookup_exact("gpt-4o", "test query").as_deref(), Some("cached response"));
    }

    #[test]
    fn test_purge_ignored_entries() {
        let temp_dir = std::env::temp_dir().join("toksi_sem_cache_purge_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let gitignore = temp_dir.join(".gitignore");
        let _ = std::fs::write(&gitignore, "target/\n");

        let cache = DualTierSemanticCache::new(0.92);
        cache.store("gpt-4o", "read file target/debug/build.log", "output content", None);
        cache.store("gpt-4o", "read file src/main.rs", "main content", None);

        let purged = cache.purge_ignored_entries(&temp_dir);
        assert_eq!(purged, 1);
        assert!(cache.lookup_exact("gpt-4o", "read file target/debug/build.log").is_none());
        assert!(cache.lookup_exact("gpt-4o", "read file src/main.rs").is_some());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
