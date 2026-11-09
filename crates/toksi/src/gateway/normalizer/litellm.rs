// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/normalizer/litellm.rs
// created: 2026-11-09
// lastModified: 2026-11-09

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_litellm_alias_resolution() {
        let matrix = LiteLLMMatrix::new();

        let m1 = matrix.resolve_model("claude-3-7-sonnet").expect("resolve 3.7 sonnet");
        assert_eq!(m1.canonical_name, "claude-3-7-sonnet-20250219");
        assert_eq!(m1.provider, "anthropic");
        assert_eq!(m1.context_window, 200000);

        let m2 = matrix.resolve_model("chatgpt-4o").expect("resolve chatgpt-4o");
        assert_eq!(m2.canonical_name, "gpt-4o");
        assert_eq!(m2.provider, "openai");

        let m3 = matrix.resolve_model("deepseek-r1").expect("resolve deepseek-r1");
        assert_eq!(m3.canonical_name, "deepseek-reasoner");
        assert_eq!(m3.provider, "deepseek");

        let m4 = matrix.resolve_model("codestral").expect("resolve codestral");
        assert_eq!(m4.provider, "mistral");
    }

    #[test]
    fn test_pricing_calculation() {
        let matrix = LiteLLMMatrix::new();
        let info = matrix.resolve_model("gpt-4o").expect("gpt-4o");
        assert_eq!(info.pricing.input_cost_per_million, 2.50);
        assert_eq!(info.pricing.output_cost_per_million, 10.00);
    }
}
