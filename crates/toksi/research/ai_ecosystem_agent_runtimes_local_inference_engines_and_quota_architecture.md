# Comprehensive Reference: AI Providers, Local Inference Engines, Agent Ecosystems, Protocols, and Quota Enforcement Architecture

---

## 1. Foundation Cloud AI Model Providers & Inference Engines

### 1.1 Technical API Endpoint Specifications

| Provider | Base URL (HTTP) | Realtime / WebSocket URL | Supported Versions | Authentication Scheme | Key Models Available (Production & Frontier) | Thinking / Extended Reasoning Parameters |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Google Gemini API (AI Studio)** | `https://generativelanguage.googleapis.com` | `wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.<version>.GenerativeService.BidiGenerateContent` | `v1`, `v1alpha`, `v1beta`, `v1beta1`, `v1beta2`, `v1beta3`, `v1main` | Query param: `?key=$GEMINI_API_KEY`<br>Header: `x-goog-api-key: $GEMINI_API_KEY`<br>Bearer token: `Authorization: Bearer $(gcloud auth print-access-token)` | `gemini-2.5-pro`<br>`gemini-2.5-flash`<br>`gemini-2.5-flash-lite`<br>`gemini-2.0-flash`<br>`gemini-2.0-flash-lite`<br>`gemini-2.0-pro-exp-02-05`<br>`gemini-1.5-pro-002`<br>`gemini-1.5-flash-002`<br>`gemini-3-pro-preview`<br>`gemini-3-flash-preview` | `generationConfig`: `{"thinkingConfig": {"thinkingBudget": <int_tokens>}}` (`0` turns off thinking, `-1` auto-tunes, positive integer sets ceiling up to 24,576) |
| **Google Cloud Vertex AI** | `https://{REGION}-aiplatform.googleapis.com` | `wss://{REGION}-aiplatform.googleapis.com/ws/google.cloud.aiplatform.<version>.LlmBidiService/BidiGenerateContent` | `v1`, `v1beta1` | Header: `Authorization: Bearer $(gcloud auth print-access-token)`<br>Project URI prefix: `projects/{PROJECT_ID}/locations/{REGION}/publishers/google/models/{MODEL_ID}` | `gemini-2.5-pro@latest`<br>`gemini-2.5-flash@latest`<br>`gemini-2.0-flash-001`<br>`claude-3-7-sonnet@20250219`<br>`claude-3-5-sonnet-v2@20241022` | `generationConfig.thinkingConfig.thinkingBudget` for Gemini; Anthropic native `thinking.budget_tokens` payload for Claude on Vertex |
| **OpenAI** | `https://api.openai.com` | `wss://api.openai.com/v1/realtime` | `v1` | Header: `Authorization: Bearer $OPENAI_API_KEY`<br>Header: `OpenAI-Organization: $ORG_ID`<br>Header: `OpenAI-Project: $PROJECT_ID` | `o3`<br>`o3-mini`<br>`o1`<br>`o1-mini`<br>`gpt-4.5-preview`<br>`gpt-4o`<br>`gpt-4o-mini` | `reasoning_effort`: `"low"` \| `"medium"` \| `"high"` (for `o1`, `o3`, `o3-mini`). Output breakdown in `usage.completion_tokens_details.reasoning_tokens` |
| **Anthropic** | `https://api.anthropic.com` | N/A (SSE over HTTP `POST /v1/messages` with `stream: true`) | `v1` | Header: `x-api-key: $ANTHROPIC_API_KEY`<br>Header: `anthropic-version: 2023-06-01`<br>Beta Header: `anthropic-beta: output-128k-2025-02-19` | `claude-3-7-sonnet-20250219`<br>`claude-3-5-sonnet-20241022`<br>`claude-3-5-haiku-20241022`<br>`claude-3-opus-20240229` | `thinking`: `{"type": "enabled", "budget_tokens": <int>}` (min 1,024, max $\le$ `max_tokens`). Raw reasoning emitted in `thinking` block. Set `type: "disabled"` for fast execution |
| **xAI** | `https://api.x.ai` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $XAI_API_KEY` | `grok-3`<br>`grok-3-mini`<br>`grok-2-1212`<br>`grok-2-vision-1212` | `reasoning_format`: `"raw"` \| `"parsed"` or `thinking`: `{"budget_tokens": <int>}` on Grok-3 checkpoints |
| **DeepSeek API** | `https://api.deepseek.com` | N/A (SSE HTTP streaming) | `v1` (OpenAI compatible) | Header: `Authorization: Bearer $DEEPSEEK_API_KEY` | `deepseek-reasoner` (R1)<br>`deepseek-chat` (V3) | `deepseek-reasoner` generates internal CoT streamed via `reasoning_content` parameter in delta responses |
| **Groq Cloud** | `https://api.groq.com/openai` | N/A (OpenAI compatible SSE) | `v1` | Header: `Authorization: Bearer $GROQ_API_KEY` | `deepseek-r1-distill-llama-70b`<br>`llama-3.3-70b-versatile`<br>`llama-3.1-8b-instant`<br>`qwen-2.5-coder-32b` | `reasoning_format`: `"parsed"` \| `"raw"` on DeepSeek R1 models; ultra-high speed LPU inference |
| **Cerebras Cloud** | `https://api.cerebras.ai` | N/A (OpenAI compatible SSE) | `v1` | Header: `Authorization: Bearer $CEREBRAS_API_KEY` | `llama-3.3-70b`<br>`llama-3.1-70b`<br>`llama-3.1-8b` | Instant execution on Wafer-Scale Engine; explicit instructions passed in system prompt |
| **Alibaba Cloud DashScope (Qwen)** | `https://dashscope-intl.aliyuncs.com/compatible-mode` | `wss://dashscope-intl.aliyuncs.com/api-ws/v1/inference` | `v1` | Header: `Authorization: Bearer $DASHSCOPE_API_KEY` | `qwen-max-latest`<br>`qwen-plus-latest`<br>`qwen2.5-coder-32b-instruct`<br>`qwq-32b-preview` | `qwq-32b-preview` streams reasoning steps dynamically in delta tokens |
| **Together AI** | `https://api.together.xyz` | `wss://api.together.xyz/v1/realtime` | `v1` | Header: `Authorization: Bearer $TOGETHER_API_KEY` | `deepseek-ai/DeepSeek-R1`<br>`deepseek-ai/DeepSeek-V3`<br>`meta-llama/Llama-3.3-70B-Instruct-Turbo` | Supports OpenAI standard `reasoning_effort` and outputs `usage.reasoning_tokens` |
| **Mistral AI** | `https://api.mistral.ai` | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $MISTRAL_API_KEY` | `mistral-large-2411`<br>`codestral-2501`<br>`ministral-8b-latest` | Tool calling orchestration and structured JSON outputs |
| **Cohere** | `https://api.cohere.com` | N/A (SSE HTTP streaming) | `v1`, `v2` | Header: `Authorization: Bearer $CO_KEY` | `command-r-plus-08-2024`<br>`command-r-08-2024` | Native tool execution with citations via `tools` array parameter |
| **Perplexity AI** | `https://api.perplexity.ai` | N/A (SSE HTTP streaming) | `v1` | Header: `Authorization: Bearer $PERPLEXITY_API_KEY` | `sonar-reasoning-pro`<br>`sonar-reasoning`<br>`sonar-pro` | Inline citation retrieval and integrated multi-step search verification |

