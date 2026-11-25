# Phase 17.6: Ground-Truth SSE Accounting, Gateway Real Dispatch (`:9099` / `:8789` / `:8790` / `:18787`), Network Heal, & ConnectRPC

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 13.3–13.5), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_17_6_TOKEN_ACCOUNTING_NETWORK_HEAL_AND_GATEWAY.md first.

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
   - `gateway/proxy.rs:106-148, 207-210, 324-327`: Invoke full shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`: `live_zone`, `CcrStore`, `SmartCrusher`, `TokenEconomizer`, `agent-opt`, `mcp_pruner`, `chroma`/`lance`) in `process_request` and `dispatch_with_cascade_async` (replacing align+prune-only processing); remove `provider.default_endpoint()` hardcoding so custom user `base_url`s from `config.toml`/`byok_profiles` are always honored; implement `/v1/responses` and `/v1/embeddings` alongside `/v1/chat/completions`, `/v1/messages`, `/v1beta/models/*`.
   - `headroom/copilot_daemon.rs` & `headroom/byok_gateway.rs`: Replace `"copilot proxy ready"` & `"wrap"` stubs on `:8789` (VS Code) and `:8790` (CLI) with authentic OAuth token refresh, Headroom tail compression, and SSE upstream streaming; route all enabled `:18787` BYOK & Vertex ADC requests through `HeadroomEffortCompressor` (`Low`/`Medium`/`High`).
   - `commands/daemon.rs`: Supervise `:18787` (`byok_gateway.rs`) alongside `:8787` (Headroom), `:8788` (Grok OAuth — explicitly separate from BYOK), `:8789`/`:8790` (Copilot), `:9098` (Protocol Bus w/ `HeadroomEffortCompressor`), `:9099` (Gateway), and `:9100` (Cockpit).
2. Authentic SSE Usage & Rate-Limit Extraction (`telemetry/{tokentracker,collector}.rs`):
   - Parse streaming OpenAI/Responses/Anthropic (`cache_creation_input_tokens`, `cache_read_input_tokens`), OpenAI reasoning tokens, and Gemini (`candidatesTokenCount`) live without buffering entire responses.
   - Parse `x-ratelimit-*` and `anthropic-ratelimit-*` headers into `quota_windows` (`hour|day|week|month|year`), updating live TUI/Web snapshots cleanly.
   - Zero-cost `/v1/models` sync feeding `provider_models` + `/model` slash completion + UI pickers.
3. Network Reconnect Auto-Healing (`gateway/reconnect.rs`) & HTTP/2 ConnectRPC (`headroom/proxy.rs`):
   - Rebuild `ArcSwap<reqwest::Client>` pools and flush dead sockets automatically when sleep/wake, VPN toggles, DNS errors, or keep-alive timeouts occur (preventing dormant agent stalls).
   - Ensure HTTP/2 ConnectRPC streaming headers, trailers, and binary frames pass safely without corruption (closing `#0033`/`#0034`).
4. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi` honoring **495 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides (`PHASE_01`..`PHASE_19_6`), `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized zero-bloat `<skills>` & `<advisory>`.
```

---

## Technical Specifications & Details

### 1. Port & Routing Matrix
- `:8787` Headroom Streaming Proxy + `:8788` Grok OAuth (`grok_daemon.rs`) + `:8789` Copilot VS Code + `:8790` Copilot CLI + `:9098` Protocol Bus + `:18787` Multi-Profile BYOK/Vertex ADC Gateway + `:9099` Enterprise AI Gateway (`mofa-engine` 7-D + `LoadBalancer` + `cascade` + `/metrics`) -> ALL 7 inference/protocol entrypoints invoke the shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`) -> `:9100` Detached Web Cockpit.

### 2. Verification Criteria
- Gateway routes real mock OpenAI/Anthropic/Gemini/Responses SSE chunks without `"AI Gateway operational"` stubs and reports `0` tokens when idle.
- ConnectRPC binary frame round-trip test and simulated pool-reset reconnect test pass 100%.

---

## Wave 18 (v5.2) Synchronization Addendum
- Add `tokview` + `langfuse` + `phoenix` OpenInference local trace & effort-level token accounting (`Low`/`Medium`/`High`), `tokenspeed` TTFT/TPOT, and `vmon` + `cuda-rust` telemetry (`PHASE_18_1`, `PHASE_18_3`, `PHASE_18_6`).
- Route `AGY_ADC_AUTH=true` profiles through native Google Cloud ADC Vertex endpoints (`https://{location}-aiplatform.googleapis.com/v1beta1/...`) with cached OAuth2 refresh-token / service-account exchange and clear `gcloud auth application-default login` diagnostics (`PHASE_18_3`), plus all **26** local/Rust engines (`mistral.rs`, `candle`, `ollama`, `llama.cpp`, `vllm`, `sglang`, `dynamo`, `frontend-crates`) and `RouteLLM`/`olla` routing across **495 mandated vaults**, **35-component `<toksi-siagent>`**, and **43** `.agents/brain/*.md` guides.
- Expose `:9098` Protocol Bus alongside `:8787..:8790`, `:9099`, `:9100`, and `:18787` (unified **8-port daemon**).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
