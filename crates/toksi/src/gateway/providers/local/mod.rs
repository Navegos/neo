// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/local/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

pub struct LocalProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl LocalProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for LocalProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.endpoint_url.trim_end_matches('/'))
    }

    fn auth_headers(&self, _api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
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
        let id = resp.get("id").and_then(Value::as_str).unwrap_or("local").to_string();
        let model = resp.get("model").and_then(Value::as_str).unwrap_or("local-model").to_string();
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

#[derive(Debug, Clone, Copy)]
pub struct LocalEngineSpec {
    pub provider_id: ProviderId,
    pub binary_name: &'static str,
    pub health_path: &'static str,
}

/// Catalog of all 23 local inference engines with CLI binary and health/model probe path.
pub const LOCAL_ENGINE_SPECS: [LocalEngineSpec; 23] = [
    LocalEngineSpec { provider_id: ProviderId::Ollama, binary_name: "ollama", health_path: "/api/tags" },
    LocalEngineSpec { provider_id: ProviderId::LlamaCpp, binary_name: "llama-server", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::LMStudio, binary_name: "lms", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::JanAI, binary_name: "jan", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::GPT4All, binary_name: "gpt4all", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::MstyAI, binary_name: "msty", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::KoboldCpp, binary_name: "koboldcpp", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::Llamafile, binary_name: "llamafile", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::LocalAI, binary_name: "local-ai", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::VLLM, binary_name: "vllm", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::SGLang, binary_name: "sglang", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::TGI, binary_name: "text-generation-launcher", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::LMDeploy, binary_name: "lmdeploy", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::TabbyAPI, binary_name: "tabbyapi", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::MlcLlm, binary_name: "mlc_llm", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::Exo, binary_name: "exo", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::FastChat, binary_name: "fastchat", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::Triton, binary_name: "tritonserver", health_path: "/v2/health/ready" },
    LocalEngineSpec { provider_id: ProviderId::TextGenerationWebUI, binary_name: "oobabooga", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::OpenWebUI, binary_name: "open-webui", health_path: "/api/config" },
    LocalEngineSpec { provider_id: ProviderId::AnythingLLM, binary_name: "anythingllm", health_path: "/api/v1/system" },
    LocalEngineSpec { provider_id: ProviderId::MlxLm, binary_name: "mlx_lm.server", health_path: "/v1/models" },
    LocalEngineSpec { provider_id: ProviderId::Aphrodite, binary_name: "aphrodite", health_path: "/v1/models" },
];

#[derive(Debug, Clone)]
pub struct LocalEngineDiscovery {
    pub provider_id: ProviderId,
    pub binary_name: &'static str,
    pub in_path: bool,
    pub endpoint: String,
    pub endpoint_reachable: bool,
}

/// Discovers all 23 local inference engines via system `PATH` without probing sockets.
pub fn discover_local_engines() -> Vec<LocalEngineDiscovery> {
    discover_local_engines_with_probe(false)
}

/// Discovers all 23 local inference engines via `PATH` and optional localhost HTTP endpoint probe.
pub fn discover_local_engines_with_probe(probe_http: bool) -> Vec<LocalEngineDiscovery> {
    let client = if probe_http {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(200))
            .build()
            .ok()
    } else {
        None
    };

    LOCAL_ENGINE_SPECS
        .iter()
        .map(|spec| {
            let endpoint = spec.provider_id.effective_endpoint();
            let in_path = crate::util::env_path::is_executable_in_path(spec.binary_name);
            let endpoint_reachable = if let Some(ref c) = client {
                let base = endpoint.trim_end_matches("/v1").trim_end_matches('/');
                let probe_url = format!("{}{}", base, spec.health_path);
                c.get(&probe_url).send().map(|r| r.status().is_success()).unwrap_or(false)
            } else {
                false
            };
            LocalEngineDiscovery {
                provider_id: spec.provider_id,
                binary_name: spec.binary_name,
                in_path,
                endpoint,
                endpoint_reachable,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_23_local_engines_discovery_catalog() {
        let found = discover_local_engines();
        assert_eq!(found.len(), 23);
        assert!(found.iter().any(|e| e.provider_id == ProviderId::Ollama && e.endpoint.contains("11434")));
        assert!(found.iter().any(|e| e.provider_id == ProviderId::Exo && e.endpoint.contains("52415")));
    }
}

