// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/state.rs
// created: 2026-11-09

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderControlItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub enabled: bool,
    pub weight: u32,
    pub priority: u32,
    pub active_in_flight: u32,
    pub avg_latency_ms: f64,
    pub cost_per_mtoken: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityTierConfig {
    pub simple_model: String,
    pub medium_model: String,
    pub complex_model: String,
    pub active_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamQuotaGauge {
    pub provider: String,
    pub remaining_requests: i64,
    pub limit_requests: i64,
    pub remaining_tokens: i64,
    pub limit_tokens: i64,
    pub reset_requests_secs: u64,
    pub reset_tokens_secs: u64,
    pub tier_badge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObservationItem {
    pub id: String,
    pub label: String,
    pub installed: bool,
    pub wired: bool,
    pub connected_provider: String,
    pub active_model: String,
    pub latency_breakdown_ms: LatencyBreakdown,
    pub tokens_saved_ground_truth: i64,
    pub cost_saved_usd: f64,
    pub paid_quota_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyBreakdown {
    pub interception_ms: f64,
    pub ast_compression_ms: f64,
    pub upstream_ms: f64,
    pub total_roundtrip_ms: f64,
}

pub struct CockpitState {
    pub providers: RwLock<HashMap<String, ProviderControlItem>>,
    pub complexity: RwLock<ComplexityTierConfig>,
    pub quotas: RwLock<HashMap<String, UpstreamQuotaGauge>>,
}

impl Default for CockpitState {
    fn default() -> Self {
        let mut providers = HashMap::new();
        let default_provs = [
            ("anthropic", "Anthropic Claude", "Frontier", 100, 1, 1.25, 4.5),
            ("openai", "OpenAI GPT-4o", "Frontier", 100, 2, 0.95, 3.8),
            ("gemini", "Google Gemini 2.0", "Frontier", 80, 3, 0.65, 1.5),
            ("deepseek", "DeepSeek-R1 / V3", "Accelerator", 120, 1, 0.28, 0.55),
            ("groq", "Groq LPU Accelerator", "Accelerator", 90, 2, 0.18, 0.40),
            ("cerebras", "Cerebras Wafer", "Accelerator", 60, 3, 0.15, 0.35),
            ("openrouter", "OpenRouter Meta-Proxy", "Aggregator", 70, 4, 1.10, 2.0),
            ("bifrost", "Bifrost Gateway", "Aggregator", 100, 1, 0.85, 1.8),
            ("ollama", "Ollama Local Engine", "Local", 50, 5, 0.0, 0.0),
        ];

        for (id, name, cat, weight, prio, lat, cost) in default_provs {
            providers.insert(
                id.to_string(),
                ProviderControlItem {
                    id: id.to_string(),
                    name: name.to_string(),
                    category: cat.to_string(),
                    enabled: true,
                    weight,
                    priority: prio,
                    active_in_flight: 0,
                    avg_latency_ms: lat * 100.0,
                    cost_per_mtoken: cost,
                },
            );
        }

        let quotas = HashMap::new();

        Self {
            providers: RwLock::new(providers),
            complexity: RwLock::new(ComplexityTierConfig {
                simple_model: "gemini-2.0-flash".into(),
                medium_model: "claude-3-5-haiku".into(),
                complex_model: "claude-3-7-sonnet".into(),
                active_algorithm: "RoundRobin".into(),
            }),
            quotas: RwLock::new(quotas),
        }
    }
}

static COCKPIT_GLOBAL_STATE: std::sync::OnceLock<CockpitState> = std::sync::OnceLock::new();

pub fn global_cockpit_state() -> &'static CockpitState {
    COCKPIT_GLOBAL_STATE.get_or_init(CockpitState::default)
}
