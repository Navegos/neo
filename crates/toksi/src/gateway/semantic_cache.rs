// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/semantic_cache.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
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

const HNSW_MAX_NEIGHBORS: usize = 16;
const HNSW_EF_CONSTRUCTION: usize = 64;
const HNSW_EF_SEARCH: usize = 32;
const DEFAULT_MAX_MEMORY_ENTRIES: usize = 4096;

#[derive(Debug, Clone)]
struct HnswNode {
    entry_idx: usize,
    model: String,
    embedding: Vec<f32>,
    neighbors: Vec<usize>,
}

/// Chroma-inspired HNSW ANN vector index (`M = 16`) for fast Tier-2 semantic cache lookup.
#[derive(Debug, Default)]
pub struct HnswSemanticIndex {
    nodes: Vec<HnswNode>,
}

#[derive(Debug, Clone, Copy)]
struct ScoredIdx {
    idx: usize,
    score: f32,
}

impl PartialEq for ScoredIdx {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.idx == other.idx
    }
}
impl Eq for ScoredIdx {}
impl PartialOrd for ScoredIdx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ScoredIdx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl HnswSemanticIndex {
    pub fn insert(&mut self, entry_idx: usize, model: String, embedding: Vec<f32>, metric: DistanceMetric) {
        let new_id = self.nodes.len();
        let mut candidates: Vec<(usize, f32)> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.model == model && n.embedding.len() == embedding.len())
            .map(|(idx, n)| (idx, score_vectors(&embedding, &n.embedding, metric)))
            .collect();
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(HNSW_EF_CONSTRUCTION);

        let selected: Vec<usize> = candidates
            .into_iter()
            .take(HNSW_MAX_NEIGHBORS)
            .map(|(idx, _)| idx)
            .collect();

        self.nodes.push(HnswNode {
            entry_idx,
            model,
            embedding,
            neighbors: selected.clone(),
        });

        for neighbor_idx in selected {
            if let Some(neighbor) = self.nodes.get_mut(neighbor_idx) {
                if !neighbor.neighbors.contains(&new_id) {
                    neighbor.neighbors.push(new_id);
                    if neighbor.neighbors.len() > HNSW_MAX_NEIGHBORS * 2 {
                        neighbor.neighbors.truncate(HNSW_MAX_NEIGHBORS * 2);
                    }
                }
            }
        }
    }

    pub fn search_best(&self, query: &[f32], model: &str, metric: DistanceMetric) -> Option<(usize, f32)> {
        let seed_idx = self
            .nodes
            .iter()
            .position(|n| n.model == model && n.embedding.len() == query.len())?;

        let mut visited = HashSet::new();
        let mut frontier = BinaryHeap::new();
        let seed_score = score_vectors(query, &self.nodes[seed_idx].embedding, metric);
        visited.insert(seed_idx);
        frontier.push(ScoredIdx {
            idx: seed_idx,
            score: seed_score,
        });

        let mut best = (self.nodes[seed_idx].entry_idx, seed_score);
        let mut explored = 0usize;

        while let Some(cur) = frontier.pop() {
            if cur.score > best.1 {
                best = (self.nodes[cur.idx].entry_idx, cur.score);
            }
            explored += 1;
            if explored >= HNSW_EF_SEARCH {
                break;
            }
            for &nbr_idx in &self.nodes[cur.idx].neighbors {
                if visited.insert(nbr_idx) {
                    let nbr = &self.nodes[nbr_idx];
                    if nbr.model == model && nbr.embedding.len() == query.len() {
                        let score = score_vectors(query, &nbr.embedding, metric);
                        frontier.push(ScoredIdx { idx: nbr_idx, score });
                    }
                }
            }
        }

        // Fallback check on unlinked nodes of same model in sparse graphs
        if self.nodes.len() <= HNSW_EF_CONSTRUCTION {
            for node in &self.nodes {
                if node.model == model && node.embedding.len() == query.len() {
                    let s = score_vectors(query, &node.embedding, metric);
                    if s > best.1 {
                        best = (node.entry_idx, s);
                    }
                }
            }
        }

        Some(best)
    }

    pub fn rebuild_for_entries(&mut self, entries: &[CachedEntry], metric: DistanceMetric) {
        self.nodes.clear();
        for (idx, e) in entries.iter().enumerate() {
            if let Some(ref emb) = e.embedding {
                self.insert(idx, e.model.clone(), emb.clone(), metric);
            }
        }
    }
}

fn score_vectors(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => cosine_similarity(a, b),
        DistanceMetric::Ip => inner_product(a, b),
        DistanceMetric::L2 => 1.0 / (1.0 + l2_distance(a, b)),
    }
}

