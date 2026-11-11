# Local Inference Engines: Comprehensive Socket & HTTP API Reference

This technical reference catalogs local LLM runtimes, engines, workstations, and frameworks that expose local HTTP sockets, OpenAI-compatible `/v1/` endpoints, native REST APIs, or gRPC interfaces.

---

## 1. Quick Socket & Port Reference Matrix

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

## 2. Core CLI Runtimes & Inference Daemons

### 2.1 Ollama
* **Nature**: Standalone daemon and CLI orchestrator wrapping `llama.cpp` for automated model pulling, quantization, and running.
* **Default Socket**: `http://localhost:11434` (configurable via `OLLAMA_HOST`)
* **Supported Endpoints**:
  * **OpenAI Compatible**:
    * `POST http://localhost:11434/v1/chat/completions`
    * `POST http://localhost:11434/v1/completions`
    * `POST http://localhost:11434/v1/embeddings`
    * `GET http://localhost:11434/v1/models`
  * **Native REST API**:
    * `POST /api/generate` (raw prompt completion with streaming)
    * `POST /api/chat` (structured multi-turn messages)
    * `POST /api/embed` & `POST /api/embeddings` (vector generation)
    * `GET /api/tags` & `POST /api/show` (list/inspect local weights)
    * `POST /api/pull`, `POST /api/push`, `DELETE /api/delete` (registry operations)
    * `POST /api/create` (custom Modelfile ingestion)
    * `GET /api/ps` (inspect running models loaded in VRAM)
* **Launch Command**:
  ```bash
  # Background daemon
  ollama serve

  # Run model directly (starts server if not running)
  ollama run llama3.3:latest
  ```
* **Key Features**: Instant single-command model acquisition via the Ollama Registry, cross-platform GPU auto-detection (CUDA, ROCm, Metal), unified Modelfile syntax, and simultaneous support for native and OpenAI API specifications.

---

### 2.2 llama.cpp (`llama-server`)
* **Nature**: Bare-metal C/C++ inference implementation; the foundational engine underneath Ollama, LM Studio, Jan, and others.
* **Default Socket**: `http://localhost:8080`
* **Supported Endpoints**:
  * `POST /v1/chat/completions` (OpenAI format)
  * `POST /v1/completions` (OpenAI format)
  * `POST /v1/embeddings`
  * `POST /completion` (native raw streaming endpoint)
  * `POST /tokenize`, `POST /detokenize`
  * `GET /health`, `GET /slots`, `GET /metrics`
* **Launch Command**:
  ```bash
  ./llama-server \
    -m ./models/qwen2.5-coder-7b-instruct.Q4_K_M.gguf \
    --host 127.0.0.1 \
    --port 8080 \
    -c 8192 \
    -ngl 99
  ```
* **Key Features**: Lowest resource overhead, zero Python dependencies, continuous slot batching, KV cache quantization (`q8_0`, `q4_0`), and support across CUDA, ROCm, Metal, Vulkan, and SYCL.

---

### 2.3 Kobold.cpp
* **Nature**: Specialized C/C++ runtime based on `llama.cpp` engineered for interactive text generation, story writing, and creative roleplay.
* **Default Socket**: `http://localhost:5001`
* **Supported Endpoints**:
  * `POST /v1/chat/completions` (OpenAI format)
  * `POST /api/v1/generate` (Native Kobold REST generation)
  * `POST /api/extra/abort`, `GET /api/extra/version`
* **Launch Command**:
  ```bash
  python koboldcpp.py --model model.gguf --port 5001 --usecublas
  ```
* **Key Features**: Advanced sampling parameters (Mirostat, Dry, Tail-Free Sampling, Grammar restrictions), bundled browser GUI, and low-latency interaction with local frontends like SillyTavern.

---

### 2.4 llamafile (Mozilla)
* **Nature**: Polyglot multi-platform single-file executable combining model weights, `llama.cpp`, and an HTTP web server.
* **Default Socket**: `http://localhost:8080`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`
  * `POST /v1/models`
  * Native `llama.cpp` endpoints (`/completion`)
* **Launch Command**:
  ```bash
  chmod +x model.llamafile
  ./model.llamafile --server --nobrowser --port 8080
  ```
* **Key Features**: Zero installation or runtime configuration; single binary runs natively across Linux, macOS, Windows, and FreeBSD.

---

### 2.5 LocalAI
* **Nature**: Container-based, self-hosted replacement for OpenAI's suite of APIs across text, audio, and vision.
* **Default Socket**: `http://localhost:8080`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`, `POST /v1/completions`
  * `POST /v1/embeddings`
  * `POST /v1/audio/transcriptions` (Whisper)
  * `POST /v1/audio/speech` (Piper / Bark / Coqui)
  * `POST /v1/images/generations` (Stable Diffusion)
* **Launch Command**:
  ```bash
  docker run -p 8080:8080 --name local-ai -ti localai/localai:latest-aio-cpu
  ```
