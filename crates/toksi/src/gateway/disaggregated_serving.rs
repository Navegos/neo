// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/disaggregated_serving.rs
// created: 2026-11-21

//! Disaggregated prefill/decode serving & fabric routing (`dynamo`, `velo`,
//! `frontend-crates`, `Switchyard`, `NeMo-Fabric`, `NeMo-Relay`, `physicsnemo-serve`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServingRole {
    Prefill,
    Decode,
    Unified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisaggregatedWorker {
    pub worker_id: String,
    pub role: ServingRole,
    pub endpoint: String,
    pub engine: String,
    pub kv_cache_ready: bool,
}

#[derive(Debug, Default)]
pub struct DisaggregatedRouter {
    workers: Vec<DisaggregatedWorker>,
}

impl DisaggregatedRouter {
    pub fn register_worker(&mut self, worker: DisaggregatedWorker) {
        self.workers.push(worker);
    }

    pub fn select_pair(&self) -> Option<(&DisaggregatedWorker, &DisaggregatedWorker)> {
        let prefill = self
            .workers
            .iter()
            .find(|w| w.role == ServingRole::Prefill && w.kv_cache_ready)?;
        let decode = self
            .workers
            .iter()
            .find(|w| w.role == ServingRole::Decode && w.kv_cache_ready)?;
        Some((prefill, decode))
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}
