// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/gemini_wire.rs
// created: 2026-11-21

//! `python-genai` `v1beta` `thinkingConfig` wire adapter for Google Gemini API.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::headroom::effort::EffortLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    pub thinking_budget: u64,
    #[serde(rename = "includeThoughts")]
    pub include_thoughts: bool,
}

impl From<EffortLevel> for GeminiThinkingConfig {
    fn from(effort: EffortLevel) -> Self {
        Self {
            thinking_budget: effort.gemini_thinking_budget(),
            include_thoughts: effort != EffortLevel::Low,
        }
    }
}

/// Builds a `v1beta` `generateContent` endpoint URL for `model`.
pub fn gemini_v1beta_url(base_url: &str, model: &str, stream: bool) -> String {
    let method = if stream {
        "streamGenerateContent?alt=sse"
    } else {
        "generateContent"
    };
    format!(
        "{}/v1beta/models/{}:{}",
        base_url.trim_end_matches('/'),
        model.trim_start_matches("models/"),
        method
    )
}

/// Injects `generationConfig.thinkingConfig` into a Gemini `v1beta` payload.
pub fn inject_v1beta_thinking_config(payload: &mut Value, effort: EffortLevel) {
    let cfg = GeminiThinkingConfig::from(effort);
    if let Some(obj) = payload.as_object_mut() {
        let gen_cfg = obj.entry("generationConfig").or_insert_with(|| json!({}));
        if let Some(gen_obj) = gen_cfg.as_object_mut() {
            gen_obj.insert(
                "thinkingConfig".to_string(),
                json!({
                    "thinkingBudget": cfg.thinking_budget,
                    "includeThoughts": cfg.include_thoughts,
                }),
            );
        }
    }
}
