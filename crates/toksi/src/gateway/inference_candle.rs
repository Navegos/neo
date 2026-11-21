// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/inference_candle.rs
// created: 2026-11-21

//! Pure-Rust `candle` & `candle-vllm` inference engine bridge (zero binary bloat).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandleDeviceKind {
    Cpu,
    Cuda,
    Metal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandleEngineConfig {
    pub endpoint: String,
    pub model_id: String,
    pub device: CandleDeviceKind,
    pub paged_attention: bool,
}

impl Default for CandleEngineConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:2000/v1".to_string(),
            model_id: "candle-vllm-default".to_string(),
            device: CandleDeviceKind::Cpu,
            paged_attention: true,
        }
    }
}

pub fn build_candle_chat_url(cfg: &CandleEngineConfig) -> String {
    format!("{}/chat/completions", cfg.endpoint.trim_end_matches('/'))
}
