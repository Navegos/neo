# Phase 15.3: Smart Agent CLI Providers & Open BYOK Quota Telemetry *(Zero Paid-Tier Restrictions)*

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_3_SMART_CLI_PROVIDERS_AND_PAID_QUOTA.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/bifrost is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new provider integrations, aggregator features, and quota handlers into consolidated toksi.

Execute Phase 15.3: Smart Agent CLI Providers & Open BYOK Quota Telemetry.
Source: neo/inspect-vault/bifrost (BYOK provider registry & aggregators)
Targets:
  - crates/toksi/src/gateway/agent_cli/mod.rs
  - crates/toksi/src/gateway/agent_cli/tier_check.rs
  - crates/toksi/src/gateway/agent_cli/{antigravity,copilot,claude,codex,cursor,grok}.rs
  - crates/toksi/src/gateway/providers/aggregators/bifrost.rs
  - crates/toksi/src/config/mod.rs
  - crates/toksi/src/db/{schema.rs,operations.rs}

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly (remove `pre_exec(setsid)` `unsafe` in `gateway/agent_cli/*.rs` via safe `CommandExt::process_group(0)` or `crates/toksi-sys`). Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors.
3. Zero Paid-Tier Restrictions: Completely DELETE `FreeTierDisqualified`, `ACTIVE_PAID_PROVIDERS`, and all paid-only qualification gates from backend, database logic, and UI! Users are 100% free to add, configure, and route any working, accessible agent or BYOK account (free, trial, OAuth, subscription, or commercial) configured in `~/.toksi/config.toml` and `toksi.db`.
4. Read-Only Quota Telemetry & Zero-Fail Fallback: When rate-limited (HTTP 429) or unbilled, NEVER crash or panic; emit a non-blocking informative log and fail over to the next enabled `<byok, model, effort>` profile. Track read-only plan labels (when exposed by free APIs) and `quota_windows` (`hour|day|week|month|year`) purely for user visibility and smart routing.
5. Configured-Only Activation Guard: Only wire, spawn, display, and dispatch across agents and `<byok, model, effort>` profiles explicitly configured/enabled by the user in `~/.toksi/config.toml` / `toksi.db` (if none are configured, pass traffic cleanly through without error). Every configured BYOK payload MUST pass through the Headroom optimizer (`headroom/proxy.rs`).

Requirements:
1. Register Local Agent CLIs (`AntigravityCli`, `CopilotCli`, `ClaudeCli`, `CodexCli`, `CursorBridge`, `GrokCli`, plus `GeminiCli`, `GooseCli`, `CrushCli`) & 65/90+ Categories 1–5 + 22 Local Engine Adapters as peer BYOK providers (multi-instance `<byok, model, effort>` per provider supported).
2. Smart Background Launch (Configured-Only + Safe PATH Check):
   - Validate binary in `PATH` via `check_executable_in_path` cleanly (never hardcoding paths; greystone/skip if missing).
   - Query OS Keyring (`gemini:antigravity` alias normalized) and SQLite `byok_profiles`.
   - Launch CLI bridge processes cleanly in background (`CommandExt::process_group(0)` / `toksi-sys`) ONLY when configured & selected.
3. Open Quota & Model Discovery (`tier_check.rs`):
   - Delete `FreeTierDisqualified` / `ACTIVE_PAID_PROVIDERS` gatekeeping.
   - Populate `quota_windows` (`hour|day|week|month|year`) & model capabilities via zero-cost `/v1/models` and response rate-limit headers (`x-ratelimit-*`, `anthropic-ratelimit-*`).
4. Bifrost BYOK Provider Integration:
   - Implement `gateway/providers/aggregators/bifrost.rs` with health probes, failover routing, custom `base_url` override support, and real `LoadBalancer` dispatch.
5. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Smart CLI Bridge Manager** (`src/gateway/agent_cli/mod.rs`): Safe background spawning ONLY for user-configured/selected CLIs present in `PATH`.
- **Open Quota Inspector** (`src/gateway/agent_cli/tier_check.rs`): Extracts optional plan tier metadata and `hour/day/week/month/year` quota windows cleanly without disqualifying free or trial BYOK accounts.
- **Bifrost Aggregator Client** (`src/gateway/providers/aggregators/bifrost.rs`): Connects to self-hosted or remote Bifrost gateway instances alongside multiple `<byok, model, effort>` instances per provider.

### 2. Verification Criteria
- `cargo check -p toksi` and `cargo clippy --all-targets -- -D warnings` pass with 0 warnings.
- `cargo test -p toksi -- agent_cli` passes 100%, verifying free/trial BYOK profiles route through Headroom without `FreeTierDisqualified` blocking and rate-limited profiles cleanly fail over.