* **Key Features**: Unifies multiple backend engines (`llama.cpp`, Diffusers, Transformers, Piper) under a single OpenAI-compatible HTTP interface.

---

### 2.6 TabbyAPI (ExLlamaV2)
* **Nature**: Lightweight, fast OpenAI-compatible REST server wrapped around the ExLlamaV2 engine.
* **Default Socket**: `http://localhost:5000`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`
  * `POST /v1/completions`
  * `GET /v1/models`
* **Launch Command**:
  ```bash
  python main.py
  ```
* **Key Features**: Extreme generation speed on NVIDIA GPUs, sub-4-bit mixed-precision quantization (EXL2), and low VRAM footprint.

---

### 2.7 MLC-LLM
* **Nature**: Compiler-accelerated LLM runtime engine based on Apache TVM, built for bare-metal execution on heterogeneous hardware.
* **Default Socket**: `http://localhost:8000`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`
  * `GET /v1/models`
* **Launch Command**:
  ```bash
  mlc_llm serve HF://mlc-ai/Llama-3-8B-Instruct-q4f16_1-MLC --port 8000
  ```
* **Key Features**: Cross-compilation for WebGPU, Vulkan, OpenCL, Metal, and CUDA; lightweight memory runtime for embedded and mobile devices.

---

### 2.8 Exo
* **Nature**: Decentralized peer-to-peer compute orchestrator that links multiple consumer devices (Macs, PCs, Linux boxes) into a single unified GPU inference cluster.
* **Default Socket**: `http://localhost:52415`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`
  * `GET /v1/models`
* **Launch Command**:
  ```bash
  exo
  ```
* **Key Features**: Automatic peer node discovery via local network (Wi-Fi/Ethernet), ring and tensor splitting of models too large for a single machine, presenting a single OpenAI endpoint.

---

## 3. Desktop Workstations with Embedded API Servers

### 3.1 LM Studio
* **Default Socket**: `http://localhost:1234`
* **Endpoints**: `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models`
* **Activation**: Navigate to the **Developer / Local Server** tab, select/load a model, configure context size, and toggle **Start Server**. Headless control is also available via the `lms` CLI (`lms server start`).
* **Highlights**: Direct Hugging Face model browser, GPU offload sliders, Apple MLX hardware integration, and JIT dynamic model loading.

### 3.2 Jan AI
* **Default Socket**: `http://localhost:1337`
* **Endpoints**: `/v1/chat/completions`, `/v1/models`
* **Activation**: `Settings` $\rightarrow$ `Advanced Settings` $\rightarrow$ enable **Local API Server**.
* **Highlights**: 100% open-source, local-first data storage, zero telemetry, and built-in Model Context Protocol (MCP) support.

### 3.3 GPT4All (Nomic AI)
* **Default Socket**: `http://localhost:4891`
* **Endpoints**: `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models`
* **Activation**: `Settings` $\rightarrow$ `Application` $\rightarrow$ enable **Local API Server**.
* **Highlights**: CPU-friendly inference, local document vector indexing (`LocalDocs`), and privacy guarantees.

### 3.4 Msty AI
* **Default Socket**: `http://localhost:10240`
* **Endpoints**: `/v1/chat/completions`, `/v1/models`
* **Activation**: `Settings` $\rightarrow$ `Local Server` $\rightarrow$ toggle **Enable Server**.
* **Highlights**: Multi-model side-by-side prompt benchmarking, local knowledge stacks, and clean UI workflow.

---

## 4. High-Concurrency Production & Enterprise Serving Engines

### 4.1 vLLM
* **Architecture**: Production-grade serving engine built around the PagedAttention memory algorithm.
* **Default Socket**: `http://localhost:8000`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`, `POST /v1/completions`, `POST /v1/embeddings`
  * `GET /v1/models`, `GET /metrics` (Prometheus)
* **Launch Command**:
  ```bash
  vllm serve meta-llama/Llama-3.1-8B-Instruct \
    --host 0.0.0.0 \
    --port 8000 \
    --gpu-memory-utilization 0.95 \
    --max-model-len 8192
  ```
* **Key Features**: Continuous request batching, multi-GPU tensor/pipeline parallelism, speculative decoding, and native support for AWQ, GPTQ, FP8, and GGUF.

---

### 4.2 SGLang
* **Architecture**: High-performance execution framework featuring RadixAttention for automatic KV cache reuse.
* **Default Socket**: `http://localhost:30000`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`
  * `POST /generate` (Native high-throughput endpoint)
  * `GET /v1/models`
* **Launch Command**:
  ```bash
  python -m sglang.launch_server \
    --model-path mistralai/Mistral-7B-Instruct-v0.3 \
    --port 30000 \
    --host 0.0.0.0
  ```
* **Key Features**: Drastic reduction in first-token latency on multi-turn conversations and multi-agent loops; optimized for constrained JSON decoding.

---

