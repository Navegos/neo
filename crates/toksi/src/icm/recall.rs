// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/recall.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use std::time::Instant;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::aimemory::tiered::TieredEntry;
use crate::icm::schema::{IcmResult, Memory, SearchHit};
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

pub fn recall_memories(store: &mut IcmStore, opts: &RecallOptions) -> IcmResult<RecallResult> {
    let start = Instant::now();
    let now = Utc::now();

    // Check if query contains temporal components
    let ranked_hits = store.search_fused(&opts.query, opts.limit * 2, now)?;

    let hits: Vec<SearchHit> = if !ranked_hits.is_empty() {
        ranked_hits
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchHit {
                memory: r.memory,
                score: r.score,
                rank: i + 1,
            })
            .collect()
    } else {
        store.search_bm25(&opts.query, opts.limit * 2)?
    };

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
pub fn recall_tiered_entries(store: &mut IcmStore, query: Option<&str>, limit: usize) -> Vec<TieredEntry> {
    let mut entries = Vec::new();

    let memories: Vec<Memory> = match query {
        Some(q) if !q.trim().is_empty() => {
            let opts = RecallOptions {
                query: q.to_string(),
                limit,
                ..Default::default()
            };
            recall_memories(store, &opts)
                .map(|r| r.hits.into_iter().map(|h| h.memory).collect())
                .unwrap_or_default()
        }
        _ => store
            .get_recent(limit)
            .map(|hits| hits.into_iter().map(|h| h.memory).collect())
            .unwrap_or_default(),
    };

    for m in memories {
        let title = format!("Memory: {} ({})", m.topic, m.importance.as_str());
        let short_desc = if m.summary.len() > 120 {
            format!("{}...", &m.summary[..120])
        } else {
            m.summary.clone()
        };
        entries.push(TieredEntry {
            key: format!("icm_mem_{}", m.id),
            title,
            abstract_text: Some(short_desc),
            summary_text: Some(m.summary.clone()),
            full_content: format!("Topic: {}\nImportance: {}\nSummary: {}\nExcerpt: {}", m.topic, m.importance.as_str(), m.summary, m.raw_excerpt.as_deref().unwrap_or("None")),
        });
    }

    // Include top active facts
    if let Ok(facts) = store.list_facts(None) {
        for f in facts.into_iter().take(limit) {
            entries.push(TieredEntry {
                key: format!("icm_fact_{}", f.id),
                title: format!("Fact: {}::{}", f.entity, f.key),
                abstract_text: Some(format!("{} = {}", f.key, f.value)),
                summary_text: Some(format!("Entity {} property {}: {}", f.entity, f.key, f.value)),
                full_content: format!("Entity: {}\nKey: {}\nValue: {}\nSource: {}", f.entity, f.key, f.value, f.source),
            });
        }
    }

    entries
}
