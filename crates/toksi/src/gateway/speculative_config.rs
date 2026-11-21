// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/speculative_config.rs
// created: 2026-11-21

//! Speculative decoding configuration (`SpecForge`, `speculators`, `EAGLE`, `Medusa`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeculativeDecodingConfig {
    pub method: String, // eagle | medusa | draft_model
    pub target_model: String,
    pub draft_model: String,
    pub num_speculative_tokens: u8,
}

impl SpeculativeDecodingConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.target_model.trim().is_empty() || self.draft_model.trim().is_empty() {
            return Err("SpeculativeDecodingConfig requires target_model and draft_model".to_string());
        }
        if self.num_speculative_tokens == 0 || self.num_speculative_tokens > 16 {
            return Err("num_speculative_tokens must be between 1 and 16".to_string());
        }
        Ok(())
    }
}
