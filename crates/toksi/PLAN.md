# toksi — System Architecture, Disciplinary Standards & Implementation Plan (Wave 19 / v6.0)

High-performance headroom proxy, token optimizer, native AI gateway (`AGY_ADC_AUTH=true` Google Cloud ADC + thinking-effort-aware routing + `vertex_rust_sdk`), multi-protocol agent wire integration (`MCP`/`A2A`/`ACP`/`ANP` `did_wba`/`stdio`/`SSE`/`HTTP`/`QUIC` on `:9098`), Dedicated Multi-Process Supervisor (`toksi _worker <role>`) + Named Pipe/UDS `DirectWireBus` (`src/ipc/`), Policy-as-Code (`opa`/`cedar`) & SPIFFE IAM attestation (`spire`), Guardrails/`llm-guard`/`openai-guardrails-python`/`safeagents`/`garak`/Anonymizer + **ATR (`agent-threat-rules`)** + **`model_spec`**, Zero-Plaintext Blind Signature Engine (`src/security/` `BLAKE3` + `sparse::DFA`) & Native Git/PR Validator (`src/validator/` + 12-DB `UpstreamDeduplicator`), ADR layer (`adr.rs`), 5-backend Isolated Agent Sandbox (`sandbox.rs` w/ 3-OS native + `sandbox-runtime` + `OpenShell` + `E2B` + Docker `sbx` CLI parity), `CommandGuard` Bypass Kill-Switch (`guard.rs`), live-safe `docker/Dockerfile.test` + `toksi test --docker|--sandbox` & `TOKSI_HOME`/`:0` isolation, externalized Zero-Pollution `<skills>` (`Navegos/skills` + `skilldoctor` + `agentskills` YAML healer + `upskill` evals + `skill-scanner` + `defenseclaw` + `<neo>/.agents/skills/` 922-dir cleanup), Temporal/Restate/LangGraph durable workflow state & plan checkpoint/resume (`.toksi/projectmem/plan_checkpoints/`), hybrid graph-vector memory (`Cognee`/`Graphiti`/`Qdrant`/`Lance`/`arrow-rs`/`Chroma`/`Milvus`/`Letta`/`funes`), **30+** local/Rust inference & serving engines (`oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `mistral.rs`, `candle`, `dynamo`, `ollama`, `llama.cpp`, `whisper.cpp`, `stable-diffusion.cpp`, `triton`, `DeepSpeed-MII`), unified **Model Training/Quantization/Serving/Eval Plane (`src/training/` `toksi train|serve|eval` + `/voice` via `livekit*`)**, unified **35-component `<toksi-siagent>`** & Multi-Agent Workforce (`OMA`, `swarms-rs`, `ChatDev`, `OpenSpec` `/spec`, `adk-rust`, `rig`, `maki`, `steer`, `chevalier`, `agentrix-os`, `ironclaw`, `brat`), **21 universal slash commands**, and 9-tab async telemetry cockpit across **432 unique mandated vaults (`[DEEP-RESEARCH-DONE]`)**.

---

## 1. System Overview & Architectural Mandates

`toksi` is a pure Safe Rust (Edition 2024, `#![deny(unsafe_code)]`) **35-component `<toksi-siagent>` Super Intelligence Runtime (`RESEARCHPLAN.md` Section 0.2)** combining headroom proxy (`live_zone` byte-range surgery + `CcrStore`), token compression engine, native AI gateway, Dedicated Multi-Process Supervisor (`toksi _worker <role>`), multi-protocol agent wire platform (`MCP`/`A2A`/`ACP`/`ANP`/`QUIC`), 3-repository security/advisory/skills governance suite, isolated sandboxed agent runtime, model training/serving orchestrator, and standalone coding CLI. It sits transparently between local AI coding assistants (`antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude`, `cursor`, `codex`, `opencode`, `grok`, and 65/90+ ecosystem connectors across Categories 1–5 + **30+** local & Rust-native inference engines) and configured BYOK providers (including Google Cloud ADC Vertex endpoints) or dedicated tool worker processes (`rtk`, `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `security`, `advisory`, `skills`, `sandbox`).

### Core Architectural Mandates:
1. **100% Native In-Tree Rust (`#![deny(unsafe_code)]`) & Zero Binary Bloat**: Complete elimination of `npm`, `pip`, `uv`, `python`, `node`, and `set_var`/`LockFileEx`/`setsid` unsafe blocks; unavoidable Win32 FFI and raw C-ABI Tree-sitter shims live in isolated workspace leaf crates (`crates/toksi-sys`, `crates/toksi-grammars`). Zero missing code across all **432 unique reference vaults** (`inspect-vault/*` across Sections `2.1`–`2.14`). Skills and rules Markdown/YAML/JSON payloads are NEVER embedded via `include_str!` inside `crates/toksi/` — stage externally in `<neo>/agents/`, `<neo>/rules/`, `Navegos/skills`, and `~/.toksi/user-skills`. Audit and clean all **922** unadapted dirs in `<neo>/.agents/skills/` (`PHASE_19_4`).
2. **Continuous Reference Vault Re-Audit Across 3-Repository Ecosystem (All 432 Unique Vaults = 294 Waves 1–18 + 138 Wave 19 — `[DEEP-RESEARCH-DONE]`)**: Enforce `<enforce>` security across `<neo>` (`Navegos/neo`), `<advisory>` (`Navegos/advisory`), and `<skills>` (`Navegos/skills`), synthesizing all **432 unique vaults** (`36` Waves 1–17 + `258` Wave 18 across Sections `2.1`–`2.10` incl. `xla` + companion repos + `138` Wave 19 additions/reconciliations across Sections `2.10`–`2.14`: `56` Sec `2.11` pure-Rust/swarm/spec/voice agents, `28` Sec `2.12` Rust systems/IPC/storage/`vertex_rust_sdk`/`pmat`/`tiktoken`/`BLAKE3`/`lancedb`/`arrow-rs`, `47` Sec `2.10`/`2.13` training/quantization/ONNX/TRT/serving/MLOps/`oxillama`/`xinfer`/`Crane`/`paddler`/`llama-swap`/`llguidance`/`whisper*`/`sd.cpp`/`cudarc`/`unsloth*`/`Automodel`/`lighteval`/`langwatch`, and `7` Sec `2.14` `agent-threat-rules`/`openai-guardrails-python`/`model_spec`/`upskill`/`qdrant-skills`/`crewAIInc-skills`/`Essential-Hermes-Skills`).
3. **Dedicated Multi-Process Supervisor (`toksi _worker <role>`) & Direct-Wire IPC (`src/ipc/` — `PHASE_19_2`)**: `toksi daemon start [--agents ...]` replaces single-process Tokio tasks (`commands/daemon.rs`) with an OS supervisor managing isolated `toksi _worker <role>` subprocesses per Network Port (`headroom:8787`, `grok:8788`, `copilot-vscode:8789`, `copilot-cli:8790`, `protocol-bus:9098`, `gateway:9099`, `dashboard:9100`, `byok-gateway:18787`), `<toksi-siagent>` (`toksi _worker siagent`), and each core tool (`codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`) interconnected over Windows Named Pipes (`//./pipe/toksi-*`) / Unix Domain Sockets (`~/.toksi/run/*.sock`) + `DirectWireBus` (`pingora`/`iggy`/`quinn`/`rpmalloc-rs`) with watchdog backoff restart and `CommandGuard` kill-switch fan-out (`supervisor_workers` SQLite table).
4. **Configurable Global Runtime (`~/.toksi/config.toml` -> `src/config/mod.rs`)**: Controls ports (`headroom=8787`, `grok_oauth=8788`, `copilot_vscode=8789`, `copilot_cli=8790`, `protocol_bus=9098`, `gateway=9099`, `dashboard=9100`, `byok_gateway=18787`), `[repositories]` (`neo`, `advisory`, `skills`), `[supervisor]` (`worker_restart_max`, `ipc_transport = "auto"`, `worker_health_interval_ms`), `[testing]` (`default_mode`, `docker_image = "toksi-test:latest"`), `[auth.gcp_adc]` (`AGY_ADC_AUTH=true`, `"gcp.project"`, `"gcp.location"` default `us-central1`), `[optimization]` (`default_effort`, `[effort]` dynamic compression), `[security]` (`defenseclaw_fail_closed`, `skill_scanner_policy`, `kill_on_bypass`, `sandbox_backend`, `blind_dfa_path`, `disclosure_embargo_days`), `[plugins]` (`plugin_dirs`, `enforce_security_scan`, `fail_closed_hooks`, `protocols`, `allow_agent_to_agent`), `[checkpoint]`, default `--agents`, detached `dashboard`, and multi-instance `[[byok_profiles]]` synchronized with `~/.toksi/toksi.db` (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`, `supervisor_workers`).
5. **Live-Safe Sandboxed & Docker Test Isolation (`PHASE_19_3`)**: `cargo test` unit suites isolate `TOKSI_HOME` tempdirs and `:0` ephemeral ports; `docker/Dockerfile.test` & `docker/docker-compose.test.yml` power `toksi test --docker|--sandbox` so full daemon, worker, port, and hook E2E verification never touches live host `toksi` processes, `~/.toksi/toksi.db`, or OS env/keyring while honoring `.toksi/test-baseline.json` (`35` suites tracked).
6. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**: Verify external binaries in system `PATH` cleanly (`rust-analyzer`, `gopls`, `clang-tidy`, `gcloud`, `docker`, `sbx`, `whisper-cli`, `unsloth`, agent CLIs) without hardcoding paths. Missing binaries remain grayed out / disabled with explicit user remediation notices and zero silent failure.
7. **Universal Forward-Slash (`/`) Path Standard**: Normalize internal paths with `/` on Windows, Linux, and macOS (escaped `\\` permitted exclusively inside `cmd.exe` command strings in JSON5/TOML configs).
8. **Strict Scope Split — Home Config vs. Repo-Only Tool Stores vs. External Skills**: `~/.toksi/` holds ONLY `toksi.db`, `config.toml`, `run/*.sock`, optional user-level `~/.toksi/plugins/`, and `~/.toksi/user-skills/`. ALL workspace intelligence stores (`codegraph`, `projectmem`, **`ctxmode`**, `icm`, `ai-memory`, `rtk`, `headroom`, `security`, repo `plugins`) live strictly inside `<repo>/.toksi/<tool>/` where `<repo>/.git/` exists — NEVER in `~/.toksi/<tool>`, never outside `<repo>`, and never inside `<repo>/.git/`. Monorepo subprojects (`<repo>/crates/*`) share root `<repo>/.toksi/` partitioned by `subproject_id`.
9. **Workspace Root Consolidation, `ctxmode` Auto-Migration & Safe Plan Checkpoints**: Auto-merge legacy dotdirs (`legacy .projectmem/`, `.context-mode/`, `<repo>/.toksi/context/`) and nested crate `.toksi/` / `.<agent>/` folders row-by-row into `<repo>/.toksi/<tool>/` without data loss; persist atomic execution snapshots (including active plugin & sandbox state) in `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` for instant crash/429 `/resume`.
10. **Mandatory `ctxmode` Rename Everywhere (`PHASE_19_1`), `<enforce>` + `defenseclaw` + `skill-scanner` + `skilldoctor`, Zero-Plaintext Blind Engine, & Unified Plugin System (`src/plugins/`)**: Uniform `<tool>-hook <agent>` (`ctxmode-hook <agent>`), normalized MCP IDs (`projectmem`, **`ctxmode`**, `codegraph`, `icm`, `aimemory`, `rtk`, `security`, `advisory`, `skills`, `sandbox` — migrating all `316` legacy `context-mode` refs cleanly), `.agents/mcp` + `.gemini/mcp` + `.copilot/mcp` + `.claude/mcp` -> `skills/<tool>`, `antigravity-cli` vs `antigravity-ide` separation, mandatory `<enforce>` guards, `defenseclaw` + `skill-scanner` + `ATR` + `openai-guardrails-python`, `skilldoctor` + `agentskills` + `upskill` healer (`src/rules/skill_validator.rs`), `src/security/` Zero-Plaintext Blind Signature Engine (`BLAKE3` + `sparse::DFA`, `toksi compile-blind`) + `src/validator/` Native Git/PR Validator (`DisclosureGate`, 12-DB `UpstreamDeduplicator`, `fp_pruner`, `shard_builder`, `git_pr_driver`, `toksi advisory`), and `DirectWireBus`.
11. **Multi-Protocol Standardization & 45-Language AST/SAST Intelligence**: Full `MCP`, `A2A` (v1.0 JSON-RPC), `ACP` (`agent-client-protocol`), **`ANP` (`did_wba`)**, `stdio`, `SSE`, `HTTP`, and **`QUIC` (`quinn`)** on `:9098` Protocol Bus (serving core subsystems + `src/plugins/` endpoints) + native parsing for all 45 language IDs in `src/codegraph/kernel/langs.rs` (`language_vue()`, `language_scss()`, `language_dockerfile()` live) powering both CodeGraph and the built-in AST static code security analyzer + OSV lockfile scanner (`src/security/`), with `.gitignore`/`.ignore` dynamic index purge.
12. **Zero Paid-Tier Restrictions, Google Cloud ADC (`AGY_ADC_AUTH=true`), & Multi-Instance `<byok, model, effort>`**: Remove `FreeTierDisqualified` / `[PAID]` / `[DISQUALIFIED-FREE]` blocks completely. Any enabled user-configured BYOK, Google Cloud ADC Vertex (`auth_mode="adc"`, `vertex_rust_sdk`), or CLI account participates freely; `Low`/`Medium`/`High` thinking efforts translate dynamically across OpenAI/Anthropic/Gemini/Vertex while scaling Headroom compression aggressiveness.
13. **Unified 35-Component `<toksi-siagent>`, Multi-Agent Workforce, Model Training/Serving/Voice, ADR, 5-Backend Sandbox, & `CommandGuard` Kill-Switch**: Unify all `<neo>/inspect-vault/*<agent>` source commands, skills, rules, MCPs, and APIs (incl. `opencode`, `adk-rust`, `rig`, `maki`, `steer`, `chevalier`, `agentrix-os`, `ironclaw`, `brat`, `OMA`, `swarms-rs`, `ChatDev`, `OpenSpec`, `livekit*`) into `toksi chat | run | exec | swarm | train | serve | eval | sandbox | advisory | skills | security | plugin | agent resume`; protect agent execution with OPA/Cedar + SPIRE + Guardrails/`llm-guard`/`openai-guardrails-python`/`ATR`/`model_spec`, `src/security/adr.rs`, `src/agent_runtime/sandbox.rs`, `src/commands/guard.rs` kill-switch, unified `src/plugins/` mounts, and **21 in-chat slash commands** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, **`/ctxmode`**, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, **`/workforce`**, **`/spec`**, **`/train`**, **`/serve`**, **`/voice`**).
14. **Ground-Truth Accounting, `tokview`/`langwatch`/`phoenix` Traces, ConnectRPC Passthrough, & 9-Tab Async UI**: Idle reads strictly `0`; zero-cost `/v1/models`; `ArcSwap<reqwest::Client>` reconnect heal; 9-tab async TUI & `:9100` Cockpit with worker health/restart, workforce, training/serving, security, and DB export/import.

---

## 2. Unified 8-Port Multi-Process Network Topology (Overridable via `~/.toksi/config.toml`)

All 8 ports, `<toksi-siagent>`, and core tools run in isolated `toksi _worker <role>` subprocesses coordinated over Named Pipes / UDS `DirectWireBus` (`PHASE_19_2`), guaranteeing zero socket buffer contention and crash isolation:

| Port / Worker Role | Service / Subsystem | Transport / IPC | Protocol | Isolation Guarantee |
|---|---|---|---|---|
| **Port 8787** (`_worker headroom`) | Streaming Headroom Proxy | TCP / HTTP/1.1 & HTTP/2 / SSE + UDS/Pipe | HTTP chunked, SSE, ConnectRPC passthrough | Dedicated subprocess for Antigravity, CloudCode, IDE base URLs (`live_zone` byte surgery, effort scaling, & `CcrStore`). |
| **Port 8788** (`_worker grok`) | Grok OAuth Proxy (`grok_daemon.rs`) | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked, SSE | Isolated xAI Grok OAuth proxy subprocess (NOT generic BYOK). |
| **Port 8789** (`_worker copilot-vscode`) | GitHub Copilot VS Code Proxy | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked | Isolated proxy subprocess (`DEFAULT_COPILOT_PROXY_PORT`). |
| **Port 8790** (`_worker copilot-cli`) | GitHub Copilot CLI Proxy | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked | Isolated proxy subprocess (`DEFAULT_COPILOT_CLI_PROXY_PORT`). |
| **Port 9098** (`_worker protocol-bus`) | **Unified Protocol Bus** | TCP / HTTP / SSE / **QUIC** + UDS/Pipe | `MCP`, `A2A` v1.0, `ACP`, **`ANP` (`did_wba`)**, checkpoint events | Isolated agent control & IDE bus passing payloads through `HeadroomEffortCompressor` (`Low`/`Med`/`High`) & `<enforce>` across all **432** unique vaults & `Plugins`. |
| **Port 9099** (`_worker gateway`) | Enterprise AI Gateway & Drop-in SDKs | TCP / HTTP / SSE + UDS/Pipe | `/v1/*`, `/v1/security/scan`, `/v1beta/models/*`, `/metrics` | Isolated inference pool (`pingora` pooling, <10 µs overhead) enforcing `HeadroomEffortCompressor`, Vertex ADC (`vertex_rust_sdk`), & failover. |
| **Port 9100** (`_worker dashboard`) | Web Telemetry & Control Dashboard Cockpit | TCP / HTTP / SSE / WS + UDS/Pipe | Browser 9-Tab HTML/CSS/JS, `tokview`, worker/training controls | Detached subprocess. Zero inference contention. |
| **Port 18787** (`_worker byok-gateway`) | BYOK & Vertex ADC Gateway (`byok_gateway.rs`) | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked, SSE | Supervised multi-instance BYOK, Vertex ADC & 30+ local engine dispatcher passing every request through `HeadroomEffortCompressor`. |
| **Tool & Agent Workers** | `_worker siagent`, `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security` | Named Pipe / UDS (`DirectWireBus`, `iggy` framing) | In-band RPC + health pings + kill-switch broadcast | Crash-isolated per-subsystem subprocesses with watchdog restart & fault containment. |

All default ports above are overridable via `~/.toksi/config.toml` `[ports]` and `[supervisor]`.

---

## 3. Disciplinary Coding Standards & Conventions

All agent workers and contributors must adhere to these non-negotiable coding standards across all tasks:

### 3.1 Rust Core Disciplines
1. **Safe Rust (Edition 2024)**:
   - Safe Rust is enforced across `crates/toksi` (`src/lib.rs` and `src/main.rs`) via `#![deny(unsafe_code)]` (`0` unsafe blocks across `456` `.rs` files).
   - Eliminate all `std::env::set_var`/`remove_var` (use `ToksiConfig` + `Command::env()`), raw `LockFileEx`/`flock` (use `std::fs::File::lock()` / `fs2`), raw `pre_exec(setsid)` (use `CommandExt::process_group(0)` or `toksi-sys`), raw signal handlers (`tokio::signal` / `ctrlc`), and raw process walkers (`sysinfo`).
   - Unavoidable Win32 FFI (`advapi32`, `user32`, console) and C-ABI Tree-sitter wrappers live strictly in isolated workspace leaf crates `crates/toksi-sys` and `crates/toksi-grammars` with explicit `// SAFETY:` proofs.
2. **Zero Warnings Policy**:
   - The project must compile with zero compiler warnings and zero Clippy warnings (`RUSTFLAGS="-D warnings"`).
   - Zero dead code: unused functions, structs, fields, and imports must be deleted immediately.
3. **Strict Error Handling (Zero Panics in Production)**:
   - Prohibited in all library, server, proxy, and daemon code paths: `.unwrap()`, `.expect()`, `panic!()`, `unreachable!()`.
   - Domain errors use `thiserror` enums (`GatewayError`, `HeadroomError`, `KeyringError`, `IpcError`, etc.).
   - CLI handlers use `anyhow::Result` with clear context (`.context("...")`).
4. **Async, Multi-Process & Concurrency Architecture**:
   - Tokio multi-threaded work-stealing scheduler inside each isolated `_worker` subprocess.
   - Never perform synchronous blocking disk I/O, file locking, or CPU-heavy parsing on async worker threads. Wrap them in `tokio::task::spawn_blocking`.
   - Inter-process Direct-Wire messaging uses Named Pipes / UDS (`src/ipc/`) plus lock-free intra-process channels (`tokio::sync::mpsc`) and atomics (`AtomicBool`, `AtomicU64`).
5. **Structured Logging**:
   - Standardized on `tracing` (`tracing::info!`, `tracing::warn!`, `tracing::error!`, `tracing::debug!`).
   - Library code must never emit unstructured `println!` or `eprintln!`.

---

## 4. Cross-Platform Portability Matrix (Windows, Linux, macOS)

| Concern | Windows Standard | Linux Standard | macOS Standard | Toksi Implementation Mandate |
|---|---|---|---|---|
| **Path Separator** | Universal forward-slash `/` (`C:/Users/...`) | Standard forward-slash `/` (`/home/...`) | Standard forward-slash `/` (`/Users/...`) | `src/util/paths.rs`: `normalize_path(&str)` converts all `\` to `/`. Escaped `\\` allowed only in cmd.exe commands inside JSON5/TOML configs. |
| **Filesystem Casing** | Case-preserving, Case-insensitive | Case-sensitive | Case-preserving, Case-insensitive | All tool and agent directory lookups lowercase search terms before comparison. |
| **Database Storage** | `C:/Users/<User>/.toksi/toksi.db` | `$HOME/.toksi/toksi.db` | `$HOME/.toksi/toksi.db` | `dirs::home_dir()` + `.toksi/toksi.db`. Automated startup migration from legacy `LOCALAPPDATA`. |
| **Inter-Process Bus** | Named Pipes (`//./pipe/toksi-*`) | Unix Domain Sockets (`~/.toksi/run/*.sock`) | Unix Domain Sockets (`~/.toksi/run/*.sock`) | `src/ipc/` (`named_pipe.rs`, `uds.rs`) connecting supervisor, port workers, `<toksi-siagent>`, and tools. |
| **File Locking** | Win32 `LockFileEx` (Mandatory) | POSIX `flock` / `fcntl` (Advisory) | POSIX `flock` / `fcntl` (Advisory) | `fs2::FileExt` (`lock_exclusive`, `lock_shared`) for atomic JSONL appends and SQLite WAL checkpoints. |
| **OS Keyring** | `advapi32.dll` (`CredReadW`, `CredWriteW`, `CredDeleteW`) | Secret Service API / `secret-tool` / D-Bus | macOS Keychain (`/usr/bin/security`) | `src/keyring/mod.rs`: `KeyringManager` with base64 OAuth envelope parser. Zeroize in-memory credentials on drop. |
| **Environment Injection** | `HKCU\Environment` Registry + `WM_SETTINGCHANGE` broadcast | `systemd --user set-environment` / `~/.config/environment.d/` | `launchctl setenv` / User session export | `src/env_injector/mod.rs`: Non-destructive injection with RAII `TeardownGuard` clearing variables on daemon shutdown. |
| **Background Processes** | `CREATE_NO_WINDOW (0x08000000)` & `DETACHED_PROCESS (0x00000008)` | `setsid()`, redirect stdio to `/dev/null` or logs | `setsid()`, redirect stdio to `/dev/null` or logs | Spawns clean detached supervisor & `_worker` subprocesses without console windows or zombie processes. |
| **Process Signals** | `tokio::signal::ctrl_c()` | `SIGINT`, `SIGTERM`, `SIGHUP` via `tokio::signal::unix` | `SIGINT`, `SIGTERM`, `SIGHUP` via `tokio::signal::unix` | Clean signal capture invoking `TeardownGuard` and graceful `_worker` shutdown. |
| **Network Binding** | Bind strictly to `127.0.0.1` / `::1` | Bind strictly to `127.0.0.1` / `::1` | Bind strictly to `127.0.0.1` / `::1` | Prohibit default `0.0.0.0` binding. Set `SO_REUSEADDR` to avoid `EADDRINUSE` on restart. |
| **Line Endings** | Handles `\r\n` and `\n` transparently | Native `\n` | Native `\n` | Normalize `\r\n` to `\n` before processing, indexing, and token counting. |
| **Executable Suffix** | Appends `.exe` dynamically | Bare executable name | Bare executable name | `std::env::consts::EXE_SUFFIX` used dynamically; `check_executable_in_path` tests both forms on Windows. |

---

## 5. Subsystem Directory Topology

```
docker/                       # Planned Wave 19.3 Live-Safe Container Test Isolation
├── Dockerfile.test
└── docker-compose.test.yml

crates/toksi/src/
├── main.rs                   # Unified CLI entrypoint (#![deny(unsafe_code)]) & full usage help
├── lib.rs                    # Public crate library API (#![deny(unsafe_code)])
│
├── ipc/                      # Planned Wave 19.2 Dedicated Multi-Process Named Pipe / UDS Direct-Wire IPC
│   ├── mod.rs, bus.rs, named_pipe.rs, uds.rs, iggy_frame.rs, quic.rs
│
├── training/                 # Planned Wave 19.6 Model Training/Quantization/Eval Orchestration
│   ├── mod.rs, recipes.rs, evals.rs
│
├── projectmem/               # Pure Rust port of ProjectMem — strict <repo>/.toksi/projectmem/ + plan_checkpoints/
│   ├── mod.rs, schema.rs, store.rs, summary.rs, precheck.rs, redaction.rs, heatmap.rs, mcp.rs, plan_checkpoint.rs, wiki_sync.rs
│
├── icm/                      # Instant Context Memory SQLite FTS5 BM25 + temporal/fusion/memoir (<repo>/.toksi/icm/)
│   ├── mod.rs, store.rs, schema.rs, recall.rs, mcp.rs
│
├── aimemory/                 # Hybrid Graph-Vector Memory: OKF v0.2 wiki, handoff, Lance (arrow-rs), Qdrant, Milvus, Cognee, Graphiti, Letta, funes, TEI (<repo>/.toksi/ai-memory/)
│   ├── mod.rs, wiki.rs, handoff.rs, tiered.rs, mem0_bridge.rs, lance_store.rs, qdrant_store.rs, milvus_adapter.rs, cognee_graph.rs, temporal_graph.rs, letta_memgpt.rs, embeddings.rs, retriever.rs
│
├── codegraph/                # 45-lang Tree-sitter AST, SCIP, ast-grep CST, jcodemunch, repomix, OpenWiki, pmat TDG, tiktoken BPE (<repo>/.toksi/codegraph/)
│   ├── mod.rs, kernel/, scip.rs, ast_grep.rs, jcodemunch.rs, repomix.rs, openwiki.rs, bpe.rs, embeddings.rs, reranker.rs, walker.rs, graph.rs, cache.rs, mcp.rs
│
├── contextmode/              # Subprocess sandbox, policy/journal, SQLite FTS5 search, WFST text_norm (Wave 19.1 -> `ctxmode`, <repo>/.toksi/ctxmode/)
│   ├── mod.rs, sandbox.rs, filter.rs, fts.rs, text_norm.rs, fetch.rs, mcp.rs
│
├── security/                 # OPA/Cedar/SPIRE IAM, Guardrails/LLM-Guard/OpenAI-Guardrails/ATR/model_spec, Zero-plaintext BLAKE3+sparse::DFA, ADR, defenseclaw, skill-scanner, 45-lang SAST, sage, OSV/EPSS (<repo>/.toksi/security/)
│   ├── mod.rs, policy_opa.rs, policy_cedar.rs, iam_attestation.rs, guardrails_validators.rs, llm_guard.rs, safeagents.rs, garak_redteam.rs,
│   ├── schema.rs, polyglot.rs, blind_compiler.rs, scanner.rs, advisory_client.rs, secure_export.rs, adr.rs,
│   ├── analyzer.rs, defenseclaw.rs, hookguard.rs, skill_scanner.rs, cel_decision.rs, pipeline_taint.rs, bytecode_audit.rs,
│   └── sarif.rs, audit.rs, yara_mcp.rs, sage_rules.rs, skill_audit.rs, osv.rs, epss.rs, semgrep_engine.rs, external_analyzers.rs
│
├── validator/                # Native CVE 5.1 / OSV 1.7.4 validation, FP pruning, 3-tree shards & REST PR driver (Navegos/advisory)
│   └── mod.rs, validate.rs, fp_pruner.rs, shard_builder.rs, git_pr_driver.rs
│
├── plugins/                  # Toksi Unified Plugin System (<enforce> scanned: agent_feature, cli_cmd, tool, chat_cmd, wire_hook, protocols, a2a/anp)
│   ├── mod.rs, manifest.rs, loader.rs, hooks.rs, slash_router.rs, cli_bridge.rs, analyzer_plugin.rs
│
├── agent_runtime/            # 35-Component <toksi-siagent> CLI, Workforce & Swarm wired off-the-back via DirectWireBus & :8787..:8790/:9098/:9099/:18787 to all 432 vaults
│   ├── mod.rs, types.rs, loop.rs, context.rs, edit_tools.rs, planning.rs, orchestrator.rs, reflection.rs, workflow_state.rs, durable_journal.rs, world_model.rs,
│   └── sandbox.rs, openshell.rs, sandbox_e2b.rs, cluster_exec.rs, rl.rs, codemode.rs, session.rs, slash.rs, event_log.rs, cron.rs, checkpoint.rs, acp.rs, guardrails.rs, graph_checkpoint.rs, deep_planning.rs, fleet.rs, modes.rs, zeroclaw_core.rs, swarm/
│
├── config/                   # Global ~/.toksi/config.toml loader, schema validator, hot-reload, [repositories], [supervisor], [testing], [auth.gcp_adc] & DB sync
│   ├── mod.rs
│
├── headroom/                 # Shared HeadroomEffortCompressor (Low/Med/High: live_zone, CcrStore, SmartCrusher, TokenEconomizer, agent-opt, mcp_pruner, tiktoken BPE) + Copilot/Grok/BYOK proxies
│   ├── mod.rs, copilot_daemon.rs, grok_daemon.rs, byok_gateway.rs, proxy.rs, effort.rs, compression/, pruner/, cache_align.rs, adaptive_mem.rs
│
├── gateway/                  # Enterprise AI Gateway (:9099), Protocol Bus (:9098), Vertex ADC (:18787/:9099, vertex_rust_sdk), Pingora pooling, 30+ Local/Rust Engines & Failover Engine — all enforcing HeadroomEffortCompressor
│   ├── mod.rs, reconnect.rs, local_engines.rs, inference_bridge.rs, inference_mistral.rs, inference_candle.rs, disaggregated_serving.rs, cost_quality_router.rs, quant_catalog.rs, vertex_adc.rs, gemini_wire.rs, effort_router.rs, providers/, agent_cli/, cascade.rs, keys.rs, loadbalancer.rs, routing.rs,
│   ├── governance.rs, semantic_cache.rs, normalizer/, compat/, circuit_breaker.rs, proxy.rs
│
├── telemetry/                # Ground-Truth Session Metrics, tokview/Langfuse/LangWatch/OpenInference spans, tokenspeed, GPU/cudarc/vmon probes & lighteval
│   ├── mod.rs, prometheus.rs, collector.rs, tokentracker.rs, tokview.rs, traces.rs, openinference.rs, tokenspeed.rs, gpu_monitor.rs, evals.rs, cost.rs, tracker.rs, cache_analytics.rs
│
├── rtk/                      # 100+ filters, ANSI stripper, SQLite Tracker (rtk gain), tee recovery, discover, learn, slash
│   ├── mod.rs, engine.rs, filters/, ansi.rs, hook.rs, tracker.rs, tee.rs, discover.rs, learn.rs, slash.rs
│
├── rules/                    # Externalized <skills> Sync (<neo>/agents/ & <neo>/rules/, zero binary bloat), <enforce> Guard, skilldoctor + agentskills + upskill Healer/Eval & 922-dir hygiene
│   ├── mod.rs, sync.rs, skills_repo.rs, skill_validator.rs, skills_catalog.rs, principles.rs, caveman.rs, ponytail.rs
│
├── env_injector/             # Non-destructive OS environment injection & legacy binary healing
│   ├── mod.rs, windows.rs, unix.rs, teardown.rs, heal.rs
│
├── dashboard/                # Background Web Telemetry Cockpit on Port 9100 (9-tab SPA + SSE + REST + worker/training controls)
│   ├── mod.rs, state.rs, sse.rs, api.rs, assets/
│
├── tui/                      # Non-blocking async 9-tab mouse TUI (ratatui + crossterm EnableMouseCapture)
│   ├── mod.rs, app.rs, views/, widgets/
│
├── agents/                   # Configured/selected agents & Category 1-5 connectors (incl. opencode) + direct-wire bus
│   ├── mod.rs, manager.rs, open_agents.rs, antigravity.rs, copilot.rs, claude.rs, codex.rs, cursor.rs, grok.rs, opencode.rs, opt.rs, mcp_adapters.rs
│
├── keyring/                  # Cross-platform secure credential manager + Google Cloud ADC (AGY_ADC_AUTH=true)
│   ├── mod.rs, adc.rs, os_windows.rs, os_macos.rs, os_linux.rs, envelope.rs
│
├── db/                       # Persistent SQLite configuration database (~/.toksi/toksi.db) + byok_profiles + plugins_registry + blind_signatures + advisory_embargoes + adr_audit_events + supervisor_workers
│   ├── mod.rs, schema.rs, operations.rs, migrate.rs
│
├── util/                     # Forward-slash path normalization, PATH validation mandate, exec, parsers
│   ├── paths.rs, env_path.rs, exec.rs, jsonc.rs, toml.rs, colors.rs
│
└── commands/                 # Native CLI command dispatcher
    ├── daemon.rs, worker.rs (planned 19.2), proxy.rs, config.rs, agent_runtime.rs, train.rs (19.6), serve.rs (19.6), eval.rs (19.6), guard.rs, compile_blind.rs, advisory.rs, skills.rs, sandbox.rs, security.rs, plugin.rs, test_runner.rs, auth.rs, db.rs, agent.rs, tui.rs, dashboard.rs, mem.rs, graph.rs, ctx.rs, rtk.rs, rules.rs, doctor.rs, mod.rs
```

---

## 6. Detailed Execution Roadmap (Phases 15.1 – 16.3)

All tasks are critical and must be executed with strict technical precision. *(8-State Badges added in Wave 19 / v6.0.)*

### Phase 15.1: Workspace Topology, Forward-Slash Normalization & Standards `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]`
- **Objectives**: Universal forward-slash `/` paths, PATH validation mandate, user-home DB storage, single root `.toksi` consolidation, standardized hook/MCP naming (`ctxmode-hook <agent>` in `PHASE_19_1`), and root/skills Zero-Pollution hygiene (`PHASE_19_4`).
- **Targets**:
  * `crates/toksi/src/util/paths.rs`: Implement `normalize_path(p: &str) -> String` converting all backward slashes `\` to `/` across all OSes. Handle drive letters (`C:/`), UNC paths (`//server`), and tilde expansions. Preserve escaped `\\` exclusively for cmd.exe command strings in JSON5/TOML configs.
  * `crates/toksi/src/util/env_path.rs`: Implement `check_executable_in_path(cmd: &str) -> Result<(), String>`. Verify required binaries exist in `PATH` without encoding absolute hardcoded paths.
  * `crates/toksi/src/db/mod.rs` & `operations.rs`: Relocate default SQLite DB to `~/.toksi/toksi.db` (`dirs::home_dir()`). Auto-migrate legacy `LOCALAPPDATA/toksi` data on startup.
  * `crates/toksi/src/commands/init.rs` & `doctor.rs`: Single root `.toksi` consolidation: discover subfolder `.toksi` directories, merge rows into root `.toksi/toksi.db`, `memory.db`, `symbols.db` without data loss, and delete nested `.toksi` folders.
  * `crates/toksi/src/agents/hooks.rs` & `mcp_adapters.rs`: Standardize hook syntax to `<tool>-hook <agent>` (`codegraph-hook agy`, `projectmem-hook agy`, `ctxmode-hook agy`, `rtk-hook agy`), normalize MCP identifiers (`projectmem`, `ctxmode`, `codegraph`), and map MCP settings to `skills/<tool>`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.2: Unified 8-Port Daemon, `config.toml`, & Copilot Supervision `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]`
- **Objectives**: Supervise all 8 dedicated endpoints simultaneously (`:8787`, `:8788` Grok, `:8789`/`:8790` Copilot, `:9098` Protocol Bus `MCP`/`A2A`/`ACP`/`ANP`/`SSE`/`QUIC`, `:9099` Gateway, `:9100` Cockpit, `:18787` BYOK — all overridable via `~/.toksi/config.toml` `[ports]`). Currently single-process Tokio tasks (`commands/daemon.rs:491-588`); split into dedicated `toksi _worker <role>` subprocesses in `PHASE_19_2`.
- **Targets**:
  * `crates/toksi/src/commands/daemon.rs`: Master daemon supervisor starting dedicated subprocesses per port (`8787..18787`), `<toksi-siagent>`, and core tools.
  * `crates/toksi/src/headroom/copilot_daemon.rs`: Dual-port supervisor managing 8789 and 8790 with live `/livez` health probes and collision prevention.
  * `crates/toksi/src/gateway/mod.rs` & `dashboard/mod.rs`: Guarantee thread, process, and socket buffer isolation between Port 9099 (inference) and Port 9100 (dashboard).
  * `crates/toksi/src/env_injector/teardown.rs`: Clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix environment on daemon shutdown.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.3: Smart Agent CLI Providers & Open BYOK Quota Telemetry `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]`
- **Objectives**: Register Local Agent CLIs (`AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, `GrokCli`) and ecosystem bridges with smart background launch; track read-only commercial/free quota windows with **zero paid-tier disqualification**.
- **Targets**: `crates/toksi/src/gateway/agent_cli/mod.rs`, `tier_check.rs`, `providers/aggregators/bifrost.rs`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]`
- **Objectives**: Expose MCP, A2A v1.0, ACP v2.0, **`ANP` (`did_wba`)**, stdio, SSE, HTTP, and **`QUIC`** across all tools; integrate `.gitignore` / `.ignore` and `<repo>`-only scoping.
- **Targets**: `crates/toksi/src/tools/multi_protocol.rs` (`rtk`, `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `A2aTaskStore`, `SessionActorQueue`), `codegraph/{walker,cache}.rs`, `contextmode/fts.rs`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Objectives**: Ground-truth session token accounting from live SSE streams; network reconnection auto-healing watchdog; configured-only universal rules sync.
- **Targets**: `crates/toksi/src/telemetry/{tokentracker,collector}.rs`, `gateway/reconnect.rs`, `rules/sync.rs`, `main.rs` (`help_text()`).

### Phase 15.6: Advanced Vault Intelligence Engines Consolidation `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Objectives**: Consolidate core code-intelligence vaults into pure Rust with real end-to-end wiring (`TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `litellm`, `TokenEconomizer`, `agent-opt`).

### Phase 16.1: High-Performance Async 9-Tab Mouse-Capable TUI `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]`
- **Objectives**: Full mouse interaction and non-blocking asynchronous event loop eliminating terminal hangs, with exact 9-tab parity (`Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`), extended in Wave 19 for `_worker` subprocess controls, `/workforce`, `/spec`, and `toksi train|serve|eval`.
- **Targets**: `crates/toksi/src/tui/{mod.rs,app.rs,views/*}`.

### Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]`
- **Objectives**: Isolated, detached background 9-tab web UI on Port `9100` (`toksi dashboard start|stop|restart`) with zero synthetic stubs + Wave 19 worker, workforce, and training telemetry.
- **Targets**: `crates/toksi/src/dashboard/{mod.rs,api.rs,state.rs,sse.rs}` & `assets/{index.html,style.css,app.js}`.

### Phase 16.3: Open Agents Directory Protocols & README Verification `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]`
- **Objectives**: Support `.agents/` (incl. `.agents/mcp -> skills/<tool>`) and `agents-structures` vendor layouts; configured/user-selected auto-install rules; Wave 19 `README.md` rewrite (with Wave 19 runtime parity completed in `PHASE_19_1`..`19_6`).
- **Targets**: `crates/toksi/src/agents/{open_agents.rs,manager.rs,mcp_adapters.rs}`, `crates/toksi/README.md`.

---

## 7. Wave 17 & Wave 18 Execution Phases (`PHASE_17_1` – `PHASE_18_6`)

*(Detailed worker prompts in [`.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`](../../.agents/brain/INDEX_AND_EXECUTION_GUIDE.md). Executed with targeted verification + `cargo check` & `cargo clippy -- -D warnings`.)*

- **`PHASE_17_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — **`~/.toksi/config.toml`, CLI Process Controls, Edition 2024 `#![deny(unsafe_code)]`, & Zero Paid-Tier Restrictions**: `src/config/mod.rs`, `FreeTierDisqualified` removal, `crates/toksi-sys` isolation, `#![deny(unsafe_code)]`.
- **`PHASE_17_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — **Multi-Model `<byok, model, effort>` DB, Keyring Fix, `headroom-core` `live_zone`, & Universal Headroom Effort Pass**: `db/{schema,operations,migrate}.rs` (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`), `gemini:antigravity` keyring, shared `HeadroomEffortCompressor` across all 7 entrypoints.
- **`PHASE_17_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — **45 Tree-sitter Languages, `<repo>/.toksi/<tool>` Repo-Only Storage, Monorepo Partitioning, & `.ignore` Purge**: `crates/toksi-grammars` (`language_vue()`, `language_scss()`, `language_dockerfile()` live in `langs.rs`), strict `.git`-root `commands/index.rs`, `ast-grep`/`scip`/`jcodemunch`/`repomix`.
- **`PHASE_17_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — **Deep Vault Parity, 65/90+ Ecosystem & Local Engines, & Hook/MCP Cleanup**: `ctx.rs` & `runmcp.rs` fixed; `PHASE_19_1` completes `ctxmode-hook` rename and `PHASE_19_2` wires subprocess Direct-Wire IPC.
- **`PHASE_17_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — **Unified `<toksi-siagent>` Coding CLI (`src/agent_runtime/`), `DirectWireBus`, & Slash Commands**: **16** core `< 5ms` slash commands live; Wave 19 expands to **21** (`/ctxmode`, `/workforce`, `/spec`, `/train`, `/serve`, `/voice`).
- **`PHASE_17_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — **Ground-Truth SSE Accounting, Gateway Real Dispatch, OS Reconnect Watchdog, & ConnectRPC Passthrough**.
- **`PHASE_17_7`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NOT-DONE]` — **Async 9-Tab Mouse TUI, Live Web Cockpit (`:9100`), Incremental `.toksi/test-baseline.json` Harness (`35` suites), CLI `--help`, & Doc Sync**: live-safe Docker/sandbox test runner completed in `PHASE_19_3`.
- **`PHASE_18_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — **294 Waves 1–18 Vaults Synthesis**: `lance_store.rs`, `qdrant_store.rs`, `cognee_graph.rs`, `temporal_graph.rs`, `openwiki.rs`, `tokview.rs`, `openinference.rs`, `tokenspeed.rs`, `gpu_monitor.rs`, `src/plugins/`, 26 local/Rust engines, 140 compute/serving vaults.
- **`PHASE_18_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — **`<enforce>` 3-Repo Security, Zero-Bloat Externalized `<skills>`, Unified Plugin System (`src/plugins/`), `defenseclaw`, `skill-scanner`, `skilldoctor` + `agentskills` Healer, Zero-Plaintext Blind Engine (`src/security/`), Native Validator (`src/validator/`), & 45-Lang SAST**.
- **`PHASE_18_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — **Google Cloud ADC (`AGY_ADC_AUTH=true`), Thinking-Effort Optimization (`Low`/`Medium`/`High`), & Safe Plan Checkpoints (`/resume`)**.
- **`PHASE_18_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-FIXED]` — **Legacy `.projectmem/` Repair, Documentation Integrity & Final Workspace Gate**: hygiene checks & `.toksi/projectmem/*` migration fixed.
- **`PHASE_18_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — **Cross-Agent Source Unification, ADR Layer (`src/security/adr.rs`), 5-Backend Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`), & `CommandGuard` Bypass Kill-Switch (`src/commands/guard.rs`)**.
- **`PHASE_18_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` — **35-Component `<toksi-siagent>` Super Intelligence Architecture & Full Inference/Compute Stack Consolidation**.

---

## 8. Wave 19 / v6.0 Execution Phases (`PHASE_19_1` – `PHASE_19_6`)

- **`PHASE_19_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — **Mandatory `ctxmode` Tool Rename Everywhere**: Replace all `316` legacy `context-mode` references and conflicting `context` tool names in `crates/toksi/src/` (`tools/contextmode.rs`, `agents/{manager,mcp_adapters,copilot,antigravity,hooks,grok}.rs`, `commands/{ctx,runmcp,index,init,doctor,doctor_probe}.rs`, `util/{instructions,versions}.rs`, `main.rs`) with **`ctxmode`** (`toksi ctxmode` w/ deprecated hidden `ctx` alias, `/ctxmode`, `ctxmode-hook <agent>`, MCP server key `"ctxmode"`, `skills/ctxmode/`, `<repo>/.toksi/ctxmode/`), auto-migrating legacy `.context-mode/` and `<repo>/.toksi/context/` SQLite stores cleanly.
- **`PHASE_19_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — **Dedicated Multi-Process Supervisor (`toksi _worker <role>`) & Direct-Wire IPC (`src/ipc/`)**: Split `src/commands/daemon.rs` in-process Tokio tasks into isolated OS worker subprocesses per Network Port (`headroom:8787`, `grok:8788`, `copilot-vscode:8789`, `copilot-cli:8790`, `protocol-bus:9098`, `gateway:9099`, `dashboard:9100`, `byok-gateway:18787`), `<toksi-siagent>` (`toksi _worker siagent`), and each core tool (`codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`) wired over Named Pipes (`//./pipe/toksi-*`) / UDS (`~/.toksi/run/*.sock`) + `DirectWireBus` (`pingora`/`iggy`/`quinn`/`rpmalloc-rs`), backed by `supervisor_workers` in `toksi.db`, `[supervisor]` TOML, exponential watchdog restart, and `CommandGuard` kill-switch fan-out.
- **`PHASE_19_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — **Live-Safe Sandboxed & Docker Test Isolation**: Build `docker/Dockerfile.test` & `docker/docker-compose.test.yml`, wire `toksi test --docker | --sandbox` in `src/commands/test_runner.rs`, and isolate every `cargo test` harness onto disposable `TOKSI_HOME` tempdirs and `:0` ephemeral ports so unit and E2E runs NEVER disturb live host `toksi` services, `~/.toksi/toksi.db`, or OS env/keyring while honoring `.toksi/test-baseline.json` (`35` suites).
- **`PHASE_19_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-NEEDS-FIX]` — **Zero-Pollution Adapt-Only Repo Hygiene & `.agents/skills/` (922 Dirs) Cleanup**: Audit all **922** directories in `<neo>/.agents/skills/` alongside `<neo>/agents/` (`28` files), `<neo>/rules/` (`24` files), and crate trees; strip unadapted foreign bash/python scripts and GitHub Actions YAML; verify zero skill/rule files embedded in `crates/toksi/`; grade vetted adapted skills via `skilldoctor` + `agentskills` + `upskill` + `skill-scanner` (`<enforce>`) and sync cleanly to `Navegos/skills` (`80` files baseline) + zero-plaintext shards to `Navegos/advisory` (`30` files baseline).
- **`PHASE_19_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — **Wave 19 SI Agentic Stack, Workforce, `ANP`, Rust Systems & ATR/Guardrails**: Implement Wave 19 pure-Rust agents & durable primitives (`adk-rust`, `rig`, `maki`, `steer`, `chevalier`, `funes`, `agentrix-os`, `tura`, `tabby`, `refact`, `native-cli-ai`, `ironwire`, `claurst`, `tidev`, `ironclaw`, `xa`, `octos`, `rust-code`, `ringlet`, `ironcode`, `brat`, `arc-cli`, `aitk`), Workforce (`OMA`, `swarms-rs`, `ChatDev`, `deer-flow`, `openworker`, `dsh-superagent-prototype`, `langgraph-swarm-py`) + `/workforce`, `OpenSpec` + `/spec`, `ANP` (`did_wba`) + `QUIC` (`quinn`) on `:9098`, `bridge-mcp`, Rust systems (`vertex_rust_sdk`, `lancedb`, `arrow-rs`, `tiktoken`, `BLAKE3`, `pmat`, `depyler`, `coredb`, `databend`, `coach`), and `agent-threat-rules` + `openai-guardrails-python` + `model_spec`.
- **`PHASE_19_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` — **Model Training, Quantization, Pure-Rust/Multimodal Serving Plane & 43-File 8-State Sync**: Build `src/training/`, `commands/{train,serve,eval}.rs`, and `/train`, `/serve`, `/voice` unifying `oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `safetensors`, `ggml`, `whisper*`, `stable-diffusion.cpp`, `cudarc`, `server`/`core` (Triton), `DeepSpeed-*`, `unsloth*`, `Automodel`, `nanotron`, `Emerging-Optimizers`, `torchtune` (recipe ref only), `torch*`, `onnx*`, `Olive`, `intel-neural-compressor`, `digestai`, `sentence-transformers`, `cuvs`, `CV-CUDA`, `warp`, `xla`, `lighteval`, `langwatch`, `wandb`, `clearml*`, `livekit*`, `moly-ai`, `robius`, `robrix`, `hf-hub`; complete **43-file `.agents/brain/*.md` + 6 master doc** Wave 19 8-State synchronization (`[DEEP-RESEARCH-DONE]` / `[PLAN-DONE]` / exact runtime state badges).