---

### 1.2 Google Gemini API Version Taxonomy

```
https://generativelanguage.googleapis.com/{API_VERSION}/models/{MODEL_ID}:{METHOD}
```

* **`v1` (Stable GA):** Strict Long-Term Support (LTS). High backwards compatibility for simple text, function calling, and structured JSON outputs. Excludes bleeding-edge thinking configs and live bidi channels.
* **`v1alpha` (Bleeding Edge Alpha):** Internal developer releases, low-level tokenizer introspection, prototype multimodal bindings, dynamic runtime memory flags.
* **`v1beta` (Public Developer Preview):** Standard for advanced agents. Supports full `thinkingConfig`, groundings with Google Search, code execution sandboxes, Context Caching, and Bidi WebSocket Streaming.
* **`v1beta1` (Enterprise & GenAI SDK Channel):** Targeted channel used by official Google Gen AI SDKs and Vertex bridges. Implements `continuationToken` for extended multi-step generations.
* **`v1beta2` (Legacy PaLM Transition):** Legacy endpoint compatibility layer for historical PaLM 2 models (`chat-bison-001`, `text-bison-001`).
* **`v1beta3` (Multi-Agent & Tooling Sandbox):** Experimental channel for asynchronous batch tasks, multi-agent evaluation runtimes, and specialized sandboxed code evaluation.
* **`v1main` (Internal Monorepo HEAD):** Real-time mapping to internal Google development trunk. Accessible by internal Google services and whitelisted enterprise projects.

---

## 2. Local Inference Engines, Runtimes, and Sockets

### 2.1 Quick Socket & Port Reference Matrix

| Engine / Platform | Default Host & Port | Native API Prefix | OpenAI Compatible (`/v1/`) | Primary Focus & Model Format |
| :--- | :--- | :--- | :--- | :--- |
| **Ollama** | `http://localhost:11434` | `/api/` | Yes | Universal CLI / daemon; GGUF |
| **llama.cpp (`llama-server`)** | `http://localhost:8080` | `/completion` | Yes | Bare-metal C/C++; GGUF |
| **LM Studio** | `http://localhost:1234` | None | Yes | Desktop workstation GUI; GGUF, MLX |
| **Jan AI** | `http://localhost:1337` | `/nitro/` | Yes | Privacy-first desktop client; GGUF |
| **GPT4All** | `http://localhost:4891` | None | Yes | CPU-optimized local client; GGUF |
| **Msty AI** | `http://localhost:10240` | None | Yes | Multi-model side-by-side workstation; GGUF |
| **Kobold.cpp** | `http://localhost:5001` | `/api/v1/` | Yes | Roleplay & creative writing; GGUF |
| **llamafile (Mozilla)** | `http://localhost:8080` | `/completion` | Yes | Polyglot single executable; GGUF |
| **LocalAI** | `http://localhost:8080` | None | Yes (LLM + Audio + Img) | Multi-modal containerized proxy; Multi |
| **vLLM** | `http://localhost:8000` | None (`/metrics`) | Yes | High-throughput production; Safetensors, AWQ, FP8 |
| **SGLang** | `http://localhost:30000`| `/generate` | Yes | RadixAttention for fast agents/JSON; Safetensors |
| **TGI (Hugging Face)** | `http://localhost:8080` | `/generate` | Yes | Production container; Safetensors, AWQ, GPTQ |
| **LMDeploy** | `http://localhost:23333`| None | Yes | TurboMind C++ backend; W4A16, KV 4-bit |
| **TabbyAPI (ExLlamaV2)**| `http://localhost:5000` | None | Yes | Ultra-fast token gen on NVIDIA; EXL2, GPTQ |
| **MLC-LLM** | `http://localhost:8000` | None | Yes | Cross-platform TVM compiler; Vulkan, WebGPU |
| **Exo** | `http://localhost:52415`| None | Yes | Distributed peer-to-peer cluster; GGUF, MLX |
| **FastChat** | `http://localhost:8000` | None | Yes | Distributed multi-worker serving; Hugging Face |
| **Triton + TensorRT-LLM**| `http://localhost:8000` | KServe v2 (`/v2/`) | Yes (via gateway) | Maximum throughput on NVIDIA Tensor Cores |
| **Text-Generation-WebUI**| `http://localhost:5000` | None | Yes (`--api`) | Modular testing & fine-tuning workbench |
| **AnythingLLM** | `http://localhost:3001` | `/api/v1/` | Yes (workspace proxy) | Document ingestion & built-in RAG pipelines |
| **LiteLLM Proxy** | `http://localhost:4000` | None | Yes | Load balancing, rate limiting, and unified routing |

---

### 2.2 CLI Execution & In-Chat Commands for Local Inference Engines

#### 1. Ollama
* **CLI Startup & Management Commands:**
  ```bash
  # Start the background daemon
  ollama serve

  # Run a model interactively (pulls automatically if missing)
  ollama run llama3.3:70b-instruct-q4_K_M

  # Model lifecycle management
  ollama list                     # List locally installed models
  ollama ps                       # List models currently loaded in VRAM
  ollama pull qwen2.5-coder:32b   # Pull checkpoint from library
  ollama rm codellama:7b          # Remove model from disk
  ollama cp llama3.2 my-custom    # Clone a local model tag
  ollama create custom-agent -f ./Modelfile  # Compile custom model from Modelfile
  ```
* **Interactive In-Chat Slash (`/`) Commands:**
  * `/set parameter <key> <val>`: Configure session parameters dynamically (e.g., `/set parameter temperature 0.2`, `/set parameter num_ctx 32768`).
  * `/set system <prompt>`: Dynamically override the active system instruction.
  * `/show info`: Display active model architecture, context window, quantization type, and stop tokens.
  * `/show license`: Output license information for the active model.
  * `/load <model>`: Switch model in the active session without restarting the terminal.
  * `/clear`: Clear conversation history and reset context memory.
  * `/bye` or `/exit`: Terminate session and release terminal prompt.
  * `/?` or `/help`: Display in-chat command manual.
  * `"""`: Start and terminate multi-line block text inputs.

