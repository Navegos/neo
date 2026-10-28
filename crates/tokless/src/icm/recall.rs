// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/icm/recall.rs

use std::time::Instant;
use serde::{Deserialize, Serialize};

use crate::icm::schema::SearchHit;
use crate::icm::store::IcmStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallOptions {
    pub query: String,
    pub limit: usize,
    pub topic: Option<String>,
    pub min_weight: Option<f32>,
    pub max_tokens: Option<usize>,
}

impl Default for RecallOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            topic: None,
            min_weight: None,
            max_tokens: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResult {
    pub hits: Vec<SearchHit>,
    pub total: usize,
    pub latency_ms: f64,
    pub query: String,
}

pub fn recall_memories(store: &mut IcmStore, opts: &RecallOptions) -> Result<RecallResult, rusqlite::Error> {
    let start = Instant::now();
    let hits = store.search_bm25(&opts.query, opts.limit * 2)?;

    // In-memory filtration for topic, min_weight, and max_tokens budget
    let mut filtered = Vec::new();
    let mut current_tokens = 0;

    for hit in hits {
        if let Some(ref t) = opts.topic {
            if !hit.memory.topic.eq_ignore_ascii_case(t) {
                continue;
            }
        }
        if let Some(min_w) = opts.min_weight {
            if hit.memory.weight < min_w {
                continue;
            }
        }

        let tokens = estimate_tokens(&hit.memory.summary);
        if let Some(budget) = opts.max_tokens {
            if current_tokens + tokens > budget && !filtered.is_empty() {
                break;
            }
        }

        current_tokens += tokens;
        let mem_id = hit.memory.id.clone();
        filtered.push(hit);

        // Touch memory to refresh last_accessed and access_count
        let _ = store.touch_memory(&mem_id);

        if filtered.len() >= opts.limit {
            break;
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let total = filtered.len();

    Ok(RecallResult {
        hits: filtered,
        total,
        latency_ms: elapsed,
        query: opts.query.clone(),
    })
}

pub fn estimate_tokens(text: &str) -> usize {
    // Zero-LLM deterministic token estimator (~4 chars per token)
    (text.len() + 3) / 4
}

