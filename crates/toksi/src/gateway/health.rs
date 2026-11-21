// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/health.rs
// created: 2026-11-21

//! `olla`-style zero-cost local inference engine health state machine.

#![deny(unsafe_code)]

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Healthy,
    Degraded,
    Unreachable,
    Unconfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineHealthState {
    pub engine_id: String,
    pub endpoint: String,
    pub status: ProbeStatus,
    pub consecutive_failures: u32,
}

#[derive(Debug, Default)]
pub struct HealthRegistry {
    states: BTreeMap<String, EngineHealthState>,
}

impl HealthRegistry {
    pub fn record_probe(&mut self, engine_id: &str, endpoint: &str, ok: bool) {
        let entry = self
            .states
            .entry(engine_id.to_string())
            .or_insert_with(|| EngineHealthState {
                engine_id: engine_id.to_string(),
                endpoint: endpoint.to_string(),
                status: ProbeStatus::Unconfigured,
                consecutive_failures: 0,
            });
        entry.endpoint = endpoint.to_string();
        if ok {
            entry.consecutive_failures = 0;
            entry.status = ProbeStatus::Healthy;
        } else {
            entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
            entry.status = if entry.consecutive_failures >= 3 {
                ProbeStatus::Unreachable
            } else {
                ProbeStatus::Degraded
            };
        }
    }

    pub fn get(&self, engine_id: &str) -> Option<&EngineHealthState> {
        self.states.get(engine_id)
    }
}
