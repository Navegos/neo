// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/providers/multimodal/mod.rs

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

pub struct MultimodalProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl MultimodalProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for MultimodalProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn endpoint(&self) -> String {
        self.endpoint_url.clone()
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        match self.provider_id {
            ProviderId::ElevenLabs => {
                h.insert("xi-api-key".to_string(), api_key.to_string());
            }
            _ => {
                h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
            }
        }
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        json!({
            "model": req.model,
            "prompt": req.messages.last().and_then(|m| m.get("content")).cloned().unwrap_or(json!("")),
        })
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        Ok(UnifiedResponse {
            id: "multimodal-resp".to_string(),
            model: "multimodal".to_string(),
            content: resp.to_string(),
            role: "assistant".to_string(),
            finish_reason: Some("stop".to_string()),
            input_tokens: 10,
            output_tokens: 10,
            raw: resp.clone(),
        })
    }
}