#### 2. llama.cpp (`llama-server` and `llama-cli`)
* **CLI Startup Commands:**
  ```bash
  # Launch high-performance HTTP / OpenAI compatible server
  ./llama-server \
    -m ./models/DeepSeek-R1-Distill-Qwen-32B-Q4_K_M.gguf \
    --host 0.0.0.0 \
    --port 8080 \
    -c 32768 \
    -ngl 99 \
    --cont-batching \
    --parallel 4 \
    --cache-type-k q8_0 \
    --cache-type-v q4_0 \
    --metrics

  # Run direct terminal CLI single-shot or interactive session
  ./llama-cli \
    -m ./models/qwen2.5-coder-7b.gguf \
    -p "Write an async TCP server in Rust" \
    -n 2048 \
    --color \
    -ngl 33 \
    -i -r "User:"
  ```
* **Interactive CLI Escapes & Control Directives:**
  * `Ctrl+C`: Interrupt current generation stream while preserving prior conversation turns.
  * `User:`: Standard reverse prompt delimiter triggering input acquisition.
  * `/reset`: Reset the continuous KV cache context buffer.
  * `/dump`: Print KV cache slot status and token occupancy metrics to stderr.

#### 3. LM Studio
* **CLI Management Commands (`lms`):**
  ```bash
  lms server start --port 1234 --cors=true
  lms server stop
  lms server status
  lms load lmstudio-community/Meta-Llama-3.1-8B-Instruct-GGUF --gpu max --context-length 16384
  lms unload --all
  lms ls                          # List cached models on disk
  lms ps                          # List active loaded models in memory
  lms bootstrap                   # Install CLI environment wrappers
  ```
* **In-Chat Controls & Directives:**
  * System Prompt Override: Dynamic slot injection through the GUI sidebar.
  * Context Window Slider: Reallocates KV cache size dynamically on new queries.
  * `@model-id`: Multi-model chat panel allows side-by-side prompt routing to specific loaded weights.

#### 4. Kobold.cpp
* **CLI Startup Commands:**
  ```bash
  python koboldcpp.py \
    --model models/gemma-2-27b-it.Q4_K_M.gguf \
    --port 5001 \
    --usecublas \
    --gpulayers 99 \
    --contextsize 16384 \
    --smartcontext \
    --skiplauncher
  ```
* **Interactive In-Chat Directives & Modifiers:**
  * `::: memory <text> :::`: Injects non-expiring memory blocks directly into the top of the prompt buffer.
  * `::: authornote <text> :::`: Injects author tone guidelines $N$ tokens from the bottom of the prompt.
  * `/abort`: Instantly cancels in-flight continuous generation.
  * `/undo`: Reverts the most recent user and assistant interaction from memory.
  * `/redo`: Regenerates the last assistant completion with updated random seed.

#### 5. LocalAI
* **CLI & Container Commands:**
  ```bash
  # Docker container execution with full GPU acceleration
  docker run -p 8080:8080 --gpus all \
    -v $PWD/models:/build/models \
    -e DEBUG=true \
    -e THREADS=8 \
    localai/localai:latest-aio-gpu-nvidia-cuda-12

  # Standalone binary execution with pre-defined gallery configuration
  local-ai run hermes-2-pro-llama-3-8b
  ```
* **API Endpoints:** Standard `/v1/chat/completions`, `/v1/models`, `/v1/audio/transcriptions`, `/v1/images/generations`.

#### 6. TabbyAPI (ExLlamaV2)
* **CLI Startup Commands:**
  ```bash
  python main.py \
    --host 0.0.0.0 \
    --port 5000 \
    --model-dir /mnt/models/ \
    --model-name Qwen2.5-Coder-32B-Instruct-exl2-4.0bpw \
    --max-seq-len 32768 \
    --gpu-split 24,24
  ```
* **Control Flags:** Dynamic runtime loading via `/v1/model/load` and `/v1/model/unload` endpoints.

#### 7. vLLM
* **CLI Startup Commands:**
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
    --enable-chunked-prefill \
    --trust-remote-code \
    --api-key $INTERNAL_VLLM_KEY
  ```
* **Runtime Management Endpoints:**
  * `GET /health`: Cluster readiness probe.
  * `GET /metrics`: Native Prometheus scrapable metrics (KV cache usage, request queue depth).
  * `POST /v1/chat/completions`: Full OpenAI payload compatibility with in-flight continuous batching.

#### 8. SGLang
* **CLI Startup Commands:**
  ```bash
  python3 -m sglang.launch_server \
    --model-path mistralai/Mistral-Small-24B-Instruct-2501 \
    --host 0.0.0.0 \
    --port 30000 \
    --tp 2 \
    --mem-fraction-static 0.88 \
    --context-length 32768 \
    --enable-radix-attention \
    --grammar-backend xgrammar
  ```
* **Native High-Speed Endpoints:**
  * `POST /generate`: Low-overhead JSON generation bypass.
  * `POST /v1/chat/completions`: Standard OpenAI interface.

#### 9. Hugging Face TGI (Text Generation Inference)
* **CLI / Docker Launch Command:**
  ```bash
  docker run --gpus all --shm-size 1g -p 8080:80 \
    -v $PWD/data:/data \
    ghcr.io/huggingface/text-generation-inference:latest \
    --model-id deepseek-ai/DeepSeek-R1-Distill-Llama-70B \
    --num-shard 4 \
    --max-input-tokens 24000 \
    --max-total-tokens 32768 \
    --max-batch-prefill-tokens 32768
  ```
* **Key Endpoints:** `POST /generate`, `POST /generate_stream`, `POST /v1/chat/completions`.

#### 10. LMDeploy
* **CLI Startup Command:**
  ```bash
  lmdeploy serve api_server internlm/internlm2_5-20b-chat \
    --server-name 0.0.0.0 \
    --server-port 23333 \
    --tp 2 \
    --cache-max-entry-count 0.85 \
    --quant-policy 4
  ```

#### 11. FastChat (LMSYS)
* **CLI Launch Multi-Tier Cluster:**
  ```bash
  # Controller
  python3 -m fastchat.serve.controller --host 0.0.0.0 --port 21001

  # Model Worker
  python3 -m fastchat.serve.model_worker --model-path meta-llama/Meta-Llama-3-8B-Instruct --controller http://localhost:21001 --port 31000 --worker http://localhost:31000 --gpus 0

  # REST API Server
  python3 -m fastchat.serve.openai_api_server --controller-address http://localhost:21001 --host 0.0.0.0 --port 8000
  ```

#### 12. Exo
* **CLI Multi-Device Cluster Execution:**
  ```bash
  # Node 1 (Master / Coordinator)
  exo --broadcast-port 52415

  # Node 2 (Peer discovered automatically over Wi-Fi / Ethernet)
  exo --peer-ip 192.168.1.50
  ```
* **Exposed Interface:** Automatically creates a single unified virtual `http://localhost:52415/v1/chat/completions` spanning all interconnected devices.

