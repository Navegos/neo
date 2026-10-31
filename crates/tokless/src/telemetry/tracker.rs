// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/telemetry/tracker.rs

use std::time::{Duration, Instant};

/// Internal Tokless processing overhead breakdown
#[derive(Debug, Clone, Copy, Default)]
pub struct OverheadBreakdown {
    pub serialization_micros: u64,
    pub compression_micros: u64,
    pub rule_injection_micros: u64,
    pub mcp_pruning_micros: u64,
}

impl OverheadBreakdown {
    pub fn total_micros(&self) -> u64 {
        self.serialization_micros
            .saturating_add(self.compression_micros)
            .saturating_add(self.rule_injection_micros)
            .saturating_add(self.mcp_pruning_micros)
    }

    pub fn total_seconds(&self) -> f64 {
        self.total_micros() as f64 / 1_000_000.0
    }
}

/// Multi-level latency record for a completed AI request
#[derive(Debug, Clone, Copy)]
pub struct LatencyRecord {
    pub transport_latency: Duration,
    pub upstream_latency: Duration,
    pub overhead: OverheadBreakdown,
    pub total_duration: Duration,
}

impl LatencyRecord {
    pub fn transport_seconds(&self) -> f64 {
        self.transport_latency.as_secs_f64()
    }

    pub fn upstream_seconds(&self) -> f64 {
        self.upstream_latency.as_secs_f64()
    }

    pub fn overhead_seconds(&self) -> f64 {
        self.overhead.total_seconds()
    }

    pub fn total_seconds(&self) -> f64 {
        self.total_duration.as_secs_f64()
    }
}

/// In-flight request latency tracker
#[derive(Debug, Clone)]
pub struct RequestTracker {
    start_time: Instant,
    transport_start: Instant,
    transport_duration: Option<Duration>,
    upstream_duration: Option<Duration>,
    overhead: OverheadBreakdown,
}

impl RequestTracker {
    pub fn start() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            transport_start: now,
            transport_duration: None,
            upstream_duration: None,
            overhead: OverheadBreakdown::default(),
        }
    }

    pub fn record_transport_end(&mut self) {
        self.transport_duration = Some(self.transport_start.elapsed());
    }

    pub fn record_upstream_duration(&mut self, duration: Duration) {
        self.upstream_duration = Some(duration);
    }

    pub fn record_compression(&mut self, micros: u64) {
        self.overhead.compression_micros = self.overhead.compression_micros.saturating_add(micros);
    }

    pub fn record_serialization(&mut self, micros: u64) {
        self.overhead.serialization_micros = self.overhead.serialization_micros.saturating_add(micros);
    }

    pub fn record_rule_injection(&mut self, micros: u64) {
        self.overhead.rule_injection_micros = self.overhead.rule_injection_micros.saturating_add(micros);
    }

    pub fn record_mcp_pruning(&mut self, micros: u64) {
        self.overhead.mcp_pruning_micros = self.overhead.mcp_pruning_micros.saturating_add(micros);
    }

    pub fn finish(self) -> LatencyRecord {
        let total = self.start_time.elapsed();
        let transport = self.transport_duration.unwrap_or_else(|| total);
        let upstream = self.upstream_duration.unwrap_or_default();

        LatencyRecord {
            transport_latency: transport,
            upstream_latency: upstream,
            overhead: self.overhead,
            total_duration: total,
        }
    }
}

