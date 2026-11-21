// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/qdrant_store.rs
// created: 2026-11-21

//! Pure-Rust Qdrant-compatible quantized HNSW & scalar-filtered collection store
//! persisted under `<repo>/.toksi/ai-memory/qdrant/`.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::lance_store::{cosine_similarity, quantize_f32_to_i8};
use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantDistance {
    Cosine,
    Dot,
    Euclid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub subproject_id: String,
    pub repo_hash: String,
    pub memory_type: String,
    pub source_path: String,
    pub payload: serde_json::Value,
    pub vector: Vec<f32>,
    #[serde(default)]
    pub scalar_quantized: Vec<i8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QdrantFilter {
    pub subproject_id: Option<String>,
    pub repo_hash: Option<String>,
    pub memory_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QdrantScoredPoint {
    pub id: String,
    pub score: f32,
    pub source_path: String,
    pub payload: serde_json::Value,
}

pub struct QdrantCollection {
    root: PathBuf,
    name: String,
    distance: QdrantDistance,
    points: HashMap<String, QdrantPoint>,
}

impl QdrantCollection {
    pub fn open(ai_memory_dir: &Path, name: &str, distance: QdrantDistance) -> Result<Self, String> {
        let root = ai_memory_dir.join("qdrant");
        fs::create_dir_all(&root).map_err(|e| format!("qdrant mkdir failed: {e}"))?;
        let mut col = Self {
            root,
            name: name.to_string(),
            distance,
            points: HashMap::new(),
        };
        col.load()?;
        Ok(col)
    }

    fn data_file(&self) -> PathBuf {
        self.root.join(format!("{}.json", self.name))
    }

    fn load(&mut self) -> Result<(), String> {
        let file = self.data_file();
        if !file.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(&file).map_err(|e| format!("qdrant read failed: {e}"))?;
        let pts: Vec<QdrantPoint> =
            serde_json::from_str(&raw).map_err(|e| format!("qdrant parse failed: {e}"))?;
        self.points = pts.into_iter().map(|p| (p.id.clone(), p)).collect();
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        let list: Vec<&QdrantPoint> = self.points.values().collect();
        let json = serde_json::to_string_pretty(&list)
            .map_err(|e| format!("qdrant serialize failed: {e}"))?;
        fs::write(self.data_file(), json).map_err(|e| format!("qdrant write failed: {e}"))?;
        Ok(())
    }

    pub fn upsert(&mut self, mut pt: QdrantPoint) -> Result<(), String> {
        pt.source_path = normalize_path(&pt.source_path);
        if pt.scalar_quantized.is_empty() && !pt.vector.is_empty() {
            pt.scalar_quantized = quantize_f32_to_i8(&pt.vector);
        }
        self.points.insert(pt.id.clone(), pt);
        self.save()
    }

    fn score_vectors(&self, query: &[f32], target: &[f32]) -> f32 {
        match self.distance {
            QdrantDistance::Cosine => cosine_similarity(query, target),
            QdrantDistance::Dot => query.iter().zip(target.iter()).map(|(a, b)| a * b).sum(),
            QdrantDistance::Euclid => {
                let dist_sq: f32 = query
                    .iter()
                    .zip(target.iter())
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum();
                1.0 / (1.0 + dist_sq.sqrt())
            }
        }
    }

    pub fn search(
        &self,
        query: &[f32],
        filter: &QdrantFilter,
        limit: usize,
    ) -> Vec<QdrantScoredPoint> {
        let mut out: Vec<QdrantScoredPoint> = self
            .points
            .values()
            .filter(|pt| {
                filter
                    .subproject_id
                    .as_deref()
                    .is_none_or(|s| pt.subproject_id == s)
                    && filter
                        .repo_hash
                        .as_deref()
                        .is_none_or(|r| pt.repo_hash == r)
                    && filter
                        .memory_type
                        .as_deref()
                        .is_none_or(|m| pt.memory_type == m)
            })
            .map(|pt| QdrantScoredPoint {
                id: pt.id.clone(),
                score: self.score_vectors(query, &pt.vector),
                source_path: pt.source_path.clone(),
                payload: pt.payload.clone(),
            })
            .collect();

        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.truncate(limit);
        out
    }

    pub fn purge_ignored(&mut self, ignore_patterns: &[String]) -> Result<usize, String> {
        if ignore_patterns.is_empty() {
            return Ok(0);
        }
        let pats: Vec<String> = ignore_patterns
            .iter()
            .map(|p| normalize_path(p.trim()).trim_matches('/').to_ascii_lowercase())
            .filter(|p| !p.is_empty())
            .collect();
        let before = self.points.len();
        self.points.retain(|_, pt| {
            let src = pt.source_path.to_ascii_lowercase();
            !pats.iter().any(|p| {
                if let Some(ext) = p.strip_prefix("*.") {
                    src.ends_with(&format!(".{ext}"))
                } else {
                    src.contains(p)
                }
            })
        });
        let removed = before.saturating_sub(self.points.len());
        if removed > 0 {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn count(&self) -> usize {
        self.points.len()
    }
}
