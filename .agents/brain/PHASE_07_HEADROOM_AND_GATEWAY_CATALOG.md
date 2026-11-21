# Phase 07: Headroom Deep Integration & Gateway Catalog

> **Status (Wave 19 / v6.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/headroom, neo/inspect-vault/bifrost, and neo/inspect-vault/tokless is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and upstream provider catalog additions into consolidated toksi.

Execute Phase 07: Headroom Deep Integration & Gateway Catalog.
Sources:
  - neo/inspect-vault/headroom (crates/headroom-core)
  - neo/inspect-vault/bifrost (core/providers/, plugins/)
  - neo/inspect-vault/tokless (crates/toksi/src/)
Targets:
  - crates/toksi/src/headroom/
  - crates/toksi/src/gateway/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly in `crates/toksi`. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors.
3. Unified 8-Port Topology (all overridable via `~/.toksi/config.toml` `[ports]`, expanded in Wave 18 with `:9098` Protocol Bus alongside `:8787..:8790, :9099, :9100, :18787`; pair with `PHASE_18_3` for `Low`/`Medium`/`High` thinking-effort scaling & `AGY_ADC_AUTH=true` Vertex routing):
   - Port `8787`: Headroom Streaming Proxy (Antigravity, Claude, Codex, Cursor, etc.)
   - Port `8788`: Grok OAuth Proxy Daemon (`grok_daemon.rs` — distinct from `:18787` BYOK!)
   - Port `8789`: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
   - Port `8790`: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
   - Port `18787`: BYOK Multi-Profile & Vertex ADC Gateway (`byok_gateway.rs` — supervised by daemon, routed through shared `HeadroomEffortCompressor`)
   - Port `9099`: Enterprise AI Gateway & Drop-in SDKs + `/metrics` (Inference thread pool, <10 µs latency overhead, enforcing full `HeadroomEffortCompressor` + `<enforce>`)
   - Port `9100`: Detached Web Dashboard Cockpit (Isolated listener, zero backpressure on LLM streaming)
4. Zero Paid-Tier Restrictions & Open Multi-Instance BYOK: Remove ALL `FreeTierDisqualified` / `ACTIVE_PAID_PROVIDERS` / `[PAID]` gating. Support multiple `<byok, model, effort>` instances per provider, editable custom API `base_url`s, configured-only routing activation, and mandatory `HeadroomEffortCompressor` (`Low`/`Medium`/`High`: `live_zone` + `CcrStore` + `SmartCrusher` + `TokenEconomizer` + `agent-opt` + `mcp_pruner` + `rtk` + `chroma`/`lance`) across ALL 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`; fix `/byok/` early return at `headroom/proxy.rs:619-631` and extend `gateway/proxy.rs` beyond align+prune).
5. Ground-Truth Token Accounting: Real tokens (`cache_creation_input_tokens`, Responses usage, Gemini `candidatesTokenCount`) parsed from live SSE streams; prohibition of virtual/synthetic math (`0` when idle); zero-cost `/v1/models` refresh & read-only `hour..year` `quota_windows`.

Requirements:
1. Delete Python `headroom` spawns (`headroom/proxy.rs:1271`, `tools/headroom.rs:64`) and `cloudcode_patch.rs`; port pure-Rust `inspect-vault/headroom/crates/headroom-core` (`compute_frozen_count` + `live_zone` byte-range surgery preserving LLM prompt cache prefixes instead of `headroom/proxy.rs:785-827` JSON rewrite, `CcrStore` `<<ccr:HASH>>` SQLite + MCP `headroom_*`, `SmartCrusher`, `SearchCompressor`, `DiffCompressor`, `AuthMode`, HTTP/2 ConnectRPC `#0033`/`#0034` passthrough).
2. Implement core token compression in `crates/toksi/src/headroom/`:
   - `compression/log.rs`: repetition-collapsing log compressor; guarantees `FATAL`, `ERROR`, and `panic` lines survive byte-for-byte.
   - `compression/code.rs`: AST-aware whitespace and comment pruner.
   - `compression/json.rs`: structural JSON minifier.
   - `compression/economizer.rs`: full 3-stage `TokenEconomizer` + `EnvFactsDistiller` + `LoopGuard`.
   - `pruner/mcp_pruner.rs`: dynamic MCP tool schema optimizer (slashes 5,000–15,000 wasted input tokens per turn).
   - `cache_align.rs`: 1024-token cache-prefix stabilizer + `live_zone` tail surgery.
   - `adaptive_mem.rs`: dynamic memory governor (scales injected memory between L0/L1/L2 based on headroom).
3. Implement Native Enterprise AI Gateway (`crates/toksi/src/gateway/`) with real dispatch (no `"AI Gateway operational"` or `50/50`/`0.0005` stubs):
   - `providers/`: full **65/90+ Categories 1–5 ecosystem + 22 local inference engines** (`Ollama`, `vLLM`, `llama.cpp`, `LM Studio`, `SGLang`, `TGI`, `Exo`, etc., self-hosted `Bifrost`, SageMaker, Mantle).
   - `cascade.rs`, `loadbalancer.rs`, `router.rs` (`mofa-engine` 7-D `route_ranked`), `governance.rs` (virtual keys + `CircuitBreakerRegistry`), `semantic_cache.rs` (Chroma ANN HNSW/USearch/SPANN + `foyer` spill wired live into `proxy.rs`), `compat/` (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`, `/v1/embeddings`, `/v1beta/models/*`).
4. Supervise Copilot dual-port architecture (Port `8789` VS Code + Port `8790` CLI) with real OAuth/SSE upstream streaming (removing `"copilot proxy ready"` & `"wrap"` stubs).
5. Wire all **26** local/Rust inference vaults (`ollama`, `llama.cpp`, `candle`, `candle-vllm`, `mistral.rs`, `dynamo`, `frontend-crates`, `nixl`, `radixshmem`), `RouteLLM`/`olla`/`Switchyard`/`NeMo-Relay`, `tokenizers` BPE, and `Guardrails`/`llm-guard` across the **35-component `<toksi-siagent>`** (`432` mandated vaults, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, externalized zero-bloat `<skills>` & `<advisory>`), ensuring 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Dynamic MCP Schema Pruner** (`src/headroom/pruner/mcp_pruner.rs`): Inspects active chat turns, detects unused tools, and strips verbose schema descriptions, saving 5k-15k tokens per request.
- **Log Compression Engine** (`src/headroom/compression/log.rs`): Collapses repeating stack traces while ensuring panic, fatal, and error lines survive byte-for-byte.
- **Prompt Cache Aligner** (`src/headroom/cache_align.rs`): Pads system prompts to multiples of 1024 tokens to maximize provider cache hits.
- **Enterprise AI Gateway & Cascade** (`src/gateway/`): Dispatches LLM calls across 39+ providers with circuit breakers and cost/latency-optimized load balancing.

### 2. Disciplinary Standards & Invariants
- High-throughput async streaming using Tokio work-stealing scheduler. Zero blocking calls on the reactor threads.
- Port 9099 dedicated strictly to inference; no web dashboard assets served from port 9099.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- headroom` and `cargo test -p toksi -- gateway` pass 100%.
- Prompt cache aligner produces exact 1024-token aligned boundaries.

---

## Wave 18 (v5.2) Synchronization Addendum
- Route all 7 inference/protocol entrypoints (`:8787`–`:8790`, `:9098`, `:9099`, `:18787`) through shared `HeadroomEffortCompressor` (`Low`/`Med`/`High` + `live_zone` + `CcrStore` + `SmartCrusher` + `TokenEconomizer` + `agent-opt` + MCP Pruner + `RTK` + `Chroma`/`Lance`/`Qdrant` + `<enforce>`) for **35-component `<toksi-siagent>`** across **432 mandated vaults**, **43** `.agents/brain/*.md` guides (`PHASE_01`..`PHASE_19_6`), canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, and externalized zero-bloat `<skills>` & `<advisory>` (`PHASE_18_3`, `PHASE_18_6`).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** Synchronized to Wave 19 / v6.0 (`432` deep-researched vaults, `43` `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, dedicated `toksi _worker <role>` subprocesses, `ctxmode` rename, `docker/Dockerfile.test` live-safe isolation, `922`-dir `.agents/skills/` hygiene, and `21` target slash commands).
