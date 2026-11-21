// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/temporal_graph.rs
// created: 2026-11-21

//! Pure-Rust Graphiti-style bi-temporal episodic & semantic knowledge graph.

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemporalFact {
    pub fact_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_at: i64,
    pub invalid_at: Option<i64>,
    pub episode_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Episode {
    pub episode_id: String,
    pub source: String,
    pub body: String,
    pub occurred_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemporalGraph {
    pub episodes: Vec<Episode>,
    pub facts: Vec<TemporalFact>,
}

impl TemporalGraph {
    pub fn open(ai_memory_dir: &Path) -> Result<(Self, PathBuf), String> {
        let dir = ai_memory_dir.join("graphiti");
        fs::create_dir_all(&dir).map_err(|e| format!("graphiti mkdir failed: {e}"))?;
        let path = dir.join("temporal_graph.json");
        if path.exists() {
            let raw = fs::read_to_string(&path).map_err(|e| format!("graphiti read failed: {e}"))?;
            let g: Self =
                serde_json::from_str(&raw).map_err(|e| format!("graphiti parse failed: {e}"))?;
            Ok((g, path))
        } else {
            Ok((Self::default(), path))
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("graphiti serialize failed: {e}"))?;
        fs::write(path, json).map_err(|e| format!("graphiti write failed: {e}"))
    }

    pub fn add_episode(&mut self, episode: Episode) {
        self.episodes.push(episode);
    }

    pub fn add_fact(&mut self, fact: TemporalFact) {
        self.facts.push(fact);
    }

    pub fn invalidate_fact(&mut self, fact_id: &str, invalid_at: i64) -> bool {
        if let Some(f) = self.facts.iter_mut().find(|f| f.fact_id == fact_id) {
            f.invalid_at = Some(invalid_at);
            true
        } else {
            false
        }
    }

    /// Point-in-time query returning facts valid at `ts`.
    pub fn query_at(&self, subject_or_query: &str, ts: i64) -> Vec<&TemporalFact> {
        let q = subject_or_query.to_ascii_lowercase();
        self.facts
            .iter()
            .filter(|f| {
                f.valid_at <= ts
                    && f.invalid_at.is_none_or(|end| ts < end)
                    && (q.is_empty()
                        || f.subject.to_ascii_lowercase().contains(&q)
                        || f.predicate.to_ascii_lowercase().contains(&q)
                        || f.object.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }
}
