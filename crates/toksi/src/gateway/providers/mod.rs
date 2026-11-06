// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-05

pub mod accelerator;
pub mod aggregators;
pub mod ecosystem;
pub mod frontier;
pub mod local;
pub mod multimodal;
pub mod specialized;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Classification of LLM & AI provider tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderCategory {
    Frontier,
    Accelerator,
    Aggregator,
    Specialized,
    Multimodal,
    Local,
    Ecosystem,
}

/// Enumeration of all 39+ in-tree AI providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    // Frontier (9)
    Anthropic,
    Azure,
    Bedrock,
    BedrockMantle,
    SageMaker,
    Databricks,
    Gemini,
    OpenAI,
    Vertex,

    // Accelerator (8)
    Cerebras,
    DeepSeek,
    Fireworks,
    Groq,
    Nebius,
    Parasail,
    Together,
    Wafer,

    // Aggregators (5)
    Bifrost,
    Cloudflare,
    HuggingFace,
    OpenRouter,
    Replicate,

    // Specialized (8)
    BytePlus,
    Cohere,
    Minimax,
    Mistral,
    Perplexity,
    Sarvam,
    TypeSafe,
    XAI,

    // Multimodal (3)
    ElevenLabs,
    Runware,
    Runway,

    // Local (4)
    LlamaCpp,
    Ollama,
    SGLang,
    VLLM,

    // Ecosystem (2)
    GitHubCopilot,
    OpenCode,
}

impl ProviderId {
    pub const ALL_39: [ProviderId; 39] = [
        ProviderId::Anthropic,
        ProviderId::Azure,
        ProviderId::Bedrock,
        ProviderId::BedrockMantle,
        ProviderId::SageMaker,
        ProviderId::Databricks,
        ProviderId::Gemini,
        ProviderId::OpenAI,
        ProviderId::Vertex,
        ProviderId::Cerebras,
        ProviderId::DeepSeek,
        ProviderId::Fireworks,
        ProviderId::Groq,
        ProviderId::Nebius,
        ProviderId::Parasail,
        ProviderId::Together,
        ProviderId::Wafer,
        ProviderId::Bifrost,
        ProviderId::Cloudflare,
        ProviderId::HuggingFace,
        ProviderId::OpenRouter,
        ProviderId::Replicate,
        ProviderId::BytePlus,
        ProviderId::Cohere,
        ProviderId::Minimax,
        ProviderId::Mistral,
        ProviderId::Perplexity,
        ProviderId::Sarvam,
        ProviderId::TypeSafe,
        ProviderId::XAI,
        ProviderId::ElevenLabs,
        ProviderId::Runware,
        ProviderId::Runway,
        ProviderId::LlamaCpp,
        ProviderId::Ollama,
        ProviderId::SGLang,
        ProviderId::VLLM,
        ProviderId::GitHubCopilot,
        ProviderId::OpenCode,
    ];

