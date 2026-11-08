# Toksi Native Consolidation: Phase Index & Execution Guide (v3.4)

> **Zero Token-Waste Execution Policy**:  
> To prevent context saturation and token budget exhaustion, **run one phase per chat session**.  
> Each file in this directory contains a complete, self-contained **Ready-to-Run Chat Prompt** with all necessary context, target paths, dependencies, and verification criteria. Copy the prompt block at the top of the phase file directly into a new chat session.

---

## Phase Directory Index

| Phase File | Subsystem | Focus | Status |
|---|---|---|---|
| [`PHASE_01_RTK_FILTER_ENGINE.md`](./PHASE_01_RTK_FILTER_ENGINE.md) | RTK Filter Engine | 100+ command output filters, ANSI stripper, streaming token killer pipeline | Implemented & Tested |
| [`PHASE_02_PROJECTMEM_PURE_RUST.md`](./PHASE_02_PROJECTMEM_PURE_RUST.md) | ProjectMem Port | Pure Rust port of Python ProjectMem (~v0.3.4), JSONL file locking, MCP server | Implemented & Tested |
| [`PHASE_03_CONTEXTMODE_PURE_RUST.md`](./PHASE_03_CONTEXTMODE_PURE_RUST.md) | Context-Mode Port | Subprocess sandbox, FTS5 document chunker & search, async web fetcher | Implemented & Tested |
| [`PHASE_04_ICM_AND_AIMEMORY.md`](./PHASE_04_ICM_AND_AIMEMORY.md) | ICM & ai-memory | SQLite FTS5 BM25 memory, claim-once cross-agent handoffs, tiered L0/L1/L2 briefs | Implemented & Tested |
| [`PHASE_05_CODEGRAPH_PURE_RUST.md`](./PHASE_05_CODEGRAPH_PURE_RUST.md) | CodeGraph Engine | Tree-sitter AST parsers (Rust, TS/JS, Py, Go, C/C++), petgraph blast radius | Implemented & Tested |
| [`PHASE_06_IN_TREE_RULES_AND_SKILLS.md`](./PHASE_06_IN_TREE_RULES_AND_SKILLS.md) | In-Tree Rules & Skills | Compile-time embedded guidelines (`include_str!`): Karpathy, Caveman, Ponytail | Implemented & Tested |
| [`PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md`](./PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md) | Headroom & Gateway | Lossless log/JSON/code compression, MCP schema pruner, 39+ BYOK providers | Implemented & Tested |
| [`PHASE_08_DOWNLOADER_PURGE.md`](./PHASE_08_DOWNLOADER_PURGE.md) | Downloader Purge | Purge npminstall, nodewin, uv, python runtimes; native in-tree compilation | Implemented & Tested |
| [`PHASE_09_LEGACY_HEALING_UNHIJACK.md`](./PHASE_09_LEGACY_HEALING_UNHIJACK.md) | Legacy Healing | Restore `*.real` binaries, delete shims/batch wrappers (`heal.rs`) | Implemented & Tested |
| [`PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md`](./PHASE_10_NON_DESTRUCTIVE_ENV_INJECTION.md) | Environment Injection | `HKCU\Environment` management on Windows with `WM_SETTINGCHANGE` broadcast | Implemented & Tested |
| [`PHASE_11_AGENT_DOCTOR_SELF_HEALING.md`](./PHASE_11_AGENT_DOCTOR_SELF_HEALING.md) | Agent Doctor | Verify & auto-heal all 17 agent configurations (`toksi doctor --fix`) | Implemented & Tested |
| [`PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md`](./PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md) | TUI & Dashboard | Native Prometheus `/metrics`, async collector, multi-tab TUI, embedded web UI | Implemented & Tested |
| [`PHASE_13_E2E_VERIFICATION_BENCHMARKS.md`](./PHASE_13_E2E_VERIFICATION_BENCHMARKS.md) | E2E Verification | 100% test pass rate, 0 warnings, token reduction and latency benchmarks | Implemented & Tested |
| [`PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md`](./PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md) | Keyring & SQLite DB | Native OS Keyring (`advapi32.dll`, Keychain, Secret Service), SQLite config DB | Implemented & Tested |
| [`PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md`](./PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md) | Workspace & Standards | Forward-slash `/` paths, `~/.toksi/toksi.db`, single root `.toksi` consolidation, tool layout, hook syntax (`<tool>-hook <agent>`), MCP renaming, PATH validation mandate | **ACTIVE PHASE (1/9)** |
| [`PHASE_15_2_UNIFIED_5_PORT_DAEMON.md`](./PHASE_15_2_UNIFIED_5_PORT_DAEMON.md) | 5-Port Daemon & Copilot | 5-port supervisor (:8787, :8789, :8790, :9099, :9100), Copilot dual-port verification, latency isolation | **ACTIVE PHASE (2/9)** |
| [`PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md`](./PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md) | Smart CLI & Paid Quota | Local Agent CLI providers (`agy`, `copilot`, `claude`, `cursor`, `grok`), smart launch, paid tier mandate, graceful bypass, Bifrost provider | **ACTIVE PHASE (3/9)** |
| [`PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md`](./PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md) | Multi-Protocol & Ignore Purge | MCP, A2A, ACP, stdio, SSE, HTTP across all tools; `.gitignore`/`.ignore` support with dynamic vector & embedding cache purge | **ACTIVE PHASE (4/9)** |
| [`PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md`](./PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md) | Token Accounting & Auto-Heal | Ground-truth session tokens, real SSE usage/quotas, zero virtual math, network reconnect auto-recovery watchdog, rules sync | **ACTIVE PHASE (5/9)** |
| [`PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md`](./PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md) | Vault Intelligence Engines | Consolidation of TokenTracker, jcodemunch-mcp, Repomix, ast-grep, SCIP, mem0, Chroma, LiteLLM | **ACTIVE PHASE (6/9)** |
| [`PHASE_16_1_MOUSE_CAPABLE_TUI.md`](./PHASE_16_1_MOUSE_CAPABLE_TUI.md) | Mouse-Capable TUI | Crossterm `EnableMouseCapture`, tab clicks, row selection, action buttons, scrollable views, fix input locks & tab freezes | **ACTIVE PHASE (7/9)** |
| [`PHASE_16_2_WEB_COCKPIT_DASHBOARD.md`](./PHASE_16_2_WEB_COCKPIT_DASHBOARD.md) | Web Cockpit Dashboard | Background Web UI on `:9100`, 17-agent live observation grid, realistic savings, quota gauges, routing/loadbalancer controls | **ACTIVE PHASE (8/9)** |
| [`PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md`](./PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md) | Open Agents & Verification | Support `.agents/` and vendor layouts, selective user vs workspace auto-install, comprehensive technical README, 100% tests pass | **ACTIVE PHASE (9/9)** |

---

## How to Execute

1. Open a **New Chat** session in Antigravity or your AI agent.
2. Open the desired phase file from this directory.
3. Copy the **Ready-to-Run Chat Prompt** block at the top of the file.
4. Paste it into the new chat and press Send.
5. The agent will execute that phase cleanly without carrying tokens from previous conversations.
