# Toksi: Native Consolidation Master Engine (Wave 17 / v4.0)

[![Crates.io](https://img.shields.io/crates/v/toksi.svg)](https://crates.io/crates/toksi)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**Toksi** is an enterprise-grade, **100% Native Safe Rust (`#![deny(unsafe_code)]`)** consolidation of token optimization, intelligent proxying, multi-protocol agent wire integration, code intelligence, autonomous coding runtime (`toksi chat|run|exec|swarm`), and 9-tab telemetry cockpits for AI coding assistants.

Toksi sits transparently between **18+ registered AI coding assistants** (expanded across **65/90+ Categories 1–5 ecosystem providers** and **Open Agents `.agents/`**) and **22 local inference engines** plus multi-instance `<byok, model, effort>` upstream providers—cutting token spend by **30% to 70%** via pure-Rust `headroom-core` `live_zone` cache-preserving compression, `CcrStore`, `TokenEconomizer`, `RTK` 100+ command filters, `Context-Mode` sandboxes, **41+ Tree-sitter languages**, SCIP/AST-grep/Repomix/jcodemunch code intelligence, and repo-scoped zero-LLM memory—with zero Node.js, Python, npm, or pip dependencies.

---

## 1. Master Architecture & Unified 7-Port Network Topology

All listening ports are overridable via `~/.toksi/config.toml` (`[ports]`) and isolated so browser dashboards never introduce TCP backpressure on inference streams:

```text
[ AI Coding Assistants / IDEs / Local CLIs / Browsers ]
  │        │        │        │        │          │         │
  │ :8787  │ :8788  │ :8789  │ :8790  │ :18787   │ :9099   │ :9100
  ▼        ▼        ▼        ▼        ▼          ▼         ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                 toksi daemon start|stop|restart [--agents ...]               │
│                                                                              │
│ ┌──────────────────────────┐        ┌──────────────────────────────────────┐ │
│ │ :8787 Headroom Proxy     │        │ :9099 Enterprise AI Gateway & SDKs   │ │
│ │ - HTTP/1.1 & HTTP/2 SSE  │        │ - /v1/{chat/completions,messages,    │ │
│ │ - ConnectRPC Passthrough │        │   responses,embeddings}, /v1beta/*   │ │
│ │ - live_zone Cache Surgery│        │ - Prometheus /metrics                │ │
│ │ - MCP Pruner & CcrStore  │        │ - 65/90+ Ecosystem & 22 Local Engines│ │
│ │ - Mandatory /byok/ Pass  │        │ - Multi-Instance <byok,model,effort> │ │
│ │ - 10 In-Chat /... Cmds   │        │ - MoFA 7-D Router & Failover Cascade │ │
│ └──────────────────────────┘        │ - Chroma ANN + foyer Semantic Cache  │ │
│                                     └──────────────────────────────────────┘ │
│ ┌──────────────────────────┐                                                 │
│ │ :8788 Grok OAuth Daemon  │        ┌──────────────────────────────────────┐ │
│ │ - xAI Grok OAuth bridge  │        │ :18787 BYOK Multi-Profile Gateway    │ │
│ └──────────────────────────┘        │ - Per-profile custom base_url/quota  │ │
│ ┌──────────────────────────┐        └──────────────────────────────────────┘ │
│ │ :8789 Copilot VS Code    │        ┌──────────────────────────────────────┐ │
│ │ :8790 Copilot CLI        │        │ :9100 Detached Web Telemetry Cockpit │ │
│ │ - Real OAuth/SSE Proxies │        │ - 9-Tab Futuristic UI + Live SSE     │ │
│ └──────────────────────────┘        └──────────────────────────────────────┘ │
│                                                                              │
│ [ In-Process DirectWireBus & Multi-Protocol Engine ]                         │
│  • RTK (100+ Filters + Tracker)  • ProjectMem (JSONL + Clickable Heatmap)    │
│  • Context-Mode (Sandbox + FTS5) • ICM (BM25 + Memoir/Feedback/Temporal)     │
│  • CodeGraph (41+ TS/SCIP/Grep)  • ai-memory (OKF v0.2 Wiki + Handoffs)      │
│  • Agent Runtime (Hermes/OpenClaw/MoFA/CrewAI/Codex/Grok-Build)              │
│  • Rules (Karpathy, Caveman v4, Ponytail compiled via include_str!)          │
│  • Protocols: MCP, A2A v1.0, ACP v2.0, stdio, SSE, HTTP                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Dedicated Ports:
* **Port `8787`**: Transparent Streaming Headroom Proxy (Claude, Codex, Antigravity, Cursor, and mandatory `/byok/` compression pass).
* **Port `8788`**: GitHub/xAI Grok OAuth Proxy Daemon (`grok_daemon.rs` — strictly separate from BYOK!).
* **Port `8789`**: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`).
* **Port `8790`**: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`).
* **Port `18787`**: Multi-Instance BYOK Gateway (`byok_gateway.rs` — supervised by daemon).
* **Port `9099`**: Enterprise AI Gateway & Drop-in SDK Endpoints (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/embeddings`, `/v1beta/models/*`, `/metrics`).
* **Port `9100`**: Detached Real-Time Web Telemetry Cockpit (`toksi dashboard start|stop|restart`).

