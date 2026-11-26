// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/proxy.rs
// created: 2026-10-31
// lastModified: 2026-11-16

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use serde_json::Value;

use crate::gateway::cascade::{classify_failure, parse_retry_after};
use crate::gateway::circuit_breaker::{CircuitBreaker, CircuitBreakerRegistry};
use crate::gateway::governance::GovernanceEngine;
use crate::gateway::keys::KeyPoolManager;
use crate::gateway::loadbalancer::LoadBalancer;
use crate::gateway::providers::{
    build_provider_client, ProviderCategory, ProviderClient, ProviderId, UnifiedRequest,
    UnifiedResponse,
};
use crate::gateway::reconnect::global_healing_pool;
use crate::gateway::router::{RouteCandidate, ReasoningEffort, Router, RoutingPolicy};
use crate::gateway::semantic_cache::DualTierSemanticCache;
use crate::headroom::effort::{EffortLevel, HeadroomEffortCompressor};
use crate::telemetry::cost::calculate_cost;
use crate::telemetry::tokentracker::{global_token_tracker, ParsedTokenUsage};

#[derive(Default)]
pub struct GatewayProxyOrchestrator {
    pub key_manager: Arc<KeyPoolManager>,
    pub load_balancer: Arc<LoadBalancer>,
    pub semantic_cache: Arc<DualTierSemanticCache>,
    pub governance: Arc<GovernanceEngine>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub circuit_registry: Arc<CircuitBreakerRegistry>,
    pub router: Router,
}

