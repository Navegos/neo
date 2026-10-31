// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase7_gateway_verification.rs

use std::collections::HashMap;
use std::time::Duration;
use serde_json::json;
use tokless::gateway::agent_cli::detect_local_agent_session;
use tokless::gateway::cascade::{classify_failure, parse_retry_after, FailureClass};
use tokless::gateway::circuit_breaker::{CircuitBreaker, CircuitBreakerReason};
use tokless::gateway::compat::{adapt_sdk_payload, detect_sdk_protocol, SdkProtocol};
use tokless::gateway::governance::{GovernanceEngine, VirtualKeyConfig};
use tokless::gateway::keys::{KeyPoolManager, ProviderApiKey};
use tokless::gateway::loadbalancer::{LoadBalanceStrategy, LoadBalancer};
use tokless::gateway::normalizer::{anthropic_to_openai, gemini_to_unified, openai_to_anthropic};
use tokless::gateway::providers::frontier::{AnthropicProvider, GeminiProvider, OpenAIProvider};
use tokless::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest};
use tokless::gateway::proxy::GatewayProxyOrchestrator;
use tokless::gateway::routing::{analyze_complexity, TaskComplexity};
use tokless::gateway::semantic_cache::{cosine_similarity, DualTierSemanticCache};
use tokless::headroom::pruner::{
    estimate_json_tokens, extract_tool_name, optimize_json_schema, prune_mcp_request_payload,
    prune_mcp_tools, McpPrunerConfig,
};

#[test]
fn test_phase7_36_providers_catalog_and_categories() {
    assert_eq!(ProviderId::ALL_36.len(), 36);

    for pid in ProviderId::ALL_36 {
        let name = pid.as_str();
        assert!(!name.is_empty());
        assert_eq!(ProviderId::from_name(name), Some(pid));

        let endpoint = pid.default_endpoint();
        assert!(!endpoint.is_empty(), "Provider {} must have default endpoint", name);

        let cat = pid.category();
        let _ = cat; // Category is resolvable
    }
}

#[test]
fn test_phase7_frontier_clients_format_and_auth() {
    // Anthropic
    let anth = AnthropicProvider::default();
    let anth_headers = anth.auth_headers("sk-ant-test-key");
    assert_eq!(anth_headers.get("x-api-key"), Some(&"sk-ant-test-key".to_string()));
    assert!(anth_headers.contains_key("anthropic-version"));

    let req = UnifiedRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![json!({"role": "user", "content": "hello"})],
        system: Some("Act as coding bot".to_string()),
        max_tokens: Some(2048),
        temperature: Some(0.2),
        stream: Some(false),
        tools: None,
        extra: HashMap::new(),
    };
    let formatted_anth = anth.format_request(&req);
    assert_eq!(formatted_anth["model"], "claude-3-5-sonnet");
    assert_eq!(formatted_anth["system"], "Act as coding bot");

    // OpenAI
    let oai = OpenAIProvider::default();
    let oai_headers = oai.auth_headers("sk-proj-test-key");
    assert_eq!(oai_headers.get("Authorization"), Some(&"Bearer sk-proj-test-key".to_string()));

    let formatted_oai = oai.format_request(&req);
    assert_eq!(formatted_oai["model"], "claude-3-5-sonnet");
    let msgs = formatted_oai["messages"].as_array().unwrap();
    assert_eq!(msgs[0]["role"], "system");

    // Gemini
    let gem = GeminiProvider::default();
    let gem_headers = gem.auth_headers("AIzaSy-test-key");
    assert_eq!(gem_headers.get("x-goog-api-key"), Some(&"AIzaSy-test-key".to_string()));
}

