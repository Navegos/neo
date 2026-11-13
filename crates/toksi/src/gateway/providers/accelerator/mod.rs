// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/accelerator/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

pub struct AcceleratorProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl AcceleratorProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for AcceleratorProvider {
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
        let mut body = json!({
            "model": req.model,
            "messages": req.messages,
        });
        if let Some(m) = req.max_tokens {
            body["max_tokens"] = json!(m);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(s) = req.stream {
            body["stream"] = json!(s);
        }
        body
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        crate::gateway::providers::parse_openai_chat_response(resp, "")
    }
}

