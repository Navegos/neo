# Toksi: Native Consolidation Master Engine (Wave 18 / v5.2)

[![Crates.io](https://img.shields.io/crates/v/toksi.svg)](https://crates.io/crates/toksi)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**Toksi** is an enterprise-grade, **100% Native Safe Rust (`#![deny(unsafe_code)]`)** consolidation across the **3-repository ecosystem** (`<neo>` `Navegos/neo`, `<advisory>` `Navegos/advisory`, `<skills>` `Navegos/skills`) unifying **295 unique mandated upstream research vaults** (`292` unique `<deep-research>` vaults in `RESEARCHPLAN.txt:28–320` + `<advisory>` + `<skills>` + `agents-structures` = `36` Waves 1–17 + `259` Wave 18 across Sections `2.1`–`2.10`) and the **35-component `<toksi-siagent>` Super Intelligence Architecture (`RESEARCHPLAN.md` Section 0.2: 26 base `<agent>` + 9 `<siagent>` extensions)** — spanning token optimization, intelligent proxying, **Toksi Unified Plugin System (`src/plugins/`)**, multi-protocol agent wire integration (`MCP` / `A2A` / `ACP` / `stdio` / `SSE` / `HTTP`), Policy-as-Code (`opa`/`cedar`) & SPIFFE IAM (`spire`), `Guardrails`/`llm-guard`/`safeagents`/`garak`/Anonymizer, **Zero-Plaintext Blind Signature Engine (`BLAKE3` + `sparse::DFA`, `src/security/`)**, **Native Git/PR Validator (`src/validator/` + 12-DB `UpstreamDeduplicator`)**, **ADR Layer (`src/security/adr.rs`)**, **5-Backend Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`: 3-OS native + `sandbox-runtime` + `OpenShell` + `E2B` + `sbx` CLI)**, **`CommandGuard` Bypass Kill-Switch (`src/commands/guard.rs`)**, fail-closed runtime governance + multi-engine/CEL skill scanning + built-in 41-language static code security & `skilldoctor`/`agentskills` YAML healer across externalized `<skills>` (zero binary bloat), Google Cloud ADC (`AGY_ADC_AUTH=true`) Vertex gateway, thinking-effort-aware (`Low`/`Medium`/`High`) compression, durable Temporal/Restate/LangGraph workflow state & atomic plan checkpoints (`/resume`), autonomous coding runtime (`toksi chat|run|exec|swarm`), and 9-tab telemetry cockpits for AI coding assistants.

Toksi sits transparently between **18+ registered AI coding assistants** (expanded across **65/90+ Categories 1–5 ecosystem providers** and **Open Agents `.agents/`**) and **26 local & Rust-native inference engines** (`ollama`, `llama.cpp`, `candle`, `candle-vllm`, `mistral.rs`, `dynamo`, `vllm`, `sglang`, `exo`, `mlx`, `lmdeploy`, `TensorRT-LLM`, `frontend-crates`, `nixl`, plus `RouteLLM`/`olla`/`Switchyard` & 140 Section 2.8–2.10 vLLM/DeepSeek/SGLang/HF/NeMo/PyTorch/TRT/Triton/GPUStack/CUDA/Slurm compute/telemetry vaults) plus multi-instance `<byok, model, effort>` & Google Cloud ADC providers — cutting token spend by **30% to 70%** via pure-Rust `headroom-core` `live_zone` cache-preserving compression, `CcrStore`, `TokenEconomizer`, `RTK` 100+ command filters, `Context-Mode` sandboxes, **45 Tree-sitter language IDs** (41+ specified languages + Dart/R/Luau with specialized `dockerfile`/`vue`/`scss` grammars), SCIP/AST-grep/Repomix/jcodemunch/OpenWiki code intelligence, hybrid graph-vector memory (`Qdrant`, `Lance`, `Chroma`, `Cognee`, `Graphiti`, `Milvus`, `Letta`), and repo-scoped zero-LLM memory — with zero Node.js, Python, npm, or pip dependencies and **zero paid-tier restrictions**.

---

## 1. Master Architecture & Unified 8-Port Network Topology

All listening ports are overridable via `~/.toksi/config.toml` (`[ports]`) and isolated so browser dashboards never introduce TCP backpressure on inference streams:

```text
[ AI Coding Assistants / IDEs / Local CLIs / Browsers / ACP Clients ]
  │        │        │        │        │         │         │         │
  │ :8787  │ :8788  │ :8789  │ :8790  │ :9098   │ :9099   │ :9100   │ :18787
  ▼        ▼        ▼        ▼        ▼         ▼         ▼         ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│                  toksi daemon start|stop|restart [--agents ...]                  │
│                                                                                  │
│ ┌────────────────────────────┐      ┌──────────────────────────────────────────┐ │
│ │ :8787 Headroom Proxy       │      │ :9099 Enterprise AI Gateway & SDKs       │ │
│ │ - HTTP/1.1 & HTTP/2 SSE    │      │ - /v1/{chat/completions,messages,        │ │
│ │ - ConnectRPC Passthrough   │      │   responses,embeddings}, /v1beta/*       │ │
│ │ - live_zone Cache Surgery  │      │ - Shared Headroom Effort Compressor      │ │
│ │ - Low/Med/High Effort Scale│      │ - /v1/security/scan & Prometheus /metrics│ │
│ │ - 16 In-Chat /... Commands │      │ - AGY_ADC_AUTH=true Vertex AI Routing    │ │
│ └────────────────────────────┘      │ - 65/90+ Ecosystem & 26 Local/Rust Engines│
│ ┌────────────────────────────┐      │ - MoFA 7-D Router & Chroma/Lance Cache   │ │
│ │ :8788 Grok OAuth Daemon    │      └──────────────────────────────────────────┘ │
│ │ :8789 Copilot VS Code      │      ┌──────────────────────────────────────────┐ │
│ │ :8790 Copilot CLI          │      │ :9098 Unified Protocol Bus               │ │
│ └────────────────────────────┘      │ - MCP/A2A/ACP/SSE + Headroom Effort Pass │ │
│ ┌────────────────────────────┐      │ - All 295 Vaults, <toksi-siagent> & State │
│ │ :18787 BYOK & Vertex ADC   │      └──────────────────────────────────────────┘ │
│ │ - Per-profile base_url/ADC │      ┌──────────────────────────────────────────┐ │
│ │ - Headroom Effort Compress │      │ :9100 Detached Web Telemetry Cockpit     │ │
│ └────────────────────────────┘      │ - 9-Tab UI, tokview, Plugins & Security  │ │
│                                     └──────────────────────────────────────────┘ │
│                                                                                  │
│ [ In-Process DirectWireBus & 35-Component <toksi-siagent> (295 Vaults / 3 Repos) ]         │
│  • RTK (100+ Filters + Tracker)    • ProjectMem (JSONL + plan_checkpoints/)      │
│  • Context-Mode (Sandbox + FTS5)   • ICM (BM25 + Memoir/Feedback/Temporal)       │
│  • CodeGraph (45 TS/SCIP/OpenWiki) • ai-memory (OKF v0.2 Wiki + Lance Vector)    │
│  • Plugins (src/plugins/: 7 mount surfaces + plugins_registry + <enforce> gate)  │
│  • Security (Zero-Plaintext BLAKE3+DFA, ADR, CommandGuard Kill-Switch,           │
│    defenseclaw, skill-scanner CEL/taint, 41-Lang SAST/OSV/EPSS, & Validator PRs) │
│  • Sandbox (3-OS Native + sandbox-runtime WFP/Proxy + sbx 29-Verb Parity)        │
│  • Agent Runtime (OpenCode/Hermes/OpenClaw/MoFA/LangGraph/DeepAgents/Bosun/ACP)  │
│  • Externalized Rules & <skills> (Navegos/skills + skilldoctor/agentskills Heal) │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Dedicated Ports:
* **Port `8787`**: Transparent Streaming Headroom Proxy (Claude, Codex, Antigravity, Cursor, mandatory `/byok/` compression, & `Low`/`Medium`/`High` thinking-effort scaling).
* **Port `8788`**: GitHub/xAI Grok OAuth Proxy Daemon (`grok_daemon.rs` — strictly separate from BYOK).
* **Port `8789`**: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`).
* **Port `8790`**: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`).
* **Port `9098`**: Unified Protocol Bus (`MCP` / `A2A` v1.0 / `ACP` `agent-client-protocol` / `stdio` / `SSE` / `HTTP` / Temporal-Restate-LangGraph checkpoint event bus) with shared `HeadroomEffortCompressor` payload compression (`Low`/`Medium`/`High`) across all **295** unique vault tools, `Headroom`, `Security`, `Advisory`, `Sandbox`, and `Plugins`.
* **Port `9099`**: Enterprise AI Gateway & Drop-in SDK Endpoints (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/embeddings`, `/v1/security/scan`, `/v1beta/models/*`, `/metrics`, Vertex AI ADC endpoints), enforcing full `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) before cache & dispatch.
* **Port `9100`**: Detached Real-Time Web Telemetry Cockpit (`toksi dashboard start|stop|restart`).
* **Port `18787`**: Multi-Instance BYOK & Google Cloud ADC Gateway (`byok_gateway.rs` — supervised by daemon, routing all enabled requests through `HeadroomEffortCompressor` `Low`/`Medium`/`High`).

---

## 2. Core Disciplines, Security Governance & Workspace Isolation

1. **Safe Rust (Edition 2024)**:
   - Strict `#![deny(unsafe_code)]` at `crates/toksi/src/{lib,main}.rs`; platform FFI and C-ABI grammar shims live isolated in `crates/toksi-sys` and `crates/toksi-grammars`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`; must compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
