// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-09

pub mod collector;
pub mod cost;
pub mod prometheus;
pub mod tokentracker;
pub mod tracker;
#[cfg(test)]
pub mod tests;

pub use collector::{
    async_metrics_sender, global_collector, init_async_collector, try_send_telemetry_event,
    MetricsCollector, TelemetryEvent, DEFAULT_TELEMETRY_CHANNEL_CAPACITY,
};
pub use cost::{calculate_cost, calculate_savings, lookup_pricing, CostTracker, ModelPricing};
pub use prometheus::{render_metrics, render_metrics_from, run_prometheus_server};
pub use tokentracker::{
    calculate_byte_savings, calculate_headroom_savings, calculate_pruned_schema_savings,
    extract_provider_rate_limits, parse_anthropic_sse_chunk, parse_gemini_chunk,
    parse_openai_sse_chunk, query_provider_quota_free, query_provider_quota_free_async,
    HeadroomSavingsReport, ParsedTokenUsage, ProviderQuotaStatus, ProviderRateLimits,
    SessionUsageRecord, SseStreamTokenParser, TokenTrackerEngine, TokenTrackerError,
    AVERAGE_TOKENS_PER_PRUNED_TOOL, BYTES_PER_TOKEN,
};
pub use tracker::{LatencyRecord, OverheadBreakdown, RequestTracker};
