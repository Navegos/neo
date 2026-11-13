// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-13

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

/// Enumeration of all 92 in-tree AI providers (Categories 1-5 + 23 local inference engines)
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

    // Accelerator (18)
    Cerebras,
    DeepSeek,
    Fireworks,
    Groq,
    Nebius,
    Parasail,
    Together,
    Wafer,
    Baseten,
    DeepInfra,
    Novita,
    Lepton,
    Modal,
    SambaNova,
    Hyperbolic,
    Featherless,
    SiliconFlow,
    Anyscale,

    // Aggregators (10)
    Bifrost,
    Cloudflare,
    HuggingFace,
    OpenRouter,
    Replicate,
    Kimchi,
    LiteLLM,
    Portkey,
    Unify,
    Martian,

    // Specialized (18)
    BytePlus,
    Cohere,
    Minimax,
    Mistral,
    Perplexity,
    Sarvam,
    TypeSafe,
    XAI,
    DashScope,
    Zhipu,
    Moonshot,
    Volcengine,
    Baichuan,
    AI21,
    Voyage,
    Jina,
    ZeroOne,
    StepFun,

    // Multimodal (12)
    ElevenLabs,
    Runware,
    Runway,
    Stability,
    Fal,
    BlackForestLabs,
    AssemblyAI,
    Deepgram,
    Cartesia,
    Luma,
    Kling,
    Suno,

    // Local Inference Engines (23)
    LlamaCpp,
    Ollama,
    SGLang,
    VLLM,
    TGI,
    LMStudio,
    LocalAI,
    TabbyAPI,
    MlcLlm,
    Exo,
    JanAI,
    GPT4All,
    MstyAI,
    LMDeploy,
    FastChat,
    Triton,
    TextGenerationWebUI,
    OpenWebUI,
    KoboldCpp,
    Llamafile,
    AnythingLLM,
    MlxLm,
    Aphrodite,

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

    pub const ALL: [ProviderId; 92] = [
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
        ProviderId::Baseten,
        ProviderId::DeepInfra,
        ProviderId::Novita,
        ProviderId::Lepton,
        ProviderId::Modal,
        ProviderId::SambaNova,
        ProviderId::Hyperbolic,
        ProviderId::Featherless,
        ProviderId::SiliconFlow,
        ProviderId::Anyscale,
        ProviderId::Bifrost,
        ProviderId::Cloudflare,
        ProviderId::HuggingFace,
        ProviderId::OpenRouter,
        ProviderId::Replicate,
        ProviderId::Kimchi,
        ProviderId::LiteLLM,
        ProviderId::Portkey,
        ProviderId::Unify,
        ProviderId::Martian,
        ProviderId::BytePlus,
        ProviderId::Cohere,
        ProviderId::Minimax,
        ProviderId::Mistral,
        ProviderId::Perplexity,
        ProviderId::Sarvam,
        ProviderId::TypeSafe,
        ProviderId::XAI,
        ProviderId::DashScope,
        ProviderId::Zhipu,
        ProviderId::Moonshot,
        ProviderId::Volcengine,
        ProviderId::Baichuan,
        ProviderId::AI21,
        ProviderId::Voyage,
        ProviderId::Jina,
        ProviderId::ZeroOne,
        ProviderId::StepFun,
        ProviderId::ElevenLabs,
        ProviderId::Runware,
        ProviderId::Runway,
        ProviderId::Stability,
        ProviderId::Fal,
        ProviderId::BlackForestLabs,
        ProviderId::AssemblyAI,
        ProviderId::Deepgram,
        ProviderId::Cartesia,
        ProviderId::Luma,
        ProviderId::Kling,
        ProviderId::Suno,
        ProviderId::LlamaCpp,
        ProviderId::Ollama,
        ProviderId::SGLang,
        ProviderId::VLLM,
        ProviderId::TGI,
        ProviderId::LMStudio,
        ProviderId::LocalAI,
        ProviderId::TabbyAPI,
        ProviderId::MlcLlm,
        ProviderId::Exo,
        ProviderId::JanAI,
        ProviderId::GPT4All,
        ProviderId::MstyAI,
        ProviderId::LMDeploy,
        ProviderId::FastChat,
        ProviderId::Triton,
        ProviderId::TextGenerationWebUI,
        ProviderId::OpenWebUI,
        ProviderId::KoboldCpp,
        ProviderId::Llamafile,
        ProviderId::AnythingLLM,
        ProviderId::MlxLm,
        ProviderId::Aphrodite,
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
            Self::Baseten => "baseten",
            Self::DeepInfra => "deepinfra",
            Self::Novita => "novita",
            Self::Lepton => "lepton",
            Self::Modal => "modal",
            Self::SambaNova => "sambanova",
            Self::Hyperbolic => "hyperbolic",
            Self::Featherless => "featherless",
            Self::SiliconFlow => "siliconflow",
            Self::Anyscale => "anyscale",
            Self::Bifrost => "bifrost",
            Self::Cloudflare => "cloudflare",
            Self::HuggingFace => "huggingface",
            Self::OpenRouter => "openrouter",
            Self::Replicate => "replicate",
            Self::Kimchi => "kimchi",
            Self::LiteLLM => "litellm",
            Self::Portkey => "portkey",
            Self::Unify => "unify",
            Self::Martian => "martian",
            Self::BytePlus => "byteplus",
            Self::Cohere => "cohere",
            Self::Minimax => "minimax",
            Self::Mistral => "mistral",
            Self::Perplexity => "perplexity",
            Self::Sarvam => "sarvam",
            Self::TypeSafe => "typesafe",
            Self::XAI => "xai",
            Self::DashScope => "dashscope",
            Self::Zhipu => "zhipu",
            Self::Moonshot => "moonshot",
            Self::Volcengine => "volcengine",
            Self::Baichuan => "baichuan",
            Self::AI21 => "ai21",
            Self::Voyage => "voyage",
            Self::Jina => "jina",
            Self::ZeroOne => "zeroone",
            Self::StepFun => "stepfun",
            Self::ElevenLabs => "elevenlabs",
            Self::Runware => "runware",
            Self::Runway => "runway",
            Self::Stability => "stability",
            Self::Fal => "fal",
            Self::BlackForestLabs => "blackforestlabs",
            Self::AssemblyAI => "assemblyai",
            Self::Deepgram => "deepgram",
            Self::Cartesia => "cartesia",
            Self::Luma => "luma",
            Self::Kling => "kling",
            Self::Suno => "suno",
            Self::LlamaCpp => "llamacpp",
            Self::Ollama => "ollama",
            Self::SGLang => "sglang",
            Self::VLLM => "vllm",
            Self::TGI => "tgi",
            Self::LMStudio => "lmstudio",
            Self::LocalAI => "localai",
            Self::TabbyAPI => "tabbyapi",
            Self::MlcLlm => "mlcllm",
            Self::Exo => "exo",
            Self::JanAI => "jan",
            Self::GPT4All => "gpt4all",
            Self::MstyAI => "msty",
            Self::LMDeploy => "lmdeploy",
            Self::FastChat => "fastchat",
            Self::Triton => "triton",
            Self::TextGenerationWebUI => "oobabooga",
            Self::OpenWebUI => "openwebui",
            Self::KoboldCpp => "koboldcpp",
            Self::Llamafile => "llamafile",
            Self::AnythingLLM => "anythingllm",
            Self::MlxLm => "mlxlm",
            Self::Aphrodite => "aphrodite",
            Self::GitHubCopilot => "githubcopilot",
            Self::OpenCode => "opencode",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let n = name.trim().to_lowercase();
        let canonical = match n.as_str() {
            "llama.cpp" | "llama-cpp" => "llamacpp",
            "sgl" => "sglang",
            "lm-studio" | "lm_studio" => "lmstudio",
            "mlc-llm" | "mlc_llm" => "mlcllm",
            "janai" | "jan-ai" => "jan",
            "mstyai" | "msty-ai" => "msty",
            "text-generation-webui" | "textgenwebui" => "oobabooga",
            "open-webui" | "open_webui" => "openwebui",
            "mlx-lm" | "mlx_lm" => "mlxlm",
            "github-copilot" | "copilot" => "githubcopilot",
            "bedrock-mantle" => "bedrockmantle",
            "hugging-face" | "hf" => "huggingface",
            "google" => "gemini",
            "grok" => "xai",
            "01ai" | "yi" => "zeroone",
            _ => n.as_str(),
        };
        Self::ALL.into_iter().find(|p| p.as_str() == canonical)
    }

    pub fn category(&self) -> ProviderCategory {
        match self {
            Self::Anthropic | Self::Azure | Self::Bedrock | Self::BedrockMantle |
            Self::SageMaker | Self::Databricks | Self::Gemini | Self::OpenAI |
            Self::Vertex => ProviderCategory::Frontier,

            Self::Cerebras | Self::DeepSeek | Self::Fireworks | Self::Groq |
            Self::Nebius | Self::Parasail | Self::Together | Self::Wafer |
            Self::Baseten | Self::DeepInfra | Self::Novita | Self::Lepton |
            Self::Modal | Self::SambaNova | Self::Hyperbolic | Self::Featherless |
            Self::SiliconFlow | Self::Anyscale => ProviderCategory::Accelerator,

            Self::Bifrost | Self::Cloudflare | Self::HuggingFace | Self::OpenRouter |
            Self::Replicate | Self::Kimchi | Self::LiteLLM | Self::Portkey |
            Self::Unify | Self::Martian => ProviderCategory::Aggregator,

            Self::BytePlus | Self::Cohere | Self::Minimax | Self::Mistral |
            Self::Perplexity | Self::Sarvam | Self::TypeSafe | Self::XAI |
            Self::DashScope | Self::Zhipu | Self::Moonshot | Self::Volcengine |
            Self::Baichuan | Self::AI21 | Self::Voyage | Self::Jina |
            Self::ZeroOne | Self::StepFun => ProviderCategory::Specialized,

            Self::ElevenLabs | Self::Runware | Self::Runway | Self::Stability |
            Self::Fal | Self::BlackForestLabs | Self::AssemblyAI | Self::Deepgram |
            Self::Cartesia | Self::Luma | Self::Kling | Self::Suno => ProviderCategory::Multimodal,

            Self::LlamaCpp | Self::Ollama | Self::SGLang | Self::VLLM |
            Self::TGI | Self::LMStudio | Self::LocalAI | Self::TabbyAPI |
            Self::MlcLlm | Self::Exo | Self::JanAI | Self::GPT4All |
            Self::MstyAI | Self::LMDeploy | Self::FastChat | Self::Triton |
            Self::TextGenerationWebUI | Self::OpenWebUI | Self::KoboldCpp |
            Self::Llamafile | Self::AnythingLLM | Self::MlxLm |
            Self::Aphrodite => ProviderCategory::Local,

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
            Self::Baseten => "https://inference.baseten.co/v1",
            Self::DeepInfra => "https://api.deepinfra.com/v1/openai",
            Self::Novita => "https://api.novita.ai/v3/openai",
            Self::Lepton => "https://{workspace}.lepton.run/api/v1",
            Self::Modal => "https://{workspace}--vllm.modal.run/v1",
            Self::SambaNova => "https://api.sambanova.ai/v1",
            Self::Hyperbolic => "https://api.hyperbolic.xyz/v1",
            Self::Featherless => "https://api.featherless.ai/v1",
            Self::SiliconFlow => "https://api.siliconflow.cn/v1",
            Self::Anyscale => "https://api.endpoints.anyscale.com/v1",
            Self::Bifrost => "http://127.0.0.1:8080/v1",
            Self::Cloudflare => "https://api.cloudflare.com/client/v4/accounts/{account}/ai/run",
            Self::HuggingFace => "https://api-inference.huggingface.co/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::Replicate => "https://api.replicate.com/v1",
            Self::Kimchi => "https://api.kimchi.ai/v1",
            Self::LiteLLM => "http://127.0.0.1:4000/v1",
            Self::Portkey => "https://api.portkey.ai/v1",
            Self::Unify => "https://api.unify.ai/v0",
            Self::Martian => "https://withmartian.com/api/openai/v1",
            Self::BytePlus => "https://ark.cn-beijing.volces.com/api/v3",
            Self::Cohere => "https://api.cohere.com/v2",
            Self::Minimax => "https://api.minimaxi.chat/v1",
            Self::Mistral => "https://api.mistral.ai/v1",
            Self::Perplexity => "https://api.perplexity.ai",
            Self::Sarvam => "https://api.sarvam.ai/v1",
            Self::TypeSafe => "https://api.typesafe.ai/v1",
            Self::XAI => "https://api.x.ai/v1",
            Self::DashScope => "https://dashscope.aliyuncs.com/compatible-mode/v1",
            Self::Zhipu => "https://open.bigmodel.cn/api/paas/v4",
            Self::Moonshot => "https://api.moonshot.cn/v1",
            Self::Volcengine => "https://ark.cn-beijing.volces.com/api/v3",
            Self::Baichuan => "https://api.baichuan-ai.com/v1",
            Self::AI21 => "https://api.ai21.com/studio/v1",
            Self::Voyage => "https://api.voyageai.com/v1",
            Self::Jina => "https://api.jina.ai/v1",
            Self::ZeroOne => "https://api.lingyiwanwu.com/v1",
            Self::StepFun => "https://api.stepfun.com/v1",
            Self::ElevenLabs => "https://api.elevenlabs.io/v1",
            Self::Runware => "https://api.runware.ai/v1",
            Self::Runway => "https://api.runwayml.com/v1",
            Self::Stability => "https://api.stability.ai/v2beta",
            Self::Fal => "https://fal.run",
            Self::BlackForestLabs => "https://api.bfl.ml/v1",
            Self::AssemblyAI => "https://api.assemblyai.com/v2",
            Self::Deepgram => "https://api.deepgram.com/v1",
            Self::Cartesia => "https://api.cartesia.ai",
            Self::Luma => "https://api.lumalabs.ai/dream-machine/v1",
            Self::Kling => "https://api.klingai.com/v1",
            Self::Suno => "https://api.suno.ai/v1",
            Self::LlamaCpp => "http://127.0.0.1:8080/v1",
            Self::Ollama => "http://127.0.0.1:11434/v1",
            Self::SGLang => "http://127.0.0.1:30000/v1",
            Self::VLLM => "http://127.0.0.1:8000/v1",
            Self::TGI => "http://127.0.0.1:8080/v1",
            Self::LMStudio => "http://127.0.0.1:1234/v1",
            Self::LocalAI => "http://127.0.0.1:8080/v1",
            Self::TabbyAPI => "http://127.0.0.1:5000/v1",
            Self::MlcLlm => "http://127.0.0.1:8000/v1",
            Self::Exo => "http://127.0.0.1:52415/v1",
            Self::JanAI => "http://127.0.0.1:1337/v1",
            Self::GPT4All => "http://127.0.0.1:4891/v1",
            Self::MstyAI => "http://127.0.0.1:10000/v1",
            Self::LMDeploy => "http://127.0.0.1:23333/v1",
            Self::FastChat => "http://127.0.0.1:8000/v1",
            Self::Triton => "http://127.0.0.1:8000/v2",
            Self::TextGenerationWebUI => "http://127.0.0.1:5000/v1",
            Self::OpenWebUI => "http://127.0.0.1:3000/api",
            Self::KoboldCpp => "http://127.0.0.1:5001/v1",
            Self::Llamafile => "http://127.0.0.1:8080/v1",
            Self::AnythingLLM => "http://127.0.0.1:3001/api/v1",
            Self::MlxLm => "http://127.0.0.1:8080/v1",
            Self::Aphrodite => "http://127.0.0.1:2242/v1",
            Self::GitHubCopilot => "https://api.githubcopilot.com",
            Self::OpenCode => "https://api.opencode.ai/v1",
        }
    }

    /// Resolve editable base URL from `[base_urls]` in `~/.toksi/config.toml` or fall back to default.
    pub fn effective_endpoint(&self) -> String {
        crate::util::ports::resolve_custom_base_url(self.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| self.default_endpoint().to_string())
    }
}