impl GatewayProxyOrchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast Tier-1 SHA-256 + Tier-2 HNSW semantic vector cache lookup
    pub fn lookup_cache(&self, provider: &str, prompt: &str) -> Option<String> {
        self.semantic_cache.lookup_or_semantic(provider, prompt)
    }

    /// Stores response in dual-tier semantic cache (exact + HNSW ANN vector)
    pub fn store_cache(&self, provider: &str, prompt: &str, content: &str) {
        self.semantic_cache.store_auto(provider, prompt, content);
    }

    /// Records tokens with circuit breaker
    pub fn record_tokens(&self, session_id: &str, tokens: u64) -> Result<(), String> {
        self.circuit_breaker
            .record_tokens(session_id, tokens)
            .map_err(|e| format!("Circuit breaker triggered: {:?}", e))
    }

    fn build_unified_request(payload: &Value) -> UnifiedRequest {
        UnifiedRequest {
            model: payload
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or("default")
                .to_string(),
            messages: payload
                .get("messages")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            system: payload
                .get("system")
                .and_then(Value::as_str)
                .map(|s| s.to_string()),
            max_tokens: payload
                .get("max_tokens")
                .and_then(Value::as_u64)
                .map(|u| u as u32),
            temperature: payload
                .get("temperature")
                .and_then(Value::as_f64)
                .map(|f| f as f32),
            stream: payload.get("stream").and_then(Value::as_bool),
            tools: payload.get("tools").and_then(Value::as_array).cloned(),
            extra: HashMap::new(),
        }
    }

    fn resolve_candidate_endpoint(
        explicit_endpoint: Option<&str>,
        candidate_base_url: Option<&str>,
        client: &dyn ProviderClient,
    ) -> String {
        if let Some(ep) = explicit_endpoint {
            let trimmed = ep.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(base) = candidate_base_url {
            let trimmed = base.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                if trimmed.ends_with("/chat/completions")
                    || trimmed.ends_with("/messages")
                    || trimmed.ends_with("/responses")
                    || trimmed.ends_with("/embeddings")
                    || trimmed.contains(":generateContent")
                {
                    return trimmed.to_string();
                }
                if client.id() == ProviderId::Anthropic {
                    if trimmed.ends_with("/v1") {
                        return format!("{}/messages", trimmed);
                    }
                    return format!("{}/v1/messages", trimmed);
                }
                return format!("{}/chat/completions", trimmed);
            }
        }
        client.endpoint()
    }

    fn emit_headroom_telemetry(hr_stats: &crate::headroom::effort::EffortCompressionStats) {
        if hr_stats.bytes_saved > 0 || hr_stats.mcp_tools_pruned > 0 {
            crate::telemetry::collector::try_send_telemetry_event(
                crate::telemetry::collector::TelemetryEvent::HeadroomOptimization {
                    raw_bytes: hr_stats.original_bytes,
                    compressed_bytes: hr_stats.compressed_bytes,
                    pruned_tools: hr_stats.mcp_tools_pruned,
                },
            );
        }
    }

    fn resolve_key(&self, provider: ProviderId, explicit_key: Option<&str>) -> Result<String, String> {
        if let Some(k) = explicit_key {
            let trimmed = k.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        if let Some(k) = self.key_manager.get_available_key(provider) {
            return Ok(k);
        }
        if provider.category() == ProviderCategory::Local {
            return Ok(String::new());
        }
        Err(format!(
            "No API key configured for provider {}",
            provider.as_str()
        ))
    }

    fn record_usage_and_cache(
        &self,
        session_id: &str,
        vk_opt: Option<&str>,
        provider: ProviderId,
        prompt_str: &str,
        parsed: &UnifiedResponse,
        latency_ms: f64,
    ) -> Result<(), String> {
        self.load_balancer.release_flight(provider, latency_ms);
        self.circuit_registry.record_success(provider);

        let total_tokens = (parsed.input_tokens + parsed.output_tokens) as u64;
        self.circuit_breaker
            .record_tokens(session_id, total_tokens)
            .map_err(|e| format!("Circuit breaker triggered: {:?}", e))?;

        let cached_read = parsed
            .raw
            .get("usage")
            .and_then(|u| {
                u.get("cache_read_input_tokens").or_else(|| {
                    u.get("prompt_tokens_details")
                        .and_then(|d| d.get("cached_tokens"))
                })
            })
            .or_else(|| {
                parsed
                    .raw
                    .get("usageMetadata")
                    .and_then(|m| m.get("cachedContentTokenCount"))
            })
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let real_cost = calculate_cost(
            &parsed.model,
            parsed.input_tokens as u64,
            parsed.output_tokens as u64,
            cached_read,
        );

        let reasoning_tokens = parsed
            .raw
            .get("usage")
            .and_then(|u| {
                u.get("completion_tokens_details")
                    .or_else(|| u.get("output_tokens_details"))
                    .and_then(|d| d.get("reasoning_tokens"))
            })
            .or_else(|| {
                parsed
                    .raw
                    .get("usageMetadata")
                    .and_then(|m| m.get("thoughtsTokenCount"))
            })
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let cache_creation_tokens = parsed
            .raw
            .get("usage")
            .and_then(|u| u.get("cache_creation_input_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let _ = global_token_tracker().record_usage(
            session_id,
            "gateway",
            provider.as_str(),
            &parsed.model,
            ParsedTokenUsage {
                prompt_tokens: parsed.input_tokens as u64,
                completion_tokens: parsed.output_tokens as u64,
                cached_tokens: cached_read,
                reasoning_tokens,
                cache_creation_tokens,
                saved_tokens: 0,
            },
            latency_ms as u64,
        );

        self.semantic_cache
            .store_auto(provider.as_str(), prompt_str, &parsed.content);

        if let Some(vk) = vk_opt {
            self.governance.release_and_record_spend(vk, real_cost);
        }
        Ok(())
    }

    fn record_upstream_failure(
        &self,
        provider: ProviderId,
        key: &str,
        status: u16,
        headers: &HashMap<String, String>,
        body: &str,
        latency_ms: f64,
    ) {
        self.load_balancer.release_flight(provider, latency_ms);
        let cls = classify_failure(status, headers, body);
        let cooldown = parse_retry_after(headers);
        let hint = if cooldown.is_zero() {
            None
        } else {
            Some(cooldown)
        };
        if !key.is_empty() {
            self.key_manager.record_result(provider, key, Some(cls), hint);
        }
        self.circuit_registry.record_failure(provider, hint);
    }

    fn record_rate_limit_windows(provider: ProviderId, headers: &HashMap<String, String>) {
        let pairs: Vec<(String, String)> = headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let now_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let windows = crate::telemetry::tokentracker::extract_quota_windows(
            provider.as_str(),
            "gateway",
            &pairs,
            now_epoch,
        );
        if !windows.is_empty()
            && let Ok(arc) = crate::db::shared_db()
            && let Ok(mut db) = arc.lock()
        {
            for w in &windows {
                let _ = db.upsert_quota_window(w);
            }
        }
    }

    /// Processes an incoming agent request synchronously through the optimization & upstream HTTP pipeline.
    pub fn process_request(
        &self,
        session_id: &str,
        vk_opt: Option<&str>,
        provider: ProviderId,
        client: &dyn ProviderClient,
        payload: &mut Value,
    ) -> Result<UnifiedResponse, String> {
        if let Some(vk) = vk_opt {
            self.governance.check_and_acquire(vk, 1000)?;
        }

        let headroom_path = if provider == ProviderId::Anthropic {
            "/v1/messages"
        } else {
            "/v1/chat/completions"
        };
        let hr_stats = HeadroomEffortCompressor::gateway_default().compress_json_payload(
            headroom_path,
            &[],
            payload,
        );
        Self::emit_headroom_telemetry(&hr_stats);

        let prompt_str = payload.to_string();
        if let Some(cached_resp) = self.lookup_cache(provider.as_str(), &prompt_str) {
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Ok(UnifiedResponse {
                id: "cache-hit".to_string(),
                model: payload
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or(provider.as_str())
                    .to_string(),
                content: cached_resp,
                role: "assistant".to_string(),
                finish_reason: Some("stop".to_string()),
                input_tokens: 0,
                output_tokens: 0,
                raw: Value::Null,
            });
        }

        let key = match self.resolve_key(provider, None) {
            Ok(k) => k,
            Err(e) => {
                if let Some(vk) = vk_opt {
                    self.governance.release_and_record_spend(vk, 0.0);
                }
                return Err(e);
            }
        };

        let unified_req = Self::build_unified_request(payload);
        let mut req_formatted = client.format_request(&unified_req);
        let raw_model = payload
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("");
        let eff = crate::headroom::effort::detect_effort_level(&[], Some(payload));
        crate::gateway::sampling::apply_sampling_params_to_payload(
            provider.as_str(),
            raw_model,
            eff,
            &crate::gateway::sampling::SamplingParams::default(),
            &mut req_formatted,
        );
        let endpoint = client.endpoint();
        let is_loopback = endpoint.starts_with("http://127.0.0.1")
            || endpoint.starts_with("http://localhost");
        let blocking_client = if is_loopback {
            reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .map_err(|e| e.to_string())?
        } else {
            global_healing_pool()
                .get_blocking_client()
                .map_err(|e| e.to_string())?
        };

        self.load_balancer.acquire_flight(provider);
        let start = Instant::now();
        let mut req_builder = blocking_client.post(&endpoint).json(&req_formatted);
        for (k, v) in client.auth_headers(&key) {
            if !key.is_empty() || k.eq_ignore_ascii_case("content-type") {
                req_builder = req_builder.header(k, v);
            }
        }

        let resp = match req_builder.send() {
            Ok(r) => r,
            Err(e) => {
                let err_str = e.to_string();
                let _ = global_healing_pool().report_transport_error(&err_str);
                self.load_balancer
                    .release_flight(provider, start.elapsed().as_secs_f64() * 1000.0);
                self.circuit_registry.record_failure(provider, None);
                if let Some(vk) = vk_opt {
                    self.governance.release_and_record_spend(vk, 0.0);
                }
                return Err(format!("Provider {} upstream request failed: {}", provider.as_str(), err_str));
            }
        };

        let status = resp.status().as_u16();
        let headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_lowercase(), s.to_string())))
            .collect();
        let body_text = resp.text().map_err(|e| e.to_string())?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Self::record_rate_limit_windows(provider, &headers);

        if status >= 400 {
            self.record_upstream_failure(provider, &key, status, &headers, &body_text, latency_ms);
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Err(format!("Provider {} returned HTTP {}: {}", provider.as_str(), status, body_text));
        }

        let resp_json: Value = serde_json::from_str(&body_text)
            .map_err(|e| format!("Invalid JSON from {}: {}", provider.as_str(), e))?;
        let parsed = client.parse_response(&resp_json)?;
        if !key.is_empty() {
            self.key_manager.record_result(provider, &key, None, None);
        }
        self.record_usage_and_cache(session_id, vk_opt, provider, &prompt_str, &parsed, latency_ms)?;
        Ok(parsed)
    }

    /// Async upstream dispatch with 7-D ranking + multi-provider cascade failover via `AutoHealingClientPool`.
    #[allow(clippy::too_many_arguments)]
    pub async fn dispatch_with_cascade_async(
        &self,
        session_id: &str,
        vk_opt: Option<&str>,
        explicit_api_key: Option<&str>,
        explicit_endpoint: Option<&str>,
        candidates: &[RouteCandidate],
        policy: &RoutingPolicy,
        payload: &mut Value,
    ) -> Result<(ProviderId, UnifiedResponse), String> {
        if let Some(vk) = vk_opt {
            self.governance.check_and_acquire(vk, 1000)?;
        }

        let mapped_effort = match policy.effort {
            Some(ReasoningEffort::Low) => Some(EffortLevel::Low),
            Some(ReasoningEffort::Medium) => Some(EffortLevel::Medium),
            Some(ReasoningEffort::High) => Some(EffortLevel::High),
            Some(ReasoningEffort::XHigh) => Some(EffortLevel::XHigh),
            Some(ReasoningEffort::Max) => Some(EffortLevel::Max),
            None => None,
        };
        let effort_headers: Vec<(String, String)> = mapped_effort
            .map(|e| vec![("x-toksi-effort".to_string(), e.as_str().to_string())])
            .unwrap_or_default();
        let hr_stats = HeadroomEffortCompressor::gateway_default().compress_json_payload(
            "/v1/chat/completions",
            &effort_headers,
            payload,
        );
        Self::emit_headroom_telemetry(&hr_stats);
        let prompt_str = payload.to_string();

        let ranked = self
            .router
            .route_ranked(candidates, policy, Some(&self.circuit_registry));
        if ranked.is_empty() {
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Err("No healthy routing candidates satisfy policy constraints".to_string());
        }

        // Dual-tier (exact SHA-256 + HNSW ANN) cache hit check against primary ranked candidate
        let primary_provider = ranked[0].candidate.provider;
        if let Some(cached_resp) = self.lookup_cache(primary_provider.as_str(), &prompt_str) {
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Ok((
                primary_provider,
                UnifiedResponse {
                    id: "cache-hit".to_string(),
                    model: payload
                        .get("model")
                        .and_then(Value::as_str)
                        .unwrap_or(&ranked[0].candidate.model)
                        .to_string(),
                    content: cached_resp,
                    role: "assistant".to_string(),
                    finish_reason: Some("stop".to_string()),
                    input_tokens: 0,
                    output_tokens: 0,
                    raw: Value::Null,
                },
            ));
        }

        let async_client = match global_healing_pool().get_async_client() {
            Ok(c) => c,
            Err(e) => {
                if let Some(vk) = vk_opt {
                    self.governance.release_and_record_spend(vk, 0.0);
                }
                return Err(e.to_string());
            }
        };

        let unified_req = Self::build_unified_request(payload);
        let mut last_error = String::from("No upstream candidates attempted");

        for route in ranked {
            let provider = route.candidate.provider;
            let key = match self.resolve_key(provider, explicit_api_key) {
                Ok(k) => k,
                Err(e) => {
                    last_error = e;
                    continue;
                }
            };

            let client = build_provider_client(provider);
            let mut req_formatted = client.format_request(&unified_req);
            let raw_model = payload.get("model").and_then(Value::as_str).unwrap_or("");
            let eff = crate::headroom::effort::detect_effort_level(&[], Some(payload));
            crate::gateway::sampling::apply_sampling_params_to_payload(
                provider.as_str(),
                raw_model,
                eff,
                &crate::gateway::sampling::SamplingParams::default(),
                &mut req_formatted,
            );
            let endpoint = Self::resolve_candidate_endpoint(
                explicit_endpoint,
                route.candidate.base_url.as_deref(),
                client.as_ref(),
            );
            let is_loopback = endpoint.starts_with("http://127.0.0.1")
                || endpoint.starts_with("http://localhost");
            let dispatch_client = if is_loopback {
                reqwest::Client::builder()
                    .no_proxy()
                    .timeout(std::time::Duration::from_secs(60))
                    .build()
                    .unwrap_or_else(|_| async_client.clone())
            } else {
                async_client.clone()
            };

            self.load_balancer.acquire_flight(provider);
            let start = Instant::now();
            let mut builder = dispatch_client.post(&endpoint).json(&req_formatted);
            for (k, v) in client.auth_headers(&key) {
                if !key.is_empty() || k.eq_ignore_ascii_case("content-type") {
                    builder = builder.header(k, v);
                }
            }

            let resp = match builder.send().await {
                Ok(r) => r,
                Err(e) => {
                    let err_str = e.to_string();
                    let _ = global_healing_pool().report_transport_error(&err_str);
                    self.load_balancer
                        .release_flight(provider, start.elapsed().as_secs_f64() * 1000.0);
                    self.circuit_registry.record_failure(provider, None);
                    last_error = format!("{} transport failed: {}", provider.as_str(), err_str);
                    continue;
                }
            };

            let status = resp.status().as_u16();
            let headers: HashMap<String, String> = resp
                .headers()
                .iter()
                .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_lowercase(), s.to_string())))
                .collect();
            let body_text = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    self.load_balancer
                        .release_flight(provider, start.elapsed().as_secs_f64() * 1000.0);
                    last_error = format!("{} body read failed: {}", provider.as_str(), e);
                    continue;
                }
            };
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            Self::record_rate_limit_windows(provider, &headers);

            if status >= 400 {
                self.record_upstream_failure(provider, &key, status, &headers, &body_text, latency_ms);
                last_error = format!("{} HTTP {}: {}", provider.as_str(), status, body_text);
                continue;
            }

            let resp_json: Value = match serde_json::from_str(&body_text) {
                Ok(v) => v,
                Err(e) => {
                    self.load_balancer.release_flight(provider, latency_ms);
                    last_error = format!("{} invalid JSON: {}", provider.as_str(), e);
                    continue;
                }
            };

            let parsed = match client.parse_response(&resp_json) {
                Ok(p) => p,
                Err(e) => {
                    self.load_balancer.release_flight(provider, latency_ms);
                    last_error = format!("{} parse error: {}", provider.as_str(), e);
                    continue;
                }
            };

            if let Some(ref gate) = policy.quality_gate
                && !gate.passes_response(&parsed)
            {
                self.load_balancer.release_flight(provider, latency_ms);
                last_error = format!("{} response failed QualityGate", provider.as_str());
                continue;
            }

            if !key.is_empty() {
                self.key_manager.record_result(provider, &key, None, None);
            }
            self.record_usage_and_cache(session_id, vk_opt, provider, &prompt_str, &parsed, latency_ms)?;
            return Ok((provider, parsed));
        }

        if let Some(vk) = vk_opt {
            self.governance.release_and_record_spend(vk, 0.0);
        }
        Err(last_error)
    }

    /// Streams an upstream SSE response directly without buffering the entire body,
    /// returning the chosen provider, raw `reqwest::Response`, and model name.
    pub async fn dispatch_stream_with_cascade_async(
        &self,
        vk_opt: Option<&str>,
        explicit_api_key: Option<&str>,
        explicit_endpoint: Option<&str>,
        candidates: &[RouteCandidate],
        policy: &RoutingPolicy,
        payload: &mut Value,
    ) -> Result<(ProviderId, reqwest::Response, String), String> {
        if let Some(vk) = vk_opt {
            self.governance.check_and_acquire(vk, 1000)?;
        }

        let mapped_effort = match policy.effort {
            Some(ReasoningEffort::Low) => Some(EffortLevel::Low),
            Some(ReasoningEffort::Medium) => Some(EffortLevel::Medium),
            Some(ReasoningEffort::High) => Some(EffortLevel::High),
            Some(ReasoningEffort::XHigh) => Some(EffortLevel::XHigh),
            Some(ReasoningEffort::Max) => Some(EffortLevel::Max),
            None => None,
        };
        let effort_headers: Vec<(String, String)> = mapped_effort
            .map(|e| vec![("x-toksi-effort".to_string(), e.as_str().to_string())])
            .unwrap_or_default();
        let hr_stats = HeadroomEffortCompressor::gateway_default().compress_json_payload(
            "/v1/chat/completions",
            &effort_headers,
            payload,
        );
        Self::emit_headroom_telemetry(&hr_stats);

        let ranked = self
            .router
            .route_ranked(candidates, policy, Some(&self.circuit_registry));
        if ranked.is_empty() {
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Err("No healthy routing candidates satisfy policy constraints".to_string());
        }

        let async_client = match global_healing_pool().get_async_client() {
            Ok(c) => c,
            Err(e) => {
                if let Some(vk) = vk_opt {
                    self.governance.release_and_record_spend(vk, 0.0);
                }
                return Err(e.to_string());
            }
        };

        let unified_req = Self::build_unified_request(payload);
        let model_name = unified_req.model.clone();
        let mut last_error = String::from("No upstream candidates attempted");

        for route in ranked {
            let provider = route.candidate.provider;
            let key = match self.resolve_key(provider, explicit_api_key) {
                Ok(k) => k,
                Err(e) => {
                    last_error = e;
                    continue;
                }
            };

            let client = build_provider_client(provider);
            let mut req_formatted = client.format_request(&unified_req);
            let raw_model = payload.get("model").and_then(Value::as_str).unwrap_or("");
            let eff = crate::headroom::effort::detect_effort_level(&[], Some(payload));
            crate::gateway::sampling::apply_sampling_params_to_payload(
                provider.as_str(),
                raw_model,
                eff,
                &crate::gateway::sampling::SamplingParams::default(),
                &mut req_formatted,
            );
            let endpoint = Self::resolve_candidate_endpoint(
                explicit_endpoint,
                route.candidate.base_url.as_deref(),
                client.as_ref(),
            );
            let is_loopback = endpoint.starts_with("http://127.0.0.1")
                || endpoint.starts_with("http://localhost");
            let dispatch_client = if is_loopback {
                reqwest::Client::builder()
                    .no_proxy()
                    .timeout(std::time::Duration::from_secs(60))
                    .build()
                    .unwrap_or_else(|_| async_client.clone())
            } else {
                async_client.clone()
            };

            self.load_balancer.acquire_flight(provider);
            let start = Instant::now();
            let mut builder = dispatch_client.post(&endpoint).json(&req_formatted);
            for (k, v) in client.auth_headers(&key) {
                if !key.is_empty() || k.eq_ignore_ascii_case("content-type") {
                    builder = builder.header(k, v);
                }
            }

            let resp = match builder.send().await {
                Ok(r) => r,
                Err(e) => {
                    let err_str = e.to_string();
                    let _ = global_healing_pool().report_transport_error(&err_str);
                    self.load_balancer
                        .release_flight(provider, start.elapsed().as_secs_f64() * 1000.0);
                    self.circuit_registry.record_failure(provider, None);
                    last_error = format!("{} transport failed: {}", provider.as_str(), err_str);
                    continue;
                }
            };

            let status = resp.status().as_u16();
            let headers: HashMap<String, String> = resp
                .headers()
                .iter()
                .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_lowercase(), s.to_string())))
                .collect();
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            Self::record_rate_limit_windows(provider, &headers);

            if status >= 400 {
                let body_text = resp.text().await.unwrap_or_default();
                self.record_upstream_failure(provider, &key, status, &headers, &body_text, latency_ms);
                last_error = format!("{} HTTP {}: {}", provider.as_str(), status, body_text);
                continue;
            }

            self.load_balancer.release_flight(provider, latency_ms);
            self.circuit_registry.record_success(provider);
            if !key.is_empty() {
                self.key_manager.record_result(provider, &key, None, None);
            }
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Ok((provider, resp, model_name));
        }

        if let Some(vk) = vk_opt {
            self.governance.release_and_record_spend(vk, 0.0);
        }
        Err(last_error)
    }
}