2. **Declarative Configuration (`~/.toksi/config.toml`) & SQLite Store (`~/.toksi/toksi.db`)**:
   - `~/.toksi/config.toml` configures `[repositories]` (`Navegos/advisory` & `Navegos/skills` checkouts + `~/.toksi/user-skills`), custom `[ports]`, `[auth.gcp_adc]` (`AGY_ADC_AUTH=true`), `[optimization]`, `[security]` (`defenseclaw_fail_closed = true`, `skill_scanner_policy = "balanced"`, `kill_all_services_on_bypass = true`, `adr_enabled = true`, `zero_plaintext_blind_signatures = true`, `sandbox_backend = "auto"`), `[plugins]` (`plugin_dirs`, `enforce_security_scan = true`, `fail_closed_hooks = true`, `protocols`, `allow_agent_to_agent = true`), `[checkpoint]`, `default_agents`, custom API `base_url` endpoints, `direct_wire` defaults, and `[[byok.instances]]`.
   - `~/.toksi/toksi.db` stores `byok_profiles` (supporting **multiple `<byok, model, thinking_effort>` instances per provider** plus `auth_mode`, `gcp_project`, `gcp_location`), `agent_profile_bindings`, `quota_windows`, `plugins_registry` (capabilities, protocols, `blake3_hash`, `security_scan_status`), `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, `provider_keys`, and `provider_models`.
   - **`toksi db export|import`** transfers all tables cleanly (honoring `--overwrite`). Zero plaintext exploits/PoCs are ever stored.
3. **Strict `<repo>/.toksi/<tool>` Repo-Only Storage & Plan Checkpoints**:
   - Workspace intelligence tools NEVER create project indexes in `~/.toksi/<tool>/` or outside a `.git` repository root, and NEVER index `<repo>/.git/`.
   - Legacy root/subcrate `.projectmem/` or dot-folders auto-migrate seamlessly into `<repo>/.toksi/<tool>/`:
     * `<repo>/.toksi/codegraph/` (45-language Tree-sitter `symbols.db`, SCIP, AST-grep, Repomix, OpenWiki)
     * `<repo>/.toksi/projectmem/` (`events.jsonl`, `plan.md`, `PROJECT_MAP.md`, `summary.md`, `heatmap`, and atomic `plan_checkpoints/<plan_id>.json`)
     * `<repo>/.toksi/context/` (Context-Mode FTS5 DB, `policy`/`journal`/`compaction`)
     * `<repo>/.toksi/icm/` (Instant Context Memory SQLite FTS5 `memory.db`)
     * `<repo>/.toksi/ai-memory/` (OKF v0.2 wiki store, handoffs, & Lance columnar vectors)
     * `<repo>/.toksi/security/` (OSV offline CVE/GHSA DB snapshot, `defenseclaw` JSONL audit evidence, & `codesig` quarantine logs)
     * `<repo>/.toksi/plugins/` & `<repo>/.agents/plugins/` (Repo-local Unified Plugin manifests validated under `<enforce>`)
     * `<repo>/.toksi/headroom/` & `<repo>/.toksi/rtk/`
4. **Unified Plugins (`src/plugins/`), Zero-Plaintext Blind Engine, Native Validator, ADR/Sandbox/Kill-Switch, `<skills>` Healer & 41-Lang SAST**:
   - **Toksi Unified Plugin System (`src/plugins/`, `toksi plugin`, `/plugin`)**: Loads **7 mount surfaces** (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook` Categories 1–3, `protocol_endpoint` across `MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP` on `:9098`, and `agent_to_agent` messaging) from `plugin.toml`, `plugin.yaml`, or `openclaw.plugin.json`. Every plugin passes mandatory `<enforce>` verification (`skill-scanner` + `defenseclaw`) before activation and persists in `plugins_registry`.
   - **Zero-Plaintext Blind Signature Engine (`src/security/`) & Native Git/PR Validator (`src/validator/`)**: `toksi compile-blind` converts patterns/AST sliding n-grams (`3,4,5`) into keyed `BLAKE3` digests and little-endian `sparse::DFA` Base64 buffers (zero raw exploits/PoCs/regexes on disk/SQLite/RAM); `toksi advisory` searches `Navegos/advisory` O(1) prefix shards + 12-DB `UpstreamDeduplicator`, enforces `DisclosureGate`, scrubs false positives vs `corpus/benign/`, rebuilds 3-tree GitHub Pages shards, and opens REST PRs without GitHub Actions.
   - **ADR Layer (`src/security/adr.rs`), Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`), & `CommandGuard` Kill-Switch (`src/commands/guard.rs`)**: Intercepts commands/files/web/A2A requests, diverts unsafe payloads into 3-OS native / `sandbox-runtime` (WFP + HTTP/SOCKS5 proxy) / Docker Sandboxes `sbx` (`toksi sandbox` 29-verb parity), enforces the 6-point offensive security prohibition, and immediately shuts down **all `toksi` services** upon any policy-bypass attempt.
   - **`<enforce>` + `defenseclaw` + `skill-scanner`**: Wraps critical rules, plugins, and hook templates inside `<enforce>...</enforce>`; enforces fail-closed JSON-RPC/tool-call hookguard, tool-chain rules (`tool-chains.json`), `codesig` verification, YARA/MCP + CEL scanning across 5 presets (`strict`, `balanced`, `low-noise`, `quiet`, `permissive`), and append-only JSONL audit evidence.
   - **Zero-Bloat Externalized `<skills>`, `skilldoctor` & `agentskills` Healer (`toksi skills`)**: Stages skills/rules outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`, synced to `Navegos/skills` + `~/.toksi/user-skills`), grades 0–100 / A–F (`skilldoctor`), repairs YAML frontmatter (`skills-ref` + byte-exact `--fix`), and deduplicates colliding skill triggers.
   - **41-Language AST Static Code Security Analyzer & OSV/EPSS Lockfile Scanner (`toksi security scan`)**: Pure-Rust AST SAST patterns ported from `semgrep`, `semgrep-rules`, `codex-security`, `deepsec`, and `sage` + lockfile CVE/SBOM/EPSS parsing from `osv-scanner`, `cve-bin-tool`, `cve-lite-cli`, emitting JSON, SARIF 2.1.0, Markdown, Table, or HTML reports with optional `PATH`-checked native linters (`rust-analyzer`, `gopls`, `clang-tidy`).