#[test]
fn test_phase7_dynamic_mcp_tool_schema_pruning() {
    let mut tools = vec![
        json!({
            "name": "read_file",
            "description": "A very detailed description of read_file with lots of text that takes many tokens..................................................................................................................................................................................................................................................................................................................................................................................................................................",
            "input_schema": {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "title": "path",
                        "description": "Path to file"
                    }
                },
                "required": []
            }
        }),
        json!({
            "name": "execute_command",
            "description": "Runs shell commands in workspace",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "title": "command"
                    }
                }
            }
        }),
        json!({
            "name": "unused_admin_tool",
            "description": "Should be filtered when whitelist supplied",
            "input_schema": { "type": "object" }
        })
    ];

    let mut cfg = McpPrunerConfig::default();
    cfg.allowed_tools = Some(vec!["read_file".to_string(), "execute_command".to_string()]);
    cfg.max_description_len = 100;

    let stats = prune_mcp_tools(&mut tools, &cfg);
    assert_eq!(stats.tools_before, 3);
    assert_eq!(stats.tools_after, 2);
    assert!(stats.tokens_saved > 0);

    // Verify schema cleanup
    let read_tool = &tools[0];
    assert_eq!(read_tool["name"], "read_file");
    let desc = read_tool["description"].as_str().unwrap();
    assert!(desc.ends_with("..."));
    assert!(desc.len() <= 104);

    let schema = &read_tool["input_schema"];
    assert!(schema.get("$schema").is_none(), "$schema URL must be pruned");
    let prop_path = &schema["properties"]["path"];
    assert!(prop_path.get("title").is_none(), "Redundant title matching prop name must be pruned");

    // Full request payload pruning
    let mut payload = json!({
        "model": "claude-3-5-sonnet",
        "tools": tools,
    });
    let res = prune_mcp_request_payload(&mut payload, &cfg);
    assert!(res.is_some());
}

#[test]
fn test_phase7_bifrost_failure_classification_and_backoff() {
    let empty_headers = HashMap::new();

    // 401 -> Credential (permanent key failure)
    let f_cred = classify_failure(401, &empty_headers, "invalid_api_key");
    assert_eq!(f_cred, FailureClass::Credential);
    assert!(f_cred.is_permanent_key_failure());

    // 429 Quota -> Quota
    let f_quota = classify_failure(429, &empty_headers, "insufficient_quota on your account");
    assert_eq!(f_quota, FailureClass::Quota);
    assert!(f_quota.is_permanent_key_failure());

    // 429 RateLimit -> RateLimit
    let f_rate = classify_failure(429, &empty_headers, "rate limit exceeded for RPM");
    assert_eq!(f_rate, FailureClass::RateLimit);
    assert!(f_rate.is_retryable());

    // 529 Anthropic Overloaded -> Transient
    let f_trans = classify_failure(529, &empty_headers, "overloaded_error");
    assert_eq!(f_trans, FailureClass::Transient);
    assert!(f_trans.is_retryable());

    // 404 Model Not Found -> ModelGone
    let f_gone = classify_failure(404, &empty_headers, "model_not_found");
    assert_eq!(f_gone, FailureClass::ModelGone);

    // Retry-After header parsing
    let mut headers = HashMap::new();
    headers.insert("retry-after".to_string(), "5".to_string());
    assert_eq!(parse_retry_after(&headers), Duration::from_secs(5));
}

#[test]
fn test_phase7_key_pool_manager_health_and_cooldown() {
    let mgr = KeyPoolManager::new();
    let mut k1 = ProviderApiKey::new("key-alpha");
    let k2 = ProviderApiKey::new("key-beta");

    mgr.add_key(ProviderId::Anthropic, k1);
    mgr.add_key(ProviderId::Anthropic, k2);

    // Round-robin selection
    let picked1 = mgr.get_available_key(ProviderId::Anthropic).unwrap();
    let picked2 = mgr.get_available_key(ProviderId::Anthropic).unwrap();
    assert_ne!(picked1, picked2);

    // Permanent failure on key-alpha excludes it
    mgr.record_result(ProviderId::Anthropic, &picked1, Some(FailureClass::Credential), None);

    let next_pick1 = mgr.get_available_key(ProviderId::Anthropic).unwrap();
    let next_pick2 = mgr.get_available_key(ProviderId::Anthropic).unwrap();
    assert_eq!(next_pick1, picked2);
    assert_eq!(next_pick2, picked2);
}

