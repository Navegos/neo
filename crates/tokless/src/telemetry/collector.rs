// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/telemetry/collector.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};

use super::cost::{calculate_cost, calculate_savings};
use super::tracker::LatencyRecord;

/// Standard histogram bucket boundaries for latency (in seconds)
pub const LATENCY_BUCKETS: [f64; 11] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Standard histogram bucket boundaries for byte sizes
pub const SIZE_BUCKETS: [f64; 10] = [
    128.0, 512.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0, 4194304.0, 16777216.0,
];

/// Fast atomic histogram with fixed buckets
#[derive(Debug)]
pub struct AtomicHistogram {
    buckets: Vec<f64>,
    counts: Vec<AtomicU64>,
    sum_micros: AtomicU64,
    count: AtomicU64,
}

impl AtomicHistogram {
    pub fn new(buckets: &[f64]) -> Self {
        let mut b = buckets.to_vec();
        b.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let counts = (0..b.len()).map(|_| AtomicU64::new(0)).collect();
        Self {
            buckets: b,
            counts,
            sum_micros: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn observe(&self, value: f64) {
        let micros = (value * 1_000_000.0) as u64;
        self.sum_micros.fetch_add(micros, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        for (i, &upper) in self.buckets.iter().enumerate() {
            if value <= upper {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn snapshot(&self) -> (Vec<(f64, u64)>, u64, f64) {
        let count = self.count.load(Ordering::Relaxed);
        let sum_micros = self.sum_micros.load(Ordering::Relaxed);
        let sum = sum_micros as f64 / 1_000_000.0;
        let mut b_out = Vec::with_capacity(self.buckets.len());
        for (i, &upper) in self.buckets.iter().enumerate() {
            b_out.push((upper, self.counts[i].load(Ordering::Relaxed)));
        }
        (b_out, count, sum)
    }
}

/// Telemetry event for asynchronous / ring buffer recording
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    HttpRequest {
        method: String,
        route: String,
        status: u16,
        duration_secs: f64,
        req_bytes: usize,
        res_bytes: usize,
    },
    UpstreamRequest {
        provider: String,
        model: String,
        status_code: u16,
        error_type: Option<String>,
        latency_secs: f64,
        input_tokens: u64,
        output_tokens: u64,
        cached_tokens: u64,
        tokens_saved: u64,
        agent: String,
        virtual_key: String,
    },
    CacheEvent {
        cache_type: String, // "direct" | "semantic"
        hit: bool,
        lookup_latency_secs: f64,
    },
    ProviderHealth {
        provider: String,
        key_id: String,
        up: bool,
    },
    KeyRotation {
        provider: String,
        reason: String,
    },
    CircuitBreakerTrip {
        provider: String,
        rule: String,
    },
    Retry {
        provider: String,
        attempt: u32,
    },
}

/// High-performance metrics collector
pub struct MetricsCollector {
    // 1. HTTP Transport
    pub http_requests_total: RwLock<HashMap<String, u64>>,
    pub http_request_duration_seconds: AtomicHistogram,
    pub http_request_size_bytes: AtomicHistogram,
    pub http_response_size_bytes: AtomicHistogram,

    // 2. Upstream Provider
    pub upstream_requests_total: RwLock<HashMap<String, u64>>,
    pub success_requests_total: RwLock<HashMap<String, u64>>,
    pub error_requests_total: RwLock<HashMap<String, u64>>,
    pub upstream_latency_seconds: AtomicHistogram,
    pub active_requests: AtomicI64,

    // 3. Token Usage & Savings
    pub input_tokens_total: RwLock<HashMap<String, u64>>,
    pub output_tokens_total: RwLock<HashMap<String, u64>>,
    pub cached_tokens_total: RwLock<HashMap<String, u64>>,
    pub tokens_saved_total: RwLock<HashMap<String, u64>>,

    // 4. Costs & ROI (stored in micro-USD for lock-free atomic arithmetic)
    pub cost_usd_micros: AtomicU64,
    pub cost_savings_usd_micros: AtomicU64,

    // 5. Cache Analytics
    pub cache_hits_direct: AtomicU64,
    pub cache_hits_semantic: AtomicU64,
    pub cache_misses_total: AtomicU64,
    pub cache_lookup_latency_seconds: AtomicHistogram,

    // 6. Health & Resilience
    pub provider_key_up: RwLock<HashMap<String, i64>>,
    pub key_rotation_events_total: RwLock<HashMap<String, u64>>,
    pub circuit_breaker_trips_total: RwLock<HashMap<String, u64>>,
    pub request_retries_total: RwLock<HashMap<String, u64>>,

    // 7. Internal Tokless Overhead Breakdown
    pub overhead_latency_seconds: AtomicHistogram,
    pub overhead_compression_seconds: AtomicHistogram,
    pub overhead_rule_injection_seconds: AtomicHistogram,
    pub overhead_mcp_pruning_seconds: AtomicHistogram,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            http_requests_total: RwLock::new(HashMap::new()),
            http_request_duration_seconds: AtomicHistogram::new(&LATENCY_BUCKETS),
            http_request_size_bytes: AtomicHistogram::new(&SIZE_BUCKETS),
            http_response_size_bytes: AtomicHistogram::new(&SIZE_BUCKETS),

            upstream_requests_total: RwLock::new(HashMap::new()),
            success_requests_total: RwLock::new(HashMap::new()),
            error_requests_total: RwLock::new(HashMap::new()),
            upstream_latency_seconds: AtomicHistogram::new(&LATENCY_BUCKETS),
            active_requests: AtomicI64::new(0),

            input_tokens_total: RwLock::new(HashMap::new()),
            output_tokens_total: RwLock::new(HashMap::new()),
            cached_tokens_total: RwLock::new(HashMap::new()),
            tokens_saved_total: RwLock::new(HashMap::new()),

            cost_usd_micros: AtomicU64::new(0),
            cost_savings_usd_micros: AtomicU64::new(0),

            cache_hits_direct: AtomicU64::new(0),
            cache_hits_semantic: AtomicU64::new(0),
            cache_misses_total: AtomicU64::new(0),
            cache_lookup_latency_seconds: AtomicHistogram::new(&[0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05]),

            provider_key_up: RwLock::new(HashMap::new()),
            key_rotation_events_total: RwLock::new(HashMap::new()),
            circuit_breaker_trips_total: RwLock::new(HashMap::new()),
            request_retries_total: RwLock::new(HashMap::new()),

            overhead_latency_seconds: AtomicHistogram::new(&[0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01]),
            overhead_compression_seconds: AtomicHistogram::new(&[0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01]),
            overhead_rule_injection_seconds: AtomicHistogram::new(&[0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01]),
            overhead_mcp_pruning_seconds: AtomicHistogram::new(&[0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01]),
        }
    }

    /// Fast lock-free active request increment
    #[inline(always)]
    pub fn inc_active_requests(&self) {
        self.active_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Fast lock-free active request decrement
    #[inline(always)]
    pub fn dec_active_requests(&self) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record completed HTTP transport metrics (<10 µs hot path)
    pub fn record_http_request(
        &self,
        method: &str,
        route: &str,
        status: u16,
        duration_secs: f64,
        req_bytes: usize,
        res_bytes: usize,
    ) {
        let key = format!("method=\"{}\",route=\"{}\",status=\"{}\"", method, route, status);
        if let Ok(mut map) = self.http_requests_total.write() {
            *map.entry(key).or_insert(0) += 1;
        }

        self.http_request_duration_seconds.observe(duration_secs);
        self.http_request_size_bytes.observe(req_bytes as f64);
        self.http_response_size_bytes.observe(res_bytes as f64);
    }

    /// Record upstream provider call metrics
    pub fn record_upstream_call(
        &self,
        provider: &str,
        model: &str,
        status_code: u16,
        error_type: Option<&str>,
        latency_secs: f64,
        input_tokens: u64,
        output_tokens: u64,
        cached_tokens: u64,
        tokens_saved: u64,
        agent: &str,
        virtual_key: &str,
    ) {
        // Upstream request counter
        let req_key = format!("provider=\"{}\",model=\"{}\"", provider, model);
        if let Ok(mut map) = self.upstream_requests_total.write() {
            *map.entry(req_key.clone()).or_insert(0) += 1;
        }

        if status_code < 400 {
            if let Ok(mut map) = self.success_requests_total.write() {
                *map.entry(req_key).or_insert(0) += 1;
            }
        } else {
            let err_key = format!(
                "provider=\"{}\",model=\"{}\",status_code=\"{}\",error_type=\"{}\"",
                provider,
                model,
                status_code,
                error_type.unwrap_or("unknown")
            );
            if let Ok(mut map) = self.error_requests_total.write() {
                *map.entry(err_key).or_insert(0) += 1;
            }
        }

        self.upstream_latency_seconds.observe(latency_secs);

        // Tokens
        let tok_key = format!(
            "provider=\"{}\",model=\"{}\",agent=\"{}\",virtual_key=\"{}\"",
            provider, model, agent, virtual_key
        );

        if let Ok(mut map) = self.input_tokens_total.write() {
            *map.entry(tok_key.clone()).or_insert(0) += input_tokens;
        }
        if let Ok(mut map) = self.output_tokens_total.write() {
            *map.entry(tok_key.clone()).or_insert(0) += output_tokens;
        }
        if let Ok(mut map) = self.cached_tokens_total.write() {
            *map.entry(tok_key.clone()).or_insert(0) += cached_tokens;
        }
        if let Ok(mut map) = self.tokens_saved_total.write() {
            *map.entry(tok_key).or_insert(0) += tokens_saved;
        }

        // Costs & ROI
        let spend = calculate_cost(model, input_tokens, output_tokens, cached_tokens);
        let savings = calculate_savings(model, tokens_saved);
        let spend_micros = (spend * 1_000_000.0) as u64;
        let savings_micros = (savings * 1_000_000.0) as u64;

        self.cost_usd_micros.fetch_add(spend_micros, Ordering::Relaxed);
        self.cost_savings_usd_micros.fetch_add(savings_micros, Ordering::Relaxed);
    }

    /// Record internal Tokless overhead latency breakdown
    pub fn record_latency_breakdown(&self, record: &LatencyRecord) {
        self.overhead_latency_seconds.observe(record.overhead_seconds());
        self.overhead_compression_seconds
            .observe(record.overhead.compression_micros as f64 / 1_000_000.0);
        self.overhead_rule_injection_seconds
            .observe(record.overhead.rule_injection_micros as f64 / 1_000_000.0);
        self.overhead_mcp_pruning_seconds
            .observe(record.overhead.mcp_pruning_micros as f64 / 1_000_000.0);
    }

    /// Record cache hit or miss
    pub fn record_cache_event(&self, cache_type: &str, hit: bool, lookup_latency_secs: f64) {
        if hit {
            if cache_type == "semantic" {
                self.cache_hits_semantic.fetch_add(1, Ordering::Relaxed);
            } else {
                self.cache_hits_direct.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            self.cache_misses_total.fetch_add(1, Ordering::Relaxed);
        }
        self.cache_lookup_latency_seconds.observe(lookup_latency_secs);
    }

    /// Update provider key health
    pub fn set_provider_key_up(&self, provider: &str, key_id: &str, up: bool) {
        let key = format!("provider=\"{}\",key_id=\"{}\"", provider, key_id);
        if let Ok(mut map) = self.provider_key_up.write() {
            map.insert(key, if up { 1 } else { 0 });
        }
    }

    /// Record key rotation event
    pub fn record_key_rotation(&self, provider: &str, reason: &str) {
        let key = format!("provider=\"{}\",reason=\"{}\"", provider, reason);
        if let Ok(mut map) = self.key_rotation_events_total.write() {
            *map.entry(key).or_insert(0) += 1;
        }
    }

    /// Record circuit breaker trip
    pub fn record_circuit_breaker_trip(&self, provider: &str, rule: &str) {
        let key = format!("provider=\"{}\",rule=\"{}\"", provider, rule);
        if let Ok(mut map) = self.circuit_breaker_trips_total.write() {
            *map.entry(key).or_insert(0) += 1;
        }
    }

    /// Record request retry
    pub fn record_retry(&self, provider: &str, attempt: u32) {
        let key = format!("provider=\"{}\",attempt=\"{}\"", provider, attempt);
        if let Ok(mut map) = self.request_retries_total.write() {
            *map.entry(key).or_insert(0) += 1;
        }
    }

    /// Ingest a structured TelemetryEvent
    pub fn ingest_event(&self, event: TelemetryEvent) {
        match event {
            TelemetryEvent::HttpRequest {
                method,
                route,
                status,
                duration_secs,
                req_bytes,
                res_bytes,
            } => {
                self.record_http_request(&method, &route, status, duration_secs, req_bytes, res_bytes);
            }
            TelemetryEvent::UpstreamRequest {
                provider,
                model,
                status_code,
                error_type,
                latency_secs,
                input_tokens,
                output_tokens,
                cached_tokens,
                tokens_saved,
                agent,
                virtual_key,
            } => {
                self.record_upstream_call(
                    &provider,
                    &model,
                    status_code,
                    error_type.as_deref(),
                    latency_secs,
                    input_tokens,
                    output_tokens,
                    cached_tokens,
                    tokens_saved,
                    &agent,
                    &virtual_key,
                );
            }
            TelemetryEvent::CacheEvent {
                cache_type,
                hit,
                lookup_latency_secs,
            } => {
                self.record_cache_event(&cache_type, hit, lookup_latency_secs);
            }
            TelemetryEvent::ProviderHealth {
                provider,
                key_id,
                up,
            } => {
                self.set_provider_key_up(&provider, &key_id, up);
            }
            TelemetryEvent::KeyRotation { provider, reason } => {
                self.record_key_rotation(&provider, &reason);
            }
            TelemetryEvent::CircuitBreakerTrip { provider, rule } => {
                self.record_circuit_breaker_trip(&provider, &rule);
            }
            TelemetryEvent::Retry { provider, attempt } => {
                self.record_retry(&provider, attempt);
            }
        }
    }
}

/// Global singleton metrics collector
static GLOBAL_COLLECTOR: OnceLock<MetricsCollector> = OnceLock::new();

pub fn global_collector() -> &'static MetricsCollector {
    GLOBAL_COLLECTOR.get_or_init(MetricsCollector::new)
}