---

## 3. Agentic CLI, Bridge, & Terminal Providers

### 3.1 Exhaustive Catalog with CLI Invocations and In-Chat Commands

#### 1. Claude Code (Anthropic)
* **Architecture:** Node.js native agentic orchestrator with bash execution, ripgrep file querying, tree-sitter indexing, and sub-agent task decomposition.
* **CLI Launch & Execution Commands:**
  ```bash
  # Start interactive REPL in the current repository
  claude

  # Direct single-task non-interactive execution
  claude -p "Implement exponential backoff in internal/transport/http.go and run tests"

  # Headless mode for CI/CD pipelines
  claude --headless -p "Analyze current branch changes and write a PR summary"

  # Manage configuration, model preferences, and telemetry
  claude config list
  claude config set verbose true
  claude config set model claude-3-7-sonnet-20250219
  claude update                   # Update to the latest CLI release
  ```
* **In-Chat Slash (`/`) Commands & Directive Flags:**
  * `/help`: Display operational guide and list available built-in tools.
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
* **CLI Invocations:**
  ```bash
  codex "Refactor the authentication middleware to use JWT standard"
  codex --model o3-mini --reasoning-effort high --approval auto "Fix SQL injection in db/user.py"
  codex diff                      # Review pending uncommitted changes synthesized by Codex
  codex apply                     # Apply changes to working directory
  ```
* **In-Chat Slash Commands:**
  * `/diff`: Output synthesized changes in standard git diff format before disk write.
  * `/accept`: Confirm and stage proposed edits.
  * `/reject`: Discard proposed edits and reset agent buffer.
  * `/retry`: Re-synthesize solution with an alternative reasoning pathway.

#### 3. Gemini CLI / Antigravity CLI (Google)
* **Architecture:** Cloud-linked CLI agent optimized for 2M+ token full-repository ingestion using Google AI Studio / Vertex AI contexts.
* **CLI Invocations:**
  ```bash
  gemini analyze --all            # Ingest entire workspace into long-context buffer
  gemini prompt "Identify dead code branches across all microservices"
  gemini fix --target src/main.rs --error-log ./target/build.log
  antigravity sync                # Sync local workspace state with cloud context cache
  antigravity chat --model gemini-2.5-pro --budget 8192
  ```
* **In-Chat Slash Commands:**
  * `/cache`: Display active Context Cache ID, expiration TTL, and cached token count.
  * `/files`: Print all project files mapped into the context payload.
  * `/grounding [on|off]`: Toggle Google Search dynamic grounding.
  * `/think [tokens]`: Adjust or inspect current `thinkingBudget`.

#### 4. Bifrost CLI (Maxim AI)
* **Architecture:** High-performance agent-to-gateway bridge, connecting CLI agents to virtualized MCP clusters and proxy routers.
* **CLI Invocations:**
  ```bash
  bifrost daemon start --port 9090 --config ./bifrost.yaml
  bifrost route add --pattern "gpt-*" --upstream "https://api.openai.com/v1"
  bifrost mcp bridge --server "github" --cmd "npx -y @modelcontextprotocol/server-github"
  bifrost status                  # Inspect routing tables, healthy upstreams, active tokens
  ```
* **In-Chat / Interactive Commands:**
  * `/route status`: Display health and latency of routed upstream endpoints.
  * `/mcp list`: Enumerate active MCP tools attached to the bridge session.
  * `/fallback <provider>`: Manually switch active inference route.

