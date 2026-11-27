# Full AI Ecosystem: Agent Runtimes, Local Inference Engines, Protocols, Gateways, and Quota Architecture

Comprehensive technical blueprint and master catalog covering cloud AI providers, local inference engines, agentic CLIs, LSP daemons, AI-native IDEs, cloud platforms, proxy gateways, protocols, and quota enforcement systems.

---

## 1. Foundation Cloud AI Model Providers & Inference Engines

### 1.1 Technical API Endpoint Specifications & Core Routes

| Provider | Base URL (HTTP) | Core REST API Paths | Realtime / WebSocket URL | Supported Versions | Authentication Scheme | Key Models Available (Production & Frontier) | Thinking / Extended Reasoning Parameters |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Google Gemini API (AI Studio)** | `https://generativelanguage.googleapis.com` | `POST /{ver}/models/{model}:generateContent`<br>`POST /{ver}/models/{model}:streamGenerateContent`<br>`POST /{ver}/models/{model}:embedContent`<br>`POST /{ver}/cachedContents`<br>`POST /upload/{ver}/files` | `wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.<version>.GenerativeService.BidiGenerateContent` | `v1`, `v1alpha`, `v1beta`, `v1beta1`, `v1beta2`, `v1beta3`, `v1main` | Query param: `?key=$GEMINI_API_KEY`<br>Header: `x-goog-api-key: $GEMINI_API_KEY`<br>Bearer token: `Authorization: Bearer $(gcloud auth print-access-token)` | `gemini-4-carbon` *(Nov 2026 GA)*<br>`gemini-4-hydrogen` *(Nov 2026 GA)*<br>`gemini-4-neon` *(Nov 2026)*<br>`gemini-4-argon`<br>`gemini-3.8-flash`<br>`gemini-3.6-flash`<br>`gemini-3.5-flash-lite`<br>`gemini-3-pro-image`<br>`gemini-3-pro-preview`<br>`gemini-3-flash-preview`<br>`gemini-2.5-pro` *(legacy)*<br>`gemini-2.5-flash` *(legacy)*<br>`gemini-2.5-flash-lite` *(legacy)* | `generationConfig`: `{"thinkingConfig": {"thinkingBudget": <int_tokens>}}` (`0` turns off thinking, `-1` auto-tunes, positive integer sets ceiling up to 24,576) |
| **Google Cloud Vertex AI** | `https://{REGION}-aiplatform.googleapis.com` | `POST /v1/projects/{proj}/locations/{reg}/publishers/google/models/{model}:generateContent`<br>`POST /v1/projects/{proj}/locations/{reg}/publishers/anthropic/models/{model}:rawPredict`<br>`POST /v1beta1/.../endpoints/openapi/chat/completions` | `wss://{REGION}-aiplatform.googleapis.com/ws/google.cloud.aiplatform.<version>.LlmBidiService/BidiGenerateContent` | `v1`, `v1beta1` | Header: `Authorization: Bearer $(gcloud auth print-access-token)`<br>Project URI prefix: `projects/{PROJECT_ID}/locations/{REGION}/publishers/google/models/{MODEL_ID}` | `gemini-4-carbon` *(Nov 2026 GA)*<br>`gemini-4-hydrogen` *(Nov 2026 GA)*<br>`gemini-4-neon` *(Nov 2026)*<br>`gemini-4-argon`<br>`gemini-3.8-flash`<br>`gemini-3.5-flash-lite`<br>`claude-fable-5-2` *(Nov 2026)*<br>`claude-opus-5-5`<br>`claude-sonnet-5-5`<br>`claude-haiku-5-5`<br>`llama-4-maverick`<br>`llama-4-scout`<br>`gemini-2.5-pro@latest` *(legacy)* | `generationConfig.thinkingConfig.thinkingBudget` for Gemini; Anthropic native `thinking.budget_tokens` payload for Claude on Vertex |
| **OpenAI** | `https://api.openai.com` | `POST /v1/responses`<br>`POST /v1/chat/completions`<br>`POST /v1/embeddings`<br>`POST /v1/audio/transcriptions`<br>`POST /v1/images/generations`<br>`POST /v1/batches`<br>`GET /v1/models` | `wss://api.openai.com/v1/realtime` | `v1` | Header: `Authorization: Bearer $OPENAI_API_KEY`<br>Header: `OpenAI-Organization: $ORG_ID`<br>Header: `OpenAI-Project: $PROJECT_ID` | `gpt-6-cyber` *(Nov 2026, vetted security)*<br>`gpt-6.2-astra`<br>`gpt-6.2-sol`<br>`gpt-6.1-sol`<br>`gpt-6.1-terra`<br>`gpt-6-luna`<br>`gpt-image-2.6` *(Sunburst & Flare, Nov 2026)*<br>`gpt-realtime-2.1`<br>`gpt-live-1`<br>`gpt-oss-120b` / `gpt-oss-20b`<br>`text-embedding-3-large`<br>`gpt-5` / `o3` / `o4-mini` *(legacy/retiring)* | `reasoning_effort`: `"low"` \| `"medium"` \| `"high"` (native across GPT-6 family & legacy o-series). Output breakdown in `usage.completion_tokens_details.reasoning_tokens` |
| **Anthropic** | `https://api.anthropic.com` | `POST /v1/messages`<br>`POST /v1/messages/count_tokens`<br>`POST /v1/messages/batches`<br>`GET /v1/models` | N/A (SSE over HTTP `POST /v1/messages` with `stream: true`) | `v1` | Header: `x-api-key: $ANTHROPIC_API_KEY`<br>Header: `anthropic-version: 2023-06-01`<br>Beta Header: `anthropic-beta: output-128k-2025-02-19` | `claude-fable-5-2` *(Nov 2026)*<br>`claude-mythos-5-2` *(Nov 2026, vetted cyber/bio)*<br>`claude-opus-5-5`<br>`claude-sonnet-5-5`<br>`claude-haiku-5-5`<br>`claude-3-7-sonnet-20250219` *(legacy)*<br>`claude-3-5-sonnet-20241022` *(legacy)* | `thinking`: `{"type": "enabled", "budget_tokens": <int>}` (min 1,024, max $\le$ `max_tokens`). Raw reasoning emitted in `thinking` block. Set `type: "disabled"` for fast execution |
| **AWS Bedrock** | `https://bedrock-runtime.{REGION}.amazonaws.com` | `POST /model/{modelId}/converse`<br>`POST /model/{modelId}/converse-stream`<br>`POST /model/{modelId}/invoke`<br>`POST /model/{modelId}/invoke-with-response-stream` | EventStream binary framing over HTTP/2 (`converse-stream`) | `2023-09-30` | AWS SigV4 Signing (`Authorization: AWS4-HMAC-SHA256 ...`) or Bedrock API Key (`Authorization: Bearer $AWS_BEARER_TOKEN_BEDROCK`) | `anthropic.claude-fable-5-2-v1:0` *(Nov 2026)*<br>`anthropic.claude-opus-5-5-v1:0`<br>`anthropic.claude-sonnet-5-5-v1:0`<br>`anthropic.claude-haiku-5-5-v1:0`<br>`amazon.nova-premier-v1:0`<br>`amazon.nova-pro-v1:0`<br>`amazon.nova-lite-v1:0`<br>`amazon.nova-micro-v1:0`<br>`meta.llama4-maverick-17b-instruct-v1:0`<br>`meta.llama4-scout-17b-instruct-v1:0`<br>`cohere.command-a-v1:0`<br>`deepseek.r1-v1:0` | `additionalModelRequestFields`: `{"thinking": {"type": "enabled", "budget_tokens": <int>}}` in Converse API |
| **Azure OpenAI / Azure AI Foundry** | `https://{RESOURCE}.openai.azure.com` | `POST /openai/deployments/{deployment}/chat/completions?api-version={ver}`<br>`POST /openai/v1/responses`<br>`POST /openai/deployments/{deployment}/embeddings` | `wss://{RESOURCE}.openai.azure.com/openai/realtime?api-version=2024-10-01-preview&deployment={dep}` | `2024-10-21`, `2025-01-01-preview`, `v1` | Header: `api-key: $AZURE_OPENAI_API_KEY`<br>Entra ID: `Authorization: Bearer $ENTRA_TOKEN` | `gpt-6.2-astra`<br>`gpt-6.2-sol`<br>`gpt-6.1-terra`<br>`gpt-6-luna`<br>`claude-fable-5-2` *(Nov 2026, Foundry)*<br>`DeepSeek-V4-Pro` / `DeepSeek-V4.1-Flash`<br>`Llama-4-Maverick` / `Llama-4-Scout`<br>`Mistral-Large-4`<br>`Cohere-Command-A` | `reasoning_effort`: `"low"` \| `"medium"` \| `"high"` |
| **xAI** | `https://api.x.ai` | `POST /v1/chat/completions`<br>`POST /v1/messages` (Anthropic compatible)<br>`POST /v1/images/generations`<br>`GET /v1/models` | `wss://api.x.ai/v1/realtime` (Grok Voice API) | `v1` (OpenAI & Anthropic compatible) | Header: `Authorization: Bearer $XAI_API_KEY` | `grok-4.7`<br>`grok-4.6`<br>`grok-4.5`<br>`grok-4.3`<br>`grok-4.20`<br>`grok-imagine-2.0` (`Image 2.0`)<br>`grok-video-1.5`<br>`grok-3` / `grok-3-mini` *(legacy)* | `reasoning_format`: `"raw"` \| `"parsed"` or `reasoning_effort`: `"low"` \| `"high"`; encrypted reasoning (`reasoning.encrypted_content`) on Grok-4.7 |
| **DeepSeek API** | `https://api.deepseek.com` | `POST /chat/completions` (or `/v1/chat/completions`)<br>`POST /beta/chat/completions` (Prefix/FIM)<br>`GET /user/balance`<br>`GET /models` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $DEEPSEEK_API_KEY` | `deepseek-v4-pro` (`DeepSeek-V4-Pro-0813`)<br>`deepseek-flash` (`DeepSeek-V4.1-Flash-1017`, text+vision)<br>`deepseek-reasoner` (R1) *(retired)*<br>`deepseek-chat` (V3) *(retired)* | Native togglable Thinking Mode on V4 models; streams internal CoT via `reasoning_content` parameter in delta responses |
| **Groq Cloud** | `https://api.groq.com/openai` | `POST /v1/chat/completions`<br>`POST /v1/audio/transcriptions`<br>`POST /v1/audio/translations`<br>`POST /v1/batches`<br>`GET /v1/models` | N/A (OpenAI compatible SSE) | `v1` | Header: `Authorization: Bearer $GROQ_API_KEY` | `openai/gpt-oss-120b`<br>`openai/gpt-oss-20b`<br>`meta-llama/llama-4-scout-17b-16e-instruct`<br>`qwen/qwen3.8-27b`<br>`llama-3.3-70b-versatile`<br>`llama-3.1-8b-instant`<br>`meta-llama/llama-prompt-guard-2-86m`<br>`whisper-large-v3-turbo` | `reasoning_format`: `"parsed"` \| `"raw"` \| `"hidden"` on reasoning checkpoints; ultra-high speed LPU inference |
| **Cerebras Cloud** | `https://api.cerebras.ai` | `POST /v1/chat/completions`<br>`POST /v1/completions`<br>`GET /v1/models` | N/A (OpenAI compatible SSE) | `v1` | Header: `Authorization: Bearer $CEREBRAS_API_KEY` | `gpt-oss-120b`<br>`qwen-3.8-27b`<br>`gemma-4-31b`<br>`llama-3.3-70b`<br>`llama-3.1-8b` | Instant execution on Wafer-Scale Engine (2,000--3,000+ tok/s); `reasoning_format` supported on reasoning checkpoints |
| **Fireworks AI** | `https://api.fireworks.ai/inference` | `POST /v1/chat/completions`<br>`POST /v1/completions`<br>`POST /v1/embeddings`<br>`POST /v1/workflows/{account}/{workflow}/run` | `wss://audio-streaming.us-virginia-1.direct.fireworks.ai/v1/audio/transcriptions/streaming` | `v1` | Header: `Authorization: Bearer $FIREWORKS_API_KEY` | `accounts/fireworks/models/deepseek-v4-pro`<br>`accounts/fireworks/models/deepseek-v4p1-flash`<br>`accounts/fireworks/models/kimi-k3`<br>`accounts/fireworks/models/kimi-k2p6`<br>`accounts/fireworks/models/glm-5p3-fast`<br>`accounts/fireworks/models/llama4-maverick-instruct-basic`<br>`accounts/fireworks/models/llama-v3p3-70b-instruct` | `reasoning_effort`: `"low"` \| `"medium"` \| `"high"` or native `<think>` tags |
| **SambaNova Cloud** | `https://api.sambanova.ai` | `POST /v1/chat/completions`<br>`POST /v1/audio/transcriptions`<br>`GET /v1/models` | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $SAMBANOVA_API_KEY` | `DeepSeek-V4-Pro`<br>`DeepSeek-R1`<br>`Llama-4-Maverick-17B-128E-Instruct`<br>`Meta-Llama-3.3-70B-Instruct`<br>`Qwen3.8-27B` | High-speed RDU inference; emits `<think>` blocks in message content |
| **Alibaba Cloud Model Studio / DashScope (Qwen)** | `https://dashscope-intl.aliyuncs.com/compatible-mode` | `POST /v1/chat/completions`<br>`POST /v1/embeddings`<br>`POST /api/v1/services/aigc/text-generation/generation` (Native) | `wss://dashscope-intl.aliyuncs.com/api-ws/v1/inference` | `v1` | Header: `Authorization: Bearer $DASHSCOPE_API_KEY` | `qwen4-max` *(preview)*<br>`qwen3.8-max` / `qwen3.7-max`<br>`qwen3.7-plus` / `qwen3.6-plus`<br>`qwen3.8-flash` / `qwen3.6-flash`<br>`qwen3-vl-plus`<br>`qwen3-omni-flash`<br>`qwen3-asr-flash` / `qwen3-tts-flash`<br>`qwen-image-2.0-pro`<br>`wan` *(video)* | `enable_thinking: true` / `thinking_budget` in `extra_body`; streams reasoning steps in `reasoning_content` |
| **Together AI** | `https://api.together.xyz` | `POST /v1/chat/completions`<br>`POST /v1/completions`<br>`POST /v1/embeddings`<br>`POST /v1/rerank`<br>`POST /v1/images/generations` | `wss://api.together.xyz/v1/realtime` | `v1` | Header: `Authorization: Bearer $TOGETHER_API_KEY` | `deepseek-ai/DeepSeek-V4-Pro`<br>`deepseek-ai/DeepSeek-V4.1-Flash`<br>`moonshotai/Kimi-K3.1-Pro` / `Kimi-K3`<br>`minimax/MiniMax-M3.1-Flash`<br>`meta/Muse-Spark-Open` / `Muse-Glimmer-30B`<br>`meta-llama/Llama-4-Maverick`<br>`nvidia/Nemotron-3.5-Super-VL`<br>`bytedance/Seedream-5.5` | Supports OpenAI standard `reasoning_effort` and outputs `usage.reasoning_tokens` |
| **Mistral AI** | `https://api.mistral.ai` | `POST /v1/chat/completions`<br>`POST /v1/fim/completions` (Codestral)<br>`POST /v1/agents/completions`<br>`POST /v1/embeddings`<br>`POST /v1/ocr` | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $MISTRAL_API_KEY` | `mistral-large-4` *(1T MoE, 1M ctx)*<br>`mistral-medium-3.5`<br>`mistral-small-4` *(unifies Magistral & Devstral)*<br>`codestral-latest`<br>`ministral-3-14b` / `8b` / `3b`<br>`voxtral-3` *(TTS/ASR)*<br>`mistral-ocr-4.1`<br>`timestral` | Native unified reasoning & coding in Large 4 / Small 4 / Medium 3.5 with configurable `reasoning_effort`; FIM infill on Codestral |
| **Cohere** | `https://api.cohere.com` | `POST /v2/chat`<br>`POST /v2/embed`<br>`POST /v2/rerank`<br>`POST /v1/classify`<br>`POST /compatibility/v1/chat/completions` | N/A (SSE HTTP streaming) | `v1`, `v2` | Header: `Authorization: Bearer $CO_KEY` | `command-a-03-2025` (`Command A`)<br>`command-a-reasoning`<br>`command-a-vision`<br>`embed-v4.0`<br>`rerank-v5.0` / `rerank-v3.5`<br>`command-r-plus-08-2024` *(legacy)* | Native tool execution with citations via `tools` array and `documents` RAG grounding; Model Vault isolated inference |
| **Perplexity AI** | `https://api.perplexity.ai` | `POST /chat/completions`<br>`POST /v1/agent` (Agent API)<br>`POST /v1/orchestrator` *(Nov 2026)* | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $PERPLEXITY_API_KEY` | `sonar-pro-search`<br>`sonar-reasoning-pro`<br>`sonar-reasoning`<br>`sonar-pro`<br>`sonar`<br>Multi-provider routing via **Orchestrator API** *(Nov 2026)* | Inline citation retrieval (`citations` array), integrated multi-step search verification (`search_domain_filter`, `search_recency_filter`), and auto-failover via Orchestrator API |
| **NVIDIA NIM API** | `https://integrate.api.nvidia.com` | `POST /v1/chat/completions`<br>`POST /v1/embeddings`<br>`POST /v1/ranking` | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $NVIDIA_API_KEY` | `nvidia/nemotron-3.5-super-vl`<br>`nvidia/nemotron-3.5-lightning`<br>`deepseek-ai/deepseek-v4-pro`<br>`meta/llama-4-maverick-17b-128e-instruct`<br>`nvidia/llama-3.1-nemotron-70b-instruct` *(legacy)* | Standard OpenAI chat completions schema backed by TensorRT-LLM microservices; hybrid MoE reasoning on Nemotron 3.5 |
| **Hugging Face Inference API** | `https://router.huggingface.co` (HF Router) | `POST /v1/chat/completions` (HF Router)<br>`POST /models/{model_id}/v1/chat/completions`<br>`POST /models/{model_id}` (Pipeline task) | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $HF_TOKEN` | Any public/private Hub model (`deepseek-ai/DeepSeek-V4-Pro`, `moonshotai/Kimi-K3.1-Pro`, `zai-org/GLM-5.4`, `meta-llama/Llama-4-Maverick`, `nvidia/Nemotron-3.5-Super-VL`) | Supported via TGI/vLLM/SGLang routing backends across Inference Providers & Dedicated Endpoints |
| **Moonshot AI (Kimi)** | `https://api.moonshot.ai` | `POST /v1/chat/completions`<br>`POST /v1/files`<br>`GET /v1/models` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $MOONSHOT_API_KEY` | `kimi-k3.1-pro` *(2.8T MoE, 1M ctx)*<br>`kimi-k3.1-highspeed`<br>`kimi-k3`<br>`kimi-k2.6` | Native long-horizon thinking & multimodal reasoning with cached-read prompt prefix optimization |
| **Zhipu AI / Z.ai (GLM)** | `https://api.z.ai/api/paas` | `POST /v4/chat/completions`<br>`POST /v4/embeddings`<br>`POST /v4/images/generations`<br>`GET /v4/models` | `wss://api.z.ai/api/paas/v4/realtime` | `v4` (OpenAI compatible) | Header: `Authorization: Bearer $ZHIPU_API_KEY` | `glm-5.4` *(744B MoE, 1M ctx)*<br>`glm-5.3`<br>`glm-5.3-flash`<br>`glm-5.3-flashx` | `thinking`: `{"type": "enabled"}` with `reasoning_content` delta streaming for coding and agentic harnesses |
| **MiniMax** | `https://api.minimax.io` | `POST /v1/chat/completions`<br>`POST /v1/t2a_v2` (Speech)<br>`POST /v1/video_generation` (H3) | `wss://api.minimax.io/ws/v1/t2a_v2` | `v1` (OpenAI & Anthropic compatible) | Header: `Authorization: Bearer $MINIMAX_API_KEY` | `MiniMax-M3.1-Flash` *(1M ctx)*<br>`MiniMax-M3`<br>`MiniMax-H3` *(Max/Lite/Highspeed Video+Audio)*<br>`speech-2.8` | Native `<think>` / `reasoning_split` reasoning traces for coding agents and high-throughput text |
| **ByteDance Seed (Volcano Engine / Ark)** | `https://ark.cn-beijing.volces.com/api/v3` | `POST /chat/completions`<br>`POST /images/generations`<br>`POST /contents/generations/tasks` (Video) | N/A (SSE / Async task polling) | `v3` (OpenAI compatible) | Header: `Authorization: Bearer $ARK_API_KEY` | `seedance-2.5` / `seedance-2.0` *(Audio+Video)*<br>`seedream-5.5-pro` / `seedream-5.5`<br>`omnihuman` / `dreamactor` | Multi-reference cinematic video/image synthesis and `thinking` mode on Doubao/Seed text endpoints |
| **OpenRouter** | `https://openrouter.ai/api` | `POST /v1/chat/completions`<br>`POST /v1/completions`<br>`GET /v1/models`<br>`GET /v1/auth/key` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $OPENROUTER_API_KEY`<br>Optional: `HTTP-Referer`, `X-Title` | 300+ unified models (`anthropic/claude-fable-5-2`, `openai/gpt-6.2-astra`, `google/gemini-4-carbon`, `deepseek/deepseek-v4-pro`, `moonshotai/kimi-k3.1-pro`, `x-ai/grok-4.7`, `nvidia/nemotron-3.5-lightning`) | Unified `reasoning`: `{"effort": "high", "max_tokens": <int>, "exclude": false}` normalized across upstream providers |
| **DeepInfra** | `https://api.deepinfra.com/v1/openai` | `POST /chat/completions`<br>`POST /embeddings`<br>`POST /images/generations`<br>`GET /models` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $DEEPINFRA_API_KEY` | `deepseek-ai/DeepSeek-V4-Pro`<br>`deepseek-ai/DeepSeek-V4.1-Flash`<br>`nvidia/Nemotron-3.5-Super-VL`<br>`nvidia/Nemotron-3.5-Lightning`<br>`meta-llama/Llama-4-Maverick`<br>`Qwen/Qwen3.8-27B` | OpenAI-compatible `reasoning_effort` and `reasoning_content` streaming on open-weight GPU clusters |
| **Replicate** | `https://api.replicate.com` | `POST /v1/predictions`<br>`POST /v1/models/{owner}/{name}/predictions`<br>`POST /v1/trainings` | N/A (SSE `stream` URL or Webhook) | `v1` | Header: `Authorization: Bearer $REPLICATE_API_TOKEN` | On-demand & fine-tuned open models (`meta/llama-4-maverick`, `nvidia/nemotron-3.5-super-vl`, `deepseek-ai/deepseek-v4-pro`, `black-forest-labs/flux-*`) | Model-specific `input` schema parameters; serverless cold-start or dedicated deployments |
| **Cloudflare Workers AI** | `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai` | `POST /v1/chat/completions`<br>`POST /v1/embeddings`<br>`POST /run/{model}` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $CLOUDFLARE_API_TOKEN` | `@cf/meta/llama-4-scout-17b-16e-instruct`<br>`@cf/meta/llama-3.3-70b-instruct-fp8-fast`<br>`@cf/qwen/qwen3-30b-a3b`<br>`@cf/openai/whisper-large-v3-turbo` | Edge-distributed serverless GPU inference with AI Gateway caching and rate limiting |
| **GitHub Models** | `https://models.github.ai/inference` | `POST /chat/completions`<br>`POST /embeddings`<br>`GET /catalog/models` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $GITHUB_TOKEN` | `openai/gpt-6.2-sol`<br>`anthropic/claude-sonnet-5-5`<br>`deepseek/deepseek-v4-pro`<br>`meta/llama-4-maverick`<br>`mistral-ai/mistral-large-4` | Pass-through `reasoning_effort` and standard chat parameters backed by Azure AI Foundry |

