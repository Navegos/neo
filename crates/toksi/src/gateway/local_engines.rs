// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/local_engines.rs
// created: 2026-11-21

//! Registry of all 26+ Section 2.7 local/pure-Rust inference engines (`ollama`, `llama.cpp`,
//! `candle`, `candle-vllm`, `mistral.rs`, `dynamo`, `velo`, `exo`, `mlx`, `vllm`, `sglang`,
//! `TensorRT-LLM`, `exllamav2`, `lmdeploy`, `tabbyAPI`, `mlc-llm`, `jan`, `Mysti`, `koboldcpp`,
//! `llamafile`, `LocalAI`, `FastChat`, `LitServe`, `aitune`, `radixshmem`, `modelexpress`, `nixl`)
//! plus Section 2.8–2.10 vLLM/DeepSeek/SGLang/HF/NeMo/PyTorch/TensorRT/Triton/GPUStack/CUDA/Slurm
//! telemetry & recipe adapters (`vmon`, `cuda-rust`, `srt-slurm`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalEngineDescriptor {
    pub id: &'static str,
    pub default_port: u16,
    pub pure_rust: bool,
    pub openai_compat: bool,
}

/// All Section 2.7 local & pure-Rust inference engines.
pub const SECTION_2_7_LOCAL_ENGINES: &[LocalEngineDescriptor] = &[
    LocalEngineDescriptor { id: "ollama", default_port: 11434, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "llama.cpp", default_port: 8080, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "candle", default_port: 8080, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "candle-vllm", default_port: 2000, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "mistral.rs", default_port: 1234, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "dynamo", default_port: 8000, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "velo", default_port: 8000, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "exo", default_port: 52415, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "mlx", default_port: 8080, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "vllm", default_port: 8000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "sglang", default_port: 30000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "TensorRT-LLM", default_port: 8000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "exllamav2", default_port: 5000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "lmdeploy", default_port: 23333, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "tabbyAPI", default_port: 5000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "mlc-llm", default_port: 8000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "jan", default_port: 1337, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "Mysti", default_port: 10240, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "koboldcpp", default_port: 5001, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "llamafile", default_port: 8080, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "LocalAI", default_port: 8080, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "FastChat", default_port: 8000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "LitServe", default_port: 8000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "aitune", default_port: 8000, pure_rust: false, openai_compat: false },
    LocalEngineDescriptor { id: "radixshmem", default_port: 0, pure_rust: true, openai_compat: false },
    LocalEngineDescriptor { id: "modelexpress", default_port: 8001, pure_rust: true, openai_compat: false },
    LocalEngineDescriptor { id: "nixl", default_port: 0, pure_rust: false, openai_compat: false },
    LocalEngineDescriptor { id: "oxillama", default_port: 11435, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "xinfer", default_port: 8082, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "crane", default_port: 8083, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "paddler", default_port: 8084, pure_rust: true, openai_compat: true },
    LocalEngineDescriptor { id: "llama-swap", default_port: 8085, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "whisper.cpp", default_port: 8086, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "stable-diffusion.cpp", default_port: 8087, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "triton-server", default_port: 8000, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "deepspeed-mii", default_port: 8088, pure_rust: false, openai_compat: true },
    LocalEngineDescriptor { id: "hyperflare-inference", default_port: 8090, pure_rust: true, openai_compat: true },
];