### 4.3 Hugging Face Text Generation Inference (TGI)
* **Architecture**: Rust, Python, and gRPC production container built for Hugging Face Hub deployments.
* **Default Socket**: `http://localhost:8080`
* **Supported Endpoints**:
  * `POST /v1/chat/completions` (OpenAI format)
  * `POST /generate` and `POST /generate_stream` (Native REST/SSE)
  * `GET /info`, `GET /health`, `GET /metrics`
* **Launch Command**:
  ```bash
  docker run --gpus all --shm-size 1g -p 8080:80 \
    ghcr.io/huggingface/text-generation-inference:latest \
    --model-id meta-llama/Llama-3.1-8B-Instruct
  ```
* **Key Features**: FlashAttention-2, multi-GPU tensor parallelism, token watermarking, and production Prometheus telemetry.

---

### 4.4 LMDeploy
* **Architecture**: Full-stack model serving framework driven by the TurboMind C++ backend.
* **Default Socket**: `http://localhost:23333`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`, `POST /v1/models`
* **Launch Command**:
  ```bash
  lmdeploy serve api_server internlm/internlm2_5-7b-chat --server-port 23333
  ```
* **Key Features**: 4-bit KV cache quantization (doubling maximum context capacity), high throughput under heavy batch loads.

---

### 4.5 FastChat (LMSYS)
* **Architecture**: Distributed multi-node model serving and evaluation platform powering the LMSYS Chatbot Arena.
* **Default Socket**: Controller on `http://localhost:21001`, OpenAI API Server on `http://localhost:8000`
* **Supported Endpoints**:
  * `POST /v1/chat/completions`, `POST /v1/completions`, `POST /v1/embeddings`, `GET /v1/models`
* **Launch Command**:
  ```bash
  # Step 1: Controller
  python3 -m fastchat.serve.controller --host 0.0.0.0 --port 21001

  # Step 2: Model Worker
  python3 -m fastchat.serve.model_worker --model-path lmsys/vicuna-7b-v1.5 --controller http://localhost:21001 --port 31000 --worker http://localhost:31000

  # Step 3: OpenAI REST Frontend
  python3 -m fastchat.serve.openai_api_server --controller-address http://localhost:21001 --host 0.0.0.0 --port 8000
  ```
* **Key Features**: Clean separation of API gateway, controller, and GPU workers; scalable across distributed nodes.

---

### 4.6 NVIDIA Triton Inference Server + TensorRT-LLM
* **Architecture**: Enterprise model serving infrastructure paired with NVIDIA TensorRT-LLM optimization kernels.
* **Default Socket**: `http://localhost:8000` (HTTP), `8001` (gRPC), `8002` (Metrics)
* **Supported Endpoints**:
  * `POST /v1/chat/completions` (OpenAI translation gateway)
  * `POST /v2/models/{model_name}/infer` (KServe v2 protocol)
* **Key Features**: Peak inference performance on NVIDIA hardware, dynamic batching, in-flight batching, and multi-model hardware co-location.

---

## 5. Web Interfaces, Knowledge Workspaces & API Proxies

### 5.1 Text-Generation-WebUI (Oobabooga)
* **Default Socket**: Web UI on `http://localhost:7860`, API on `http://localhost:5000`
* **Endpoints**: `/v1/chat/completions`, `/v1/completions`, `/v1/embeddings`, `/v1/models`
* **Launch Command**:
  ```bash
  python server.py --api --api-port 5000 --listen
  ```
* **Highlights**: Flexible model testing across Transformers, llama.cpp, ExLlamaV2, and HQQ with on-the-fly backend hot-swapping.

### 5.2 AnythingLLM
* **Default Socket**: `http://localhost:3001`
* **Endpoints**:
  * `POST /api/v1/workspace/{slug}/chat` (Native RAG query endpoint)
  * `POST /api/v1/openai/chat/completions` (OpenAI proxy endpoint)
* **Highlights**: Integrated vector storage, document parsing pipeline, multi-user access control, and complete RAG automation.

### 5.3 LiteLLM Proxy
* **Default Socket**: `http://localhost:4000`
* **Endpoints**: `/v1/chat/completions`, `/v1/embeddings`, `/v1/models`
* **Launch Command**:
  ```bash
  litellm --config config.yaml --port 4000
  ```
* **Highlights**: Acts as an intermediary proxy layer in front of local servers (Ollama, vLLM, llama-server) to handle load balancing, user quotas, failover routing, and key management.

---

## 6. Selection Recommendation

```
                                  Your Primary Goal
                                          │
         ┌────────────────────────────────┼────────────────────────────────┐
         ▼                                ▼                                ▼
[Desktop Workspace]             [Developer Daemon]              [Production Concurrency]
  • LM Studio (GUI)               • Ollama (CLI / Modelfile)      • vLLM (General throughput)
  • Jan AI (Open-Source / MCP)    • llama-server (Bare-metal)     • SGLang (Agents & JSON)
  • GPT4All (CPU / LocalDocs)     • TabbyAPI (ExLlamaV2 / EXL2)   • TGI (Hugging Face ecosystem)
                                  • Exo (Distributed cluster)     • Triton + TRT-LLM (Enterprise)
```