5. **Universal Forward-Slash (`/`) Path Standard & `DirectWireBus`**:
   - Forward slashes `/` across Windows, Linux, and macOS (`\\` escaped exclusively in Windows `cmd.exe` hook strings).
   - Hook syntax `<tool>-hook <agent>`; normalized MCP identifiers `projectmem`, `context`, `codegraph`, `icm`, `aimemory`, `rtk`, `security`; zero-config-edit `DirectWireBus`.

---

## 3. Open BYOK, Google Cloud ADC (`AGY_ADC_AUTH=true`), & Thinking-Effort Optimization

* **Zero Paid-Tier Restrictions**: Zero `FreeTierDisqualified` or `[PAID]` paywalls exist anywhere in Toksi. Any user-configured BYOK, Google Cloud ADC, OAuth, or local CLI profile routes freely across all 65/90+ ecosystem providers and **26** local & Rust-native inference engines (`Ollama`, `vLLM`, `llama.cpp`, `candle`, `candle-vllm`, `mistral.rs`, `dynamo`, `LM Studio`, `SGLang`, `Exo`, `MLX`, `TensorRT-LLM`, `frontend-crates`, `nixl`, etc.).
* **First-Class Google Cloud ADC Gateway (`AGY_ADC_AUTH=true`)**:
  - Reads `AGY_ADC_AUTH=true`, VS Code `"gcp.project"` / `"gcp.location"` (`us-central1` default), `~/.toksi/config.toml` `[auth.adc]`, and `~/.config/gcloud/application_default_credentials.json` (`%APPDATA%/gcloud/application_default_credentials.json` on Windows).
  - Performs native pure-Rust OAuth2 refresh-token / service-account JWT exchange against `oauth2.googleapis.com/token` and signs requests to `https://{location}-aiplatform.googleapis.com/v1beta1/projects/{project}/locations/{location}/publishers/google/models/{model}`.
  - Emits exact remediation (`gcloud auth application-default login --project <GCP_PROJECT>`) whenever credentials or project IDs are missing.
