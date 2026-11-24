// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/serving.rs
// created: 2026-11-24

//! Pure-Rust / multimodal local serving descriptors & `llguidance` structured JSON constraint validator.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::gateway::local_engines::lookup_local_engine;
use crate::headroom::effort::EffortLevel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServingProfile {
    pub engine: String,
    pub port: u16,
    pub model: String,
    pub modality: String,
    pub pure_rust: bool,
    pub effort: EffortLevel,
    pub endpoint_url: String,
}

fn infer_modality(engine_id: &str) -> &'static str {
    let low = engine_id.to_ascii_lowercase();
    if low.contains("whisper") {
        "audio-asr"
    } else if low.contains("stable-diffusion") {
        "image-diffusion"
    } else {
        "text-multimodal"
    }
}

pub fn build_serving_profile(
    engine: &str,
    model: &str,
    effort: EffortLevel,
) -> Result<ServingProfile, String> {
    let desc = lookup_local_engine(engine)
        .ok_or_else(|| format!("Unknown local serving engine '{engine}'"))?;
    let port = if desc.default_port == 0 { 8080 } else { desc.default_port };
    Ok(ServingProfile {
        engine: desc.id.to_string(),
        port,
        model: model.trim().to_string(),
        modality: infer_modality(desc.id).to_string(),
        pure_rust: desc.pure_rust,
        effort,
        endpoint_url: format!("http://127.0.0.1:{}/v1", port),
    })
}

/// Deterministic `llguidance`-inspired JSON schema field constraint validator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlguidanceJsonMask {
    pub required_fields: Vec<String>,
    pub allow_additional_properties: bool,
}

impl LlguidanceJsonMask {
    pub fn validate_json_output(&self, raw: &str) -> Result<Value, String> {
        let val: Value =
            serde_json::from_str(raw.trim()).map_err(|e| format!("Invalid constrained JSON: {e}"))?;
        let Some(obj) = val.as_object() else {
            return Err("Constrained output must be a JSON object".to_string());
        };
        for req in &self.required_fields {
            if !obj.contains_key(req) {
                return Err(format!("Constrained JSON missing required field '{req}'"));
            }
        }
        if !self.allow_additional_properties {
            for k in obj.keys() {
                if !self.required_fields.iter().any(|r| r == k) {
                    return Err(format!("Constrained JSON rejected undeclared field '{k}'"));
                }
            }
        }
        Ok(val)
    }
}
