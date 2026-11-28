# Toksi (`toksi`)

**Pure-Rust 35-Component Super-Intelligence Runtime (`<toksi-siagent>`), Streaming Token Optimization Proxy, 45-Language Tree-Sitter AST Engine, Hybrid Memory Fabric, & `<enforce>` Security Gateway**

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](../../LICENSE)
[![Rust Edition: 2024](https://img.shields.io/badge/Rust-2024%20%7C%20deny(unsafe__code)-orange.svg)](src/lib.rs)

Toksi (`crates/toksi`) is the unified Rust runtime and multi-agent control plane inside `<neo>`. It eliminates token waste across CLI, IDE, and gateway workflows through lossless structural compression, repository-scoped AST indexing, SQLite FTS5 & hybrid graph-vector memory, blind-signature security scanning, and single-owner 8-port supervision.

---

## 1. Core Architectural Pillars

- **Rust Edition 2024 & Memory Safety**: Core `crates/toksi` compiles cleanly under `#![deny(unsafe_code)]` with zero production `.unwrap()`, `.expect()`, or `panic!()` calls. Low-level FFI/tree-sitter C parsers are strictly isolated in leaf crates (`crates/toksi-sys`, `crates/toksi-grammars`).
- **3-Repo Clean Boundary**:
  1. `<neo>` (`d:/AI/neo`) — Core runtime engine (`crates/toksi`), build tooling (`crates/nbuild`, `crates/archinfo`), and Docker sandboxes (`docker/`).
  2. `<advisory>` (`d:/AI/advisory`) — 12-DB vulnerability/threat intelligence & Native Git/PR validator target (`toksi advisory` / `toksi compile-blind`).
  3. `<skills>` (`d:/AI/skills` / `~/.toksi/skills`) — Externalized zero-bloat skills catalog & `<enforce>` rules synced cleanly outside the binary.
- **Zero Binary Bloat**: Skill packs and guidelines live externally outside `crates/toksi/` and are validated via `skilldoctor`, `agentskills`, `upskill`, and `skill-scanner`/`defenseclaw`.
- **Single-Owner `NetworkManager`, `DirectWireBus` & `toksi _worker` IPC**: `toksi daemon start` spawns the single-owner TCP `NetworkManager` across active service ports (`:8787`..`:18787`, keeping web cockpit `:9100` detached unless explicitly requested) and dispatches live HTTP traffic over cross-platform Named Pipes / Unix Domain Sockets (`IpcOpcode::NetworkForward`) to `toksi _worker <role>` processes, while `DirectWireBus` provides zero-hop in-process routing across tools and `HeadroomEffortCompressor` scales token compression by effort tier (`Low`/`Medium`/`High`/`XHigh`/`Max`).
- **Key Security & Runtime Modules**: Unified Plugin host (`src/plugins/`), Native Git/PR Validator (`src/validator/`), Agent Detection & Response (`src/security/adr.rs`), 5-backend Sandbox & `compose.test.yaml` verifier (`src/agent_runtime/sandbox.rs`), and pre-exec Destructive Command Guard (`src/commands/guard.rs`).
- **Repository-Scoped Memory & Indexes**: Every repository tool persists strictly inside `<repo>/.toksi/<tool>/`:
  - `rtk` — 100+ pure-Rust command filters, zero-allocation ANSI stripper, TEE raw failure recovery, SQLite savings tracker
  - `projectmem` — Append-only JSONL ledger (`events.jsonl`), derived `summary.md`, crash-resilient `plan_checkpoints/`, `orchestration_journal.jsonl`
  - `ctxmode` — Isolated execution sandbox & SQLite FTS5 BM25 search (`ctxmode-hook <agent>` / `ctxmode mcp`)
  - `codegraph` — 45-language Tree-sitter AST indexer, SCIP, blast-radius analysis, OpenWiki (`codegraph-hook` / `toksi graph`)
  - `icm` — Hierarchical Instant Context Memory (`icm-hook <agent>` / `toksi icm`)
  - `aimemory` — Hybrid BM25 + graph-vector memory (`aimemory-hook <agent>` / `toksi aimemory` / `toksi handoff`)
  - `security` — OPA/Cedar IAM + SPIFFE/SPIRE attestation, 41-language SAST, OSV/EPSS scanner, zero-plaintext blind signatures (`BLAKE3` + `sparse::DFA`), ADR audit log

---

## 2. Dedicated 8-Port Network Topology

| Port | Role | Service / Responsibility |
| :--- | :--- | :--- |
| `8787` | `Port8787` | Streaming **Headroom Proxy** (`live_zone` byte surgery, 1024-token cache alignment, CCR store, effort scaling) |
| `8788` | `Port8788` | **Grok OAuth Proxy** (dedicated xAI failover & response normalization) |
| `8789` | `Port8789` | **GitHub Copilot VS Code Proxy** (`DEFAULT_COPILOT_PROXY_PORT`) |
| `8790` | `Port8790` | **GitHub Copilot CLI Proxy** (`DEFAULT_COPILOT_CLI_PROXY_PORT`) |
| `9098` | `Port9098` | **Unified Protocol Bus** (`MCP`, `A2A v1.0`, `ACP v2.0`, `ANP`, `QUIC`, `stdio`, `SSE`, `HTTP` + 13-IDE bridge) |
| `9099` | `Port9099` | **Enterprise AI Gateway** (`/v1/chat/completions`, `/v1/responses`, `/v1/messages`, `/v1/embeddings`, `/v1/models`, `/v1/security/scan`, `/v1beta/models*`, `/metrics`) |
| `9100` | `Port9100` | **Web Telemetry & Control Cockpit** (10-tab SPA + live SSE — managed separately via `toksi dashboard`) |
| `18787` | `Port18787` | **BYOK & Google Vertex ADC Gateway** (`AGY_ADC_AUTH=true`, multi-instance `<byok, model, effort>` cascade) |

All routes support universal forward-slash paths (`/`), ground-truth SSE token accounting (`tokentracker`), Prometheus `/metrics`, OpenTelemetry (`gen_ai.1.29` semconv on OTLP `:4317`/`:4318`), and automatic reconnect healing via `ArcSwap<reqwest::Client>`.

---

## 3. CLI Reference (`crates/toksi/src/main.rs`)

### Interactive Runtimes, UI & IDEs
- `toksi chat` / `run` / `exec` / `swarm` / `resume` — Autonomous `<toksi-siagent>` loop, Codex/Grok chat TUI with collapsible tool cards, diff previews, HITL modals, 5-backend sandbox execution, and crash-resilient cross-chat resume
- `toksi tui` — Full mouse-capable **10-tab** terminal cockpit (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`)
- `toksi dashboard [start|stop|restart|status] [--port 9100] [--open]` — Detached 10-tab web UI & SSE telemetry cockpit
- `toksi top [--once]` — Live interactive terminal request & savings stream
- `toksi ide-bridge [--list | --ide <id> --transport <http|stdio>]` — External 13-IDE plugin host protocol (`vscode`, `visual-studio`, `jetbrains`, `android-studio`, `zed`, `xcode`, `eclipse`, `cursor`, `windsurf`, `apache-netbeans`, `kiro`, `qoder`, `lapce`)

### Daemon, Proxies, Config & Security
- `toksi daemon [start|stop|restart|status] [--agents <list>]` — Single-owner `NetworkManager` + IPC worker fleet supervisor
- `toksi proxy [start|stop|restart|status]` — Manage Headroom / Copilot / Grok streaming proxies
- `toksi config` — Manage `~/.toksi/config.toml` (see [`full_example_config.toml`](full_example_config.toml))
- `toksi byok` — Configure `<byok, model, effort, temperature, top_p, top_k>` profiles & inspect read-only `quota_windows` (zero paid-tier restrictions)
- `toksi auth [status|get|set|delete|sync]` — Native OS Keyring (`zeroize`) & Google Cloud ADC (`AGY_ADC_AUTH=true`)
- `toksi db [status|export|import|backup|restore]` — Manage `~/.toksi/toksi.db`
- `toksi security` / `compile-blind` / `advisory` — OPA/Cedar, SPIFFE/SPIRE, ADR, 41-lang SAST, blind signatures, 12-DB advisory checks
- `toksi skills` / `rules` / `plugin` / `sandbox` — Externalized skills validator, `<enforce>` guidelines sync, plugin host, and 5-backend sandbox (`native`, `sandbox-runtime`, `OpenShell`, `E2B`, `sbx` CLI)

### Agent Lifecycle, Tools & 11 AI/ML Pillars
- `toksi [init]` / `update` / `doctor [--fix] [--workspace]` / `agent` / `disable` / `uninstall` / `self-update`
- `toksi rtk` / `projectmem` (`mem`) / `ctxmode` (`ctx`) / `graph` (`codegraph`) / `icm` / `aimemory` (`handoff`) / `index` / `prime` / `gain` / `info`
- `toksi dataset` / `toksi train` / `toksi eval` / `toksi compress` / `toksi serve` / `toksi cluster` / `toksi otel` / `toksi orchestrator` — Declarative recipes (`unsloth`, `axolotl`, `LLaMA-Factory`, `torchtune`, `trl`, `Megatron-Bridge`, `nemo-rl`, `OpenRLHF`), fail-closed `.safetensors` verification, 22 eval harnesses (`lm-evaluation-harness`, `deepeval`, `ragas`, `guidellm`, `genai-bench`, `autofa`), KvPress/BnB plans, 37+ local engines, NVCRE cluster readiness & OpenTelemetry collector YAML
- `toksi heal-encoding` / `toksi refactor-router` *(Wave 22 Roadmap)* — Pure 7-bit ASCII validator & `MojibakeLab`/`python-ftfy` mojibake healer across source and docs, plus non-destructive prompt-architecture classification & router-index extraction; paired with `r3bl-open-core` (`r3bl_tui`) Chat/TUI migration, Headroom-central `:8787` `<toksi-siagent>` routing, `.git/config` dynamic repo detection, and externalized `<neo>/agents/rules/` + `<neo>/agents/agent_instructions.md` staging
- `toksi test [--all | --force | --sandbox | --docker]` — Incremental BLAKE3 `watched_paths` + `Cargo.lock` test runner, sandbox, and `docker/Dockerfile.test` + `docker/compose.test.yaml` isolation runner

---

## 4. 29 Universal In-Chat Slash Commands & 22 Orchestrator Modes

Every `<5ms` slash command works uniformly inside `toksi chat`, hooked external agent CLIs (`claude`, `codex`, `cursor`, `antigravity`, `grok`, `copilot`, `opencode`, `cline`, `droid`, `pi`, `omp`, `kilo`, `amp`, `kimi`, `zcode`, `dsh`), and via `toksi /<cmd>`:

```text
/byok        /model       /effort      /byokmodel   /mode        /sampling
/router      /projectmem  /ctxmode     /codegraph   /icm         /aimemory
/rtk         /security    /advisory    /skills      /sandbox     /plugin
/resume      /workforce   /spec        /dataset     /train       /eval
/compress    /serve       /cluster     /otel        /voice
```

The Super-Intelligence router (`[toksi_siagent_router.*]`) organizes **22 orchestrator modes across 7 cognitive domains**:

1. `1_meta_cognition_governance`: `orchestrate_route`, `reflect_self_critique`, `align_guardrail`, `evolve_self_improve`
2. `2_memory_world_modeling`: `recall_retrieve`, `consolidate_memorize`, `simulate_predict`
3. `3_discovery_epistemics`: `chat_interface`, `brainstorm_diverge`, `research_ground`, `synthesize_abstract`
4. `4_pedagogy_knowledge_transfer`: `study_deconstruct`, `coach_socratic`
5. `5_strategy_architecture`: `plan_decompose`, `toolsmith_architect`
6. `6_autonomous_execution`: `write_code_build`, `refactor_optimize`, `act_operate`
7. `7_verification_resilience`: `running_tests_verify`, `debug_heal`, `audit_red_team`, `document_log`

---

## 5. Configuration Reference (`full_example_config.toml`)

See [`crates/toksi/full_example_config.toml`](full_example_config.toml) for a complete, parse-verified reference covering all 26 `ToksiTomlConfig` sections (`[general]`, `[ports]`, `[base_urls]`, `[auth.gcp_adc]`, `[byok.*]`, `[[byok_profiles]]`, `[repositories]`, `[security]`, `[test]`, `[supervisor]`, `[headroom]`, `[gateway]`, `[plugins]`, `[checkpoint]`, `[sandbox]`, `[cluster]`, `[training]`, `[telemetry]`, `[dashboard]`, `[tui]`, `[keyring]`, `[tools]`, `[hooks]`, `[ide_bridge]`, and all 22 modes under `[toksi_siagent_router.*]`).

---

## 6. Verification, Live-Safe Testing & Docker Sandbox

All workspace tests isolate state via `TempDir` (`TOKSI_HOME`), `TOKSI_TEST=1`, unique `TOKSI_IPC_NS`, and `:0` ephemeral ports so running daemons or live `~/.toksi/toksi.db` are never touched.

```bash
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Live-safe incremental runner (.toksi/test-baseline.json)
cargo run -p toksi -- test --all --force
cargo run -p toksi -- test --sandbox

# Isolated Docker verification (docker/Dockerfile.test + docker/compose.test.yaml)
docker compose -f docker/compose.test.yaml run --rm toksi-test
```

---

## 7. Architecture Plans & Research

All **60 remediation phase specs** (`PHASE_01_RTK_FILTER_ENGINE.md` through `PHASE_22_6_WAVE22_VERIFICATION_12STATE_AUDIT_AND_61FILE_SYNC.md`, tracked under the **12-State Progress Taxonomy** across **61 guides**) spanning the **493** Wave 20 baseline vaults, Wave 21 additions (**495** vaults), and Wave 22 additions (**502** mandated vaults total, incorporating `ratatui`, `ratatui-image`, `tui-realm`, `iocraft`, `r3bl-open-core`, `MojibakeLab`, and `python-ftfy`) are indexed in [`.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`](../../.agents/brain/INDEX_AND_EXECUTION_GUIDE.md) and tracked in [`PLAN.md`](PLAN.md), [`CONSOLIDATION_PLAN.md`](CONSOLIDATION_PLAN.md), [`research/RESEARCHPLAN.md`](research/RESEARCHPLAN.md), and [`research/WAVE22_MASTER_EXECUTION_PROMPT.md`](research/WAVE22_MASTER_EXECUTION_PROMPT.md).

---

## 8. License

Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.
