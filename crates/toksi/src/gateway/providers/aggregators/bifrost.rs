// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/aggregators/bifrost.rs
// created: 2026-11-09

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde_json::{json, Value};

use crate::db::operations::ToksiConfigDb;
use crate::db::schema::ProviderModelRecord;
use crate::gateway::agent_cli::tier_check::{
    discover_models_zero_cost, record_ratelimit_headers, TierCheckError,
};
use crate::gateway::loadbalancer::LoadBalancer;
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};
use crate::gateway::reconnect::global_healing_pool;
use crate::gateway::router::{RouteCandidate, Router, RoutingPolicy};
use crate::headroom::effort::HeadroomEffortCompressor;
use crate::util::logger::L;

/// Bifrost BYOK Provider Integration Client.
/// Connects to local or remote Bifrost gateway instances with health probes,
/// custom `base_url` overrides, failover routing, `/v1/models` discovery, `/v1/mcp` bridge,
/// and `HeadroomEffortCompressor` + `LoadBalancer` + 7-D `Router` dispatch.
#[derive(Debug, Clone)]
pub struct BifrostProviderClient {
    pub endpoint_url: String,
    pub fallback_endpoints: Vec<String>,
}

impl BifrostProviderClient {
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.trim_end_matches('/').to_string(),
            fallback_endpoints: Vec::new(),
        }
    }

    /// Resolves custom `base_url` override from `config.toml` / runtime overrides,
    /// falling back to `ProviderId::Bifrost.default_endpoint()`.
    pub fn from_config() -> Self {
        let base = crate::util::ports::resolve_custom_base_url("bifrost")
            .unwrap_or_else(|| ProviderId::Bifrost.default_endpoint().to_string());
        Self::new(&base)
    }

    pub fn with_fallbacks(mut self, fallbacks: Vec<String>) -> Self {
        self.fallback_endpoints = fallbacks
            .into_iter()
            .map(|u| u.trim().trim_end_matches('/').to_string())
            .filter(|u| !u.is_empty())
            .collect();
        self
    }

    pub fn all_endpoints(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(1 + self.fallback_endpoints.len());
        out.push(self.endpoint_url.clone());
        for fb in &self.fallback_endpoints {
            if !out.contains(fb) {
                out.push(fb.clone());
            }
        }
        out
    }

    /// Bifrost MCP endpoint (`/v1/mcp`).
    pub fn mcp_endpoint(&self) -> String {
        let root = self.endpoint_url.trim_end_matches("/v1");
        format!("{}/v1/mcp", root)
    }

    /// Health probe checking reachability of a specific Bifrost endpoint.
    pub fn probe_endpoint(endpoint_url: &str, timeout: Duration) -> bool {
        let root = endpoint_url.trim_end_matches('/').trim_end_matches("/v1");
        let ep = format!("{}/healthz", root);
        let is_loopback = ep.starts_with("http://127.0.0.1") || ep.starts_with("http://localhost");
        let mut builder = reqwest::blocking::Client::builder().timeout(timeout);
        if is_loopback {
            builder = builder.no_proxy();
        }
        let Ok(client) = builder.build() else {
            return false;
        };
        match client.get(&ep).send() {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Health probe checking reachability of primary Bifrost instance.
    pub fn health_probe(&self, timeout: Duration) -> bool {
        Self::probe_endpoint(&self.endpoint_url, timeout)
    }

    /// Returns the first healthy endpoint across primary and fallbacks.
    pub fn first_healthy_endpoint(&self, timeout: Duration) -> Option<String> {
        self.all_endpoints()
            .into_iter()
            .find(|ep| Self::probe_endpoint(ep, timeout))
    }

    /// Zero-cost `/v1/models` discovery against Bifrost.
    pub fn discover_models(
        &self,
        api_key: Option<&str>,
        timeout: Duration,
    ) -> Result<Vec<ProviderModelRecord>, TierCheckError> {
        discover_models_zero_cost(&self.endpoint_url, api_key, ProviderId::Bifrost.as_str(), timeout)
    }

    /// Dispatches a chat/completions payload through `HeadroomEffortCompressor`,
    /// `LoadBalancer`, and 7-D `Router::route_ranked`, failing over cleanly across
    /// configured endpoints and API keys on HTTP 429 or unbilled responses.
    pub fn dispatch_with_failover(
        &self,
        payload: &mut Value,
        api_keys: &[String],
        load_balancer: &LoadBalancer,
        router: &Router,
        policy: &RoutingPolicy,
        mut db: Option<&mut ToksiConfigDb>,
    ) -> Result<UnifiedResponse, String> {
        // Every configured BYOK payload MUST pass through Headroom optimizer.
        let _hr_stats = HeadroomEffortCompressor::gateway_default()
            .compress_json_payload("/v1/chat/completions", &[], payload);

        let model = payload
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("default")
            .to_string();

        // Evaluate 7-D candidate ranking via mofa-engine Router.
        let candidate = RouteCandidate::for_provider(ProviderId::Bifrost, &model);
        let _ranked = router.route_ranked(&[candidate], policy, None);

        let keys: Vec<String> = if api_keys.is_empty() {
            vec![String::new()]
        } else {
            api_keys.to_vec()
        };

        let req_formatted = self.format_request(&UnifiedRequest {
            model,
            messages: payload
                .get("messages")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            system: payload.get("system").and_then(Value::as_str).map(str::to_string),
            max_tokens: payload.get("max_tokens").and_then(Value::as_u64).map(|u| u as u32),
            temperature: payload.get("temperature").and_then(Value::as_f64).map(|f| f as f32),
            stream: payload.get("stream").and_then(Value::as_bool),
            tools: payload.get("tools").and_then(Value::as_array).cloned(),
            extra: HashMap::new(),
        });

        let mut last_err = String::from("No Bifrost endpoints or keys available");
        for ep in self.all_endpoints() {
            let chat_url = format!("{}/chat/completions", ep.trim_end_matches('/'));
            let is_loopback =
                chat_url.starts_with("http://127.0.0.1") || chat_url.starts_with("http://localhost");
            let client = if is_loopback {
                reqwest::blocking::Client::builder()
                    .no_proxy()
                    .timeout(Duration::from_secs(60))
                    .build()
                    .map_err(|e| e.to_string())?
            } else {
                global_healing_pool()
                    .get_blocking_client()
                    .map_err(|e| e.to_string())?
            };

            for (idx, key) in keys.iter().enumerate() {
                load_balancer.acquire_flight(ProviderId::Bifrost);
                let start = Instant::now();
                let mut req_builder = client.post(&chat_url).json(&req_formatted);
                for (hk, hv) in self.auth_headers(key) {
                    if !key.is_empty() || hk.eq_ignore_ascii_case("content-type") {
                        req_builder = req_builder.header(hk, hv);
                    }
                }

                let resp = match req_builder.send() {
                    Ok(r) => r,
                    Err(e) => {
                        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                        load_balancer.release_flight(ProviderId::Bifrost, latency_ms);
                        let err_str = e.to_string();
                        let _ = global_healing_pool().report_transport_error(&err_str);
                        L.warn(&format!(
                            "Bifrost transport failed on '{}' (key #{}): {}; failing over",
                            chat_url, idx, err_str
                        ));
                        last_err = err_str;
                        continue;
                    }
                };

                let status = resp.status().as_u16();
                let headers: HashMap<String, String> = resp
                    .headers()
                    .iter()
                    .filter_map(|(k, v)| {
                        v.to_str()
                            .ok()
                            .map(|s| (k.as_str().to_ascii_lowercase(), s.to_string()))
                    })
                    .collect();
                let body_text = resp.text().unwrap_or_default();
                let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                load_balancer.release_flight(ProviderId::Bifrost, latency_ms);

                if let Some(db_ref) = db.as_deref_mut() {
                    let _ = record_ratelimit_headers(
                        db_ref,
                        ProviderId::Bifrost.as_str(),
                        &format!("ep-{}", idx),
                        &headers,
                    );
                }

                if status == 429 || status == 402 || status >= 500 {
                    L.warn(&format!(
                        "Bifrost '{}' (key #{}) returned HTTP {}; failing over to next profile",
                        chat_url, idx, status
                    ));
                    last_err = format!("HTTP {} from {}: {}", status, chat_url, body_text);
                    continue;
                }

                if status >= 400 {
                    last_err = format!("HTTP {} from {}: {}", status, chat_url, body_text);
                    continue;
                }

                let resp_json: Value = serde_json::from_str(&body_text)
                    .map_err(|e| format!("Invalid JSON from {}: {}", chat_url, e))?;
                let parsed = self.parse_response(&resp_json)?;
                L.info(&format!(
                    "Bifrost dispatch succeeded on '{}' (model={})",
                    chat_url, parsed.model
                ));
                return Ok(parsed);
            }
        }

        Err(last_err)
    }
}

impl Default for BifrostProviderClient {
    fn default() -> Self {
        Self::from_config()
    }
}

impl ProviderClient for BifrostProviderClient {
    fn id(&self) -> ProviderId {
        ProviderId::Bifrost
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.endpoint_url)
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        h.insert("content-type".to_string(), "application/json".to_string());
        h.insert("X-Bifrost-Client".to_string(), "toksi-gateway/3.4".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        json!({
            "model": req.model,
            "messages": req.messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": req.stream.unwrap_or(false),
        })
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        let id = resp.get("id").and_then(Value::as_str).unwrap_or("bifrost-resp").to_string();
        let model = resp.get("model").and_then(Value::as_str).unwrap_or("bifrost").to_string();
        let content = resp
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let (prompt_tokens, completion_tokens) =
            crate::gateway::providers::parse_openai_usage(resp);

        Ok(UnifiedResponse {
            id,
            model,
            content,
            role: "assistant".to_string(),
            finish_reason: Some("stop".to_string()),
            input_tokens: prompt_tokens,
            output_tokens: completion_tokens,
            raw: resp.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bifrost_provider_client() {
        let client = BifrostProviderClient::default()
            .with_fallbacks(vec!["http://127.0.0.1:8081/v1".to_string()]);
        assert_eq!(client.id(), ProviderId::Bifrost);
        assert!(client.endpoint().contains("/chat/completions"));
        assert!(client.mcp_endpoint().ends_with("/v1/mcp"));
        assert_eq!(client.all_endpoints().len(), 2);

        let headers = client.auth_headers("sk-bifrost-test");
        assert_eq!(headers.get("Authorization"), Some(&"Bearer sk-bifrost-test".to_string()));
        assert_eq!(headers.get("X-Bifrost-Client"), Some(&"toksi-gateway/3.4".to_string()));

        let req = UnifiedRequest {
            model: "gpt-4o".to_string(),
            messages: vec![json!({"role": "user", "content": "hello"})],
            system: None,
            max_tokens: Some(2048),
            temperature: None,
            stream: None,
            tools: None,
            extra: HashMap::new(),
        };
        let formatted = client.format_request(&req);
        assert_eq!(formatted["model"], "gpt-4o");

        let mock_resp = json!({
            "id": "bifrost-123",
            "model": "gpt-4o",
            "choices": [{
                "message": {"role": "assistant", "content": "pong"}
            }],
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 5
            }
        });
        let parsed = client.parse_response(&mock_resp);
        assert!(parsed.is_ok());
        if let Ok(p) = parsed {
            assert_eq!(p.id, "bifrost-123");
            assert_eq!(p.content, "pong");
            assert_eq!(p.input_tokens, 15);
            assert_eq!(p.output_tokens, 5);
        }
    }
}
