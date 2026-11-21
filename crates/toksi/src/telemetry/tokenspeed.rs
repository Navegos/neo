// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/tokenspeed.rs
// created: 2026-11-21

//! `tokenspeed` TTFT, TPOT, and Inter-Token Latency (ITL) percentile instrumentation.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LatencyPercentiles {
    pub count: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenSpeedSummary {
    pub ttft: LatencyPercentiles,
    pub tpot: LatencyPercentiles,
    pub itl: LatencyPercentiles,
    pub tokens_per_sec: f64,
}

#[derive(Debug, Default)]
pub struct TokenSpeedTracker {
    ttft_ms: Vec<f64>,
    tpot_ms: Vec<f64>,
    itl_ms: Vec<f64>,
    total_tokens: u64,
    total_decode_ms: f64,
}

impl TokenSpeedTracker {
    pub fn record_sample(&mut self, ttft_ms: f64, tpot_ms: f64, itl_ms: f64, tokens: u64, decode_ms: f64) {
        self.ttft_ms.push(ttft_ms.max(0.0));
        self.tpot_ms.push(tpot_ms.max(0.0));
        self.itl_ms.push(itl_ms.max(0.0));
        self.total_tokens = self.total_tokens.saturating_add(tokens);
        self.total_decode_ms += decode_ms.max(0.0);
    }

    pub fn summary(&self) -> TokenSpeedSummary {
        let tokens_per_sec = if self.total_decode_ms > 0.0 {
            (self.total_tokens as f64) / (self.total_decode_ms / 1000.0)
        } else {
            0.0
        };
        TokenSpeedSummary {
            ttft: compute_percentiles(&self.ttft_ms),
            tpot: compute_percentiles(&self.tpot_ms),
            itl: compute_percentiles(&self.itl_ms),
            tokens_per_sec,
        }
    }
}

pub fn compute_percentiles(samples: &[f64]) -> LatencyPercentiles {
    if samples.is_empty() {
        return LatencyPercentiles::default();
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pick = |q: f64| -> f64 {
        let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    };
    LatencyPercentiles {
        count: sorted.len(),
        p50_ms: pick(0.50),
        p95_ms: pick(0.95),
        p99_ms: pick(0.99),
    }
}
