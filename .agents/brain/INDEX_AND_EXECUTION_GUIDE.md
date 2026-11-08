# Toksi Native Consolidation: Phase Index & Execution Guide (v3.4)

> **Zero Token-Waste Execution Policy**:  
> To prevent context saturation and token budget exhaustion, **run one phase per chat session**.  
> Each file in this directory contains a complete, self-contained **Ready-to-Run Chat Prompt** with all necessary context, upstream reference vaults, target paths, dependencies, disciplinary standards, portability invariants, and verification criteria. Copy the prompt block at the top of the phase file directly into a new chat session.

---

## 1. Disciplinary Coding Standards & Conventions Mandate

All worker agents executing any phase in this roadmap must adhere strictly to these non-negotiable standards:
1. **Safe Rust (Edition 2024)**: `#![deny(unsafe_code)]` enforced crate-wide. Unsafe code permitted exclusively in isolated OS FFI leaf modules (`src/keyring/os_windows.rs`, `src/env_injector/windows.rs`) with mandatory `// SAFETY:` rationale.
2. **Zero Warnings Policy**: Must compile cleanly under `RUSTFLAGS="-D warnings"` with zero compiler warnings and zero Clippy lints (`cargo clippy -p toksi --all-targets -- -D warnings`).
3. **Strict Error Handling**: Zero panics, zero `.unwrap()` or `.expect()` in production library and daemon code. Typed domain errors via `thiserror`; CLI errors via `anyhow::Result` with `.context()`.
4. **Universal Forward-Slash (`/`) Path Standard**: Strict prohibition of backward slashes (`\`) across Windows, Linux, and macOS. Escaped `\\` allowed exclusively in `cmd.exe` command strings in JSON5/TOML configs (`hooks.json`).
5. **Environment PATH Verification Mandate ("Rule of Test and Inform User")**: For all external binaries, tools, shells, and agent CLIs, test their presence in system `PATH`. Never encode absolute paths. If missing, log clean actionable remediation instruction.
6. **Continuous Reference Vault Re-Audit**: All 20+ reference vaults in `neo/inspect-vault/*` must ALWAYS be treated as updated/new code to inspect, convert, and import, verifying against missing features, addons, updates, new agents, and bugfixes.
7. **Single Root `.toksi` Directory**: Workspace root contains the only `.toksi` directory. Subfolder `.toksi` folders are auto-discovered, table rows merged into root SQLite DBs, and subfolders deleted.
8. **Consolidated Sub-Tool Layout**: `.toksi/{codegraph, headroom, projectmem, context, icm, ai-memory, rtk}` in user home and workspace root.
9. **Multi-Protocol Standardization**: Every tool exposes MCP, A2A, ACP, stdio, SSE, and HTTP interfaces.
10. **Strict Paid Quota Enforcement**: All BYOK providers must have active paid commercial tiers. Free-tier accounts are gracefully disqualified without failure, logging an informative notice and falling back cleanly.

---

## 2. Master Phase Directory Index

| Phase File | Subsystem | Focus & Key Deliverable | Status |
|---|---|---|---|
| [`PHASE_01_RTK_FILTER_ENGINE.md`](./PHASE_01_RTK_FILTER_ENGINE.md) | RTK Filter Engine | 100+ command output filters, ANSI stripper, streaming token killer pipeline, root-cause failure extractor | Implemented & Tested |
| [`PHASE_02_PROJECTMEM_PURE_RUST.md`](./PHASE_02_PROJECTMEM_PURE_RUST.md) | ProjectMem Port | Pure safe Rust port of ProjectMem (~v0.3.4), JSONL file locking (`fs2`), deterministic summary, precheck, MCP server | Implemented & Tested |
| [`PHASE_03_CONTEXTMODE_PURE_RUST.md`](./PHASE_03_CONTEXTMODE_PURE_RUST.md) | Context-Mode Port | Subprocess sandbox, CPU/memory limits, FTS5 document chunker, async fetcher, SSRF per-hop guard, `ctx_*` MCP suite | Implemented & Tested |
| [`PHASE_04_ICM_AND_AIMEMORY.md`](./PHASE_04_ICM_AND_AIMEMORY.md) | ICM & ai-memory | SQLite FTS5 BM25 memory (`~/.toksi/icm/memory.db`), claim-once cross-agent handoffs, tiered L0/L1/L2 briefs | Implemented & Tested |
| [`PHASE_05_CODEGRAPH_PURE_RUST.md`](./PHASE_05_CODEGRAPH_PURE_RUST.md) | CodeGraph Engine | Tree-sitter AST parsers (Rust, TS/JS, Py, Go, C/C++), petgraph blast radius, AST symbol cache, suffix coordinate resolution | Implemented & Tested |
| [`PHASE_06_IN_TREE_RULES_AND_SKILLS.md`](./PHASE_06_IN_TREE_RULES_AND_SKILLS.md) | In-Tree Rules & Skills | Compile-time embedded guidelines (`include_str!`): Karpathy principles, Caveman ASD-STE100, Ponytail build discipline | Implemented & Tested |
| [`PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md`](./PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md) | Headroom & Gateway | Lossless log/JSON/code compression, MCP schema pruner (saves 5k-15k tokens/turn), 39+ BYOK providers, semantic cache | Implemented & Tested |
| [`PHASE_08_DOWNLOADER_PURGE.md`](./PHASE_08_DOWNLOADER_PURGE.md) | Downloader Purge | Purge npminstall, nodewin, uv, python runtimes; 100% native in-tree compilation with zero package manager dependencies | Implemented & Tested |
| [`PHASE_09_LEGACY_HEALING_UNHIJACK.md`](./PHASE_09_LEGACY_HEALING_UNHIJACK.md) | Legacy Healing | Restore `*.real` binaries, delete shims and batch wrappers (`heal.rs`), clean environment | Implemented & Tested |
| [`PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md`](./PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md) | Environment Injection | `HKCU\Environment` management on Windows with `WM_SETTINGCHANGE` broadcast; Unix session env; RAII `TeardownGuard` | Implemented & Tested |
| [`PHASE_11_AGENT_DOCTOR_SELF_HEALING.md`](./PHASE_11_AGENT_DOCTOR_SELF_HEALING.md) | Agent Doctor | Verify & auto-heal all 17 agent configurations (`toksi doctor --fix`), purge broken legacy MCP runners, sync rules | Implemented & Tested |
| [`PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md`](./PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md) | TUI & Dashboard | Native Prometheus `/metrics`, async zero-latency collector, multi-tab TUI, embedded web dashboard | Implemented & Tested |
| [`PHASE_13_E2E_VERIFICATION_BENCHMARKS.md`](./PHASE_13_E2E_VERIFICATION_BENCHMARKS.md) | E2E Verification | 100% test pass rate, 0 compiler warnings, proven token reduction (30-70%) and latency benchmarks | Implemented & Tested |
| [`PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md`](./PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md) | Keyring & SQLite DB | Native OS Keyring (`advapi32.dll`, Keychain, Secret Service), SQLite config DB (`~/.toksi/toksi.db`), OAuth envelope parser | Implemented & Tested |
| [`PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md`](./PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md) | Workspace & Standards | Forward-slash `/` paths, `~/.toksi/toksi.db`, single root `.toksi` consolidation, tool layout, hook syntax (`<tool>-hook <agent>`), MCP renaming, PATH validation mandate | **ACTIVE PHASE (1/9)** |
| [`PHASE_15_2_UNIFIED_5_PORT_DAEMON.md`](./PHASE_15_2_UNIFIED_5_PORT_DAEMON.md) | 5-Port Daemon & Copilot | 5-port supervisor (:8787, :8789, :8790, :9099, :9100), Copilot dual-port verification, latency and thread pool isolation | **ACTIVE PHASE (2/9)** |
| [`PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md`](./PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md) | Smart CLI & Paid Quota | Local Agent CLI providers (`agy`, `copilot`, `claude`, `codex`, `cursor`, `grok`), smart launch, paid tier mandate, graceful bypass, Bifrost provider | **ACTIVE PHASE (3/9)** |
| [`PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md`](./PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md) | Multi-Protocol & Ignore Purge | MCP, A2A, ACP, stdio, SSE, HTTP across all tools; `.gitignore`/`.ignore` support with dynamic vector & embedding cache purge | **ACTIVE PHASE (4/9)** |
| [`PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md`](./PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md) | Token Accounting & Auto-Heal | Ground-truth session tokens, real SSE usage/quotas, zero virtual math, network reconnect auto-recovery watchdog, rules sync | **ACTIVE PHASE (5/9)** |
| [`PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md`](./PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md) | Vault Intelligence Engines | Consolidation of TokenTracker, jcodemunch-mcp, Repomix, ast-grep, SCIP, mem0, Chroma, LiteLLM | **ACTIVE PHASE (6/9)** |
| [`PHASE_16_1_MOUSE_CAPABLE_TUI.md`](./PHASE_16_1_MOUSE_CAPABLE_TUI.md) | Mouse-Capable TUI | Crossterm `EnableMouseCapture`, tab clicks, row selection, action buttons, scrollable views, fix input locks & tab freezes | **ACTIVE PHASE (7/9)** |
| [`PHASE_16_2_WEB_COCKPIT_DASHBOARD.md`](./PHASE_16_2_WEB_COCKPIT_DASHBOARD.md) | Web Cockpit Dashboard | Background Web UI on `:9100`, 17-agent live observation grid, realistic savings, quota gauges, routing/loadbalancer controls | **ACTIVE PHASE (8/9)** |
| [`PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md`](./PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md) | Open Agents & Verification | Support `.agents/` and vendor layouts, selective user vs workspace auto-install, comprehensive technical README, 100% tests pass | **ACTIVE PHASE (9/9)** |

---

## 3. How to Execute

1. Open a **New Chat** session in Antigravity or your AI agent.
2. Open the desired phase file from this directory (`.agents/brain/PHASE_*.md`).
3. Copy the **Ready-to-Run Chat Prompt** code block at the top of the file.
4. Paste it directly into the new chat and press Send.
5. The agent will execute that phase cleanly with full architectural awareness and zero cross-phase token pollution.
