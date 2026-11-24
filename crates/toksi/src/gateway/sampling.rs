// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/sampling.rs
// created: 2026-11-24

//! Per-provider & per-model sampling compatibility matrix (`temperature`, `top_p`, `top_k`)
//! honoring `full_ai_ecosystem_..._protocols_gateways_and_quota_architecture.md`.
//!
//! Unsupported sampling parameters are silently omitted rather than crashing upstream APIs.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::headroom::effort::EffortLevel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct SamplingParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderSamplingSupport {
    pub allows_temperature: bool,
    pub requires_unit_temperature_on_thinking: bool,
    pub allows_top_p: bool,
    pub allows_top_k: bool,
}

/// Determines exact sampling support for `(provider, model, effort)`.
pub fn sampling_support_for(
    provider: &str,
    model: &str,
    effort: EffortLevel,
) -> ProviderSamplingSupport {
    let p = provider.trim().to_ascii_lowercase();
    let m = model.trim().to_ascii_lowercase();

    let is_openai_reasoning = m.starts_with("o1")
        || m.starts_with("o3")
        || m.starts_with("o4")
        || m.contains("gpt-5")
        || m.contains("gpt-6-astra")
        || m.contains("gpt-6.2")
        || m.contains("gpt-6.1");

    if is_openai_reasoning && !m.contains("chat") {
        return ProviderSamplingSupport {
            allows_temperature: false,
            requires_unit_temperature_on_thinking: false,
            allows_top_p: false,
            allows_top_k: false,
        };
    }

    if p == "deepseek" || m.contains("deepseek-r1") || m.contains("deepseek-reasoner") {
        let is_reasoner = m.contains("r1") || m.contains("reasoner");
        return ProviderSamplingSupport {
            allows_temperature: !is_reasoner,
            requires_unit_temperature_on_thinking: false,
            allows_top_p: !is_reasoner,
            allows_top_k: false,
        };
    }

    if p == "anthropic" || m.contains("claude") {
        let thinking_active = effort != EffortLevel::Low && !m.contains("haiku");
        return ProviderSamplingSupport {
            allows_temperature: true,
            requires_unit_temperature_on_thinking: thinking_active,
            allows_top_p: true,
            allows_top_k: true,
        };
    }

    if p == "google" || p == "gemini" || p == "vertex" || p == "agy" || m.contains("gemini") {
        let unit_baseline = m.contains("gemini-3") || m.contains("gemini-4");
        return ProviderSamplingSupport {
            allows_temperature: true,
            requires_unit_temperature_on_thinking: unit_baseline,
            allows_top_p: true,
            allows_top_k: true,
        };
    }

    let is_local = matches!(
        p.as_str(),
        "ollama" | "llama.cpp" | "vllm" | "sglang" | "candle" | "mistral.rs" | "mlx" | "local"
    );
    if is_local {
        return ProviderSamplingSupport {
            allows_temperature: true,
            requires_unit_temperature_on_thinking: false,
            allows_top_p: true,
            allows_top_k: true,
        };
    }

    // Standard OpenAI-compatible remote endpoints: temperature & top_p allowed; top_k silently omitted.
    ProviderSamplingSupport {
        allows_temperature: true,
        requires_unit_temperature_on_thinking: false,
        allows_top_p: true,
        allows_top_k: false,
    }
}

/// Filters `(temperature, top_p, top_k)` into a safe JSON map, silently omitting unsupported keys.
pub fn filter_sampling_params_for_provider(
    provider: &str,
    model: &str,
    effort: EffortLevel,
    params: &SamplingParams,
) -> Map<String, Value> {
    let support = sampling_support_for(provider, model, effort);
    let mut out = Map::new();

    if support.allows_temperature {
        if support.requires_unit_temperature_on_thinking {
            out.insert("temperature".to_string(), json!(1.0));
        } else if let Some(t) = params.temperature {
            out.insert("temperature".to_string(), json!(t.clamp(0.0, 2.0)));
        }
    }

    if support.allows_top_p
        && let Some(tp) = params.top_p
    {
        if support.requires_unit_temperature_on_thinking && (provider.contains("anthropic") || model.contains("claude")) {
            if tp >= 0.95 {
                out.insert("top_p".to_string(), json!(tp.clamp(0.0, 1.0)));
            }
        } else {
            out.insert("top_p".to_string(), json!(tp.clamp(0.0, 1.0)));
        }
    }

    if support.allows_top_k
        && let Some(tk) = params.top_k
        && tk > 0
    {
        out.insert("top_k".to_string(), json!(tk));
    }

    out
}

/// Applies provider-safe sampling compatibility filtering directly to an outgoing JSON body.
pub fn apply_sampling_params_to_payload(
    provider: &str,
    model: &str,
    effort: EffortLevel,
    params: &SamplingParams,
    payload: &mut Value,
) {
    let Some(obj) = payload.as_object_mut() else {
        return;
    };
    let effective = SamplingParams {
        temperature: params
            .temperature
            .or_else(|| obj.get("temperature").and_then(Value::as_f64)),
        top_p: params
            .top_p
            .or_else(|| obj.get("top_p").and_then(Value::as_f64)),
        top_k: params
            .top_k
            .or_else(|| obj.get("top_k").and_then(Value::as_u64).map(|v| v as u32)),
    };

    obj.remove("temperature");
    obj.remove("top_p");
    obj.remove("top_k");

    for (k, v) in filter_sampling_params_for_provider(provider, model, effort, &effective) {
        obj.insert(k, v);
    }
}