/// Section 2.8 (48), Section 2.9 (41), and Section 2.10 (51) compute & serving vaults (140 total).
pub const SECTION_2_8_TO_2_10_VAULTS: &[&str] = &[
    // 2.8 vLLM (22)
    "vllm", "recipes", "llm-compressor", "tpu-inference", "vllm-omni", "guidellm", "vllm-torchtpu",
    "vllm-dashboard", "speculators", "vllm-gaudi", "perf-eval", "vllm-ascend", "router", "agentic-api",
    "compressed-tensors", "humming", "vllm-neuron", "vllm-gguf-plugin", "vime", "FlashKDA", "vmon",
    "lm-evaluation-harness",
    // 2.8 DeepSeek (14)
    "MSA", "flash-attention", "DeepSelect", "FlashMLA", "DeepGEMM", "deepseek-harness", "3FS",
    "DeepJIT", "DeepInfer", "DeepEP-Ascend", "DeepEP", "deepseek-recipe", "DeepGEMM-Ascend", "TileKernels",
    // 2.8 SGLang (12)
    "sglang", "sglang-omni", "sgl-nvidia-mega", "sgl-kernel-xpu", "sgl-kernel-npu", "SpecForge",
    "sglang-torchtpu", "sgl-eval", "genai-bench", "sglang-jax", "sglang-neuron", "sgl-flash-attn",
    // 2.9 HuggingFace & Optimum (21)
    "transformers", "trl", "peft", "accelerate", "kernels", "datasets", "diffusers",
    "huggingface_hub", "hf-serve", "optimum", "optimum-intel", "optimum-quanto", "optimum-amd",
    "optimum-habana", "optimum-executorch", "optimum-neuron", "optimum-graphcore", "optimum-onnx",
    "optimum-furiosa", "optimum-nvidia", "optimum-benchmark",
    // 2.9 NVIDIA NeMo & Eval (20)
    "Gym", "nemo-helix", "RL", "Megatron-Bridge", "Curator", "Switchyard", "Speech", "UserSim",
    "Nemotron", "Evaluator", "nemo-compass", "eval-author", "DataDesigner", "Export-Deploy",
    "DataDesignerPlugins", "neural-compressor", "physicsnemo", "physicsnemo-serve", "NeMo-Fabric", "NeMo-Relay",
    // 2.10 PyTorch & Lightning (14)
    "pytorch", "helion", "torchtitan", "executorch", "FBGEMM", "torch-tensorrt", "ao", "torch-rl",
    "tensordict", "pytorch-lightning", "lightning-thunder", "litgpt", "torchmetrics", "LitModels",
    // 2.10 TensorRT, Triton & Compilers (12)
    "TensorRT", "triton", "triton-ascend", "triton-windows", "triton-ext", "DeepSpeed", "cutlass",
    "Model-Optimizer", "Megatron-LM", "TransformerEngine", "TensorRT-Model-Connect", "xla_staging",
    // 2.10 GPUStack, CUDA (`cuda-rust`), RAPIDS & Cluster (25)
    "gpustack", "gpustack-operator", "gpustack-runtime", "cuml", "cudf", "cuda-python", "kumo-engine",
    "cudnn-frontend", "aistore", "Nomos", "slurm", "srt-slurm", "numba-cuda-mlir", "cudaq-pulse",
    "cuopt", "cuda-rust", "gpu-operator", "cuCollections", "DeepStream", "DALI", "cuCascade",
    "NVFlare", "OSMO", "nccl", "cuEquivariance",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeTelemetryAdapter {
    pub adapter_id: String,
    pub kind: String,
    pub available_in_path: bool,
}

/// Probes availability of `vmon`, `cuda-rust` (`nvidia-smi`), and `srt-slurm` (`squeue`) adapters.
pub fn probe_compute_adapters() -> Vec<ComputeTelemetryAdapter> {
    vec![
        ComputeTelemetryAdapter {
            adapter_id: "vmon".to_string(),
            kind: "vllm-metrics".to_string(),
            available_in_path: which::which("vmon").is_ok(),
        },
        ComputeTelemetryAdapter {
            adapter_id: "cuda-rust".to_string(),
            kind: "gpu-vram-telemetry".to_string(),
            available_in_path: which::which("nvidia-smi").is_ok(),
        },
        ComputeTelemetryAdapter {
            adapter_id: "srt-slurm".to_string(),
            kind: "slurm-cluster".to_string(),
            available_in_path: which::which("squeue").is_ok(),
        },
    ]
}

pub fn lookup_local_engine(id: &str) -> Option<&'static LocalEngineDescriptor> {
    let low = id.trim().to_ascii_lowercase();
    SECTION_2_7_LOCAL_ENGINES
        .iter()
        .find(|e| e.id.to_ascii_lowercase() == low)
}

/// Wave 20 Section 2.15 (26), Section 2.16 (26), and Section 2.17 (9) vaults (61 total).
pub const SECTION_2_15_TO_2_17_VAULTS: &[&str] = &[
    // 2.15 Distributed Cluster, Kubernetes, Kubeflow & Spark (26)
    "kubernetes", "minikube", "kompose", "kops", "helm", "autoscaler", "node-problem-detector",
    "cluster-readiness-engine", "NVSentinel", "aicr", "knavigator", "AI-Factory-Operations-Agent",
    "deepops", "kubeflow-hub", "kubeflow-pipelines", "kubeflow-trainer", "kubeflow-sdk",
    "kubeflow-kale", "kubeflow-spark-operator", "kubeflow-dashboard", "kubeflow-notebooks",
    "kubeflow-semantic-operator", "kubeflow-katib", "Run", "spark", "cudf-spark",
    // 2.16 11-Pillar Datasets, Serving, Compression, RAG, Semantic Routing & IDEs (26)
    "ossie", "ossie-converters", "auto-ontology", "structured-data-models", "Megatron-Energon",
    "cuPhoton", "hyperflare-inference", "kvpress", "bitsandbytes", "Personal-AI-Router",
    "semantic-router", "NeMo-Agent-Toolkit", "NeMo-Agent-Toolkit-UI", "NeMo-Relay-Plugins",
    "elements", "llama_index", "llama-agents", "liteparse", "haystack", "prompt-ops", "llama",
    "llama-models", "ogx", "ogx-client-go", "vscode", "aider",
    // 2.17 Evals, PurpleLlama & OpenTelemetry (9)
    "AutoFA", "deepeval", "ragas", "PurpleLlama", "opentelemetry-rust", "opentelemetry-collector",
    "opentelemetry-operator", "opentelemetry-python", "opentelemetry-python-genai",
];

pub fn total_mandated_vault_count() -> usize {
    crate::tools::multi_protocol::CONSOLIDATED_VAULT_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave20_local_engines_and_493_vaults() {
        let hf = lookup_local_engine("hyperflare-inference").expect("hyperflare-inference registered");
        assert!(hf.pure_rust);
        assert_eq!(hf.default_port, 8090);
        assert_eq!(SECTION_2_15_TO_2_17_VAULTS.len(), 61);
        assert_eq!(total_mandated_vault_count(), 493);
    }
}
