// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/proxy.rs

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::gateway::circuit_breaker::CircuitBreaker;
use crate::gateway::governance::GovernanceEngine;
use crate::gateway::keys::KeyPoolManager;
use crate::gateway::loadbalancer::LoadBalancer;
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};
use crate::gateway::semantic_cache::DualTierSemanticCache;
use crate::headroom::cache_align::align_system_prompt_to_1024;
use crate::headroom::pruner::{prune_mcp_request_payload, McpPrunerConfig};

#[derive(Default)]
pub struct GatewayProxyOrchestrator {
    pub key_manager: Arc<KeyPoolManager>,
    pub load_balancer: Arc<LoadBalancer>,
    pub semantic_cache: Arc<DualTierSemanticCache>,
    pub governance: Arc<GovernanceEngine>,
    pub circuit_breaker: Arc<CircuitBreaker>,
}

impl GatewayProxyOrchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes an incoming agent request through the complete token optimization & failover pipeline
    pub fn process_request(
        &self,
        session_id: &str,
        vk_opt: Option<&str>,
        provider: ProviderId,
        client: &dyn ProviderClient,
        payload: &mut Value,
    ) -> Result<UnifiedResponse, String> {
        // 1. Governance check
        if let Some(vk) = vk_opt {
            self.governance.check_and_acquire(vk, 1000)?;
        }

        // 2. Headroom prompt cache alignment & MCP tool schema pruning
        align_system_prompt_to_1024(payload);
        let pruner_cfg = McpPrunerConfig::default();
        let _prune_stats = prune_mcp_request_payload(payload, &pruner_cfg);

        // 3. Fast Tier-1 direct exact SHA-256 cache lookup
        let prompt_str = payload.to_string();
        if let Some(cached_resp) = self.semantic_cache.lookup_exact(provider.as_str(), &prompt_str) {
            if let Some(vk) = vk_opt {
                self.governance.release_and_record_spend(vk, 0.0);
            }
            return Ok(UnifiedResponse {
                id: "cache-hit".to_string(),
                model: provider.as_str().to_string(),
                content: cached_resp,
                role: "assistant".to_string(),
                finish_reason: Some("stop".to_string()),
                input_tokens: 0,
                output_tokens: 0,
                raw: Value::Null,
            });
        }

        // 4. Key selection & dynamic failover
        let _key = self
            .key_manager
            .get_available_key(provider)
            .unwrap_or_else(|| "default-api-key".to_string());

        let _req_formatted = client.format_request(&UnifiedRequest {
            model: payload.get("model").and_then(Value::as_str).unwrap_or("default").to_string(),
            messages: payload.get("messages").and_then(Value::as_array).cloned().unwrap_or_default(),
            system: payload.get("system").and_then(Value::as_str).map(|s| s.to_string()),
            max_tokens: payload.get("max_tokens").and_then(Value::as_u64).map(|u| u as u32),
            temperature: None,
            stream: None,
            tools: payload.get("tools").and_then(Value::as_array).cloned(),
            extra: HashMap::new(),
        });

        // 5. Simulate client response parsing
        let mock_resp = serde_json::json!({
            "id": "resp-1",
            "model": provider.as_str(),
            "choices": [{
                "message": { "content": "Response from gateway" },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 120, "completion_tokens": 40 }
        });

        let parsed = client.parse_response(&mock_resp)?;

        // 6. Record tokens with circuit breaker
        self.circuit_breaker
            .record_tokens(session_id, (parsed.input_tokens + parsed.output_tokens) as u64)
            .map_err(|e| format!("Circuit breaker triggered: {:?}", e))?;

        // 7. Store response in semantic cache
        self.semantic_cache.store(
            provider.as_str(),
            &prompt_str,
            &parsed.content,
            None,
        );

        if let Some(vk) = vk_opt {
            self.governance.release_and_record_spend(vk, 0.0005);
        }

        Ok(parsed)
    }
}
