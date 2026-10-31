// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/telemetry/cost.rs

use std::collections::HashMap;

/// Pricing in USD per 1,000,000 tokens
#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input_per_m: f64,
    pub output_per_m: f64,
    pub cached_per_m: f64,
}

impl ModelPricing {
    pub const fn new(input_per_m: f64, output_per_m: f64, cached_per_m: f64) -> Self {
        Self {
            input_per_m,
            output_per_m,
            cached_per_m,
        }
    }
}

/// In-tree model catalog pricing table
pub fn lookup_pricing(model: &str) -> ModelPricing {
    let lower = model.to_lowercase();

    // Anthropic Claude
    if lower.contains("claude-3-7") || lower.contains("claude-3.7") {
        ModelPricing::new(3.00, 15.00, 0.30)
    } else if lower.contains("claude-3-5-sonnet") || lower.contains("claude-3.5-sonnet") || lower.contains("sonnet") {
        ModelPricing::new(3.00, 15.00, 0.30)
    } else if lower.contains("opus") {
        ModelPricing::new(15.00, 75.00, 1.50)
    } else if lower.contains("haiku") {
        ModelPricing::new(0.25, 1.25, 0.025)
    // OpenAI
    } else if lower.contains("gpt-4o-mini") {
        ModelPricing::new(0.15, 0.60, 0.075)
    } else if lower.contains("gpt-4o") {
        ModelPricing::new(2.50, 10.00, 1.25)
    } else if lower.contains("o1") {
        ModelPricing::new(15.00, 60.00, 7.50)
    } else if lower.contains("o3-mini") {
        ModelPricing::new(1.10, 4.40, 0.55)
    // Google Gemini
    } else if lower.contains("gemini-2.0-flash") || lower.contains("gemini-2-flash") {
        ModelPricing::new(0.10, 0.40, 0.025)
    } else if lower.contains("gemini-1.5-flash") {
        ModelPricing::new(0.075, 0.30, 0.01875)
    } else if lower.contains("gemini-1.5-pro") {
        ModelPricing::new(1.25, 5.00, 0.3125)
    // DeepSeek
    } else if lower.contains("deepseek-reasoner") || lower.contains("deepseek-r1") || lower.contains("r1") {
        ModelPricing::new(0.55, 2.19, 0.14)
    } else if lower.contains("deepseek-chat") || lower.contains("deepseek-v3") || lower.contains("deepseek") {
        ModelPricing::new(0.14, 0.28, 0.014)
    // xAI Grok
    } else if lower.contains("grok-3") {
        ModelPricing::new(3.00, 15.00, 0.75)
    } else if lower.contains("grok-2") || lower.contains("grok") {
        ModelPricing::new(2.00, 10.00, 0.50)
    // Meta Llama 3
    } else if lower.contains("llama-3.3") || lower.contains("llama-3-70b") {
        ModelPricing::new(0.60, 0.80, 0.10)
    // Default baseline pricing
    } else {
        ModelPricing::new(1.00, 3.00, 0.20)
    }
}

/// Calculate USD cost for tokens
pub fn calculate_cost(
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
) -> f64 {
    let pricing = lookup_pricing(model);
    let uncached_in = input_tokens.saturating_sub(cached_tokens);
    let in_cost = (uncached_in as f64 / 1_000_000.0) * pricing.input_per_m;
    let cached_cost = (cached_tokens as f64 / 1_000_000.0) * pricing.cached_per_m;
    let out_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_per_m;
    in_cost + cached_cost + out_cost
}

/// Calculate USD savings for compressed or saved tokens
pub fn calculate_savings(model: &str, tokens_saved: u64) -> f64 {
    let pricing = lookup_pricing(model);
    (tokens_saved as f64 / 1_000_000.0) * pricing.input_per_m
}

/// Financial tracking per provider, model, agent, and virtual key
#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    spend_by_key: HashMap<String, f64>,
    savings_by_key: HashMap<String, f64>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_transaction(
        &mut self,
        provider: &str,
        model: &str,
        agent: &str,
        virtual_key: &str,
        spend_usd: f64,
        savings_usd: f64,
    ) {
        let key = format!("{}:{}:{}:{}", provider, model, agent, virtual_key);
        *self.spend_by_key.entry(key.clone()).or_insert(0.0) += spend_usd;
        *self.savings_by_key.entry(key).or_insert(0.0) += savings_usd;
    }

    pub fn total_spend_usd(&self) -> f64 {
        self.spend_by_key.values().sum()
    }

    pub fn total_savings_usd(&self) -> f64 {
        self.savings_by_key.values().sum()
    }
}

