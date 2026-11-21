// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/model_catalog.rs
// created: 2026-11-21

//! Unified local & cloud model catalog bridging all 26 Section 2.7 local/Rust
//! inference engines (`ollama`, `llama.cpp`, `jan`, `Mysti`, `koboldcpp`, `llamafile`,
//! `LocalAI`, `lmdeploy`, `tabbyAPI`, `mlc-llm`, `exo`, `mlx`, `FastChat`,
//! `TensorRT-LLM`, `exllamav2`, `LitServe`, `candle`, `mistral.rs`, `dynamo`, etc.).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::local_engines::SECTION_2_7_LOCAL_ENGINES;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogModelEntry {
    pub model_id: String,
    pub engine_id: String,
    pub context_window: usize,
    pub pure_rust: bool,
}

pub fn list_supported_local_engine_ids() -> Vec<&'static str> {
    SECTION_2_7_LOCAL_ENGINES.iter().map(|e| e.id).collect()
}

pub fn build_default_local_catalog_entry(
    engine_id: &str,
    model_id: &str,
    context_window: usize,
) -> Option<CatalogModelEntry> {
    let desc = SECTION_2_7_LOCAL_ENGINES
        .iter()
        .find(|e| e.id.eq_ignore_ascii_case(engine_id.trim()))?;
    Some(CatalogModelEntry {
        model_id: model_id.to_string(),
        engine_id: desc.id.to_string(),
        context_window,
        pure_rust: desc.pure_rust,
    })
}
