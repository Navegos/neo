// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/prometheus.rs
// created: 2026-10-31
// lastModified: 2026-11-08

use std::fmt::Write;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::collector::{global_collector, AtomicHistogram, MetricsCollector};

/// Renders all metrics in Prometheus text format (version 0.0.4)
pub fn render_metrics() -> String {
    render_metrics_from(global_collector())
}

/// Renders metrics from a specific collector instance with zero unwraps/panics
pub fn render_metrics_from(c: &MetricsCollector) -> String {
    let mut out = String::with_capacity(16384);

    // -------------------------------------------------------------------------
    // 1. HTTP Transport Metrics
    // -------------------------------------------------------------------------
    let _ = writeln!(out, "# HELP toksi_http_requests_total Total HTTP requests handled by Toksi gateway");
    let _ = writeln!(out, "# TYPE toksi_http_requests_total counter");
    if let Ok(map) = c.http_requests_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_http_requests_total{{method=\"POST\",route=\"/v1/chat/completions\",status=\"200\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_http_requests_total{{{}}} {}", labels, val);
            }
        }
    }

    render_histogram(&mut out, "toksi_http_request_duration_seconds", "HTTP request duration in seconds", &c.http_request_duration_seconds);
    render_histogram(&mut out, "toksi_http_request_size_bytes", "HTTP request body size in bytes", &c.http_request_size_bytes);
    render_histogram(&mut out, "toksi_http_response_size_bytes", "HTTP response body size in bytes", &c.http_response_size_bytes);

    // -------------------------------------------------------------------------
    // 2. Upstream Provider Metrics
    // -------------------------------------------------------------------------
    let _ = writeln!(out, "# HELP toksi_upstream_requests_total Total requests dispatched to upstream AI providers");
    let _ = writeln!(out, "# TYPE toksi_upstream_requests_total counter");
    if let Ok(map) = c.upstream_requests_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_upstream_requests_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_upstream_requests_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_success_requests_total Successful requests completed by upstream providers");
    let _ = writeln!(out, "# TYPE toksi_success_requests_total counter");
    if let Ok(map) = c.success_requests_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_success_requests_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_success_requests_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_error_requests_total Failed requests from upstream providers by status and error type");
    let _ = writeln!(out, "# TYPE toksi_error_requests_total counter");
    if let Ok(map) = c.error_requests_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_error_requests_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",status_code=\"429\",error_type=\"rate_limit\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_error_requests_total{{{}}} {}", labels, val);
            }
        }
    }

    render_histogram(&mut out, "toksi_upstream_latency_seconds", "Upstream AI provider response latency in seconds", &c.upstream_latency_seconds);

    let _ = writeln!(out, "# HELP toksi_active_requests In-flight concurrent requests currently processing");
    let _ = writeln!(out, "# TYPE toksi_active_requests gauge");
    let _ = writeln!(out, "toksi_active_requests {}", c.active_requests.load(Ordering::Relaxed));

    // -------------------------------------------------------------------------
    // 3. Token Usage & Savings
    // -------------------------------------------------------------------------
    let _ = writeln!(out, "# HELP toksi_input_tokens_total Cumulative input tokens consumed");
    let _ = writeln!(out, "# TYPE toksi_input_tokens_total counter");
    if let Ok(map) = c.input_tokens_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_input_tokens_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_input_tokens_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_output_tokens_total Cumulative output completion tokens generated");
    let _ = writeln!(out, "# TYPE toksi_output_tokens_total counter");
    if let Ok(map) = c.output_tokens_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_output_tokens_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_output_tokens_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_cached_tokens_total Cumulative tokens served via prompt cache discount");
    let _ = writeln!(out, "# TYPE toksi_cached_tokens_total counter");
    if let Ok(map) = c.cached_tokens_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_cached_tokens_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_cached_tokens_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_tokens_saved_total Cumulative tokens saved via compression, pruning, and deduplication");
    let _ = writeln!(out, "# TYPE toksi_tokens_saved_total counter");
    if let Ok(map) = c.tokens_saved_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_tokens_saved_total{{provider=\"anthropic\",model=\"claude-3-7-sonnet\",agent=\"claude\",virtual_key=\"default\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_tokens_saved_total{{{}}} {}", labels, val);
            }
        }
    }

    let raw_bytes = c.headroom_raw_bytes_total.load(Ordering::Relaxed);
    let compressed_bytes = c.headroom_compressed_bytes_total.load(Ordering::Relaxed);
    let pruned_schemas = c.headroom_mcp_pruned_tools_total.load(Ordering::Relaxed);

    let _ = writeln!(out, "# HELP toksi_headroom_raw_bytes_total Cumulative prompt and tool bytes before compression");
    let _ = writeln!(out, "# TYPE toksi_headroom_raw_bytes_total counter");
    let _ = writeln!(out, "toksi_headroom_raw_bytes_total {}", raw_bytes);

    let _ = writeln!(out, "# HELP toksi_headroom_compressed_bytes_total Cumulative prompt and tool bytes after headroom compression");
    let _ = writeln!(out, "# TYPE toksi_headroom_compressed_bytes_total counter");
    let _ = writeln!(out, "toksi_headroom_compressed_bytes_total {}", compressed_bytes);

    let _ = writeln!(out, "# HELP toksi_headroom_mcp_pruned_tools_total Cumulative MCP tool schemas pruned from prompt context");
    let _ = writeln!(out, "# TYPE toksi_headroom_mcp_pruned_tools_total counter");
    let _ = writeln!(out, "toksi_headroom_mcp_pruned_tools_total {}", pruned_schemas);

    // -------------------------------------------------------------------------
    // 4. Costs & ROI Metrics
    // -------------------------------------------------------------------------
    let spend_usd = c.cost_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    let savings_usd = c.cost_savings_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;

    let _ = writeln!(out, "# HELP toksi_cost_usd_total Estimated total API cost in USD");
    let _ = writeln!(out, "# TYPE toksi_cost_usd_total counter");
    let _ = writeln!(out, "toksi_cost_usd_total {:.6}", spend_usd);

    let _ = writeln!(out, "# HELP toksi_cost_savings_usd_total Estimated financial savings in USD from token optimization");
    let _ = writeln!(out, "# TYPE toksi_cost_savings_usd_total counter");
    let _ = writeln!(out, "toksi_cost_savings_usd_total {:.6}", savings_usd);

    // -------------------------------------------------------------------------
    // 5. Cache Analytics
    // -------------------------------------------------------------------------
    let _ = writeln!(out, "# HELP toksi_cache_hits_total Cumulative cache hits by cache type");
    let _ = writeln!(out, "# TYPE toksi_cache_hits_total counter");
    let _ = writeln!(out, "toksi_cache_hits_total{{cache_type=\"direct\"}} {}", c.cache_hits_direct.load(Ordering::Relaxed));
    let _ = writeln!(out, "toksi_cache_hits_total{{cache_type=\"semantic\"}} {}", c.cache_hits_semantic.load(Ordering::Relaxed));

    let _ = writeln!(out, "# HELP toksi_cache_misses_total Cumulative cache misses");
    let _ = writeln!(out, "# TYPE toksi_cache_misses_total counter");
    let _ = writeln!(out, "toksi_cache_misses_total {}", c.cache_misses_total.load(Ordering::Relaxed));

    render_histogram(&mut out, "toksi_cache_lookup_latency_seconds", "Cache lookup latency in seconds", &c.cache_lookup_latency_seconds);

    // -------------------------------------------------------------------------
    // 6. Health & Resilience Metrics
    // -------------------------------------------------------------------------
    let _ = writeln!(out, "# HELP toksi_provider_key_up Health status of provider API keys (1 = healthy, 0 = degraded/down)");
    let _ = writeln!(out, "# TYPE toksi_provider_key_up gauge");
    if let Ok(map) = c.provider_key_up.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_provider_key_up{{provider=\"anthropic\",key_id=\"primary\"}} 1");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_provider_key_up{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_key_rotation_events_total Cumulative provider key rotation failover events");
    let _ = writeln!(out, "# TYPE toksi_key_rotation_events_total counter");
    if let Ok(map) = c.key_rotation_events_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_key_rotation_events_total{{provider=\"anthropic\",reason=\"429_rate_limit\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_key_rotation_events_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_circuit_breaker_trips_total Cumulative circuit breaker trip events");
    let _ = writeln!(out, "# TYPE toksi_circuit_breaker_trips_total counter");
    if let Ok(map) = c.circuit_breaker_trips_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_circuit_breaker_trips_total{{provider=\"anthropic\",rule=\"consecutive_5xx\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_circuit_breaker_trips_total{{{}}} {}", labels, val);
            }
        }
    }

    let _ = writeln!(out, "# HELP toksi_request_retries Cumulative request retry attempts");
    let _ = writeln!(out, "# TYPE toksi_request_retries counter");
    if let Ok(map) = c.request_retries_total.read() {
        if map.is_empty() {
            let _ = writeln!(out, "toksi_request_retries{{provider=\"anthropic\",attempt=\"1\"}} 0");
        } else {
            for (labels, val) in map.iter() {
                let _ = writeln!(out, "toksi_request_retries{{{}}} {}", labels, val);
            }
        }
    }

    // -------------------------------------------------------------------------
    // 7. Internal Toksi Overhead Breakdown
    // -------------------------------------------------------------------------
    render_histogram(&mut out, "toksi_overhead_latency_seconds", "Total internal Toksi processing overhead", &c.overhead_latency_seconds);
    render_histogram(&mut out, "toksi_overhead_compression_seconds", "Headroom log, code, and JSON compression overhead", &c.overhead_compression_seconds);
    render_histogram(&mut out, "toksi_overhead_rule_injection_seconds", "In-tree rules injection latency overhead", &c.overhead_rule_injection_seconds);
    render_histogram(&mut out, "toksi_overhead_mcp_pruning_seconds", "Dynamic MCP tool schema pruning overhead", &c.overhead_mcp_pruning_seconds);

    out
}

