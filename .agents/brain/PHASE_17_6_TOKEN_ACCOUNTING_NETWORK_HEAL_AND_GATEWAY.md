# Phase 17.6: Ground-Truth SSE Accounting, Gateway Real Dispatch (`:9099` / `:8789` / `:8790` / `:18787`), Network Heal, & ConnectRPC

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 13.3–13.5), .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md first.

Execute Phase 17.6: Ground-Truth SSE Accounting, Gateway Real Dispatch, OS Reconnect Watchdog, & ConnectRPC Passthrough.
Sources:
  - neo/inspect-vault/TokenTracker
  - neo/inspect-vault/mofa-engine
  - neo/inspect-vault/bifrost
  - neo/inspect-vault/litellm
  - neo/inspect-vault/headroom
Targets:
  - crates/toksi/src/gateway/{server.rs, proxy.rs, router.rs, reconnect.rs, providers/aggregators/{mod.rs, bifrost.rs}}
  - crates/toksi/src/headroom/{copilot_daemon.rs, grok_daemon.rs, byok_gateway.rs, proxy.rs}
  - crates/toksi/src/telemetry/{tokentracker.rs, collector.rs}
  - crates/toksi/src/commands/daemon.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly; zero `.unwrap()`/`.expect()`/`panic!()`.
2. Zero Synthetic Telemetry: Idle totals MUST equal strictly `0`. Never invent hardcoded `50/50` tokens, `0.0005` USD costs, or fake 429 rows.
3. Zero Inference Token Burn for Discovery: `/v1/models` and free quota endpoint refreshes must NEVER execute paid/inference completion calls.
4. Configured-Only Routing: Only route across user-enabled `byok_profiles` (`src/config/mod.rs` + `toksi.db`); pass through transparently if none are enabled.

Requirements:
1. Replace All Static Gateway & Copilot Stubs with Real Dispatch:
   - `gateway/server.rs:259-371` & `aggregators/mod.rs`: Replace `"AI Gateway operational"` static text and `50/50`/`0.0005` stubs with real `LoadBalancer` dispatch, `cascade` failover, and `mofa-engine` deterministic 7-D `Router::route_ranked` (`DataClass::Confidential` local-engine pinning, `CircuitBreakerRegistry`, `QualityGate`).
   - `gateway/proxy.rs:106-148`: Remove `provider.default_endpoint()` hardcoding so custom user `base_url`s from `config.toml`/`byok_profiles` are always honored; implement `/v1/responses` and `/v1/embeddings` alongside `/v1/chat/completions`, `/v1/messages`, `/v1beta/models/*`.
   - `headroom/copilot_daemon.rs`: Replace `"copilot proxy ready"` & `"wrap"` stubs on `:8789` (VS Code) and `:8790` (CLI) with authentic OAuth token refresh, Headroom tail compression, and SSE upstream streaming.
   - `commands/daemon.rs`: Supervise `:18787` (`byok_gateway.rs`) alongside `:8787` (Headroom), `:8788` (Grok OAuth — explicitly separate from BYOK), `:8789`/`:8790` (Copilot), `:9099` (Gateway), and `:9100` (Cockpit).
2. Authentic SSE Usage & Rate-Limit Extraction (`telemetry/{tokentracker,collector}.rs`):
   - Parse streaming OpenAI/Responses/Anthropic (`cache_creation_input_tokens`, `cache_read_input_tokens`), OpenAI reasoning tokens, and Gemini (`candidatesTokenCount`) live without buffering entire responses.
   - Parse `x-ratelimit-*` and `anthropic-ratelimit-*` headers into `quota_windows` (`hour|day|week|month|year`), updating live TUI/Web snapshots cleanly.
   - Zero-cost `/v1/models` sync feeding `provider_models` + `/model` slash completion + UI pickers.
3. Network Reconnect Auto-Healing (`gateway/reconnect.rs`) & HTTP/2 ConnectRPC (`headroom/proxy.rs`):
   - Rebuild `ArcSwap<reqwest::Client>` pools and flush dead sockets automatically when sleep/wake, VPN toggles, DNS errors, or keep-alive timeouts occur (preventing dormant agent stalls).
   - Ensure HTTP/2 ConnectRPC streaming headers, trailers, and binary frames pass safely without corruption (closing `#0033`/`#0034`).
4. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Port & Routing Matrix
- `:8787` Headroom Streaming Proxy -> `:8788` Grok OAuth (`grok_daemon.rs`) -> `:8789` Copilot VS Code -> `:8790` Copilot CLI -> `:18787` Multi-Profile BYOK Gateway -> `:9099` Enterprise AI Gateway (`mofa-engine` 7-D + `LoadBalancer` + `cascade` + `/metrics`) -> `:9100` Detached Web Cockpit.

### 2. Verification Criteria
- Gateway routes real mock OpenAI/Anthropic/Gemini/Responses SSE chunks without `"AI Gateway operational"` stubs and reports `0` tokens when idle.
- ConnectRPC binary frame round-trip test and simulated pool-reset reconnect test pass 100%.
