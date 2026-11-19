// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/normalizer/litellm.rs
// created: 2026-11-09
// lastModified: 2026-11-16

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LiteLLMError {
    #[error("Unknown model alias: '{0}'")]
    UnknownModel(String),
    #[error("Unsupported provider conversion: '{from}' to '{to}'")]
    UnsupportedConversion { from: String, to: String },
    #[error("Payload serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Model pricing details per 1 million tokens (USD).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_cost_per_million: f64,
    pub output_cost_per_million: f64,
    pub cache_read_input_cost_per_million: f64,
}

/// Canonical gateway endpoint surfaces mapped by LiteLLM normalizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EndpointSurface {
    ChatCompletions,
    Messages,
    Responses,
    Embeddings,
}

impl EndpointSurface {
    pub fn canonical_path(&self) -> &'static str {
        match self {
            Self::ChatCompletions => "/v1/chat/completions",
            Self::Messages => "/v1/messages",
            Self::Responses => "/v1/responses",
            Self::Embeddings => "/v1/embeddings",
        }
    }
}

/// Maps a provider identifier and canonical endpoint surface to the upstream path.
pub fn map_endpoint_for_provider(provider: &str, surface: EndpointSurface) -> &'static str {
    match (provider.trim().to_ascii_lowercase().as_str(), surface) {
        ("anthropic", EndpointSurface::ChatCompletions) => "/v1/messages",
        ("anthropic", EndpointSurface::Messages) => "/v1/messages",
        ("gemini", EndpointSurface::ChatCompletions) => "/v1beta/openai/chat/completions",
        ("gemini", EndpointSurface::Embeddings) => "/v1beta/openai/embeddings",
        ("ollama", EndpointSurface::ChatCompletions) => "/v1/chat/completions",
        ("ollama", EndpointSurface::Embeddings) => "/api/embeddings",
        (_, s) => s.canonical_path(),
    }
}

/// Metadata description for an LLM catalog model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub canonical_name: String,
    pub provider: String,
    pub context_window: u64,
    pub max_output_tokens: u64,
    pub pricing: ModelPricing,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
}

/// Global registry of 100+ model naming aliases and pricing tables modeled after LiteLLM.
pub struct LiteLLMMatrix {
    aliases: HashMap<String, String>,
    models: HashMap<String, ModelInfo>,
}

impl Default for LiteLLMMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl LiteLLMMatrix {
    pub fn new() -> Self {
        let mut matrix = Self {
            aliases: HashMap::new(),
            models: HashMap::new(),
        };
        matrix.populate_catalog();
        matrix
    }

    /// Registers a canonical model definition.
    pub fn register_model(&mut self, info: ModelInfo) {
        self.aliases.insert(info.canonical_name.to_ascii_lowercase(), info.canonical_name.clone());
        self.models.insert(info.canonical_name.clone(), info);
    }

    /// Registers an alias pointing to a canonical model name.
    pub fn register_alias(&mut self, alias: &str, canonical: &str) {
        self.aliases.insert(alias.to_ascii_lowercase(), canonical.to_string());
    }

    /// Resolves any model alias to its canonical name and metadata.
    pub fn resolve_model(&self, name_or_alias: &str) -> Option<&ModelInfo> {
        let clean = name_or_alias.trim().to_ascii_lowercase();
        let canonical = self.aliases.get(&clean)?;
        self.models.get(canonical)
    }

    /// Returns the provider for a given model or alias.
    pub fn get_provider(&self, name_or_alias: &str) -> Option<&str> {
        self.resolve_model(name_or_alias).map(|m| m.provider.as_str())
    }

