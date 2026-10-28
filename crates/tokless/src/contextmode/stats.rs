// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/stats.rs
// created: 2026-10-27

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub calls: HashMap<String, usize>,
    pub bytes_returned: HashMap<String, usize>,
    pub bytes_indexed: usize,
    pub bytes_sandboxed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_bytes_saved: usize,
    pub session_start: i64,
}

impl Default for ContextStats {
    fn default() -> Self {
        Self {
            calls: HashMap::new(),
            bytes_returned: HashMap::new(),
            bytes_indexed: 0,
            bytes_sandboxed: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_bytes_saved: 0,
            session_start: chrono::Utc::now().timestamp_millis(),
        }
    }
}

impl ContextStats {
    pub fn record_call(&mut self, tool: &str, returned_bytes: usize) {
        *self.calls.entry(tool.to_string()).or_insert(0) += 1;
        *self.bytes_returned.entry(tool.to_string()).or_insert(0) += returned_bytes;
    }

    pub fn record_sandbox_execution(&mut self, consumed_bytes: usize) {
        self.bytes_sandboxed += consumed_bytes;
    }

    pub fn record_indexed(&mut self, bytes: usize) {
        self.bytes_indexed += bytes;
    }

    pub fn record_cache_hit(&mut self, saved_bytes: usize) {
        self.cache_hits += 1;
        self.cache_bytes_saved += saved_bytes;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn total_bytes_returned(&self) -> usize {
        self.bytes_returned.values().sum()
    }

    pub fn estimated_tokens_saved(&self) -> usize {
        let raw_avoided = self.bytes_sandboxed + self.cache_bytes_saved;
        let returned = self.total_bytes_returned();
        let net = raw_avoided.saturating_sub(returned);
        net / 4
    }
}

pub static GLOBAL_STATS: LazyLock<Mutex<ContextStats>> = LazyLock::new(|| Mutex::new(ContextStats::default()));

pub fn get_stats() -> ContextStats {
    GLOBAL_STATS.lock().map(|s| s.clone()).unwrap_or_default()
}

pub fn record_tool_call(tool: &str, returned_bytes: usize) {
    if let Ok(mut guard) = GLOBAL_STATS.lock() {
        guard.record_call(tool, returned_bytes);
    }
}

pub fn record_sandbox_bytes(bytes: usize) {
    if let Ok(mut guard) = GLOBAL_STATS.lock() {
        guard.record_sandbox_execution(bytes);
    }
}

pub fn record_indexed_bytes(bytes: usize) {
    if let Ok(mut guard) = GLOBAL_STATS.lock() {
        guard.record_indexed(bytes);
    }
}

pub fn record_cache_hit(bytes: usize) {
    if let Ok(mut guard) = GLOBAL_STATS.lock() {
        guard.record_cache_hit(bytes);
    }
}

pub fn record_cache_miss() {
    if let Ok(mut guard) = GLOBAL_STATS.lock() {
        guard.record_cache_miss();
    }
}
