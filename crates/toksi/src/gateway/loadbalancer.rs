// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/loadbalancer.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use crate::gateway::providers::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoadBalanceStrategy {
    #[default]
    RoundRobin,
    LeastLatency,
    LowestCost,
    LeastInFlight,
    Weighted,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderMetrics {
    pub active_in_flight: u32,
    pub total_requests: u64,
    pub avg_latency_ms: f64,
    pub cost_per_mtoken_usd: f64,
    pub weight: u32,
}

#[derive(Debug, Default)]
pub struct LoadBalancer {
    strategy: RwLock<LoadBalanceStrategy>,
    metrics: RwLock<HashMap<ProviderId, ProviderMetrics>>,
    round_robin_counter: Mutex<usize>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalanceStrategy) -> Self {
        Self {
            strategy: RwLock::new(strategy),
            metrics: RwLock::new(HashMap::new()),
            round_robin_counter: Mutex::new(0),
        }
    }

    pub fn set_strategy(&self, s: LoadBalanceStrategy) {
        let mut st = self.strategy.write().unwrap_or_else(|e| e.into_inner());
        *st = s;
    }

    pub fn register_provider(&self, id: ProviderId, cost_per_mtoken: f64, weight: u32) {
        let mut m = self.metrics.write().unwrap_or_else(|e| e.into_inner());
        m.entry(id).or_insert(ProviderMetrics {
            active_in_flight: 0,
            total_requests: 0,
            avg_latency_ms: 100.0,
            cost_per_mtoken_usd: cost_per_mtoken,
            weight: weight.max(1),
        });
    }

    pub fn select_provider(&self, candidates: &[ProviderId]) -> Option<ProviderId> {
        if candidates.is_empty() {
            return None;
        }
        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        let strat = *self.strategy.read().unwrap_or_else(|e| e.into_inner());
        let m = self.metrics.read().unwrap_or_else(|e| e.into_inner());

        match strat {
            LoadBalanceStrategy::RoundRobin => {
                let mut c = self.round_robin_counter.lock().unwrap_or_else(|e| e.into_inner());
                let chosen = candidates[*c % candidates.len()];
                *c = (*c + 1) % candidates.len();
                Some(chosen)
            }
            LoadBalanceStrategy::LeastLatency => {
                candidates
                    .iter()
                    .min_by(|a, b| {
                        let lat_a = m.get(a).map(|v| v.avg_latency_ms).unwrap_or(9999.0);
                        let lat_b = m.get(b).map(|v| v.avg_latency_ms).unwrap_or(9999.0);
                        lat_a.partial_cmp(&lat_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
            }
            LoadBalanceStrategy::LowestCost => {
                candidates
                    .iter()
                    .min_by(|a, b| {
                        let cost_a = m.get(a).map(|v| v.cost_per_mtoken_usd).unwrap_or(9999.0);
                        let cost_b = m.get(b).map(|v| v.cost_per_mtoken_usd).unwrap_or(9999.0);
                        cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
            }
            LoadBalanceStrategy::LeastInFlight => {
                candidates
                    .iter()
                    .min_by_key(|p| m.get(p).map(|v| v.active_in_flight).unwrap_or(0))
                    .copied()
            }
            LoadBalanceStrategy::Weighted => {
                let total_weight: u32 = candidates
                    .iter()
                    .map(|p| m.get(p).map(|v| v.weight).unwrap_or(1))
                    .sum();
                if total_weight == 0 {
                    return Some(candidates[0]);
                }
                let mut c = self.round_robin_counter.lock().unwrap_or_else(|e| e.into_inner());
                let mut acc = 0;
                let pick = (*c as u32) % total_weight;
                *c += 1;
                for p in candidates {
                    let w = m.get(p).map(|v| v.weight).unwrap_or(1);
                    acc += w;
                    if pick < acc {
                        return Some(*p);
                    }
                }
                Some(candidates[0])
            }
        }
    }

    pub fn acquire_flight(&self, id: ProviderId) {
        let mut m = self.metrics.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = m.get_mut(&id) {
            entry.active_in_flight += 1;
            entry.total_requests += 1;
        }
    }

    pub fn release_flight(&self, id: ProviderId, latency_ms: f64) {
        let mut m = self.metrics.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = m.get_mut(&id) {
            entry.active_in_flight = entry.active_in_flight.saturating_sub(1);
            // Exponential moving average: 0.8 old + 0.2 new
            entry.avg_latency_ms = entry.avg_latency_ms * 0.8 + latency_ms * 0.2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loadbalancer_round_robin() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let list = vec![ProviderId::Anthropic, ProviderId::OpenAI, ProviderId::Gemini];
        let p1 = lb.select_provider(&list).unwrap();
        let p2 = lb.select_provider(&list).unwrap();
        let p3 = lb.select_provider(&list).unwrap();
        let p4 = lb.select_provider(&list).unwrap();

        assert_eq!(p1, ProviderId::Anthropic);
        assert_eq!(p2, ProviderId::OpenAI);
        assert_eq!(p3, ProviderId::Gemini);
        assert_eq!(p4, ProviderId::Anthropic);
    }

    #[test]
    fn test_loadbalancer_least_latency() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::LeastLatency);
        lb.register_provider(ProviderId::Anthropic, 15.0, 1);
        lb.register_provider(ProviderId::Groq, 0.5, 1);

        // Update Groq latency to 30ms, Anthropic to 250ms
        lb.release_flight(ProviderId::Groq, 30.0);
        lb.release_flight(ProviderId::Anthropic, 250.0);

        let list = vec![ProviderId::Anthropic, ProviderId::Groq];
        let chosen = lb.select_provider(&list).unwrap();
        assert_eq!(chosen, ProviderId::Groq);
    }

    #[test]
    fn test_loadbalancer_lowest_cost() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::LowestCost);
        lb.register_provider(ProviderId::Anthropic, 15.0, 1);
        lb.register_provider(ProviderId::DeepSeek, 0.27, 1);

        let list = vec![ProviderId::Anthropic, ProviderId::DeepSeek];
        let chosen = lb.select_provider(&list).unwrap();
        assert_eq!(chosen, ProviderId::DeepSeek);
    }

    #[test]
    fn test_loadbalancer_least_in_flight() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::LeastInFlight);
        lb.register_provider(ProviderId::Anthropic, 15.0, 1);
        lb.register_provider(ProviderId::OpenAI, 10.0, 1);

        lb.acquire_flight(ProviderId::Anthropic);
        lb.acquire_flight(ProviderId::Anthropic);

        let list = vec![ProviderId::Anthropic, ProviderId::OpenAI];
        let chosen = lb.select_provider(&list).unwrap();
        assert_eq!(chosen, ProviderId::OpenAI);
    }

    #[test]
    fn test_loadbalancer_weighted() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::Weighted);
        lb.register_provider(ProviderId::Anthropic, 15.0, 10);
        lb.register_provider(ProviderId::OpenAI, 10.0, 1);

        let list = vec![ProviderId::Anthropic, ProviderId::OpenAI];
        let mut anthropic_picks = 0;
        for _ in 0..11 {
            if lb.select_provider(&list).unwrap() == ProviderId::Anthropic {
                anthropic_picks += 1;
            }
        }
        assert!(anthropic_picks >= 10);
    }
}