---

### 1.2 Google Gemini API Version Taxonomy

```
https://generativelanguage.googleapis.com/{API_VERSION}/models/{MODEL_ID}:{METHOD}
```

* **`v1` (Stable GA):** Strict Long-Term Support (LTS). High backwards compatibility for simple text, function calling, and structured JSON outputs. Excludes experimental thinking configs and live bidi channels.
* **`v1alpha` (Bleeding Edge Alpha):** Internal developer releases, low-level tokenizer introspection, prototype multimodal bindings, dynamic runtime memory flags.
* **`v1beta` (Public Developer Preview):** Standard for advanced agents. Supports full `thinkingConfig`, groundings with Google Search, code execution sandboxes, Context Caching, and Bidi WebSocket Streaming.
* **`v1beta1` (Enterprise & GenAI SDK Channel):** Targeted channel used by official Google Gen AI SDKs and Vertex bridges. Implements `continuationToken` for extended multi-step generations.
* **`v1beta2` (Legacy PaLM Transition):** Legacy endpoint compatibility layer for historical PaLM 2 models (`chat-bison-001`, `text-bison-001`).
* **`v1beta3` (Multi-Agent & Tooling Sandbox):** Experimental channel for asynchronous batch tasks, multi-agent evaluation runtimes, and specialized sandboxed code evaluation.
* **`v1main` (Internal Monorepo HEAD):** Real-time mapping to internal Google development trunk. Accessible by internal Google services and whitelisted enterprise projects.

---

### 1.3 Cross-Provider Wire Protocol Comparison

| Capability | OpenAI Chat Completions (`/v1/chat/completions`) | OpenAI Responses API (`/v1/responses`) | Anthropic Messages API (`/v1/messages`) | Google Gemini REST (`:generateContent`) | AWS Bedrock Converse (`/converse`) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **System Prompt** | `messages: [{"role": "system", ...}]` | Top-level `instructions` string | Top-level `system` string / block array | Top-level `systemInstruction` object | Top-level `system: [{"text": ...}]` |
| **Streaming Flag** | `"stream": true` (SSE `data: {...}`) | `"stream": true` (Typed SSE events) | `"stream": true` (Named SSE events) | `:streamGenerateContent?alt=sse` | `/converse-stream` (Binary EventStream) |
| **Tool Definitions** | `tools: [{"type": "function", "function": {...}}]` | `tools: [{"type": "function", ...}]` + built-in tools | `tools: [{"name": ..., "input_schema": {...}}]` | `tools: [{"functionDeclarations": [...]}]` | `toolConfig: {"tools": [{"toolSpec": {...}}]}` |
| **Structured Output** | `response_format: {"type": "json_schema", ...}` | `text: {"format": {"type": "json_schema", ...}}` | Forced `tool_choice` or prompt prefill | `generationConfig.responseSchema` | Tool use schema enforcement |
| **Vision / Media** | `content: [{"type": "image_url", ...}]` | `input: [{"type": "input_image", ...}]` | `content: [{"type": "image", "source": {...}}]` | `parts: [{"inlineData": {...}}]` or `fileData` | `content: [{"image": {"format": ..., "source": ...}}]` |

---

### 1.4 Sampling Parameters Support Matrix (`temperature`, `top_p`, `top_k`)

#### Cloud Providers Matrix

| Provider | `temperature` | `top_p` | `top_k` | Default (`temp` / `top_p` / `top_k`) | Ranges, Wire Naming & Restrictions |
| :--- | :---: | :---: | :---: | :--- | :--- |
| **Google Gemini API (AI Studio)** | Yes | Yes | Yes | `1.0` / `0.95` / `40` *(or `64` on Gemini 2.5/3/4)* | `generationConfig`: `temperature` (0.0--2.0), `topP` (0.0--1.0), `topK` (int $\ge 1$, model-dependent/fixed on some Gemini checkpoints). |
| **Google Cloud Vertex AI** | Yes | Yes | Yes | `1.0` / `0.95` / `40`--`64` *(Gemini)*; `1.0` / unset / unset *(Claude)* | `generationConfig` (`temperature`, `topP`, `topK`) for Gemini; Anthropic native rules apply for Claude on Vertex. |
| **OpenAI** | Yes* | Yes* | **No** | `1.0` / `1.0` / `N/A` | `temperature` (0.0--2.0), `top_p` (0.0--1.0). `top_k` rejected (HTTP 400). *Disallowed/rejected on reasoning models (`o1`, `o3`, `o4-mini`, and GPT-5/GPT-6 when reasoning is active). |
| **Anthropic** | Yes* | Yes* | Yes* | `1.0` / unset / unset | `temperature` (0.0--1.0), `top_p` (0.0--1.0), `top_k` ($\ge 1$). *Newer Claude models disallow setting both `temperature` and `top_p` together. With Extended Thinking enabled: `temperature` must be `1.0` (or omitted), `top_k` is disallowed, and `top_p` is restricted/disallowed. |
| **AWS Bedrock** | Yes | Yes | Partial | Model-dependent *(e.g., Nova `0.7`/`0.9`/`N/A`; Claude `1.0`/unset/unset; Llama `0.5`/`0.9`/`N/A`)* | Converse `inferenceConfig` supports `temperature` and `topP`. `top_k` is not in `inferenceConfig`; must be passed via `additionalModelRequestFields` (allowed for Claude, Cohere, Mistral, Llama, Nova; subject to model rules). |
| **Azure OpenAI / Azure AI Foundry** | Yes* | Yes* | **No*** | `1.0` / `1.0` / `N/A` *(OpenAI)*; publisher default *(Foundry)* | Follows OpenAI schema (`temperature`, `top_p`; no `top_k`). *Same OpenAI reasoning-model restrictions; non-OpenAI Foundry models (e.g., Claude) follow their publisher schema. |
| **xAI** | Yes | Yes | **No** | `1.0` / `1.0` / `N/A` | `temperature` (0.0--2.0), `top_p` (0.0--1.0) on `/v1/chat/completions`. `top_k` not supported on OpenAI-compat route. Grok reasoning models still accept `temperature` and `top_p`. |
| **DeepSeek API** | Yes* | Yes* | **No** | `1.0` / `1.0` / `N/A` | `temperature` (0.0--2.0), `top_p` (0.0--1.0). `top_k` not supported. *Silently ignored (no-op) on `deepseek-reasoner` (R1) and V4 Thinking Mode. |
| **Groq Cloud** | Yes | Yes | **No** | `1.0` / `1.0` / `N/A` | `temperature` (0.0--2.0), `top_p` (0.0--1.0). `top_k` not supported on OpenAI-compatible endpoint. |
| **Cerebras Cloud** | Yes | Yes | **No** | `1.0` / `1.0` / `N/A` | `temperature` (0.0--1.5), `top_p` (0.0--1.0). `top_k` not supported. |
| **Fireworks AI** | Yes | Yes | Yes | `1.0` *(or `0.6` on some chat models)* / `1.0` / `0` *(disabled)* | `temperature` (0.0--2.0), `top_p` (0.0--1.0), `top_k` (0--100). |
| **SambaNova Cloud** | Yes | Yes | Yes | `0.0` / `1.0` / `50` *(or unset)* | `temperature` (0.0--1.0/2.0), `top_p` (0.0--1.0), `top_k` (1--100). |
| **Alibaba Cloud Model Studio / DashScope (Qwen)** | Yes | Yes | Yes | `0.7`--`1.0` / `0.8` *(or `0.95` thinking)* / `20` *(or unset)* | `temperature` (0.0--2.0), `top_p` (0.0--1.0); `top_k` supported natively and via `extra_body` in OpenAI-compatible mode. |
| **Together AI** | Yes | Yes | Yes | `0.7` *(or `1.0` model-dependent)* / `0.7`--`1.0` / `50` *(or unset)* | `temperature` (0.0--2.0), `top_p` (0.0--1.0), `top_k` (int). |
| **Mistral AI** | Yes | Yes | **No** | `0.7` / `1.0` / `N/A` | `temperature` (0.0--1.5), `top_p` (0.0--1.0). `top_k` not supported. |
| **Cohere** | Yes | Yes | Yes | `0.3` / `0.75` / `0` *(disabled)* | Native `/v2/chat` uses `temperature` (0.0--1.0), `p` (0.01--0.99), `k` (0--500). OpenAI-compat route exposes `temperature` and `top_p`. |
| **Perplexity AI** | Yes | Yes | Yes | `0.2` / `0.9` / `0` *(disabled)* | `temperature` (0.0--2.0), `top_p` (0.0--1.0), `top_k` (0--2048). |
| **NVIDIA NIM API** | Yes | Yes | Partial | Model-dependent *(typically `0.2`--`1.0`)* / `1.0` / `-1` *(disabled)* | `temperature` and `top_p` supported; `top_k` available via `extra_body` on TensorRT-LLM / vLLM-backed NIMs, not all endpoints. |
| **Hugging Face Inference API (HF Router)** | Yes | Yes | Partial | `1.0` *(or `generation_config.json`)* / `1.0` / unset | `temperature` and `top_p` standard; `top_k` depends on routed backend (supported on TGI/vLLM/SGLang via `extra_body`, rejected by some third-party providers). |
| **Moonshot AI (Kimi)** | Yes* | Yes | **No** | `0.3` *(non-thinking; `0.6`--`1.0` thinking)* / `1.0` / `N/A` | `temperature` (0.0--1.0), `top_p` (0.0--1.0). `top_k` not supported. *Thinking modes on Kimi K2.5/K3 fix or restrict `temperature` (e.g., locked to `1.0` in thinking mode). |
| **Zhipu AI / Z.ai (GLM)** | Yes | Yes | **No** | `0.95` *(or `1.0` on GLM-5.x)* / `0.7` *(or `0.95` on GLM-5.x)* / `N/A` | `temperature` (0.0--1.0), `top_p` (0.0--1.0). `top_k` not supported. |
| **MiniMax** | Yes* | Yes | **No** | `1.0` / `0.95` / `N/A` | `temperature` (`(0.0, 1.0]` --- strictly `> 0.0`, `0.0` is rejected), `top_p` (0.0--1.0). `top_k` not supported on OpenAI-compat chat. |
| **ByteDance Seed (Volcano Engine / Ark)** | Yes | Yes | **No** | `1.0` / `0.7` *(or `0.95` model-dependent)* / `N/A` | `temperature` (0.0--2.0), `top_p` (0.0--1.0). `top_k` not supported on standard OpenAI-compat endpoint. |
| **OpenRouter** | Yes | Yes | Pass-through | `1.0` *(or upstream model default)* / `1.0` / `0` *(disabled)* | Accepts `temperature`, `top_p`, and `top_k`; forwards only if the upstream provider/model supports them (drops/rejects otherwise). |
| **DeepInfra** | Yes | Yes | Yes | `1.0` / `1.0` / `0` *(disabled)* | `temperature` (0.0--2.0), `top_p` (0.0--1.0), `top_k` (int). |
| **Replicate** | Yes* | Yes* | Yes* | Model-dependent *(commonly `0.7`--`1.0` / `0.9`--`0.95` / `50`)* | *Model-dependent `input` schema; most open-weight text models expose `temperature`, `top_p`, and `top_k`. |
| **Cloudflare Workers AI** | Yes | Yes | Yes | `0.6` / `1.0` *(or `0.9`)* / unset | `temperature` (0.0--5.0), `top_p` (0.0--2.0), `top_k` (1--50) on text generation models. |
| **GitHub Models** | Yes* | Yes* | **No** | `1.0` / `1.0` / `N/A` | `temperature`, `top_p` supported (*subject to underlying model rules, e.g., OpenAI reasoning models reject both). `top_k` not supported. |