#### 5. Aider
* **Architecture:** Terminal pair programmer enforcing strict Git-centric operations, repository maps (tree-sitter), and automated commit synthesis.
* **CLI Invocations:**
  ```bash
  # Launch with Claude 3.7 Sonnet as architect and 3.5 Haiku as editor
  aider --model claude-3-7-sonnet-20250219 --editor-model claude-3-5-haiku-20241022

  # Launch pointing to local Ollama instance
  aider --model ollama/qwen2.5-coder:32b --api-base http://localhost:11434/v1

  # Launch with DeepSeek-R1 via OpenRouter
  aider --model openrouter/deepseek/deepseek-r1 --map-tokens 2048

  # Headless execution with auto-commit
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
* **CLI Invocations:**
  ```bash
  goose session                   # Start interactive terminal session
  goose session --name "feature-auth" --resume
  goose run --text "Run linting on the repo and fix all PEP8 violations"
  goose configure                 # Configure providers (Databricks, OpenAI, Ollama, Anthropic)
  goose plugins list              # List active MCP plugins
  ```
* **In-Chat Slash Commands:**
  * `/tools`: Enumerate all MCP tools registered to the Goose daemon.
  * `/plugin add <name>`: Register and initialize an external MCP provider.
  * `/context`: View injected environment context, active shell path, and system state.
  * `/reset`: Clear current session conversation buffers.

#### 7. Kilo CLI
* **Architecture:** High-performance, low-memory Rust coding agent with parallel multi-file fuzzy search and in-place code modification.
* **CLI Invocations:**
  ```bash
  kilo edit --prompt "Migrate from Tokio 0.2 to Tokio 1.0" --path ./crates
  kilo scan --security            # Run local AST vulnerability scan
  kilo interactive                # Launch terminal UI (TUI)
  ```
* **In-Chat Commands:**
  * `/focus <path>`: Narrow agent attention exclusively to a specific directory subtree.
  * `/review`: Print proposed edits side-by-side in terminal TUI.
  * `/revert`: Undo the last file modification batch.

#### 8. OpenCode
* **Architecture:** Open-source command-line software agent featuring sandboxed Docker execution loops and automated test verification.
* **CLI Invocations:**
  ```bash
  opencode run --task "Implement OAuth2 PKCE flow" --docker-sandbox
  opencode resume --session-id sess_987654
  ```
* **In-Chat Commands:**
  * `/test`: Execute current workspace tests inside the container.
  * `/inspect`: Inspect container filesystem diff against host repository.
  * `/stop`: Abort autonomous repair cycle.

#### 9. Qwen Code (Alibaba Cloud)
* **Architecture:** CLI development agent tuned for Qwen-2.5-Coder and QwQ models via DashScope or local vLLM.
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
* **CLI Invocations:**
  ```bash
  gh copilot suggest "Find all open TCP sockets listening on port 8080"
  gh copilot explain "tar -zxvf archive.tar.gz -C /opt/"
  gh copilot alias -- bash >> ~/.bashrc   # Inject shell shortcuts `??`, `git?`, `gh?`
  ```
* **In-Session Directives:**
  * Select `Execute command`: Directly run synthesized shell command.
  * Select `Revise command`: Provide natural language feedback to revise command.
  * Select `Copy to clipboard`: Stage snippet in system clipboard without executing.

#### 12. Amazon Q Developer CLI
* **Architecture:** Enterprise terminal agent bridging AWS Bedrock, local project context, and AWS infrastructure management.
* **CLI Invocations:**
  ```bash
  q chat                          # Launch interactive terminal session
  q login                         # Authenticate with AWS Builder ID / IAM Identity Center
  q translate                     # Convert natural language to AWS CLI commands
  q doctor                        # Inspect AWS credentials, profile, and local permissions
  ```
* **In-Chat Slash Commands:**
  * `/help`: Display operational commands.
  * `/clear`: Clear active conversation buffer.
  * `/dev`: Enter autonomous developer mode to generate multi-file code solutions.
  * `/transform`: Initiate code transformation (e.g., Java 8/11 to Java 17/21).
  * `/quit`: Exit the session.

#### 13. Warp AI / Agentic Terminal
* **Architecture:** Rust-native hardware-accelerated terminal emulator with integrated agentic sub-shells.
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
* **CLI Invocations:**
  ```bash
  mentat src/ tests/              # Launch session restricted to specific paths
  mentat --model gpt-4o
  ```
* **In-Chat Slash Commands:**
  * `/exclude <path>`: Remove paths from active context.
  * `/undo`: Revert file modifications made during the last turn.
  * `/commit`: Stage and commit accepted modifications.
  * `/config`: Inspect active configuration values.

#### 15. Plandex
* **Architecture:** Go-based terminal agent built for long-running architectural tasks using isolated sandboxed git branches.
* **CLI Invocations:**
  ```bash
  plandex new                     # Create a new planning plan
  plandex load                    # Load project files into plan context
  plandex tell "Refactor API routing to use Fiber v3"
  plandex changes                 # Review pending code modifications
  plandex apply                   # Apply changes directly to your real branch
  plandex discard                 # Revert and drop proposed plan
  ```
* **In-Chat Commands:**
  * `/context`: Print list of files and terminal outputs loaded in current plan.
  * `/diff`: Output full color diff across all modified files.
  * `/branch`: Switch plan branches to explore alternative architectures.

#### 16. Crush
* **Architecture:** Terminal UI (TUI) based fast code transformation agent.
* **CLI Invocations:**
  ```bash
  crush file.py "Add type hints to all functions"
  crush tui                       # Open interactive split-screen TUI
  ```
* **TUI Shortcuts & In-Chat Commands:**
  * `/apply`: Write changes to disk immediately.
  * `/reject`: Cancel pending changes.
  * `/diff`: Toggle inline unified diff preview.

#### 17. Devika
* **Architecture:** Open-source agentic alternative featuring web browsing, research planning, and sub-task division.
* **CLI Invocations:**
  ```bash
  python devika.py server         # Start backend orchestrator on port 1337
  python devika.py ui             # Start frontend client on port 3000
  ```
* **In-Chat Commands (UI & Headless):**
  * `/create <project>`: Initialize a new isolated workspace.
  * `/browse <url>`: Instruct browser agent to scrape and screenshot a webpage.
  * `/plan`: Force generation of a high-level execution checklist before coding.

#### 18. OpenHands CLI (OpenDevin)
* **Architecture:** Complete autonomous software engineer in Docker, evaluating tasks against SWE-bench benchmarks.
* **CLI Invocations:**
  ```bash
  docker run -it --pull=always \
    -e SANDBOX_USER_ID=$(id -u) \
    -e WORKSPACE_BASE=$PWD \
    -e LLM_API_KEY=$OPENAI_API_KEY \
    -p 3000:3000 \
    docker.all-hands.dev/all-hands-ai/openhands:latest
  ```
* **Interactive Commands:**
  * `openhands --task "Fix bug in issue #12"`: Direct task submission from terminal.
  * `/pause`: Pause running agent loop to inspect container state.
  * `/resume`: Continue autonomous step execution.
  * `/abort`: Kill container execution and dump state logs.

#### 19. SWE-agent
* **Architecture:** Benchmark evaluation harness and developer agent developed by Princeton NLP.
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
* **CLI Invocations:**
  ```bash
  gpt-engineer projects/my-new-app
  gpt-engineer --improve projects/my-new-app  # Iteratively enhance existing codebase
  ```
* **Directives:** Ingests `prompt` file, runs dependency synthesis, generates code, and prompts to execute scaffolding commands.

#### 21. Shell-Genie
* **Architecture:** Natural language CLI translation utility with safety verification before execution.
* **CLI Invocations:**
  ```bash
  shell-genie ask "Find all files larger than 100MB modified in the last 7 days"
  shell-genie init                # Configure backend (OpenAI, free community backend)
  ```
* **Interactive Flags:** Prompts user `[Y/n]` to confirm before executing any synthesized destructive command.

---

## 4. Agentic LSP & Protocol Providers

### 4.1 Exhaustive Catalog with Daemons, Protocols, and Command Sets

#### 1. Model Context Protocol (MCP) Daemons
* **Architecture:** Universal open standard for connecting AI agents to local resources, tools, and execution environments over JSON-RPC 2.0.
* **Common MCP Server Initialization Commands:**
  ```bash
  # Filesystem MCP Server
  npx -y @modelcontextprotocol/server-filesystem /path/to/allowed/dir

  # SQLite Database MCP Server
  npx -y @modelcontextprotocol/server-sqlite --db-path /path/to/database.db

  # PostgreSQL MCP Server
  npx -y @modelcontextprotocol/server-postgres postgresql://user:password@localhost:5432/mydb

  # GitHub MCP Server
  GITHUB_PERSONAL_ACCESS_TOKEN=$GITHUB_TOKEN npx -y @modelcontextprotocol/server-github

  # Git Local MCP Server
  npx -y @modelcontextprotocol/server-git --repository /path/to/repo

  # Memory / Knowledge Graph MCP Server
  npx -y @modelcontextprotocol/server-memory

  # Puppeteer Headless Browser MCP Server
  npx -y @modelcontextprotocol/server-puppeteer
  ```
* **Supported Protocol Methods (JSON-RPC):**
  * `tools/list`: Enumerate available tools, names, and JSON Schema definitions.
  * `tools/call`: Execute an agentic tool with parameterized arguments.
  * `resources/list` & `resources/read`: Inspect static contextual data feeds.
  * `prompts/list` & `prompts/get`: Pull standardized multi-shot prompt templates.

#### 2. Continue.dev LSP
* **Architecture:** Sidecar daemon and language server implementing LSP 3.17 to expose code intelligence to Neovim, VS Code, and JetBrains.
* **Daemon Management:**
  ```bash
  # Standalone CLI / Daemon invocation
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
* **Context Providers (`@` Mentations):**
  * `@Files`: Search and attach specific workspace files.
  * `@Codebase`: Vector search over entire repository via local embeddings.
  * `@Git`: Attach recent git diffs, commits, or branch status.
  * `@Docs`: Scrape and ingest indexed external documentation sets.
  * `@Terminal`: Grab the latest terminal contents and error streams.
  * `@URL`: Fetch live web page content into the prompt context.

