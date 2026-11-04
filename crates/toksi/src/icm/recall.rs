// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/recall.rs
// created: 2026-10-28
// lastModified: 2026-10-31

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

/// Headroom budgeting hook: recalls top memories and active facts as TieredEntry items
/// for direct consumption by the progressive disclosure packer.
pub fn recall_tiered_entries(store: &mut IcmStore, query: Option<&str>, limit: usize) -> Vec<crate::aimemory::TieredEntry> {
    let mut entries = Vec::new();

    // 1. Fetch active facts (decisions, gotchas, architecture slots)
    if let Ok(facts) = store.list_facts(None) {
        for f in facts.into_iter().take(limit) {
            let desc = format!("{}::{} = {}", f.entity, f.key, f.value);
            entries.push(crate::aimemory::TieredEntry {
                key: format!("fact_{}", f.id),
                title: format!("Fact: {}::{}", f.entity, f.key),
                abstract_text: Some(desc.clone()),
                summary_text: Some(desc.clone()),
                full_content: format!("{} (source: {})", desc, f.source),
            });
        }
    }

    // 2. Fetch top memories (query-based or most recent)
    let hits = match query {
        Some(q) if !q.trim().is_empty() => store.search_bm25(q, limit).unwrap_or_default(),
        _ => store.get_recent(limit).unwrap_or_default(),
    };

    for hit in hits {
        let full = hit.memory.raw_excerpt.clone().unwrap_or_else(|| hit.memory.summary.clone());
        entries.push(crate::aimemory::TieredEntry {
            key: format!("icm_{}", hit.memory.id),
            title: format!("Memory ({}): {}", hit.memory.topic, hit.memory.importance.as_str()),
            abstract_text: Some(hit.memory.summary.clone()),
            summary_text: Some(hit.memory.summary.clone()),
            full_content: full,
        });
    }

    entries
}


