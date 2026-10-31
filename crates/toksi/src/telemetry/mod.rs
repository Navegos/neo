// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/mod.rs
// created: 2026-10-31
// lastModified: 2026-10-31

pub mod collector;
pub mod cost;
pub mod prometheus;
pub mod tracker;
#[cfg(test)]
pub mod tests;

pub use collector::{global_collector, MetricsCollector, TelemetryEvent};
pub use cost::{calculate_cost, calculate_savings, lookup_pricing, CostTracker, ModelPricing};
pub use prometheus::render_metrics;
pub use tracker::{LatencyRecord, OverheadBreakdown, RequestTracker};