#### 3. Cline
* **Architecture:** Autonomous coding agent extension host for VS Code, built natively on top of MCP servers with human-in-the-loop permissioning.
* **In-Chat Commands & Mode Switches:**
  * Mode: **Code Mode** (Full autonomy to read/write files and execute commands).
  * Mode: **Architect Mode** (Planning, diagramming, and requirements mapping without file modifications).
  * Mode: **Ask Mode** (Answering general questions about the workspace).
* **Direct In-Chat Control Actions:**
  * `Approve`: Grant permission for Cline to run a specific terminal command or edit a file.
  * `Deny`: Refuse action and provide natural language instructions.
  * `/clear`: Reset task conversation context.
  * `/history`: Browse and restore previous autonomous sessions.

#### 4. Roo Code (formerly Roo Clinic)
* **Architecture:** Multi-mode fork of Cline adding specialized system personalities and dynamic MCP tool configuration per workspace.
* **In-Chat Modes & Slash Commands:**
  * `/mode [code|architect|ask|test|custom]`: Dynamically switch operational persona and tool access policies.
  * `/mcp`: Open MCP server management console.
  * `/prompts`: Open custom prompt template manager.
  * `/compact`: Compress context history using semantic summarization.

#### 5. lsp-ai
* **Architecture:** Open-source language server acting as a bridge between any LSP-compatible editor (Neovim, Helix, Emacs) and local/cloud LLMs.
* **CLI Invocations:**
  ```bash
  lsp-ai --config /path/to/lsp-ai.json --stdio
  ```
* **Capabilities Exposed via LSP Protocol:**
  * `textDocument/completion`: Inline code completions.
  * `textDocument/codeAction`: Triggers LLM refactorings and doc generation.
  * `textDocument/formatting`: Automated code formatting via LLM rules.

#### 6. Tabby
* **Architecture:** Self-hosted code completion and search daemon exposing an LSP-compatible server and web UI.
* **CLI Invocations:**
  ```bash
  # Launch Tabby with GPU acceleration
  docker run -p 8080:8080 --gpus all \
    -v ~/.tabby:/data \
    tabbyml/tabby serve \
    --model StarCoder-1B \
    --device cuda
  ```
* **LSP Integration:** Direct endpoint mapping via `http://localhost:8080/v1/lsp`.

#### 7. FauxPilot LSP
* **Architecture:** Open-source backend compatible with the official GitHub Copilot extension, redirecting completions to local engines (TGI, vLLM).
* **CLI Invocations:**
  ```bash
  ./launch.sh --gpu 0 --model-name codellama/CodeLlama-13b-Instruct-hf
  ```

#### 8. Sourcegraph Cody Agent
* **Architecture:** Headless JSON-RPC daemon wrapping Cody's enterprise context retrieval, embeddings, and multi-repo graph queries.
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
* **CLI Invocations:**
  ```bash
  # Background daemon managed automatically by editor extensions
  ~/.supermaven/bin/sm-agent --stdio
  ```
* **In-Chat Commands (Supermaven Chat Preview):**
  * `/clear`: Clear active chat history.
  * `/model`: Switch between Standard and Pro inference tiers.

#### 10. OpenCtx
* **Architecture:** Open Context protocol standardizing context retrieval plugins (code search, devops metrics, database schemas) into editors.
* **CLI / Library Commands:**
  ```bash
  openctx test-provider @openctx/provider-github
  ```

---

## 5. Agentic Editors & Native IDE Providers

### 5.1 Exhaustive Catalog with Invocations, Shortcuts, and In-Chat Slash Commands

#### 1. Cursor (Anysphere)
* **Architecture:** VS Code fork featuring Merkle tree codebase indexing, shadow workspaces, fast speculative autocomplete, and an agent loop.
* **CLI Launch Command:**
  ```bash
  cursor .                        # Open current workspace
  cursor --diff file_a file_b     # Launch side-by-side visual diff
  cursor install-extension <id>  # Install VS Code compatible extension
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
* **CLI Launch Command:**
  ```bash
  windsurf .                      # Open current directory in Windsurf
  windsurf --version
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
* **CLI Command:**
  ```bash
  void .
  ```
* **In-Chat Slash Commands:**
  * `/edit`: Apply localized edits directly to the active editor buffer.
  * `/models`: Switch between local endpoints (Ollama, vLLM) and cloud APIs.
  * `/settings`: Configure custom headers, base URLs, and token limits.