#[test]
fn test_phase7_load_balancer_strategies() {
    let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
    lb.register_provider(ProviderId::Groq, 0.05, 1);
    lb.register_provider(ProviderId::DeepSeek, 0.14, 1);
    lb.register_provider(ProviderId::OpenAI, 2.50, 1);

    let candidates = [ProviderId::Groq, ProviderId::DeepSeek, ProviderId::OpenAI];

    // Round robin
    let p1 = lb.select_provider(&candidates).unwrap();
    let p2 = lb.select_provider(&candidates).unwrap();
    let p3 = lb.select_provider(&candidates).unwrap();
    assert_eq!(p1, ProviderId::Groq);
    assert_eq!(p2, ProviderId::DeepSeek);
    assert_eq!(p3, ProviderId::OpenAI);

    // Lowest cost strategy
    lb.set_strategy(LoadBalanceStrategy::LowestCost);
    let cheap = lb.select_provider(&candidates).unwrap();
    assert_eq!(cheap, ProviderId::Groq);

    // Least in-flight strategy
    lb.set_strategy(LoadBalanceStrategy::LeastInFlight);
    lb.acquire_flight(ProviderId::Groq);
    lb.acquire_flight(ProviderId::Groq);
    let least_busy = lb.select_provider(&candidates).unwrap();
    assert_ne!(least_busy, ProviderId::Groq);
    lb.release_flight(ProviderId::Groq, 45.0);
}

#[test]
fn test_phase7_task_complexity_router() {
    let simple_msgs = vec![json!({"role": "user", "content": "What is 2 + 2?"})];
    let simple_rec = analyze_complexity(&simple_msgs, None);
    assert_eq!(simple_rec.complexity, TaskComplexity::Simple);

    let complex_content = "Please architect, refactor, and verify a complete distributed transaction manager step-by-step with code: ```rust\nfn commit() {}\n```";
    let complex_msgs = vec![json!({"role": "user", "content": complex_content})];
    let tools = vec![json!({"name": "run_command"}), json!({"name": "edit_file"})];
    let complex_rec = analyze_complexity(&complex_msgs, Some(&tools));
    assert_eq!(complex_rec.complexity, TaskComplexity::Complex);
    assert_eq!(complex_rec.reasoning_effort, Some("high"));
}

#[test]
fn test_phase7_virtual_key_governance_and_budgets() {
    let gov = GovernanceEngine::new();
    let vk_cfg = VirtualKeyConfig {
        vk: "vk-agent-alpha".to_string(),
        name: "Agent Alpha".to_string(),
        max_budget_usd: 1.00,
        max_rpm: 2,
        max_tpm: 10000,
        max_concurrent_requests: 1,
        allowed_models: None,
    };
    gov.register_vk(vk_cfg);

    // 1st request succeeds
    assert!(gov.check_and_acquire("vk-agent-alpha", 500).is_ok());

    // 2nd request fails due to concurrent request limit (1)
    let err_concur = gov.check_and_acquire("vk-agent-alpha", 500);
    assert!(err_concur.is_err());
    assert!(err_concur.unwrap_err().contains("Max concurrent requests"));

    // Release and add budget spend
    gov.release_and_record_spend("vk-agent-alpha", 1.50);

    // Now exceeds max_budget_usd (1.50 >= 1.00)
    let err_budget = gov.check_and_acquire("vk-agent-alpha", 100);
    assert!(err_budget.is_err());
    assert!(err_budget.unwrap_err().contains("USD budget exceeded"));
}

#[test]
fn test_phase7_dual_tier_semantic_cache() {
    let cache = DualTierSemanticCache::new(0.90);
    let prompt = "Explain TCP three-way handshake";
    let resp = "SYN, SYN-ACK, ACK establishes reliable bidirectional stream.";

    // Initially cache miss
    assert!(cache.lookup_exact("claude-3-5-sonnet", prompt).is_none());

    // Store exact prompt
    let emb = vec![0.1, 0.2, 0.3, 0.4];
    cache.store("claude-3-5-sonnet", prompt, resp, Some(emb.clone()));

    // Exact lookup hit
    let exact_hit = cache.lookup_exact("claude-3-5-sonnet", prompt);
    assert_eq!(exact_hit, Some(resp.to_string()));

    // Semantic cosine lookup hit
    let close_emb = vec![0.11, 0.20, 0.31, 0.39]; // High cosine similarity
    let sem_hit = cache.lookup_semantic(&close_emb, "claude-3-5-sonnet");
    assert!(sem_hit.is_some());
    let (match_content, sim) = sem_hit.unwrap();
    assert_eq!(match_content, resp);
    assert!(sim >= 0.90);

    // Cosine similarity check
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]), 1.0);
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
}

