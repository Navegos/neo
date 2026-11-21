// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/letta_memgpt.rs
// created: 2026-11-21

//! Pure-Rust Letta / MemGPT OS-style tiered core, recall, and archival memory paging.

#![deny(unsafe_code)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMemoryBlock {
    pub label: String,
    pub value: String,
    pub max_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalEntry {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LettaMemoryManager {
    pub core_blocks: HashMap<String, CoreMemoryBlock>,
    pub recall_queue: Vec<String>,
    pub archival: Vec<ArchivalEntry>,
}

impl LettaMemoryManager {
    pub fn new() -> Self {
        let mut mgr = Self::default();
        mgr.set_core_block("persona", "Toksi 35-component <toksi-siagent>", 2048);
        mgr.set_core_block("human", "Workspace developer", 2048);
        mgr
    }

    pub fn set_core_block(&mut self, label: &str, value: &str, max_chars: usize) {
        let bounded: String = value.chars().take(max_chars).collect();
        self.core_blocks.insert(
            label.to_string(),
            CoreMemoryBlock {
                label: label.to_string(),
                value: bounded,
                max_chars,
            },
        );
    }

    pub fn core_memory_append(&mut self, label: &str, addition: &str) -> Result<(), String> {
        let block = self
            .core_blocks
            .get_mut(label)
            .ok_or_else(|| format!("core block '{label}' not found"))?;
        let candidate = if block.value.is_empty() {
            addition.to_string()
        } else {
            format!("{}\n{}", block.value, addition)
        };
        if candidate.chars().count() > block.max_chars {
            // Page oldest lines out to archival before appending
            let overflow_id = format!("arch-{}", chrono::Utc::now().timestamp_millis());
            self.archival.push(ArchivalEntry {
                id: overflow_id,
                content: block.value.clone(),
                tags: vec![label.to_string(), "paged_out".to_string()],
                created_at: chrono::Utc::now().timestamp(),
            });
            block.value = addition.chars().take(block.max_chars).collect();
        } else {
            block.value = candidate;
        }
        Ok(())
    }

    pub fn archival_insert(&mut self, id: &str, content: &str, tags: Vec<String>) {
        self.archival.push(ArchivalEntry {
            id: id.to_string(),
            content: content.to_string(),
            tags,
            created_at: chrono::Utc::now().timestamp(),
        });
    }

    pub fn archival_search(&self, query: &str, limit: usize) -> Vec<&ArchivalEntry> {
        let q = query.to_ascii_lowercase();
        self.archival
            .iter()
            .filter(|e| {
                e.content.to_ascii_lowercase().contains(&q)
                    || e.tags.iter().any(|t| t.to_ascii_lowercase().contains(&q))
            })
            .take(limit)
            .collect()
    }
}