/// Shared OpenAI/Anthropic/Gemini/Responses token usage extractor (returns `0` when absent — never synthetic).
pub fn parse_openai_usage(resp: &Value) -> (u32, u32) {
    if let Some(usage) = resp.get("usage") {
        let input = usage
            .get("prompt_tokens")
            .or_else(|| usage.get("input_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let output = usage
            .get("completion_tokens")
            .or_else(|| usage.get("output_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        return (input, output);
    }
    if let Some(meta) = resp.get("usageMetadata") {
        let input = meta.get("promptTokenCount").and_then(Value::as_u64).unwrap_or(0) as u32;
        let output = meta.get("candidatesTokenCount").and_then(Value::as_u64).unwrap_or(0) as u32;
        return (input, output);
    }
    (0, 0)
}

/// Shared response normalizer for OpenAI-compatible provider payloads.
pub fn parse_openai_chat_response(resp: &Value, fallback_model: &str) -> Result<UnifiedResponse, String> {
    let id = resp.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    let model = resp
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(fallback_model)
        .to_string();
    let first_choice = resp.get("choices").and_then(Value::as_array).and_then(|a| a.first());
    let content = first_choice
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .or_else(|| first_choice.and_then(|c| c.get("text")).and_then(Value::as_str))
        .unwrap_or("")
        .to_string();
    let finish_reason = first_choice
        .and_then(|c| c.get("finish_reason"))
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let (input_tokens, output_tokens) = parse_openai_usage(resp);

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

/// Build the concrete `ProviderClient` for any cataloged `ProviderId`.
pub fn build_provider_client(id: ProviderId) -> Box<dyn ProviderClient> {
    let custom = crate::util::ports::resolve_custom_base_url(id.as_str());
    match id {
        ProviderId::Anthropic => {
            let mut p = frontier::AnthropicProvider::default();
            if let Some(url) = custom {
                p.base_url = url;
            }
            Box::new(p)
        }
        ProviderId::OpenAI => {
            let mut p = frontier::OpenAIProvider::default();
            if let Some(url) = custom {
                p.base_url = url;
            }
            Box::new(p)
        }
        ProviderId::Gemini => {
            let mut p = frontier::GeminiProvider::default();
            if let Some(url) = custom {
                p.base_url = url;
            }
            Box::new(p)
        }
        ProviderId::Bifrost => {
            let mut p = aggregators::bifrost::BifrostProviderClient::default();
            if let Some(url) = custom {
                p.endpoint_url = url.trim_end_matches('/').to_string();
            }
            Box::new(p)
        }
        _ => match id.category() {
            ProviderCategory::Frontier => {
                let mut p = frontier::GenericFrontierProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
            ProviderCategory::Accelerator => {
                let mut p = accelerator::AcceleratorProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
            ProviderCategory::Aggregator => {
                let mut p = aggregators::AggregatorProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
            ProviderCategory::Specialized => {
                let mut p = specialized::SpecializedProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
            ProviderCategory::Multimodal => {
                let mut p = multimodal::MultimodalProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
            ProviderCategory::Local => {
                let mut p = local::LocalProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
            ProviderCategory::Ecosystem => {
                let mut p = ecosystem::EcosystemProvider::new(id);
                if let Some(url) = custom {
                    p.endpoint_url = url;
                }
                Box::new(p)
            }
        },
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

