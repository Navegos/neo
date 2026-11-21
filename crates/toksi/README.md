# Toksi: Native Consolidation Master Engine (Wave 19 / v6.0)

[![Crates.io](https://img.shields.io/crates/v/toksi.svg)](https://crates.io/crates/toksi)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**Toksi** is an enterprise-grade, **100% Native Safe Rust (`#![deny(unsafe_code)]`)** consolidation across the **3-repository ecosystem** (`<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, `<skills>` `Navegos/skills`) unifying **432 unique mandated upstream research vaults (`[DEEP-RESEARCH-DONE]`)** (`294` Waves 1–18 vaults across Sections `2.1`–`2.10` + companion repos `<advisory>` & `<skills>` + `agents-structures` + `138` Wave 19 additions/reconciliations across Sections `2.10`–`2.14`) and the **35-component `<toksi-siagent>` Super Intelligence Architecture (`RESEARCHPLAN.md` Section 0.2: 26 base `<agent>` + 9 `<siagent>` extensions)** — spanning token optimization (`HeadroomEffortCompressor` + `tiktoken` + `RTK` + `ctxmode`), intelligent proxying, **Dedicated Multi-Process Supervisor (`toksi _worker <role>`) + Named Pipe/UDS `DirectWireBus` (`src/ipc/`)**, **Toksi Unified Plugin System (`src/plugins/`)**, multi-protocol agent wire integration (`MCP` / `A2A` / `ACP` / **`ANP` (`did_wba`)** / `stdio` / `SSE` / `HTTP` / **`QUIC`**), Policy-as-Code (`opa`/`cedar`) & SPIFFE IAM (`spire`), `Guardrails`/`llm-guard`/`openai-guardrails-python`/`safeagents`/`garak`/Anonymizer + **ATR (`agent-threat-rules`)** + **`model_spec`**, **Zero-Plaintext Blind Signature Engine (`BLAKE3` + `sparse::DFA`, `src/security/`)**, **Native Git/PR Validator (`src/validator/` + 12-DB `UpstreamDeduplicator`)**, **ADR Layer (`src/security/adr.rs`)**, **5-Backend Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`: 3-OS native + `sandbox-runtime` + `OpenShell` + `E2B` + `sbx` CLI)**, **`CommandGuard` Bypass Kill-Switch (`src/commands/guard.rs`)**, live-safe `docker/Dockerfile.test` + `toksi test --docker|--sandbox` isolation, externalized Zero-Pollution `<skills>` (`skilldoctor` + `agentskills` + `upskill` + `skill-scanner` + `defenseclaw`, zero binary bloat + `<neo>/.agents/skills/` 922-dir hygiene), Google Cloud ADC (`AGY_ADC_AUTH=true` + `vertex_rust_sdk`), thinking-effort-aware (`Low`/`Medium`/`High`) compression, durable Temporal/Restate/LangGraph/`chevalier` workflow state & atomic plan checkpoints (`/resume`), autonomous coding/workforce/spec runtime (`toksi chat|run|exec|swarm` + `OMA`/`swarms-rs`/`ChatDev`/`OpenSpec`), model training/quantization/serving/eval/voice plane (`src/training/` `toksi train|serve|eval` + `livekit*`), and 9-tab telemetry cockpits for AI coding assistants.

Toksi sits transparently between **18+ registered AI coding assistants** (expanded across **65/90+ Categories 1–5 ecosystem providers** and **Open Agents `.agents/`**) and **30+ local & Rust-native inference/multimodal engines** (`ollama`, `llama.cpp`, `candle`, `candle-vllm`, `mistral.rs`, `dynamo`, `oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `whisper.cpp`, `stable-diffusion.cpp`, `vllm`, `sglang`, `triton`, `DeepSpeed-MII`, `exo`, `mlx`, `lmdeploy`, `TensorRT-LLM`, `frontend-crates`, `nixl`, plus `RouteLLM`/`olla`/`Switchyard` & 180+ vLLM/DeepSeek/SGLang/HF/NeMo/PyTorch/ONNX/TRT/Triton/GPUStack/CUDA/Slurm compute/training/eval vaults) plus multi-instance `<byok, model, effort>` & Google Cloud ADC providers — cutting token spend by **30% to 70%** via pure-Rust `headroom-core` `live_zone` cache-preserving compression, `CcrStore`, `TokenEconomizer`, `RTK` 100+ command filters, **`ctxmode`** (`Context-Mode`) sandboxes, **45 Tree-sitter language IDs** (with specialized `language_dockerfile()`, `language_vue()`, `language_scss()` grammars in `crates/toksi-grammars`), SCIP/AST-grep/Repomix/jcodemunch/OpenWiki/`pmat` code intelligence, hybrid graph-vector memory (`Qdrant`, `Lance` + `arrow-rs`, `Chroma`, `Cognee`, `Graphiti`, `Milvus`, `Letta`, `funes`), and repo-scoped zero-LLM memory — with zero Node.js, Python, npm, or pip build requirements and **zero paid-tier restrictions**.

---

## 1. Master Architecture & Unified 8-Port Multi-Process Supervisor Topology

All listening ports are overridable via `~/.toksi/config.toml` (`[ports]` & `[supervisor]`) and isolated into dedicated OS worker subprocesses (`toksi _worker <role>`) bridged via Windows Named Pipes / Unix Domain Sockets `DirectWireBus` (`src/ipc/`) so browser dashboards, tool indexing, and agent loops never induce TCP backpressure or cascading crashes on inference streams:

```text
[ AI Coding Assistants / IDEs / Local CLIs / Browsers / ACP & ANP Clients ]
  │        │        │        │        │         │         │         │
  │ :8787  │ :8788  │ :8789  │ :8790  │ :9098   │ :9099   │ :9100   │ :18787
  ▼        ▼        ▼        ▼        ▼         ▼         ▼         ▼
┌──────────────────────────────────────────────────────────────────────────────────────┐
│      toksi daemon start|stop|restart [--agents ...]  (Supervisor + supervisor_workers) │
│                                                                                      │
│ ┌─ _worker headroom ───────────┐   ┌─ _worker gateway ─────────────────────────────┐ │
│ │ :8787 Headroom Proxy         │   │ :9099 Enterprise AI Gateway & SDKs            │ │
│ │ - HTTP/1.1 & HTTP/2 SSE      │   │ - /v1/{chat/completions,messages,             │ │
│ │ - ConnectRPC Passthrough     │   │   responses,embeddings}, /v1beta/*            │ │
│ │ - live_zone Cache Surgery    │   │ - Shared Headroom Effort Compressor + Pingora │ │
│ │ - Low/Med/High Effort Scale  │   │ - /v1/security/scan & Prometheus /metrics     │ │
│ │ - 21 Target /... Commands    │   │ - AGY_ADC_AUTH=true Vertex (vertex_rust_sdk)  │ │
│ └──────────────────────────────┘   │ - 65/90+ Ecosystem & 30+ Local/Rust Engines   │ │
│ ┌─ _worker grok / copilot-* ───┐   └───────────────────────────────────────────────┘ │
│ │ :8788 Grok OAuth Daemon      │   ┌─ _worker protocol-bus ────────────────────────┐ │
│ │ :8789 Copilot VS Code        │   │ :9098 Unified Protocol Bus                    │ │
│ │ :8790 Copilot CLI            │   │ - MCP/A2A/ACP/ANP (did_wba)/SSE/HTTP/QUIC     │ │
│ └──────────────────────────────┘   │ - All 432 Vaults, <toksi-siagent> & State     │ │
│ ┌─ _worker byok-gateway ───────┐   └───────────────────────────────────────────────┘ │
│ │ :18787 BYOK & Vertex ADC     │   ┌─ _worker dashboard ───────────────────────────┐ │
│ │ - Per-profile base_url/ADC   │   │ :9100 Detached Web Telemetry Cockpit          │ │
│ │ - Headroom Effort Compress   │   │ - 9-Tab UI, tokview, Workers, Train & Security│ │
│ └──────────────────────────────┘   └───────────────────────────────────────────────┘ │
│                                                                                      │
│ [ Named Pipe / UDS DirectWireBus (`src/ipc/`) -> Dedicated Subsystem Workers ]       │
│  _worker siagent     • 35-Component <toksi-siagent>, Workforce (OMA/Swarms), /spec   │
│  _worker rtk         • RTK (100+ Filters + Tracker)                                  │
│  _worker projectmem  • ProjectMem (JSONL + plan_checkpoints/)                        │
│  _worker ctxmode     • CtxMode (`ctxmode`, Sandbox + FTS5 + WFST `<repo>/.toksi/ctxmode/`)│
│  _worker icm         • ICM (BM25 + Memoir/Feedback/Temporal + funes recall)          │
│  _worker aimemory    • ai-memory (OKF v0.2 Wiki + Lance/arrow-rs + Qdrant Vector)    │
│  _worker codegraph   • CodeGraph (45 TS/SCIP/OpenWiki/pmat TDG)                      │
│  _worker security    • Zero-Plaintext BLAKE3+DFA, ATR, Guardrails, ADR, CommandGuard,│
│                        defenseclaw, skill-scanner CEL, 45-Lang SAST, Validator PRs   │
│  + Plugins (`src/plugins/`) • Sandbox (`native`/`sandbox-runtime`/`sbx`)             │
│  + Training (`src/training/`) & Externalized <skills> (`Navegos/skills`)             │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### Dedicated Ports:
* **Port `8787`** (`_worker headroom`): Transparent Streaming Headroom Proxy (Claude, Codex, Antigravity, Cursor, mandatory `/byok/` compression, & `Low`/`Medium`/`High` thinking-effort scaling).
* **Port `8788`** (`_worker grok`): GitHub/xAI Grok OAuth Proxy Daemon (`grok_daemon.rs` — strictly separate from BYOK).
* **Port `8789`** (`_worker copilot-vscode`): GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`).
* **Port `8790`** (`_worker copilot-cli`): GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`).
* **Port `9098`** (`_worker protocol-bus`): Unified Protocol Bus (`MCP` / `A2A` v1.0 / `ACP` `agent-client-protocol` / **`ANP` (`did_wba`)** / `stdio` / `SSE` / `HTTP` / **`QUIC`** / Temporal-Restate-LangGraph checkpoint event bus) with shared `HeadroomEffortCompressor` payload compression (`Low`/`Medium`/`High`) across all **432** unique vault tools, `Headroom`, `Security`, `Advisory`, `Sandbox`, and `Plugins`.
* **Port `9099`** (`_worker gateway`): Enterprise AI Gateway & Drop-in SDK Endpoints (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/embeddings`, `/v1/security/scan`, `/v1beta/models/*`, `/metrics`, Vertex AI ADC endpoints via `vertex_rust_sdk`), enforcing full `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) before cache & dispatch.
* **Port `9100`** (`_worker dashboard`): Detached Real-Time Web Telemetry Cockpit (`toksi dashboard start|stop|restart`).
* **Port `18787`** (`_worker byok-gateway`): Multi-Instance BYOK & Google Cloud ADC Gateway (`byok_gateway.rs` — supervised by daemon, routing all enabled requests through `HeadroomEffortCompressor` `Low`/`Medium`/`High`).

---

## 2. Core Disciplines, Security Governance & Workspace Isolation

1. **Safe Rust (Edition 2024)**:
   - Strict `#![deny(unsafe_code)]` at `crates/toksi/src/{lib,main}.rs` (`0` unsafe blocks across `456` `.rs` files); platform FFI and C-ABI grammar shims live isolated in `crates/toksi-sys` and `crates/toksi-grammars`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`; must compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
2. **Declarative Configuration (`~/.toksi/config.toml`) & SQLite Store (`~/.toksi/toksi.db`)**:
   - `~/.toksi/config.toml` configures `[repositories]` (`Navegos/advisory` & `Navegos/skills` checkouts + `~/.toksi/user-skills`), custom `[ports]`, `[supervisor]` (`worker_restart_max`, `ipc_transport`), `[testing]` (`default_mode`, `docker_image`), `[auth.gcp_adc]` (`AGY_ADC_AUTH=true`), `[optimization]`, `[security]` (`defenseclaw_fail_closed = true`, `skill_scanner_policy = "balanced"`, `kill_all_services_on_bypass = true`, `adr_enabled = true`, `zero_plaintext_blind_signatures = true`, `sandbox_backend = "auto"`), `[plugins]` (`plugin_dirs`, `enforce_security_scan = true`, `fail_closed_hooks = true`, `protocols`, `allow_agent_to_agent = true`), `[checkpoint]`, `default_agents`, custom API `base_url` endpoints, `direct_wire` defaults, and `[[byok.instances]]`.
   - `~/.toksi/toksi.db` stores `byok_profiles` (supporting **multiple `<byok, model, thinking_effort>` instances per provider** plus `auth_mode`, `gcp_project`, `gcp_location`), `agent_profile_bindings`, `quota_windows`, `plugins_registry` (capabilities, protocols, `blake3_hash`, `security_scan_status`), `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, **`supervisor_workers`**, `provider_keys`, and `provider_models`.
   - **`toksi db export|import`** transfers all tables cleanly (honoring `--overwrite`). Zero plaintext exploits/PoCs are ever stored.
3. **Strict `<repo>/.toksi/<tool>` Repo-Only Storage, `ctxmode` Migration & Plan Checkpoints**:
   - Workspace intelligence tools NEVER create project indexes in `~/.toksi/<tool>/` or outside a `.git` repository root, and NEVER index `<repo>/.git/`.
   - Legacy root/subcrate dot-folders, `.context-mode/`, and `<repo>/.toksi/context/` auto-migrate seamlessly into `<repo>/.toksi/<tool>/`:
     * `<repo>/.toksi/codegraph/` (45-language Tree-sitter `symbols.db`, SCIP, AST-grep, Repomix, OpenWiki, `pmat`)
     * `<repo>/.toksi/projectmem/` (`events.jsonl`, `plan.md`, `PROJECT_MAP.md`, `summary.md`, `heatmap`, and atomic `plan_checkpoints/<plan_id>.json`)
     * **`<repo>/.toksi/ctxmode/`** (`ctxmode` FTS5 DB, `policy`/`journal`/`compaction`; auto-migrates `.context-mode/` and `<repo>/.toksi/context/` in `PHASE_19_1`)
     * `<repo>/.toksi/icm/` (Instant Context Memory SQLite FTS5 `memory.db`)
     * `<repo>/.toksi/ai-memory/` (OKF v0.2 wiki store, handoffs, & Lance columnar vectors)
     * `<repo>/.toksi/security/` (OSV offline CVE/GHSA DB snapshot, `defenseclaw` JSONL audit evidence, & `codesig` quarantine logs)
     * `<repo>/.toksi/plugins/` & `<repo>/.agents/plugins/` (Repo-local Unified Plugin manifests validated under `<enforce>`)
     * `<repo>/.toksi/headroom/` & `<repo>/.toksi/rtk/`
4. **Unified Plugins (`src/plugins/`), Zero-Plaintext Blind Engine, Native Validator, ADR/Sandbox/Kill-Switch, ATR/Guardrails, `<skills>` Healer & 45-Lang SAST**:
   - **Toksi Unified Plugin System (`src/plugins/`, `toksi plugin`, `/plugin`)**: Loads **7 mount surfaces** (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook` Categories 1–3, `protocol_endpoint` across `MCP`/`A2A`/`ACP`/`ANP`/`stdio`/`SSE`/`HTTP`/`QUIC` on `:9098`, and `agent_to_agent` messaging) from `plugin.toml`, `plugin.yaml`, or `openclaw.plugin.json`. Every plugin passes mandatory `<enforce>` verification (`skill-scanner` + `defenseclaw`) before activation and persists in `plugins_registry`.
   - **Zero-Plaintext Blind Signature Engine (`src/security/`) & Native Git/PR Validator (`src/validator/`)**: `toksi compile-blind` converts patterns/AST sliding n-grams (`3,4,5`) into keyed `BLAKE3` (`1.8.7`) digests and little-endian `sparse::DFA` Base64 buffers (zero raw exploits/PoCs/regexes on disk/SQLite/RAM); `toksi advisory` searches `Navegos/advisory` O(1) prefix shards + 12-DB `UpstreamDeduplicator`, enforces `DisclosureGate`, scrubs false positives vs `corpus/benign/`, rebuilds 3-tree GitHub Pages shards, and opens REST PRs without GitHub Actions.
   - **ADR Layer (`src/security/adr.rs`), Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`), & `CommandGuard` Kill-Switch (`src/commands/guard.rs`)**: Intercepts commands/files/web/A2A requests, diverts unsafe payloads into 3-OS native / `sandbox-runtime` (WFP + HTTP/SOCKS5 proxy) / Docker Sandboxes `sbx` (`toksi sandbox` parity), enforces the 6-point offensive security prohibition, and immediately shuts down **all `toksi` services** upon any policy-bypass attempt.
   - **`<enforce>` + `defenseclaw` + `skill-scanner` + `ATR` + `openai-guardrails-python` + `model_spec`**: Wraps critical rules, plugins, and hook templates inside `<enforce>...</enforce>`; enforces fail-closed JSON-RPC/tool-call hookguard, tool-chain rules (`tool-chains.json`), `codesig` verification, ATR (`19,699` Sigma-like rule specs) + OpenAI Guardrails tripwires + `model_spec` instruction hierarchy + YARA/MCP + CEL scanning across 5 presets, and append-only JSONL audit evidence.
   - **Zero-Bloat Externalized `<skills>`, `skilldoctor` + `agentskills` + `upskill` Healer (`toksi skills`)**: Stages skills/rules outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`, synced to `Navegos/skills` + `~/.toksi/user-skills`), grades 0–100 / A–F (`skilldoctor`), evaluates distilled skill uplift (`upskill`), repairs YAML frontmatter (`skills-ref` + `--fix`), deduplicates colliding triggers, and audits/cleans `<neo>/.agents/skills/` (`922` dirs in `PHASE_19_4`).
   - **45-Language AST Static Code Security Analyzer & OSV/EPSS Lockfile Scanner (`toksi security scan`)**: Pure-Rust AST SAST patterns + lockfile CVE/SBOM/EPSS parsing emitting JSON, SARIF 2.1.0, Markdown, Table, or HTML reports with optional `PATH`-checked native linters (`rust-analyzer`, `gopls`, `clang-tidy`).
5. **Universal Forward-Slash (`/`) Path Standard & `DirectWireBus`**:
   - Forward slashes `/` across Windows, Linux, and macOS (`\\` escaped exclusively in Windows `cmd.exe` hook strings).
   - Hook syntax `<tool>-hook <agent>` (`ctxmode-hook <agent>` in `PHASE_19_1`); normalized MCP identifiers `projectmem`, `ctxmode`, `codegraph`, `icm`, `aimemory`, `rtk`, `security`; IPC `DirectWireBus`.

---

## 3. Open BYOK, Google Cloud ADC (`AGY_ADC_AUTH=true`), & Thinking-Effort Optimization

* **Zero Paid-Tier Restrictions**: Zero `FreeTierDisqualified` or `[PAID]` paywalls exist anywhere in Toksi. Any user-configured BYOK, Google Cloud ADC, OAuth, or local CLI profile routes freely across all 65/90+ ecosystem providers and **30+** local & Rust-native inference engines (`Ollama`, `vLLM`, `llama.cpp`, `candle`, `candle-vllm`, `mistral.rs`, `dynamo`, `oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `whisper.cpp`, `stable-diffusion.cpp`, `LM Studio`, `SGLang`, `triton`, `DeepSpeed-MII`, `Exo`, `MLX`, `TensorRT-LLM`, `frontend-crates`, `nixl`, etc.).
* **First-Class Google Cloud ADC Gateway (`AGY_ADC_AUTH=true`)**:
  - Reads `AGY_ADC_AUTH=true`, VS Code `"gcp.project"` / `"gcp.location"` (`us-central1` default), `~/.toksi/config.toml` `[auth.adc]`, and `~/.config/gcloud/application_default_credentials.json` (`%APPDATA%/gcloud/application_default_credentials.json` on Windows), accelerated by pure-Rust `vertex_rust_sdk`.
  - Performs native pure-Rust OAuth2 refresh-token / service-account JWT exchange against `oauth2.googleapis.com/token` and signs requests to `https://{location}-aiplatform.googleapis.com/v1beta1/projects/{project}/locations/{location}/publishers/google/models/{model}`.
  - Emits exact remediation (`gcloud auth application-default login --project <GCP_PROJECT>`) whenever credentials or project IDs are missing.
* **Thinking-Effort-Aware Token Optimization (`Low` / `Medium` / `High`)**:
  - Single `/effort` interface (`--effort`, header `x-toksi-effort`, or gateway JSON body) maps seamlessly across OpenAI (`reasoning_effort`), Anthropic (`thinking.budget_tokens`), and Gemini/Vertex (`generationConfig.thinkingConfig.thinkingBudget`) — silently omitting thinking fields on unsupported models.
  - Dynamically scales shared `HeadroomEffortCompressor` across ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098` Protocol Bus, `:9099` Enterprise AI Gateway, `:18787` BYOK & Vertex ADC Gateway).

---

## 4. Native Coding CLI (`toksi chat|run|exec|swarm|train|serve|eval`), Unified Plugins, Safe Resume & 21 Target Slash Commands

Launch the 35-component `<toksi-siagent>` directly as your standalone AI coding CLI, multi-agent workforce/spec orchestrator, or training/serving plane — wired out of the box ("off the back") via Named Pipe/UDS `DirectWireBus` and local endpoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`) across all **432** unique deep-researched vaults:

```bash
# Interactive coding session with 5-backend sandbox, OPA/Cedar/SPIRE/<enforce> guardrails & durable plan checkpoints
toksi chat --effort medium

# Single-shot autonomous task, blind signature compile, advisory search, or skill/plugin scan
toksi run "Refactor gateway failover tests and verify clippy"
toksi security scan --format sarif
toksi advisory search "rustls"
toksi skills sync && toksi plugin list

# Multi-agent workforce/DAG swarm, spec-driven workflow, & crash/429-safe checkpoint resume
toksi swarm "Audit all 45 language grammars and heal SKILL.md frontmatter"
toksi agent resume [<plan_id>]

# Model training, local serving, evals & live-safe Docker/sandbox isolated tests (Wave 19)
toksi train --recipe unsloth-qlora && toksi serve --engine oxillama && toksi eval --suite lighteval
toksi test --docker   # or: toksi test --sandbox (zero impact on live host services)
```

**Target 21 Instant In-Chat Slash Commands** (`16` core live + `PHASE_19_1`/`19_5`/`19_6` expansions `< 5ms` non-blocking execution in `toksi chat` and hooked agents):
- `/byok [filter]` -> select BYOK provider or Google ADC profile
- `/model [filter]` -> select model discovered via zero-cost `/v1/models`
- `/effort [low|medium|high]` -> set thinking budget & scale Headroom compression
- `/byokmodel <byok> <model> [effort]` -> atomic profile binding update
- `/projectmem ...`, `/ctxmode ...` *(Wave 19.1 rename from `/context`)*, `/codegraph ...`, `/icm ...`, `/ai-memory ...`, `/rtk ...`
- `/security [scan [path]|scan-skills [path]|guardrails|audit-evidence|adr|rules|audit-skills]` -> run 45-lang SAST/OSV, blind signature scan, ADR audit, `skill-scanner`, `defenseclaw` + ATR + OpenAI guardrails, & `agentskills` YAML healer
- `/advisory [search <q>|validate|prune-fp|build-shards|publish-pr|set-dir <path>]` -> query `Navegos/advisory` + 12 upstream DBs, `DisclosureGate`, & publish PRs
- `/skills [sync|list|import <dir>|heal|grade|scan|push|set-dir <path>]` -> fetch vetted skills from `Navegos/skills` + user dir, grade/heal/eval via `skilldoctor` + `agentskills` + `upskill` (zero binary bloat)
- `/sandbox [run|create|exec|ls|inspect|stop|rm|policy|secret]` -> launch & control 3-OS native / `sandbox-runtime` / `sbx` isolated sandboxes
- `/plugin [list|load <id>|unload <id>|reload <id>|info <id>|scan <path>|enable <id>|disable <id>]` -> manage Unified Plugins (`src/plugins/`) under mandatory `<enforce>` scan
- `/resume [plan_id]` -> hydrate `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` and resume from last uncompleted step
- `/workforce [status|spawn|handoff|pause|resume]` *(Wave 19.5)* -> orchestrate multi-agent swarms (`OMA`, `swarms-rs`, `ChatDev`, `deer-flow`)
- `/spec [propose|apply|verify|archive]` *(Wave 19.5)* -> `OpenSpec` spec-driven change lifecycle
- `/train [start|status|recipes|stop]` *(Wave 19.6)* -> `toksi train` LoRA/QLoRA/GRPO/Automodel/Nanotron pipelines
- `/serve [start|swap|unload|status|quant]` *(Wave 19.6)* -> hot-swap `oxillama`/`xinfer`/`Crane`/`paddler`/`llama-swap`/`triton` models
- `/voice [join|mute|transcribe|status]` *(Wave 19.6)* -> real-time `livekit*` + `whisper.cpp` voice turn-taking

---

## 5. Ground-Truth Telemetry, `tokview` & Unified 9-Tab UI (TUI + Web Cockpit)

* **Zero Synthetic Metrics**: Idle sessions read strictly `0`. Tokens, `tokview` / Langfuse / LangWatch spans, per-effort costs, and quotas parse directly from live SSE/Responses streams and HTTP rate-limit headers.
* **Exact 9-Tab Parity (`toksi tui`/`top` and `http://127.0.0.1:9100`)**:
  1. **Savings**: `tokview` trace attribution, configured-only `<byok, model, effort>` & ADC savings.
  2. **The War Room**: Live active `_worker` subprocesses, `:9098` Protocol Bus (`MCP`/`A2A`/`ACP`/`ANP`/`QUIC`), 429 backoff/cascade telemetry, `ArcSwap` reconnect heal, workforce live state.
  3. **AI Agent Fleet & Quotas**: Registered agents (greys out missing `PATH` binaries), `Direct-Wire`/`Config-Wire`, `hour..year` quota gauges.
  4. **Memory Decision DAG**: Interactive `projectmem` DAG, OpenWiki maps, paused `.toksi/projectmem/plan_checkpoints/` with `/resume`, clickable `.heatmap-cell` modal.
  5. **CodeGraph Explorer**: Live `symbols.db` stats across 45 language IDs, SCIP/Repomix/AST-grep/`pmat`, `Navegos/advisory` O(1) prefix search + `publish-pr` status, `skill-scanner` + ATR + built-in SAST security findings.
  6. **Sandbox Inspector**: Live OS sandbox (`native`/`sandbox-runtime`/`sbx`), ADR interception & quarantine events, `CommandGuard` offensive-policy + bypass kill-switch audit log, `<enforce>` & `defenseclaw` + `openai-guardrails-python` blocks, FTS5 search bar.
  7. **System Controls**: Live `~/.toksi/config.toml` editor (`[repositories]`, `[supervisor]`, `[testing]`, `[plugins]`, `[security]`), `_worker` process restart/controls, interactive Unified Plugin Manager (`/api/plugins`), External `<skills>` Sync Manager (`Navegos/skills` + `922`-dir cleanup audit, zero binary bloat), `AGY_ADC_AUTH=true` & `/effort` toggles, `skilldoctor`/`agentskills` healer diff preview, granular component installer.
  8. **AI Gateway & Quotas**: Multi-instance `<byok, model, effort>` & Google ADC CRUD, custom `base_url`, zero-cost `/v1/models` refresh, MoFA 7-D router, local serving/training/quant status (`llama-swap`/`paddler`/`oxillama`/`unsloth`).
  9. **Keyring & Database**: Configured-only OS Keyring (`gemini:antigravity` + ADC), SQLite table explorer (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, `supervisor_workers`), DB `Export` / `Import` / `Backup` / `Restore`.

---

## 6. Supported Agent Layouts & CLI Quick Reference

* **Agent Conventions & User-Level vs Explicit Repo Opt-In**:
  - **Configured-Only & User-Level Auto-Install**: Hooks, MCPs, instructions, rules, skills, and configs auto-install ONLY to user-level directories (`~/.<agent>/` or `~/.agents/`) ONLY for user-configured/selected agents, with granular component toggles (`--components hooks,mcp,skills,rules,instructions,configs`).
  - **Explicit Repo Opt-In**: Workspace repository installation (`<repo>/.<agent>/`) is NEVER automatic; it only installs into root `<repo>/` when explicitly requested (`--workspace`), never into nested crate subfolders. Nested `.<agent>/` and `.toksi/` folders are auto-merged into `<repo>/` on `init` / `doctor --fix` / `agent fix --workspace`.
  - **Open Agents Standard (`.agents/`)**: `mcp.json`, `models.json`, `system-prompt.md`, `agents.md`, `agents/`, `skills/`, `tasks/`, `knowledge/`, `memories/`, `layouts/`, `.agents/mcp -> skills/<tool>`, plus `"openagents"` adapter (`DirectWireBus` `direct-wired` + `mcp_adapters.rs`).
  - **Vendor Directory Alignment**: `.gemini/` (`settings.json`, `commands/*.toml`, split `antigravity-cli` vs `antigravity-ide`), `.copilot/` (`hooks/`) & `.github/copilot/` (split `copilot-cli` vs `copilot-vscode`), `.claude/` (`.mcp.json`), `.codex/` (`agents/`, `plugins/`, `rules/`, `skills/`, `worktrees/`, `config.toml`, `hooks.json`), `.cursor/`, `.grok/` (`hooks/*.json`), `.opencode/` (`hooks.json`).
  - **Lifecycle Operations**: `toksi agent install|health|mismatch|upgrade|fix|restore|remove|edit|list|resume`.
* **Declarative `~/.toksi/config.toml` Reference**:
  ```toml
  default_agents = ["claude", "antigravity-cli", "copilot-cli", "openagents"]

  [ports]
  headroom = 8787
  grok_oauth = 8788
  copilot_vscode = 8789
  copilot_cli = 8790
  protocol_bus = 9098
  gateway = 9099
  dashboard = 9100
  byok_gateway = 18787

  [supervisor] # Wave 19.2
  worker_restart_max = 5
  ipc_transport = "auto" # auto | named-pipe | uds
  worker_health_interval_ms = 2000

  [testing] # Wave 19.3
  default_mode = "sandbox" # sandbox | docker | local-isolated
  docker_image = "toksi-test:latest"

  [repositories]
  neo = "d:/AI/neo"
  advisory = "d:/AI/neo/inspect-vault/advisory"
  skills = "d:/AI/neo/inspect-vault/skills"
  user_skills = "~/.toksi/user-skills"

  [auth.gcp_adc]
  enabled = true # AGY_ADC_AUTH=true
  project = "my-gcp-project"
  location = "us-central1"

  [optimization]
  default_effort = "medium" # low | medium | high

  [security]
  defenseclaw_fail_closed = true
  skill_scanner_policy = "balanced"
  kill_all_services_on_bypass = true
  adr_enabled = true
  zero_plaintext_blind_signatures = true
  sandbox_backend = "auto" # native | sandbox-runtime | openshell | e2b | sbx

  [plugins]
  plugin_dirs = ["~/.toksi/plugins", ".toksi/plugins", ".agents/plugins"]
  enforce_security_scan = true
  fail_closed_hooks = true
  protocols = ["mcp", "a2a", "acp", "anp", "stdio", "sse", "http", "quic"]
  allow_agent_to_agent = true
  ```
* **Complete CLI Command Reference (`toksi --help` 1:1)**:
  - `toksi chat` — Interactive `<toksi-siagent>` coding CLI (`chat|run|exec|swarm`)
  - `toksi run` — Single-shot autonomous task execution with durable checkpoints
  - `toksi exec` — Execute task inside 5-backend isolated agent sandbox (`native`, `sandbox-runtime`, `OpenShell`, `E2B`, `sbx`)
  - `toksi swarm` — Multi-agent parallel workforce/swarm orchestrator (`OMA`, `swarms-rs`, `ChatDev`, `OpenSpec`)
  - `toksi train` *(planned 19.6)* — External/subprocess model fine-tuning orchestration (`unsloth*`, `Automodel`, `nanotron`)
  - `toksi serve` *(planned 19.6)* — Manage local pure-Rust/C++ serving engines (`oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `triton`)
  - `toksi eval` *(planned 19.6)* — Standardized model & guardrail benchmark evaluation (`lighteval`, `langwatch`)
  - `toksi [init]` — Install + wire configured agents and tools (default; safe to re-run)
  - `toksi update` — Update Toksi CLI, inspect version diff, and upgrade tools
  - `toksi doctor` — Audit 18+ agents, consolidate `<repo>/.toksi/<tool>`, migrate `.context-mode/` -> `.toksi/ctxmode/`, fix regressions (`--fix`, `--workspace`)
  - `toksi gain` — Display ground-truth token savings & effort-tier `tokview` attribution
  - `toksi top` — Live interactive terminal monitor for proxies and active agent requests
  - `toksi tui` — Full mouse-capable 9-tab async terminal control center
  - `toksi dashboard` — Web telemetry cockpit on `:9100` (`start|stop|restart --port --open --agents`)
  - `toksi prime` — Generate unified context primer for AI agent sessions
  - `toksi daemon` — Unified 8-port multi-process supervisor (`start|stop|restart|status --agents`) spawning `toksi _worker <role>` subprocesses
  - `toksi _worker` *(internal supervisor child, planned 19.2)* — Run isolated port/agent/tool worker over Named Pipe / UDS IPC
  - `toksi proxy` — Manage streaming Headroom/Copilot/Grok proxies (`start|stop|restart|status --agents`)
  - `toksi config` — Manage `~/.toksi/config.toml` (`[repositories]`, `[supervisor]`, `[testing]`, `[plugins]`, `[security]`, `[ports]`, `[auth.gcp_adc]`)
  - `toksi byok` — Multi-instance `<byok, model, effort>` profiles & read-only `quota_windows` (zero paid-tier blocks)
  - `toksi auth` — OS Keyring & Google Cloud ADC (`AGY_ADC_AUTH=true`) manager: `status|get|set|delete|sync`
  - `toksi db` — SQLite `~/.toksi/toksi.db` manager: `status|export|import|backup|restore`
  - `toksi security` — `opa`/`cedar` Policy-as-Code, `spire` IAM, `Guardrails`/`llm-guard`/`openai-guardrails`/`ATR`/`model_spec`, ADR, 45-lang SAST & OSV/EPSS scanner
  - `toksi compile-blind` — Zero-Plaintext Blind Signature Engine (`BLAKE3` + `sparse::DFA`)
  - `toksi advisory` — Native Git/PR Validator & 12-DB `UpstreamDeduplicator` for `Navegos/advisory`
  - `toksi skills` — Externalized zero-bloat `<skills>` sync, `audit-neo` (`PHASE_19_4` 922-dir hygiene), `skilldoctor`, `agentskills` & `upskill`
  - `toksi sandbox` — 5-backend isolated sandbox (`native|sandbox-runtime|OpenShell|E2B|sbx` CLI)
  - `toksi plugin` — Toksi Unified Plugin System (`<enforce>` scanned, 7 mount surfaces)
  - `toksi test` — Live-safe incremental BLAKE3 test runner (`--all`, `--force`, **`--docker`**, **`--sandbox`** via `.toksi/test-baseline.json`)
  - `toksi agent` — Agent directory lifecycle: `install|health|mismatch|upgrade|fix|restore|remove|edit|list|resume`
  - `toksi mem` — ProjectMem (`<repo>/.toksi/projectmem/`) persistent memory & `plan_checkpoints`
  - **`toksi ctxmode`** *(renamed in `PHASE_19_1`; deprecated hidden `ctx` alias)* — CtxMode (`<repo>/.toksi/ctxmode/`) isolated sandbox & SQLite FTS5 chunker
  - `toksi graph` — CodeGraph (`<repo>/.toksi/codegraph/`) 45-lang Tree-sitter AST, SCIP, `pmat` & OpenWiki
  - `toksi icm` — Instant Context Memory (`<repo>/.toksi/icm/`) FTS5 BM25 & `funes` temporal recall
  - `toksi handoff` — Hybrid Graph-Vector Memory (`Cognee`/`Graphiti`/`Qdrant`/`Lance`/`Letta`) & handoffs
  - `toksi rtk` — Execute command with 100+ token-reduction filters and ANSI stripping
  - `toksi rules` — Sync externalized `<enforce>` Karpathy/Caveman v4/Ponytail guidelines
  - `toksi info` — Show installation details, paths, config locations, and database status
  - `toksi index` — Build per-project AST code graph index under `<repo>/.toksi/codegraph/`
  - `toksi disable` — Disable agent hooks and proxy routing without removing files
  - `toksi uninstall` — Remove everything Toksi ever touched, hooks, and environment variables
  - `toksi self-update` — In-place binary self-update to latest release
  - **Architecture Guides (8-State Progress Taxonomy)**: **43** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_19_6`: `18` `[PLAN-IMPLEMENTED]`, `6` `[PLAN-FIXED]`, `7` `[PLAN-NOT-DONE]`, `7` `[PLAN-NEEDS-FIX]`, `4` `[PLAN-UNIMPLEMENTED]`; all `42` phases `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]`) across all **432** mandated vaults.
* **Quick Commands**:
  ```bash
  toksi config show                                 # Inspect ~/.toksi/config.toml, [supervisor], [testing] & AGY_ADC_AUTH
  toksi daemon start --agents claude,agy            # Supervise 8-port multi-process workers (:8787..:18787)
  toksi dashboard start|stop|restart                # Detached Web Cockpit on :9100
  toksi byok add|list|update|delete ...             # Multi-instance <byok, model, effort> & ADC CRUD
  toksi plugin list|install|enable|disable|scan|run # Manage Unified Plugins under <enforce>
  toksi security scan|scan-skills|guardrails|audit-evidence|adr --format sarif # ADR + ATR + defenseclaw + skill-scanner + 45-lang SAST
  toksi compile-blind <input>                       # Keyed BLAKE3 + sparse::DFA zero-plaintext compiler
  toksi advisory search|validate|prune-fp|build-shards|publish-pr # Navegos/advisory 12-DB dedup & REST PR driver
  toksi skills sync|audit-neo|heal|grade|scan|push  # External Navegos/skills + 922-dir hygiene + skilldoctor/upskill
  toksi sandbox run|create|exec|ls|inspect|stop|rm|policy|secret # 3-OS native + sandbox-runtime + sbx parity
  toksi ctxmode search "query"                      # Query <repo>/.toksi/ctxmode/ FTS5 index (legacy .toksi/context auto-migrates)
  toksi test --docker                               # Run full live-safe container E2E test suite
  toksi agent resume [<plan_id>]                    # Resume paused .toksi/projectmem/plan_checkpoints/
  toksi doctor [--fix]                              # Audit PATH, direct-wire IPC, legacy dotdir/ctxmode merge
  toksi tui | top                                   # 9-tab async mouse TUI
  ```

---

## 7. Externalized Rules & `<skills>` Staging (`Zero Binary Bloat`) + Live-Safe Build / Verification

Curated rules and skills (**Principles** Karpathy, **Caveman v4**, **Ponytail**, and vetted skill packs cleaned from `<neo>/.agents/skills/`) live externally outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`, synced to `<skills>` `Navegos/skills` and configurable `~/.toksi/user-skills`) wrapped in `<enforce>...</enforce>` delimiters and validated by `skilldoctor` + `agentskills` + `upskill` + `skill-scanner`/`defenseclaw` — keeping the compiled `toksi` binary lean with zero embedded `.md`/`.yaml`/`.json` skill bloat.

```bash
cargo build --release -p toksi
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p toksi                               # Ephemeral TOKSI_HOME & :0 ports; honors .toksi/test-baseline.json
toksi test --docker                               # Full isolated container E2E (never touches live host services)
```
Supports `.toksi/test-baseline.json` (`35` suites) BLAKE3 incremental unchanged-suite skipping (`[SKIP-UNCHANGED] <suite>`, override with `--all`/`--force`) while always running `cargo check` and `cargo clippy -- -D warnings`.

---

## 8. License

Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