    /// Populates 100+ standard model aliases and current market rates.
    fn populate_catalog(&mut self) {
        // --- OpenAI Frontier & Legacy ---
        let openai_models = [
            ("gpt-4o", 128000, 16384, 2.50, 10.00, 1.25),
            ("gpt-4o-mini", 128000, 16384, 0.15, 0.60, 0.075),
            ("gpt-4-turbo", 128000, 4096, 10.00, 30.00, 5.00),
            ("gpt-4", 8192, 4096, 30.00, 60.00, 30.00),
            ("gpt-3.5-turbo", 16385, 4096, 0.50, 1.50, 0.50),
            ("o1", 200000, 100000, 15.00, 60.00, 7.50),
            ("o1-mini", 128000, 65536, 1.10, 4.40, 0.55),
            ("o3-mini", 200000, 100000, 1.10, 4.40, 0.55),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in openai_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "openai".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: true,
                supports_function_calling: true,
            });
        }
        // OpenAI Aliases
        self.register_alias("chatgpt-4o", "gpt-4o");
        self.register_alias("openai/gpt-4o", "gpt-4o");
        self.register_alias("openai/gpt-4o-mini", "gpt-4o-mini");
        self.register_alias("openai/o1", "o1");
        self.register_alias("openai/o1-mini", "o1-mini");
        self.register_alias("openai/o3-mini", "o3-mini");

        // --- Anthropic Claude 3 & 3.5 & 3.7 ---
        let anthropic_models = [
            ("claude-3-7-sonnet-20250219", 200000, 8192, 3.00, 15.00, 0.30),
            ("claude-3-5-sonnet-20241022", 200000, 8192, 3.00, 15.00, 0.30),
            ("claude-3-5-haiku-20241022", 200000, 8192, 0.80, 4.00, 0.08),
            ("claude-3-opus-20240229", 200000, 4096, 15.00, 75.00, 1.50),
            ("claude-3-sonnet-20240229", 200000, 4096, 3.00, 15.00, 0.30),
            ("claude-3-haiku-20240307", 200000, 4096, 0.25, 1.25, 0.03),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in anthropic_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "anthropic".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: true,
                supports_function_calling: true,
            });
        }
        // Claude Aliases
        self.register_alias("claude-3-7-sonnet", "claude-3-7-sonnet-20250219");
        self.register_alias("claude-3.7-sonnet", "claude-3-7-sonnet-20250219");
        self.register_alias("claude-3-5-sonnet", "claude-3-5-sonnet-20241022");
        self.register_alias("claude-3.5-sonnet", "claude-3-5-sonnet-20241022");
        self.register_alias("claude-3-5-haiku", "claude-3-5-haiku-20241022");
        self.register_alias("claude-3.5-haiku", "claude-3-5-haiku-20241022");
        self.register_alias("claude-3-opus", "claude-3-opus-20240229");
        self.register_alias("anthropic/claude-3-7-sonnet", "claude-3-7-sonnet-20250219");
        self.register_alias("anthropic/claude-3-5-sonnet", "claude-3-5-sonnet-20241022");

        // --- Google Gemini ---
        let google_models = [
            ("gemini-2.0-flash", 1048576, 8192, 0.10, 0.40, 0.025),
            ("gemini-2.0-pro-exp-02-05", 2097152, 8192, 1.25, 5.00, 0.31),
            ("gemini-1.5-pro", 2097152, 8192, 1.25, 5.00, 0.31),
            ("gemini-1.5-flash", 1048576, 8192, 0.075, 0.30, 0.018),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in google_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "gemini".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: true,
                supports_function_calling: true,
            });
        }
        self.register_alias("gemini-2.0", "gemini-2.0-flash");
        self.register_alias("gemini-2.5-pro", "gemini-2.0-pro-exp-02-05");
        self.register_alias("google/gemini-2.0-flash", "gemini-2.0-flash");
        self.register_alias("google/gemini-1.5-pro", "gemini-1.5-pro");

        // --- DeepSeek ---
        let deepseek_models = [
            ("deepseek-chat", 64000, 8192, 0.14, 0.28, 0.014),
            ("deepseek-reasoner", 64000, 8192, 0.55, 2.19, 0.14),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in deepseek_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "deepseek".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: false,
                supports_function_calling: true,
            });
        }
        self.register_alias("deepseek-v3", "deepseek-chat");
        self.register_alias("deepseek-r1", "deepseek-reasoner");
        self.register_alias("deepseek/deepseek-chat", "deepseek-chat");
        self.register_alias("deepseek/deepseek-r1", "deepseek-reasoner");

        // --- xAI Grok ---
        let grok_models = [
            ("grok-2-1212", 131072, 8192, 2.00, 10.00, 1.00),
            ("grok-2-vision-1212", 32768, 8192, 2.00, 10.00, 1.00),
            ("grok-3", 131072, 8192, 3.00, 15.00, 1.50),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in grok_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "grok".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: name.contains("vision"),
                supports_function_calling: true,
            });
        }
        self.register_alias("grok-2", "grok-2-1212");
        self.register_alias("xai/grok-2", "grok-2-1212");
        self.register_alias("xai/grok-3", "grok-3");

        // --- Meta Llama 3.1 & 3.3 ---
        let llama_models = [
            ("meta-llama/llama-3.3-70b-instruct", 128000, 8192, 0.40, 0.40, 0.10),
            ("meta-llama/llama-3.1-405b-instruct", 128000, 8192, 2.00, 2.00, 0.50),
            ("meta-llama/llama-3.1-8b-instruct", 128000, 8192, 0.05, 0.05, 0.01),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in llama_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "together".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: false,
                supports_function_calling: true,
            });
        }
        self.register_alias("llama-3.3-70b", "meta-llama/llama-3.3-70b-instruct");
        self.register_alias("llama-3.1-405b", "meta-llama/llama-3.1-405b-instruct");
        self.register_alias("llama-3.1-8b", "meta-llama/llama-3.1-8b-instruct");

        // --- Mistral ---
        let mistral_models = [
            ("mistral-large-2411", 128000, 8192, 2.00, 6.00, 0.50),
            ("codestral-2501", 256000, 8192, 0.30, 0.90, 0.10),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in mistral_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "mistral".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: false,
                supports_function_calling: true,
            });
        }
        self.register_alias("mistral-large", "mistral-large-2411");
        self.register_alias("codestral", "codestral-2501");

        // --- Qwen ---
        let qwen_models = [
            ("qwen-2.5-coder-32b-instruct", 128000, 8192, 0.20, 0.20, 0.05),
            ("qwen-2.5-72b-instruct", 128000, 8192, 0.40, 0.40, 0.10),
        ];
        for (name, ctx, max_out, in_cost, out_cost, cache_cost) in qwen_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: "together".to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: false,
                supports_function_calling: true,
            });
        }
        self.register_alias("qwen-2.5-coder", "qwen-2.5-coder-32b-instruct");
        self.register_alias("qwen-2.5-72b", "qwen-2.5-72b-instruct");

        // --- Extended 100+ Frontier, Ecosystem & 22 Local Engines ---
        let extra_models = [
            ("gpt-4.1", "openai", 1047576, 32768, 2.00, 8.00, 0.50),
            ("gpt-4.1-mini", "openai", 1047576, 32768, 0.40, 1.60, 0.10),
            ("gpt-4.1-nano", "openai", 1047576, 32768, 0.10, 0.40, 0.025),
            ("o3", "openai", 200000, 100000, 10.00, 40.00, 2.50),
            ("o4-mini", "openai", 200000, 100000, 1.10, 4.40, 0.275),
            ("text-embedding-3-large", "openai", 8191, 0, 0.13, 0.0, 0.0),
            ("text-embedding-3-small", "openai", 8191, 0, 0.02, 0.0, 0.0),
            ("claude-sonnet-4-20250514", "anthropic", 200000, 64000, 3.00, 15.00, 0.30),
            ("claude-opus-4-20250514", "anthropic", 200000, 32000, 15.00, 75.00, 1.50),
            ("gemini-2.5-pro-preview", "gemini", 1048576, 65536, 1.25, 10.00, 0.31),
            ("gemini-2.5-flash", "gemini", 1048576, 65536, 0.15, 0.60, 0.0375),
            ("command-r-plus", "cohere", 128000, 4096, 2.50, 10.00, 0.0),
            ("sonar-pro", "perplexity", 200000, 8192, 3.00, 15.00, 0.0),
        ];
        for (name, prov, ctx, max_out, in_cost, out_cost, cache_cost) in extra_models {
            self.register_model(ModelInfo {
                canonical_name: name.to_string(),
                provider: prov.to_string(),
                context_window: ctx,
                max_output_tokens: max_out,
                pricing: ModelPricing {
                    input_cost_per_million: in_cost,
                    output_cost_per_million: out_cost,
                    cache_read_input_cost_per_million: cache_cost,
                },
                supports_vision: true,
                supports_function_calling: max_out > 0,
            });
        }

        // 22 Local Inference Engines (zero-cost local billing)
        let local_engines = [
            "ollama", "vllm", "lmstudio", "llamacpp", "mistralrs", "candle",
            "sglang", "tgi", "tabby", "exo", "localai", "jan", "gpt4all",
            "koboldcpp", "mlx", "tensorrt_llm", "triton", "xinference",
            "gpustack", "aphrodite", "llamafile", "ramalama",
        ];
        for engine in local_engines {
            let canonical = format!("{}/local-default", engine);
            self.register_model(ModelInfo {
                canonical_name: canonical.clone(),
                provider: engine.to_string(),
                context_window: 131072,
                max_output_tokens: 16384,
                pricing: ModelPricing {
                    input_cost_per_million: 0.0,
                    output_cost_per_million: 0.0,
                    cache_read_input_cost_per_million: 0.0,
                },
                supports_vision: false,
                supports_function_calling: true,
            });
            self.register_alias(engine, &canonical);
            self.register_alias(&format!("{}/qwen2.5-coder", engine), &canonical);
            self.register_alias(&format!("{}/llama3.3", engine), &canonical);
        }

        // Bulk 100+ provider/model aliases
        let alias_pairs = [
            ("gpt-4o-2024-11-20", "gpt-4o"),
            ("gpt-4o-2024-08-06", "gpt-4o"),
            ("gpt-4o-mini-2024-07-18", "gpt-4o-mini"),
            ("azure/gpt-4o", "gpt-4o"),
            ("azure/gpt-4o-mini", "gpt-4o-mini"),
            ("azure/gpt-4.1", "gpt-4.1"),
            ("openai/gpt-4.1", "gpt-4.1"),
            ("openai/gpt-4.1-mini", "gpt-4.1-mini"),
            ("openai/gpt-4.1-nano", "gpt-4.1-nano"),
            ("openai/o3", "o3"),
            ("openai/o4-mini", "o4-mini"),
            ("openai/text-embedding-3-large", "text-embedding-3-large"),
            ("openai/text-embedding-3-small", "text-embedding-3-small"),
            ("claude-sonnet-4", "claude-sonnet-4-20250514"),
            ("claude-opus-4", "claude-opus-4-20250514"),
            ("anthropic/claude-sonnet-4", "claude-sonnet-4-20250514"),
            ("anthropic/claude-opus-4", "claude-opus-4-20250514"),
            ("bedrock/anthropic.claude-3-7-sonnet", "claude-3-7-sonnet-20250219"),
            ("bedrock/anthropic.claude-3-5-sonnet", "claude-3-5-sonnet-20241022"),
            ("vertex_ai/claude-3-7-sonnet", "claude-3-7-sonnet-20250219"),
            ("vertex_ai/gemini-2.5-pro", "gemini-2.5-pro-preview"),
            ("vertex_ai/gemini-2.5-flash", "gemini-2.5-flash"),
            ("google/gemini-2.5-pro", "gemini-2.5-pro-preview"),
            ("google/gemini-2.5-flash", "gemini-2.5-flash"),
            ("openrouter/anthropic/claude-3.7-sonnet", "claude-3-7-sonnet-20250219"),
            ("openrouter/openai/gpt-4o", "gpt-4o"),
            ("openrouter/deepseek/deepseek-r1", "deepseek-reasoner"),
            ("fireworks_ai/deepseek-v3", "deepseek-chat"),
            ("together_ai/deepseek-r1", "deepseek-reasoner"),
            ("groq/llama-3.3-70b-versatile", "meta-llama/llama-3.3-70b-instruct"),
            ("cerebras/llama-3.3-70b", "meta-llama/llama-3.3-70b-instruct"),
            ("xai/grok-2-vision", "grok-2-vision-1212"),
            ("cohere/command-r-plus", "command-r-plus"),
            ("perplexity/sonar-pro", "sonar-pro"),
        ];
        for (alias, canonical) in alias_pairs {
            self.register_alias(alias, canonical);
        }
    }

    /// Returns total registered aliases (including canonical names).
    pub fn alias_count(&self) -> usize {
        self.aliases.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_litellm_alias_resolution() {
        let matrix = LiteLLMMatrix::new();

        let m1 = matrix.resolve_model("claude-3-7-sonnet");
        assert!(m1.is_some());
        if let Some(info) = m1 {
            assert_eq!(info.canonical_name, "claude-3-7-sonnet-20250219");
            assert_eq!(info.provider, "anthropic");
            assert_eq!(info.context_window, 200000);
        }

        let m2 = matrix.resolve_model("chatgpt-4o");
        assert!(m2.is_some());
        if let Some(info) = m2 {
            assert_eq!(info.canonical_name, "gpt-4o");
            assert_eq!(info.provider, "openai");
        }

        let m3 = matrix.resolve_model("deepseek-r1");
        assert!(m3.is_some());
        if let Some(info) = m3 {
            assert_eq!(info.canonical_name, "deepseek-reasoner");
            assert_eq!(info.provider, "deepseek");
        }

        let m4 = matrix.resolve_model("codestral");
        assert!(m4.is_some());
        if let Some(info) = m4 {
            assert_eq!(info.provider, "mistral");
        }
    }

    #[test]
    fn test_pricing_calculation() {
        let matrix = LiteLLMMatrix::new();
        let info = matrix.resolve_model("gpt-4o");
        assert!(info.is_some());
        if let Some(m) = info {
            assert_eq!(m.pricing.input_cost_per_million, 2.50);
            assert_eq!(m.pricing.output_cost_per_million, 10.00);
        }
    }

    #[test]
    fn test_100_plus_aliases_local_engines_and_endpoints() {
        let matrix = LiteLLMMatrix::new();
        assert!(matrix.alias_count() >= 100);
        assert_eq!(
            matrix.resolve_model("ollama/qwen2.5-coder").map(|m| m.provider.as_str()),
            Some("ollama")
        );
        assert_eq!(
            matrix.resolve_model("mistralrs").map(|m| m.pricing.input_cost_per_million),
            Some(0.0)
        );
        assert_eq!(
            map_endpoint_for_provider("anthropic", EndpointSurface::ChatCompletions),
            "/v1/messages"
        );
        assert_eq!(
            map_endpoint_for_provider("openai", EndpointSurface::Responses),
            "/v1/responses"
        );
        assert_eq!(
            map_endpoint_for_provider("openai", EndpointSurface::Embeddings),
            "/v1/embeddings"
        );
    }
}
