# Tokless Native Consolidation Master Architecture Plan (v2.1)
**Project:** `neo/crates/tokless`  
**Status:** Approved Master Architectural Blueprint  
**Objective:** 100% Native Rust consolidation of all token-optimization, memory, code-intelligence, rules, and telemetry engines directly embedded into `crates/tokless/src/*`. Complete removal of external package manager downloads (npm, pip, uv, curl). Pure in-process execution, zero missing code, full feature parity, rich TUI, and the ultimate zero-dependency real-time Telemetry & Control Dashboard.

---

## 1. Core Architectural Mandates

1. **100% Native In-Tree Embedding (`crates/tokless/src/*`)**:
   - Every tool (`projectmem`, `icm`, `aimemory`, `codegraph`, `contextmode`, `headroom`, `rtk`, `rules`) is implemented as an internal Rust module.
   - Tokless agent harnesses, MCP proxies, and CLI commands call internal Rust functions directly via zero-cost Rust abstractions—**never via spawned external tool binaries or shell wrappers**.
2. **Purge of Runtime Downloader / Installer Slop**:
   - Completely remove `npminstall.rs`, `nodewin.rs`, `nodelinux.rs`, `uv.rs`, and Python/pip/uvx bootstrapping code from `tokless`.
   - `tokless` never downloads or installs runtimes. If an external tool is optional (e.g. system `git`, or an optional `python`/`node` interpreter for user scripts), `tokless` only detects it on system `PATH` via `which`.
   - Toolchain installation is cleanly decoupled and reserved for `crates/nbuild`.
3. **In-Tree Rules & Skills (`caveman`, `ponytail`, `principles`)**:
   - Ingest all instructions, guidelines, and rules from `inspect-vault/andrej-karpathy-skills`, `inspect-vault/caveman`, etc., directly into `crates/tokless/src/rules/` and compile them with `include_str!`.
   - Zero external repo dependencies. All rules are distributed with the single `tokless` binary and directly accessible from the project directory.
4. **Zero-Token Waste Memory Foundation**:
   - Zero background LLM re-summarization loops.
   - Sub-10ms local SQLite FTS5 BM25 search (from ICM).
   - Claim-once cross-agent handoff state machine + progressive L0/L1/L2 disclosure (from ai-memory).
   - Append-only structured issue/attempt/fix/decision tracking (from ProjectMem).
5. **Full Observability & Control**:
   - **Full-Featured Interactive TUI (`tokless tui` / `tokless top`)**: Multi-tab live monitoring, agent wire manager, memory explorer, and graph inspector.
   - **Ultimate Telemetry & Control Dashboard (`tokless dashboard`)**: Embedded local web server with zero external dependencies, providing real-time token compression waterfalls, dollar savings counters, error telemetry, agent fleet management, CodeGraph topology visualizer, and memory decision DAGs.

---

## 2. System Directory Topology

