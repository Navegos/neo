# toksi --- System Architecture, Disciplinary Standards & Implementation Plan (Wave 22 / v9.0)

High-performance headroom proxy, token optimizer, native AI gateway (concurrent standard `agy` + `AGY_ADC_AUTH=true` Google Cloud ADC + 5-tier thinking-effort routing `Max|XHigh|High|Medium|Low` + per-model sampling compatibility matrix + auto `/v1` completion + `vertex_rust_sdk`), multi-protocol agent wire integration (`MCP`/`A2A`/`ACP`/`ANP` `did_wba`/`stdio`/`SSE`/`HTTP`/`QUIC` on `:9098`), **Single Dedicated `NetworkService` Network Manager (`toksi _worker network-manager`, preventing `os error 10048`) + Distinct Collision-Free Per-Service Ports + Headroom-Central `:8787` Routing (Copilot `:8789`/`:8790` Consolidated onto `:8787`, Zero Synthetic HTTP `502` Bypass) + On-Demand `<toksi-siagent>` Decoupled from `daemon start`** (`toksi daemon start|stop|restart|heal|status|dashboard|service <name> <action>`) + Named Pipe/UDS `DirectWireBus` (`src/ipc/`), **`r3bl-open-core` (`r3bl_tui`) + `ratatui-image` + `tui-realm` + `iocraft` Interactive Chat & 10-Tab TUI (`src/agent_runtime/chat_ui.rs`, `src/tui/`) with `KeyEventKind::Press` Double-Keystroke Fix, Full Line-Editor, Live Pause/Resume/Edit-While-Working, Mouse Hover & Git-Engineer `.git/config` Context (`src/tools/git_engineer.rs`) + 13-IDE External Plugin Host Protocol (`src/tools/ide_plugin_host.rs`)**, **Multi-Agent Orchestration Resilience & Cross-Chat Resume (`src/agent_runtime/orchestration_resume.rs`)**, **Dynamic `.git/config` Repo Auto-Detection (Zero `neo_dir`) & `<neo>/agents/rules/` + `<neo>/agents/agent_instructions.md` External Staging + Non-Destructive Prompt Architecture Router (`toksi refactor-router`)**, **Strict Pure 7-Bit ASCII Guard & `MojibakeLab`/`python-ftfy` Auto-Healer (`toksi heal-encoding`)**, Policy-as-Code (`opa`/`cedar`) & SPIFFE IAM attestation (`spire`), Guardrails/`llm-guard`/`openai-guardrails-python`/`safeagents`/`garak`/`PurpleLlama`/Anonymizer + **ATR (`agent-threat-rules`)** + **`model_spec`**, Zero-Plaintext Blind Signature Engine (`src/security/` `BLAKE3` + `sparse::DFA`) & Native Git/PR Validator (`src/validator/` + 12-DB `UpstreamDeduplicator`), ADR layer (`adr.rs`), Multi-Surface SAST (standalone/MCP/`<toksi-siagent>`), Expanded Plugin Mounts (`src/plugins/`), Bidirectional DB/TOML BYOK Sync, 5-backend + **Docker Compose & `sbxenv.yaml`/Kit v3** Isolated Agent Sandbox (`sandbox.rs`, `docker/compose.test.yaml`), `CommandGuard` Bypass Kill-Switch (`guard.rs`), live-safe `docker/Dockerfile.test` + `docker/compose.test.yaml` + `toksi test --docker|--sandbox`, externalized Zero-Pollution `<skills>` (`Navegos/skills` + `skilldoctor` + `agentskills` YAML healer + `upskill` evals + `skill-scanner` + `defenseclaw`), Temporal/Restate/LangGraph durable workflow state & plan checkpoint/resume (`.toksi/projectmem/plan_checkpoints/`), hybrid graph-vector memory (`Cognee`/`Graphiti`/`Qdrant`/`Lance`/`arrow-rs`/`Chroma`/`Milvus`/`Letta`/`funes`/`ossie*`/`auto-ontology`/`liteparse`), **30+** local/Rust inference & serving engines (`hyperflare-inference`, `oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `mistral.rs`, `candle`, `dynamo`, `ollama`, `llama.cpp`, `whisper.cpp`, `stable-diffusion.cpp`, `triton`, `DeepSpeed-MII`), unified **11-Pillar AI/ML Lifecycle (`src/training/` `toksi train|serve|eval|dataset|compress` + `/voice` + `AutoFA`/`deepeval`/`ragas`/`kvpress`/`bitsandbytes`/`Megatron-Energon`)**, **Distributed Cluster Stack (`src/cluster/` `toksi cluster`: K8s/Minikube/Kompose/Kops/Helm/Autoscaler/NPD, NVCRE/`NVSentinel`/`aicr`/Knavigator/DeepOps, Kubeflow/`Run`, Spark/`cudf-spark`/`cuPhoton`, `Personal-AI-Router` LAN mesh)**, **OpenTelemetry Rust LLMOps (`src/telemetry/otel.rs` `gen_ai.*`)**, unified **35-component `<toksi-siagent>`** with **22 Orchestrator Modes across 7 Cognitive Domains (`[toksi_siagent_router.*]`)** & Multi-Agent Workforce (`OMA`, `swarms-rs`, `ChatDev`, `agent-orchestrator`, `OpenSpec` `/spec`) + `Zoo-Code`, **29 universal slash commands**, separated per-service DBs (`~/.toksi/services/*.db`), and decoupled **10-tab** async telemetry cockpit across **502 unique mandated vaults (`[DEEP-RESEARCH-DONE]` --- `493` Wave 20 + `2` Wave 21 + `7` Wave 22: `ratatui`, `ratatui-image`, `tui-realm`, `iocraft`, `r3bl-open-core`, `MojibakeLab`, `python-ftfy`)**.

---

## 1. System Overview & Architectural Mandates

`toksi` is a pure Safe Rust (Edition 2024, `#![deny(unsafe_code)]`) **35-component `<toksi-siagent>` Super Intelligence Runtime (`RESEARCHPLAN.md` Section 0.2)** combining headroom proxy (`live_zone` byte-range surgery + `CcrStore`), token compression engine, native AI gateway, Dedicated Multi-Process Supervisor (`toksi _worker <role>`), multi-protocol agent wire platform (`MCP`/`A2A`/`ACP`/`ANP`/`QUIC`), 3-repository security/advisory/skills governance suite, isolated sandboxed agent runtime, model training/serving orchestrator, pure 7-bit ASCII mojibake healer, and standalone coding CLI. It sits transparently between local AI coding assistants (`antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude`, `cursor`, `codex`, `opencode`, `grok`, and 65/90+ ecosystem connectors across Categories 1--5 + **30+** local & Rust-native inference engines) and configured BYOK providers (including Google Cloud ADC Vertex endpoints) or dedicated tool worker processes (`rtk`, `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `security`, `advisory`, `skills`, `sandbox`).

### Core Architectural Mandates:
1. **100% Native In-Tree Rust (`#![deny(unsafe_code)]`) & Zero Binary Bloat**: Complete elimination of `npm`, `pip`, `uv`, `python`, `node`, and `set_var`/`LockFileEx`/`setsid` unsafe blocks; unavoidable Win32 FFI and raw C-ABI Tree-sitter shims live in isolated workspace leaf crates (`crates/toksi-sys`, `crates/toksi-grammars`). Zero missing code across all **502 unique reference vaults** (`493` Wave 20 + `2` Wave 21 + `7` Wave 22 reconciled across Sections `2.1`--`2.18`). Skills and rules Markdown/YAML/JSON payloads are NEVER embedded via `include_str!` inside `crates/toksi/` --- stage externally in `<neo>/agents/`, `<neo>/agents/rules/`, `<neo>/agents/agent_instructions.md`, `Navegos/skills`, and `~/.toksi/user-skills`.
2. **Continuous Reference Vault Re-Audit Across 3-Repository Ecosystem (All 502 Unique Vaults = 294 Waves 1--18 + 138 Wave 19 + 61 Wave 20 + 2 Wave 21 + 7 Wave 22 --- `[DEEP-RESEARCH-DONE]`)**: Enforce `<enforce>` security across `<neo>` (`Navegos/neo`), `<advisory>` (`Navegos/advisory`), and `<skills>` (`Navegos/skills`), synthesizing all **502 unique vaults** while tracking true source completeness under the **12-State Progress Taxonomy**.
3. **Single `NetworkService` Network Manager (`_worker network-manager`), DirectWire IPC & On-Demand `<toksi-siagent>` (`PHASE_19_2`, `PHASE_20_2`, `PHASE_21_1`)**: `toksi daemon start|stop|restart|heal|status|dashboard|service <name> <action>` spawns ONE dedicated `toksi _worker network-manager` owning all active daemon sockets (`headroom:8787`, `grok:8788`, `copilot-vscode:8789`, `copilot-cli:8790`, `protocol-bus:9098`, `gateway:9099`, `byok-gateway:18787`)---preventing Windows `os error 10048`---and routes traffic over Named Pipes (`//./pipe/toksi-*`) / Unix Domain Sockets (`~/.toksi/run/*.sock`) + `DirectWireBus` (`src/ipc/`) to dedicated service and tool worker subprocesses (`codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`, `cluster`). `<toksi-siagent>` (`toksi chat|run|exec|swarm|orchestrator`) launches **on demand** outside `daemon start` and connects to running daemon services; `:9100` Web Cockpit remains detached (`toksi dashboard` / `toksi daemon dashboard`).
4. **Configurable Global Runtime (`~/.toksi/config.toml` -> `src/config/mod.rs`) & Split DBs (`~/.toksi/toksi.db` + `~/.toksi/services/*.db`)**: Controls ports (`headroom=8787`, `grok_oauth=8788`, `copilot_vscode=8789`, `copilot_cli=8790`, `protocol_bus=9098`, `gateway=9099`, `dashboard=9100`, `byok_gateway=18787`), `[repositories]`, `[supervisor]`, `[testing]`, `[auth.gcp_adc]`, `[optimization]`, `[security]`, `[plugins]`, `[checkpoint]`, `[toksi_siagent_router.*]`, default `--agents`, and multi-instance `[[byok_profiles]]` synchronized with `~/.toksi/toksi.db` while live telemetry lives in `~/.toksi/services/*.db`.
5. **Live-Safe Sandboxed, Docker & Compose/Kit v3 Isolation (`PHASE_19_3`, `PHASE_20_3`)**: `cargo test` unit suites isolate `TOKSI_HOME` tempdirs and `:0` ephemeral ports; `docker/Dockerfile.test`, `docker/compose.test.yaml`, and declarative `sbxenv.yaml`/Kit v3 manifests power `toksi test --docker|--sandbox` & `toksi sandbox compose|sbxenv` while honoring `.toksi/test-baseline.json` (`35+` suites tracked).
6. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**: Verify external binaries in system `PATH` cleanly (`rust-analyzer`, `gopls`, `clang-tidy`, `gcloud`, `docker`, `sbx`, `whisper-cli`, `unsloth`, agent CLIs) without hardcoding paths. Missing binaries remain grayed out / disabled with explicit user remediation notices and zero silent failure.
7. **Universal Forward-Slash (`/`) Path Standard**: Normalize internal paths with `/` on Windows, Linux, and macOS (escaped `\\` permitted exclusively inside `cmd.exe` command strings in JSON5/TOML configs).
8. **Strict Scope Split --- Home Config vs. Service DBs vs. Repo-Only Tool Stores vs. External Skills**: `~/.toksi/` holds `toksi.db` (config), `config.toml`, `services/*.db` (telemetry/quotas/otel/cluster), `run/*.sock`, and optional `~/.toksi/{plugins,user-skills}/`. ALL workspace intelligence stores (`codegraph`, `projectmem`, **`ctxmode`**, `icm`, **`aimemory`**, `rtk`, `headroom`, `security`, repo `plugins`) live strictly inside `<repo>/.toksi/<tool>/` where `<repo>/.git/` exists.
9. **Workspace Root Consolidation, `ctxmode`/`aimemory` Migration & Safe Plan Checkpoints**: Auto-merge legacy dotdirs (`.context-mode/`, `<repo>/.toksi/context/`, `.ai-memory/`, `<repo>/.toksi/ai-memory/`) and nested crate folders into `<repo>/.toksi/<tool>/` row-by-row; persist atomic checkpoints in `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` for `/resume`.
10. **Mandatory `ctxmode` & `aimemory` Renames (`PHASE_19_1`), `<enforce>` + `PurpleLlama` + `ATR`, & Unified Plugins (`src/plugins/`)**: Uniform `<tool>-hook <agent>` (`ctxmode-hook <agent>`, `aimemory-hook <agent>`), normalized MCP IDs (`projectmem`, `ctxmode`, `codegraph`, `icm`, `aimemory`, `rtk`, `security`, `advisory`, `skills`, `sandbox`), `NeMo-Relay-Plugins` (`relay-plugin.toml`), `defenseclaw` + `skill-scanner` + `ATR` + `openai-guardrails-python` + `PurpleLlama`, Zero-Plaintext Blind Engine + Native Validator.
11. **Multi-Protocol Standardization & 45-Language AST/SAST Intelligence**: `MCP`, `A2A`, `ACP`, `ANP` (`did_wba`), `stdio`, `SSE`, `HTTP`, `QUIC` on `:9098` + 45 Tree-sitter language IDs + `.gitignore`/`.ignore` dynamic index purge.
12. **Zero Paid-Tier Restrictions, Concurrent `agy` + Google Cloud ADC (`AGY_ADC_AUTH=true`), Auto `/v1`, 5 Effort Tiers & Sampling Matrix**: Free/OAuth/ADC/BYOK route freely; `Max|XHigh|High|Medium|Low` efforts (default `Medium`) + strict per-model `(temperature, top_p, top_k)` compatibility filtering + automatic `/v1` base URL completion + concurrent standard `agy` and `AGY_ADC_AUTH=true` Vertex ADC.
13. **Real `<toksi-siagent>` Subprocess (`_worker siagent`), 22 Orchestrator Modes (`[toksi_siagent_router.*]`), Distributed Cluster Stack (`src/cluster/`), & 29 Slash Commands**: Kill local stubs in `loop_engine.rs:173-176, 249-251`; run real `_worker siagent` across all 22 modes / 7 Cognitive Domains (`orchestrate_route`..`document_log`) and **29 universal slash commands** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/mode`, `/sampling`, `/router`, `/projectmem`, `/ctxmode`, `/codegraph`, `/icm`, `/aimemory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, `/workforce`, `/spec`, `/dataset`, `/train`, `/eval`, `/compress`, `/serve`, `/cluster`, `/otel`, `/voice`).
14. **Ground-Truth OpenTelemetry (`gen_ai.*`), Zero-CLI-Spawn 10-Tab UI**: Zero synthetic stubs, zero `"opencode"` UI hardcoding/spawning, decoupled **10-Tab** TUI & `:9100` Cockpit (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`).

### 2.1 Telemetry & Cluster Modules (Wave 20)
- `src/cluster/` (`PHASE_20_5`): `topology.rs`, `kubernetes.rs`, `readiness.rs` (NVCRE + `NVSentinel` + `aicr`), `kubeflow.rs`, `spark.rs`, `lan_pair.rs`, `otel_pipeline.rs`.
- `src/telemetry/otel.rs` (`PHASE_20_6`): `opentelemetry-rust` OTLP + `gen_ai.*` semantic spans buffered to `~/.toksi/services/otel.db`.
- `docker/compose.test.yaml` + `sbxenv.yaml`/Kit v3 (`PHASE_20_3`) alongside `docker/Dockerfile.test` (`PHASE_19_3`).

---

## 2. Unified 8-Port Multi-Process Network Topology (Overridable via `~/.toksi/config.toml`)

All daemon listening ports (`:8787..:18787`) are owned by ONE dedicated **`NetworkService` Network Manager (`toksi _worker network-manager`)** (`PHASE_21_1`, preventing `os error 10048`) routing via Named Pipes / UDS `DirectWireBus` to isolated service & tool worker subprocesses, while `<toksi-siagent>` launches on demand and `:9100` Web Cockpit runs detached:

| Port / Worker Role | Service / Subsystem | Transport / IPC | Protocol | Isolation Guarantee |
|---|---|---|---|---|
| **Port 8787** (`_worker network-manager` -> `headroom`) | Streaming Headroom Proxy | TCP / HTTP/1.1 & HTTP/2 / SSE + UDS/Pipe | HTTP chunked, SSE, ConnectRPC passthrough | Single-bind socket manager dispatching via IPC to isolated `headroom` worker (`live_zone` byte surgery, effort scaling, & `CcrStore`). |
| **Port 8788** (`_worker network-manager` -> `grok`) | Grok OAuth Proxy (`grok_daemon.rs`) | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked, SSE | Isolated xAI Grok OAuth proxy service (NOT generic BYOK). |
| **Port 8789** (`_worker network-manager` -> `copilot-vscode`) | GitHub Copilot VS Code Proxy | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked | Isolated proxy service (`DEFAULT_COPILOT_PROXY_PORT`). |
| **Port 8790** (`_worker network-manager` -> `copilot-cli`) | GitHub Copilot CLI Proxy | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked | Isolated proxy service (`DEFAULT_COPILOT_CLI_PROXY_PORT`). |
| **Port 9098** (`_worker network-manager` -> `protocol-bus`) | **Unified Protocol Bus & 13-IDE Host** | TCP / HTTP / SSE / **QUIC** + UDS/Pipe | `MCP`, `A2A` v1.0, `ACP`, **`ANP` (`did_wba`)**, 13-IDE host, checkpoint events | Isolated agent control & IDE bus passing payloads through `HeadroomEffortCompressor` (`Max|XHigh|High|Medium|Low`) & `<enforce>` across all **502** unique vaults & `Plugins`. |
| **Port 9099** (`_worker network-manager` -> `gateway`) | Enterprise AI Gateway & Drop-in SDKs | TCP / HTTP / SSE + UDS/Pipe | `/v1/*`, `/v1/security/scan`, `/v1beta/models/*`, `/metrics` | Isolated inference pool enforcing `HeadroomEffortCompressor`, sampling compatibility matrix, auto `/v1`, Vertex ADC, & failover. |
| **Port 9100** (`dashboard`, detached) | Web Telemetry & Control Dashboard Cockpit | TCP / HTTP / SSE / WS + UDS/Pipe | Browser 10-Tab HTML/CSS/JS, `tokview`, `Quotas`, Cluster/OTel controls | Detached process launched only via `toksi dashboard` or `toksi daemon dashboard`. Zero inference contention & never spawned on `daemon start`. |
| **Port 18787** (`_worker network-manager` -> `byok-gateway`) | BYOK & Vertex ADC Gateway (`byok_gateway.rs`) | TCP / HTTP / SSE + UDS/Pipe | HTTP/1.1 chunked, SSE | Supervised multi-instance BYOK, concurrent `agy` + ADC, & 30+ local engines through `HeadroomEffortCompressor`. |
| **On-Demand & Tool Workers** | On-demand `<toksi-siagent>`, `_worker codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`, `cluster` | Named Pipe / UDS (`DirectWireBus`, `iggy` framing) | In-band RPC + health pings + kill-switch broadcast | Crash-isolated per-subsystem subprocesses; `<toksi-siagent>` connects on demand without auto-spawning on `daemon start`. |

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
docker/                       # Wave 19.3 / 20.3 Live-Safe Container Test Isolation
|-- Dockerfile.test
`-- compose.test.yaml

crates/toksi/src/
|-- main.rs                   # Unified CLI entrypoint (#![deny(unsafe_code)]) & full usage help
|-- lib.rs                    # Public crate library API (#![deny(unsafe_code)])
|
|-- ipc/                      # Named Pipe / UDS Direct-Wire IPC & Single NetworkService Manager
|   |-- mod.rs, bus.rs, named_pipe.rs, uds.rs, iggy_frame.rs, quic.rs, network_manager.rs
|
|-- training/                 # Planned Wave 19.6 Model Training/Quantization/Eval Orchestration
|   |-- mod.rs, recipes.rs, evals.rs
|
|-- tools/                    # Multi-protocol tool bus, DirectWireBus & 13-IDE Plugin Host (`ide_plugin_host.rs`)
|   |-- mod.rs, direct_wire.rs, multi_protocol.rs, autoindex.rs, ide_plugin_host.rs
|
|-- projectmem/               # Pure Rust port of ProjectMem --- strict <repo>/.toksi/projectmem/ + plan_checkpoints/
|   |-- mod.rs, schema.rs, store.rs, summary.rs, precheck.rs, redaction.rs, heatmap.rs, mcp.rs, plan_checkpoint.rs, wiki_sync.rs
|
|-- icm/                      # Instant Context Memory SQLite FTS5 BM25 + temporal/fusion/memoir (<repo>/.toksi/icm/)
|   |-- mod.rs, store.rs, schema.rs, recall.rs, mcp.rs
|
|-- aimemory/                 # Hybrid Graph-Vector Memory: OKF v0.2 wiki, handoff, Lance (arrow-rs), Qdrant, Milvus, Cognee, Graphiti, Letta, funes, TEI (<repo>/.toksi/ai-memory/)
|   |-- mod.rs, wiki.rs, handoff.rs, tiered.rs, mem0_bridge.rs, lance_store.rs, qdrant_store.rs, milvus_adapter.rs, cognee_graph.rs, temporal_graph.rs, letta_memgpt.rs, embeddings.rs, retriever.rs
|
|-- codegraph/                # 45-lang Tree-sitter AST, SCIP, ast-grep CST, jcodemunch, repomix, OpenWiki, pmat TDG, tiktoken BPE (<repo>/.toksi/codegraph/)
|   |-- mod.rs, kernel/, scip.rs, ast_grep.rs, jcodemunch.rs, repomix.rs, openwiki.rs, bpe.rs, embeddings.rs, reranker.rs, walker.rs, graph.rs, cache.rs, mcp.rs
|
|-- contextmode/              # Subprocess sandbox, policy/journal, SQLite FTS5 search, WFST text_norm (Wave 19.1 -> `ctxmode`, <repo>/.toksi/ctxmode/)
|   |-- mod.rs, sandbox.rs, filter.rs, fts.rs, text_norm.rs, fetch.rs, mcp.rs
|
|-- security/                 # OPA/Cedar/SPIRE IAM, Guardrails/LLM-Guard/OpenAI-Guardrails/ATR/model_spec, Zero-plaintext BLAKE3+sparse::DFA, ADR, defenseclaw, skill-scanner, 45-lang SAST, sage, OSV/EPSS, ascii_guard (<repo>/.toksi/security/)
|   |-- mod.rs, policy_opa.rs, policy_cedar.rs, iam_attestation.rs, guardrails_validators.rs, llm_guard.rs, safeagents.rs, garak_redteam.rs,
|   |-- schema.rs, polyglot.rs, blind_compiler.rs, scanner.rs, advisory_client.rs, secure_export.rs, adr.rs,
|   |-- analyzer.rs, defenseclaw.rs, hookguard.rs, skill_scanner.rs, cel_decision.rs, pipeline_taint.rs, bytecode_audit.rs,
|   `-- sarif.rs, audit.rs, yara_mcp.rs, sage_rules.rs, skill_audit.rs, osv.rs, epss.rs, semgrep_engine.rs, external_analyzers.rs, ascii_guard.rs
|
|-- validator/                # Native CVE 5.1 / OSV 1.7.4 validation, FP pruning, 3-tree shards & REST PR driver (Navegos/advisory)
|   `-- mod.rs, validate.rs, fp_pruner.rs, shard_builder.rs, git_pr_driver.rs
|
|-- plugins/                  # Toksi Unified Plugin System (<enforce> scanned: agent_feature, cli_cmd, tool, chat_cmd, wire_hook, protocols, a2a/anp)
|   |-- mod.rs, manifest.rs, loader.rs, hooks.rs, slash_router.rs, cli_bridge.rs, analyzer_plugin.rs
|
|-- agent_runtime/            # 35-Component <toksi-siagent> CLI, Headroom-Central r3bl_tui Chat TUI, Resume Engine, Workforce & Swarm wired off-the-back via DirectWireBus & :8787..:8790/:9098/:9099/:18787 to all 502 vaults
|   |-- mod.rs, types.rs, loop_engine.rs, chat_ui.rs, orchestration_resume.rs, prompt_router.rs, context.rs, edit_tools.rs, planning.rs, orchestrator.rs, reflection.rs, workflow_state.rs, durable_journal.rs, world_model.rs,
|   `-- sandbox.rs, openshell.rs, sandbox_e2b.rs, cluster_exec.rs, rl.rs, codemode.rs, session.rs, slash.rs, event_log.rs, cron.rs, checkpoint.rs, acp.rs, guardrails.rs, graph_checkpoint.rs, deep_planning.rs, fleet.rs, modes.rs, zeroclaw_core.rs, swarm/
|
|-- config/                   # Global ~/.toksi/config.toml loader (no neo_dir), .git/config auto-detect, schema validator, hot-reload, [repositories], [supervisor], [testing], [auth.gcp_adc] & DB sync
|   |-- mod.rs
|
|-- headroom/                 # Shared HeadroomEffortCompressor (Low/Med/High: live_zone, CcrStore, SmartCrusher, TokenEconomizer, agent-opt, mcp_pruner, tiktoken BPE) + Copilot->Headroom/Grok/BYOK proxies
|   |-- mod.rs, copilot_daemon.rs, grok_daemon.rs, byok_gateway.rs, proxy.rs, effort.rs, compression/, pruner/, cache_align.rs, adaptive_mem.rs
|
|-- gateway/                  # Enterprise AI Gateway (:9099), Protocol Bus (:9098), Vertex ADC (:18787/:9099, vertex_rust_sdk), Pingora pooling, 30+ Local/Rust Engines & Failover Engine --- all enforcing HeadroomEffortCompressor
|   |-- mod.rs, reconnect.rs, local_engines.rs, inference_bridge.rs, inference_mistral.rs, inference_candle.rs, disaggregated_serving.rs, cost_quality_router.rs, quant_catalog.rs, vertex_adc.rs, gemini_wire.rs, effort_router.rs, providers/, agent_cli/, cascade.rs, keys.rs, loadbalancer.rs, routing.rs,
|   |-- governance.rs, semantic_cache.rs, normalizer/, compat/, circuit_breaker.rs, proxy.rs
|
|-- telemetry/                # Ground-Truth Session Metrics, tokview/Langfuse/LangWatch/OpenInference spans, tokenspeed, GPU/cudarc/vmon probes & lighteval
|   |-- mod.rs, prometheus.rs, collector.rs, tokentracker.rs, tokview.rs, traces.rs, openinference.rs, tokenspeed.rs, gpu_monitor.rs, evals.rs, cost.rs, tracker.rs, cache_analytics.rs
|
|-- rtk/                      # 100+ filters, ANSI stripper, SQLite Tracker (rtk gain), tee recovery, discover, learn, slash
|   |-- mod.rs, engine.rs, filters/, ansi.rs, hook.rs, tracker.rs, tee.rs, discover.rs, learn.rs, slash.rs
|
|-- rules/                    # Externalized <skills> Sync (<neo>/agents/, <neo>/agents/rules/, <neo>/agents/agent_instructions.md, zero binary bloat), <enforce> Guard, skilldoctor + agentskills + upskill Healer/Eval
|   |-- mod.rs, sync.rs, skills_repo.rs, skill_validator.rs, skills_catalog.rs, principles.rs, caveman.rs, ponytail.rs
|
|-- env_injector/             # Non-destructive OS environment injection & legacy binary healing
|   |-- mod.rs, windows.rs, unix.rs, teardown.rs, heal.rs
|
|-- dashboard/                # Background Web Telemetry Cockpit on Port 9100 (10-tab SPA + SSE + REST + worker/training/cluster controls)
|   |-- mod.rs, state.rs, sse.rs, api.rs, assets/
|
|-- tui/                      # Non-blocking async 10-tab mouse TUI (r3bl_tui / ratatui + ratatui-image + crossterm KeyEventKind::Press)
|   |-- mod.rs, app.rs, views/, widgets/
|
|-- agents/                   # Configured/selected agents & Category 1-5 connectors (incl. opencode, Zoo-Code) + direct-wire bus
|   |-- mod.rs, manager.rs, open_agents.rs, antigravity.rs, copilot.rs, claude.rs, codex.rs, cursor.rs, grok.rs, opencode.rs, zoo_code.rs, opt.rs, mcp_adapters.rs
|
|-- keyring/                  # Cross-platform secure credential manager + Google Cloud ADC (AGY_ADC_AUTH=true)
|   |-- mod.rs, adc.rs, os_windows.rs, os_macos.rs, os_linux.rs, envelope.rs
|
|-- db/                       # Persistent SQLite configuration database (~/.toksi/toksi.db) + byok_profiles + plugins_registry + blind_signatures + advisory_embargoes + adr_audit_events + supervisor_workers
|   |-- mod.rs, schema.rs, operations.rs, migrate.rs
|
|-- util/                     # Forward-slash path normalization, PATH validation mandate, mojibake/ftfy healer, exec, parsers
|   |-- paths.rs, env_path.rs, mojibake.rs, ftfy.rs, exec.rs, jsonc.rs, toml.rs, colors.rs
|
`-- commands/                 # Native CLI command dispatcher
    |-- daemon.rs, worker.rs, proxy.rs, config.rs, agent_runtime.rs, orchestrator.rs, train.rs, serve.rs, eval.rs, dataset.rs, compress.rs, cluster.rs, otel.rs, guard.rs, compile_blind.rs, advisory.rs, skills.rs, sandbox.rs, security.rs, plugin.rs, test_runner.rs, auth.rs, db.rs, agent.rs, tui.rs, dashboard.rs, mem.rs, graph.rs, ctx.rs, rtk.rs, rules.rs, doctor.rs, heal_encoding.rs, refactor_router.rs, mod.rs
```

---

## 6. Detailed Execution Roadmap (Phases 15.1 -- 16.3)

All tasks are critical and must be executed with strict technical precision. *(8-State Badges synchronized in Wave 20 / v7.0.)*

### Phase 15.1: Workspace Topology, Forward-Slash Normalization & Standards `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Universal forward-slash `/` paths, PATH validation mandate, user-home DB storage, single root `.toksi` consolidation, standardized hook/MCP naming (`ctxmode-hook <agent>` in `PHASE_19_1`), and root/skills Zero-Pollution hygiene (`PHASE_19_4`).
- **Targets**:
  * `crates/toksi/src/util/paths.rs`: Implement `normalize_path(p: &str) -> String` converting all backward slashes `\` to `/` across all OSes. Handle drive letters (`C:/`), UNC paths (`//server`), and tilde expansions. Preserve escaped `\\` exclusively for cmd.exe command strings in JSON5/TOML configs.
  * `crates/toksi/src/util/env_path.rs`: Implement `check_executable_in_path(cmd: &str) -> Result<(), String>`. Verify required binaries exist in `PATH` without encoding absolute hardcoded paths.
  * `crates/toksi/src/db/mod.rs` & `operations.rs`: Relocate default SQLite DB to `~/.toksi/toksi.db` (`dirs::home_dir()`). Auto-migrate legacy `LOCALAPPDATA/toksi` data on startup.
  * `crates/toksi/src/commands/init.rs` & `doctor.rs`: Single root `.toksi` consolidation: discover subfolder `.toksi` directories, merge rows into root `.toksi/toksi.db`, `memory.db`, `symbols.db` without data loss, and delete nested `.toksi` folders.
  * `crates/toksi/src/agents/hooks.rs` & `mcp_adapters.rs`: Standardize hook syntax to `<tool>-hook <agent>` (`codegraph-hook agy`, `projectmem-hook agy`, `ctxmode-hook agy`, `rtk-hook agy`), normalize MCP identifiers (`projectmem`, `ctxmode`, `codegraph`), and map MCP settings to `skills/<tool>`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.2: Unified 8-Port Daemon, `config.toml`, & Copilot Supervision `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Supervise all dedicated endpoints (`:8787`, `:8788` Grok, `:8789`/`:8790` Copilot, `:9098` Protocol Bus `MCP`/`A2A`/`ACP`/`ANP`/`SSE`/`QUIC`, `:9099` Gateway, `:18787` BYOK --- all overridable via `~/.toksi/config.toml` `[ports]`) via isolated `toksi _worker <role>` subprocesses (`PHASE_19_2`), with detached `:9100` launched strictly by `toksi dashboard` / `toksi daemon dashboard` (`PHASE_20_2`).
- **Targets**:
  * `crates/toksi/src/commands/daemon.rs`: Master daemon supervisor starting dedicated subprocesses per port (`8787..18787`), `<toksi-siagent>`, and core tools.
  * `crates/toksi/src/headroom/copilot_daemon.rs`: Dual-port supervisor managing 8789 and 8790 with live `/livez` health probes and collision prevention.
  * `crates/toksi/src/gateway/mod.rs` & `dashboard/mod.rs`: Guarantee thread, process, and socket buffer isolation between Port 9099 (inference) and Port 9100 (dashboard).
  * `crates/toksi/src/env_injector/teardown.rs`: Clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix environment on daemon shutdown.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.3: Smart Agent CLI Providers & Open BYOK Quota Telemetry `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Register Local Agent CLIs (`AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, `GrokCli`) and ecosystem bridges with smart background launch; track read-only commercial/free quota windows with **zero paid-tier disqualification**.
- **Targets**: `crates/toksi/src/gateway/agent_cli/mod.rs`, `tier_check.rs`, `providers/aggregators/bifrost.rs`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Expose MCP, A2A v1.0, ACP v2.0, **`ANP` (`did_wba`)**, stdio, SSE, HTTP, and **`QUIC`** across all tools; integrate `.gitignore` / `.ignore` and `<repo>`-only scoping.
- **Targets**: `crates/toksi/src/tools/multi_protocol.rs` (`rtk`, `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `A2aTaskStore`, `SessionActorQueue`), `codegraph/{walker,cache}.rs`, `contextmode/fts.rs`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Ground-truth session token accounting from live SSE streams; network reconnection auto-healing watchdog; configured-only universal rules sync.
- **Targets**: `crates/toksi/src/telemetry/{tokentracker,collector}.rs`, `gateway/reconnect.rs`, `rules/sync.rs`, `main.rs` (`help_text()`).

### Phase 15.6: Advanced Vault Intelligence Engines Consolidation `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Consolidate core code-intelligence vaults into pure Rust with real end-to-end wiring (`TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `litellm`, `TokenEconomizer`, `agent-opt`).

### Phase 16.1: High-Performance Async 10-Tab Mouse-Capable TUI `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Full mouse interaction and non-blocking asynchronous event loop eliminating terminal hangs, with exact **10-Tab** parity (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`), extended for `_worker` subprocess controls, `/workforce`, `/spec`, cluster, OTel, and `toksi train|serve|eval`.
- **Targets**: `crates/toksi/src/tui/{mod.rs,app.rs,views/*}`.

### Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Isolated, detached background **10-tab** web UI on Port `9100` (`toksi dashboard` / `toksi daemon dashboard`) with zero synthetic stubs, zero `"opencode"` UI hardcoding, zero CLI spawning, and separated `~/.toksi/services/*.db` telemetry.
- **Targets**: `crates/toksi/src/dashboard/{mod.rs,api.rs,state.rs,sse.rs}` & `assets/{index.html,style.css,app.js}`.

### Phase 16.3: Open Agents Directory Protocols & README Verification `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`
- **Objectives**: Support `.agents/` (incl. `.agents/mcp -> skills/<tool>`) and `agents-structures` vendor layouts; configured/user-selected auto-install rules; Wave 20 `README.md` & full runtime CLI parity synchronized.
- **Targets**: `crates/toksi/src/agents/{open_agents.rs,manager.rs,mcp_adapters.rs}`, `crates/toksi/README.md`.

---

## 7. Wave 17 & Wave 18 Execution Phases (`PHASE_17_1` -- `PHASE_18_6`)

*(Detailed worker prompts in [`.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`](../../.agents/brain/INDEX_AND_EXECUTION_GUIDE.md). Executed with targeted verification + `cargo check` & `cargo clippy -- -D warnings`.)*

- **`PHASE_17_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **`~/.toksi/config.toml`, Edition 2024 `#![deny(unsafe_code)]`, & Zero Paid-Tier Restrictions**: `src/config/mod.rs`, `FreeTierDisqualified` removal, `crates/toksi-sys` isolation, `#![deny(unsafe_code)]`.
- **`PHASE_17_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Multi-Model `<byok, model, effort>` DB, Keyring Fix, `headroom-core` `live_zone`, & Universal Headroom Effort Pass**: `db/{schema,operations,migrate}.rs` (`byok_profiles`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`), `gemini:antigravity` keyring, shared `HeadroomEffortCompressor` across all 8 ports.
- **`PHASE_17_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **45 Tree-sitter Languages, `<repo>/.toksi/<tool>` Repo-Only Storage, Monorepo Partitioning, & `.ignore` Purge**: `crates/toksi-grammars` (`language_vue()`, `language_scss()`, `language_dockerfile()` live in `langs.rs`), strict `.git`-root `commands/index.rs`, `ast-grep`/`scip`/`jcodemunch`/`repomix`.
- **`PHASE_17_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Deep Vault Parity, 65/90+ Ecosystem & Local Engines, & Hook/MCP Cleanup**: `ctx.rs` & `runmcp.rs` fixed; `PHASE_19_1` `ctxmode-hook` rename and `PHASE_19_2` subprocess Direct-Wire IPC completed.
- **`PHASE_17_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Unified `<toksi-siagent>` Coding CLI (`src/agent_runtime/`), `DirectWireBus`, & Slash Commands**: Local stubs killed, `_worker siagent` wired, **29** core `< 5ms` slash commands live.
- **`PHASE_17_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Ground-Truth SSE Accounting, Gateway Real Dispatch, OS Reconnect Watchdog, & ConnectRPC Passthrough**.
- **`PHASE_17_7`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Async 10-Tab Mouse TUI, Live Web Cockpit (`:9100`), Incremental `.toksi/test-baseline.json` Harness (`43` suites), CLI `--help`, & Doc Sync**: live-safe Docker/Compose/sandbox test runner completed in `PHASE_19_3`/`20_3`.
- **`PHASE_18_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **294 Waves 1--18 Vaults Synthesis**: `lance_store.rs`, `qdrant_store.rs`, `cognee_graph.rs`, `temporal_graph.rs`, `openwiki.rs`, `tokview.rs`, `openinference.rs`, `tokenspeed.rs`, `gpu_monitor.rs`, `src/plugins/`, 26 local/Rust engines, 140 compute/serving vaults.
- **`PHASE_18_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **`<enforce>` 3-Repo Security, Zero-Bloat Externalized `<skills>`, Unified Plugin System (`src/plugins/`), `defenseclaw`, `skill-scanner`, `skilldoctor` + `agentskills` Healer, Zero-Plaintext Blind Engine (`src/security/`), Native Validator (`src/validator/`), & 45-Lang SAST**.
- **`PHASE_18_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Google Cloud ADC (`AGY_ADC_AUTH=true`), Thinking-Effort Optimization (`Max..Low`), & Safe Plan Checkpoints (`/resume`)**.
- **`PHASE_18_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Legacy `.projectmem/` Repair, Documentation Integrity & Final Workspace Gate**: hygiene checks & `.toksi/projectmem/*` migration fixed.
- **`PHASE_18_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Cross-Agent Source Unification, ADR Layer (`src/security/adr.rs`), 5-Backend Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`), & `CommandGuard` Bypass Kill-Switch (`src/commands/guard.rs`)**.
- **`PHASE_18_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **35-Component `<toksi-siagent>` Super Intelligence Architecture & Full Inference/Compute Stack Consolidation**.

---

## 8. Wave 19 / v6.0 Execution Phases (`PHASE_19_1` -- `PHASE_19_6`)

- **`PHASE_19_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Mandatory `ctxmode` & `aimemory` Tool Rename Everywhere**: Replaced legacy `context-mode` & `ai-memory` references in `crates/toksi/src/` (`tools/contextmode.rs`, `agents/{manager,mcp_adapters,copilot,antigravity,hooks,grok}.rs`, `commands/{ctx,runmcp,index,init,doctor,doctor_probe}.rs`, `util/{instructions,versions}.rs`, `main.rs`) with **`ctxmode`** & **`aimemory`** (`toksi ctxmode|aimemory`, `/ctxmode`, `/aimemory`, `ctxmode-hook <agent>`, `aimemory-hook <agent>`, MCP keys `"ctxmode"`/`"aimemory"`, `skills/{ctxmode,aimemory}/`, `<repo>/.toksi/{ctxmode,aimemory}/`), auto-migrating legacy `.context-mode/`, `<repo>/.toksi/context/`, `.ai-memory/`, and `.toksi/ai-memory/` stores cleanly.
- **`PHASE_19_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Dedicated Multi-Process Supervisor (`toksi _worker <role>`) & Direct-Wire IPC (`src/ipc/`)**: Split `src/commands/daemon.rs` in-process Tokio tasks into isolated OS worker subprocesses per Network Port (`headroom:8787`, `grok:8788`, `copilot-vscode:8789`, `copilot-cli:8790`, `protocol-bus:9098`, `gateway:9099`, `byok-gateway:18787`), `<toksi-siagent>` (`toksi _worker siagent`), and core tools wired over Named Pipes (`//./pipe/toksi-*`) / UDS (`~/.toksi/run/*.sock`) + `DirectWireBus`, backed by `supervisor_workers` in `toksi.db`, `[supervisor]` TOML, exponential watchdog restart, and `CommandGuard` kill-switch fan-out.
- **`PHASE_19_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Live-Safe Sandboxed & Docker Test Isolation**: Built `docker/Dockerfile.test` & `docker/compose.test.yaml`, wired `toksi test --docker | --sandbox` in `src/commands/test_runner.rs`, and isolated test harnesses onto disposable `TOKSI_HOME` tempdirs and `:0` ephemeral ports so unit and E2E runs NEVER disturb live host `toksi` services, `~/.toksi/toksi.db`, or OS env/keyring while honoring `.toksi/test-baseline.json` (`43` suites).
- **`PHASE_19_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Zero-Pollution Adapt-Only Repo Hygiene & `.agents/skills/` Cleanup**: Audited `<neo>/.agents/skills/` alongside `<neo>/agents/`, `<neo>/agents/rules/`, and crate trees; stripped unadapted foreign bash/python scripts and GitHub Actions YAML; verified zero skill/rule files embedded in `crates/toksi/`; graded vetted adapted skills via `skilldoctor` + `agentskills` + `upskill` + `skill-scanner` (`<enforce>`) and synced cleanly to `Navegos/skills` + zero-plaintext shards to `Navegos/advisory`.
- **`PHASE_19_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Wave 19 SI Agentic Stack, Workforce, `ANP`, Rust Systems & ATR/Guardrails**: Implemented Wave 19 pure-Rust agents & durable primitives (`adk-rust`, `rig`, `maki`, `steer`, `chevalier`, `funes`, `agentrix-os`, `tura`, `tabby`, `refact`, `native-cli-ai`, `ironwire`, `claurst`, `tidev`, `ironclaw`, `xa`, `octos`, `rust-code`, `ringlet`, `ironcode`, `brat`, `arc-cli`, `aitk`), Workforce (`OMA`, `swarms-rs`, `ChatDev`, `deer-flow`, `openworker`, `dsh-superagent-prototype`, `langgraph-swarm-py`) + `/workforce`, `OpenSpec` + `/spec`, `ANP` (`did_wba`) + `QUIC` (`quinn`) on `:9098`, `bridge-mcp`, Rust systems (`vertex_rust_sdk`, `lancedb`, `arrow-rs`, `tiktoken`, `BLAKE3`, `pmat`, `depyler`, `coredb`, `databend`, `coach`), and `agent-threat-rules` + `openai-guardrails-python` + `model_spec`.
- **`PHASE_19_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Model Training, Quantization, Pure-Rust/Multimodal Serving Plane & 49-File 8-State Sync**: Built `src/training/`, `commands/{train,serve,eval}.rs`, and `/train`, `/serve`, `/voice` unifying `oxillama`, `xinfer`, `Crane`, `paddler`, `llama-swap`, `llguidance`, `safetensors`, `ggml`, `whisper*`, `stable-diffusion.cpp`, `cudarc`, `server`/`core` (Triton), `DeepSpeed-*`, `unsloth*`, `Automodel`, `nanotron`, `Emerging-Optimizers`, `torchtune` (recipe ref only), `torch*`, `onnx*`, `Olive`, `intel-neural-compressor`, `digestai`, `sentence-transformers`, `cuvs`, `CV-CUDA`, `warp`, `xla`, `lighteval`, `langwatch`, `wandb`, `clearml*`, `livekit*`, `moly-ai`, `robius`, `robrix`, `hf-hub`.

---

## 9. Wave 20 / v7.0 Execution Phases (`PHASE_20_1` -- `PHASE_20_6`)

- **`PHASE_20_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Real `<toksi-siagent>` Subprocess, 22-Mode Router, 5 Efforts, Sampling Matrix & 29 Slash Commands**: Eliminated `src/agent_runtime/loop_engine.rs:173-176, 249-251` local echo/mock stubs; routed through `_worker siagent` + Direct-Wire IPC + real Gateway inference; wired all **22 Orchestrator Modes** across 7 Cognitive Domains (`[toksi_siagent_router.*]`, `orchestrator_mode_routes`); added `Max|XHigh|High|Medium|Low` efforts (default `Medium`) + strict per-model `(temperature, top_p, top_k)` compatibility filtering (`src/gateway/sampling.rs`); expanded `CORE_SLASH_COMMANDS` to **29** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/mode`, `/sampling`, `/router`, `/projectmem`, `/ctxmode`, `/codegraph`, `/icm`, `/aimemory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`, `/workforce`, `/spec`, `/dataset`, `/train`, `/eval`, `/compress`, `/serve`, `/cluster`, `/otel`, `/voice`, keeping `/context` & `/ai-memory` aliases).
- **`PHASE_20_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Daemon `heal|dashboard|service`, Detached `:9100`, Per-Service DBs, Auto `/v1`, Dual `agy`/ADC & 10-Tab UI**: Set `DaemonServicePlan::dashboard = false` on `toksi daemon start`; added `toksi daemon heal`, `toksi daemon dashboard`, `toksi daemon service <name> <action>`; moved telemetry to `~/.toksi/services/*.db`; auto-completed `/v1` base URLs; coexisted standard `agy` + `AGY_ADC_AUTH=true` Vertex ADC; removed `"opencode"` UI stub & guaranteed zero CLI spawning in UI; delivered **10-Tab UI** (`Savings`, `Activity Room`, `AI Agent Fleet`, `Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `AI Gateway`, `Keyring & Database`, `System Controls`).
- **`PHASE_20_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Docker CLI/Compose OS Virtualization (`compose.test.yaml`) & `sbxenv.yaml`/Kit v3 Sandbox Engine**: Authored `docker/compose.test.yaml` + pure-Rust `ComposeSpec` & `SbxEnvManifest`/`SbxKitV3Manifest` validators (`toksi sandbox compose|sbxenv`).
- **`PHASE_20_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **11-Pillar AI/ML Lifecycle & Wave 20 Vaults (`26` Sec `2.16` + `4` Sec `2.17`)**: Integrated `liteparse`, `hyperflare-inference`, `NeMo-Relay-Plugins` (`relay-plugin.toml`), `kvpress`, `bitsandbytes`, `Megatron-Energon`, `structured-data-models`, `ossie*`, `auto-ontology`, `llama_index`, `llama-agents`, `haystack`, `prompt-ops`, `llama-models`, `ogx*`, `semantic-router`, `NeMo-Agent-Toolkit*`, `elements`, `AutoFA`, `deepeval`, `ragas`, `PurpleLlama`, `vscode`, `aider` (`toksi dataset|compress|eval`).
- **`PHASE_20_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Toksi Distributed Cluster Stack (`src/cluster/`, `toksi cluster`)**: Unified `kubernetes`, `minikube`, `kompose`, `kops`, `helm`, `autoscaler`, `node-problem-detector`, NVCRE (`cluster-readiness-engine`), `NVSentinel`, `aicr`, `knavigator`, `AI-Factory-Operations-Agent`, `deepops`, Kubeflow (`hub`/`pipelines`/`trainer`/`sdk`/`kale`/`spark-operator`/`dashboard`/`notebooks`/`semantic-operator`/`katib` ref), `Run`, `spark`, `cudf-spark`, `cuPhoton`, and `Personal-AI-Router` LAN mesh (`cluster_nodes` SQLite + `/api/cluster/*`).
- **`PHASE_20_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **OpenTelemetry Native LLMOps (`opentelemetry-rust`, `opentelemetry-collector`, `opentelemetry-operator`, `opentelemetry-python*`) & 49-File 8-State Sync**: Built `src/telemetry/otel.rs` (`gen_ai.*` OTLP -> `~/.toksi/services/otel.db`), `src/commands/otel.rs`, `src/cluster/otel_pipeline.rs`, and full 49-file `.agents/brain/*.md` + 6 master doc synchronization.

---

## 10. Wave 21 / v8.0 Execution Phases (`PHASE_21_1` -- `PHASE_21_6`)

- **`PHASE_21_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Single Dedicated `NetworkService` Manager (`toksi _worker network-manager`) & On-Demand `<toksi-siagent>` Decoupling**: Consolidated daemon socket binding into ONE `NetworkService` process (`src/ipc/network_manager.rs`, fixing `os error 10048`) routing over DirectWire IPC to service/tool workers (`Cluster` added), and removed `WorkerRole::Siagent` from `daemon start` so `<toksi-siagent>` runs on-demand.
- **`PHASE_21_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Codex CLI + Grok-Build CLI Grade `<toksi-siagent>` Chat TUI & Full CLI Bridge (`src/agent_runtime/chat_ui.rs`)**: Replaced `loop_engine.rs` plain `stdin.read_line` REPL with interactive `ratatui` streaming Chat TUI supporting all 22 modes (`chat|run|exec|swarm|orchestrator|plan|study|research`) and executing ALL `toksi` CLI commands in-chat with `--plain` pipe fallback.
- **`PHASE_21_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **13-IDE External Plugin Host Protocol (`src/tools/ide_plugin_host.rs`, `toksi ide-bridge`)**: Standardized `:9098/ide/*`/IPC host contract for external plugins built outside `crates/toksi/` across VS Code, Visual Studio IDE, JetBrains, Android Studio, Zed, Xcode, Eclipse, Cursor, Windsurf, Apache NetBeans, Kiro, Qoder, and Lapce.
- **`PHASE_21_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **`Zoo-Code`, `agent-orchestrator` & 11-Pillar AI/ML Deep Integration (`495` Vaults)**: Updated `total_mandated_vault_count()` to `495` (`493` Wave 20 + `Zoo-Code` + `agent-orchestrator`), wired Semble semantic search, `DCG`, per-task git-worktree Kanban lanes (`src/agent_runtime/swarm/mod.rs`), and deepened 11-pillar AI/ML adapters (`ELEVEN_AIML_PILLARS`).
- **`PHASE_21_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Multi-Agent Orchestration Resilience & Cross-Chat Resume Engine (`src/agent_runtime/orchestration_resume.rs`)**: Write-ahead `orchestration_journal.jsonl`, disjoint `worker_leases.json`, 5-store atomic flush on network/error/429/quota exhaustion, and BLAKE3-verified cross-chat `/resume`.
- **`PHASE_21_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` --- **Wave 21 Integration Verification & 55-File 8-State Master Synchronization**: Added `phase21_1_to_21_6_verification.rs`, refreshed `.toksi/test-baseline.json`, passed `cargo check`/`clippy`/`cargo test`, and updated all 6 master docs + 55 `.agents/brain/*.md` guides.

---

## 11. Wave 22 / v9.0 Execution Phases (`PHASE_22_1` -- `PHASE_22_6`)

- **`PHASE_22_1`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- **`<toksi-siagent>` Chat Double-Keystroke Fix, Gateway `502` / Headroom-Central Fix, Line Editor & Git Engineer (`chat_ui.rs`, `loop_engine.rs`, `gateway/server.rs`, `src/tools/git_engineer.rs`)**: Gate `crossterm` events on `KeyEventKind::Press`; stop `loop_engine.rs:198-205` from bypassing `:8787` with synthetic `"gpt-6.2-astra"`; add longest-prefix/provider+family Gateway matching; add line-editor controls, Pause/Resume/Edit-While-Working, and `.git/config` + branch Git-Engineer awareness.
- **`PHASE_22_2`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- **`r3bl-open-core` (`r3bl_tui`) TUI Migration & `winit`-Style Mouse/Keyboard Ergonomics (`src/tui/`, `chat_ui.rs`, `top.rs`, `rtk.rs`)**: Adopt `r3bl_tui` (`r3bl-open-core`), `ratatui-image`, `tui-realm`, and `iocraft` patterns across `toksi chat|gain|top|tui` with press-only input gating and mouse hover states.
- **`PHASE_22_3`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- **Dynamic `.git/config` Repo Auto-Detection (`No neo_dir`), `<neo>/agents/rules/` Consolidation & Non-Destructive Prompt Architecture (`config/mod.rs`, `skills_repo.rs`, `full_example_config.toml`, `commands/refactor_router.rs`)**: Remove hardcoded `neo_dir`, point rules sync strictly to `<neo>/agents/rules/` and `<neo>/agents/agent_instructions.md`, and implement `toksi refactor-router` for classifying and extracting compact prompt-routing indexes without deleting detail.
- **`PHASE_22_4`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- **Strict Pure 7-Bit ASCII Guard, `MojibakeLab`/`python-ftfy` Auto-Healer (`toksi heal-encoding`) & 118 `.rs` Source Remediation (`src/security/ascii_guard.rs`, `src/util/{mojibake,ftfy}.rs`, `commands/heal_encoding.rs`)**: Implement pure-Rust mojibake detection/fixing and ASCII enforcement across source, config, and docs, and remediate non-ASCII bytes across `118` existing `.rs` files in `crates/toksi/src/`.
- **`PHASE_22_5`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- **Separate Service Port Isolation, Copilot -> Headroom Consolidation, Standalone/MCP/`<toksi-siagent>` SAST, Plugin Mounts, Bidirectional BYOK & 502 Vault Count**: Enforce distinct per-service ports, consolidate `:8789`/`:8790` onto `:8787`, wire multi-surface SAST + plugin mounts (`src/plugins/`) + bidirectional DB/TOML BYOK sync, and promote `CONSOLIDATED_VAULT_COUNT` to `502`.
- **`PHASE_22_6`** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]` `[SOURCE-CODE-NOT-IMPLEMENTED]` --- **Wave 22 Integration Verification, Pure-ASCII Audit & 12-State 61-File Synchronization**: Add `phase22_1_to_22_6_verification.rs`, refresh `.toksi/test-baseline.json`, pass `cargo check`/`clippy`/`cargo test`, verify zero non-ASCII bytes in enforced files, and promote all **61** `.agents/brain/*.md` guides to `[SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]`.
