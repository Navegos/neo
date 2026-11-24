# Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing.
Targets:
  - crates/toksi/src/telemetry/tokentracker.rs
  - crates/toksi/src/telemetry/collector.rs
  - crates/toksi/src/gateway/reconnect.rs
  - crates/toksi/src/rules/sync.rs
  - crates/toksi/src/main.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Ground-Truth Accounting: Real tokens parsed from live SSE streams. Strict prohibition of synthetic/virtual tokens (counter reads exactly 0 when idle).
4. Network Healing Watchdog: Automatically recover from connection drops, VPN disconnects, and socket timeouts without dormant process locking.
5. Comprehensive Usage Help: Detailed CLI `--help` text documenting all commands, ports, flags, tools, and conventions.

Requirements:
1. Ground-Truth Session Token Accounting (`telemetry/{tokentracker,collector}.rs` + `headroom/proxy.rs`):
   - Parse actual prompt, completion, reasoning, `cache_creation_input_tokens`, Responses API usage, and Google `candidatesTokenCount` directly from live SSE streams and non-stream responses (47 `TokenTracker` parsers).
   - Extract upstream provider rate limits and quotas from HTTP response headers (`x-ratelimit-remaining-*`, `anthropic-ratelimit-*`) into `quota_windows` (`hour|day|week|month|year`).
   - Calculate true headroom token savings as exact compressed delta (`raw_tokens - compressed_tokens`) and exact pruned MCP schema tokens (`0` when idle; zero synthetic placeholders).
   - Query `/v1/models` and free provider quota endpoints without spending inference tokens.
2. Network Reconnection Auto-Healing Watchdog (`crates/toksi/src/gateway/reconnect.rs`):
   - Detects connection drops, DNS invalidation, sleep/wake transitions, or VPN interface toggles.
   - Atomically swaps `ArcSwap<reqwest::Client>` pools and closes/re-arms stale connections without requiring daemon restarts.
   - Preserves HTTP/2 ConnectRPC framing & streaming headers cleanly (resolving `#0033`/`#0034`).
3. Universal Rules, Skills & Instructions Sync (`rules/sync.rs` + `agents/manager.rs`):
   - Sync externalized `<enforce>` `Karpathy`, `Caveman` v4, and `Ponytail` guidelines (staged outside `crates/toksi/` in `<neo>/agents/` & `<neo>/rules/`, zero binary bloat) across ONLY user-configured/selected agents (`18+` built-in + `.agents` + Category 1–5) with granular component toggles (`hooks|mcp|skills|rules|instructions|configs`), respecting `user` auto-install vs explicit `repo` opt-in.
4. Comprehensive CLI Help (`crates/toksi/src/main.rs`):
   - Update `help_text()` with exhaustive documentation of all commands (`chat|run|exec|swarm`, `daemon|proxy|dashboard start|stop|restart --agents`, `config`, `byok`, `db`, `auth`, `security`, `compile-blind`, `advisory`, `skills`, `sandbox`, `plugin`, `agent`, `mem`, `graph`, `ctx`, `rtk`, `doctor`), 8 ports (`8787..18787`, incl. `:9098` Protocol Bus), `~/.toksi/config.toml` (`[repositories]`, `[plugins]`, `[security]`), **16** slash commands (`/byok`..`/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`), `direct-wired` bus, effort-tier `tokview` attribution, **35-component `<toksi-siagent>`** across **432 mandated vaults** (**43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, zero paid-tier restrictions, externalized zero-bloat `<skills>` & `<advisory>`), and `<repo>/.toksi/<tool>` rules.
5. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **SSE Stream Token Parser** (`src/telemetry/tokentracker.rs`): Inspects chunks in-flight, parsing provider JSON usage blocks without buffering the entire stream.
- **Network Reconnect Watchdog** (`src/gateway/reconnect.rs`): Monitors upstream TCP connections using keep-alive probes and connection pool TTLs. On connection reset, resets connection pool and retries smoothly.
- **Universal Synchronizer** (`src/rules/sync.rs`): Distributes embedded guidelines across all agent config paths.
- **CLI Usage Help** (`src/main.rs`): Formats clean terminal help with commands, flags, examples, and port assignments.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- tokentracker` and `cargo test -p toksi -- reconnect` pass 100%.
- `toksi --help` prints full usage manual without truncation or syntax errors.

---

## Wave 18 (v5.2) Synchronization Addendum
- Integrate `tokview`, `langfuse`, `phoenix` OpenInference spans, `tokenspeed` TTFT/TPOT latency, `vmon` + `cuda-rust` telemetry, and `tokenizers` exact BPE counting across all **432 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized `<skills>` & `<advisory>`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** Synchronized to Wave 19 / v6.0 (`432` deep-researched vaults, `43` `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, dedicated `toksi _worker <role>` subprocesses, `ctxmode` rename, `docker/Dockerfile.test` live-safe isolation, `922`-dir `.agents/skills/` hygiene, and `21` target slash commands).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
