// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/providers/frontier/mod.rs

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

/// Anthropic Messages client (Claude 3.5 / 3.7)
pub struct AnthropicProvider {
    pub base_url: String,
    pub api_version: String,
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self {
            base_url: ProviderId::Anthropic.default_endpoint().to_string(),
            api_version: "2023-06-01".to_string(),
        }
    }
}

impl ProviderClient for AnthropicProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Anthropic
    }

    fn endpoint(&self) -> String {
        self.base_url.clone()
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("x-api-key".to_string(), api_key.to_string());
        h.insert("anthropic-version".to_string(), self.api_version.clone());
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        let mut body = json!({
            "model": req.model,
            "messages": req.messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
        });
        if let Some(ref sys) = req.system {
            body["system"] = json!(sys);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(s) = req.stream {
            body["stream"] = json!(s);
        }
        if let Some(ref tools) = req.tools {
            body["tools"] = json!(tools);
        }
        body
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        let id = resp.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let model = resp.get("model").and_then(Value::as_str).unwrap_or("").to_string();
        let content = if let Some(arr) = resp.get("content").and_then(Value::as_array) {
            arr.iter()
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("")
        } else if let Some(s) = resp.get("content").and_then(Value::as_str) {
            s.to_string()
        } else if let Some(c) = resp
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
        {
            c.to_string()
        } else {
            String::new()
        };
        let finish_reason = resp.get("stop_reason").and_then(Value::as_str).map(|s| s.to_string());
        let input_tokens = resp.get("usage").and_then(|u| u.get("input_tokens")).and_then(Value::as_u64).unwrap_or(0) as u32;
        let output_tokens = resp.get("usage").and_then(|u| u.get("output_tokens")).and_then(Value::as_u64).unwrap_or(0) as u32;

        Ok(UnifiedResponse {
            id,
            model,
            content,
            role: "assistant".to_string(),
            finish_reason,
            input_tokens,
            output_tokens,
            raw: resp.clone(),
        })
    }
}

/// OpenAI ChatCompletions client (GPT-4o, o1, o3-mini)
pub struct OpenAIProvider {
    pub base_url: String,
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self {
            base_url: ProviderId::OpenAI.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for OpenAIProvider {
    fn id(&self) -> ProviderId {
        ProviderId::OpenAI
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        let mut msgs = Vec::new();
        if let Some(ref sys) = req.system {
            msgs.push(json!({ "role": "system", "content": sys }));
        }
        msgs.extend(req.messages.clone());

        let mut body = json!({
            "model": req.model,
            "messages": msgs,
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
        if let Some(ref tools) = req.tools {
            body["tools"] = json!(tools);
        }
        body
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        let id = resp.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let model = resp.get("model").and_then(Value::as_str).unwrap_or("").to_string();
        let choice = resp.get("choices").and_then(Value::as_array).and_then(|a| a.first());
        let content = choice
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let finish_reason = choice
            .and_then(|c| c.get("finish_reason"))
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let input_tokens = resp.get("usage").and_then(|u| u.get("prompt_tokens")).and_then(Value::as_u64).unwrap_or(0) as u32;
        let output_tokens = resp.get("usage").and_then(|u| u.get("completion_tokens")).and_then(Value::as_u64).unwrap_or(0) as u32;

        Ok(UnifiedResponse {
            id,
            model,
            content,
            role: "assistant".to_string(),
            finish_reason,
            input_tokens,
            output_tokens,
            raw: resp.clone(),
        })
    }
}

/// Google AI Studio Gemini client
pub struct GeminiProvider {
    pub base_url: String,
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self {
            base_url: ProviderId::Gemini.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for GeminiProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Gemini
    }

    fn endpoint(&self) -> String {
        format!("{}/models/gemini-2.0-flash:generateContent", self.base_url)
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("x-goog-api-key".to_string(), api_key.to_string());
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        let contents = req.messages.iter().map(|m| {
            let role = m.get("role").and_then(Value::as_str).unwrap_or("user");
            let text = m.get("content").and_then(Value::as_str).unwrap_or("");
            json!({
                "role": if role == "assistant" { "model" } else { "user" },
                "parts": [{ "text": text }]
            })
        }).collect::<Vec<_>>();

        json!({
            "contents": contents,
        })
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        let cand = resp.get("candidates").and_then(Value::as_array).and_then(|a| a.first());
        let content = cand
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(Value::as_array)
            .and_then(|p| p.first())
            .and_then(|part| part.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let finish_reason = cand.and_then(|c| c.get("finishReason")).and_then(Value::as_str).map(|s| s.to_string());
        let meta = resp.get("usageMetadata");
        let input_tokens = meta.and_then(|m| m.get("promptTokenCount")).and_then(Value::as_u64).unwrap_or(0) as u32;
        let output_tokens = meta.and_then(|m| m.get("candidatesTokenCount")).and_then(Value::as_u64).unwrap_or(0) as u32;

        Ok(UnifiedResponse {
            id: "gemini-resp".to_string(),
            model: "gemini-2.0-flash".to_string(),
            content,
            role: "assistant".to_string(),
            finish_reason,
            input_tokens,
            output_tokens,
            raw: resp.clone(),
        })
    }
}

/// Generic Frontier client for Azure, Bedrock, Vertex, Databricks
pub struct GenericFrontierProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl GenericFrontierProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for GenericFrontierProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn endpoint(&self) -> String {
        self.endpoint_url.clone()
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        match self.provider_id {
            ProviderId::Azure => {
                h.insert("api-key".to_string(), api_key.to_string());
            }
            ProviderId::Bedrock => {
                h.insert("x-amz-security-token".to_string(), api_key.to_string());
            }
            ProviderId::Vertex | ProviderId::Databricks => {
                h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
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
            "messages": req.messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
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
