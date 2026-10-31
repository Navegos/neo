// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/keys.rs

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::gateway::cascade::FailureClass;
use crate::gateway::providers::ProviderId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderApiKey {
    pub key: String,
    pub weight: u32,
    pub priority: u32,
    pub key_up: u8, // 1 for active/healthy, 0 for down/cooldown
    #[serde(skip)]
    pub cooldown_until: Option<Instant>,
    pub consecutive_failures: u32,
}

impl ProviderApiKey {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            weight: 1,
            priority: 1,
            key_up: 1,
            cooldown_until: None,
            consecutive_failures: 0,
        }
    }

    pub fn is_available(&self) -> bool {
        if self.key_up == 0 {
            if let Some(until) = self.cooldown_until {
                if Instant::now() >= until {
                    return true;
                }
            }
            return false;
        }
        true
    }

    pub fn mark_failure(&mut self, failure: FailureClass, cooldown_hint: Option<Duration>) {
        self.consecutive_failures += 1;
        if failure.is_permanent_key_failure() {
            self.key_up = 0;
            // Far future cooldown for permanent failures
            self.cooldown_until = Some(Instant::now() + Duration::from_secs(86400));
        } else {
            self.key_up = 0;
            let dur = cooldown_hint.unwrap_or_else(|| {
                Duration::from_millis((500 * (1 << self.consecutive_failures.min(5))) as u64)
            });
            self.cooldown_until = Some(Instant::now() + dur);
        }
    }

    pub fn mark_success(&mut self) {
        self.key_up = 1;
        self.cooldown_until = None;
        self.consecutive_failures = 0;
    }
}

/// Multi-key pool per provider
#[derive(Debug, Default)]
pub struct KeyPoolManager {
    pools: RwLock<HashMap<ProviderId, Vec<ProviderApiKey>>>,
    round_robin_idx: RwLock<HashMap<ProviderId, usize>>,
}

impl KeyPoolManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_key(&self, provider: ProviderId, key: ProviderApiKey) {
        let mut pools = self.pools.write().unwrap();
        pools.entry(provider).or_default().push(key);
    }

    pub fn get_available_key(&self, provider: ProviderId) -> Option<String> {
        let mut pools = self.pools.write().unwrap();
        let list = pools.get_mut(&provider)?;
        if list.is_empty() {
            return None;
        }

        // Check if any cooldowns have expired
        for k in list.iter_mut() {
            if k.key_up == 0 {
                if let Some(until) = k.cooldown_until {
                    if Instant::now() >= until {
                        k.key_up = 1;
                        k.cooldown_until = None;
                    }
                }
            }
        }

        // Collect indices of available keys
        let available_indices: Vec<usize> = list
            .iter()
            .enumerate()
            .filter(|(_, k)| k.is_available())
            .map(|(i, _)| i)
            .collect();

        if available_indices.is_empty() {
            return None;
        }

        let mut idx_map = self.round_robin_idx.write().unwrap();
        let cur = idx_map.entry(provider).or_insert(0);
        let chosen_idx = available_indices[*cur % available_indices.len()];
        *cur = (*cur + 1) % available_indices.len();

        Some(list[chosen_idx].key.clone())
    }

    pub fn record_result(&self, provider: ProviderId, key: &str, failure: Option<FailureClass>, cooldown_hint: Option<Duration>) {
        let mut pools = self.pools.write().unwrap();
        if let Some(list) = pools.get_mut(&provider) {
            if let Some(k) = list.iter_mut().find(|item| item.key == key) {
                if let Some(f) = failure {
                    k.mark_failure(f, cooldown_hint);
                } else {
                    k.mark_success();
                }
            }
        }
    }
}