    pub const ALL_36: [ProviderId; 36] = [
        ProviderId::Anthropic,
        ProviderId::Azure,
        ProviderId::Bedrock,
        ProviderId::Databricks,
        ProviderId::Gemini,
        ProviderId::OpenAI,
        ProviderId::Vertex,
        ProviderId::Cerebras,
        ProviderId::DeepSeek,
        ProviderId::Fireworks,
        ProviderId::Groq,
        ProviderId::Nebius,
        ProviderId::Parasail,
        ProviderId::Together,
        ProviderId::Wafer,
        ProviderId::Cloudflare,
        ProviderId::HuggingFace,
        ProviderId::OpenRouter,
        ProviderId::Replicate,
        ProviderId::BytePlus,
        ProviderId::Cohere,
        ProviderId::Minimax,
        ProviderId::Mistral,
        ProviderId::Perplexity,
        ProviderId::Sarvam,
        ProviderId::TypeSafe,
        ProviderId::XAI,
        ProviderId::ElevenLabs,
        ProviderId::Runware,
        ProviderId::Runway,
        ProviderId::LlamaCpp,
        ProviderId::Ollama,
        ProviderId::SGLang,
        ProviderId::VLLM,
        ProviderId::GitHubCopilot,
        ProviderId::OpenCode,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Azure => "azure",
            Self::Bedrock => "bedrock",
            Self::BedrockMantle => "bedrockmantle",
            Self::SageMaker => "sagemaker",
            Self::Databricks => "databricks",
            Self::Gemini => "gemini",
            Self::OpenAI => "openai",
            Self::Vertex => "vertex",
            Self::Cerebras => "cerebras",
            Self::DeepSeek => "deepseek",
            Self::Fireworks => "fireworks",
            Self::Groq => "groq",
            Self::Nebius => "nebius",
            Self::Parasail => "parasail",
            Self::Together => "together",
            Self::Wafer => "wafer",
            Self::Bifrost => "bifrost",
            Self::Cloudflare => "cloudflare",
            Self::HuggingFace => "huggingface",
            Self::OpenRouter => "openrouter",
            Self::Replicate => "replicate",
            Self::BytePlus => "byteplus",
            Self::Cohere => "cohere",
            Self::Minimax => "minimax",
            Self::Mistral => "mistral",
            Self::Perplexity => "perplexity",
            Self::Sarvam => "sarvam",
            Self::TypeSafe => "typesafe",
            Self::XAI => "xai",
            Self::ElevenLabs => "elevenlabs",
            Self::Runware => "runware",
            Self::Runway => "runway",
            Self::LlamaCpp => "llamacpp",
            Self::Ollama => "ollama",
            Self::SGLang => "sglang",
            Self::VLLM => "vllm",
            Self::GitHubCopilot => "githubcopilot",
            Self::OpenCode => "opencode",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let n = name.to_lowercase();
        Self::ALL_39.into_iter().find(|p| p.as_str() == n)
    }

    pub fn category(&self) -> ProviderCategory {
        match self {
            Self::Anthropic | Self::Azure | Self::Bedrock | Self::BedrockMantle |
            Self::SageMaker | Self::Databricks | Self::Gemini | Self::OpenAI |
            Self::Vertex => ProviderCategory::Frontier,

            Self::Cerebras | Self::DeepSeek | Self::Fireworks | Self::Groq |
            Self::Nebius | Self::Parasail | Self::Together | Self::Wafer => ProviderCategory::Accelerator,

            Self::Bifrost | Self::Cloudflare | Self::HuggingFace | Self::OpenRouter |
            Self::Replicate => ProviderCategory::Aggregator,

            Self::BytePlus | Self::Cohere | Self::Minimax | Self::Mistral |
            Self::Perplexity | Self::Sarvam | Self::TypeSafe | Self::XAI => ProviderCategory::Specialized,

            Self::ElevenLabs | Self::Runware | Self::Runway => ProviderCategory::Multimodal,

            Self::LlamaCpp | Self::Ollama | Self::SGLang | Self::VLLM => ProviderCategory::Local,

            Self::GitHubCopilot | Self::OpenCode => ProviderCategory::Ecosystem,
        }
    }

