# Phase 19.6 — Model Training, Quantization, Pure-Rust/Multimodal Serving Plane & 43-File 8-State Sync (47 Vaults)

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read ALL 10 canonical workspace files FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. .toksi/projectmem/plan.md
5. .toksi/projectmem/PROJECT_MAP.md
6. .toksi/projectmem/summary.md
7. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
8. crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
9. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
10. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
Plus .agents/brain/PHASE_19_6_TRAINING_SERVING_MULTIMODAL_PLANE_AND_8STATE_SYNC.md.

Execute Phase 19.6: Model Training, Fine-Tuning, Quantization, ONNX/TensorRT Optimization, Pure-Rust/Multimodal Serving Plane & 43-File 8-State Master Sync across all 47 Deep-Researched Group C + `xla` Vaults.
Sources (all verified `[DEEP-RESEARCH-DONE]` in RESEARCHPLAN.md Sections 2.10 & 2.13):
  - Pure-Rust & Local Serving (6): `oxillama` (`0.1.4`, 11 crates), `xinfer` (`0.14.7`), `Crane` (`26.9.0`, `crane-core`/`crane-serve`), `paddler` (`5.0.0`, 35 crates), `llama-swap` (Go 1.27.3), `llguidance` (`10` crates `parser::earley`/`toktrie`)
  - Weights, Compute & Multimodal (6 + `livekit*`): `safetensors` (`0.9.0-dev.0`), `ggml`, `whisper.cpp`, `whisper`, `stable-diffusion.cpp`, `cudarc` (`0.19.10`), `livekit`, `livekit-agents`, `livekit-cli`, `moly-ai`
  - Triton & DeepSpeed (4): `server`, `core` (`tritonserver` KServe v2), `DeepSpeed-MII`, `DeepSpeed-Kernels`
  - Training & Fine-Tuning (13): `unsloth`, `unsloth-studio`, `Automodel` (`nemo-automodel` DTensor SPMD), `nanotron`, `Emerging-Optimizers` (`Shampoo`/`SOAP`/`Muon`), `torchtune` (archived 2025 — recipe ref ONLY), `torchdata`, `torchvision`, `torchcodec`, `torchinfer` (`aot_tensor`), `torchrec`, `torchaudio`, `torchignite`
  - ONNX, Compression, GPU Vector/Vision & Sim (12 + `xla`): `onnx`, `optimizer` (`onnxoptimizer`), `ir-py` (`onnx-ir`), `onnxmltools`, `onnx-tensorrt`, `Olive`, `intel-neural-compressor`, `digestai`, `sentence-transformers`, `cuvs`, `CV-CUDA`, `warp` (`warp-lang`), `xla` (OpenXLA)
  - Evals & MLOps (5): `lighteval`, `langwatch`, `wandb` (`parquet-rust-wrapper`), `clearml`, `clearml-agent`
