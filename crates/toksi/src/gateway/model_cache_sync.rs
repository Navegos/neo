// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/model_cache_sync.rs
// created: 2026-11-21

//! Model weight & artifact cache synchronization descriptors (`modelexpress`,
//! `huggingface_hub`, `hf-serve`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCacheEntry {
    pub model_id: String,
    pub revision: String,
    pub local_path: String,
    pub source: String, // modelexpress | huggingface_hub | hf-serve
    pub verified_sha256: Option<String>,
}

#[derive(Debug, Default)]
pub struct ModelCacheRegistry {
    entries: Vec<ModelCacheEntry>,
}

impl ModelCacheRegistry {
    pub fn record(&mut self, entry: ModelCacheEntry) {
        self.entries.push(entry);
    }

    pub fn find_by_model(&self, model_id: &str) -> Option<&ModelCacheEntry> {
        self.entries.iter().find(|e| e.model_id == model_id)
    }
}
