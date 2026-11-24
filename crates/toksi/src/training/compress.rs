// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/training/compress.rs
// created: 2026-11-24

//! Pillar 7 Compression & KV-Cache Eviction contracts (`kvpress`, `bitsandbytes`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KvPressPolicy {
    SnapKv,
    ExpectedAttention,
    Knorm,
    StreamingLlm,
    ObservedAttention,
}

impl KvPressPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SnapKv => "snap_kv",
            Self::ExpectedAttention => "expected_attention",
            Self::Knorm => "knorm",
            Self::StreamingLlm => "streaming_llm",
            Self::ObservedAttention => "observed_attention",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "snapkv" | "snap_kv" => Some(Self::SnapKv),
            "expected" | "expected_attention" => Some(Self::ExpectedAttention),
            "knorm" => Some(Self::Knorm),
            "streaming" | "streaming_llm" => Some(Self::StreamingLlm),
            "observed" | "observed_attention" => Some(Self::ObservedAttention),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KvPressPlan {
    pub policy: KvPressPolicy,
    pub compression_ratio: f64,
    pub context_len: usize,
    pub retained_tokens: usize,
    pub estimated_vram_saved_pct: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BnbQuantMethod {
    Nf4,
    Fp4,
    Int8Llm,
}

impl BnbQuantMethod {
    pub const fn bits_per_param(self) -> f64 {
        match self {
            Self::Nf4 | Self::Fp4 => 4.25,
            Self::Int8Llm => 8.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BnbQuantConfig {
    pub method: BnbQuantMethod,
    pub double_quant: bool,
    pub compute_dtype: String,
}

impl Default for BnbQuantConfig {
    fn default() -> Self {
        Self {
            method: BnbQuantMethod::Nf4,
            double_quant: true,
            compute_dtype: "bfloat16".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BnbQuantPlan {
    pub model_id: String,
    pub params_billions: f64,
    pub method: BnbQuantMethod,
    pub double_quant: bool,
    pub estimated_vram_gb: f64,
}

/// Computes bounded KV-cache eviction savings under `kvpress` policies.
pub fn plan_kvpress(policy: KvPressPolicy, compression_ratio: f64, context_len: usize) -> KvPressPlan {
    let ratio = compression_ratio.clamp(0.05, 0.90);
    let kept = ((context_len as f64) * (1.0 - ratio)).round().max(1.0) as usize;
    KvPressPlan {
        policy,
        compression_ratio: ratio,
        context_len,
        retained_tokens: kept.min(context_len.max(1)),
        estimated_vram_saved_pct: (ratio * 1000.0).round() / 10.0,
    }
}

/// Estimates quantized footprint for `bitsandbytes` QLoRA/int8 configurations.
pub fn plan_bnb_quant(model_id: &str, params_billions: f64, cfg: &BnbQuantConfig) -> BnbQuantPlan {
    let p = params_billions.max(0.1);
    let mut bytes_per_param = cfg.method.bits_per_param() / 8.0;
    if cfg.double_quant && matches!(cfg.method, BnbQuantMethod::Nf4 | BnbQuantMethod::Fp4) {
        bytes_per_param *= 0.92;
    }
    let gb = ((p * bytes_per_param) * 100.0).round() / 100.0;
    BnbQuantPlan {
        model_id: model_id.to_string(),
        params_billions: p,
        method: cfg.method,
        double_quant: cfg.double_quant,
        estimated_vram_gb: gb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kvpress_and_bnb_quant_planners() {
        let kv = plan_kvpress(KvPressPolicy::SnapKv, 0.5, 32768);
        assert_eq!(kv.retained_tokens, 16384);
        assert!((kv.estimated_vram_saved_pct - 50.0).abs() < 0.1);

        let nf4 = plan_bnb_quant("llama-3.1-8b", 8.0, &BnbQuantConfig::default());
        assert!(nf4.estimated_vram_gb < 5.0);
    }
}
