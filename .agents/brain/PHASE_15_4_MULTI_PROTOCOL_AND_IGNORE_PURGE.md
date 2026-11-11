# Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

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
3. Multi-Protocol & DirectWire Standard: All tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`) MUST support **real** `MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, and `HTTP`, plus config-less `DirectWireBus` (`direct-wired`) alongside `config-wired`.
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
