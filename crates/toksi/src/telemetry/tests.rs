// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/tests.rs
// created: 2026-10-31
// lastModified: 2026-10-31

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
    assert!(text.contains("toksi_http_requests_total"));
    assert!(text.contains("toksi_http_request_duration_seconds"));
    assert!(text.contains("toksi_http_request_size_bytes"));
    assert!(text.contains("toksi_http_response_size_bytes"));
    assert!(text.contains("toksi_upstream_requests_total"));
    assert!(text.contains("toksi_success_requests_total"));
    assert!(text.contains("toksi_error_requests_total"));
    assert!(text.contains("toksi_upstream_latency_seconds"));
    assert!(text.contains("toksi_active_requests"));
    assert!(text.contains("toksi_input_tokens_total"));
    assert!(text.contains("toksi_output_tokens_total"));
    assert!(text.contains("toksi_cached_tokens_total"));
    assert!(text.contains("toksi_tokens_saved_total"));
    assert!(text.contains("toksi_cost_usd_total"));
    assert!(text.contains("toksi_cost_savings_usd_total"));
    assert!(text.contains("toksi_cache_hits_total{cache_type=\"direct\"} 1"));
    assert!(text.contains("toksi_cache_hits_total{cache_type=\"semantic\"} 1"));
    assert!(text.contains("toksi_cache_misses_total 1"));
    assert!(text.contains("toksi_cache_lookup_latency_seconds"));
    assert!(text.contains("toksi_provider_key_up{provider=\"anthropic\",key_id=\"primary_key\"} 1"));
    assert!(text.contains("toksi_key_rotation_events_total"));
    assert!(text.contains("toksi_circuit_breaker_trips_total"));
    assert!(text.contains("toksi_request_retries"));
    assert!(text.contains("toksi_overhead_latency_seconds"));
}

#[test]
fn test_dropped_request_skips_upstream_metrics() {
    let collector = MetricsCollector::new();

    // 1. request_dropped (rejection before queueing) should NOT count towards upstream_requests_total
    // or upstream_latency_seconds, but SHOULD count in error_requests_total
    collector.record_upstream_call(
        "openai",
        "gpt-4o",
        503,
        Some("request_dropped"),
        0.001,
        0,
        0,
        0,
        0,
        "copilot",
        "default",
    );

    let (_, count, _) = collector.upstream_latency_seconds.snapshot();
    assert_eq!(count, 0, "Latency count should be 0 for dropped requests");

    {
        let upstream_map = collector.upstream_requests_total.read().unwrap();
        assert!(
            upstream_map.is_empty(),
            "Upstream requests total should be empty for dropped requests"
        );

        let error_map = collector.error_requests_total.read().unwrap();
        assert_eq!(error_map.len(), 1, "Error requests total should record the 503");
    }

    // 2. An upstream provider 503 (e.g. overloaded_error) DOES count towards upstream traffic
    collector.record_upstream_call(
        "openai",
        "gpt-4o",
        503,
        Some("overloaded_error"),
        0.420,
        0,
        0,
        0,
        0,
        "copilot",
        "default",
    );

    let (_, count_after, _) = collector.upstream_latency_seconds.snapshot();
    assert_eq!(count_after, 1, "Latency count should be 1 for provider 503");

    let upstream_map_after = collector.upstream_requests_total.read().unwrap();
    assert_eq!(
        *upstream_map_after.get("provider=\"openai\",model=\"gpt-4o\"").unwrap_or(&0),
        1
    );
}