#### 5. Zed (Zed AI)
* **Architecture:** Ultra-fast, GPU-accelerated code editor written in Rust with built-in assistant panels and slash-command context injections.
* **CLI Launch Command:**
  ```bash
  zed .                           # Open current repository
  zed --wait                      # Open as external editor waiting for file close
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
* **CLI Command:**
  ```bash
  trae .
  ```
* **In-Chat Slash Commands:**
  * `/build`: Run project build pipeline and diagnose any compilation errors.
  * `/fix`: Repair syntax errors in active buffer.
  * `/test`: Generate test cases.

#### 7. Replit Agent
* **Architecture:** Cloud-native software development agent operating within containerized Linux workspaces.
* **Interface:** Integrated chat panel inside Replit browser IDE.
* **In-Chat Commands & Action Directives:**
  * Natural language prompts trigger automated database creation, package installation, code editing, and server deployment.
  * Rollback Button: Direct UI action reverting container to snapshot taken before agent run.
  * `/checkpoint`: Force creation of a recoverable container snapshot.

#### 8. Positron AI (Posit)
* **Architecture:** Data science focused IDE (VS Code fork) with agents inspecting active R and Python kernel memory and dataframes.
* **CLI Command:**
  ```bash
  positron .
  ```
* **In-Chat Commands:**
  * `/data <df_name>`: Ingest summary schema and statistical profiling of active dataframe.
  * `/plot`: Synthesize visualization code using ggplot2 or matplotlib.
  * `/explain`: Break down complex statistical modeling routines.

#### 9. PearAI
* **Architecture:** Open-source AI code editor integrating Continue.dev core with curated plugins.
* **CLI Command:**
  ```bash
  pearai .
  ```
* **In-Chat Slash Commands:** Same as Continue.dev (`/edit`, `/comment`, `/test`, `/share`).

#### 10. Melty
* **Architecture:** Experimental VS Code fork tracking git history and terminal sessions to continuously suggest micro-refactorings.
* **CLI Command:**
  ```bash
  melty .
  ```
* **In-Chat Commands:**
  * `/diff`: Review inferred intent based on last 10 minutes of manual typing.
  * `/commit`: Generate an intention-aware git commit message.

#### 11. Amazon Q Developer for JetBrains & VS Code
* **Architecture:** IDE extension suite with chat, automated code reviews, and cross-version code transformations.
* **In-Chat Slash Commands:**
  * `/dev`: Start an autonomous multi-file development session.
  * `/transform`: Initiate automated Java language upgrade (Java 8/11 to 17/21).
  * `/review`: Run security and code-quality vulnerability scan on active project.
  * `/clear`: Clear active conversation context.

---

## 6. Agentic Services & Cloud Platforms

### 6.1 Exhaustive Catalog with Interfaces, Quotas, and Directives

#### 1. Cognition AI (Devin)
* **Architecture:** Autonomous cloud software engineer running in isolated Debian Linux micro-VMs with shell, browser, and code editor.
* **Interfaces:** Cloud Web UI, Slack bot integration, GitHub App webhook integration.
* **Task Submission & Chat Directives:**
  * Direct issue ingestion via `@Devin` mention in GitHub issues and pull requests.
  * In-chat directives:
    * `devin: pause`: Halts micro-VM execution.
    * `devin: resume`: Continues autonomous workflow.
    * `devin: restart`: Flushes current VM workspace and restarts from source commit.
* **Enforced Limits & Billing:** Billed per Devin Compute Unit (ACU) / wall-clock execution hour; strict concurrent runner caps (typically 1–10 parallel tasks per team).

#### 2. Factory AI (Code Droids)
* **Architecture:** Specialized autonomous enterprise cloud workers (Review Droid, Refactor Droid, Documentation Droid, Test Droid).
* **Interfaces:** GitHub/GitLab App integrations, Slack App, Web Control Plane.
* **PR Directives:**
  * `@factory review`: Forces deep semantic review on open PR.
  * `@factory refactor <target>`: Triggers architectural refactor of specified module.
  * `@factory test`: Generates missing edge-case unit tests.
* **Quotas:** Monthly seat tiers; token consumption caps per repository per 24-hour cycle.

#### 3. All-Hands AI (OpenHands Cloud)
* **Architecture:** Enterprise managed SaaS runtime for OpenHands (OpenDevin) agent swarms running inside microVM sandboxes.
* **Interfaces:** Web UI, REST API, GitHub Webhooks.
* **API Invocations:**
  ```bash
  curl -X POST "https://api.all-hands.dev/v1/tasks" \
    -H "Authorization: Bearer $ALLHANDS_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"prompt": "Fix test failures in PR #42", "repository": "org/repo"}'
  ```

#### 4. Augment Code
* **Architecture:** Enterprise code intelligence platform indexing 10M+ line monorepos using proprietary global semantic graphs.
* **Interfaces:** Native extensions for VS Code, JetBrains, and Neovim.
* **In-Chat Commands:**
  * Context search automatically traverses multi-repository boundaries without explicit tagging.
  * `/explain`: Monorepo-aware symbol and flow explanation.

#### 5. Magic.dev
* **Architecture:** Ultra-long-context software development models (LTM series, 100M+ tokens) capable of ingesting entire enterprise codebases.
* **Interfaces:** Private enterprise preview API and dedicated cloud developer consoles.

#### 6. Poolside AI
* **Architecture:** RL-driven coding models trained via Reinforcement Learning from Code Execution Feedback (RLCEF).
* **Interfaces:** Enterprise on-premise appliances and dedicated private cloud APIs.

#### 7. Sweep AI
* **Architecture:** Autonomous GitHub issue resolution agent converting issue tickets into tested, verified pull requests.
* **PR & Issue Directives:**
  * `Sweep: <instructions>` in issue description triggers automated PR creation.
  * Comments on PRs starting with `Sweep:` instruct the agent to update and push new commits to the branch.
* **Quotas:** Monthly PR creation limits per repository.

#### 8. CodeRabbit
* **Architecture:** AI code review agent running AST and reasoning passes over pull request diffs.
* **PR Directives:**
  * `@coderabbitai pause`: Temporarily disable reviews on PR.
  * `@coderabbitai resume`: Re-enable reviews.
  * `@coderabbitai review`: Force immediate review pass.
  * `@coderabbitai full review`: Ignore incremental cache and review entire PR from scratch.
  * `@coderabbitai summary`: Re-generate high-level PR walkthrough summary.
  * `@coderabbitai resolve`: Mark all CodeRabbit comments as resolved.

#### 9. Qodo (formerly CodiumAI)
* **Architecture:** Test synthesis and code integrity platform (Qodo Gen, Qodo Merge, Qodo Cover).
* **PR Directives (Qodo Merge):**
  * `/review`: Perform automated PR review.
  * `/describe`: Auto-generate PR description and changelog.
  * `/improve`: Suggest concrete code improvements and patches.
  * `/test`: Generate test cases for modified methods.

#### 10. Grit.io
* **Architecture:** Automated software migration agent running hybrid CodeQL AST queries and LLM transformations.
* **CLI Invocations:**
  ```bash
  grit apply <pattern_name>       # Apply migration pattern locally
  grit check                      # Identify deprecated syntax across repo
  grit list                       # Enumerate available standard migrations
  ```

#### 11. Ellipsis
* **Architecture:** Automated code review and bug fixing agent running inside ephemeral cloud containers.
* **PR Directives:**
  * In-line comment: `ellipsis: fix this race condition`: Directly commits a fix to the PR branch.
  * In-line comment: `ellipsis: test this`: Generates regression test cases.

#### 12. GitHub Copilot Agent HQ
* **Architecture:** Centralized enterprise control hub inside GitHub Enterprise managing multi-vendor agent deployments (Devin, Copilot, third-party).
* **Directives:** Centralized administration through GitHub Enterprise Cloud settings.

---

## 7. Agentic Proxies, Gateways, & Routers

### 7.1 Exhaustive Catalog with Setup, Routing, and Slash Directives

#### 1. Bifrost (Maxim AI)
* **Architecture:** High-throughput Rust/Go proxy connecting agents to models and MCP servers with automated rate-limit smoothing.
* **CLI Setup & Execution:**
  ```bash
  bifrost-server --port 8080 --redis redis://localhost:6379
  ```
* **Routing Directives:** Configured via HTTP headers:
  * `x-bifrost-fallback-providers: anthropic,openai,groq`
  * `x-bifrost-cache-ttl: 3600`

#### 2. OpenRouter
* **Architecture:** Multi-provider API routing layer translating requests across 100+ AI backends with auto-fallbacks and cost optimization.
* **HTTP Endpoint:** `https://openrouter.ai/api/v1/chat/completions`
* **Custom Headers & Parameters:**
  * `HTTP-Referer: https://your-site.com`
  * `X-Title: My Agent Application`
  * Model Routing Array: `"model": "anthropic/claude-3.7-sonnet,openai/o3-mini"` (automatically falls back if primary is throttled or down).
  * Auto-routing alias: `"model": "openrouter/auto"` (picks lowest cost/latency provider dynamically).