fn render_histogram(out: &mut String, name: &str, help: &str, hist: &AtomicHistogram) {
    let _ = writeln!(out, "# HELP {} {}", name, help);
    let _ = writeln!(out, "# TYPE {} histogram", name);

    let (buckets, count, sum) = hist.snapshot();
    for (upper, b_count) in buckets {
        let _ = writeln!(out, "{}_bucket{{le=\"{}\"}} {}", name, upper, b_count);
    }
    let _ = writeln!(out, "{}_bucket{{le=\"+Inf\"}} {}", name, count);
    let _ = writeln!(out, "{}_sum {:.6}", name, sum);
    let _ = writeln!(out, "{}_count {}", name, count);
}

/// Standalone Prometheus metrics HTTP server listening on 127.0.0.1:<port>
pub async fn run_prometheus_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            if let Ok(n) = stream.read(&mut buf).await {
                if n > 0 {
                    let req_str = String::from_utf8_lossy(&buf[..n]);
                    if req_str.starts_with("GET /metrics") {
                        let body = render_metrics();
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(resp.as_bytes()).await;
                    } else if req_str.starts_with("GET /livez") || req_str.starts_with("GET /healthz") {
                        let body = r#"{"service":"toksi-metrics","status":"ok"}"#;
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(resp.as_bytes()).await;
                    } else {
                        let body = "Not Found\n";
                        let resp = format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(resp.as_bytes()).await;
                    }
                }
            }
        });
    }
}
