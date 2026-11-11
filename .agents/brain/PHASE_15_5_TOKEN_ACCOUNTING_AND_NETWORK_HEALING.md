# Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

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
   - Sync embedded `Karpathy`, `Caveman` v4, and `Ponytail` guidelines across ONLY user-configured/selected agents (`18+` built-in + `.agents` + Category 1–5) with granular component toggles (`hooks|mcp|skills|rules|instructions|configs`), respecting `user` auto-install vs explicit `repo` opt-in.
4. Comprehensive CLI Help (`crates/toksi/src/main.rs`):
   - Update `help_text()` with exhaustive documentation of all commands (`chat|run|exec|swarm`, `daemon|proxy|dashboard start|stop|restart --agents`, `config`, `byok`, `db`, `auth`, `agent`, `mem`, `graph`, `ctx`, `rtk`, `doctor`), 7 ports (`8787..18787`), `~/.toksi/config.toml`, 10 slash commands, `direct-wired` bus, and `<repo>/.toksi/<tool>` rules.
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
