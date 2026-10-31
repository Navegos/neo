// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/telemetry/prometheus.rs

use std::fmt::Write;
use std::sync::atomic::Ordering;

use super::collector::{global_collector, AtomicHistogram, MetricsCollector};

/// Renders all metrics in Prometheus text format (version 0.0.4)
pub fn render_metrics() -> String {
    render_metrics_from(global_collector())
}

/// Renders metrics from a specific collector instance
pub fn render_metrics_from(c: &MetricsCollector) -> String {
    let mut out = String::with_capacity(16384);

    // -------------------------------------------------------------------------
    // 1. HTTP Transport Metrics
    // -------------------------------------------------------------------------
    writeln!(out, "# HELP tokless_http_requests_total Total HTTP requests handled by Tokless gateway").unwrap();
    writeln!(out, "# TYPE tokless_http_requests_total counter").unwrap();
    if let Ok(map) = c.http_requests_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_http_requests_total{{method=\"POST\",route=\"/v1/chat/completions\",status=\"200\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_http_requests_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    render_histogram(&mut out, "tokless_http_request_duration_seconds", "HTTP request duration in seconds", &c.http_request_duration_seconds);
    render_histogram(&mut out, "tokless_http_request_size_bytes", "HTTP request body size in bytes", &c.http_request_size_bytes);
    render_histogram(&mut out, "tokless_http_response_size_bytes", "HTTP response body size in bytes", &c.http_response_size_bytes);

    // -------------------------------------------------------------------------
    // 2. Upstream Provider Metrics
    // -------------------------------------------------------------------------
    writeln!(out, "# HELP tokless_upstream_requests_total Total requests dispatched to upstream AI providers").unwrap();
    writeln!(out, "# TYPE tokless_upstream_requests_total counter").unwrap();
    if let Ok(map) = c.upstream_requests_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_upstream_requests_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_upstream_requests_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_success_requests_total Successful requests completed by upstream providers").unwrap();
    writeln!(out, "# TYPE tokless_success_requests_total counter").unwrap();
    if let Ok(map) = c.success_requests_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_success_requests_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_success_requests_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_error_requests_total Failed requests from upstream providers by status and error type").unwrap();
    writeln!(out, "# TYPE tokless_error_requests_total counter").unwrap();
    if let Ok(map) = c.error_requests_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_error_requests_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",status_code=\"429\",error_type=\"rate_limit\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_error_requests_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    render_histogram(&mut out, "tokless_upstream_latency_seconds", "Upstream AI provider response latency in seconds", &c.upstream_latency_seconds);

    writeln!(out, "# HELP tokless_active_requests In-flight concurrent requests currently processing").unwrap();
    writeln!(out, "# TYPE tokless_active_requests gauge").unwrap();
    writeln!(out, "tokless_active_requests {}", c.active_requests.load(Ordering::Relaxed)).unwrap();

    // -------------------------------------------------------------------------
    // 3. Token Usage & Savings
    // -------------------------------------------------------------------------
    writeln!(out, "# HELP tokless_input_tokens_total Cumulative input tokens consumed").unwrap();
    writeln!(out, "# TYPE tokless_input_tokens_total counter").unwrap();
    if let Ok(map) = c.input_tokens_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_input_tokens_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_input_tokens_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_output_tokens_total Cumulative output completion tokens generated").unwrap();
    writeln!(out, "# TYPE tokless_output_tokens_total counter").unwrap();
    if let Ok(map) = c.output_tokens_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_output_tokens_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_output_tokens_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_cached_tokens_total Cumulative tokens served via prompt cache discount").unwrap();
    writeln!(out, "# TYPE tokless_cached_tokens_total counter").unwrap();
    if let Ok(map) = c.cached_tokens_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_cached_tokens_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_cached_tokens_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_tokens_saved_total Cumulative tokens saved via compression, pruning, and deduplication").unwrap();
    writeln!(out, "# TYPE tokless_tokens_saved_total counter").unwrap();
    if let Ok(map) = c.tokens_saved_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_tokens_saved_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_tokens_saved_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. Costs & ROI Metrics
    // -------------------------------------------------------------------------
    let spend_usd = c.cost_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    let savings_usd = c.cost_savings_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;

    writeln!(out, "# HELP tokless_cost_usd_total Estimated total API cost in USD").unwrap();
    writeln!(out, "# TYPE tokless_cost_usd_total counter").unwrap();
    writeln!(out, "tokless_cost_usd_total {:.6}", spend_usd).unwrap();

    writeln!(out, "# HELP tokless_cost_savings_usd_total Estimated financial savings in USD from token optimization").unwrap();
    writeln!(out, "# TYPE tokless_cost_savings_usd_total counter").unwrap();
    writeln!(out, "tokless_cost_savings_usd_total {:.6}", savings_usd).unwrap();

    // -------------------------------------------------------------------------
    // 5. Cache Analytics
    // -------------------------------------------------------------------------
    writeln!(out, "# HELP tokless_cache_hits_total Cumulative cache hits by cache type").unwrap();
    writeln!(out, "# TYPE tokless_cache_hits_total counter").unwrap();
    writeln!(out, "tokless_cache_hits_total{{cache_type=\"direct\"}} {}", c.cache_hits_direct.load(Ordering::Relaxed)).unwrap();
    writeln!(out, "tokless_cache_hits_total{{cache_type=\"semantic\"}} {}", c.cache_hits_semantic.load(Ordering::Relaxed)).unwrap();

    writeln!(out, "# HELP tokless_cache_misses_total Cumulative cache misses").unwrap();
    writeln!(out, "# TYPE tokless_cache_misses_total counter").unwrap();
    writeln!(out, "tokless_cache_misses_total {}", c.cache_misses_total.load(Ordering::Relaxed)).unwrap();

    render_histogram(&mut out, "tokless_cache_lookup_latency_seconds", "Cache lookup latency in seconds", &c.cache_lookup_latency_seconds);

    // -------------------------------------------------------------------------
    // 6. Health & Resilience Metrics
    // -------------------------------------------------------------------------
    writeln!(out, "# HELP tokless_provider_key_up Health status of provider API keys (1 = healthy, 0 = degraded/down)").unwrap();
    writeln!(out, "# TYPE tokless_provider_key_up gauge").unwrap();
    if let Ok(map) = c.provider_key_up.read() {
        if map.is_empty() {
            writeln!(out, "tokless_provider_key_up{{provider=\"anthropic\",key_id=\"primary\"}} 1").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_provider_key_up{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_key_rotation_events_total Cumulative provider key rotation failover events").unwrap();
    writeln!(out, "# TYPE tokless_key_rotation_events_total counter").unwrap();
    if let Ok(map) = c.key_rotation_events_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_key_rotation_events_total{{provider=\"anthropic\",reason=\"429_rate_limit\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_key_rotation_events_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_circuit_breaker_trips_total Cumulative circuit breaker trip events").unwrap();
    writeln!(out, "# TYPE tokless_circuit_breaker_trips_total counter").unwrap();
    if let Ok(map) = c.circuit_breaker_trips_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_circuit_breaker_trips_total{{provider=\"anthropic\",rule=\"consecutive_5xx\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_circuit_breaker_trips_total{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    writeln!(out, "# HELP tokless_request_retries Cumulative request retry attempts").unwrap();
    writeln!(out, "# TYPE tokless_request_retries counter").unwrap();
    if let Ok(map) = c.request_retries_total.read() {
        if map.is_empty() {
            writeln!(out, "tokless_request_retries{{provider=\"anthropic\",attempt=\"1\"}} 0").unwrap();
        } else {
            for (labels, val) in map.iter() {
                writeln!(out, "tokless_request_retries{{{}}} {}", labels, val).unwrap();
            }
        }
    }

    // -------------------------------------------------------------------------
    // 7. Internal Tokless Overhead Breakdown
    // -------------------------------------------------------------------------
    render_histogram(&mut out, "tokless_overhead_latency_seconds", "Total internal Tokless processing overhead", &c.overhead_latency_seconds);
    render_histogram(&mut out, "tokless_overhead_compression_seconds", "Headroom log, code, and JSON compression overhead", &c.overhead_compression_seconds);
    render_histogram(&mut out, "tokless_overhead_rule_injection_seconds", "In-tree rules injection latency overhead", &c.overhead_rule_injection_seconds);
    render_histogram(&mut out, "tokless_overhead_mcp_pruning_seconds", "Dynamic MCP tool schema pruning overhead", &c.overhead_mcp_pruning_seconds);

    out
}

fn render_histogram(out: &mut String, name: &str, help: &str, hist: &AtomicHistogram) {
    writeln!(out, "# HELP {} {}", name, help).unwrap();
    writeln!(out, "# TYPE {} histogram", name).unwrap();

    let (buckets, count, sum) = hist.snapshot();
    for (upper, b_count) in buckets {
        writeln!(out, "{}_bucket{{le=\"{}\"}} {}", name, upper, b_count).unwrap();
    }
    writeln!(out, "{}_bucket{{le=\"+Inf\"}} {}", name, count).unwrap();
    writeln!(out, "{}_sum {:.6}", name, sum).unwrap();
    writeln!(out, "{}_count {}", name, count).unwrap();
}