---

## 2. Core Disciplines, Configuration & Workspace Isolation

1. **Safe Rust (Edition 2024)**:
   - Strict `#![deny(unsafe_code)]` at `crates/toksi/src/{lib,main}.rs` (zero `set_var`, `LockFileEx`, or `setsid` `unsafe` inside `crates/toksi`).
   - Platform FFI (`advapi32`/`user32`/Job Objects/console) and legacy C-ABI grammar shims live isolated in leaf crates `crates/toksi-sys` and `crates/toksi-grammars`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`; must compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
2. **Declarative Configuration (`~/.toksi/config.toml`) & SQLite Store (`~/.toksi/toksi.db`)**:
   - `~/.toksi/config.toml` configures custom `[ports]`, `default_agents`, custom API `base_url` endpoints, `direct_wire` defaults, incremental test settings, and `[[byok.profiles]]`.
   - `~/.toksi/toksi.db` stores `byok_profiles` (supporting **multiple `<byok, model, thinking_effort>` instances per provider**), `agent_profile_bindings`, `quota_windows`, `provider_keys`, and `provider_models` (with automatic startup migration from legacy `LOCALAPPDATA/toksi`).
   - **`toksi db export|import`** transfers all tables cleanly including `provider_models` and `byok_profiles` (honoring `--overwrite`).
3. **Strict `<repo>/.toksi/<tool>` Repo-Only Storage & Monorepo Partitioning**:
   - Workspace intelligence tools NEVER create project indexes in `~/.toksi/<tool>/` or outside a `.git` repository root, and NEVER index `<repo>/.git/`.
   - All workspace tool artifacts live strictly inside root **`<repo>/.toksi/<tool>/`**:
     * `<repo>/.toksi/codegraph/` (41+ language Tree-sitter `symbols.db`, SCIP, AST-grep, Repomix)
     * `<repo>/.toksi/projectmem/` (Atomic JSONL `events.jsonl`, `summary.md`, `heatmap`)
     * `<repo>/.toksi/context/` (Context-Mode FTS5 DB, `policy`/`journal`/`compaction`)
     * `<repo>/.toksi/icm/` (Instant Context Memory SQLite FTS5 `memory.db`)
     * `<repo>/.toksi/ai-memory/` (OKF v0.2 wiki store and claim-once handoffs)
     * `<repo>/.toksi/headroom/` & `<repo>/.toksi/rtk/`
   - Monorepos (`<repo>/crates/project1..N/`) are automatically resolved to the `.git` root and partitioned by `subproject_id`; nested crate `.toksi/` or `.<agent>/` dirs are merged into root and deleted.
4. **Universal Forward-Slash (`/`) Path Standard & PATH Check**:
   - Forward slashes `/` across Windows, Linux, and macOS (`C:/Users/name/.toksi/toksi.db`); `\\` escaped exclusively in Windows `cmd.exe` hook strings.
   - `check_executable_in_path` verifies agent binaries on `PATH` without hardcoding paths or blocking startup (missing binaries are greyed out cleanly in `AI Agent Fleet & Quotas`).
5. **Tool Hook, MCP, Skill, & `DirectWireBus` Conventions**:
   - Hook syntax: `<tool>-hook <agent>` (`codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`, `icm-hook agy`, `ai-memory-hook agy`).
   - Normalized MCP identifiers: `projectmem`, `context`, `codegraph`, `icm`, `aimemory`, `rtk`; skill dirs mapped via `.{gemini,copilot,claude,agents}/mcp` -> `skills/<tool>`.
   - CLI vs IDE split: `antigravity-cli` vs `antigravity-ide`, `copilot-cli` vs `copilot-vscode`, `claude-cli` vs `claude-desktop`, `cursor-cli` vs `cursor-ide`, `grok-cli` vs `grok-ide`, plus `"openagents"`.
   - **Direct-Wired Mode (`DirectWireBus`)**: Zero-config-file in-process hook/protocol dispatch (`MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`), selectable alongside `config-wired` JSON/TOML installation.
6. **Dynamic `.gitignore`/`.ignore` Purge & Network Auto-Heal**:
   - File watcher automatically purges newly ignored files from `symbols.db`, FTS5 indexes, and Chroma ANN vector caches.
   - `ArcSwap<reqwest::Client>` reconnect watchdog resets stale pools cleanly across VPN toggles and sleep/wake cycles; HTTP/2 ConnectRPC passthrough preserves binary gRPC/Connect frames.

---

## 3. Open Multi-Instance BYOK, Ecosystem Adapters & Local Engines

* **Zero Paid-Tier Restrictions**: No `FreeTierDisqualified` or `[PAID]` paywalls exist anywhere in Toksi. Any user-configured agent or BYOK profile (free, trial, OAuth, subscription, or commercial) routes freely.
* **Configured-Only Visibility & Activation**: Toksi only launches, displays in `Savings`/`AI Gateway`/`Keyring`, and routes through agents and `<byok, model, effort>` profiles configured and enabled by the user.
* **Multi-Instance `<byok, model, effort>`**: Add the same provider multiple times with distinct models, custom `base_url`s, thinking efforts (`none|low|medium|high|max|xhigh`), and separate `hour|day|week|month|year` rate-limit `quota_windows`.
* **Full 65/90+ Ecosystem & 22 Local Inference Engines**:
  - Frontier & Cloud, Accelerators, Aggregators (`Bifrost`, `OpenRouter`, `Portkey`, `LiteLLM Proxy`, `Cloudflare AI Gateway`, `Helicone`, etc.), Agentic CLIs/IDEs/LSPs/Platforms, and Local Engines (`Ollama`, `vLLM`, `llama.cpp`, `LM Studio`, `SGLang`, `TGI`, `Exo`, `MLC`, etc.) powered by deterministic `mofa-engine` 7-D routing and `Chroma` ANN + `foyer` semantic caching.
* **Mandatory Headroom Optimization**: Every configured BYOK payload flows through Headroom `live_zone` cache-preserving compression, `MCP` schema pruning, and `TokenEconomizer`.

---

## 4. Native Coding CLI (`toksi chat|run|exec|swarm`) & 10 In-Chat Slash Commands

Launch Toksi directly as your standalone AI coding CLI or multi-agent swarm orchestrator (fused from `hermes-agent`, `openclaw`, `openclaw-rs`, `mofa`, `mofaclaw`, `crewAI`, `codex`, `crustly`, `crush`, `goose`, `grok-build`, `gemini-cli`):

```bash
# Interactive coding session with 3-OS sandbox, apply_patch, hashline edits, & tool loop detector
toksi chat

