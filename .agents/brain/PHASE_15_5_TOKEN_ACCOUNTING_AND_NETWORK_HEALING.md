# Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_5_TOKEN_ACCOUNTING_AND_NETWORK_HEALING.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.5: Ground-Truth Token Accounting & Network Reconnect Healing.
Targets:
  - crates/toksi/src/telemetry/collector.rs
  - crates/toksi/src/telemetry/tokentracker.rs
  - crates/toksi/src/gateway/reconnect.rs
  - crates/toksi/src/gateway/proxy.rs
  - crates/toksi/src/rules/sync.rs
  - crates/toksi/src/main.rs

Requirements:
1. Ground-Truth Session Token Accounting:
   - Extract real prompt/completion/cached tokens directly from live SSE response streams and upstream response headers:
     * `usage.prompt_tokens`, `usage.completion_tokens`, `cache_read_input_tokens`
     * `x-ratelimit-remaining-tokens`, `anthropic-ratelimit-requests-remaining`
   - Strict prohibition of synthetic or virtual math: when no agent is streaming, token savings counter must read 0.
   - Query provider quota status via free agent AI API channels without incurring token waste.
2. Network Reconnection Auto-Healing Watchdog:
   - Implement `gateway/reconnect.rs` detecting network interface disconnects, VPN toggles, and socket timeouts.
   - Automatically refresh connection pools and reconnect to upstream endpoints without entering a dormant state or locking out agents.
3. Universal Rules, Skills & Instructions Sync:
   - Implement universal synchronizer updating `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md` across both workspace directories and user config homes for all 17 agents.
4. Complete Usage Help:
   - Update `help_text()` in `crates/toksi/src/main.rs` with comprehensive details on all commands, tools, ports, flags, and conventions.
5. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Reconnection Watchdog in `gateway/reconnect.rs`
```rust
pub struct NetworkWatchdog {
    last_probe: std::time::Instant,
    is_healthy: std::sync::atomic::AtomicBool,
}

impl NetworkWatchdog {
    pub fn new() -> Self {
        Self {
            last_probe: std::time::Instant::now(),
            is_healthy: std::sync::atomic::AtomicBool::new(true),
        }
    }

    pub async fn check_and_reload_if_needed(&self, client_pool: &mut reqwest::Client) {
        if self.last_probe.elapsed() > std::time::Duration::from_secs(15) {
            // Probe reachability
            let probe_res = client_pool.get("http://127.0.0.1:9099/metrics").timeout(std::time::Duration::from_millis(500)).send().await;
            if probe_res.is_err() {
                // Reinitialize pool to purge stale sockets
                *client_pool = reqwest::Client::builder()
                    .pool_idle_timeout(std::time::Duration::from_secs(30))
                    .tcp_keepalive(std::time::Duration::from_secs(15))
                    .build()
                    .unwrap_or_default();
            }
        }
    }
}
```
