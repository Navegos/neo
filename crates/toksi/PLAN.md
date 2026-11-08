# toksi — System Architecture, Disciplinary Standards & Implementation Plan (v3.4)

High-performance headroom proxy, token optimizer, native AI gateway, multi-protocol agent wire integration, and telemetry cockpit for AI coding agents.

---

## 1. System Overview & Architectural Mandates

`toksi` is an enterprise-grade native Rust headroom proxy, token compression engine, native AI gateway, and multi-protocol agent wire platform. It transparently sits between local AI coding assistants (Google Antigravity, GitHub Copilot, Claude Code, Cursor, Codex, OpenCode, Grok, and others) and 39+ upstream LLM providers or in-process multi-protocol tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`).

`toksi` slashes token consumption by 30-70% through intelligent prompt compression, tool output condensing (RTK), code graph indexing, and structured cross-session project memory.

### Core Architectural Mandates:
1. **100% Native In-Tree Rust**: Complete removal of external package manager downloads (npm, pip, uv, curl). Pure in-process execution with zero missing code and full feature parity across all 23 reference engines in `inspect-vault/*`.
2. **Continuous Reference Vault Re-Audit**: All folders in `neo/inspect-vault/*` (`tokless`, `bifrost`, `headroom`, `rtk`, `context-mode`, `codegraph`, `projectmem`, `ai-memory`, `icm`, `ponytail`, `caveman`, `andrej-karpathy-skills`, `TokenTracker`, `jcodemunch-mcp`, `repomix`, `ast-grep`, `scip`, `mem0`, `chroma`, `litellm`, `TokenEconomizer`, `agent-opt`, `agents-structures`) are ALWAYS treated as new/updated code to inspect, convert, import, and recheck for new features, bugfixes, and agent capabilities.
3. **Smart Unified Single-Daemon Launch (`toksi daemon start`)**: One background daemon command initializes and coordinates all core subsystems simultaneously across dedicated, isolated network ports.
4. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**: For all external binaries, commands, shells, tools, and agent CLIs, Toksi verifies their presence in `PATH`. Never hardcode absolute binary paths everywhere. If missing, log a clean, actionable error instruction and do not fail silently.
5. **Universal Forward-Slash (`/`) Path Standard**: Strict prohibition of backward slashes (`\`) in all internal directory structures, path joins, logs, configuration files, SQLite records, and CLI outputs across Windows, Linux, and macOS. Escaped `\\` is permitted exclusively in `cmd.exe` commands inside JSON5/TOML configs (`hooks.json`).
6. **User Home Database Storage (`~/.toksi/toksi.db`)**: Default configuration database stored in user home, NOT in `localdata/toksi`, with automated migration from legacy paths.
7. **Workspace Root Consolidation**: Only ONE `.toksi` directory exists per workspace, strictly in the primary root directory. Nested subfolder `.toksi` directories are auto-merged row-by-row into root SQLite DBs and deleted. Sub-tool layouts are consolidated under `.toksi/<tool>`.
8. **Standardized Tool Hook Syntax & MCP Naming**: Standard `<tool>-hook <agent>` syntax (e.g., `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`). Normalized MCP tool identifiers: `projectmem`, `context`, `codegraph`. MCP instruction directory is `context`.
9. **Multi-Protocol Standardization**: All tools and agent connectors support MCP, A2A, ACP, stdio, SSE, and HTTP.
10. **Dynamic Ignore Purging**: Automatically purges already-indexed files from SQLite FTS5 indexes and vector embedding caches when `.gitignore` or `.ignore` patterns change.
11. **Guaranteed Local Agent CLIs as First-Class BYOK Providers**: `AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, and `GrokCli` are first-class BYOK routing peers with smart background launch.
12. **Strict Paid Quota Enforcement**: Free-tier and unbilled trial accounts are strictly disqualified from active BYOK gateway rotation. Graceful inform and zero-fail fallback: never crash or panic; emit informative notice and route to next paid provider or direct agent execution.
13. **Ground-Truth Session Token Accounting**: Real tokens parsed from live SSE completion chunks and upstream headers. Strict prohibition of synthetic or virtual tokens (counter stays at 0 when idle).
14. **Mouse-Capable Interactive TUI**: `ratatui` + `crossterm` with `EnableMouseCapture` and non-blocking asynchronous event loop, eliminating terminal hangs.
15. **Isolated Background Web Telemetry Cockpit**: Dedicated listener on port 9100 for browser dashboards, preventing socket or thread contention with LLM inference streams on port 9099.

---

## 2. Dedicated 5-Port Network Topology

To guarantee zero socket buffer contention, eliminate TCP backpressure, and prevent latency jitter between ultra-low-latency LLM inference streaming and high-bandwidth browser UI traffic:

| Port | Service / Subsystem | Transport | Protocol | Isolation Guarantee |
|---|---|---|---|---|
| **Port 8787** | Streaming Headroom Proxy | TCP / HTTP / SSE | HTTP/1.1 chunked, SSE | Dedicated to Google Antigravity, Grok, and IDE base URL overrides. |
| **Port 8789** | GitHub Copilot VS Code Proxy | TCP / HTTP / SSE | HTTP/1.1 chunked | Dedicated proxy for Copilot VS Code extension (`DEFAULT_COPILOT_PROXY_PORT`). |
| **Port 8790** | GitHub Copilot CLI Proxy | TCP / HTTP / SSE | HTTP/1.1 chunked | Dedicated proxy for Copilot CLI (`DEFAULT_COPILOT_CLI_PROXY_PORT`). |
| **Port 9099** | Enterprise AI Gateway & Drop-in SDKs | TCP / HTTP / SSE | `/v1/chat/completions`, `/v1/messages`, `/metrics` | Dedicated Tokio thread pool strictly for LLM inference (<10 µs latency overhead). |
| **Port 9100** | Web Telemetry & Control Dashboard Cockpit | TCP / HTTP / SSE / WS | Browser HTML/CSS/JS, SSE metrics, DB export | Non-blocking background service. Large browser downloads never choke active inference. |

---

## 3. Disciplinary Coding Standards & Conventions

All agent workers and contributors must adhere to these non-negotiable coding standards across all tasks:

### 3.1 Rust Core Disciplines
1. **Safe Rust (Edition 2024)**:
   - Safe Rust is enforced crate-wide via `#![deny(unsafe_code)]`.
   - The ONLY permissible exceptions are strictly isolated, leaf OS-FFI modules:
     * `src/keyring/os_windows.rs`: Win32 Credential Manager (`advapi32.dll`)
     * `src/env_injector/windows.rs`: Win32 Registry and setting change broadcast (`advapi32.dll`, `user32.dll`)
   - Every `unsafe` block must be accompanied by an explicit `// SAFETY:` rationale documenting invariants.
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
├── projectmem/               # Pure Rust port of ProjectMem (~v0.3.4)
│   ├── mod.rs, schema.rs, store.rs, summary.rs, precheck.rs, redaction.rs, heatmap.rs, global.rs, mcp.rs
│
├── icm/                      # Instant Context Memory SQLite FTS5 BM25 Engine
│   ├── mod.rs, store.rs, schema.rs, recall.rs, mcp.rs
│
├── aimemory/                 # Open Knowledge Format (OKF v0.2) wiki store & mem0
│   ├── mod.rs, wiki.rs, handoff.rs, tiered.rs, mem0_bridge.rs
│
├── codegraph/                # Tree-sitter AST, SCIP, ast-grep, repomix, petgraph blast-radius
│   ├── mod.rs, kernel/, scip.rs, ast_grep.rs, jcodemunch.rs, repomix.rs, walker.rs, graph.rs, cache.rs, mcp.rs
│
├── contextmode/              # Subprocess sandbox runner & SQLite FTS5 search
│   ├── mod.rs, sandbox.rs, filter.rs, fts.rs, fetch.rs, mcp.rs
│
├── headroom/                 # Headroom token compression & Copilot dual-port supervisor
│   ├── mod.rs, copilot_daemon.rs, compression/, pruner/, cache_align.rs, adaptive_mem.rs
│
├── gateway/                  # Enterprise AI Gateway & BYOK Failover Engine (:9099)
│   ├── mod.rs, reconnect.rs, providers/, agent_cli/, cascade.rs, keys.rs, loadbalancer.rs, routing.rs,
│   ├── governance.rs, semantic_cache.rs, normalizer/, compat/, circuit_breaker.rs, proxy.rs
│
├── telemetry/                # Ground-truth session metrics & TokenTracker integration
│   ├── mod.rs, prometheus.rs, collector.rs, tokentracker.rs, cost.rs, tracker.rs, cache_analytics.rs
│
├── rtk/                      # 100+ command output filters & zero-allocation ANSI stripper
│   ├── mod.rs, engine.rs, filters/, ansi.rs, hook.rs
│
├── rules/                    # In-tree embedded rules (Karpathy, Caveman, Ponytail) & universal sync
│   ├── mod.rs, sync.rs, principles.rs, caveman.rs, ponytail.rs, embedded/
│
├── env_injector/             # Non-destructive OS environment injection & legacy binary healing
│   ├── mod.rs, windows.rs, unix.rs, teardown.rs, heal.rs
│
├── dashboard/                # Background Web Telemetry Cockpit on Port 9100
│   ├── mod.rs, state.rs, sse.rs, api.rs, assets/
│
├── tui/                      # Interactive terminal user interface (ratatui + crossterm mouse capture)
│   ├── mod.rs, app.rs, views/, widgets/
│
├── agents/                   # Native configurations for 17 AI coding agents & Open Protocols
│   ├── mod.rs, manager.rs, open_agents.rs, antigravity.rs, copilot.rs, claude.rs, codex.rs, cursor.rs, grok.rs, mcp_adapters.rs
│
├── keyring/                  # Cross-platform secure credential manager (advapi32, Keychain, Secret Service)
│   ├── mod.rs, os_windows.rs, os_macos.rs, os_linux.rs, envelope.rs
│
├── db/                       # Persistent SQLite configuration database (~/.toksi/toksi.db)
│   ├── mod.rs, schema.rs, operations.rs, migrate.rs
│
├── util/                     # Forward-slash path normalization, PATH validation mandate, exec, parsers
│   ├── paths.rs, env_path.rs, exec.rs, jsonc.rs, toml.rs, colors.rs
│
└── commands/                 # Native CLI command dispatcher
    ├── daemon.rs, auth.rs, db.rs, agent.rs, tui.rs, dashboard.rs, mem.rs, graph.rs, ctx.rs, rtk.rs, doctor.rs, mod.rs
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

### Phase 15.2: Unified 5-Port Daemon & Copilot Supervision
- **Objectives**: Supervise all 5 dedicated endpoints simultaneously in `toksi daemon start` with complete latency and socket isolation.
- **Targets**:
  * `crates/toksi/src/commands/daemon.rs`: Master daemon supervisor starting Port 8787 (Headroom), Port 8789 (Copilot VS Code), Port 8790 (Copilot CLI), Port 9099 (AI Gateway & `/metrics`), and Port 9100 (Web Dashboard Cockpit).
  * `crates/toksi/src/headroom/copilot_daemon.rs`: Dual-port supervisor managing 8789 and 8790 with live `/livez` health probes and collision prevention.
  * `crates/toksi/src/gateway/mod.rs` & `dashboard/mod.rs`: Guarantee thread and socket buffer isolation between Port 9099 (inference) and Port 9100 (dashboard).
  * `crates/toksi/src/env_injector/teardown.rs`: Clean teardown of Windows `HKCU\Environment` variables (`WM_SETTINGCHANGE` broadcast) and Unix environment on daemon shutdown.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.3: Smart Agent CLI Providers & Paid Quota Enforcement
- **Objectives**: Register 6 Local Agent CLIs as first-class BYOK providers with smart background launch; enforce strict paid commercial quota mandate.
- **Targets**:
  * `crates/toksi/src/gateway/agent_cli/mod.rs`: Registry for `AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, and `GrokCli`.
  * `crates/toksi/src/gateway/agent_cli/tier_check.rs`: Billing and subscription validator checking paid status for all providers. Disqualify free accounts (unbilled AI Studio trial keys, expired credits, free Copilot trials) from active rotation.
  * Graceful Inform & Zero-Fail: Never panic, crash, or fail. Log `[INFO] Provider '<Name>' is on a Free Tier... Paid quota required... Bypassing from active pool without error.` Fall back gracefully to next paid provider or direct agent execution.
  * `crates/toksi/src/gateway/providers/aggregators/bifrost.rs`: Integrate Bifrost provider client with health probes and failover routing.
  * Activation Safety Guard: Gateway engages when `active_paid_providers >= 1`; graceful passthrough if zero.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging
- **Objectives**: Expose MCP, A2A, ACP, stdio, SSE, and HTTP across all tools; integrate `.gitignore` and dynamic index purging.
- **Targets**:
  * `crates/toksi/src/tools/multi_protocol.rs`: Multi-protocol dispatch for `rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`.
  * `crates/toksi/src/codegraph/walker.rs`, `cache.rs`, and `contextmode/fts.rs`: Integrate ripgrep-grade ignore walker respecting `.gitignore` and `.ignore`.
  * Dynamic Vector & Cache Purging: When ignore rules change, automatically purge matching existing records from SQLite FTS5 search indexes and vector embedding caches.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing
- **Objectives**: Ground-truth session token accounting from live SSE streams; network reconnection auto-healing watchdog; universal rules sync.
- **Targets**:
  * `crates/toksi/src/telemetry/tokentracker.rs` & `collector.rs`: Parse real usage tokens directly from live SSE response streams and upstream response headers (`x-ratelimit-remaining-tokens`, `anthropic-ratelimit-requests-remaining`). Strict prohibition of virtual/synthetic math (counter reads 0 when idle).
  * `crates/toksi/src/gateway/reconnect.rs`: Automatic reconnection watchdog detecting network disconnects, VPN toggles, and socket timeouts. Invalidate stale connection pools and auto-reconnect without entering dormant states.
  * `crates/toksi/src/rules/sync.rs`: Universal synchronizer for rules, skills, and instructions across all 17 agents.
  * `crates/toksi/src/main.rs`: Update `help_text()` with comprehensive details on all commands, flags, tools, ports, and conventions.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 15.6: Advanced Vault Intelligence Engines Consolidation
- **Objectives**: Consolidate remaining reference vaults into pure Rust.
- **Targets**:
  * `TokenTracker` -> `crates/toksi/src/telemetry/tokentracker.rs`
  * `jcodemunch-mcp` -> `crates/toksi/src/codegraph/jcodemunch.rs`
  * `repomix` -> `crates/toksi/src/codegraph/repomix.rs`
  * `ast-grep` -> `crates/toksi/src/codegraph/ast_grep.rs`
  * `scip` -> `crates/toksi/src/codegraph/scip.rs`
  * `mem0` -> `crates/toksi/src/aimemory/mem0_bridge.rs`
  * `chroma` -> `crates/toksi/src/gateway/semantic_cache.rs`
  * `litellm` -> `crates/toksi/src/gateway/normalizer/litellm.rs`
  * `TokenEconomizer` -> `crates/toksi/src/headroom/compression/economizer.rs`
  * `agent-opt` -> `crates/toksi/src/agents/opt.rs`
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 16.1: High-Performance Mouse-Capable Interactive TUI
- **Objectives**: Full mouse interaction and non-blocking asynchronous event loop to eliminate terminal input hangs.
- **Targets**:
  * `crates/toksi/src/tui/mod.rs` & `app.rs`: Enable crossterm `EnableMouseCapture`. Route mouse clicks for tab navigation, entity selection, and action buttons. Mouse wheel scrolling for logs and inspection panels.
  * Event loop: Replace blocking event calls with non-blocking async event polling. Zero keyboard freezes or tab locks.
  * Realistic real-time telemetry: True token consumption, ground-truth savings, and active BYOK quota gauges.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 16.2: Enterprise Web Telemetry Cockpit Dashboard
- **Objectives**: Isolated, non-blocking background web UI on Port 9100.
- **Targets**:
  * `crates/toksi/src/dashboard/mod.rs`, `api.rs`, `state.rs`, `sse.rs`: Embedded HTTP & SSE server on `127.0.0.1:9100` (`toksi dashboard`).
  * `crates/toksi/src/dashboard/assets/`: Single-page dark glassmorphic UI embedded via `include_str!`.
  * Features: 17-agent live observation grid, ground-truth token savings, upstream quota gauges, provider toggle/weights/latency controls, OS Keyring and DB export/import.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings.

### Phase 16.3: Open Agents Directory Protocols & README Verification
- **Objectives**: Support `.agents/` and vendor directory layouts; selective user-level vs workspace installation; technical README rewrite.
- **Targets**:
  * `crates/toksi/src/agents/open_agents.rs` & `manager.rs`: Full support for `.agents/`, `.gemini/`, `.copilot/`, `.claude/`, `.codex/`, `.cursor/`, `.grok/`.
  * Installation policy: Auto-install to user directory (`~/.<agent>/`) if selected; workspace repo installation is strictly optional and user-selected.
  * `crates/toksi/README.md`: Comprehensive rewrite covering 5-port architecture, single daemon launch, 17 agents, 39+ providers, smart agent CLI provider bridges, paid quota enforcement, multi-protocol support, rules sync, and full CLI reference.
- **Verification**: `cargo check -p toksi` and `cargo test -p toksi` pass with 0 warnings. 100% test pass rate.
