# toksi — System Architecture, Disciplinary Standards & Implementation Plan (Wave 18 / v5.0)

High-performance headroom proxy, token optimizer, native AI gateway (`AGY_ADC_AUTH=true` Google Cloud ADC + effort-aware routing), multi-protocol agent wire integration (`MCP`/`A2A`/`ACP` on `:9098`), built-in 41-language static security analyzer & `agentskills` YAML healer (`<enforce>`), safe plan checkpoint/resume (`.toksi/projectmem/plan_checkpoints/`), native coding CLI/swarm runtime (`toksi chat|run|exec|swarm`), and 9-tab async telemetry cockpit for AI coding agents.

---

## 1. System Overview & Architectural Mandates

`toksi` is a pure Safe Rust (Edition 2024, `#![deny(unsafe_code)]`) headroom proxy (`live_zone` byte-range surgery + `CcrStore`), token compression engine, native AI gateway, multi-protocol agent wire platform, built-in security/skill governance suite, and standalone coding CLI. It sits transparently between local AI coding assistants (`antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude`, `cursor`, `codex`, `opencode`, `grok`, and 65/90+ ecosystem connectors across Categories 1–5 + 22 local inference engines) and configured BYOK providers (including Google Cloud ADC Vertex endpoints) or in-process tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`, `security`).

### Core Architectural Mandates:
1. **100% Native In-Tree Rust (`#![deny(unsafe_code)]`)**: Complete elimination of `npm`, `pip`, `uv`, `python`, `node`, and `set_var`/`LockFileEx`/`setsid` unsafe blocks; unavoidable Win32 FFI and raw C-ABI Tree-sitter shims live in isolated workspace leaf crates (`crates/toksi-sys`, `crates/toksi-grammars`). Zero missing code across all **67 reference vaults** in `inspect-vault/*`.
2. **Continuous Reference Vault Re-Audit (All 67 Vaults = 36 Prior + 31 Wave 18)**: All 67 vaults — the 36 Waves 1–17 vaults plus the 31 Wave 18 additions (`lance`, `openwiki`, `tokview`, `agent-control`, `agent-client-protocol`, `python-genai`, `python-aiplatform`, `langgraph`, `langchain`, `langfuse`, `langflow`, `deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `sage`, `codex-security`, `deepsec`, `osv-scanner`, `semgrep`, `semgrep-rules`, `research-securety-skills`, `agent-skills`, `openagent-skills`, `openclaw-agent-skills`, `agentskills`, `defenseclaw`, `skill-scanner`) — are active specifications to synthesize, enforce, and verify.
3. **Configurable Global Runtime (`~/.toksi/config.toml` -> `src/config/mod.rs`)**: Controls ports (`headroom=8787`, `grok_oauth=8788`, `copilot_vscode=8789`, `copilot_cli=8790`, `protocol_bus=9098`, `gateway=9099`, `dashboard=9100`, `byok_gateway=18787`), `[auth.adc]` (`AGY_ADC_AUTH=true`, `"gcp.project"`, `"gcp.location"` default `us-central1`), `[effort]`, `[security]` (`defenseclaw_fail_closed`, `skill_scanner_policy`), `[plugins]` (`plugin_dirs`, `enforce_security_scan`, `fail_closed_hooks`, `protocols`, `allow_agent_to_agent`), `[checkpoint]`, default `--agents`, detached `dashboard`, sandbox profile, and multi-instance `[[byok_profiles]]` synchronized with `~/.toksi/toksi.db` (`byok_profiles`, `plugins_registry`).
4. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**: Verify external binaries in system `PATH` cleanly (`rust-analyzer`, `gopls`, `clang-tidy`, `gcloud`, agent CLIs) without hardcoding paths. Missing binaries remain grayed out / disabled with explicit user remediation notices and zero silent failure.
5. **Universal Forward-Slash (`/`) Path Standard**: Normalize internal paths with `/` on Windows, Linux, and macOS (escaped `\\` permitted exclusively inside `cmd.exe` command strings in JSON5/TOML configs).
6. **Strict Scope Split — Home Config vs. Repo-Only Tool Stores**: `~/.toksi/` holds ONLY `toksi.db` (`byok_profiles`, `plugins_registry`), `config.toml`, and optional user-level `~/.toksi/plugins/`. ALL workspace intelligence stores (`codegraph`, `projectmem`, `context`, `icm`, `ai-memory`, `rtk`, `headroom`, `security`, repo `plugins`) live strictly inside `<repo>/.toksi/<tool>/` where `<repo>/.git/` exists — NEVER in `~/.toksi/<tool>`, never outside `<repo>`, and never inside `<repo>/.git/`. Monorepo subprojects (`<repo>/crates/*`) share root `<repo>/.toksi/` partitioned by `subproject_id`.
7. **Workspace Root Consolidation & Safe Plan Checkpoints**: Auto-merge legacy dotdirs (`.projectmem/`, etc.) and nested crate `.toksi/` / `.<agent>/` folders row-by-row into `<repo>/.toksi/` without data loss; persist atomic execution snapshots (including active plugin state) in `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` for instant crash/429 `/resume`.
8. **Standardized Tool Hook Syntax, `direct wired` Bus, `<enforce>` + `defenseclaw` + `skill-scanner`, & Toksi Unified Plugin System (`src/plugins/`)**: Uniform `<tool>-hook <agent>`, normalized MCP IDs (`projectmem`, `context`, `codegraph`, `security` — never `toksi-*`/`tokless_*`/`-index`), `.agents/mcp` + `.gemini/mcp` + `.copilot/mcp` + `.claude/mcp` -> `skills/<tool>`, `antigravity-cli` vs `antigravity-ide` separation, mandatory `<enforce>` boundary guards, `defenseclaw` fail-closed hookguard & audit evidence + `skill-scanner` multi-engine/CEL scan pre-load gate across all plugins (`src/plugins/`) & skills, `agentskills` YAML frontmatter auto-healer + collision deduplicator (`src/rules/skill_validator.rs`), and in-process `direct wired` mode.
9. **Multi-Protocol Standardization & 41+ Language AST/SAST Intelligence**: Full `MCP`, `A2A` (v1.0 JSON-RPC), `ACP` (`agent-client-protocol`), `stdio`, `SSE`, `HTTP` on `:9098` Protocol Bus (serving core subsystems + `src/plugins/` endpoints) + native parsing for all 45 language IDs in `src/codegraph/kernel/langs.rs` (closing `"dockerfile"`/`"vue"`/`"scss"` grammar gaps) powering both CodeGraph and the built-in 41-language static code security analyzer + OSV lockfile scanner (`src/security/`), with `.gitignore`/`.ignore` dynamic index purge.
10. **Zero Paid-Tier Restrictions, Google Cloud ADC (`AGY_ADC_AUTH=true`), & Multi-Instance `<byok, model, effort>`**: Remove `FreeTierDisqualified` / `[PAID]` / `[DISQUALIFIED-FREE]` blocks completely. Any enabled user-configured BYOK, Google Cloud ADC Vertex (`auth_mode="adc"`), or CLI account participates freely; `Low`/`Medium`/`High` thinking efforts translate dynamically across OpenAI/Anthropic/Gemini/Vertex while scaling Headroom compression aggressiveness.
11. **Native Toksi Coding CLI & Multi-Agent Swarm (`src/agent_runtime/`) & Unified Plugins**: `toksi chat | run | exec | swarm | plugin | agent resume` with Hermes/OpenClaw/MoFA/CrewAI/Codex/Grok-Build/LangGraph/DeepAgents/Bosun/ZeroClaw/Cline/KiloCode/AlphaCode capabilities, 3-OS sandbox, `apply_patch`, unified plugin mounts (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `protocol_endpoint`, `agent_to_agent`), and **13 in-chat slash commands** (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/plugin`, `/resume`).
12. **Ground-Truth Accounting, `tokview` Traces, ConnectRPC Safe Passthrough, & Network Auto-Heal**: Idle reads strictly `0`; zero-cost `/v1/models` fetching; local `tokview` + Langfuse-compatible span attribution per model/effort tier; OS network reconnect rebuilds HTTP client pools (`#0033`/`#0034` HTTP/2 ConnectRPC passthrough).
13. **9-Tab Async TUI & Web Cockpit (`:9100`) + Incremental Test Baseline (`.toksi/test-baseline.json`)**: Non-blocking UI with exact 9-tab parity surfacing `tokview` metrics, paused plan checkpoints, `skill-scanner` CEL/SARIF + `defenseclaw` guardrails/audit-evidence (Tabs 5–6), interactive Plugin Manager (Tab 7), `plugins_registry` (Tab 9), and Google ADC/effort controls alongside BLAKE3 unchanged-suite skipping without test weakening.

---

## 2. Unified 8-Port Network Topology (Overridable via `~/.toksi/config.toml`)

To guarantee zero socket buffer contention, eliminate TCP backpressure, and prevent latency jitter between ultra-low-latency LLM inference streaming, multi-protocol agent buses, and high-bandwidth browser UI traffic:

| Port | Service / Subsystem | Transport | Protocol | Isolation Guarantee |
|---|---|---|---|---|
| **Port 8787** | Streaming Headroom Proxy | TCP / HTTP/1.1 & HTTP/2 / SSE | HTTP chunked, SSE, ConnectRPC passthrough | Dedicated to Google Antigravity, CloudCode, IDE base URLs (`live_zone` byte surgery, effort scaling, & `CcrStore`). |
| **Port 8788** | Grok OAuth Proxy (`grok_daemon.rs`) | TCP / HTTP / SSE | HTTP/1.1 chunked, SSE | Dedicated xAI Grok OAuth proxy (NOT generic BYOK). |
| **Port 8789** | GitHub Copilot VS Code Proxy | TCP / HTTP / SSE | HTTP/1.1 chunked | Dedicated proxy for Copilot VS Code extension (`DEFAULT_COPILOT_PROXY_PORT`). |
| **Port 8790** | GitHub Copilot CLI Proxy | TCP / HTTP / SSE | HTTP/1.1 chunked | Dedicated proxy for Copilot CLI (`DEFAULT_COPILOT_CLI_PROXY_PORT`). |
| **Port 9098** | **Unified Protocol Bus** | TCP / HTTP / SSE / JSON-RPC | `MCP`, `A2A` v1.0, `ACP` (`agent-client-protocol`), checkpoint events | Dedicated non-blocking agent control & IDE protocol bus (`PHASE_15_4` / `PHASE_18_1`) passing payloads through `HeadroomEffortCompressor` (`Low`/`Med`/`High`) & `<enforce>` across all 67 vaults, `Headroom`, `Security`, and `Plugins`. |
| **Port 9099** | Enterprise AI Gateway & Drop-in SDKs | TCP / HTTP / SSE | `/v1/chat/completions`, `/v1/responses`, `/v1/messages`, `/v1/models`, `/v1/security/scan`, `/v1beta/models/*`, `/metrics` | Dedicated Tokio inference pool (<10 µs overhead) enforcing full `HeadroomEffortCompressor` (`Low`/`Med`/`High`), `AGY_ADC_AUTH=true` Vertex routing, & failover. |
| **Port 9100** | Web Telemetry & Control Dashboard Cockpit | TCP / HTTP / SSE / WS | Browser 9-Tab HTML/CSS/JS, `tokview` SSE metrics, REST control, DB export | Detached background service. Zero inference contention. |
| **Port 18787** | BYOK Gateway Listener (`byok_gateway.rs`) | TCP / HTTP / SSE | HTTP/1.1 chunked, SSE | Supervised multi-instance BYOK & Vertex ADC dispatcher passing every enabled request through the shared `HeadroomEffortCompressor` (`Low`/`Med`/`High`). |

All default ports above are overridable via `~/.toksi/config.toml` `[ports]`.

---

## 3. Disciplinary Coding Standards & Conventions

All agent workers and contributors must adhere to these non-negotiable coding standards across all tasks:

### 3.1 Rust Core Disciplines
1. **Safe Rust (Edition 2024)**:
   - Safe Rust is enforced across `crates/toksi` (`src/lib.rs` and `src/main.rs`) via `#![deny(unsafe_code)]`.
   - Eliminate all `std::env::set_var`/`remove_var` (use `ToksiConfig` + `Command::env()`), raw `LockFileEx`/`flock` (use `std::fs::File::lock()` / `fs2`), raw `pre_exec(setsid)` (use `CommandExt::process_group(0)` or `toksi-sys`), raw signal handlers (`tokio::signal` / `ctrlc`), and raw process walkers (`sysinfo`).
   - Unavoidable Win32 FFI (`advapi32`, `user32`, console) and C-ABI Tree-sitter wrappers live strictly in isolated workspace leaf crates `crates/toksi-sys` and `crates/toksi-grammars` with explicit `// SAFETY:` proofs.
2. **Zero Warnings Policy**:
   - The project must compile with zero compiler warnings and zero Clippy warnings (`RUSTFLAGS="-D warnings"`).
   - Zero dead code: unused functions, structs, fields, and imports must be deleted immediately.
3. **Strict Error Handling (Zero Panics in Production)**:
   - Prohibited in all library, server, proxy, and daemon code paths: `.unwrap()`, `.expect()`, `panic!()`, `unreachable!()`.
   - Domain errors use `thiserror` enums (`GatewayError`, `HeadroomError`, `KeyringError`, etc.).
   - CLI handlers use `anyhow::Result` with clear context (`.context("...")`).
   - Use `?` operator or explicit fallbacks.
4. **Async & Concurrency Architecture**:
   - Tokio multi-threaded work-stealing scheduler.
   - Never perform synchronous blocking disk I/O, file locking, or CPU-heavy parsing on async worker threads. Wrap them in `tokio::task::spawn_blocking`.
   - Communication between the gateway, headroom, and telemetry uses bounded, lock-free channels (`tokio::sync::mpsc`) and atomics (`AtomicBool`, `AtomicU64`).
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
| **File Locking** | Win32 `LockFileEx` (Mandatory) | POSIX `flock` / `fcntl` (Advisory) | POSIX `flock` / `fcntl` (Advisory) | `fs2::FileExt` (`lock_exclusive`, `lock_shared`) for atomic JSONL appends and SQLite WAL checkpoints. |
| **OS Keyring** | `advapi32.dll` (`CredReadW`, `CredWriteW`, `CredDeleteW`) | Secret Service API / `secret-tool` / D-Bus | macOS Keychain (`/usr/bin/security`) | `src/keyring/mod.rs`: `KeyringManager` with base64 OAuth envelope parser. Zeroize in-memory credentials on drop. |
| **Environment Injection** | `HKCU\Environment` Registry + `WM_SETTINGCHANGE` broadcast | `systemd --user set-environment` / `~/.config/environment.d/` | `launchctl setenv` / User session export | `src/env_injector/mod.rs`: Non-destructive injection with RAII `TeardownGuard` clearing variables on daemon shutdown. |
| **Background Processes** | `CREATE_NO_WINDOW (0x08000000)` & `DETACHED_PROCESS (0x00000008)` | `setsid()`, redirect stdio to `/dev/null` or logs | `setsid()`, redirect stdio to `/dev/null` or logs | Spawns clean detached daemon/CLI bridge processes without console windows or zombie processes. |
| **Process Signals** | `tokio::signal::ctrl_c()` | `SIGINT`, `SIGTERM`, `SIGHUP` via `tokio::signal::unix` | `SIGINT`, `SIGTERM`, `SIGHUP` via `tokio::signal::unix` | Clean signal capture invoking `TeardownGuard` and shutting down socket listeners. |
| **Network Binding** | Bind strictly to `127.0.0.1` / `::1` | Bind strictly to `127.0.0.1` / `::1` | Bind strictly to `127.0.0.1` / `::1` | Prohibit default `0.0.0.0` binding. Set `SO_REUSEADDR` to avoid `EADDRINUSE` on restart. |
| **Line Endings** | Handles `\r\n` and `\n` transparently | Native `\n` | Native `\n` | Normalize `\r\n` to `\n` before processing, indexing, and token counting. |
| **Executable Suffix** | Appends `.exe` dynamically | Bare executable name | Bare executable name | `std::env::consts::EXE_SUFFIX` used dynamically; `check_executable_in_path` tests both forms on Windows. |

---

## 5. Subsystem Directory Topology

```
crates/toksi/src/
├── main.rs                   # Unified CLI entrypoint & full usage help
├── lib.rs                    # Public crate library API
│
│   ├── mod.rs, wiki.rs, handoff.rs, tiered.rs, mem0_bridge.rs
│
├── projectmem/               # Pure Rust port of ProjectMem — strict <repo>/.toksi/projectmem/ + plan_checkpoints/
│   ├── mod.rs, schema.rs, store.rs, summary.rs, precheck.rs, redaction.rs, heatmap.rs, mcp.rs, plan_checkpoint.rs, wiki_sync.rs
│
├── icm/                      # Instant Context Memory SQLite FTS5 BM25 + temporal/fusion/memoir (<repo>/.toksi/icm/)
│   ├── mod.rs, store.rs, schema.rs, recall.rs, mcp.rs
│
├── aimemory/                 # Open Knowledge Format (OKF v0.2) wiki, handoff, Lance columnar store, mem0 (<repo>/.toksi/ai-memory/)
│   ├── mod.rs, wiki.rs, handoff.rs, tiered.rs, mem0_bridge.rs, lance_store.rs
│
├── codegraph/                # 45-lang Tree-sitter AST, SCIP, ast-grep CST, jcodemunch, repomix, OpenWiki (<repo>/.toksi/codegraph/)
│   ├── mod.rs, kernel/, scip.rs, ast_grep.rs, jcodemunch.rs, repomix.rs, openwiki.rs, walker.rs, graph.rs, cache.rs, mcp.rs
│
├── contextmode/              # Subprocess sandbox, policy/journal, SQLite FTS5 search (<repo>/.toksi/context/)
│   ├── mod.rs, sandbox.rs, filter.rs, fts.rs, fetch.rs, mcp.rs
│
├── security/                 # defenseclaw fail-closed guardrails, skill-scanner CEL/taint, 41-lang AST SAST, sage, OSV (<repo>/.toksi/security/)
│   ├── mod.rs, analyzer.rs, defenseclaw.rs, hookguard.rs, skill_scanner.rs, cel_decision.rs, pipeline_taint.rs, bytecode_audit.rs, sarif.rs, audit.rs, yara_mcp.rs, sage_rules.rs, skill_audit.rs, osv.rs, semgrep_engine.rs, external_analyzers.rs, rules/builtin/
│
├── plugins/                  # Toksi Unified Plugin System (<enforce> scanned: agent_feature, cli_cmd, tool, chat_cmd, wire_hook, protocols, a2a)
│   ├── mod.rs, manifest.rs, loader.rs, hooks.rs, slash_router.rs, cli_bridge.rs, analyzer_plugin.rs
│
├── agent_runtime/            # Native Toksi coding CLI & swarm wired out of the box ("off the back") via DirectWireBus & :8787/:9098/:9099/:18787 to all gateways, Protocol Bus, HeadroomEffortCompressor, src/plugins/, & all 67 vaults
│   ├── mod.rs, types.rs, loop.rs, context.rs, edit_tools.rs, planning.rs, orchestrator.rs, reflection.rs, sandbox.rs, slash.rs, event_log.rs, cron.rs, checkpoint.rs, acp.rs, guardrails.rs, graph_checkpoint.rs, deep_planning.rs, fleet.rs, modes.rs, zeroclaw_core.rs
│
├── config/                   # Global ~/.toksi/config.toml loader, schema validator, hot-reload, [auth.adc] & DB sync
│   ├── mod.rs
│
├── headroom/                 # Shared HeadroomEffortCompressor (Low/Med/High: live_zone, CcrStore, SmartCrusher, TokenEconomizer, agent-opt, mcp_pruner) + Copilot/Grok/BYOK proxies
│   ├── mod.rs, copilot_daemon.rs, grok_daemon.rs, byok_gateway.rs, proxy.rs, effort.rs, compression/, pruner/, cache_align.rs, adaptive_mem.rs
│
├── gateway/                  # Enterprise AI Gateway (:9099), Protocol Bus (:9098), Vertex ADC (:18787/:9099) & Failover Engine — all enforcing HeadroomEffortCompressor before dispatch/cache
│   ├── mod.rs, reconnect.rs, vertex_adc.rs, gemini_wire.rs, effort_router.rs, providers/, agent_cli/, cascade.rs, keys.rs, loadbalancer.rs, routing.rs,
│   ├── governance.rs, semantic_cache.rs, normalizer/, compat/, circuit_breaker.rs, proxy.rs
│
├── telemetry/                # Ground-truth session metrics, tokview/Langfuse spans & TokenTracker
│   ├── mod.rs, prometheus.rs, collector.rs, tokentracker.rs, tokview.rs, traces.rs, cost.rs, tracker.rs, cache_analytics.rs
│
├── rtk/                      # 100+ filters, ANSI stripper, SQLite Tracker (rtk gain), tee recovery, discover, learn, slash
│   ├── mod.rs, engine.rs, filters/, ansi.rs, hook.rs, tracker.rs, tee.rs, discover.rs, learn.rs, slash.rs
│
├── rules/                    # In-tree rules (<enforce>, Karpathy, Caveman v4, Ponytail) & agentskills YAML frontmatter healer/dedup
│   ├── mod.rs, sync.rs, skill_validator.rs, skills_catalog.rs, principles.rs, caveman.rs, ponytail.rs, embedded/
│
├── env_injector/             # Non-destructive OS environment injection & legacy binary healing
│   ├── mod.rs, windows.rs, unix.rs, teardown.rs, heal.rs
│
├── dashboard/                # Background Web Telemetry Cockpit on Port 9100 (9-tab SPA + SSE + REST)
│   ├── mod.rs, state.rs, sse.rs, api.rs, assets/
│
├── tui/                      # Non-blocking async 9-tab mouse TUI (ratatui + crossterm EnableMouseCapture)
│   ├── mod.rs, app.rs, views/, widgets/
│
├── agents/                   # Configured/selected agents & Category 1-5 connectors + direct-wire bus
│   ├── mod.rs, manager.rs, open_agents.rs, antigravity.rs, copilot.rs, claude.rs, codex.rs, cursor.rs, grok.rs, opt.rs, mcp_adapters.rs
│
├── keyring/                  # Cross-platform secure credential manager + Google Cloud ADC (AGY_ADC_AUTH=true)
│   ├── mod.rs, adc.rs, os_windows.rs, os_macos.rs, os_linux.rs, envelope.rs
│
├── db/                       # Persistent SQLite configuration database (~/.toksi/toksi.db) + byok_profiles (gcp_project/gcp_location) + plugins_registry
│   ├── mod.rs, schema.rs, operations.rs, migrate.rs
│
├── util/                     # Forward-slash path normalization, PATH validation mandate, exec, parsers
│   ├── paths.rs, env_path.rs, exec.rs, jsonc.rs, toml.rs, colors.rs
│
└── commands/                 # Native CLI command dispatcher
    ├── daemon.rs, proxy.rs, config.rs, agent_runtime.rs, security.rs, plugin.rs, auth.rs, db.rs, agent.rs, tui.rs, dashboard.rs, mem.rs, graph.rs, ctx.rs, rtk.rs, rules.rs, doctor.rs, mod.rs
```

---

## 6. Detailed Execution Roadmap (Phases 15.1 – 16.3)

All tasks are critical and must be executed with strict technical precision.

### Phase 15.1: Workspace Topology, Forward-Slash Normalization & Standards
- **Objectives**: Universal forward-slash `/` paths, PATH validation mandate, user-home DB storage, single root `.toksi` consolidation, and standardized hook/MCP naming.
- **Targets**:
  * `crates/toksi/src/util/paths.rs`: Implement `normalize_path(p: &str) -> String` converting all backward slashes `\` to `/` across all OSes. Handle drive letters (`C:/`), UNC paths (`//server`), and tilde expansions. Preserve escaped `\\` exclusively for cmd.exe command strings in JSON5/TOML configs.
  * `crates/toksi/src/util/env_path.rs`: Implement `check_executable_in_path(cmd: &str) -> Result<(), String>`. Verify required binaries exist in `PATH` without encoding absolute hardcoded paths. On missing binary, log clean error message with remediation instructions.
  * `crates/toksi/src/db/mod.rs` & `operations.rs`: Relocate default SQLite DB to `~/.toksi/toksi.db` (`dirs::home_dir()`). Auto-migrate legacy `LOCALAPPDATA/toksi` data on startup.
  * `crates/toksi/src/commands/init.rs` & `doctor.rs`: Single root `.toksi` consolidation: discover subfolder `.toksi` directories, merge rows into root `.toksi/toksi.db`, `memory.db`, `symbols.db` without data loss, and delete nested `.toksi` folders.
  * `crates/toksi/src/agents/hooks.rs` & `mcp_adapters.rs`: Standardize hook syntax to `<tool>-hook <agent>` (e.g., `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`). Normalize MCP tool identifiers: `projectmem`, `context`, `codegraph`. Map MCP settings to `skills/<tool>`.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.2: Unified 8-Port Daemon, `config.toml`, & Copilot Supervision
- **Objectives**: Supervise all 8 dedicated endpoints simultaneously (`:8787`, `:8788` Grok, `:8789`/`:8790` Copilot, `:9098` Protocol Bus `MCP`/`A2A`/`ACP`/`SSE`, `:9099` Gateway, `:9100` Cockpit, `:18787` BYOK — all overridable via `~/.toksi/config.toml` `[ports]`) via `toksi daemon start --agents` with complete latency and socket isolation.
- **Targets**:
  * `crates/toksi/src/commands/daemon.rs`: Master daemon supervisor starting Port `8787` (Headroom), Port `8788` (Grok OAuth), Port `8789` (Copilot VS Code), Port `8790` (Copilot CLI), Port `9098` (Protocol Bus), Port `9099` (AI Gateway & `/metrics`), Port `9100` (Web Dashboard Cockpit), and Port `18787` (BYOK Gateway).
  * `crates/toksi/src/headroom/copilot_daemon.rs`: Dual-port supervisor managing 8789 and 8790 with live `/livez` health probes and collision prevention.
  * `crates/toksi/src/gateway/mod.rs` & `dashboard/mod.rs`: Guarantee thread and socket buffer isolation between Port 9099 (inference) and Port 9100 (dashboard).
  * `crates/toksi/src/env_injector/teardown.rs`: Clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix environment on daemon shutdown.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.3: Smart Agent CLI Providers & Open BYOK Quota Telemetry
- **Objectives**: Register Local Agent CLIs (`AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, `GrokCli`) and ecosystem bridges with smart background launch; track read-only commercial/free quota windows with **zero paid-tier disqualification**.
- **Targets**:
  * `crates/toksi/src/gateway/agent_cli/mod.rs`: Registry for `AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, and `GrokCli` (using safe process spawning).
  * `crates/toksi/src/gateway/agent_cli/tier_check.rs`: Remove `FreeTierDisqualified` / `ACTIVE_PAID_PROVIDERS` blocking gates; expose read-only account label and `quota_windows` (`hour|day|week|month|year`) cleanly.
  * Graceful Inform & Zero-Fail: Never panic, crash, or fail on unbilled/rate-limited responses. Log rate-limit exhaustion clearly and fall back to next enabled profile.
  * `crates/toksi/src/gateway/providers/aggregators/bifrost.rs`: Wire Bifrost provider client into gateway dispatch with real usage extraction and failover routing.
  * Activation Guard: Engage routing strictly across user-configured & enabled BYOK profiles (`src/config/mod.rs` + `byok_profiles`); pass traffic cleanly through when zero profiles are enabled.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging
- **Objectives**: Expose MCP, A2A v1.0, ACP v2.0, stdio, SSE, and HTTP across all tools; integrate `.gitignore` / `.ignore` and `<repo>`-only scoping.
- **Targets**:
  * `crates/toksi/src/tools/multi_protocol.rs`: Multi-protocol dispatch for `rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory` (including real `A2aTaskStore` & `SessionActorQueue`).
  * `crates/toksi/src/codegraph/walker.rs`, `cache.rs`, and `contextmode/fts.rs`: Integrate ripgrep-grade ignore walker respecting `.gitignore` and `.ignore` without indexing outside `<repo>` or inside `<repo>/.git/`.
  * Dynamic Vector & Cache Purging: When ignore rules change, automatically purge matching existing records from SQLite FTS5 search indexes and vector embedding caches.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing
- **Objectives**: Ground-truth session token accounting from live SSE streams; network reconnection auto-healing watchdog; configured-only universal rules sync.
- **Targets**:
  * `crates/toksi/src/telemetry/tokentracker.rs` & `collector.rs`: Parse real usage tokens (`cache_creation_input_tokens`, Responses usage, Gemini `candidatesTokenCount`) directly from live SSE response streams and upstream rate-limit headers (`x-ratelimit-remaining-*`, `anthropic-ratelimit-*`). Strict prohibition of virtual/synthetic math (`0` when idle).
  * `crates/toksi/src/gateway/reconnect.rs`: Automatic reconnection watchdog detecting network disconnects, VPN toggles, and socket timeouts (`ArcSwap<reqwest::Client>` rebuild).
  * `crates/toksi/src/rules/sync.rs`: Universal synchronizer for rules, skills, and instructions across configured agents (`18` built-in adapters + `.agents` + Category 1–5 layouts, with granular item toggles).
  * `crates/toksi/src/main.rs`: Update `help_text()` with comprehensive details on all commands, flags, `config.toml`, slash commands, ports, and conventions.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.6: Advanced Vault Intelligence Engines Consolidation
- **Objectives**: Consolidate remaining code-intelligence vaults into pure Rust with real end-to-end wiring (removing stub fallbacks).
- **Targets**:
  * `TokenTracker` -> `crates/toksi/src/telemetry/tokentracker.rs`
  * `jcodemunch-mcp` -> `crates/toksi/src/codegraph/jcodemunch.rs` (O(1) SQLite byte-offset slices)
  * `repomix` -> `crates/toksi/src/codegraph/repomix.rs` (secretlint + BPE + `.gitignore`/`.repomixignore`)
  * `ast-grep` -> `crates/toksi/src/codegraph/ast_grep.rs` (real Tree-sitter CST + `MatchStrictness`)
  * `scip` -> `crates/toksi/src/codegraph/scip.rs` (real Protobuf + descriptor AST)
  * `mem0` -> `crates/toksi/src/aimemory/mem0_bridge.rs`
  * `chroma` -> `crates/toksi/src/gateway/semantic_cache.rs` (ANN HNSW/USearch/SPANN + `foyer` spill)
  * `litellm` -> `crates/toksi/src/gateway/normalizer/litellm.rs`
  * `TokenEconomizer` -> `crates/toksi/src/headroom/compression/economizer.rs` (3-stage + LoopGuard + EnvFacts)
  * `agent-opt` -> `crates/toksi/src/agents/opt.rs` (SHA-256 1-hop context dedup + oscillation detector)
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 16.1: High-Performance Async 9-Tab Mouse-Capable TUI
- **Objectives**: Full mouse interaction and non-blocking asynchronous event loop eliminating terminal hangs, with exact 9-tab parity (`Savings`, `The War Room`, `AI Agent Fleet & Quotas`, `Memory Decision DAG`, `CodeGraph Explorer`, `Sandbox Inspector`, `System Controls`, `AI Gateway & Quotas`, `Keyring & Database`).
- **Targets**:
  * `crates/toksi/src/tui/mod.rs`, `app.rs`, `views/*`: Enable crossterm `EnableMouseCapture`. Route mouse clicks for tab navigation, row selection, heatmap cells, and action buttons; remove all fake/hardcoded placeholder rows and `[PAID]`/`[DISQUALIFIED-FREE]` badges.
  * Event loop: Fully non-blocking async snapshot receiver (zero synchronous DB/HTTP locks inside the 50ms render loop).
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard
- **Objectives**: Isolated, detached background 9-tab web UI on Port `9100` (`toksi dashboard start|stop|restart`) with zero synthetic stubs.
- **Targets**:
  * `crates/toksi/src/dashboard/{mod.rs,api.rs,state.rs,sse.rs}` & `assets/{index.html,style.css,app.js}`: Remove fake waterfall (`0.38/0.42/0.20`), fake `:8788 BYOK` label, fake heatmap vec, fake 6-symbol CodeGraph fallback, fake `1428 KB` vacuum savings, and raw `<AGENT>_API_KEY` Keyring stub; wire real data, clickable `.heatmap-cell` modal drilldown, `Export`/`Import` DB buttons, and `<byok, model, effort>` profile CRUD.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 16.3: Open Agents Directory Protocols & README Verification
- **Objectives**: Support `.agents/` (incl. `.agents/mcp -> skills/<tool>`) and `agents-structures` vendor layouts; configured/user-selected auto-install rules; Wave 18 `README.md` rewrite.
- **Targets**:
  * `crates/toksi/src/agents/{open_agents.rs,manager.rs,mcp_adapters.rs}`: Reconcile `agents-structures` parity (`.grok/hooks/*.json`, `.copilot/hooks/`, `.github/copilot/`, `.gemini/{settings.json,commands/*.toml}`, `.claude/.mcp.json`) and wire `"openagents"` cleanly.
  * `crates/toksi/README.md`: Complete Wave 18 documentation covering all 8 ports, `~/.toksi/config.toml` (`[plugins]`, `[security]`), `toksi chat|run|exec|swarm|security|plugin|agent resume`, 13 slash commands (incl. `/plugin`), `src/plugins/` Unified Plugin System, `<repo>/.toksi/<tool>`, `direct wired` bus, `<enforce>` security (`defenseclaw` + `skill-scanner`) & `agentskills` YAML healer, `AGY_ADC_AUTH=true`, effort optimization, 41+ Tree-sitter languages, and 65/90+ ecosystem providers.
- **Verification**: `cargo check -p toksi`, `cargo clippy --all-targets -- -D warnings`, and incremental `.toksi/test-baseline.json` suite pass 100%.

---

## 7. Wave 17 Master Execution Phases (`PHASE_17_1` – `PHASE_17_7`)

*(Detailed worker prompts in [`.agents/brain/INDEX_AND_EXECUTION_GUIDE.md`](../../.agents/brain/INDEX_AND_EXECUTION_GUIDE.md). Execute incrementally; run targeted verification per phase plus `cargo check` & `cargo clippy -- -D warnings`.)*

- **`PHASE_17_1` — `~/.toksi/config.toml`, CLI Process Controls, Edition 2024 `#![deny(unsafe_code)]`, & Zero Paid-Tier Restrictions**: Implement `src/config/mod.rs` (incl. `[plugins]` + `[security]` `defenseclaw_fail_closed` / `skill_scanner_policy`), add `--agents` to `daemon`/`proxy` and `start|stop|restart` to `dashboard`, delete `FreeTierDisqualified`/`[PAID]` gates, eliminate all `set_var`/`LockFileEx`/`setsid` unsafe blocks, isolate unavoidable FFI into `crates/toksi-sys`, and enforce `#![deny(unsafe_code)]` on `crates/toksi`.
- **`PHASE_17_2` — Multi-Model `<byok, model, effort>` DB, Keyring Fix, `headroom-core` `live_zone`, & Universal Headroom Effort Pass**: Upgrade `db/{schema,operations,migrate}.rs` (`byok_profiles` w/ `auth_mode`/`gcp_project`/`gcp_location`, `agent_profile_bindings`, `quota_windows`, `plugins_registry`, fix `export_dump`/`import_dump`), normalize `gemini:antigravity` keyring, port pure-Rust `headroom-core` (`compute_frozen_count` + `live_zone` byte surgery + `CcrStore` + `SmartCrusher` + `AuthMode`), route ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`) through shared `HeadroomEffortCompressor` + `chroma`/`lance` ANN `semantic_cache`, and retire Python `headroom` spawns + `cloudcode_patch.rs`.
- **`PHASE_17_3` — 41+ Tree-sitter Languages, `<repo>/.toksi/<tool>` Repo-Only Storage, Monorepo Subproject Partitioning, & `.ignore` Purge**: Isolate `crates/toksi-grammars`, close `grammar_for()` gaps in `src/codegraph/kernel/langs.rs` (`"dockerfile"`, `"vue"`, `"scss"` specialization), port remaining `codegraph-kernel` extractors, consolidate real `ast-grep`/`scip`/`jcodemunch`/`repomix`, fix `commands/index.rs::find_project_dir()` so `<repo>/crates/*` stay under root `<repo>/.toksi/`, eliminate home-dir project caches (`projectmem`/`icm`/`aimemory`/`contextmode`/`codegraph`), merge subfolder `.<agent>`/`.toksi`, and guard non-`.git` / `.git/` paths.
- **`PHASE_17_4` — Deep Vault Parity (`TokenEconomizer`, `agent-opt`, `rtk`, `icm`, `ai-memory`, `projectmem`, `caveman` v4, `ponytail`), 65/90+ Ecosystem & 22 Local Engines, & Hook/MCP Cleanup**: Fix `commands/ctx.rs` passthrough bug and `commands/runmcp.rs` `aimemory->icm` misroute, close MCP tool gaps (`projectmem` 16 tools, `icm` 28 tools), add 47-tool `TokenTracker` parsers & local engine discovery across all 67 vaults, enforce `<tool>-hook <agent>` + `antigravity-cli|ide` + `.agents/mcp` + `openagents` adapter, and purge legacy `toksi-*`/`tokless_*`/`rtk hook cursor`/`resolve_npm_binary`/`wrap`.
- **`PHASE_17_5` — Native `toksi` Coding CLI & Swarm (`src/agent_runtime/`), `direct wired` Hook Bus, & 13 In-Chat Slash Commands**: Build `src/agent_runtime/*` + `toksi chat|run|exec|swarm|agent resume` wired out of the box ("off the back") via `DirectWireBus` & `:8787`/`:9098`/`:9099`/`:18787` to every gateway, Protocol Bus, `HeadroomEffortCompressor`, `src/plugins/`, and all 67 vault tools (fusing Hermes/OpenClaw/MoFA/MofaClaw/CrewAI/Codex/Grok-Build: `ToolLoopDetector`, `RunawayStreamWatch`, `SubtaskDAG`, `MoAChatCompletions`, `ReflectionRunner`, 3-OS sandbox, Lark `apply_patch`, `HashlineEdit`), real `A2aTaskStore` v1.0 & `SessionActorQueue` ACP v2.0, `DirectWireBus`, and `/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/plugin`, `/resume`.
- **`PHASE_17_6` — Ground-Truth SSE Accounting, Gateway Real Dispatch (`:9098` / `:9099` / `:8789` / `:8790` / `:18787`), OS Reconnect Watchdog, & ConnectRPC Passthrough**: Replace gateway/copilot stubs, wire full `HeadroomEffortCompressor` inside `gateway/proxy.rs` (`:9099`), `byok_gateway.rs` (`:18787`), & `tools/multi_protocol.rs` (`:9098`), wire `LoadBalancer` + `cascade` + `mofa-engine` 7-D `Router::route_ranked`, zero-cost `/v1/models` refresh & rate-limit `quota_windows`, `ArcSwap<reqwest::Client>` reconnect heal, and HTTP/2 ConnectRPC safe passthrough (`#0033`/`#0034`).
- **`PHASE_17_7` — Async 9-Tab Mouse TUI, Live Web Cockpit (`:9100`), Incremental `.toksi/test-baseline.json` Harness, CLI `--help`, & Full Doc Sync**: Finalize 9-tab UI parity (including `skill-scanner` Tab 5, `defenseclaw` Tab 6, Plugin Manager Tab 7, `plugins_registry` Tab 9), remove every synthetic placeholder, implement clickable heatmap/DB/routing modals, ship `.toksi/test-baseline.json` BLAKE3 unchanged-suite skip harness, add `phase14..18_4` verification suites, and finalize `crates/toksi/README.md` + `main.rs` `--help`.

---

## 8. Wave 18 / v5.0 Execution Phases (`PHASE_18_1` – `PHASE_18_4`)

- **`PHASE_18_1` — 31 New Vaults Synthesis (`lance`, `openwiki`, `tokview`, `agent-control`, `acp`, `python-genai`/`aiplatform`, `langgraph`/`langchain`/`langflow`/`langfuse`/`deepagents`, `bosun`, `Codewhale`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `defenseclaw`, `skill-scanner` + `src/plugins/`)**: Pure-Rust `lance_store.rs` columnar vector engine, `openwiki.rs` CodeGraph architecture wiki synthesizer, `tokview.rs` local span/effort observability on `:9100`, ACP JSON-RPC on `:9098` w/ shared `HeadroomEffortCompressor`, unified plugin manifest/discovery foundations (`src/plugins/`), and modular out-of-the-box `agent_runtime` DAG/fleet/mode/zero-overhead traits wired across all gateways, Protocol Bus, and 67 vaults.
- **`PHASE_18_2` — `<enforce>` Security, Unified Plugin System (`src/plugins/`), `defenseclaw`, `skill-scanner`, `agentskills` Healer, & Built-In 41-Language Static Code Security Analyzer (`src/security/`)**: Wrap security-critical skills/rules/plugins/scripts in `<enforce>...</enforce>`, build the **Toksi Unified Plugin System (`src/plugins/*`, `toksi plugin`, `/plugin`, SQLite `plugins_registry`)** across 7 mount surfaces (`agent_feature`, `cli_cmd`, `tool`, `chat_cmd`, `wire_hook`, `MCP`/`A2A`/`ACP`/`stdio`/`SSE`/`HTTP`, `agent_to_agent`), integrate **`defenseclaw`** fail-closed hookguard/tool-chains/JSONL audit/`codesig` + **`skill-scanner`** multi-engine taint/bytecode/bounded CEL policy, port `inspect-vault/agentskills/skills-ref` YAML validation + frontmatter auto-healer + trigger dedup (`src/rules/skill_validator.rs`), and ship the zero-dependency 41-language AST security scanner + OSV lockfile parser (`semgrep`, `semgrep-rules`, `codex-security`, `deepsec`, `sage`, `osv-scanner`, `research-securety-skills`, plus optional `PATH` `rust-analyzer`/`gopls`/`clang-tidy`).
- **`PHASE_18_3` — Google Cloud ADC (`AGY_ADC_AUTH=true`), Thinking-Effort Optimization (`Low`/`Medium`/`High`), & Safe Plan Checkpoints (`/resume`)**: Native pure-Rust Google Cloud ADC Vertex AI OAuth2/service-account token exchange honoring `"gcp.project"` / `"gcp.location"` (`us-central1` default) with clear `gcloud auth application-default login --project <GCP_PROJECT>` remediation, cross-provider `/effort` translation paired with dynamic shared `HeadroomEffortCompressor` scaling across ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`) & `src/agent_runtime/`, and atomic `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` (incl. active plugin state) + `events.jsonl` recovery after `429`/network drop/crash.
- **`PHASE_18_4` — Legacy `.projectmem/` Repair, Documentation Integrity & Final Workspace Verification**: Enforce strict `<repo>/.toksi/<tool>/` migration and doc hygiene (`zero` stale `.projectmem/` pre-read refs, `35` indexed `.agents/brain/*.md` guides, exact module alignment across `67` vaults, `13` slash commands, `src/plugins/`, `defenseclaw`, `skill-scanner`, and `plugins_registry`), verified by `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `.toksi/test-baseline.json`.
