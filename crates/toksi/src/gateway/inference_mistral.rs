// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/inference_mistral.rs
// created: 2026-11-21

//! Pure-Rust `mistral.rs` inference engine bridge (zero binary bloat; optional runtime).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MistralEngineConfig {
    pub endpoint: String,
    pub model_id: String,
    pub is_quantized: bool,
    pub in_situ_quant: Option<String>,
}

impl Default for MistralEngineConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:1234/v1".to_string(),
            model_id: "mistral-default".to_string(),
            is_quantized: true,
            in_situ_quant: Some("Q4_K_M".to_string()),
        }
    }
}

pub fn build_mistral_chat_url(cfg: &MistralEngineConfig) -> String {
    format!("{}/chat/completions", cfg.endpoint.trim_end_matches('/'))
}