#### 3. Portkey
* **Architecture:** Enterprise AI gateway and control plane with load balancing, caching, and budget limits.
* **API Endpoint:** `https://api.portkey.ai/v1`
* **Configuration Headers:**
  * `x-portkey-api-key: $PORTKEY_API_KEY`
  * `x-portkey-virtual-key: $VIRTUAL_KEY`
  * `x-portkey-config: pc-retry-fallback-policy-123`
  * `x-portkey-cache: semantic`

#### 4. LiteLLM Proxy
* **Architecture:** Python/AsyncIO proxy translating 100+ LLM APIs to the OpenAI format with Redis-backed rate limiting.
* **CLI Commands:**
  ```bash
  # Start proxy with configuration file
  litellm --config config.yaml --port 4000 --num_workers 8

  # Quick test inference through running proxy
  litellm --test
  ```
* **In-Chat Directives (when using LiteLLM web UI):**
  * `/models`: Enumerate active backend routing keys.
  * `/spend`: Inspect current user/key token spend.

#### 5. Cloudflare AI Gateway
* **Architecture:** Edge AI proxy running on Cloudflare's network with edge caching and request logging.
* **Base URL:** `https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_slug}/{provider}`
* **Configuration Headers:**
  * `cf-aig-cache: true`
  * `cf-aig-skip-cache: false`
  * `cf-aig-cache-ttl: 86400`

#### 6. Helicone
* **Architecture:** Open-source proxy with observability, cost monitoring, and automated caching.
* **Base URL:** `https://oai.helicone.ai/v1`
* **Configuration Headers:**
  * `Helicone-Auth: Bearer $HELICONE_API_KEY`
  * `Helicone-Cache-Enabled: true`
  * `Helicone-RateLimit-Policy: 1000;w=60;u=user_123`

#### 7. Martian
* **Architecture:** Algorithmic prompt router sending simple queries to smaller models and complex tasks to frontier reasoning models.
* **API Base:** `https://api.withmartian.com/v1`
* **Model Parameter:** `"model": "martian/auto"`

#### 8. Vercel AI Gateway
* **Architecture:** Edge infrastructure optimized for streaming responses to the Vercel AI SDK.
* **Configuration:** Managed via `@ai-sdk/provider` wrappers with automated edge-region failover.

#### 9. BerriAI
* **Architecture:** Enterprise proxy deployment built on LiteLLM for isolated VPC tenant quota governance.

#### 10. APIClaw
* **Architecture:** High-speed gateway featuring anomaly detection, automated circuit breakers, and predictive rate-limit throttling.

#### 11. Langfuse
* **Architecture:** Open-source LLM observability, prompt management, and tracing gateway.
* **Integration:** SDK-level wrappers for Python and TypeScript capturing inputs, outputs, token counts, and execution latencies.

#### 12. Unify AI
* **Architecture:** Dynamic router querying real-time performance metrics to route prompts to the fastest or cheapest provider hosting that model.
* **Model ID Format:** `model_name@provider_name` (e.g., `llama-3.3-70b@together-ai` or `llama-3.3-70b@groq`).

---

## 8. Technical Quota Limits, Enforcement Types, & Headers

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
| **Google AI Studio (Free Tier)** | 15 RPM (Flash) / 2 RPM (Pro) | 1,000,000 TPM (Flash) / 32,000 TPM (Pro) | 1,500 RPD (Flash) / 50 RPD (Pro) | 1–2 in-flight |
| **Google AI Studio (Pay-Go Tier 1)** | 1,000 RPM (Flash) / 360 RPM (Pro) | 4,000,000 TPM (Flash) / 2,000,000 TPM (Pro) | Unlimited | Up to 100 in-flight |
| **Google AI Studio (Pay-Go Tier 2+)** | 4,000+ RPM | 10,000,000+ TPM | Unlimited | Custom |
| **OpenAI (Tier 1: $5 paid)** | 500 RPM | 30,000 TPM (`gpt-4o`) / 10,000 TPM (`o3-mini`) | Unlimited | 5 in-flight |
| **OpenAI (Tier 4: $250 paid)** | 10,000 RPM | 1,000,000 TPM (`gpt-4o`) / 400,000 TPM (`o3-mini`) | Unlimited | 100 in-flight |
| **OpenAI (Tier 5: $1,000+ paid)** | 15,000 RPM | 4,000,000 TPM (`gpt-4o`) / 1,000,000 TPM (`o3-mini`) | Unlimited | 500 in-flight |
| **Anthropic (Tier 1: $5 deposit)** | 50 RPM | 20,000 TPM (`claude-3-7-sonnet`) | Unlimited | 2 in-flight |
| **Anthropic (Tier 4: $400 deposit)** | 4,000 RPM | 400,000 TPM (`claude-3-7-sonnet`) | Unlimited | 80 in-flight |
| **DeepSeek API (Standard Pay-Go)** | 1,000 RPM | Dynamic cluster capacity throttling | Unlimited | Dynamic based on cluster load |
| **Groq Cloud (Developer Free)** | 30 RPM | 6,000 to 30,000 TPM | 14,400 RPD | 1–3 in-flight |

---

### 8.3 Rate-Limit Header Specifications Across Providers

#### 1. Standard OpenAI Rate-Limit Headers:
```http
x-ratelimit-limit-requests: 10000
x-ratelimit-limit-tokens: 2000000
x-ratelimit-remaining-requests: 9989
x-ratelimit-remaining-tokens: 1984210
x-ratelimit-reset-requests: 66ms
x-ratelimit-reset-tokens: 473ms
```

#### 2. Anthropic Rate-Limit Headers:
```http
anthropic-ratelimit-requests-limit: 4000
anthropic-ratelimit-requests-remaining: 3995
anthropic-ratelimit-requests-reset: 2026-03-31T20:00:00Z
anthropic-ratelimit-tokens-limit: 400000
anthropic-ratelimit-tokens-remaining: 385000
anthropic-ratelimit-tokens-reset: 2026-03-31T20:00:00Z
retry-after: 3
```

#### 3. Google Gemini / Vertex AI Error Payload (HTTP 429):
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
                        if retry_after_header:
                            sleep_duration = float(retry_after_header)
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