#[test]
fn test_phase7_bidirectional_protocol_normalizer() {
    // OpenAI -> Anthropic
    let oai_req = json!({
        "model": "gpt-4o",
        "messages": [
            {"role": "system", "content": "You are a Rust expert."},
            {"role": "user", "content": "Explain zero-cost abstractions."}
        ],
        "max_tokens": 1024
    });
    let anth_norm = openai_to_anthropic(&oai_req);
    assert_eq!(anth_norm["system"], "You are a Rust expert.");
    assert_eq!(anth_norm["messages"][0]["role"], "user");

    // Anthropic -> OpenAI
    let anth_req = json!({
        "model": "claude-3-5-sonnet",
        "system": "System instructions here",
        "messages": [
            {"role": "user", "content": "Hello world"}
        ]
    });
    let oai_norm = anthropic_to_openai(&anth_req);
    let msgs = oai_norm["messages"].as_array().unwrap();
    assert_eq!(msgs[0]["role"], "system");
    assert_eq!(msgs[1]["role"], "user");

    // Gemini -> Unified
    let gem_req = json!({
        "contents": [
            {"role": "user", "parts": [{"text": "Query text"}]}
        ]
    });
    let uni = gemini_to_unified(&gem_req);
    assert_eq!(uni.messages[0]["content"], "Query text");

    // SDK protocol detector & adapter
    assert_eq!(detect_sdk_protocol("/v1/chat/completions"), Some(SdkProtocol::OpenAI));
    assert_eq!(detect_sdk_protocol("/v1/messages"), Some(SdkProtocol::Anthropic));
    assert_eq!(detect_sdk_protocol("/v1beta/models/gemini-2.0-flash"), Some(SdkProtocol::Gemini));

    let adapted = adapt_sdk_payload(SdkProtocol::OpenAI, SdkProtocol::Anthropic, &oai_req);
    assert!(adapted.get("system").is_some());
}

#[test]
fn test_phase7_circuit_breaker_loop_detection() {
    let cb = CircuitBreaker::new(3, 3, 1000);

    // 1. Repeating identical tool call detection
    let tool_args = json!({"path": "src/main.rs"});
    assert!(cb.inspect_tool_call("session-1", "read_file", &tool_args).is_ok());
    assert!(cb.inspect_tool_call("session-1", "read_file", &tool_args).is_ok());
    let err_loop = cb.inspect_tool_call("session-1", "read_file", &tool_args);
    assert!(err_loop.is_err());
    assert!(matches!(err_loop.unwrap_err(), CircuitBreakerReason::RepeatingToolCallLoop { .. }));

    // 2. Repeated identical error detection
    assert!(cb.inspect_error("session-2", "EPERM: Permission denied").is_ok());
    assert!(cb.inspect_error("session-2", "EPERM: Permission denied").is_ok());
    let err_repeat = cb.inspect_error("session-2", "EPERM: Permission denied");
    assert!(err_repeat.is_err());
    assert!(matches!(err_repeat.unwrap_err(), CircuitBreakerReason::RepeatedErrorLoop { .. }));

    // 3. Runaway token ceiling
    assert!(cb.record_tokens("session-3", 600).is_ok());
    let err_token = cb.record_tokens("session-3", 500);
    assert!(err_token.is_err());
    assert!(matches!(err_token.unwrap_err(), CircuitBreakerReason::RunawayTokenLimitExceeded { .. }));
}

#[test]
fn test_phase7_gateway_proxy_orchestrator_end_to_end() {
    let orchestrator = GatewayProxyOrchestrator::new();
    let anth = AnthropicProvider::default();

    let mut payload = json!({
        "model": "claude-3-5-sonnet",
        "system": "System instructions",
        "messages": [{"role": "user", "content": "Echo this back"}],
        "tools": [
            {
                "name": "calc",
                "description": "Performs arithmetic calculations",
                "input_schema": {"type": "object", "properties": {"expr": {"type": "string"}}}
            }
        ]
    });

    let resp = orchestrator.process_request("test-session", None, ProviderId::Anthropic, &anth, &mut payload);
    assert!(resp.is_ok());
    let res_obj = resp.unwrap();
    assert_eq!(res_obj.content, "Response from gateway");

    // Second request with identical payload hits Tier-1 exact semantic cache
    let resp2 = orchestrator.process_request("test-session", None, ProviderId::Anthropic, &anth, &mut payload);
    assert!(resp2.is_ok());
    assert_eq!(resp2.unwrap().id, "cache-hit");
}