#### Model-Level Restrictions (Disallowed, Locked, or Ignored)

* **OpenAI Reasoning Models (`o1`, `o3`, `o3-mini`, `o4-mini`, GPT-5 / GPT-6 with reasoning active):** Reject `temperature` and `top_p` (HTTP 400) unless reasoning is explicitly disabled (`reasoning_effort: "none"`) on models that support non-reasoning mode. `top_k` is never supported.
* **Anthropic Claude Extended Thinking (`thinking.type = "enabled"`):** `temperature` is locked to `1.0` (custom values rejected), `top_k` is disallowed, and `top_p` is restricted or disallowed. On Claude 4.x / 5.x models, passing both `temperature` and `top_p` in the same request is rejected even without thinking.
* **DeepSeek Thinking Models (`deepseek-reasoner` / R1, V4 Thinking Mode):** `temperature` and `top_p` are accepted without error but silently ignored (no-op). `top_k` is unsupported.
* **Moonshot Kimi Thinking Mode (`kimi-k2.5` / `kimi-k3`):** Fixes or constrains `temperature` when thinking mode is enabled (e.g., locked to `1.0`).
* **MiniMax Text Models:** Reject `temperature: 0.0` (must be strictly `> 0.0`, e.g., `0.01`).
* **Non-Text / Modality-Specific Models (All Providers):** Embedding (`text-embedding-3-*`, `embed-v4.0`), reranking (`rerank-*`), ASR/TTS (`whisper-*`, `speech-*`), and image/video diffusion models (`gpt-image-*`, `seedance-*`, `seedream-*`, `grok-imagine-*`, `flux-*`) do not use `top_p` or `top_k` (and most do not use `temperature`, except ASR decoding temperature on Whisper).

---

## 2. Local Inference Engines, Workstations, Daemons & Sockets

### 2.1 Quick Socket, Port & API Route Reference Matrix

| Engine / Platform | Default Host & Port | Native API Endpoints | OpenAI-Compatible Routes | Primary Focus & Model Format |
| :--- | :--- | :--- | :--- | :--- |
| **Ollama** | `http://localhost:11434` | `POST /api/generate`, `POST /api/chat`, `POST /api/embed`, `POST /api/pull`, `GET /api/tags`, `GET /api/ps` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | Universal CLI / daemon; GGUF |
| **llama.cpp (`llama-server`)** | `http://localhost:8080` | `POST /completion`, `POST /infill`, `POST /tokenize`, `POST /detokenize`, `GET /slots`, `GET /props`, `GET /health` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | Bare-metal C/C++; GGUF |
| **LM Studio** | `http://localhost:1234` | `POST /api/v0/chat/completions` (extended telemetry stats) | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | Desktop workstation GUI; GGUF, MLX |
| **Jan AI** | `http://localhost:1337` | `POST /nitro/inferences`, `GET /v1/threads`, `GET /v1/assistants` | `/v1/chat/completions`, `/v1/embeddings`, `/v1/models` | Privacy-first desktop client; GGUF |
| **GPT4All** | `http://localhost:4891` | None | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | CPU-optimized local client; GGUF |
| **Msty AI** | `http://localhost:10240` | None | `/v1/chat/completions`, `/v1/models` | Multi-model side-by-side workstation; GGUF |
| **Kobold.cpp** | `http://localhost:5001` | `POST /api/v1/generate`, `POST /api/extra/tokencount`, `POST /api/extra/abort`, `GET /api/v1/model` | `/v1/chat/completions`, `/v1/completions`, `/v1/models` | Roleplay & creative writing; GGUF |
| **llamafile (Mozilla)** | `http://localhost:8080` | `POST /completion`, `POST /tokenize`, `GET /health` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings` | Polyglot single executable; GGUF |
| **LocalAI** | `http://localhost:8080` | `POST /models/apply`, `GET /system` | `/v1/chat/completions`, `/v1/images/generations`, `/v1/audio/speech`, `/v1/audio/transcriptions` | Multi-modal containerized proxy; Multi |
| **vLLM** | `http://localhost:8000` | `POST /tokenize`, `POST /detokenize`, `GET /metrics`, `GET /health`, `GET /version` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | High-throughput production; Safetensors, AWQ, FP8 |
| **SGLang** | `http://localhost:30000`| `POST /generate`, `POST /flush_cache`, `GET /get_model_info`, `GET /health` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | RadixAttention for fast agents/JSON; Safetensors |
| **TGI (Hugging Face)** | `http://localhost:8080` | `POST /generate`, `POST /generate_stream`, `GET /info`, `GET /metrics`, `GET /health` | `/v1/chat/completions`, `/v1/completions`, `/v1/models` | Production container; Safetensors, AWQ, GPTQ |
| **LMDeploy** | `http://localhost:23333`| `POST /v1/chat/interactive` | `/v1/chat/completions`, `/v1/completions`, `/v1/models` | TurboMind C++ backend; W4A16, KV 4-bit |
| **TabbyAPI (ExLlamaV2)**| `http://localhost:5000` | `POST /v1/model/load`, `POST /v1/model/unload`, `GET /v1/model/list` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | Ultra-fast token gen on NVIDIA; EXL2, GPTQ |
| **MLC-LLM** | `http://localhost:8000` | `POST /chat/completions` | `/v1/chat/completions`, `/v1/completions`, `/v1/models` | Cross-platform TVM compiler; Vulkan, WebGPU |
| **Exo** | `http://localhost:52415`| `GET /state`, `POST /place_instance` | `/v1/chat/completions`, `/v1/models` | Distributed peer-to-peer cluster; GGUF, MLX |
| **FastChat** | `http://localhost:8000` | `POST /worker_generate_stream`, `GET /list_models` (Controller `:21001`) | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | Distributed multi-worker serving; Hugging Face |
| **Triton + TensorRT-LLM**| `http://localhost:8000` | `POST /v2/models/{model}/infer`, `POST /v2/models/{model}/generate_stream`, `GET /v2/health/ready` | `/v1/chat/completions` (via Triton OpenAI frontend) | Maximum throughput on NVIDIA Tensor Cores |
| **Text-Generation-WebUI**| `http://localhost:5000` | `POST /api/v1/internal/model/load`, `POST /api/v1/internal/encode` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | Modular testing & fine-tuning workbench |
| **AnythingLLM** | `http://localhost:3001` | `POST /api/v1/workspace/{slug}/chat`, `POST /api/v1/document/upload` | `/api/v1/openai/chat/completions`, `/api/v1/openai/models` | Document ingestion & built-in RAG pipelines |
| **LiteLLM Proxy** | `http://localhost:4000` | `POST /key/generate`, `GET /spend/logs`, `GET /health` | `/v1/chat/completions`, `/v1/embeddings`, `/v1/models`, `/v1/messages` | Load balancing, rate limiting, and unified routing |
| **MLX-LM (Apple Silicon)** | `http://localhost:8080` | None | `/v1/chat/completions`, `/v1/completions`, `/v1/models` | Apple Metal unified memory; Safetensors 4/8-bit |
| **Aphrodite Engine** | `http://localhost:2242` | `GET /v1/models`, `GET /health`, `GET /metrics` | `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models` | PagedAttention serving with creative sampling |

---

### 2.2 CLI Execution & In-Chat Commands for Local Engines

#### 1. Ollama
* **Nature:** Standalone background daemon and CLI orchestrator wrapping `llama.cpp` for automated pulling, running, and quantization.
* **Native API Endpoints:** `POST http://localhost:11434/api/chat`, `POST /api/generate`, `POST /api/embed`, `GET /api/tags`, `GET /api/ps`.
* **CLI Startup & Lifecycle:**
  ```bash
  ollama serve                                  # Background daemon
  ollama run llama3.3:70b-instruct-q4_K_M       # Interactive terminal session
  ollama list                                   # List local weights
  ollama ps                                     # List models active in VRAM
  ollama pull qwen2.5-coder:32b                 # Pull model tag
  ollama rm codellama:7b                        # Delete model
  ollama cp llama3.2 my-custom                  # Copy/alias tag
  ollama create custom-agent -f ./Modelfile     # Build custom model
  ```
* **Interactive In-Chat Slash (`/`) Commands:**
  * `/set parameter <key> <val>`: Configure session parameters (`temperature`, `num_ctx 32768`, `top_p`).
  * `/set system <prompt>`: Dynamically set or replace system prompt.
  * `/show info`: Display active model architecture, quantization, context size, and stop tokens.
  * `/show license`: Print model license details.
  * `/load <model>`: Switch model in active REPL without quitting.
  * `/clear`: Clear conversation history and reset context memory.
  * `/bye` or `/exit`: Safely terminate session.
  * `/?` or `/help`: Display operational manual.
  * `"""`: Start and terminate multi-line block text input.

#### 2. llama.cpp (`llama-server` and `llama-cli`)
* **Nature:** Bare-metal C/C++ inference implementation with minimal runtime overhead.
* **API Endpoints:** `POST http://localhost:8080/v1/chat/completions`, `POST /completion`, `POST /infill`, `GET /slots`, `GET /metrics`.
* **CLI Invocations:**
  ```bash
  ./llama-server \
    -m ./models/qwen2.5-coder-32b-instruct.Q4_K_M.gguf \
    --host 0.0.0.0 \
    --port 8080 \
    -c 32768 \
    -ngl 99 \
    --cont-batching \
    --parallel 4 \
    --cache-type-k q8_0 \
    --cache-type-v q4_0 \
    --metrics

  ./llama-cli -m model.gguf -p "Prompt" -n 2048 -ngl 33 -i -r "User:"
  ```
* **Interactive CLI Escapes & Control Directives:**
  * `Ctrl+C`: Interrupt generation stream while preserving prior turn context.
  * `User:`: Standard reverse prompt delimiter triggering input acquisition.
  * `/reset`: Reset continuous KV cache buffer.
  * `/dump`: Output KV cache slot status and token occupancy metrics to stderr.

#### 3. LM Studio
* **Nature:** Desktop GUI workstation with local server and CLI (`lms`).
* **Socket & APIs:** `http://localhost:1234/v1/chat/completions`, `POST /api/v0/chat/completions`.
* **CLI Management Commands (`lms`):**
  ```bash
  lms server start --port 1234 --cors=true
  lms server stop
  lms server status
  lms load lmstudio-community/Meta-Llama-3.1-8B-Instruct-GGUF --gpu max --context-length 16384
  lms unload --all
  lms ls                                        # List cached models on disk
  lms ps                                        # List active models in VRAM
  lms bootstrap                                 # Install PATH environment wrappers
  ```
* **In-Chat & Workstation Controls:**
  * System Prompt Override: Dynamic slot injection in the sidebar.
  * `@model-id`: Multi-model chat routing to compare completions side-by-side.
  * Dynamic Context Slider: Re-allocates KV cache size on demand.

#### 4. Jan AI
* **Nature:** 100% open-source local-first desktop client powered by the Nitro engine.
* **Socket & APIs:** `http://localhost:1337/v1/chat/completions`, `POST /nitro/inferences`, `GET /v1/threads`.
* **Controls:** Toggle `Enable Local API Server` in Advanced Settings; built-in Model Context Protocol (MCP) server manager.

#### 5. GPT4All (Nomic AI)
* **Nature:** CPU-friendly desktop client and local inference engine.
* **Socket & APIs:** `http://localhost:4891/v1/chat/completions`, `POST /v1/embeddings`, `GET /v1/models`.
* **Controls:** Enable `Local API Server` in Application Settings; integrates local document collections via `LocalDocs`.

#### 6. Msty AI
* **Nature:** Multi-model desktop workstation with side-by-side prompt benchmarking.
* **Socket & APIs:** `http://localhost:10240/v1/chat/completions`, `GET /v1/models`.
* **Controls:** Toggle `Enable Local Server` in settings; built-in local knowledge stacks.

#### 7. Kobold.cpp
* **Nature:** Specialized C/C++ runtime based on `llama.cpp` for creative writing, long contexts, and roleplay.
* **Socket & APIs:** `http://localhost:5001/api/v1/generate`, `POST /v1/chat/completions`, `POST /api/extra/abort`.
* **CLI Startup:**
  ```bash
  python koboldcpp.py --model model.gguf --port 5001 --usecublas --gpulayers 99 --contextsize 16384 --smartcontext
  ```
* **Interactive In-Chat Directives:**
  * `::: memory <text> :::`: Injects permanent memory block into the top of the context buffer.
  * `::: authornote <text> :::`: Injects guidance notes $N$ tokens from the bottom of prompt.
  * `/abort`: Instantly cancels in-flight continuous generation.
  * `/undo`: Reverts the most recent interaction turn.
  * `/redo`: Regenerates the last assistant completion with updated seed.

#### 8. llamafile (Mozilla)
* **Nature:** Polyglot multi-platform single-file executable combining weights and `llama.cpp` server.
* **Socket & APIs:** `http://localhost:8080/v1/chat/completions`, `POST /completion`.
* **CLI Startup:**
  ```bash
  chmod +x model.llamafile
  ./model.llamafile --server --nobrowser --port 8080
  ```

#### 9. LocalAI
* **Nature:** Containerized drop-in OpenAI API replacement across text, audio, and vision.
* **Socket & APIs:** `http://localhost:8080/v1/chat/completions`, `/v1/images/generations`, `/v1/audio/speech`, `/v1/audio/transcriptions`.
* **CLI / Container Commands:**
  ```bash
  docker run -p 8080:8080 --gpus all -v $PWD/models:/build/models localai/localai:latest-aio-gpu-nvidia-cuda-12
  local-ai run hermes-2-pro-llama-3-8b
  ```

#### 10. vLLM
* **Nature:** Production-grade high-throughput serving engine utilizing PagedAttention.
* **Socket & APIs:** `http://localhost:8000/v1/chat/completions`, `POST /v1/embeddings`, `POST /tokenize`, `GET /health`, `GET /metrics`.
* **CLI Startup:**
  ```bash
  vllm serve meta-llama/Llama-3.3-70B-Instruct \
    --host 0.0.0.0 \
    --port 8000 \
    --tensor-parallel-size 4 \
    --gpu-memory-utilization 0.95 \
    --max-model-len 32768 \
    --quantization fp8 \
    --kv-cache-dtype fp8 \
    --enable-prefix-caching \
    --enable-chunked-prefill
  ```

#### 11. SGLang
* **Nature:** High-performance execution framework featuring RadixAttention for automatic KV cache reuse across multi-turn agent loops.
* **Socket & APIs:** `http://localhost:30000/generate`, `POST /v1/chat/completions`, `POST /flush_cache`, `GET /get_model_info`.
* **CLI Startup:**
  ```bash
  python3 -m sglang.launch_server \
    --model-path mistralai/Mistral-Small-24B-Instruct-2501 \
    --host 0.0.0.0 \
    --port 30000 \
    --tp 2 \
    --enable-radix-attention \
    --grammar-backend xgrammar
  ```

#### 12. Hugging Face TGI (Text Generation Inference)
* **Nature:** Rust, Python, and gRPC production container for Hugging Face Hub deployments.
* **Socket & APIs:** `http://localhost:8080/generate`, `POST /generate_stream`, `POST /v1/chat/completions`, `GET /info`, `GET /metrics`.
* **Launch Command:**
  ```bash
  docker run --gpus all --shm-size 1g -p 8080:80 \
    ghcr.io/huggingface/text-generation-inference:latest \
    --model-id deepseek-ai/DeepSeek-R1-Distill-Llama-70B --num-shard 4
  ```

