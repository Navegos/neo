// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/telemetry/tests.rs

use std::time::{Duration, Instant};

use super::collector::MetricsCollector;
use super::cost::{calculate_cost, calculate_savings, lookup_pricing, CostTracker};
use super::prometheus::render_metrics_from;
use super::tracker::RequestTracker;

#[test]
fn test_pricing_and_cost_calculation() {
    let sonnet_pricing = lookup_pricing("claude-3-7-sonnet");
    assert_eq!(sonnet_pricing.input_per_m, 3.00);
    assert_eq!(sonnet_pricing.output_per_m, 15.00);
    assert_eq!(sonnet_pricing.cached_per_m, 0.30);

    let gpt4o_pricing = lookup_pricing("gpt-4o");
    assert_eq!(gpt4o_pricing.input_per_m, 2.50);
    assert_eq!(gpt4o_pricing.output_per_m, 10.00);

    let gemini_pricing = lookup_pricing("gemini-2.0-flash");
    assert_eq!(gemini_pricing.input_per_m, 0.10);

    let deepseek_pricing = lookup_pricing("deepseek-reasoner");
    assert_eq!(deepseek_pricing.input_per_m, 0.55);

    // Calculate cost: 1,000,000 input tokens on sonnet = $3.00
    let cost = calculate_cost("claude-3-7-sonnet", 1_000_000, 0, 0);
    assert!((cost - 3.00).abs() < 1e-6);

    // 1,000,000 cached tokens on sonnet = $0.30
    let cached_cost = calculate_cost("claude-3-7-sonnet", 1_000_000, 0, 1_000_000);
    assert!((cached_cost - 0.30).abs() < 1e-6);

    // Savings: 500,000 saved tokens on sonnet = $1.50
    let savings = calculate_savings("claude-3-7-sonnet", 500_000);
    assert!((savings - 1.50).abs() < 1e-6);

    let mut tracker = CostTracker::new();
    tracker.record_transaction("anthropic", "claude-3-7-sonnet", "claude", "default", 3.00, 1.50);
    assert_eq!(tracker.total_spend_usd(), 3.00);
    assert_eq!(tracker.total_savings_usd(), 1.50);
}

#[test]
fn test_latency_tracker_breakdown() {
    let mut tracker = RequestTracker::start();
    tracker.record_compression(120);
    tracker.record_serialization(45);
    tracker.record_rule_injection(80);
    tracker.record_mcp_pruning(250);
    tracker.record_upstream_duration(Duration::from_millis(150));
    tracker.record_transport_end();

    let record = tracker.finish();
    assert_eq!(record.overhead.compression_micros, 120);
    assert_eq!(record.overhead.serialization_micros, 45);
    assert_eq!(record.overhead.rule_injection_micros, 80);
    assert_eq!(record.overhead.mcp_pruning_micros, 250);
    assert_eq!(record.overhead.total_micros(), 495);
    assert!(record.upstream_seconds() >= 0.14);
}

#[test]
fn test_collector_hot_path_latency_under_10_micros() {
    let collector = MetricsCollector::new();

    // Verify hot path execution time (< 10 µs)
    let start = Instant::now();
    for _ in 0..100 {
        collector.inc_active_requests();
        collector.record_http_request("POST", "/v1/chat/completions", 200, 0.045, 1200, 480);
        collector.dec_active_requests();
    }
    let elapsed = start.elapsed();
    let per_op = elapsed / 100;

    // Must be well below 10 µs per hot-path invocation
    assert!(
        per_op < Duration::from_micros(10),
        "Hot path per-op latency was {:?}, expected < 10 µs",
        per_op
    );
}

#[test]
fn test_prometheus_exposition_rendering() {
    let collector = MetricsCollector::new();

    collector.record_http_request("POST", "/v1/chat/completions", 200, 0.035, 1024, 2048);
    collector.record_upstream_call(
        "anthropic",
        "claude-3-7-sonnet",
        200,
        None,
        0.450,
        10000,
        1500,
        5000,
        4000,
        "claude",
        "primary_key",
    );
    collector.record_upstream_call(
        "anthropic",
        "claude-3-7-sonnet",
        429,
        Some("rate_limit"),
        0.050,
        0,
        0,
        0,
        0,
        "claude",
        "primary_key",
    );
    collector.record_cache_event("direct", true, 0.0002);
    collector.record_cache_event("semantic", true, 0.0008);
    collector.record_cache_event("direct", false, 0.0003);
    collector.set_provider_key_up("anthropic", "primary_key", true);
    collector.record_key_rotation("anthropic", "429_rate_limit");
    collector.record_circuit_breaker_trip("anthropic", "consecutive_5xx");
    collector.record_retry("anthropic", 1);

    let text = render_metrics_from(&collector);

    // Verify all required Phase 12 metrics are present
    assert!(text.contains("tokless_http_requests_total"));
    assert!(text.contains("tokless_http_request_duration_seconds"));
    assert!(text.contains("tokless_http_request_size_bytes"));
    assert!(text.contains("tokless_http_response_size_bytes"));
    assert!(text.contains("tokless_upstream_requests_total"));
    assert!(text.contains("tokless_success_requests_total"));
    assert!(text.contains("tokless_error_requests_total"));
    assert!(text.contains("tokless_upstream_latency_seconds"));
    assert!(text.contains("tokless_active_requests"));
    assert!(text.contains("tokless_input_tokens_total"));
    assert!(text.contains("tokless_output_tokens_total"));
    assert!(text.contains("tokless_cached_tokens_total"));
    assert!(text.contains("tokless_tokens_saved_total"));
    assert!(text.contains("tokless_cost_usd_total"));
    assert!(text.contains("tokless_cost_savings_usd_total"));
    assert!(text.contains("tokless_cache_hits_total{cache_type=\"direct\"} 1"));
    assert!(text.contains("tokless_cache_hits_total{cache_type=\"semantic\"} 1"));
    assert!(text.contains("tokless_cache_misses_total 1"));
    assert!(text.contains("tokless_cache_lookup_latency_seconds"));
    assert!(text.contains("tokless_provider_key_up{provider=\"anthropic\",key_id=\"primary_key\"} 1"));
    assert!(text.contains("tokless_key_rotation_events_total"));
    assert!(text.contains("tokless_circuit_breaker_trips_total"));
    assert!(text.contains("tokless_request_retries"));
    assert!(text.contains("tokless_overhead_latency_seconds"));
}