```
crates/tokless/src/
├── main.rs                   # Unified CLI entrypoint
├── lib.rs                    # Public crate library API
│
├── projectmem/               # Native Rust Port of ProjectMem (~v0.3.4)
│   ├── mod.rs                # Public API facade
│   ├── schema.rs             # Serde models: Event, Issue, Attempt, Fix, Decision, Note
│   ├── store.rs              # Thread-safe JSONL append with Windows/Unix file locking
│   ├── summary.rs            # Deterministic zero-LLM summary generator
│   ├── precheck.rs           # Git staged diff scanner vs gotchas & past failure locations
│   ├── redaction.rs          # Zero-allocation regex sanitizer (strips keys & tokens)
│   ├── heatmap.rs            # Failure clustering & ROI metrics
│   ├── global.rs             # Cross-project global memory (~/.projectmem)
│   └── mcp.rs                # In-process stdio/proxy MCP server handlers
│
├── icm/                      # Native Rust Port of RTK-AI ICM Engine
│   ├── mod.rs                # ICM subsystem orchestrator
│   ├── store.rs              # Single SQLite DB (`.tokless/memory.db`) with FTS5 BM25
│   ├── schema.rs             # Memory items, topics, sessions, facts
│   ├── recall.rs             # Sub-10ms keyword and recency recall (Zero LLM tokens)
│   └── mcp.rs                # In-process memory MCP endpoints
│
├── aimemory/                 # Native Rust Adaptation of ai-memory (v3.2.8)
│   ├── mod.rs                # ai-memory subsystem orchestrator
│   ├── wiki.rs               # Open Knowledge Format (OKF v0.2) markdown wiki store
│   ├── handoff.rs            # Claim-once multi-agent handoff state machine
│   └── tiered.rs             # L0 (TOC) / L1 (Abstract) / L2 (Full) progressive packer
│
├── codegraph/                # Native Rust Tree-Sitter & Call-Graph Engine (Drop Node.js)
│   ├── mod.rs                # CodeGraph facade
│   ├── kernel/               # Embedded Tree-sitter parsers (Rust, TS/JS, Py, Go, C/C++)
│   ├── walker.rs             # High-speed ripgrep-grade file traversal (`ignore` crate)
│   ├── graph.rs              # Petgraph AST dependency, call-path, and blast-radius
│   ├── cache.rs              # Incremental AST symbol cache (.codegraph/symbols.db)
│   └── mcp.rs                # In-process MCP server for `codegraph_explore`
│
├── contextmode/              # Native Rust Port of Context-Mode (Drop Node.js)
│   ├── mod.rs                # ContextMode facade
│   ├── sandbox.rs            # Subprocess runner with CPU timeout & memory bounds
│   ├── filter.rs             # Output slicing, ANSI stripping, head/tail truncation
│   ├── fts.rs                # SQLite FTS5 document chunker & BM25 search
│   ├── fetch.rs              # Async HTTP fetcher + HTML reader (`reqwest`)
│   └── mcp.rs                # In-process `ctx_*` MCP tools
│
├── headroom/                 # Native Rust Headroom Proxy & Token Compressor
│   ├── mod.rs                # Headroom orchestrator
│   ├── compression/          # Lossless log deduplication, AST code pruner, JSON minifier
│   ├── cache_align.rs        # 1024-token prompt cache alignment engine
│   ├── adaptive_mem.rs       # Dynamic memory brief scaling based on context headroom
│   ├── byok_gateway.rs       # Multi-provider routing (Anthropic, OpenAI, Gemini)
│   ├── router.rs             # Cascade, load-shedding, and quota coordinator
│   └── proxy.rs              # High-throughput HTTP/SSE reverse proxy
│
├── rtk/                      # Native Rust RTK (Rust Token Killer) Engine
│   ├── mod.rs                # RTK filter facade & streaming interceptor
│   ├── engine.rs             # Command parser and filter dispatcher
│   ├── filters/              # 100+ command-specific output filters (git, cargo, test)
│   ├── ansi.rs               # Zero-allocation ANSI escape sequence stripper
│   └── hook.rs               # Terminal hook interceptor
│
├── rules/                    # In-Tree Rules, Skills & Principles (Zero External Repos)
│   ├── mod.rs                # Rule injector and compiler
│   ├── principles.rs         # Karpathy guidelines (think, simplify, surgical, verify)
│   ├── caveman.rs            # ASD-STE100 primitive English compressor
│   ├── ponytail.rs           # Lazy senior developer build discipline ladder
│   └── embedded/             # Compile-time embedded markdown resources (`include_str!`)
│       ├── principles.md
│       ├── caveman.md
│       └── ponytail.md
│
├── dashboard/                # Ultimate Telemetry & Control Web Server (Zero-Dependency)
│   ├── mod.rs                # Embedded local HTTP & SSE/WebSocket server (`127.0.0.1:9099`)
│   ├── state.rs              # Shared in-memory metrics ring buffer and event bus
│   ├── sse.rs                # Server-Sent Events real-time streaming broadcaster
│   ├── api/                  # REST endpoints:
│   │   ├── metrics.rs        # Historical and live token compression & savings
│   │   ├── errors.rs         # 429 backoffs, timeouts, 5xx errors, context overflows
│   │   ├── agents.rs         # Fleet matrix, wire/unwire actions, quota allocations
│   │   ├── memory.rs         # Decision DAG, failure heatmap, event search
│   │   ├── graph.rs          # AST symbols, blast-radius queries, index status
│   │   └── control.rs        # Proxy start/stop, BYOK key swap, cache purge
│   └── ui/                   # Bundled single-page application (Embedded via `include_str!`)
│       ├── index.html        # Clean semantic markup with modern dashboard layout
│       ├── app.js            # Reactive rendering, WebSocket/SSE consumer, Canvas charts
│       └── style.css         # Dark-mode optimized responsive design
│
├── tui/                      # Full-Featured Terminal User Interface (`ratatui` + `crossterm`)
│   ├── mod.rs                # TUI event loop & terminal guard
│   ├── app.rs                # Global TUI state & active tab model
│   ├── views/                # Tab view renderers:
│   │   ├── monitor.rs        # Live token throughput waterfall & compression stats
│   │   ├── agents.rs         # Interactive agent wiring table (Space to wire/unwire)
│   │   ├── memory.rs         # ProjectMem & ICM event viewer, search, and precheck
│   │   ├── graph.rs          # Interactive CodeGraph symbol search & blast-radius
│   │   ├── headroom.rs       # Cache alignment inspector & BYOK quotas
│   │   └── logs.rs           # Diagnostic and daemon error logs
│   └── widgets/              # Gauges, charts, sparklines, tables
│
├── agents/                   # Native configurations for 17 AI coding agents
│   ├── antigravity.rs, claude.rs, cursor.rs, codex.rs, opencode.rs, etc.
│   └── mcp_adapters.rs       # Agent MCP config read/write/patch engine
│
├── util/                     # Pure System & OS Utilities (All Downloaders Purged)
│   ├── paths.rs, exec.rs, jsonc.rs, toml.rs, colors.rs, etc.
│   └── (Purged: npminstall.rs, nodewin.rs, nodelinux.rs, uv.rs)
│
└── commands/                 # Native CLI Command Dispatcher
    ├── mem.rs                # `tokless mem <init|log|attempt|fix|decision|note|summary|precheck>`
    ├── graph.rs              # `tokless graph <explore|impact|symbols|index>`
    ├── ctx.rs                # `tokless ctx <eval|search|index>`
    ├── rtk.rs                # `tokless rtk <command...>`
    ├── tui.rs                # `tokless tui` (Full interactive control center)
    ├── dashboard.rs          # `tokless dashboard` (Web telemetry server)
    ├── daemon.rs, doctor.rs, proxy.rs, prime.rs, top.rs, init.rs
    └── mod.rs
```