#### 13. LMDeploy
* **Nature:** TurboMind C++ backend with 4-bit KV cache quantization.
* **Socket & APIs:** `http://localhost:23333/v1/chat/completions`, `POST /v1/chat/interactive`.
* **Launch Command:**
  ```bash
  lmdeploy serve api_server internlm/internlm2_5-20b-chat --server-port 23333 --tp 2 --quant-policy 4
  ```

#### 14. TabbyAPI (ExLlamaV2)
* **Nature:** Low-latency OpenAI-compatible REST server optimized for EXL2 format on NVIDIA GPUs.
* **Socket & APIs:** `http://localhost:5000/v1/chat/completions`, `POST /v1/model/load`, `POST /v1/model/unload`.
* **Launch Command:**
  ```bash
  python main.py --host 0.0.0.0 --port 5000 --model-name Qwen2.5-Coder-32B-exl2-4.0bpw --max-seq-len 32768
  ```

#### 15. MLC-LLM
* **Nature:** Compiler-accelerated runtime engine based on Apache TVM across Vulkan, Metal, CUDA, and WebGPU.
* **Socket & APIs:** `http://localhost:8000/v1/chat/completions`, `POST /chat/completions`.
* **Launch Command:**
  ```bash
  mlc_llm serve HF://mlc-ai/Llama-3-8B-Instruct-q4f16_1-MLC --port 8000
  ```

#### 16. Exo
* **Nature:** Decentralized peer-to-peer compute orchestrator that pools consumer hardware into a single GPU cluster.
* **Socket & APIs:** `http://localhost:52415/v1/chat/completions`, `GET /state`.
* **Launch Command:**
  ```bash
  exo --broadcast-port 52415
  exo --peer-ip 192.168.1.50
  ```

#### 17. FastChat (LMSYS)
* **Nature:** Distributed multi-node model serving and evaluation platform powering LMSYS Chatbot Arena.
* **Sockets & APIs:** Controller on `http://localhost:21001` (`POST /worker_generate_stream`), OpenAI Server on `http://localhost:8000/v1/chat/completions`.

#### 18. NVIDIA Triton Inference Server + TensorRT-LLM
* **Nature:** Enterprise model serving infrastructure paired with TensorRT-LLM kernels.
* **Sockets & APIs:** HTTP `http://localhost:8000/v2/models/{model}/infer` (and `/v1/chat/completions`), gRPC `8001` (`GRPCInferenceService`), Metrics `8002`.

#### 19. Text-Generation-WebUI (Oobabooga)
* **Nature:** Modular testing, benchmarking, and fine-tuning workbench.
* **Sockets & APIs:** Web UI on `http://localhost:7860`, OpenAI API on `http://localhost:5000/v1/chat/completions`, Internal API `/api/v1/internal/model/load`.
* **Launch Command:** `python server.py --api --api-port 5000 --listen`

#### 20. AnythingLLM
* **Nature:** Local document ingestion workspace with integrated vector storage and OpenAI-compatible proxy.
* **Socket & APIs:** `http://localhost:3001/api/v1/openai/chat/completions`, `POST /api/v1/workspace/{slug}/chat`, `POST /api/v1/document/upload`.

#### 21. MLX-LM (Apple Silicon)
* **Nature:** Native inference engine engineered for Apple Silicon unified memory using Apple MLX.
* **Socket & APIs:** `http://localhost:8080/v1/chat/completions`, `POST /v1/completions`, `GET /v1/models`.
* **Launch Command:**
  ```bash
  python -m mlx_lm.server --model mlx-community/Qwen2.5-Coder-32B-Instruct-4bit --port 8080
  ```

#### 22. Aphrodite Engine
* **Nature:** High-performance serving engine based on PagedAttention with support for novel samplers (Mirostat, Dry, Min-P).
* **Socket & APIs:** `http://localhost:2242/v1/chat/completions`, `POST /v1/completions`, `GET /health`, `GET /metrics`.
* **Launch Command:**
  ```bash
  aphrodite run meta-llama/Meta-Llama-3.1-8B-Instruct --port 2242
  ```

---

### 2.3 Local Engine Selection Matrix

```
                                  Your Primary Goal
                                          |
         +--------------------------------+--------------------------------+
         v                                v                                v
[Desktop Workspace]             [Developer Daemon]              [Production Concurrency]
  - LM Studio (GUI / lms)         - Ollama (CLI / Modelfile)      - vLLM (PagedAttention)
  - Jan AI (Open-Source / MCP)    - llama-server (Bare-metal)     - SGLang (RadixAttention / JSON)
  - GPT4All (CPU / LocalDocs)     - TabbyAPI (ExLlamaV2 / EXL2)   - TGI (Hugging Face ecosystem)
  - Msty AI (Side-by-side eval)   - MLX-LM (Apple Silicon)        - Triton + TRT-LLM (Enterprise)
                                  - Exo (Distributed cluster)     - Aphrodite Engine (Creative/Batch)
```

---

### 2.4 Sampling Parameters Support Matrix (`temperature`, `top_p`, `top_k`)

> **OpenAI-Compatible Route Note:** The standard OpenAI `/v1/chat/completions` schema only defines `temperature` and `top_p`. Local engines that support `top_k` expose it either on their native endpoint, as an accepted extra top-level JSON field, or via `extra_body: {"top_k": <int>}` in OpenAI SDK clients.

| Engine / Platform | `temperature` | `top_p` | `top_k` | Default (`temp` / `top_p` / `top_k`) | Exposure & Engine Notes |
| :--- | :---: | :---: | :---: | :--- | :--- |
| **Ollama** | Yes | Yes | Native only | `0.8` / `0.9` / `40` *(overridden by `Modelfile`)* | All 3 supported in native `/api/chat`, `/api/generate` (`options`), and `Modelfile` (`PARAMETER`). OpenAI `/v1/chat/completions` supports `temperature` and `top_p` (`top_k` ignored on `/v1`). |
| **llama.cpp (`llama-server`)** | Yes | Yes | Yes | `0.8` / `0.95` / `40` | All 3 supported on `/completion` and `/v1/chat/completions` (plus `min_p`, `typical_p`, `mirostat`). |
| **LM Studio** | Yes | Yes | Yes | `0.8` / `0.95` / `40` *(preset/model-dependent)* | `temperature` and `top_p` on `/v1/chat/completions`; `top_k` via GUI, native `/api/v0/chat/completions`, and `extra_body`. |
| **Jan AI** | Yes | Yes | Native / UI | `0.7` / `0.95` / `40` | `temperature` and `top_p` on `/v1/chat/completions`; `top_k` configurable in engine/Nitro settings. |
| **GPT4All** | Yes | Yes | Native / UI | `0.7` / `0.4` / `40` | `temperature` and `top_p` on `/v1/chat/completions`; `top_k` exposed in native SDK/GUI only. |
| **Msty AI** | Yes | Yes | UI only | `0.8` / `0.9` / `40` | `temperature` and `top_p` on `/v1/chat/completions`; `top_k` set via workstation model parameters. |
| **Kobold.cpp** | Yes | Yes | Yes | `0.7` / `0.92` / `0` *(disabled)* | All 3 supported on native `/api/v1/generate` and `/v1/chat/completions` (full creative sampler suite). |
| **llamafile (Mozilla)** | Yes | Yes | Yes | `0.8` / `0.95` / `40` | Inherits `llama.cpp` server; all 3 supported. |
| **LocalAI** | Yes | Yes | Yes | Backend-dependent *(`llama.cpp`: `0.8`/`0.95`/`40`; `vllm`: `1.0`/`1.0`/`-1`)* | All 3 supported and forwarded to the active backend (`llama.cpp`, `vllm`, `transformers`, etc.). |
| **vLLM** | Yes | Yes | Yes (`extra_body`) | `1.0` / `1.0` / `-1` *(disabled; or `generation_config.json`)* | `temperature` and `top_p` top-level; `top_k` supported via `extra_body` (`-1` disables `top_k`). |
| **SGLang** | Yes | Yes | Yes (`extra_body`) | `1.0` / `1.0` / `-1` *(disabled)* | All 3 supported in native `/generate` (`sampling_params`) and via `extra_body` on `/v1/chat/completions`. |
| **TGI (Hugging Face)** | Yes* | Yes | Yes | `1.0` *(greedy if `do_sample=false`)* / `1.0` / unset | All 3 supported on `/generate` (*`temperature` must be `> 0` when sampling); `temperature` and `top_p` on `/v1/chat/completions`. |
| **LMDeploy** | Yes | Yes | Yes (`extra_body`) | `0.8` *(native)* or `1.0` *(OpenAI)* / `1.0` / `50` *(native)* or unset | All 3 supported (`top_k` via `extra_body` on `/v1/chat/completions`). |
| **TabbyAPI (ExLlamaV2)** | Yes | Yes | Yes | `1.0` / `1.0` / `0` *(disabled)* | All 3 supported directly on `/v1/chat/completions` (plus `min_p`, `tfs`, `typical`). |
| **MLC-LLM** | Yes | Yes | **No** | `1.0` *(or `mlc-chat-config.json`)* / `1.0` / `N/A` | `temperature` and `top_p` supported; `top_k` not exposed in standard `/v1/chat/completions`. |
| **Exo** | Yes | Limited | **No** | `0.0` *(or `0.7` model-dependent)* / `1.0` / `N/A` | `temperature` supported; `top_p` and `top_k` not consistently exposed across distributed MLX/tinygrad backends. |
| **FastChat** | Yes | Yes | Partial | `0.7` / `1.0` / `-1` *(disabled)* | `temperature` and `top_p` supported; `top_k` depends on worker backend (`vllm`/`transformers`) and is dropped by the default OpenAI API server. |
| **Triton + TensorRT-LLM** | Yes | Yes | Yes | `1.0` / `0.0` *(disabled)* / `1` *(greedy)* or `0` *(disabled)* | All 3 supported natively (`/v2/models/{model}/infer`, `generate_stream`) and via `extra_body` on the OpenAI frontend. |
| **Text-Generation-WebUI** | Yes | Yes | Yes | `0.7` *(UI)* or `1.0` *(API)* / `0.9` or `1.0` / `20` or `0` | All 3 supported on `/v1/chat/completions` and Web UI. |
| **AnythingLLM** | Yes | **No** | **No** | `0.7` / Upstream default / Upstream default | Exposes `temperature` per workspace; `top_p` and `top_k` are not exposed on workspace chat endpoints (delegated to upstream LLM defaults). |
| **LiteLLM Proxy** | Yes | Yes | Pass-through | Omitted *(defers to target provider/model default)* | Accepts all 3 and forwards or strips (`drop_params=True`) according to the target provider/model. |
| **MLX-LM (Apple Silicon)** | Yes | Yes | Partial | `0.0` *(greedy)* / `1.0` / `0` *(disabled)* | `temperature` and `top_p` on `/v1/chat/completions`; `top_k` supported in CLI and newer `mlx_lm.server` builds. |
| **Aphrodite Engine** | Yes | Yes | Yes | `1.0` / `1.0` / `-1` *(disabled)* | All 3 supported (plus `min_p`, `mirostat`, `dry`, `xtc`). |

---

## 3. Agentic CLI & Bridge Providers

### 3.1 Exhaustive Catalog with APIs, Invocations, Flags, and In-Chat Slash Commands

#### 1. Claude Code (Anthropic)
* **Architecture:** Node.js native agentic CLI orchestrator executing bash commands, ripgrep queries, tree-sitter AST queries, and sub-agent task decomposition.
* **Upstream APIs & SDKs:** Anthropic Messages API (`POST /v1/messages`), AWS Bedrock (`InvokeModelWithResponseStream`), Google Vertex AI (`:streamRawPredict`), MCP JSON-RPC (`stdio` / `sse`), and `@anthropic-ai/claude-agent-sdk` (programmatic headless execution).
* **CLI Invocations:**
  ```bash
  claude                                        # Interactive REPL in repository
  claude -p "Implement exponential backoff in internal/transport/http.go and run tests"
  claude --headless -p "Analyze current branch changes and write a PR summary"
  claude --output-format stream-json -p "Audit auth.go"
  claude config list
  claude config set verbose true
  claude config set model claude-3-7-sonnet-20250219
  claude update                                 # Update CLI binary
  ```
* **In-Chat Slash (`/`) Commands & In-Session Modifiers:**
  * `/help`: Display operational commands and list active tools.
  * `/clear`: Flush current conversation context history while keeping git state.
  * `/compact`: Prune and compress the active context window to preserve token budget.
  * `/config`: Interactive UI to inspect and change active model and system parameters.
  * `/cost`: Print accumulated API spend and token usage for the active terminal session.
  * `/doctor`: Run health check verifying local Git, Node.js, network, and API authentication.
  * `/bug`: File an issue report including sanitized session telemetry.
  * `/init`: Scaffolds a project-level `CLAUDE.md` context instruction file in the repository root.
  * `/exit`: Safely exit the agent environment.

#### 2. Codex CLI (OpenAI)
* **Architecture:** Standalone autonomous binary synthesizing multi-file diffs and auto-committing validated changes via OpenAI models (`o3-mini`, `gpt-4o`).
* **Upstream APIs:** OpenAI Responses API (`POST https://api.openai.com/v1/responses`), Chat Completions API (`/v1/chat/completions`), and MCP stdio/HTTP servers.
* **CLI Invocations:**
  ```bash
  codex "Refactor the authentication middleware to use JWT standard"
  codex --model o3-mini --reasoning-effort high --approval auto "Fix SQL injection in db/user.py"
  codex diff                                    # Review uncommitted changes synthesized by Codex
  codex apply                                   # Apply changes to working directory
  ```
* **In-Chat Slash Commands:**
  * `/diff`: Output synthesized changes in standard git diff format before disk write.
  * `/accept`: Confirm and stage proposed edits.
  * `/reject`: Discard proposed edits and reset agent buffer.
  * `/retry`: Re-synthesize solution with an alternative reasoning pathway.

#### 3. Gemini CLI / Antigravity CLI (Google)
* **Architecture:** Cloud-linked CLI agent optimized for 2M+ token full-repository ingestion using Google AI Studio / Vertex AI contexts.
* **Upstream APIs:** Google Gemini API (`POST /v1beta/models/{model}:streamGenerateContent`), Context Caching API (`POST /v1beta/cachedContents`), Vertex AI (`aiplatform.googleapis.com`), and MCP JSON-RPC.
* **CLI Invocations:**
  ```bash
  gemini analyze --all                          # Ingest entire workspace into long-context buffer
  gemini prompt "Identify dead code branches across all microservices"
  gemini fix --target src/main.rs --error-log ./target/build.log
  antigravity sync                              # Sync local workspace state with cloud context cache
  antigravity chat --model gemini-2.5-pro --budget 8192
  ```
* **In-Chat Slash Commands:**
  * `/cache`: Display active Context Cache ID, expiration TTL, and cached token count.
  * `/files`: Print all project files mapped into the context payload.
  * `/grounding [on|off]`: Toggle Google Search dynamic grounding.
  * `/think [tokens]`: Adjust or inspect current `thinkingBudget`.
  * `/goal`: Run long-running autonomous task without stopping until verified.
  * `/schedule`: Schedule recurring instructions or timers.

#### 4. Bifrost CLI (Maxim AI)
* **Architecture:** Agent-to-gateway bridge, connecting CLI agents to virtualized MCP clusters and high-throughput proxy routers.
* **Exposed Local APIs:** OpenAI-compatible proxy (`POST http://localhost:9090/v1/chat/completions`), Anthropic proxy (`POST /v1/messages`), MCP SSE/Streamable HTTP bridge (`/mcp`).
* **CLI Invocations:**
  ```bash
  bifrost daemon start --port 9090 --config ./bifrost.yaml
  bifrost route add --pattern "gpt-*" --upstream "https://api.openai.com/v1"
  bifrost mcp bridge --server "github" --cmd "npx -y @modelcontextprotocol/server-github"
  bifrost status                                # Inspect routing tables, healthy upstreams, active tokens
  ```
* **In-Chat / Interactive Commands:**
  * `/route status`: Display health and latency of routed upstream endpoints.
  * `/mcp list`: Enumerate active MCP tools attached to the bridge session.
  * `/fallback <provider>`: Manually switch active inference route.

#### 5. Aider
* **Architecture:** Terminal pair programmer enforcing strict Git-centric operations, repository maps (tree-sitter), and automated commit synthesis.
* **Upstream APIs:** LiteLLM unified provider adapter (OpenAI `/v1/chat/completions`, Anthropic `/v1/messages`, Gemini `:generateContent`, Ollama `/api/chat`).
* **CLI Invocations:**
  ```bash
  aider --model claude-3-7-sonnet-20250219 --editor-model claude-3-5-haiku-20241022
  aider --model ollama/qwen2.5-coder:32b --api-base http://localhost:11434/v1
  aider --model openrouter/deepseek/deepseek-r1 --map-tokens 2048
  aider --message "Fix unit tests in tests/test_auth.py" --yes --auto-commits
  ```
