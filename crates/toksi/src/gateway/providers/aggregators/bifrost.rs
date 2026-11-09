// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/aggregators/bifrost.rs
// created: 2026-11-09

use std::collections::HashMap;
use std::time::Duration;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

/// Bifrost BYOK Provider Integration Client
/// Connects to local or remote Bifrost gateway instance with health probes,
/// failover routing, and load balancing.
pub struct BifrostProviderClient {
    pub endpoint_url: String,
}

impl BifrostProviderClient {
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.trim_end_matches('/').to_string(),
        }
    }

    /// Health probe checking reachability of Bifrost instance.
    pub fn health_probe(&self, timeout: Duration) -> bool {
        let ep = format!("{}/healthz", self.endpoint_url);
        let client = match reqwest::blocking::Client::builder().timeout(timeout).build() {
            Ok(c) => c,
            Err(_) => return false,
        };
        match client.get(&ep).send() {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

impl Default for BifrostProviderClient {
    fn default() -> Self {
        Self::new(ProviderId::Bifrost.default_endpoint())
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

        let prompt_tokens = resp
            .get("usage")
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(50) as u32;

        let completion_tokens = resp
            .get("usage")
            .and_then(|u| u.get("completion_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(50) as u32;

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
        let client = BifrostProviderClient::default();
        assert_eq!(client.id(), ProviderId::Bifrost);
        assert!(client.endpoint().contains("/chat/completions"));

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
        let parsed = client.parse_response(&mock_resp).expect("Failed to parse response");
        assert_eq!(parsed.id, "bifrost-123");
        assert_eq!(parsed.content, "pong");
        assert_eq!(parsed.input_tokens, 15);
        assert_eq!(parsed.output_tokens, 5);
    }
}
