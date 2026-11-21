// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/lance_store.rs
// created: 2026-11-21

//! Pure-Rust Lance-style columnar vector & hybrid storage inside `<repo>/.toksi/ai-memory/lance/`
//! with scalar pre-filtering (`subproject_id`, `repo_hash`, `memory_type`) and instant `.ignore` deletion.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanceRecord {
    pub id: String,
    pub subproject_id: String,
    pub repo_hash: String,
    pub memory_type: String,
    pub source_path: String,
    pub text: String,
    pub vector: Vec<f32>,
    #[serde(default)]
    pub quantized_i8: Vec<i8>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanceScalarFilter {
    pub subproject_id: Option<String>,
    pub repo_hash: Option<String>,
    pub memory_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanceSearchHit {
    pub id: String,
    pub score: f32,
    pub source_path: String,
    pub text: String,
    pub memory_type: String,
}

/// Scalar-quantizes an f32 vector into signed 8-bit integers for compact columnar storage.
pub fn quantize_f32_to_i8(vec: &[f32]) -> Vec<i8> {
    let max_abs = vec
        .iter()
        .fold(0.0_f32, |acc, v| acc.max(v.abs()));
    if max_abs <= f32::EPSILON {
        return vec![0; vec.len()];
    }
    vec.iter()
        .map(|v| ((v / max_abs) * 127.0).round().clamp(-127.0, 127.0) as i8)
        .collect()
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0_f32;
    let mut na = 0.0_f32;
    let mut nb = 0.0_f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

pub struct LanceStore {
    root: PathBuf,
    records: HashMap<String, LanceRecord>,
}

impl LanceStore {
    pub fn open(ai_memory_dir: &Path) -> Result<Self, String> {
        let root = ai_memory_dir.join("lance");
        fs::create_dir_all(&root).map_err(|e| format!("lance mkdir failed: {e}"))?;
        let mut store = Self {
            root,
            records: HashMap::new(),
        };
        store.reload()?;
        Ok(store)
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("segments.jsonl")
    }

    pub fn reload(&mut self) -> Result<(), String> {
        self.records.clear();
        let path = self.manifest_path();
        if !path.exists() {
            return Ok(());
        }
        let file = fs::File::open(&path).map_err(|e| format!("lance open failed: {e}"))?;
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(rec) = serde_json::from_str::<LanceRecord>(trimmed) {
                self.records.insert(rec.id.clone(), rec);
            }
        }
        Ok(())
    }

    fn flush(&self) -> Result<(), String> {
        let path = self.manifest_path();
        let tmp = self.root.join("segments.jsonl.tmp");
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp)
                .map_err(|e| format!("lance flush open failed: {e}"))?;
            for rec in self.records.values() {
                let line = serde_json::to_string(rec)
                    .map_err(|e| format!("lance serialize failed: {e}"))?;
                writeln!(file, "{line}").map_err(|e| format!("lance write failed: {e}"))?;
            }
            file.sync_all()
                .map_err(|e| format!("lance sync failed: {e}"))?;
        }
        fs::rename(&tmp, &path).map_err(|e| format!("lance rename failed: {e}"))?;
        Ok(())
    }

    pub fn upsert(&mut self, mut rec: LanceRecord) -> Result<(), String> {
        rec.source_path = normalize_path(&rec.source_path);
        if rec.quantized_i8.is_empty() && !rec.vector.is_empty() {
            rec.quantized_i8 = quantize_f32_to_i8(&rec.vector);
        }
        self.records.insert(rec.id.clone(), rec);
        self.flush()
    }

    pub fn search(
        &self,
        query: &[f32],
        filter: &LanceScalarFilter,
        top_k: usize,
    ) -> Vec<LanceSearchHit> {
        let mut hits: Vec<LanceSearchHit> = self
            .records
            .values()
            .filter(|rec| {
                filter
                    .subproject_id
                    .as_deref()
                    .is_none_or(|s| rec.subproject_id == s)
                    && filter
                        .repo_hash
                        .as_deref()
                        .is_none_or(|r| rec.repo_hash == r)
                    && filter
                        .memory_type
                        .as_deref()
                        .is_none_or(|m| rec.memory_type == m)
            })
            .map(|rec| LanceSearchHit {
                id: rec.id.clone(),
                score: cosine_similarity(query, &rec.vector),
                source_path: rec.source_path.clone(),
                text: rec.text.clone(),
                memory_type: rec.memory_type.clone(),
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }

    /// Instantly removes any indexed vectors whose `source_path` matches an ignore pattern.
    pub fn purge_ignored(&mut self, ignore_patterns: &[String]) -> Result<usize, String> {
        if ignore_patterns.is_empty() {
            return Ok(0);
        }
        let norm_pats: Vec<String> = ignore_patterns
            .iter()
            .map(|p| normalize_path(p.trim()).trim_matches('/').to_string())
            .filter(|p| !p.is_empty())
            .collect();
        let before = self.records.len();
        self.records.retain(|_, rec| {
            let src = rec.source_path.to_ascii_lowercase();
            !norm_pats.iter().any(|pat| {
                let p = pat.to_ascii_lowercase();
                if let Some(ext) = p.strip_prefix("*.") {
                    src.ends_with(&format!(".{ext}"))
                } else {
                    src.contains(&p)
                }
            })
        });
        let removed = before.saturating_sub(self.records.len());
        if removed > 0 {
            self.flush()?;
        }
        Ok(removed)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