* **In-Chat Slash (`/`) Commands & In-Session Modifiers:**
  * `/add <file>`: Add file(s) to the active context session (supports globs, e.g., `/add src/**/*.py`).
  * `/drop <file>`: Remove file(s) from the context session.
  * `/read-only <file>`: Add file for reading only (the agent will not edit it).
  * `/ls`: List all files currently in the context session with token counts.
  * `/map`: Print current repository AST map generated by tree-sitter.
  * `/architect`: Switch to architect/editor split mode (complex reasoning model plans, fast model codes).
  * `/ask <query>`: Ask a question about the code without allowing file modifications.
  * `/commit`: Force a Git commit with an AI-generated commit message.
  * `/diff`: Show the git diff of the changes made in the current session.
  * `/undo`: Revert the last commit made by Aider.
  * `/tokens`: Display context window token consumption broken down by files, map, and history.
  * `/run <cmd>` or `!<cmd>`: Execute shell command and automatically feed output into chat.
  * `/test <cmd>`: Run test suite; automatically feed stack trace back to the agent if tests fail.
  * `/voice`: Speak prompts via microphone input.
  * `/web <url>`: Scrape URL and add cleaned Markdown representation into context.
  * `/clear`: Clear conversation history (keeps added files in context).
  * `/reset`: Clear conversation history AND drop all files from context.

#### 6. Goose (Block / Linux Foundation)
* **Architecture:** Extensible on-machine autonomous agent engine driven completely by Model Context Protocol (MCP) server plugins.
* **Exposed & Upstream APIs:** Local `goosed` HTTP/SSE daemon (`POST /reply`, `GET /status`), MCP JSON-RPC (`stdio` / `sse`), and multi-provider LLM endpoints (OpenAI, Anthropic, Databricks, Ollama).
* **CLI Invocations:**
  ```bash
  goose session                                 # Start interactive terminal session
  goose session --name "feature-auth" --resume
  goose run --text "Run linting on the repo and fix all PEP8 violations"
  goose configure                               # Configure providers (Databricks, OpenAI, Ollama, Anthropic)
  goose plugins list                            # List active MCP plugins
  ```
* **In-Chat Slash Commands:**
  * `/tools`: Enumerate all MCP tools registered to the Goose daemon.
  * `/plugin add <name>`: Register and initialize an external MCP provider.
  * `/context`: View injected environment context, active shell path, and system state.
  * `/reset`: Clear current session conversation buffers.

#### 7. Kilo CLI
* **Architecture:** Low-memory Rust coding agent with parallel multi-file fuzzy search and in-place code modification.
* **Upstream APIs:** OpenRouter, Anthropic `/v1/messages`, OpenAI `/v1/chat/completions`, local Ollama/vLLM endpoints, and MCP servers.
* **CLI Invocations:**
  ```bash
  kilo edit --prompt "Migrate from Tokio 0.2 to Tokio 1.0" --path ./crates
  kilo scan --security                          # Run local AST vulnerability scan
  kilo interactive                              # Launch terminal UI (TUI)
  ```
* **In-Chat Commands:**
  * `/focus <path>`: Narrow agent attention exclusively to a specific directory subtree.
  * `/review`: Print proposed edits side-by-side in terminal TUI.
  * `/revert`: Undo the last file modification batch.

#### 8. OpenCode
* **Architecture:** Open-source command-line software agent featuring sandboxed Docker execution loops and automated test verification.
* **Exposed & Upstream APIs:** Local HTTP/SSE server (`opencode serve --port 4096` exposing `POST /session`, `POST /session/{id}/message`, `GET /event`), plus Anthropic/OpenAI/Ollama/MCP providers.
* **CLI Invocations:**
  ```bash
  opencode run --task "Implement OAuth2 PKCE flow" --docker-sandbox
  opencode serve --port 4096                    # Expose headless REST/SSE agent server
  opencode resume --session-id sess_987654
  ```
* **In-Chat Commands:**
  * `/test`: Execute current workspace tests inside the container.
  * `/inspect`: Inspect container filesystem diff against host repository.
  * `/stop`: Abort autonomous repair cycle.

#### 9. Qwen Code (Alibaba Cloud)
* **Architecture:** CLI development agent tuned for Qwen-2.5-Coder and QwQ models via DashScope or local vLLM.
* **Upstream APIs:** Alibaba DashScope OpenAI-Compatible API (`https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions`), local vLLM/Ollama `/v1/chat/completions`.
* **CLI Invocations:**
  ```bash
  qwen-code start --model qwen2.5-coder-32b-instruct
  qwen-code generate --file src/service.ts --specs specs/api.md
  ```
* **In-Chat Commands:**
  * `/translate <lang>`: Convert highlighted source code file to another language.
  * `/explain`: Generate deep algorithmic explanation of selected block.
  * `/doc`: Synthesize JSDoc/Sphinx documentation across all functions.

#### 10. Grok Build (xAI)
* **Architecture:** Terminal debugging and compilation agent utilizing Grok-3 extended reasoning models to resolve deep compiler and build failures.
* **Upstream APIs:** xAI REST API (`POST https://api.x.ai/v1/chat/completions` and `/v1/messages`).
* **CLI Invocations:**
  ```bash
  grok-build "cargo build --release"
  grok-build --watch "npm run build"
  ```
* **In-Chat Commands:**
  * `/trace`: Inspect low-level build trace and compiler error trees.
  * `/reason`: Show raw step-by-step thinking traces emitted by Grok-3.

#### 11. GitHub Copilot CLI
* **Architecture:** Shell-embedded developer copilot integrated via GitHub CLI (`gh`).
* **Upstream APIs:** GitHub Copilot Chat & Completions API (`POST https://api.githubcopilot.com/chat/completions`) authenticated via GitHub OAuth session token.
* **CLI Invocations:**
  ```bash
  gh copilot suggest "Find all open TCP sockets listening on port 8080"
  gh copilot explain "tar -zxvf archive.tar.gz -C /opt/"
  gh copilot alias -- bash >> ~/.bashrc         # Inject shell shortcuts ??, git?, gh?
  ```
* **In-Session Directives:**
  * Select `Execute command`: Directly run synthesized shell command.
  * Select `Revise command`: Provide natural language feedback to revise command.
  * Select `Copy to clipboard`: Stage snippet in system clipboard without executing.

#### 12. Amazon Q Developer CLI
* **Architecture:** Enterprise terminal agent bridging AWS Bedrock, local project context, and AWS infrastructure management.
* **Upstream APIs:** AWS CodeWhisperer / Amazon Q Streaming API (`GenerateAssistantResponse` over HTTP/2) & AWS IAM Identity Center OIDC endpoints.
* **CLI Invocations:**
  ```bash
  q chat                                        # Launch interactive terminal session
  q login                                       # Authenticate with AWS Builder ID / IAM Identity Center
  q translate                                   # Convert natural language to AWS CLI commands
  q doctor                                      # Inspect AWS credentials, profile, and local permissions
  ```
* **In-Chat Slash Commands:**
  * `/help`: Display operational commands.
  * `/clear`: Clear active conversation buffer.
  * `/dev`: Enter autonomous developer mode to generate multi-file code solutions.
  * `/transform`: Initiate code transformation (e.g., Java 8/11 to Java 17/21).
  * `/quit`: Exit the session.

#### 13. Warp AI / Agentic Terminal
* **Architecture:** Rust-native hardware-accelerated terminal emulator with integrated agentic sub-shells.
* **Upstream APIs:** Warp Cloud AI Gateway (`gRPC` / HTTPS streaming proxy routing to Anthropic Claude, OpenAI GPT, and Google Gemini models).
* **Key Commands & Triggers:**
  * `Ctrl+Space`: Trigger inline terminal command completions.
  * `Ctrl+R`: Agentic historical search with natural language semantic matching.
  * `Cmd+I` / `Ctrl+I`: Open Warp AI Agent Panel.
* **In-Chat Commands & Context Triggers:**
  * `#terminal_error`: Automatically grabs the last non-zero exit code and stderr output.
  * `#git_diff`: Pulls unstaged git modifications into agent context.
  * `/explain`: Explain terminal command block under cursor.
  * `/debug`: Diagnose why the previous pipeline command failed.

#### 14. Mentat
* **Architecture:** Autonomous coding assistant running in terminal with AST tracking and interactive multi-line terminal diff approvals.
* **Upstream APIs:** OpenAI `/v1/chat/completions`, Anthropic `/v1/messages`, and LiteLLM endpoints.
* **CLI Invocations:**
  ```bash
  mentat src/ tests/                            # Launch session restricted to specific paths
  mentat --model gpt-4o
  ```
* **In-Chat Slash Commands:**
  * `/exclude <path>`: Remove paths from active context.
  * `/undo`: Revert file modifications made during the last turn.
  * `/commit`: Stage and commit accepted modifications.
  * `/config`: Inspect active configuration values.

#### 15. Plandex
* **Architecture:** Go-based terminal agent built for long-running architectural tasks using isolated sandboxed git branches.
* **Exposed & Upstream APIs:** Plandex Server REST API (`http://localhost:8099/api/v1/plans`, `/branches`, `/tell`), routing upstream to OpenAI, Anthropic, Google Gemini, OpenRouter, and Ollama.
* **CLI Invocations:**
  ```bash
  plandex new                                   # Create a new planning plan
  plandex load                                  # Load project files into plan context
  plandex tell "Refactor API routing to use Fiber v3"
  plandex changes                               # Review pending code modifications
  plandex apply                                 # Apply changes directly to your real branch
  plandex discard                               # Revert and drop proposed plan
  ```
* **In-Chat Commands:**
  * `/context`: Print list of files and terminal outputs loaded in current plan.
  * `/diff`: Output full color diff across all modified files.
  * `/branch`: Switch plan branches to explore alternative architectures.

#### 16. Crush (Charmbracelet)
* **Architecture:** Terminal UI (TUI) based fast code transformation agent written in Go.
* **Upstream APIs:** OpenAI `/v1/chat/completions`, Anthropic `/v1/messages`, Google Gemini, Groq, OpenRouter, and MCP JSON-RPC tools.
* **CLI Invocations:**
  ```bash
  crush file.py "Add type hints to all functions"
  crush tui                                     # Open interactive split-screen TUI
  ```
* **TUI Shortcuts & In-Chat Commands:**
  * `/apply`: Write changes to disk immediately.
  * `/reject`: Cancel pending changes.
  * `/diff`: Toggle inline unified diff preview.

#### 17. Devika
* **Architecture:** Open-source agentic research and coding alternative featuring web browsing, research planning, and sub-task division.
* **Exposed & Upstream APIs:** Local FastAPI/Flask REST & Socket.IO server (`http://localhost:1337/api/messages`, `/api/create-project`), Bing/Google Search APIs, and Ollama/OpenAI/Claude APIs.
* **CLI Invocations:**
  ```bash
  python devika.py server                       # Start backend orchestrator on port 1337
  python devika.py ui                           # Start frontend client on port 3000
  ```
* **In-Chat Commands:**
  * `/create <project>`: Initialize a new isolated workspace.
  * `/browse <url>`: Instruct browser agent to scrape and screenshot a webpage.
  * `/plan`: Force generation of a high-level execution checklist before coding.

#### 18. OpenHands CLI (OpenDevin)
* **Architecture:** Autonomous software engineer in Docker, evaluating tasks against SWE-bench benchmarks.
* **Exposed & Upstream APIs:** OpenHands REST & WebSocket API (`http://localhost:3000/api/conversations`, `POST /api/conversations/{id}/messages`, `GET /api/options/models`) and Runtime Action Execution Server (`POST /execute_action`).
* **CLI Invocations:**
  ```bash
  docker run -it --pull=always \
    -e SANDBOX_USER_ID=$(id -u) \
    -e WORKSPACE_BASE=$PWD \
    -e LLM_API_KEY=$OPENAI_API_KEY \
    -p 3000:3000 \
    docker.all-hands.dev/all-hands-ai/openhands:latest
  openhands --task "Fix bug in issue #12"
  ```
* **Interactive Commands:**
  * `/pause`: Pause running agent loop to inspect container state.
  * `/resume`: Continue autonomous step execution.
  * `/abort`: Kill container execution and dump state logs.

#### 19. SWE-agent
* **Architecture:** Benchmark evaluation harness and developer agent developed by Princeton NLP.
* **Upstream APIs:** LiteLLM backend (Anthropic, OpenAI, Together, vLLM) + Docker container execution socket (`SWE-ReX` runtime API).
* **CLI Invocations:**
  ```bash
  python run.py \
    --model_name claude-3-7-sonnet-20250219 \
    --data_path /path/to/issue.json \
    --repo_path /path/to/repo \
    --config_file config/default_from_url.yaml \
    --per_instance_cost_limit 3.00
  ```
* **Agent-Computer Interface (ACI) Directives:**
  * `open <file> [line]`: Open file in virtual window.
  * `scroll_up` / `scroll_down`: Move view through source file.
  * `search_dir <pattern>`: Search across repository directory tree.
  * `submit`: Declare issue resolved and generate formal patch file.

#### 20. GPT-Engineer
* **Architecture:** Scaffolds full repositories from high-level natural language prompt descriptions.
* **Upstream APIs:** LangChain provider adapters (`ChatOpenAI`, `ChatAnthropic`, Azure OpenAI, local OpenAI-compatible servers).
* **CLI Invocations:**
  ```bash
  gpt-engineer projects/my-new-app
  gpt-engineer --improve projects/my-new-app    # Iteratively enhance existing codebase
  ```
* **Directives:** Ingests `prompt` file, runs dependency synthesis, generates code, and prompts to execute scaffolding commands.

#### 21. Shell-Genie
* **Architecture:** Natural language CLI translation utility with safety verification before execution.
* **Upstream APIs:** OpenAI `/v1/chat/completions` and Free-Genie hosted REST endpoint.
* **CLI Invocations:**
  ```bash
  shell-genie ask "Find all files larger than 100MB modified in the last 7 days"
  shell-genie init                              # Configure backend
  ```
* **Interactive Flags:** Prompts user `[Y/n]` to confirm before executing any synthesized destructive command.

#### 22. OpenInterpreter
* **Architecture:** Open-source local code interpreter executing Python, JavaScript, and Bash commands directly on the user's host machine or in Docker.
* **Exposed & Upstream APIs:** Local FastAPI/SSE Server (`interpreter --server` exposing `POST /chat`, `GET /settings`), LiteLLM upstream routing, and OS Accessibility/Screen APIs.
* **CLI Invocations:**
  ```bash
  interpreter                                   # Launch interactive terminal interpreter
  interpreter -y                                # Auto-run commands without interactive approval
  interpreter --model ollama/llama3.3           # Connect to local inference engine
  interpreter --os                              # Computer-use mode (GUI mouse/keyboard control)
  ```
* **In-Chat Commands:**
  * `%reset`: Reset conversation context and state memory.
  * `%undo`: Revert the last execution turn.
  * `%tokens`: View context token count.

#### 23. ShellGPT (`sgpt`)
* **Architecture:** CLI tool integrating OpenAI models directly into shell pipelines.
* **Upstream APIs:** OpenAI `/v1/chat/completions` or any custom OpenAI-compatible `--api-base` (Ollama, vLLM, LiteLLM).
* **CLI Invocations:**
  ```bash
  sgpt --shell "Extract audio from all MP4 files in directory"
  sgpt --code "Write a Rust function to calculate SHA256"
  cat error.log | sgpt "Analyze stack trace and find root cause"
  ```
* **Flags:** `--shell` / `-s` (generate shell command), `--code` / `-c` (generate pure code), `--chat <session_name>` (maintain persistent multi-turn chat).

#### 24. Mods (Charmbracelet)
* **Architecture:** Pipeline-oriented AI CLI built for terminal streams and shell scripts.
* **Upstream APIs:** OpenAI, Anthropic, Cohere, Groq, Google Gemini, Azure OpenAI, and Ollama REST APIs.
* **CLI Invocations:**
  ```bash
  git diff | mods "Write a concise conventional commit message"
  mods -f "Summarize this JSON schema" < schema.json
  mods --model gpt-4o --continue                # Continue previous turn
  ```

#### 25. Fabric (Daniel Miessler)
* **Architecture:** Open-source modular prompt and agent framework designed to pipe terminal streams through structured patterns.
* **Exposed & Upstream APIs:** Local REST API (`fabric --serve` exposing `POST /chat`, `GET /patterns/names`), YouTube Transcript/yt-dlp integration, and multi-provider LLM APIs.
* **CLI Invocations:**
  ```bash
  fabric --listpatterns                         # List installed reasoning patterns
  cat README.md | fabric --pattern extract_wisdom
  fabric -y "https://youtube.com/watch?v=..." --pattern summarize
  ```

#### 26. Simon Willison's LLM CLI (`llm`)
* **Architecture:** Extensible CLI utility and Python library for running prompts against local and cloud LLMs with SQLite logging.
* **Exposed & Upstream APIs:** Python `llm` SDK (`llm.get_model()`), SQLite local store (`logs.db`), and plugin-based REST adapters (`llm-claude-3`, `llm-gemini`, `llm-ollama`).
* **CLI Invocations:**
  ```bash
  llm "Explain SQLite WAL mode"
  llm -m claude-3-7-sonnet "Refactor this function" -c
  llm install llm-ollama                        # Install local plugin
  llm logs -n 5                                 # Inspect last 5 queries logged to SQLite
  ```

