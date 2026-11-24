// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/router.rs
// created: 2026-11-13
//
// 7-Dimensional Gateway Router ported from mofa-engine-core (`Router::route_ranked`).

use serde::{Deserialize, Serialize};

use crate::gateway::circuit_breaker::CircuitBreakerRegistry;
use crate::gateway::providers::{ProviderCategory, ProviderId};

/// Sensitivity classification of payload data (controls local-only enforcement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DataClass {
    #[default]
    Public,
    Internal,
    Confidential,
}

impl DataClass {
    /// Returns true if payload must remain on a local inference engine.
    pub fn requires_local(self) -> bool {
        matches!(self, Self::Confidential)
    }
}

/// Optimization objective for the 7-D scoring function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Prefer {
    Latency,
    Cost,
    Quality,
    #[default]
    Balanced,
}

/// Target reasoning effort tier used to rank candidate endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ReasoningEffort {
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
}

/// Quality gate enforced by the 7-D router and cascade failover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub min_capability_score: f64,
    pub require_non_empty: bool,
    pub allow_tool_calls: bool,
}

impl Default for QualityGate {
    fn default() -> Self {
        Self {
            min_capability_score: 0.0,
            require_non_empty: true,
            allow_tool_calls: true,
        }
    }
}

impl QualityGate {
    pub fn passes_candidate(&self, candidate: &RouteCandidate) -> bool {
        candidate.capability_score >= self.min_capability_score
    }

    pub fn passes_response(&self, resp: &crate::gateway::providers::UnifiedResponse) -> bool {
        if !self.require_non_empty {
            return true;
        }
        if !resp.content.trim().is_empty() {
            return true;
        }
        if self.allow_tool_calls
            && (resp.raw.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("tool_calls")).is_some()
                || resp.raw.get("content").and_then(|c| c.as_array()).is_some_and(|a| !a.is_empty()))
        {
            return true;
        }
        false
    }
}

/// Routing policy passed with each gateway inference request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingPolicy {
    pub data_class: DataClass,
    pub prefer: Prefer,
    pub effort: Option<ReasoningEffort>,
    pub max_latency_ms: Option<u32>,
    pub budget_bytes: Option<u64>,
    pub required_tools: bool,
    #[serde(default)]
    pub quality_gate: Option<QualityGate>,
}

/// Candidate endpoint descriptor evaluated by [`Router::route_ranked`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteCandidate {
    pub provider: ProviderId,
    pub model: String,
    pub is_local: bool,
    pub healthy: bool,
    pub latency_ms: f64,
    pub cost_per_mtoken_usd: f64,
    pub capability_score: f64,
    pub priority: u32,
    pub active_in_flight: u32,
    pub max_concurrency: u32,
    pub estimated_memory_bytes: u64,
    pub supports_tools: bool,
    pub supported_effort: ReasoningEffort,
    #[serde(default)]
    pub base_url: Option<String>,
}

impl RouteCandidate {
    /// Build a baseline descriptor from a `ProviderId` using conservative defaults.
    pub fn for_provider(provider: ProviderId, model: impl Into<String>) -> Self {
        let is_local = provider.category() == ProviderCategory::Local;
        Self {
            provider,
            model: model.into(),
            is_local,
            healthy: true,
            latency_ms: if is_local { 35.0 } else { 220.0 },
            cost_per_mtoken_usd: if is_local { 0.0 } else { 3.0 },
            capability_score: match provider.category() {
                ProviderCategory::Frontier => 0.95,
                ProviderCategory::Accelerator => 0.88,
                ProviderCategory::Aggregator => 0.85,
                ProviderCategory::Local => 0.78,
                _ => 0.80,
            },
            priority: if is_local { 80 } else { 60 },
            active_in_flight: 0,
            max_concurrency: 32,
            estimated_memory_bytes: 0,
            supports_tools: true,
            supported_effort: ReasoningEffort::Medium,
            base_url: None,
        }
    }
}