Targets:
  - crates/toksi/src/training/{mod.rs,recipes.rs,runner.rs,quant.rs} (NEW — zero binary bloat, `PATH`-verified subprocess orchestration)
  - crates/toksi/src/gateway/providers/local/{oxillama.rs,xinfer.rs,crane.rs,paddler.rs,llama_swap.rs,whisper_cpp.rs,sd_cpp.rs,triton_server.rs,deepspeed_mii.rs}
  - crates/toksi/src/agent_runtime/{tools_schema.rs,slash.rs} (wire `/train`, `/serve`, `/voice` completing all **21** slash commands!)
  - crates/toksi/src/security/binary_audit.rs (`safetensors` pickle-free header verification)
  - crates/toksi/src/telemetry/{gpu_monitor.rs,evals.rs,traces.rs} (`cudarc`/`lighteval`/`langwatch`/`wandb`/`clearml`)
  - crates/toksi/src/commands/{train.rs,serve.rs,eval.rs,mod.rs}, crates/toksi/src/main.rs
  - crates/toksi/src/tui/* & crates/toksi/src/dashboard/* (Tab 7/8 `train`, `serve`, `_worker`, & multimodal panels)
  - All 6 canonical plan destinations + all 43 `.agents/brain/*.md` guides (8-State Progress Taxonomy sync)
  - crates/toksi/tests/phase19_6_training_serving_multimodal_verification.rs (NEW)

Requirements:
1. Pure-Rust & Multimodal Serving Plane (`toksi serve` & `/serve` & `/voice`):
   - Register and support `oxillama` (pure-Rust GGUF), `xinfer` (paged attention), `Crane` (`crane-serve`), `paddler` (cluster slot balancer), `llama-swap` (VRAM-safe hot-swap), `whisper.cpp` (offline STT for `/voice`), `stable-diffusion.cpp` (`sd-server`), `server`/`core` (Triton KServe v2 `/v2/models/*`), and `DeepSpeed-MII` alongside `mistral.rs`/`candle`/`ollama`/`llama.cpp`/`vllm`/`sglang` (feature-gated/zero binary bloat).
   - Port `llguidance` Earley/Trie CFG + JSON-Schema token masking into `tools_schema.rs` for guaranteed valid tool-call JSON on local engines.
   - Integrate `livekit*` WebRTC/VAD voice signaling contracts for `/voice` and `safetensors` header verification in `binary_audit.rs`.
2. Unified Training, Quantization & Eval Plane (`toksi train`, `toksi eval`, `/train`):
   - Generate reproducible recipes & launch isolated `PATH`-checked external jobs (never bundling Python/CUDA inside `toksi`) across `unsloth`, `Automodel`, `nanotron`, `Emerging-Optimizers`, `Olive`, `intel-neural-compressor`, `onnx`/`onnx-tensorrt`, `clearml-agent`, and historical `torchtune` recipe templates (marked archived).
   - Integrate `lighteval`, `langwatch`, `wandb` (`parquet-rust-wrapper`), and `digestai` footprint/quant-sensitivity reports into `telemetry/evals.rs`.
3. Complete **21** Unified Slash Commands (`CORE_SLASH_COMMANDS` updated to 21 in `src/agent_runtime/slash.rs` — migrating `/context` -> `/ctxmode` and adding `/workforce`, `/spec`, `/train`, `/serve`, `/voice` with `< 5ms` dispatch).
4. Synchronize ALL 6 canonical plan destinations (`RESEARCHPLAN.md`, `CONSOLIDATION_PLAN.md`, `PLAN.md`, `README.md`, `.toksi/projectmem/{plan.md,PROJECT_MAP.md,summary.md}`, plus all 43 `.agents/brain/*.md` guides) with truthful **8-State Progress Taxonomy** tags (`[PLAN-IMPLEMENTED]`/`[PLAN-FIXED]`).
5. Update `.toksi/test-baseline.json`; verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p toksi --test phase19_6_training_serving_multimodal_verification`, and full `.agents/brain/` 43-file hygiene checks.
```

---

## Technical Specifications & Details

### 1. Verified Crate & Upstream Constraints
- **`oxillama@0.1.4`:** 11 workspace crates (`oxillama-gguf`, `oxillama-quant`, `oxillama-arch`, `oxillama-runtime`, `oxillama-server`, `oxillama-bench`, `oxillama-py`, `oxillama-wasm`) — zero C/C++/Fortran dependencies.
- **`Crane@26.9.0`:** `crane-core` (`generation`, `models`, `quantized`, `onnx`, `autotokenizer`) + `crane-serve` (`openai_api`, `sglang_api`, `reasoning`, `tools`).
- **`paddler@5.0.0`:** 35 crates including `paddler_balancer` (`agent_controller_pool`, `agent_controller_slot_guard`).
- **`torchtune`:** Archived upstream in 2025 — treat strictly as historical YAML recipe reference, never as a live runtime dependency.

## Verification Criteria
- `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `phase19_6_training_serving_multimodal_verification` pass 100%.
- All 21 slash commands execute in `< 5ms` non-blocking; all 43 `.agents/brain/*.md` guides + 6 canonical plans accurately reflect `[PLAN-IMPLEMENTED]` / `[PLAN-FIXED]`.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