#### 27. CrewAI CLI
* **Architecture:** Orchestration CLI for multi-agent autonomous teams with specialized roles.
* **Exposed & Upstream APIs:** CrewAI Enterprise REST API (`POST /kickoff`, `GET /status/{kickoff_id}`), OpenAI/Anthropic/LiteLLM endpoints, and MCP tools.
* **CLI Invocations:**
  ```bash
  crewai create crew my-agency                  # Scaffold new multi-agent project
  crewai run                                    # Run multi-agent execution pipeline
  crewai test --n 3                             # Run 3 test iterations and evaluate outputs
  ```

#### 28. AutoGPT CLI
* **Architecture:** Autonomous recursive goal-driven agent engine with internet browsing, file management, and long-term memory.
* **Exposed & Upstream APIs:** Agent Protocol v1 REST API (`POST /ap/v1/agent/tasks`, `POST /ap/v1/agent/tasks/{task_id}/steps`, `GET /ap/v1/agent/tasks/{task_id}/artifacts`).
* **CLI Invocations:**
  ```bash
  autogpt run --continuous                      # Autonomous execution loop
  autogpt serve                                 # Expose Agent Protocol HTTP endpoints
  ```

#### 29. MetaGPT
* **Architecture:** Multi-agent framework encoding Standard Operating Procedures (SOPs) into software development teams (Product Manager, Architect, Engineer, QA).
* **Upstream APIs:** OpenAI `/v1/chat/completions`, Anthropic `/v1/messages`, DashScope, Azure OpenAI, and Ollama APIs.
* **CLI Invocations:**
  ```bash
  metagpt "Create a 2048 game with HTML and JavaScript"
  metagpt --code-review --project-path ./repo
  ```

#### 30. ChatDev
* **Architecture:** Virtual software company of agents communicating via chat chains to design, code, test, and document applications.
* **Upstream APIs:** OpenAI `/v1/chat/completions` and OpenAI-compatible local inference servers.
* **CLI Invocations:**
  ```bash
  python run.py --task "Develop a desktop Markdown editor" --name "MarkdownApp"
  ```

---

## 4. Agentic LSP & Protocol Providers

### 4.1 Exhaustive Catalog with Daemons, Protocols, APIs, and Command Sets

#### 1. Model Context Protocol (MCP) Daemons
* **Architecture:** Universal open standard for connecting AI agents to local resources, tools, and execution environments over JSON-RPC 2.0.
* **Transports & Endpoints:** `stdio` (subprocess pipes), Streamable HTTP (`POST /mcp` + `GET /mcp` SSE stream), and Legacy HTTP+SSE (`GET /sse` + `POST /messages`).
* **Common MCP Server Initialization Commands:**
  ```bash
  npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/dir
  npx -y @modelcontextprotocol/server-sqlite --db-path /path/to/database.db
  npx -y @modelcontextprotocol/server-postgres postgresql://user:password@localhost:5432/mydb
  GITHUB_PERSONAL_ACCESS_TOKEN=$GITHUB_TOKEN npx -y @modelcontextprotocol/server-github
  npx -y @modelcontextprotocol/server-git --repository /path/to/repo
  npx -y @modelcontextprotocol/server-memory
  npx -y @modelcontextprotocol/server-puppeteer
  ```
* **Supported Protocol Methods (JSON-RPC):**
  * `initialize` & `notifications/initialized`: Negotiate protocol version and capabilities.
  * `tools/list` & `tools/call`: Enumerate available tools and execute parameterized agentic actions.
  * `resources/list`, `resources/read`, & `resources/subscribe`: Inspect and watch contextual data feeds.
  * `prompts/list` & `prompts/get`: Pull standardized multi-shot prompt templates.
  * `sampling/createMessage`: Allow an MCP server to request LLM inference back through the host client.

#### 2. Continue.dev LSP
* **Architecture:** Sidecar daemon and language server implementing LSP 3.17 to expose code intelligence to Neovim, VS Code, and JetBrains.
* **Protocol & APIs:** LSP JSON-RPC 2.0 (`stdio` / TCP `:65432`), custom Continue JSON-RPC methods (`config/getSerializedProfileInfo`, `llm/streamChat`), plus upstream OpenAI/Anthropic/Ollama/MCP APIs.
* **Daemon Management:**
  ```bash
  continue-lsp --stdio
  continue-lsp --port 65432
  ```
* **In-Chat Slash (`/`) Commands:**
  * `/edit`: Apply inline code transformations to selected text blocks.
  * `/comment`: Add documentation comments to selected classes or functions.
  * `/test`: Generate comprehensive unit tests for highlighted code.
  * `/cmd`: Generate shell command from prompt and stage in terminal.
  * `/clear`: Clear active chat history.
  * `/share`: Generate exportable Markdown transcript of current session.
* **Context Providers (`@` Mentions):**
  * `@Files`: Search and attach specific workspace files.
  * `@Codebase`: Vector search over entire repository via local embeddings.
  * `@Git`: Attach recent git diffs, commits, or branch status.
  * `@Docs`: Scrape and ingest indexed external documentation sets.
  * `@Terminal`: Grab the latest terminal contents and error streams.
  * `@URL`: Fetch live web page content into the prompt context.

#### 3. Cline (MCP-Native Extension Host)
* **Architecture:** Autonomous coding agent extension host for VS Code, built natively on top of MCP servers with human-in-the-loop permissioning.
* **Protocol & APIs:** VS Code Extension Host IPC, MCP JSON-RPC (`stdio` / `sse`), and direct provider APIs (Anthropic `/v1/messages`, OpenRouter, AWS Bedrock, Google Gemini, OpenAI, Ollama).
* **In-Chat Commands & Mode Switches:**
  * Mode: **Code Mode** (Full autonomy to read/write files and execute commands).
  * Mode: **Architect Mode** (Planning, diagramming, and requirements mapping without file modifications).
  * Mode: **Ask Mode** (Answering general questions about the workspace).
* **Direct In-Chat Control Actions:**
  * `Approve`: Grant permission for Cline to run a specific terminal command or edit a file.
  * `Deny`: Refuse action and provide natural language instructions.
  * `/clear`: Reset task conversation context.
  * `/history`: Browse and restore previous autonomous sessions.

#### 4. Roo Code (formerly Roo Cline)
* **Architecture:** Multi-mode fork of Cline adding specialized system personalities and dynamic MCP tool configuration per workspace.
* **Protocol & APIs:** VS Code Extension Host IPC, MCP JSON-RPC, and multi-provider REST/SSE APIs (Anthropic, OpenAI, Gemini, OpenRouter, LM Studio, Ollama).
* **In-Chat Modes & Slash Commands:**
  * `/mode [code|architect|ask|test|custom]`: Dynamically switch operational persona and tool access policies.
  * `/mcp`: Open MCP server management console.
  * `/prompts`: Open custom prompt template manager.
  * `/compact`: Compress context history using semantic summarization.

#### 5. lsp-ai
* **Architecture:** Open-source language server acting as a bridge between any LSP-compatible editor (Neovim, Helix, Emacs) and local/cloud LLMs.
* **Protocol & APIs:** LSP JSON-RPC 2.0 (`--stdio`) translating editor requests to OpenAI `/v1/chat/completions`, Anthropic `/v1/messages`, Gemini, Ollama `/api/generate`, and `llama.cpp`.
* **CLI Invocations:**
  ```bash
  lsp-ai --config /path/to/lsp-ai.json --stdio
  ```
* **Capabilities Exposed via LSP Protocol:**
  * `textDocument/completion`: Inline code completions.
  * `textDocument/codeAction`: Triggers LLM refactorings and doc generation.
  * `textDocument/formatting`: Automated code formatting via LLM rules.

#### 6. Tabby (Self-Hosted Code Engine & LSP)
* **Architecture:** Self-hosted code completion and search daemon exposing an LSP-compatible server and web UI.
* **Exposed APIs:** `POST http://localhost:8080/v1/completions` (FIM completion), `POST /v1/chat/completions` (OpenAI-compatible), `POST /v1/events`, `GET /v1/health`, and LSP over WebSocket/HTTP (`/v1/lsp`).
* **CLI Invocations:**
  ```bash
  docker run -p 8080:8080 --gpus all -v ~/.tabby:/data tabbyml/tabby serve --model StarCoder-1B --device cuda
  ```

#### 7. FauxPilot LSP
* **Architecture:** Open-source backend compatible with the official GitHub Copilot extension, redirecting completions to local engines (TGI, vLLM).
* **Exposed APIs:** Copilot-compatible `POST /v1/engines/copilot-codex/completions` and OpenAI `/v1/completions` proxied to Triton/vLLM.
* **CLI Invocations:**
  ```bash
  ./launch.sh --gpu 0 --model-name codellama/CodeLlama-13b-Instruct-hf
  ```

#### 8. Sourcegraph Cody Agent / LSP Daemon
* **Architecture:** Headless JSON-RPC daemon wrapping Cody's enterprise context retrieval, embeddings, and multi-repo graph queries.
* **Protocol & APIs:** JSON-RPC 2.0 (`cody-agent --stdio`), Sourcegraph GraphQL API (`POST https://sourcegraph.com/.api/graphql`), and Sourcegraph LLM Proxy (`POST /.api/llm/chat/completions`).
* **CLI Launch Command:**
  ```bash
  cody-agent --stdio
  ```
* **In-Chat Slash Commands:**
  * `/explain`: Explain code structure and functionality.
  * `/smell`: Run linting and identify architectural anti-patterns.
  * `/test`: Generate test harness.
  * `/doc`: Generate inline documentation.
* **Context Mentions (`@`):**
  * `@file`: Pin specific workspace file.
  * `@folder`: Attach an entire folder hierarchy.
  * `@sym`: Reference a symbol defined in the codebase AST.
  * `@web`: Scrape external technical documentation.

#### 9. Supermaven Engine
* **Architecture:** Ultra-low latency (<100ms) inline completion daemon written in C++, communicating with editor extensions over a local Unix socket/named pipe.
* **Protocol & APIs:** Local binary/JSON-RPC stream (`sm-agent --stdio`) + Supermaven Cloud Babble/1M-token context sync API (`wss://api.supermaven.com`).
* **Daemon Invocation:**
  ```bash
  ~/.supermaven/bin/sm-agent --stdio
  ```
* **In-Chat Commands (Supermaven Chat Preview):**
  * `/clear`: Clear active chat history.
  * `/model`: Switch between Standard and Pro inference tiers.

#### 10. OpenCtx
* **Architecture:** Open Context protocol standardizing context retrieval plugins (code search, devops metrics, database schemas) into editors.
* **Protocol & APIs:** OpenCtx Provider Interface (`annotations()`, `items()`, `mentions()`) over JSON-RPC / ESM modules.
* **CLI / Library Commands:**
  ```bash
  openctx test-provider @openctx/provider-github
  ```

#### 11. Tabnine Engine & LSP
* **Architecture:** Enterprise-grade hybrid local/cloud completion daemon (`TabnineLocal`) communicating via stdin/stdout JSON-RPC protocol.
* **Protocol & APIs:** Local JSON-RPC 2.0 (`Autocomplete`, `Chat`, `State`) + Tabnine Enterprise Cloud/VPC API (`https://api.tabnine.com`).
* **CLI / Binary Execution:**
  ```bash
  ~/.tabnine/bin/Tabnine --client-name vscode --client-version 3.10.0
  ```
* **In-Chat Commands:**
  * `/explain`: Explain highlighted block.
  * `/fix`: Detect vulnerabilities and bugs.
  * `/test`: Generate unit tests.

#### 12. Refact.ai LSP
* **Architecture:** Open-source AI code assistant language server and proxy supporting local fine-tuned models, completions, and code-refactoring actions.
* **Exposed & Upstream APIs:** LSP JSON-RPC (`--stdio`), Refact Server REST API (`POST /v1/code-completion`, `POST /v1/chat`, `POST /v1/ast-file-symbols`), and MCP tools.
* **CLI Invocations:**
  ```bash
  refact-lsp --address-url https://my-refact-server/ --api-key $REFACT_KEY --stdio
  ```

#### 13. llm-ls (Hugging Face)
* **Architecture:** High-speed Rust-based Language Server for code completion using Hugging Face Inference API, TGI, or local Ollama endpoints.
* **Protocol & APIs:** LSP 3.17 (`textDocument/completion`, custom `llm-ls/getCompletions`) -> Hugging Face `/models/{id}`, TGI `/generate`, or Ollama `/api/generate`.

#### 14. Agent-to-Agent Protocol (A2A --- Google / Linux Foundation)
* **Architecture:** Open protocol enabling opaque, multi-vendor AI agents to discover capabilities, negotiate modalities, and delegate long-running tasks across organizational boundaries.
* **Protocol & Endpoints:** JSON-RPC 2.0 over HTTP(S) + SSE, Agent discovery via `GET /.well-known/agent.json` (Agent Card), `tasks/send`, `tasks/sendSubscribe`, `tasks/get`, and `tasks/cancel`.

---

## 5. Agentic Editors & Native IDE Providers

### 5.1 Exhaustive Catalog with Backend APIs, Invocations, Shortcuts, and In-Chat Slash Commands

#### 1. Cursor (Anysphere)
* **Architecture:** VS Code fork featuring Merkle tree codebase indexing, shadow workspaces, fast speculative autocomplete, and an agent loop.
* **Backend & Extension APIs:** Cursor Cloud gRPC/HTTP2 API (`api2.cursor.sh` for Merkle sync, speculative tab completions, and Composer orchestration), BYOK OpenAI/Anthropic/Gemini endpoints, and MCP JSON-RPC servers.
* **CLI Launch Command:**
  ```bash
  cursor .                                      # Open current workspace
  cursor --diff file_a file_b                   # Launch side-by-side visual diff
  cursor install-extension <id>                # Install VS Code compatible extension
  ```
* **Editor Shortcuts:**
  * `Ctrl+K` / `Cmd+K`: Inline code generator and refactor prompt.
  * `Ctrl+L` / `Cmd+L`: Open the AI Chat / Agent panel.
  * `Ctrl+I` / `Cmd+I`: Open Composer (full-workspace multi-file editing mode).
  * `Tab`: Accept speculative code completion.
* **In-Chat Slash (`/`) Commands:**
  * `/edit`: Force active selection into inline editing mode.
  * `/explain`: Generate deep breakdown of highlighted code.
  * `/fix`: Diagnose compiler/linter error on highlighted line and propose patch.
  * `/test`: Generate unit tests for selected functions.
  * `/commit`: Synthesize git commit message from staged changes.
  * `/terminal`: Execute terminal commands directly through the Composer agent.
* **Context Mentions (`@` Triggers):**
  * `@Files`: Include specific workspace file(s) in prompt context.
  * `@Folders`: Include an entire directory tree.
  * `@Codebase`: Query the local vector store for semantic code snippets.
  * `@Docs`: Reference pre-indexed or custom technical documentation URLs.
  * `@Git`: Reference git commit history, diffs, or PR descriptions.
  * `@Web`: Perform live Google searches to augment answer.
  * `@Definitions`: Include symbol definitions and signatures.
  * `@Lint`: Feed active linter errors directly into agent prompt.

#### 2. Windsurf / Cascade (Cognition / Codeium)
* **Architecture:** VS Code fork powered by the "Cascade" collaborative agent engine with dynamic context indexing and proactive multi-file diffing.
* **Backend & Extension APIs:** Codeium/Windsurf gRPC Language Server (`language_server_macos_arm` / `linux_x64`), Cloud Inference API (`server.codeium.com`), and MCP JSON-RPC integration.
* **CLI Launch Command:**
  ```bash
  windsurf .                                    # Open current directory
  ```
* **Core Shortcuts:**
  * `Ctrl+L` / `Cmd+L`: Open Cascade Agent Panel.
  * `Ctrl+I` / `Cmd+I`: Open inline Cascade modification prompt.
* **Cascade Interaction Modes:**
  * **Chat Mode:** Read-only mode for exploration, code explanation, and architecture design.
  * **Write / Agent Mode:** Full autonomy to modify multiple files, run commands, and verify changes.
* **In-Chat Slash Commands:**
  * `/clear`: Clear active turn history in Cascade.
  * `/undo`: Revert the last multi-file changeset proposed by Cascade.
  * `/run`: Request execution of a shell command in the built-in terminal.
* **Cascade Context Triggers (`@`):**
  * `@file`: Target specific source files.
  * `@directory`: Target folder hierarchies.
  * `@problems`: Automatically pull VS Code Problems tab errors.
  * `@terminal`: Ingest terminal stdout and stderr.
  * `@preview`: Integrate internal webview live browser state.

#### 3. GitHub Copilot Agent Mode (VS Code & Visual Studio)
* **Architecture:** Agent mode inside official Copilot extension, allowing iterative tool-calling and multi-file code editing loops.
* **Backend & Extension APIs:** GitHub Copilot API (`https://api.githubcopilot.com/chat/completions`), VS Code Language Model API (`vscode.lm.selectChatModels`), Chat Participant API (`vscode.chat.createChatParticipant`), and MCP (`mcp.json`).
* **Key Shortcuts:**
  * `Ctrl+Shift+I` / `Cmd+Shift+I`: Open Copilot Edits / Agent panel.
