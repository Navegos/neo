// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/funes_recall.rs
// created: 2026-11-24

//! Pure-Rust hybrid temporal-decay + lexical/salience episodic memory recall (`funes`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunesMemoryEntry {
    pub id: String,
    pub content: String,
    pub salience: f64,
    pub age_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunesRankedHit {
    pub id: String,
    pub content: String,
    pub combined_score: f64,
    pub decay_factor: f64,
    pub lexical_score: f64,
}

/// Ranks episodic memory entries combining exponential recency decay, lexical overlap, and salience.
pub fn rank_recall(
    query: &str,
    entries: &[FunesMemoryEntry],
    half_life_hours: f64,
    top_k: usize,
) -> Vec<FunesRankedHit> {
    let q_terms: Vec<String> = query
        .split_whitespace()
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let hl = half_life_hours.max(1.0);

    let mut hits: Vec<FunesRankedHit> = entries
        .iter()
        .map(|e| {
            let low = e.content.to_ascii_lowercase();
            let matched = q_terms
                .iter()
                .filter(|t| low.contains(t.as_str()))
                .count();
            let lexical_score = if q_terms.is_empty() {
                0.0
            } else {
                matched as f64 / q_terms.len() as f64
            };
            let decay_factor = 0.5_f64.powf(e.age_hours.max(0.0) / hl);
            let combined_score = (0.55 * lexical_score)
                + (0.25 * e.salience.clamp(0.0, 1.0))
                + (0.20 * decay_factor);
            FunesRankedHit {
                id: e.id.clone(),
                content: e.content.clone(),
                combined_score,
                decay_factor,
                lexical_score,
            }
        })
        .collect();

    hits.sort_by(|a, b| b.combined_score.total_cmp(&a.combined_score));
    hits.truncate(top_k);
    hits
}