/// Ranked routing result returned with the 7-D composite score breakdown.
#[derive(Debug, Clone)]
pub struct RankedRoute {
    pub candidate: RouteCandidate,
    pub score: f64,
}

const TIER_MATCH_BONUS: f64 = 5_000.0;

/// 7-Dimensional inference router (`mofa-engine` port).
#[derive(Debug, Default)]
pub struct Router;

impl Router {
    pub fn new() -> Self {
        Self
    }

    /// Filter and rank candidates across all 7 scoring dimensions:
    /// `residency * 1000 + health * 800 + locality * 100 + cost * 50 + capability * 200 + priority * 25 + capacity * 20 + tier_bonus`
    pub fn route_ranked(
        &self,
        candidates: &[RouteCandidate],
        policy: &RoutingPolicy,
        breakers: Option<&CircuitBreakerRegistry>,
    ) -> Vec<RankedRoute> {
        let mut ranked: Vec<RankedRoute> = candidates
            .iter()
            .filter(|c| {
                if !c.healthy {
                    return false;
                }
                if policy.data_class.requires_local() && !c.is_local {
                    return false;
                }
                if policy.required_tools && !c.supports_tools {
                    return false;
                }
                if let Some(max_lat) = policy.max_latency_ms {
                    if c.latency_ms > max_lat as f64 {
                        return false;
                    }
                }
                if let Some(budget) = policy.budget_bytes {
                    if c.estimated_memory_bytes > budget {
                        return false;
                    }
                }
                if let Some(reg) = breakers {
                    if reg.is_open(c.provider) {
                        return false;
                    }
                }
                if let Some(ref gate) = policy.quality_gate {
                    if !gate.passes_candidate(c) {
                        return false;
                    }
                }
                true
            })
            .map(|c| {
                let score = score_candidate(c, policy);
                RankedRoute {
                    candidate: c.clone(),
                    score,
                }
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }
}

fn score_candidate(c: &RouteCandidate, policy: &RoutingPolicy) -> f64 {
    // 1. Residency compliance (guaranteed non-zero by filter, bonus for strict local match)
    let residency = if c.is_local { 1.0 } else if !policy.data_class.requires_local() { 0.8 } else { 0.0 };

    // 2. Health / headroom (open circuit filtered above)
    let health = if c.healthy { 1.0 } else { 0.0 };

    // 3. Locality / latency proximity
    let base_locality = 1.0 / (1.0 + (c.latency_ms.max(0.0) / 100.0));
    let locality_weight = match policy.prefer {
        Prefer::Latency => 2.5,
        Prefer::Balanced => 1.0,
        _ => 0.7,
    };

    // 4. Cost efficiency
    let base_cost = 1.0 / (1.0 + c.cost_per_mtoken_usd.max(0.0));
    let cost_weight = match policy.prefer {
        Prefer::Cost => 3.0,
        Prefer::Balanced => 1.0,
        _ => 0.6,
    };

    // 5. Capability score
    let capability = c.capability_score.clamp(0.0, 1.0);
    let capability_weight = match policy.prefer {
        Prefer::Quality => 2.5,
        Prefer::Balanced => 1.0,
        _ => 0.8,
    };

    // 6. Configured priority normalized
    let priority = (c.priority as f64 / 100.0).clamp(0.0, 2.0);

    // 7. Remaining capacity ratio
    let capacity = if c.max_concurrency == 0 {
        0.0
    } else {
        1.0 - (c.active_in_flight.min(c.max_concurrency) as f64 / c.max_concurrency as f64)
    };

    let tier_bonus = policy
        .effort
        .map(|eff| if c.supported_effort == eff { TIER_MATCH_BONUS } else { 0.0 })
        .unwrap_or(0.0);

    residency * 1000.0
        + health * 800.0
        + (base_locality * locality_weight) * 100.0
        + (base_cost * cost_weight) * 50.0
        + (capability * capability_weight) * 200.0
        + priority * 25.0
        + capacity * 20.0
        + tier_bonus
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRouteRule {
    pub intent: String,
    pub keywords: Vec<String>,
    pub target_provider: ProviderId,
    pub target_model: String,
    pub preferred_effort: ReasoningEffort,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticRouter {
    pub rules: Vec<SemanticRouteRule>,
}

impl SemanticRouter {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_default_rules() -> Self {
        let mut r = Self::new();
        r.add_rule(SemanticRouteRule {
            intent: "architecture_reasoning".to_string(),
            keywords: vec!["architecture".into(), "refactor".into(), "theorem".into(), "proof".into()],
            target_provider: ProviderId::Anthropic,
            target_model: "claude-sonnet-4".to_string(),
            preferred_effort: ReasoningEffort::High,
        });
        r.add_rule(SemanticRouteRule {
            intent: "fast_lookup".to_string(),
            keywords: vec!["summarize".into(), "status".into(), "list".into(), "grep".into()],
            target_provider: ProviderId::Gemini,
            target_model: "gemini-2.5-flash".to_string(),
            preferred_effort: ReasoningEffort::Low,
        });
        r
    }

    pub fn add_rule(&mut self, rule: SemanticRouteRule) {
        self.rules.push(rule);
    }

    pub fn match_prompt(&self, prompt: &str) -> Option<&SemanticRouteRule> {
        let low = prompt.to_ascii_lowercase();
        self.rules.iter().find(|rule| {
            rule.keywords
                .iter()
                .any(|kw| !kw.is_empty() && low.contains(&kw.to_ascii_lowercase()))
        })
    }
}

/// Deterministic `prompt-ops`-compatible prompt normalizer collapsing redundant whitespace and boilerplate.
pub fn optimize_prompt_ops(prompt: &str) -> String {
    let mut cleaned = prompt.trim().to_string();
    for prefix in ["Please help me to ", "Could you please ", "Kindly "] {
        if cleaned.starts_with(prefix) {
            cleaned = cleaned[prefix.len()..].trim_start().to_string();
        }
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Selects a failover LAN `Personal-AI-Router` (`PAIR`) peer endpoint when cloud routes saturate.
pub fn select_lan_pair_failover_endpoint(
    mesh: &crate::cluster::LanPairMesh,
    required_vram_gb: u32,
    model: &str,
) -> Option<String> {
    mesh.select_best_peer(required_vram_gb, model)
        .map(|p| p.endpoint.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidential_data_enforces_local_only() {
        let router = Router::new();
        let candidates = vec![
            RouteCandidate::for_provider(ProviderId::OpenAI, "gpt-4o"),
            RouteCandidate::for_provider(ProviderId::Ollama, "qwen2.5-coder"),
        ];
        let policy = RoutingPolicy {
            data_class: DataClass::Confidential,
            ..Default::default()
        };
        let ranked = router.route_ranked(&candidates, &policy, None);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].candidate.provider, ProviderId::Ollama);
    }

    #[test]
    fn open_circuit_is_excluded_from_ranked_routes() {
        let breakers = CircuitBreakerRegistry::new(2, std::time::Duration::from_secs(60));
        breakers.record_failure(ProviderId::OpenAI, None);
        breakers.record_failure(ProviderId::OpenAI, None);

        let router = Router::new();
        let candidates = vec![
            RouteCandidate::for_provider(ProviderId::OpenAI, "gpt-4o"),
            RouteCandidate::for_provider(ProviderId::Anthropic, "claude-sonnet-4"),
        ];
        let ranked = router.route_ranked(&candidates, &RoutingPolicy::default(), Some(&breakers));
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].candidate.provider, ProviderId::Anthropic);
    }

    #[test]
    fn semantic_router_and_prompt_ops() {
        let sr = SemanticRouter::with_default_rules();
        let matched = sr.match_prompt("Refactor the gateway module architecture");
        assert_eq!(matched.map(|r| r.intent.as_str()), Some("architecture_reasoning"));
        assert_eq!(
            optimize_prompt_ops("Could you please   summarize   the diff  "),
            "summarize the diff"
        );
    }
}