# Single-shot autonomous task or headless CI run
toksi run "Refactor gateway failover tests and verify clippy"
toksi exec "Summarize repo architecture via codegraph"

# Multi-agent DAG swarm with ReflectionAgent & MoA aggregation
toksi swarm "Audit all 41 tree-sitter extractors and fix edge cases"
```

**10 Instant In-Chat Slash Commands** (`< 5ms` non-blocking execution in both native `toksi chat` AND external agents hooked through Headroom `:8787`, updating `toksi.db` mid-conversation without hangs or loops):
- `/byok [filter]` -> select BYOK provider first
- `/model [filter]` -> select model discovered via zero-cost `/v1/models`
- `/effort [none|low|medium|high|max|xhigh]` -> select thinking effort (ignored gracefully if model unsupported)
- `/byokmodel <byok> <model> [effort]` -> one-shot binding update
- `/projectmem ...`, `/context ...`, `/codegraph ...`, `/icm ...`, `/ai-memory ...`, `/rtk ...`

---

## 5. Ground-Truth Telemetry & Unified 9-Tab UI (TUI + Web Cockpit)

* **Zero Synthetic Metrics**: Idle sessions read strictly `0` (no hardcoded samples, no `0.38/0.42/0.20`, no `1428 KB` vacuum stubs, no fake 429s). Tokens & quotas parse directly from live SSE/Responses streams and HTTP rate-limit headers.
* **Exact 9-Tab Parity (`toksi tui`/`top` and `http://127.0.0.1:9100`)**:
  1. **Savings**: Configured-only agent & `<byok, model, effort>` token/USD savings + per-tool breakdown.
  2. **The War Room**: Live active tools, 429 backoff/cascade telemetry, `ArcSwap` reconnect heal status.
  3. **AI Agent Fleet & Quotas**: All supported agents; greys out binaries missing from `PATH` with remediation hints; one-click `Direct-Wire` / `Config-Wire` / `Unwire`; read-only plan & `hour..year` quota gauges.
  4. **Memory Decision DAG**: Interactive `projectmem` DAG, monorepo subproject selector, and clickable `.heatmap-cell` diagnostic modal.
  5. **CodeGraph Explorer**: Live `symbols.db` stats across all `41+` languages, `ast-grep`, `SCIP`, `Repomix`, and `.ignore` purge.
  6. **Sandbox Inspector**: Live OS sandbox (`bwrap`/`Landlock`/`Seatbelt`/`WindowsRestrictedToken`), `< 2KB` projected stdout, FTS5 search test bar.
  7. **System Controls**: Live `~/.toksi/config.toml` editor, `daemon|proxy|dashboard` controls, granular per-component installer (`hooks|mcp|skills|rules|instructions|configs`) with user-auto vs explicit repo opt-in.
  8. **AI Gateway & Quotas**: Multi-instance `<byok, model, effort>` CRUD, custom `base_url`, zero-cost `/v1/models` refresh, per-agent Dynamic Complexity & Load Balancer routing.
  9. **Keyring & Database**: Configured-only OS Keyring (`gemini:antigravity` fixed), SQLite table explorer, and one-click DB `Export` / `Import` / `Backup` / `Restore`.