* **Thinking-Effort-Aware Token Optimization (`Low` / `Medium` / `High`)**:
  - Single `/effort` interface (`--effort`, header `x-toksi-effort`, or gateway JSON body) maps seamlessly across OpenAI (`reasoning_effort`), Anthropic (`thinking.budget_tokens`), and Gemini/Vertex (`generationConfig.thinkingConfig.thinkingBudget`) — silently omitting thinking fields on unsupported models.
  - Dynamically scales shared `HeadroomEffortCompressor` across ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098` Protocol Bus, `:9099` Enterprise AI Gateway, `:18787` BYOK & Vertex ADC Gateway): `Low` (aggressive compaction + top-K signatures), `Medium` (balanced AST/semantic compression), `High` (lossless reasoning, deep CodeGraph SCIP + blast radius).

---

## 4. Native Coding CLI (`toksi chat|run|exec|swarm`), Unified Plugins, Safe Resume & 16 Slash Commands

Launch the 35-component `<toksi-siagent>` directly as your standalone AI coding CLI or multi-agent swarm orchestrator — wired out of the box ("off the back") via `DirectWireBus` and local endpoints (`:8787`, `:8788`, `:8789`, `:8790`, `:9098`, `:9099`, `:18787`) to Enterprise AI Gateway, BYOK & Vertex ADC Gateway, Protocol Bus (`MCP`/`A2A`/`ACP`/`SSE`), `HeadroomEffortCompressor`, Unified Plugin System (`src/plugins/`), OPA/Cedar/SPIRE IAM, Guardrails/LLM-Guard, ADR/5-backend Sandbox/`CommandGuard`, Temporal/Restate/LangGraph checkpoints, hybrid `Cognee`/`Graphiti`/`Qdrant`/`Lance`/`Chroma` memory, and all **295** unique consolidated vault engines (`rtk`, `headroom`, `context-mode`, `codegraph`, `projectmem`, `ai-memory`, `icm`, `ponytail`, `caveman`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `lance`, `qdrant`, `cognee`, `graphiti`, `letta`, `agent-client-protocol`, `agent-opt`, `agent-control`, `opencode`, `sandbox-runtime`, `OpenShell`, `E2B`, `opa`, `cedar`, `spire`, `Guardrails`, `llm-guard`, `cve-*`, `osv-schema`, `skilldoctor`, `sage`, `defenseclaw`, `skill-scanner`, `mistral.rs`, `candle`, `vmon`, `cuda-rust`, `srt-slurm`, plus `hermes-agent`, `openclaw`, `mofa`, `crewAI`, `codex`, `goose`, `grok-build`, `gemini-cli`, `langgraph`, `temporal`, `restate`, `deepagents`, `bosun`, `cline`, `kilocode`, `alphacode`, `zeroclaw`):

```bash
# Interactive coding session with 5-backend sandbox, OPA/Cedar/SPIRE/<enforce> guardrails & durable plan checkpoints
toksi chat --effort medium

