// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/providers/specialized/mod.rs

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

pub struct SpecializedProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl SpecializedProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for SpecializedProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.endpoint_url.trim_end_matches('/'))
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        json!({
            "model": req.model,
            "messages": req.messages,
            "max_tokens": req.max_tokens.unwrap_or(2048),
        })
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        let id = resp.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let model = resp.get("model").and_then(Value::as_str).unwrap_or("").to_string();
        let content = resp
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        Ok(UnifiedResponse {
            id,
            model,
            content,
            role: "assistant".to_string(),
            finish_reason: Some("stop".to_string()),
            input_tokens: 100,
            output_tokens: 100,
            raw: resp.clone(),
        })
    }
}

