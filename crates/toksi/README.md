# Toksi: Native Consolidation Master Engine (Wave 18 / v5.0)

[![Crates.io](https://img.shields.io/crates/v/toksi.svg)](https://crates.io/crates/toksi)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**Toksi** is an enterprise-grade, **100% Native Safe Rust (`#![deny(unsafe_code)]`)** consolidation of **67 audited upstream research vaults** (`36` Waves 1–17 + `31` Wave 18 incl. `defenseclaw` and `skill-scanner`), token optimization, intelligent proxying, **Toksi Unified Plugin System (`src/plugins/`)**, multi-protocol agent wire integration (`MCP` / `A2A` / `ACP` / `stdio` / `SSE` / `HTTP`), fail-closed runtime governance + multi-engine/CEL skill scanning + built-in 41-language static code security & `agentskills` YAML frontmatter governance (`<enforce>`), Google Cloud ADC (`AGY_ADC_AUTH=true`) Vertex gateway, thinking-effort-aware (`Low`/`Medium`/`High`) compression, atomic plan checkpoints (`/resume`), autonomous coding runtime (`toksi chat|run|exec|swarm`), and 9-tab telemetry cockpits for AI coding assistants.

Toksi sits transparently between **18+ registered AI coding assistants** (expanded across **65/90+ Categories 1–5 ecosystem providers** and **Open Agents `.agents/`**) and **22 local inference engines** plus multi-instance `<byok, model, effort>` & Google Cloud ADC providers — cutting token spend by **30% to 70%** via pure-Rust `headroom-core` `live_zone` cache-preserving compression, `CcrStore`, `TokenEconomizer`, `RTK` 100+ command filters, `Context-Mode` sandboxes, **45 Tree-sitter language IDs** (41+ specified languages + Dart/R/Luau), SCIP/AST-grep/Repomix/jcodemunch/OpenWiki code intelligence, Lance columnar + SQLite hybrid memory, and repo-scoped zero-LLM memory — with zero Node.js, Python, npm, or pip dependencies and **zero paid-tier restrictions**.

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
│ │ - 13 In-Chat /... Commands │      │ - AGY_ADC_AUTH=true Vertex AI Routing    │ │
│ └────────────────────────────┘      │ - 65/90+ Ecosystem & 22 Local Engines    │ │
│ ┌────────────────────────────┐      │ - MoFA 7-D Router & Chroma/Lance Cache   │ │
│ │ :8788 Grok OAuth Daemon    │      └──────────────────────────────────────────┘ │
│ │ :8789 Copilot VS Code      │      ┌──────────────────────────────────────────┐ │
│ │ :8790 Copilot CLI          │      │ :9098 Unified Protocol Bus               │ │
│ └────────────────────────────┘      │ - MCP/A2A/ACP/SSE + Headroom Effort Pass │ │
│ ┌────────────────────────────┐      │ - All 67 Vaults, Plugins & Checkpoints   │ │
│ │ :18787 BYOK & Vertex ADC   │      └──────────────────────────────────────────┘ │
│ │ - Per-profile base_url/ADC │      ┌──────────────────────────────────────────┐ │
│ │ - Headroom Effort Compress │      │ :9100 Detached Web Telemetry Cockpit     │ │
│ └────────────────────────────┘      │ - 9-Tab UI, tokview, Plugins & Security  │ │
│                                     └──────────────────────────────────────────┘ │
│                                                                                  │
│ [ In-Process DirectWireBus & Native Rust Engines (67 Vaults) ]                   │
│  • RTK (100+ Filters + Tracker)    • ProjectMem (JSONL + plan_checkpoints/)      │
│  • Context-Mode (Sandbox + FTS5)   • ICM (BM25 + Memoir/Feedback/Temporal)       │
│  • CodeGraph (45 TS/SCIP/OpenWiki) • ai-memory (OKF v0.2 Wiki + Lance Vector)    │
│  • Plugins (src/plugins/: 7 mount surfaces + plugins_registry + <enforce> gate)  │
│  • Security (defenseclaw fail-closed hookguard/audit, skill-scanner CEL/taint,   │
│    41-Lang SAST/OSV, & agentskills YAML Frontmatter Healer)                      │
│  • Agent Runtime (Hermes/OpenClaw/MoFA/LangGraph/DeepAgents/Bosun/ZeroClaw/ACP)  │
│  • Rules (Karpathy, Caveman v4, Ponytail compiled via include_str! w/ <enforce>) │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Dedicated Ports:
* **Port `8787`**: Transparent Streaming Headroom Proxy (Claude, Codex, Antigravity, Cursor, mandatory `/byok/` compression, & `Low`/`Medium`/`High` thinking-effort scaling).
* **Port `8788`**: GitHub/xAI Grok OAuth Proxy Daemon (`grok_daemon.rs` — strictly separate from BYOK).
* **Port `8789`**: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`).
* **Port `8790`**: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`).
* **Port `9098`**: Unified Protocol Bus (`MCP` / `A2A` v1.0 / `ACP` `agent-client-protocol` / `stdio` / `SSE` / `HTTP` / checkpoint event bus) with shared `HeadroomEffortCompressor` payload compression (`Low`/`Medium`/`High`) across all 67 vault tools, `Headroom`, `Security`, and `Plugins`.
* **Port `9099`**: Enterprise AI Gateway & Drop-in SDK Endpoints (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/embeddings`, `/v1/security/scan`, `/v1beta/models/*`, `/metrics`, Vertex AI ADC endpoints), enforcing full `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) before cache & dispatch.
* **Port `9100`**: Detached Real-Time Web Telemetry Cockpit (`toksi dashboard start|stop|restart`).
* **Port `18787`**: Multi-Instance BYOK & Google Cloud ADC Gateway (`byok_gateway.rs` — supervised by daemon, routing all enabled requests through `HeadroomEffortCompressor` `Low`/`Medium`/`High`).

---

## 2. Core Disciplines, Security Governance & Workspace Isolation

1. **Safe Rust (Edition 2024)**:
   - Strict `#![deny(unsafe_code)]` at `crates/toksi/src/{lib,main}.rs`; platform FFI and C-ABI grammar shims live isolated in `crates/toksi-sys` and `crates/toksi-grammars`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`; must compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
2. **Declarative Configuration (`~/.toksi/config.toml`) & SQLite Store (`~/.toksi/toksi.db`)**:
   - `~/.toksi/config.toml` configures custom `[ports]`, `[auth.adc]` (`AGY_ADC_AUTH=true`), `[effort]`, `[security]` (`defenseclaw_fail_closed = true`, `skill_scanner_policy = "balanced"`), `[plugins]` (`plugin_dirs`, `enforce_security_scan = true`, `fail_closed_hooks = true`, `protocols`, `allow_agent_to_agent = true`), `[checkpoint]`, `default_agents`, custom API `base_url` endpoints, `direct_wire` defaults, and `[[byok_profiles]]`.
   - `~/.toksi/toksi.db` stores `byok_profiles` (supporting **multiple `<byok, model, thinking_effort>` instances per provider** plus `auth_mode`, `gcp_project`, `gcp_location`), `agent_profile_bindings`, `quota_windows`, `plugins_registry` (capabilities, protocols, `blake3_hash`, `security_scan_status`), `provider_keys`, and `provider_models`.
   - **`toksi db export|import`** transfers all tables cleanly (honoring `--overwrite`).
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
4. **Toksi Unified Plugin System (`src/plugins/`), `<enforce>` Security, `defenseclaw`, `skill-scanner`, `agentskills` YAML Healer & 41-Language Static Analyzer (`src/security/`, `src/rules/`)**:
   - **Toksi Unified Plugin System (`src/plugins/`, `toksi plugin`, `/plugin`)**: Loads **7 mount surfaces** (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook` Categories 1–3, `protocol_endpoint` across `MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP` on `:9098`, and `agent_to_agent` messaging) from `plugin.toml`, `plugin.yaml`, or `openclaw.plugin.json`. Every plugin passes mandatory `<enforce>` verification (`skill-scanner` + `defenseclaw`) before activation and persists in `plugins_registry`.
   - **`<enforce>` + `defenseclaw` Runtime Governance**: Wraps critical rules, plugins, and hook templates inside `<enforce>...</enforce>`; enforces fail-closed JSON-RPC/tool-call hookguard, tool-chain rules (`tool-chains.json`), HMAC gateway auth, `codesig` filesystem code-signature verification, YARA/MCP detection, and append-only JSONL audit evidence (`toksi security guardrails|audit-evidence`).
   - **`skill-scanner` Multi-Engine & Bounded CEL Analyzer (`toksi security scan-skills`)**: Combines static YARA/AST/entropy/link resolution, pipeline taint tracking, `.pyc`/bytecode mismatch auditing, and bounded non-Turing CEL policy decisions across 5 presets (`strict`, `balanced`, `low-noise`, `quiet`, `permissive`).
   - **`agentskills` Frontmatter Validator & Auto-Healer (`toksi rules heal-skills`)**: Validates all `SKILL.md` files against `agentskills/skills-ref` (`name` NFKC lowercase slug `<= 64` matching dir name, `description <= 1024`, `compatibility <= 500`, allowed keys only), auto-heals missing `---` fences / unquoted colons / misplaced keys into `metadata:`, and deduplicates colliding skill triggers across skill repos.
   - **41-Language AST Static Code Security Analyzer & OSV Lockfile Scanner (`toksi security scan`)**: Pure-Rust AST SAST patterns ported from `semgrep`, `semgrep-rules`, `codex-security`, `deepsec`, and `sage` + lockfile CVE parsing from `osv-scanner` (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `poetry.lock`, `requirements.txt`, `go.sum`, `pom.xml`, `Gemfile.lock`), emitting JSON, SARIF 2.1.0, Markdown, Table, or HTML reports with optional `PATH`-checked native linters (`rust-analyzer`, `gopls`, `clang-tidy`).
5. **Universal Forward-Slash (`/`) Path Standard & `DirectWireBus`**:
   - Forward slashes `/` across Windows, Linux, and macOS (`\\` escaped exclusively in Windows `cmd.exe` hook strings).
   - Hook syntax `<tool>-hook <agent>`; normalized MCP identifiers `projectmem`, `context`, `codegraph`, `icm`, `aimemory`, `rtk`, `security`; zero-config-edit `DirectWireBus`.

---

## 3. Open BYOK, Google Cloud ADC (`AGY_ADC_AUTH=true`), & Thinking-Effort Optimization

* **Zero Paid-Tier Restrictions**: Zero `FreeTierDisqualified` or `[PAID]` paywalls exist anywhere in Toksi. Any user-configured BYOK, Google Cloud ADC, OAuth, or local CLI profile routes freely across all 65/90+ ecosystem providers and 22 local engines (`Ollama`, `vLLM`, `llama.cpp`, `LM Studio`, `SGLang`, `Exo`, etc.).
* **First-Class Google Cloud ADC Gateway (`AGY_ADC_AUTH=true`)**:
  - Reads `AGY_ADC_AUTH=true`, VS Code `"gcp.project"` / `"gcp.location"` (`us-central1` default), `~/.toksi/config.toml` `[auth.adc]`, and `~/.config/gcloud/application_default_credentials.json` (`%APPDATA%/gcloud/application_default_credentials.json` on Windows).
  - Performs native pure-Rust OAuth2 refresh-token / service-account JWT exchange against `oauth2.googleapis.com/token` and signs requests to `https://{location}-aiplatform.googleapis.com/v1beta1/projects/{project}/locations/{location}/publishers/google/models/{model}`.
  - Emits exact remediation (`gcloud auth application-default login --project <GCP_PROJECT>`) whenever credentials or project IDs are missing.
* **Thinking-Effort-Aware Token Optimization (`Low` / `Medium` / `High`)**:
  - Single `/effort` interface (`--effort`, header `x-toksi-effort`, or gateway JSON body) maps seamlessly across OpenAI (`reasoning_effort`), Anthropic (`thinking.budget_tokens`), and Gemini/Vertex (`generationConfig.thinkingConfig.thinkingBudget`) — silently omitting thinking fields on unsupported models.
  - Dynamically scales shared `HeadroomEffortCompressor` across ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098` Protocol Bus, `:9099` Enterprise AI Gateway, `:18787` BYOK & Vertex ADC Gateway): `Low` (aggressive compaction + top-K signatures), `Medium` (balanced AST/semantic compression), `High` (lossless reasoning, deep CodeGraph SCIP + blast radius).

---

## 4. Native Coding CLI (`toksi chat|run|exec|swarm`), Unified Plugins, Safe Resume & 13 Slash Commands

Launch Toksi directly as your standalone AI coding CLI or multi-agent swarm orchestrator — wired out of the box ("off the back") via `DirectWireBus` and local endpoints (`:8787`, `:9098`, `:9099`, `:18787`) to Enterprise AI Gateway, BYOK & Vertex ADC Gateway, Protocol Bus (`MCP`/`A2A`/`ACP`/`SSE`), `HeadroomEffortCompressor`, Unified Plugin System (`src/plugins/`), and all 67 consolidated vault engines (`rtk`, `headroom`, `context-mode`, `codegraph`, `projectmem`, `ai-memory`, `icm`, `ponytail`, `caveman`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `lance`, `agent-client-protocol`, `agent-opt`, `agent-control`, `sage`, `defenseclaw`, `skill-scanner`, plus `hermes-agent`, `openclaw`, `mofa`, `crewAI`, `codex`, `goose`, `grok-build`, `gemini-cli`, `langgraph`, `deepagents`, `bosun`, `cline`, `kilocode`, `alphacode`, `zeroclaw`):

```bash
# Interactive coding session with 3-OS sandbox, <enforce> guardrails & plan checkpoints
toksi chat --effort medium

# Single-shot autonomous task or headless CI security / plugin scan
toksi run "Refactor gateway failover tests and verify clippy"
toksi security scan --format sarif
toksi plugin list

# Multi-agent DAG swarm & crash/429-safe checkpoint resume
toksi swarm "Audit all 45 language grammars and heal SKILL.md frontmatter"
toksi agent resume [<plan_id>]
```

**13 Instant In-Chat Slash Commands** (`< 5ms` non-blocking execution in `toksi chat` and hooked agents):
- `/byok [filter]` -> select BYOK provider or Google ADC profile
- `/model [filter]` -> select model discovered via zero-cost `/v1/models`
- `/effort [low|medium|high]` -> set thinking budget & scale Headroom compression
- `/byokmodel <byok> <model> [effort]` -> atomic profile binding update
- `/projectmem ...`, `/context ...`, `/codegraph ...`, `/icm ...`, `/ai-memory ...`, `/rtk ...`
- `/security [scan [path]|scan-skills [path]|guardrails|audit-evidence|rules|audit-skills]` -> run 41-lang SAST/OSV, `skill-scanner`, `defenseclaw` guardrails, & `agentskills` YAML healer
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
  5. **CodeGraph Explorer**: Live `symbols.db` stats across 45 language IDs, SCIP/Repomix/AST-grep, `skill-scanner` CEL policy/taint + built-in 41-language static security findings.
  6. **Sandbox Inspector**: Live OS sandbox (`bwrap`/`Landlock`/`Seatbelt`/`WindowsRestrictedToken`), `<enforce>` & `defenseclaw` fail-closed hookguard/tool-chains/audit evidence & `sage` threat blocks, FTS5 search bar.
  7. **System Controls**: `~/.toksi/config.toml` editor, interactive Unified Plugin Manager (`/api/plugins`), `AGY_ADC_AUTH=true` & `/effort` toggles, `agentskills` healer diff preview, granular component installer.
  8. **AI Gateway & Quotas**: Multi-instance `<byok, model, effort>` & Google ADC CRUD, custom `base_url`, zero-cost `/v1/models` refresh, MoFA 7-D router.
  9. **Keyring & Database**: Configured-only OS Keyring (`gemini:antigravity` + ADC), SQLite table explorer (`byok_profiles` + `plugins_registry`), DB `Export` / `Import` / `Backup` / `Restore`.

---

## 6. Supported Agent Layouts & CLI Quick Reference

* **Agent Conventions**: `.agents/` (`agents-structures/` + `agentskills` + `.agents/mcp -> skills/<tool>` + `.agents/plugins/`), `.gemini/` (`settings.json`, `commands/*.toml`), `.copilot/` & `.github/`, `.claude/` (`.mcp.json`), `.codex/`, `.cursor/`, `.grok/` (`hooks/*.json`).
* **Quick Commands**:
  ```bash
  toksi config show                                 # Inspect ~/.toksi/config.toml, [plugins] & AGY_ADC_AUTH
  toksi daemon start --agents claude,agy            # Supervise 8-port daemon (:8787..:18787)
  toksi dashboard start|stop|restart                # Detached Web Cockpit on :9100
  toksi byok add|list|update|delete ...             # Multi-instance <byok, model, effort> & ADC CRUD
  toksi plugin list|install|enable|disable|scan|run # Manage Unified Plugins under <enforce>
  toksi security scan|scan-skills|guardrails|audit-evidence --format sarif # defenseclaw + skill-scanner + 41-lang SAST
  toksi rules sync | toksi rules heal-skills        # Sync <enforce> rules & heal SKILL.md frontmatter
  toksi agent resume [<plan_id>]                    # Resume paused .toksi/projectmem/plan_checkpoints/
  toksi doctor [--fix]                              # Audit PATH, direct-wire, legacy .projectmem/ merge
  toksi tui | top                                   # 9-tab async mouse TUI
  ```

---

## 7. In-Tree Rules & Build / Incremental Verification

Embedded at compile time via `include_str!` with `<enforce>` safety delimiters: **Principles** (Karpathy), **Caveman v4** (`/ultracave`, `/megacave`, `shrink`, `browse`), and **Ponytail** (modes & statusline).

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
