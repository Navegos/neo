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
1. Ground-Truth Session Token Accounting:
   - Parse actual prompt tokens, completion tokens, and cached tokens directly from live SSE response streams (`usage.prompt_tokens`, `usage.completion_tokens`, `cache_read_input_tokens`, Google `candidatesTokenCount`).
   - Extract upstream provider rate limits and quotas from HTTP response headers: `x-ratelimit-remaining-tokens`, `x-ratelimit-remaining-requests`, `anthropic-ratelimit-requests-remaining`.
   - Calculate true headroom token savings as byte reduction (`raw_bytes - compressed_bytes`) and exact counts of pruned MCP tool schemas.
   - Query provider quota status via free agent AI API channels without incurring additional token costs.
2. Network Reconnection Auto-Healing Watchdog (`crates/toksi/src/gateway/reconnect.rs`):
   - Detects connection drops, DNS invalidation, or network interface toggles.
   - Automatically invalidates stale socket connection pools and reconnects upstream client pools without requiring process restarts.
   - Eliminates dormant states where agents are locked out after Wi-Fi or VPN reconnects.
3. Universal Rules, Skills & Instructions Sync:
   - Implement universal synchronizer updating `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md` across both workspace roots and user config homes for all 17 agents.
4. Comprehensive CLI Help:
   - Update `help_text()` in `crates/toksi/src/main.rs` with exhaustive documentation of all commands, tools, ports, flags, and conventions.
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
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