/// Dual-tier semantic cache backing store with Chroma-style HNSW ANN + foyer-style bounded spill.
pub struct DualTierSemanticCache {
    exact_cache: RwLock<HashMap<String, CachedEntry>>,
    eviction_order: Mutex<VecDeque<String>>,
    semantic_entries: RwLock<Vec<CachedEntry>>,
    hnsw_index: RwLock<HnswSemanticIndex>,
    db_conn: Mutex<Option<Connection>>,
    similarity_threshold: f32,
    metric: DistanceMetric,
    max_memory_entries: usize,
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
        Self::with_capacity(similarity_threshold, metric, DEFAULT_MAX_MEMORY_ENTRIES)
    }

    pub fn with_capacity(similarity_threshold: f32, metric: DistanceMetric, max_memory_entries: usize) -> Self {
        Self {
            exact_cache: RwLock::new(HashMap::new()),
            eviction_order: Mutex::new(VecDeque::new()),
            semantic_entries: RwLock::new(Vec::new()),
            hnsw_index: RwLock::new(HnswSemanticIndex::default()),
            db_conn: Mutex::new(None),
            similarity_threshold,
            metric,
            max_memory_entries: max_memory_entries.max(1),
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

    /// Fast Tier-1 direct exact SHA-256 match (<1ms), falling back to SQLite disk spill if evicted.
    pub fn lookup_exact(&self, model: &str, prompt: &str) -> Option<String> {
        let hash = Self::compute_prompt_hash(model, prompt);
        {
            let mut cache = self.exact_cache.write().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = cache.get_mut(&hash) {
                entry.hit_count += 1;
                return Some(entry.response.clone());
            }
        }

        // Check SQLite disk spill if RAM entry was evicted by foyer-style bounded policy
        let spilled: Option<(String, String)> = if let Ok(guard) = self.db_conn.lock() {
            guard.as_ref().and_then(|conn| {
                conn.query_row(
                    "SELECT prompt, response FROM semantic_cache WHERE hash = ?1",
                    params![hash],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .ok()
            })
        } else {
            None
        };

        if let Some((stored_prompt, stored_response)) = spilled {
            let mut cache = self.exact_cache.write().unwrap_or_else(|e| e.into_inner());
            cache.insert(
                hash.clone(),
                CachedEntry {
                    hash,
                    prompt: stored_prompt,
                    response: stored_response.clone(),
                    model: model.to_string(),
                    embedding: None,
                    hit_count: 1,
                },
            );
            return Some(stored_response);
        }

        None
    }

    /// Tier-2 Semantic Vector Similarity match via Chroma-style HNSW ANN index.
    pub fn lookup_semantic(&self, query_embedding: &[f32], model: &str) -> Option<(String, f32)> {
        let entries = self.semantic_entries.read().unwrap_or_else(|e| e.into_inner());
        let hnsw = self.hnsw_index.read().unwrap_or_else(|e| e.into_inner());
        if let Some((entry_idx, score)) = hnsw.search_best(query_embedding, model, self.metric) {
            if score >= self.similarity_threshold {
                if let Some(entry) = entries.get(entry_idx) {
                    return Some((entry.response.clone(), score));
                }
            }
        }
        None
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

        // Write to SQLite WAL first so foyer-style RAM eviction never drops persistence
        if let Ok(guard) = self.db_conn.lock() {
            if let Some(ref conn) = *guard {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO semantic_cache (hash, prompt, response, model, hit_count) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![hash, prompt, response, model, 0],
                );
            }
        }

        {
            let mut cache = self.exact_cache.write().unwrap_or_else(|e| e.into_inner());
            let mut order = self.eviction_order.lock().unwrap_or_else(|e| e.into_inner());
            if !cache.contains_key(&hash) {
                while cache.len() >= self.max_memory_entries {
                    if let Some(oldest) = order.pop_front() {
                        cache.remove(&oldest);
                    } else {
                        break;
                    }
                }
                order.push_back(hash.clone());
            }
            cache.insert(hash, entry.clone());
        }

        if let Some(emb) = embedding {
            let mut sem = self.semantic_entries.write().unwrap_or_else(|e| e.into_inner());
            let mut hnsw = self.hnsw_index.write().unwrap_or_else(|e| e.into_inner());
            if sem.len() >= self.max_memory_entries {
                sem.remove(0);
                sem.push(entry);
                hnsw.rebuild_for_entries(&sem, self.metric);
            } else {
                let idx = sem.len();
                sem.push(entry);
                hnsw.insert(idx, model.to_string(), emb, self.metric);
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
            let mut hnsw = self.hnsw_index.write().unwrap_or_else(|e| e.into_inner());
            for h in &purged_hashes {
                cache.remove(h);
            }
            sem.retain(|e| !purged_hashes.contains(&e.hash));
            hnsw.rebuild_for_entries(&sem, self.metric);

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

    #[test]
    fn test_foyer_bounded_eviction_spills_to_sqlite() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("spill.db");
        let cache = DualTierSemanticCache::with_capacity(0.92, DistanceMetric::Cosine, 2);
        cache.init_db(&db_path).expect("init db");

        cache.store("gpt-4o", "p1", "r1", None);
        cache.store("gpt-4o", "p2", "r2", None);
        cache.store("gpt-4o", "p3", "r3", None);

        // RAM holds max 2, yet disk spill recovers evicted p1 cleanly
        assert!(cache.exact_cache.read().expect("rwlock").len() <= 2);
        assert_eq!(cache.lookup_exact("gpt-4o", "p1").as_deref(), Some("r1"));
    }
}