* **In-Chat Slash Commands:**
  * `/clear`: Reset conversation context.
  * `/explain`: Explain highlighted snippet or selected file.
  * `/fix`: Propose resolution for selected compiler/linter problem.
  * `/tests`: Generate unit test suites.
  * `/doc`: Generate code documentation.
  * `/new`: Scaffold a new project or file from natural language.
  * `/newNotebook`: Create a new Jupyter notebook configured with dependencies.
* **Context Variables (`#` or `@` depending on IDE):**
  * `@workspace`: Search the entire repository using Copilot's local semantic index.
  * `@terminal`: Attach the state of the active integrated terminal.
  * `@vscode`: Ask questions about VS Code settings and keybindings.
  * `#file`: Attach a specific file.
  * `#selection`: Limit context strictly to highlighted text.
  * `#editor`: Use the currently visible editor state.

#### 4. Void Editor
* **Architecture:** 100% open-source VS Code fork with zero telemetry and BYOK (Bring Your Own Key) architecture for local and cloud models.
* **Backend APIs:** Direct client-side calls to Anthropic `/v1/messages`, OpenAI `/v1/chat/completions`, Gemini `:generateContent`, Ollama `/api/chat`, and vLLM.
* **CLI Command:** `void .`
* **In-Chat Slash Commands:**
  * `/edit`: Apply localized edits directly to the active editor buffer.
  * `/models`: Switch between local endpoints (Ollama, vLLM) and cloud APIs.
  * `/settings`: Configure custom headers, base URLs, and token limits.

#### 5. Zed (Zed AI)
* **Architecture:** Ultra-fast, GPU-accelerated code editor written in Rust with built-in assistant panels and slash-command context injections.
* **Backend & Extension APIs:** Zed Cloud LLM Proxy (`https://cloud.zed.dev`), direct BYOK adapters (Anthropic, OpenAI, Google Gemini, Ollama, OpenRouter), and Context Server (MCP) protocol.
* **CLI Launch Command:**
  ```bash
  zed .                                         # Open current repository
  zed --wait                                    # Open as external editor waiting for file close
  ```
* **Editor Shortcuts:**
  * `Ctrl+Shift+A` / `Cmd+Shift+A`: Open Zed AI Assistant Panel.
  * `Ctrl+Enter`: Send prompt to configured assistant model.
* **In-Chat Slash (`/`) Context Injectors:**
  * `/file <path>`: Injects contents of a specific file into the prompt.
  * `/tab`: Injects contents of the currently active editor tab.
  * `/directory <path>`: Injects file tree and contents of a directory.
  * `/prompt <name>`: Injects custom saved system prompt templates.
  * `/rules`: Injects `.zed/rules.md` workspace instruction file.

#### 6. Trae (ByteDance)
* **Architecture:** VS Code fork with "Builder" mode, integrating automated workspace error diagnosis and multi-turn terminal commands.
* **Backend APIs:** ByteDance Cloud Model Gateway, Anthropic/OpenAI/Gemini BYOK endpoints, and MCP server integration.
* **CLI Command:** `trae .`
* **In-Chat Slash Commands:**
  * `/build`: Run project build pipeline and diagnose any compilation errors.
  * `/fix`: Repair syntax errors in active buffer.
  * `/test`: Generate test cases.

#### 7. Replit Agent
* **Architecture:** Cloud-native software development agent operating within containerized Linux workspaces.
* **Backend APIs:** Replit Nix/Container Orchestration API, Replit PostgreSQL/Object Storage provisioning APIs, and Anthropic/OpenAI cloud inference endpoints.
* **In-Chat Directives & Actions:**
  * Natural language prompts trigger automated database creation, package installation, code editing, and server deployment.
  * Rollback Button: Direct UI action reverting container to snapshot taken before agent run.
  * `/checkpoint`: Force creation of a recoverable container snapshot.

#### 8. Positron AI (Posit)
* **Architecture:** Data science focused IDE (VS Code fork) with agents inspecting active R and Python kernel memory and dataframes.
* **Backend APIs:** Jupyter/Ark Kernel Introspection Protocol (inspecting live R/Python variables in memory) + Anthropic/OpenAI/AWS Bedrock LLM endpoints.
* **CLI Command:** `positron .`
* **In-Chat Commands:**
  * `/data <df_name>`: Ingest summary schema and statistical profiling of active dataframe.
  * `/plot`: Synthesize visualization code using ggplot2 or matplotlib.
  * `/explain`: Break down complex statistical modeling routines.

#### 9. PearAI
* **Architecture:** Open-source AI code editor integrating Continue.dev core with curated plugins.
* **Backend APIs:** PearAI Router API (`https://server.trypear.ai`) + Continue.dev LSP and local Ollama endpoints.
* **CLI Command:** `pearai .`
* **In-Chat Slash Commands:** Same as Continue.dev (`/edit`, `/comment`, `/test`, `/share`).

#### 10. Melty
* **Architecture:** Experimental VS Code fork tracking git history and terminal sessions to continuously suggest micro-refactorings.
* **Backend APIs:** Git shadow-branch diff tracker + Anthropic `/v1/messages` and OpenAI `/v1/chat/completions`.
* **CLI Command:** `melty .`
* **In-Chat Commands:**
  * `/diff`: Review inferred intent based on last 10 minutes of manual typing.
  * `/commit`: Generate an intention-aware git commit message.

#### 11. Amazon Q Developer for JetBrains & VS Code
* **Architecture:** IDE extension suite with chat, automated code reviews, and cross-version code transformations.
* **Backend APIs:** AWS CodeWhisperer / Q Developer Runtime API (`SendMessage`, `StartCodeAnalysis`, `StartTransformation`) via AWS IAM Identity Center / Builder ID.
* **In-Chat Slash Commands:**
  * `/dev`: Start an autonomous multi-file development session.
  * `/transform`: Initiate automated Java language upgrade (Java 8/11 to 17/21).
  * `/review`: Run security and code-quality vulnerability scan on active project.
  * `/clear`: Clear active conversation context.

#### 12. JetBrains AI Assistant & Junie
* **Architecture:** Integrated enterprise AI assistant for IntelliJ IDEA, PyCharm, and WebStorm, paired with Junie (JetBrains Autonomous Coding Agent).
* **Backend APIs:** JetBrains AI Platform Gateway (`https://api.jetbrains.ai`), PSI (Program Structure Interface) AST engine, and local Ollama/LM Studio connectors.
* **Shortcuts:** `Alt+Enter` (AI Actions), `Ctrl+\` (Inline Completion).
* **In-Chat Commands:**
  * `/explain`: Explain code structure.
  * `/refactor`: Propose idiomatic language refactoring.
  * `/generate-tests`: Synthesize framework-specific test suites (JUnit, pytest).

#### 13. Xcode 16 Predictive Code Completion & Swift Assist (Apple)
* **Architecture:** Specialized on-device and cloud agentic pair programmer tuned for Apple SDKs, Swift 6 concurrency, and SwiftUI view hierarchies.
* **Backend APIs:** On-device Apple Silicon MLX/CoreML local inference engine + Apple Private Cloud Compute (PCC) Swift Assist endpoint.

#### 14. Eclipse Theia AI / Theia IDE
* **Architecture:** Fully customizable, open-source AI-native IDE platform managed by the Eclipse Foundation with vendor-neutral protocol bindings.
* **Backend APIs:** `@theia/ai-core` provider registry (`LanguageModelRegistry`), OpenAI, Anthropic, Hugging Face, Ollama, and MCP JSON-RPC.

#### 15. Browser-Native Agentic Workspaces (Bolt.new, Lovable.dev, v0)
* **Bolt.new (StackBlitz):** In-browser Node.js/WebContainer runtime API (`@webcontainer/api`) enabling full-stack agentic development with instant dev-server boot.
* **Lovable.dev:** Full-stack browser application generator running autonomous UI/backend iteration loops with Supabase Management API & GitHub sync.
* **v0 (Vercel):** Generative component engineering canvas with OpenAI-compatible Model API (`POST https://api.v0.dev/v1/chat/completions`) translating design prompts to verified React/Tailwind/Shadcn code.

---

## 6. Agentic Service & Cloud Platform Providers

### 6.1 Exhaustive Catalog with REST APIs, Interfaces, Quotas, and Directives

#### 1. Cognition AI (Devin)
* **Architecture:** Autonomous cloud software engineer running in isolated Debian Linux micro-VMs with shell, browser, and code editor.
* **Interfaces & REST API:** Cloud Web UI, Slack bot, GitHub App, and Devin REST API (`POST https://api.devin.ai/v1/sessions`, `POST /v1/sessions/{id}/message`, `GET /v1/sessions/{id}`).
* **Task Submission & Chat Directives:**
  * Direct issue ingestion via `@Devin` mention in GitHub issues and pull requests.
  * In-chat directives:
    * `devin: pause`: Halts micro-VM execution.
    * `devin: resume`: Continues autonomous workflow.
    * `devin: restart`: Flushes current VM workspace and restarts from source commit.
* **Enforced Limits & Billing:** Billed per Devin Compute Unit (ACU) / wall-clock execution hour; strict concurrent runner caps (typically 1--10 parallel tasks per team).

#### 2. Factory AI (Code Droids)
* **Architecture:** Specialized autonomous enterprise cloud workers (Review Droid, Refactor Droid, Documentation Droid, Test Droid).
* **Interfaces & APIs:** GitHub/GitLab App webhooks, Slack App, Web Control Plane, and Factory Sessions REST API (`POST https://app.factory.ai/api/sessions`).
* **PR Directives:**
  * `@factory review`: Forces deep semantic review on open PR.
  * `@factory refactor <target>`: Triggers architectural refactor of specified module.
  * `@factory test`: Generates missing edge-case unit tests.
* **Quotas:** Monthly seat tiers; token consumption caps per repository per 24-hour cycle.

#### 3. All-Hands AI (OpenHands Cloud)
* **Architecture:** Enterprise managed SaaS runtime for OpenHands (OpenDevin) agent swarms running inside microVM sandboxes.
* **Interfaces & REST API:** Web UI, GitHub Webhooks, and OpenHands Cloud REST API (`POST https://api.all-hands.dev/v1/tasks`, `GET /v1/tasks/{id}`).
* **API Invocations:**
  ```bash
  curl -X POST "https://api.all-hands.dev/v1/tasks" \
    -H "Authorization: Bearer $ALLHANDS_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"prompt": "Fix test failures in PR #42", "repository": "org/repo"}'
  ```

#### 4. Augment Code
* **Architecture:** Enterprise code intelligence platform indexing 10M+ line monorepos using proprietary global semantic graphs.
* **Interfaces & APIs:** Native extensions for VS Code, JetBrains, Neovim, Auggie CLI (`auggie`), and Augment Context Engine API (`https://api.augmentcode.com`).
* **In-Chat Commands:** Context search automatically traverses multi-repository boundaries without explicit tagging; `/explain` for monorepo-aware symbol explanation.

#### 5. Magic.dev
* **Architecture:** Ultra-long-context software development models (LTM series, 100M+ tokens) capable of ingesting entire enterprise codebases.
* **Interfaces & APIs:** Private enterprise preview OpenAI-compatible API (`POST https://api.magic.dev/v1/chat/completions`) and dedicated cloud developer consoles.

#### 6. Poolside AI
* **Architecture:** RL-driven coding models trained via Reinforcement Learning from Code Execution Feedback (RLCEF).
* **Interfaces & APIs:** Enterprise on-premise appliances, AWS Bedrock / Azure private endpoints, and Poolside REST API (`POST /v1/chat/completions`).

#### 7. Sweep AI
* **Architecture:** Autonomous GitHub issue resolution agent converting issue tickets into tested, verified pull requests.
* **Interfaces & APIs:** GitHub App Webhook API (`issues`, `issue_comment`, `pull_request_review_comment`).
* **PR & Issue Directives:**
  * `Sweep: <instructions>` in issue description triggers automated PR creation.
  * Comments on PRs starting with `Sweep:` instruct the agent to update and push new commits to the branch.

#### 8. CodeRabbit
* **Architecture:** AI code review agent running AST and reasoning passes over pull request diffs.
* **Interfaces & APIs:** GitHub/GitLab/Azure DevOps App Webhooks and CodeRabbit Reports API (`GET https://api.coderabbit.ai/api/v1/report.generate`).
* **PR Directives:**
  * `@coderabbitai pause`: Temporarily disable reviews on PR.
  * `@coderabbitai resume`: Re-enable reviews.
  * `@coderabbitai review`: Force immediate review pass.
  * `@coderabbitai full review`: Ignore incremental cache and review entire PR from scratch.
  * `@coderabbitai summary`: Re-generate high-level PR walkthrough summary.
  * `@coderabbitai resolve`: Mark all CodeRabbit comments as resolved.

#### 9. Qodo (formerly CodiumAI)
* **Architecture:** Test synthesis and code integrity platform (Qodo Gen, Qodo Merge, Qodo Cover).
* **Interfaces & APIs:** IDE plugins, GitHub/GitLab Webhook API (Qodo Merge), and CLI runner (`qodo-cover`).
* **PR Directives (Qodo Merge):**
  * `/review`: Perform automated PR review.
  * `/describe`: Auto-generate PR description and changelog.
  * `/improve`: Suggest concrete code improvements and patches.
  * `/test`: Generate test cases for modified methods.

#### 10. Grit.io
* **Architecture:** Automated software migration agent running hybrid CodeQL AST queries and LLM transformations.
* **Interfaces & APIs:** GritQL Engine CLI, GitHub Check Runs API, and Grit Cloud API (`https://api.grit.io`).
* **CLI Invocations:**
  ```bash
  grit apply <pattern_name>                     # Apply migration pattern locally
  grit check                                    # Identify deprecated syntax across repo
  grit list                                     # Enumerate available standard migrations
  ```

#### 11. Ellipsis
* **Architecture:** Automated code review and bug fixing agent running inside ephemeral cloud containers.
* **Interfaces & APIs:** GitHub App Webhook API & Ellipsis REST API (`POST https://api.ellipsis.dev/v1/review`).
* **PR Directives:**
  * In-line comment: `ellipsis: fix this race condition`: Directly commits a fix to the PR branch.
  * In-line comment: `ellipsis: test this`: Generates regression test cases.

#### 12. GitHub Copilot Agent HQ
* **Architecture:** Centralized enterprise control hub inside GitHub Enterprise managing multi-vendor agent deployments (Devin, Copilot, third-party).
* **Interfaces & APIs:** GitHub GraphQL & REST Agent Tasks API (`POST /repos/{owner}/{repo}/issues` assignee triggers, Copilot Extensions API).

#### 13. E2B (Code Interpreting & Agent Sandbox Cloud)
* **Architecture:** Cloud platform providing secure isolated microVM sandboxes for AI agents to run terminal commands, code, and web browsers.
* **REST & SDK APIs:** E2B Control Plane (`POST https://api.e2b.dev/sandboxes`, `DELETE /sandboxes/{sandboxID}`) + Sandbox Envd gRPC/HTTP port `49983` (`/process`, `/filesystem`).
* **SDK Invocation (Python):**
  ```python
  from e2b_code_interpreter import Sandbox
  with Sandbox() as sandbox:
      execution = sandbox.run_code("print('Running inside E2B agent sandbox')")
  ```

#### 14. Modal Labs
* **Architecture:** Serverless GPU/CPU container infrastructure designed for running multi-agent workflows and model inference with sub-second cold starts.
* **APIs:** Modal gRPC Control Plane (`api.modal.com`), `@modal.fastapi_endpoint` web hooks, and `modal.Sandbox.create()` programmatic execution API.

#### 15. Daytona & Gitpod Flex
* **Architecture:** Standardized developer environment managers creating ephemeral cloud dev-workspaces accessible by autonomous coding agents.
* **APIs:** Daytona REST & Python/TS SDK (`POST /api/workspace`, `toolbox.process.code_run()`, `toolbox.fs.upload_file()`) and Gitpod Flex gRPC/REST Environment API.

#### 16. GitLab Duo & Agentic Duo Workflow
* **Architecture:** Native GitLab platform agent executing multi-turn repository audits, vulnerability remediation, and automated MR generation.
* **APIs:** GitLab GraphQL AI API (`aiAction` mutation), Duo Workflow gRPC Service, and GitLab AI Gateway (`https://cloud.gitlab.com/ai/v1/proxy`).

---

## 7. Agentic Proxy, Gateway & Router Providers

### 7.1 Exhaustive Catalog with Endpoints, Setup, Routing, and Slash Directives

#### 1. Bifrost (Maxim AI)
* **Architecture:** High-throughput Rust/Go gateway connecting agents to models and MCP servers with automated rate-limit smoothing.
* **API Endpoints:** `POST /v1/chat/completions`, `POST /v1/messages`, `POST /v1/embeddings`, `GET /metrics`.
* **CLI Setup & Execution:**
  ```bash
  bifrost-server --port 8080 --redis redis://localhost:6379
  ```