# Single-shot autonomous task, blind signature compile, advisory search, or skill/plugin scan
toksi run "Refactor gateway failover tests and verify clippy"
toksi security scan --format sarif
toksi advisory search "rustls"
toksi skills sync && toksi plugin list

# Multi-agent DAG swarm & crash/429-safe checkpoint resume
toksi swarm "Audit all 45 language grammars and heal SKILL.md frontmatter"
toksi agent resume [<plan_id>]
```

**16 Instant In-Chat Slash Commands** (`< 5ms` non-blocking execution in `toksi chat` and hooked agents):
- `/byok [filter]` -> select BYOK provider or Google ADC profile
- `/model [filter]` -> select model discovered via zero-cost `/v1/models`
- `/effort [low|medium|high]` -> set thinking budget & scale Headroom compression
- `/byokmodel <byok> <model> [effort]` -> atomic profile binding update
- `/projectmem ...`, `/context ...`, `/codegraph ...`, `/icm ...`, `/ai-memory ...`, `/rtk ...`
- `/security [scan [path]|scan-skills [path]|guardrails|audit-evidence|adr|rules|audit-skills]` -> run 41-lang SAST/OSV, blind signature scan, ADR audit, `skill-scanner`, `defenseclaw` guardrails, & `agentskills` YAML healer
- `/advisory [search <q>|validate|prune-fp|build-shards|publish-pr|set-dir <path>]` -> query `Navegos/advisory` + 12 upstream DBs, `DisclosureGate`, & publish PRs
- `/skills [sync|list|import <dir>|heal|grade|scan|push|set-dir <path>]` -> fetch vetted skills from `Navegos/skills` + user dir, grade/heal via `skilldoctor` + `agentskills` (zero binary bloat)
- `/sandbox [run|create|exec|ls|inspect|stop|rm|policy|secret]` -> launch & control 3-OS native / `sandbox-runtime` / `sbx` isolated sandboxes
- `/plugin [list|load <id>|unload <id>|reload <id>|info <id>|scan <path>|enable <id>|disable <id>]` -> manage Unified Plugins (`src/plugins/`) under mandatory `<enforce>` scan
- `/resume [plan_id]` -> hydrate `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` and resume from last uncompleted step

---

## 5. Ground-Truth Telemetry, `tokview` & Unified 9-Tab UI (TUI + Web Cockpit)

* **Zero Synthetic Metrics**: Idle sessions read strictly `0`. Tokens, `tokview` / Langfuse spans, per-effort costs, and quotas parse directly from live SSE/Responses streams and HTTP rate-limit headers.
* **Exact 9-Tab Parity (`toksi tui`/`top` and `http://127.0.0.1:9100`)**:
  1. **Savings**: `tokview` trace attribution, configured-only `<byok, model, effort>` & ADC savings.
  2. **The War Room**: Live active tools, `:9098` Protocol Bus, 429 backoff/cascade telemetry, `ArcSwap` reconnect heal.
  3. **AI Agent Fleet & Quotas**: Registered agents (greys out missing `PATH` binaries), `Direct-Wire`/`Config-Wire`, `hour..year` quota gauges.
  4. **Memory Decision DAG**: Interactive `projectmem` DAG, OpenWiki maps, paused `.toksi/projectmem/plan_checkpoints/` with `/resume`, clickable `.heatmap-cell` modal.
  5. **CodeGraph Explorer**: Live `symbols.db` stats across 45 language IDs, SCIP/Repomix/AST-grep, `Navegos/advisory` O(1) prefix search + `publish-pr` status, `skill-scanner` CEL policy/taint + built-in 41-language static security findings.
  6. **Sandbox Inspector**: Live OS sandbox (`native`/`sandbox-runtime`/`sbx`), ADR interception & quarantine events, `CommandGuard` offensive-policy + bypass kill-switch audit log, `<enforce>` & `defenseclaw` fail-closed hookguard/tool-chains/audit evidence & `sage` threat blocks, FTS5 search bar.
  7. **System Controls**: Live `~/.toksi/config.toml` editor (`[repositories]`, `[plugins]`, `[security]`), interactive Unified Plugin Manager (`/api/plugins`), External `<skills>` Sync Manager (`Navegos/skills` + user skills dir, zero binary bloat), `AGY_ADC_AUTH=true` & `/effort` toggles, `skilldoctor`/`agentskills` healer diff preview, granular component installer.
  8. **AI Gateway & Quotas**: Multi-instance `<byok, model, effort>` & Google ADC CRUD, custom `base_url`, zero-cost `/v1/models` refresh, MoFA 7-D router.
  9. **Keyring & Database**: Configured-only OS Keyring (`gemini:antigravity` + ADC), SQLite table explorer (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`), DB `Export` / `Import` / `Backup` / `Restore`.

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
  protocols = ["mcp", "a2a", "acp", "stdio", "sse", "http"]
  allow_agent_to_agent = true
  ```
* **Complete CLI Command Reference (`toksi --help` 1:1)**:
  - `toksi chat` — Interactive `<toksi-siagent>` coding CLI (`chat|run|exec|swarm`)
  - `toksi run` — Single-shot autonomous task execution with durable checkpoints
  - `toksi exec` — Execute task inside 5-backend isolated agent sandbox (`native`, `sandbox-runtime`, `OpenShell`, `E2B`, `sbx`)
  - `toksi swarm` — Multi-agent parallel swarm orchestrator with `DirectWireBus`
  - `toksi [init]` — Install + wire configured agents and tools (default; safe to re-run)
  - `toksi update` — Update Toksi CLI, inspect version diff, and upgrade tools
  - `toksi doctor` — Audit 18+ agents, consolidate `<repo>/.toksi/<tool>`, fix regressions (`--fix`, `--workspace`)
  - `toksi gain` — Display ground-truth token savings & effort-tier `tokview` attribution
  - `toksi top` — Live interactive terminal monitor for proxies and active agent requests
  - `toksi tui` — Full mouse-capable 9-tab async terminal control center
  - `toksi dashboard` — Web telemetry cockpit on `:9100` (`start|stop|restart --port --open --agents`)
  - `toksi prime` — Generate unified context primer for AI agent sessions
  - `toksi daemon` — Unified 8-port supervisor (`start|stop|restart|status --agents`)
  - `toksi proxy` — Manage streaming Headroom/Copilot/Grok proxies (`start|stop|restart|status --agents`)
  - `toksi config` — Manage `~/.toksi/config.toml` (`[repositories]`, `[plugins]`, `[security]`, `[ports]`, `[auth.gcp_adc]`)
  - `toksi byok` — Multi-instance `<byok, model, effort>` profiles & read-only `quota_windows` (zero paid-tier blocks)
  - `toksi auth` — OS Keyring & Google Cloud ADC (`AGY_ADC_AUTH=true`) manager: `status|get|set|delete|sync`
  - `toksi db` — SQLite `~/.toksi/toksi.db` manager: `status|export|import|backup|restore`
  - `toksi security` — `opa`/`cedar` Policy-as-Code, `spire` IAM, `Guardrails`/`llm-guard`, ADR, 41-lang SAST & OSV/EPSS scanner
  - `toksi compile-blind` — Zero-Plaintext Blind Signature Engine (`BLAKE3` + `sparse::DFA`)
  - `toksi advisory` — Native Git/PR Validator & 12-DB `UpstreamDeduplicator` for `Navegos/advisory`
  - `toksi skills` — Externalized zero-bloat `<skills>` sync, `skilldoctor` & `agentskills` YAML healer
  - `toksi sandbox` — 5-backend isolated sandbox (`native|sandbox-runtime|OpenShell|E2B|sbx` CLI)
  - `toksi plugin` — Toksi Unified Plugin System (`<enforce>` scanned, 7 mount surfaces)
  - `toksi agent` — Agent directory lifecycle: `install|health|mismatch|upgrade|fix|restore|remove|edit|list|resume`
  - `toksi mem` — ProjectMem (`<repo>/.toksi/projectmem/`) persistent memory & `plan_checkpoints`
  - `toksi ctx` — Context-mode (`<repo>/.toksi/context/`) isolated sandbox & SQLite FTS5 chunker
  - `toksi graph` — CodeGraph (`<repo>/.toksi/codegraph/`) 45-lang Tree-sitter AST, SCIP & OpenWiki
  - `toksi icm` — Instant Context Memory (`<repo>/.toksi/icm/`) FTS5 BM25 & temporal recall
  - `toksi handoff` — Hybrid Graph-Vector Memory (`Cognee`/`Graphiti`/`Qdrant`/`Lance`/`Letta`) & handoffs
  - `toksi rtk` — Execute command with 100+ token-reduction filters and ANSI stripping
  - `toksi rules` — Sync externalized `<enforce>` Karpathy/Caveman v4/Ponytail guidelines
  - `toksi info` — Show installation details, paths, config locations, and database status
  - `toksi index` — Build per-project AST code graph index under `<repo>/.toksi/codegraph/`
  - `toksi disable` — Disable agent hooks and proxy routing without removing files
  - `toksi uninstall` — Remove everything Toksi ever touched, hooks, and environment variables
  - `toksi self-update` — In-place binary self-update to latest release
  - **Architecture Guides**: **37** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01`..`PHASE_18_6`) across all **295** mandated vaults (`36` + `259` Wave 18 Sections `2.1`–`2.10`).
* **Quick Commands**:
  ```bash
  toksi config show                                 # Inspect ~/.toksi/config.toml, [repositories], [plugins] & AGY_ADC_AUTH
  toksi daemon start --agents claude,agy            # Supervise 8-port daemon (:8787..:18787)
  toksi dashboard start|stop|restart                # Detached Web Cockpit on :9100
  toksi byok add|list|update|delete ...             # Multi-instance <byok, model, effort> & ADC CRUD
  toksi plugin list|install|enable|disable|scan|run # Manage Unified Plugins under <enforce>
  toksi security scan|scan-skills|guardrails|audit-evidence|adr --format sarif # ADR + defenseclaw + skill-scanner + 41-lang SAST
  toksi compile-blind <input>                       # Keyed BLAKE3 + sparse::DFA zero-plaintext compiler
  toksi advisory search|validate|prune-fp|build-shards|publish-pr # Navegos/advisory 12-DB dedup & REST PR driver
  toksi skills sync|list|import|heal|grade|scan|push # External Navegos/skills + skilldoctor/agentskills healer
  toksi sandbox run|create|exec|ls|inspect|stop|rm|policy|secret # 3-OS native + sandbox-runtime + sbx 29-verb parity
  toksi rules sync | toksi rules heal-skills        # Sync <enforce> externalized rules & heal SKILL.md frontmatter
  toksi agent resume [<plan_id>]                    # Resume paused .toksi/projectmem/plan_checkpoints/
  toksi doctor [--fix]                              # Audit PATH, direct-wire, legacy .projectmem/ merge
  toksi tui | top                                   # 9-tab async mouse TUI
  ```

---

## 7. Externalized Rules & `<skills>` Staging (`Zero Binary Bloat`) + Build / Verification

Curated rules and skills (**Principles** Karpathy, **Caveman v4**, **Ponytail**, and vetted skill packs) live externally outside `crates/toksi/` (`<neo>/agents/`, `<neo>/rules/`, synced to `<skills>` `Navegos/skills` and configurable `~/.toksi/user-skills`) wrapped in `<enforce>...</enforce>` delimiters and validated by `skilldoctor` + `agentskills` + `skill-scanner`/`defenseclaw` — keeping the compiled `toksi` binary lean with zero embedded `.md`/`.yaml`/`.json` skill bloat.

```bash
cargo build --release -p toksi
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p toksi
```
Supports `.toksi/test-baseline.json` BLAKE3 incremental unchanged-suite skipping (`[SKIP-UNCHANGED] <suite>`, override with `--all`/`--force`) while always running `cargo check` and `cargo clippy -- -D warnings`.

---

## 8. License

Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
