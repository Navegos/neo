// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/aggregators/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-13

pub mod bifrost;

pub use bifrost::BifrostProviderClient;

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

pub struct AggregatorProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl AggregatorProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.effective_endpoint(),
        }
    }
}

impl ProviderClient for AggregatorProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.endpoint_url.trim_end_matches('/'))
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        if self.provider_id == ProviderId::OpenRouter {
            h.insert("HTTP-Referer".to_string(), "https://toksi.ai".to_string());
            h.insert("X-Title".to_string(), "Toksi AI Gateway".to_string());
        } else if self.provider_id == ProviderId::Portkey {
            h.insert("x-portkey-api-key".to_string(), api_key.to_string());
            h.insert("x-portkey-provider".to_string(), "openai".to_string());
        } else if self.provider_id == ProviderId::Cloudflare {
            h.insert("cf-aig-authorization".to_string(), format!("Bearer {}", api_key));
        }
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        let mut messages = req.messages.clone();
        if let Some(ref sys) = req.system {
            if !sys.is_empty() && !messages.iter().any(|m| m.get("role").and_then(Value::as_str) == Some("system")) {
                messages.insert(0, json!({ "role": "system", "content": sys }));
            }
        }
        let mut body = json!({
            "model": req.model,
            "messages": messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
        });
        if let Some(temp) = req.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(stream) = req.stream {
            body["stream"] = json!(stream);
        }
        if let Some(ref tools) = req.tools {
            body["tools"] = json!(tools);
        }
        body
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

        let (input_tokens, output_tokens) = crate::gateway::providers::parse_openai_usage(resp);

        Ok(UnifiedResponse {
            id,
            model,
            content,
            role: "assistant".to_string(),
            finish_reason: Some("stop".to_string()),
            input_tokens,
            output_tokens,
            raw: resp.clone(),
        })
    }
}

/// Optional router-specific header metadata for Category 5 proxy/gateway/router providers.
#[derive(Debug, Clone, Default)]
pub struct RouterHeaderConfig {
    pub upstream_provider: Option<String>,
    pub config_id: Option<String>,
    pub custom_host: Option<String>,
    pub cache_ttl_secs: Option<u64>,
    pub skip_cache: bool,
    pub trace_property: Option<(String, String)>,
}

/// All 12 Category 5 Agentic Proxy, Gateway & Router Providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category5GatewayId {
    Bifrost,
    OpenRouter,
    Portkey,
    LiteLLM,
    Cloudflare,
    Helicone,
    Martian,
    VercelAIGateway,
    BerriAI,
    APIClaw,
    Langfuse,
    UnifyAI,
}

#[derive(Debug, Clone, Copy)]
pub struct Category5GatewayProfile {
    pub id: Category5GatewayId,
    pub name: &'static str,
    pub default_endpoint: &'static str,
    pub primary_header: &'static str,
}

pub const CATEGORY_5_GATEWAYS: [Category5GatewayProfile; 12] = [
    Category5GatewayProfile { id: Category5GatewayId::Bifrost, name: "bifrost", default_endpoint: "http://127.0.0.1:8080/v1", primary_header: "x-bifrost-route" },
    Category5GatewayProfile { id: Category5GatewayId::OpenRouter, name: "openrouter", default_endpoint: "https://openrouter.ai/api/v1", primary_header: "HTTP-Referer" },
    Category5GatewayProfile { id: Category5GatewayId::Portkey, name: "portkey", default_endpoint: "https://api.portkey.ai/v1", primary_header: "x-portkey-api-key" },
    Category5GatewayProfile { id: Category5GatewayId::LiteLLM, name: "litellm", default_endpoint: "http://127.0.0.1:4000/v1", primary_header: "Authorization" },
    Category5GatewayProfile { id: Category5GatewayId::Cloudflare, name: "cloudflare", default_endpoint: "https://gateway.ai.cloudflare.com/v1", primary_header: "cf-aig-authorization" },
    Category5GatewayProfile { id: Category5GatewayId::Helicone, name: "helicone", default_endpoint: "https://oai.helicone.ai/v1", primary_header: "Helicone-Auth" },
    Category5GatewayProfile { id: Category5GatewayId::Martian, name: "martian", default_endpoint: "https://withmartian.com/api/openai/v1", primary_header: "Authorization" },
    Category5GatewayProfile { id: Category5GatewayId::VercelAIGateway, name: "vercel-ai-gateway", default_endpoint: "https://ai-gateway.vercel.sh/v1", primary_header: "ai-gateway-auth-token" },
    Category5GatewayProfile { id: Category5GatewayId::BerriAI, name: "berriai", default_endpoint: "https://api.berri.ai/v1", primary_header: "Authorization" },
    Category5GatewayProfile { id: Category5GatewayId::APIClaw, name: "apiclaw", default_endpoint: "https://api.apiclaw.dev/v1", primary_header: "x-apiclaw-key" },
    Category5GatewayProfile { id: Category5GatewayId::Langfuse, name: "langfuse", default_endpoint: "https://cloud.langfuse.com/api/public", primary_header: "x-langfuse-public-key" },
    Category5GatewayProfile { id: Category5GatewayId::UnifyAI, name: "unify", default_endpoint: "https://api.unify.ai/v0", primary_header: "Authorization" },
];