* **Routing Directives:** Configured via HTTP headers:
  * `x-bifrost-fallback-providers: anthropic,openai,groq`
  * `x-bifrost-cache-ttl: 3600`

#### 2. OpenRouter
* **Architecture:** Multi-provider API routing layer translating requests across 100+ AI backends with auto-fallbacks and cost optimization.
* **HTTP Endpoints:** `POST https://openrouter.ai/api/v1/chat/completions`, `POST /api/v1/completions`, `GET /api/v1/models`, `GET /api/v1/generation?id={id}`, `GET /api/v1/auth/key`.
* **Custom Headers & Parameters:**
  * `HTTP-Referer: https://your-site.com`
  * `X-Title: My Agent Application`
  * Model Routing Array: `"model": "anthropic/claude-3.7-sonnet,openai/o3-mini"` (automatically falls back if primary is throttled or down).
  * Auto-routing alias: `"model": "openrouter/auto"` (picks lowest cost/latency provider dynamically).

#### 3. Portkey
* **Architecture:** Enterprise AI gateway and control plane with load balancing, caching, and budget limits.
* **API Endpoints:** `POST https://api.portkey.ai/v1/chat/completions`, `POST /v1/embeddings`, `POST /v1/prompts/{id}/completions`, `GET /v1/logs`.
* **Configuration Headers:**
  * `x-portkey-api-key: $PORTKEY_API_KEY`
  * `x-portkey-virtual-key: $VIRTUAL_KEY`
  * `x-portkey-config: pc-retry-fallback-policy-123`
  * `x-portkey-cache: semantic`

#### 4. LiteLLM Proxy
* **Architecture:** Python/AsyncIO proxy translating 100+ LLM APIs to the OpenAI format with Redis-backed rate limiting.
* **API Endpoints:** `POST /v1/chat/completions`, `POST /v1/messages`, `POST /v1/embeddings`, `POST /key/generate`, `POST /team/new`, `GET /spend/logs`, `GET /health`.
* **CLI Commands:**
  ```bash
  litellm --config config.yaml --port 4000 --num_workers 8
  litellm --test
  ```
* **In-Chat Directives (LiteLLM Web UI):**
  * `/models`: Enumerate active backend routing keys.
  * `/spend`: Inspect current user/key token spend.

#### 5. Cloudflare AI Gateway
* **Architecture:** Edge AI proxy running on Cloudflare's network with edge caching and request logging.
* **Base & Universal Endpoints:**
  * Provider-specific: `https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_slug}/{provider}`
  * Universal fallback endpoint: `POST https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_slug}` (accepts array of provider payloads)
  * OpenAI-compatible unified endpoint: `POST https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_slug}/compat/chat/completions`
* **Configuration Headers:**
  * `cf-aig-cache: true`
  * `cf-aig-skip-cache: false`
  * `cf-aig-cache-ttl: 86400`

#### 6. Helicone
* **Architecture:** Open-source proxy with observability, cost monitoring, and automated caching.
* **Base Endpoints:** `https://oai.helicone.ai/v1` (OpenAI), `https://anthropic.helicone.ai/v1` (Anthropic), `https://gateway.helicone.ai` (Unified).
* **Configuration Headers:**
  * `Helicone-Auth: Bearer $HELICONE_API_KEY`
  * `Helicone-Cache-Enabled: true`
  * `Helicone-RateLimit-Policy: 1000;w=60;u=user_123`

#### 7. Martian
* **Architecture:** Algorithmic prompt router sending simple queries to smaller models and complex tasks to frontier reasoning models.
* **API Endpoints:** `POST https://api.withmartian.com/v1/chat/completions`, `GET /v1/models`.
* **Model Parameter:** `"model": "martian/auto"`

#### 8. Vercel AI Gateway
* **Architecture:** Edge infrastructure optimized for streaming responses to the Vercel AI SDK with automated region failover.
* **API Endpoints:** `POST https://ai-gateway.vercel.sh/v1/chat/completions`, `GET /v1/models` (authenticated via `AI_GATEWAY_API_KEY` or Vercel OIDC).

#### 9. BerriAI
* **Architecture:** Enterprise proxy deployment built on LiteLLM for isolated VPC tenant quota governance.
* **API Endpoints:** OpenAI-compatible `/v1/chat/completions` + Tenant Admin REST API (`/key/generate`, `/user/update`).

#### 10. APIClaw
* **Architecture:** High-speed gateway featuring anomaly detection, automated circuit breakers, and predictive rate-limit throttling.
* **API Endpoints:** `POST /v1/chat/completions`, `GET /v1/circuit-breakers/status`.

#### 11. Langfuse
* **Architecture:** Open-source LLM observability, prompt management, and tracing gateway.
* **API Endpoints & Integration:** Ingestion API (`POST https://cloud.langfuse.com/api/public/ingestion`), Prompts API (`GET /api/public/v2/prompts/{name}`), and OpenTelemetry OTLP endpoint (`/api/public/otel/v1/traces`).

#### 12. Unify AI
* **Architecture:** Dynamic router querying real-time performance metrics to route prompts to the fastest or cheapest provider hosting that model.
* **API Endpoints:** `POST https://api.unify.ai/v0/chat/completions`, `GET /v0/models`, `GET /v0/endpoints`.
* **Model ID Format:** `model_name@provider_name` (e.g., `llama-3.3-70b@together-ai` or `llama-3.3-70b@groq`).

#### 13. Kong AI Gateway
* **Architecture:** Open-source enterprise API gateway with AI plugins providing multi-LLM routing, semantic caching, token rate limiting, and prompt guardrails.
* **API Endpoints:** Kong Proxy (`POST /v1/chat/completions` via `ai-proxy` plugin) & Admin API (`POST :8001/services/{id}/plugins`).
* **CLI Setup:** `deck gateway sync kong.yaml`

#### 14. Envoy AI Gateway (CNCF)
* **Architecture:** Cloud-native proxy layer built on Envoy for enterprise LLM request routing, load balancing, and traffic shadowing.
* **API Endpoints:** Unified OpenAI `/v1/chat/completions` & Anthropic `/v1/messages` ingress routes configured via Kubernetes CRDs (`AIGatewayRoute`, `AIServiceBackend`).

#### 15. TrueFoundry AI Gateway
* **Architecture:** Unified control plane providing rate limiting, cost tracking, access policies, and model routing across cloud and local deployments.
* **API Endpoints:** `POST /api/llm/v1/chat/completions`, `POST /api/llm/v1/embeddings`, `GET /api/llm/v1/models`.

#### 16. One API / New API
* **Architecture:** High-performance Go-based multi-provider management interface and billing proxy standardizing disparate LLM APIs into OpenAI format.
* **API Endpoints:** `POST /v1/chat/completions`, `POST /v1/messages`, `POST /v1/images/generations`, `GET /api/token/`, `GET /api/channel/`.

---

## 8. Technical Quotas, Limits, Rate-Limit Headers & Error Payloads

### 8.1 Quantitative Quota Categories

1. **RPM (Requests Per Minute):** Fixed or sliding window count of discrete HTTP/WebSocket requests, regardless of token size.
2. **RPD (Requests Per Day):** 24-hour request limit to prevent automated abuse on free/low tiers.
3. **TPM (Tokens Per Minute):** Aggregate sum of prompt input tokens plus generated completion tokens in a rolling 60-second window.
4. **TPD (Tokens Per Day):** Aggregate token ceiling over a 24-hour cycle.
5. **Concurrency Limits:** Maximum simultaneous in-flight HTTP connections or open WebSockets permitted before receiving immediate `429 Too Many Requests`.

---

### 8.2 Quota Limits Matrix Across Major Cloud Providers

| Provider / Tier | Requests Per Minute (RPM) | Tokens Per Minute (TPM) | Requests Per Day (RPD) | Concurrency Limit |
| :--- | :--- | :--- | :--- | :--- |
| **Google AI Studio (Free Tier)** | 15 RPM (Flash) / 2 RPM (Pro) | 1,000,000 TPM (Flash) / 32,000 TPM (Pro) | 1,500 RPD (Flash) / 50 RPD (Pro) | 1--2 in-flight |
| **Google AI Studio (Pay-Go Tier 1)** | 1,000 RPM (Flash) / 360 RPM (Pro) | 4,000,000 TPM (Flash) / 2,000,000 TPM (Pro) | Unlimited | Up to 100 in-flight |
| **Google AI Studio (Pay-Go Tier 2+)** | 4,000+ RPM | 10,000,000+ TPM | Unlimited | Custom |
| **OpenAI (Tier 1: $5 paid)** | 500 RPM | 30,000 TPM (`gpt-4o`) / 10,000 TPM (`o3-mini`) | Unlimited | 5 in-flight |
| **OpenAI (Tier 4: $250 paid)** | 10,000 RPM | 1,000,000 TPM (`gpt-4o`) / 400,000 TPM (`o3-mini`) | Unlimited | 100 in-flight |
| **OpenAI (Tier 5: $1,000+ paid)** | 15,000 RPM | 4,000,000 TPM (`gpt-4o`) / 1,000,000 TPM (`o3-mini`) | Unlimited | 500 in-flight |
| **Anthropic (Tier 1: $5 deposit)** | 50 RPM | 20,000 TPM (`claude-3-7-sonnet`) | Unlimited | 2 in-flight |
| **Anthropic (Tier 4: $400 deposit)** | 4,000 RPM | 400,000 TPM (`claude-3-7-sonnet`) | Unlimited | 80 in-flight |
| **AWS Bedrock (On-Demand Default)** | 250--1,000 RPM (model/region dependent) | 200,000--2,000,000 TPM | Unlimited | Account Service Quota |
| **Azure OpenAI (Standard Pay-Go)** | 600--6,000 RPM (based on provisioned TPM) | 100,000 to 1,000,000+ TPM | Unlimited | Dynamic per deployment |
| **DeepSeek API (Standard Pay-Go)** | 1,000 RPM | Dynamic cluster capacity throttling | Unlimited | Dynamic based on cluster load |
| **Groq Cloud (Developer Free)** | 30 RPM | 6,000 to 30,000 TPM | 14,400 RPD | 1--3 in-flight |
| **Fireworks AI (Developer / Pay-Go)** | 600 RPM (default) / 6,000+ RPM (scaled) | 250,000 to 2,500,000 TPM | Unlimited | 50+ in-flight |
| **Mistral AI (Experiment / Scale)** | 60 RPM (Free) / 1,000+ RPM (Scale) | 500,000 to 2,000,000 TPM | Unlimited | 10--100 in-flight |

---

### 8.3 Rate-Limit Header Specifications Across Providers

#### 1. Standard OpenAI / Groq / Together / OpenRouter Rate-Limit Headers:
```http
x-ratelimit-limit-requests: 10000
x-ratelimit-limit-tokens: 2000000
x-ratelimit-remaining-requests: 9989
x-ratelimit-remaining-tokens: 1984210
x-ratelimit-reset-requests: 66ms
x-ratelimit-reset-tokens: 473ms
```

#### 2. Anthropic Rate-Limit Headers (Separate Input & Output Token Buckets):
```http
anthropic-ratelimit-requests-limit: 4000
anthropic-ratelimit-requests-remaining: 3995
anthropic-ratelimit-requests-reset: 2026-03-31T20:00:00Z
anthropic-ratelimit-tokens-limit: 400000
anthropic-ratelimit-tokens-remaining: 385000
anthropic-ratelimit-tokens-reset: 2026-03-31T20:00:00Z
anthropic-ratelimit-input-tokens-limit: 320000
anthropic-ratelimit-input-tokens-remaining: 310000
anthropic-ratelimit-output-tokens-limit: 80000
anthropic-ratelimit-output-tokens-remaining: 75000
retry-after: 3
```

#### 3. AWS Bedrock & Azure OpenAI Rate-Limit Headers:
```http
# AWS Bedrock
x-amzn-errortype: ThrottlingException
x-amzn-requestid: 8a1b2c3d-4e5f-6789-abcd-ef0123456789

# Azure OpenAI
x-ratelimit-remaining-requests: 599
x-ratelimit-remaining-tokens: 98500
retry-after-ms: 450
```

#### 4. Google Gemini / Vertex AI Error Payload (HTTP 429):
```json
{
  "error": {
    "code": 429,
    "message": "Resource has been exhausted (e.g. check quota).",
    "status": "RESOURCE_EXHAUSTED",
    "details": [
      {
        "@type": "type.googleapis.com/google.rpc.QuotaFailure",
        "violations": [
          {
            "subject": "project:YOUR_PROJECT_ID",
            "description": "Quota exceeded for quota metric 'GenerateContent requests per minute' and limit 'GenerateContent requests per minute per user'"
          }
        ]
      },
      {
        "@type": "type.googleapis.com/google.rpc.ErrorInfo",
        "reason": "RATE_LIMIT_EXCEEDED",
        "domain": "generativelanguage.googleapis.com"
      }
    ]
  }
}
```

---

## 9. Enforcement Algorithms & Engineering Implementations

### 9.1 Distributed Token Bucket Algorithm (Redis + Lua Script)

```lua
-- KEYS[1]: Token Bucket Key (e.g., ratelimit:user_123:tokens)
-- ARGV[1]: Max Capacity (bucket size, e.g., TPM limit)
-- ARGV[2]: Refill Rate (tokens added per millisecond)
-- ARGV[3]: Current Epoch Time in Milliseconds
-- ARGV[4]: Cost (Tokens requested for current prompt + expected completion)

local bucket = redis.call('HMGET', KEYS[1], 'tokens', 'last_updated')

local capacity = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4])

local current_tokens
local last_updated

if not bucket[1] then
    current_tokens = capacity
    last_updated = now
else
    current_tokens = tonumber(bucket[1])
    last_updated = tonumber(bucket[2])
end

-- Calculate leaked/refilled tokens based on elapsed time
local delta = math.max(0, now - last_updated)
local refilled = delta * refill_rate
current_tokens = math.min(capacity, current_tokens + refilled)
last_updated = now

if current_tokens >= cost then
    current_tokens = current_tokens - cost
    redis.call('HMSET', KEYS[1], 'tokens', current_tokens, 'last_updated', last_updated)
    redis.call('PEXPIRE', KEYS[1], math.ceil(capacity / refill_rate))
    return {1, math.floor(current_tokens)} -- 1 = Allowed
else
    redis.call('HMSET', KEYS[1], 'tokens', current_tokens, 'last_updated', last_updated)
    local wait_time = math.ceil((cost - current_tokens) / refill_rate)
    return {0, wait_time} -- 0 = Denied, wait_time in ms
end
```

---

### 9.2 Production Client-Side Retry Architecture (Python)

```python
import asyncio
import random
import time
from typing import Any, Dict, List, Optional
import httpx

class RobustAgentDispatcher:
    def __init__(
        self,
        primary_endpoint: str,
        fallback_endpoints: List[str],
        api_keys: Dict[str, str],
        base_backoff: float = 1.0,
        max_backoff: float = 60.0,
        max_retries: int = 5
    ):
        self.endpoints = [primary_endpoint] + fallback_endpoints
        self.api_keys = api_keys
        self.base_backoff = base_backoff
        self.max_backoff = max_backoff
        self.max_retries = max_retries
        self.client = httpx.AsyncClient(timeout=120.0)

    async def execute_with_failover(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        """
        Executes generation requests across an ordered fallback ladder of endpoints,
        handling HTTP 429 with full-jitter exponential backoff.
        """
        last_exception = None

        for endpoint in self.endpoints:
            api_key = self.api_keys.get(endpoint, "")
            headers = {
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json"
            }
            
            for attempt in range(self.max_retries):
                try:
                    response = await self.client.post(
                        endpoint,
                        json=payload,
                        headers=headers
                    )
                    
                    if response.status_code == 200:
                        return response.json()

                    elif response.status_code == 429:
                        retry_after_header = response.headers.get("retry-after")
                        retry_after_ms = response.headers.get("retry-after-ms")
                        if retry_after_header:
                            sleep_duration = float(retry_after_header)
                        elif retry_after_ms:
                            sleep_duration = float(retry_after_ms) / 1000.0
                        else:
                            # Full Jitter Backoff Formula: Uniform(0, Min(MaxBackoff, Base * 2^Attempt))
                            backoff = min(self.max_backoff, self.base_backoff * (2 ** attempt))
                            sleep_duration = random.uniform(0, backoff)
                        
                        await asyncio.sleep(sleep_duration)
                        continue

                    elif response.status_code in [500, 502, 503, 504]:
                        backoff = min(self.max_backoff, self.base_backoff * (2 ** attempt))
                        await asyncio.sleep(random.uniform(0, backoff))
                        continue

                    else:
                        response.raise_for_status()

                except (httpx.ConnectError, httpx.ReadTimeout) as e:
                    last_exception = e
                    backoff = min(self.max_backoff, self.base_backoff * (2 ** attempt))
                    await asyncio.sleep(random.uniform(0, backoff))
                    continue

        raise RuntimeError(f"All providers exhausted. Last error: {last_exception}")
```
