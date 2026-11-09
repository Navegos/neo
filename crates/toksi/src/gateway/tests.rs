// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/tests.rs
// created: 2026-11-06

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_json::json;

    use crate::gateway::cascade::{classify_failure, parse_retry_after, FailureClass};
    use crate::gateway::compat::{adapt_sdk_payload, detect_sdk_protocol, SdkProtocol};
    use crate::gateway::governance::{GovernanceEngine, VirtualKeyConfig, TOKSI_VK_HEADER};
    use crate::gateway::loadbalancer::{LoadBalanceStrategy, LoadBalancer};
    use crate::gateway::providers::{ProviderCategory, ProviderId};
    use crate::gateway::routing::{analyze_complexity, TaskComplexity};
    use crate::gateway::semantic_cache::DualTierSemanticCache;

    #[test]
    fn test_phase_07_provider_catalog_full() {
        assert_eq!(ProviderId::ALL_39.len(), 39);
        assert_eq!(ProviderId::from_name("anthropic"), Some(ProviderId::Anthropic));
        assert_eq!(ProviderId::from_name("bifrost"), Some(ProviderId::Bifrost));
        assert_eq!(ProviderId::from_name("sagemaker"), Some(ProviderId::SageMaker));
        assert_eq!(ProviderId::from_name("bedrockmantle"), Some(ProviderId::BedrockMantle));

        assert_eq!(ProviderId::Anthropic.category(), ProviderCategory::Frontier);
        assert_eq!(ProviderId::Cerebras.category(), ProviderCategory::Accelerator);
        assert_eq!(ProviderId::Bifrost.category(), ProviderCategory::Aggregator);
        assert_eq!(ProviderId::XAI.category(), ProviderCategory::Specialized);
        assert_eq!(ProviderId::ElevenLabs.category(), ProviderCategory::Multimodal);
        assert_eq!(ProviderId::Ollama.category(), ProviderCategory::Local);
        assert_eq!(ProviderId::GitHubCopilot.category(), ProviderCategory::Ecosystem);
    }

    #[test]
    fn test_phase_07_cascade_dynamic_failover() {
        let mut headers = HashMap::new();
        headers.insert("retry-after".to_string(), "3".to_string());
        assert_eq!(parse_retry_after(&headers), std::time::Duration::from_secs(3));

        // 8 failure categories
        assert_eq!(classify_failure(401, &headers, "invalid key"), FailureClass::Credential);
        assert_eq!(classify_failure(402, &headers, "out of credits"), FailureClass::Quota);
        assert_eq!(classify_failure(429, &headers, "rate limit exceeded"), FailureClass::RateLimit);
        assert_eq!(classify_failure(403, &headers, "model forbidden"), FailureClass::ModelAccess);
        assert_eq!(classify_failure(404, &headers, "model_not_found"), FailureClass::ModelGone);
        assert_eq!(classify_failure(403, &headers, "country not supported"), FailureClass::RegionBlocked);
        assert_eq!(classify_failure(503, &headers, "backend overloaded"), FailureClass::Transient);
        assert_eq!(classify_failure(408, &headers, "timeout"), FailureClass::Timeout);
    }

    #[test]
    fn test_phase_07_loadbalancer_selectors() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        lb.register_provider(ProviderId::OpenAI, 10.0, 5);
        lb.register_provider(ProviderId::Anthropic, 15.0, 5);

        let candidates = [ProviderId::OpenAI, ProviderId::Anthropic];
        let p1 = lb.select_provider(&candidates).unwrap();
        let p2 = lb.select_provider(&candidates).unwrap();
        assert_ne!(p1, p2);

        // Switch to lowest cost
        lb.set_strategy(LoadBalanceStrategy::LowestCost);
        let cheapest = lb.select_provider(&candidates).unwrap();
        assert_eq!(cheapest, ProviderId::OpenAI);
    }

    #[test]
    fn test_phase_07_task_complexity_router() {
        let simple_msgs = vec![json!({"role": "user", "content": "ping"})];
        assert_eq!(analyze_complexity(&simple_msgs, None).complexity, TaskComplexity::Simple);

        let complex_msgs = vec![json!({
            "role": "user",
            "content": "Step-by-step architect and refactor design to debug and verify raft consensus:\n```rust\nstruct Raft;\n```"
        })];
        let tools = vec![json!({"name": "bash"}), json!({"name": "read_file"})];
        assert_eq!(analyze_complexity(&complex_msgs, Some(&tools)).complexity, TaskComplexity::Complex);
    }

    #[test]
    fn test_phase_07_virtual_key_governance() {
        assert_eq!(TOKSI_VK_HEADER, "x-toksi-vk");
        let gov = GovernanceEngine::new();
        gov.register_vk(VirtualKeyConfig {
            vk: "vk-enterprise".to_string(),
            name: "Enterprise Client".to_string(),
            max_budget_usd: 5.0,
            max_rpm: 5,
            max_tpm: 2000,
            max_concurrent_requests: 1,
            allowed_models: None,
        });

        assert!(gov.check_and_acquire("vk-enterprise", 100).is_ok());
        // Concurrent limit blocks
        assert!(gov.check_and_acquire("vk-enterprise", 100).is_err());
        gov.release_and_record_spend("vk-enterprise", 5.50);

        // Budget exceeded blocks
        assert!(gov.check_and_acquire("vk-enterprise", 50).is_err());
    }

    #[test]
    fn test_phase_07_dual_tier_semantic_cache() {
        let cache = DualTierSemanticCache::new(0.92);
        let model = "gpt-4o";
        let prompt = "fn add(a: i32, b: i32) -> i32";
        let resp = "a + b";

        cache.store(model, prompt, resp, Some(vec![1.0, 0.0]));
        assert_eq!(cache.lookup_exact(model, prompt).as_deref(), Some(resp));

        let query_sim = vec![0.99, 0.05];
        let sem_hit = cache.lookup_semantic(&query_sim, model);
        assert!(sem_hit.is_some());
    }

    #[test]
    fn test_phase_07_drop_in_sdk_compat() {
        assert_eq!(detect_sdk_protocol("/v1/chat/completions"), Some(SdkProtocol::OpenAI));
        assert_eq!(detect_sdk_protocol("/v1/messages"), Some(SdkProtocol::Anthropic));
        assert_eq!(detect_sdk_protocol("/v1beta/models/gemini-2.0-flash"), Some(SdkProtocol::Gemini));

        let openai_body = json!({
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "hi"}]
        });
        let anthropic_body = adapt_sdk_payload(SdkProtocol::OpenAI, SdkProtocol::Anthropic, &openai_body);
        assert!(anthropic_body.get("messages").is_some());
    }

    #[tokio::test]
    async fn test_phase_15_2_gateway_server_endpoints() {
        let server = crate::gateway::server::GatewayServer::new("127.0.0.1", 19099);
        let task = tokio::spawn(async move {
            let _ = server.run().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let Ok(client) = reqwest::Client::builder().build() else {
            task.abort();
            return;
        };

        // 1. /healthz
        if let Ok(resp) = client.get("http://127.0.0.1:19099/healthz").send().await {
            assert!(resp.status().is_success());
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(body.get("service").and_then(|v| v.as_str()), Some("ai-gateway"));
        }

        // 2. /metrics
        if let Ok(resp) = client.get("http://127.0.0.1:19099/metrics").send().await {
            assert!(resp.status().is_success());
        }

        // 3. /v1/chat/completions
        let payload = json!({
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "hello"}]
        });
        if let Ok(resp) = client.post("http://127.0.0.1:19099/v1/chat/completions")
            .json(&payload)
            .send().await
        {
            assert!(resp.status().is_success());
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            assert_eq!(body.get("object").and_then(|v| v.as_str()), Some("chat.completion"));
        }

        task.abort();
    }
}
