// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/governance.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub const TOKSI_VK_HEADER: &str = "x-toksi-vk";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualKeyConfig {
    pub vk: String,
    pub name: String,
    pub max_budget_usd: f64,
    pub max_rpm: u32,
    pub max_tpm: u32,
    pub max_concurrent_requests: u32,
    pub allowed_models: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct VirtualKeyUsage {
    pub current_spend_usd: f64,
    pub in_flight_requests: u32,
    pub minute_requests: u32,
    pub minute_tokens: u32,
    pub window_start: Option<Instant>,
}

#[derive(Debug, Default)]
pub struct GovernanceEngine {
    configs: RwLock<HashMap<String, VirtualKeyConfig>>,
    usage: Mutex<HashMap<String, VirtualKeyUsage>>,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_vk(&self, cfg: VirtualKeyConfig) {
        let mut c = self.configs.write().unwrap_or_else(|e| e.into_inner());
        c.insert(cfg.vk.clone(), cfg);
    }

    pub fn check_and_acquire(&self, vk: &str, est_tokens: u32) -> Result<(), String> {
        let configs = self.configs.read().unwrap_or_else(|e| e.into_inner());
        let Some(cfg) = configs.get(vk) else {
            // Default allow if VK not explicitly configured
            return Ok(());
        };

        let mut usage_map = self.usage.lock().unwrap_or_else(|e| e.into_inner());
        let u = usage_map.entry(vk.to_string()).or_default();

        // 1. Budget check
        if cfg.max_budget_usd > 0.0 && u.current_spend_usd >= cfg.max_budget_usd {
            return Err(format!("USD budget exceeded for VK '{}' (${:.2} / ${:.2})", vk, u.current_spend_usd, cfg.max_budget_usd));
        }

        // 2. Concurrency check
        if cfg.max_concurrent_requests > 0 && u.in_flight_requests >= cfg.max_concurrent_requests {
            return Err(format!("Max concurrent requests reached for VK '{}' ({})", vk, cfg.max_concurrent_requests));
        }

        // 3. Sliding rate limit check (1 min window)
        let now = Instant::now();
        if let Some(start) = u.window_start {
            if now.duration_since(start) >= Duration::from_secs(60) {
                u.minute_requests = 0;
                u.minute_tokens = 0;
                u.window_start = Some(now);
            }
        } else {
            u.window_start = Some(now);
        }

        if cfg.max_rpm > 0 && u.minute_requests >= cfg.max_rpm {
            return Err(format!("RPM limit exceeded for VK '{}' ({} req/min)", vk, cfg.max_rpm));
        }

        if cfg.max_tpm > 0 && u.minute_tokens + est_tokens > cfg.max_tpm {
            return Err(format!("TPM limit exceeded for VK '{}' ({} tokens/min)", vk, cfg.max_tpm));
        }

        u.in_flight_requests += 1;
        u.minute_requests += 1;
        u.minute_tokens += est_tokens;

        Ok(())
    }

    pub fn release_and_record_spend(&self, vk: &str, cost_usd: f64) {
        let mut usage_map = self.usage.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(u) = usage_map.get_mut(vk) {
            u.in_flight_requests = u.in_flight_requests.saturating_sub(1);
            u.current_spend_usd += cost_usd;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_budget_enforcement() {
        let gov = GovernanceEngine::new();
        gov.register_vk(VirtualKeyConfig {
            vk: "vk-test-1".to_string(),
            name: "Test Key".to_string(),
            max_budget_usd: 1.0,
            max_rpm: 10,
            max_tpm: 1000,
            max_concurrent_requests: 2,
            allowed_models: None,
        });

        // 1st request ok
        assert!(gov.check_and_acquire("vk-test-1", 100).is_ok());
        gov.release_and_record_spend("vk-test-1", 0.60);

        // 2nd request ok
        assert!(gov.check_and_acquire("vk-test-1", 100).is_ok());
        gov.release_and_record_spend("vk-test-1", 0.50);

        // Now spend is 1.10, exceeds 1.0
        let res = gov.check_and_acquire("vk-test-1", 50);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("USD budget exceeded"));
    }

    #[test]
    fn test_governance_concurrency_limit() {
        let gov = GovernanceEngine::new();
        gov.register_vk(VirtualKeyConfig {
            vk: "vk-concurrency".to_string(),
            name: "Concurrency Key".to_string(),
            max_budget_usd: 100.0,
            max_rpm: 100,
            max_tpm: 10000,
            max_concurrent_requests: 1,
            allowed_models: None,
        });

        assert!(gov.check_and_acquire("vk-concurrency", 100).is_ok());
        // 2nd concurrent should fail
        let res2 = gov.check_and_acquire("vk-concurrency", 100);
        assert!(res2.is_err());
        assert!(res2.unwrap_err().contains("Max concurrent requests reached"));

        // Release flight
        gov.release_and_record_spend("vk-concurrency", 0.01);
        // Now next succeeds
        assert!(gov.check_and_acquire("vk-concurrency", 100).is_ok());
    }

    #[test]
    fn test_governance_rpm_and_tpm_limits() {
        let gov = GovernanceEngine::new();
        gov.register_vk(VirtualKeyConfig {
            vk: "vk-rates".to_string(),
            name: "Rates Key".to_string(),
            max_budget_usd: 100.0,
            max_rpm: 2,
            max_tpm: 500,
            max_concurrent_requests: 10,
            allowed_models: None,
        });

        assert!(gov.check_and_acquire("vk-rates", 200).is_ok());
        gov.release_and_record_spend("vk-rates", 0.0);

        assert!(gov.check_and_acquire("vk-rates", 200).is_ok());
        gov.release_and_record_spend("vk-rates", 0.0);

        // 3rd in minute exceeds RPM (max 2)
        let res3 = gov.check_and_acquire("vk-rates", 50);
        assert!(res3.is_err());
        assert!(res3.unwrap_err().contains("RPM limit exceeded"));
    }
}