---

## 6. Supported Agent Layouts & CLI Quick Reference

* **Agent Conventions**: `.agents/` (`agents-structures/` parity + `.agents/mcp -> skills/<tool>`), `.gemini/` (`settings.json`, `commands/*.toml`), `.copilot/` & `.github/`, `.claude/` (`.mcp.json`), `.codex/`, `.cursor/`, `.grok/` (`hooks/*.json`). User-level hooks install automatically only for selected agents; workspace repo installs require explicit opt-in (`--workspace`).
* **Process & Config Control**:
  ```bash
  toksi config show                        # Inspect ~/.toksi/config.toml
  toksi daemon start --agents claude,agy   # Supervise configured ports & agents
  toksi daemon stop | restart --agents ...
  toksi proxy start|stop|restart --agents ...
  toksi dashboard start|stop|restart       # Detached on :9100 (--foreground to attach)
  toksi byok add|list|update|delete ...    # Multi-instance <byok, model, effort> CRUD
  ```
* **Lifecycle, Auth, DB & In-Process Tools**:
  ```bash
  toksi agent list|install|health|mismatch|upgrade|fix|restore|remove|edit
  toksi tui | top                          # 9-tab async mouse TUI
  toksi doctor [--fix]                     # Audit PATH, direct-wire, rules, root .toksi merge
  toksi auth status|set|sync               # Keyring (configured-only, gemini:antigravity fixed)
  toksi db status|export|import [--overwrite]
  toksi mem ... | toksi graph ... | toksi ctx ... | toksi rtk ... | toksi rules sync|status
  ```

---

## 7. In-Tree Rules & Build / Incremental Verification

Embedded at compile time via `include_str!`: **Principles** (Karpathy), **Caveman v4** (`/ultracave`, `/megacave`, `shrink`, `browse`), and **Ponytail** (modes & statusline).

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
