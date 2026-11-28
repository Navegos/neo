# Phase 08: Downloader Purge & System Refactor

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]` [SOURCE-CODE-IMPLEMENTED-AND-FULLY-COMPLETE]

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_08_DOWNLOADER_PURGE.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 08: Downloader Purge & System Refactor.
Targets:
  - crates/toksi/src/util/
  - crates/toksi/src/agents/
  - crates/toksi/Cargo.toml

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. 100% Native Pure In-Tree Compilation: Zero external runtime downloaders (npm, pip, uv, curl, node, python).
4. PATH Validation Mandate: All required external commands must be verified in system PATH, never downloaded dynamically on the fly.
5. Universal Forward-Slash (/) Standard: Use normalize_path() for all internal paths and configuration references.

Requirements:
1. Completely purge legacy downloader/runtime bootstrap modules and remnants:
   - Delete `crates/toksi/src/util/npminstall.rs`
   - Delete `crates/toksi/src/util/nodewin.rs`
   - Delete `crates/toksi/src/util/nodelinux.rs`
   - Delete `crates/toksi/src/util/uv.rs`
   - Delete `crates/toksi/src/util/projectmem.rs` & legacy `cloudcode_patch.rs`
   - Purge `resolve_npm_binary` / `resolve_node_binary` / `resolve_bun_binary` (`util/exec.rs`), `toksi-codegraph-init.js` (`tools/autoindex.rs`), Python `headroom` spawns (`headroom/proxy.rs:1271`, `tools/headroom.rs:64`), and `wrap` Copilot stubs (`copilot_daemon.rs`).
2. Remove all downloader references, subprocess spawning of `npm`/`pip`/`uv`, and temporary runtime bootstrap logic across `crates/toksi/src/agents/*` (and eliminate `std::env::set_var` `unsafe` in `util/exec.rs:196`).
3. Rewire all `18+` built-in & Category 1--5 agent harnesses and tool setups to use internal, pure in-tree Rust modules and `DirectWireBus` (`direct-wired`) alongside `config-wired`:
   - Tool `projectmem` -> internal pure Rust `src/projectmem/`
   - Tool `context` -> internal pure Rust `src/contextmode/`
   - Tool `codegraph` -> internal pure Rust `src/codegraph/`
   - Tool `rtk` -> internal pure Rust `src/rtk/`
   - Tool `icm` -> internal pure Rust `src/icm/`
   - Tool `ai-memory` (`aimemory`) -> internal pure Rust `src/aimemory/` (fixing `runmcp.rs` routing!)
   - Unified into **35-component `<toksi-siagent>`** (`DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, **495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, canonical `<repo>/.toksi/projectmem/`, zero paid-tier restrictions, externalized zero-bloat `<skills>` & `<advisory>`).
4. Ensure 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Legacy Purge**: Eliminates brittle external package manager bootstrap steps that fail in offline, corporate, or restricted sandbox environments.
- **Direct Native Tool Dispatch**: Replaces Node.js and Python CLI wrappers with direct invocations of in-tree compiled Rust handlers (`toksi mem`, `toksi ctx`, `toksi graph`, `toksi rtk`).
- **Cargo.toml Refactor**: Cleans up unused downloader dependencies while keeping core async, parsing, and cryptographic crates intact.

### 2. Disciplinary Standards & Invariants
- No dead code: unused functions or legacy downloader imports must be removed immediately.
- The build must succeed without any Node.js, Python, or npm binaries installed on the host system.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi` passes 100%.
- Search for `npminstall`, `nodewin`, or `uv.rs` across `crates/toksi/src/` returns 0 results.

---

## Wave 18 (v5.2) Synchronization Addendum
- Keep zero runtime downloaders (`npm`/`pip`/`uv`/`curl`) and zero embedded `.md`/`.yaml`/`.json` skills/rules inside `crates/toksi/`; sync vetted external `<skills>` (`Navegos/skills`) and `<advisory>` (`Navegos/advisory`) cleanly under `<enforce>`, `skilldoctor`/`agentskills`, and `defenseclaw`/`skill-scanner` across **495 mandated vaults**, **35-component `<toksi-siagent>`**, **43** `.agents/brain/*.md` guides, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, and canonical `<repo>/.toksi/projectmem/`.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
