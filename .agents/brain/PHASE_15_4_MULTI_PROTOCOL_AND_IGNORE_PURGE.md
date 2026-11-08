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
3. Multi-Protocol Standard: All tools (rtk, codegraph, context, projectmem, icm, aimemory) must support MCP, A2A, ACP, stdio, SSE, and HTTP.
4. Universal Forward-Slash (/) Standard: Use normalize_path() across all ignore matchers and vector index paths.
5. Dynamic Vector & Cache Purging: When ignore rules change, automatically purge matching records from SQLite FTS5 search indexes and vector embedding caches.

Requirements:
1. Multi-Protocol Engine (`crates/toksi/src/tools/multi_protocol.rs`):
   - Expose unified dispatcher routing incoming requests across all 6 protocols:
     * **MCP** (Model Context Protocol - tools, resources, prompts)
     * **A2A** (Agent-to-Agent / Agent2Agent collaborative message protocol)
     * **ACP** (Agent Communication Protocol / Agent Client Protocol)
     * **stdio** (Standard Input / Standard Output streaming bridge)
     * **SSE** (Server-Sent Events real-time event streaming)
     * **HTTP** (RESTful endpoints and chunked stream transport)
   - Implement handlers across all tools: `rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`.
2. Ripgrep-Grade Ignore Walker:
   - High-performance respect of `.gitignore` and `.ignore` rules across indexing subsystems (`codegraph`, `context`, `icm`, `aimemory`).
   - Hides build artifacts (`target/`, `node_modules/`, `dist/`, `*.lock`) and vendor bundles from LLM context.
3. Dynamic Vector Index Purging:
   - Watch `.gitignore` and `.ignore` files for modifications.
   - Automatically delete already-indexed files matching new ignore patterns from SQLite FTS5 indexes and vector embedding caches.
4. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
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