---

## 3. The Ultimate Dashboard: Full Capabilities & Specifications

The Tokless Dashboard (`tokless dashboard`, default `http://127.0.0.1:9099`) is a high-performance, single-binary telemetry and control center. It requires **zero Node.js, zero npm, zero external CDNs, and zero build steps**—everything is embedded into the compiled Rust binary.

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│  TOKLESS TELEMETRY & CONTROL                                      [Proxy: UP 8080] [Daemon: OK] [v1.0] │
├───────────────────┬───────────────────┬───────────────────┬───────────────────┬────────────────────────┤
│ TOKENS SAVED      │ DOLLAR SAVINGS    │ AVERAGE SAVINGS   │ PROMPT CACHE RATE │ ACTIVE AGENTS          │
│ 14,892,104        │ $184.26 USD       │ 68.4%             │ 91.2%             │ 4 Wired / 2 Active     │
├───────────────────┴───────────────────┴───────────────────┴───────────────────┴────────────────────────┤
│ [1. Live Token Compression Waterfall]                                                                 │
│ 22:15:02 [Claude]  POST /v1/messages  │ Raw: 62.4k tok ──▶ Compressed: 19.8k tok (-68.2%) │ RTK+Headroom │
│ 22:15:10 [Cursor]  POST /v1/chat      │ Raw: 24.1k tok ──▶ Compressed:  7.2k tok (-70.1%) │ CacheHit 90% │
│ 22:15:18 [Antigrav] POST /v1/chat     │ Raw: 18.5k tok ──▶ Compressed:  5.1k tok (-72.4%) │ L1 Injected  │
├───────────────────────────────────────┬────────────────────────────────────────────────────────────────┤
│ [2. Error & Rate-Limit War Room]      │ [3. Multi-Agent Fleet Matrix]                                  │
│ • 429 Backoffs: 2 (Auto-shedded)      │ Agent         Port   BYOK Provider    Status     Tokens Used   │
│ • Context Overflows: 0 (Governed)     │ Claude Code   8080   Anthropic Direct  [ACTIVE]   4.2M ($63.0)  │
│ • Upstream 503s: 1 (Swapped to Gemini)│ Cursor        8081   OpenAI Direct     [ACTIVE]   1.8M ($18.0)  │
│ • Tool Failures: 1 (Caught by RTK)    │ Antigravity   8082   Gemini 2.0 Flash  [WIRED]    0.9M ($0.9)   │
├───────────────────────────────────────┼────────────────────────────────────────────────────────────────┤
│ [4. Project Memory Decision DAG]      │ [5. CodeGraph Blast-Radius & Topology]                         │
│ [0001: Use SQLite FTS5] ──▶ [ACTIVE]  │ Target: `OrderService::process_payment`                        │
│   ▲                                   │ Direct Callers: 4 functions | Blast Radius: 12 files           │
│   └── [0002: In-Memory Map] (RETIRED) │ Dependency Depth: 3 layers | Affected Tests: 6 test suites     │
└───────────────────────────────────────┴────────────────────────────────────────────────────────────────┘
```

### Full Dashboard Capability Matrix

#### 1. Live Token Compression Waterfall & Real-Time Savings
- **Layer-by-Layer Savings Breakdown**:
  - **RTK Layer**: Bash output savings (terminal outputs reduced by up to 90%).
  - **Headroom Log Layer**: Deduplication metrics (collapsed recurring logs, stack traces preserved).
  - **Headroom AST Layer**: Whitespace, comment, and code pruning metrics.
  - **Rules Layer**: Caveman instruction compression savings.
- **Dynamic Cost Calculator**:
  - Live dollar counter tracking savings across exact model billing tiers (Claude 3.7 Sonnet, GPT-4o, o3-mini, Gemini 2.0 Flash, DeepSeek R1).
- **Prompt Cache Efficiency Gauge**:
  - Visual indicator showing how accurately prompts match the 1,024-token boundary to achieve the 90% discount on cached tokens.

#### 2. The "War Room": Error, Outage & Rate-Limit Telemetry
- **HTTP 429 Rate-Limit Monitor**: Real-time graph of rate-limit backoffs with exponential backoff timers.
- **Provider Fallback Cascade Visualizer**: Displays when an upstream model experiences an outage or 5xx error, tracing how Tokless automatically reroutes requests to standby providers (e.g., Anthropic -> Gemini -> OpenAI).
- **Context Window Overflow Pre-Emption**: Displays how close each active agent is to its context limit (e.g., 200k tokens), showing how the Tokless Headroom Governor scales memory down from L2 to L0 to prevent crashes.
- **Tool Crash Inspector**: Real-time stack trace capture for failed agent tool executions.

#### 3. Multi-Agent Fleet Control Center
- **Interactive Matrix for All 17 Agents**:
  - Live toggle switch to Wire / Unwire any agent (`claude`, `cursor`, `antigravity`, `codex`, `opencode`, `copilot`, `cline`, etc.) with a single click.
  - Active port bindings, assigned BYOK keys, and live socket traffic.
- **Live Agent Session Inspector**:
  - Displays streaming tokens, active tool calls, and response latencies in real time.

#### 4. Project Memory & Decision Graph Visualizer
- **Interactive SVG Decision DAG**:
  - Visual nodes representing architectural choices (`add_decision`).
  - Edge arrows showing `supersedes` relationships. Differentiates active decisions from superseded decisions with click-to-expand rationales.
- **Failure Heatmap Matrix**:
  - Color-coded heatmap showing repositories, directories, and files with the highest failure rates, matching past issues to verify fixes.
- **Live Gotchas & Precheck Monitor**:
  - Real-time alert feed displaying staged git diff warnings before commits are finalized.
- **Instant FTS5 Memory Search**:
  - In-browser search box querying `.tokless/memory.db` via SQLite FTS5 BM25 with sub-10ms response times.

#### 5. CodeGraph Interactive Topology & Blast-Radius Simulator
- **Interactive Symbol Graph Explorer**:
  - Search any function, class, or trait in the repository. Renders an interactive canvas/SVG dependency tree showing callers, callees, and type references.
- **Blast-Radius Impact Calculator**:
  - Select any file or function to simulate the impact of changes across the entire codebase. Visualizes all dependent files and affected unit tests.
- **Incremental Index Telemetry**:
  - Live statistics: number of files indexed, AST nodes parsed, and disk cache utilization.

#### 6. Context-Mode Sandbox Inspector
- **Sandbox Execution Audit**:
  - Live log of all `ctx_execute` scripts run by agents.
  - Side-by-side comparison: Raw Tool Output (e.g. 250 KB) vs. Filtered Stdout injected into the model context (e.g. 3.2 KB).
- **FTS5 Document Chunker Browser**:
  - Inspect indexed session chunks and full-text search rankings.

#### 7. System Supervisor & Telemetry Controls
- **Live Process Health**:
  - Real-time CPU, RAM, active worker threads, and open socket metrics.
- **One-Click Proxy Management**:
  - Start, stop, or restart the Headroom proxy.
  - Dynamic BYOK key swap and model routing adjustments without restarting agent sessions.
- **Cache Purge & Vacuum**:
  - Clean temporary AST caches and vacuum SQLite memory files with one click.

---

## 4. Execution Roadmap

| Phase | Subsystem | Actions | Deliverables |
|---|---|---|---|
| **Phase 1** | **RTK Engine** | Embed 100+ filters into `crates/tokless/src/rtk/`. | Native RTK filter engine, ANSI stripper, CLI command. |
| **Phase 2** | **ProjectMem Port** | Port models, storage, summary, precheck into `crates/tokless/src/projectmem/`. | Native `tokless mem` CLI, in-process MCP server. |
| **Phase 3** | **ContextMode Port** | Port sandbox runner, FTS5 chunker, fetcher into `crates/tokless/src/contextmode/`. | Native `ctx_*` MCP tools, RTK sandbox piping. |
| **Phase 4** | **ICM & ai-memory** | Embed SQLite FTS5 store, claim-once handoffs, L0/L1/L2 briefs in `crates/tokless/src/icm/` & `aimemory/`. | Sub-10ms zero-LLM memory, multi-agent handoffs. |
| **Phase 5** | **CodeGraph Engine** | Integrate Tree-sitter parsers, petgraph blast radius in `crates/tokless/src/codegraph/`. | Sub-millisecond code intelligence without Node.js. |
| **Phase 6** | **Rules & Skills** | Embed Karpathy principles, Caveman, Ponytail into `crates/tokless/src/rules/`. | In-tree rules compiled with `include_str!`. |
| **Phase 7** | **Headroom Deep Integration** | Port compression, 1024-token cache aligner, adaptive governor to `crates/tokless/src/headroom/`. | Streaming compression, BYOK proxy, 50–80% token cut. |
| **Phase 8** | **Purge Downloader Slop** | Remove `npminstall.rs`, `nodewin.rs`, `nodelinux.rs`, `uv.rs`. Rewire all tools to native modules. | Clean `crates/tokless/src/util/`, zero external tool dependencies. |
| **Phase 9** | **Full-Featured TUI** | Implement `ratatui` + `crossterm` multi-tab interface in `crates/tokless/src/tui/`. | Interactive control center via `tokless tui` / `tokless top`. |
| **Phase 10** | **Ultimate Dashboard** | Build embedded zero-dependency local web dashboard in `crates/tokless/src/dashboard/`. | Real-time browser dashboard via `tokless dashboard` with full telemetry & controls. |
| **Phase 11** | **System Verification & Benchmarks** | End-to-end integration tests across all 17 agent harnesses. | 0 compiler warnings, 100% tests passing, verified token savings. |

---

## 5. Ready-to-Run Prompts for Each Phase

Use these prompts when starting a new chat for each phase:

### Phase 1: RTK Engine Integration
```text
Execute Phase 1 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and embed neo\inspect-vault\rtk into crates/tokless/src/rtk.
Embed 100+ command-specific output filters (git, cargo, test runners, system tools) and zero-allocation ANSI stripper.
Adapt and connect to tokless CLI and proxy pipelines.
Ensure full functional parity, zero missing filter code, zero compiler warnings, and passing unit tests.
```

### Phase 2: ProjectMem Pure Rust Port
```text
Execute Phase 2 of crates/tokless/CONSOLIDATION_PLAN.md.
Full convert and port Python neo\inspect-vault\projectmem into pure Rust in crates/tokless/src/projectmem.
Implement schema (Event/Issue/Attempt/Fix/Decision/Note), thread-safe JSONL store with cross-platform file locking, deterministic summary generator, git precheck, credential redactor, failure heatmap, and in-process MCP server.
Drop Python/pip/uvx dependencies completely. Wire into tokless mem CLI.
Ensure full functional parity with Python projectmem, zero missing code, zero compiler warnings, and passing unit tests.
```

### Phase 3: Context-Mode Pure Rust Port
```text
Execute Phase 3 of crates/tokless/CONSOLIDATION_PLAN.md.
Full convert and port TypeScript neo\inspect-vault\context-mode into pure Rust in crates/tokless/src/contextmode.
Implement sandbox subprocess runner (with CPU/memory/time bounds), output slicer/filter, SQLite FTS5 chunker & BM25 search, async reqwest fetcher, and in-process ctx_* MCP server.
Pipe execution output through crates/tokless/src/rtk filters. Drop Node.js dependency completely.
Ensure full functional parity, zero missing code, zero compiler warnings, and passing unit tests.
```

### Phase 4: ICM & ai-memory Integration
```text
Execute Phase 4 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and integrate neo\inspect-vault\icm into crates/tokless/src/icm and neo\inspect-vault\ai-memory into crates/tokless/src/aimemory.
From ICM: embed single-file SQLite FTS5 BM25 memory store (zero-LLM token cost on store/recall, sub-10ms latency).
From ai-memory: embed claim-once multi-agent handoff state machine and tiered L0/L1/L2 progressive disclosure packer.
Ensure strict zero-LLM background token spend, zero compiler warnings, and passing unit tests.
```

### Phase 5: CodeGraph Pure Rust Engine
```text
Execute Phase 5 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and embed neo\inspect-vault\codegraph into pure Rust in crates/tokless/src/codegraph.
Integrate Tree-sitter parsers (Rust, TS/JS, Python, Go, C/C++), ripgrep-grade file walker (ignore crate), petgraph AST call-graph/blast-radius engine, incremental disk cache, and in-process MCP server (codegraph_explore).
Drop Node.js wrapper completely. Wire into tokless graph CLI.
Ensure full functional parity, zero missing code, zero compiler warnings, and passing unit tests.
```

### Phase 6: In-Tree Rules & Skills Integration
```text
Execute Phase 6 of crates/tokless/CONSOLIDATION_PLAN.md.
Integrate rules and skills from neo\inspect-vault\andrej-karpathy-skills and neo\inspect-vault\caveman directly into crates/tokless/src/rules/.
Implement in-tree compile-time embedding via include_str! for Karpathy principles, Caveman ASD-STE100 English compressor, and Ponytail build discipline ladder.
Remove all external repository clone links. Make rules distributable inside the tokless binary.
Ensure zero compiler warnings and passing unit tests.
```

### Phase 7: Headroom Deep Integration
```text
Execute Phase 7 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and deep integration of neo\inspect-vault\headroom into crates/tokless/src/headroom.
Implement lossless log deduplication, AST-aware code compression, JSON pruner, 1024-token prompt cache alignment, and dynamic memory governor (scaling L0 vs L1 vs L2 based on context headroom).
Unify with BYOK gateway and streaming reverse proxy. Drop Python headroom-py.
Coordinate with crates/tokless/src/{rtk, projectmem, icm, contextmode}.
Ensure zero compiler warnings and passing unit tests.
```

### Phase 8: Downloader Purge & System Refactor
```text
Execute Phase 8 of crates/tokless/CONSOLIDATION_PLAN.md.
Completely purge runtime downloader/installer modules from crates/tokless/src/util/ (npminstall.rs, nodewin.rs, nodelinux.rs, uv.rs, projectmem.rs).
Rewire all tokless agent harnesses, MCP adapters, and CLI commands to call native in-tree modules directly.
Ensure system binary detection uses which on system PATH only. No runtime package installations.
Ensure zero compiler warnings, zero unused dead code, and all tests passing.
```

### Phase 9: Full-Featured Terminal User Interface (TUI)
```text
Execute Phase 9 of crates/tokless/CONSOLIDATION_PLAN.md.
Implement the full multi-tab interactive TUI in crates/tokless/src/tui/ using ratatui and crossterm.
Build views for: Live Token Monitor & Waterfall, Multi-Agent Wire Manager (Space to wire/unwire), Memory Explorer & Precheck, CodeGraph Visualizer, Headroom BYOK & Cache Inspector, and Diagnostic Logs.
Wire into tokless tui and tokless top CLI commands.
Ensure smooth 60fps rendering, responsive keybindings, zero compiler warnings, and passing tests.
```

### Phase 10: Telemetry & Control Web Dashboard
```text
Execute Phase 10 of crates/tokless/CONSOLIDATION_PLAN.md.
Implement the embedded zero-dependency Telemetry & Control Web Dashboard in crates/tokless/src/dashboard/.
Build local HTTP/SSE server (127.0.0.1:9099) with bundled SPA (HTML/CSS/JS embedded via include_str!).
Implement live token waterfall, real-time dollar cost savings counter, error telemetry (429s, outages, context overflows), agent fleet matrix with wire/unwire toggle, and SVG decision DAG visualizer.
Wire into tokless dashboard CLI command.
Ensure zero external npm/node build dependencies, zero compiler warnings, and passing tests.
```

### Phase 11: End-to-End System Verification & Benchmarks
```text
Execute Phase 11 of crates/tokless/CONSOLIDATION_PLAN.md.
Run full end-to-end integration verification across all 17 supported agent harnesses (Claude Code, Cursor, Codex, OpenCode, Antigravity, etc.).
Verify in-process MCP multiplexer, unified tokless CLI commands (mem, graph, ctx, rtk, proxy, doctor, tui, dashboard), and streaming proxy compression.
Run cargo test --workspace, ensure 0 compiler warnings, 0 test failures, and verify end-to-end token reduction benchmarks.
```

---

## 5. Ready-to-Run Prompts for Each Phase

Use these prompts when starting a new chat for each phase:

### Phase 1: RTK Engine Integration
```text
Execute Phase 1 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and embed neo\inspect-vault\rtk into crates/tokless/src/rtk.
Embed 100+ command-specific output filters (git, cargo, test runners, system tools) and zero-allocation ANSI stripper.
Adapt and connect to tokless CLI and proxy pipelines.
Ensure full functional parity, zero missing filter code, zero compiler warnings, and passing unit tests.
```

### Phase 2: ProjectMem Pure Rust Port
```text
Execute Phase 2 of crates/tokless/CONSOLIDATION_PLAN.md.
Full convert and port Python neo\inspect-vault\projectmem into pure Rust in crates/tokless/src/projectmem.
Implement schema (Event/Issue/Attempt/Fix/Decision/Note), thread-safe JSONL store with cross-platform file locking, deterministic summary generator, git precheck, credential redactor, failure heatmap, and in-process MCP server.
Drop Python/pip/uvx dependencies completely. Wire into tokless mem CLI.
Ensure full functional parity with Python projectmem, zero missing code, zero compiler warnings, and passing unit tests.
```

### Phase 3: Context-Mode Pure Rust Port
```text
Execute Phase 3 of crates/tokless/CONSOLIDATION_PLAN.md.
Full convert and port TypeScript neo\inspect-vault\context-mode into pure Rust in crates/tokless/src/contextmode.
Implement sandbox subprocess runner (with CPU/memory/time bounds), output slicer/filter, SQLite FTS5 chunker & BM25 search, async reqwest fetcher, and in-process ctx_* MCP server.
Pipe execution output through crates/tokless/src/rtk filters. Drop Node.js dependency completely.
Ensure full functional parity, zero missing code, zero compiler warnings, and passing unit tests.
```

### Phase 4: ICM & ai-memory Integration
```text
Execute Phase 4 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and integrate neo\inspect-vault\icm into crates/tokless/src/icm and neo\inspect-vault\ai-memory into crates/tokless/src/aimemory.
From ICM: embed single-file SQLite FTS5 BM25 memory store (zero-LLM token cost on store/recall, sub-10ms latency).
From ai-memory: embed claim-once multi-agent handoff state machine and tiered L0/L1/L2 progressive disclosure packer.
Ensure strict zero-LLM background token spend, zero compiler warnings, and passing unit tests.
```

### Phase 5: CodeGraph Pure Rust Engine
```text
Execute Phase 5 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and embed neo\inspect-vault\codegraph into pure Rust in crates/tokless/src/codegraph.
Integrate Tree-sitter parsers (Rust, TS/JS, Python, Go, C/C++), ripgrep-grade file walker (ignore crate), petgraph AST call-graph/blast-radius engine, incremental disk cache, and in-process MCP server (codegraph_explore).
Drop Node.js wrapper completely. Wire into tokless graph CLI.
Ensure full functional parity, zero missing code, zero compiler warnings, and passing unit tests.
```

### Phase 6: In-Tree Rules & Skills Integration
```text
Execute Phase 6 of crates/tokless/CONSOLIDATION_PLAN.md.
Integrate rules and skills from neo\inspect-vault\andrej-karpathy-skills and neo\inspect-vault\caveman directly into crates/tokless/src/rules/.
Implement in-tree compile-time embedding via include_str! for Karpathy principles, Caveman ASD-STE100 English compressor, and Ponytail build discipline ladder.
Remove all external repository clone links. Make rules distributable inside the tokless binary.
Ensure zero compiler warnings and passing unit tests.
```

### Phase 7: Headroom Deep Integration
```text
Execute Phase 7 of crates/tokless/CONSOLIDATION_PLAN.md.
Full port and deep integration of neo\inspect-vault\headroom into crates/tokless/src/headroom.
Implement lossless log deduplication, AST-aware code compression, JSON pruner, 1024-token prompt cache alignment, and dynamic memory governor (scaling L0 vs L1 vs L2 based on context headroom).
Unify with BYOK gateway and streaming reverse proxy. Drop Python headroom-py.
Coordinate with crates/tokless/src/{rtk, projectmem, icm, contextmode}.
Ensure zero compiler warnings and passing unit tests.
```

### Phase 8: Downloader Purge & System Refactor
```text
Execute Phase 8 of crates/tokless/CONSOLIDATION_PLAN.md.
Completely purge runtime downloader/installer modules from crates/tokless/src/util/ (npminstall.rs, nodewin.rs, nodelinux.rs, uv.rs, projectmem.rs).
Rewire all tokless agent harnesses, MCP adapters, and CLI commands to call native in-tree modules directly.
Ensure system binary detection uses which on system PATH only. No runtime package installations.
Ensure zero compiler warnings, zero unused dead code, and all tests passing.
```

### Phase 9: Full-Featured Terminal User Interface (TUI)
```text
Execute Phase 9 of crates/tokless/CONSOLIDATION_PLAN.md.
Implement the full multi-tab interactive TUI in crates/tokless/src/tui/ using ratatui and crossterm.
Build views for: Live Token Monitor & Waterfall, Multi-Agent Wire Manager (Space to wire/unwire), Memory Explorer & Precheck, CodeGraph Visualizer, Headroom BYOK & Cache Inspector, and Diagnostic Logs.
Wire into tokless tui and tokless top CLI commands.
Ensure smooth 60fps rendering, responsive keybindings, zero compiler warnings, and passing tests.
```

### Phase 10: Telemetry & Control Web Dashboard
```text
Execute Phase 10 of crates/tokless/CONSOLIDATION_PLAN.md.
Implement the embedded zero-dependency Telemetry & Control Web Dashboard in crates/tokless/src/dashboard/.
Build local HTTP/SSE server (127.0.0.1:9099) with bundled SPA (HTML/CSS/JS embedded via include_str!).
Implement live token waterfall, real-time dollar cost savings counter, error telemetry (429s, outages, context overflows), agent fleet matrix with wire/unwire toggle, and SVG decision DAG visualizer.
Wire into tokless dashboard CLI command.
Ensure zero external npm/node build dependencies, zero compiler warnings, and passing tests.
```

### Phase 11: End-to-End System Verification & Benchmarks
```text
Execute Phase 11 of crates/tokless/CONSOLIDATION_PLAN.md.
Run full end-to-end integration verification across all 17 supported agent harnesses (Claude Code, Cursor, Codex, OpenCode, Antigravity, etc.).
Verify in-process MCP multiplexer, unified tokless CLI commands (mem, graph, ctx, rtk, proxy, doctor, tui, dashboard), and streaming proxy compression.
Run cargo test --workspace, ensure 0 compiler warnings, 0 test failures, and verify end-to-end token reduction benchmarks.
```