    pub fn default_endpoint(&self) -> &'static str {
        match self {
            Self::Anthropic => "https://api.anthropic.com/v1/messages",
            Self::Azure => "https://{resource}.openai.azure.com",
            Self::Bedrock => "https://bedrock-runtime.{region}.amazonaws.com",
            Self::BedrockMantle => "https://bedrock-mantle.{region}.api.aws",
            Self::SageMaker => "https://runtime.sagemaker.{region}.amazonaws.com",
            Self::Databricks => "https://{workspace}.cloud.databricks.com/serving-endpoints",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::OpenAI => "https://api.openai.com/v1",
            Self::Vertex => "https://{region}-aiplatform.googleapis.com/v1",
            Self::Cerebras => "https://api.cerebras.ai/v1",
            Self::DeepSeek => "https://api.deepseek.com/v1",
            Self::Fireworks => "https://api.fireworks.ai/inference/v1",
            Self::Groq => "https://api.groq.com/openai/v1",
            Self::Nebius => "https://api.studio.nebius.ai/v1",
            Self::Parasail => "https://api.parasail.io/v1",
            Self::Together => "https://api.together.xyz/v1",
            Self::Wafer => "https://api.wafer.ai/v1",
            Self::Bifrost => "http://127.0.0.1:8080/v1",
            Self::Cloudflare => "https://api.cloudflare.com/client/v4/accounts/{account}/ai/run",
            Self::HuggingFace => "https://api-inference.huggingface.co/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::Replicate => "https://api.replicate.com/v1",
            Self::BytePlus => "https://ark.cn-beijing.volces.com/api/v3",
            Self::Cohere => "https://api.cohere.com/v2",
            Self::Minimax => "https://api.minimaxi.chat/v1",
            Self::Mistral => "https://api.mistral.ai/v1",
            Self::Perplexity => "https://api.perplexity.ai",
            Self::Sarvam => "https://api.sarvam.ai/v1",
            Self::TypeSafe => "https://api.typesafe.ai/v1",
            Self::XAI => "https://api.x.ai/v1",
            Self::ElevenLabs => "https://api.elevenlabs.io/v1",
            Self::Runware => "https://api.runware.ai/v1",
            Self::Runway => "https://api.runwayml.com/v1",
            Self::LlamaCpp => "http://127.0.0.1:8080/v1",
            Self::Ollama => "http://127.0.0.1:11434/v1",
            Self::SGLang => "http://127.0.0.1:30000/v1",
            Self::VLLM => "http://127.0.0.1:8000/v1",
            Self::GitHubCopilot => "https://api.githubcopilot.com",
            Self::OpenCode => "https://api.opencode.ai/v1",
        }
    }
}

/// Generic unified request across providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub model: String,
    pub messages: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Generic response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub role: String,
    pub finish_reason: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub raw: Value,
}

/// Provider client trait implemented across all providers
pub trait ProviderClient: Send + Sync {
    fn id(&self) -> ProviderId;
    fn endpoint(&self) -> String;
    fn auth_headers(&self, api_key: &str) -> HashMap<String, String>;
    fn format_request(&self, req: &UnifiedRequest) -> Value;
    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_39_provider_catalog_completeness() {
        assert_eq!(ProviderId::ALL_39.len(), 39);

        let mut unique_ids = std::collections::HashSet::new();
        let mut unique_names = std::collections::HashSet::new();

        for p in &ProviderId::ALL_39 {
            assert!(unique_ids.insert(*p), "Duplicate ProviderId: {:?}", p);
            assert!(unique_names.insert(p.as_str()), "Duplicate provider name: {}", p.as_str());

            // Every provider must be resolvable from its string representation
            assert_eq!(ProviderId::from_name(p.as_str()), Some(*p));

            // Default endpoint must be non-empty URL
            let ep = p.default_endpoint();
            assert!(ep.starts_with("http://") || ep.starts_with("https://"), "Invalid endpoint for {:?}", p);
        }
    }

    #[test]
    fn test_provider_category_distribution() {
        let mut frontier_count = 0;
        let mut accelerator_count = 0;
        let mut aggregator_count = 0;
        let mut specialized_count = 0;
        let mut multimodal_count = 0;
        let mut local_count = 0;
        let mut ecosystem_count = 0;

        for p in &ProviderId::ALL_39 {
            match p.category() {
                ProviderCategory::Frontier => frontier_count += 1,
                ProviderCategory::Accelerator => accelerator_count += 1,
                ProviderCategory::Aggregator => aggregator_count += 1,
                ProviderCategory::Specialized => specialized_count += 1,
                ProviderCategory::Multimodal => multimodal_count += 1,
                ProviderCategory::Local => local_count += 1,
                ProviderCategory::Ecosystem => ecosystem_count += 1,
            }
        }

        assert_eq!(frontier_count, 9);
        assert_eq!(accelerator_count, 8);
        assert_eq!(aggregator_count, 5);
        assert_eq!(specialized_count, 8);
        assert_eq!(multimodal_count, 3);
        assert_eq!(local_count, 4);
        assert_eq!(ecosystem_count, 2);
    }
}