/// Injects Category 5 router headers (`x-portkey-*`, `cf-aig-*`, `Helicone-*`) into outbound requests.
pub fn inject_category5_router_headers(
    gateway: Category5GatewayId,
    api_key: &str,
    cfg: &RouterHeaderConfig,
    headers: &mut HashMap<String, String>,
) {
    match gateway {
        Category5GatewayId::Portkey => {
            headers.insert("x-portkey-api-key".to_string(), api_key.to_string());
            if let Some(p) = &cfg.upstream_provider {
                headers.insert("x-portkey-provider".to_string(), p.clone());
            }
            if let Some(c) = &cfg.config_id {
                headers.insert("x-portkey-config".to_string(), c.clone());
            }
            if let Some(h) = &cfg.custom_host {
                headers.insert("x-portkey-custom-host".to_string(), h.clone());
            }
        }
        Category5GatewayId::Cloudflare => {
            headers.insert("cf-aig-authorization".to_string(), format!("Bearer {}", api_key));
            if let Some(ttl) = cfg.cache_ttl_secs {
                headers.insert("cf-aig-cache-ttl".to_string(), ttl.to_string());
            }
            if cfg.skip_cache {
                headers.insert("cf-aig-skip-cache".to_string(), "true".to_string());
            }
        }
        Category5GatewayId::Helicone => {
            headers.insert("Helicone-Auth".to_string(), format!("Bearer {}", api_key));
            if !cfg.skip_cache {
                headers.insert("Helicone-Cache-Enabled".to_string(), "true".to_string());
            }
            if let Some((k, v)) = &cfg.trace_property {
                headers.insert(format!("Helicone-Property-{}", k), v.clone());
            }
        }
        _ => {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category5_12_gateways_and_router_headers() {
        assert_eq!(CATEGORY_5_GATEWAYS.len(), 12);
        let cfg = RouterHeaderConfig {
            upstream_provider: Some("anthropic".to_string()),
            config_id: Some("pc-1".to_string()),
            custom_host: None,
            cache_ttl_secs: Some(300),
            skip_cache: false,
            trace_property: Some(("env".to_string(), "prod".to_string())),
        };
        let mut h_portkey = HashMap::new();
        inject_category5_router_headers(Category5GatewayId::Portkey, "pk", &cfg, &mut h_portkey);
        assert_eq!(h_portkey.get("x-portkey-provider").map(String::as_str), Some("anthropic"));

        let mut h_cf = HashMap::new();
        inject_category5_router_headers(Category5GatewayId::Cloudflare, "cf", &cfg, &mut h_cf);
        assert_eq!(h_cf.get("cf-aig-cache-ttl").map(String::as_str), Some("300"));

        let mut h_hel = HashMap::new();
        inject_category5_router_headers(Category5GatewayId::Helicone, "hl", &cfg, &mut h_hel);
        assert_eq!(h_hel.get("Helicone-Property-env").map(String::as_str), Some("prod"));
    }
}