#[test]
fn test_tokentracker_ground_truth_zero_idle() {
    use super::tokentracker::{TokenTrackerEngine, ParsedTokenUsage};

    let engine = TokenTrackerEngine::new(50);
    // When idle with no sessions, all metrics must be strictly 0
    let (prompt, completion, cached, saved, cost, savings) = engine.lifetime_summary();
    assert_eq!(prompt, 0);
    assert_eq!(completion, 0);
    assert_eq!(cached, 0);
    assert_eq!(saved, 0);
    assert_eq!(cost, 0.0);
    assert_eq!(savings, 0.0);
    assert!(engine.get_recent_sessions().is_empty());

    // Record one ground-truth session
    let usage = ParsedTokenUsage {
        prompt_tokens: 10_000,
        completion_tokens: 2_000,
        cached_tokens: 4_000,
        saved_tokens: 8_000,
    };
    let rec = engine.record_usage("sess-1", "claude", "anthropic", "claude-3-7-sonnet", usage, 450)
        .expect("record_usage must succeed");

    assert_eq!(rec.input_tokens, 10_000);
    assert_eq!(rec.output_tokens, 2_000);
    assert_eq!(rec.cached_tokens, 4_000);
    assert_eq!(rec.saved_tokens, 8_000);
    assert!(rec.cost_usd > 0.0);
    assert!(rec.saved_usd > 0.0);

    let (prompt2, completion2, cached2, saved2, cost2, savings2) = engine.lifetime_summary();
    assert_eq!(prompt2, 10_000);
    assert_eq!(completion2, 2_000);
    assert_eq!(cached2, 4_000);
    assert_eq!(saved2, 8_000);
    assert!(cost2 > 0.0);
    assert!(savings2 > 0.0);
}

#[test]
fn test_sse_chunk_token_parsers() {
    use super::tokentracker::{parse_anthropic_sse_chunk, parse_gemini_chunk, parse_openai_sse_chunk};

    // 1. OpenAI completion chunk
    let openai_chunk = r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","usage":{"prompt_tokens":1200,"completion_tokens":350,"prompt_tokens_details":{"cached_tokens":800}}}"#;
    let parsed_openai = parse_openai_sse_chunk(openai_chunk).expect("OpenAI chunk must parse");
    assert_eq!(parsed_openai.prompt_tokens, 1200);
    assert_eq!(parsed_openai.completion_tokens, 350);
    assert_eq!(parsed_openai.cached_tokens, 800);

    // OpenAI [DONE] returns None
    assert!(parse_openai_sse_chunk("data: [DONE]").is_none());

    // 2. Anthropic message_start & message_delta
    let anthropic_start = r#"data: {"type":"message_start","message":{"id":"msg_123","usage":{"input_tokens":5400,"cache_read_input_tokens":3200}}}"#;
    let parsed_start = parse_anthropic_sse_chunk(anthropic_start).expect("Anthropic start must parse");
    assert_eq!(parsed_start.prompt_tokens, 5400);
    assert_eq!(parsed_start.cached_tokens, 3200);
    assert_eq!(parsed_start.completion_tokens, 0);

    let anthropic_delta = r#"data: {"type":"message_delta","usage":{"output_tokens":620}}"#;
    let parsed_delta = parse_anthropic_sse_chunk(anthropic_delta).expect("Anthropic delta must parse");
    assert_eq!(parsed_delta.completion_tokens, 620);
    assert_eq!(parsed_delta.prompt_tokens, 0);

    // 3. Gemini usageMetadata
    let gemini_json = r#"{"candidates":[],"usageMetadata":{"promptTokenCount":4500,"candidatesTokenCount":1100,"totalTokenCount":5600,"cachedContentTokenCount":2000}}"#;
    let parsed_gemini = parse_gemini_chunk(gemini_json).expect("Gemini chunk must parse");
    assert_eq!(parsed_gemini.prompt_tokens, 4500);
    assert_eq!(parsed_gemini.completion_tokens, 1100);
    assert_eq!(parsed_gemini.cached_tokens, 2000);
}

#[tokio::test]
async fn test_async_metrics_channel_ingestion() {
    use super::collector::{init_async_collector, try_send_telemetry_event, TelemetryEvent};

    let _tx = init_async_collector(100);

    let sent = try_send_telemetry_event(TelemetryEvent::UpstreamRequest {
        provider: "openai".to_string(),
        model: "gpt-4o-mini".to_string(),
        status_code: 200,
        error_type: None,
        latency_secs: 0.05,
        input_tokens: 1500,
        output_tokens: 300,
        cached_tokens: 500,
        tokens_saved: 700,
        agent: "cursor".to_string(),
        virtual_key: "default".to_string(),
    });
    assert!(sent, "try_send_telemetry_event must succeed");

    // Allow worker loop to drain
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}


