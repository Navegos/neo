# Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging.
Targets:
  - crates/toksi/src/tools/multi_protocol.rs
  - crates/toksi/src/codegraph/walker.rs
  - crates/toksi/src/codegraph/cache.rs
  - crates/toksi/src/contextmode/fts.rs
  - crates/toksi/src/icm/store.rs
  - crates/toksi/src/aimemory/wiki.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Multi-Protocol, Shared Effort Compression & DirectWire Standard: Expand `ToolTarget` in `tools/multi_protocol.rs` beyond the initial 6 variants (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`) to expose `Headroom`, `Security`, `Advisory`, `Sandbox`, `Plugins`, and all **432** consolidated `inspect-vault/*` engines over **real** `MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, and `HTTP` on `:9098`, route large payloads/responses through the shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`), and bind config-less `DirectWireBus` (`direct-wired`) alongside `config-wired` for **35-component `<toksi-siagent>`** (**43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, zero paid-tier restrictions, externalized zero-bloat `<skills>` & `<advisory>`, and canonical `<repo>/.toksi/projectmem/`).
4. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` across all ignore matchers and vector index paths.
5. Repo-Only & Dynamic Vector Purge: Restrict indexing strictly inside `<repo>` (never outside `.git` repo root and never inside `<repo>/.git/`), store caches only in `<repo>/.toksi/<tool>`, track monorepo `subproject_id`, and automatically purge SQLite FTS5 & Chroma ANN embedding caches when `.gitignore` / `.ignore` rules change.

Requirements:
1. Multi-Protocol Engine (`crates/toksi/src/tools/multi_protocol.rs` & `direct_wire.rs`):
   - Expose unified dispatcher routing incoming requests across all 6 protocols:
     * **MCP** (Model Context Protocol — tools, resources, prompts; fix `commands/runmcp.rs` `aimemory->icm` misroute!)
     * **A2A** v1.0 JSON-RPC 2.0 (`A2aTaskStore`: `SendMessage`/`GetTask` + `"message/send"`/`"tasks/get"` aliases, SHA-256 constant-time bearer auth, 1 MiB cap, `(peer, messageId)` dedupe)
     * **ACP** v2.0 (`SessionActorQueue` per-session serialized dispatch)
     * **stdio** (Standard Input / Standard Output streaming bridge)
     * **SSE** (Server-Sent Events real-time event streaming)
     * **HTTP** (RESTful endpoints and chunked stream transport)
   - Ship `DirectWireBus` so configured/hooked agents can invoke `toksi` tools & agent-to-agent commands directly in-process without needing config-file entries (`direct-wired` status).
   - Implement real handlers across all tools: `rtk`, `codegraph`, `context` (fix `ctx-hook` passthrough `0` stub), `projectmem`, `icm`, `aimemory`.
2. Ripgrep-Grade Ignore Walker:
   - Respect `.gitignore` and `.ignore` rules across indexing subsystems (`codegraph`, `context`, `icm`, `aimemory`).
   - Refuse to index outside `.git` repo roots or inside `.git/`, hiding build artifacts (`target/`, `node_modules/`, `dist/`, `*.lock`).
3. Dynamic Vector Index Purging:
   - Watch `.gitignore` and `.ignore` files for modifications.
   - Automatically delete already-indexed files matching new ignore patterns from SQLite FTS5 indexes (`context`, `icm`), `symbols.db`, and Chroma ANN vector embedding caches.
4. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Multi-Protocol Multiplexer** (`src/tools/multi_protocol.rs`): Abstract protocol transport layer mapping JSON-RPC MCP frames, A2A messages, and HTTP REST requests to identical tool logic.
- **Dynamic Ignore Purger**: When a new pattern (e.g. `/build/`) is appended to `.gitignore`, a background sweep identifies all entries in SQLite FTS5 matching the pattern and issues `DELETE FROM fts_index WHERE path LIKE 'build/%'`.
- **Embedding Cache Cleanup**: Drops orphaned vector embeddings corresponding to purged files.

### 2. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- multi_protocol` passes 100%.
- Adding `*.tmp` to `.gitignore` triggers automatic removal of `.tmp` records from FTS5 index.

---

## Wave 18 (v5.2) Synchronization Addendum
- Bind `:9098` as the dedicated Protocol Bus port (`MCP` stdio/SSE/HTTP, `A2A` v1.0, and `ACP` Agent Client Protocol from `inspect-vault/agent-client-protocol`) alongside `:8787..:8790`, `:9099`, `:9100`, and `:18787` (`PHASE_18_1`), serving all **495 mandated vaults** and **35-component `<toksi-siagent>`** (**43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized `<skills>` & `<advisory>`).
- Mount Unified Plugin System (`src/plugins/hooks.rs`) protocol endpoints (`MCP`, `A2A`, `ACP`, `stdio`, `SSE`, `HTTP` on `:9098`), `DirectWireBus` wire hooks (Categories 1–3), and `agent_to_agent` messaging under `opa`/`cedar`, `spire`, `defenseclaw` fail-closed hookguard & `skill-scanner` `<enforce>` verification (`PHASE_18_2` / `PHASE_18_6`).
- Publish `temporal`/`restate`/`langgraph` `plan_checkpoints` pause/resume events and static security findings on `:9098` streams.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 19 Reconciliation:** MCP/A2A/ACP/SSE protocol bus (`:9098`) active; `PHASE_19_5` adds **ANP (`did_wba`)** and **QUIC (`quinn`)** transport parity.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
